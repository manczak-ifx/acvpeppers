---
name: add-crypto-backend
description: >-
  Step-by-step recipe for integrating a new cryptographic module or algorithm into
  ACVPeppers — whether an in-process Rust library, a remote service, a secure element,
  or an FFI bridge. Use when adding a new algorithm implementation or wiring up a new
  execution backend so it participates in AFT/MCT/LDT test execution and produces the
  correct ACVP response fields.
---

# Add a crypto backend / algorithm to ACVPeppers

Use this when you want a new crypto module to answer ACVP test cases. There are two
integration styles; pick based on where the crypto lives.

> Precondition: know the target algorithm's exact ACVP request/response field names
> (see the `acvp-protocol-reference` skill and the algorithm spec).

## A. In-process algorithm (built-in library backend)
This is the current pattern (`SHA2`, `AesEcbOp`).
1. Create `src/cryptography/<algo>.rs`.
2. Implement the execution contract for the algorithm:
   - Today: `impl CryptoOperation` with `fn execute(&self, tc: &TestCase) -> String`
     (and `execute_streaming` if it supports LDT).
   - After CRYPTO-1/2 land: implement `CryptoBackend::run_case` returning a
     `CaseResponse { fields }` map with the correct ACVP field names.
3. Register it in `src/cryptography/registry.rs` under its exact ACVP algorithm name
   (e.g. `"ACVP-AES-CBC"`), matching what you register in `capabilities.json`.
4. Ensure `TestCase` (in `src/parser.rs`) carries the fields your algorithm needs. If a
   field is missing, add it (prefer `#[serde(flatten)]` into a map so future fields
   don't require parser edits).
5. Confirm the three test types behave:
   - **AFT**: single `run_case` per case.
   - **MCT**: many chained calls (inner/outer loops) — follow the spec's chaining.
   - **LDT**: streaming over a `Read` source (see `LDT.rs` / `execute_streaming`).

## B. External module (service / secure element / FFI)
Use the transport-agnostic `CryptoBackend` (see ANALYSIS §5.2). The executor calls
`run_case`; the backend decides how to reach the crypto.
1. Create a backend type implementing `CryptoBackend` (`supports`, `run_case`).
2. Choose the transport:
   - **Service**: send `CaseRequest` over HTTP/gRPC/TCP; implement in `src/transport/`.
   - **Secure element**: PKCS#11 / APDU; implement in `src/cryptography/hw_proxy.rs`.
   - **FFI**: bind a C/Java/Python implementation; keep the FFI surface small and safe.
3. Serialize `CaseRequest` and parse the module's answer back into `CaseResponse`
   fields with the exact ACVP names.
4. Register the backend for the algorithm(s) it `supports()`.

## Always
- **Add a known-answer test.** Validate against ACVP `isSample=true` vectors — the
  server returns expected values you can diff.
- **No panics.** A malformed case fails that case, not the run.
- **Match capabilities.** The algorithm name + parameters in `capabilities.json` must
  match what the backend claims to support, or the server won't send those vectors.
- **SPDX header.** New files need `// SPDX-License-Identifier: MIT` (see the
  `reuse-spdx-compliance` skill).

## Checklist
- [ ] Algorithm name matches `capabilities.json` and registry key.
- [ ] Request fields parsed; response fields named per spec.
- [ ] AFT/MCT/LDT paths handled as applicable.
- [ ] KAT test passes on sample vectors.
- [ ] clippy/fmt/doc/reuse gates green.
