//! Contextual participant resolution and deterministic authority evidence.
//!
//! Intrinsic parsing proves that each [`ParticipantArtifact`] is internally
//! coherent. Resolution additionally requires validated [`ApiArtifact`] values
//! keyed by exact canonical API ID and pinned to the digests in the participant.
//! It verifies selected surfaces, operation cancellation and signals, event
//! consumers, provider transfer mappings, and schema extraction pointers.
//!
//! Pointer guarantees are stronger than syntactic JSON Pointer validity: every
//! schema-valid payload must contain the target with the required type. Required
//! object properties and array `minItems` establish reachability. `allOf`
//! combines guarantees, while `anyOf` and `oneOf` require every possible branch
//! to guarantee the pointer. Local `#/...` references and anchors are resolved;
//! runtime extraction follows `jsonptr` token decoding and canonical array-index
//! rules.
//!
//! Derived needs are alias-independent and separate required from optional grant
//! sets and private resources. Capability evidence explains fully requested
//! permissions but never expands the exact grant set; unmatched atoms remain in
//! `uncovered_permissions`. A proposal is review evidence only. Derivation does
//! not approve, reject, classify, persist, or materialize authority.

use std::collections::{BTreeMap, BTreeSet};

use jsonptr::{index::Index, Pointer, PointerBuf, Token};
use serde::Serialize;
use serde_json::{Map, Value};

use crate::{
    canonicalize_json, digest_json, identifiers::compare_protocol_strings,
    schema_profile::resolve_local_schema, ApiArtifact, ApiSurfaceKind, ConsentMetadata,
    DerivedEventSubjects, GrantSet, ParticipantArtifact, ParticipantKind, ParticipantResourceKind,
    PermissionAction, PermissionAtom, PermissionTarget, ProtocolError, ResolutionErrorCode,
};

/// The first canonical participant-needs format.
pub const PARTICIPANT_NEEDS_FORMAT_V1: &str = "trellis.participant-needs.v1";

/// The first deterministic authority-proposal format.
pub const AUTHORITY_PROPOSAL_FORMAT_V1: &str = "trellis.authority-proposal.v1";

/// An exact API identity participating in needs.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProvidedApiNeed {
    api: String,
    api_digest: String,
}

impl ProvidedApiNeed {
    /// Return the canonical API identifier.
    pub fn api(&self) -> &str {
        &self.api
    }

    /// Return the exact API digest.
    pub fn api_digest(&self) -> &str {
        &self.api_digest
    }
}

/// Alias-independent private resource needs grouped by resource family.
///
/// Resource identity uses participant ID and resource name, never the local API
/// aliases used to wire dependencies.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParticipantResourceNeeds {
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    state: BTreeMap<String, Value>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    job_queues: BTreeMap<String, Value>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    event_consumers: BTreeMap<String, Value>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    kv: BTreeMap<String, Value>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    stores: BTreeMap<String, Value>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    operation_transfers: BTreeMap<String, Value>,
}

impl ParticipantResourceNeeds {
    /// Return whether this section has no resource needs.
    pub fn is_empty(&self) -> bool {
        self.state.is_empty()
            && self.job_queues.is_empty()
            && self.event_consumers.is_empty()
            && self.kv.is_empty()
            && self.stores.is_empty()
            && self.operation_transfers.is_empty()
    }

    /// Return participant-local state needs.
    pub fn state(&self) -> &BTreeMap<String, Value> {
        &self.state
    }

    /// Return private Jobs queue needs.
    pub fn job_queues(&self) -> &BTreeMap<String, Value> {
        &self.job_queues
    }

    /// Return durable event-consumer needs.
    pub fn event_consumers(&self) -> &BTreeMap<String, Value> {
        &self.event_consumers
    }

    /// Return KV resource needs.
    pub fn kv(&self) -> &BTreeMap<String, Value> {
        &self.kv
    }

    /// Return object-store needs.
    pub fn stores(&self) -> &BTreeMap<String, Value> {
        &self.stores
    }

    /// Return provider operation-transfer bindings.
    pub fn operation_transfers(&self) -> &BTreeMap<String, Value> {
        &self.operation_transfers
    }
}

/// One required or optional participant-needs section.
///
/// Required and optional sections remain separate so deployment policy can
/// distinguish mandatory authority from optional integrations.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParticipantNeedsSection {
    apis: Vec<ProvidedApiNeed>,
    grant_set: GrantSet,
    resources: ParticipantResourceNeeds,
}

impl ParticipantNeedsSection {
    /// Return exact referenced API identities in canonical order.
    pub fn apis(&self) -> &[ProvidedApiNeed] {
        &self.apis
    }

    /// Return the exact normalized grant set.
    pub fn grant_set(&self) -> &GrantSet {
        &self.grant_set
    }

    /// Return private resource needs for this requirement level.
    pub fn resources(&self) -> &ParticipantResourceNeeds {
        &self.resources
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct NeedsParticipant {
    id: String,
    kind: ParticipantKind,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
struct ProvidedNeeds {
    apis: Vec<ProvidedApiNeed>,
}

/// Canonical alias-independent machine needs for one participant.
///
/// # Digest identity
///
/// The digest covers the format, participant ID and kind, exact provided API
/// IDs and digests, and both normalized needs sections. It excludes the
/// participant artifact digest, local API aliases, and human presentation text.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParticipantNeeds {
    format: String,
    participant: NeedsParticipant,
    provides: ProvidedNeeds,
    required: ParticipantNeedsSection,
    optional: ParticipantNeedsSection,
}

impl ParticipantNeeds {
    /// Serialize the canonical semantic needs value.
    ///
    /// # Errors
    ///
    /// Returns [`ProtocolError::Json`] if the derived value cannot be encoded.
    pub fn normalized_value(&self) -> Result<Value, ProtocolError> {
        Ok(serde_json::to_value(self)?)
    }

    /// Render needs as RFC 8785 canonical JSON.
    ///
    /// # Errors
    ///
    /// Returns a serialization or canonicalization [`ProtocolError`].
    pub fn canonical_json(&self) -> Result<String, ProtocolError> {
        canonicalize_json(&self.normalized_value()?)
    }

    /// Compute the content digest of canonical machine needs.
    ///
    /// # Errors
    ///
    /// Returns a serialization or canonicalization [`ProtocolError`].
    pub fn digest(&self) -> Result<String, ProtocolError> {
        digest_json(&self.normalized_value()?)
    }

    /// Return required machine needs.
    pub fn required(&self) -> &ParticipantNeedsSection {
        &self.required
    }

    /// Return optional machine needs.
    pub fn optional(&self) -> &ParticipantNeedsSection {
        &self.optional
    }

    /// Return exact APIs provided by the participant.
    pub fn provided_apis(&self) -> &[ProvidedApiNeed] {
        &self.provides.apis
    }
}

/// Complete alias-independent provider-side evidence derived from one pinned API.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedProvidedApi {
    api: String,
    api_digest: String,
    rpc: BTreeMap<String, String>,
    operations: BTreeMap<String, ResolvedProvidedOperation>,
    events: BTreeMap<String, DerivedEventSubjects>,
    feeds: BTreeMap<String, String>,
    state: Vec<String>,
}

/// Canonical provider evidence for one operation and its named signals.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ResolvedProvidedOperation {
    subject: String,
    signals: Vec<String>,
}

