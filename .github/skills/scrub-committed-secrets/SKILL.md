---
name: scrub-committed-secrets
description: >-
  Procedure to remove secrets that were committed to a git repository — from both the
  working tree and the entire history — and to rotate the leaked credentials. Use before
  open-sourcing ACVPeppers, which currently tracks client certificates, private keys, a
  PKCS#12 bundle, and a TOTP seed, plus a hardcoded password. Covers git filter-repo/BFG,
  .gitignore hardening, and secret-scanning guardrails.
---

# Scrub committed secrets (and rotate them)

Removing a secret in a new commit does **not** remove it from history — anyone can check
out an old commit and read it. You must rewrite history *and* rotate the credential.

## 0. Inventory (ACVPeppers, verify before acting)
Currently tracked secret material:
- `src/acvp_client/certs/client.p12`, `client.key`, `client_combined.pem`, `client.cer`
- `src/acvp_client/certs/*.csr` (also contains PII — a person's name)
- `src/acvp_client/certs/totp.txt` (TOTP seed)
- Hardcoded password `***REMOVED***` in `acvp.toml` and the default in
  `src/logging/config.rs`

Confirm with: `git ls-files | grep -Ei 'cert|\.p12|\.key|\.pem|\.cer|\.csr|totp'`.

## 1. Rotate FIRST (assume compromised)
History rewriting cannot un-leak anything already pushed. Before or in parallel with
cleanup, have the owner:
- Revoke/reissue the ACVP **client certificate** with NIST.
- Regenerate the **TOTP seed**.
- Change any password reused elsewhere.
Treat every committed secret as burned.

## 2. Remove from the working tree
```bash
git rm -r --cached src/acvp_client/certs
# keep a local, untracked copy if you still need to run the tool
```
Replace the hardcoded password with a runtime source (env var or a credential
provider — coordinate with `credential-abstraction-architect`). Never leave secrets in
defaults.

## 3. Harden .gitignore (prevent recurrence)
Add entries such as:
```gitignore
/target
# secrets
src/acvp_client/certs/
*.p12
*.key
*.pem
*.cer
*.csr
totp.txt
# generated artifacts / logs
logs/
*.log
downloaded_vs_*.json
result_upload_*.json
```

## 4. Rewrite history
Prefer `git filter-repo` (install separately) over `filter-branch`.
```bash
# with git-filter-repo:
git filter-repo --path src/acvp_client/certs --invert-paths
git filter-repo --replace-text <(printf '***REMOVED***==>REDACTED\n')
```
Or use BFG:
```bash
bfg --delete-folders certs --delete-files 'client.*' --replace-text replacements.txt
git reflog expire --expire=now --all && git gc --prune=now --aggressive
```
This changes commit hashes. Coordinate with all collaborators; everyone must re-clone.
`git push --force` to the (new, public) remote **only after** confirming with the owner.

## 5. Guardrails
- Add a `gitleaks` pre-commit hook and a CI job so secrets can't be re-committed.
- Consider `git-secrets` patterns for cert/key/PKCS#12 file names.

## Safety rules
- Never print secret contents to logs, chat, commits, or PRs.
- Confirm with the human before any history rewrite or force-push — it is destructive.
- Rotation is mandatory, not optional.

## Done when
- No secret material in the working tree or in any commit.
- `.gitignore` blocks re-introduction; secret-scan guardrail active.
- Owner has confirmed all leaked credentials are rotated/revoked.
