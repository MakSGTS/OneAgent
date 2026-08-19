# OneAgent Roadmap

## Vision

**OneAgent** — кроссплатформенная платформа интеллектуальной разработки для `1C:Enterprise`, основанная на семантической модели конфигурации, графе знаний и интеграции с локальными и облачными LLM.

## Product roadmap

The sprint execution roadmap below is the single source of truth for execution
order, dependencies, and status. Versions group sprint outcomes into release
boundaries; they do not define a second task sequence.

| Version | Outcome | Sprint range | Status |
|---|---|---:|---|
| v0.1 — Foundation | Workspace, Runtime foundation, discovery, EDT reader, and base metadata model | Sprint 1 | completed |
| v0.2 — Semantic Core | Typed semantic graph, semantic index, and deterministic incremental indexing | Sprints 2–5 | completed |
| v0.3 — 1C Knowledge Model | Broader metadata semantics and Designer XML ingestion | Sprints 6–14 | planned |
| v0.4 — Runtime API | Long-running services, APIs, cache, and a usable CLI client | Sprints 15–21 | planned |
| v0.5 — AI Integration | Context engine and local or OpenAI-compatible LLM providers | Sprints 22–27 | planned |
| v0.6 — MCP and IDE | MCP, VS Code, LSP, EDT, and external AI client integrations | Sprints 28–35 | planned |
| v0.7 — Intelligence | Diagnostics, Git-aware change ingestion, impact, refactoring, and safe edits | Sprints 36–41 | planned |
| v1.0 — Stable Platform | Stable APIs, plugin SDK, hardening, documentation, and release | Sprints 42–46 | planned |

Calendar forecasts are intentionally kept outside this document until capacity,
scope, and release criteria are baselined. Adding a forecast must not duplicate
or override the dependency order recorded in the sprint tables.

The v0.2 boundary is closed with a `pass` decision in the
[v0.2 release review](reviews/v0.2-release-review.md).

## Roadmap reconciliation

The project audit remediation is recorded here as completed governance work:

- [x] Use the sprint execution roadmap as the only execution sequence.
- [x] Remove conflicting fixed Gantt dates and duplicated task ordering.
- [x] Define the Sprint 4 Semantic Index architecture boundary in ADR-0026.
- [x] Give CLI, LSP, Designer XML, and Git integration explicit sprint ownership.
- [x] Align README and architecture documents with implemented and planned scope.
- [x] Replace the stale architecture audit with a current point-in-time audit.
- [x] Add retrospective completion evidence for Sprints 1–2 and the v0.1 release review.

## Release goals

### v0.1 — Foundation
Stable Rust workspace, Runtime composition root, cross-platform CI, EDT workspace discovery, base metadata model.

### v0.2 — Semantic Core
Typed semantic graph, real EDT UUIDs, module/procedure/function nodes, local and cross-module call graph, semantic and incremental indexes.

### v0.3 — 1C Knowledge Model
Attributes, tabular sections, forms, commands, registers, queries, roles, access rights, subsystems, SKD, XDTO, services, and Designer XML ingestion.

### v0.4 — Runtime API
Long-running runtime, workspace lifecycle, file watching, graph-query API, persistent cache, and a usable CLI client over supported runtime contracts.

### v0.5 — AI Integration
Context engine, LLM abstraction, LM Studio, Ollama and OpenAI-compatible endpoints.

### v0.6 — MCP and IDE
MCP server, VS Code extension, LSP adapter, EDT integration, navigation and semantic tools.
OneAgent should expose its semantic graph, query, validation, diagnostics,
impact, and context capabilities through MCP so external AI clients such as
Codex, Cursor, and cloud agents can consume OneAgent without product-specific
core integrations.

### v0.7 — Intelligence
Diagnostics, Git-aware change ingestion, impact analysis, planning, refactoring and safe edit transactions.

### v1.0 — Stable Platform
Stable APIs, plugin SDK, performance/security hardening, documentation and examples.

## Sprint execution roadmap

Versions define product outcomes and release boundaries. Sprints are the primary
execution units inside those versions. Each sprint has one focused goal, explicit
dependencies, implementation and validation work, and an integration review
before it is marked complete. A version closes with a release integration review
after its final sprint; release gates are not separate numbered sprints unless
they acquire independent implementation scope.

Sprint status uses three values:

- `completed` — implementation and integration review are complete;
- `next` — the next planning and kickoff target, but not yet active;
- `planned` — ordered future work whose detailed scope is defined at kickoff.

### Task prompt template readiness forecast

Task prompt templates are a pre-kickoff governance dependency. A stage named
`Task prompt template update required` must be completed before detailed task
decomposition begins for the first affected sprint. These stages do not add a
second execution sequence, change sprint numbering, or become product sprints
unless they later acquire independent implementation scope.

At each stage, audit the live [Codex Framework](codex/README.md) and accepted
architecture first.
Create or revise only the smallest reusable Profile, Workflow, and Template set
that the affected task family needs. If the existing framework already satisfies
the accepted contracts, close the stage with recorded evidence instead of
creating speculative modules. Reassess planned reuse at every later sprint
kickoff because distant scope remains provisional.

