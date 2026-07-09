# Suggested MCP servers for ACVPeppers

[Model Context Protocol](https://modelcontextprotocol.io) servers give the Copilot CLI
extra tools. Below are well-known, actively maintained servers that fit this project,
why each helps, and a ready-to-paste config block.

## How to add one
Configure servers in `~/.copilot/mcp-config.json`, or run `/mcp add` inside the CLI, or
use `/mcp` to manage them interactively. Config shape:

```json
{
  "mcpServers": {
    "<name>": {
      "type": "local",              // "local" (stdio), "http", or "sse"
      "command": "npx",             // for type=local
      "args": ["-y", "<package>"],
      "env": {},
      "tools": ["*"]                // or an allow-list of tool names
    }
  }
}
```
HTTP/SSE servers use `"url"` (and optional `"headers"`) instead of `command`/`args`.

> Only enable servers you trust — MCP servers can read inputs you pass them. Prefer the
> minimal `tools` allow-list over `"*"` for anything that touches secrets or the network.

---

## Tier 1 — highest value here

### 1. GitHub MCP server  (official)
Manage issues, PRs, milestones, code search, and releases from the CLI. Directly useful
for **converting `docs/BACKLOG.md` into GitHub issues/milestones**, running the OSS
migration (OSS-2/OSS-3), and reviewing PRs as abstractions land.
> Note: the GitHub MCP toolset appears to already be available in this environment
> (github tools). If so, you can skip re-adding it. Remote (hosted) option:
```json
{
  "mcpServers": {
    "github": {
      "type": "http",
      "url": "https://api.githubcopilot.com/mcp/",
      "tools": ["*"]
    }
  }
}
```
Local (Docker) alternative: `ghcr.io/github/github-mcp-server` with a
`GITHUB_PERSONAL_ACCESS_TOKEN` env var. See github/github-mcp-server.

### 2. Git MCP server  (local repo operations)
Read history, diffs, blame, and status through structured tools — handy while planning
the **secret-history scrub (SEC-1)** and auditing what is tracked, without hand-running
many `git` commands.
```json
{
  "mcpServers": {
    "git": {
      "type": "local",
      "command": "uvx",
      "args": ["mcp-server-git", "--repository", "."],
      "tools": ["*"]
    }
  }
}
```

### 3. Fetch MCP server  (web → markdown)
Pull the **NIST ACVP specification pages** and algorithm sub-specs into context when the
`acvp-algorithm-engineer` needs exact chaining rules or field names.
```json
{
  "mcpServers": {
    "fetch": {
      "type": "local",
      "command": "uvx",
      "args": ["mcp-server-fetch"],
      "tools": ["*"]
    }
  }
}
```

---

## Tier 2 — useful accelerators

### 4. Context7  (up-to-date library docs)
Fetches current docs for crates you rely on (`reqwest`, `aes`, `sha2`, `clap`,
`serde`, `rayon`) so refactors don't drift from real APIs. Great for the
`crypto-backend-integrator` and `rust-quality-guardian`.
```json
{
  "mcpServers": {
    "context7": {
      "type": "local",
      "command": "npx",
      "args": ["-y", "@upstash/context7-mcp"],
      "tools": ["*"]
    }
  }
}
```

### 5. Filesystem MCP server  (scoped file access)
Sandboxed read/write limited to a directory you choose — a safer surface when driving
bulk edits (e.g. adding SPDX headers across many files for OSS-1).
```json
{
  "mcpServers": {
    "filesystem": {
      "type": "local",
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/home/manczak/acvpeppers"],
      "tools": ["*"]
    }
  }
}
```

### 6. Sequential Thinking MCP  (structured reasoning)
Helps decompose the larger refactors (CryptoBackend abstraction CRYPTO-2, credential
provider CRED-1) into ordered steps. Pure reasoning tool, no secrets exposure.
```json
{
  "mcpServers": {
    "sequential-thinking": {
      "type": "local",
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-sequential-thinking"],
      "tools": ["*"]
    }
  }
}
```

---

## Optional

- **Memory MCP** (`@modelcontextprotocol/server-memory`) — a persistent knowledge graph
  of decisions across sessions. Note: the Copilot CLI also has native `/memory`; only add
  this if you want a separate, exportable store.
- **Time MCP** (`mcp-server-time`) — timezone-aware timestamps; minor here (TOTP already
  uses system time correctly).

## Mapping to the work
| Task / epic | Server(s) |
|---|---|
| Backlog → issues, PR review, OSS migration (OSS-2/3, SEC-*) | GitHub |
| Plan/execute git-history secret scrub (SEC-1) | Git, GitHub |
| Exact ACVP spec rules & fields (ALGO-1/2) | Fetch |
| Crate API accuracy during refactors (CRYPTO-*, CRED-*) | Context7 |
| Bulk SPDX header edits (OSS-1) | Filesystem |
| Decomposing large refactors | Sequential Thinking |

> Package/command names occasionally change upstream. If `npx`/`uvx` reports a package
> isn't found, check the server's current README (via the Fetch server or a browser)
> for the latest invocation.
