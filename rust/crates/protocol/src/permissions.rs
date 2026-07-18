use serde::{de::Error as _, Deserialize, Deserializer, Serialize};

use crate::{
    canonicalize_json, identifiers::compare_protocol_strings, sha256_base64url, ProtocolError,
};

/// The first canonical grant-set wire format.
pub const GRANT_SET_FORMAT_V1: &str = "trellis.grant-set.v1";

/// An externally visible API surface kind.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ApiSurfaceKindV1 {
    /// A request/reply RPC.
    Rpc,
    /// A caller-visible asynchronous operation.
    Operation,
    /// A published event.
    Event,
    /// A subscribable feed.
    Feed,
    /// Shared state.
    State,
}

impl ApiSurfaceKindV1 {
    fn as_str(self) -> &'static str {
        match self {
            Self::Rpc => "rpc",
            Self::Operation => "operation",
            Self::Event => "event",
            Self::Feed => "feed",
            Self::State => "state",
        }
    }
}

/// A participant-private resource kind.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ParticipantResourceKindV1 {
    /// A NATS key-value resource.
    Kv,
    /// A blob store resource.
    Store,
    /// A private Jobs queue.
    JobQueue,
    /// A durable contract-event consumer.
    EventConsumer,
    /// Participant-local private state.
    State,
}

impl ParticipantResourceKindV1 {
    fn as_str(self) -> &'static str {
        match self {
            Self::Kv => "kv",
            Self::Store => "store",
            Self::JobQueue => "jobQueue",
            Self::EventConsumer => "eventConsumer",
            Self::State => "state",
        }
    }
}

/// A machine-enforceable permission action.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PermissionActionV1 {
    /// Call an RPC.
    Call,
    /// Start an operation.
    Invoke,
    /// Observe an operation.
    Observe,
    /// Cancel an operation.
    Cancel,
    /// Send an operation control signal.
    Control,
    /// Publish an event.
    Publish,
    /// Subscribe to an event or feed.
    Subscribe,
    /// Read state or a resource.
    Read,
    /// Write state or a resource.
    Write,
    /// Delete a resource entry.
    Delete,
    /// Submit a private job.
    Submit,
    /// Process a private job.
    Process,
    /// Consume from a durable event consumer.
    Consume,
}

impl PermissionActionV1 {
    fn as_str(self) -> &'static str {
        match self {
            Self::Call => "call",
            Self::Invoke => "invoke",
            Self::Observe => "observe",
            Self::Cancel => "cancel",
            Self::Control => "control",
            Self::Publish => "publish",
            Self::Subscribe => "subscribe",
            Self::Read => "read",
            Self::Write => "write",
            Self::Delete => "delete",
            Self::Submit => "submit",
            Self::Process => "process",
            Self::Consume => "consume",
        }
    }
}

/// The exact API surface or participant resource targeted by a permission.
///
/// Owner and local-name strings must be nonempty, have no surrounding
/// whitespace, and contain no ASCII control characters.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum PermissionTargetV1 {
    /// An externally visible surface owned by an API artifact.
    #[serde(rename = "apiSurface")]
    ApiSurface {
        /// The owning versioned API identifier.
        api: String,
        /// The surface family.
        surface: ApiSurfaceKindV1,
        /// The API-local surface name.
        name: String,
    },
    /// A private resource owned by a participant artifact.
    #[serde(rename = "participantResource")]
    ParticipantResource {
        /// The owning participant identifier.
        participant: String,
        /// The resource family.
        resource: ParticipantResourceKindV1,
        /// The participant-local resource name.
        name: String,
    },
}

impl<'de> Deserialize<'de> for PermissionTargetV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(tag = "kind", deny_unknown_fields)]
        enum WireTarget {
            #[serde(rename = "apiSurface")]
            ApiSurface {
                api: String,
                surface: ApiSurfaceKindV1,
                name: String,
            },
            #[serde(rename = "participantResource")]
            ParticipantResource {
                participant: String,
                resource: ParticipantResourceKindV1,
                name: String,
            },
        }

        match WireTarget::deserialize(deserializer)? {
            WireTarget::ApiSurface { api, surface, name } => {
                Self::api_surface(api, surface, name).map_err(D::Error::custom)
            }
            WireTarget::ParticipantResource {
                participant,
                resource,
                name,
            } => Self::participant_resource(participant, resource, name).map_err(D::Error::custom),
        }
    }
}

