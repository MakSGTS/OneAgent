# Define Sprint 33 AI Chat and Context Panel

Continue OneAgent development.

## Reporting

- Prompt and repository artifacts: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/architecture.md`

## Template

`docs/codex/templates/architecture-task.md`

## Authoritative ADRs / architecture documents

- `docs/architecture/ai-chat-context-panel-investigation.md`
- `docs/architecture/semantic-model-2.md`
- `docs/adr/0044-context-engine.md`
- `docs/adr/0049-tool-execution-policy.md`
- `docs/adr/0050-mcp-server.md`
- `docs/adr/0051-mcp-semantic-tools.md`
- `docs/adr/0052-vscode-extension-foundation.md`
- `docs/adr/0053-navigation-symbol-search.md`
- `docs/Roadmap.md`

## Prerequisites / Required gate

Task 1 is committed and its investigation has no blocking unknown or unapproved
production dependency.

## Task

Create and accept `docs/adr/0055-ai-chat-context-panel.md`.

## Scope

### Included

Define exact pinned platform authority; extension/Runtime/Context/model
ownership; semantic seed and Context state; request/result decoding; operation
serialization; Chat contribution and request lifecycle; deterministic model
messages; context and prompt visibility; bounds; text streaming and
cancellation; error/redaction behavior; read-only panel rendering and CSP;
activation, reconnect, invalidation and disposal; packaging, compatibility,
public evidence, prerequisites, rejected alternatives, and deferred scope.

### Excluded

Production code, new Rust or MCP capability, Runtime LLM integration, secrets,
source reads, implicit context, model tools/edits, scripts in the Context panel,
conversation persistence, remote/web/multi-root/EDT integration, diagnostics UI,
Marketplace, telemetry, and broad quality/performance/security claims.

## Acceptance Criteria

- ADR-0055 records one implementable stable-API contract with exact owners,
  states, fields, bounds, ordering, lifecycle, errors, cancellation, rendering,
  security, compatibility, and evidence gates.
- Every fact sent to a model is explicitly selected, bounded, and inspectable;
  the extension invents no semantic fact and reads no source fallback.
- Existing Runtime catalog, Context semantics, symbol navigation, lifecycle,
  package, and dependency contracts remain compatible.
- Every implementation task has a repository-owned executable oracle.

## Task-specific Validation

- Validate links and pinned sources, ADR numbering/index conventions, consumer
  inventory, accepted/deferred agreement, and Roadmap consistency.
- Run `git diff --check`.

## Suggested commit message

`Define Sprint 33 AI chat and context panel`

## Final report additions

Report accepted platform, ownership, selection/context, message/stream,
panel/security, lifecycle, bounds/error, dependency, compatibility, and
implementation-gate decisions.
