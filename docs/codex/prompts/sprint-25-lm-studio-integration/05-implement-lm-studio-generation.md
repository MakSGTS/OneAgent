# Implement Sprint 25 LM Studio Text Generation

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

Require committed Task 4 discovery, clean task-owned state, and passing LM
Studio discovery plus existing provider regression targets.

## Task

Implement only one ADR-0047 non-streaming LM Studio text-generation attempt for
one validated `TextGenerationRequest`. Preserve exact provider/model identity,
input and output bounds, terminal finish mapping, local byte usage, redacted
typed failures, total timeout, cooperative cancellation, exactly one attempt,
and complete cleanup.

## Provider-neutral ownership and API boundary

Use ADR-0045 request/response/error/execution types unchanged. Provider-specific
wire values remain private. Do not import LM Studio chat, model-management,
reasoning, tool, usage, stats, or metadata types into `oneagent-llm`.

## Model identity, discovery, and capability contract

Reject a request whose provider is not `lm-studio` before I/O. Preserve the
exact model selected from an accepted text-capable descriptor and require the
accepted response identity contract. Do not perform implicit discovery,
loading, aliasing, model selection, or fallback during generation.

## Request, response, usage, finish, and compatibility contract

- Serialize exactly the ADR-0047 non-streaming request fields and preserve the
  validated input and exact model ID.
- Treat output-token fields only as the conservative wire ceiling accepted by
  ADR-0047; `max_output_bytes` and local `TextGenerationResponse` validation
  remain authoritative.
- Accept only the exact response object, choice/message/text, index, model, and
  finish values selected by ADR-0047.
- Map only accepted normal/output-limit finishes to ADR-0045 and reject empty,
  over-bound, mismatched, partial, multiple, unknown, null, or malformed
  terminal values atomically.
- Ignore provider IDs, usage tokens, stats, timing, reasoning, and unknown
  additions unless ADR-0047 explicitly requires rejection.

## Configuration and secret-handling contract

Use the foundation's exact endpoint and optional authentication. Request input,
output, URLs, headers, bodies, DTOs, provider errors, and synthetic sentinels
must not appear in implicit formatting, diagnostics, snapshots, or retained
sources.

## Timeout, retry, cancellation, and cleanup contract

Apply provider mismatch then pre-cancellation precedence, start exactly one
operation, race it against in-flight cancellation and the total timer with
ADR-0047 ordering, perform no retry/fallback/stream/background task, and drop
all losing futures and temporary state before returning.

## Error taxonomy and provider mapping

Implement exact request serialization/bound, status, redirect, transport,
protocol, identity, invalid-response, timeout, cancellation, and internal
mappings. Never parse or retain an unbounded provider error body.

## Contract oracle

Use controlled `127.0.0.1:0` servers and synthetic wires for exact request,
Unicode, maximum escaping, normal/output-limit finishes, unknown additions,
provider usage/stats ignore, response-model mismatch, zero/multiple choices,
index, finish, empty/over-output, malformed/missing/mistyped/trailing/partial
JSON, request/response body bounds, status, redirect, transport, timeout,
cancellation, auth/redaction, exactly one request, cleanup, and fresh repeated
generation.

## Consumer and provider-adapter compatibility

Preserve LM Studio discovery and the complete generic adapter behavior. Run
provider-neutral request/response tests. No Context, Runtime, protocol, CLI,
graph, or source behavior changes.

## Scope

### Included

- LM Studio generation production code and focused controlled-loopback tests.
- Minimal accepted helper adjustments needed for generation.

### Excluded

- Chat/history/state, streaming, tools, MCP, model lifecycle, live LM Studio,
  public Sprint 25 conformance target, docs synchronization, Runtime, prompt
  policy, retry/fallback, and deferred features.

## Acceptance Criteria

- One exact bounded non-streaming attempt succeeds through `&dyn LlmProvider`.
- Identity, output, usage, and finish semantics match ADR-0045/0047.
- Every malformed, mismatch, bound, status, transport, timeout, and cancellation
  case is terminal, typed, redacted, and atomic.
- Exactly one request occurs and no operation state survives success or failure.
- LM Studio discovery, generic adapter, and provider-neutral regressions pass.

## Repository Safety

State exact files before editing. Preserve prompts, Roadmap, docs outside task
scope, `.codex/`, unrelated code, and local LM Studio state. Do not contact a
live provider.

## Task-specific Validation

- Non-zero focused LM Studio generation tests and test listing.
- Non-zero focused LM Studio discovery regression tests.
- `cargo test -p oneagent-openai-compatible --lib --offline`
- `cargo test -p oneagent-openai-compatible --test conformance --offline`
- `cargo test -p oneagent-llm --offline`
- Exact dependency/redaction audit.
- Canonical full workspace validation from `docs/codex/core/validation.md`.

## Required commit

After every task-specific validation command succeeds, stage only the exact
task-owned paths, create one commit with the exact message below, verify its
paths and resulting `HEAD`, and continue only from clean task-owned state:

`Implement Sprint 25 LM Studio generation`

Do not commit after failed validation or when unrelated changes cannot be
excluded.

## Final report additions

Report exact generation wire/mapping, identity/finish/output behavior, error and
redaction evidence, timeout/cancellation/cleanup, adapter compatibility,
focused/full validation, commit, and final Git state.