impl PermissionTargetV1 {
    /// Construct an API-surface permission target.
    ///
    /// # Errors
    ///
    /// Returns [`ProtocolError::InvalidIdentifier`] when the API identifier or
    /// surface name violates the target string rules.
    pub fn api_surface(
        api: impl Into<String>,
        surface: ApiSurfaceKindV1,
        name: impl Into<String>,
    ) -> Result<Self, ProtocolError> {
        let target = Self::ApiSurface {
            api: api.into(),
            surface,
            name: name.into(),
        };
        target.validate()?;
        Ok(target)
    }

    /// Construct a participant-resource permission target.
    ///
    /// # Errors
    ///
    /// Returns [`ProtocolError::InvalidIdentifier`] when the participant
    /// identifier or resource name violates the target string rules.
    pub fn participant_resource(
        participant: impl Into<String>,
        resource: ParticipantResourceKindV1,
        name: impl Into<String>,
    ) -> Result<Self, ProtocolError> {
        let target = Self::ParticipantResource {
            participant: participant.into(),
            resource,
            name: name.into(),
        };
        target.validate()?;
        Ok(target)
    }

    /// Return the API identifier, surface kind, and local name for an API target.
    pub fn as_api_surface(&self) -> Option<(&str, ApiSurfaceKindV1, &str)> {
        match self {
            Self::ApiSurface { api, surface, name } => Some((api, *surface, name)),
            Self::ParticipantResource { .. } => None,
        }
    }

    fn validate(&self) -> Result<(), ProtocolError> {
        match self {
            Self::ApiSurface { api, name, .. } => {
                validate_identifier("API identifier", api)?;
                validate_identifier("surface name", name)
            }
            Self::ParticipantResource {
                participant, name, ..
            } => {
                validate_identifier("participant identifier", participant)?;
                validate_identifier("resource name", name)
            }
        }
    }

    fn ordering_key(&self) -> (&'static str, &str, &'static str, &str) {
        match self {
            Self::ApiSurface { api, surface, name } => ("apiSurface", api, surface.as_str(), name),
            Self::ParticipantResource {
                participant,
                resource,
                name,
            } => ("participantResource", participant, resource.as_str(), name),
        }
    }

    fn description(&self) -> &'static str {
        match self {
            Self::ApiSurface { surface, .. } => surface.as_str(),
            Self::ParticipantResource { resource, .. } => resource.as_str(),
        }
    }
}

/// One exact machine-enforceable permission.
///
/// Valid actions depend on the target family. For example, RPCs accept `call`,
/// events accept `publish` or `subscribe`, and private Jobs queues accept
/// `submit` or `process`.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PermissionAtomV1 {
    target: PermissionTargetV1,
    action: PermissionActionV1,
}

impl PermissionAtomV1 {
    /// Construct and validate a permission atom.
    ///
    /// # Errors
    ///
    /// Returns [`ProtocolError::InvalidIdentifier`] for an invalid target or
    /// [`ProtocolError::InvalidPermission`] when the action is not valid for the
    /// target family.
    pub fn new(
        target: PermissionTargetV1,
        action: PermissionActionV1,
    ) -> Result<Self, ProtocolError> {
        target.validate()?;
        let atom = Self { target, action };
        atom.validate_action()?;
        Ok(atom)
    }

    /// Return the exact permission target.
    pub fn target(&self) -> &PermissionTargetV1 {
        &self.target
    }

    /// Return the machine action.
    pub fn action(&self) -> PermissionActionV1 {
        self.action
    }

