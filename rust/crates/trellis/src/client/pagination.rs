use serde::{Deserialize, Serialize};

/// Bounded pagination request used by public Trellis list APIs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PageRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Zero-based entry offset.
    pub offset: Option<u64>,
    /// Maximum entries requested.
    pub limit: u64,
}

/// Bounded pagination response used by public Trellis list APIs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PageResponse<TEntry> {
    /// Entries in this page.
    pub entries: Vec<TEntry>,
    /// Number of entries in this page.
    pub count: u64,
    /// Zero-based offset of this page.
    pub offset: u64,
    /// Maximum entries requested.
    pub limit: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Offset for the next page, when one exists.
    pub next_offset: Option<u64>,
}
