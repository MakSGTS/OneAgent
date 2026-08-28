# Sprint 35 External AI Client Compatibility Evidence

## Status and scope

This document records the Task 5 evidence executed on 2026-08-28 from committed
Task 4 head `71425e50`. It verifies the additive ADR-0057 compatibility
boundary and introduces no production behavior. Sprint 35 remains `active`
pending the independent Task 6 integration review.

The production process supports these exact revisions:

1. `2026-07-28`: existing stateless OneAgent request metadata and response
   shapes;
2. `2025-11-25`: negotiated connection lifecycle used by the pinned Cursor
   Agent;
3. `2025-06-18`: negotiated connection lifecycle used by the pinned Codex CLI.

All three revisions share the immutable lexicographically ordered catalog:
`oneagent.context`, `oneagent.diagnostics`, `oneagent.graph`,
`oneagent.impact`, `oneagent.query`, `oneagent.symbols`, and
`oneagent.validation`. They also share the same semantic handlers, Tool Policy
gate, startup Workspace snapshot, bounds, ordering, and redaction rules.

## Pinned executable evidence

| Client artifact | Exact version | Executed SHA-256 | Source and local policy |
| --- | --- | --- | --- |
| Codex CLI in the ChatGPT application | `0.150.0-alpha.8` | `4ff5e75f028e913cfeb53bd7319f87573cdce6538c1b1ccc44ce62d5ce51ca1d` | Existing exact local executable authorized by the repository owner; no download or installation |
| Cursor Agent `darwin/arm64` | `2026.08.25-3e8eec8` | `2ccc9a8e167797641448b5e5c936f006ba137a2555f117f38c5eb76a5238a233` | Official versioned package retained only under ignored `local-artifacts/sprint-35/` |
| Cursor package archive | `2026.08.25-3e8eec8` | `81d4de7349e208d4ce441ca9c2d4e7d019ec2fbeb1137a79099fd8c4b8662f5f` | Locally observed archive digest; the vendor source did not publish a checksum |

The [investigation](external-ai-client-compatibility-investigation.md) records
the official product sources, immutable MCP tag commits, original failing
requests, and authorization boundary. The exact captured first requests are
checked in as untrusted fixtures under
`tests/fixtures/mcp/external-client-compatibility/`.

## Exact Codex CLI matrix

The executed commands resolved `ONEAGENT_REPOSITORY` with
`git rev-parse --show-toplevel`, used the existing exact Codex executable, and
used this common isolated command prefix:

```bash
ONEAGENT_REPOSITORY="$(git rev-parse --show-toplevel)"
/Applications/ChatGPT.app/Contents/Resources/codex exec \
  --ignore-user-config --ignore-rules --ephemeral --skip-git-repo-check \
  --sandbox read-only --json \
  -C "$ONEAGENT_REPOSITORY/apps/runtime/tests/fixtures/workspace_service" \
  -c "mcp_servers.oneagent.command=\"$ONEAGENT_REPOSITORY/target/debug/oneagent-mcp\"" \
  -c 'mcp_servers.oneagent.required=true' \
  -c 'mcp_servers.oneagent.startup_timeout_sec=10' \
  '<ROW-SPECIFIC PROMPT>'
```

No user config was loaded, no session was persisted, the sandbox was read-only,
and the only configured MCP server was the repository-built production binary.
The CLI retained its existing account authentication; no credential appeared
in the command, output, fixture, or tracked file.

| Row | Prompt oracle | Direct event evidence | Exit |
| --- | --- | --- | ---: |
| Success and domain failure, run 1 | Call `oneagent.graph` with `{"limit":1}`, then with `{"limit":0}`, and finish with `SUCCESS_AND_DOMAIN_ERROR` | First call `completed` with `total=2`, one result and `truncated=true`; second call `failed` with structured `code=invalid_arguments`; exact final message observed | 0 |
| Success and domain failure, run 2 | Identical prompt and command | Same semantic success and domain-failure categories; exact final message observed from a fresh ephemeral invocation | 0 |
| Seven-tool visibility | Call each exact catalog tool once in canonical order; `oneagent.graph` receives `{"limit":1}` and the other tools receive `{}` | Seven `mcp_tool_call` events named every exact tool on server `oneagent`; Graph succeeded, the six invalid empty inputs returned bounded structured `invalid_arguments`; final message `SEVEN_TOOLS_CALLED` | 0 |

