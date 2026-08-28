# Sprint 35 External AI Client Compatibility Evidence

## Status and scope

This document records the Task 5 evidence executed on 2026-08-28 from committed
Task 4 head `71425e50` and the corrective evidence executed after successive
Task 6 reviews. Task 5 introduced no production behavior. The corrective
changes close revision-aware initialization, lifecycle, mode-selection, error-
precedence, and generic-metadata gaps found by those reviews. The
[Sprint 35 integration review](../reviews/sprint-35-external-ai-client-compatibility.md)
records `pass with non-blocking follow-ups`, and Sprint 35 is `completed`.

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

The corrective evidence resolved `ONEAGENT_REPOSITORY` with
`git rev-parse --show-toplevel` and executed these exact commands. The first two
commands are intentionally identical fresh ephemeral runs:

```bash
ONEAGENT_REPOSITORY="$(git rev-parse --show-toplevel)"
/Applications/ChatGPT.app/Contents/Resources/codex exec \
  --ignore-user-config --ignore-rules --ephemeral --skip-git-repo-check \
  --sandbox read-only --json \
  -C "$ONEAGENT_REPOSITORY/apps/runtime/tests/fixtures/workspace_service" \
  -c "mcp_servers.oneagent.command=\"$ONEAGENT_REPOSITORY/target/debug/oneagent-mcp\"" \
  -c 'mcp_servers.oneagent.required=true' \
  -c 'mcp_servers.oneagent.startup_timeout_sec=10' \
  'Use only the oneagent MCP server. Call oneagent.graph with {"limit":1}. Then call oneagent.graph with {"limit":0}. Confirm that the first call succeeds with total 2 and truncated true and that the second call returns the structured invalid_arguments domain error. Reply with exactly SUCCESS_AND_DOMAIN_ERROR.'

/Applications/ChatGPT.app/Contents/Resources/codex exec \
  --ignore-user-config --ignore-rules --ephemeral --skip-git-repo-check \
  --sandbox read-only --json \
  -C "$ONEAGENT_REPOSITORY/apps/runtime/tests/fixtures/workspace_service" \
  -c "mcp_servers.oneagent.command=\"$ONEAGENT_REPOSITORY/target/debug/oneagent-mcp\"" \
  -c 'mcp_servers.oneagent.required=true' \
  -c 'mcp_servers.oneagent.startup_timeout_sec=10' \
  'Use only the oneagent MCP server. Call oneagent.graph with {"limit":1}. Then call oneagent.graph with {"limit":0}. Confirm that the first call succeeds with total 2 and truncated true and that the second call returns the structured invalid_arguments domain error. Reply with exactly SUCCESS_AND_DOMAIN_ERROR.'

/Applications/ChatGPT.app/Contents/Resources/codex exec \
  --ignore-user-config --ignore-rules --ephemeral --skip-git-repo-check \
  --sandbox read-only --json \
  -C "$ONEAGENT_REPOSITORY/apps/runtime/tests/fixtures/workspace_service" \
  -c "mcp_servers.oneagent.command=\"$ONEAGENT_REPOSITORY/target/debug/oneagent-mcp\"" \
  -c 'mcp_servers.oneagent.required=true' \
  -c 'mcp_servers.oneagent.startup_timeout_sec=10' \
  'Use only the oneagent MCP server. Call each of these seven tools exactly once in this exact order: oneagent.context with {}, oneagent.diagnostics with {}, oneagent.graph with {"limit":1}, oneagent.impact with {}, oneagent.query with {}, oneagent.symbols with {}, and oneagent.validation with {}. After all seven calls, reply with exactly SEVEN_TOOLS_CALLED.'
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

## Exact existing-client compatibility commands

The final corrective VS Code compatibility run used the Node executable from
the ignored pinned Cursor package and the existing installed extension
dependencies. From the repository root, the exact commands were:

```bash
ONEAGENT_REPOSITORY="$(git rev-parse --show-toplevel)"
ONEAGENT_NODE="$ONEAGENT_REPOSITORY/local-artifacts/sprint-35/cursor-agent-2026.08.25-3e8eec8/darwin-arm64/node"
cd "$ONEAGENT_REPOSITORY/extensions/vscode"
"$ONEAGENT_NODE" node_modules/typescript/bin/tsc -p tsconfig.json --noEmit
"$ONEAGENT_NODE" node_modules/typescript/bin/tsc -p tsconfig.test.json --noEmit
"$ONEAGENT_NODE" node_modules/typescript/bin/tsc -p tsconfig.json
"$ONEAGENT_NODE" node_modules/typescript/bin/tsc -p tsconfig.test.json
"$ONEAGENT_NODE" --test dist-test/test/unit/*.test.js
ONEAGENT_MCP_BIN="$ONEAGENT_REPOSITORY/target/debug/oneagent-mcp" \
  "$ONEAGENT_NODE" --test dist-test/test/integration/*.test.js
```

All four TypeScript commands exited zero. The unit row passed 62 of 62 with
zero failures, cancellations, skips, or todos. An initial integration command
without `ONEAGENT_MCP_BIN` exited one before spawning Runtime and failed 2 of 2;
that invocation is not acceptance evidence. The exact corrected command above
passed 2 of 2 with all negative counters zero.

The final corrective EDT host run used the exact command below from
`extensions/edt`:

```bash
ONEAGENT_REPOSITORY="$(git rev-parse --show-toplevel)"
cd "$ONEAGENT_REPOSITORY/extensions/edt"
ONEAGENT_MCP_EXECUTABLE="$ONEAGENT_REPOSITORY/target/debug/oneagent-mcp" \
ONEAGENT_MCP_FIXTURE="$ONEAGENT_REPOSITORY/apps/runtime/tests/fixtures/workspace_service" \
  ./mvnw --batch-mode --no-transfer-progress clean verify
```

The command exited zero with `BUILD SUCCESS`; all 41 tests passed with zero
failures, errors, or skips, and the public Runtime process was exercised twice.
The known Tycho platform-shutdown job warning remained non-fatal.

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

The final corrective focused counts are:

- protocol session: 12 passed;
- Runtime stdio: 8 passed;
- public `oneagent-mcp` process: 16 passed;
- semantic MCP tools: 6 passed;
- LSP process regression: 7 passed;
- VS Code: compilation passed, 62 unit tests and 2 real-process tests passed;
- EDT: Tycho/PDE build passed, 41 tests passed with zero failures, errors, or
  skips, including the real process twice.

The revision-aware protocol tests accept the exact revision-specific shapes
and reject scalar initialize `_meta`, non-string/non-number progress tokens,
non-boolean `roots.listChanged`, malformed `2025-11-25`
sampling/elicitation/tasks fields, `2025-11-25`-only fields in `2025-06-18`,
and wrong known implementation field types. Every invalid request returns
`-32602`, retains an undetermined session, and permits a following valid
initialize. The public process repeats the atomic rejection boundary through
the built `oneagent-mcp`.

A second fresh-context review found two uncovered lifecycle branches. The
follow-up correction validates notification `_meta` before accepting
`notifications/initialized` and distinguishes unknown requests from known
operational requests while the connection awaits that notification. Protocol
and public-process regressions prove for both legacy revisions that malformed
initialized metadata stays silent without activating the session, an unknown
request returns `-32601`, a known `ping` remains gated by `-32002`, and a later
valid initialized notification activates the same connection.

A third fresh-context review found that the undetermined state selected the
legacy decoder solely from the method name `initialize`, before recognizing a
valid modern envelope. The final correction gives a valid `2026-07-28`
request the existing stateless dispatch semantics before attempting legacy
initialize. Protocol equality and public-process regressions prove that modern
`initialize` returns `-32601`, selects Modern, and permits a following modern
discover, while the full legacy initialize matrix remains unchanged.

A fourth fresh-context review found that one neighboring pre-initialize path
still exposed the modern unsupported-version error for operational methods.
The final precedence correction maps every failed modern decode of `ping`,
`tools/list`, or `tools/call` in Undetermined to `-32002`, after raw
parse/envelope validation and before mode selection. Protocol and public-
process rows cover all three methods with well-typed `2025-11-25` metadata,
prove the state remains Undetermined, and then complete a valid initialize.

A fifth fresh-context review found two adjacent generic-metadata gaps. A schema-
valid open notification `_meta` object could be rejected because it was checked
with the request-only progress-token validator, and an active legacy `ping`
could reject schema-valid generic request metadata. The latest correction uses
the open notification metadata contract for `notifications/initialized`,
allows only an already validated `_meta` field on `ping`, and retains
`-32602` for scalar notification metadata, non-string/non-number request
progress tokens, and every unexpected ping field. Protocol and public-process
rows cover both legacy revisions, string and number tokens, open notification
metadata values, negative forms, state transitions, and response shapes.

The final accepted corrective canonical gate is:

| Command | Exit and exact outcome |
| --- | --- |
| `cargo fmt --all --check` | 0 |
| `cargo check --workspace --all-targets` | 0 |
| `cargo test --workspace --all-targets` | 0; 72 test-result targets, 1,140 passed, 0 failed, ignored, measured, or filtered; four binary targets contain zero tests and are not acceptance filters |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | 0 |
| `cargo doc --workspace --no-deps` | 0 |
| `git diff --check` | 0 |

The workspace-test aggregate was computed from the complete output under
`set -o pipefail`; the temporary raw log remains ignored under
`local-artifacts/sprint-35/`. An earlier development gate found one Clippy
`redundant_closure` warning in the new validator and was not accepted. The
smallest mechanical fix was applied before the complete successful cycle above.

The first final EDT host attempt encountered one timing-sensitive existing
`lateCancellationInterruptsPostFrameExitWait` failure (`PROCESS_FAILED` rather
than `CANCELLED`) and failed the build with 40 of 41 tests passing. No EDT code
is in the final corrective diff. One immediate sequential rerun of the exact
full command completed with `BUILD SUCCESS`, all 41 tests passing, zero
failures, errors, or skips, and the public Runtime process exercised twice.
The later final correction's EDT host run passed on its first attempt with the
same 41-test, zero-failure/error/skip result.

During the latest correction, running the seven LSP public-process tests in
parallel with two other process-heavy suites exhausted their five-second test
deadline and produced seven timeouts. The isolated immediate rerun passed all
7 tests in 0.69 seconds, and the subsequent sequential canonical workspace run
also passed those tests. This scheduling-induced attempt is not acceptance
evidence. The exact Codex success/domain commands passed twice, the exact
seven-tool command passed after the repository owner explicitly authorized its
result payload, and both Cursor discovery commands passed. The first attempted
seven-tool host approval was rejected before Codex started and changed no
state; it is not a client execution result.

## Post-review follow-up hardening

On 2026-08-29, the accepted non-blocking Sprint 35 follow-ups were completed
without changing production code. One table-driven protocol test now locks the
unknown-method precedence for an Undetermined connection across absent,
malformed, supported-legacy, and valid-modern metadata at `-32602`, `-32602`,
`-32022`, and `-32601`. Named positive rows also cover explicit empty-object
`ping` params and valid generic request metadata on `tools/list` for both
legacy revisions.

`cargo test -p oneagent-protocol --test mcp_session` passed 13 tests with zero
failures. The subsequent complete workspace gate passed 1,141 tests with zero
failures; `cargo fmt --all --check`, `cargo check --workspace --all-targets`,
strict all-target Clippy, warnings-as-errors Rustdoc, and `git diff --check`
also exited zero.

## Audits and limitations

- Sprint 35 added connection-owned legacy negotiation, lifecycle validation,
  error precedence, generic metadata handling, and revision-specific legacy
  initialize/list/call/ping response projection. The existing modern response
  schema, immutable catalog, Tool Policy rules, semantic implementations,
  Cargo manifests, lockfile, production dependencies, third-party packages,
  and license inventory remain unchanged. Task 5 changed no production source;
  its subsequent corrective changes remained inside the protocol session and
  production-process compatibility boundaries.
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
