//! Credential abstraction for the ACVP client.
//!
//! The ACVP server authenticates callers with two factors:
//!
//! 1. a **mutual-TLS client identity** (a PKCS#12 bundle today), and
//! 2. a **time-based one-time password (TOTP)** presented at `/login`.
//!
//! [`CredentialProvider`] hides *where* that material comes from, so the ACVP
//! client never hard-codes file paths or passwords. Two providers ship out of
//! the box:
//!
//! * [`FileCredentialProvider`] — read the identity and TOTP seed from local
//!   files.
//! * [`EnvCredentialProvider`] — read every value from environment variables,
//!   which keeps secrets out of the repository entirely.
//!
//! Additional sources (cloud secret managers, PKCS#11/HSM devices, ...) only
//! need to implement [`CredentialProvider`] and be wired into
//! [`provider_from_config`].

use anyhow::{anyhow, Context, Result};
use base64::engine::general_purpose::STANDARD;
use base64::Engine as _;
use reqwest::Identity;
use std::path::PathBuf;
use std::sync::Arc;
use std::{env, fs};

use crate::acvp_client::hotp::totp;
use crate::logging::AppConfig;

/// ACVP TOTP step, in seconds.
const TOTP_STEP_SECS: u64 = 30;
/// ACVP TOTP length, in digits.
const TOTP_DIGITS: u32 = 8;

/// Environment-variable names understood by [`EnvCredentialProvider`]. The
/// password variable is also honored by the file provider (see
/// [`provider_from_config`]) so it never has to live in a config file.
pub mod env_vars {
    /// Filesystem path to the PKCS#12 client identity.
    pub const PKCS12_PATH: &str = "ACVP_CLIENT_PKCS12_PATH";
    /// Base64-encoded PKCS#12 client identity (alternative to a path).
    pub const PKCS12_BASE64: &str = "ACVP_CLIENT_PKCS12_BASE64";
    /// Password protecting the PKCS#12 identity.
    pub const PKCS12_PASSWORD: &str = "ACVP_CLIENT_PKCS12_PASSWORD";
    /// Base64-encoded TOTP seed value.
    pub const TOTP_SEED: &str = "ACVP_TOTP_SEED";
    /// Filesystem path to a file containing the base64 TOTP seed.
    pub const TOTP_SEED_PATH: &str = "ACVP_TOTP_SEED_PATH";
}

/// Everything the ACVP client needs to authenticate, sourced abstractly.
///
/// Implement this trait to plug in a new credential backend; the ACVP client
/// depends only on the trait, not on any concrete storage mechanism.
pub trait CredentialProvider: Send + Sync {
    /// The mTLS identity presented to the ACVP server during the TLS handshake.
    fn http_identity(&self) -> Result<Identity>;

    /// A freshly generated one-time password for the ACVP `/login` call.
    fn totp_now(&self) -> Result<String>;
}

/// Build a [`reqwest::Identity`] from raw PKCS#12 DER bytes and its password.
fn identity_from_pkcs12(der: &[u8], password: &str) -> Result<Identity> {
    Identity::from_pkcs12_der(der, password)
        .context("failed to build mTLS identity from PKCS#12 material")
}

/// Generate the ACVP TOTP from a base64-encoded seed.
fn totp_from_seed_b64(seed_b64: &str) -> Result<String> {
    let seed = STANDARD
        .decode(seed_b64.trim())
        .map_err(|e| anyhow!("failed to base64-decode TOTP seed: {e}"))?;
    Ok(totp(&seed, TOTP_STEP_SECS, TOTP_DIGITS))
}

/// Reads the mTLS identity and TOTP seed from local files.
///
/// The PKCS#12 password is supplied at construction time — typically sourced
/// from an environment variable rather than a committed config file.
pub struct FileCredentialProvider {
    pkcs12_path: PathBuf,
    pkcs12_password: String,
    totp_seed_path: PathBuf,
}

impl FileCredentialProvider {
    /// Create a file-backed provider from the identity path, its password, and
    /// the TOTP seed path.
    pub fn new(
        pkcs12_path: impl Into<PathBuf>,
        pkcs12_password: impl Into<String>,
        totp_seed_path: impl Into<PathBuf>,
    ) -> Self {
        Self {
            pkcs12_path: pkcs12_path.into(),
            pkcs12_password: pkcs12_password.into(),
            totp_seed_path: totp_seed_path.into(),
        }
    }
}

impl CredentialProvider for FileCredentialProvider {
    fn http_identity(&self) -> Result<Identity> {
        let der = fs::read(&self.pkcs12_path).with_context(|| {
            format!(
                "reading PKCS#12 identity file {}",
                self.pkcs12_path.display()
            )
        })?;
        identity_from_pkcs12(&der, &self.pkcs12_password)
    }

