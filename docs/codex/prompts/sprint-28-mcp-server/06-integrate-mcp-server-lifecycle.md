# Integrate MCP Server Lifecycle and Process Composition

Continue OneAgent development.

## Reporting

- Prompt and repository changes: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/runtime-service-implementation.md`

## Template

`docs/codex/templates/runtime-service-task.md`

## Authoritative ADRs and architecture documents

- `docs/Roadmap.md`, Sprint 28 execution plan
- `docs/adr/0050-mcp-server.md`
- `docs/architecture/mcp-server-investigation.md`
- `docs/adr/0037-runtime-service-container.md`
- `docs/adr/0038-http-api-health.md`
- `docs/adr/0040-graph-query-api.md`
- `docs/adr/0043-cli-client.md`
- the exact versioned official stdio specification accepted by ADR-0050

## Prerequisites / Required gate

Task 5 is committed, its complete protocol/transport tests and full workspace
validation succeeded, and the working tree has no conflicting task-created
change.

## Task

Integrate only the ADR-0050 public MCP server process and its accepted Runtime
or process lifecycle composition around the implemented stdio adapter.

## Runtime and service ownership

Use the exact composition root, protocol/transport owner, stdin/stdout/stderr
owner, service/task owner, and shutdown observation accepted by ADR-0050. Keep
all dependencies explicit and retain no global process or I/O state.

## Lifecycle, cancellation, failure, and shutdown

Implement the accepted startup acknowledgement, running observation,
stdin-EOF-driven graceful shutdown, Runtime cancellation, protocol/transport
failure propagation, complete task join, terminal process result, and repeated
fresh-run behavior. Preserve ADR-0037 failure precedence and existing services.

## Scope

### Included

- Public executable or binary mode selected by ADR-0050.
- Real standard-stream binding, structured Runtime/process composition,
  startup/EOF/cancellation/failure/shutdown wiring, diagnostic separation, and
  focused lifecycle/process tests.
- Minimal approved manifest/internal dependency changes not already completed.

### Excluded

New protocol values or dispatch semantics, semantic tools, changes to existing
HTTP health/Graph Query/CLI wires, remote transports, external-client support,
auth, real signals as acceptance evidence, packaging, current-state docs, or
Sprint completion.

## Acceptance Criteria

- The public entry point writes no non-protocol byte to stdout and sends only
  accepted diagnostics to stderr.
- Every standard stream, service/task, channel, cancellation source, and
  process-shutdown signal has one structured owner and terminal rule.
- Successful startup precedes request service; EOF initiates graceful shutdown;
  failures remain distinguishable; all work is joined before exit.
- Existing Runtime HTTP, Workspace, Graph Query, lifecycle, and CLI behavior
  remains unchanged and passes its focused regressions.
- Tests use injected streams/process pipes, explicit synchronization, and
  bounded hang guards rather than arbitrary sleeps, real signals, fixed ports,
  or Unix-only behavior.
- No semantic capability or external-client support is claimed.

## Repository Safety

Enumerate exact Runtime/protocol/process/manifest/test paths before editing.
Preserve `.codex/`, existing API wires, semantic/source/provider crates, docs,
prompt suites, and unrelated files.

## Task-specific Validation

- List and run non-zero MCP lifecycle/process integration tests.
- Run complete `oneagent-protocol` and affected `oneagent-runtime` tests.
- Run existing Runtime service-container, health, Graph Query, Workspace, and
  CLI compatibility targets applicable to changed behavior.
- Audit stdout/stderr writes, task/channel ownership, EOF/cancellation/error
  paths, executable inventory, and deferred features.
- Run the canonical full workspace gate.

## Suggested commit message

`Integrate Sprint 28 MCP server lifecycle`

## Final report additions

Report public entry point, ownership inventory, lifecycle/EOF/failure/cleanup
behavior, compatibility results, tests, exclusions, commit hash, and final Git
state.
