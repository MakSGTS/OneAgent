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
| Task prompt template update completed — Sequential Sprint Planning and Execution | Sprint 7 | Live readiness audit, explicit prerequisite gates, dependency-ordered task manifests, current-instruction commit authorization, already-complete evidence, failure stopping, integration-review transitions, and final repository-state reporting implemented by the [Sprint planning template](codex/templates/sprint-planning-task.md), [Sprint execution-loop template](codex/templates/sprint-execution-loop.md), [sequential execution workflow](codex/workflows/sequential-sprint-execution.md), and updated task templates. | Sprints 7–46 | completed |
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
investigation, Semantic Index, review, sprint-planning, and sequential-execution
contracts are forecast to cover Sprints 4–13 without another domain task-template
family. Later sprints are covered by the first applicable stage above plus its
planned reuse; every sprint still performs a focused readiness check at kickoff.

### Completed sprints

| Sprint | Version | Goal | Evidence | Status |
|---|---|---|---|---|
| Sprint 1 — Foundation | v0.1 | Establish the Cargo workspace, quality gates, Runtime foundation, workspace discovery, EDT configuration reader, and metadata domain model. | [Sprint review](reviews/sprint-1-foundation.md), [v0.1 release review](reviews/v0.1-release-review.md) | completed |
| Sprint 2 — Semantic Core Foundation | v0.2 | Establish the typed semantic graph, EDT metadata and module nodes, BSL declaration extraction, and local and cross-module call resolution. | [Sprint review](reviews/sprint-2-semantic-core-foundation.md) | completed |
| Sprint 3 — Semantic Coverage | v0.2 | Audit and complete graph-domain and EDT semantic coverage, close all Critical, High, and Medium gaps, and complete the integration review. | Semantic Coverage Audit and integration-review records below | completed |
| Sprint 4 — Semantic Index | v0.2 | Build the deterministic complete-snapshot index defined by [ADR-0026](adr/0026-semantic-index-boundary.md) and migrate Query and Resolution compatibility facades. | [Sprint review](reviews/sprint-4-semantic-index.md) | completed |
| Sprint 5 — Incremental Indexing | v0.2 | Update the shared semantic index deterministically from canonical graph snapshot changes while retaining unaffected derived lookup state. | [Sprint review](reviews/sprint-5-incremental-indexing.md), [v0.2 release review](reviews/v0.2-release-review.md) | completed |
| Sprint 6 — Attributes and Tabular Sections | v0.3 | Preserve repository-proven optional Attribute and TabularSection synonym as typed member content without changing identity, immediate ownership, or the completed reference slice. | [Sprint review](reviews/sprint-6-attributes-tabular-sections.md) | completed |
| Sprint 7 — Forms and Commands | v0.3 | Add accepted Form and Command modules, mapped Command parameter references, and precise static Form navigation without expanding the deferred UI model. | [Sprint review](reviews/sprint-7-forms-commands.md) | completed |
| Sprint 8 — Registers and Queries | v0.3 | Expand direct persistent Query sources through public requests, retained Reads, and normalized Query dependencies without broadening the accepted grammar. | [Sprint review](reviews/sprint-8-registers-queries.md) | completed |
| Sprint 9 — Roles and Access Rights | v0.3 | Preserve optional opaque EDT row restrictions as typed conditional direct Grants without changing unconditional AccessRight compatibility or claiming effective authorization. | [Sprint review](reviews/sprint-9-roles-access-rights.md) | completed |
| Sprint 10 — Subsystems and Composition | v0.3 | Preserve strict nested Subsystem source agreement, direct hierarchy and composition, and computed transitive membership without persisted closure. | [Sprint review](reviews/sprint-10-subsystems-composition.md) | completed |
| Sprint 11 — Event Subscriptions | v0.3 | Preserve stable Event Subscription identity, typed event content, exact and family source References, and owned handler References and Triggers without runtime-dispatch claims. | [Sprint review](reviews/sprint-11-event-subscriptions.md) | completed |
| Sprint 12 — SKD and Report Model | v0.3 | Preserve Report-owned Data Composition Schemas, direct Data Sets and Fields, and metadata-owned Queries without speculative nested identities or partial query dependencies. | [Sprint review](reviews/sprint-12-skd-report-model.md) | completed |

Sprint 5 is completed under
[ADR-0027](adr/0027-incremental-semantic-index-maintenance.md) with a `pass`
decision. The separate v0.2 release review also records `pass`.

Sprint 6 is completed under
[ADR-0028](adr/0028-attribute-tabular-section-semantics.md) with a `pass`
decision in the
[Sprint 6 integration review](reviews/sprint-6-attributes-tabular-sections.md).
Sprint 7 is completed under
[ADR-0029](adr/0029-form-command-navigation-semantics.md) with a `pass`
decision in the
[Sprint 7 integration review](reviews/sprint-7-forms-commands.md). Sprint 8 is
completed under [ADR-0030](adr/0030-register-query-semantics.md) with a `pass`
decision in the
[Sprint 8 integration review](reviews/sprint-8-registers-queries.md). Sprint 9
is completed under [ADR-0031](adr/0031-conditional-grants-semantics.md) with a
`pass` decision in the
[Sprint 9 integration review](reviews/sprint-9-roles-access-rights.md). Sprint
10 is completed under
[ADR-0032](adr/0032-subsystem-hierarchy-semantics.md) with a `pass` decision in
the [Sprint 10 integration review](reviews/sprint-10-subsystems-composition.md).
Sprint 11 is completed under
[ADR-0033](adr/0033-event-subscription-semantics.md) with a `pass` decision in
the [Sprint 11 integration review](reviews/sprint-11-event-subscriptions.md).
Sprint 12 is completed under
[ADR-0034](adr/0034-report-data-composition-semantics.md) with a `pass` decision
in the [Sprint 12 integration review](reviews/sprint-12-skd-report-model.md).
Sprint 13 is the next planning target; v0.3 remains planned through Sprint 14.

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
| Sprint 10 — Subsystems and Composition | Add hierarchy, nested discovery, composition, and transitive membership contracts where justified. | completed |
| Sprint 11 — Event Subscriptions | Model event subscriptions, handlers, references, and resulting semantic relations. | completed |
| Sprint 12 — SKD and Report Model | Add data-composition and report-specific semantic entities and relations. | completed |
| Sprint 13 — XDTO and Service Model | Expand XDTO, HTTP service, and Web service semantics beyond top-level metadata-node coverage. | next |
| Sprint 14 — Designer XML Adapter | Ingest supported Designer XML configuration dumps through a source adapter without changing canonical semantic identities. | planned |

#### Sprint 6 Attributes and Tabular Sections execution plan

Sprint 6 expands only the attribute and tabular-section capability justified by
repository-owned EDT evidence. It does not reopen the completed Sprint 3 first
slice or assume that an EDT XML shape has source-independent meaning before it
has been investigated and accepted.

The implemented baseline at Sprint 6 planning time is:

- `Attribute` and `TabularSection` are independently addressable graph nodes;
- top-level attributes and tabular sections are owned by their metadata object,
  while a nested attribute is owned only by its nearest enclosing tabular
  section;
- source UUIDs remain canonical identities, and the UUID-less attribute
  fallback includes its immediate owner so equal names under different tabular
  sections do not collide;
- Attribute type observations use the public metadata-type reference-request
  lifecycle and can project the accepted nine-kind `References` and
  `DependsOn` first slice, typed diagnostics, statistics, and deterministic
  provenance;
- graph Validation, Query ownership navigation, Diff, Impact ownership policy,
  Coverage, and a real EDT production fixture already cover that first slice;
- repeated builds and source-order-independent graph construction are already
  proven for the existing fixture.

Those facts remain compatibility constraints, not new Sprint 6 deliverables.
In particular, Sprint 6 must not add a second metadata-object owner for a nested
attribute, change UUID identity, infer references from unaccepted content, or
copy member nodes and relations into top-level typed metadata payload.

The current repository fixture proves one Document with a top-level attribute,
one tabular section, and one nested reference attribute. It does not by itself
authorize additional member properties, deeper nesting, collection ordering
semantics, kind-specific standard attributes, tabular-section references, or
new target-kind mappings. The first task therefore owns the real-source and
gap investigation. Any source form not established by repository-owned
artifacts remains deferred until that investigation supplies evidence.

The existing [investigation profile](codex/profiles/investigation.md),
[architecture profile](codex/profiles/architecture.md),
[graph implementation profile](codex/profiles/graph-implementation.md),
[parser implementation profile](codex/profiles/parser-implementation.md), and
[review profile](codex/profiles/review.md), together with their existing task
templates, cover the ordered work below. The Sprint 6 readiness audit found no
concrete reusable prompt-contract gap, so no Codex Framework change is planned.

The evidence-backed implementation boundary is accepted in
[ADR-0028](adr/0028-attribute-tabular-section-semantics.md): preserve the
repository-proven optional direct synonym value as typed content of Attribute
and TabularSection nodes. Identity, immediate ownership, canonical names, and
the completed reference-request slice remain unchanged. Other observed member
fields and source variants remain deferred. Because the accepted slice adds no
reference behavior, Task 6 is a regression gate and must be closed as
`already_complete` without an empty commit when its evidence remains green.

The Task 7 implementation and evidence baseline is complete. The
source-independent model exposes optional member synonym through a payload
compatible only with Attribute and TabularSection nodes. The EDT reader accepts
the single direct `synonym/value` form, produces typed invalid and duplicate
outcomes, preserves nearest-owner nesting, and the production builder emits an
explicit member payload for both present and absent values. Query, Validation,
Diff, Impact, repeated builds, source-order independence, UUID and owner-scoped
UUID-less identity, equal names under different owners, and real grants and
ownership fixtures are covered. Task 6 was closed as `already_complete` because
synonym adds no reference observation or endpoint.

Graph-domain and EDT Coverage for `SemanticNode(Attribute)` and
`SemanticNode(TabularSection)` now require and provide
`SemanticPayloadPreserved`; their `Supported` status and aggregate gap counts
remain unchanged. Number qualifiers, history/search flags, produced types,
line-number settings and standard attributes, multiple locale values,
alternative synonym encodings, deeper nesting, non-Document owner families,
and duplicate-identity policy remain deferred. The independent Task 8
[integration review](reviews/sprint-6-attributes-tabular-sections.md) records
`pass` and completes Sprint 6.

| Order | Task | Owned outcome | Prerequisite | Suggested commit message |
|---:|---|---|---|---|
| 1 | Investigate the live Attribute and TabularSection source boundary. | Repository-owned artifact corpus, implemented-versus-missing matrix, consumer inventory, and candidate source contracts. | Sprint 5 and the v0.2 release review are completed. | `Investigate Sprint 6 member source contracts` |
| 2 | Accept the Sprint 6 semantic and compatibility contract. | One architecture decision and executable scope for the smallest evidence-backed expansion. | Task 1 evidence is complete. | `Define Sprint 6 member semantics` |
| 3 | Implement source-independent member-model prerequisites. | Accepted metadata/graph types, schema invariants, and Query, Diff, and Impact compatibility. | Task 2 is accepted. | `Implement Sprint 6 member graph model` |
| 4 | Implement the accepted EDT member parser contract. | Source parsing, identity inputs, nearest-owner observations, typed invalid outcomes, and parser determinism. | Tasks 2 and 3 are complete. | `Parse Sprint 6 EDT member semantics` |
| 5 | Emit canonical member nodes and ownership facts. | Graph contribution, canonical identity use, immediate containment, provenance, and build validation. | Task 4 is complete. | `Emit Sprint 6 member ownership` |
| 6 | Integrate accepted member references. | Reference requests, exact resolution, projections, diagnostics, statistics, and reference determinism for only the accepted new source forms. | Task 5 is complete. | `Integrate Sprint 6 member references` |
| 7 | Complete production evidence, Coverage, and current-state documentation. | End-to-end fixture matrix, registry transition, aggregate verification, and synchronized current-state text. | Tasks 3 through 6 are complete. | `Complete Sprint 6 member coverage` |
| 8 | Review the integrated Sprint 6 baseline. | Findings, command evidence, Sprint decision, and Sprint 7 hand-off. | Task 7 and all implementation validation are complete. | `Complete Sprint 6 attributes and tabular sections review` |

##### Task 1 — Investigate the live source boundary

Use the [investigation profile](codex/profiles/investigation.md) and
[investigation task template](codex/templates/investigation-task.md). Inspect
the live metadata-domain and graph types, EDT metadata-structure reader and
builder, real fixtures, Coverage entries, tests, and all Query, Validation,
Diff, Impact, and reference-request consumers. Record a repository-owned sample
matrix for top-level attributes, tabular sections, and nested attributes across
every metadata-owner family that the corpus actually proves.

The investigation owns classification of duplicate names, UUID presence and
absence, immediate and deeper XML nesting, direct and composite types, source
ordering, malformed content, and any candidate member properties. It must say
which observations are already supported, which are accepted source facts but
not modeled, and which remain unknown. It must not promote a sample-specific
XML element into architecture, edit production code, or change Coverage.

Acceptance evidence is an investigation record with exact artifact paths,
parser entry points, graph consumers, positive and negative examples, and a
smallest candidate Sprint 6 delta. Focused validation is Markdown diff/link
validation. No broad Rust validation is required for a documentation-only
investigation.

##### Task 2 — Accept semantics and compatibility before implementation

Use the [architecture profile](codex/profiles/architecture.md) and
[architecture task template](codex/templates/architecture-task.md). Convert
only Task 1's proven source facts into an accepted source-independent contract.
The decision owns the canonical member vocabulary and content boundary;
identity inputs and fallback encoding; immediate-owner and invalid-owner rules;
duplicate-name behavior; reference categories and target allowlists;
provenance responsibility; deterministic ordering and equality; malformed,
missing, ambiguous, and partial outcomes; and public API compatibility.

The contract must preserve source UUIDs, owner-scoped UUID-less identity,
single nearest-owner containment, exact name-and-kind resolution, immutable
ordered request ledgers, and modified-not-remove/add Diff behavior for semantic
content changes. It must explicitly decide whether the existing graph model is
sufficient. A new graph kind, metadata payload field, containment rule,
reference family, or public API is permitted only when the investigation proves
that the existing contract cannot represent an accepted semantic fact.

Acceptance requires an accepted ADR or an evidence-backed amendment to an
existing applicable decision, a consumer migration inventory, rejected
alternatives, compatibility impact, Coverage criteria, and ordered
implementation prerequisites. Focused validation is documentation diff and
link validation; the task changes no production behavior.

##### Task 3 — Implement source-independent member-model prerequisites

