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


    pub base_url : String, 
    pub client_pkcs12_path: String , 
    pub client_pkcs12_password : String , 
    pub totp_file_path: String,
    pub capability_file : String,
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
            base_url : "https://demo.acvts.nist.gov/acvp/v1/".into(),
            client_pkcs12_path : "src/acvp_client/certs/client.p12".into(),
            client_pkcs12_password : "***REMOVED***".into(),
            totp_file_path : "src/acvp_client/certs/totp.txt".into(),
            capability_file : "src/capabilities.json".into(),
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