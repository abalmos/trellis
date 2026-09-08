use crate::platform::auth::UserProfileRecord;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;

pub(super) const NOW: i64 = 1_800_000_000_000;

pub(super) fn digest(byte: u8) -> String {
    URL_SAFE_NO_PAD.encode([byte; 32])
}

pub(super) fn profile_for(principal_id: &str) -> UserProfileRecord {
    UserProfileRecord {
        principal_id: principal_id.to_owned(),
        display_name: Some("User".to_owned()),
        email: None,
        image_url: None,
        created_at: NOW,
        updated_at: NOW,
        version: 1,
    }
}
