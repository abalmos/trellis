use serde_json::Value;
use trellis_protocol::{
    lint_api_v1_authoring, lint_participant_v1_authoring, parse_api_v1, parse_participant_v1,
};

const AUTH_API_DIGEST: &str = "nyfMRub9NVIpXgo3CkJZ07_FeMvu599wV3NaXTLrrpQ";
const AUTH_PARTICIPANT_DIGEST: &str = "U9XsOROqKFKS7uDfCuyJv5xuEeDQxCJco3X2u1F1s1A";
const ADMIN_PARTICIPANT_DIGEST: &str = "c99Tmz1QGCWU8XxvGgTR93M9vmtALE9d7W9M8tATYv4";

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
}