impl ResolvedProvidedOperation {
    /// Return the canonical operation subject.
    pub fn subject(&self) -> &str {
        &self.subject
    }

    /// Return logical operation signal names.
    pub fn signals(&self) -> &[String] {
        &self.signals
    }
}

impl ResolvedProvidedApi {
    /// Return the canonical API identifier.
    pub fn api(&self) -> &str {
        &self.api
    }

    /// Return the exact API digest.
    pub fn api_digest(&self) -> &str {
        &self.api_digest
    }

    /// Return logical RPC names and canonical subjects.
    pub fn rpc(&self) -> &BTreeMap<String, String> {
        &self.rpc
    }

    /// Return logical operations with canonical subjects and signal names.
    pub fn operations(&self) -> &BTreeMap<String, ResolvedProvidedOperation> {
        &self.operations
    }

    /// Return logical event names and canonical subject patterns.
    pub fn events(&self) -> &BTreeMap<String, DerivedEventSubjects> {
        &self.events
    }

    /// Return logical feed names and canonical subjects.
    pub fn feeds(&self) -> &BTreeMap<String, String> {
        &self.feeds
    }

    /// Return logical state surface names.
    pub fn state(&self) -> &[String] {
        &self.state
    }
}

/// A resolved implemented API retaining its local wiring alias.
///
/// The alias is retained for runtime wiring, while [`Self::provided`] carries the
/// alias-independent authority identity.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedImplementedApi {
    alias: String,
    provided: ResolvedProvidedApi,
}

impl ResolvedImplementedApi {
    /// Return the participant-local API alias.
    pub fn alias(&self) -> &str {
        &self.alias
    }

    /// Return complete alias-independent provider evidence.
    pub fn provided(&self) -> &ResolvedProvidedApi {
        &self.provided
    }
}

/// A resolved used API retaining its local wiring alias and exact grants.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedUsedApi {
    alias: String,
    api: String,
    api_digest: String,
    grant_set: GrantSet,
}

impl ResolvedUsedApi {
    /// Return the participant-local API alias.
    pub fn alias(&self) -> &str {
        &self.alias
    }

    /// Return the canonical API identifier.
    pub fn api(&self) -> &str {
        &self.api
    }

    /// Return exact grants selected through this API reference.
    pub fn grant_set(&self) -> &GrantSet {
        &self.grant_set
    }

    /// Return the exact pinned API digest.
    pub fn api_digest(&self) -> &str {
        &self.api_digest
    }
}

/// One fully requested named capability and its optional consent evidence.
///
/// Evidence includes the exact API digest that defined the capability. It is an
/// explanation of atoms already present in a grant set, never permission
/// expansion. Consent remains non-authoritative presentation text.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorityCapabilityEvidence {
    api: String,
    api_digest: String,
    name: String,
    allows: Vec<PermissionAtom>,
    #[serde(skip_serializing_if = "Option::is_none")]
    consent: Option<ConsentMetadata>,
}

impl AuthorityCapabilityEvidence {
    /// Return the API that owns the capability.
    pub fn api(&self) -> &str {
        &self.api
    }

    /// Return the exact API digest that defined this capability.
    pub fn api_digest(&self) -> &str {
        &self.api_digest
    }

    /// Return the capability name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Return exact permissions explained by this capability.
    pub fn allows(&self) -> &[PermissionAtom] {
        &self.allows
    }

    /// Return matching human consent evidence, when declared.
    pub fn consent(&self) -> Option<&ConsentMetadata> {
        self.consent.as_ref()
    }
}

/// One exact required or optional authority-proposal section.
///
/// Permissions not covered by a fully requested named capability are preserved
/// explicitly in [`Self::uncovered_permissions`].
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorityProposalSection {
    grant_set: GrantSet,
    capabilities: Vec<AuthorityCapabilityEvidence>,
    uncovered_permissions: Vec<PermissionAtom>,
    resources: ParticipantResourceNeeds,
}

impl AuthorityProposalSection {
    /// Return the exact grant set; capability evidence never expands it.
    pub fn grant_set(&self) -> &GrantSet {
        &self.grant_set
    }

    /// Return fully requested capability evidence.
    pub fn capabilities(&self) -> &[AuthorityCapabilityEvidence] {
        &self.capabilities
    }

    /// Return exact permissions not explained by a fully requested capability.
    pub fn uncovered_permissions(&self) -> &[PermissionAtom] {
        &self.uncovered_permissions
    }

    /// Return private resource needs reviewed in this section.
    pub fn resources(&self) -> &ParticipantResourceNeeds {
        &self.resources
    }
}

/// Deterministic owner-reviewable evidence for one participant request.
///
/// The complete normalized value includes the presented participant digest,
/// provider evidence, grant sets, capabilities, uncovered atoms, and resources.
/// The fingerprint deliberately includes only format, participant ID and kind,
/// and needs digest, making equivalent authority changes independent of artifact
/// presentation and local aliases.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorityProposal {
    format: String,
    participant_id: String,
    participant_kind: ParticipantKind,
    participant_digest: String,
    needs_digest: String,
    provides: Vec<ResolvedProvidedApi>,
    required: AuthorityProposalSection,
    optional: AuthorityProposalSection,
}

impl AuthorityProposal {
    /// Return the digest of canonical machine needs.
    pub fn needs_digest(&self) -> &str {
        &self.needs_digest
    }

    /// Serialize complete review evidence, including participant artifact evidence.
    ///
    /// # Errors
    ///
    /// Returns [`ProtocolError::Json`] if the proposal cannot be encoded.
    pub fn normalized_value(&self) -> Result<Value, ProtocolError> {
        Ok(serde_json::to_value(self)?)
    }

    /// Render complete proposal evidence as canonical JSON.
    ///
    /// # Errors
    ///
    /// Returns a serialization or canonicalization [`ProtocolError`].
    pub fn canonical_json(&self) -> Result<String, ProtocolError> {
        canonicalize_json(&self.normalized_value()?)
    }

    /// Compute the authority-change fingerprint, excluding participant artifact identity.
    ///
    /// # Errors
    ///
    /// Returns a serialization or canonicalization [`ProtocolError`].
    pub fn fingerprint(&self) -> Result<String, ProtocolError> {
        digest_json(&serde_json::json!({
            "format": self.format,
            "participantId": self.participant_id,
            "participantKind": self.participant_kind,
            "needsDigest": self.needs_digest,
        }))
    }

    /// Return required authority review evidence.
    pub fn required(&self) -> &AuthorityProposalSection {
        &self.required
    }

    /// Return optional authority review evidence.
    pub fn optional(&self) -> &AuthorityProposalSection {
        &self.optional
    }

    /// Return the exact presented participant artifact digest.
    pub fn participant_digest(&self) -> &str {
        &self.participant_digest
    }

    /// Return alias-independent provider evidence.
    pub fn provides(&self) -> &[ResolvedProvidedApi] {
        &self.provides
    }
}