    fn validate_action(&self) -> Result<(), ProtocolError> {
        let valid = match (&self.target, self.action) {
            (PermissionTargetV1::ApiSurface { surface, .. }, action) => match surface {
                ApiSurfaceKindV1::Rpc => matches!(action, PermissionActionV1::Call),
                ApiSurfaceKindV1::Operation => matches!(
                    action,
                    PermissionActionV1::Invoke
                        | PermissionActionV1::Observe
                        | PermissionActionV1::Cancel
                        | PermissionActionV1::Control
                ),
                ApiSurfaceKindV1::Event => matches!(
                    action,
                    PermissionActionV1::Publish | PermissionActionV1::Subscribe
                ),
                ApiSurfaceKindV1::Feed => matches!(action, PermissionActionV1::Subscribe),
                ApiSurfaceKindV1::State => {
                    matches!(action, PermissionActionV1::Read | PermissionActionV1::Write)
                }
            },
            (PermissionTargetV1::ParticipantResource { resource, .. }, action) => match resource {
                ParticipantResourceKindV1::Kv | ParticipantResourceKindV1::Store => matches!(
                    action,
                    PermissionActionV1::Read
                        | PermissionActionV1::Write
                        | PermissionActionV1::Delete
                ),
                ParticipantResourceKindV1::JobQueue => {
                    matches!(
                        action,
                        PermissionActionV1::Submit | PermissionActionV1::Process
                    )
                }
                ParticipantResourceKindV1::EventConsumer => {
                    matches!(action, PermissionActionV1::Consume)
                }
                ParticipantResourceKindV1::State => {
                    matches!(action, PermissionActionV1::Read | PermissionActionV1::Write)
                }
            },
        };

        if valid {
            Ok(())
        } else {
            Err(ProtocolError::InvalidPermission {
                action: self.action.as_str().to_string(),
                target: self.target.description(),
            })
        }
    }

    fn ordering_key(&self) -> (&'static str, &str, &'static str, &str, &'static str) {
        let (kind, owner, target_kind, name) = self.target.ordering_key();
        (kind, owner, target_kind, name, self.action.as_str())
    }
}

impl<'de> Deserialize<'de> for PermissionAtomV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct WireAtom {
            target: PermissionTargetV1,
            action: PermissionActionV1,
        }

        let wire = WireAtom::deserialize(deserializer)?;
        Self::new(wire.target, wire.action).map_err(D::Error::custom)
    }
}

/// A named capability's normalized machine permissions.
///
/// A capability is an authoring and explanation grouping. It does not itself
/// confer authority; enforcement uses exact atoms in a [`GrantSetV1`].
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CapabilityDefinitionV1 {
    allows: Vec<PermissionAtomV1>,
}

impl CapabilityDefinitionV1 {
    /// Construct a capability and normalize duplicate permissions.
    pub fn new(allows: Vec<PermissionAtomV1>) -> Self {
        Self {
            allows: normalize_permissions(allows),
        }
    }

    /// Return normalized machine permissions in canonical order.
    pub fn allows(&self) -> &[PermissionAtomV1] {
        &self.allows
    }
}

impl<'de> Deserialize<'de> for CapabilityDefinitionV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct WireCapability {
            allows: Vec<PermissionAtomV1>,
        }

        Ok(Self::new(WireCapability::deserialize(deserializer)?.allows))
    }
}

/// Human-facing consent copy, separate from enforceable permissions.
///
/// These strings may be shown during owner review, but are non-authoritative.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ConsentMetadataV1 {
    /// Short consent heading.
    pub title: String,
    /// Plain-language capability description.
    pub description: String,
    /// Plain-language consequence of granting consent.
    pub consequence: String,
}

/// A normalized, content-addressed set of machine permissions.
///
/// Construction sorts by UTF-16 code units and removes duplicate atoms. The
/// digest therefore identifies the enforceable set rather than authored order.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GrantSetV1 {
    format: String,
    permissions: Vec<PermissionAtomV1>,
}

impl GrantSetV1 {
    /// Construct a normalized grant set in the current format.
    pub fn new(permissions: Vec<PermissionAtomV1>) -> Self {
        Self {
            format: GRANT_SET_FORMAT_V1.to_string(),
            permissions: normalize_permissions(permissions),
        }
    }

    /// Return the grant-set wire format.
    pub fn format(&self) -> &str {
        &self.format
    }

    /// Return permissions in canonical semantic order.
    pub fn permissions(&self) -> &[PermissionAtomV1] {
        &self.permissions
    }

    /// Render the normalized grant set as canonical Trellis JSON.
    ///
    /// # Errors
    ///
    /// Returns a serialization or canonicalization [`ProtocolError`] if the
    /// normalized value cannot be encoded.
    pub fn canonical_json(&self) -> Result<String, ProtocolError> {
        canonicalize_json(&serde_json::to_value(self)?)
    }

