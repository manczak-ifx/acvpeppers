use crate::acvp_client::client::{AcvpClient, Session};
use crate::capabilities::load_capabilities;
use crate::cryptography::registry::initialize_crypto_registry;
use crate::logging::{init_logger, load_config, trunc_json, LogSwitches};
use crate::parser::TestVectorSet;
use crate::test_executor::execute_test_vector;

use anyhow::{Context, Result};
use colored::Colorize;
use log::{debug, info};
use reqwest::Method;
use serde_json::{json, Value};
use std::time::Instant;

use crate::cli::{Cli, RunArgs, ConfigCmd, ValidateArgs};

pub fn dispatch(cli: Cli) -> Result<()> {
    // Load app config
    let cfg = load_config(&cli.config.to_string_lossy())
        .with_context(|| format!("failed to load config from {}", cli.config.display()))?;

    // Derive effective log level from CLI verbosity/quiet + cfg
    let mut level = cfg.log_level.clone();
    if cli.verbose >= 2 {
        level = "debug".into();
    } else if cli.verbose == 1 {
        level = "info".into();
    }
      
    match &cli.command {
        crate::cli::Command::Run(args)      => cmd_run(&cli, args.clone(), cfg),
        crate::cli::Command::Config(cmd)    => cmd_config(&cli, cmd.clone(),cfg),
        crate::cli::Command::Validate(args) => cmd_validate(&cli, args.clone()),
    }
}

pub fn cmd_config(_cli: &Cli, cmd: ConfigCmd, cfg: crate::logging::AppConfig) -> Result<()> {
    if cmd.show || cmd.set_base_url.is_none() {
        println!("Config:");
        println!("  base_url              = {}", cfg.base_url);
        println!("  credential_source     = {}", cfg.credential_source.as_deref().unwrap_or("file"));
        println!("  client_pkcs12_path    = {}", cfg.client_pkcs12_path.as_deref().unwrap_or("<unset>"));
        println!("  totp_file_path        = {}", cfg.totp_file_path.as_deref().unwrap_or("<unset>"));
        println!("  capability_file       = {}", cfg.capability_file);
        println!("  log_dir               = {}", cfg.log_dir);
        println!("  log_level             = {}", cfg.log_level);
        println!("  wire_log              = {}", cfg.wire_log);
        println!("  redact                = {}", cfg.redact);
        println!("  log_vectors/results/uploads/downloads = {}/{}/{}/{}",
            cfg.log_vectors, cfg.log_results, cfg.log_uploads, cfg.log_downloads);
        return Ok(());
    }
    if let Some(url) = cmd.set_base_url {
        println!("(Edit your TOML to set base_url = \"{url}\")");
    }
    Ok(())
}

pub fn cmd_run(cli: &Cli, args: RunArgs, cfg: crate::logging::AppConfig) -> Result<()> {
   // Capability path: CLI override or config default
    let cap_path = cli.capability.clone().unwrap_or_else(|| cfg.capability_file.clone());
    let caps_json = load_capabilities(&cap_path)
        .with_context(|| format!("loading capabilities from {}", cap_path))?;

    // Resolve credentials via the configured provider (file or env), then build client.
    let provider = crate::acvp_client::credentials::provider_from_config(&cfg)
        .context("resolving ACVP credentials")?;
    let mut acvp = AcvpClient::from_provider(&cfg.base_url, provider)
        .context("building ACVP client")?;

    acvp.login().context("ACVP login failed")?;
    log::info!("Login successful");

    // Create session with normalized capabilities (same as before)
    let capabilities = caps_json;

    let caps = capabilities.clone();
log::debug!("session create payload (truncated): {}",
    trunc_json(&caps, cfg.max_body_chars, cfg.redact));
    let session: Session = acvp.test_session_create(&capabilities, args.publish, args.immediate)
        .context("creating test session failed")?;

    // Re-init logger with session.id
    init_logger(session.id, &cfg.log_dir, &cfg.log_level)?;
    log::info!("Created test session: {}", session.id);

    let switches = LogSwitches::from(&cfg);
    info!(
        "Config: lvl={}, dir={}, wire={}, redact={}",
        cfg.log_level, cfg.log_dir, cfg.wire_log, cfg.redact
    );

    // Waiting for vectorSetUrls
    let vs_urls = acvp.wait_for_vector_set_urls(session.id, &session.token, 30, 5)
        .context("waiting for vectorSetUrls failed")?;
    info!("Vector sets ready: {}", vs_urls.len());
    debug!("VectorSetsUrls = {:?}", vs_urls);

    // Algorithm registry
    let crypto_registry = initialize_crypto_registry();

    // Process each vector set
    for url_or_path in vs_urls {
        // Fetch VS JSON
        let vs_json = if is_absolute_url(&url_or_path) {
            acvp.request_abs_with_token(&url_or_path, Method::GET, None, &session.token)?
        } else {
            let rel = normalize_to_relative(acvp.base_url(), &url_or_path);
            acvp.request_with_token(&rel, Method::GET, None, &session.token)?
        };
        info!("Fetching vector set: {}", url_or_path);
        debug!("Vector set (truncated):\n{}", trunc_json(&vs_json, 4000, true));

        let tvs: TestVectorSet = serde_json::from_value(vs_json[1].clone())
            .context("parsing TestVectorSet")?;
        info!("Downloaded vector set {}", tvs.vs_id);

        std::fs::write(
            format!("downloaded_vs_{}.json", tvs.vs_id),
            serde_json::to_string_pretty(&vs_json)?,
        )?;
        debug!("Wrote file: downloaded_vs_{}.json", tvs.vs_id);

        // Execute tests
        info!("Executing vector set {}", tvs.vs_id);
        let t0 = Instant::now();
        let result_set = execute_test_vector(&crypto_registry, &tvs);
        let dt = t0.elapsed();
        println!("parallel executor elapsed: {:?}", dt);

        // Build upload body
        let results_path = format!("testSessions/{}/vectorSets/{}/results", session.id, tvs.vs_id);
        let mut payload: Value = serde_json::to_value(&result_set)?;
        if let Some(obj) = payload.as_object_mut() {
            obj.insert("showExpected".to_string(), Value::Bool(true));
        }
        let upload_body = json!([ { "acvVersion": "1.0" }, payload ]);
        debug!("Upload -> {}: {}", results_path, trunc_json(&upload_body, 4000, true));

        // Save and upload
        std::fs::write(
            format!("result_upload_vs_{}.json", tvs.vs_id),
            serde_json::to_string_pretty(&upload_body)?,
        )?;
        debug!("Wrote file: result_upload_vs_{}.json", tvs.vs_id);
        acvp.request_with_token(&results_path, Method::POST, Some(&upload_body), &session.token)
            .context("uploading results failed")?;
        info!("Uploaded results for VS {}", tvs.vs_id);

        // Poll per-VS results
        let vs_results = super::poll_vs_results(&acvp, &session, tvs.vs_id, 20, 5)
            .context("polling VS results failed")?;
        humanize_vs_results(tvs.vs_id, &vs_results, cli.json);

        super::log_vs_results(tvs.vs_id, &vs_results);
    }

    // Session summary
    let results = acvp.get_session_results(session.id, &session.token, true, 20, 5)
        .context("getting session results")?;
    humanize_session_summary(session.id, &results, cli.json);
    super::log_session_summary(session.id, &results);
    info!("All vectors processed for session {}", session.id);
    Ok(())
}

