# ACVPeppers — Project Analysis Report

> Status: analysis only. No existing source files were modified to produce this report.
> Date: 2026-07-07

## 1. Executive summary

**ACVPeppers** is a Rust CLI (`peppers`) that automates testing against the NIST
**ACVP** (Automated Cryptographic Validation Protocol) server. It authenticates
with mutual TLS + TOTP, registers algorithm capabilities, downloads test vectors,
executes cryptographic operations locally, uploads computed results, and polls the
server for pass/fail dispositions.

The core engine is solid and idiomatic: a parallel executor (Rayon), a pluggable
`CryptoOperation` trait with a registry, and support for the three main ACVP test
types (AFT, MCT, LDT). Three algorithms are implemented today (SHA2-256, SHA2-512,
AES-ECB).

To reach the stated goal — **open-source, with abstracted credentials and an
abstracted test-execution interface that any crypto module can plug into** — the
project needs work in four areas, in priority order:

1. **Security / OSS hygiene (P0, blocking).** Live credentials and a hardcoded
   password are committed to the repository and its history. These must be purged
   and rotated before any public release.
2. **Credential abstraction (P1).** Authentication is hardwired to a PKCS#12 file +
   password + TOTP seed file. It should sit behind a provider trait.
3. **Crypto-backend abstraction (P1).** The `CryptoOperation` trait is a good start
   but is hash-centric (results are always a single `md` string). It needs a more
   general contract and a transport-agnostic backend concept (in-process library,
   remote service, secure element, FFI, …).
4. **Correctness, tests, and OSS collateral (P1).** MCT logic needs validation, panics
   should become recoverable errors, and the project needs a license, tests, portable
   CI, and real documentation.

## 2. Architecture overview

```
main.rs ──► cli.rs (clap) ──► runner::dispatch
                                   │
        ┌──────────────────────────┼───────────────────────────────┐
        ▼                          ▼                               ▼
  acvp_client (HTTP)         capabilities.rs                 cryptography
   - client.rs  (mTLS+JWT)    - load JSON                     - mod.rs: CryptoOperation trait
   - hotp.rs    (TOTP/HOTP)                                   - registry.rs: name -> impl
   - certs/     (SECRETS!)                                    - sha2_alg.rs, aes_ecb.rs
        │                                                     - hw_proxy.rs (EMPTY stub)
        ▼                                                            ▲
   parser.rs (TestVectorSet/TestGroup/TestCase)                     │
        │                                                            │
        ▼                                                            │
   test_executor.rs ──(Rayon par_iter)──► test_types/ ──────────────┘
        │                                   - AFT.rs / MCT.rs / LDT.rs
        ▼
   result_format.rs (TestResultSet -> md)  ──► runner uploads + polls
                                                  │
   logging/ (flexi_logger, redaction)             ▼
   transport/TCP.rs (EMPTY stub)             acvp_results.log / session logs
```

### End-to-end flow (`peppers run`)
1. Load `acvp.toml` (`logging/config.rs::AppConfig`) and `capabilities.json`.
2. Build `AcvpClient` from a PKCS#12 identity; log in via TOTP → login JWT.
3. Create a test session (`testSessions`) → session id + session token.
4. Poll for `vectorSetUrls`; download each vector set.
5. `execute_test_vector` runs groups/cases in parallel; dispatch by `TestType`.
6. Serialize `TestResultSet`, POST results, poll per-VS dispositions, summarize.

