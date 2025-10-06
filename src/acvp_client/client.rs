use anyhow::{anyhow, Result};
use base64::engine::general_purpose::STANDARD;
use base64::Engine as _;
use reqwest::blocking::{Client, Response};
use reqwest::{Identity, Method};
use serde_json::{json, Value};
use std::{fs, thread, time::Duration};

use crate::acvp_client::hotp::totp; 

/// Struct for sessio tokenss
pub struct Session {
    pub id: u64,
    pub token: String,
    pub raw: Value,
}

pub struct AcvpClient {
    base_url: String,            
    http: Client,
    login_jwt: Option<String>,   
}

impl AcvpClient {
    pub fn from_pkcs12(base_url: &str, p12_path: &str, p12_password: &str, _otp_seed_path_hint: &str) -> Result<Self> {
        let der = fs::read(p12_path)?;
        let identity = Identity::from_pkcs12_der(&der, p12_password)?;
        let http = Client::builder().connect_timeout(Duration::from_secs(10)).identity(identity).build()?;

        Ok(Self {
            base_url: ensure_trailing_slash(base_url),
            http,
            login_jwt: None,
        })
    }

    /// Getter for URL
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Login with TOTP 
    /// Stores the **login JWT** for subsequent authorized requests
    pub fn login_with_totp(&mut self, seed_path: &str) -> Result<()> {
        // 1) Read base64-encoded TOTP seed
        let seed_b64 = fs::read_to_string(seed_path)?.trim().to_string();
        let seed = STANDARD
            .decode(seed_b64)
            .map_err(|e| anyhow!("failed to decode base64 seed: {e}"))?;

        // 2) Generate TOTP (ACVP: 30s step, 8 digits, SHA-256 in your totp())
        let password = totp(&seed, 30, 8);

        // 3) ACVP-wrapped login body and POST /login 
        let body = json!([
            { "acvVersion": "1.0" },
            { "password": password }
        ]);
        let resp = self.request("login", Method::POST, Some(&body), /*with_auth=*/false)?;

        // 4) Extract and store login JWT
        let token = resp[1]["accessToken"]
            .as_str()
            .ok_or_else(|| anyhow!("login response missing accessToken"))?;
        self.login_jwt = Some(token.to_string());
        Ok(())
    }

    // Create a new test session using the **login JWT**.
    pub fn test_session_create(&self, algorithms: &Value, is_sample: bool, encrypt_at_rest: bool) -> Result<Session> {
        let algs = algorithms
            .as_array()
            .ok_or_else(|| anyhow!("`algorithms` must be a JSON array of capability objects"))?;
        if algs.is_empty() {
            return Err(anyhow!("`algorithms` array is empty"));
        }

        let body = json!([
            { "acvVersion": "1.0" },
            {
              "isSample": is_sample,
              "encryptAtRest": encrypt_at_rest,
              "algorithms": algs
            }
        ]);

        let mut resp = self.request("testSessions", Method::POST, Some(&body), /*with_auth(login_jwt)*/ true)?;
        if resp.is_object() {
            let ver = resp.get("acvVersion").and_then(|v| v.as_str()).unwrap_or("1.0");
            let mut obj = resp.as_object().unwrap().clone();
            obj.remove("acvVersion");
            resp = json!([{ "acvVersion": ver }, Value::Object(obj)]);
        }
        let url = resp[1]["url"].as_str().ok_or_else(|| anyhow!("missing `url` in test_session_create response"))?;
        let id = url
            .rsplit('/')
            .next()
            .ok_or_else(|| anyhow!("malformed url in response: {url}"))?
            .parse::<u64>()
            .map_err(|e| anyhow!("failed to parse test session id from `{url}`: {e}"))?;

        let session_token = resp[1]["accessToken"]
            .as_str()
            .ok_or_else(|| anyhow!("missing session accessToken in response"))?
            .to_string();

        Ok(Session { id, token: session_token, raw: resp })
    }

    pub fn get_session_with_retry(
        &self,
        session_id: u64,
        session_token: &str,
        max_attempts: u32,
        default_sleep_secs: u64,
    ) -> Result<Value> {
        let path = format!("testSessions/{}/", session_id);
        let mut attempts = 0u32;

        loop {
            attempts += 1;
            let resp = self.request_with_token(&path, Method::GET, None, session_token)?;

            let payload = &resp[1];
            if let Some(retry_secs) = payload.get("retry").and_then(|v| v.as_u64()) {
                thread::sleep(Duration::from_secs(retry_secs.max(1)));
                if attempts >= max_attempts {
                    return Err(anyhow!(
                        "ACVP retry limit reached ({} attempts). Last retry hint: {}s",
                        attempts,
                        retry_secs
                    ));
                }
                continue;
            }
            if !payload.get("vectorSetUrls").is_some() {
                if attempts >= max_attempts {
                    return Err(anyhow!(
                        "missing vectorSetUrls after {} attempts; payload was: {}",
                        attempts,
                        payload
                    ));
                }
                thread::sleep(Duration::from_secs(default_sleep_secs.max(1)));
                continue;
            }

            if payload.get("status").and_then(|v| v.as_str()) == Some("expired") {
                return Err(anyhow!("Test session {} has expired", session_id));
            }

            return Ok(resp);
        }
    }

