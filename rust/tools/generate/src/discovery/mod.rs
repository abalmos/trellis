mod metadata;
mod scan;

pub use metadata::{
    discover_contract_kind, discover_contract_metadata, discover_static_typescript_metadata,
    parse_contract_kind,
};
pub use scan::{
    discover_contracts, discover_local_contracts, DiscoveredContractSource, SourceLanguage,
};