A no-call prompt cannot introspect Codex's internal tool registry: diagnostic
runs returned `DIFFERENT` and then `[]`. Those two exit-zero runs are not
accepted as catalog evidence. The seven direct call events above are the
catalog oracle. Codex emitted five bounded client-side state-database warning
lines and an input notice before its JSON event stream on each invocation; no
OneAgent startup or transport failure was reported. Each accepted run
terminated normally with exit zero after the MCP input closed.

## Exact Cursor Agent matrix

Cursor used the ignored repository-local Git workspace
`local-artifacts/sprint-35/cursor-client-workspace/` and its sole project-local
`.cursor/mcp.json` entry pointed directly at the repository-built binary. The
exact command was executed twice from that directory:

```bash
"$ONEAGENT_REPOSITORY/local-artifacts/sprint-35/cursor-agent-2026.08.25-3e8eec8/darwin-arm64/cursor-agent" \
  mcp list-tools oneagent
```

Both runs exited zero and printed `Tools for oneagent (7)` followed by all seven
exact names and their argument names. Each command created a fresh server
process, completed initialize/list, closed its input, and returned without a
server diagnostic.

A newly created disposable workspace first exited one before server startup
with `MCP server "oneagent" has not been approved`. No `mcp enable` command was
run because approval storage is not the project MCP definition and could have
expanded the global mutation boundary. Reusing the repository-local workspace
that had already been approved during the authorized investigation succeeded
without another approval or config mutation.

This Cursor version exposes only `mcp login`, `list`, `list-tools`, `enable`,
and `disable`. It has no public non-interactive `mcp` tool-call command.
Therefore actual Cursor tool success and domain-failure calls are not claimed;
their absence is a client-command limitation, not a server failure. Synthetic
process evidence and Codex direct calls cover those server rows.

## Platform-neutral conformance

Repository tests consume the exact captured client fixtures and execute every
claimed server behavior through the production `oneagent-mcp` binary. The
matrix covers both legacy revisions, unsupported-version fallback, modern
`2026-07-28`, initialize/initialized ordering, pre-initialize and duplicate
initialize errors, list, success and domain-failing call projection, ping,
unknown method, `shutdown` as method-not-found, silent `exit` and cancelled
notifications, malformed JSON, LF/CRLF, request IDs, EOF, repeated processes,
two simultaneous sessions, stdout/stderr, transport cancellation/failures, and
fresh-session reuse.

The focused accepted counts at Task 5 are:

- protocol session: 9 passed;
- Runtime stdio: 8 passed;
- public `oneagent-mcp` process: 12 passed;
- semantic MCP tools: 6 passed;
- LSP process regression: 7 passed;
- VS Code: compilation passed, 62 unit tests and 2 real-process tests passed;
- EDT: Tycho/PDE build passed, 41 tests passed with zero failures, errors, or
  skips, including the real process twice.

The final canonical workspace-gate result is recorded in the Task 5 commit and
Sprint report rather than inferred from these focused counts.

## Audits and limitations

- No Cargo manifest, lockfile, production dependency, third-party package,
  license inventory, production source, catalog, schema, Tool Policy rule, or
  semantic implementation changed in Task 5.
- No credential, token, personal absolute path, client binary, archive, raw
  trace, generated package, client cache, or global client configuration is
  tracked. Downloaded clients and disposable configs remain ignored under
  `local-artifacts/sprint-35/`.
- Codex used command-line MCP configuration with `--ignore-user-config` and
  `--ephemeral`. Cursor used only its existing approved repository-local
  workspace definition. No login, enable, disable, install, update, or global
  config command was executed.
- The protocol source pinning remains the exact official MCP tag commits
  recorded by the investigation: `f5ccad944fdf2b7d9cc70cf817f66ca5a8aa03a4`
  (`2025-06-18`), `38c84e9f93ad191d9eb26d92b945d17bd0efcaf3`
  (`2025-11-25`), and `5f5440bb26a62e2cf3440b92da5a667efa03b267`
  (`2026-07-28`).
- Remote transport, authentication, prompts/resources/completions, concurrent
  calls, in-flight cancellation, pagination, mutable snapshots, additional
  clients, client packaging, and global configuration automation remain
  deferred.
