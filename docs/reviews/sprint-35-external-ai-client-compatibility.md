# Sprint 35 External AI Client Compatibility Review

## Decision

`pass with non-blocking follow-ups`

The effective decision matches the independent reviewer recommendation. Sprint
35 satisfies ADR-0057 for the exact pinned Codex CLI and Cursor Agent: one
connection-local compatibility state machine negotiates MCP `2025-06-18` or
`2025-11-25`, preserves the existing stateless `2026-07-28` path, projects
revision-correct legacy results, and reuses the immutable seven-tool catalog,
Tool Policy, semantic handlers, Workspace snapshot, framing, bounds, cleanup,
and redaction contracts.

This decision does not claim compatibility with unexecuted clients, Cursor
direct tool calls that its public CLI cannot issue, remote MCP transports,
authentication, prompts, resources, completions, concurrent calls, in-flight
cancellation, pagination, mutable snapshots, client packaging, global
configuration automation, additional tools, or changed semantic results.

## Reviewed baseline

- Completed Sprint 34 prerequisite: `c83adc4a`.
- Sprint 35 planning anchor: `cd876c83`.
- Task 5 head: `c70173e7`.
- Final corrective head: `2f809f6d`.
- Exact reviewed range:
  `cd876c836014fbd1ee15c0683da67d036f181b3c..2f809f6dddafa2586bd33ce7d442500c100d1e3c`.
- Range size: 10 commits, 19 paths, 3,177 additions, 115 deletions.

The dependency-ordered commits are:

| Step | Commit | Subject | Result |
| --- | --- | --- | --- |
| Planning anchor | `cd876c83` | `Plan Sprint 35 External AI Client Compatibility` | pass |
| Investigation | `8c1d5887` | `Investigate Sprint 35 external AI client compatibility` | pass |
| ADR-0057 | `89f947d2` | `Define Sprint 35 external AI client compatibility` | pass |
| Protocol | `a731a8d6` | `Implement Sprint 35 legacy MCP protocol` | remediated |
| Runtime lifecycle | `71425e50` | `Integrate Sprint 35 MCP client lifecycle` | remediated |
| External evidence | `c70173e7` | `Complete Sprint 35 external client evidence` | remediated |
| Revision validation | `88ccac48` | `Correct Sprint 35 legacy initialize evidence` | remediated |
| Awaiting lifecycle | `237d5881` | `Fix legacy MCP initialized validation and awaiting error projection` | remediated |
| Modern precedence | `6a808fc1` | `Preserve modern MCP dispatch before legacy initialization` | remediated |
| Pre-initialize precedence | `16fc33c9` | `Close legacy MCP pre-initialize error precedence` | remediated |
| Generic metadata | `2f809f6d` | `Accept legacy MCP generic metadata shapes` | pass |

The final reviewer began and ended at
`2f809f6dddafa2586bd33ce7d442500c100d1e3c` with an empty
`git status --short`. The reviewer remained fresh-context and read-only,
delegated no work, and made no edit, creation, deletion, staging, commit,
client launch, Roadmap transition, or prompt retirement.

## Independent reviewer and reconciliation

Reviewer `/root/sprint_35_metadata_final_reviewer` received only the repository
root, exact range, authorities, acceptance and exclusion matrices, validation
contract, and structured output requirements. It was not given an expected
decision. The reviewer recommended `pass with non-blocking follow-ups`, found
no protocol, lifecycle, Runtime, modern-compatibility, security, dependency, or
scope blocker, and independently passed the focused and full Rust gates.

Primary reconciliation is complete:

| Reviewer item | Primary classification | Reconciliation |
| --- | --- | --- |
| Evidence audit text says the corrective change touched only initialize validation and ambiguously says response schema was unchanged. | Accepted, non-blocking documentation debt. | This review records the complete production change accurately. The evidence wording remains an explicit follow-up; it is not silently changed in the review commit. Catalog, Tool Policy, semantic handlers, and dependencies are unchanged. |
| No one table-driven test covers unknown `Undetermined` methods across absent, malformed, legacy-version, and valid-modern metadata. | Accepted, non-blocking hardening. | Independent production-process oracles returned `-32602`, `-32602`, `-32022`, and `-32601`; the shared decoder and adjacent committed tests cover the behavior. |
| No separate named rows cover explicit empty-object `ping` or valid generic metadata on `tools/list`. | Accepted, non-blocking hardening. | The shared legacy decoder, method validators, ping/call metadata tests, lifecycle tests, and process matrix establish the accepted behavior. |
| Focused commands are not listed literally in the Task 5 evidence. | Accepted, non-blocking evidence presentation gap. | The exact commands and outcomes are recorded below and were independently and primarily executed. |
| Host global-configuration state and ignored client artifacts cannot be reproduced from Git alone. | Accepted bounded limitation. | Primary ran Codex with `--ignore-user-config --ephemeral`; Cursor used only the existing approved ignored repository-local workspace. No login, enable, disable, install, update, or global-config command ran. |

No blocking disagreement or unresolved evidence item remains.

