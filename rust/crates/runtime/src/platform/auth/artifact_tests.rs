use serde_json::Value;
use trellis_protocol::{
    lint_api_v1_authoring, lint_participant_v1_authoring, parse_api_v1, parse_participant_v1,
};

const AUTH_API_DIGEST: &str = "k-AVuZetf28XCxaYc2HEIzbafPeA63WeRgg0YmHtia0";
const AUTH_PARTICIPANT_DIGEST: &str = "plMjQVw7Fp3Q5R---qSTLjypJJaUU7A5hT_ayaTpd0k";
const ADMIN_PARTICIPANT_DIGEST: &str = "lQoimvKOcLmB4Acn3Q5roDNXQe4KlY3RjUvJ10hJ6CY";
const ADMIN_PARTICIPANT_NEEDS_DIGEST: &str = "bqA3XWyeUSFZUzDOLAjpCODp__crKL4hwd6mVf7nrIU";

#[test]
fn source_auth_artifacts_are_valid_and_digest_pinned() {
    let api_value: Value = serde_json::from_str(include_str!("../../../trellis.api.json"))
        .expect("parse auth API JSON");
    lint_api_v1_authoring(&api_value).expect("lint auth API");
    let api = parse_api_v1(&api_value).expect("validate auth API");
    assert_eq!(api.digest().expect("digest auth API"), AUTH_API_DIGEST);

    let participant_value: Value =
        serde_json::from_str(include_str!("../../../trellis.participant.json"))
            .expect("parse auth participant JSON");
    lint_participant_v1_authoring(&participant_value).expect("lint auth participant");
    let participant = parse_participant_v1(&participant_value).expect("validate auth participant");
    assert_eq!(
        participant.digest().expect("digest auth participant"),
        AUTH_PARTICIPANT_DIGEST
    );
    assert_eq!(
        participant
            .normalized_value()
            .expect("normalize auth participant")["implements"]["auth"]["apiDigest"],
        AUTH_API_DIGEST
    );

    let admin_value: Value = serde_json::from_str(include_str!(
        "../../../../trellis/artifacts/trellis.admin.participant.json"
    ))
    .expect("parse admin participant JSON");
    lint_participant_v1_authoring(&admin_value).expect("lint admin participant");
    let admin = parse_participant_v1(&admin_value).expect("validate admin participant");
    assert_eq!(
        admin.digest().expect("digest admin participant"),
        ADMIN_PARTICIPANT_DIGEST
    );
    let auth = parse_api_v1(&api_value).expect("validate auth API");
    let resolved = trellis_protocol::resolve_participant_v1(
        &admin,
        &std::collections::BTreeMap::from([(auth.id().to_owned(), auth)]),
    )
    .expect("resolve admin participant");
    assert_eq!(
        resolved.needs().digest().expect("digest admin needs"),
        ADMIN_PARTICIPANT_NEEDS_DIGEST
    );
}