Use the [graph implementation profile](codex/profiles/graph-implementation.md)
with the [graph model task template](codex/templates/graph-model-task.md). This
task exclusively owns source-independent metadata and graph types required by
Task 2, graph construction invariants, endpoint/ownership schema changes, and
public Query, Diff, and Impact integration. It must keep adapters out of graph
and metadata domains and retain compatibility constructors where Task 2
requires them.

Tests must cover equality and canonical ordering, duplicate identities,
permitted and forbidden owners, payload-kind or other invalid-state rejection,
exact Query navigation, stable node identity across semantic-content changes,
Diff classification, Impact ownership behavior, and repeated deterministic
validation. If Task 2 proves that no model change is required, this task is
closed by a focused evidence update rather than a speculative abstraction.
Focused validation targets affected metadata and graph packages; full
validation is the repository Definition of Done.

##### Task 4 — Implement the accepted EDT parser contract

Use the [parser implementation profile](codex/profiles/parser-implementation.md)
and [parser task template](codex/templates/parser-task.md). Extend only the
source forms accepted by Task 2. The parser owns source UUID preservation,
owner-scoped fallback identity creation, immediate-owner observations,
source-order normalization, accepted member content, and typed outcomes for
missing identifiers and names, duplicate or conflicting observations,
malformed values, unsupported nesting, and unsupported source variants.

Parser tests must use raw repository-owned EDT artifacts and cover present and
missing UUIDs, same-name members under different owners, exact duplicate
behavior, nearest-owner selection, accepted direct and composite types,
malformed and unknown values, reordered equivalent input, and repeated reads.
This task does not insert graph nodes or edges, resolve references, transition
Coverage, or implement forms and commands. Focused validation targets the EDT
reader and parser tests; full validation is the repository Definition of Done.

##### Task 5 — Emit canonical nodes and immediate ownership

Use the [graph implementation profile](codex/profiles/graph-implementation.md)
and [graph emission task template](codex/templates/graph-emission-task.md). Map
Task 4 observations to the accepted source-independent model, reuse their
canonical identities, emit member nodes, and emit exactly one provenance-backed
`Contains` edge from the immediate accepted owner. Node collection must remain
separate from ownership-edge insertion so XML completion order cannot change
the graph.

Production-builder evidence must cover top-level and nested members, equal
names under different owners, UUID and UUID-less identities, missing or invalid
owners, forbidden second owners, deterministic provenance, Query owner/child
navigation, graph validation, build Diff, optional Impact ownership traversal,
source-order independence, and repeated builds. This task does not broaden
reference targets or create placeholder nodes. Focused validation targets EDT
graph construction and graph validation; full validation is the repository
Definition of Done.

##### Task 6 — Integrate only accepted member references

Use the [graph implementation profile](codex/profiles/graph-implementation.md)
and [graph emission task template](codex/templates/graph-emission-task.md). This
task owns conversion of any newly accepted Task 2 member-reference observations
to the public request lifecycle, collection and resolver provenance, exact
name-and-kind resolution, deterministic aggregation, terminal request states,
candidate sets, direct `References` projections, justified derived
`DependsOn` projections, diagnostics, and statistics. Existing Attribute,
Dimension, and Resource metadata-type behavior must remain unchanged.

Tests must cover resolved, missing, ambiguous, incompatible-kind, partial,
duplicate, invalid-owner, and repeated-build outcomes, plus ledger/report/diff
consistency and the absence of unresolved or placeholder edges. No reference
category, target-kind mapping, tabular-section reference, or dependency edge is
added unless Task 2 explicitly accepts it. Focused validation targets graph
reference-request and EDT resolution tests; full validation is the repository
Definition of Done.

##### Task 7 — Complete production evidence and Coverage

Use the [graph implementation profile](codex/profiles/graph-implementation.md)
and [graph emission task template](codex/templates/graph-emission-task.md). Add
the smallest representative real-format EDT fixture matrix that closes the
accepted Sprint 6 criteria across Tasks 3 through 6. Prove graph and build
validation, Query, Diff, Impact where applicable, reference ledgers and reports,
diagnostics, deterministic provenance and ordering, source-order independence,
and equal repeated builds through the production builder.

Only after that evidence passes, update the graph-domain and EDT Coverage
entries independently, recompute aggregate counts from the live registries, and
synchronize `docs/architecture/semantic-model-2.md` and this Roadmap with the
implemented boundary and remaining limitations. Planning and architecture
acceptance alone do not add evidence or change status. Focused registry and
production-fixture tests and the complete repository Definition of Done must
pass in the same commit.

##### Task 8 — Review the integrated Sprint 6 baseline

Use the [review profile](codex/profiles/review.md) and
[review task template](codex/templates/review-task.md). Audit every Task 1–7
commit against the accepted decision and current repository state. Re-run the
full Definition of Done and focused graph-domain, EDT parser, production
builder, reference-request, Coverage, Query, Diff, Impact, validation, and
determinism tests. Verify that existing Sprint 3 facts remain compatible and
that no Forms, Commands, Sprint 7 concern, or later-sprint source contract was
pulled forward.

The review record owns findings, exact commands and outcomes, residual risks,
deferred scope, and one decision: `pass`, `pass with non-blocking follow-ups`,
or `blocked`. Sprint 6 may become `completed` only after a non-blocking review
decision, all accepted capabilities and documentation are synchronized, and
the full validation cycle succeeds. Sprint 7 becomes the next planning target
only after that transition.

##### Sprint 6 state gates

Sprint 6 is `completed`. Tasks 1 through 5 and Task 7 are committed in
dependency order, Task 6 is recorded as `already_complete` under ADR-0028, and
Task 8 records `pass` in the
[Sprint 6 integration review](reviews/sprint-6-attributes-tabular-sections.md).
Every accepted identity, ownership, content, provenance, invalid-state,
ordering, and repeated-build criterion is proven; the completed reference slice
remains green; Coverage reflects live production evidence; and the full
Definition of Done passes.

Forms, Commands, queries, roles, subsystems, event subscriptions, Designer XML,
Runtime, persistence, AI, MCP, IDE, and all unproven EDT member forms remain
deferred. This completed baseline provided the hand-off into Sprint 7 Forms and
Commands.

#### Sprint 7 Forms and Commands execution plan

Sprint 7 expands the completed Form and Command declaration slice with only the
executable, reference, and navigation facts accepted by
[ADR-0029](adr/0029-form-command-navigation-semantics.md). It does not replace
the existing flat graph with the complete conceptual UI taxonomy and does not
infer semantics from directory names, command placement, or an `OpenForm`
spelling alone.

The immutable planning baseline at `eab870a` was:

- Common Forms are top-level
  `NodeKind::Metadata(MetadataKind::CommonForm)` nodes;
- Common Commands are top-level
  `NodeKind::Metadata(MetadataKind::Command)` nodes;
- subordinate Forms and Commands are `NodeKind::Form` and
  `NodeKind::Command` children of their immediate metadata object;
- UUID or owner-scoped UUID-less identity, declared provenance, canonical
  `Contains`, Query ownership navigation, Validation, Diff, Impact, repeated
  builds, and current Coverage are already implemented for those declaration
  facts;
- top-level Common Form `Module.bsl` can already enter the generic module path,
  and its existing identity is a compatibility constraint;
- subordinate Form `Module.bsl`, Common/subordinate Command
  `CommandModule.bsl`, command parameter types, and static form-opening
  navigation do not reach the current production graph;
- the live `EdgeKind` enum does not contain `Opens`;
- `Form.form` internals and Command Groups have no accepted production model.

Tasks 1–7 now implement the accepted delta: `EdgeKind::Opens` and its graph
consumers are live; canonical subordinate Form and Common/subordinate Command
modules enter the existing BSL pipeline; mapped Command parameter types use the
public reference-request lifecycle; and exact static navigation emits
provenance-backed `Procedure --Opens--> Form` facts. The planning-baseline
statements above remain historical gate evidence rather than current production
limitations.

Those implemented facts are `already_complete` planning prerequisites, not new
Sprint 7 tasks. No empty commit may recreate or restate them. Their proving
baseline remains committed and must stay green throughout the sprint.

The repository-owned
[Form and Command source investigation](architecture/form-command-source-investigation.md)
records the current artifact corpus, implemented-versus-missing matrix,
consumer inventory, supported source forms, and confirmed unknowns. ADR-0029
accepts the smallest coherent delta:

1. canonical Form and Command entities own recognized executable Module nodes;
2. mapped command parameter metadata types use the public request lifecycle and
   may emit precise `References` and `DependsOn` facts;
3. a complete static literal `OpenForm(...)` inside an accepted Command-module
   Procedure may emit one resolved `Procedure --Opens--> Form` relation.

Default-form and shorthand targets, dynamic expressions, calls outside the
accepted source scope, Form internals, Form commands and events, Command Groups,
localized subordinate payload, explicit `Executes` semantics, placeholder
Forms, Designer XML, and other UI relations remain deferred.

##### Prerequisite gate

Sprint 6 and its integration review are complete. Before Task 1 begins, the
Sprint 7 planning change containing the source investigation, ADR-0029, this
execution plan, and the synchronized Semantic Model 2.0 boundary must be
committed or otherwise proven as the exact immutable execution baseline.

Every later task begins only from the committed result of its predecessor or a
current `already_complete` proof satisfying every acceptance criterion. A
stored prompt, historical status, or uncommitted working-tree change is not a
prerequisite.

##### Readiness and template decision

The live audit covered `MetadataKind`, `NodeKind`, `EdgeKind`, graph node
payload compatibility, ownership and relation validation, Query, Semantic
Index, incremental indexing, Diff, Impact, Coverage, EDT discovery,
metadata-structure parsing, module discovery, BSL analysis, production-builder
tests, and real EDT Form, Command, Common Form, Common Command, Command Group,
`Form.form`, `Module.bsl`, and `CommandModule.bsl` artifacts.

The existing [graph implementation profile](codex/profiles/graph-implementation.md),
[parser implementation profile](codex/profiles/parser-implementation.md),
[review profile](codex/profiles/review.md), [graph model
template](codex/templates/graph-model-task.md), [parser
template](codex/templates/parser-task.md), [graph emission
template](codex/templates/graph-emission-task.md), and [review
template](codex/templates/review-task.md) cover every accepted task boundary.
The sprint-planning and sequential-execution contracts provide the required
prerequisite, `already_complete`, failure, review, and final-state gates. No
Codex Framework change is planned.

##### Ordered task manifest

| Order | Task | Profile / Template | Owned outcome | Prerequisite | Suggested commit message |
|---:|---|---|---|---|---|
| 1 | Implement the source-independent Form/Command graph prerequisites. | Graph implementation / graph model | `Opens`, precise endpoints and consumers, Form/Command module ownership, and Command reference/dependency schema. | Accepted Sprint 7 planning baseline. | `Define Sprint 7 graph navigation model` |
| 2 | Parse accepted Form and Command module layouts. | Parser implementation / parser | Typed, deterministic module observations joined to canonical Form and Command owners. | Task 1. | `Parse Sprint 7 form and command modules` |
| 3 | Emit Form and Command modules and BSL declarations. | Graph implementation / graph emission | Canonical Module ownership plus existing BSL semantic contribution through the production builder. | Task 2. | `Emit Sprint 7 form and command modules` |
| 4 | Parse typed Command parameter references. | Parser implementation / parser | Source observations with a distinct role and the accepted nine-kind allowlist. | Task 3. | `Parse Sprint 7 command parameter references` |
| 5 | Integrate Command parameter reference resolution. | Graph implementation / graph emission | Public request lifecycle, terminal outcomes, diagnostics, `References`, and justified `DependsOn`. | Task 4. | `Integrate Sprint 7 command references` |
| 6 | Parse static Command-module form-opening candidates. | Parser implementation / parser | Complete-statement `OpenForm` candidates with exact source scope and typed rejections. | Task 5. | `Parse Sprint 7 static form navigation` |
| 7 | Resolve and emit canonical Form navigation. | Graph implementation / graph emission | Owner-scoped/Common Form resolution, diagnostics, and deterministic provenance-backed `Opens`. | Task 6. | `Emit Sprint 7 form navigation` |
| 8 | Complete production evidence, Coverage, and current-state documentation. | Graph implementation / graph emission | Representative end-to-end evidence, independent Coverage transitions, aggregate verification, and synchronized docs. | Tasks 1–7. | `Complete Sprint 7 production evidence` |
| 9 | Review the integrated Sprint 7 baseline. | Review / review | Findings, full command evidence, sprint decision, and Sprint 8 hand-off. | Task 8 and all implementation validation. | `Complete Sprint 7 forms and commands review` |

The dependency graph is strictly sequential:

```text
Accepted planning baseline
    -> Task 1 graph contract
    -> Task 2 module parser
    -> Task 3 module emission
    -> Task 4 parameter parser
    -> Task 5 parameter reference integration
    -> Task 6 navigation parser
    -> Task 7 navigation emission
    -> Task 8 production evidence and Coverage
    -> Task 9 integration review
    -> Sprint 8 planning eligibility
```

##### Task 1 — Implement the source-independent graph prerequisites

Use the [graph implementation profile](codex/profiles/graph-implementation.md)
and [graph model task template](codex/templates/graph-model-task.md). Add only
the source-independent model required by ADR-0029:

- `EdgeKind::Opens` with stable machine-readable identity;
- exact `Procedure --Opens--> Form` and
  `Procedure --Opens--> Metadata(CommonForm)` validation;
- generic Query edge filtering, dependency/usage classification, bounded
  traversal, Diff, Impact, Semantic Index, incremental-index, report, and
  Coverage integration;
- precise `Form --Contains--> Module`, `Command --Contains--> Module`, and
  top-level Common Command module ownership;
- precise Command parameter `References` and `DependsOn` source/target
  matrices for only the accepted nine target kinds.

The task must update every exhaustive enum consumer and preserve public
constructors and existing result ordering where compatible. `Opens` is a direct
dependency/usage and reverse-Impact relation, but it emits no companion
`References` or `DependsOn`. Existing Form, Command, Module, Contains,
References, DependsOn, Calls, Query, Diff, and Impact behavior remains
unchanged outside the additive contract.

Excluded scope is all EDT parsing or emission, new UI node kinds, placeholder
nodes, command execution edges, Form internals, Command Groups, and payload
changes. Acceptance evidence includes exhaustive positive and negative endpoint
tests, dependency/usage and Impact direction, deterministic identity and
ordering, complete/incremental index equivalence, Diff behavior, and unchanged
existing coverage capabilities.

Focused validation additions:

```bash
cargo test -p oneagent-graph --lib
cargo test -p oneagent-graph --test validation
cargo test -p oneagent-graph --test diff
```

Run the complete repository implementation gate after focused checks.

##### Task 2 — Parse accepted Form and Command module layouts

Use the [parser implementation profile](codex/profiles/parser-implementation.md)
and [parser task template](codex/templates/parser-task.md). Extend EDT module
discovery without graph insertion:

