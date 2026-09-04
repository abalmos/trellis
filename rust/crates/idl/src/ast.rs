use std::{collections::BTreeMap, ops::Range, path::PathBuf};

#[derive(Clone, Debug)]
pub(crate) struct Source {
    pub path: PathBuf,
    pub text: String,
}

#[derive(Clone, Debug)]
pub(crate) struct Spanned<T> {
    pub value: T,
    pub source: usize,
    pub span: Range<usize>,
}

#[derive(Debug)]
pub(crate) struct Project {
    pub sources: Vec<Source>,
    pub apis: Vec<Spanned<Api>>,
    pub participants: Vec<Spanned<Participant>>,
}

#[derive(Debug)]
pub(crate) struct Api {
    pub id: String,
    pub version: Option<Spanned<String>>,
    pub display_name: Option<Spanned<String>>,
    pub description: Option<Spanned<String>>,
    pub docs: Option<Docs>,
    pub schemas: BTreeMap<String, Spanned<SchemaDecl>>,
    pub exports: Vec<Spanned<String>>,
    pub errors: BTreeMap<String, Spanned<()>>,
    pub rpcs: BTreeMap<String, Spanned<Surface>>,
    pub operations: BTreeMap<String, Spanned<Surface>>,
    pub events: BTreeMap<String, Spanned<Surface>>,
    pub feeds: BTreeMap<String, Spanned<Surface>>,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Docs {
    pub summary: Option<String>,
    pub markdown: Option<String>,
}

#[derive(Debug)]
pub(crate) enum SchemaDecl {
    Model(Vec<Field>),
    Alias(Type),
    Enum(Vec<String>),
}

#[derive(Debug)]
pub(crate) struct Field {
    pub name: String,
    pub optional: bool,
    pub ty: Spanned<Type>,
}

#[derive(Clone, Debug)]
pub(crate) enum Type {
    Named(String),
    String(Vec<Constraint>),
    Bool,
    Integer {
        unsigned: bool,
        constraints: Vec<Constraint>,
    },
    Number(Vec<Constraint>),
    List(Box<Spanned<Type>>),
    Map(Box<Spanned<Type>>),
    Literal(String),
    Null,
    Union(Vec<Spanned<Type>>),
}

#[derive(Clone, Debug)]
pub(crate) struct Constraint {
    pub name: String,
    pub value: ConstraintValue,
}

#[derive(Clone, Debug)]
pub(crate) enum ConstraintValue {
    Integer(i64),
    String(String),
}

#[derive(Debug, Default)]
pub(crate) struct Surface {
    pub version: Option<Spanned<String>>,
    pub input: Option<Spanned<String>>,
    pub output: Option<Spanned<String>>,
    pub progress: Option<Spanned<String>>,
    pub event: Option<Spanned<String>>,
    pub errors: Vec<Spanned<String>>,
    pub transfer: Option<Transfer>,
    pub cancellable: bool,
    pub capabilities: BTreeMap<String, Vec<String>>,
    pub docs: Option<Docs>,
}

#[derive(Debug)]
pub(crate) enum Transfer {
    Receive,
    Send,
}

#[derive(Debug)]
pub(crate) struct Participant {
    pub id: String,
    pub kind: String,
    pub implements: Vec<Spanned<String>>,
    pub uses: BTreeMap<String, Spanned<ApiUse>>,
    pub state: BTreeMap<String, Spanned<State>>,
    pub stores: BTreeMap<String, Spanned<Resource>>,
    pub kv: BTreeMap<String, Spanned<Resource>>,
    pub jobs: BTreeMap<String, Spanned<Resource>>,
    pub bindings: BTreeMap<String, Spanned<Binding>>,
}

#[derive(Debug)]
pub(crate) struct State {
    pub kind: String,
    pub schema: Spanned<String>,
    pub state_version: Option<String>,
    pub docs: Option<Docs>,
}

#[derive(Debug)]
pub(crate) struct ApiUse {
    pub required: bool,
    pub api: Spanned<String>,
    pub selections: Vec<Spanned<Selection>>,
}

#[derive(Debug)]
pub(crate) struct Selection {
    pub action: String,
    pub surface: String,
    pub name: String,
    pub signal: Option<String>,
}

#[derive(Debug, Default)]
pub(crate) struct Resource {
    pub purpose: Option<String>,
    pub schema: Option<Spanned<String>>,
    pub payload: Option<Spanned<String>>,
    pub result: Option<Spanned<String>>,
    pub history: Option<u64>,
    pub ttl_ms: Option<u64>,
    pub max_object_bytes: Option<u64>,
    pub max_total_bytes: Option<u64>,
    pub docs: Option<Docs>,
}

#[derive(Debug, Default)]
pub(crate) struct Binding {
    pub store: Option<Spanned<String>>,
    pub key: Option<String>,
    pub content_type: Option<String>,
    pub metadata: Option<String>,
    pub expires_in_ms: Option<u64>,
}
