# Complete Ollama Provider Evidence

Continue OneAgent development.

## Reporting

- Prompt and repository changes: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/llm-provider-implementation.md`

## Template

`docs/codex/templates/llm-provider-task.md`

## Authoritative ADRs and architecture documents

- `docs/adr/0048-ollama-integration.md`
- `docs/adr/0045-llm-provider-abstraction.md`
- `docs/adr/0046-openai-compatible-provider.md`
- `docs/adr/0047-lm-studio-integration.md`
- `docs/architecture/ollama-integration-investigation.md`
- `docs/architecture/semantic-model-2.md`

## Prerequisites / Required gate

The committed Task 5 implementation satisfies the focused ADR-0048 discovery
and generation contract and leaves no uncommitted task-created change.

## Task

Complete public repository-owned Ollama provider evidence and synchronize
truthful current-state documentation.

Add a non-zero public conformance target that uses only exported provider-neutral
values and `&dyn LlmProvider` against controlled loopback. Prove construction,
fresh capability-aware discovery, terminal generation, typed failures, bounds,
redaction, timeout, cancellation, one attempt, repeated calls, and cleanup.
Rerun complete provider-neutral and existing concrete-provider regressions plus
Analysis and Runtime compatibility. Update only `README.md`,
`docs/Architecture.md`, and `docs/architecture/semantic-model-2.md` to describe
the implemented bounded leaf and its deferrals.

## Scope

### Included

- Public Ollama conformance tests, local test-support exposure needed by those
  tests, compatibility evidence, and the three named current-state documents.

### Excluded

- Implementation behavior changes beyond evidence-only fixes within task-owned
  test seams; Sprint completion, review artifact, Roadmap transition, live
  provider calls, model/cloud dependency, Runtime registration, and later scope.

## Acceptance Criteria

- Public tests are non-zero, controlled-loopback only, and exercise the provider
  through `&dyn LlmProvider` with exported provider-neutral values.
- Complete Ollama, OpenAI-compatible, LM Studio, provider-neutral, Analysis, and
  Runtime targets pass without live state or external network.
- Documentation states only implemented behavior and preserves all accepted
  exclusions.
- Dependency, public-surface, sensitive-state, ignored-test, and no-live-state
  audits find no unsupported completion claim.
- Sprint 26 remains incomplete until Task 7.

## Repository Safety

Modify only Ollama evidence/test-support paths and the three named documents.
Preserve `.codex/`, Roadmap, prompt suites, ADRs, other implementation, and
unrelated files.

## Task-specific Validation

- List and run non-zero Ollama unit and public conformance tests.
- Run complete OpenAI-compatible, LM Studio, and provider-neutral tests.
- Run affected Analysis and Runtime tests.
- Audit direct/reverse dependencies, public API, redaction, ignored tests,
  environment inputs, live-state references, and documentation links.
- Run the canonical full workspace validation.

## Suggested commit message

`Complete Sprint 26 Ollama evidence`

## Final report additions

Report unit/public/regression test counts, compatibility evidence, documentation
changes, audits, full validation, commit hash, and final Git state.
