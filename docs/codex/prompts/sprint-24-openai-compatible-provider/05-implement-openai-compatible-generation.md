# Implement Sprint 24 OpenAI-Compatible Text Generation

Continue OneAgent development.

## Reporting

- Prompt and repository content: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/llm-provider-implementation.md`

## Template

`docs/codex/templates/llm-provider-task.md`

## Authoritative documents

- `docs/Roadmap.md`, Sprint 24 execution plan
- `docs/architecture/openai-compatible-provider-investigation.md`
- `docs/adr/0045-llm-provider-abstraction.md`
- `docs/adr/0046-openai-compatible-provider.md`
- committed Tasks 3-4 adapter foundation and discovery

## Prerequisites / Required gate

Require committed Task 4, successful discovery/package/workspace validation,
and clean task-owned state. Stop rather than weakening provider-neutral bounds
or accepted transport policy.

## Task

Implement only one non-streaming OpenAI-compatible text-generation attempt
through `POST /v1/completions` and map it to the originating validated
`TextGenerationRequest`.

## Request, response, usage, finish, and compatibility contract

Send only the accepted `model`, `prompt`, `max_tokens`, and `stream=false`
fields in deterministic JSON. Preserve exact input bytes and provider-scoped
model identity. Require the accepted top-level object, exact model, one valid
choice/index, non-empty locally bounded text, and only `stop` or `length` finish
mapping. Ignore provider token usage and accepted unknown extension fields; do
not introduce them into shared-domain authority.

Reject missing, empty, partial, duplicate/multiple, wrong-index, wrong-model,
unknown-finish, malformed, oversized, and provider-fallback responses exactly
as ADR-0046. The live single-model unknown-request fallback case must be
represented by deterministic loopback evidence.

## Timeout, retry, cancellation, and error contract

Apply accepted authentication, body/status/transport/protocol mapping, total
timeout, cancellation before/during headers/body, terminal precedence, and
cleanup. Perform exactly one attempt under `RetryPolicy::Never`; no replay,
redirect, fallback, stream, detached task, or partial response may survive.
Never expose credential, URL secret, prompt, completion, or provider body.

## Scope

### Included

Adapter generation implementation, focused exact-wire controlled-loopback
tests, public provider substitution evidence assigned here, and minimum Rustdoc.

### Excluded

Chat/Responses APIs, streaming/SSE, tools, structured output, roles/messages,
prompt policy, sampling controls, tokenizer or token usage authority, automatic
retry, Runtime/Context/protocol/CLI changes, live services, docs synchronization,
sprint transition, and prompt retirement.

## Acceptance Criteria

- `LlmProvider::generate` rejects provider mismatch and existing cancellation
  before I/O, performs one exact request, and returns one validated terminal
  response or typed error.
- Finish, model identity, output bytes, timeout/cancellation, body/status/error,
  redaction, and cleanup match ADR-0046 exactly.
- Controlled-loopback tests cover positive, length, malformed, partial,
  multiple, identity fallback, unknown finish, bound, status, timeout,
  cancellation, redaction, cleanup, and repeated cases without external network
  or arbitrary sleep.
- The complete adapter and workspace gates pass.

## Repository Safety

Modify only the adapter crate files and focused tests required for generation.
Preserve manifests unless mechanically required by an approved dev dependency,
provider-neutral APIs, other crates/adapters, Runtime, docs, prompts, ignored
artifacts, `.codex/`, and unrelated paths. Do not access external services.

## Task-specific Validation

- List and run non-zero exact-wire generation/finish/identity/bound/status/
  timeout/cancellation/redaction/cleanup/repetition tests.
- Run complete adapter and provider-neutral package tests.
- Run the canonical complete workspace validation.
- Verify diff scope and `git status --short`.

## Suggested commit message

`Implement Sprint 24 text generation`

## Final report additions

Report exact request/response mapping, byte/token treatment, identity/fallback,
finish/usage behavior, errors/redaction, timeout/cancellation/cleanup, loopback
matrix, focused/full validation, changed paths, commit, and final Git state.