/// Fully contextually validated participant resolution.
///
/// This is the boundary at which exact API pins and selected API members have
/// been proven against supplied artifacts. It retains aliases for wiring while
/// exposing alias-independent needs and proposal evidence.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedParticipant {
    participant_id: String,
    participant_digest: String,
    participant_kind: ParticipantKind,
    implemented_apis: Vec<ResolvedImplementedApi>,
    required_apis: Vec<ResolvedUsedApi>,
    optional_apis: Vec<ResolvedUsedApi>,
    needs: ParticipantNeeds,
    proposal: AuthorityProposal,
}

impl ResolvedParticipant {
    /// Return the participant identifier.
    pub fn participant_id(&self) -> &str {
        &self.participant_id
    }

    /// Return the presented participant artifact digest.
    pub fn participant_digest(&self) -> &str {
        &self.participant_digest
    }

    /// Return the participant kind.
    pub fn participant_kind(&self) -> ParticipantKind {
        self.participant_kind
    }

    /// Return alias-independent canonical machine needs.
    pub fn needs(&self) -> &ParticipantNeeds {
        &self.needs
    }

    /// Return deterministic owner-reviewable proposal evidence.
    pub fn proposal(&self) -> &AuthorityProposal {
        &self.proposal
    }

    /// Return resolved implemented APIs with local aliases retained for wiring.
    pub fn implemented_apis(&self) -> &[ResolvedImplementedApi] {
        &self.implemented_apis
    }

    /// Return resolved required API uses.
    pub fn required_apis(&self) -> &[ResolvedUsedApi] {
        &self.required_apis
    }

    /// Return resolved optional API uses.
    pub fn optional_apis(&self) -> &[ResolvedUsedApi] {
        &self.optional_apis
    }
}

/// Resolve one validated participant against its exact, validated API artifacts.
///
/// The API map must contain every implemented, required, and optional API under
/// its canonical ID, and each artifact digest must equal the participant pin.
/// Diagnostics use typed [`ResolutionErrorCode`] values and RFC 6901 paths into
/// participant-authored members.
///
/// # Errors
///
/// Returns [`ProtocolError::ParticipantResolution`] for missing or mismatched
/// APIs, unknown selected surfaces or signals, invalid event-consumer or transfer
/// mappings, and extraction pointers that are not guaranteed for every
/// schema-valid payload. Serialization, digest, subject, and schema errors are
/// propagated.
pub fn resolve_participant(
    participant: &ParticipantArtifact,
    apis: &BTreeMap<String, ApiArtifact>,
) -> Result<ResolvedParticipant, ProtocolError> {
    let participant_value = participant.normalized_value()?;
    let participant_id = participant.id().to_owned();
    let participant_digest = participant.digest()?;
    let schemas = object_at(&participant_value, "schemas");
    let mut resolved_by_alias: BTreeMap<String, (&ApiArtifact, String)> = BTreeMap::new();

    for (section, references) in [
        ("implements", object_at(&participant_value, "implements")),
        (
            "uses/required",
            nested_object_at(&participant_value, &["uses", "required"]),
        ),
        (
            "uses/optional",
            nested_object_at(&participant_value, &["uses", "optional"]),
        ),
    ] {
        for (alias, reference) in references {
            let api_id = string_at(reference, "api");
            let pinned_digest = string_at(reference, "apiDigest");
            let path = pointer(section.split('/').chain(std::iter::once(alias.as_str())));
            let api = apis.get(api_id).ok_or_else(|| {
                resolution_error(
                    ResolutionErrorCode::MissingApi,
                    &participant_id,
                    Some(alias),
                    Some(api_id),
                    path.clone(),
                    format!("API '{api_id}' was not supplied"),
                )
            })?;
            if api.id() != api_id {
                return Err(resolution_error(
                    ResolutionErrorCode::MissingApi,
                    &participant_id,
                    Some(alias),
                    Some(api_id),
                    path.clone(),
                    format!(
                        "API map entry '{api_id}' contains artifact '{}' instead",
                        api.id()
                    ),
                ));
            }
            let actual_digest = api.digest()?;
            if actual_digest != pinned_digest {
                return Err(resolution_error(
                    ResolutionErrorCode::ApiDigestMismatch,
                    &participant_id,
                    Some(alias),
                    Some(api_id),
                    path.with_trailing_token("apiDigest"),
                    format!("expected digest '{pinned_digest}', received '{actual_digest}'"),
                ));
            }
            validate_api_schema_pointers(participant.id(), alias, api)?;
            resolved_by_alias.insert(alias.to_owned(), (api, actual_digest));
        }
    }

    validate_job_pointers(&participant_id, &participant_value, schemas)?;

    let mut implemented_apis = Vec::new();
    let mut provided_needs = Vec::new();
    let mut provided_authority = Vec::new();
    let mut required_resources = derive_required_resources(
        &participant_id,
        &participant_value,
        schemas,
        &resolved_by_alias,
    )?;
    let optional_resources = derive_optional_resources(&participant_value, schemas)?;

    for (alias, implementation) in object_at(&participant_value, "implements") {
        let (api, digest) = &resolved_by_alias[alias];
        validate_transfers(
            &participant_id,
            alias,
            implementation,
            api,
            &participant_value,
            &mut required_resources.operation_transfers,
        )?;
        let provided = derive_provided_api(api, digest)?;
        provided_needs.push(ProvidedApiNeed {
            api: api.id().to_owned(),
            api_digest: digest.to_owned(),
        });
        provided_authority.push(provided.clone());
        implemented_apis.push(ResolvedImplementedApi {
            alias: alias.to_owned(),
            provided,
        });
    }
    provided_needs.sort_by(|left, right| compare_protocol_strings(&left.api, &right.api));
    provided_authority.sort_by(|left, right| compare_protocol_strings(&left.api, &right.api));
    implemented_apis.sort_by(|left, right| {
        compare_protocol_strings(left.provided.api(), right.provided.api())
            .then_with(|| compare_protocol_strings(&left.alias, &right.alias))
    });

    let required_apis = resolve_used_group(
        &participant_id,
        "required",
        nested_object_at(&participant_value, &["uses", "required"]),
        &resolved_by_alias,
    )?;
    let optional_apis = resolve_used_group(
        &participant_id,
        "optional",
        nested_object_at(&participant_value, &["uses", "optional"]),
        &resolved_by_alias,
    )?;
    let mut required_permissions = required_apis
        .iter()
        .flat_map(|used| used.grant_set.permissions().iter().cloned())
        .collect::<Vec<_>>();
    for provided in &provided_authority {
        for event in provided.events().keys() {
            required_permissions.push(PermissionAtom::new(
                PermissionTarget::api_surface(provided.api(), ApiSurfaceKind::Event, event)?,
                PermissionAction::Publish,
            )?);
        }
    }
    required_permissions.extend(resource_permissions(&participant_id, &required_resources)?);
    let required_grants = GrantSet::new(required_permissions);
    let mut optional_permissions = optional_apis
        .iter()
        .flat_map(|used| used.grant_set.permissions().iter().cloned())
        .collect::<Vec<_>>();
    optional_permissions.extend(resource_permissions(&participant_id, &optional_resources)?);
    let optional_grants = GrantSet::new(optional_permissions);
    let needs = ParticipantNeeds {
        format: PARTICIPANT_NEEDS_FORMAT_V1.to_owned(),
        participant: NeedsParticipant {
            id: participant_id.clone(),
            kind: participant.kind(),
        },
        provides: ProvidedNeeds {
            apis: provided_needs,
        },
        required: ParticipantNeedsSection {
            apis: api_needs(&required_apis),
            grant_set: required_grants.clone(),
            resources: required_resources.clone(),
        },
        optional: ParticipantNeedsSection {
            apis: api_needs(&optional_apis),
            grant_set: optional_grants.clone(),
            resources: optional_resources.clone(),
        },
    };
    let needs_digest = needs.digest()?;
    let proposal = AuthorityProposal {
        format: AUTHORITY_PROPOSAL_FORMAT_V1.to_owned(),
        participant_id: participant_id.clone(),
        participant_kind: participant.kind(),
        participant_digest: participant_digest.clone(),
        needs_digest,
        provides: provided_authority,
        required: derive_proposal_section(
            &required_grants,
            required_resources,
            &required_apis,
            &implemented_apis,
            &resolved_by_alias,
        )?,
        optional: derive_proposal_section(
            &optional_grants,
            optional_resources,
            &optional_apis,
            &[],
            &resolved_by_alias,
        )?,
    };

    Ok(ResolvedParticipant {
        participant_id,
        participant_digest,
        participant_kind: participant.kind(),
        implemented_apis,
        required_apis,
        optional_apis,
        needs,
        proposal,
    })
}