- join `Forms/<Name>/Module.bsl` to the exact subordinate Form declared by the
  same metadata object;
- join `Commands/<Name>/CommandModule.bsl` to the exact subordinate Command;
- join `CommonCommands/<Name>/CommandModule.bsl` to the exact Common Command;
- preserve the existing Common Form `Module.bsl` behavior and identity;
- create deterministic `form_module` and `command_module` descriptors from the
  canonical owner identity;
- return typed outcomes for missing optional modules, orphan directories,
  mismatched names, duplicate observations, unreadable files, and unsupported
  layouts.

The parser must not synthesize owners, recurse into `Form.form`, analyze BSL,
emit graph facts, or change Coverage. Acceptance evidence uses raw
repository-owned layouts plus generated reordered, missing, orphaned,
duplicate, and malformed cases. Repeated reads must be equal.

Focused validation addition:

```bash
cargo test -p oneagent-edt module_reader
```

Run the complete repository implementation gate after focused checks.

##### Task 3 — Emit modules and existing BSL semantics

Use the [graph implementation profile](codex/profiles/graph-implementation.md)
and [graph emission task template](codex/templates/graph-emission-task.md).
Convert only Task 2 observations into canonical Module nodes and exactly one
provenance-backed `Contains` edge from the accepted Form or Command owner.

Feed accepted module descriptors through the existing BSL declaration, Query,
Calls, diagnostic, and provenance pipeline without introducing a parallel UI
symbol model. Node collection must precede ownership insertion. Existing
Common Form module identity, ordinary metadata modules, call resolution,
Query/Reads/Writes behavior, and source ordering must remain compatible.

Acceptance evidence covers subordinate Form, subordinate Command, Common
Command, and existing Common Form module paths; Module, Procedure, and Function
ownership; missing modules; invalid owners; repeated builds; reversed
discovery order; graph/build Diff; Validation; Query; and deterministic
provenance. This task emits no Command parameter references or `Opens` edge.

Focused validation addition:

```bash
cargo test -p oneagent-edt
```

Run the complete repository implementation gate after focused checks.

##### Task 4 — Parse typed Command parameter references

Use the [parser implementation profile](codex/profiles/parser-implementation.md)
and [parser task template](codex/templates/parser-task.md). Preserve direct
`commandParameterType/types` observations from Common and subordinate Commands
with a distinct semantic role, canonical Command source identity, raw token,
mapped target kind, and canonical target name.

Accept only the ADR-0029 nine-kind allowlist. Primitive, Defined Type,
platform, unrecognized, malformed, duplicate, missing, and multiple values must
have deterministic accepted, ignored, unsupported, or typed-invalid outcomes.
The parser must not resolve targets, create requests, insert graph facts, infer
parameter types from BSL, or broaden Attribute/Dimension/Resource type parsing.

Acceptance evidence includes real Catalog, Document, Task, Common Command, and
deferred Defined Type samples; empty parameter types; duplicate/multiple
values; malformed names; reordered input; and repeated reads.

Focused validation addition:

```bash
cargo test -p oneagent-edt metadata_structure
```

Run the complete repository implementation gate after focused checks.

##### Task 5 — Integrate Command parameter references

Use the [graph implementation profile](codex/profiles/graph-implementation.md)
and [graph emission task template](codex/templates/graph-emission-task.md).
Convert Task 4 observations into the ADR-0024 public request lifecycle with
collection provenance, immutable canonical identity, exact name-and-kind
resolution, ordered candidates, deterministic aggregation, terminal request
states, diagnostics, statistics, reports, and build Diff.

Unique accepted resolution emits one direct `References` and one derived
`DependsOn` edge from the canonical Common or subordinate Command. Missing,
ambiguous, incompatible, partial, malformed, unsupported, and duplicate
outcomes must match ADR-0029 and emit no placeholder or lower-confidence edge.

Existing metadata-member and access-right requests and their Coverage remain
unchanged. Acceptance evidence covers both Command source kinds, all accepted
target kinds through the smallest representative matrix, negative outcomes,
duplicate provenance aggregation, source-order independence, repeated builds,
Query, Diff, Impact, validation, reports, diagnostics, and statistics.

Focused validation additions:

```bash
cargo test -p oneagent-graph --test reference_request_build
cargo test -p oneagent-edt
```

Run the complete repository implementation gate after focused checks.

##### Task 6 — Parse static form-opening candidates

Use the [parser implementation profile](codex/profiles/parser-implementation.md)
and [parser task template](codex/templates/parser-task.md). Implement a typed
complete-statement extractor for the ADR-0029 first navigation grammar. A
candidate must preserve the accepted Command-module identity, containing
Procedure identity, complete static first-argument literal, parsed target kind,
owner name where applicable, Form name, source location, and deterministic
candidate order.

Positive forms are only `CommonForm.<Name>` and
`<SupportedKind>.<Owner>.Form.<Name>`. Dynamic expressions, concatenation,
variables, default Form aliases, ListForm/ObjectForm shorthand, Functions,
unsupported prefixes, calls outside accepted Command modules, incomplete
statements, comments, and strings that only contain `OpenForm(` must not become
edge-producing candidates.

This task does not resolve a graph target, emit edges, change Command parameter
behavior, parse `Form.form`, or implement general BSL platform-call semantics.
Focused tests must be backed by exact repository-owned positive and negative
source excerpts and cover multiline calls, reordered candidates, malformed
input, and repeated extraction.

Focused validation addition:

```bash
cargo test -p oneagent-bsl
```

Run the complete repository implementation gate after focused checks.

##### Task 7 — Resolve and emit Form navigation

Use the [graph implementation profile](codex/profiles/graph-implementation.md)
and [graph emission task template](codex/templates/graph-emission-task.md).
Resolve Task 6 candidates only after all metadata, Form, Command, Module, and
callable nodes have been collected.

Common Form targets use exact name-and-kind resolution. Subordinate targets
resolve the exact typed metadata owner and then the exact `NodeKind::Form`
child under that owner. Unique success emits one canonical
`Procedure --Opens--> Form` edge with deterministic resolved provenance.
Missing, ambiguous, incompatible, partial, dynamic, default, shorthand, and
unsupported outcomes emit typed diagnostics and no edge.

Acceptance evidence covers Common and subordinate targets, equal Form names
under different owners, wrong owner, missing and ambiguous owners, missing and
ambiguous children, incompatible kinds, partial workspaces, duplicate evidence,
Query dependency/usage, reverse Impact, Diff, validation, diagnostics, reports,
source-order independence, and repeated builds. No companion `References`,
`DependsOn`, `Calls`, or placeholder edge may appear.

Focused validation additions:

```bash
cargo test -p oneagent-graph --test validation
cargo test -p oneagent-edt
```

Run the complete repository implementation gate after focused checks.

##### Task 8 — Complete production evidence and Coverage

Use the [graph implementation profile](codex/profiles/graph-implementation.md)
and [graph emission task template](codex/templates/graph-emission-task.md). Add
the smallest representative real-format EDT fixture matrix that closes every
accepted Task 1–7 criterion through `FileSystemEdtSemanticGraphBuilder`.

The matrix must prove module ownership and BSL contribution, Common and
subordinate Command parameter references, explicit subordinate and Common Form
navigation, negative and partial outcomes, deterministic provenance and
diagnostics, Query, Validation, Diff, Impact, request ledgers, reports,
statistics, source-order independence, repeated builds, and incremental-index
equivalence with a clean rebuild.

Only after all evidence passes, transition the independent graph-domain and EDT
Coverage capabilities required by ADR-0029, recompute aggregate counts from the
live registries, and synchronize
`architecture/semantic-model-2.md` and this Roadmap with the implemented
boundary and remaining limitations. Existing declaration capabilities must not
be reopened or double-counted. Deferred Form internals, Command Groups,
dynamic/default targets, payload, and execution relations remain outside the
completion claim.

Focused validation additions:

```bash
cargo test -p oneagent-graph coverage
cargo test -p oneagent-edt coverage
```

Run the complete repository implementation gate after focused checks.

Task 8 production evidence is now present in the repository-owned
`adapters/edt/tests/fixtures/sprint7_forms_commands_project` layout and its
full-builder integration test. The matrix covers subordinate Form,
subordinate/Common Command, and existing Common Form modules; Procedure,
Function, Query, ownership, Command reference/dependency projections, explicit
Common and subordinate Form navigation, typed negative and partial outcomes,
duplicate provenance, equal-name owner scoping, reports, statistics, Query,
Diff, Impact, Validation, source-order independence, repeated builds, and the
existing complete/incremental Semantic Index clean-rebuild oracle. EDT
`semantic_edge.opens` is now `Supported` with complete evidence. Live registry
tests verify 101 EDT capabilities (96 `Supported`, 5 `NotApplicable`) and 85
graph-domain capabilities (82 `Supported`, 3 `NotApplicable`), with no
Critical, High, or Medium gaps. Task 9 subsequently records the non-blocking
review decision that completes Sprint 7.

##### Task 9 — Review the integrated Sprint 7 baseline

Use the [review profile](codex/profiles/review.md) and
[review task template](codex/templates/review-task.md). Audit every Task 1–8
commit against ADR-0029, the source investigation, the live implementation, and
the current Roadmap. Re-run the complete repository Definition of Done and all
focused graph model, module parser, BSL, production builder, reference request,
navigation, Query, Diff, Impact, Validation, Coverage, determinism, and
incremental-equivalence checks.

Verify that existing Form and Command identities, ownership, Common Form module
identity, completed metadata references, Calls, Reads, Writes, Includes,
Grants, Sprint 6 member behavior, and aggregate Coverage remain compatible.
Verify that no `Form.form` internals, Command Groups, dynamic/default targets,
new payload, explicit execution relation, Designer XML, or later-sprint concern
was pulled forward.

The review owns one decision: `pass`, `pass with non-blocking follow-ups`, or
`blocked`. It may create the authorized review artifact and update Roadmap state
only when the decision is non-blocking and all validation succeeds. It must not
silently fix findings in the review change.

Task 9 records `pass` in the
[Sprint 7 integration review](reviews/sprint-7-forms-commands.md). The review
covers the committed range
`77a52c6821e64f8fe7b9c71d2304a4ab77585cd7..c16e136eeff2df3296669f8ad682adbd9cdd3180`,
reports no blocking or non-blocking findings, missing evidence, compatibility
breaks, or scope violations, and records the successful focused and complete
workspace validation. Sprint 8 therefore became eligible for planning and is
now active under the committed ADR-0030 baseline.

##### Planning validation and suggested commit

The Sprint 7 kickoff was documentation-only. Its validation covered Markdown
consistency, relative links, manifest numbering, dependency order,
accepted-versus-deferred scope, and the then-unchanged `next` status. The
planning change required only `git diff --check` because it modified no
production file.

Suggested planning commit message, as a recommendation only:

```text
Plan Sprint 7 forms and commands
```

The message does not authorize staging or committing.

##### Sprint 7 state gates and completion criteria

Sprint 7 remained `next` during planning. It became active when the
accepted planning baseline is committed and Task 1 begins under an explicit
execution instruction.

The current state is `completed`: Tasks 1–8 are committed in dependency order,
Task 9 records `pass`, and the required focused and complete workspace
validation succeeded.

A task is `already_complete` only when current committed evidence and successful
required validation prove every acceptance criterion. Record the proving
baseline and do not create an empty commit. This rule applies especially to the
already implemented Form/Command declaration and ownership slice and existing
Common Form module behavior.

Stop the sprint sequence after any prerequisite, implementation, validation,
staging, commit, or review failure. Do not skip a blocked task or start a
dependent task. A blocked Task 9 leaves Sprint 7 incomplete and Sprint 8
ineligible for planning.

Sprint 7 may transition to `completed` only when:

- Tasks 1–8 are committed in dependency order or proven `already_complete`;
- every ADR-0029 module, reference, navigation, provenance, diagnostic,
  determinism, Query, Diff, Impact, incremental-equivalence, and Coverage
  criterion is proven through the production builder;
- existing compatibility behavior remains green;
- current-state architecture and Roadmap text matches live implementation;
- the complete repository Definition of Done passes;
- Task 9 records `pass` or `pass with non-blocking follow-ups`.

The non-blocking review transition makes Sprint 8 Registers and Queries
eligible as the next planning target. Planning or architecture acceptance alone
does not change sprint status, capability status, or Coverage counts.

The v0.3 release integration review follows Sprint 14.

#### Sprint 8 Registers and Queries execution plan

Sprint 8 implements only the direct register Query and normalized dependency
boundary accepted by
[ADR-0030](adr/0030-register-query-semantics.md). It does not treat Query
declaration sources, query-language data sources, register virtual tables,
register metadata members, Reads, Writes, and DependsOn as interchangeable
concepts.

The completed implementation baseline is:

- static named BSL Query declarations inside known Procedures or Functions
  already produce stable Query nodes with canonical ownership and provenance;
- the minimum query-language parser completely accepts one `SELECT` with one
  direct Catalog, Information Register, Accumulation Register, or Accounting
  Register source and rejects unsupported or unconsumed source-producing
  grammar without partial Reads;
- accepted parsed sources enter the public
  `SemanticReferenceCategory::QuerySource` ledger, whose deterministic
  terminal outcomes drive diagnostics, statistics, Reads, and Query-origin
  DependsOn exactly once;
- `Reads`, `DependsOn`, and ReferenceRequest capabilities are already Supported
  for their completed first slices, so this sprint expands evidence without a
  status or aggregate-count transition;
- Writes is independently Supported for one Document Procedure to Accumulation
  Register contract and remains unchanged;
- existing register metadata nodes, Dimension/Resource members, Accounting
  Register Measure mapping, ownership, metadata type references, Query, Diff,
  Impact, reports, validation, complete/incremental indexes, and deterministic
  builds are compatibility constraints.

Tasks 1–5 are implemented in dependency order. The representative real-format
EDT fixture and `sprint8_full_builder_matrix_is_complete_deterministic_and_consumer_visible`
test cover both new register families, the existing Catalog and Information
Register compatibility matrix, public requests, Reads and DependsOn,
diagnostics, statistics, validation, Query, Diff, Impact, reports, deterministic
builds, and source-order independence. Existing focused index evidence proves
that the expanded Query register edges remain equivalent to a clean rebuild.
The EDT Coverage Registry remains exactly 101 capabilities: 96 `Supported` and
5 `NotApplicable`; the graph registry remains exactly 85 capabilities: 82
`Supported` and 3 `NotApplicable`. Both retain zero Critical, High, and Medium
gaps. Task 6 records `pass` in the
[Sprint 8 integration review](reviews/sprint-8-registers-queries.md).

