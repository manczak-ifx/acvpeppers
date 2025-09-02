// src/main.rs
use anyhow::Result;
use serde_json::json;
use reqwest::Method;
use serde_json::Value;
use std::time::Duration;
use std::thread::sleep;
use std::fs::OpenOptions;
use std::io::Write;
use crate::acvp_client::client::{AcvpClient, Session};
use crate::cryptography::registry::{initialize_crypto_registry};
use crate::parser::TestVectorSet;
use crate::test_executor::execute_test_vector;
use crate::test_types::registry::{initialize_test_type_registry};
mod logging ; 
mod acvp_client;   
mod cryptography;
mod test_types;
mod parser;
mod result_format;
mod test_executor;

fn main() -> Result<()> {
    // 1) Build ACVP HTTP client 
    let mut acvp = AcvpClient::from_pkcs12(
        "https://demo.acvts.nist.gov/acvp/v1/",
        "src/acvp_client/certs/client.p12",
        "***REMOVED***",                        
        "src/acvp_client/certs/totp.txt",   
    )?;

    // 2) Login via TOTP 
    acvp.login_with_totp("src/acvp_client/certs/totp.txt")?;
    println!("Login successful.");

    // 3) Capabilities 
    let algorithms = json!([
        {
            "algorithm": "SHA2-256",
            "revision": "1.0",
            "messageLength": [{ "min": 0, "max": 65535, "increment": 1 }]
        }
    ]);

    // 4) Create test session → returns per-session token
    let session: Session = acvp.test_session_create(&algorithms, false, false)?;
    println!("Created test session: {}", session.id);

    // 5) Wait for vectorSetUrls (because its asynchronous !!!!!!!!!!!!!!)
    let vs_urls = acvp.wait_for_vector_set_urls(session.id, &session.token, 30, 5)?;
    println!("Vector sets ready: {}", vs_urls.len());

    // 6) registries 
    let crypto_registry = initialize_crypto_registry();
    let test_type_registry = initialize_test_type_registry();

    // 7) For each test vector set : download → execute → upload results 
    for url_or_path in vs_urls {
    // Fetch the vector set JSON 
    let vs_json = if is_absolute_url(&url_or_path) {
        acvp.request_abs_with_token(&url_or_path, Method::GET, None, &session.token)?
    } else {
        let rel = normalize_to_relative(acvp.base_url(), &url_or_path);
        acvp.request_with_token(&rel, Method::GET, None, &session.token)?
    };

    // Parsing !
    let tvs: TestVectorSet = serde_json::from_value(vs_json[1].clone())?;

    // saving the downloaded vector set !
    std::fs::write(
        format!("downloaded_vs_{}.json", tvs.vs_id),
        serde_json::to_string_pretty(&vs_json)?
    )?;

    // executing tests
    let result_set = execute_test_vector(&crypto_registry, &test_type_registry, &tvs);

    // Build upload body (this is the result set)
    let results_path = format!("testSessions/{}/vectorSets/{}/results", session.id, tvs.vs_id);
    let mut payload: Value = serde_json::to_value(&result_set)?;
    if let Some(obj) = payload.as_object_mut() {
        // Ask server to echo expected/provided on failures
        obj.insert("showExpected".to_string(), Value::Bool(true));
    }
    let upload_body = json!([
        { "acvVersion": "1.0" },
        payload
    ]);
    // save the result into a file !
std::fs::write(
    format!("result_upload_vs_{}.json", tvs.vs_id),
    serde_json::to_string_pretty(&upload_body)?
)?;

    // uploading results
    acvp.request_with_token(&results_path, Method::POST, Some(&upload_body), &session.token)?;
    println!("Uploaded results for VS {}", tvs.vs_id);

    // Poll per-VS results (wait for validation ) + logging ! 
    let vs_results = poll_vs_results(&acvp, &session, tvs.vs_id, 20, 5)?;
    println!("Vector set {} results:", tvs.vs_id);
    log_vs_results(tvs.vs_id, &vs_results);
}
    //  validation results (summary) + printing
let results = acvp.get_session_results(
    session.id,
    &session.token,
     true,
     20,
   5,
)?;
log_session_summary(session.id, &results);
println!("Session {} results (raw):", session.id);
println!("{}", serde_json::to_string_pretty(&results)?);

println!("Session {} results (summary):", session.id);
if let Some(arr) = results
    .get(1)
    .and_then(|p| p.get("results"))
    .and_then(|r| r.as_array())
{
    for entry in arr {
        let vs_url = entry
            .get("vectorSetUrl")
            .and_then(|v| v.as_str())
            .unwrap_or("<unknown>");
        let disp = entry
            .get("disposition")
            .and_then(|v| v.as_str())
            .unwrap_or("<pending>");
        println!("- {} → {}", vs_url, disp);
    }
} else {
    println!("- <no results yet>");
}

println!("All vectors processed for session {}", session.id);

    Ok(())
}


