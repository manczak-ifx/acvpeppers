---
name: acvp-protocol-reference
description: >-
  Reference for the NIST ACVP (Automated Cryptographic Validation Protocol) REST flow
  as used by ACVPeppers: authentication (mTLS + TOTP), the request/response envelope,
  the session lifecycle (login → register capabilities → retrieve vector sets → submit
  results → poll dispositions), and the demo vs production endpoints. Use when working
  on the ACVP client, session handling, or when you need to know which endpoint or JSON
  shape to use.
---

# ACVP protocol reference (as used by ACVPeppers)

This summarizes how ACVPeppers talks to the ACVP server. Treat the official NIST ACVP
specification as authoritative; this is an implementer's cheat sheet.

## Endpoints
- Demo base URL: `https://demo.acvts.nist.gov/acvp/v1/` (used for `isSample` runs).
- Production base URL differs and requires a production certificate.
- The base URL is configured in `acvp.toml` (`base_url`) and normalized to end with `/`.

## Authentication
1. **Mutual TLS.** The HTTP client presents a client identity (today a PKCS#12
   bundle). See `AcvpClient::from_pkcs12` in `src/acvp_client/client.rs`.
2. **TOTP login.** POST to `login` with a TOTP one-time password to obtain a **login
   JWT**. ACVP TOTP parameters (see `src/acvp_client/hotp.rs`): HMAC-**SHA256**,
   **8 digits**, **30-second** step, seed is base64-encoded.
3. Subsequent calls use `Authorization: Bearer <jwt>`. Session-scoped calls use the
   per-session `accessToken` returned by session creation.

## Request/response envelope
Every request and response is a **two-element JSON array**:
```json
[ { "acvVersion": "1.0" }, { /* payload */ } ]
```
- Element `[0]` is the version object; element `[1]` is the actual payload.
- ACVPeppers reads `resp[1]` for payloads and wraps outgoing bodies the same way.

## Session lifecycle (`peppers run`)
1. `POST login` → login JWT.
2. `POST testSessions` with `{ isSample, encryptAtRest, algorithms: [...] }` (the
   capabilities from `capabilities.json`) → session `url` (id parsed from tail) +
   session `accessToken`.
3. `GET testSessions/{id}/` → poll until `vectorSetUrls` appears. Responses may include
   a `retry` hint (seconds) telling you to wait and re-poll.
4. For each vector-set URL → `GET` it → parse into `TestVectorSet` (`src/parser.rs`).
5. Execute locally → build a `TestResultSet` → `POST testSessions/{id}/vectorSets/{vsId}/results`.
6. `GET testSessions/{id}/vectorSets/{vsId}/results` → poll until a `disposition`/
   `results`/`tests` field appears (honor `retry`).
7. `GET testSessions/{id}/results` → overall session summary.

## Key JSON shapes
- **Vector set**: `{ vsId, algorithm, revision, testGroups: [ { tgId, testType, tests: [ { tcId, ... } ] } ] }`.
  `testType` ∈ `AFT` | `MCT` | `LDT`. Case fields are algorithm-specific
  (`msg`/`len` for hashes; `key`/`pt`/`ct`/`iv` for AES; `largeMsg` for LDT).
- **Result set** (upload): `{ vsId, algorithm, revision, testGroups: [ { tgId, tests: [ { tcId, <responseFields> } ] } ] }`.
  Today ACVPeppers emits a single `md` per case; a generalized model should emit the
  correct field(s) per algorithm (`md`, `ct`, `pt`, `tag`, `mac`, `signature`, …).

## `retry` polling pattern
When a payload contains `{ "retry": N }`, sleep `max(N,1)` seconds and re-request, up to
a max attempt count. This is used for both vector-set readiness and result dispositions.

## Practical tips
- Use `isSample=true` (the `--publish`/`--immediate` flags map to session options) so
  the server returns expected answers you can validate against.
- Wire logs redact `token`/`jwt`/`password`/`totp` (see `logging/logging.rs::trunc_json`);
  keep that redaction intact.
- Never commit certificates or the TOTP seed (see the `scrub-committed-secrets` skill).