The repository-owned
[Register and Query source investigation](architecture/register-query-source-investigation.md)
confirms static direct sources for
`AccumulationRegister.InventoryCost` and
`AccountingRegister.FinancialAccounting`, their known Common Module owners,
Configuration declarations, and top-level target descriptors. It also records
that the complete real programs contain grammar beyond the current minimum.
Sprint 8 therefore accepts provenance-backed reduced fixtures for the existing
complete one-source grammar rather than claiming general projection, `WHERE`,
or `ORDER BY` support.

The selected delta is:

1. exact direct Accumulation and Accounting Register Query source categories;
2. public QuerySource collection and terminal request lifecycle;
3. retained `Query --Reads--> Metadata` plus derived
   `Query --DependsOn--> Metadata` for every uniquely resolved accepted source,
   including existing Catalog and Information Register sources;
4. end-to-end evidence through the filesystem EDT builder and existing graph
   consumers.

Calculation Registers, virtual tables, JOIN, UNION, nesting, batches,
temporary/external/parameter tables, broader expression grammar, new Query
declaration families, Query mutation, write-derived dependencies, register
payload/member expansion, and placeholders remain deferred.

##### Sprint 8 prerequisite gate

Sprint 7 and its integration review are complete. Before Task 1 begins, the
Sprint 8 planning change containing the source investigation, accepted
ADR-0030, this execution plan, synchronized Semantic Model boundary, and the
prompt suite must be committed or otherwise proven as one immutable planning
baseline.

Every later task begins only from the committed result of its predecessor or a
current `already_complete` proof satisfying every acceptance criterion. Stored
prompt text and uncommitted changes are not prerequisites.

##### Readiness and template decision

The readiness audit covered Query extraction, query-language parsing,
multiline decoding and locations, Query source resolution, public reference
requests, graph validation, Query, Diff, Impact, reports, complete/incremental
indexes, Coverage, current Reads and Writes production paths, fixture
conventions, and real Accumulation, Accounting, Calculation, and virtual-table
source evidence.

The existing [graph implementation profile](codex/profiles/graph-implementation.md),
[parser implementation profile](codex/profiles/parser-implementation.md),
[implementation profile](codex/profiles/implementation.md),
[review profile](codex/profiles/review.md), [graph model
template](codex/templates/graph-model-task.md), [parser
template](codex/templates/parser-task.md), [implementation
template](codex/templates/implementation-task.md), [graph emission
template](codex/templates/graph-emission-task.md), and [review
template](codex/templates/review-task.md) cover every accepted task boundary.
No reusable Codex Framework change or post-sprint Framework audit task is
planned.

##### Ordered task manifest

| Order | Task | Profile / Template | Owned outcome | Prerequisite | Suggested commit message |
|---:|---|---|---|---|---|
| 1 | Define the source-independent register Query graph rules. | Graph implementation / graph model | Exact additive Reads and Query-origin DependsOn endpoint matrices plus consumer compatibility. | Accepted Sprint 8 planning baseline. | `Define Sprint 8 register query graph rules` |
| 2 | Parse direct Accumulation and Accounting Register Query sources. | Parser implementation / parser | Two typed persistent categories with provenance-backed reduced fixtures and unchanged completeness policy. | Task 1. | `Parse Sprint 8 direct register query sources` |
| 3 | Resolve Query source observations through public requests. | Implementation / implementation | Collected and terminal QuerySource requests with deterministic identity, provenance, outcomes, and no production projection change. | Task 2. | `Resolve Sprint 8 query source requests` |
| 4 | Integrate Query data access and normalized dependencies. | Graph implementation / graph emission | Terminal request projections, diagnostics, statistics, retained Reads, and derived DependsOn. | Task 3. | `Emit Sprint 8 query data dependencies` |
| 5 | Complete production evidence, Coverage evidence, and current-state documentation. | Graph implementation / graph emission | Representative full-builder matrix, index equivalence, live evidence synchronization, and unchanged statuses/counts. | Tasks 1–4. | `Complete Sprint 8 production evidence` |
| 6 | Review the integrated Sprint 8 baseline. | Review / review | Findings, complete register/query evidence, sprint decision, and Sprint 9 hand-off. | Task 5 and all implementation validation. | `Complete Sprint 8 registers and queries review` |

The dependency graph is strictly sequential:

```text
Accepted Sprint 8 planning baseline
    -> Task 1 graph rules
    -> Task 2 direct register source parser
    -> Task 3 public QuerySource requests and resolution
    -> Task 4 Reads and DependsOn production projections
    -> Task 5 production and Coverage evidence
    -> Task 6 integration review
    -> Sprint 9 planning eligibility
```

##### Task 1 — Define register Query graph rules

Use the [graph implementation profile](codex/profiles/graph-implementation.md)
and [graph model task template](codex/templates/graph-model-task.md). Extend
only the source-independent endpoint contract:

- `Query --Reads--> Metadata(AccumulationRegister | AccountingRegister)` is
  additive to the existing Catalog and Information Register targets;
- `Query --DependsOn--> Metadata(Catalog | InformationRegister |
  AccumulationRegister | AccountingRegister)` is additive to the existing
  member and Command dependency matrices;
- every unrelated source, target, direction, Unknown, wildcard Metadata,
  member, flat semantic, placeholder, and missing endpoint remains invalid.

No new NodeKind, EdgeKind, identity, serialization, parser, resolver, producer,
or Coverage status belongs to this task. Existing generic Query filtering,
direct dependency and usage navigation, Diff, Impact reason aggregation,
reports, Semantic Index, and incremental-index behavior already enumerate both
edge kinds and must gain focused compatibility evidence rather than new public
APIs.

Focused validation additions:

```bash
cargo test -p oneagent-graph --test validation
cargo test -p oneagent-graph --test query
cargo test -p oneagent-graph --test impact
cargo test -p oneagent-graph --test coverage
```

Run the complete repository implementation gate after focused checks.

##### Task 2 — Parse direct register Query sources

Use the [parser implementation profile](codex/profiles/parser-implementation.md)
and [parser task template](codex/templates/parser-task.md). Add exactly
`QuerySourceCategory::AccumulationRegister` and
`QuerySourceCategory::AccountingRegister` with the accepted English namespace
spellings and deterministic raw source locations.

Create raw query-language fixtures and a provenance manifest derived from the
two real Common Module examples. Reduction to the already accepted
single-projection, single-source grammar must be explicit and must preserve the
real qualified source, alias, source path/range, and target descriptor mapping.

Keep the current complete-source proof unchanged. Do not accept real program
tails, general projections, JOIN, UNION, nesting, batches, temporary tables,
virtual tables, Calculation Registers, Russian spellings, metadata resolution,
requests, graph emission, or Coverage changes. Existing Catalog and Information
Register parser behavior and every typed rejection must remain green.

Focused validation addition:

```bash
cargo test -p oneagent-bsl query_language
```

Run the complete repository implementation gate after focused checks.

##### Task 3 — Resolve public QuerySource requests

Use the [implementation profile](codex/profiles/implementation.md) and
[implementation task template](codex/templates/implementation-task.md). Convert
accepted parsed occurrences into the existing public
`SemanticReferenceCategory::QuerySource` lifecycle and adapt the private
resolver to return deterministic terminal requests.

Collection must use the canonical Query source node, target reference, one
exact expected kind, raw occurrence/location evidence, and collection
provenance. Resolution must preserve current case normalization, exact-kind
partitioning, ambiguity precedence, ordered candidates, complete/partial
workspace policy, and no-placeholder behavior. Request identity excludes state,
candidates, occurrence order, and provenance. Equivalent observations aggregate
deterministically; conflicting terminal content remains an invariant error.

This task must not change production `Reads`, emit `DependsOn`, replace legacy
statistics in the builder, alter diagnostics, or change Coverage. Focused tests
prove collected-to-terminal transitions for all four accepted target kinds,
missing, ambiguous, incompatible, partial, duplicate, reordered, and repeated
resolution.

Focused validation additions:

```bash
cargo test -p oneagent-graph reference_request
cargo test -p oneagent-edt query_source_resolution
```

Run the complete repository implementation gate after focused checks.

##### Task 4 — Emit Query data dependencies

Use the [graph implementation profile](codex/profiles/graph-implementation.md)
and [graph emission task template](codex/templates/graph-emission-task.md).
Replace the production Query source outcome path with Task 3 terminal requests.
Derive diagnostics, statistics, and projections exactly once from the request
ledger.

Unique resolution retains one resolved exact `Reads` edge and adds one derived
exact `DependsOn` edge for the same Query-target pair. Both aggregate sorted,
deduplicated provenance before insertion. The dependency provenance identifies
the terminal request and retained Reads fact. Failures and parser rejections
emit neither edge and never create placeholders.

Acceptance evidence covers all four targets, public request Query/report/build
Diff visibility, statistics compatibility, Query dependency and usage
relations, reverse Impact with unique affected nodes and deterministic
per-edge reasons, graph/build Diff, validation, diagnostics, duplicates,
source-order independence, and repeated builds. Existing Writes, metadata
references, Commands, Calls, Opens, and ownership remain unchanged.

Focused validation additions:

```bash
cargo test -p oneagent-graph --test reference_request_build
cargo test -p oneagent-graph --test validation
cargo test -p oneagent-graph --test impact
cargo test -p oneagent-edt reads
```

Run the complete repository implementation gate after focused checks.

##### Task 5 — Complete production and Coverage evidence

Use the [graph implementation profile](codex/profiles/graph-implementation.md)
and [graph emission task template](codex/templates/graph-emission-task.md). Add
the smallest representative real-format EDT fixture project and full-builder
test proving the complete Sprint 8 boundary.

The fixture manifest must map reduced static Query declarations to the exact
real Common Module sources, Configuration declarations, and Accumulation and
Accounting Register descriptors. The matrix must cover both new families and
existing Catalog/Information Register compatibility; parser rejections; missing,
ambiguous, incompatible, and partial resolution; request identity and
provenance; Reads and DependsOn; diagnostics; statistics; validation; Query;
Diff; Impact; reports; source-order independence; repeated builds; and
complete/incremental index equivalence with a clean rebuild.

After all evidence passes, update representative Coverage evidence and current
limitations for Reads, DependsOn, and ReferenceRequest as applicable. Their
Supported statuses and registry aggregate counts must remain unchanged. Sync
Semantic Model 2.0 and this Roadmap to the implemented boundary without claiming
virtual tables, broader grammar, Calculation Registers, new Query sources,
write-derived dependencies, or register payload.

Focused validation additions:

```bash
cargo test -p oneagent-bsl query_language
cargo test -p oneagent-graph coverage
cargo test -p oneagent-edt reads
cargo test -p oneagent-edt coverage
```

Run the complete repository implementation gate after focused checks.

##### Task 6 — Review the integrated Sprint 8 baseline

Use the [review profile](codex/profiles/review.md) and [review task
template](codex/templates/review-task.md). Audit Tasks 1–5 against ADR-0030,
the source investigation, live implementation, prompt suite, and this Roadmap.
Resolve the exact committed planning and Task 5 hashes from live history.

Re-run the full focused parser, resolver, request, graph validation, Query,
Diff, Impact, report, Coverage, production-builder, determinism, and
complete/incremental index-equivalence matrix plus the complete repository
Definition of Done. Verify that existing Query identity, ownership, Catalog and
Information Register Reads, Writes, metadata and Command references,
DependsOn origins, Calls, Opens, diagnostics, reports, and Coverage aggregates
remain compatible.

Create `docs/reviews/sprint-8-registers-queries.md` and update Roadmap state only
for a `pass` or `pass with non-blocking follow-ups` decision with successful
validation. A blocked decision leaves Sprint 8 incomplete and Sprint 9
ineligible. The review must not silently fix findings.

Focused validation additions:

```bash
cargo test -p oneagent-bsl
cargo test -p oneagent-graph
cargo test -p oneagent-edt
```

Run the complete repository implementation gate and record exact test counts.

##### Planning validation and suggested commit

The Sprint 8 kickoff is documentation-only. Validate Markdown consistency,
relative links, prompt numbering, manifest order, dependency gates, suggested
commit messages, accepted-versus-deferred scope, and unchanged `next` status.
No production test is required for the planning change.

Suggested planning commit message, as a recommendation only:

```text
Plan Sprint 8 registers and queries
```

The message does not authorize staging or committing.

##### Sprint 8 state gates and completion criteria

Sprint 8 remained `next` during planning and became `active` when the accepted
planning baseline was committed and Task 1 began under the explicit execution
instruction. Tasks 1–5 were implemented in dependency order, and Task 6 now
records the passing completion decision.

A task is `already_complete` only when current committed evidence and successful
required validation prove every acceptance criterion. Do not create empty
commits. Stop after the first prerequisite, implementation, validation,
staging, commit, or review failure; do not skip a blocked task.

Sprint 8 may transition to `completed` only when:

- Tasks 1–5 are committed in dependency order or proven `already_complete`;
- every ADR-0030 parser, request, resolution, endpoint, projection, provenance,
  diagnostic, statistics, Query, Diff, Impact, report, determinism, Coverage
  evidence, and complete/incremental index-equivalence criterion is proven
  through the production builder;
- existing compatibility behavior and exact registry aggregates remain green;
- current-state architecture and Roadmap match live implementation;
- the complete repository Definition of Done passes;
- Task 6 records `pass` or `pass with non-blocking follow-ups`.

Task 6 records `pass` against committed Task 5 head
`5fce866448a5559a78b812955cda28ebd0492406`; the required validation passed,
Sprint 8 is `completed`, and Sprint 9 Roles and Access Rights is eligible as the
next planning target. The v0.3 release remains planned through Sprint 14.

#### Sprint 9 Roles and Access Rights execution plan

Sprint 9 preserves repository-proven row restrictions on direct EDT role
grants without interpreting them as effective authorization. The accepted
boundary is [ADR-0031](adr/0031-conditional-grants-semantics.md): a conditional
explicit allow remains `Role --Grants--> AccessRight`, while the AccessRight
typed payload and identity preserve the optional opaque row-restriction
condition. Existing unconditional AccessRight identities remain byte-for-byte
compatible.

The planning data gate is satisfied by the real EDT
`restrictionByCondition/condition` artifact in
`adapters/edt/tests/fixtures/role_rights/BaseUser/Rights.rights`, its
provenance-backed copy in the full Grants fixture, the typed
`EdtRoleRowRestriction` parser model and malformed-input tests, the accepted
ADR-0019 direct-only Grants semantics, and the production Grants builder and
consumer suites. The parser already preserves the exact field, so Sprint 9 does
not need a parser task. The private grant resolution observation and insertion
pipeline are one small production boundary in `adapters/edt/src/lib.rs`; they
remain one emission task rather than creating an unobservable intermediate
state.

