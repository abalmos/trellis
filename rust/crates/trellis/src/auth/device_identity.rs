use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use ed25519_dalek::SigningKey;
use hkdf::Hkdf;
use sha2::Sha256;

use super::{DeviceIdentity, TrellisAuthError};

const DEVICE_IDENTITY_HKDF_INFO: &str = "trellis/device-identity/v1";
const DEVICE_ACTIVATION_HKDF_INFO: &str = "trellis/device-activate/v1";

#[doc = concat!("Trellis API operation `", stringify!(derive_device_identity), "`.")]
pub fn derive_device_identity(
    device_root_secret: &[u8],
) -> Result<DeviceIdentity, TrellisAuthError> {
    if device_root_secret.len() != 32 {
        return Err(TrellisAuthError::InvalidArgument(format!(
            "invalid device root secret length: {} (expected 32)",
            device_root_secret.len()
        )));
    }

    let hkdf = Hkdf::<Sha256>::new(Some(&[]), device_root_secret);
    let mut identity_seed = [0u8; 32];
    hkdf.expand(DEVICE_IDENTITY_HKDF_INFO.as_bytes(), &mut identity_seed)
        .map_err(|error| {
            TrellisAuthError::InvalidArgument(format!(
                "failed to derive device identity seed: {error}"
            ))
        })?;
    let mut activation_key = [0u8; 32];
    hkdf.expand(DEVICE_ACTIVATION_HKDF_INFO.as_bytes(), &mut activation_key)
        .map_err(|error| {
            TrellisAuthError::InvalidArgument(format!(
                "failed to derive device activation key: {error}"
            ))
        })?;
    let public_identity_key = URL_SAFE_NO_PAD.encode(
        SigningKey::from_bytes(&identity_seed)
            .verifying_key()
            .to_bytes(),
    );

    Ok(DeviceIdentity {
        identity_seed_base64url: URL_SAFE_NO_PAD.encode(identity_seed),
        public_identity_key,
        activation_key_base64url: URL_SAFE_NO_PAD.encode(activation_key),
    })
}

#[cfg(test)]
mod tests {
    use super::derive_device_identity;

    #[test]
    fn identity_derivation_is_deterministic_and_validates_length() {
        let identity = derive_device_identity(&[7; 32]).expect("first identity");
        assert_eq!(
            identity,
            derive_device_identity(&[7; 32]).expect("second identity")
        );
        assert_eq!(
            identity.identity_seed_base64url,
            "ANrLNfV6eakMEoleiHoPuE9bQL1BkOE4VTDqAU3jvPQ"
        );
        assert_eq!(
            identity.public_identity_key,
            "PJOPafbG8Sq47Ra0sOSYmG2pJQj5FRgPrlwynA5Dq0I"
        );
        assert_eq!(
            identity.activation_key_base64url,
            "z89beQNUvhI08xF7ceiwvCD_kUF_RtBGcvDFsyiErgA"
        );
        assert!(derive_device_identity(&[7; 31]).is_err());
    }
}
