# Define Sprint 23 LLM Provider Abstraction

Continue OneAgent development.

## Reporting

- Prompt and repository content: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/architecture.md`

## Template

`docs/codex/templates/architecture-task.md`

## Authoritative documents

- `docs/Roadmap.md`, Sprint 23 execution plan
- `docs/architecture/llm-provider-investigation.md`
- `docs/Architecture.md`
- `docs/architecture/semantic-model-2.md`
- `docs/adr/0044-context-engine.md`
- `docs/reviews/sprint-22-context-engine.md`

## Prerequisites / Required gate

Require committed Task 1 evidence that every first-slice decision has a
repository-owned input and deterministic oracle. Stop if the investigation
reports missing or conflicting evidence.

## Task

Create and accept `docs/adr/0045-llm-provider-abstraction.md`, defining the
smallest complete provider-independent LLM contract. Synchronize only planning-
level architecture text required to make the decision unambiguous. Implement no
Rust.

## Scope

### Included

- Crate and ownership boundary, dependency direction, provider construction,
  request lifetime, async object-safety strategy, and Context/Runtime separation.
- Provider and model identity, deterministic model discovery projection,
  capability vocabulary, ordering, compatibility, refresh/caching absence, and
  unsupported-provider-extension behavior.
- Closed text-only request input, bounds, validation and precedence, response,
  usage, finish, partial/empty/malformed behavior, and provider-neutral errors.
- Secret-bearing configuration value behavior, redaction, Debug/Display/clone/
  serialization restrictions, and diagnostic size/content policy.
- Provider execution and cancellation seam; explicit timeout and retry policy,
  classifications, replay safety, attempt accounting, and whether orchestration
  is implemented, represented only, disabled, or deferred in the first slice.
- Repository-owned fake/conformance corpus, exact oracles, repeatability,
  compatibility, dependency choice, public test strategy, first slice, rejected
  alternatives, implementation order, and deferred scope.

### Excluded

Rust/Cargo/fixture changes, concrete OpenAI-compatible/LM Studio/Ollama schemas
or adapters, HTTP/JSON/SSE, live discovery/execution, credentials or environment
loading, tokenizers/token counting, streaming, tools, structured output, images,
audio, conversations, prompt policy/templates, Context assembly changes,
Runtime/HTTP/CLI/protocol exposure, persistence, MCP/IDE, automatic retry or
clock implementation without evidence, new production dependencies without
approval, sprint completion, and prompt retirement.

## Acceptance Criteria

- ADR-0045 answers every Task 1 decision question with one canonical contract
  grounded in live evidence and accepted architecture.
- Shared domain types and execution seam are independent from provider wire
  schemas, Runtime transport, and concrete provider construction.
- Identity, discovery, capability, request, response, usage, finish, error,
  validation, ordering, and compatibility vocabularies are closed and observable.
- Secret values cannot leak through accepted formatting, serialization,
  diagnostics, fixtures, or source-control paths.
- Timeout, retry, cancellation, replay, attempt, partial-result, cleanup, and
  terminal precedence are explicit; unsupported behavior is not implied.
- Public fake/conformance evidence has exact repository-owned oracles covering
  positive, negative, boundary, incompatible, error, cancellation, reordered,
  and repeated cases without live services or credentials.
- Dependency and crate choices are explicit. If a new external production
  dependency is required, Task 3 remains gated on separate user approval.
- Rejected alternatives, compatibility, first slice, implementation order,
  Coverage impact, Sprint 24 hand-off, and later deferrals are explicit. Sprint
  23 remains `next`; current-state docs do not claim implementation.

## Repository Safety

Create only `docs/adr/0045-llm-provider-abstraction.md` and modify only the
minimum planning-level architecture document if required. Preserve `.codex/`,
Rust, manifests, lockfile, fixtures, prompts, Roadmap state, current-state
implementation claims, credentials, and unrelated files. Stage only ADR-owned
paths when authorized.

## Task-specific Validation

- Verify decision/evidence consistency with Task 1 and cited public contracts.
- Validate internal links, ADR status, closed ownership/identity/discovery/
  capability/request/response/usage/finish/secret/error/execution/policy/
  evidence matrices, alternatives, prerequisites, accepted/deferred scope, and
  `git diff --check`.
- `git status --short`

## Suggested commit message

`Define Sprint 23 LLM Provider abstraction`

## Final report additions

Report accepted ownership, identity/discovery/capability, request/response,
usage/finish, configuration/secret, error, async execution, timeout/retry/
cancellation, conformance, compatibility, dependency, first-slice, and deferred
decisions; changed paths; validation; commit; Git state; and whether Task 3 is
unblocked.