fn resolve_used_group(
    participant_id: &str,
    requirement: &str,
    references: &Map<String, Value>,
    resolved: &BTreeMap<String, (&ApiArtifact, String)>,
) -> Result<Vec<ResolvedUsedApi>, ProtocolError> {
    let mut used = references
        .iter()
        .map(|(alias, selection)| {
            let (api, digest) = &resolved[alias];
            let path = pointer(["uses", requirement, alias]);
            let permissions = derive_permissions(participant_id, alias, &path, selection, api)?;
            Ok(ResolvedUsedApi {
                alias: alias.clone(),
                api: api.id().to_owned(),
                api_digest: digest.clone(),
                grant_set: GrantSet::new(permissions),
            })
        })
        .collect::<Result<Vec<_>, ProtocolError>>()?;
    used.sort_by(|left, right| {
        compare_protocol_strings(&left.api, &right.api)
            .then_with(|| compare_protocol_strings(&left.alias, &right.alias))
    });
    Ok(used)
}

fn derive_permissions(
    participant_id: &str,
    alias: &str,
    path: &PointerBuf,
    selection: &Value,
    api: &ApiArtifact,
) -> Result<Vec<PermissionAtom>, ProtocolError> {
    let api_value = api.normalized_value()?;
    let mut permissions = Vec::new();
    for (selection_path, api_section, surface, action) in [
        (
            &["rpc", "call"][..],
            "rpc",
            ApiSurfaceKind::Rpc,
            PermissionAction::Call,
        ),
        (
            &["operations", "invoke"][..],
            "operations",
            ApiSurfaceKind::Operation,
            PermissionAction::Invoke,
        ),
        (
            &["operations", "observe"][..],
            "operations",
            ApiSurfaceKind::Operation,
            PermissionAction::Observe,
        ),
        (
            &["operations", "cancel"][..],
            "operations",
            ApiSurfaceKind::Operation,
            PermissionAction::Cancel,
        ),
        (
            &["events", "publish"][..],
            "events",
            ApiSurfaceKind::Event,
            PermissionAction::Publish,
        ),
        (
            &["events", "subscribe"][..],
            "events",
            ApiSurfaceKind::Event,
            PermissionAction::Subscribe,
        ),
        (
            &["feeds", "subscribe"][..],
            "feeds",
            ApiSurfaceKind::Feed,
            PermissionAction::Subscribe,
        ),
        (
            &["state", "read"][..],
            "state",
            ApiSurfaceKind::State,
            PermissionAction::Read,
        ),
        (
            &["state", "write"][..],
            "state",
            ApiSurfaceKind::State,
            PermissionAction::Write,
        ),
    ] {
        for (index, name) in nested_array_strings(selection, selection_path).enumerate() {
            let item_path =
                join_tokens(path, selection_path.iter().copied()).with_trailing_token(index);
            let definition = object_at(&api_value, api_section)
                .get(name)
                .ok_or_else(|| {
                    resolution_error(
                        ResolutionErrorCode::MissingSurface,
                        participant_id,
                        Some(alias),
                        Some(api.id()),
                        item_path.clone(),
                        format!("selected {api_section} surface '{name}' does not exist"),
                    )
                })?;
            if action == PermissionAction::Cancel
                && definition.get("cancel").and_then(Value::as_bool) != Some(true)
            {
                return Err(resolution_error(
                    ResolutionErrorCode::InvalidCancelSelection,
                    participant_id,
                    Some(alias),
                    Some(api.id()),
                    item_path,
                    format!("operation '{name}' is not cancelable"),
                ));
            }
            permissions.push(PermissionAtom::new(
                PermissionTarget::api_surface(api.id(), surface, name)?,
                action,
            )?);
        }
    }

    for (operation, signals) in nested_object_at(selection, &["operations", "control"]) {
        let operation_definition = object_at(&api_value, "operations")
            .get(operation)
            .ok_or_else(|| {
                resolution_error(
                    ResolutionErrorCode::MissingSurface,
                    participant_id,
                    Some(alias),
                    Some(api.id()),
                    join_tokens(path, ["operations", "control", operation]),
                    format!("selected operation '{operation}' does not exist"),
                )
            })?;
        let available_signals = object_at(operation_definition, "signals");
        for (index, signal) in array_strings(signals).enumerate() {
            if !available_signals.contains_key(signal) {
                return Err(resolution_error(
                    ResolutionErrorCode::MissingOperationSignal,
                    participant_id,
                    Some(alias),
                    Some(api.id()),
                    join_tokens(path, ["operations", "control", operation])
                        .with_trailing_token(index),
                    format!("signal '{signal}' does not exist on operation '{operation}'"),
                ));
            }
            permissions.push(PermissionAtom::new(
                PermissionTarget::operation_signal(api.id(), operation, signal)?,
                PermissionAction::Control,
            )?);
        }
    }
    Ok(permissions)
}

fn derive_provided_api(
    api: &ApiArtifact,
    digest: &str,
) -> Result<ResolvedProvidedApi, ProtocolError> {
    let value = api.normalized_value()?;
    let subjects = api.derived_subjects()?;
    let operations = object_at(&value, "operations")
        .iter()
        .map(|(name, definition)| {
            let mut signals = object_at(definition, "signals")
                .keys()
                .cloned()
                .collect::<Vec<_>>();
            signals.sort_by(|left, right| compare_protocol_strings(left, right));
            Ok((
                name.clone(),
                ResolvedProvidedOperation {
                    subject: subjects.operations[name].clone(),
                    signals,
                },
            ))
        })
        .collect::<Result<_, ProtocolError>>()?;
    let mut state = object_at(&value, "state")
        .keys()
        .cloned()
        .collect::<Vec<_>>();
    state.sort_by(|left, right| compare_protocol_strings(left, right));
    Ok(ResolvedProvidedApi {
        api: api.id().to_owned(),
        api_digest: digest.to_owned(),
        rpc: subjects.rpc,
        operations,
        events: subjects.events,
        feeds: subjects.feeds,
        state,
    })
}

