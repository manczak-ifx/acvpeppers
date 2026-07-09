# ACVPeppers — Backlog

Prioritized backlog derived from [`ANALYSIS.md`](./ANALYSIS.md). Priorities: **P0**
(blocking / must-do before public release), **P1** (core goal), **P2** (extension).
Effort: S / M / L. Each epic maps to a custom agent in
[`agentic-environment/`](./agentic-environment/README.md).

Legend for suggested owner agent:
- 🔐 `oss-readiness-auditor`
- 🪪 `credential-abstraction-architect`
- 🧩 `crypto-backend-integrator`
- 🧮 `acvp-algorithm-engineer`
- 🦀 `rust-quality-guardian`

---

## Epic: OSS Readiness  (make it safe + legal to publish)

| ID | Priority | Effort | Item | Owner |
|----|----------|--------|------|-------|
| SEC-1 | **P0** | M | Purge committed secrets from working tree **and** git history (`client.p12`, `client.key`, `client_combined.pem`, `client.cer`, `*.csr`, `totp.txt`). Use `git filter-repo`/BFG, then force-push. | 🔐 |
| SEC-2 | **P0** | S | **Rotate all leaked credentials.** Revoke/reissue the ACVP client cert and TOTP seed with NIST; treat the old material as compromised. | 🔐 |
| SEC-3 | **P0** | S | Remove hardcoded password `***REMOVED***` from `acvp.toml` and the default in `src/logging/config.rs`; source it from the credential provider/env instead. | 🔐 / 🪪 |
| SEC-4 | **P0** | S | Add a real `.gitignore` (certs, `*.p12`/`*.key`/`*.pem`, `logs/`, `downloaded_vs_*.json`, `result_upload_*.json`, `*.log`). | 🔐 |
| SEC-5 | P1 | S | Add secret-scanning guardrails: `gitleaks` pre-commit hook + CI job to block future secret commits. | 🔐 |
| OSS-1 | **P0** | M | Add **MIT** `LICENSE` + `LICENSES/MIT.txt` + per-file `SPDX-License-Identifier: MIT` headers so `reuse lint` passes. | 🔐 |
| OSS-4 | P1 | S | Remove internal references (`gitlab.intra.infineon.com`, internal registry images). | 🔐 |
| OSS-2 | P1 | M | Rewrite `README`; add `CONTRIBUTING.md`, `SECURITY.md`, `CODE_OF_CONDUCT.md`. | 🔐 |
| OSS-3 | P1 | M | Portable public CI (GitHub Actions) mirroring the existing clippy/fmt/doc/reuse gates. | 🔐 / 🦀 |

> ⚠️ **Do SEC-1 → SEC-4 and OSS-1 before the first public push.** Everything else can
> follow in the open.

---

## Epic: Credential Abstraction  (goal: pluggable credentials)

| ID | Priority | Effort | Item | Owner |
|----|----------|--------|------|-------|
| CRED-1 | P1 | M | Design the `CredentialProvider` trait (`http_identity()`, `totp_now()`, optional `base_url()`). See ANALYSIS §5.1. | 🪪 |
| CRED-2 | P1 | M | Refactor `AcvpClient` to `from_provider(...)`; keep `from_pkcs12` as a thin `FileCredentialProvider` wrapper for backward compatibility. | 🪪 |
| CRED-3 | P2 | S | Config-/env-driven provider selection; document file, env, and secret-manager/PKCS#11 options. | 🪪 |

---

## Epic: Crypto Backend Abstraction  (goal: any module can integrate)

| ID | Priority | Effort | Item | Owner |
|----|----------|--------|------|-------|
| CRYPTO-1 | P1 | M | Generalize the result model beyond a single `md` (support `ct`/`pt`/`tag`/`mac`/`signature`/…). Retain unknown parser fields via `#[serde(flatten)]`. | 🧩 |
| CRYPTO-2 | P1 | L | Define the transport-agnostic `CryptoBackend` contract (`supports`, `run_case`, optional streaming). See ANALYSIS §5.2. | 🧩 |
| CRYPTO-3 | P2 | M | Port SHA2/AES-ECB onto the abstraction as the built-in **LibraryBackend**. | 🧩 |
| CRYPTO-4 | P2 | L | **ServiceBackend**: forward cases to a remote crypto service (fills in `transport/`). | 🧩 |
| CRYPTO-5 | P2 | L | **SecureElementBackend** (PKCS#11/APDU) + **FfiBackend** (C/Java/Python) — fills in `hw_proxy.rs`. | 🧩 |

---

## Epic: Algorithm Coverage

| ID | Priority | Effort | Item | Owner |
|----|----------|--------|------|-------|
| ALGO-1 | P1 | M | Validate/fix AES-MCT and hash-MCT against NIST sample vectors (current AES MCT is an incomplete scaffold). | 🧮 |
| ALGO-2 | P2 | L | Add algorithms (HMAC, SHA3, AES-CBC/CTR/GCM, DRBG, …) via the new backend abstraction. | 🧮 |

---

## Epic: Code Quality

| ID | Priority | Effort | Item | Owner |
|----|----------|--------|------|-------|
| QUAL-1 | P1 | M | Replace `panic!`/`unwrap` in `sha2_alg.rs` and `MCT.rs` with recoverable per-case errors. | 🦀 |
| QUAL-2 | P1 | M | Keep clippy `-D warnings`, rustfmt, and rustdoc gates green. | 🦀 |
| QUAL-3 | P1 | M | Add unit + known-answer tests (per-algorithm KATs, parser round-trips). | 🦀 |

---

## Suggested sequencing

1. **Phase 0 (unblock publish):** SEC-1, SEC-2, SEC-3, SEC-4, OSS-1, OSS-4.
2. **Phase 1 (abstractions):** CRED-1, CRED-2, CRYPTO-1, CRYPTO-2, CRYPTO-3.
3. **Phase 2 (trust):** ALGO-1, QUAL-1, QUAL-2, QUAL-3, OSS-2, OSS-3, SEC-5.
4. **Phase 3 (reach):** CRYPTO-4, CRYPTO-5, ALGO-2, CRED-3.

> This backlog is mirrored in the session database (`backlog` table) for agent
> tracking. When you move to GitHub, `oss-readiness-auditor` can help convert these
> into issues/milestones using the GitHub MCP server (see
> [`agentic-environment/MCP-SERVERS.md`](./agentic-environment/MCP-SERVERS.md)).
