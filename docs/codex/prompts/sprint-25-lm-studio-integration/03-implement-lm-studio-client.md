# Implement Sprint 25 LM Studio Client Foundation

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
- `docs/reviews/sprint-24-openai-compatible-provider.md`

## Prerequisites / Required gate

Require committed ADR-0047, clean task-owned state, and explicit user approval
for every new direct dependency or feature beyond ADR-0046. Stop before Cargo
or production edits when approval is absent. Recheck all existing public and
reverse consumers before changing a reusable or public seam.

## Task

Implement only the ADR-0047 LM Studio client foundation: the accepted package
or composition boundary, stable provider identity, deterministic construction,
bounded client/transport policy, optional credential handling, private endpoint
and wire foundations, and focused construction/redaction evidence. Do not
perform model discovery or generation.

## Provider-neutral ownership and API boundary

Keep `oneagent-llm` unchanged unless ADR-0047 explicitly proves a minimal
provider-neutral API prerequisite. The LM Studio adapter depends inward on it;
Context, Runtime, protocol, CLI, graph, workspace, and source adapters do not
gain a dependency. Preserve the observable `OpenAiCompatibleProvider` contract.

## Model identity, discovery, and capability contract

Implement the stable `lm-studio` identity and only construction-time values
accepted by ADR-0047. Private discovery DTOs/constants may be added when needed
for the next task, but no request is sent and no catalog is produced.

## Request, response, usage, finish, and compatibility contract

Private generation DTOs/constants may be added only as accepted foundation.
Do not implement `generate`, reinterpret provider-neutral byte usage/finish, or
add chat/provider-extension concepts.

## Configuration and secret-handling contract

Implement only explicit/default root, locality, URL, scheme/host/port/path,
authentication, redirect, proxy, TLS, user-agent, and redaction rules accepted
by ADR-0047. Construction performs no I/O and exposes no secret, endpoint, raw
provider error, or sensitive value through implicit formatting.

## Timeout, retry, cancellation, and cleanup contract

Prepare only the accepted client and execution helpers. Preserve one attempt,
no retry/fallback, represented total timeout, cooperative cancellation, and no
spawned/background state. No operation executes in this task.

## Error taxonomy and provider mapping

Map construction failures to the exact ADR-0047 `LlmErrorKind` and closed
redacted diagnostics. Do not expose URL, credential, library source, headers,
or bodies.

## Contract oracle

Use focused construction tests for accepted and rejected IDs, URL/root/locality,
scheme/host/port/path, authentication, client policy, dependency features,
endpoint joining, Send/Sync, no-I/O construction, and sentinel redaction.

## Consumer and provider-adapter compatibility

Run existing OpenAI-compatible construction and public conformance regression
targets. If ADR-0047 accepts a reusable seam, prove the generic adapter retains
its exact provider ID, URLs, wire constants, no-proxy/no-redirect behavior,
public type, and dependency direction.

## Scope

### Included

- Exact ADR-0047 foundation files and approved manifest/lock changes.
- Focused unit tests for construction, identity, policy, and redaction.
- Minimal Rustdoc for the new public boundary.

### Excluded

- Discovery/generation I/O, public conformance target, current-state docs,
  Runtime/configuration integration, live LM Studio, and every deferred feature.

## Acceptance Criteria

- The accepted foundation constructs deterministically with no I/O.
- Only the exact approved dependencies/features are present.
- Provider identity, endpoints, client policy, authentication, and redaction
  match ADR-0047.
- No discovery or generation request can occur yet.
- Existing generic adapter behavior and public tests remain compatible.
- No sensitive value appears through error, adapter, or test diagnostics.

## Repository Safety

Before editing, state exact files to create and modify from the live ADR and
source tree. Preserve prompts, Roadmap, unrelated code, `.codex/`, local LM
Studio state, and developer-local artifacts. Do not contact a live provider.

## Task-specific Validation

- Non-zero focused LM Studio construction/identity/URL/auth/redaction tests.
- `cargo test -p oneagent-openai-compatible --lib --offline`
- `cargo test -p oneagent-openai-compatible --test conformance --offline`
- `cargo test -p oneagent-llm --offline`
- Exact dependency tree/feature audit for every affected package.
- Canonical full workspace validation from `docs/codex/core/validation.md`.

## Suggested commit message

`Implement Sprint 25 LM Studio client`

## Final report additions

Report exact files and dependencies, foundation/public API, construction and
redaction behavior, generic-adapter compatibility, focused/full validation,
commit, and final Git state.
