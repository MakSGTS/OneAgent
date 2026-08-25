# Complete Sprint 24 OpenAI-Compatible Provider Evidence

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
- `docs/Architecture.md`
- `docs/architecture/semantic-model-2.md`
- committed Tasks 3-5 implementation

## Prerequisites / Required gate

Require committed Task 5, all focused implementation validation successful, and
clean task-owned state. Stop rather than fixing architecture or production
defects in this evidence task.

## Task

Add the complete public repository-owned OpenAI-compatible provider conformance
target and synchronize truthful current-state documentation. Do not mark Sprint
24 completed.

## Contract evidence

Exercise the public concrete adapter only through deterministic controlled
loopback servers and synthetic fixtures. Prove construction, URL/client policy,
optional bearer behavior, discovery, generation, provider/model identity,
canonical ordering, exact request fields, response/finish mapping, local byte
usage, status/transport/protocol errors, body limits, timeout, cancellation,
redaction, cleanup, and repeated fresh execution.

Cover positive, empty, missing, malformed, duplicate, reordered, unknown-field,
wrong-model fallback, wrong choice/index, unknown finish, oversized body/output,
redirect, retry absence, credential sentinel, prompt/response sentinel, timeout,
cancellation before/during work, and zero surviving operation state as
applicable. Live service or credential access is prohibited.

## Consumer and compatibility evidence

Prove `oneagent-llm` remains provider-neutral and std-only, the concrete adapter
depends inward, and existing `oneagent-analysis` and `oneagent-runtime` behavior
and dependencies remain unchanged. No Runtime provider registration or Context
prompt semantics may enter this task.

## Scope

### Included

- Minimum public adapter conformance target and synthetic fixtures.
- Dependency/API/redaction audit and affected compatibility checks.
- `README.md`, `docs/Architecture.md`, and
  `docs/architecture/semantic-model-2.md` current-state synchronization.

### Excluded

Production fixes, architecture reselection, new unapproved dependencies, live
network/credentials, Runtime/CLI/protocol configuration, chat/Responses APIs,
streaming, tools, prompt policy, retry, broad compatibility/performance/security
claims, Sprint 25 work, sprint completion, and prompt retirement.

## Acceptance Criteria

- A non-zero public conformance target proves every accepted ADR-0046 contract
  without external network, credentials, environment, ignored state, or sleeps.
- Exact loopback observations prove no secret, prompt, response, unrestricted
  URL, header, or provider body leaks through implicit formatting or errors.
- Dependency direction and provider-neutral/API compatibility remain intact.
- Current-state docs agree on the implemented bounded first slice and explicitly
  defer excluded behavior.
- Sprint 24 remains incomplete pending Task 7.

## Repository Safety

Create only minimum public test/fixture files under the adapter; modify its
manifest only for already approved dev dependencies, `Cargo.lock` only when
mechanically required, and the three listed current-state docs. Preserve
production code, other crates/adapters, Roadmap, prompts, ignored artifacts,
`.codex/`, and unrelated paths. Do not access external services.

## Task-specific Validation

- List and run the exact non-zero public adapter conformance target.
- Run complete adapter and provider-neutral package tests.
- `cargo test -p oneagent-analysis`
- `cargo test -p oneagent-runtime --lib` with local-bind permission when needed.
- Audit exact dependency direction/features and redaction sentinels.
- Run the canonical complete workspace validation.
- Validate docs, diff scope, and `git status --short`.

## Suggested commit message

`Complete Sprint 24 OpenAI-compatible evidence`

## Final report additions

Report the public conformance matrix, loopback/fixture oracles, dependency and
redaction audits, compatibility, current-state docs, deferred scope,
focused/full validation, changed paths, commit, and final Git state.