| Stage | Required before | Required task-contract coverage | Planned reuse | Status |
|---|---|---|---|---|
| Task prompt template update completed — Semantic Index | Sprint 4 | Read-only investigation, snapshot and incremental index boundaries, query/resolution equivalence, lifecycle and staleness, and sprint integration-review evidence implemented by the [Semantic Index profile](codex/profiles/semantic-index-implementation.md), [Semantic Index template](codex/templates/semantic-index-task.md), [investigation template](codex/templates/investigation-task.md), and [review template](codex/templates/review-task.md). | Sprints 4–5 | completed |
| Task prompt template update required — Source Adapter Ingestion | Sprint 14 | Multi-artifact source discovery and parsing, partial and malformed input, canonical identity equivalence across adapters, and end-to-end adapter conformance. | Sprint 14 | planned |
| Task prompt template update required — Runtime Services and APIs | Sprint 15 | Long-running service lifecycle, ownership, concurrency, cancellation, shutdown, health, transport compatibility, observability, and client/server integration evidence. | Sprints 15–19 and 21; baseline for Sprints 28 and 32 | planned |
| Task prompt template update required — Persistent State | Sprint 20 | Persisted schema ownership, deterministic invalidation, compatibility, corruption handling, migration, recovery, and clean-rebuild equivalence. | Sprint 20 | planned |
| Task prompt template update required — Context Engine | Sprint 22 | Deterministic context selection, provenance, budgets, truncation, relevance evidence, reproducible evaluation, and data-boundary rules. | Sprints 22 and 33 | planned |
| Task prompt template update required — LLM Providers | Sprint 23 | Provider capabilities, request/response compatibility, discovery, secrets, timeouts, retries, cancellation, error taxonomy, and contract tests. | Sprints 23–26 | planned |
| Task prompt template update required — AI Tool Policy | Sprint 27 | Authorization, denial, side-effect classification, confirmation boundaries, audit evidence, failure containment, and policy regression tests. | Sprints 27, 29, and 33 | planned |
| Task prompt template update required — MCP and Protocol Tools | Sprint 28 | Server lifecycle, transport and schema compatibility, capability negotiation, semantic tool contracts, protocol conformance, and external-client evidence. | Sprints 28–29 and 35; protocol baseline for Sprint 32 | planned |
| Task prompt template update required — IDE and Extension Integration | Sprint 30 | Cross-language build and validation, packaging, activation, configuration, connectivity, UI state, editor lifecycle, and integration-test evidence. | Sprints 30–34 | planned |
| Task prompt template update required — Diagnostics and Rules | Sprint 36 | Stable diagnostic identity, severity and configuration, deterministic rule registration and execution, suppression, reporting, and regression evidence. | Sprints 36–37 and 39 | planned |
| Task prompt template update required — Git Change Adapter | Sprint 38 | Repository change-set identity, rename/delete/conflict behavior, ordering, workspace-change equivalence, and the boundary between Git evidence and semantic authority. | Sprint 38 | planned |
| Task prompt template update required — Refactoring and Safe Edits | Sprint 40 | Plan preconditions, conflict detection, preview, atomicity, rollback, reversibility, filesystem safety, and post-edit semantic validation. | Sprints 40–41 | planned |
| Task prompt template update required — API Stability and Plugin SDK | Sprint 42 | Compatibility policy, deprecation, versioning, migration, extension isolation, capability negotiation, SDK examples, and consumer conformance. | Sprints 42–43 | planned |
| Task prompt template update required — Performance and Security | Sprint 44 | Reproducible benchmark baselines, profiling, regression thresholds, threat models, security findings, remediation evidence, and residual-risk acceptance. | Sprint 44 | planned |
| Task prompt template update required — Documentation and Examples | Sprint 45 | Audience and artifact inventory, executable examples, link and snippet validation, documentation builds, and source-to-documentation consistency. | Sprint 45 | planned |
| Task prompt template update required — Release | Sprint 46 | Version and packaging checks, release candidate evidence, artifact publication, rollback, release notes, final acceptance gates, and release decision. | Sprint 46 | planned |

The current architecture, implementation, graph-model, graph-emission, parser,
investigation, Semantic Index, and review contracts are forecast to cover
Sprints 4–13 without another task-template family. Later sprints are covered by
the first applicable stage above plus its planned reuse; every sprint still
performs a focused readiness check at kickoff.

### Completed sprints

| Sprint | Version | Goal | Evidence | Status |
|---|---|---|---|---|
| Sprint 1 — Foundation | v0.1 | Establish the Cargo workspace, quality gates, Runtime foundation, workspace discovery, EDT configuration reader, and metadata domain model. | [Sprint review](reviews/sprint-1-foundation.md), [v0.1 release review](reviews/v0.1-release-review.md) | completed |
| Sprint 2 — Semantic Core Foundation | v0.2 | Establish the typed semantic graph, EDT metadata and module nodes, BSL declaration extraction, and local and cross-module call resolution. | [Sprint review](reviews/sprint-2-semantic-core-foundation.md) | completed |
| Sprint 3 — Semantic Coverage | v0.2 | Audit and complete graph-domain and EDT semantic coverage, close all Critical, High, and Medium gaps, and complete the integration review. | Semantic Coverage Audit and integration-review records below | completed |
| Sprint 4 — Semantic Index | v0.2 | Build the deterministic complete-snapshot index defined by [ADR-0026](adr/0026-semantic-index-boundary.md) and migrate Query and Resolution compatibility facades. | [Sprint review](reviews/sprint-4-semantic-index.md) | completed |
| Sprint 5 — Incremental Indexing | v0.2 | Update the shared semantic index deterministically from canonical graph snapshot changes while retaining unaffected derived lookup state. | [Sprint review](reviews/sprint-5-incremental-indexing.md), [v0.2 release review](reviews/v0.2-release-review.md) | completed |

Sprint 5 is completed under
[ADR-0027](adr/0027-incremental-semantic-index-maintenance.md) with a `pass`
decision. The separate v0.2 release review also records `pass`.

#### Sprint 4 Semantic Index execution plan

This plan implements the accepted boundary in
[ADR-0026](adr/0026-semantic-index-boundary.md). `SemanticGraph` remains the
only canonical owner of nodes, edges, identities, provenance, and validation
state. The Semantic Index is one crate-internal, deterministic, read-only view
derived from a complete borrowed graph snapshot. Querying or constructing it
must not normalize, repair, omit, or reinterpret graph facts.

Current lookup responsibilities and compatibility constraints are:

| Surface | Current responsibility | Sprint 4 constraint |
|---|---|---|
| `SemanticGraph` | Owns nodes in a `BTreeMap` and edges in a `BTreeSet`; node identity lookup uses the map, while node-kind and incoming or outgoing edge operations scan canonical storage. | Remains the semantic authority and public snapshot construction source; direct graph behavior is preserved. |
| `SemanticGraphQuery` | Provides the public source-independent read facade; exact name, node kind, stable edge identity, edge kind, adjacency, and derived ownership operations include scan-based paths. | Remains the public read facade with the same results, ordering, traversal policy, and construction entry points. |
| `SemanticResolutionIndex` | Builds node-id, exact-name, owner-and-child-name, and child-to-owner maps from a complete graph snapshot and returns typed missing, ambiguous, incompatible-kind, and invalid-owner errors. | Becomes a compatibility facade over the shared index representation without changing resolution policy, errors, or candidate ordering. |
| Validation | Scans canonical edges and builds its own ownership view so invalid and multiple-owner states remain observable. | Continues to validate canonical graph facts; the index must not hide or repair invalid states. |
| Diff and build Diff | Compare complete canonical snapshots; Diff independently derives the same stable edge identity used by Query and Validation. | Keep snapshot and change semantics unchanged while using one centralized edge-identity implementation. |
| Impact | Uses Query over previous and current snapshots for node and edge seeds, reverse dependency propagation, and optional ownership propagation. | Query migration must preserve affected nodes, reasons, directions, depth, and deterministic ordering. |
| Coverage | Observes canonical node, edge, query, reference, and provenance capabilities. | No Coverage Registry status or evidence transition belongs to Sprint 4. |
| EDT | Production resolution uses `SemanticResolutionIndex`; production and integration tests use Query identity, kind, edge-kind, adjacency, and ownership operations. | Existing source resolution, graph emission, diagnostics, statistics, and repeated-build results remain unchanged. |

