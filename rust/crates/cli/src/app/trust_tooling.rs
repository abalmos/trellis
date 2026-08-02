use std::fs::{self, OpenOptions};
use std::io::Write as _;
use std::path::{Path, PathBuf};

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use ed25519_dalek::SigningKey;
use miette::{miette, IntoDiagnostic as _};
use serde::Serialize;
use serde_json::{Map, Value};
use sha2::{Digest as _, Sha256};
use time::OffsetDateTime;
use trellis_protocol::{
    canonicalize_json, parse_issuer_manifest_v1, sign_issuer_manifest_v1,
    verify_issuer_manifest_v1, AuthorizationIssuerManifestEntryV1, AuthorizationTrustRootV1,
    AuthorizationVerificationPolicyV1, SignedAuthorizationIssuerManifestV1,
    UnsignedAuthorizationIssuerManifestV1, AUTHORIZATION_ISSUER_MANIFEST_FORMAT_V1,
};
use ulid::Ulid;

use crate::cli::{
    InfraTrustCommand, InfraTrustInitArgs, InfraTrustRotateIssuerArgs, InfraTrustSubcommand,
    OutputFormat,
};
use crate::output;

const ROOT_FILE: &str = "authorization-root.json";
const ROOT_SEED_FILE: &str = "authorization-root.seed";
const ISSUER_SEED_FILE: &str = "authorization-issuer.seed";
const MANIFEST_FILE: &str = "authorization-issuer-manifest.json";

