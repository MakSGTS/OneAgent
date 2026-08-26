# Implement the Tool Request Domain

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
- `docs/adr/0044-context-engine.md`
- `docs/adr/0045-llm-provider-abstraction.md`
- `docs/architecture/semantic-model-2.md`

## Prerequisites / Required gate

- ADR-0049 is accepted and committed.
- Explicit user approval exists for every new external repository production
  dependency or feature identified by ADR-0049. An additive local std-only
  workspace member does not require separate dependency approval.
- The working tree contains no conflicting task-created change.

## Task

Implement only the ADR-0049 source-independent domain foundation: workspace and
package registration when accepted; bounded tool, actor, request, policy-
revision, and related identities; sensitive bounded argument representation or
summary; conservative side-effect values; immutable validated tool requests;
closed domain error taxonomy; safe formatting; and deterministic focused tests.

Do not implement policy rules or evaluation, confirmation, executor invocation,
terminal audit records, Runtime composition, or concrete tools in this task.

## Scope

### Included

- Exact ADR-0049 domain code, manifest/lock changes, Rustdoc, and unit tests.

### Excluded

- Authorization evaluation, confirmation, execution, public conformance target,
  current-state docs, transports, persistence, and unrelated refactors.

## Acceptance Criteria

- Ownership, dependency graph, public surface, values, bounds, validation
  precedence, equality/ordering, and formatting exactly match ADR-0049.
- Malformed or ambiguous identities, requests, effects, and sensitive inputs
  fail atomically without retaining rejected content.
- Sensitive arguments are available only through explicitly accepted access and
  never appear in implicit Debug/Display/error output.
- Construction performs no I/O, authorization, confirmation, or execution.
- Focused tests are non-zero, deterministic, and cover exact boundaries,
  Unicode/byte cases, malformed combinations, canonical ordering, redaction,
  and repeated construction as applicable.

## Repository Safety

Enumerate exact ADR-0049 task-owned paths before editing. Preserve `.codex/`,
Roadmap, prompt suites, unrelated code/docs, and existing crates except root
workspace/lock registration explicitly accepted by the ADR.

## Task-specific Validation

- List and run non-zero focused domain tests.
- Audit direct/reverse dependencies, public surface, sensitive traits,
  construction side effects, validation precedence, and redaction.
- Run affected LLM, Analysis, and Runtime compatibility tests when the dependency
  audit requires them.
- Run the canonical full workspace validation.

## Suggested commit message

`Implement Sprint 27 tool request domain`

## Final report additions

Report dependency approval evidence, package/public surface, domain values and
bounds, focused test count, full validation, paths, commit hash, and Git state.
