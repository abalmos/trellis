//! Public project compilation and authoring diagnostics.

use std::{fs, path::PathBuf};

#[test]
fn compiles_runtime_acceptance_project() {
    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../integration/fixtures/runtime");
    let mut dependencies = std::collections::BTreeMap::new();
    for project in ["runtime", "jobs-runtime", "eventlog-runtime"] {
        dependencies.extend(
            trellis_idl::compile_apis(
                &trellis_idl::parse_project(
                    &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                        .join("..")
                        .join(project),
                )
                .unwrap(),
            )
            .unwrap(),
        );
    }
    let project = trellis_idl::compile_project(&root, dependencies).unwrap();
    assert!(project.apis.contains_key("test.runtime@v1"));
    assert!(project
        .participants
        .iter()
        .any(|participant| participant.id() == "test.provider"));
}

#[test]
fn malformed_idl_reports_source_and_span() {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join("trellis.toml"), "format = 1\n").unwrap();
    fs::write(
        root.path().join("contract.trellis"),
        "api \"example@v1\" { version \"1.0.0\" }",
    )
    .unwrap();
    let error = trellis_idl::compile_project(root.path(), Default::default()).unwrap_err();
    let diagnostic = format!("{error:?}");
    assert!(diagnostic.contains("contract.trellis"), "{diagnostic}");
    assert!(diagnostic.contains("expected"), "{diagnostic}");
    assert!(
        error
            .labels()
            .is_some_and(|mut labels| labels.next().is_some()),
        "{diagnostic}"
    );
}

#[test]
fn selections_resolve_exact_names_independently_of_declaration_order() {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join("trellis.toml"), "format = 1\n").unwrap();
    for names in [["Get", "Sites.Get"], ["Sites.Get", "Get"]] {
        for selected in ["Get", "Sites.Get", "Invented.Get"] {
            let declarations = names
                .map(|name| {
                    format!(r#"rpc "{name}" {{ version "v1"; input Empty; output Empty; }}"#)
                })
                .join("\n");
            fs::write(
                root.path().join("contract.trellis"),
                format!(
                    r#"
api "source@v1" {{
  version "1.0.0"; display_name "Source"; description "Exact selections.";
  model Empty {{}}
  {declarations}
}}
api "caller@v1" {{ version "1.0.0"; display_name "Caller"; description "Caller."; }}
participant "caller" app {{
  implements "caller@v1";
  use required source "source@v1" {{ call rpc "{selected}"; }}
}}
"#
                ),
            )
            .unwrap();
            let compiled = trellis_idl::compile_project(root.path(), Default::default());
            if selected == "Invented.Get" {
                assert!(compiled.is_err(), "invented qualification was accepted");
            } else {
                let compiled = compiled.unwrap();
                let participant = compiled
                    .participants
                    .iter()
                    .find(|value| value.id() == "caller")
                    .unwrap()
                    .normalized_value()
                    .unwrap();
                assert_eq!(
                    participant["uses"]["required"]["source"]["rpc"]["call"],
                    serde_json::json!([selected])
                );
            }
        }
    }
}

#[test]
fn surface_members_have_defined_meaning_for_their_kind() {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join("trellis.toml"), "format = 1\n").unwrap();
    for (kind, body, invalid) in [
        ("rpc", "input Empty; output Empty;", "payload Empty;"),
        (
            "operation",
            "input Empty; output Empty; progress Empty; cancellable;",
            "params [];",
        ),
        ("event", "payload Empty;", "input Empty;"),
        ("event", "payload Empty;", "event Empty;"),
        ("feed", "input Empty; event Empty;", "output Empty;"),
        ("rpc", "input Empty; output Empty;", "subject \"ignored\";"),
        ("rpc", "input Empty; output Empty;", "class control;"),
        ("feed", "input Empty; event Empty;", "errors [];"),
    ] {
        for member in ["", invalid] {
            fs::write(
                root.path().join("contract.trellis"),
                format!(
                    r#"
api "members@v1" {{
  version "1.0.0"; display_name "Members"; description "Surface members.";
  model Empty {{}}
  {kind} "Example" {{ version "v1"; {body} {member} }}
}}
"#
                ),
            )
            .unwrap();
            let compiled = trellis_idl::compile_project(root.path(), Default::default());
            if member.is_empty() {
                compiled.unwrap();
            } else {
                let error = compiled.unwrap_err();
                let rendered = format!("{error:?}");
                assert!(rendered.contains("contract.trellis"), "{rendered}");
                assert!(rendered.contains(kind), "{rendered}");
                assert!(
                    error
                        .labels()
                        .is_some_and(|mut labels| labels.next().is_some()),
                    "{rendered}"
                );
            }
        }
    }
}

#[test]
fn constraints_follow_type_semantics_and_accept_numeric_bounds() {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join("trellis.toml"), "format = 1\n").unwrap();
    for (ty, valid) in [
        ("number(minimum = -1, maximum = 2.5)", true),
        ("number(minimum = -1e2, maximum = 2.5e1)", true),
        ("string(min_length = 1, max_length = 5)", true),
        ("string(pattern = \"^.+$\", format = \"email\")", true),
        ("list<string>(min_items = 1, max_items = 5)", true),
        ("int(minimum = -1)", true),
        ("uint(minimum = 1)", true),
        (
            "int(minimum = -9007199254740991, maximum = 9007199254740991)",
            true,
        ),
        ("int(minimum = -1e2, maximum = 2.0)", true),
        ("int(minimum = -1.5)", false),
        ("uint(maximum = 2.5)", false),
        ("int(minimum = -9007199254740992)", false),
        ("uint(maximum = 9007199254740992)", false),
        ("string(minimum = 1)", false),
        ("int(min_length = 1)", false),
        ("number(pattern = \"x\")", false),
        ("number(format = \"email\")", false),
        ("string(min_items = 1)", false),
        ("string(min_length = -1)", false),
        ("list<string>(max_items = 2.5)", false),
        ("number(minimum = \"1\")", false),
        ("string(pattern = 1)", false),
        ("uint(minimum = -1)", false),
        ("uint(maximum = -1)", false),
        ("number(minimum = 3, maximum = 1)", false),
        ("string(min_length = 3, max_length = 1)", false),
        ("list<string>(min_items = 3, max_items = 1)", false),
        ("int(minimum = 1, minimum = 2)", false),
    ] {
        fs::write(
            root.path().join("contract.trellis"),
            format!(
                r#"
api "bounds@v1" {{
  version "1.0.0"; display_name "Bounds"; description "Meaningful constraints.";
  model Input {{ value: {ty}; }}
}}
"#
            ),
        )
        .unwrap();
        let compiled = trellis_idl::compile_project(root.path(), Default::default());
        if valid {
            let compiled = compiled.unwrap_or_else(|error| panic!("{ty}: {error:?}"));
            if ty == "number(minimum = -1, maximum = 2.5)" {
                let api = compiled.apis["bounds@v1"].normalized_value().unwrap();
                assert_eq!(
                    api["schemas"]["Input"]["properties"]["value"]["minimum"],
                    serde_json::json!(-1)
                );
                assert_eq!(
                    api["schemas"]["Input"]["properties"]["value"]["maximum"],
                    serde_json::json!(2.5)
                );
            }
        } else {
            let error = compiled.expect_err(ty);
            assert!(
                error
                    .labels()
                    .is_some_and(|mut labels| labels.next().is_some()),
                "{ty}: {error:?}"
            );
        }
    }
}