    fn totp_now(&self) -> Result<String> {
        let seed_b64 = fs::read_to_string(&self.totp_seed_path)
            .with_context(|| format!("reading TOTP seed file {}", self.totp_seed_path.display()))?;
        totp_from_seed_b64(&seed_b64)
    }
}

/// Reads all credential material from environment variables, so nothing has to
/// live on disk inside the repository. See [`env_vars`] for the variable names.
pub struct EnvCredentialProvider;

impl CredentialProvider for EnvCredentialProvider {
    fn http_identity(&self) -> Result<Identity> {
        let password = env::var(env_vars::PKCS12_PASSWORD).map_err(|_| {
            anyhow!(
                "{} must be set for the env credential provider",
                env_vars::PKCS12_PASSWORD
            )
        })?;

        let der = if let Ok(b64) = env::var(env_vars::PKCS12_BASE64) {
            STANDARD
                .decode(b64.trim())
                .with_context(|| format!("decoding {}", env_vars::PKCS12_BASE64))?
        } else if let Ok(path) = env::var(env_vars::PKCS12_PATH) {
            fs::read(&path).with_context(|| format!("reading PKCS#12 identity file {path}"))?
        } else {
            return Err(anyhow!(
                "set {} (base64 identity) or {} (identity file path)",
                env_vars::PKCS12_BASE64,
                env_vars::PKCS12_PATH
            ));
        };

        identity_from_pkcs12(&der, &password)
    }

    fn totp_now(&self) -> Result<String> {
        let seed_b64 = if let Ok(seed) = env::var(env_vars::TOTP_SEED) {
            seed
        } else if let Ok(path) = env::var(env_vars::TOTP_SEED_PATH) {
            fs::read_to_string(&path).with_context(|| format!("reading TOTP seed file {path}"))?
        } else {
            return Err(anyhow!(
                "set {} (base64 seed) or {} (seed file path)",
                env_vars::TOTP_SEED,
                env_vars::TOTP_SEED_PATH
            ));
        };
        totp_from_seed_b64(&seed_b64)
    }
}

/// Select and build a [`CredentialProvider`] from application configuration.
///
/// The `credential_source` field picks the provider:
///
/// * `"file"` (the default) — a [`FileCredentialProvider`] using
///   `client_pkcs12_path` and `totp_file_path` from the config. The PKCS#12
///   password is read from the `ACVP_CLIENT_PKCS12_PASSWORD` environment
///   variable and is never stored in the config file.
/// * `"env"` — an [`EnvCredentialProvider`] reading every value from the
///   environment (see [`env_vars`]).
pub fn provider_from_config(cfg: &AppConfig) -> Result<Arc<dyn CredentialProvider>> {
    let source = cfg
        .credential_source
        .as_deref()
        .unwrap_or("file")
        .to_ascii_lowercase();

    match source.as_str() {
        "env" => {
            let provider: Arc<dyn CredentialProvider> = Arc::new(EnvCredentialProvider);
            Ok(provider)
        }
        "file" => {
            let pkcs12_path = cfg.client_pkcs12_path.as_deref().ok_or_else(|| {
                anyhow!("credential_source = \"file\" requires `client_pkcs12_path` in the config")
            })?;
            let totp_seed_path = cfg.totp_file_path.as_deref().ok_or_else(|| {
                anyhow!("credential_source = \"file\" requires `totp_file_path` in the config")
            })?;
            let password = env::var(env_vars::PKCS12_PASSWORD).map_err(|_| {
                anyhow!(
                    "{} must be set (the PKCS#12 password is never stored in the config file)",
                    env_vars::PKCS12_PASSWORD
                )
            })?;

            let provider: Arc<dyn CredentialProvider> = Arc::new(FileCredentialProvider::new(
                pkcs12_path,
                password,
                totp_seed_path,
            ));
            Ok(provider)
        }
        other => Err(anyhow!(
            "unknown credential_source \"{other}\" (expected \"file\" or \"env\")"
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn totp_rejects_invalid_base64_seed() {
        assert!(totp_from_seed_b64("not valid base64 !!!").is_err());
    }

    #[test]
    fn file_provider_errors_on_missing_identity_file() {
        let provider = FileCredentialProvider::new(
            "/nonexistent/acvpeppers/client.p12",
            "unused",
            "/nonexistent/acvpeppers/totp.txt",
        );
        assert!(provider.http_identity().is_err());
    }

    #[test]
    fn provider_from_config_rejects_unknown_source() {
        let cfg = AppConfig {
            credential_source: Some("bogus".to_string()),
            ..AppConfig::default()
        };
        assert!(provider_from_config(&cfg).is_err());
    }
}