##### Public API and lifecycle compatibility gate

The accepted implementation path requires no public API redesign. Preserve the
public types, method names, parameters, return types, error variants, and
construction entry points `SemanticGraph::query()`, `SemanticGraphQuery::new`,
`SemanticGraph::resolution_index()`, and `SemanticResolutionIndex::new`,
including the current `const` construction capability of the Query entry
points. The facades may own or lazily materialize the shared internal lookup
representation, but callers must not need a new public index type or a new
construction sequence.

Every facade instance observes the complete graph snapshot it borrows. A graph
mutation requires construction of a new facade and derived view; no index state
may survive as authority across snapshots. If repository evidence shows that
this compatibility contract cannot be implemented, stop before changing a
public API and prepare a separate architecture compatibility decision. Do not
combine that decision with an implementation slice.

##### Lookup ownership

Each required ADR-0026 lookup dimension has exactly one implementation owner:

| ADR-0026 lookup dimension | Owning task |
|---|---|
| Node identity | Task 1 — Snapshot identity and classification index |
| Exact canonical node name | Task 1 — Snapshot identity and classification index |
| Node kind | Task 1 — Snapshot identity and classification index |
| Stable edge identity | Task 1 — Snapshot identity and classification index |
| Edge kind | Task 1 — Snapshot identity and classification index |
| Outgoing adjacency by node and by node plus edge kind | Task 2 — Adjacency and containment index |
| Incoming adjacency by node and by node plus edge kind | Task 2 — Adjacency and containment index |
| Containment ownership and owned-child lookup | Task 2 — Adjacency and containment index |

Tasks 3 and 4 migrate compatibility facades to those lookup dimensions; they
must not create alternative maps or policies. Task 5 reviews the integrated
result and owns no new lookup behavior.