fn derive_required_resources(
    participant_id: &str,
    participant: &Value,
    schemas: &Map<String, Value>,
    resolved: &BTreeMap<String, (&ApiArtifact, String)>,
) -> Result<ParticipantResourceNeeds, ProtocolError> {
    let state = resolve_schema_fields(object_at(participant, "state"), schemas, &["schema"]);
    let job_queues = resolve_schema_fields(
        object_at(participant, "jobQueues"),
        schemas,
        &["payload", "update", "result"],
    );
    let mut event_consumers = BTreeMap::new();
    for (name, consumer) in object_at(participant, "eventConsumers") {
        let mut value = without_human_fields(consumer.clone());
        let events = object_at(&value, "events")
            .iter()
            .map(|(alias, names)| {
                let (api, digest) = &resolved[alias];
                if nested_object_at(participant, &["uses", "optional"]).contains_key(alias) {
                    return Err(resolution_error(
                        ResolutionErrorCode::RequiredConsumerUsesOptionalApi,
                        participant_id,
                        Some(alias),
                        Some(api.id()),
                        pointer(["eventConsumers", name, "events", alias]),
                        "required event consumers cannot depend on optional API authority"
                            .to_owned(),
                    ));
                }
                let api_value = api.normalized_value()?;
                let available_events = object_at(&api_value, "events");
                let mut normalized_names =
                    array_strings(names).map(str::to_owned).collect::<Vec<_>>();
                normalized_names.sort_by(|left, right| compare_protocol_strings(left, right));
                for (index, event) in normalized_names.iter().enumerate() {
                    if !available_events.contains_key(event) {
                        return Err(resolution_error(
                            ResolutionErrorCode::MissingSurface,
                            participant_id,
                            Some(alias),
                            Some(api.id()),
                            pointer(["eventConsumers", name, "events", alias])
                                .with_trailing_token(index),
                            format!("event consumer selects missing event '{event}'"),
                        ));
                    }
                }
                Ok(serde_json::json!({
                    "api": api.id(),
                    "apiDigest": digest,
                    "names": normalized_names,
                }))
            })
            .collect::<Result<Vec<_>, ProtocolError>>()?;
        let mut events = events;
        events.sort_by(|left, right| {
            compare_protocol_strings(string_at(left, "api"), string_at(right, "api"))
        });
        value
            .as_object_mut()
            .expect("normalized consumer is an object")
            .insert("events".to_owned(), Value::Array(events));
        event_consumers.insert(name.clone(), value);
    }
    let (kv, _) = split_resources(
        object_at(nested_value(participant, &["resources"]), "kv"),
        schemas,
        true,
    );
    let (stores, _) = split_resources(
        object_at(nested_value(participant, &["resources"]), "store"),
        schemas,
        false,
    );
    Ok(ParticipantResourceNeeds {
        state,
        job_queues,
        event_consumers,
        kv,
        stores,
        operation_transfers: BTreeMap::new(),
    })
}

fn resource_permissions(
    participant_id: &str,
    resources: &ParticipantResourceNeeds,
) -> Result<Vec<PermissionAtom>, ProtocolError> {
    let mut permissions = Vec::new();
    for (kind, names, actions) in [
        (
            ParticipantResourceKind::State,
            resources.state.keys(),
            &[
                PermissionAction::Read,
                PermissionAction::Write,
                PermissionAction::Delete,
            ][..],
        ),
        (
            ParticipantResourceKind::Kv,
            resources.kv.keys(),
            &[
                PermissionAction::Read,
                PermissionAction::Write,
                PermissionAction::Delete,
            ][..],
        ),
        (
            ParticipantResourceKind::Store,
            resources.stores.keys(),
            &[
                PermissionAction::Read,
                PermissionAction::Write,
                PermissionAction::Delete,
            ][..],
        ),
        (
            ParticipantResourceKind::JobQueue,
            resources.job_queues.keys(),
            &[PermissionAction::Process][..],
        ),
        (
            ParticipantResourceKind::EventConsumer,
            resources.event_consumers.keys(),
            &[PermissionAction::Consume][..],
        ),
    ] {
        for name in names {
            let target = PermissionTarget::participant_resource(participant_id, kind, name)?;
            for action in actions {
                permissions.push(PermissionAtom::new(target.clone(), *action)?);
            }
        }
    }
    Ok(permissions)
}

fn derive_optional_resources(
    participant: &Value,
    schemas: &Map<String, Value>,
) -> Result<ParticipantResourceNeeds, ProtocolError> {
    let (_, kv) = split_resources(
        object_at(nested_value(participant, &["resources"]), "kv"),
        schemas,
        true,
    );
    let (_, stores) = split_resources(
        object_at(nested_value(participant, &["resources"]), "store"),
        schemas,
        false,
    );
    Ok(ParticipantResourceNeeds {
        kv,
        stores,
        ..ParticipantResourceNeeds::default()
    })
}

fn split_resources(
    resources: &Map<String, Value>,
    schemas: &Map<String, Value>,
    schema_backed: bool,
) -> (BTreeMap<String, Value>, BTreeMap<String, Value>) {
    let mut required = BTreeMap::new();
    let mut optional = BTreeMap::new();
    for (name, definition) in resources {
        let is_required = definition.get("required").and_then(Value::as_bool) != Some(false);
        let mut value = without_human_fields(definition.clone());
        value
            .as_object_mut()
            .expect("normalized resource is an object")
            .remove("required");
        if schema_backed {
            resolve_schema_field(&mut value, "schema", schemas);
        }
        if is_required {
            required.insert(name.clone(), value);
        } else {
            optional.insert(name.clone(), value);
        }
    }
    (required, optional)
}

fn resolve_schema_fields(
    definitions: &Map<String, Value>,
    schemas: &Map<String, Value>,
    fields: &[&str],
) -> BTreeMap<String, Value> {
    definitions
        .iter()
        .map(|(name, definition)| {
            let mut value = without_human_fields(definition.clone());
            for field in fields {
                resolve_schema_field(&mut value, field, schemas);
            }
            if let Some(accepted) = value
                .get_mut("acceptedVersions")
                .and_then(Value::as_object_mut)
            {
                for schema in accepted.values_mut() {
                    *schema = schemas[string_at(schema, "schema")].clone();
                }
            }
            (name.clone(), value)
        })
        .collect()
}

fn resolve_schema_field(value: &mut Value, field: &str, schemas: &Map<String, Value>) {
    let Some(reference) = value.get_mut(field) else {
        return;
    };
    *reference = schemas[string_at(reference, "schema")].clone();
}

