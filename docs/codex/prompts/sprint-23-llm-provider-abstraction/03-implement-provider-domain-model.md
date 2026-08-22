# Implement Sprint 23 Provider Domain Model

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

## Prerequisites / Required gate

Require committed Task 2 with accepted ADR-0045, no unresolved first-slice
ownership or dependency question, and clean task-owned state. Stop if the ADR
requires an unapproved external production dependency.

## Task

Implement only the accepted provider-neutral library crate and public domain
model: provider/model identities, capability/discovery projections, secret-safe
configuration input, execution policy values, response/usage/finish values, and
stable error classification. Do not implement request construction or provider
execution.

## Provider-neutral ownership and API boundary

Create the exact crate and public modules accepted by ADR-0045. Keep them
independent from `oneagent-analysis`, Runtime, concrete adapters, HTTP, JSON,
provider SDKs, and global configuration.

## Model identity, discovery, and capability contract

Implement accepted provider-scoped identity validation, model descriptors,
closed first-slice capability values, canonical ordering/deduplication, and the
owned discovery result shape without performing discovery I/O.

## Request, response, usage, finish, and compatibility contract

Implement only the response, usage, finish, and compatibility-supporting domain
values assigned to this task. Task 4 owns validated request construction and
capability checks.

## Configuration and secret-handling contract

Implement the accepted secret-bearing input and configuration value behavior.
Prove redaction and forbidden formatting/serialization behavior without placing
a real secret in repository content or diagnostics.

## Timeout, retry, cancellation, and cleanup contract

Implement only accepted policy/configuration values and stable classifications
needed by later execution. Do not start tasks, wait on clocks, retry, or own
cancellation execution in this task.

## Error taxonomy and provider mapping

Implement the accepted provider-neutral error kinds and bounded/redacted owned
diagnostic data. Concrete transport/provider mappings remain later sprints.

## Contract corpus, fake, fixture, or controlled-endpoint oracle

Use pure Rust unit cases with exact identities, ordering, bounds, redaction,
response/usage/finish, policy, and error expectations. No network or fixture
containing credentials is allowed.

## Consumer and provider-adapter compatibility

Audit the workspace and keep existing crates source-compatible. No existing
crate is required to consume the new boundary in this task.

## Scope

### Included

- Root workspace membership and the exact new crate manifest/source accepted by
  ADR-0045.
- Public domain values, validation, accessors, formatting constraints, errors,
  Rustdoc, and focused tests assigned above.

### Excluded

Validated text requests, provider trait/execution, concrete adapters/protocols,
HTTP/JSON/SSE, environment/file/CLI configuration, live secrets/services,
Context/Runtime integration, tokenizer/streaming/tools/conversations, automatic
retry or timeout execution, current-state docs, sprint transition, and prompt
retirement.

## Acceptance Criteria

- The new crate follows ADR-0045 ownership and dependency direction with no
  provider-specific schema or unapproved external production dependency.
- Provider/model identities, model ordering, capability projections, policy,
  response, usage, finish, and errors match the accepted closed contracts.
- Invalid, empty, duplicate, reordered, unknown, partial, and boundary inputs
  owned by this slice produce exact deterministic values or typed failures.
- Secret-bearing values cannot reveal content through accepted Debug, Display,
  errors, diagnostics, or serialization behavior.
- Public values are documented and exhibit only the accepted Clone/Eq/Hash/
  ordering semantics; secret types do not acquire unsafe conveniences.
- Focused tests are non-zero and the existing workspace remains green.

## Repository Safety

Create or modify only the ADR-accepted provider-neutral crate under `crates/`,
the root `Cargo.toml`, and `Cargo.lock` only if mechanically required. Preserve
analysis, Runtime, protocol, adapters, docs, prompts, `.codex/`, credentials,
and unrelated paths. Stage only task-owned files.

## Task-specific Validation

- Run non-zero identity/capability/discovery/configuration/redaction/policy/
  response/usage/finish/error unit tests.
- Run the new provider-neutral package tests.
- `cargo tree -p <accepted-package-name> --edges normal`
- Run the canonical complete workspace validation.
- Verify diff scope and `git status --short`.

## Suggested commit message

`Implement Sprint 23 provider domain model`

## Final report additions

Report crate ownership and dependencies, domain vocabulary, validation/order,
secret/redaction behavior, response/usage/finish/errors, focused/full validation,
changed paths, commit, and final Git state.