All implementation tasks use the
[Semantic Index implementation profile](codex/profiles/semantic-index-implementation.md)
and [Semantic Index task template](codex/templates/semantic-index-task.md).
They run their focused checks first and then the common full implementation
gate:

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
git diff --check
```

##### Task 1 — Snapshot identity and classification index

**Prerequisites:** the public API and lifecycle compatibility gate above is
accepted; ADR-0026 remains accepted; the Semantic Index task-template readiness
stage remains complete.

**Included scope:**

- add one crate-internal complete-snapshot index representation in
  `oneagent-graph`, borrowing canonical nodes and edges rather than copying a
  competing graph model;
- implement deterministic lookup state for node identity, exact
  `GraphNode::name`, node kind, stable edge identity, and edge kind;
- centralize stable edge-id construction so Query, Diff, Validation, and the
  new index cannot drift while preserving the current encoded identity;
- define ordered values by `NodeId` for node results and `EdgeId` for edge
  results; preserve all nodes sharing an exact name and treat missing keys as
  empty or absent without placeholders;
- prove construction equivalence with canonical `SemanticGraph::nodes()` and
  `SemanticGraph::edges()` over representative and empty graphs.

**Excluded scope:** adjacency and containment maps; Query or Resolution lookup
migration; public API changes; graph mutation hooks; benchmarks or performance
targets.

**Acceptance evidence:** focused tests cover an empty graph, missing keys,
duplicate exact names, every represented node and edge kind, canonical edge-id
equivalence, reversed insertion order, repeated construction from the same
snapshot, and unchanged Diff and Validation edge identities. The index exposes
borrowed canonical objects and cannot create or mutate graph facts.

**Focused validation:**

```bash
cargo test -p oneagent-graph --lib semantic_index::tests
cargo test -p oneagent-graph --test diff
cargo test -p oneagent-graph --test validation
```

Run the common full implementation gate after the focused checks.

##### Task 2 — Adjacency and containment index

**Prerequisites:** Task 1 is complete and its identity and classification maps
are the only shared snapshot representation.

**Included scope:**

- extend that representation with outgoing and incoming adjacency keyed by
  node and by node plus `EdgeKind`;
- derive containment owner, all-owner, child, child-kind, and
  owner-plus-exact-child-name lookup data from canonical `Contains` edges and
  the Task 1 node maps;
- retain every canonical `Contains` fact so multiple owners, duplicate local
  child names, wrong-owner references, self-loops, and other invalid ownership
  states remain observable to Resolution and Validation;
- preserve `EdgeId` ordering for adjacency, `NodeId` ordering for owner and
  child results, and empty results for unknown nodes or absent kinds.

**Excluded scope:** Query and Resolution facade migration; ownership
normalization or validation policy changes; traversal policy changes;
incremental updates.

**Acceptance evidence:** focused tests cover outgoing and incoming lookups with
multiple edge kinds, node-plus-kind filtering, unknown nodes, empty snapshots,
self-loops, multiple owners, same-named children under different owners,
same-named children under one owner, wrong-owner membership, reversed insertion
order, and repeated construction. Every result is equivalent to a canonical
scan of the same graph snapshot.

**Focused validation:**

```bash
cargo test -p oneagent-graph --lib semantic_index::tests
cargo test -p oneagent-graph --test validation
```

Run the common full implementation gate after the focused checks.

##### Task 3 — Resolution compatibility migration

**Prerequisites:** Tasks 1 and 2 are complete; their shared snapshot
representation covers all lookup state currently derived by
`SemanticResolutionIndex`.

**Included scope:**

- make `SemanticResolutionIndex` delegate node-id, exact-name,
  owner-and-child-name, child-to-owner, and kind-filtered resolution to the
  shared representation;
- preserve every public constructor and resolver signature, borrowed return,
  `ResolutionError` variant, candidate list, and deterministic candidate order;
- add explicit pre-replacement equivalence evidence for successful, missing,
  ambiguous, incompatible-kind, missing-owner, multiple-owner, and
  invalid-owner outcomes;
- verify EDT production consumers for metadata references, Includes, and
  Grants retain graph emission, diagnostics, resolution statistics, and
  repeated-build behavior.

**Excluded scope:** Query scan migration; new resolution rules; name
normalization; adapter-specific index state; changes to graph or EDT semantic
facts.

**Acceptance evidence:** existing Resolution tests remain green and focused
tests add any missing empty-snapshot, missing-owner, multiple-owner, repeated
construction, and indexed-versus-canonical cases. The complete EDT test suite
passes without changing accepted source or semantic contracts.

**Focused validation:**

```bash
cargo test -p oneagent-graph --lib resolution::tests
cargo test -p oneagent-edt
```

Run the common full implementation gate after the focused checks.

##### Task 4 — Query migration and consumer equivalence

**Prerequisites:** Task 3 is complete; Resolution compatibility has proved that
the shared representation preserves name and ownership policy.

**Included scope:**

- keep `SemanticGraphQuery` as the public source-independent facade while
  delegating node identity, exact name, node kind, stable edge identity, edge
  kind, outgoing and incoming adjacency, and containment operations to the
  shared representation;
- preserve exact results and ordering for `node`, `node_by_entity_id`,
  `contains_node`, name and kind queries, `edge`, `contains_edge`, edge-kind
  queries, incoming and outgoing queries, owners, owner edges, children, and
  children by kind;
- preserve neighbor, dependency, usage, bounded traversal, cycle, self-loop,
  deduplication, and edge-filter behavior that composes those primitive
  lookups;
- establish indexed-versus-canonical scan equivalence before removing eligible
  scan paths, including representative and empty graphs, and prove repeated
  Query construction returns the same ordered results;
- verify Impact and EDT Query consumers without changing their public behavior.

**Excluded scope:** changes to Query result semantics or dependency-edge
policy; removal of canonical graph iteration APIs; migration of unrelated
Validation, Diff, Coverage, or EDT logic to an index; new query operations;
performance targets without a benchmark baseline.

**Acceptance evidence:** the Query suite covers every migrated primitive and
the derived traversal behavior for empty, missing, duplicate-name,
multiple-owner, invalid-owner, insertion-order, cycle, self-loop, and repeated
construction cases. Query and Resolution equivalence is recorded before any
scan implementation is removed. Query, Impact, Validation, Diff, build Diff,
Coverage, and EDT suites remain green with unchanged observable results.

**Focused validation:**

```bash
cargo test -p oneagent-graph --test query
cargo test -p oneagent-graph --lib resolution::tests
cargo test -p oneagent-graph --test impact
cargo test -p oneagent-graph --test validation
cargo test -p oneagent-graph --test diff
cargo test -p oneagent-graph --test build_diff
cargo test -p oneagent-graph --test coverage
cargo test -p oneagent-edt
```

Run the common full implementation gate after the focused checks.

##### Task 5 — Sprint 4 Semantic Index integration review

**Prerequisites:** Tasks 1 through 4 are complete in dependency order with
their focused and full validation evidence. Record the Sprint 4 activation
baseline and review the complete implementation diff from that baseline.

Use the [Review profile](codex/profiles/review.md) and
[Review task template](codex/templates/review-task.md). This task owns no
production implementation. Its prompt may explicitly authorize creation of
`docs/reviews/sprint-4-semantic-index.md` and the corresponding Roadmap status
update only after a `pass` completion decision.

**Included scope:**

- verify one canonical `SemanticGraph`, one shared derived snapshot
  representation, borrowed canonical objects, and rebuild-after-mutation
  lifecycle;
- trace every ADR-0026 lookup dimension to its single owning task and executed
  focused evidence;
- verify Query public construction and results, Resolution typed behavior,
  Validation visibility of invalid facts, Diff identity and snapshot behavior,
  Impact propagation, Coverage invariants, and EDT production and integration
  behavior;
- verify empty, missing, duplicate-name, ambiguous, invalid-ownership,
  insertion-order, and repeated-construction evidence and confirm Query and
  Resolution equivalence preceded scan replacement;
- confirm Sprint 5 incremental maintenance and all persistence, Runtime,
  transport, source-adapter, IDE, and unsupported search concerns remain
  deferred.

**Excluded scope:** fixing findings, changing production code, adding lookup
dimensions, changing Coverage Registry status, or accepting missing evidence as
implementation success. Blocking findings return the task to the owning
implementation slice.

**Acceptance evidence:** record exact commands, test counts, zero-match
filters, findings, missing evidence, scope conformance, and one decision:
`pass`, `pass with non-blocking follow-ups`, or `blocked`. Sprint 4 completion
requires `pass`; only then record the review and change Sprint 4 from `active`
to `completed`.

**Focused validation:** execute the complete Task 4 focused matrix against the
reviewed baseline, then run the common full implementation gate.

##### Sprint 4 state gates

Sprint 4 is `completed`. Tasks 1 through 4 are committed in dependency order,
Task 5 issued `pass`, all focused and full validation commands executed
successfully, the integration review is recorded, public compatibility is
preserved, every ADR-0026 lookup dimension has equivalence evidence, and no
Sprint 5 or later concern was pulled forward.

Incremental maintenance, invalidation, structural sharing, and retained state
across graph mutations remain owned exclusively by Sprint 5. Persistence,
cache formats, Runtime services, HTTP, CLI, MCP, LSP, IDE integration,
source-adapter-specific indexing, fuzzy or ranked search, and unsupported
performance targets remain assigned to later accepted work.

#### Sprint 5 Incremental Indexing execution plan

Sprint 5 implements
[ADR-0027](adr/0027-incremental-semantic-index-maintenance.md) as eight ordered,
atomic outcomes. The work reuses `SemanticGraphDiff`, retains unaffected owned
lookup membership, preserves the Sprint 4 public Query and Resolution surface,
and proves equality with a clean full rebuild. It does not add source events,
persistence, Runtime services, or a second semantic model.

Tasks 2 through 7 use the semantic-index Codex profile, task template, and
workflow. Task 8 uses the integration-review profile and template. Each task
has one commit boundary and must begin from the successful commit produced by
its predecessor.

| Task | Atomic outcome | Primary ownership |
|---|---|---|
| 1 | Accept the architecture and executable plan. | ADR, lifecycle boundary, task ownership, and state gates. |
| 2 | Normalize canonical snapshot changes deterministically. | Change vocabulary, ordering, dependency validation, and typed normalization failures. |
| 3 | Maintain node lookup dimensions incrementally. | Identity, exact-name, node-kind, and node replacement projections. |
| 4 | Maintain edge, adjacency, and containment dimensions incrementally. | Edge identity/kind, endpoint indexes, containment, and incident-edge invalidation. |
| 5 | Integrate accepted state with the index lifecycle, Query, and Resolution. | Freshness, atomic publication, retry, fallback, and public compatibility. |
| 6 | Prove full-rebuild equivalence. | Independent oracle and transition/sequence matrix for every Sprint 4 dimension. |
| 7 | Complete downstream consumer evidence. | Validation, Diff, Impact, Coverage, reference-request build, and EDT compatibility. |
| 8 | Review the integrated Sprint 5 baseline. | Scope audit, complete gates, findings, Sprint decision, and v0.2 hand-off. |

The common full implementation gate for every Rust-changing task is:

```bash
cargo fmt --all --check
cargo check --workspace --all-targets
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo doc --workspace --no-deps
git diff --check
```

##### Task 1 — Accept the architecture and executable plan

**Prerequisites:** Sprint 4 is completed with a recorded `pass`; ADR-0026 and
the shared semantic-index implementation are the accepted baseline; the full
workspace gate passes before Sprint 5 changes begin.

**Included scope:** accept ADR-0027; define canonical previous/current/diff
input, owned retained state, operation ordering, invalidation, freshness,
failure/retry, fallback, lifecycle integration, rebuild equivalence, complexity
expectations, task ownership, and Sprint state gates; publish this eight-task
plan.

**Excluded scope:** Rust implementation, public API changes, benchmarks,
persistence, source adapters, Runtime, transport, and future semantic-model
expansion.

**Acceptance evidence:** ADR-0027 is `Accepted`; every required architecture
decision and rejected alternative is explicit; this plan gives each outcome a
prerequisite, scope boundary, evidence requirement, focused validation, and
commit boundary; no prompt-framework change is required by repository
evidence.

**Focused validation:** inspect the ADR and roadmap links and headings, run
`git diff --check`, review `git diff -- docs/Roadmap.md
docs/adr/0027-incremental-semantic-index-maintenance.md`, and confirm with
`git status --short` that no unrelated path is staged.

**Commit boundary:** commit only the ADR and roadmap as
`Plan Sprint 5 incremental indexing`.

##### Task 2 — Normalize canonical snapshot changes deterministically

**Prerequisites:** Task 1 is committed; ADR-0027 is the accepted authority;
the baseline full gate remains green.

**Included scope:** add a crate-internal normalization boundary over an exact
previous graph, current graph, and canonical `SemanticGraphDiff`; represent
unique node/edge removal, addition, replacement, and refresh work; impose the
accepted total phase and stable-id order; validate supplied-diff freshness,
old/new entity presence, endpoint completeness, incident-edge deletion, stable
identity consistency, no-op transitions, cancellation, and deterministic
retry input without mutating lookup state.

**Excluded scope:** applying operations to index maps, Query or Resolution
integration, consumer changes, source events, persistence, and performance
claims.

**Acceptance evidence:** focused unit tests cover empty, no-op, node and edge
change classes, mixed/reversed construction, deterministic order, cancellation,
wrong-pair or contradictory supplied diffs, missing dependencies, and repeated
normalization. Typed failures leave all inputs untouched.

**Focused validation:** run `cargo test -p oneagent-graph --lib
incremental_index::tests`, `cargo test -p oneagent-graph --test diff`,
`cargo test -p oneagent-graph --test build_diff`, then the common full gate.

**Commit boundary:** commit normalization and its tests as
`Add deterministic incremental index changes`.

##### Task 3 — Maintain node lookup dimensions incrementally

**Prerequisites:** Task 2 is committed and its normalized operation ordering is
stable.

**Included scope:** introduce the owned semantic-index state and incrementally
apply node identity, exact-name, and node-kind projection changes; retain
unaffected entries; correctly remove old and add new name/kind keys for node
replacement; keep payload/provenance-only refreshes lookup-neutral; expose
crate-internal comparison helpers needed by later oracle tests.

**Excluded scope:** edge identity, adjacency, containment, public lifecycle
integration, consumer migration, persistence, and optimization guarantees.

**Acceptance evidence:** tests cover node add/remove, duplicate exact names,
name and kind replacement, combined replacement, payload/provenance refresh,
empty buckets, missing keys, reversed construction, mixed node batches, stale
or invalid input, retry, and preservation of unaffected node projections.

**Focused validation:** run `cargo test -p oneagent-graph --lib
incremental_index::tests`, `cargo test -p oneagent-graph --lib
semantic_index::tests`, `cargo test -p oneagent-graph --test query`, then the
common full gate.

**Commit boundary:** commit node-state maintenance and tests as
`Update semantic node indexes incrementally`.

##### Task 4 — Maintain edge, adjacency, and containment dimensions incrementally

**Prerequisites:** Task 3 is committed; node projection application and
normalization dependency checks are green.

**Included scope:** apply edge identity and kind changes, incoming/outgoing and
kind-filtered adjacency, containment owner/child/name/kind projections,
provenance refresh, endpoint/kind replacement, incident-edge deletion, and
containment rekeying after node name/kind replacement. Retain unaffected edge
and containment membership.

**Excluded scope:** public lifecycle constructors, downstream consumer
migration, new validation policy, persistence, Runtime, and benchmark targets.

**Acceptance evidence:** tests cover edge add/remove/refresh/replacement,
self-loops, cycles, duplicate names, multiple owners, invalid ownership,
node deletion with incident edges, containment child name/kind replacement,
mixed batches, deterministic order, stale failure, retry, and unaffected
membership preservation.

**Focused validation:** run `cargo test -p oneagent-graph --lib
incremental_index::tests`, `cargo test -p oneagent-graph --lib
semantic_index::tests`, and the graph integration targets `validation`, `diff`,
`query`, and `resolution`, then the common full gate.

**Commit boundary:** commit edge-state maintenance and tests as
`Update semantic edge indexes incrementally`.

##### Task 5 — Integrate lifecycle, Query, and Resolution

**Prerequisites:** Task 4 is committed and every Sprint 4 lookup dimension can
be represented by the owned state.

**Included scope:** pair accepted state with its exact graph snapshot; build a
clean state, apply a complete normalized transition atomically, validate before
publication, preserve the previous state after failure, distinguish retry from
stale-base application, provide clean-rebuild fallback, and let crate-internal
Query and Resolution construction reuse accepted state without changing their
public constructors, signatures, ordering, covariance, or typed failures.

**Excluded scope:** public mutable indexes, lazy graph-borrowing cells, Runtime
publication, async orchestration, source adapters, persistence, and new Query or
Resolution semantics.

**Acceptance evidence:** lifecycle tests prove clean build, successful
transition, no-op current-to-current, deterministic retry, already-current and
unrelated stale failure, wrong target, failure followed by retry, unchanged
previous state, fallback, borrowed canonical results, and compatibility of all
existing public entry points.

**Focused validation:** run focused incremental-index unit tests and graph
integration targets `query`, `resolution`, and `validation`, plus EDT tests,
then the common full gate.

**Commit boundary:** commit lifecycle and facade integration as
`Integrate incremental semantic index lifecycle`.

##### Task 6 — Prove full-rebuild equivalence

**Prerequisites:** Task 5 is committed; both clean-build and incremental
lifecycle paths are available from crate-internal tests.

**Included scope:** construct an independent clean full-rebuild oracle; compare
all Sprint 4 state dimensions, Query primitives and derived results, Resolution
successes and typed failures, ordering, and invalid-state visibility after each
single transition and every step of supported multi-step sequences. Include
previous/current/missing key universes so stale entries are observable.

**Excluded scope:** using incremental invalidation helpers to calculate
expected results, benchmark claims, persistence, new public APIs, and future
semantic facts.

**Acceptance evidence:** the matrix covers empty/no-op; every node and edge
change class; adjacency and containment changes; duplicate names; multiple
owners; invalid ownership; self-loops; cycles; mixed/reversed batches; incident
deletion; retry; stale failure; failure/retry; and multi-step sequences, with
the clean graph constructed independently.

**Focused validation:** run incremental unit tests and graph integration
targets `query`, `resolution`, `validation`, `diff`, and `build_diff`, then the
common full gate.

**Commit boundary:** commit the oracle and equivalence matrix as
`Prove incremental index rebuild equivalence`.

##### Task 7 — Complete downstream consumer evidence

**Prerequisites:** Task 6 is committed and full-rebuild equivalence is green.

**Included scope:** audit and, only where necessary, adapt Validation, Diff,
Impact, Coverage, reference-request building, and EDT so canonical ownership and
their accepted public behavior remain unchanged while Query and Resolution use
the shared lifecycle consistently. Add integration evidence for clean and
incremental graph sequences without moving invalidation policy into consumers.

**Excluded scope:** new consumer policy, source-specific change generation,
Runtime services, transports, persistence, public API expansion, and unrelated
refactors.

**Acceptance evidence:** focused tests prove Validation still reads canonical
facts, Diff/build Diff stay directional snapshot comparisons, Impact uses the
matching previous/current Query views, Coverage and reference requests preserve
ordering and missing/invalid behavior, EDT remains a producer, and no consumer
owns index invalidation.

**Focused validation:** run incremental unit tests; graph targets `query`,
`resolution`, `validation`, `diff`, `build_diff`, `impact`, `coverage`, and
`reference_request_build`; EDT tests; then the common full gate.

**Commit boundary:** commit only required consumer integration and evidence as
`Complete incremental index integration evidence`.

##### Task 8 — Review the integrated Sprint 5 baseline

**Prerequisites:** Tasks 1 through 7 are committed in order; their focused and
full gates are green; the worktree contains no unexplained Sprint changes.

**Included scope:** review architecture conformance, normalized input and
ordering, every invalidation dimension, freshness and atomic failure behavior,
retry and fallback, public compatibility, rebuild equivalence, downstream
ownership, tests, documentation, commit boundaries, and Sprint/v0.2 readiness.
Record findings and exact command outcomes in a Sprint 5 integration-review
artifact; update this roadmap only when the evidence permits the state change.

**Excluded scope:** silently fixing material findings inside the review commit,
new features, broad refactors, benchmark claims, persistence, Runtime, and
future-sprint work. A material finding keeps the Sprint blocked for a dedicated
fix task and rerun.

**Acceptance evidence:** record exact commands, test counts, zero-match
filters, findings, missing evidence, scope conformance, and one decision:
`pass`, `pass with non-blocking follow-ups`, or `blocked`. Sprint 5 completion
requires `pass`; the separate v0.2 release review remains the next decision.

**Focused validation:** execute the complete Task 7 focused matrix against the
reviewed baseline, then run the common full implementation gate.

**Commit boundary:** commit the review artifact and justified roadmap state as
`Complete Sprint 5 incremental indexing review`.

##### Sprint 5 state gates

Sprint 5 is `completed`. Tasks 1 through 7 are committed in dependency order,
Task 8 records `pass`, all focused and full validation commands completed
successfully, every ADR-0027 lookup dimension has independent rebuild
equivalence evidence, public compatibility is preserved, and no later-sprint
concern was pulled forward.

The separate v0.2 release integration review records `pass`, so the explicit
version status is `completed`.

Persistence, cache serialization, cross-process identity, Runtime services,
async publication, filesystem/Git/workspace watchers, HTTP, CLI, MCP, LSP, IDE
integration, source-specific invalidation, new semantic facts, and unsupported
performance claims remain deferred.

### Planned sprints

#### v0.3 — 1C Knowledge Model

| Sprint | Goal | Status |
|---|---|---|
| Sprint 6 — Attributes and Tabular Sections | Expand the knowledge model for attributes, tabular sections, ownership, identity, and references. | planned |
| Sprint 7 — Forms and Commands | Model forms, commands, their ownership, references, and navigation semantics. | planned |
| Sprint 8 — Registers and Queries | Expand register and query-language semantics, additional Query sources, and justified data dependencies. | planned |
| Sprint 9 — Roles and Access Rights | Expand authorization modeling beyond the accepted Grants first slice where architecture evidence supports it. | planned |
| Sprint 10 — Subsystems and Composition | Add hierarchy, nested discovery, composition, and transitive membership contracts where justified. | planned |
| Sprint 11 — Event Subscriptions | Model event subscriptions, handlers, references, and resulting semantic relations. | planned |
| Sprint 12 — SKD and Report Model | Add data-composition and report-specific semantic entities and relations. | planned |
| Sprint 13 — XDTO and Service Model | Expand XDTO, HTTP service, and Web service semantics beyond top-level metadata-node coverage. | planned |
| Sprint 14 — Designer XML Adapter | Ingest supported Designer XML configuration dumps through a source adapter without changing canonical semantic identities. | planned |

The v0.3 release integration review follows Sprint 14.

#### v0.4 — Runtime API

| Sprint | Goal | Status |
|---|---|---|
| Sprint 15 — Runtime Service Container | Establish the long-running runtime composition and service lifecycle. | planned |
| Sprint 16 — HTTP API and Health | Expose the runtime through an HTTP API with health and readiness behavior. | planned |
| Sprint 17 — Workspace Service | Add workspace lifecycle and semantic-build orchestration services. | planned |
| Sprint 18 — Graph Query API | Expose stable graph and semantic query capabilities through the runtime API. | planned |
| Sprint 19 — File Watching | Detect workspace changes and connect them to runtime update orchestration. | planned |
| Sprint 20 — Persistent Cache | Persist validated semantic state with deterministic invalidation. | planned |
| Sprint 21 — CLI Client | Replace the CLI placeholder with a supported client for runtime workspace and graph-query operations. | planned |

The v0.4 release integration review follows Sprint 21.

#### v0.5 — AI Integration

| Sprint | Goal | Status |
|---|---|---|
| Sprint 22 — Context Engine | Build deterministic semantic context selection and assembly. | planned |
| Sprint 23 — LLM Provider Abstraction | Define provider-independent model, request, response, and capability contracts. | planned |
| Sprint 24 — OpenAI-Compatible Provider | Implement the first OpenAI-compatible provider integration. | planned |
| Sprint 25 — LM Studio Integration | Add local LM Studio discovery and execution support. | planned |
| Sprint 26 — Ollama Integration | Add local Ollama discovery and execution support. | planned |
| Sprint 27 — Tool Execution Policy | Define and enforce safe AI tool execution boundaries. | planned |

The v0.5 release integration review follows Sprint 27.

#### v0.6 — MCP and IDE

| Sprint | Goal | Status |
|---|---|---|
| Sprint 28 — MCP Server | Establish the MCP server, lifecycle, and transport boundary. | planned |
| Sprint 29 — MCP Semantic Tools | Expose graph, query, validation, diagnostics, impact, and context capabilities through MCP. | planned |
| Sprint 30 — VS Code Extension Foundation | Establish extension packaging, activation, configuration, and runtime connectivity. | planned |
| Sprint 31 — Navigation and Symbol Search | Add semantic navigation and symbol-search experiences. | planned |
| Sprint 32 — LSP Adapter | Expose supported navigation, symbol, and diagnostic capabilities through an editor-neutral LSP boundary. | planned |
| Sprint 33 — AI Chat and Context Panel | Add IDE chat and inspectable semantic context UI. | planned |
| Sprint 34 — EDT Integration Prototype | Prove the EDT integration boundary and user workflow. | planned |
| Sprint 35 — External AI Client Compatibility | Validate Codex, Cursor, and other MCP-capable client workflows. | planned |

The v0.6 release integration review follows Sprint 35.

#### v0.7 — Intelligence

| Sprint | Goal | Status |
|---|---|---|
| Sprint 36 — Diagnostics Engine | Build semantic diagnostic orchestration and reporting. | planned |
| Sprint 37 — Rules Engine | Define deterministic rule registration, execution, and result contracts. | planned |
| Sprint 38 — Git Change Adapter | Convert repository change sets into deterministic workspace change inputs without making Git a semantic authority. | planned |
| Sprint 39 — Change Impact Analysis | Expand impact analysis into a product-facing workflow. | planned |
| Sprint 40 — Refactoring Planner | Produce validated semantic refactoring plans. | planned |
| Sprint 41 — Safe Edit Transactions | Apply planned edits through checked, reversible transactions. | planned |

The v0.7 release integration review follows Sprint 41.

#### v1.0 — Stable Platform

| Sprint | Goal | Status |
|---|---|---|
| Sprint 42 — Public API Stabilization | Stabilize supported public APIs and compatibility policy. | planned |
| Sprint 43 — Plugin SDK | Define and publish the supported extension SDK. | planned |
| Sprint 44 — Performance and Security Hardening | Complete profiling, performance, threat-model, and security hardening work. | planned |
| Sprint 45 — Documentation and Examples | Complete user, operator, contributor, and API documentation with examples. | planned |
| Sprint 46 — OneAgent 1.0 Release | Complete final release validation, packaging, and publication. | planned |

The v1.0 release integration review and release decision are part of Sprint 46.

Deferred Sprint 3 scope is not implicitly promoted by this schedule. Query-derived
`DependsOn` and broader Query sources belong to Sprint 8 only after an accepted
contract; deny, inheritance, and effective authorization belong to Sprint 9;
Subsystem hierarchy and transitive membership belong to Sprint 10; other
reference-request families migrate in the sprint that owns their source contract.

## Sprint 3 Semantic Coverage

- [x] Add a deterministic Semantic Coverage Audit for graph-domain and EDT-specific capabilities.
- [x] Complete Semantic Coverage; the audit does not mark completion itself.
- [x] Complete Sprint 3 Semantic Coverage Integration Review with no blocking findings.

Ordered follow-up work:

1. **Critical — completed:** retain unresolved BSL calls as typed diagnostics and resolution statistics.
2. **High — completed:** discover and emit top-level Common Command metadata (`metadata_entity.command`); typed payload preservation is also complete.
3. **High — completed:** classify generic top-level Form entity and node capabilities as not applicable to EDT; real common and subordinate forms use distinct semantic kinds.
4. **High — completed:** discover and emit top-level Common Template metadata (`metadata_entity.template`); typed payload preservation is also complete.
5. **High — completed:** classify fallback-only `metadata_entity.unknown` as not applicable to EDT without emitting synthetic entities.
6. **High — completed:** map EDT accounting-register resources to stable, provenance-backed `Measure` nodes (`semantic_node.measure`).
7. **High — completed:** classify fallback-only `semantic_node.metadata.unknown` as not applicable to EDT without emitting synthetic metadata nodes.
8. **High — completed:** emit static BSL Query declarations as stable, provenance-backed `NodeKind::Query` nodes; the accepted first query-language parsing and `Reads` slice is completed separately in item 20, while query-derived `DependsOn` and broader query-language support remain deferred.
9. **High — completed:** emit flat EDT role semantic nodes while preserving `NodeKind::Metadata(MetadataKind::Role)` object nodes.
10. **High — completed:** derive EDT document `StandardAttribute` nodes with stable identity, ownership, and provenance.
11. **High — completed:** emit flat EDT `Subsystem` semantic nodes while preserving `NodeKind::Metadata(MetadataKind::Subsystem)` object nodes.
12. **High — completed:** classify fallback-only flat `semantic_node.unknown` as not applicable to EDT without emitting synthetic unknown nodes.
13. **High — completed:** recognize EDT accounting-register `Measure` ownership through the existing `Contains` edge from the owning metadata object.
14. **High — completed:** recognize EDT document `StandardAttribute` ownership through the existing `Contains` edge from the owning metadata object.
15. **High — completed:** implement the first production slice for declared `DependsOn` semantic edges using the accepted contract in `docs/adr/0017-depends-on-semantics.md`.
16. **High — completed:** preserve immediate tabular-section ownership for nested attributes with owner-scoped fallback identity, provenance-backed `Contains` edges, generic Query navigation, and deterministic production integration evidence.
17. **High — completed:** implement the first production slice for declared `Extends` semantic edges using the accepted contract in `docs/adr/0018-extends-semantics.md`.
18. **High — completed:** implement the first production slice for declared `Grants` semantic edges using the accepted contract in `docs/adr/0019-grants-semantics.md`; EDT role-right declarations now resolve to scoped `AccessRight` nodes and canonical Grants edges with deterministic provenance.
19. **High — completed:** implement the direct top-level EDT Subsystem `<content>` slice for `Includes` using `docs/adr/0020-includes-semantics.md`; the production builder now normalizes the explicit allowlist, resolves exact metadata targets, emits deterministic provenance-backed Includes edges, reports typed negative outcomes, enforces the precise validator matrix, and verifies generic queries and Impact exclusion.
20. **High — completed:** implement and transition the first `Reads` slice defined by `docs/adr/0021-reads-semantics.md`; parsing, multiline BSL decoding and private source mapping, typed positive and negative classification, exact resolution, validation, emission, provenance, raw fixtures, deterministic parser/full-builder tests, and `semantic_edge.reads` Coverage evidence are complete.
21. **High — completed:** implement and transition the first `Writes` slice defined by `docs/adr/0022-writes-semantics.md`; typed Document register declarations, complete zero-argument `RegisterRecords.<Name>.Write()` candidate extraction in Document Object Module Procedures, exact declaration and metadata resolution, precise validation, deterministic provenance-backed production emission, typed diagnostics, Query and Impact behavior, negative, duplicate, and repeated-build evidence, and `semantic_edge.writes` Coverage evidence are complete. Writes and query-derived `DependsOn` are not inferred from Reads, Calls, Grants, declarations alone, or the bare method name.
22. **Medium — completed:** the
    typed metadata payload contract from
    `docs/adr/0023-typed-metadata-payload.md` is implemented across the metadata
    domain, graph equality/diff/Query integration, common EDT synonym and typed
    Document register-record conversion, and complete deterministic per-kind
    production evidence. All 21 applicable EDT metadata-entity capabilities now
    have complete evidence and `Supported` status; Form and Unknown remain
    `NotApplicable`.
23. **Medium — completed:** add successful production-builder evidence for all
    nine mapped metadata reference target kinds; Catalog, Document,
    Enumeration, Information Register, Accumulation Register, Accounting
    Register, Calculation Register, Business Process, and Task references now
    share one deterministic representative integration test covering exact
    targets, `References`, companion `DependsOn`, provenance, Query, validation,
    resolution statistics, and repeated builds.
24. **Medium — completed:** implement the public source-independent
    reference-request ledger defined by
    `docs/adr/0024-reference-request-provenance.md`; graph-domain identity,
    lifecycle, Query/report/diff/validation integration, EDT metadata-reference
    migration, collection-time provenance, projection consistency, production
    evidence, and independent graph-domain and EDT Coverage transitions are
    complete for the accepted metadata-reference first slice.
25. **Medium — completed:** implement
    `docs/adr/0025-references-endpoint-validation.md`; `References` now accepts
    exactly the 27 metadata-member pairs and five AccessRight resource pairs
    emitted by current production, with exhaustive deterministic positive and
    negative validator evidence. All nine EdgeKind rules now match their
    accepted ownership or first-slice contracts. Coverage status and counts are
    unchanged.

The EDT Coverage Registry currently contains 0 Critical gaps, 0 High gaps, and
0 Medium gaps. Combined with the Graph Domain registry, Semantic Coverage
contains 0 Critical gaps, 0 High gaps, and 0 Medium gaps. Sprint 3 Semantic
Coverage Integration Review is complete with no blocking findings.

Completion does not broaden the accepted first-slice contracts. Deferred work
remains: query-derived `DependsOn` and broader query-language source forms;
deny, inheritance, and effective authorization; Subsystem hierarchy, nested
Subsystem discovery, and transitive membership; and reference-request migration
for BSL calls, query sources, Writes targets, protected resources, Subsystem
content, and extension targets.

The first production slice for `semantic_edge.depends_on` is implemented and
the capability is supported. The first production slice for
`semantic_edge.extends` and `semantic_edge.grants` are implemented and
supported. The first production slice for `semantic_edge.includes` is also
implemented and supported: direct `<content>` observations from discovered
top-level Subsystem descriptors are normalized through the explicit allowlist,
resolved by exact metadata kind and name, emitted as canonical Includes edges,
and covered by precise validation, typed failures, deterministic provenance,
generic query, and Impact-exclusion evidence.

The first `semantic_edge.reads` architecture contract is accepted and the EDT
pipeline emits canonical provenance-backed Reads edges from existing Query nodes
for one completely parsed top-level `SELECT` with one direct Catalog or
Information Register source. Parser investigation, typed diagnostics, exact
resolution, precise endpoint validation, emission, Query and Impact behavior,
negative outcomes, and repeated-build production evidence are complete for the
accepted forms. The confirmed multiline BSL decoder and private source map,
explicit unsupported-structure, virtual-table, and temporary-table diagnostics,
raw fixtures, deterministic parser/full-builder negative evidence, and the
registry-only Coverage transition are complete. The capability is `Supported`.
The first `semantic_edge.writes` architecture contract is accepted in
`docs/adr/0022-writes-semantics.md`. Its canonical direction is
`Procedure --Writes--> Metadata(AccumulationRegister)` for the exact
Document-register source contract; file, binary, text, archive, UI, external,
dynamic, local-object, argument-bearing, and otherwise unresolved writes remain
outside the first slice. Typed Document register declarations, complete
candidate extraction, exact resolution, precise validation, production
emission, deterministic provenance, typed diagnostics, integration, Query,
Impact, negative, duplicate, and repeated-build evidence, and the registry-only
Coverage transition are complete. The capability is `Supported`. No deferred
Writes family or write-derived `DependsOn` origin is added by this transition.

The detailed capability inventory, missing evidence, acceptance criteria, and
out-of-scope boundaries are recorded in `docs/architecture/semantic-model-2.md`.

## Definition of Done

- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo doc --workspace --no-deps`
- Architecture changes documented by ADR.
- Public APIs documented.
- Tests cover success and failure paths.
