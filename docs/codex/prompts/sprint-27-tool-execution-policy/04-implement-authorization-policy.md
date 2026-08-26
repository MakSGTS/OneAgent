# Implement Deterministic Tool Authorization

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
- `docs/architecture/semantic-model-2.md`

## Prerequisites / Required gate

The committed Task 3 domain exactly matches ADR-0049 and leaves no uncommitted
task-created change.

## Task

Implement only ADR-0049 policy construction and evaluation. Validate and
canonicalize the accepted actor/tool/effect/scope rule vocabulary, evaluate
every request with exact conflict and decision precedence, default to deny when
no rule applies or input is untrustworthy, and return a stable request- and
policy-revision-bound decision.

Cover explicit allow, explicit deny, confirmation-required, default deny,
exact/wildcard scope where accepted, multiple matches, conflicts, duplicate and
reordered rules, unknown or malformed values, effect conservatism, policy
revision changes, repeated evaluation, and redacted diagnostics. Never invoke an
executor or treat prior decisions/audit evidence as authorization.

## Scope

### Included

- Policy/rule/decision implementation, local exports, Rustdoc, and focused tests.

### Excluded

- Confirmation construction or consumption, executor invocation, cancellation,
  terminal outcomes/audit, persistence/configuration, public conformance, docs,
  Runtime, MCP, provider mapping, and concrete tools.

## Acceptance Criteria

- Accepted rules canonicalize deterministically and caller order cannot weaken
  conflict/deny precedence.
- Missing, unknown, malformed, ambiguous, or unmatched policy input returns the
  exact fail-closed result with no partial authorization.
- Decisions bind to exact accepted request evidence and policy revision and
  expose no sensitive arguments through formatting.
- Evaluation has no I/O, clock, global state, executor call, or side effect.
- Focused tests are non-zero and prove the full accepted decision matrix,
  reordering, duplication, revision changes, redaction, and repetition.

## Repository Safety

Modify only authorization-owned files and necessary local exports inside the
accepted package. Preserve `.codex/`, Roadmap, prompt suites, manifests/lockfile,
other crates, current-state docs, and unrelated files.

## Task-specific Validation

- List and run non-zero focused authorization tests.
- Run complete domain regressions.
- Audit precedence, default deny, no-execution, decision binding, canonical
  ordering, sensitive formatting, and repeated evaluation.
- Run the canonical full workspace validation.

## Suggested commit message

`Implement Sprint 27 authorization policy`

## Final report additions

Report the rule and decision matrix, default-deny/precedence evidence, focused
test count, validation, paths, commit hash, and final Git state.
