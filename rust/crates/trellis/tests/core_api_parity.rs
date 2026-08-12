//! Canonical core API parity coverage.

#[test]
fn generated_rust_core_api_matches_canonical_json_and_digest() {
    let canonical: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../generated/protocol/apis/trellis.core@v1.json"
    ))
    .expect("canonical core API JSON");
    let embedded: serde_json::Value =
        serde_json::from_str(trellis_rs::sdk::core::api::API_JSON).expect("embedded core API JSON");
    assert_eq!(embedded, canonical);
    assert_eq!(trellis_rs::sdk::core::api::API_ID, "trellis.core@v1");
    assert_eq!(
        trellis_rs::sdk::core::api::API_DIGEST,
        trellis_contracts::ApiBuilder::new(canonical)
            .build()
            .expect("valid core API")
            .digest()
            .expect("core API digest")
    );
}
