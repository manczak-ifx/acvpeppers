use anyhow::Context;
use serde::Deserialize;
use std::{fs, path::Path};

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub log_level: String,
    pub log_dir: String,
    pub wire_log: bool,
    pub redact: bool,
    pub max_body_chars: usize,

    pub log_vectors: bool,
    pub log_results: bool,
    pub log_uploads: bool,
    pub log_downloads: bool,


    pub base_url: String,
    /// Where credentials come from: "file" (default) or "env".
    pub credential_source: Option<String>,
    /// Path to the PKCS#12 client identity for the file provider (not committed).
    pub client_pkcs12_path: Option<String>,
    /// Path to the base64 TOTP seed for the file provider (not committed).
    pub totp_file_path: Option<String>,
    pub capability_file: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            log_level: "info".into(),
            log_dir: "logs".into(),
            wire_log: false,
            redact: true,
            max_body_chars: 4000,
            log_vectors: true,
            log_results: true,
            log_uploads: true,
            log_downloads: true,
            base_url: "https://demo.acvts.nist.gov/acvp/v1/".into(),
            credential_source: Some("file".into()),
            client_pkcs12_path: Some("secrets/client.p12".into()),
            totp_file_path: Some("secrets/totp.txt".into()),
            capability_file: "src/capabilities.json".into(),
        }
    }
}

pub fn load_config(path: &str) -> anyhow::Result<AppConfig> {
    if !Path::new(path).exists() {
        return Ok(AppConfig::default());
    }
    let s = fs::read_to_string(path)
        .with_context(|| format!("reading config file {}", path))?;
    let cfg: AppConfig = toml::from_str(&s)
        .with_context(|| format!("parsing TOML {}", path))?;
    Ok(cfg)
}