    pub fn wait_for_vector_set_urls(
        &self,
        session_id: u64,
        session_token: &str,
        max_attempts: u32,
        default_sleep_secs: u64,
    ) -> Result<Vec<String>> {
        let resp = self.get_session_with_retry(session_id, session_token, max_attempts, default_sleep_secs)?;
        let urls = resp[1]["vectorSetUrls"]
            .as_array()
            .ok_or_else(|| anyhow!("vectorSetUrls is not an array"))?
            .iter()
            .map(|v| v.as_str().ok_or_else(|| anyhow!("bad vectorSetUrl entry")).map(|s| s.to_string()))
            .collect::<Result<Vec<_>>>()?;
        Ok(urls)
    }
   pub fn get_session_results(
    &self,
    session_id: u64,
    session_token: &str,
    wait: bool,
    max_attempts: u32,
    sleep_secs: u64,
) -> Result<Value> {
    let path = format!("testSessions/{}/results", session_id);
    let mut attempts = 0u32;
    loop {
        attempts += 1;
        let resp = self.request_with_token(&path, Method::GET, None, session_token)?;
        let payload = resp.get(1).ok_or_else(|| anyhow!("results payload missing [1]"))?;
        if let Some(retry_secs) = payload.get("retry").and_then(|v| v.as_u64()) {
            if !wait || attempts >= max_attempts {
                return Ok(resp); 
            }
            std::thread::sleep(std::time::Duration::from_secs(retry_secs.max(1)));
            continue;
        }
        if wait {
            let ready = payload.get("results")
                .and_then(|r| r.as_array())
                .map(|arr| !arr.is_empty() && arr.iter().all(|it|
                    it.get("disposition").is_some()
                ))
                .unwrap_or(false);

            if ready || attempts >= max_attempts {
                return Ok(resp);
            }

            std::thread::sleep(std::time::Duration::from_secs(sleep_secs.max(1)));
            continue;
        }

        return Ok(resp);
    }
}
   
    fn request(&self, path: &str, method: Method, body: Option<&Value>, with_auth: bool) -> Result<Value> {
        let url = format!("{}{}", self.base_url, path);
        let mut req = self.http
            .request(method, &url)
            .header("Accept", "application/json")
            .header("Content-Type", "application/json");

        if with_auth {
            let jwt = self
                .login_jwt
                .as_ref()
                .ok_or_else(|| anyhow!("missing login JWT; call login_with_totp() first"))?;
            req = req.bearer_auth(jwt);
        }

        if let Some(b) = body {
            req = req.json(b);
        }

        let resp = req.send()?;
        let status = resp.status();
        if !status.is_success() {
            let txt = try_read_body(resp);
            return Err(anyhow!("HTTP {} for url ({}): {}", status, url, txt));
        }
        Ok(resp.json()?)
    }
    pub fn request_with_token(
        &self,
        path: &str,
        method: Method,
        body: Option<&Value>,
        bearer: &str,
    ) -> Result<Value> {
        let url = format!("{}{}", self.base_url, path);
        let mut req = self.http
            .request(method, &url)
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .bearer_auth(bearer);

        if let Some(b) = body {
            req = req.json(b);
        }

        let resp = req.send()?;
        let status = resp.status();
        if !status.is_success() {
            let txt = try_read_body(resp);
            return Err(anyhow!("HTTP {} for url ({}): {}", status, url, txt));
        }
        Ok(resp.json()?)
    }
    pub fn request_abs_with_token(
        &self,
        absolute_url: &str,
        method: Method,
        body: Option<&Value>,
        bearer: &str,
    ) -> Result<Value> {
        let mut req = self.http
            .request(method, absolute_url)
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .bearer_auth(bearer);

        if let Some(b) = body {
            req = req.json(b);
        }

        let resp = req.send()?;
        let status = resp.status();
        if !status.is_success() {
            let txt = try_read_body(resp);
            return Err(anyhow!("HTTP {} for url ({}): {}", status, absolute_url, txt));
        }
        Ok(resp.json()?)
    }
}


fn try_read_body(resp: Response) -> String {
    resp.text().unwrap_or_else(|_| "<no body>".into())
}

fn ensure_trailing_slash(s: &str) -> String {
    if s.ends_with('/') { s.to_string() } else { format!("{}/", s) }
}