fn validate_transfers(
    participant_id: &str,
    alias: &str,
    implementation: &Value,
    api: &ApiArtifact,
    participant: &Value,
    needs: &mut BTreeMap<String, Value>,
) -> Result<(), ProtocolError> {
    let api_value = api.normalized_value()?;
    let operations = object_at(&api_value, "operations");
    let mappings = object_at(implementation, "operationTransfers");
    let base = pointer(["implements", alias, "operationTransfers"]);
    for (operation, definition) in operations {
        let sends = definition
            .get("transfer")
            .and_then(|transfer| transfer.get("direction"))
            .and_then(Value::as_str)
            == Some("send");
        if sends && !mappings.contains_key(operation) {
            return Err(resolution_error(
                ResolutionErrorCode::MissingRequiredTransfer,
                participant_id,
                Some(alias),
                Some(api.id()),
                base.clone(),
                format!("send-transfer operation '{operation}' requires exactly one mapping"),
            ));
        }
    }
    for (operation, mapping) in mappings {
        let path = base.with_trailing_token(operation.as_str());
        let Some(definition) = operations.get(operation) else {
            return Err(resolution_error(
                ResolutionErrorCode::InvalidImplementedTransfer,
                participant_id,
                Some(alias),
                Some(api.id()),
                path,
                format!("mapped operation '{operation}' does not exist"),
            ));
        };
        if definition
            .get("transfer")
            .and_then(|transfer| transfer.get("direction"))
            .and_then(Value::as_str)
            != Some("send")
        {
            return Err(resolution_error(
                ResolutionErrorCode::InvalidImplementedTransfer,
                participant_id,
                Some(alias),
                Some(api.id()),
                path,
                format!("operation '{operation}' does not declare send transfer"),
            ));
        }
        let store = string_at(mapping, "store");
        let stores = object_at(nested_value(participant, &["resources"]), "store");
        let Some(store_definition) = stores.get(store) else {
            return Err(resolution_error(
                ResolutionErrorCode::InvalidImplementedTransfer,
                participant_id,
                Some(alias),
                Some(api.id()),
                path.with_trailing_token("store"),
                format!("local store '{store}' does not exist"),
            ));
        };
        if store_definition.get("required").and_then(Value::as_bool) == Some(false) {
            return Err(resolution_error(
                ResolutionErrorCode::OptionalStoreForRequiredTransfer,
                participant_id,
                Some(alias),
                Some(api.id()),
                path.with_trailing_token("store"),
                format!("required provider transfer cannot use optional store '{store}'"),
            ));
        }
        let input_name = string_at(definition.get("input").expect("operation input"), "schema");
        let root = &object_at(&api_value, "schemas")[input_name];
        validate_typed_pointer(
            PointerValidationContext {
                participant_id,
                alias,
                api: api.id(),
                authored_path: &path.with_trailing_token("key"),
            },
            string_at(mapping, "key"),
            root,
            ExpectedSchemaType::String,
            false,
        )?;
        for (field, expected) in [
            ("contentType", ExpectedSchemaType::String),
            ("metadata", ExpectedSchemaType::Object),
        ] {
            if let Some(pointer) = mapping.get(field).and_then(Value::as_str) {
                validate_typed_pointer(
                    PointerValidationContext {
                        participant_id,
                        alias,
                        api: api.id(),
                        authored_path: &path.with_trailing_token(field),
                    },
                    pointer,
                    root,
                    expected,
                    true,
                )?;
            }
        }
        let mut need = mapping.clone();
        need.as_object_mut()
            .expect("normalized transfer is an object")
            .insert("api".to_owned(), Value::String(api.id().to_owned()));
        need.as_object_mut()
            .expect("normalized transfer is an object")
            .insert("apiDigest".to_owned(), Value::String(api.digest()?));
        needs.insert(format!("{}:{operation}", api.id()), need);
    }
    Ok(())
}

fn validate_api_schema_pointers(
    participant_id: &str,
    alias: &str,
    api: &ApiArtifact,
) -> Result<(), ProtocolError> {
    let value = api.normalized_value()?;
    let schemas = object_at(&value, "schemas");
    for (event_name, event) in object_at(&value, "events") {
        let schema_name = string_at(event.get("event").expect("event schema"), "schema");
        for (index, pointer_value) in
            array_strings(event.get("params").unwrap_or(&Value::Null)).enumerate()
        {
            validate_typed_pointer(
                PointerValidationContext {
                    participant_id,
                    alias,
                    api: api.id(),
                    authored_path: &pointer(["events", event_name, "params"])
                        .with_trailing_token(index),
                },
                pointer_value,
                &schemas[schema_name],
                ExpectedSchemaType::SubjectToken,
                false,
            )?;
        }
    }
    Ok(())
}

fn validate_job_pointers(
    participant_id: &str,
    participant: &Value,
    schemas: &Map<String, Value>,
) -> Result<(), ProtocolError> {
    for (queue_name, queue) in object_at(participant, "jobQueues") {
        let Some(keyed) = queue.get("keyConcurrency") else {
            continue;
        };
        let payload_name = string_at(queue.get("payload").expect("queue payload"), "schema");
        for (index, pointer_value) in
            array_strings(keyed.get("key").expect("key pointers")).enumerate()
        {
            if !pointer_value.starts_with('/') {
                continue;
            }
            validate_typed_pointer(
                PointerValidationContext {
                    participant_id,
                    alias: "",
                    api: "",
                    authored_path: &pointer(["jobQueues", queue_name, "keyConcurrency", "key"])
                        .with_trailing_token(index),
                },
                pointer_value,
                &schemas[payload_name],
                ExpectedSchemaType::JobKey,
                false,
            )?;
        }
    }
    Ok(())
}

#[derive(Clone, Copy)]
enum ExpectedSchemaType {
    String,
    Object,
    Array,
    SubjectToken,
    JobKey,
}

struct PointerValidationContext<'a> {
    participant_id: &'a str,
    alias: &'a str,
    api: &'a str,
    authored_path: &'a PointerBuf,
}

fn validate_typed_pointer(
    context: PointerValidationContext<'_>,
    pointer_value: &str,
    schema: &Value,
    expected: ExpectedSchemaType,
    allow_optional: bool,
) -> Result<(), ProtocolError> {
    let parsed =
        Pointer::parse(pointer_value).expect("artifact pointers were parsed during validation");
    let tokens = parsed.tokens().collect::<Vec<_>>();
    let resolved = resolve_schema_pointer(
        schema,
        schema,
        &tokens,
        &mut BTreeSet::new(),
        allow_optional,
    )
    .ok_or_else(|| {
        resolution_error(
            ResolutionErrorCode::UnresolvableSchemaPointer,
            context.participant_id,
            (!context.alias.is_empty()).then_some(context.alias),
            (!context.api.is_empty()).then_some(context.api),
            context.authored_path.clone(),
            format!("schema pointer '{pointer_value}' cannot be proven to resolve"),
        )
    })?;
    if resolved
        .iter()
        .all(|node| schema_proves_type(schema, node, expected, &mut BTreeSet::new()))
    {
        Ok(())
    } else {
        Err(resolution_error(
            ResolutionErrorCode::SchemaPointerTypeMismatch,
            context.participant_id,
            (!context.alias.is_empty()).then_some(context.alias),
            (!context.api.is_empty()).then_some(context.api),
            context.authored_path.clone(),
            format!("schema pointer '{pointer_value}' does not prove the required value type"),
        ))
    }
}

