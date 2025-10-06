use anyhow::{Context, Result};
use serde_json::Value;
use std::fs;
use std::path::Path;

/// Load the capability JSON
pub fn load_capabilities<P: AsRef<Path>>(path: P) -> Result<Value> {
    let text = fs::read_to_string(&path)
        .with_context(|| format!("reading capability file {}", path.as_ref().display()))?;
    let v: Value = serde_json::from_str(&text)
        .with_context(|| format!("parsing JSON in {}", path.as_ref().display()))?;
    Ok(v)
}