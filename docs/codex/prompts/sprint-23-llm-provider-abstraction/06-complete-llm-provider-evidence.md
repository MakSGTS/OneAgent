# Complete Sprint 23 LLM Provider Evidence

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
- `docs/adr/0044-context-engine.md`
- `docs/Architecture.md`
- `docs/architecture/semantic-model-2.md`
- committed Tasks 3-5 provider-neutral implementation

## Prerequisites / Required gate

Require committed Task 5, all focused implementation validation successful, and
clean task-owned state. Stop rather than repairing architecture or production
defects inside an evidence-only task.

## Task

Add the complete public repository-owned LLM Provider abstraction conformance
evidence required by ADR-0045 and synchronize truthful current-state
documentation. Do not mark Sprint 23 completed.

## Provider-neutral ownership and API boundary

Exercise only the public provider-neutral package through independent fake
providers and accepted public inputs/outputs. Prove no concrete adapter, Runtime,
transport, global configuration, or live provider is required.

## Model identity, discovery, and capability contract

Cover every accepted public identity, descriptor, capability, discovery order,
duplicate, empty, invalid, unsupported, and repeated outcome.

## Request, response, usage, finish, and compatibility contract

Cover every accepted request/default/bound/validation/precedence and capability
compatibility case plus successful, empty, partial, malformed-placeholder,
usage, finish, and unsupported response behavior that ADR-0045 assigns to the
provider-neutral layer.

## Configuration and secret-handling contract

Prove secret values and sensitive content remain absent from Debug, Display,
errors, diagnostics, snapshots, and fixtures as accepted. Use synthetic sentinels
only and assert their absence from observable output.

## Timeout, retry, cancellation, and cleanup contract

Cover exact enabled, disabled, or representation-only behavior, terminal
precedence, attempt accounting, replay absence, cancellation before/during work,
and complete fake cleanup without arbitrary sleeps.

## Error taxonomy and provider mapping

Exercise every public error kind and bounded/redacted diagnostic path accepted
for Sprint 23. Do not claim concrete provider wire mappings.

## Contract corpus, fake, fixture, or controlled-endpoint oracle

Create the minimum public integration target under the accepted provider crate's
`tests/` directory. Use checked-in Rust fakes and exact values only; ensure the
target executes non-zero tests and remains independent from network, credentials,
environment, filesystem state, and developer-local services.

## Consumer and provider-adapter compatibility

Prove public Context Engine output remains independently usable as deterministic
text evidence without adding prompt semantics, and existing Runtime public
service/cancellation behavior remains source-compatible. Concrete provider
adapter conformance begins in Sprint 24.

## Scope

### Included

- Public conformance evidence for positive, empty, invalid, missing, duplicate,
  reordered, incompatible, unsupported, error, redacted, policy, cancellation,
  cleanup, and repeated cases as applicable to ADR-0045.
- `README.md`, `docs/Architecture.md`, and
  `docs/architecture/semantic-model-2.md` current-state synchronization.
- Dependency/API/Context/Runtime compatibility evidence and full workspace
  checks.

### Excluded

Production fixes, architecture reselection, new external dependencies, concrete
providers/wire fixtures, live network/credentials, Runtime/CLI/protocol surface,
prompt/tool policy, tokenizer, streaming, conversations, MCP/IDE, performance/
quality/security claims, Sprint 24 implementation, sprint completion, and prompt
retirement.

## Acceptance Criteria

- A non-zero public conformance target proves every accepted provider-neutral
  identity, capability, request, response, secret, error, execution, policy,
  cancellation, and cleanup contract through public APIs.
- Exact expected values and diagnostics act as reproducible oracles; no test
  relies on hash order, sleeps, network, live provider, credential, or ignored
  local data.
- Independent fake implementations, equivalent reordered inputs, and repeated
  fresh executions remain contract-equivalent where ADR-0045 requires it.
- Existing Context Engine and Runtime tests remain green; public APIs outside
  the additive provider-neutral slice stay compatible.
- Current-state docs agree on the implemented first slice and explicitly defer
  concrete providers, wire protocols, live configuration/discovery, Runtime
  exposure, prompt/tool policy, streaming, MCP, and IDE.
- Sprint 23 remains incomplete pending Task 7.

## Repository Safety

Create only the minimum public test files under the accepted provider crate's
`tests/`; modify only its manifest for proven dev dependencies, `Cargo.lock`
only if mechanically required, and the three current-state docs listed above.
Preserve production code, analysis/Runtime/protocol/adapters, prompts, Roadmap,
`.codex/`, credentials, and unrelated paths. Stage task-owned paths only.

## Task-specific Validation

- List and run the exact non-zero public LLM Provider conformance target.
- Run the complete provider-neutral package tests.
- `cargo test -p oneagent-analysis`
- `cargo test -p oneagent-runtime --lib` with only local loopback permission
  when required by the sandbox.
- Run the canonical complete workspace validation.
- Validate dependency tree, doc links/claims, diff scope, and
  `git status --short`.

## Suggested commit message

`Complete Sprint 23 LLM Provider evidence`

## Final report additions

Report the public conformance matrix, exact fake/oracles, identity/discovery/
capability/request/response outcomes, secret/redaction evidence, errors,
execution/cancellation/cleanup, repetition, compatibility, current-state docs,
deferred scope, focused/full validation, changed paths, commit, and final Git
state.
