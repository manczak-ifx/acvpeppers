use anyhow::Result;
use clap::Parser;
use log::{error};
mod acvp_client;
mod cryptography;
mod test_types;
mod parser;
mod result_format;
mod test_executor;

mod logging;
mod cli;
mod runner;
mod capabilities;


fn main() {
    if let Err(e) = acvpeppers() {
        error!("fatal error: {:#}", e);
        std::process::exit(1);
    }
}

fn acvpeppers() -> Result<()> {
    let cli = cli::Cli::parse();
    runner::dispatch(cli)
}

use reqwest::Method;
use std::fs::OpenOptions;
use std::io::Write;
use crate::acvp_client::client::{AcvpClient, Session};

fn poll_vs_results(
    acvp: &AcvpClient,
    session: &Session,
    vs_id: u64,
    max_attempts: u32,
    sleep_secs: u64,
) -> anyhow::Result<serde_json::Value> {
    use std::{ time::Duration};
    let path = format!("testSessions/{}/vectorSets/{}/results", session.id, vs_id);
    let mut attempts = 0u32;
    loop {
        attempts += 1;
        let resp = acvp.request_with_token(&path, Method::GET, None, &session.token)?;
        let payload = resp.get(1).ok_or_else(|| anyhow::anyhow!("missing payload[1]"))?;
        if let Some(retry) = payload.get("retry").and_then(|v| v.as_u64()) {
            if attempts >= max_attempts {
                return Ok(resp);
            }
            std::thread::sleep(Duration::from_secs(retry.max(1)));
            continue;
        }
        let ready = payload.get("disposition").is_some()
            || payload.get("results").is_some()
            || payload.get("tests").is_some();
        if ready || attempts >= max_attempts {
            return Ok(resp);
        }
        std::thread::sleep(Duration::from_secs(sleep_secs.max(1)));
    }
}

const LOG_PATH: &str = "acvp_results.log";
fn append_log(line: &str) {
    if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(LOG_PATH) {
        let _ = writeln!(f, "{}", line);
    }
}

fn log_vs_results(vs_id: u64, vs_results: &serde_json::Value) {
    let payload = match vs_results.get(1) {
        Some(p) => p,
        None => { append_log(&format!("[VS {}] <no payload>", vs_id)); return; }
    };
    if let Some(disp) = payload.get("disposition").and_then(|v| v.as_str()) {
        append_log(&format!("[VS {}] disposition: {}", vs_id, disp));
    }
    if let Some(tests) = payload.get("tests").and_then(|v| v.as_array()) {
        let mut failed = 0usize;
        for t in tests {
            let res = t.get("result").and_then(|v| v.as_str()).unwrap_or("<unknown>");
            if res != "passed" {
                failed += 1;
                let tc = t.get("tcId").and_then(|v| v.as_u64()).unwrap_or(0);
                append_log(&format!("[VS {}] tcId {}: {}", vs_id, tc, res));
                if let Some(exp) = t.get("expected") { append_log(&format!(" expected: {}", exp)); }
                if let Some(prov) = t.get("provided") { append_log(&format!(" provided: {}", prov)); }
                if let Some(reason) = t.get("reason") { append_log(&format!(" reason: {}", reason)); }
            }
        }
        append_log(&format!("[VS {}] failed tests: {}", vs_id, failed));
    }
    if let Some(results) = payload.get("results").and_then(|v| v.as_array()) {
        for r in results {
            let url = r.get("vectorSetUrl").and_then(|v| v.as_str()).unwrap_or("<url>");
            let disp = r.get("disposition").and_then(|v| v.as_str())
                .or_else(|| r.get("status").and_then(|v| v.as_str()))
                .unwrap_or("<pending>");
            append_log(&format!("[VS {}] {} → {}", vs_id, url, disp));
        }
    }
}

fn log_session_summary(session_id: u64, results: &serde_json::Value) {
    append_log(&format!("[SESSION {}] summary:", session_id));
    if let Some(arr) = results.get(1).and_then(|p| p.get("results")).and_then(|r| r.as_array()) {
        for entry in arr {
            let vs_url = entry.get("vectorSetUrl").and_then(|v| v.as_str()).unwrap_or("<unknown>");
            let disp = entry.get("disposition").and_then(|v| v.as_str()).unwrap_or("<pending>");
            append_log(&format!(" {} → {}", vs_url, disp));
        }
    } else {
        append_log(" <no results yet>");
    }
}