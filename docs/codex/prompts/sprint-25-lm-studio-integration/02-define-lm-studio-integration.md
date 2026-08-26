# Define Sprint 25 LM Studio Integration

Continue OneAgent development.

## Reporting

- Prompt and repository content: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/architecture.md`

## Template

`docs/codex/templates/architecture-task.md`

## Authoritative documents

- `docs/Roadmap.md`, Sprint 25 execution plan
- `docs/adr/0045-llm-provider-abstraction.md`
- `docs/adr/0046-openai-compatible-provider.md`
- `docs/architecture/lm-studio-integration-investigation.md`
- `docs/architecture/openai-compatible-provider-investigation.md`
- `docs/reviews/sprint-24-openai-compatible-provider.md`
- `docs/codex/workflows/llm-provider.md`

## Prerequisites / Required gate

Require committed Task 1 evidence, clean task-owned state, and no unresolved
data blocker in the investigation. Do not make another live LM Studio call
unless the current user instruction authorizes it.

## Task

Create `docs/adr/0047-lm-studio-integration.md` and accept the smallest complete
LM Studio provider contract behind ADR-0045. Select the implementation boundary
only from Task 1 evidence: compose the existing adapter, introduce a minimal
reusable seam without weakening ADR-0046, or create an independent leaf adapter.

## Scope

### Included

- Canonical ownership, dependency direction, public provider type and stable
  provider ID.
- Exact direct dependency/version/feature set and an explicit approval gate for
  every addition beyond the already approved ADR-0046 graph.
- Public construction, explicit/default server root, locality, HTTP/HTTPS,
  authentication, URL normalization, redirect, proxy, TLS, and redaction rules.
- Exact model-discovery endpoint, required/optional/ignored fields, LLM versus
  embedding mapping, loaded/downloaded behavior, identity, canonical ordering,
  duplicate/ambiguous/unknown handling, and body bounds.
- Exact non-streaming generation endpoint and request/response fields compatible
  with the existing raw text request, including output-bound interpretation,
  identity, finish, status, protocol, and local byte usage.
- Provider mismatch, existing/in-flight cancellation, total timeout, one
  attempt, no retry/fallback, simultaneous-ready precedence, and cleanup.
- Generic OpenAI-compatible compatibility and regression requirements.
- Controlled-loopback fixture and public conformance matrix without live LM
  Studio, local models, credentials, external network, sleeps, or quality
  assertions.
- Implementation order, documentation completion, and deferred scope.

### Excluded

- Rust, Cargo, tests, fixtures, or production implementation.
- LM Studio installation, daemon/GUI/server/model lifecycle, download, load,
  unload, JIT, TTL, and auto-evict.
- Chat history, stateful chat, roles/messages, Responses API, Anthropic API,
  streaming, tools, MCP, structured output, reasoning, vision, embeddings, and
  shared provider metadata.
- Prompt policy, model selection, aliases, retry/backoff, cache/refresh,
  registry, Runtime registration, configuration sources, protocol/CLI, MCP,
  LSP, IDE, UI, graph, and Coverage changes.
- Sprint state transition, review artifact, and prompt retirement.

## Acceptance Criteria

- ADR-0047 has status `Accepted` and explicitly preserves ADR-0045 and ADR-0046.
- One implementation boundary is selected from recorded evidence, with rejected
  alternatives and compatibility consequences documented.
- The discovery contract cannot advertise an embedding-only model as
  `TextGeneration` and defines every malformed/ambiguous terminal outcome.
- The generation mapping is compatible with ADR-0045 without silently importing
  chat, prompt-template, state, tool, or provider-extension semantics.
- URL/locality/authentication and sensitive-data boundaries are exact and
  deterministic.
- Dependency additions/features and their user-approval gates are exact.
- Every success and failure path has an observable repository-owned oracle.
- Implementation prerequisites and Tasks 3-6 ordering are executable.
- Unknown or unsupported behavior remains deferred rather than guessed.

## Repository Safety

Create only ADR-0047. Preserve investigation evidence, Rust, Cargo, tests,
prompts, Roadmap, `.codex/`, live LM Studio state, and unrelated paths.

## Task-specific Validation

- Validate ADR status, decisions, rejected alternatives, dependency approval,
  implementation order, deferred scope, links, and agreement with ADR-0045,
  ADR-0046, Task 1 evidence, and the Roadmap plan.
- `git diff --check`
- Verify diff scope and `git status --short`.

## Required commit

After every task-specific validation command succeeds, stage only the exact
task-owned paths, create one commit with the exact message below, verify its
paths and resulting `HEAD`, and continue only from clean task-owned state:

`Define Sprint 25 LM Studio integration`

Do not commit after failed validation or when unrelated changes cannot be
excluded.

## Final report additions

Report the accepted ownership/composition, discovery and generation contracts,
dependency approval gate, rejected alternatives, deferred scope, implementation
prerequisites, validation, commit, and final Git state.