## Acceptance evidence matrix

| Criterion | Independent and primary evidence | Result |
| --- | --- | --- |
| Revisions and negotiation | Exact `2025-06-18`, `2025-11-25`, and `2026-07-28` constants/order, pinned initialize fixtures, revision-aware validation, fallback, and atomic rejection tests | pass |
| Raw envelope and mode precedence | Parse/invalid-request precedence, modern-first decode, modern method named `initialize`, operational pre-initialize errors, and unknown-method process oracles | pass |
| Legacy lifecycle | Undetermined, Awaiting, Active, duplicate initialize, initialized notification, silent notification, ping, shutdown/exit-as-unknown, EOF, repetition, and isolation rows | pass |
| Metadata contracts | Open notification `_meta`; request-only string/number progress token; scalar, boolean, array, wrong-known-field, and unexpected-param rejection without invalid state transition | pass |
| Response projection | Revision-correct initialize/list/call/ping results, omitted modern-only legacy fields, preserved modern complete/cache fields, standard and domain errors | pass |
| Catalog and semantics | Exact lexicographic seven-tool catalog, common Tool Policy, semantic success, bounded structured domain errors, startup snapshot, redaction, and path confinement | pass |
| Runtime transport | LF/CRLF, fragmentation, UTF-8, depth/size, stdout/stderr, flush, EOF, cancellation, failures, cleanup, repeated processes, and two-session isolation | pass |
| Codex | Exact pinned binary, two fresh semantic success/domain runs, one ordered seven-tool direct-call run, read-only ephemeral configuration | pass |
| Cursor | Exact pinned ignored binary and project-local config, two fresh `list-tools` runs, exact seven tools and arguments; direct-call limitation stated | pass |
| Existing clients | VS Code compilation, 62 unit tests, 2 Runtime-process tests; LSP 7 process tests; EDT Tycho/PDE 41 tests and Runtime twice | pass |
| Scope, API, dependency, security | Additive connection API and connection-owned Runtime stdio composition; preserved stateless API, public transport signatures, and framing; no new transport, manifest, lockfile, dependency, license, tool, semantic, authentication, global-config, credential, or tracked binary | pass |

## Exact primary validation

The primary reran the complete matrix after the independent review at exact
head `2f809f6d`.

### Focused and canonical Rust gates

```bash
cargo test -p oneagent-protocol --test mcp_session
cargo test -p oneagent-runtime --test mcp_stdio
cargo test -p oneagent-runtime --test mcp_process
cargo test -p oneagent-runtime --test mcp_semantic_tools
cargo test -p oneagent-runtime --test lsp_process
cargo fmt --all --check
cargo check --workspace --all-targets
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo doc --workspace --no-deps
git diff --check
```

Focused results were respectively 12, 8, 16, 6, and 7 passed, with every
negative counter zero. The canonical workspace test produced 72 test-result
targets, 1,140 passed, zero failed/ignored/measured/filtered, and four zero-test
binary targets that are not acceptance filters. Format, check, strict Clippy,
Rustdoc, and diff checks exited zero. The independent reviewer additionally
passed `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps`.

### Exact Codex CLI matrix

Codex CLI `0.150.0-alpha.8`, SHA-256
`4ff5e75f028e913cfeb53bd7319f87573cdce6538c1b1ccc44ce62d5ce51ca1d`,
was invoked with:

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
```

That exact command ran twice. Both runs exited zero, returned Graph
`total=2`/`truncated=true`, returned structured `invalid_arguments` for limit
zero, and ended with `SUCCESS_AND_DOMAIN_ERROR`.

The separate catalog command used the same flags and configuration with this
literal prompt:

```text
Use only the oneagent MCP server. Call each of these seven tools exactly once in this exact order: oneagent.context with {}, oneagent.diagnostics with {}, oneagent.graph with {"limit":1}, oneagent.impact with {}, oneagent.query with {}, oneagent.symbols with {}, and oneagent.validation with {}. After all seven calls, reply with exactly SEVEN_TOOLS_CALLED.
```

It exited zero, emitted seven direct `mcp_tool_call` events in exact order,
returned one Graph success and six bounded structured `invalid_arguments`
results, and ended with `SEVEN_TOOLS_CALLED`. The repository owner explicitly
authorized sending those seven result payloads. Five bounded Codex client-state
database warnings appeared per run; no OneAgent startup or transport failure
occurred.

### Exact Cursor Agent matrix

Cursor Agent `2026.08.25-3e8eec8`, executable SHA-256
`2ccc9a8e167797641448b5e5c936f006ba137a2555f117f38c5eb76a5238a233`,
ran this command twice from the approved ignored repository-local workspace:

```bash
"$ONEAGENT_REPOSITORY/local-artifacts/sprint-35/cursor-agent-2026.08.25-3e8eec8/darwin-arm64/cursor-agent" \
  mcp list-tools oneagent
