# Complete Tool Execution Policy Evidence

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

The committed Task 5 implementation satisfies the focused ADR-0049 domain,
authorization, confirmation, execution, outcome, and audit contract and leaves
no uncommitted task-created change.

## Task

Complete public repository-owned Tool Execution Policy evidence and synchronize
truthful current-state documentation.

Add a non-zero public conformance target that uses only exported values and a
substitutable fake executor. Prove bounded construction, conservative effects,
canonical policy evaluation, explicit/default denial, confirmation binding and
replay rejection, zero-call denial, one-attempt allow, cancellation, timeout,
failure/partial/completed outcomes, audit ordering, redaction, repetition, and
cleanup. Rerun unchanged LLM, Analysis, and Runtime compatibility targets.
Update only `README.md`, `docs/Architecture.md`, and
`docs/architecture/semantic-model-2.md` to describe the implemented bounded
policy library and its deferrals.

## Scope

### Included

- Public package conformance tests, package-local test support needed by them,
  compatibility evidence, and the three named current-state documents.

### Excluded

- New production behavior beyond evidence-only fixes within task-owned seams;
  Sprint completion, review artifact, Roadmap transition, real tool execution,
  Runtime/MCP/provider/IDE integration, persistence, and later scope.

## Acceptance Criteria

- Public tests are non-zero, use only exported values and repository-owned
  fakes, and perform no external, privileged, credentialed, or destructive work.
- Complete Tool Policy, LLM, Analysis, and Runtime targets pass without live
  state, external network, filesystem/shell mutation, or wall-clock oracle.
- Documentation states only implemented behavior and preserves all accepted
  exclusions, including the absence of concrete tools and Runtime integration.
- Dependency, public-surface, sensitive-state, ignored-test, no-real-effect, and
  documentation-link audits find no unsupported completion claim.
- Sprint 27 remains incomplete until Task 7.

## Repository Safety

Modify only Tool Policy evidence/test-support paths and the three named
documents. Preserve `.codex/`, Roadmap, prompt suites, ADRs, other
implementation, and unrelated files.

## Task-specific Validation

- List and run non-zero Tool Policy unit and public conformance tests.
- Run complete LLM and Analysis tests plus affected Runtime targets.
- Audit direct/reverse dependencies, public API, redaction, ignored tests,
  environment/live inputs, real-side-effect calls, and documentation links.
- Run the canonical full workspace validation.

## Suggested commit message

`Complete Sprint 27 tool policy evidence`

## Final report additions

Report unit/public/compatibility test counts, no-real-effect evidence,
documentation changes, audits, full validation, commit hash, and Git state.