pub fn cmd_validate(cli: &Cli, args: ValidateArgs) -> Result<()> {
    if !cli.json {
        println!("Validating {} …", args.algorithm.bold());
    }
    
    let out_hex = "<computed-hex>";
    let pass = args.expected.as_deref() == Some(out_hex);

    if cli.json {
        let j = json!({
            "algorithm": args.algorithm,
            "input": args.input,
            "output": out_hex,
            "expected": args.expected,
            "result": if pass { "passed" } else { "failed" }
        });
        println!("{}", serde_json::to_string_pretty(&j)?);
    } else {
        println!(" output : {}", out_hex.bold());
        if let Some(exp) = args.expected {
            if pass {
                println!(" result : {}", "PASSED".bold().green());
            } else {
                println!(" result : {}", "FAILED".bold().red());
                println!(" expected: {}", exp);
            }
        }
    }
    Ok(())
}

fn humanize_vs_results(vs_id: u64, vs_results: &serde_json::Value, json_out: bool) {
    if json_out {
        println!("{}", serde_json::to_string_pretty(vs_results).unwrap_or_else(|_| "{}".into()));
        return;
    }
    //  pretty printer
    let payload = vs_results.get(1);
    if let Some(p) = payload {
        if let Some(disp) = p.get("disposition").and_then(|v| v.as_str()) {
            let color = match disp {
                "passed" | "approved" => "PASSED".bold().green(),
                "failed" | "error" => "FAILED".bold().red(),
                _ => disp.bold().yellow(),
            };
            println!("[VS {vs_id}] disposition: {color}");
        }
        if let Some(tests) = p.get("tests").and_then(|v| v.as_array()) {
            for t in tests {
                let tid = t.get("tcId").and_then(|v| v.as_u64()).unwrap_or(0);
                let res = t.get("result").and_then(|v| v.as_str()).unwrap_or("<unknown>");
                let colored = match res {
                    "passed" => res.green(),
                    "failed" => res.red(),
                    _ => res.yellow(),
                };
                println!("  tcId {tid:>5}: {colored}");
            }
        }
    }
}

fn humanize_session_summary(session_id: u64, results: &serde_json::Value, json_out: bool) {
    if json_out {
        println!("{}", serde_json::to_string_pretty(results).unwrap_or_else(|_| "{}".into()));
        return;
    }
    println!("\nSession {} summary:", session_id);
    if let Some(arr) = results.get(1).and_then(|p| p.get("results")).and_then(|r| r.as_array()) {
        for entry in arr {
            let vs_url = entry.get("vectorSetUrl").and_then(|v| v.as_str()).unwrap_or("<unknown>");
            let disp = entry.get("disposition").and_then(|v| v.as_str()).unwrap_or("<pending>");
            let colored = match disp {
                "passed" | "approved" => disp.green(),
                "failed" | "error" => disp.red(),
                _ => disp.yellow(),
            };
            println!("  {} -> {}", vs_url, colored);
        }
    } else {
        println!("  <no results yet>");
    }
}

//  helpers 
fn is_absolute_url(s: &str) -> bool {
    s.starts_with("http://") || s.starts_with("https://")
}
fn normalize_to_relative(base_url: &str, url_or_path: &str) -> String {
    let mut s = url_or_path.strip_prefix(base_url).unwrap_or(url_or_path);
    s = s.trim_start_matches('/');
    if let Some(rest) = s.strip_prefix("acvp/v1/") { s = rest; }
    s.trim_start_matches('/').to_string()
}