The current architecture, graph implementation, graph emission, review,
sprint-planning, and sequential-execution framework contracts express the
complete evidence, safety, validation, and reporting requirements. No Codex
Framework change or post-sprint framework audit is justified.

##### Sprint 9 objective

Preserve optional EDT row-restriction conditions as deterministic typed
AccessRight content and conditional direct Grants, prove every generic graph
consumer and production builder remains deterministic, and keep unconditional
identity, deny policy, Coverage status, and deferred authorization semantics
unchanged.

##### Included scope

- typed optional row-restriction content for `NodeKind::AccessRight`;
- additive conditional AccessRight construction and deterministic identity;
- unchanged unconditional AccessRight identity and display compatibility;
- propagation of existing parsed conditions through private grant resolution,
  aggregation, provenance, AccessRight insertion, Grants, and References;
- positive and negative graph-model and EDT production evidence;
- Query, Diff, Impact, report, complete-index, incremental-index, validation,
  repeated-build, provenance, and Coverage-regression evidence;
- synchronized Semantic Model, Coverage evidence text, Roadmap state, and final
  integration review.

##### Excluded scope

- RLS expression parsing, normalization beyond outer whitespace, validation,
  equivalence, compilation, execution, or effective row filtering;
- explicit deny, false-value inference, inherited/default/transitive rights,
  or effective authorization;
- `setForNewObjects`, `setForAttributesByDefault`, and
  `independentRightsOfChildObjects` semantics;
- access profiles, access groups, BSP policy data, runtime users, assignments,
  and role aggregation;
- unsupported protected-resource families, placeholder targets, new NodeKind or
  EdgeKind variants, new condition nodes, and direct Role-to-Metadata grants;
- new persistence, transport, runtime API, CLI, MCP, LSP, or IDE surfaces;
- Coverage capability or aggregate-count changes.

##### Sprint 9 prerequisite gate

Task 01 may begin only from one committed Sprint 9 planning baseline containing
ADR-0031, this Roadmap plan, the Semantic Model synchronization, and the complete
prompt suite under `docs/codex/prompts/sprint-9-roles-access-rights/`. Every
dependent task requires the preceding task's committed outcome. Stored prompt
text never authorizes staging or committing; authorization comes only from the
current execution instruction.

##### Ordered task manifest

| Order | Task | Profile / template | Owned outcome | Required committed prerequisite | Suggested commit message |
|---:|---|---|---|---|---|
| 1 | Implement the conditional AccessRight graph model. | Graph implementation / graph model | Typed payload, additive conditional identity, unconditional compatibility, and generic graph consumer evidence. | Accepted Sprint 9 planning baseline. | `Implement Sprint 9 conditional access rights` |
| 2 | Emit conditional direct role grants. | Graph implementation / graph emission | Existing parsed row restrictions propagated through private resolution, deterministic aggregation, provenance, AccessRight nodes, References, and Grants. | Task 1. | `Emit Sprint 9 conditional role grants` |
| 3 | Complete Sprint 9 production evidence. | Graph implementation / graph emission | Representative full-builder, consumer, index-equivalence, Coverage-regression, and current-state documentation evidence. | Tasks 1–2. | `Complete Sprint 9 production evidence` |
| 4 | Review the integrated Sprint 9 baseline. | Review / review | Findings, full validation evidence, sprint decision, and Sprint 10 hand-off. | Task 3 and all implementation validation. | `Complete Sprint 9 roles and access rights review` |

Dependency graph:

```text
Committed Sprint 9 planning baseline
    -> Task 1 conditional AccessRight graph model
    -> Task 2 conditional Grants production
    -> Task 3 production and documentation evidence
    -> Task 4 integration review
    -> Sprint 10 planning eligibility
```

##### Task 1 — Conditional AccessRight graph model

**Included:** add the ADR-0031 typed AccessRight payload and optional restriction
value, preserve existing constructors and unconditional identity, add an
additive conditional constructor, store payload through
`SemanticGraph::insert_access_right`, reject wrong-kind payloads, and prove
Query, Diff, Impact, report, complete-index, incremental-index, validation, and
Coverage declarations remain deterministic.

**Excluded:** EDT parser or builder changes, new endpoint kinds, edge emission,
fixtures, Coverage status changes, and authorization evaluation.

**Acceptance evidence:** unconditional IDs and names remain exact; equal
conditional inputs deduplicate; absent, present, outer-whitespace, empty,
different-condition, different-right, and different-resource cases are typed
and deterministic; graph consumers retain and compare the payload correctly;
all existing graph behavior remains green.

**Focused validation:**

```bash
cargo test -p oneagent-graph --lib access_right::tests
cargo test -p oneagent-graph --lib node::tests
cargo test -p oneagent-graph --test query
cargo test -p oneagent-graph --test diff
cargo test -p oneagent-graph --test impact
cargo test -p oneagent-graph --test coverage
```

Run the complete workspace implementation gate afterward.

##### Task 2 — Conditional direct Grants production

**Included:** carry `EdtRoleRightDeclaration::row_restriction()` through the
existing private resolved observation; include it in access-right, References,
and Grants aggregation keys; create conditional AccessRight nodes; attach
restriction-aware deterministic provenance; and add focused builder evidence for
conditional/unconditional distinction, identical and distinct conditions,
duplicates, reordered input, resolution outcomes, false values, and repeated
builds.

**Excluded:** parser source-shape changes unless required only to reject an
already-defined empty condition, public reference-request migration, new
diagnostic categories, unsupported resource families, Coverage transitions,
and documentation completion.

**Acceptance evidence:** the real Grants fixture exposes typed conditions for
the two proven Catalog.Product rights; existing unconditional fact counts and
identities change only where the source condition requires a distinct node;
conditional and unconditional declarations never merge; diagnostics and
reference statistics remain correct; all provenance and graph validation pass.

**Focused validation:**

```bash
cargo test -p oneagent-edt role_rights
cargo test -p oneagent-edt --test grants
cargo test -p oneagent-graph --test validation
```

Run the complete workspace implementation gate afterward.

##### Task 3 — Complete production evidence

**Included:** extend representative full-builder and graph-consumer assertions
for conditional payload lookup, Diff, Impact, reports, complete and incremental
index equivalence, deterministic ordering, and unchanged unrelated semantic
facts; synchronize current-state Semantic Model and Coverage evidence; keep
registry statuses and exact aggregate counts unchanged.

**Excluded:** new production semantics, parser grammar, evaluator APIs,
condition-specific query services, Coverage capability additions, and Sprint 9
completion before review.

**Acceptance evidence:** the production builder proves present and absent
restrictions, consumer observability, clean-rebuild equivalence, unrelated
metadata/Calls/Reads/Writes/Includes/Extends/Opens/DependsOn compatibility,
unchanged graph and EDT Coverage aggregates, and complete workspace validation.

Tasks 1–3 are now implemented in dependency order. The real EDT Grants fixture,
generic Query and Resolution facades, Diff, Impact, reports, validation,
complete and incremental Semantic Index transitions, and Coverage regression
tests preserve the accepted conditional direct-grant boundary. Graph Coverage
remains `85` total (`82` Supported, `3` NotApplicable); EDT Coverage remains
`101` total (`96` Supported, `5` NotApplicable). Task 4 records `pass` in the
[Sprint 9 integration review](reviews/sprint-9-roles-access-rights.md).

**Focused validation:**

```bash
cargo test -p oneagent-graph
cargo test -p oneagent-edt --test grants
cargo test -p oneagent-edt --test coverage
cargo test -p oneagent-edt --test semantic_index
```

Run the complete workspace implementation gate afterward.

##### Task 4 — Sprint 9 integration review

Review the exact planning and Task 1–3 commit range without silently fixing
implementation findings. Recheck ADR-0031, every acceptance criterion and
exclusion, exact unconditional compatibility, conditional production evidence,
generic consumers, Coverage aggregates, documentation, repository safety, and
the complete validation matrix. Create
`docs/reviews/sprint-9-roles-access-rights.md` and transition Sprint 9 to
`completed` only for `pass` or `pass with non-blocking follow-ups`.

Focused review additions:

```bash
cargo test -p oneagent-graph
cargo test -p oneagent-edt role_rights
cargo test -p oneagent-edt --test grants
cargo test -p oneagent-edt --test coverage
cargo test -p oneagent-edt --test semantic_index
```

Run the complete workspace validation and record exact command results.

##### Sprint 9 state gates and completion criteria

Sprint 9 remains `next` during planning. It becomes `active` only after the
planning baseline is committed and Task 1 begins. A task is `already_complete`
only when current committed evidence and successful required validation prove
every criterion; no empty commit is created.

Stop after the first prerequisite, implementation, validation, staging, commit,
or review failure. Do not skip, reorder, combine, or partially commit dependent
tasks. A blocked Task 4 leaves Sprint 9 incomplete and Sprint 10 ineligible.

Sprint 9 may transition to `completed` only when Tasks 1–3 are committed or
proven already complete, the complete ADR-0031 model/production/consumer/
provenance/determinism evidence passes, unconditional identity and deferred
scope remain intact, Coverage status and aggregates are unchanged, the complete
repository Definition of Done passes, and Task 4 records a non-blocking review
decision.

Task 4 records `pass` against committed Task 3 head
`0a7b4d7e4d080be92f7a64ddcc9a8eb336a46165`; the focused and complete
validation matrices passed, Sprint 9 is `completed`, and Sprint 10 Subsystems
and Composition is eligible as the next planning target. The v0.3 release
remains planned through Sprint 14.

Planning is documentation-only. Validate Markdown structure, links, prompt
numbering, manifest order, prerequisite graph, commit-message agreement,
accepted-versus-deferred scope, unchanged `next` state, `git diff --check`, and
absence of unrelated changes. Suggested planning commit message:

```text
Plan Sprint 9 roles and access rights
```

#### Sprint 10 Subsystems and Composition execution plan

Sprint 10 extends the completed top-level Subsystem and direct Includes slice
through the bounded hierarchy contract accepted by
[ADR-0032](adr/0032-subsystem-hierarchy-semantics.md). Nested EDT Subsystems
retain the existing UUID-derived metadata and flat semantic nodes. Direct
parent-child hierarchy reuses `Includes` between flat Subsystem nodes, while
transitive metadata membership is computed through Query and never persisted as
derived closure.

The planning data gate is satisfied by the
[source investigation](architecture/subsystem-hierarchy-source-investigation.md)
and the repository-local `OneAgent_EDTproject/src/Subsystems/` corpus selected
by the sprint bootstrap. The root `.gitignore` excludes this real-source tree,
so it is planning evidence rather than a committed fixture. The live corpus contains
127 parseable Subsystem descriptors: 13 top-level and 114 nested through five
levels. Every nested descriptor has exactly one qualified `parentSubsystem`,
every immediate parent has the matching direct `subsystems` declaration, and
all 114 relations agree with physical nesting. Duplicate local names under
different parents prove that complete-path and UUID identity are required.
Existing metadata, Subsystem, Includes, Query, Validation, Diff, Impact,
Coverage, and complete/incremental index consumers provide executable oracles.

The current architecture, parser implementation, graph implementation, graph
model, graph emission, review, sprint-planning, and sequential-execution
framework contracts express the required evidence, safety, validation, and
reporting requirements. No Codex Framework change or post-sprint framework
audit is justified.

##### Sprint 10 objective

Discover repository-proven nested EDT Subsystems deterministically, preserve
their direct hierarchy and direct content as canonical Includes facts, expose
cycle-safe transitive metadata membership without persisted closure, and prove
all existing top-level and unrelated semantic behavior remains compatible.

##### Included scope

- additive `Subsystem --Includes--> Subsystem` graph endpoint and deterministic
  hierarchy-cycle validation;
- one source-independent transitive metadata-membership Query projection;
- recursive discovery through explicit `Subsystems/<Name>` descendants;
- agreement validation among parent `<subsystems>`, child
  `<parentSubsystem>`, and immediate physical nesting;
- existing UUID-derived metadata/flat Subsystem identities and configuration
  ownership for nested descriptors;
- ADR-0020 direct content parsing, resolution, diagnostics, statistics, and
  Includes emission for nested Subsystems;
- deterministic hierarchy provenance, ordering, duplicate handling, repeated
  builds, Diff, reports, and complete/incremental index equivalence;
- one tracked provenance-backed reduced production fixture, Coverage-regression
  evidence, Semantic Model current-
  state synchronization, Roadmap transition, and integration review.

##### Excluded scope

- new NodeKind or EdgeKind variants, persisted transitive closure, or a second
  membership authority;
- hierarchy as `Contains`, metadata Subsystem hierarchy endpoints, directory-
  only inference, or silent repair of contradictory source projections;
- semantic meaning for `Subsystem.<...>` content tokens, including observed
  self-content declarations;
- command-interface files, configuration inventory hierarchy, aliases,
  localized/case-insensitive resolution, extension or cross-project hierarchy,
  external or placeholder Subsystems, and partial recovery from hierarchy
  errors;
- dependency or Impact propagation through Includes;
- unrelated content-prefix or metadata-family expansion;
- persistence, Runtime, API, CLI, MCP, LSP, IDE, authorization, or later-sprint
  behavior;
- speculative Coverage capability or aggregate-count changes.

##### Sprint 10 prerequisite gate

Task 01 may begin only from one committed Sprint 10 planning baseline containing
the source investigation, ADR-0032, this Roadmap plan, the Semantic Model
planning synchronization, and the complete prompt suite under
`docs/codex/prompts/sprint-10-subsystems-composition/`. Every dependent task
requires the preceding task's committed outcome. Stored prompt text never
authorizes staging or committing; authorization comes only from the current
execution instruction.

The immediately preceding prompt suite is exactly
`docs/codex/prompts/sprint-9-roles-access-rights/`, containing its tracked master
prompt and Tasks 01–04. It remains untouched during planning and implementation.
Only the final Sprint 10 review may retire those exact tracked files after a
non-blocking decision and successful complete validation.

##### Ordered task manifest

| Order | Task | Profile / template | Owned outcome | Required committed prerequisite | Suggested commit message |
|---:|---|---|---|---|---|
| 1 | Implement Subsystem hierarchy graph rules. | Graph implementation / graph model | Additive hierarchy endpoint, deterministic cycle validation, transitive membership Query, and generic graph/index evidence. | Accepted Sprint 10 planning baseline. | `Implement Sprint 10 subsystem hierarchy graph rules` |
| 2 | Parse nested Subsystem hierarchy. | Parser implementation / parser | Deterministic recursive source model and strict three-projection agreement with typed failures, without graph emission. | Task 1. | `Parse Sprint 10 nested subsystem hierarchy` |
| 3 | Emit nested Subsystem composition. | Graph implementation / graph emission | Nested metadata/flat nodes, configuration ownership, direct hierarchy and content Includes, provenance, diagnostics, and statistics. | Tasks 1–2. | `Emit Sprint 10 nested subsystem composition` |
| 4 | Complete Sprint 10 production evidence. | Graph implementation / graph emission | Representative corpus/fixture, consumers, index equivalence, Coverage regression, and synchronized current-state docs. | Tasks 1–3. | `Complete Sprint 10 production evidence` |
| 5 | Review the integrated Sprint 10 baseline. | Review / review | Findings, full validation evidence, sprint decision, previous-suite retirement, and Sprint 11 hand-off. | Task 4 and all implementation validation. | `Complete Sprint 10 subsystems and composition review` |

