# Define the MCP Server Contract

Continue OneAgent development.

## Reporting

- Prompt and repository changes: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/architecture.md`

## Template

`docs/codex/templates/architecture-task.md`

## Authoritative ADRs and architecture documents

- `docs/Roadmap.md`, Sprint 28 execution plan
- `docs/architecture/mcp-server-investigation.md`
- `docs/adr/0037-runtime-service-container.md`
- `docs/adr/0038-http-api-health.md`
- `docs/adr/0040-graph-query-api.md`
- `docs/adr/0043-cli-client.md`
- `docs/adr/0049-tool-execution-policy.md`
- `docs/architecture/semantic-model-2.md`
- the versioned official MCP sources selected by the investigation

## Prerequisites / Required gate

Task 1 is committed, the investigation proves a complete deterministic oracle,
and no official-source conflict or missing-data blocker remains.

## Task

Create only `docs/adr/0050-mcp-server.md` and mark it `Accepted` only when the
bounded Sprint 28 contract is complete and implementable.

## Scope

### Included

- Protocol/schema authority, revision support, ownership, dependency direction,
  compatibility, and public surface.
- JSON-RPC/MCP message, identifier, metadata, result, notification, error,
  validation precedence, serialization, and resource-bound contracts.
- Truthful discovery/capability and deterministic method registration/dispatch.
- Newline-framed stdio, input/output/diagnostic channel ownership, EOF,
  cancellation, failure, shutdown, and cleanup.
- Runtime/process composition, startup acknowledgement, terminal failure,
  observability, public conformance, and repeated fresh-run evidence.
- Explicit rejected alternatives, implementation prerequisites, and deferred
  semantic tool, transport, version, auth, client, packaging, and IDE scope.

### Excluded

Rust/Cargo implementation, semantic tool schemas, external-client acceptance,
remote transport, real tool effects, current-state support claims, Roadmap
completion, or changes to accepted graph/Runtime/CLI/Context/Tool Policy
behavior.

## Acceptance Criteria

- Every unresolved investigation question has an accepted decision, rejected
  alternative, implementation prerequisite, or explicit deferral.
- The protocol revision and every wire field/error are traceable to the pinned
  official authority without combining incompatible eras.
- Ownership separates protocol/dispatch, transport, Runtime/process, semantic,
  and Tool Policy responsibilities.
- Capabilities cannot advertise deferred methods and notifications cannot
  receive responses.
- Every I/O/task/channel resource and terminal path has one owner and a
  deterministic oracle.
- The first slice requires no live client, credential, external service, real
  signal, platform-specific pipe, or real tool action.
- Only the named ADR changes.

## Repository Safety

Preserve `.codex/`, Roadmap, prompt suites, investigation, Rust/Cargo,
current-state docs, reviews, and unrelated files.

## Task-specific Validation

- Reconcile every ADR decision with the investigation, selected official
  revision/schema, ADR-0037/0038/0040/0043/0049, and Sprint 28 scope.
- Audit normative vocabulary, error precedence, ownership, lifecycle,
  capability truthfulness, testability, alternatives, deferrals, and internal
  links.
- Run `git diff --check`.

## Suggested commit message

`Define Sprint 28 MCP server`

## Final report additions

Report protocol revision, accepted ownership/lifecycle/wire decisions, rejected
alternatives, implementation prerequisites, deferred scope, validation, commit
hash, and final Git state.
