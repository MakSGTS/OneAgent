# Implement Confirmation-Gated Tool Execution

Continue OneAgent development.

## Reporting

- Prompt and repository changes: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/ai-tool-policy-implementation.md`

## Template

`docs/codex/templates/ai-tool-policy-task.md`

## Authoritative ADRs and architecture documents

- `docs/adr/0049-tool-execution-policy.md`
- `docs/architecture/tool-execution-policy-investigation.md`
- `docs/adr/0037-runtime-service-container.md`
- `docs/architecture/semantic-model-2.md`

## Prerequisites / Required gate

The committed Task 4 authorization implementation exactly matches ADR-0049 and
leaves no uncommitted task-created change.

## Task

Implement only the ADR-0049 confirmation and execution boundary. Bind accepted
confirmation evidence to the exact current request and decision, reject missing,
mismatched, stale, duplicated, or replayed confirmation before execution, and
invoke the substitutable executor exactly once only after an allow decision or
valid required confirmation.

Map existing cancellation, accepted timeout behavior, executor success,
failure, partial outcome, invalid response, and cleanup into one bounded typed
terminal result with deterministic redacted audit correlation and ordering.
Prove through repository-owned fakes that denied or unconfirmed requests never
invoke execution, one decision cannot authorize another request, retry/fallback
does not occur, and no active work survives a terminal result.

## Scope

### Included

- Confirmation values/gate, executor seam, terminal outcome and audit evidence,
  local exports, Rustdoc, deterministic fakes, and focused tests.

### Excluded

- Concrete tools, real side effects, automatic retry/fallback, rollback,
  persistence, audit sink/export, Runtime lifecycle, transport, MCP/provider/IDE
  mapping, public conformance target, and current-state docs.

## Acceptance Criteria

- Deny/default-deny and missing/invalid confirmation paths produce zero executor
  calls and one inspectable terminal outcome.
- Allow and exactly confirmed paths perform at most one accepted attempt; stale,
  mismatched, duplicate, or replayed evidence cannot authorize work.
- Cancellation, timeout, failure, partial, invalid, and completed outcomes follow
  exact ADR precedence, are bounded/redacted, and leave no retained active work.
- Audit evidence correlates request, policy revision, decision, confirmation
  state, attempt, and result without retaining unrestricted arguments/output.
- Tests are non-zero, deterministic, clock/network/filesystem independent, and
  perform no real external or destructive action.

## Repository Safety

Modify only confirmation/execution/audit-owned files, necessary local exports,
and package-local focused tests. Preserve `.codex/`, Roadmap, prompts,
manifests/lockfile, other crates, docs, and unrelated files.

## Task-specific Validation

- List and run non-zero confirmation, execution, cancellation, failure,
  redaction, audit, repetition, and cleanup tests.
- Run complete domain and authorization regressions.
- Audit zero-invocation denial, exact binding, one attempt, no retry/fallback,
  terminal completeness, sensitive formatting, and no-real-effect behavior.
- Run the canonical full workspace validation.

## Suggested commit message

`Implement Sprint 27 confirmed execution`

## Final report additions

Report confirmation binding, executor attempt/no-invocation evidence, outcome
and audit matrix, focused counts, validation, paths, commit hash, and Git state.
