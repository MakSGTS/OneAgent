---
prompt_contract: v2
task_kind: implementation
profile: docs/codex/profiles/refactoring-safe-edits-implementation.md
template: docs/codex/templates/refactoring-safe-edits-task.md
fresh_context: required
context_static_max_percent: 15
context_authorities_max_percent: 20
context_prework_hard_stop_percent: 50
context_working_min_percent: 35
context_reserve_min_percent: 15
---

# Integrate Sprint 40 Product Refactoring Planning

## Reporting

- Communicate with the user in Russian.
- Keep code, APIs, schemas, tests, docs, errors, and the commit message in
  English.

## Context manifest

### Must read

- `AGENTS.md` — sections: repository scope, change discipline, validation, Git
  branch/release workflow, and GUI validation.
- `docs/adr/0063-refactoring-planner.md` — sections: product surface, Tool
  Policy, schema/projection, bounds, errors, lifecycle, compatibility, and
  public-process evidence.
- `docs/Roadmap.md` — sections: Sprint 40 exclusions and Task 6.
- `docs/codex/profiles/refactoring-safe-edits-implementation.md`,
  `docs/codex/templates/refactoring-safe-edits-task.md`, and workflows:
  `refactoring-safe-edits.md`, `runtime-service.md`, `mcp-protocol.md`, and
  `ai-tool-policy.md` — selected and conditional public-boundary contracts.
- committed Task 5 public Workspace planning API and focused tests — exact
  symbols found by bounded `rg`.
- `apps/runtime/src/mcp_tools.rs` — symbols: tool catalog/schema, dispatch,
  immutable snapshot capture, policy evaluation, bounds/errors, and existing
  impact/symbol tool projections.
- `crates/protocol`, `crates/tool-policy`, and `apps/runtime/tests` — exact
  catalog, schema, policy, semantic-tool, stdio, and process consumers found by
  bounded queries for the affected tool/revision.

### Lookup on demand

- VS Code, EDT, LSP, HTTP, and CLI consumers — trigger: the accepted catalog or
  capability change has a direct consumer; query: exact tool name, revision, or
  schema symbol only.
- MCP process fixture implementation — trigger: public-process setup or cleanup
  is not evident from named tests; exact helper functions only.
- prior protocol ADRs — trigger: ADR-0063 cites one compatibility rule that live
  code does not make explicit; matching decision section only.

### Excluded from initial context

- complete clients and unrelated protocol tools;
- generated JS/Java artifacts, downloaded hosts, and successful GUI logs;
- write authorization, edit application, code actions, new UI, and Sprint 41.

### Preflight

- Record effective window or `unknown`, measurement basis, admitted material,
  and `pass|warning|blocked` before implementation.
- Narrow consumer/process selectors at warning and stop at the hard limit.

## Prerequisites / required gate

- `HEAD` is exactly the committed Task 5 result with subject
  `Integrate Sprint 40 Workspace refactoring plans`.
- Workspace focused and compatibility tests pass and the task-owned worktree is
  clean.

## Task

Expose the accepted ADR-0063 read-only product refactoring-plan projection
through existing Runtime, Tool Policy, and MCP boundaries with real process
evidence.

## Scope

### Included

- Accepted catalog/revision behavior, request schema, Tool Policy action and
  effect classification, immutable-call snapshot, projection, preview,
  completeness, summaries, bounds, redacted errors, cancellation/lifecycle,
  repeated calls, process framing, and affected compatibility docs/tests.

### Excluded

- Mutation, confirmation for edits, code actions, model-generated changes,
  HTTP/CLI/LSP/IDE UI, remote transport, authentication, telemetry, and Sprint
  41 transaction APIs.

## Acceptance criteria

- The public surface is truthfully read-only and cannot be used as edit
  authorization; policy and schema agree.
- One request uses one immutable Workspace snapshot and returns deterministic,
  bounded, reconciled, redacted output or one closed error without partial data.
- Catalog, revision, capabilities, existing tools, process framing, shutdown,
  and supported clients remain compatible exactly as ADR-0063 requires.
- Non-zero in-memory, stdio, and real public-process tests cover positive,
  negative, bounds, policy, repetition, and later-publication behavior.

## Task-specific validation

- Run non-zero affected Protocol, Tool Policy, MCP semantic, stdio, and public-
  process targets with no filtered result used as completion evidence.
- Run direct consumer checks required by the accepted compatibility matrix and
  the canonical validation triggered by `docs/codex/core/validation.md`.

## Suggested commit message

`Integrate Sprint 40 product refactoring planning`

## Final report additions

- Report schema/policy agreement, immutable snapshot behavior, bounds/redaction,
  catalog compatibility, public-process evidence, and deferred mutation scope.
