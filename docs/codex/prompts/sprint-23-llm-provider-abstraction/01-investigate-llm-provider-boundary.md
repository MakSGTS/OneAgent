# Investigate Sprint 23 LLM Provider Boundary

Continue OneAgent development.

## Reporting

- Prompt and repository content: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/investigation.md`

## Template

`docs/codex/templates/investigation-task.md`

## Authoritative documents

- `docs/Roadmap.md`, Sprint 23 execution plan
- `docs/Architecture.md`
- `docs/architecture/semantic-model-2.md`
- `docs/adr/0044-context-engine.md`
- `docs/reviews/sprint-22-context-engine.md`
- `docs/codex/profiles/llm-provider-implementation.md`
- `docs/codex/workflows/llm-provider.md`
- `docs/codex/templates/llm-provider-task.md`

## Prerequisites / Required gate

Require the committed Sprint 23 planning baseline containing this complete
prompt suite and matching Roadmap manifest. Require Sprint 23 to be the unique
eligible target and preserve a clean task-owned state.

## Investigation objective

Create `docs/architecture/llm-provider-investigation.md` with verified evidence
for the smallest complete provider-neutral LLM abstraction and the exact
questions ADR-0045 must decide. Do not select architecture, add dependencies,
access a live provider or credential, or modify production behavior.

## Questions to answer

- Which existing or new library boundary can own provider-neutral identities,
  capabilities, requests, responses, configuration inputs, errors, and async
  execution without reversing current dependency direction?
- Which current Context Engine output can become deterministic text input, and
  which prompt templates, roles, conversations, tokenization, tools, source
  text, or Runtime integration remain unavailable or deferred?
- Which provider/model identity and discovery/capability vocabulary is needed
  for the Sprint 23 first slice without copying an OpenAI, LM Studio, or Ollama
  schema into the shared domain?
- Which bounded text request, response, usage, finish, validation, ordering,
  empty, missing, duplicate, incompatible, partial, malformed, and unknown cases
  can be specified and tested provider-independently?
- Which secret-bearing configuration values are necessary at the abstraction
  boundary, and which Debug, Display, clone, serialization, diagnostic, URL,
  header, body, fixture, and source-control behaviors require a safe contract?
- Which existing `Future`, boxed async trait seam, cancellation, timeout, test
  synchronization, and error-classification patterns are reusable? Which retry,
  delay, streaming, concurrency, connection, rate-limit, and shutdown policies
  lack evidence and must remain disabled or deferred?
- Can stable timeout/cancel/retry classifications be represented without
  implementing a clock, transport, or automatic retry executor in Sprint 23?
- Which repository-owned fake provider and constructed request/response cases
  can form exact non-network conformance oracles, including reorder and repeat?
- Would the first slice require a new workspace crate, production dependency,
  manifest/lock change, Runtime consumer, serialization, external fixture, or
  live-provider data? Identify approval gates exactly.
- Which compatibility, platform, security, performance, cost, quality, MCP,
  IDE, tool-policy, and concrete-provider concerns are unsupported?

## Evidence scope

- Workspace manifests, dependency graph, CI platforms, public library patterns,
  tests, history, and consumers.
- `crates/analysis/` Context Engine public request/bundle/error boundary.
- `apps/runtime/` configuration, service future, cancellation, lifecycle, error,
  and deterministic test-double patterns only as compatibility evidence.
- `crates/protocol/` and current docs only to confirm absences and boundaries.
- Accepted ADR-0044, Sprint 22 review, LLM Provider framework, and recent prompt
  suites.

## Evidence sources / fixtures

At minimum inspect:

- `Cargo.toml` and every current crate manifest
- `Cargo.lock`
- `crates/analysis/src/context/mod.rs`
- `crates/analysis/tests/context_engine.rs`
- `apps/runtime/src/config/provider.rs`
- `apps/runtime/src/service/definition.rs`
- `apps/runtime/src/service/cancellation.rs`
- `apps/runtime/src/error/mod.rs`
- `apps/runtime/tests/service_container.rs`
- `crates/protocol/src/lib.rs`
- `.github/workflows/ci.yml`

Record exact provenance for every proposed test oracle. Do not make external
provider docs, wire payloads, live models/services, network access, credentials,
arbitrary sleeps, or ignored local state prerequisites.

## Excluded

ADR acceptance, Rust/Cargo/public API/fixture changes, provider implementation,
concrete adapters/protocols, Runtime routes/configuration, current-state docs,
prompt retirement, Roadmap transition, external research, benchmarks, and
unsupported compatibility/quality/security/performance claims.

## Completion Criteria

- The document separates confirmed evidence, accepted constraints,
  compatibility-sensitive behavior, unsupported cases, unknowns, and decisions.
- It inventories exact ownership candidates, APIs, consumers, dependency and CI
  constraints, Context/Runtime boundaries, identity/capability/request/response/
  configuration/error/execution vocabulary, and deterministic non-zero oracles.
- It defines the minimum ADR matrix for ownership, identity, discovery,
  capabilities, requests, responses, usage, finish, validation, secrets,
  errors, execution, timeouts, retries, cancellation, evidence, compatibility,
  and deferred scope.
- It states whether a standard-library-only workspace crate is feasible and
  identifies every addition that would require explicit dependency approval.
- Missing or conflicting evidence blocks Task 2 instead of being replaced by an
  invented provider schema, credential source, transport, retry, or streaming
  contract.
- No production, manifest, fixture, Roadmap-state, current-state, or prompt-suite
  file is changed.

## Repository Safety

Create only `docs/architecture/llm-provider-investigation.md`. Preserve
`.codex/`, production code, manifests, lockfile, fixtures, prompts, Roadmap
state, live credentials, and unrelated files. Stage only the investigation
document when commit mode is authorized.

## Task-specific Validation

- Verify every cited path, API, dependency, platform, test, consumer, and oracle
  from the live repository.
- `cargo test -p oneagent-analysis`
- `cargo test -p oneagent-runtime --lib` with only the existing local loopback
  permission when required by the sandbox; use no external network.
- Validate links and `git diff --check`.
- `git status --short`

## Suggested commit message

`Investigate Sprint 23 LLM Provider boundary`

## Final report additions

Report confirmed ownership/API/dependency boundaries, Context and Runtime
compatibility, identity/capability/request/response/secret/execution findings,
test oracles, dependency and platform impact, unresolved ADR questions,
decision readiness, changed path, validation, commit, and Git state.