fn resolve_schema_pointer<'a>(
    root: &'a Value,
    schema: &'a Value,
    tokens: &[Token<'_>],
    refs: &mut BTreeSet<String>,
    allow_optional: bool,
) -> Option<Vec<&'a Value>> {
    if let Some(reference) = schema.get("$ref").and_then(Value::as_str) {
        if !refs.insert(reference.to_owned()) {
            return None;
        }
        return resolve_local_schema(root, reference).and_then(|(resolved, _)| {
            resolve_schema_pointer(root, resolved, tokens, refs, allow_optional)
        });
    }
    let Some((token, remaining)) = tokens.split_first() else {
        return Some(vec![schema]);
    };
    for keyword in ["anyOf", "oneOf"] {
        if let Some(branches) = schema.get(keyword).and_then(Value::as_array) {
            let mut resolved = Vec::new();
            for branch in branches {
                resolved.extend(resolve_schema_pointer(
                    root,
                    branch,
                    tokens,
                    &mut refs.clone(),
                    allow_optional,
                )?);
            }
            return Some(resolved);
        }
    }
    if let Some(branches) = schema.get("allOf").and_then(Value::as_array) {
        let resolved = branches
            .iter()
            .filter_map(|branch| {
                resolve_schema_pointer(root, branch, tokens, &mut refs.clone(), allow_optional)
            })
            .flatten()
            .collect::<Vec<_>>();
        if !resolved.is_empty() {
            return Some(resolved);
        }
    }
    let required = schema.get("required").and_then(Value::as_array);
    let decoded = token.decoded();
    if schema_proves_type(
        root,
        schema,
        ExpectedSchemaType::Object,
        &mut BTreeSet::new(),
    ) && (allow_optional
        || required
            .is_some_and(|required| required.iter().any(|name| name.as_str() == Some(&decoded))))
    {
        let property = schema
            .get("properties")
            .and_then(Value::as_object)
            .and_then(|properties| properties.get(decoded.as_ref()))?;
        return resolve_schema_pointer(root, property, remaining, refs, allow_optional);
    }
    if !schema_proves_type(
        root,
        schema,
        ExpectedSchemaType::Array,
        &mut BTreeSet::new(),
    ) {
        return None;
    }
    let Index::Num(index) = token.to_index().ok()? else {
        return None;
    };
    if !allow_optional {
        let min_items = schema.get("minItems").and_then(Value::as_u64)?;
        if u64::try_from(index).ok()?.checked_add(1)? > min_items {
            return None;
        }
    }
    if let Some(item) = schema
        .get("prefixItems")
        .and_then(Value::as_array)
        .and_then(|items| items.get(index))
        .or_else(|| schema.get("items"))
    {
        return resolve_schema_pointer(root, item, remaining, refs, allow_optional);
    }
    None
}

fn schema_proves_type(
    root: &Value,
    schema: &Value,
    expected: ExpectedSchemaType,
    refs: &mut BTreeSet<String>,
) -> bool {
    if let Some(reference) = schema.get("$ref").and_then(Value::as_str) {
        return refs.insert(reference.to_owned())
            && resolve_local_schema(root, reference)
                .is_some_and(|(resolved, _)| schema_proves_type(root, resolved, expected, refs));
    }
    for keyword in ["anyOf", "oneOf"] {
        if let Some(branches) = schema.get(keyword).and_then(Value::as_array) {
            return !branches.is_empty()
                && branches
                    .iter()
                    .all(|branch| schema_proves_type(root, branch, expected, &mut refs.clone()));
        }
    }
    if let Some(branches) = schema.get("allOf").and_then(Value::as_array) {
        return branches
            .iter()
            .any(|branch| schema_proves_type(root, branch, expected, &mut refs.clone()));
    }
    if let Some(value) = schema.get("const") {
        return literal_proves_type(value, expected);
    }
    if let Some(values) = schema.get("enum").and_then(Value::as_array) {
        return !values.is_empty()
            && values
                .iter()
                .all(|value| literal_proves_type(value, expected));
    }
    let types = match schema.get("type") {
        Some(Value::String(value)) => vec![value.as_str()],
        Some(Value::Array(values)) => values.iter().filter_map(Value::as_str).collect(),
        _ => return false,
    };
    types.iter().all(|actual| match expected {
        ExpectedSchemaType::String => *actual == "string",
        ExpectedSchemaType::Object => *actual == "object",
        ExpectedSchemaType::Array => *actual == "array",
        ExpectedSchemaType::SubjectToken => matches!(*actual, "string" | "number" | "integer"),
        ExpectedSchemaType::JobKey => {
            matches!(*actual, "string" | "number" | "integer" | "boolean")
        }
    })
}

fn derive_proposal_section(
    grant_set: &GrantSet,
    resources: ParticipantResourceNeeds,
    used: &[ResolvedUsedApi],
    provided: &[ResolvedImplementedApi],
    resolved: &BTreeMap<String, (&ApiArtifact, String)>,
) -> Result<AuthorityProposalSection, ProtocolError> {
    let requested = grant_set.permissions().to_vec();
    let requested_set = requested.clone();
    let mut capabilities = Vec::new();
    let mut covered = Vec::new();
    for alias in used
        .iter()
        .map(|used| &used.alias)
        .chain(provided.iter().map(|provided| &provided.alias))
    {
        let (api, api_digest) = &resolved[alias];
        let value = api.normalized_value()?;
        for (name, capability) in object_at(&value, "capabilities") {
            let allows = serde_json::from_value::<Vec<PermissionAtom>>(
                capability
                    .get("allows")
                    .cloned()
                    .unwrap_or_else(|| Value::Array(Vec::new())),
            )?;
            if !allows.is_empty() && allows.iter().all(|atom| requested_set.contains(atom)) {
                covered.extend(allows.iter().cloned());
                let consent = object_at(&value, "consent")
                    .get(name)
                    .cloned()
                    .map(serde_json::from_value)
                    .transpose()?;
                capabilities.push(AuthorityCapabilityEvidence {
                    api: api.id().to_owned(),
                    api_digest: api_digest.clone(),
                    name: name.clone(),
                    allows,
                    consent,
                });
            }
        }
    }
    let uncovered_permissions = requested
        .into_iter()
        .filter(|atom| !covered.contains(atom))
        .collect();
    capabilities.sort_by(|left, right| {
        compare_protocol_strings(&left.api, &right.api)
            .then_with(|| compare_protocol_strings(&left.name, &right.name))
    });
    Ok(AuthorityProposalSection {
        grant_set: grant_set.clone(),
        capabilities,
        uncovered_permissions,
        resources,
    })
}

fn literal_proves_type(value: &Value, expected: ExpectedSchemaType) -> bool {
    match expected {
        ExpectedSchemaType::String => value.is_string(),
        ExpectedSchemaType::Object => value.is_object(),
        ExpectedSchemaType::Array => value.is_array(),
        ExpectedSchemaType::SubjectToken => value.is_string() || value.is_number(),
        ExpectedSchemaType::JobKey => value.is_string() || value.is_number() || value.is_boolean(),
    }
}

