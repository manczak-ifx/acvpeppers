---
name: reuse-spdx-compliance
description: >-
  How to make a repository REUSE-compliant with SPDX license headers so the `reuse lint`
  CI check passes. Use when adding the project license (MIT for ACVPeppers), adding
  per-file SPDX headers, or handling files that cannot carry a header (via .license files
  or REUSE.toml). ACVPeppers CI already runs `reuse lint`, which currently fails because
  there is no license.
---

# REUSE / SPDX compliance (MIT)

ACVPeppers' CI runs `reuse lint`. To pass, every file must have clear license and
copyright info, expressed with SPDX. The project license is **MIT**.

## 1. Add the license text for REUSE
Place the license under `LICENSES/`:
```
LICENSES/MIT.txt
```
(Use the exact SPDX MIT text.) Also keep a conventional root `LICENSE` for humans and
GitHub's license detector — it can be the same MIT text.

## 2. Add per-file SPDX headers
Every source file gets a copyright line and an SPDX identifier. Use the correct comment
syntax for the file type.

Rust / C / Java:
```rust
// SPDX-FileCopyrightText: <year> <copyright holder>
//
// SPDX-License-Identifier: MIT
```
TOML / YAML / shell (`#` comments):
```toml
# SPDX-FileCopyrightText: <year> <copyright holder>
#
# SPDX-License-Identifier: MIT
```

Tip: `reuse annotate --license MIT --copyright "<holder>" <files...>` adds headers
automatically for supported file types.

## 3. Files that can't hold a header
For binary files, JSON, or generated data where a header is impossible, either:
- add a sidecar `<file>.license` containing the SPDX header, or
- record licensing in a `REUSE.toml` (or `.reuse/dep5`) `annotations` block, e.g.:
```toml
version = 1
[[annotations]]
path = ["capabilities.json", "assets/**"]
SPDX-FileCopyrightText = "<year> <holder>"
SPDX-License-Identifier = "MIT"
```
> Better: don't track generated/secret files at all (see `scrub-committed-secrets`).

## 4. Verify locally
```bash
reuse lint
```
Fix every "files without copyright/license" finding until it reports compliant.

## Checklist
- [ ] `LICENSES/MIT.txt` present; root `LICENSE` present.
- [ ] All source files carry `SPDX-License-Identifier: MIT` + copyright.
- [ ] Non-headerable files covered by `.license` sidecars or `REUSE.toml`.
- [ ] `reuse lint` passes.
- [ ] New files added later also get headers (remind contributors in `CONTRIBUTING.md`).
