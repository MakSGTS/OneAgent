# Implement Sprint 23 Capability-Aware Requests

Continue OneAgent development.

## Reporting

- Prompt and repository content: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/llm-provider-implementation.md`

## Template

`docs/codex/templates/llm-provider-task.md`

## Authoritative documents

- `docs/Roadmap.md`, Sprint 23 execution plan
- `docs/architecture/llm-provider-investigation.md`
- `docs/adr/0045-llm-provider-abstraction.md`
- committed Task 3 provider domain model

## Prerequisites / Required gate

Require committed Task 3, successful provider-domain validation, accepted public
domain values matching ADR-0045, and clean task-owned state. Stop rather than
changing identity, capability, secret, policy, response, or error architecture.

## Task

Implement only ADR-0045 validated bounded text request construction and
deterministic compatibility checking against one selected provider-scoped model
descriptor. Do not invoke a provider.

## Provider-neutral ownership and API boundary

Keep request validation inside the provider-neutral crate. The request owns its
accepted inputs and does not retain Context Engine, Runtime, transport, adapter,
or credential-source state.

## Model identity, discovery, and capability contract

Consume only Task 3 model identity and capability values. Define no new implicit
capability, provider fallback, model alias, discovery, cache, or refresh behavior.

## Request, response, usage, finish, and compatibility contract

Implement the exact text-only request input, bounds, defaults, canonical order,
validation precedence, selected-model compatibility checks, and typed failures
accepted by ADR-0045. Preserve Task 3 response/usage/finish values unchanged.

## Configuration and secret-handling contract

Requests must not embed credentials or provider-specific headers/URLs. Sensitive
request content follows the accepted formatting and diagnostic restrictions.

## Timeout, retry, cancellation, and cleanup contract

Attach only policy values accepted as part of one request, if ADR-0045 assigns
them there. Do not run clocks, retries, cancellation, tasks, or cleanup.

## Error taxonomy and provider mapping

Map invalid and incompatible requests to the exact Task 3 error taxonomy before
provider I/O. Preserve deterministic validation precedence and bounded/redacted
diagnostics.

## Contract corpus, fake, fixture, or controlled-endpoint oracle

Use pure Rust request/model cases covering every accepted bound, default,
capability combination, missing/duplicate/reordered input, incompatible feature,
diagnostic redaction, and repeated equality.

## Consumer and provider-adapter compatibility

Preserve Task 3 public values and workspace consumers. Do not add a concrete
provider, Context adapter, Runtime route, or protocol projection.

## Scope

### Included

- Provider-neutral request/input types, constructors, validation, compatibility
  checks, errors, Rustdoc, and focused tests in the Task 3 crate.

### Excluded

Provider execution/discovery I/O, concrete schemas/adapters, HTTP/JSON/SSE,
prompt templates/policy, conversations/history, tokenizer/token counts,
streaming, tools/structured output/media, Context/Runtime/protocol integration,
live credentials/services, automatic retries/timeouts, current-state docs,
sprint transition, and prompt retirement.

## Acceptance Criteria

- Only the ADR-0045 text first slice can form a valid owned request.
- Bounds, required/optional inputs, defaults, canonicalization, duplicates,
  validation precedence, model identity, and capability compatibility are exact
  and deterministic.
- Incompatible requests fail before provider I/O with stable typed errors and no
  partial request.
- Equivalent reordered inputs produce equal accepted requests where ADR-0045
  declares order irrelevant; meaningful input order is preserved where required.
- Requests and errors do not leak secrets or unrestricted sensitive content.
- No provider-specific field, fallback, transport, or unsupported capability is
  implied, and focused tests are non-zero with the workspace green.

## Repository Safety

Modify only the provider-neutral crate files and minimum focused tests accepted
for Task 4. Preserve manifests unless mechanically required by accepted local
test dependencies, analysis, Runtime, protocol, adapters, docs, prompts,
`.codex/`, credentials, and unrelated paths. Stage only task-owned files.

## Task-specific Validation

- Run non-zero request/default/bound/precedence/capability/incompatible/
  reorder/redaction/repetition tests.
- Run the provider-neutral package tests.
- Run the canonical complete workspace validation.
- Verify diff scope and `git status --short`.

## Suggested commit message

`Implement Sprint 23 capability-aware requests`

## Final report additions

Report request vocabulary, validation/precedence, capability compatibility,
ordering, sensitive-content handling, preserved contracts, focused/full
validation, changed paths, commit, and final Git state.