Dependency graph:

```text
Committed Sprint 10 planning baseline
    -> Task 1 hierarchy graph rules
    -> Task 2 nested hierarchy parser
    -> Task 3 nested composition production
    -> Task 4 production and documentation evidence
    -> Task 5 integration review and conditional Sprint 9 suite retirement
    -> Sprint 11 planning eligibility
```

##### Task 1 — Subsystem hierarchy graph rules

**Included:** extend Includes validation additively for flat Subsystem targets;
reject self-loops and deterministic directed hierarchy cycles; add the ADR-0032
read-only transitive metadata-membership Query; prove stable direct edge
identity, duplicate-path result deduplication, wrong/unknown inputs, cycle-safe
query behavior, Diff, reports, complete-index, incremental-index, and unchanged
dependency/Impact policy.

**Excluded:** EDT source parsing, recursive discovery, producer emission,
fixtures, provenance production, Coverage transitions, and unrelated Query
redesign.

**Acceptance evidence:** the old metadata-member Includes endpoint remains
exact; only flat Subsystem-to-Subsystem is added; every other endpoint remains
invalid; transitive results contain unique metadata members ordered by stable
identity and no persisted derived edges; invalid cycles are reported
deterministically; generic consumers and clean index rebuilds remain equivalent.

**Focused validation:**

```bash
cargo test -p oneagent-graph --test query
cargo test -p oneagent-graph --test validation
cargo test -p oneagent-graph --test impact
cargo test -p oneagent-graph --lib incremental_index::tests
```

Run the complete workspace implementation gate afterward.

##### Task 2 — Nested Subsystem hierarchy parser

**Included:** add one focused recursive EDT hierarchy source model; parse direct
`subsystems` and `parentSubsystem`; require exact agreement with immediate
physical nesting and the complete qualified ancestor path; preserve descriptor
UUID/name/path/content inputs; sort deterministically; and return typed fatal
errors for missing, multiple, duplicate, malformed, mismatched, cyclic, escaped,
or unreadable source structures.

**Excluded:** graph node or edge insertion, provenance emission, content target
resolution, diagnostics/statistics projection, Coverage changes, and fallback or
repair behavior.

**Acceptance evidence:** real descriptors across depths 1–5 parse; six
repository-proven duplicate local names remain distinct by ancestor path and
UUID; missing/extra/duplicate declarations and directories, malformed parent
tokens, wrong roots, reordering, cycles, and path escapes have deterministic
outcomes; no graph behavior changes in this task.

**Focused validation:**

```bash
cargo test -p oneagent-edt --lib subsystem_hierarchy::tests
cargo test -p oneagent-edt --lib metadata_object::tests
cargo test -p oneagent-edt --lib subsystem_content::tests
```

Run the complete workspace implementation gate afterward.

##### Task 3 — Nested Subsystem composition production

**Included:** integrate the committed parser into the filesystem graph builder;
insert nested metadata and flat Subsystem nodes with existing IDs; preserve
configuration Contains ownership; emit direct hierarchy Includes with complete
resolved provenance; run nested descriptors through ADR-0020 direct content
resolution; and preserve deterministic diagnostics, statistics, aggregation,
validation, and repeated-build behavior.

**Excluded:** new source grammar, graph kinds, transitive stored edges,
Subsystem content semantics, Coverage transitions, current-state documentation
completion, and unrelated metadata discovery.

**Acceptance evidence:** representative nested fixtures emit exact nodes and
direct edges, including duplicate local names under different parents and
nested metadata content; all source-agreement failures are fatal without partial
graph output; content failures remain recoverable under ADR-0020; repeated and
reordered builds are identical; top-level IDs, facts, diagnostics, and
statistics remain compatible.

**Focused validation:**

```bash
cargo test -p oneagent-edt --test subsystem_hierarchy
cargo test -p oneagent-edt --test includes
cargo test -p oneagent-graph --test validation
```

Run the complete workspace implementation gate afterward.

##### Task 4 — Complete production evidence

**Included:** add a tracked provenance-documented reduced hierarchy fixture
derived from the live ignored source corpus; prove depth, direct and transitive membership,
duplicate local names, add/remove/reparent/content transitions, Query, Diff,
reports, validation, complete and incremental index clean-rebuild equivalence,
unchanged dependency/Impact behavior, Coverage registry stability, and
unrelated semantic compatibility; synchronize Semantic Model and Roadmap
current-state text without completing the sprint.

**Excluded:** new parser or graph semantics, hierarchy-aware dependency/Impact,
new Coverage capability rows without live registry proof, command-interface or
unsupported content families, and Sprint 11 planning.

**Acceptance evidence:** every ADR-0032 completion criterion is executable;
repository-proven depth and duplicate-name shapes are represented; no persisted
transitive edge exists; generic and indexed consumers agree with clean builds;
graph and EDT Coverage statuses and aggregate counts change only if derived live
registry evidence requires it; the complete workspace gate passes.

**Implemented current state:** the tracked
`adapters/edt/tests/fixtures/sprint10_subsystems_project/` fixture records exact
live-source paths, selected fields, source hashes, and reduced-artifact hashes.
Production-builder tests cover depths 1–5, duplicate local names, shared and
nested direct members, deferred self-content, provenance, deterministic Query,
Diff, reports, validation, and repeated builds. Complete and incremental index
tests cover hierarchy/member add, remove, reparent, and content replacement
against clean rebuilds. EDT Coverage remains 101 capabilities (96 `Supported`,
5 `NotApplicable`); graph Coverage remains 85 capabilities (82 `Supported`, 3
`NotApplicable`). Task 5 records `pass`; Sprint 10 is `completed`.

**Focused validation:**

```bash
cargo test -p oneagent-graph
cargo test -p oneagent-edt --test subsystem_hierarchy
cargo test -p oneagent-edt --test includes
cargo test -p oneagent-edt --test coverage
cargo test -p oneagent-edt --test semantic_index
```

Run the complete workspace implementation gate afterward.

##### Task 5 — Sprint 10 integration review

Review the exact planning and Task 1–4 commit range without silently fixing
implementation findings. Recheck ADR-0032, source-agreement failures, graph
endpoint/cycle behavior, nested production, provenance, direct and transitive
membership, all consumers, Coverage, documentation, repository safety, and the
complete validation matrix. Create
`docs/reviews/sprint-10-subsystems-composition.md` and transition Sprint 10 to
`completed` only for `pass` or `pass with non-blocking follow-ups`.

After a non-blocking decision and successful complete validation, atomically
retire every tracked prompt file under the verified immediately preceding suite
`docs/codex/prompts/sprint-9-roles-access-rights/` in the same review commit.
Any inventory mismatch, untracked endangered file, or retained link dependency
blocks retirement and the final commit.

Focused review additions:

```bash
cargo test -p oneagent-graph
cargo test -p oneagent-edt --lib subsystem_hierarchy::tests
cargo test -p oneagent-edt --test subsystem_hierarchy
cargo test -p oneagent-edt --test includes
cargo test -p oneagent-edt --test coverage
cargo test -p oneagent-edt --test semantic_index
```

Run the complete workspace validation and record exact command results.

**Review result:** the exact planning and Task 1–4 range passed the focused and
complete validation matrices with no blocking or non-blocking findings,
missing evidence, open questions, or scope violations. The
[Sprint 10 integration review](reviews/sprint-10-subsystems-composition.md)
records the acceptance matrix and executed results. Sprint 11 Event
Subscriptions is `next`; the verified Sprint 9 prompt suite is retired in the
same review commit.

##### Sprint 10 state gates and completion criteria

Sprint 10 remains `next` during planning. It becomes `active` only after the
planning baseline is committed and Task 1 begins. A task is `already_complete`
only when current committed evidence and successful required validation prove
every criterion; no empty commit is created.

Stop after the first prerequisite, implementation, validation, staging, commit,
or review failure. Do not skip, reorder, combine, or partially commit dependent
tasks. A blocked Task 5 leaves Sprint 10 incomplete, keeps the Sprint 9 prompt
suite intact, and leaves Sprint 11 ineligible.

Sprint 10 may transition to `completed` only when Tasks 1–4 are committed or
proven already complete, the complete ADR-0032 parser/model/production/query/
provenance/determinism evidence passes, top-level and unrelated compatibility
remain intact, Coverage state is truthful, the complete repository Definition
of Done passes, and Task 5 records a non-blocking review decision. Only then may
Sprint 11 Event Subscriptions become `next` and the exact Sprint 9 prompt suite
be retired in the final review commit.

Planning is documentation-only. Validate Markdown structure, links, prompt
numbering, manifest order, prerequisite graph, commit-message agreement,
accepted-versus-deferred scope, unchanged `next` state, verified previous-suite
inventory, `git diff --check`, and absence of unrelated changes. Suggested
planning commit message:

```text
Plan Sprint 10 subsystems and composition
```

#### Sprint 11 Event Subscriptions execution plan

Sprint 11 was planned from committed Sprint 10 review head
`62d22c53d0e0c7f077d477398fe899c311dd5cc3`. At the planning baseline the review
recorded `pass`, Sprint 10 was `completed`, and Sprint 11 was the only sprint
marked `next`. Tasks 1-5 were implemented in dependency order, and Task 6 now
records `pass`; Sprint 11 is `completed` and Sprint 12 is `next`.

The repository-owned
[source investigation](architecture/event-subscription-source-investigation.md)
proves 99 real EDT descriptors, 314 source-selector occurrences, 18 event
values, and 93 unique handler paths. Every handler resolves to an existing
exported Common Module Procedure, and every qualified selector belonging to a
currently modeled metadata family has a real target. Four multiline Procedure
declarations were initially misclassified by a line-oriented export audit; the
tracked reduced fixture documents the correction and recomposes an exact live
non-exported owned Procedure to test the accepted export-agnostic binding rule.
[ADR-0033](adr/0033-event-subscription-semantics.md) accepts the bounded
metadata, payload, source-selection, handler-resolution, References, and
Triggers contracts. Unsupported source families remain observable diagnostics
and do not authorize speculative metadata entities.

The current graph implementation, parser implementation, graph model, graph
emission, review, sprint-planning, and sequential-execution framework contracts
express the required evidence, safety, validation, and reporting requirements.
The Task prompt template readiness forecast requires no framework update until
Sprint 14, and no concrete Sprint 11 gap was found. No Codex Framework change or
post-sprint framework audit is justified.

##### Sprint 11 objective

Discover repository-proven EDT Event Subscriptions, preserve stable identity
and typed event content, resolve supported source selectors and Common Module
handler procedures, emit direct provenance-backed References and Triggers
relations, and prove deterministic compatibility across generic graph and
index consumers without inventing unsupported metadata families or runtime
dispatch semantics.

##### Included scope

- additive `MetadataKind::EventSubscription` and typed event-name payload;
- additive `EdgeKind::Triggers` with one precise EventSubscription-to-Procedure
  endpoint;
- additive References endpoints from Event Subscription to supported source
  metadata and Procedure;
- top-level `src/EventSubscriptions` discovery, UUID/name/synonym/event/source/
  handler parsing, and configuration ownership;
- exact qualified and complete family source selection for Catalog, Document,
  Information Register, Accumulation Register, Accounting Register,
  Calculation Register, Business Process, and Task;
- exact Common Module ownership resolution to Procedure, including
  non-exported handlers;
- typed malformed, unsupported, missing, ambiguous, incompatible, and invalid-
  owner outcomes without placeholder graph facts;
- deterministic provenance, diagnostics, legacy statistics, Query, Validation,
  Diff, Impact-policy, reports, complete index, incremental index, and repeated
  builds;
- one tracked provenance-backed reduced production fixture, truthful Coverage
  transitions, Semantic Model current-state synchronization, Roadmap transition,
  and integration review.

##### Excluded scope

- Constant, Defined Type, Exchange Plan, Chart of Accounts, Chart of
  Calculation Types, and Chart of Characteristic Types metadata modeling;
- public multi-target ADR-0024 request-ledger migration;
- Function handlers, case-insensitive aliases, extensions, cross-project or
  partial-workspace source selection;
- handler signature validation, runtime dispatch, event frequency, ordering,
  priorities, activation conditions, and closed platform event enumeration;
- Event Subscription comments or unproven XML fields;
- Triggers-based dependency or Impact propagation, event-specific Query APIs,
  derived reachability, or stored closure;
- persistence, Runtime, API, CLI, MCP, LSP, IDE, Designer XML, SKD, XDTO, or
  service behavior;
- unrelated Coverage transitions or metadata-family expansion.

##### Sprint 11 prerequisite gate

Task 01 may begin only from one committed Sprint 11 planning baseline containing
the source investigation, ADR-0033, this Roadmap plan, Semantic Model planning
synchronization, and the complete prompt suite under
`docs/codex/prompts/sprint-11-event-subscriptions/`. Every dependent task
requires the preceding task's committed outcome. Stored prompt text never
authorizes staging or committing; authorization comes only from the current
execution instruction.

The immediately preceding prompt suite is exactly
`docs/codex/prompts/sprint-10-subsystems-composition/`, containing these six
tracked files:

- `00-sprint-10-execution-loop.md`;
- `01-implement-subsystem-hierarchy-graph-rules.md`;
- `02-parse-nested-subsystem-hierarchy.md`;
- `03-emit-nested-subsystem-composition.md`;
- `04-complete-sprint-10-production-evidence.md`;
- `05-sprint-10-integration-review.md`.

It remains untouched during planning and implementation. Only the final Sprint
11 review may retire those exact tracked files after a non-blocking decision
and successful complete validation.

##### Ordered task manifest

