use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::{Algorithm, Argon2, Params, Version};

use super::AuthorizationStateError;

const HASH_PROFILE: u32 = 1;
const MEMORY_KIB: u32 = 19_456;
const ITERATIONS: u32 = 2;
const PARALLELISM: u32 = 1;
const HASH_BYTES: usize = 32;
const SALT_BYTES: usize = 16;
const DEFAULT_MIN_PASSWORD_LENGTH: usize = 12;
const MIN_PASSWORD_LENGTH: usize = 8;

pub(super) fn normalize_username(value: &str) -> Result<String, AuthorizationStateError> {
    let normalized = value.trim().to_lowercase();
    if normalized.is_empty() {
        return Err(AuthorizationStateError::InvalidRecord(
            "username must not be empty".to_owned(),
        ));
    }
    Ok(normalized)
}

pub(super) fn hash_password(
    password: &str,
    minimum_length: Option<usize>,
) -> Result<(String, u32), AuthorizationStateError> {
    let minimum_length = minimum_length.unwrap_or(DEFAULT_MIN_PASSWORD_LENGTH);
    if minimum_length < MIN_PASSWORD_LENGTH {
        return Err(AuthorizationStateError::InvalidRecord(
            "password minimum length must be at least 8".to_owned(),
        ));
    }
    if password.chars().count() < minimum_length {
        return Err(AuthorizationStateError::InvalidRecord(format!(
            "password must be at least {minimum_length} characters"
        )));
    }
    let mut salt = [0_u8; SALT_BYTES];
    getrandom::fill(&mut salt).map_err(|error| {
        AuthorizationStateError::Storage(format!("password salt generation failed: {error}"))
    })?;
    let salt = SaltString::encode_b64(&salt).map_err(|error| {
        AuthorizationStateError::Storage(format!("password salt encoding failed: {error}"))
    })?;
    let hash = password_hasher()?
        .hash_password(password.as_bytes(), &salt)
        .map_err(|error| {
            AuthorizationStateError::Storage(format!("password hashing failed: {error}"))
        })?
        .to_string();
    Ok((hash, HASH_PROFILE))
}

pub(super) fn verify_password(encoded: &str, password: &str) -> bool {
    let Ok(hash) = PasswordHash::new(encoded) else {
        return false;
    };
    password_hasher().is_ok_and(|hasher| hasher.verify_password(password.as_bytes(), &hash).is_ok())
}

fn password_hasher() -> Result<Argon2<'static>, AuthorizationStateError> {
    let params =
        Params::new(MEMORY_KIB, ITERATIONS, PARALLELISM, Some(HASH_BYTES)).map_err(|error| {
            AuthorizationStateError::Storage(format!("invalid Argon2id profile: {error}"))
        })?;
    Ok(Argon2::new(Algorithm::Argon2id, Version::V0x13, params))
}

#[cfg(test)]
mod tests {
    use super::{hash_password, normalize_username, verify_password};

    #[test]
    fn password_profile_round_trips_and_rejects_wrong_password() {
        let (encoded, profile) = hash_password("correct horse battery staple", None).unwrap();
        assert_eq!(profile, 1);
        assert!(encoded.starts_with("$argon2id$v=19$m=19456,t=2,p=1$"));
        assert!(verify_password(&encoded, "correct horse battery staple"));
        assert!(!verify_password(&encoded, "wrong password"));
    }

    #[test]
    fn username_and_password_policy_are_canonical() {
        assert_eq!(
            normalize_username("  Admin@Example.COM ").unwrap(),
            "admin@example.com"
        );
        assert!(hash_password("short", None).is_err());
        assert!(hash_password("long-enough", Some(7)).is_err());
        assert!(!verify_password("not-a-phc-hash", "anything"));
    }
}