### Module inventory
| Module | Role | Notes |
|---|---|---|
| `acvp_client/client.rs` | ACVP HTTP client, mTLS, JWT, session lifecycle | Auth hardwired to PKCS#12 + TOTP file |
| `acvp_client/hotp.rs` | HOTP/TOTP (HMAC-SHA256, 8 digits, 30s) | ACVP-specific |
| `cryptography/mod.rs` | `CryptoOperation` trait | `execute -> String` (md), `execute_streaming` for LDT |
| `cryptography/registry.rs` | name → `Arc<dyn CryptoOperation>` | Static registration |
| `cryptography/sha2_alg.rs` | SHA2-256/512 | `panic!` on bad input |
| `cryptography/aes_ecb.rs` | AES-ECB enc/dec (128/192/256) | Returns `""` on error |
| `cryptography/hw_proxy.rs` | **Empty** | Intended secure-element/hardware hook |
| `test_types/AFT.rs` | Algorithm Functional Test | Thin wrapper |
| `test_types/MCT.rs` | Monte Carlo Test | Hash MCT + AES MCT scaffold (needs validation) |
| `test_types/LDT.rs` | Large Data Test | Streaming via `RepeatingReader` |
| `parser.rs` | Vector-set deserialization | Fields are SHA/AES-specific |
| `result_format.rs` | Result serialization | Hardcoded `md` field |
| `runner.rs` | Orchestration, polling, humanized output | |
| `logging/` | flexi_logger config + redaction | Good token redaction already |
| `transport/TCP.rs` | **Empty** | Intended alt transport hook |

## 3. What works well today
- Clean separation of protocol client, execution, and result formatting.
- Trait-based algorithm dispatch (`CryptoOperation` + `CryptoRegistry`) — the right
  seam to grow the backend abstraction from.
- Parallel execution with Rayon at both group and case level.
- Streaming abstraction for LDT via `Read` + a repeating-pattern reader.
- Log redaction of tokens/passwords in wire logs (`trunc_json`).
- A CI pipeline already exists with clippy `-D warnings`, rustfmt, rustdoc, and
  `reuse lint` gates — a strong quality baseline to preserve.

## 4. Findings — blockers and gaps

### 4.1 Security (P0 — must fix before any public push)
- **Committed private keys / identities:** `src/acvp_client/certs/` contains
  `client.p12`, `client.key`, `client_combined.pem`, `client.cer`, a `.csr`, and
  `totp.txt` (TOTP seed). These are tracked in git and thus in history.
- **Hardcoded password:** `***REMOVED***` appears in `acvp.toml` **and** as a default in
  `src/logging/config.rs`. Even after deleting files, the password persists in history.
- **PII:** a certificate/CSR is named after a specific individual.
- **Consequence:** deleting files is insufficient — history must be rewritten and the
  credentials rotated (treat all as compromised).

### 4.2 Credential coupling (P1)
- `AcvpClient::from_pkcs12(base_url, p12_path, p12_password, totp_path)` bakes the
  auth mechanism into the client. There is no seam for env vars, OS keychains, cloud
  secret managers, or HSM/PKCS#11-backed identities.

### 4.3 Crypto-execution interface is too narrow (P1)
- `CryptoOperation::execute(&TestCase) -> String` returns a single value that the
  result layer always stores as `md`. Real ACVP responses are algorithm-specific
  (`ct`, `pt`, `tag`, `mac`, `signature`, `r`/`s`, multiple fields, arrays…).
- `TestCase` (parser) and `TestCaseResult` (result_format) hardcode SHA/AES fields.
- There is **no transport-agnostic backend concept** yet: everything is in-process.
  The empty `hw_proxy.rs` and `transport/TCP.rs` are the intended (but unimplemented)
  seams for secure elements and remote execution.

### 4.4 Correctness / robustness (P1)
- `sha2_alg.rs` and `MCT.rs` use `panic!`/`unwrap` on malformed input — a bad vector
  can crash a whole run instead of failing one case.
- The AES-MCT routine looks like an incomplete scaffold (key-schedule reconstruction
  for AES Monte Carlo differs by key size; needs validation against NIST sample data).
- No automated tests (no known-answer tests, no parser round-trip tests).

### 4.5 Open-source readiness (P0/P1)
- **No LICENSE** and no SPDX headers, yet CI runs `reuse lint` (will fail).
- **Committed artifacts:** `downloaded_vs_*.json`, `result_upload_*.json`, `logs/`,
  `acvp_debug.log`, `acvp_results.log`. `.gitignore` only ignores `/target`.
- **Internal references:** `gitlab.intra.infineon.com` URLs and internal container
  registry images in `.gitlab-ci.yml`; default GitLab template README.

## 5. Proposed target abstractions

These are **design proposals** for the backlog — not yet implemented, and no existing
files were changed.

### 5.1 Credential abstraction — `CredentialProvider`
Goal: `AcvpClient` should depend on *what* it needs (an HTTP identity + a rotating
second factor), not *where* those come from.