| Order | Task | Profile / template | Owned outcome | Required committed prerequisite | Suggested commit message |
|---:|---|---|---|---|---|
| 1 | Implement Event Subscription graph model. | Graph implementation / graph model | Metadata kind, typed payload, References/Triggers endpoint rules, public exhaustive-consumer migration, and generic graph evidence. | Accepted Sprint 11 planning baseline. | `Implement Sprint 11 event subscription graph model` |
| 2 | Parse Event Subscription descriptors. | Parser implementation / parser | Deterministic typed EDT descriptor, selector, event, and handler parsing without resolution or graph emission. | Task 1. | `Parse Sprint 11 event subscription descriptors` |
| 3 | Resolve Event Subscription targets. | Graph implementation / graph emission | Adapter-private exact/family source and owned-handler resolution outcomes, diagnostics, statistics policy, and focused tests without production integration. | Tasks 1–2. | `Resolve Sprint 11 event subscription targets` |
| 4 | Emit Event Subscription semantics. | Graph implementation / graph emission | Production discovery, metadata/ownership insertion, source and handler References, Triggers, provenance, diagnostics, statistics, and determinism. | Tasks 1–3. | `Emit Sprint 11 event subscription semantics` |
| 5 | Complete Sprint 11 production evidence. | Graph implementation / graph emission | Provenance-backed fixture, consumer/index transitions, Coverage transitions, aggregate verification, and current-state documentation. | Tasks 1–4. | `Complete Sprint 11 production evidence` |
| 6 | Review the integrated Sprint 11 baseline. | Review / review | Findings, full validation evidence, sprint decision, Sprint 10 suite retirement, and Sprint 12 hand-off. | Task 5 and all implementation validation. | `Complete Sprint 11 event subscriptions review` |

Dependency graph:

```text
Committed Sprint 11 planning baseline
    -> Task 1 graph model
    -> Task 2 parser
    -> Task 3 target resolution
    -> Task 4 production emission
    -> Task 5 production, Coverage, and documentation evidence
    -> Task 6 integration review and conditional Sprint 10 suite retirement
    -> Sprint 12 planning eligibility
```

##### Task 1 — Event Subscription graph model

**Included:** add `MetadataKind::EventSubscription`, stable machine code,
closed typed event payload, payload-kind validation, `EdgeKind::Triggers`,
precise Triggers and additive ADR-0025 References endpoint matrices, exhaustive
enum-consumer updates, canonical identity/equality/diff behavior, generic Query,
Validation, report, Impact-policy, complete-index, incremental-index, and
Coverage-model evidence.

**Excluded:** EDT parsing, production discovery, target resolution, edge
emission, fixtures, EDT diagnostics/statistics, and final Coverage transitions.

**Acceptance evidence:** payload-only changes preserve UUID node identity and
produce semantic-content modification; existing MetadataKind and EdgeKind codes
remain unchanged; all accepted source-metadata and Procedure References pairs
and the one Triggers pair validate; every reversed, Unknown, Function, Module,
unsupported metadata, and unrelated pair fails deterministically; Triggers is
queryable and diffable but does not enter dependency/Impact classification;
generic and indexed behavior is deterministic.

**Focused validation:**

```bash
cargo test -p oneagent-metadata
cargo test -p oneagent-graph
```

Run the complete workspace implementation gate afterward.

##### Task 2 — Event Subscription descriptor parser

**Included:** add one focused EDT parser for the exact root UUID, name, optional
synonym, at least one direct source `types`, non-empty event, and one
`CommonModule.<module>.<procedure>` handler; preserve occurrence ordinals;
classify supported exact/family, unsupported, and malformed source selectors;
return typed fatal descriptor errors and deterministic ordered observations.

**Excluded:** graph node or edge insertion, metadata target lookup, handler
ownership lookup, diagnostics/statistics projection, public request-ledger
migration, production fixture, and Coverage changes.

**Acceptance evidence:** raw repository shapes with 1, 30, 41, and 94 source
entries parse; present/absent/non-ASCII synonym and all 18 observed event names
remain exact; exported and non-exported handler paths parse identically;
missing, duplicate, empty, malformed, wrong-root, invalid UUID/name,
three-component selector, bad handler namespace/depth, multiple descriptor,
unreadable, and reordered cases have deterministic outcomes; no graph behavior
changes.

**Focused validation:**

```bash
cargo test -p oneagent-edt --lib event_subscription::tests
cargo test -p oneagent-edt --lib metadata_object::tests
```

Run the complete workspace implementation gate afterward.

##### Task 3 — Event Subscription target resolution

**Included:** add adapter-private deterministic resolution for exact supported
source names, complete supported source families, and exact Common Module
Module/Procedure ownership; preserve duplicate observation context; classify
resolved, missing, ambiguous, incompatible, invalid-owner, malformed, and
unsupported outcomes; define one statistics outcome per selector or handler
without graph edge insertion.

**Excluded:** production directory discovery, Event Subscription node or
ownership insertion, References/Triggers emission, public ADR-0024 ledger
migration, fixture/Coverage completion, and unsupported metadata entities.

**Acceptance evidence:** every supported prefix maps to the accepted kind;
family results are complete, unique, and stable-ID ordered; exact results use
name and kind; equivalent manager/object observations retain distinct
provenance inputs without duplicate targets; handler resolution accepts owned
non-exported Procedure and rejects Function, wrong owner, missing, ambiguous,
and malformed paths; reordered inputs are equal.

**Focused validation:**

```bash
cargo test -p oneagent-edt --lib event_subscription_resolution::tests
cargo test -p oneagent-graph --test resolution
```

Run the complete workspace implementation gate afterward.

##### Task 4 — Event Subscription semantic emission

**Included:** add production `EventSubscriptions` discovery; insert typed
metadata nodes and configuration Contains ownership; resolve after metadata and
BSL declarations exist; emit source and handler References plus handler
Triggers with deterministic aggregated provenance; project typed diagnostics
and reference statistics once per observation; preserve recoverable resolution
failure behavior and complete repeated-build determinism.

**Excluded:** new parser grammar, new graph kinds beyond Task 1, unsupported
source metadata, public request ledger, final fixture/Coverage transition,
current-state documentation completion, and Triggers dependency policy.

**Acceptance evidence:** representative generated production projects prove
exact and family sources, manager/object duplicate target aggregation, exported
and non-exported handlers, event payload, ownership, Query, validation,
provenance, diagnostics/statistics, missing/ambiguous/incompatible/unsupported
outcomes, no placeholder facts, source reordering, repeated builds, and
unchanged unrelated graph behavior.

**Focused validation:**

```bash
cargo test -p oneagent-edt --test event_subscriptions
cargo test -p oneagent-graph --test validation
```

Run the complete workspace implementation gate afterward.

##### Task 5 — Complete production evidence

**Included:** add a tracked provenance-documented reduced fixture derived from
the live ignored corpus; prove exact/family/duplicate/unsupported selectors,
exported/non-exported handlers, payload and relation changes, Query, Diff,
Impact-policy, reports, validation, complete/incremental index equivalence,
Coverage transitions, aggregate recomputation, and synchronized Semantic Model
and Roadmap current-state text without completing the sprint.

**Excluded:** new parser or graph semantics, unsupported metadata families,
request-ledger migration, Triggers dependency propagation, Runtime behavior,
and Sprint 12 planning.

**Acceptance evidence:** every ADR-0033 completion criterion is executable; the
fixture README records exact source paths and hashes; add/remove/modify source,
event, and handler transitions match clean rebuilds; graph and EDT Coverage
statuses and aggregate counts are derived from live registries; all unrelated
capabilities and full workspace behavior remain compatible.

**Focused validation:**

```bash
cargo test -p oneagent-metadata
cargo test -p oneagent-graph
cargo test -p oneagent-edt --test event_subscriptions
cargo test -p oneagent-edt --test coverage
cargo test -p oneagent-edt --test semantic_index
```

Run the complete workspace implementation gate afterward.

**Current state:** Tasks 1-5 are implemented in dependency order. The tracked
Sprint 11 fixture records exact live paths, source hashes, reduction treatment,
and reduced-artifact SHA-256 values; it proves exact, family, equivalent
manager/object, unsupported, exported, and recomposed live non-exported
handler cases. Production-builder tests cover typed payload and ownership,
References/Triggers, provenance, diagnostics/statistics, unchanged public
request ledger, generic Query, Diff, report, Validation, dependency/Impact
policy, repeated builds, and source/handler/subscription transitions. Complete
and incremental Semantic Index evidence matches clean rebuilds for add, remove,
event change, source add/remove/retarget, and handler retarget.

The executable Graph Domain registry remains 88 capabilities: 84 `Supported`
and 4 `NotApplicable`. The EDT registry remains 104 capabilities and now has 99
`Supported` and 5 `NotApplicable`; the Event Subscription metadata entity,
metadata node, and Triggers edge transitions close the three planned High gaps.
Both registries have zero Critical, High, or Medium gaps. Unsupported source
families, public multi-target requests, Triggers dependency/Impact propagation,
and runtime dispatch remain deferred. The Sprint 11 integration review records
`pass`; Sprint 11 is `completed` and Sprint 12 is eligible as `next`.

##### Task 6 — Sprint 11 integration review

Review the exact planning and Task 1–5 commit range without silently fixing
implementation findings. Recheck ADR-0033, parser failures, source and handler
resolution, endpoint policy, payload, production emission, provenance,
diagnostics/statistics, consumers, indexes, Coverage, documentation, repository
safety, and the complete validation matrix. Create
`docs/reviews/sprint-11-event-subscriptions.md` and transition Sprint 11 to
`completed` only for `pass` or `pass with non-blocking follow-ups`.

After a non-blocking decision and successful complete validation, atomically
retire every tracked prompt file under the verified immediately preceding suite
`docs/codex/prompts/sprint-10-subsystems-composition/` in the same review
commit. Any inventory mismatch, untracked endangered file, or retained link
dependency blocks retirement and the final commit.

Focused review additions:

```bash
cargo test -p oneagent-metadata
cargo test -p oneagent-graph
cargo test -p oneagent-edt --lib event_subscription::tests
cargo test -p oneagent-edt --lib event_subscription_resolution::tests
cargo test -p oneagent-edt --test event_subscriptions
cargo test -p oneagent-edt --test coverage
cargo test -p oneagent-edt --test semantic_index
```

Run the complete workspace validation and record exact command results.

**Review outcome:** The exact planning-through-Task-5 range
`62d22c53d0e0c7f077d477398fe899c311dd5cc3..ea2294e12505f80dce0d55e43a30fab8f2b78756`
passed the complete acceptance matrix and focused/full validation with no
findings or missing evidence. The
[Sprint 11 integration review](reviews/sprint-11-event-subscriptions.md)
records the evidence and decision. Sprint 11 is `completed`; Sprint 12 SKD and
Report Model is `next`.

##### Sprint 11 state gates and completion criteria

Sprint 11 remains `next` during planning. It becomes `active` only after the
planning baseline is committed and Task 1 begins. A task is `already_complete`
only when current committed evidence and successful required validation prove
every criterion; no empty commit is created.

Stop after the first prerequisite, implementation, validation, staging, commit,
or review failure. Do not skip, reorder, combine, or partially commit dependent
tasks. A blocked Task 6 leaves Sprint 11 incomplete, keeps the Sprint 10 prompt
suite intact, and leaves Sprint 12 ineligible.

Sprint 11 may transition to `completed` only when Tasks 1–5 are committed or
proven already complete, the complete ADR-0033 metadata/parser/resolution/
production/provenance/determinism evidence passes, unsupported source families
remain deferred, unrelated compatibility and truthful Coverage state are
preserved, the complete repository Definition of Done passes, and Task 6
records a non-blocking review decision. Only then may Sprint 12 SKD and Report
Model become `next` and the exact Sprint 10 prompt suite be retired in the final
review commit.

Planning is documentation-only. Validate Markdown structure, links, prompt
numbering, manifest order, prerequisite graph, commit-message agreement,
accepted-versus-deferred scope, unchanged `next` state, verified previous-suite
inventory, `git diff --check`, and absence of unrelated changes. Suggested
planning commit message:

```text
Plan Sprint 11 event subscriptions
```

#### Sprint 12 SKD and Report Model execution plan

Sprint 12 was planned from committed Sprint 11 review head
`8b0d22ef955129d4bf6eb88549529a81baf9c466`. The review records `pass`, Sprint
11 is `completed`, and Sprint 12 completed its ordered implementation,
evidence, and review tasks with a `pass` decision recorded in the
[Sprint 12 integration review](reviews/sprint-12-skd-report-model.md).

The repository-owned
[source investigation](architecture/report-data-composition-source-investigation.md)
proves 56 Report descriptors, 56 UUID-backed Data Composition Schema artifacts,
70 uniquely named direct data sets, 970 uniquely named direct fields, and 38
direct Query data sets. [ADR-0034](adr/0034-report-data-composition-semantics.md)
accepts the smallest identity-safe entity and ownership slice. Eight nested
duplicate-name Union query data sets and six field folders remain typed deferred
observations. A live audit proves that none of 46 direct-or-nested DCS queries
satisfies the current complete-source Query parser, so Sprint 12 emits no
partial Reads, DependsOn, References, or QuerySource requests.

The current graph implementation, parser implementation, graph emission,
review, sprint-planning, and sequential-execution framework contracts express
the required source, identity, validation, safety, Coverage, and reporting
requirements. The Task prompt template readiness forecast requires no framework
update until Sprint 14, and no concrete Sprint 12 gap was found. No Codex
Framework change or post-sprint framework audit is justified.

##### Sprint 12 objective

Preserve repository-proven Report-owned Data Composition Schemas, direct Data
Sets, named direct Data Composition Fields, and metadata-owned Query declarations
as deterministic provenance-backed graph entities with exact immediate
ownership, typed content, diagnostics, and generic consumer/index compatibility,
without inventing nested Union identities or partially resolving unsupported
DCS query grammar.

##### Included scope

- additive Data Composition Schema, Data Set, and Data Composition Field node
  kinds with closed typed payloads;
- collision-safe Schema UUID, owner-scoped direct Data Set/Field, and fixed-role
  metadata-owned Query identities;
- precise Report-to-Schema, Schema-to-DataSet, DataSet-to-Field, and
  DataSet-to-Query Contains ownership;
- Report DCS template declarations, optional main selection, exact artifact
  correspondence, Data Composition Schema root, local data source, direct Query,
  Object, and Union data sets, direct named fields, and complete query text;
- production discovery through the existing Reports path, node/ownership
  emission, provenance, typed fatal/deferred outcomes, diagnostics, statistics,
  filesystem/XML reordering, and repeated-build determinism;
- generic Query, Diff, Impact exclusion, reports, Validation, complete index,
  incremental clean-rebuild equivalence, Coverage, and aggregate evidence;
- one tracked provenance-backed reduced production fixture, current-state
  synchronization, and integration review.

##### Excluded scope

- nested Union child identities, nested data sets/fields/queries, and field
  folders;
- DCS parameters, calculated fields outside the accepted direct field element,
  roles, appearances, templates, settings, variants, layouts, filters, totals,
  and runtime composition behavior;
- DCS query-language expansion, virtual tables, batches, temporary tables,
  QuerySource requests, Reads, DependsOn, References, result schemas, and
  field-level lineage;