    /// Return the grant set's SHA-256/base64url content digest.
    ///
    /// # Errors
    ///
    /// Returns a serialization or canonicalization [`ProtocolError`] if the
    /// normalized value cannot be encoded before hashing.
    pub fn digest(&self) -> Result<String, ProtocolError> {
        Ok(sha256_base64url(&self.canonical_json()?))
    }
}

impl<'de> Deserialize<'de> for GrantSetV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct WireGrantSet {
            format: String,
            permissions: Vec<PermissionAtomV1>,
        }

        let wire = WireGrantSet::deserialize(deserializer)?;
        if wire.format != GRANT_SET_FORMAT_V1 {
            return Err(D::Error::custom(ProtocolError::InvalidGrantSetFormat(
                wire.format,
            )));
        }
        Ok(Self::new(wire.permissions))
    }
}

fn normalize_permissions(mut permissions: Vec<PermissionAtomV1>) -> Vec<PermissionAtomV1> {
    permissions.sort_by(compare_permission_atoms);
    permissions.dedup();
    permissions
}

fn compare_permission_atoms(
    left: &PermissionAtomV1,
    right: &PermissionAtomV1,
) -> std::cmp::Ordering {
    let left = left.ordering_key();
    let right = right.ordering_key();
    compare_protocol_strings(left.0, right.0)
        .then_with(|| compare_protocol_strings(left.1, right.1))
        .then_with(|| compare_protocol_strings(left.2, right.2))
        .then_with(|| compare_protocol_strings(left.3, right.3))
        .then_with(|| compare_protocol_strings(left.4, right.4))
}