```

Both runs exited zero and printed `Tools for oneagent (7)` with every exact
name and argument list. This Cursor version has no public non-interactive tool-
call command, so no Cursor direct-call result is claimed.

### Exact VS Code and EDT compatibility

The primary ran the exact VS Code commands recorded in the Task 5 evidence:
four TypeScript compilation/typecheck commands, the Node unit suite, and the
real Runtime integration suite. They exited zero with 62/62 unit and 2/2
integration tests, zero failures, cancellations, skips, or todos.

The exact EDT command was:

```bash
ONEAGENT_REPOSITORY="$(git rev-parse --show-toplevel)"
cd "$ONEAGENT_REPOSITORY/extensions/edt"
ONEAGENT_MCP_EXECUTABLE="$ONEAGENT_REPOSITORY/target/debug/oneagent-mcp" \
ONEAGENT_MCP_FIXTURE="$ONEAGENT_REPOSITORY/apps/runtime/tests/fixtures/workspace_service" \
  ./mvnw --batch-mode --no-transfer-progress clean verify
```

It ran sequentially on the approved host from `extensions/edt`, exited zero
with `BUILD SUCCESS`, passed 41/41 with zero failures/errors/skips, and
exercised the public Runtime twice. The platform-owned shutdown-job warning was
non-fatal.

## Findings and missing evidence

### Blocking

None remain at `2f809f6d`.

### Non-blocking follow-ups

1. Correct the stale final audit wording in
   `docs/architecture/external-ai-client-compatibility-evidence.md` so it lists
   all corrective production boundaries and distinguishes newly added legacy
   projection from unchanged catalog/semantic/dependency contracts.
2. Add one table-driven test for unknown `Undetermined` requests across absent,
   malformed, supported-legacy, and valid-modern metadata.
3. Add named positive rows for explicit empty-object `ping` params and valid
   generic metadata on `tools/list`.

No mandatory product evidence is missing. Raw client traces, binaries, logs,
and disposable workspaces intentionally remain ignored, so the immutable Git
range records their versions, hashes, commands, results, and bounded claims
rather than their bytes. Cursor direct tool calls remain unexecuted because the
pinned public CLI exposes no such command.

## Rejected validation attempts

The following are not acceptance evidence and do not weaken the passing rows:

- two Codex no-call diagnostics that returned `DIFFERENT` and `[]`;
- one fresh Cursor workspace approval rejection before server startup;
- one VS Code integration invocation without `ONEAGENT_MCP_BIN`, which failed
  0/2 before Runtime spawn and was immediately rerun correctly at 2/2;
- one historical timing-sensitive EDT cancellation result at 40/41, followed
  by successful sequential 41/41 runs;
- seven LSP timeouts under an accidental parallel process-heavy run, followed
  by isolated and canonical sequential 7/7 passes;
- an early Clippy `redundant_closure` finding fixed before the accepted gate;
- one host approval rejection before the seven-tool Codex process started,
  followed by explicit user authorization and successful exact execution.

## Scope, security, and cleanup

- No Cargo manifest, lockfile, production dependency, third-party package,
  license inventory, catalog definition, Tool Policy rule, semantic handler,
  VS Code production source, EDT production source, or global client
  configuration changed.
- The public addition is bounded to connection-local revision/lifecycle facts;
  existing `McpServer::dispatch` and Runtime transport signatures remain.
- No new network, filesystem, authentication, secret, background-task, or
  remote-transport authority was introduced.
- No credential, token, personal absolute path, raw trace, generated package,
  executable, archive, or client cache is tracked. Ignored artifacts remain
  under `local-artifacts/sprint-35/`.
- Runtime stdout remains protocol-only; diagnostics remain bounded on stderr;
  EOF, cancellation, failures, repeated processes, and session cleanup are
  closed by tests.
- Deferred scope and Sprint 36 implementation remain untouched.

## Residual risks

- Cursor discovery depends on one previously approved repository-local ignored
  workspace because the public CLI stores approval outside the MCP definition.
- The pinned Cursor CLI cannot independently demonstrate direct success/domain
  calls; Codex direct calls and synthetic public-process tests cover the server
  behavior.
- The non-blocking documentation and table-driven test-hardening items above
  remain explicit follow-ups.

## Next action

After the same reviewer passes artifact consistency, mark Sprint 35
`completed`, make the v0.6 release integration review the next eligible gate,
keep Sprint 36 `planned`, retire exactly the eight tracked Sprint 34 prompt
files, and commit those changes atomically with this review artifact.

## Artifact consistency

The same fresh-context reviewer performed the required read-only consistency
check at `2f809f6dddafa2586bd33ce7d442500c100d1e3c`. Its first pass found two
draft-only inaccuracies: the scope row overstated transport preservation, and
the EDT snippet omitted the repository-root and working-directory commands.
The primary corrected only those two statements. The reviewer then returned
`PASS` with no file/line finding and explicitly approved the exact Roadmap
transition and eight-file retirement inventory. The final artifact preserves
the decision, range, hashes, commit subjects, counts, commands, outcomes,
findings, missing-evidence limits, rejected attempts, residual risks, v0.6
handoff, and Sprint 36 state without weakening the independent report.
