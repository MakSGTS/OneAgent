# Investigate Sprint 21 CLI Client Boundary

Continue OneAgent development.

## Reporting

- Prompt and repository content: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/investigation.md`

## Template

`docs/codex/templates/investigation-task.md`

## Authoritative documents

- `docs/Roadmap.md`, Sprint 21 execution plan
- `docs/Architecture.md`
- `docs/architecture/semantic-model-2.md`
- `docs/reviews/sprint-20-persistent-cache.md`
- `docs/adr/0037-runtime-service-container.md`
- `docs/adr/0038-http-api-health.md`
- `docs/adr/0039-workspace-service.md`
- `docs/adr/0040-graph-query-api.md`
- `docs/adr/0041-file-watching.md`
- `docs/adr/0042-persistent-cache.md`

## Prerequisites / Required gate

Require the committed Sprint 21 planning baseline containing this complete
prompt suite and matching Roadmap manifest. Require Sprint 21 to be the unique
eligible target and preserve a clean task-owned state.

## Investigation objective

Create `docs/architecture/cli-client-investigation.md` with verified evidence
for the smallest complete, testable supported CLI Client slice and the exact
questions ADR-0043 must decide. Do not select architecture, add a dependency,
or modify production behavior.

## Questions to answer

- What exactly does the current CLI binary own, and which reusable library/main
  boundary can expose deterministic parsing, execution, output, and exit status?
- Which exact Runtime health and Graph Query routes, methods, parameters,
  defaults, bounds, JSON media/body contracts, errors, readiness rules, ordering,
  and snapshot semantics are stable client inputs?
- Which endpoint inputs are supported by current Runtime configuration and
  source evidence without inventing DNS, URL, environment, file, proxy, TLS, or
  service-discovery contracts?
- Which command/subcommand/option grammar, required/optional/default values,
  duplicate/unknown/missing cases, help/version behavior, stdout/stderr policy,
  and exit classifications must ADR-0043 decide?
- Can a complete accepted HTTP/1.1 client use only `std` and opaque JSON
  passthrough? Inventory request encoding, status/header/body framing,
  content-length/chunking/connection-close cases, response limits, malformed or
  truncated responses, timeouts, and connection cleanup.
- Would any viable supported client require a new production dependency,
  `oneagent-protocol`, Runtime public API/wire changes, or manifest approval?
- Which real CLI process/library entry points, fake-server seams, production
  Runtime construction, tracked EDT/Designer fixtures, loopback observers, and
  deterministic synchronization can prove all commands and failures?
- Which compatibility, platform, security, performance, packaging, and later
  integration concerns are unsupported and must remain deferred?

## Evidence scope

- `apps/cli/` manifest, binary, consumers, history, and executable testability.
- `apps/runtime/` configuration, HTTP health/Graph Query adapters, production
  composition, public library, tests, fixtures, and listener observations.
- `crates/protocol/`, all Cargo manifests/lockfile/tree, CI platforms, current
  documentation, Roadmap, reviews, accepted ADRs, and recent prompt suites.

## Evidence sources / fixtures

At minimum inspect:

- `apps/cli/Cargo.toml`
- `apps/cli/src/main.rs`
- `apps/runtime/src/config/mod.rs`
- `apps/runtime/src/http/mod.rs`
- `apps/runtime/src/http/graph_query.rs`
- `apps/runtime/src/main.rs`
- `apps/runtime/tests/http_health.rs`
- `apps/runtime/tests/graph_query_api.rs`
- `apps/runtime/tests/fixtures/workspace_service/`
- `crates/protocol/Cargo.toml`
- `crates/protocol/src/lib.rs`

Record exact provenance for every public integration oracle. Do not make
external services, arbitrary sleeps, fixed ports, real signals, ignored local
corpora, or unapproved dependencies prerequisites.

## Excluded

ADR acceptance, Rust/Cargo/public API/fixture changes, CLI implementation,
Runtime route/schema/configuration changes, protocol activation, graph/parser/
adapter semantics, current-state documentation, prompt retirement, Roadmap
transition, packaging, benchmarks, security/performance claims, and external
research.

## Completion Criteria

- The document separates confirmed evidence, accepted constraints,
  compatibility-sensitive behavior, unsupported cases, unknowns, and decisions.
- It inventories the CLI and Runtime ownership/API boundaries, exact stable wire
  inputs, feasible HTTP behavior, dependency impact, platforms, consumers,
  fixtures, and non-zero deterministic oracles.
- It defines the minimum ADR matrix for ownership, command grammar, endpoint,
  requests, response framing/limits, JSON presentation, streams, errors, exits,
  resource lifecycle, compatibility, dependencies, testing, and deferred scope.
- It states whether implementation can remain on existing approved production
  dependencies or which exact addition would require explicit approval.
- Missing/conflicting evidence blocks Task 2 instead of being replaced by an
  invented CLI, transport, discovery, retry, timeout, or security contract.
- No production, manifest, fixture, Roadmap-state, current-state, or prompt-suite
  file is changed.

## Repository Safety

Create only `docs/architecture/cli-client-investigation.md`. Preserve `.codex/`,
production code, manifests, fixtures, prompts, Roadmap state, and unrelated
files. Stage only the investigation document when commit mode is authorized.

## Task-specific Validation

- Verify every cited path, API, route, parameter, fixture, dependency, platform,
  test, and consumer from the live repository.
- Run non-mutating focused `--list` or existing tests only when needed; report
  zero matches separately.
- Validate links and `git diff --check`.
- `git status --short`

## Suggested commit message

`Investigate Sprint 21 CLI Client`

## Final report additions

Report confirmed CLI/Runtime/transport boundaries, dependency and platform
findings, fixture/test oracles, unresolved ADR questions, decision readiness,
changed path, validation, commit, and final Git state.
