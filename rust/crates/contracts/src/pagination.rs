use serde::{Deserialize, Serialize};

/// Bounded pagination request used by public Trellis list APIs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PageRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(offset), "` contract value.")]
    pub offset: Option<u64>,
    #[doc = concat!("The `", stringify!(limit), "` contract value.")]
    pub limit: u64,
}

/// Bounded pagination response used by public Trellis list APIs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PageResponse<TEntry> {
    #[doc = concat!("The `", stringify!(entries), "` contract value.")]
    pub entries: Vec<TEntry>,
    #[doc = concat!("The `", stringify!(count), "` contract value.")]
    pub count: u64,
    #[doc = concat!("The `", stringify!(offset), "` contract value.")]
    pub offset: u64,
    #[doc = concat!("The `", stringify!(limit), "` contract value.")]
    pub limit: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(next_offset), "` contract value.")]
    pub next_offset: Option<u64>,
}