/// Returns true if the string looks like an absolute URL.
fn is_absolute_url(s: &str) -> bool {
    s.starts_with("http://") || s.starts_with("https://")
}

/// Normalize FULL URLS TO CLIENT PATHS (ex : httppss: // .... to testSession/123/..)
fn normalize_to_relative(base_url: &str, url_or_path: &str) -> String {
    let mut s = url_or_path.strip_prefix(base_url).unwrap_or(url_or_path);
    s = s.trim_start_matches('/');
    if let Some(rest) = s.strip_prefix("acvp/v1/") {
        s = rest;
    }
    s.trim_start_matches('/').to_string()
}
fn poll_vs_results(
    acvp: &AcvpClient,
    session: &Session,
    vs_id: u64,
    max_attempts: u32,
    sleep_secs: u64,
) -> anyhow::Result<serde_json::Value> {
    use std::{thread, time::Duration};
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
            thread::sleep(Duration::from_secs(retry.max(1)));
            continue;
        }
        let ready = payload.get("disposition").is_some()
            || payload.get("results").is_some()
            || payload.get("tests").is_some();

        if ready || attempts >= max_attempts {
            return Ok(resp);
        }

        thread::sleep(Duration::from_secs(sleep_secs.max(1)));
    }
}

fn print_vs_failures(vs_results: &serde_json::Value) {
    let payload = match vs_results.get(1) {
        Some(p) => p,
        None => {
            println!("\t(no payload)");
            return;
        }
    };

    if let Some(disposition) = payload.get("disposition").and_then(|v| v.as_str()) {
        println!("\tdisposition: {}", disposition);
    } else if let Some(results) = payload.get("results").and_then(|v| v.as_array()) {
        for r in results {
            let url = r.get("vectorSetUrl").and_then(|v| v.as_str()).unwrap_or("<url>");
            let disp = r
                .get("disposition").and_then(|v| v.as_str())
                .or_else(|| r.get("status").and_then(|v| v.as_str()))
                .unwrap_or("<pending>");
            println!("\t{} → {}", url, disp);
        }
    } else {
        println!("\t(no disposition in payload)");
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
        None => {
            append_log(&format!("[VS {}] <no payload>", vs_id));
            return;
        }
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
                if let Some(exp) = t.get("expected") {
                    append_log(&format!("  expected: {}", exp)); 
                }
                if let Some(prov) = t.get("provided") {
                    append_log(&format!("  provided: {}", prov));
                }
                if let Some(reason) = t.get("reason") {
                    append_log(&format!("  reason: {}", reason));
                }
            }
        }
        append_log(&format!("[VS {}] failed tests: {}", vs_id, failed));
    }
    if let Some(results) = payload.get("results").and_then(|v| v.as_array()) {
        for r in results {
            let url  = r.get("vectorSetUrl").and_then(|v| v.as_str()).unwrap_or("<url>");
            let disp = r.get("disposition").and_then(|v| v.as_str())
                        .or_else(|| r.get("status").and_then(|v| v.as_str()))
                        .unwrap_or("<pending>");
            append_log(&format!("[VS {}] {} → {}", vs_id, url, disp));
        }
    }
}

fn log_session_summary(session_id: u64, results: &serde_json::Value) {
    append_log(&format!("[SESSION {}] summary:", session_id));
    if let Some(arr) = results
        .get(1)
        .and_then(|p| p.get("results"))
        .and_then(|r| r.as_array())
    {
        for entry in arr {
            let vs_url = entry.get("vectorSetUrl").and_then(|v| v.as_str()).unwrap_or("<unknown>");
            let disp   = entry.get("disposition").and_then(|v| v.as_str()).unwrap_or("<pending>");
            append_log(&format!("  {} → {}", vs_url, disp));
        }
    } else {
        append_log("  <no results yet>");
    }
}