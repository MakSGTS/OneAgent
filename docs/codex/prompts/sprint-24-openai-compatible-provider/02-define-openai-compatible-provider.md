# Define Sprint 24 OpenAI-Compatible Provider

Continue OneAgent development.

## Reporting

- Prompt and repository content: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/architecture.md`

## Template

`docs/codex/templates/architecture-task.md`

## Authoritative documents

- `docs/Roadmap.md`, Sprint 24 execution plan
- `docs/architecture/openai-compatible-provider-investigation.md`
- `docs/adr/0045-llm-provider-abstraction.md`
- `docs/codex/workflows/llm-provider.md`

## Prerequisites / Required gate

Require committed Task 1 evidence, clean task-owned state, and no conflict
between pinned/live wire evidence and ADR-0045. Stop on missing source evidence
instead of inventing a provider contract.

## Task

Create and accept `docs/adr/0046-openai-compatible-provider.md`. Decide only the
smallest concrete non-streaming adapter slice required by Sprint 24; do not
implement production behavior.

## Required architecture decisions

- Concrete crate ownership and dependency direction.
- Exact production and dev dependency set, features, rationale, and explicit
  user-approval gate before implementation.
- Provider identity, construction API, base-URL validation/normalization,
  HTTP/HTTPS, TLS roots, redirect and implicit proxy/configuration policy.
- Optional bearer credential behavior and complete redaction boundary.
- Exact discovery request, accepted list/data/model fields, unknown-field
  treatment, capability assignment, canonical order, duplicates, bounds, and
  empty success.
- Exact non-streaming completion request fields and deterministic omission of
  unsupported sampling/prompt semantics.
- Relationship between provider-neutral output-byte bound and provider token
  ceiling, plus authoritative local response validation.
- Response object, model identity, choice count/index, output, `stop`/`length`,
  provider usage, unknown/malformed/partial behavior, and fallback rejection.
- Request/response body limits; HTTP status, provider, transport, protocol,
  timeout, cancellation, cleanup, diagnostic, and terminal precedence mapping.
- Controlled-loopback conformance, consumer compatibility, implementation
  order, documentation completion, and deferred scope.

## Scope

### Included

ADR-0046 and its confirmed evidence, alternatives, accepted decision, rejected
alternatives, prerequisites, completion criteria, risks, and deferred scope.

### Excluded

Rust/Cargo changes, dependency installation or approval assumption, fixtures,
live calls, chat/Responses APIs, streaming, tools, retry, Runtime exposure,
current-state support claims, sprint transition, and prompt retirement.

## Acceptance Criteria

- Every unresolved Task 1 decision is accepted or explicitly deferred.
- ADR-0045 remains unchanged and provider-neutral values contain no wire schema.
- The selected dependency set and approval gate are exact.
- Identity mismatch, unknown model fallback, output bounds, response-body
  bounds, secret/content redaction, timeout, cancellation, and cleanup are
  deterministic and testable.
- Tasks 3-6 can implement the ADR without architecture reselection.

## Repository Safety

Create only ADR-0046. Preserve source, Cargo, tests, prompts, Roadmap, accepted
ADRs, ignored artifacts, `.codex/`, and unrelated paths. Do not access external
paths.

## Task-specific Validation

- Validate ADR structure, links, evidence citations, decision completeness,
  implementation order, and Roadmap agreement.
- `git diff --check`
- Verify diff scope and `git status --short`.

## Suggested commit message

`Define Sprint 24 OpenAI-compatible provider`

## Final report additions

Report the accepted ownership, dependencies/approval gate, URL/auth/client
policy, wire mappings, errors/bounds/redaction, timeout/cancellation, rejected
alternatives, deferred scope, validation, commit, and final Git state.