- non-Report schemas, Common Templates, external resources, extensions, partial
  workspaces, Designer XML, XDTO, services, Runtime, API, CLI, MCP, LSP, IDE,
  persistence, or serialization;
- unrelated graph/EDT capabilities, dependencies, refactors, or Coverage
  transitions.

##### Sprint 12 prerequisite gate

Task 01 may begin only from one committed Sprint 12 planning baseline containing
the source investigation, accepted ADR-0034, this Roadmap plan, Semantic Model
planning synchronization, and the complete prompt suite under
`docs/codex/prompts/sprint-12-skd-report-model/`. Every dependent task requires
the preceding task's committed outcome. Stored prompt text never authorizes
staging or committing; authorization comes only from the current execution
instruction.

The immediately preceding prompt suite is exactly
`docs/codex/prompts/sprint-11-event-subscriptions/`, containing these seven
tracked files:

- `00-sprint-11-execution-loop.md`;
- `01-implement-event-subscription-graph-model.md`;
- `02-parse-event-subscription-descriptors.md`;
- `03-resolve-event-subscription-targets.md`;
- `04-emit-event-subscription-semantics.md`;
- `05-complete-sprint-11-production-evidence.md`;
- `06-sprint-11-integration-review.md`.

It remains untouched during planning and implementation. Only the final Sprint
12 review may retire those exact tracked files after a non-blocking decision
and successful complete validation.

##### Ordered task manifest

| Order | Task | Profile / template | Owned outcome | Required committed prerequisite | Suggested commit message |
|---:|---|---|---|---|---|
| 1 | Implement the Data Composition graph model. | Graph implementation / graph model | Typed nodes/payloads, collision-safe identities, precise Contains ownership, public exhaustive-consumer migration, indexes, and graph Coverage evidence. | Accepted Sprint 12 planning baseline. | `Implement Sprint 12 data composition graph model` |
| 2 | Parse Report Data Composition Schemas. | Parser implementation / parser | Deterministic typed Report/DCS source model with accepted direct entities, fatal errors, and deferred nested/folder outcomes, without graph emission. | Task 1. | `Parse Sprint 12 report data composition schemas` |
| 3 | Emit Report Data Composition semantics. | Graph implementation / graph emission | Production Report/DCS joining, nodes, ownership, metadata-owned Queries, provenance, diagnostics, statistics, and deterministic generated-project evidence. | Tasks 1–2. | `Emit Sprint 12 report data composition semantics` |
| 4 | Complete Sprint 12 production evidence. | Graph implementation / graph emission | Provenance-backed fixture, generic consumers, complete/incremental indexes, EDT Coverage, aggregate counts, and current-state documentation. | Tasks 1–3. | `Complete Sprint 12 production evidence` |
| 5 | Review the integrated Sprint 12 baseline. | Review / review | Findings, full validation evidence, sprint decision, Sprint 11 suite retirement, and Sprint 13 hand-off. | Task 4 and all implementation validation. | `Complete Sprint 12 SKD and report model review` |

Dependency graph:

```text
Committed Sprint 12 planning baseline
    -> Task 1 graph model
    -> Task 2 parser
    -> Task 3 production emission
    -> Task 4 production, Coverage, and documentation evidence
    -> Task 5 integration review and conditional Sprint 11 suite retirement
    -> Sprint 13 planning eligibility
```

##### Task 1 — Data Composition graph model

**Included:** add `NodeKind::DataCompositionSchema`, `NodeKind::DataSet`, and
`NodeKind::DataCompositionField`; closed Schema main-role, Data Set kind/source,
and Field data-path payloads; length-prefixed owner-scoped identity helpers;
precise Contains endpoints and unique immediate ownership; additive DataSet
ownership for Query; exhaustive public consumer updates; generic Query, Diff,
Impact policy, reports, Validation, complete/incremental index, and graph-domain
Coverage evidence.

**Excluded:** EDT parsing, Report discovery changes, production insertion,
diagnostics/statistics, fixtures, and EDT Coverage transitions.

**Acceptance evidence:** existing variants/codes/identities remain unchanged;
new payloads reject every wrong kind; content-only changes preserve IDs and are
modified Diff facts; delimiter-containing identity inputs do not collide;
exactly four new Contains pairs validate while reversed/transitive/unrelated
pairs fail; every accepted child has one owner; existing Procedure/Function
Query ownership remains compatible; generic consumers and complete/incremental
indexes are deterministic; Contains adds no Impact dependency.

Focused validation:

```bash
cargo test -p oneagent-metadata
cargo test -p oneagent-graph
```

Run the complete workspace implementation gate afterward.

##### Task 2 — Report Data Composition parser

**Included:** add one focused parser joining an existing Report descriptor's
UUID/name to direct DCS template UUID/name/type declarations, optional exact
main selection, exact `Templates/<name>/Template.dcs` artifacts, DCS namespace,
root local data source, direct Query/Object/Union data sets, named direct fields,
and one complete Query text per direct Query data set; return canonical typed
accepted and nested/folder/unknown deferred observations.

**Excluded:** graph node/edge insertion, production diagnostics/statistics,
query-language parsing, target resolution, Reads/DependsOn/References, tracked
production fixture, and Coverage changes.

**Acceptance evidence:** all 56 live declarations and files agree; valid empty,
main, non-main, Query, Object, and Union shapes parse; accepted direct names are
unique and order-independent; missing/extra/ambiguous artifacts, duplicate
UUID/name, malformed main selection, wrong root/namespace, malformed XML,
unreadable file, invalid required values, data-source mismatch, query
cardinality mismatch, and duplicate direct entities are typed deterministic
errors; eight nested duplicate-name Union children and six folders are distinct
deferred outcomes; repeated reads are equal.

Focused validation:

```bash
cargo test -p oneagent-edt --lib report_data_composition::tests
cargo test -p oneagent-edt --lib metadata_object::tests
```

Run the complete workspace implementation gate afterward.

##### Task 3 — Report Data Composition semantic emission

**Included:** integrate the committed parser with the existing Reports
production path; retain Report identity/payload/members/modules; insert Schema,
direct Data Set, direct named Field, and metadata-owned Query nodes plus exact
Contains ownership; attach deterministic content-bearing provenance; project
deferred/unsupported outcomes through diagnostics and legacy statistics; add
generated project success/failure/reordering/repeated-build tests.

**Excluded:** new graph/parser semantics, query-language analysis or source
relations, final provenance-backed fixture, broad index/Coverage evidence,
current-state documentation completion, and unsupported nested entities.

**Acceptance evidence:** valid main/non-main/empty/Query/Object/Union projects
emit exact typed nodes and immediate owners without changing Report behavior;
Query text changes preserve Query ID and produce modified evidence; nested/folder
observations emit no placeholder nodes and are typed/counted once; fatal source
errors produce no successful partial build; no QuerySource request, Reads,
DependsOn, or References is created; Query/Validation observe accepted facts;
reordered source and repeated builds are equal; unrelated regressions pass.

Focused validation:

```bash
cargo test -p oneagent-edt --test report_data_composition
cargo test -p oneagent-graph --test validation
```

Run the complete workspace implementation gate afterward.

##### Task 4 — Complete production evidence

**Included:** add a tracked provenance-documented reduced fixture covering
Query, Object, Union, empty, non-main, nested-deferred, and folder-deferred live
shapes; prove generic Query, Diff, Impact exclusion, reports, Validation,
reordered/repeated builds, complete index, and incremental clean-rebuild
transitions; transition only justified graph/EDT Coverage evidence; recompute
aggregate counts; synchronize Semantic Model and Roadmap current-state text
without completing the sprint.

**Excluded:** new entity, parser, identity, ownership, query grammar, relation,
diagnostic, or statistics semantics; Sprint 13 planning; previous-suite
retirement; unrelated Coverage changes.

**Acceptance evidence:** every applicable ADR-0034 completion criterion is
executable; the fixture README records exact live paths, source hashes,
reduction treatment, and reduced hashes; add/remove/modify/main/data-set/field/
query/deferred transitions match clean rebuilds; Coverage statuses/evidence and
aggregate counts derive from live registries; all query-source relations remain
absent; full workspace validation succeeds.

**Current state after Task 4:** the production Report path joins tracked
live-derived Report descriptors and DCS artifacts into typed Schema, direct
Data Set, direct named Field, and metadata-owned Query nodes with exact
immediate ownership and deterministic provenance. The fixture under
`adapters/edt/tests/fixtures/sprint12_report_data_composition_project/` records
every ignored live origin, source hash, reduction treatment, and reduced hash.
Generic Query, Diff, report, Validation, Impact exclusion, complete index, and
incremental clean-rebuild transitions cover accepted payload, ownership,
add/remove, reordered, repeated, and deferred-observation changes. The
executable Graph Domain registry has 91 capabilities: 87 `Supported` and 4
`NotApplicable`; EDT has 110 capabilities: 105 `Supported` and 5
`NotApplicable`. Both registries have no gaps. DCS QuerySource requests and
`Reads`, `DependsOn`, and `References` remain absent. This was the committed
Task 4 gate; Task 5 subsequently recorded `pass` without changing production
behavior.

Focused validation:

```bash
cargo test -p oneagent-metadata
cargo test -p oneagent-graph
cargo test -p oneagent-edt --test report_data_composition
cargo test -p oneagent-edt --test coverage
cargo test -p oneagent-edt --test semantic_index
```

Run the complete workspace implementation gate afterward.

##### Task 5 — Sprint 12 integration review

Review the exact planning-through-Task-4 commit range against ADR-0034, source
investigation, prompt suite, and live repository. Verify entity/payload/identity,
source parser, fatal/deferred policy, emission, ownership, provenance,
diagnostics/statistics, no partial query relations, generic consumers, complete
and incremental indexes, Coverage, documentation, compatibility, exclusions,
and repository safety. Do not silently fix findings.

Create `docs/reviews/sprint-12-skd-report-model.md` and transition Sprint 12 to
`completed` with Sprint 13 as `next` only after `pass` or `pass with
non-blocking follow-ups` and successful focused/full validation. A blocked
decision creates no completion transition or review commit.

After a non-blocking decision and successful complete validation, atomically
retire every tracked prompt file under the verified immediately preceding suite
`docs/codex/prompts/sprint-11-event-subscriptions/` in the same review commit.
Any inventory mismatch, endangered untracked file, or retained link dependency
blocks retirement and the final commit.

Focused review additions:

```bash
cargo test -p oneagent-metadata
cargo test -p oneagent-graph
cargo test -p oneagent-edt --lib report_data_composition::tests
cargo test -p oneagent-edt --test report_data_composition
cargo test -p oneagent-edt --test coverage
cargo test -p oneagent-edt --test semantic_index
```

Run the complete workspace validation and record exact command results.

**Final state after Task 5:** the independent integration review records
`pass` against committed Task 4 head
`ba9f8350bc78784052a56ab95680a019719a1792`. Sprint 12 is `completed`, Sprint
13 XDTO and Service Model is `next`, and the exact verified Sprint 11 prompt
suite is retired in the review commit. No implementation, public API, Coverage,
or deferred-scope change is part of the review transition.

##### Sprint 12 state gates and completion criteria

Sprint 12 remains `next` during planning. It becomes `active` only after the
planning baseline is committed and Task 1 begins. A task is `already_complete`
only when current committed evidence and successful required validation prove
every criterion; no empty commit is created.

Stop after the first prerequisite, implementation, validation, staging, commit,
or review failure. Do not skip, reorder, combine, or partially commit dependent
tasks. A blocked Task 5 leaves Sprint 12 incomplete, keeps the Sprint 11 prompt
suite intact, and leaves Sprint 13 ineligible.

Sprint 12 may transition to `completed` only when Tasks 1–4 are committed or
proven already complete, the complete ADR-0034 graph/parser/production/
provenance/determinism evidence passes, nested/folder/query-language scope
remains deferred, unrelated compatibility and truthful Coverage state are
preserved, the complete repository Definition of Done passes, and Task 5
records a non-blocking review decision. Only then may Sprint 13 XDTO and Service
Model become `next` and the exact Sprint 11 prompt suite be retired in the final
review commit.

Planning is documentation-only. Validate Markdown structure, links, prompt
numbering, manifest order, prerequisite graph, commit-message agreement,
accepted-versus-deferred scope, unchanged `next` state, verified previous-suite
inventory, `git diff --check`, and absence of unrelated changes. Suggested
planning commit message:

```text
Plan Sprint 12 data composition and reports
```

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

Deferred Sprint 3 scope is not implicitly promoted by this schedule. The
accepted direct Query-source-derived `DependsOn` slice belongs to Sprint 8 and
is implemented; broader Query grammar and source families remain deferred.
Deny, inheritance, defaults, profiles, groups, users, and effective
authorization remain deferred beyond the completed Sprint 9 conditional
direct-grant slice. Subsystem hierarchy and transitive membership belong to
Sprint 10; other reference-request families migrate in the sprint that owns
their source contract.

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
8. **High — completed:** emit static BSL Query declarations as stable, provenance-backed `NodeKind::Query` nodes; the accepted query-language parsing, direct Reads, and Sprint 8 Query-origin `DependsOn` slices are complete, while broader query-language support remains deferred.
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
20. **High — completed:** implement and transition the `Reads` slices defined by `docs/adr/0021-reads-semantics.md` and ADR-0030; parsing, multiline BSL decoding, public QuerySource requests, typed positive and negative classification, exact resolution, validation, emission for four direct persistent source families, provenance, raw fixtures, deterministic parser/full-builder tests, and `semantic_edge.reads` Coverage evidence are complete.
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
    negative validator evidence. All ten EdgeKind rules now match their
    accepted ownership or first-slice contracts. Coverage status and counts are
    unchanged.
26. **High — completed:** implement and transition the bounded Sprint 7
    `Opens` slice defined by
    `docs/adr/0029-form-command-navigation-semantics.md`; repository-owned EDT
    evidence now proves exact Common and subordinate Form resolution,
    provenance-backed production emission, typed negative and partial
    outcomes, Query, Diff, Impact, Validation, reports, determinism, and
    compatibility with complete and incremental Semantic Index consumers.

The EDT Coverage Registry currently contains 0 Critical gaps, 0 High gaps, and
0 Medium gaps. Combined with the Graph Domain registry, Semantic Coverage
contains 0 Critical gaps, 0 High gaps, and 0 Medium gaps. Sprint 3 Semantic
Coverage Integration Review is complete with no blocking findings.

Completion does not broaden the accepted first-slice contracts. Deferred work
remains: broader query-language grammar and source forms beyond the four direct
persistent namespaces;
deny, inheritance, and effective authorization; Subsystem hierarchy, nested
Subsystem discovery, and transitive membership; and reference-request migration
for BSL calls, Writes targets, protected resources, Subsystem content, and
extension targets. Query sources have completed public request migration for
the accepted direct-source boundary.

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
