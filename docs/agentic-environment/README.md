# ACVPeppers agentic development environment

This folder documents the AI-assisted development setup created to help continue the
ACVPeppers project toward an open-source release with abstracted credentials and an
abstracted crypto-execution interface.

Start with the analysis, then the backlog:
- [`../ANALYSIS.md`](../ANALYSIS.md) — architecture, findings, and the two target
  abstractions (credentials + crypto backend).
- [`../BACKLOG.md`](../BACKLOG.md) — prioritized epics/items (P0 → P2).

## Custom agents (`.github/agents/`)
The Copilot CLI auto-discovers these. Browse/select with `/agent` (or `/agent <name>`).
Each is scoped to one epic so responsibilities don't overlap.

| Agent | Owns | Key backlog IDs |
|-------|------|-----------------|
| `oss-readiness-auditor` | Secrets purge + rotation, MIT license/REUSE, repo hygiene, CI port | SEC-1..5, OSS-1..4 |
| `credential-abstraction-architect` | `CredentialProvider` trait + provider impls | CRED-1..3 |
| `crypto-backend-integrator` | Generalized result model + `CryptoBackend` (library/service/secure-element/FFI) | CRYPTO-1..5 |
| `acvp-algorithm-engineer` | Algorithm correctness, AFT/MCT/LDT, sample-vector validation | ALGO-1..2, QUAL-3 |
| `rust-quality-guardian` | clippy/fmt/doc/reuse gates, panics→errors, tests, reviews | QUAL-1..3 |

## Skills (`.github/skills/`)
Reusable know-how the agents (and you) can pull in. Manage with `/skills`.

| Skill | What it provides |
|-------|------------------|
| `acvp-protocol-reference` | ACVP REST flow, auth, JSON envelope, session lifecycle, endpoints |
| `add-crypto-backend` | Recipe to onboard a new algorithm/module (in-proc, service, secure element, FFI) |
| `scrub-committed-secrets` | Remove secrets from git history + rotate + guardrails |
| `reuse-spdx-compliance` | SPDX headers + REUSE so `reuse lint` passes (MIT) |

Agents reference relevant skills in their front matter, so the right know-how loads
automatically when an agent is active.

## MCP servers
See [`MCP-SERVERS.md`](./MCP-SERVERS.md) for suggested Model Context Protocol servers
(GitHub, Git, Fetch, Context7, Filesystem, Sequential Thinking) with ready-to-paste
config and a mapping to the backlog. Manage servers with `/mcp`.

## Suggested workflow
1. **Phase 0 — unblock publishing.** Run `oss-readiness-auditor` on SEC-1..4 + OSS-1.
   Rotate credentials with NIST. Do **not** push publicly until this is done.
2. **Phase 1 — abstractions.** `credential-abstraction-architect` (CRED-1/2) and
   `crypto-backend-integrator` (CRYPTO-1/2/3) build the two seams the project is about.
3. **Phase 2 — trust.** `acvp-algorithm-engineer` validates MCT (ALGO-1);
   `rust-quality-guardian` replaces panics and adds tests (QUAL-1/2/3); port CI (OSS-3).
4. **Phase 3 — reach.** Service + secure-element/FFI backends (CRYPTO-4/5), more
   algorithms (ALGO-2), config-driven provider selection (CRED-3).

Hand a specific backlog ID to the matching agent, e.g.:
> `/agent crypto-backend-integrator` then: "Implement CRYPTO-1: generalize the result
> model beyond `md`, keeping SHA2/AES output identical. Add a KAT test."

## Notes
- These files are committed to the repo, so anyone who clones the open-source project
  inherits the same agents, skills, and MCP guidance.
- The agents intentionally do **not** modify credentials or secrets on their own beyond
  what the `oss-readiness-auditor` proposes and you approve.