fn validate_identifier(field: &'static str, value: &str) -> Result<(), ProtocolError> {
    let reason = if value.is_empty() {
        Some("must not be empty")
    } else if value.trim() != value {
        Some("must not have leading or trailing whitespace")
    } else if value.chars().any(|character| character.is_ascii_control()) {
        Some("must not contain ASCII control characters")
    } else {
        None
    };

    reason.map_or(Ok(()), |reason| {
        Err(ProtocolError::InvalidIdentifier { field, reason })
    })
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use serde::Deserialize;
    use serde_json::Value;

    use super::*;

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Vector {
        name: String,
        valid: bool,
        input: Value,
        #[serde(rename = "normalizedJson")]
        normalized_json: Option<String>,
        digest: Option<String>,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields, rename_all = "camelCase")]
    struct OrderingRule {
        tuple: Vec<String>,
        string_comparison: String,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Fixture {
        ordering: OrderingRule,
        vectors: Vec<Vector>,
    }

    #[test]
    fn grant_sets_match_shared_conformance_vectors() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../conformance/grant-set/vectors.json");
        let fixture: Fixture = serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();
        assert_eq!(
            fixture.ordering.tuple,
            [
                "target.kind",
                "target.owner",
                "target.surfaceOrResource",
                "target.name",
                "action"
            ]
        );
        assert_eq!(
            fixture.ordering.string_comparison,
            "UTF-16 code-unit lexicographic"
        );

        for vector in fixture.vectors {
            let parsed = serde_json::from_value::<GrantSetV1>(vector.input);
            if vector.valid {
                let grant_set = parsed.unwrap_or_else(|error| panic!("{}: {error}", vector.name));
                assert_eq!(
                    grant_set.canonical_json().unwrap(),
                    vector.normalized_json.unwrap(),
                    "{}",
                    vector.name
                );
                assert_eq!(
                    grant_set.digest().unwrap(),
                    vector.digest.unwrap(),
                    "{}",
                    vector.name
                );
            } else {
                assert!(parsed.is_err(), "{} unexpectedly parsed", vector.name);
            }
        }
    }

    #[test]
    fn permission_atom_round_trips_and_accepts_current_names() {
        let atom = PermissionAtomV1::new(
            PermissionTargetV1::api_surface(
                "trellis.core@v1",
                ApiSurfaceKindV1::Rpc,
                "Documents.Get",
            )
            .unwrap(),
            PermissionActionV1::Call,
        )
        .unwrap();
        let json = serde_json::to_string(&atom).unwrap();
        assert_eq!(
            serde_json::from_str::<PermissionAtomV1>(&json).unwrap(),
            atom
        );

        PermissionTargetV1::participant_resource(
            "billing-refunds",
            ParticipantResourceKindV1::JobQueue,
            "reindex",
        )
        .unwrap();
    }

    #[test]
    fn permission_target_deserialization_validates_direct_values() {
        for invalid in [
            serde_json::json!({
                "kind": "apiSurface",
                "api": "",
                "surface": "rpc",
                "name": "Documents.Get"
            }),
            serde_json::json!({
                "kind": "participantResource",
                "participant": " documents-worker",
                "resource": "jobQueue",
                "name": "reindex"
            }),
            serde_json::json!({
                "kind": "apiSurface",
                "api": "documents@v1",
                "surface": "rpc",
                "name": "Documents.\nGet"
            }),
            serde_json::json!({
                "kind": "apiSurface",
                "api": "documents@v1",
                "surface": "rpc",
                "name": "Documents.Get",
                "extra": true
            }),
        ] {
            assert!(serde_json::from_value::<PermissionTargetV1>(invalid).is_err());
        }

        let value = serde_json::json!({
            "kind": "apiSurface",
            "api": "trellis.core@v1",
            "surface": "rpc",
            "name": "Documents.Get"
        });
        let target: PermissionTargetV1 = serde_json::from_value(value.clone()).unwrap();
        assert_eq!(serde_json::to_value(target).unwrap(), value);
    }

    #[test]
    fn consent_metadata_is_not_part_of_grant_identity() {
        let grant_set = GrantSetV1::new(Vec::new());
        let before = grant_set.digest().unwrap();
        let _: ConsentMetadataV1 = serde_json::from_value(serde_json::json!({
            "title": "View documents",
            "description": "Read documents available to your account.",
            "consequence": "This application can view your documents."
        }))
        .unwrap();
        assert_eq!(grant_set.digest().unwrap(), before);
    }

    #[test]
    fn grant_identity_is_independent_of_input_order() {
        let first = PermissionAtomV1::new(
            PermissionTargetV1::api_surface("documents@v1", ApiSurfaceKindV1::Rpc, "Documents.Get")
                .unwrap(),
            PermissionActionV1::Call,
        )
        .unwrap();
        let second = PermissionAtomV1::new(
            PermissionTargetV1::participant_resource(
                "documents-worker",
                ParticipantResourceKindV1::JobQueue,
                "reindex",
            )
            .unwrap(),
            PermissionActionV1::Process,
        )
        .unwrap();

        let forward = GrantSetV1::new(vec![first.clone(), second.clone()]);
        let reverse = GrantSetV1::new(vec![second, first.clone()]);
        assert_eq!(
            forward.canonical_json().unwrap(),
            reverse.canonical_json().unwrap()
        );
        assert_eq!(forward.digest().unwrap(), reverse.digest().unwrap());

        let capability = CapabilityDefinitionV1::new(vec![first.clone(), first]);
        assert_eq!(capability.allows().len(), 1);
    }

    #[test]
    fn capability_and_consent_objects_are_closed() {
        assert!(
            serde_json::from_value::<CapabilityDefinitionV1>(serde_json::json!({
                "allows": [],
                "title": "not machine policy"
            }))
            .is_err()
        );
        assert!(
            serde_json::from_value::<ConsentMetadataV1>(serde_json::json!({
                "title": "View documents",
                "description": "Read documents.",
                "consequence": "Documents are visible.",
                "allows": []
            }))
            .is_err()
        );
    }

    #[test]
    fn permission_and_grant_objects_are_closed() {
        for atom in [
            serde_json::json!({
                "target": { "kind": "apiSurface", "api": "documents@v1", "surface": "rpc", "name": "Documents.Get" },
                "action": "call",
                "extra": true
            }),
            serde_json::json!({
                "target": { "kind": "apiSurface", "api": "documents@v1", "surface": "rpc", "name": "Documents.Get", "extra": true },
                "action": "call"
            }),
        ] {
            assert!(serde_json::from_value::<PermissionAtomV1>(atom).is_err());
        }

        assert!(serde_json::from_value::<GrantSetV1>(serde_json::json!({
            "format": GRANT_SET_FORMAT_V1,
            "permissions": [],
            "extra": true
        }))
        .is_err());
    }
}