fn api_needs(used: &[ResolvedUsedApi]) -> Vec<ProvidedApiNeed> {
    let mut needs = used
        .iter()
        .map(|used| ProvidedApiNeed {
            api: used.api.clone(),
            api_digest: used.api_digest.clone(),
        })
        .collect::<Vec<_>>();
    needs.sort_by(|left, right| compare_protocol_strings(&left.api, &right.api));
    needs
}

fn without_human_fields(mut value: Value) -> Value {
    if let Some(object) = value.as_object_mut() {
        object.remove("purpose");
        object.remove("docs");
    }
    value
}

fn resolution_error(
    code: ResolutionErrorCode,
    participant: &str,
    alias: Option<&str>,
    api: Option<&str>,
    path: PointerBuf,
    message: String,
) -> ProtocolError {
    ProtocolError::ParticipantResolution {
        code,
        participant: participant.to_owned(),
        alias: alias.map(str::to_owned),
        api: api.map(str::to_owned),
        path: Box::new(path),
        message,
    }
}

fn object_at<'a>(value: &'a Value, key: &str) -> &'a Map<String, Value> {
    match value.get(key).and_then(Value::as_object) {
        Some(object) => object,
        None => empty_object(),
    }
}

fn nested_object_at<'a>(value: &'a Value, keys: &[&str]) -> &'a Map<String, Value> {
    match nested_value(value, keys).as_object() {
        Some(object) => object,
        None => empty_object(),
    }
}

fn nested_value<'a>(mut value: &'a Value, keys: &[&str]) -> &'a Value {
    for key in keys {
        let Some(next) = value.get(*key) else {
            return &Value::Null;
        };
        value = next;
    }
    value
}

fn empty_object() -> &'static Map<String, Value> {
    static EMPTY: std::sync::OnceLock<Map<String, Value>> = std::sync::OnceLock::new();
    EMPTY.get_or_init(Map::new)
}

fn string_at<'a>(value: &'a Value, key: &str) -> &'a str {
    value
        .get(key)
        .and_then(Value::as_str)
        .expect("validated normalized artifact string")
}

fn array_strings(value: &Value) -> impl Iterator<Item = &str> {
    value
        .as_array()
        .into_iter()
        .flatten()
        .map(|value| value.as_str().expect("validated string selection"))
}

fn nested_array_strings<'a>(value: &'a Value, keys: &[&str]) -> impl Iterator<Item = &'a str> {
    array_strings(nested_value(value, keys))
}

fn pointer<'a>(tokens: impl IntoIterator<Item = &'a str>) -> PointerBuf {
    PointerBuf::from_tokens(tokens)
}

fn join_tokens<'a>(path: &PointerBuf, tokens: impl IntoIterator<Item = &'a str>) -> PointerBuf {
    tokens
        .into_iter()
        .fold(path.clone(), |path, token| path.with_trailing_token(token))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{parse_api, parse_participant};

    #[test]
    fn consent_wording_changes_evidence_but_not_needs_or_fingerprint() {
        let mut api_value = fixture_api("consumer@v1");
        let base_api = parse_api(&api_value).unwrap();
        api_value["consent"]["rpcOnly"]["description"] =
            Value::String("Changed review wording.".to_owned());
        let changed_api = parse_api(&api_value).unwrap();
        assert_eq!(base_api.digest().unwrap(), changed_api.digest().unwrap());
        let expected_api_digest = base_api.digest().unwrap();

        let participant = participant_using("consent-app", &base_api);
        let base = resolve_participant(
            &participant,
            &BTreeMap::from([(base_api.id().to_owned(), base_api)]),
        )
        .unwrap();
        let changed = resolve_participant(
            &participant,
            &BTreeMap::from([(changed_api.id().to_owned(), changed_api)]),
        )
        .unwrap();

        assert_eq!(
            base.needs().digest().unwrap(),
            changed.needs().digest().unwrap()
        );
        assert_eq!(
            base.proposal().fingerprint().unwrap(),
            changed.proposal().fingerprint().unwrap()
        );
        assert_ne!(
            base.proposal().normalized_value().unwrap(),
            changed.proposal().normalized_value().unwrap()
        );
        assert_eq!(
            base.proposal().required().capabilities()[0].api_digest(),
            expected_api_digest
        );
    }

    #[test]
    fn exact_api_digest_changes_needs_identity() {
        let mut changed_value = fixture_api("consumer@v1");
        let base_api = parse_api(&changed_value).unwrap();
        changed_value["state"]["Consumer.Extra"] = serde_json::json!({
            "kind": "value",
            "schema": { "schema": "Any" }
        });
        let changed_api = parse_api(&changed_value).unwrap();
        assert_ne!(base_api.digest().unwrap(), changed_api.digest().unwrap());
        let base_api_digest = base_api.digest().unwrap();
        let changed_api_digest = changed_api.digest().unwrap();

        let base_participant = participant_using("digest-app", &base_api);
        let changed_participant = participant_using("digest-app", &changed_api);
        let base = resolve_participant(
            &base_participant,
            &BTreeMap::from([(base_api.id().to_owned(), base_api)]),
        )
        .unwrap();
        let changed = resolve_participant(
            &changed_participant,
            &BTreeMap::from([(changed_api.id().to_owned(), changed_api)]),
        )
        .unwrap();
        assert_ne!(
            base.needs().digest().unwrap(),
            changed.needs().digest().unwrap()
        );
        assert_eq!(
            base.proposal().required().capabilities()[0].api_digest(),
            base_api_digest
        );
        assert_eq!(
            changed.proposal().required().capabilities()[0].api_digest(),
            changed_api_digest
        );
    }

    fn fixture_api(id: &str) -> Value {
        serde_json::json!({
            "format": "trellis.api.v1",
            "id": id,
            "version": "1.0.0",
            "displayName": "Consumer API",
            "description": "Exercises API identity.",
            "schemas": { "Any": true },
            "rpc": {
                "Consumer.Get": {
                    "version": "v1",
                    "input": { "schema": "Any" },
                    "output": { "schema": "Any" }
                }
            },
            "state": {
                "Consumer.Settings": {
                    "kind": "value",
                    "schema": { "schema": "Any" }
                }
            },
            "capabilities": {
                "rpcOnly": {
                    "allows": [{
                        "target": {
                            "kind": "apiSurface",
                            "api": id,
                            "surface": "rpc",
                            "name": "Consumer.Get"
                        },
                        "action": "call"
                    }]
                }
            },
            "consent": {
                "rpcOnly": {
                    "title": "Read consumer records",
                    "description": "Read records through the consumer API.",
                    "consequence": "The participant can read consumer records."
                }
            }
        })
    }

    fn participant_using(id: &str, api: &ApiArtifact) -> ParticipantArtifact {
        parse_participant(&serde_json::json!({
            "format": "trellis.participant.v1",
            "id": id,
            "displayName": "Digest participant",
            "description": "Exercises API identity.",
            "kind": "app",
            "uses": {
                "required": {
                    "api": {
                        "api": api.id(),
                        "apiDigest": api.digest().unwrap(),
                        "rpc": { "call": ["Consumer.Get"] }
                    }
                }
            }
        }))
        .unwrap()
    }
}