```rust
/// Everything the ACVP client needs to authenticate, sourced abstractly.
pub trait CredentialProvider: Send + Sync {
    /// mTLS identity for the reqwest client (PKCS#12 today; could be PKCS#11/HSM).
    fn http_identity(&self) -> anyhow::Result<reqwest::Identity>;
    /// Current one-time password for ACVP login (TOTP today; could be remote signer).
    fn totp_now(&self) -> anyhow::Result<String>;
    /// Optional: base URL / account hints.
    fn base_url(&self) -> Option<String> { None }
}
```
Reference implementations to ship: `FileCredentialProvider` (current PKCS#12 + seed
file, for backward compatibility), `EnvCredentialProvider` (base64 material via env),
and a documented extension point for secret managers / PKCS#11.
`AcvpClient` gains `from_provider(base_url, Arc<dyn CredentialProvider>)`; the existing
`from_pkcs12` becomes a thin wrapper. See **CRED-1..3** in the backlog.

### 5.2 Crypto-backend abstraction — pluggable modules
Two layers, so any module (library / service / secure element / FFI / “something
else”) can be integrated:

1. **Generalized case/result model.** Replace the fixed `md` result with a
   structured, algorithm-agnostic response (e.g. a typed enum or a `serde_json::Value`
   map keyed by ACVP field names). The parser should retain unknown fields
   (`#[serde(flatten)]` into a map) so new algorithms don’t require parser changes for
   every field.

2. **Transport-agnostic backend trait.** Keep `CryptoOperation` as the per-algorithm
   contract, but introduce a `CryptoBackend` that a registry resolves per algorithm and
   that may be local or remote:

```rust
pub struct CaseRequest<'a> { pub algorithm: &'a str, pub test_type: TestType, pub case: &'a TestCase }
pub struct CaseResponse { pub fields: serde_json::Map<String, serde_json::Value> }

pub trait CryptoBackend: Send + Sync {
    fn supports(&self, algorithm: &str) -> bool;
    fn run_case(&self, req: &CaseRequest) -> anyhow::Result<CaseResponse>;
    // Optional streaming hook for LDT-style tests, mirroring today's execute_streaming.
}
```
Then provide adapters:
- **LibraryBackend** (in-process) — wraps today’s SHA2/AES code (**CRYPTO-3**).
- **ServiceBackend** — forwards `CaseRequest` to a remote crypto service over
  HTTP/gRPC/TCP; this is where `transport/` gets filled in (**CRYPTO-4**).
- **SecureElementBackend / FfiBackend** — PKCS#11/APDU or C/Java/Python FFI bridge;
  this is where `hw_proxy.rs` gets filled in (**CRYPTO-5**).

This directly realizes the README’s “Remote executors / embedded devices / FFI”
roadmap while keeping the parallel executor and test-type logic unchanged.

## 6. Suggested roadmap (phases)
- **Phase 0 — Make it publishable (P0):** purge + rotate secrets, `.gitignore`,
  license + SPDX, scrub internal references. (SEC-1..5, OSS-1, OSS-4)
- **Phase 1 — Abstractions (P1):** `CredentialProvider`, generalized result model,
  `CryptoBackend` with a library adapter. (CRED-1..2, CRYPTO-1..3)
- **Phase 2 — Correctness & trust (P1):** validate MCT, replace panics, add KAT tests,
  restore green CI on GitHub. (ALGO-1, QUAL-1..3, OSS-2..3)
- **Phase 3 — Reach (P2):** service + secure-element/FFI backends, more algorithms,
  config-driven provider/backend selection. (CRYPTO-4..5, ALGO-2, CRED-3)

The full, tracked list lives in [`BACKLOG.md`](./BACKLOG.md). The agents and skills in
[`agentic-environment/`](./agentic-environment/README.md) are scoped to these epics.

## 7. Decisions
- **License:** MIT (selected). Requires an MIT `LICENSE`, `LICENSES/MIT.txt` for REUSE,
  and per-file `SPDX-License-Identifier: MIT` headers (see OSS-1).
- **Hosting:** repo currently targets internal GitLab; open-sourcing implies a public
  host (e.g. GitHub) and portable CI (OSS-3).
