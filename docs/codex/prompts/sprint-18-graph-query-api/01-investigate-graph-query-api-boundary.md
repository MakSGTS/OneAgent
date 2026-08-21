# Investigate Sprint 18 Graph Query API Boundary

Continue OneAgent development.

## Reporting

- Prompt and repository content: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/investigation.md`

## Template

`docs/codex/templates/investigation-task.md`

## Authoritative documents

- `docs/Roadmap.md`, Sprint 18 execution plan
- `docs/architecture/semantic-model-2.md`
- `docs/reviews/sprint-17-workspace-service.md`
- `docs/adr/0002-runtime-composition-root.md`
- `docs/adr/0008-semantic-model-2-knowledge-graph.md`
- `docs/adr/0026-semantic-index-boundary.md`
- `docs/adr/0027-incremental-semantic-index-maintenance.md`
- `docs/adr/0037-runtime-service-container.md`
- `docs/adr/0038-http-api-health.md`
- `docs/adr/0039-workspace-service.md`

## Prerequisites / Required gate

Require the committed Sprint 18 planning baseline containing this complete
prompt suite and matching Roadmap manifest. Require Sprint 18 to be the unique
eligible target and preserve a clean task-owned state.

## Investigation objective

Create `docs/architecture/graph-query-api-investigation.md` with verified
evidence for the smallest testable Runtime Graph Query API and the exact
questions ADR-0040 must decide. Do not select architecture or modify production
behavior.

## Questions to answer

- Which current types own canonical graph facts, stable node and edge identity,
  node payloads, provenance, exact-name and kind lookup, adjacency,
  containment, dependency/usage navigation, bounded traversal, validation,
  diagnostics, reports, and Impact?
- Which `SemanticGraphQuery` operations have complete deterministic behavior
  and non-zero tests suitable for a bounded first Runtime slice, and which
  operations named by Semantic Model 2.0 remain unimplemented or unsupported?
- How are immutable Workspace configurations selected and observed, including
  absent snapshots, empty workspaces, unknown configuration IDs, multiple
  configurations, shutdown clearing, and future replacement without inventing
  stale-state policy?
- Which graph values can be projected into stable owned transport-neutral
  results without exposing internal collections, Rust lifetimes, adapter state,
  diagnostic prose, or unsupported payload serialization?
- Which route versioning, path, method, request bound, status, JSON schema,
  error-code, media-type, fallback, and compatibility decisions remain open?
- How can the existing HTTP service receive query dependencies without adding a
  second listener, global mutable state, hidden service construction, mutable
  readiness, or detached work?
- Which current consumers and future Sprint 21 client boundary constrain public
  Runtime exports or use of the placeholder `oneagent-protocol` crate? Can the
  first slice remain within the existing dependency surface?
- Which repository-owned fixtures and tests prove production EDT and Designer
  XML graphs, exact identities and relations, multi-configuration selection,
  missing/invalid input, deterministic ordering, request bounds, health
  compatibility, shutdown, cleanup, and repeated fresh runs?

## Evidence scope

- `crates/graph/` query, identity, node, edge, payload, provenance, validation,
  report, Impact, diff, semantic-index definitions, consumers, and tests.
- `apps/runtime/` Workspace snapshots and observer, AppState, HTTP service,
  production composition, manifests, unit tests, public integration tests, and
  fixtures.
- `crates/protocol/` and `apps/cli/` current placeholder boundaries and all
  repository consumers of graph-query and Workspace observation APIs.
- Relevant Coverage registries, CI platforms, Git history, Roadmap, reviews,
  ADRs, current-state documents, and locked dependency evidence.

## Evidence sources / fixtures

At minimum inspect:

- `crates/graph/src/query.rs`
- `crates/graph/tests/query.rs`
- `crates/graph/src/node.rs`
- `crates/graph/src/edge.rs`
- `apps/runtime/src/workspace/mod.rs`
- `apps/runtime/src/http/mod.rs`
- `apps/runtime/tests/http_health.rs`
- `apps/runtime/tests/workspace_service.rs`
- `apps/runtime/tests/fixtures/workspace_service/`

Record exact provenance for every proposed public integration oracle. Do not
make ignored local corpora, external services, or timing races a prerequisite.

## Excluded

- ADR acceptance, production Rust changes, Cargo changes, public API changes,
  new routes or schemas, graph semantic changes, file watching, persistence,
  supported CLI implementation, prompt retirement, Roadmap transition,
  performance claims, dependency additions, and external research.

## Completion Criteria

- The investigation separates confirmed repository evidence, accepted
  constraints, compatibility-sensitive behavior, unsupported operations,
  unknowns, and decision questions.
- It inventories exact existing public types, value vocabularies, consumers,
  dependencies, fixtures, and non-zero test oracles relevant to the first slice.
- Every candidate first-slice capability has a canonical production entry point
  and observable positive, missing, invalid, bounded, deterministic, lifecycle,
  and repeated-run oracle where applicable.
- The document defines the minimum ADR matrix for ownership, snapshot and
  configuration selection, operations, limits, result projection, routes,
  methods, schemas, errors, compatibility, lifecycle, shutdown, observation,
  testing, and deferred scope.
- Missing or conflicting evidence blocks Task 2 instead of being replaced with
  invented routes, serialized fields, errors, or semantic behavior.
- No production, manifest, Roadmap-state, current-state, or prompt-suite file is
  changed.

## Repository Safety

Create only `docs/architecture/graph-query-api-investigation.md`. Preserve
`.codex/`, production code, manifests, fixtures, current prompt suites, Roadmap
state, and unrelated files. Stage only the investigation document when commit
mode is authorized.

## Task-specific Validation

- Verify every cited path, type, API, fixture, test, dependency, and consumer
  from the live repository.
- Run non-mutating focused `--list` or existing tests only when needed to prove
  an oracle; report zero matches separately.
- Validate document links and `git diff --check`.
- `git status --short`

## Suggested commit message

`Investigate Sprint 18 Graph Query API`

## Final report additions

Report confirmed query and snapshot APIs, compatibility constraints,
fixture/test oracles, unresolved ADR questions, dependency findings, decision
readiness, changed path, validation, commit, and final Git state.
