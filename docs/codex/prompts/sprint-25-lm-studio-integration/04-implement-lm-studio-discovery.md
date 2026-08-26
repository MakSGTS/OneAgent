# Implement Sprint 25 LM Studio Model Discovery

Continue OneAgent development.

## Reporting

- Prompt and repository content: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/llm-provider-implementation.md`

## Template

`docs/codex/templates/llm-provider-task.md`

## Authoritative documents

- `docs/Roadmap.md`, Sprint 25 execution plan
- `docs/adr/0045-llm-provider-abstraction.md`
- `docs/adr/0046-openai-compatible-provider.md`
- `docs/adr/0047-lm-studio-integration.md`
- `docs/architecture/lm-studio-integration-investigation.md`

## Prerequisites / Required gate

Require committed Task 3 foundation, clean task-owned state, exact approved
dependencies, and passing foundation plus existing generic-adapter regressions.

## Task

Implement only fresh LM Studio model discovery through the exact ADR-0047
endpoint and wire mapping. Distinguish model type before assigning
`TextGeneration`, construct one canonical provider-scoped `ModelCatalog`, and
apply the accepted bounds, error, authentication, timeout, cancellation,
redaction, one-attempt, and cleanup policy.

## Provider-neutral ownership and API boundary

Use existing ADR-0045 identities, `ModelCapability`, `ModelDescriptor`,
`ModelCatalog`, errors, execution context, and `LlmProvider` seam without adding
LM Studio wire fields to the shared domain.

## Model identity, discovery, and capability contract

- Send exactly one fresh accepted discovery request per call.
- Preserve exact valid LM Studio model keys as `ModelId` values under provider
  `lm-studio`.
- Assign `TextGeneration` only to entries meeting ADR-0047's exact accepted LLM
  and loaded/downloaded criteria.
- Never advertise embedding, unknown, malformed, or ambiguous entries as text
  generation models.
- Apply exact empty, maximum, over-count, duplicate, reordered, unknown-field,
  variant/instance, missing, mistyped, partial, body-bound, and canonical-order
  behavior.
- Do not retain display name, publisher, architecture, quantization, context,
  reasoning, vision, tool, path, or other provider metadata in shared values.

## Request, response, usage, finish, and compatibility contract

This task performs no generation and changes no provider-neutral request,
response, usage, or finish semantics.

## Configuration and secret-handling contract

Use the Task 3 client and exact optional authentication. Captured tests use only
synthetic secrets. Public errors never retain or format URLs, headers, bodies,
model payloads, provider error text, or sentinels.

## Timeout, retry, cancellation, and cleanup contract

Reject pre-cancellation before discovery work, race exactly one request against
the total timeout and in-flight cancellation with ADR-0047 precedence, perform
no retry/fallback/cache/refresh, and release every request, response, buffer,
DTO, and helper future before returning.

## Error taxonomy and provider mapping

Implement the exact accepted status, transport, protocol, invalid-catalog,
timeout, cancellation, and internal mappings. Reject the whole catalog
atomically on a terminal error.

## Contract oracle

Use controlled `127.0.0.1:0` servers and bounded synthetic fixtures. Cover exact
method/path/headers, positive LLM plus embedding filtering, empty/maximum,
reordered, unknown additions, unloaded/loaded criteria, multiple instances,
duplicates, invalid IDs, missing/mistyped/type ambiguity, malformed/trailing/
partial JSON, status, redirect, advertised/streamed body bounds, transport,
timeout, cancellation, authentication/redaction, one request, cleanup, and
fresh repeated calls.

## Consumer and provider-adapter compatibility

Preserve existing generic `/v1/models` behavior exactly. Run provider-neutral
catalog tests and the complete generic adapter unit/public conformance targets.
No Context, Runtime, protocol, or CLI behavior changes.

## Scope

### Included

- LM Studio discovery production code and focused controlled-loopback tests.
- Minimal foundation adjustments proven necessary by the accepted ADR.

### Excluded

- Generation, server/model lifecycle, cache/refresh, live LM Studio, public
  Sprint 25 conformance target, docs synchronization, and deferred features.

## Acceptance Criteria

- Discovery is fresh, bounded, deterministic, provider-scoped, and canonical.
- Mixed LLM/embedding input yields only the exact accepted text-capable LLM
  descriptors; no embedding false positive is possible.
- Every malformed, ambiguous, bound, status, timeout, cancellation, and
  transport path returns the exact typed terminal result with no partial catalog.
- Exactly one request occurs and no operation state survives any terminal path.
- Generic adapter and provider-neutral behavior remain unchanged.

## Repository Safety

State exact files before editing. Preserve prompts, Roadmap, docs outside task
scope, `.codex/`, unrelated code, and local LM Studio state. Do not contact a
live provider.

## Task-specific Validation

- Non-zero focused LM Studio discovery tests and test listing.
- `cargo test -p oneagent-openai-compatible --lib --offline`
- `cargo test -p oneagent-openai-compatible --test conformance --offline`
- `cargo test -p oneagent-llm --offline`
- Exact dependency/redaction audit.
- Canonical full workspace validation from `docs/codex/core/validation.md`.

## Required commit

After every task-specific validation command succeeds, stage only the exact
task-owned paths, create one commit with the exact message below, verify its
paths and resulting `HEAD`, and continue only from clean task-owned state:

`Implement Sprint 25 LM Studio discovery`

Do not commit after failed validation or when unrelated changes cannot be
excluded.

## Final report additions

Report discovery wire/mapping, embedding exclusion, canonical/error behavior,
timeout/cancellation/cleanup, generic-adapter compatibility, focused/full
validation, commit, and final Git state.
