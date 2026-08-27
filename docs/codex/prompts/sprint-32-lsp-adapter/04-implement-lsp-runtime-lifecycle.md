# Implement Sprint 32 LSP Runtime Lifecycle

Continue OneAgent development.

## Reporting

- Prompt and repository artifacts: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/runtime-service-implementation.md`

## Template

`docs/codex/templates/runtime-service-task.md`

## Authoritative documents

- `docs/adr/0054-lsp-adapter.md`
- `docs/architecture/lsp-adapter-investigation.md`
- `docs/adr/0037-runtime-service-container.md`
- `docs/adr/0039-workspace-service.md`
- `docs/adr/0050-mcp-server.md`

## Prerequisites / Required gate

Task 3 is committed and its focused and full validation passes. The accepted
transport-independent LSP core is available through public APIs.

## Task

Implement the public `oneagent-lsp` stdio process and accepted Runtime lifecycle.

## Scope

### Included

Content-Length byte framing and bounds; injected async reader/writer adapter;
immutable Workspace snapshot construction and root compatibility; dedicated
binary startup; initialize/initialized/shutdown/exit/EOF sequencing; channel
purity; cancellation/failure classification; task/resource ownership; stable
bounded stderr; in-memory and raw public-process tests; repetition and cleanup;
and preservation of `oneagent-runtime` and `oneagent-mcp` entry points.

### Excluded

Semantic navigation/symbol/diagnostic handlers beyond truthful empty/deferred
capabilities, mutable-document sync, file watching/reload, sockets/pipes,
background services, IDE code, MCP changes, external clients, and new
dependencies unless approved.

## Acceptance Criteria

- The public process follows ADR-0054 lifecycle, framing, channel, root,
  failure, exit-status, and cleanup contracts deterministically.
- Every reader, writer, task, snapshot, cancellation source, and process
  resource has one bounded owner; repeated fresh runs leave no orphan.
- In-memory and real-process tests cover exact/over bounds, partial headers/
  bodies, malformed frames, EOF, shutdown/exit, failures, purity, and repetition.
- Existing Runtime/MCP/HTTP/CLI lifecycle behavior is preserved and the full
  Rust gate passes.

## Task-specific Validation

- Run focused Runtime LSP adapter and public `oneagent-lsp` process tests.
- Run existing Runtime service/MCP/CLI regression tests affected by binary and
  composition changes.
- Run the canonical Rust workspace gate and `git diff --check`.

## Suggested commit message

`Implement Sprint 32 LSP runtime lifecycle`

## Final report additions

Report binary/process contract, lifecycle, framing, snapshot/root ownership,
failure/cleanup behavior, public process test counts, compatibility, and
dependency impact.
