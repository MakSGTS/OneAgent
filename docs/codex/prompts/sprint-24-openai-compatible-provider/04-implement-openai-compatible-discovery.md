# Implement Sprint 24 OpenAI-Compatible Model Discovery

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
- committed Task 3 client foundation

## Prerequisites / Required gate

Require committed Task 3, exact approved dependency tree, successful adapter
package validation, and clean task-owned state.

## Task

Implement only fresh OpenAI-compatible model discovery through
`GET /v1/models` and strict mapping to the provider-neutral canonical catalog.

## Model identity, discovery, and capability contract

Map only the accepted list object and `data[].id` fields. Scope every model to
the configured provider, advertise only `TextGeneration`, canonicalize order,
and enforce empty, maximum, duplicate, invalid ID, malformed, missing,
reordered, unknown-field, and provider-bound behavior exactly as ADR-0046.

## Configuration, execution, and error contract

Apply accepted base URL, optional bearer header, body limit, status mapping,
transport/protocol distinction, total timeout, already/in-flight cancellation,
terminal precedence, and cleanup. Never retain provider bodies or sensitive
request/response data in diagnostics. Perform exactly one request and no retry,
redirect, cache, fallback, or background refresh.

## Contract oracle

Use controlled loopback endpoints with exact method/path/header/body/status and
response oracles. Cover success, empty, maximum/over-limit, duplicate,
reordered, malformed JSON, wrong top-level shape, missing/invalid IDs, unknown
fields, oversized body/content length, statuses, timeout, cancellation,
redaction, cleanup, and repeated calls as applicable.

## Scope

### Included

Adapter discovery implementation, focused unit/integration loopback tests, and
minimum public exports or Rustdoc accepted by ADR-0046.

### Excluded

Generation, prompt mapping, Runtime/Context/protocol/CLI changes, catalog cache,
refresh, model selection, live services, chat/Responses APIs, streaming, tools,
retry, current-state docs, sprint transition, and prompt retirement.

## Acceptance Criteria

- `LlmProvider::discover_models` performs one exact bounded request and returns
  only a canonical validated `ModelCatalog` or one typed terminal error.
- Cancellation/timeout/status/transport/protocol/redaction/cleanup behavior
  matches ADR-0046.
- Controlled-loopback tests are deterministic, non-zero, credential-safe, and
  require no external network or sleep.
- Generation remains unimplemented and the complete workspace gate passes.

## Repository Safety

Modify only the Task 3 adapter crate files and focused tests required for
discovery. Preserve manifests unless mechanically required by an already
approved dev dependency, other crates/adapters, Runtime, docs, prompts, ignored
artifacts, `.codex/`, and unrelated paths. Do not access external services.

## Task-specific Validation

- List and run non-zero discovery/mapping/status/body-bound/timeout/
  cancellation/redaction/cleanup/repetition tests.
- Run the complete adapter package tests.
- Run provider-neutral compatibility tests.
- Run the canonical complete workspace validation.
- Verify diff scope and `git status --short`.

## Suggested commit message

`Implement Sprint 24 model discovery`

## Final report additions

Report request/auth behavior, model mapping/canonicalization, error/timeout/
cancellation/body/redaction policy, loopback matrix, focused/full validation,
changed paths, commit, and final Git state.