pub(super) fn run(format: OutputFormat, command: InfraTrustCommand) -> miette::Result<()> {
    let summary = match command.command {
        InfraTrustSubcommand::Init(args) => initialize(&args)?,
        InfraTrustSubcommand::RotateIssuer(args) => rotate_issuer(&args)?,
    };
    if output::is_json(format) {
        output::print_json(&summary)?;
    } else {
        println!(
            "authorization trust generation {} active issuer {}",
            summary.generation, summary.active_issuer_key_id
        );
        println!("root: {}", summary.root_file.display());
        println!("manifest: {}", summary.manifest_file.display());
        println!("issuer seed: {}", summary.issuer_seed_file.display());
    }
    Ok(())
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct TrustToolSummary {
    authority: String,
    generation: u64,
    root_file: PathBuf,
    root_seed_file: PathBuf,
    manifest_file: PathBuf,
    issuer_seed_file: PathBuf,
    active_issuer_key_id: String,
}

fn initialize(args: &InfraTrustInitArgs) -> miette::Result<TrustToolSummary> {
    validate_lifetime(args.manifest_lifetime_seconds)?;
    fs::create_dir_all(&args.out).into_diagnostic()?;
    if manifest_generation_path(&args.out, 1).exists() {
        return Err(miette!(
            "authorization trust is already initialized; choose a new empty output directory"
        ));
    }
    let root_seed: [u8; 32] = rand::random();
    let issuer_seed: [u8; 32] = rand::random();
    let root_key = SigningKey::from_bytes(&root_seed);
    let issuer_key = SigningKey::from_bytes(&issuer_seed);
    if root_key.verifying_key() == issuer_key.verifying_key() {
        return Err(miette!(
            "authorization root and issuer keys must be distinct"
        ));
    }
    let root = AuthorizationTrustRootV1::new(
        args.authority.clone(),
        URL_SAFE_NO_PAD.encode(root_key.verifying_key().to_bytes()),
    )
    .map_err(|_| miette!("invalid authorization authority or root key"))?;
    let now = OffsetDateTime::now_utc().unix_timestamp();
    let manifest = sign_manifest(
        &root,
        &root_key,
        1,
        now,
        args.manifest_lifetime_seconds,
        vec![AuthorizationIssuerManifestEntryV1 {
            key_id: key_id(&issuer_key),
            public_key: URL_SAFE_NO_PAD.encode(issuer_key.verifying_key().to_bytes()),
        }],
    )?;

    let root_path = args.out.join(ROOT_FILE);
    let root_seed_path = args.out.join(ROOT_SEED_FILE);
    let issuer_seed_path = args.out.join(ISSUER_SEED_FILE);
    let manifest_path = args.out.join(MANIFEST_FILE);
    let generation_path = manifest_generation_path(&args.out, 1);
    write_atomic(
        &root_path,
        root.canonical_json()
            .map_err(|_| miette!("failed to encode authorization root"))?
            .as_bytes(),
        false,
        args.force,
    )?;
    write_atomic(
        &root_seed_path,
        format!("{}\n", URL_SAFE_NO_PAD.encode(root_seed)).as_bytes(),
        true,
        args.force,
    )?;
    write_atomic(
        &issuer_seed_path,
        format!("{}\n", URL_SAFE_NO_PAD.encode(issuer_seed)).as_bytes(),
        true,
        args.force,
    )?;
    write_canonical(&generation_path, &manifest, false)?;
    write_canonical(&manifest_path, &manifest, args.force)?;

    Ok(TrustToolSummary {
        authority: root.authority().to_owned(),
        generation: 1,
        root_file: root_path,
        root_seed_file: root_seed_path,
        manifest_file: manifest_path,
        issuer_seed_file: issuer_seed_path,
        active_issuer_key_id: key_id(&issuer_key),
    })
}

fn rotate_issuer(args: &InfraTrustRotateIssuerArgs) -> miette::Result<TrustToolSummary> {
    validate_lifetime(args.manifest_lifetime_seconds)?;
    let now = OffsetDateTime::now_utc().unix_timestamp();
    let root_value = read_canonical(&args.dir.join(ROOT_FILE))?;
    let root = AuthorizationTrustRootV1::parse(&root_value)
        .map_err(|_| miette!("authorization trust root is invalid"))?;
    let root_key = read_signing_key(&args.dir.join(ROOT_SEED_FILE))?;
    if root
        .verifying_key()
        .map_err(|_| miette!("authorization root key is invalid"))?
        != root_key.verifying_key()
    {
        return Err(miette!(
            "authorization root seed does not match the pinned root"
        ));
    }
    let manifest_value = read_canonical(&args.dir.join(MANIFEST_FILE))?;
    let current_manifest = parse_issuer_manifest_v1(&manifest_value)
        .map_err(|_| miette!("authorization issuer manifest is invalid"))?;
    let policy = verification_policy(now, current_manifest.unsigned.generation)?;
    verify_issuer_manifest_v1(&root, &current_manifest, &policy)
        .map_err(|_| miette!("authorization issuer manifest verification failed"))?;
    let generation = current_manifest
        .unsigned
        .generation
        .checked_add(1)
        .ok_or_else(|| miette!("issuer manifest generation overflow"))?;
    let mut entries = current_manifest.unsigned.issuers.clone();
    let issuer_seed: [u8; 32] = rand::random();
    let issuer_key = SigningKey::from_bytes(&issuer_seed);
    let active_issuer_key_id = key_id(&issuer_key);
    write_atomic(
        &args.dir.join(ISSUER_SEED_FILE),
        format!("{}\n", URL_SAFE_NO_PAD.encode(issuer_seed)).as_bytes(),
        true,
        true,
    )?;
    entries.push(AuthorizationIssuerManifestEntryV1 {
        key_id: active_issuer_key_id.clone(),
        public_key: URL_SAFE_NO_PAD.encode(issuer_key.verifying_key().to_bytes()),
    });
    entries.sort_by(|left, right| left.key_id.cmp(&right.key_id));
    let manifest = sign_manifest(
        &root,
        &root_key,
        generation,
        now,
        args.manifest_lifetime_seconds,
        entries,
    )?;
    let generation_path = manifest_generation_path(&args.dir, generation);
    write_canonical(&generation_path, &manifest, false)?;
    write_canonical(&args.dir.join(MANIFEST_FILE), &manifest, true)?;

    Ok(TrustToolSummary {
        authority: root.authority().to_owned(),
        generation,
        root_file: args.dir.join(ROOT_FILE),
        root_seed_file: args.dir.join(ROOT_SEED_FILE),
        manifest_file: args.dir.join(MANIFEST_FILE),
        issuer_seed_file: args.dir.join(ISSUER_SEED_FILE),
        active_issuer_key_id,
    })
}

fn sign_manifest(
    root: &AuthorizationTrustRootV1,
    root_key: &SigningKey,
    generation: u64,
    now: i64,
    lifetime: i64,
    issuers: Vec<AuthorizationIssuerManifestEntryV1>,
) -> miette::Result<SignedAuthorizationIssuerManifestV1> {
    let expires_at = now
        .checked_add(lifetime)
        .ok_or_else(|| miette!("issuer manifest expiry overflow"))?;
    sign_issuer_manifest_v1(
        UnsignedAuthorizationIssuerManifestV1 {
            format: AUTHORIZATION_ISSUER_MANIFEST_FORMAT_V1.to_owned(),
            authority: root.authority().to_owned(),
            root_key_id: root.key_id().to_owned(),
            generation,
            issued_at: now,
            not_before: now.saturating_sub(300),
            expires_at,
            issuers,
            extensions: Map::new(),
            critical: Vec::new(),
        },
        root_key,
    )
    .map_err(|_| miette!("failed to sign authorization issuer manifest"))
}

fn verification_policy(
    now: i64,
    minimum_generation: u64,
) -> miette::Result<AuthorizationVerificationPolicyV1> {
    AuthorizationVerificationPolicyV1::new(now, 30, 3_600, 16_384, 4_096, 256, minimum_generation)
        .map_err(|_| miette!("failed to construct authorization verification policy"))
}

fn validate_lifetime(manifest: i64) -> miette::Result<()> {
    if manifest <= 0 {
        return Err(miette!("authorization manifest lifetime must be positive"));
    }
    Ok(())
}

fn key_id(key: &SigningKey) -> String {
    URL_SAFE_NO_PAD.encode(Sha256::digest(key.verifying_key().to_bytes()))
}

fn manifest_generation_path(directory: &Path, generation: u64) -> PathBuf {
    directory.join(format!(
        "authorization-issuer-manifest.{generation:020}.json"
    ))
}

fn read_canonical(path: &Path) -> miette::Result<Value> {
    let text = fs::read_to_string(path)
        .into_diagnostic()
        .map_err(|error| miette!("failed to read {}: {error}", path.display()))?;
    let value = serde_json::from_str::<Value>(&text)
        .into_diagnostic()
        .map_err(|error| miette!("invalid JSON in {}: {error}", path.display()))?;
    let canonical = canonicalize_json(&value)
        .map_err(|_| miette!("failed to canonicalize {}", path.display()))?;
    if text.trim() != canonical {
        return Err(miette!(
            "trust artifact is not canonical JSON: {}",
            path.display()
        ));
    }
    Ok(value)
}

fn read_signing_key(path: &Path) -> miette::Result<SigningKey> {
    let text = fs::read_to_string(path)
        .into_diagnostic()
        .map_err(|error| miette!("failed to read signing seed {}: {error}", path.display()))?;
    let encoded = text.trim();
    let mut bytes = URL_SAFE_NO_PAD
        .decode(encoded)
        .map_err(|_| miette!("signing seed file is not canonical base64url"))?;
    if URL_SAFE_NO_PAD.encode(&bytes) != encoded {
        bytes.fill(0);
        return Err(miette!("signing seed file is not canonical base64url"));
    }
    let seed = <[u8; 32]>::try_from(bytes.as_slice()).map_err(|_| {
        bytes.fill(0);
        miette!("signing seed must contain exactly 32 bytes")
    })?;
    bytes.fill(0);
    Ok(SigningKey::from_bytes(&seed))
}

fn write_canonical<T: Serialize>(path: &Path, value: &T, replace: bool) -> miette::Result<()> {
    let json = canonicalize_json(
        &serde_json::to_value(value).map_err(|_| miette!("failed to encode trust artifact"))?,
    )
    .map_err(|_| miette!("failed to canonicalize trust artifact"))?;
    write_atomic(path, format!("{json}\n").as_bytes(), false, replace)
}

fn write_atomic(path: &Path, bytes: &[u8], secret: bool, replace: bool) -> miette::Result<()> {
    if path.exists() && !replace {
        return Err(miette!(
            "refusing to overwrite existing file: {}",
            path.display()
        ));
    }
    let parent = path
        .parent()
        .ok_or_else(|| miette!("output path has no parent: {}", path.display()))?;
    fs::create_dir_all(parent).into_diagnostic()?;
    let temporary = parent.join(format!(".trellis-trust-{}.tmp", Ulid::new()));
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt as _;
        options.mode(if secret { 0o600 } else { 0o644 });
    }
    let mut file = options.open(&temporary).into_diagnostic()?;
    file.write_all(bytes).into_diagnostic()?;
    file.sync_all().into_diagnostic()?;
    drop(file);
    fs::rename(&temporary, path).into_diagnostic()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initialization_and_overlap_rotation_produce_verifiable_artifacts() {
        let directory = tempfile::tempdir().unwrap();
        let initialized = initialize(&InfraTrustInitArgs {
            out: directory.path().to_path_buf(),
            authority: "trellis.test".to_owned(),
            manifest_lifetime_seconds: 43_200,
            force: false,
        })
        .unwrap();
        assert_eq!(initialized.generation, 1);
        assert_ne!(
            fs::read_to_string(&initialized.root_seed_file).unwrap(),
            fs::read_to_string(&initialized.issuer_seed_file).unwrap()
        );
        let root_seed = fs::read_to_string(&initialized.root_seed_file).unwrap();
        assert!(initialize(&InfraTrustInitArgs {
            out: directory.path().to_path_buf(),
            authority: "trellis.replacement".to_owned(),
            manifest_lifetime_seconds: 43_200,
            force: true,
        })
        .is_err());
        assert_eq!(
            fs::read_to_string(&initialized.root_seed_file).unwrap(),
            root_seed
        );

        let rotated = rotate_issuer(&InfraTrustRotateIssuerArgs {
            dir: directory.path().to_path_buf(),
            manifest_lifetime_seconds: 43_200,
            revoke: None,
        })
        .unwrap();
        assert_eq!(rotated.generation, 2);
        assert_ne!(
            initialized.active_issuer_key_id,
            rotated.active_issuer_key_id
        );
        let manifest =
            parse_issuer_manifest_v1(&read_canonical(&rotated.manifest_file).unwrap()).unwrap();
        assert_eq!(manifest.unsigned.issuers.len(), 2);
        assert!(manifest
            .unsigned
            .issuers
            .iter()
            .all(|issuer| !issuer.key_id.is_empty() && !issuer.public_key.is_empty()));
    }
}
