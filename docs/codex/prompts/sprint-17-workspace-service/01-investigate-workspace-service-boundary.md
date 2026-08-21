# Investigate Sprint 17 Workspace Service Boundary

Continue OneAgent development.

## Reporting

- Prompt and repository content: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/investigation.md`

## Template

`docs/codex/templates/investigation-task.md`

## Authoritative documents

- `docs/Roadmap.md`, Sprint 17 execution plan
- `docs/architecture/semantic-model-2.md`
- `docs/reviews/sprint-16-http-api-health.md`
- `docs/adr/0002-runtime-composition-root.md`
- `docs/adr/0004-filesystem-workspace-discovery.md`
- `docs/adr/0036-designer-xml-adapter.md`
- `docs/adr/0037-runtime-service-container.md`
- `docs/adr/0038-http-api-health.md`

## Prerequisites / Required gate

Require the committed Sprint 17 planning baseline containing this complete
prompt suite and matching Roadmap manifest. Require Sprint 17 to be the unique
eligible target and preserve a clean task-owned state.

## Investigation objective

Create `docs/architecture/workspace-service-investigation.md` with verified
evidence for the smallest testable Runtime Workspace service and the exact
questions ADR-0039 must decide. Do not select architecture or modify production
behavior.

## Questions to answer

- Which current types own workspace discovery, configuration identity and
  format, EDT and Designer XML semantic builds, diagnostics, graph validation,
  Runtime configuration, shared state, lifecycle, service startup, failure,
  cancellation, and shutdown?
- Which public APIs can orchestrate a complete EDT build and a complete Designer
  XML build, and where do their result and diagnostic contracts differ?
- How can one configured workspace root, zero, one, or multiple discovered
  configurations, stable ordering, duplicate identities, unsupported formats,
  partial results, and build failures be observed without inventing semantics?
- What immutable snapshot shape and publication seam can serve Sprint 18 without
  creating a second graph authority or a transport-specific API?
- Must initial discovery/build finish before service startup acknowledgement,
  and how should that affect lifecycle-derived readiness?
- Which configuration, ownership, error-chain, cancellation, and observability
  decisions remain unresolved?
- Which existing fixtures prove positive EDT and Designer XML builds, empty and
  malformed roots, mixed-format discovery, deterministic repetition, and
  resource cleanup? What bounded Runtime-owned fixture must be derived if an
  existing fixture cannot provide a stable public oracle?
- Which path dependencies are already workspace members and locked, which
  package boundaries would consume them, and whether any new external
  production dependency is actually required?

## Evidence scope

- `apps/runtime/` public library, configuration, AppState, service container,
  HTTP readiness, binary composition, and public integration tests.
- `crates/workspace/`, `adapters/filesystem/`, `adapters/edt/`,
  `adapters/designer-xml/`, and `crates/graph/` definitions, consumers, tests,
  error contracts, and manifests.
- Repository-owned EDT and Designer XML fixtures and real source corpora needed
  only to establish serialization-independent build oracles.
- Relevant Coverage registries, CI platforms, Git history, Roadmap, reviews,
  ADRs, and current-state documents.

## Evidence sources / fixtures

At minimum inspect:

- `adapters/edt/tests/fixtures/`
- `adapters/designer-xml/tests/fixtures/sprint14_conformance/`
- `adapters/filesystem/src/lib.rs` tests
- `apps/runtime/tests/service_container.rs`
- `apps/runtime/tests/http_health.rs`

Record exact provenance for every proposed public integration fixture. Do not
make ignored local corpora a Runtime or CI prerequisite.

## Excluded

- ADR acceptance, production Rust changes, Cargo changes, public API changes,
  graph-query endpoints, file watching, rebuild triggers, persistence, CLI,
  HTTP workspace routes, semantic graph changes, prompt retirement, Roadmap
  transition, performance claims, and external research.

## Completion Criteria

- The investigation separates confirmed repository evidence, accepted
  constraints, compatibility-sensitive behavior, unknowns, and decision
  questions.
- Every proposed first-slice capability has a repository-owned test oracle and
  discoverable production entry point, including failure and repeated-build
  behavior.
- The document defines the minimal ADR decision matrix for root configuration,
  discovery/build dispatch, result shape, atomicity, ordering, ownership,
  readiness, errors, cancellation, shutdown, observability, and deferred scope.
- Missing evidence blocks Task 2 instead of being replaced with assumptions.
- No production, manifest, Roadmap-state, current-state, or prompt-suite file is
  changed.

## Repository Safety

Create only `docs/architecture/workspace-service-investigation.md`. Preserve
`.codex/`, production code, manifests, fixtures, current prompt suites, Roadmap
state, and unrelated files. Stage only the investigation document when commit
mode is authorized.

## Task-specific Validation

- Verify every cited path, type, API, fixture, test, and dependency from the
  live repository.
- Run non-mutating focused `--list` or existing tests only when needed to prove
  an oracle; report zero matches separately.
- Validate document links and `git diff --check`.
- `git status --short`

## Suggested commit message

`Investigate Sprint 17 Workspace service`

## Final report additions

Report confirmed APIs and constraints, fixture/test oracles, unresolved ADR
questions, dependency findings, decision readiness, changed path, validation,
commit, and final Git state.
