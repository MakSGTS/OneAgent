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
| v0.3 — 1C Knowledge Model | Broader metadata semantics and Designer XML ingestion | Sprints 6–14 | completed |
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
The v0.3 boundary is closed with a `pass` decision in the
[v0.3 release review](reviews/v0.3-release-review.md). Sprint 15 Runtime Service
Container and Sprint 16 HTTP API and Health are completed with `pass`
decisions. Sprint 17 Workspace Service, Sprint 18 Graph Query API, Sprint 19
File Watching, Sprint 20 Persistent Cache, and Sprint 21 CLI Client are
completed with `pass` decisions; the v0.4 release integration review is the
unique next gate.

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
| Task prompt template update completed — Source Adapter Ingestion | Sprint 14 | Multi-artifact source discovery and parsing, partial and malformed input, canonical identity equivalence across adapters, and end-to-end adapter conformance implemented by the [Source Adapter profile](codex/profiles/source-adapter-implementation.md), [Source Adapter workflow](codex/workflows/source-adapter.md), and [Source Adapter template](codex/templates/source-adapter-task.md). | Sprint 14 | completed |
| Task prompt template update completed — Runtime Services and APIs | Sprint 15 | Long-running service lifecycle, ownership, concurrency, cancellation, shutdown, health, transport compatibility, observability, and client/server integration evidence implemented by the [Runtime Service profile](codex/profiles/runtime-service-implementation.md), [Runtime Service workflow](codex/workflows/runtime-service.md), and [Runtime Service template](codex/templates/runtime-service-task.md). | Sprints 15–19 and 21; baseline for Sprints 28 and 32 | completed |
| Task prompt template update completed — Persistent State | Sprint 20 | Persisted schema ownership, deterministic invalidation, compatibility, corruption handling, migration, recovery, and clean-rebuild equivalence implemented by the [Persistent State profile](codex/profiles/persistent-state-implementation.md), [Persistent State workflow](codex/workflows/persistent-state.md), and [Persistent State template](codex/templates/persistent-state-task.md). | Sprint 20 | completed |
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

The Source Adapter audit at committed baseline
`80c25a69e50a572220d4c1380ee15934792b68b8` found that the existing parser
contract intentionally covers one artifact family and does not own project
detection, multi-artifact assembly, explicit workspace completeness, or
cross-adapter conformance. The new reusable modules add only those missing
contracts while preserving the existing parser, graph, and semantic-authority
boundaries. Sprint 14 remains `next`; its kickoff must still recheck real
Designer XML source evidence and accepted architecture before decomposition.

The real-source prerequisite was satisfied on 2026-08-21 by the registered
[OneAgent Designer XML source corpus](architecture/designer-xml-source-corpus.md).
Designer 8.3.27.2214 produced the hierarchical export with
`ConfigDumpInfo.xml`; its configuration UUID, name, version, and representative
normalized module content match `OneAgent_EDTproject`. The ignored full corpus
has tracked provenance, tree and representative hashes, a documented
EDT-to-Designer loss boundary, and successful clean-infobase XML and CF
round-trip validation. Sprint 14 is therefore eligible for safe task
decomposition. Kickoff must still define the exact source detector,
completeness policy, accepted semantic mapping, and cross-adapter conformance
projection instead of assuming field-for-field losslessness.

The current architecture, implementation, graph-model, graph-emission, parser,
source-adapter, investigation, Semantic Index, review, sprint-planning, and
sequential-execution contracts are forecast to cover Sprints 4–14 without
another domain task-template family. Later sprints are covered by the first
applicable stage above plus its planned reuse; every sprint still performs a
focused readiness check at kickoff.

The Runtime Services and APIs audit at committed v0.3 release head
`a90cd869fe6c75062298ab46368b63c32afb37b7` found that the generic
implementation contract does not require structured ownership of long-lived
tasks and resources, startup rollback, cancellation propagation, deterministic
shutdown, lifecycle-derived readiness, transport compatibility, or public
client/server evidence. The Runtime Service modules add only those reusable
execution and evidence requirements. They do not select an executor, service
registry, cancellation primitive, timeout, health schema, transport, endpoint,
or client protocol; those remain architecture and task decisions. Sprint 15
kickoff must still investigate the live Runtime baseline and accept its bounded
first service-container contract before implementation.

The Persistent State audit at committed Sprint 19 review head
`4b8d7efc664d7f5942287b810d822afa320669d3` found that the generic
implementation and Runtime Service contracts do not require persisted schema
ownership, validity inputs, deterministic invalidation, version compatibility,
migration containment, corruption classification, replacement safety, recovery,
or clean-rebuild equivalence. The Persistent State modules add only those
reusable execution and evidence requirements. They do not select a format,
schema, cache key, fingerprint, checksum, storage path, replacement primitive,
migration policy, eviction policy, or Runtime lifecycle; those remain
architecture and task decisions. Sprint 20 kickoff must still investigate the
live semantic snapshot and filesystem boundaries and accept a bounded first
persistent-cache contract before implementation.

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
Sprint 13 is completed under
[ADR-0035](adr/0035-xdto-service-semantics.md) with a `pass` decision in the
[Sprint 13 integration review](reviews/sprint-13-xdto-service-model.md). Sprint
14 is completed under [ADR-0036](adr/0036-designer-xml-adapter.md) with a
`pass` decision in the
[Sprint 14 integration review](reviews/sprint-14-designer-xml-adapter.md). The
[v0.3 release review](reviews/v0.3-release-review.md) also records `pass`, so
Sprint 15 Runtime Service Container entered execution as the unique eligible
target. It is now completed under
[ADR-0037](adr/0037-runtime-service-container.md) with a `pass` decision in the
[Sprint 15 integration review](reviews/sprint-15-runtime-service-container.md).

#### Post-Sprint 13 correction — multiple Web Service XDTO packages

The registered Retail EDT corpus adds corrective source evidence after the
completed Sprint 13 review. Its 18 Web Services contain direct `xdtoPackages`
cardinalities zero, one, two, and four. `EquipmentService` declares four
repository packages, `MobileService` mixes one repository package with one
external namespace, and `SiteExchange2` proves that repository XDTO type
resolution cannot be scoped to the service's package declaration list. The
corpus-separated evidence and representative source hashes are recorded in the
[Web Service XDTO package investigation](architecture/web-service-xdto-packages-source-investigation.md).

The amended [ADR-0035](adr/0035-xdto-service-semantics.md) accepts direct
cardinality zero-or-more. Declarations form a deterministic typed collection
sorted by variant and exact value and deduplicated within one service. Every
unique repository declaration creates one unscoped exact-name `XdtoPackage`
request; external declarations remain payload-only. Package requests may
resolve or terminate missing, ambiguous, or incompatible. Invalid-owner is not
applicable to top-level package requests and remains limited to owner-scoped
`XdtoType` and `Callable` requests. Exact XDTO type resolution continues over
the complete repository namespace index, independent of declaration or source
order. Existing node kinds, payload fields, request categories, identity,
References/Triggers endpoint matrices, Function handler targets, and Coverage
status remain unchanged.

This correction is not a new sprint and does not reopen the historical Sprint
13 `pass`. At the committed correction baseline, its completion made Sprint 14
the unique eligible `next` sprint:

```text
Committed multiple-package architecture correction
    -> completed bounded EDT parser/emission implementation
    -> passed focused and complete workspace validation
    -> Sprint 14 planning eligibility
```

The completed implementation satisfies these requirements:

- replace singular `EdtWebServiceDescriptor` package storage/access with a
  deterministic collection and parse every direct valid declaration;
- preserve fatal descriptor/build failure when any declaration is malformed,
  unsupported, or wrongly nested;
- project the complete canonical collection into the existing
  `WebServiceMetadataPayload`;
- collect one canonical package request per unique repository declaration,
  preserve external declarations without local resolution, and project each
  package request independently to its terminal diagnostic or precise
  `References` edge;
- preserve complete-snapshot XDTO namespace/type candidate collection and the
  deterministic ambiguous resolution committed in
  `Fix ambiguous XDTO namespace resolution`;
- add generated parser and production evidence for zero, one, multiple
  repository, mixed repository/external, equivalent duplicate, reordered,
  malformed-member, and repeated-build cases;
- cover resolved, missing, ambiguous, and incompatible package requests, while
  retaining invalid-owner tests only for owner-scoped XDTO type and callable
  requests;
- derive the smallest tracked provenance-backed reduction from the documented
  Retail `EquipmentService`, `MobileService`, and, where required for the
  complete-snapshot resolution contract, `SiteExchange2` artifacts, recording
  exact source paths, source hashes, reduction treatment, and reduced hashes;
- prove exact payloads, requests, candidate order, References, provenance,
  diagnostics, statistics, Query, Diff, Validation, complete/incremental index
  equivalence, source reordering, and repeated-build equality;
- leave Coverage statuses and aggregate counts unchanged unless executable
  registry evidence independently proves a necessary transition;
- when the ignored Retail corpus is installed, run a local whole-project
  builder probe and prove that repeated `xdtoPackages` no longer produces
  `DuplicateField`. A later unrelated corpus failure is reported separately;
  neither the corpus nor that optional probe becomes a CI prerequisite.

Focused implementation validation is:

```bash
cargo test -p oneagent-edt --lib service_descriptor::tests
cargo test -p oneagent-edt --lib xdto_service_emission::tests
cargo test -p oneagent-edt --test xdto_services
cargo test -p oneagent-edt --test coverage
cargo test -p oneagent-edt --test semantic_index
cargo test -p oneagent-graph --test validation
cargo test -p oneagent-graph --test reference_request_build
```

Run the complete workspace implementation gate afterward. The suggested
implementation commit message is:

```text
Support multiple Web Service XDTO packages
```

The tracked `multiple_xdto_packages_project` reduction preserves exact Retail
source and reduced hashes and proves the four-package, mixed-package, and
global namespace/type cases without an ignored-corpus CI dependency. Generated
tests cover duplicates, reordering, malformed members, independent package
terminals, exact projections, and repeated builds. Coverage statuses and
aggregate counts remain unchanged. The optional whole-Retail probe passes
repeated `xdtoPackages` and reaches the later unrelated
`RoleRights(DuplicateField("restrictionByCondition.condition"))` boundary.

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
| Sprint 13 — XDTO and Service Model | Expand XDTO, HTTP service, and Web service semantics beyond top-level metadata-node coverage. | completed |
| Sprint 14 — Designer XML Adapter | Ingest supported Designer XML configuration dumps through a source adapter without changing canonical semantic identities. | completed |

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

#### Sprint 13 XDTO and Service Model execution plan

Sprint 13 was planned from committed Sprint 12 review head
`cf59854baebc6fe88add0de5a0e5b6858b755a19`. Sprint 12 is `completed`, and
Sprint 13 subsequently completed its ordered implementation, recovery, and
independent integration review. The
[Sprint 13 integration review](reviews/sprint-13-xdto-service-model.md) records
`pass` against committed recovery head
`5af338cd679a950c3ed262d1b777892186c92e22`. Sprint 14 Designer XML Adapter is
the unique `next` target after the post-Sprint 13 multiple-package correction
and its full validation gate complete; the v0.3 release integration review
remains ineligible until Sprint 14 completes.

Repository-owned
[source investigation](architecture/xdto-service-source-investigation.md)
proves 20 descriptor/schema XDTO Package pairs, 12,666 uniquely named direct
Value/Object types, two HTTP Services with 35 URL Templates and 35 Methods, and
eight Web Services with 119 Operations and 360 Parameters. All 154 service
handler declarations resolve uniquely to existing owned BSL callables: all 35
HTTP and 119 Web handlers are Functions, and zero are Procedures. The accepted
[ADR-0035](adr/0035-xdto-service-semantics.md) limits the first slice to direct
types, service declaration structure, internal package/type references, and
declarative handler dispatch. XDTO properties, imports, restrictions, external
namespace nodes, transport/runtime behavior, and Designer XML remain deferred.

The current graph implementation, parser implementation, graph emission,
review, sprint-planning, and sequential-execution framework contracts express
the required identity, payload, parsing, resolution, validation, provenance,
Coverage, and reporting boundaries. The Task prompt template readiness forecast
requires no framework update until Sprint 14, and the live audit found no
concrete Sprint 13 framework gap. No Codex Framework change or post-sprint
framework audit is justified.

##### Sprint 13 objective

Preserve repository-proven direct XDTO Value/Object types, HTTP URL Templates
and Methods, and Web Service Operations and Parameters as deterministic typed
graph entities with exact immediate ownership; enrich the existing service and
package metadata payloads; resolve repository-owned XDTO package/type and
handler declarations through public requests; and emit precise References and
Triggers without inventing external schema nodes or runtime transport behavior.

##### Included scope

- closed typed HTTP Service, Web Service, and XDTO Package metadata payloads;
- additive XDTO Type, HTTP URL Template/Method, and Web Operation/Parameter
  node kinds and compatible typed payloads;
- UUID child identities plus collision-safe owner/name XDTO Type identity;
- precise immediate Contains ownership;
- exact XDTO descriptor/`Package.xdto` joins and deterministic direct-type
  parsing;
- exact HTTP/Web descriptor structure, optional/required value, duplicate,
  malformed, and unsupported-value behavior;
- public XDTO package/type and callable request lifecycle with deterministic
  provenance, diagnostics, and statistics;
- internal References plus HTTP and Web declarative Triggers to existing
  Function nodes, with external
  namespaces preserved but not materialized;
- generated positive/negative/reordered/repeated production evidence;
- one tracked provenance-backed reduced production fixture;
- generic Query, Diff, reports, Validation, Impact policy, complete and
  incremental indexes, Coverage, aggregate counts, and current-state docs;
- integration review and conditional Sprint 12 prompt-suite retirement.

##### Excluded scope

- XDTO imports, properties, enum values, patterns, inline definitions, bases,
  restrictions, bounds, inheritance, ordering, property/type dependency edges,
  and external platform schema/type nodes;
- HTTP route grammar or matching, inferred verbs, URL parameter entities,
  sessions, authentication, publication, transport, request/response schemas,
  and runtime invocation;
- WSDL/descriptors, SOAP, data-lock behavior, runtime Web Service execution,
  and external type resolution;
- BSL body behavior beyond existing module/symbol extraction and exact declared
  handler binding;
- Designer XML, cross-adapter conformance, partial workspaces, persistence,
  Runtime/API/CLI, MCP/LSP/IDE, serialization, benchmarks, or performance claims;
- unrelated graph/EDT capabilities, dependencies, refactors, or speculative
  Coverage transitions.

##### Sprint 13 prerequisite gate

Task 01 may begin only from one committed Sprint 13 planning baseline containing
the source investigation, accepted ADR-0035, this Roadmap plan, Semantic Model
planning synchronization, and the complete prompt suite under
`docs/codex/prompts/sprint-13-xdto-service-model/`. Every dependent task
requires the preceding task's committed outcome. Stored prompt text never
authorizes staging or committing; authorization comes only from the current
execution instruction.

The immediately preceding prompt suite is exactly
`docs/codex/prompts/sprint-12-skd-report-model/`, containing these six tracked
files:

- `00-sprint-12-execution-loop.md`;
- `01-implement-data-composition-graph-model.md`;
- `02-parse-report-data-composition-schemas.md`;
- `03-emit-report-data-composition-semantics.md`;
- `04-complete-sprint-12-production-evidence.md`;
- `05-sprint-12-integration-review.md`.

It remains untouched during planning and implementation. Only the final Sprint
13 review may retire those exact tracked files after a non-blocking decision
and successful complete validation.

##### Ordered task manifest

| Order | Task | Profile / template | Owned outcome | Required committed prerequisite | Suggested commit message |
|---:|---|---|---|---|---|
| 1 | Implement the XDTO and service graph model. | Graph implementation / graph model | Public node/payload/request contracts, identities, precise Contains/References/Triggers validation, exhaustive consumers, indexes, and graph Coverage evidence. | Accepted Sprint 13 planning baseline. | `Implement Sprint 13 XDTO and service graph model` |
| 2 | Parse XDTO Package schemas. | Parser implementation / parser | Deterministic descriptor/artifact join and typed direct XDTO Value/Object model with fatal and deferred outcomes, without graph emission. | Task 1. | `Parse Sprint 13 XDTO package schemas` |
| 3 | Parse HTTP and Web Service descriptors. | Parser implementation / parser | Deterministic typed HTTP/Web service structure, package/type/callable declarations, and malformed/unsupported outcomes, without graph emission. | Tasks 1–2. | `Parse Sprint 13 HTTP and Web service descriptors` |
| 4 | Emit XDTO and service semantics. | Graph implementation / graph emission | Production metadata enrichment, child nodes/ownership, public requests, resolution, References/Triggers, provenance, diagnostics, statistics, and generated determinism evidence. | Tasks 1–3. | `Emit Sprint 13 XDTO and service semantics` |
| 5 | Complete Sprint 13 production evidence. | Graph implementation / graph emission | Provenance fixture, generic consumers, complete/incremental indexes, EDT Coverage, aggregate counts, and current-state documentation. | Tasks 1–4. | `Complete Sprint 13 production evidence` |
| 6 | Review the integrated Sprint 13 baseline. | Review / review | Findings, full validation evidence, sprint decision, Sprint 12 suite retirement, Sprint 14 hand-off, and v0.3 release-review eligibility. | Task 5 and all implementation validation. | `Complete Sprint 13 XDTO and service model review` |

Dependency graph:

```text
Committed Sprint 13 planning baseline
    -> Task 1 graph model
    -> Task 2 XDTO parser
    -> Task 3 service parsers
    -> Task 4 production emission and resolution
    -> Task 5 production, Coverage, and documentation evidence
    -> Task 6 integration review and conditional Sprint 12 suite retirement
    -> Sprint 14 framework readiness and planning eligibility
```

##### Task 1 — XDTO and service graph model

**Included:** add the five ADR-0035 node kinds; closed XDTO Type, HTTP URL
Template/Method, and Web Operation/Parameter payloads; compatible HTTP/Web/XDTO
metadata payloads; stable codes and collision-safe XDTO identity; public XDTO
package/type request categories; exact Contains, additive References, and
additive Triggers endpoint rules; exhaustive public consumer, Query, Diff,
report, Validation, complete/incremental index, Impact-policy, and graph Coverage
evidence.

**Excluded:** EDT parsing, production insertion/resolution, diagnostics or
statistics, fixtures, and EDT Coverage transitions.

**Acceptance evidence:** existing codes/identities remain unchanged; wrong
payloads are rejected; content-only changes preserve IDs; delimiter-containing
identity inputs cannot collide; only the five new ownership pairs and exact
reference/dispatch matrices validate; every child requires one owner; public
requests preserve deterministic identity/lifecycle; generic consumers and both
index modes are deterministic; Contains/Triggers remain non-propagating.

Focused validation:

```bash
cargo test -p oneagent-metadata
cargo test -p oneagent-graph
```

Run the complete workspace implementation gate afterward.

##### Task 2 — XDTO Package parser

**Included:** add one focused parser joining an existing XDTO Package descriptor
to exactly one `Package.xdto`; validate exact roots/namespaces and matching
descriptor/schema namespace; parse direct named Value/Object types; canonicalize
by exact name; retain imports and nested constructs as typed deferred evidence;
return typed deterministic errors without graph emission.

**Excluded:** HTTP/Web parsing, graph insertion, XDTO properties/import
resolution, diagnostics/statistics, fixtures, and Coverage changes.

**Acceptance evidence:** all 20 live pairs join and their 12,666 direct names are
unique; small and large real shapes parse without source-order dependence;
missing/extra/ambiguous/unreadable/malformed artifacts, wrong roots/namespaces,
namespace mismatch, missing/empty/duplicate direct names, and repeated reads are
typed and deterministic; nested/import content creates no speculative entity.

Focused validation:

```bash
cargo test -p oneagent-edt --lib xdto_package::tests
cargo test -p oneagent-edt --lib metadata_object::tests
```

Run the complete workspace implementation gate afterward.

##### Task 3 — HTTP and Web Service parsers

**Included:** add focused parsers for UUID-backed HTTP URL Templates/Methods and
Web Operations/Parameters; preserve accepted service metadata content, route/
method and XDTO type payload input, package/type/callable declarations, optional
Boolean/direction values, exact hierarchy, and canonical ordering.

**Excluded:** graph emission/resolution, XDTO schema parsing changes,
diagnostics/statistics, fixtures, and Coverage changes.

**Acceptance evidence:** all live 35/35 HTTP and 119/360 Web children parse;
all 154 live handler declarations are retained; two internal package refs, five
external package namespaces, one missing package declaration, 478 external type
occurrences, and one internal type occurrence are classified exactly; missing,
duplicate, malformed, invalid Boolean/reference/direction, reordered, and
repeated inputs have deterministic typed outcomes.

Focused validation:

```bash
cargo test -p oneagent-edt --lib service_descriptor::tests
cargo test -p oneagent-edt --lib metadata_object::tests
```

Run the complete workspace implementation gate afterward.

##### Task 4 — XDTO and service semantic emission

**Included:** integrate committed parsers with existing XDTO/HTTP/Web discovery;
enrich existing metadata payloads; retain existing modules/symbols; insert
direct XDTO Type and service child nodes plus exact ownership; collect and
resolve public package/type/callable requests after symbol insertion; emit exact
References and Triggers; attach deterministic provenance; project terminal
diagnostics/statistics; add generated positive/failure/external/reordering/
repeated-build tests.

**Excluded:** new graph/parser semantics, final tracked fixture, broad consumer/
index/Coverage evidence, current-state documentation completion, and deferred
XDTO or runtime behavior.

**Acceptance evidence:** internal package/type and all HTTP/Web Function
handlers resolve; external namespaces emit no requests,
placeholders, edges, or false missing diagnostics; missing/ambiguous/
incompatible/wrong-owner internal declarations have exact request outcomes and
diagnostics; fatal source errors yield no successful partial build; accepted
siblings survive deferred inputs; Query and Validation observe exact facts;
reordered/repeated builds are equal; unrelated behavior passes.

Focused validation:

```bash
cargo test -p oneagent-edt --test xdto_services
cargo test -p oneagent-graph --test validation
cargo test -p oneagent-graph --test reference_request_build
```

Run the complete workspace implementation gate afterward.

##### Task 5 — Complete production evidence

**Included:** add a tracked provenance-documented reduced fixture covering
small/large XDTO packages, HTTP GET-implicit/POST-explicit methods, internal and
external Web package/type declarations, operations/parameters/directions, and
handler dispatch; prove generic Query, Diff, reports, Validation, Impact policy,
reordered/repeated builds, complete index, and incremental clean-rebuild
transitions; transition only justified graph/EDT Coverage evidence; recompute
aggregate counts; synchronize Semantic Model and Roadmap current-state text
without completing the sprint.

**Excluded:** new semantic/parser/resolution behavior, deferred XDTO properties
or runtime transport, Sprint 14 planning/framework updates, previous-suite
retirement, and unrelated Coverage changes.

**Acceptance evidence:** every applicable ADR-0035 completion criterion is
executable; fixture README records exact live paths, source hashes, reduction,
and reduced hashes; add/remove/modify/ownership/request/reference/dispatch/
external/deferred transitions equal clean rebuilds; Coverage and aggregate
counts derive from live registries; full workspace validation succeeds.

**Current state after Task 5:** the tracked
`adapters/edt/tests/fixtures/sprint13_xdto_services_project/` reduction proves
small/mixed/large direct XDTO shapes, explicit/absent HTTP method values,
internal/external/absent Web package forms, internal/external types,
absent/`Out`/`InOut` directions, and exact owned Function dispatch. Generic
Query, Diff, reports, Validation, Impact policy, complete indexes, and
incremental clean-rebuild transitions cover all five new node kinds, immediate
ownership, payload/target/request/reference/dispatch changes, and deferred
property stability. The executable Graph Domain registry has 96 capabilities:
92 `Supported` and 4 `NotApplicable`; EDT has 120 capabilities: 115 `Supported`
and 5 `NotApplicable`. Both registries have no gaps. Sprint 13 remained `active`
until Task 6 recorded the independent integration-review decision.

The
[Sprint 13 integration review](reviews/sprint-13-xdto-service-model.md)
subsequently records `pass` against committed recovery head
`5af338cd679a950c3ed262d1b777892186c92e22`. Sprint 13 is `completed`, Sprint 14
Designer XML Adapter is `next`, and the v0.3 release integration review remains
gated on Sprint 14 completion and review.

Focused validation:

```bash
cargo test -p oneagent-metadata
cargo test -p oneagent-graph
cargo test -p oneagent-edt --test xdto_services
cargo test -p oneagent-edt --test coverage
cargo test -p oneagent-edt --test semantic_index
```

Run the complete workspace implementation gate afterward.

##### Task 6 — Sprint 13 integration review

Review the exact planning-through-Task-5 commit range against ADR-0035, source
investigation, prompt suite, and live repository. Verify public model, identity,
payload, parser joins and outcomes, production emission, ownership, public
requests, external/internal policy, References/Triggers, provenance,
diagnostics/statistics, determinism, generic consumers, complete/incremental
indexes, Coverage, documentation, compatibility, and deferred scope. Do not
silently fix findings.

Create `docs/reviews/sprint-13-xdto-service-model.md` and transition Sprint 13
to `completed` with Sprint 14 as `next` only after `pass` or `pass with
non-blocking follow-ups` and successful focused/full validation. The transition
also makes the v0.3 release integration review eligible after Sprint 14, not
before. A blocked decision creates no completion transition or review commit.

After a non-blocking decision and successful complete validation, atomically
retire every tracked prompt file under the verified immediately preceding suite
`docs/codex/prompts/sprint-12-skd-report-model/` in the same review commit. Any
inventory mismatch, endangered untracked file, or retained link dependency
blocks retirement and the final commit.

Focused review additions:

```bash
cargo test -p oneagent-metadata
cargo test -p oneagent-graph
cargo test -p oneagent-edt --lib xdto_package::tests
cargo test -p oneagent-edt --lib service_descriptor::tests
cargo test -p oneagent-edt --test xdto_services
cargo test -p oneagent-edt --test coverage
cargo test -p oneagent-edt --test semantic_index
```

Run the complete workspace validation and record exact command results.

##### Sprint 13 state gates and completion criteria

Sprint 13 remains `next` during planning. It becomes `active` only after the
planning baseline is committed and Task 1 begins. A task is `already_complete`
only when current committed evidence and successful required validation prove
every criterion; no empty commit is created.

Stop after the first prerequisite, implementation, validation, staging, commit,
or review failure. Do not skip, reorder, combine, or partially commit dependent
tasks. A blocked Task 6 leaves Sprint 13 incomplete, keeps the Sprint 12 prompt
suite intact, and leaves Sprint 14 ineligible.

Sprint 13 may transition to `completed` only when Tasks 1–5 are committed or
proven already complete, complete ADR-0035 graph/parser/production/provenance/
determinism evidence passes, external and deferred scope remains explicit,
unrelated compatibility and truthful Coverage state are preserved, the full
repository Definition of Done passes, and Task 6 records a non-blocking review
decision. Only then may Sprint 14 Designer XML Adapter become `next` and the
exact Sprint 12 prompt suite be retired in the final review commit.

Planning is documentation-only. Validate Markdown structure, links, prompt
numbering, manifest order, prerequisite graph, commit-message agreement,
accepted-versus-deferred scope, unchanged `next` state, verified previous-suite
inventory, `git diff --check`, and absence of unrelated changes. Suggested
planning commit message:

```text
Plan Sprint 13 XDTO and service model
```

#### Sprint 14 Designer XML Adapter execution plan

Sprint 14 was planned from committed readiness head
`5b8c57b44247ffed5b26a52877b3b333bbf64703` and completed through the
[Sprint 14 integration review](reviews/sprint-14-designer-xml-adapter.md), which
records `pass` against committed Task 7 head
`19d56818a1345b4cced43db7275165ff24ce0748`. The subsequent
[v0.3 release review](reviews/v0.3-release-review.md) records `pass`, completes
the v0.3 boundary, and makes Sprint 15 Runtime Service Container the unique
`next` planning target.

**Completed current state:** Tasks 1–8 are complete. The dedicated adapter
detects hierarchical Designer XML 2.20,
loads explicit complete or partial scopes, parses the accepted configuration,
20 top-level metadata families and generic Object/Manager/Common modules, and
emits canonical configuration, metadata, Module, Procedure, Function, and
immediate `Contains` facts with exact provenance. The tracked official-tool
Designer and provenance-backed EDT reduction proves a non-empty equal canonical
projection, an exact one-node controlled synonym difference, negative and
partial outcomes, public consumers, complete indexes, and incremental
clean-rebuild equivalence. Designer-specific Coverage reports 58 capabilities:
55 `supported`, one `unsupported`, two `not_applicable`, and the single
Calculation Register evidence gap. The review introduced no production, public
API, Coverage, or deferred-scope change and retired the exact Sprint 13 prompt
suite. The Sprint 14 suite and reusable bootstrap remain present.

The ignored paired `OneAgent_DesignerXML/` and `OneAgent_EDTproject/` corpora,
the tracked [Designer XML corpus registration](architecture/designer-xml-source-corpus.md),
accepted graph contracts, current EDT builder, and filesystem/workspace tests
provide repository-owned evidence. The corpus proves real hierarchical
`ConfigDumpInfo.xml` and `Configuration.xml` markers, matching configuration
identity, matching representative normalized BSL modules, distinct layouts,
and a documented four-binding bridge loss boundary. It does not pre-accept a
detector, completeness policy, field mapping, or whole-graph equivalence.

The committed Source Adapter profile, workflow, and task template cover
multi-artifact discovery, assembly, parsing, completeness, canonical mapping,
conformance, determinism, and production evidence. Existing investigation,
architecture, review, sprint-planning, and sequential-execution contracts cover
the other task families. No concrete reusable framework gap remains.

##### Sprint 14 objective

Ingest the evidence-backed first slice of complete or explicitly partial
hierarchical Designer XML dumps through a dedicated source adapter, preserving
canonical configuration, supported top-level metadata, module, and BSL
declaration identities and semantics across EDT and Designer sources without
claiming field-for-field or whole-graph equivalence.

##### Included scope

- exact investigation of markers, hierarchy, artifact roles, joins, serialized
  values, module layouts, failures, and paired EDT compatibility;
- an accepted ADR for detection, completeness, identity, mapping, provenance,
  conformance projection, first slice, and deferred scope;
- deterministic filesystem detection, conflict handling, and project boundaries;
- a dedicated `oneagent-designer-xml` adapter and configuration loader;
- accepted top-level metadata enumeration and parsing;
- accepted module assembly and BSL source loading;
- canonical graph contribution for configuration, metadata, modules,
  Procedures, Functions, and immediate `Contains` facts;
- typed complete/partial, missing, duplicate, malformed, unsupported,
  ambiguous, incompatible, unreadable, overlap, and marker-conflict outcomes;
- provenance-backed paired fixtures and a non-empty canonical conformance
  projection with one controlled semantic change;
- deterministic production, reordering, repeated-build, Query, Diff,
  Validation, report, complete-index, and incremental-index evidence;
- documentation, truthful adapter-specific coverage evidence, integration
  review, and conditional Sprint 13 prompt-suite retirement.

##### Excluded scope

- field-for-field EDT/Designer equality and affected form-event payloads;
- metadata members, role rights, Subsystem hierarchy/content, Event
  Subscriptions, report DCS, XDTO/service children, reference requests, and
  semantic relations beyond accepted ownership and BSL declarations unless
  Task 2 proves a bounded prerequisite;
- conversion into an EDT tree, runtime 1C tooling, or ignored-corpus CI use;
- extensions, flat dumps, binary interpretation, parent-configuration
  semantics, unknown fallback nodes, persistence, Runtime/API/CLI, MCP/LSP/IDE,
  packaging, benchmarks, and performance claims;
- graph/public semantic model expansion, unrelated EDT changes, dependencies,
  refactors, or speculative Coverage transitions.

##### Sprint 14 prerequisite and retirement gate

Task 01 requires one committed Sprint 14 planning baseline containing this plan
and the complete suite under
`docs/codex/prompts/sprint-14-designer-xml-adapter/`. Every dependent task
requires the preceding committed outcome. Stored prompts do not authorize
commits; authorization comes only from the launching user instruction.

The immediately preceding suite is exactly
`docs/codex/prompts/sprint-13-xdto-service-model/`, with these seven tracked
files: `00-sprint-13-execution-loop.md`,
`01-implement-xdto-service-graph-model.md`, `02-parse-xdto-packages.md`,
`03-parse-http-web-services.md`, `04-emit-xdto-service-semantics.md`,
`05-complete-sprint-13-production-evidence.md`, and
`06-sprint-13-integration-review.md`. It remains untouched through Task 7.
Only Task 8 may retire this exact inventory after a non-blocking review and
successful complete validation.

##### Ordered task manifest

| Order | Task | Profile / template | Owned outcome | Required committed prerequisite | Suggested commit message |
|---:|---|---|---|---|---|
| 1 | Investigate Designer XML source contracts. | Investigation / investigation | Detector, artifact, completeness, identity, module, fixture, failure, consumer, and conformance evidence with explicit unknowns. | Accepted Sprint 14 planning baseline. | `Investigate Sprint 14 Designer XML source contracts` |
| 2 | Define the Designer XML adapter contract. | Architecture / architecture | Accepted detection, assembly, failure, canonical mapping, provenance, conformance, first-slice, deferred-scope, and completion contract. | Task 1. | `Define Sprint 14 Designer XML adapter contract` |
| 3 | Implement Designer XML discovery and configuration loading. | Source adapter / source adapter | Workspace detection, project boundaries, explicit scope, configuration assembly/loading, typed failures, and focused production tests. | Task 2. | `Implement Sprint 14 Designer XML discovery` |
| 4 | Parse Designer XML metadata objects. | Source adapter / source adapter | Deterministic accepted top-level artifact enumeration and descriptor mapping without graph emission. | Task 3. | `Parse Sprint 14 Designer XML metadata` |
| 5 | Parse Designer XML modules. | Source adapter / source adapter | Deterministic accepted module assembly and BSL source observations without graph emission. | Task 4. | `Parse Sprint 14 Designer XML modules` |
| 6 | Emit Designer XML semantics. | Source adapter / source adapter | Canonical graph contribution for accepted configuration, metadata, module, Procedure, Function, and ownership facts. | Task 5. | `Emit Sprint 14 Designer XML semantics` |
| 7 | Complete Sprint 14 conformance evidence. | Source adapter / source adapter | Paired fixtures, canonical projection, negative/determinism/consumer/index evidence, truthful coverage, and current-state docs. | Task 6. | `Complete Sprint 14 conformance evidence` |
| 8 | Review the integrated Sprint 14 baseline. | Review / review | Findings, validation, sprint decision, Sprint 13 suite retirement, v0.3 release-review eligibility, and Sprint 15 hand-off. | Task 7 and all implementation validation. | `Complete Sprint 14 Designer XML adapter review` |

```text
Committed Sprint 14 planning baseline
    -> Task 1 source investigation
    -> Task 2 accepted adapter architecture
    -> Task 3 discovery and configuration loading
    -> Task 4 metadata parsing
    -> Task 5 module parsing
    -> Task 6 semantic contribution
    -> Task 7 paired conformance and current-state evidence
    -> Task 8 integration review and conditional Sprint 13 suite retirement
    -> v0.3 release review eligibility and Sprint 15 planning
```

##### Task contracts

Task 1 creates only
`docs/architecture/designer-xml-source-investigation.md`. It records exact
source markers, namespaces, versions, artifact paths and roles, join keys,
module paths, positive/negative shapes, complete/partial evidence, paired
mappings, deliberate differences, the smallest non-empty conformance
projection, consumers, fixture candidates, and unknowns. It selects no
architecture and changes no production behavior.

Task 2 creates `docs/adr/0036-designer-xml-adapter.md` and synchronizes only
planning-level architecture text required by the decision. It defines the exact
detector, conflicts, project boundary, complete/partial caller contract,
assembly, failure scope, canonical mapping, module roles, provenance,
conformance oracle, rejected alternatives, first slice, Coverage completion,
and deferred scope. It implements no production behavior and marks no support
complete.

Task 3 adds the dedicated crate and smallest workspace/filesystem changes for
accepted detection, configuration loading, explicit scope, and typed errors.
Task 4 adds only top-level metadata enumeration and parsing. Task 5 adds only
module assembly and BSL source observations. Task 6 orchestrates these committed
stages into canonical graph facts through existing public graph and BSL APIs.
Each implementation task keeps stages independently testable, uses real-source
fixtures, covers applicable malformed/missing/duplicate/reordered/repeated
cases, preserves EDT behavior, and runs focused plus full workspace validation.

Task 7 adds the smallest provenance-backed paired fixture allowed by corpus
policy. Its canonical projection compares accepted nodes, identities, names,
payloads, ownership, declarations, terminal outcomes, and public consumers,
while excluding only ADR-0036 deliberate differences such as source paths,
producer identifiers, serialization order, encoding, and line endings. It adds
non-empty assertions and a controlled semantic change, then synchronizes
Semantic Model, Roadmap current state, fixture provenance, and truthful
adapter-specific Coverage evidence without completing the sprint.

Task 8 reviews the planning-through-Task-7 range and creates
`docs/reviews/sprint-14-designer-xml-adapter.md` only for `pass` or `pass with
non-blocking follow-ups` after all focused and full validation succeeds. That
decision transitions Sprint 14 to `completed`, makes the v0.3 release review
eligible, and leaves Sprint 15 as the next planning target after the release
gate. It atomically retires the exact Sprint 13 suite in the same review commit;
an inventory mismatch, endangered untracked file, or retained link dependency
blocks retirement and the final commit. The review never silently fixes code.

##### Sprint 14 state gates and completion criteria

Sprint 14 remains `next` during planning and becomes `active` only after the
planning commit and Task 1 start. `already_complete` requires current committed
evidence and successful required validation; no empty commit is created. Stop
after the first prerequisite, implementation, validation, staging, commit, or
review failure. Do not skip, reorder, combine, or partially commit tasks.

Completion requires committed or proven Tasks 1-7, the accepted ADR-0036 first
slice, successful production and conformance evidence, preserved identities and
EDT behavior, explicit deferred scope, truthful Coverage, the complete workspace
gate, and a non-blocking Task 8 decision. A blocked review keeps Sprint 14
incomplete, preserves the Sprint 13 suite, and leaves the v0.3 release review
and Sprint 15 ineligible.

Planning validation covers Markdown links and structure, prompt numbering,
manifest/prerequisite/commit-message agreement, scope, unchanged `next` state,
previous-suite inventory, `git diff --check`, and unrelated-change absence.
Suggested planning commit message:

```text
Plan Sprint 14 Designer XML adapter
```

#### v0.4 — Runtime API

| Sprint | Goal | Status |
|---|---|---|
| Sprint 15 — Runtime Service Container | Establish the long-running runtime composition and service lifecycle. | completed |
| Sprint 16 — HTTP API and Health | Expose the runtime through an HTTP API with health and readiness behavior. | completed |
| Sprint 17 — Workspace Service | Add workspace lifecycle and semantic-build orchestration services. | completed |
| Sprint 18 — Graph Query API | Expose stable graph and semantic query capabilities through the runtime API. | completed |
| Sprint 19 — File Watching | Detect workspace changes and connect them to runtime update orchestration. | completed |
| Sprint 20 — Persistent Cache | Persist validated semantic state with deterministic invalidation. | completed |
| Sprint 21 — CLI Client | Replace the CLI placeholder with a supported client for runtime workspace and graph-query operations. | completed |

The v0.4 release integration review is the unique next gate.

#### Sprint 15 Runtime Service Container execution plan

Sprint 15 is planned from committed readiness head
`bac838be07bbf9b9686e60419397e91e702adec1`. The
[v0.3 release review](reviews/v0.3-release-review.md) records `pass`; Sprint 15
was the unique `next` target at planning time, and the required Runtime Services
and APIs Codex Framework stage is committed. The
[Sprint 15 integration review](reviews/sprint-15-runtime-service-container.md)
now records `pass`, so Sprint 16 is eligible as the unique `next` target.

At the planning baseline, `oneagent-runtime` was a binary-only composition
foundation. ADR-0002 assigned construction to `AppBuilder`, immutable shared
state to `AppState`, and explicit transitions to `Lifecycle`; synchronous
`App::run` printed one banner and immediately shut down. Tasks 3-5 have since
implemented the public library, ordered service container, owned tasks,
per-service cancellation, asynchronous App lifecycle, injected shutdown, and
deterministic public evidence without adding a dependency.

The committed [Runtime Service profile](codex/profiles/runtime-service-implementation.md),
[workflow](codex/workflows/runtime-service.md), and
[template](codex/templates/runtime-service-task.md), together with the existing
investigation, architecture, review, sprint-planning, and sequential-execution
contracts, cover every planned task. No further framework change is planned.
Repository code, locked dependencies, macOS/Windows CI, and deterministic
in-memory probe services provide sufficient data and test oracles; no external
service, socket, fixture, or new dependency is required.

Task 1 evidence is committed, and
[ADR-0037](adr/0037-runtime-service-container.md) accepts the bounded service
identity, ownership, startup, rollback, cancellation, shutdown, failure,
lifecycle, public-library, and deterministic-test contracts. Tasks 3 and 4
implement the container and asynchronous App lifecycle. Task 5 adds the
six-test public `service_container` integration target and synchronizes current
state. Task 6 records a `pass` integration review. Sprint 15 is completed and
Sprint 16 is the unique `next` target.

##### Sprint 15 objective

Establish the first accepted long-running Runtime service container with
explicit service and task ownership, deterministic startup and shutdown,
cancellation and failure propagation, and an asynchronously running composition
root, without pulling later v0.4 services or transports forward.

##### Included scope

- exact investigation of the current Runtime lifecycle, ownership, dependencies,
  public boundary, consumers, failures, and deterministic testability;
- an accepted ADR for service identity/registration, task/resource ownership,
  startup ordering and rollback, cancellation, shutdown, failure propagation,
  lifecycle state, public Runtime boundary, observability, and first slice;
- reusable service-container and cancellation primitives with complete task
  ownership and deterministic terminal handling;
- AppBuilder/App/lifecycle/main integration into a genuinely long-running async
  Runtime with injected test shutdown and production signal ownership;
- public in-memory integration evidence for startup, rollback, service failure,
  requested shutdown, cleanup, ordering, and repeated fresh runs;
- current-state documentation, full cross-platform workspace validation,
  integration review, and conditional Sprint 14 prompt-suite retirement.

##### Excluded scope

- HTTP routes, listener binding, public health/readiness schema, and protocol
  compatibility owned by Sprint 16;
- workspace lifecycle/build orchestration, graph-query APIs, file watching,
  persistence, and supported CLI behavior owned by Sprints 17–21;
- semantic graph changes, source adapters, AI, MCP, LSP, IDE, global mutable
  state, dependency-injection frameworks, detached tasks, new dependencies,
  real-signal tests, arbitrary-sleep acceptance, packaging, benchmarks, and
  performance claims.

##### Sprint 15 prerequisite and retirement gate

Task 1 requires one committed Sprint 15 planning baseline containing this plan
and the complete suite under
`docs/codex/prompts/sprint-15-runtime-service-container/`. Every dependent task
requires the preceding committed outcome. Stored prompts do not authorize
commits; authorization comes only from the launching user instruction.

The immediately preceding suite is exactly
`docs/codex/prompts/sprint-14-designer-xml-adapter/`, with these nine tracked
files: `00-sprint-14-execution-loop.md`,
`01-investigate-designer-xml-source-contracts.md`,
`02-define-designer-xml-adapter-contract.md`,
`03-implement-designer-xml-discovery.md`,
`04-parse-designer-xml-metadata.md`,
`05-parse-designer-xml-modules.md`,
`06-emit-designer-xml-semantics.md`,
`07-complete-sprint-14-conformance-evidence.md`, and
`08-sprint-14-integration-review.md`. It remains untouched through Task 5. Only
Task 6 may retire this exact inventory after a non-blocking review and successful
complete validation.

##### Ordered task manifest

| Order | Task | Profile / template | Owned outcome | Required committed prerequisite | Suggested commit message |
|---:|---|---|---|---|---|
| 1 | Investigate the Runtime service-container boundary. | Investigation / investigation | Current lifecycle, ownership, dependency, consumer, failure, compatibility, and deterministic-test evidence with explicit decision questions. | Accepted Sprint 15 planning baseline. | `Investigate Sprint 15 Runtime service container` |
| 2 | Define the Runtime service-container contract. | Architecture / architecture | Accepted ADR-0037 ownership, lifecycle, concurrency, cancellation, shutdown, failure, public-boundary, observability, first-slice, and deferred-scope contract. | Task 1. | `Define Sprint 15 Runtime service container contract` |
| 3 | Implement Runtime service-container primitives. | Runtime Service / runtime service | Owned service registration/execution, cancellation, startup rollback, failure propagation, shutdown, cleanup, and focused unit evidence. | Task 2. | `Implement Sprint 15 Runtime service container` |
| 4 | Integrate the Runtime application lifecycle. | Runtime Service / runtime service | Async AppBuilder/App/lifecycle/main composition, injected shutdown, production signal ownership, and focused lifecycle evidence. | Task 3. | `Integrate Sprint 15 Runtime application lifecycle` |
| 5 | Complete public production evidence and current-state documentation. | Runtime Service / runtime service | Public deterministic probe-service integration matrix, cleanup/repeated-run evidence, and synchronized current-state docs. | Task 4. | `Complete Sprint 15 Runtime service container evidence` |
| 6 | Review the integrated Sprint 15 baseline. | Review / review | Findings, complete validation, sprint decision, Sprint 14 suite retirement, and Sprint 16 hand-off. | Task 5 and all implementation validation. | `Complete Sprint 15 Runtime service container review` |

```text
Committed Sprint 15 planning baseline
    -> Task 1 Runtime investigation
    -> Task 2 accepted ADR-0037
    -> Task 3 service container
    -> Task 4 asynchronous App lifecycle
    -> Task 5 public integration evidence and current-state docs
    -> Task 6 integration review and conditional Sprint 14 suite retirement
    -> Sprint 16 planning eligibility
```

##### Task contracts

Task 1 creates only
`docs/architecture/runtime-service-container-investigation.md`. It records exact
definitions, transitions, dependencies, consumers, CI constraints, existing and
missing tests, compatibility-sensitive behavior, failure questions, candidate
public boundary, deterministic probe oracle, and decision readiness. It selects
no architecture and changes no production behavior.

Task 2 creates `docs/adr/0037-runtime-service-container.md` and synchronizes only
planning-level architecture text required by the decision. It defines the
composition and public library boundary, service identity and registration,
task/resource ownership, startup and rollback, cancellation, shutdown, service
and join failure propagation, App lifecycle, internal observability, deterministic
testing, first slice, migration, rejected alternatives, and deferred scope. It
implements no production behavior.

Task 3 implements only the accepted reusable container boundary and focused unit
tests. Task 4 composes the committed container into the async App and production
entry point with injected test shutdown and no HTTP behavior. Both tasks use the
locked dependency surface, prohibit detached work and arbitrary-sleep evidence,
preserve ADR-0002 responsibilities, and run focused plus full workspace
validation.

Task 5 exercises the public production boundary with deterministic in-memory
probe services and synchronizes current-state documentation. It proves a pending
Runtime remains active until injected shutdown, every accepted terminal path
cleans up owned work, fresh repeated runs are equal, and tests remain compatible
with macOS and Windows CI. It does not complete the sprint or add later services.

Task 6 reviews the planning-through-Task-5 range and creates
`docs/reviews/sprint-15-runtime-service-container.md` only for `pass` or `pass
with non-blocking follow-ups` after all focused and full validation succeeds.
That decision transitions Sprint 15 to `completed`, makes Sprint 16 HTTP API and
Health the unique `next` target, and atomically retires the exact Sprint 14 suite
in the same review commit. An inventory mismatch, endangered untracked file, or
retained link dependency blocks retirement and the final commit. The review
never silently fixes code.

##### Sprint 15 state gates and completion criteria

Sprint 15 remains `next` during planning and becomes `active` only after the
planning commit and Task 1 start. `already_complete` requires current committed
evidence and successful required validation; no empty commit is created. Stop
after the first prerequisite, implementation, validation, staging, commit, or
review failure. Do not skip, reorder, combine, or partially commit tasks.

Completion requires committed or proven Tasks 1–5, accepted ADR-0037, a
long-running async App, explicit service/task ownership, deterministic startup,
rollback, cancellation, shutdown and failure evidence, no detached work,
cross-platform public integration tests, preserved deferred scope, the complete
workspace gate, and a non-blocking Task 6 decision. A blocked review keeps
Sprint 15 incomplete, preserves the Sprint 14 suite, and leaves Sprint 16
ineligible.

Planning validation covers Markdown links and structure, prompt numbering,
manifest/prerequisite/commit-message agreement, accepted-versus-deferred scope,
unchanged `next` state, verified previous-suite inventory, `git diff --check`,
and unrelated-change absence. Suggested planning commit message:

```text
Plan Sprint 15 Runtime service container
```

#### Sprint 16 HTTP API and Health execution plan

Sprint 16 is planned from committed readiness head
`8ca1c0ce3c83dae8bb76fa52a40423bead693f40`. The
[Sprint 15 integration review](reviews/sprint-15-runtime-service-container.md)
records `pass`, so Sprint 16 is the unique `next` target. The committed Runtime
Service profile, workflow, and template cover transport ownership, lifecycle,
health, readiness, cancellation, shutdown, compatibility, and public
client/server evidence; the framework readiness audit found no reusable gap.

The [Sprint 16 integration review](reviews/sprint-16-http-api-health.md) now
records `pass`. Sprint 16 is completed, its accepted listener and health
boundary remain the current Runtime baseline, and Sprint 17 Workspace Service
is the unique `next` target.

The live Runtime already exposes a public, long-running service container,
transport-neutral lifecycle observation, deterministic shutdown injection, and
distinct startup and task failures. Its locked dependency surface includes
Axum 0.8.9, Tokio 1.53.0, Serde, and Serde JSON. Repository code, locally
available locked dependency sources, macOS/Windows CI, loopback sockets, and
bounded channel coordination provide sufficient investigation data and test
oracles without an external service, remote fixture, arbitrary sleep, or new
production dependency.

Task 1 records the exact HTTP, configuration, lifecycle, failure, ownership,
wire, and test boundary. Task 2 accepts ADR-0038 before production behavior is
implemented. Task 3 adds only transport-neutral lifecycle-derived health state.
Task 4 implements and composes the owned HTTP service and accepted probe routes.
Task 5 completes public loopback client/server, failure, shutdown, repeated-run,
and current-state evidence. Task 6 records the passing integration review,
state transition, and Sprint 15 prompt-suite retirement.

##### Sprint 16 objective

Expose the long-running Runtime through the first owned HTTP listener with
stable liveness and lifecycle-derived readiness probes, deterministic startup
failure and graceful shutdown, and public cross-platform client/server evidence,
without pulling workspace, graph-query, watcher, cache, or supported CLI
capabilities forward.

##### Included scope

- exact investigation of the existing Runtime, locked HTTP APIs, configuration,
  listener ownership, lifecycle observation, failures, consumers, wire choices,
  and deterministic cross-platform testability;
- an accepted ADR for bind configuration, route and method matrix, response
  schema, liveness/readiness semantics, status codes, compatibility boundary,
  ownership, startup acknowledgement, cancellation, shutdown, errors,
  observability, first slice, and deferred scope;
- one transport-neutral health snapshot derived from owned Runtime lifecycle
  state, with no independently mutable readiness label;
- one Runtime-owned Axum service with explicit listener ownership, accepted
  liveness/readiness routes, startup failure propagation, cooperative graceful
  shutdown, and thin composition-root registration;
- public loopback HTTP evidence for the accepted wire contract, lifecycle
  transitions, negative routes and methods, bind failure, shutdown, listener
  release, and repeated fresh runs;
- current-state documentation, complete workspace validation, integration
  review, and conditional Sprint 15 prompt-suite retirement.

##### Excluded scope

- workspace lifecycle and semantic-build orchestration owned by Sprint 17;
- graph and semantic query endpoints owned by Sprint 18;
- file watching, persistent cache, supported CLI behavior, MCP, LSP, IDE, AI,
  TLS, authentication, authorization, CORS, compression, metrics, tracing
  export, API version negotiation, OpenAPI, streaming, request bodies, and
  domain error mapping;
- semantic graph or adapter changes, dynamic service registration, restart,
  retries, forced termination, a newly selected shutdown timeout, global mutable
  health state, detached tasks, external services, arbitrary-sleep evidence,
  platform-specific socket assumptions, packaging, and performance claims.

##### Sprint 16 prerequisite and retirement gate

Task 1 requires one committed Sprint 16 planning baseline containing this plan
and the complete suite under
`docs/codex/prompts/sprint-16-http-api-health/`. Every dependent task requires
the preceding committed outcome. Stored prompts do not authorize commits;
authorization comes only from the launching user instruction.

The immediately preceding suite is exactly
`docs/codex/prompts/sprint-15-runtime-service-container/`, with these seven
tracked files: `00-sprint-15-execution-loop.md`,
`01-investigate-runtime-service-container.md`,
`02-define-runtime-service-container-contract.md`,
`03-implement-runtime-service-container.md`,
`04-integrate-runtime-application-lifecycle.md`,
`05-complete-runtime-service-container-evidence.md`, and
`06-sprint-15-integration-review.md`. It remains untouched through Task 5. Only
Task 6 may retire this exact inventory after a non-blocking review and successful
complete validation.

##### Ordered task manifest

| Order | Task | Profile / template | Owned outcome | Required committed prerequisite | Suggested commit message |
|---:|---|---|---|---|---|
| 1 | Investigate the HTTP API and health boundary. | Investigation / investigation | Current Runtime, Axum/Tokio, configuration, endpoint, wire, ownership, lifecycle, failure, consumer, and public-test evidence with explicit decision questions. | Accepted Sprint 16 planning baseline. | `Investigate Sprint 16 HTTP API and health` |
| 2 | Define the HTTP API and health contract. | Architecture / architecture | Accepted ADR-0038 listener, configuration, route, method, schema, liveness, readiness, status, lifecycle, ownership, failure, shutdown, compatibility, first-slice, and deferred-scope contract. | Task 1. | `Define Sprint 16 HTTP API and health contract` |
| 3 | Implement lifecycle-derived Runtime health state. | Runtime Service / runtime service | Transport-neutral health snapshot derived only from canonical lifecycle state, with focused transition and repeated-run evidence. | Task 2. | `Implement Sprint 16 Runtime health state` |
| 4 | Implement and compose the HTTP service. | Runtime Service / runtime service | Runtime-owned listener, accepted Axum routes and schema, bind/start failure propagation, cancellation-driven graceful shutdown, and production registration. | Task 3. | `Implement Sprint 16 HTTP service` |
| 5 | Complete public HTTP and health evidence. | Runtime Service / runtime service | Public loopback wire matrix, lifecycle/readiness transitions, negative requests, bind failure, cleanup/rebind, repeated-run evidence, and synchronized current-state docs. | Task 4. | `Complete Sprint 16 HTTP API and health evidence` |
| 6 | Review the integrated Sprint 16 baseline. | Review / review | Findings, complete validation, sprint decision, Sprint 15 suite retirement, and Sprint 17 hand-off. | Task 5 and all implementation validation. | `Complete Sprint 16 HTTP API and health review` |

```text
Committed Sprint 16 planning baseline
    -> Task 1 HTTP and health investigation
    -> Task 2 accepted ADR-0038
    -> Task 3 lifecycle-derived Runtime health state
    -> Task 4 owned HTTP service and composition
    -> Task 5 public client/server evidence and current-state docs
    -> Task 6 integration review and conditional Sprint 15 suite retirement
    -> Sprint 17 planning eligibility
```

##### Task contracts

Task 1 creates only `docs/architecture/http-api-health-investigation.md`. It
records exact Runtime and dependency APIs, configuration and listener choices,
endpoint candidates, lifecycle evidence, failure and ownership questions,
compatibility-sensitive behavior, a raw loopback test oracle, and decision
readiness. It selects no architecture and changes no production behavior.

Task 2 creates `docs/adr/0038-http-api-health.md` and synchronizes only the
planning-level architecture text required by the decision. It fixes the public
bind/configuration contract, exact route/method/status/schema matrix, liveness
and readiness meaning, lifecycle derivation, listener and connection ownership,
startup and runtime failures, cancellation and graceful shutdown, observability,
compatibility, first slice, rejected alternatives, and deferred scope. It
implements no production behavior.

Task 3 implements only the accepted transport-neutral health model and focused
tests. Health must be a deterministic projection of the canonical lifecycle,
must not report ready before `Running` or after `Stopping`, and must not create a
second mutable state authority. It changes no HTTP listener or production route.

Task 4 implements the accepted HTTP service and composition boundary. The
listener is acquired during service startup, its task is Runtime-owned, and
Runtime cancellation drives graceful server completion. The accepted routes
read only shared state and expose no workspace or graph behavior. Focused and
complete workspace validation are required.

Task 5 exercises the public production boundary over loopback TCP using actual
HTTP requests. It proves the exact success and negative wire matrix, readiness
changes from lifecycle evidence, named bind failure, cancellation and listener
release, and equal fresh runs without arbitrary sleeps. It synchronizes README,
Architecture, and Semantic Model current-state text but does not complete the
sprint.

Task 6 reviewed the planning-through-Task-5 range and created
`docs/reviews/sprint-16-http-api-health.md` after all focused and full
validation succeeded. Its `pass` decision transitions Sprint 16 to `completed`,
makes Sprint 17 Workspace Service the unique `next` target, and atomically
retires the exact Sprint 15 suite in the same review commit. No inventory
mismatch, endangered untracked file, or retained link dependency was found;
the review changed no production code.

##### Sprint 16 state gates and completion criteria

Sprint 16 remained `next` during planning and became `active` after the planning
commit and Task 1 start. Its Task 6 `pass` decision now records it as
`completed`. Sprint 17 is the unique `next` target.

Completion requires committed or proven Tasks 1-5, accepted ADR-0038, one owned
HTTP listener, stable liveness and lifecycle-derived readiness behavior, exact
wire compatibility, deterministic startup failure and cancellation-driven
shutdown, no detached listener or task, public cross-platform loopback evidence,
preserved deferred scope, the complete workspace gate, and a non-blocking Task
6 decision. A blocked review keeps Sprint 16 incomplete, preserves the Sprint 15
suite, and leaves Sprint 17 ineligible.

Planning validation covers Markdown links and structure, prompt numbering,
manifest/prerequisite/commit-message agreement, accepted-versus-deferred scope,
unchanged `next` state, verified previous-suite inventory, `git diff --check`,
and unrelated-change absence. Suggested planning commit message:

```text
Plan Sprint 16 HTTP API and health
```

#### Sprint 17 Workspace Service execution plan

Sprint 17 is planned from committed readiness head
`dd08923a54f5eacf5aad5a3cbc1a16267dadaa21`. The
[Sprint 16 integration review](reviews/sprint-16-http-api-health.md) records
`pass`, Sprint 16 is `completed`, and Sprint 17 is the unique `next` target.

The live repository provides sufficient production and test evidence for a
bounded sprint. `oneagent-workspace` owns configuration and discovery ports;
`oneagent-workspace-fs` deterministically discovers EDT and Designer XML roots;
the EDT and Designer XML adapters expose production semantic graph builders;
`oneagent-graph` owns canonical graph state; and `oneagent-runtime` exposes a
public long-running service container, lifecycle-derived readiness, owned HTTP
service, deterministic cancellation, shutdown, and public integration seams.
Repository-owned fixtures cover both supported formats, discovery errors,
semantic builds, repeated construction, and cross-adapter conformance without
an external service or ignored local corpus requirement.

Architecture is not yet sufficient to implement the service directly. Existing
ADRs assign composition, discovery, adapter, graph, lifecycle, and health
ownership, but do not decide Workspace root configuration, multi-configuration
result shape, build dispatch, publication atomicity, duplicate/collision policy,
startup acknowledgement, readiness, or build-task ownership. Task 1 records
the live boundary and test oracles; Task 2 accepts ADR-0039 before production
behavior changes.

The committed Runtime Service, Investigation, Architecture, and Review profiles,
their task templates, and the sequential sprint workflow express every planned
evidence, ownership, validation, and reporting contract. No reusable Codex
Framework gap exists and no framework change or post-sprint framework audit is
planned.

##### Sprint 17 objective

Add the first Runtime-owned Workspace lifecycle and semantic-build orchestration
service for repository-owned EDT and Designer XML inputs, with deterministic
configuration and discovery, immutable published semantic state, truthful
readiness, explicit failure and cancellation behavior, and public integration
evidence, without pulling graph-query APIs, file watching, persistence, or
supported CLI behavior forward.

##### Included scope

- exact investigation of current workspace/discovery, EDT and Designer XML
  builders, graph/result contracts, Runtime configuration/state/lifecycle,
  consumers, errors, dependencies, fixtures, and deterministic testability;
- accepted ADR-0039 for ownership and dependency direction, root configuration,
  discovery/build dispatch, supported formats, immutable snapshot/result shape,
  identity and ordering, diagnostics, atomicity, duplicates/collisions, startup,
  readiness, errors, cancellation, shutdown, observability, first slice, and
  deferred scope;
- the accepted source-neutral immutable Workspace snapshot and semantic-build
  dispatch boundary with focused deterministic evidence;
- one Runtime-owned Workspace service with configured discovery, initial builds,
  atomic snapshot publication, lifecycle/readiness integration, typed failures,
  cooperative cancellation, cleanup, and thin composition-root registration;
- public integration evidence through the real filesystem detector and both
  production builders over provenance-backed repository-owned inputs, including
  positive, empty/multiple, failure, ordering, atomicity, readiness, shutdown,
  cleanup, and repeated fresh-run cases;
- current-state documentation, complete workspace validation, integration
  review, and conditional Sprint 16 prompt-suite retirement.

##### Excluded scope

- graph and semantic query endpoints owned by Sprint 18;
- file watching and rebuild triggers owned by Sprint 19, persistent cache owned
  by Sprint 20, and supported CLI behavior owned by Sprint 21;
- HTTP Workspace routes, MCP, LSP, IDE, AI, authentication, authorization,
  TLS, metrics, tracing export, streaming, request bodies, and domain error wire
  mapping;
- graph semantic/model changes, adapter parser changes, source-format expansion,
  partial/incremental builds not already accepted by ADR-0039, dynamic service
  registration, global mutable state, detached tasks, retries, restart, forced
  termination, a new timeout, external services, ignored local corpus runtime
  dependencies, arbitrary-sleep evidence, new external production dependencies,
  packaging, benchmarks, and unsupported performance claims.

##### Sprint 17 prerequisite and retirement gate

Task 1 requires one committed Sprint 17 planning baseline containing this plan
and the complete suite under
`docs/codex/prompts/sprint-17-workspace-service/`. Every dependent task requires
the preceding committed outcome. Stored prompts do not authorize commits;
authorization comes only from the launching user instruction.

The immediately preceding suite is exactly
`docs/codex/prompts/sprint-16-http-api-health/`, with these seven tracked files:
`00-sprint-16-execution-loop.md`,
`01-investigate-http-api-health-boundary.md`,
`02-define-http-api-health-contract.md`,
`03-implement-runtime-health-state.md`,
`04-implement-http-service.md`,
`05-complete-http-api-health-evidence.md`, and
`06-sprint-16-integration-review.md`. The tracked and filesystem inventories
match and contain no untracked file at planning time. The suite remains
untouched through Task 5. Only Task 6 may retire this exact inventory after a
non-blocking review and successful complete validation.

##### Ordered task manifest

| Order | Task | Profile / template | Owned outcome | Required committed prerequisite | Suggested commit message |
|---:|---|---|---|---|---|
| 1 | Investigate the Workspace service boundary. | Investigation / investigation | Current discovery, builders, graph/result, Runtime state/lifecycle, consumer, failure, dependency, fixture, and deterministic-test evidence with explicit ADR questions. | Accepted Sprint 17 planning baseline. | `Investigate Sprint 17 Workspace service` |
| 2 | Define the Workspace service contract. | Architecture / architecture | Accepted ADR-0039 ownership, configuration, discovery/build, snapshot, atomicity, ordering, lifecycle/readiness, failure, cancellation, shutdown, first-slice, and deferred-scope contract. | Task 1. | `Define Sprint 17 Workspace service contract` |
| 3 | Implement the immutable Workspace snapshot. | Runtime Service / runtime service | Source-neutral immutable Workspace snapshot and supported semantic-build dispatch boundary with focused evidence. | Task 2. | `Implement Sprint 17 Workspace snapshot` |
| 4 | Implement and compose the Workspace service. | Runtime Service / runtime service | Runtime-owned configured discovery/build service, snapshot publication, lifecycle/readiness, failure, cancellation, shutdown, and composition. | Task 3. | `Implement Sprint 17 Workspace service` |
| 5 | Complete public Workspace service evidence. | Runtime Service / runtime service | Public production EDT/Designer XML orchestration matrix, atomicity/cleanup/repeated-run evidence, and synchronized current-state docs. | Task 4. | `Complete Sprint 17 Workspace service evidence` |
| 6 | Review the integrated Sprint 17 baseline. | Review / review | Findings, complete validation, sprint decision, Sprint 16 suite retirement, and Sprint 18 hand-off. | Task 5 and all implementation validation. | `Complete Sprint 17 Workspace service review` |

```text
Committed Sprint 17 planning baseline
    -> Task 1 Workspace service investigation
    -> Task 2 accepted ADR-0039
    -> Task 3 immutable Workspace snapshot and build dispatch
    -> Task 4 Runtime-owned Workspace service and composition
    -> Task 5 public production evidence and current-state docs
    -> Task 6 integration review and conditional Sprint 16 suite retirement
    -> Sprint 18 planning eligibility
```

##### Task contracts

Task 1 creates only
`docs/architecture/workspace-service-investigation.md`. It records exact
definitions, dependency and consumer boundaries, supported builder contracts,
result/diagnostic differences, fixtures, CI constraints, lifecycle/readiness,
failure and cancellation questions, candidate snapshot/public-test seams, and
decision readiness. It selects no architecture and changes no production
behavior.

Task 2 creates `docs/adr/0039-workspace-service.md` and synchronizes only
planning-level architecture text required by the decision. It fixes composition
and dependency direction, root configuration, deterministic discovery and
format dispatch, canonical snapshot/result authority, ordering and identity,
diagnostics, publication atomicity, duplicates/collisions, startup and readiness,
blocking-work ownership, errors, cancellation, shutdown, observability,
deterministic testing, first slice, rejected alternatives, migration, and
deferred scope. It implements no production behavior.

Task 3 implements only the ADR-0039 source-neutral immutable snapshot and build
dispatch boundary with focused tests. It does not register a Runtime service,
change App lifecycle/readiness, add transport behavior, or choose later rebuild
and persistence semantics.

Task 4 composes the committed Task 3 boundary into one ADR-0037 Runtime service.
It implements configured discovery and initial builds, atomic snapshot
publication, accepted readiness, named startup/runtime failures, cancellation,
shutdown, cleanup, and production registration. All blocking work and channels
remain owned; no graph or Workspace endpoint, watcher, cache, or supported CLI
behavior is added.

Task 5 exercises the public Runtime boundary through the real filesystem
detector and both production format builders. It proves the accepted positive,
empty/multiple, invalid/failing, atomicity, lifecycle/readiness, shutdown,
cleanup, ordering, and repeated fresh-run matrix with provenance-backed
cross-platform inputs and no arbitrary sleeps. It synchronizes README,
Architecture, and Semantic Model current-state text but does not complete the
sprint.

Task 6 reviews the planning-through-Task-5 range and creates
`docs/reviews/sprint-17-workspace-service.md` only for `pass` or `pass with
non-blocking follow-ups` after all focused and full validation succeeds. That
decision transitions Sprint 17 to `completed`, makes Sprint 18 Graph Query API
the unique `next` target, and atomically retires the exact Sprint 16 suite in
the same review commit. An inventory mismatch, endangered untracked file, or
retained link dependency blocks retirement and the final commit. The review
never silently fixes code.

##### Sprint 17 state gates and completion criteria

Sprint 17 remains `next` during planning and becomes `active` only after the
planning commit and Task 1 start. `already_complete` requires current committed
evidence and successful required validation; no empty commit is created. Stop
after the first prerequisite, implementation, validation, staging, commit, or
review failure. Do not skip, reorder, combine, or partially commit tasks.

Completion requires committed or proven Tasks 1-5, accepted ADR-0039, one
canonical immutable published Workspace snapshot boundary, deterministic
supported-format discovery/build orchestration, truthful lifecycle/readiness,
atomic failures, complete task/resource ownership, public production evidence
for EDT and Designer XML, preserved graph and adapter authority, explicit
deferred scope, the complete workspace gate, and a non-blocking Task 6 decision.
A blocked review keeps Sprint 17 incomplete, preserves the Sprint 16 suite, and
leaves Sprint 18 ineligible.

The [Sprint 17 integration review](reviews/sprint-17-workspace-service.md)
records `pass` against committed Task 5 head
`7626c46a11f683b11fc8fc007738a85e5e65ea86`. Sprint 17 is `completed`, Sprint
18 Graph Query API is the unique `next` target, and the exact verified Sprint 16
prompt suite is retired in the atomic review commit.

Planning validation covers Markdown links and structure, prompt numbering,
manifest/prerequisite/commit-message agreement, accepted-versus-deferred scope,
unchanged `next` state, verified previous-suite inventory, `git diff --check`,
and unrelated-change absence. Suggested planning commit message:

```text
Plan Sprint 17 Workspace service
```

#### Sprint 18 Graph Query API execution plan

Sprint 18 is planned from committed readiness head
`dac86be41eed3356e230079ab2607503a85f5b87`. The
[Sprint 17 integration review](reviews/sprint-17-workspace-service.md) records
`pass`, Sprint 17 is `completed`, and Sprint 18 is the unique `next` target.

The live repository provides sufficient production and test evidence for a
bounded sprint. `oneagent-graph` exposes deterministic read-only node, edge,
containment, dependency, usage, and bounded traversal queries over canonical
immutable graphs. ADR-0039 and `oneagent-runtime` expose separate immutable
per-configuration graph snapshots through a cloneable observer. ADR-0038 and
the Runtime HTTP service provide an owned Axum listener, exact method and
fallback behavior, lifecycle-derived readiness, loopback integration seams,
and cooperative shutdown. Repository-owned Runtime fixtures build real EDT and
Designer XML graphs and already prove multi-configuration selection,
deterministic ordering, negative startup behavior, and repeated fresh runs.

Architecture is not yet sufficient to implement the API directly. Existing
ADRs do not decide the transport-neutral query boundary, configuration and node
selection, bounded first-slice operations, owned response projection, versioned
routes, request limits, status/error mapping, wire schemas, compatibility, or
HTTP access to Workspace observation. Task 1 records the live query, snapshot,
transport, consumer, dependency, and test oracles; Task 2 accepts ADR-0040
before production behavior changes. Planning does not invent route names,
serialized fields, or unsupported query semantics.

The committed Runtime Service, Investigation, Architecture, Implementation, and
Review profiles, their task templates, and the sequential sprint workflow
express every planned evidence, compatibility, ownership, validation, and
reporting contract. The Runtime Services and APIs readiness stage explicitly
covers Sprints 15-19. No reusable Codex Framework gap exists and no framework
change or post-sprint framework audit is planned.

##### Sprint 18 objective

Expose the first stable bounded graph and semantic query API over one selected
published Workspace configuration, with deterministic transport-neutral
results, exact HTTP compatibility, truthful lifecycle behavior, explicit
limits and errors, and public production evidence, without changing semantic
facts or pulling file watching, persistence, or the supported CLI forward.

##### Included scope

- exact investigation of existing graph queries, identifiers, payload and
  provenance surfaces, Workspace observation, configuration selection, Runtime
  HTTP state, consumers, dependencies, compatibility constraints, fixtures,
  and deterministic cross-platform testability;
- accepted ADR-0040 for ownership and dependency direction, snapshot and
  configuration selection, first-slice operations, request bounds, stable
  owned result projection, missing/invalid state, route and method matrix,
  status/error/schema mapping, compatibility, cancellation, shutdown,
  observability, deterministic testing, and deferred scope;
- one transport-neutral Runtime graph-query boundary over immutable selected
  configuration snapshots, preserving canonical graph authority and exact
  deterministic ordering with focused positive and negative evidence;
- one versioned Runtime-owned HTTP graph-query surface that maps only the
  accepted operations and failures, shares the existing listener and lifecycle,
  and preserves the Sprint 16 health contract;
- public loopback evidence over real production EDT and Designer XML builds for
  configuration and node selection, accepted graph operations, bounded inputs,
  exact success and error wire contracts, ordering, lifecycle/shutdown,
  cleanup, and repeated fresh runs;
- current-state documentation, complete workspace validation, integration
  review, and conditional Sprint 17 prompt-suite retirement.

##### Excluded scope

- new graph node, edge, identity, query, resolution, validation, diff, Impact,
  report, provenance, or Coverage semantics;
- file watching, rebuild triggers, stale-snapshot or invalidation policy owned
  by Sprint 19; persistent cache owned by Sprint 20; supported CLI client owned
  by Sprint 21;
- aggregate or merged cross-configuration graphs, cross-configuration
  traversal, mutation, write routes, batch requests, streaming, subscriptions,
  pagination beyond an explicitly accepted bounded first slice, fuzzy or
  full-text search, shortest path, unbounded closure, source-fragment download,
  and arbitrary query languages;
- MCP, LSP, IDE, AI/context, authentication, authorization, TLS, CORS,
  compression, rate limiting, request IDs, metrics/tracing export, OpenAPI,
  general version negotiation, retries, restart, forced termination, new
  external production dependencies, packaging, benchmarks, and unsupported
  performance or security claims.

##### Sprint 18 prerequisite and retirement gate

Task 1 requires one committed Sprint 18 planning baseline containing this plan
and the complete suite under
`docs/codex/prompts/sprint-18-graph-query-api/`. Every dependent task requires
the preceding committed outcome. Stored prompts do not authorize commits;
authorization comes only from the launching user instruction.

The immediately preceding suite is exactly
`docs/codex/prompts/sprint-17-workspace-service/`, with these seven tracked
files: `00-sprint-17-execution-loop.md`,
`01-investigate-workspace-service-boundary.md`,
`02-define-workspace-service-contract.md`,
`03-implement-workspace-snapshot.md`,
`04-implement-workspace-service.md`,
`05-complete-workspace-service-evidence.md`, and
`06-sprint-17-integration-review.md`. The tracked and filesystem inventories
match and contain no untracked file at planning time. The suite remains
untouched through Task 5. Only Task 6 may retire this exact inventory after a
non-blocking review and successful complete validation.

##### Ordered task manifest

| Order | Task | Profile / template | Owned outcome | Required committed prerequisite | Suggested commit message |
|---:|---|---|---|---|---|
| 1 | Investigate the Graph Query API boundary. | Investigation / investigation | Current graph-query, snapshot, Runtime HTTP, consumer, compatibility, dependency, fixture, and deterministic-test evidence with explicit ADR questions. | Accepted Sprint 18 planning baseline. | `Investigate Sprint 18 Graph Query API` |
| 2 | Define the Graph Query API contract. | Architecture / architecture | Accepted ADR-0040 ownership, selection, operation, bounds, result, route, method, schema, error, compatibility, lifecycle, first-slice, and deferred-scope contract. | Task 1. | `Define Sprint 18 Graph Query API contract` |
| 3 | Implement the transport-neutral Graph Query service. | Runtime Service / runtime service | Read-only selected-snapshot query boundary, owned deterministic results, stable errors, limits, and focused evidence. | Task 2. | `Implement Sprint 18 Graph Query service` |
| 4 | Implement and compose the Graph Query HTTP API. | Runtime Service / runtime service | Accepted versioned routes, exact request/response and error mapping, Workspace observation wiring, lifecycle preservation, and focused HTTP evidence. | Task 3. | `Implement Sprint 18 Graph Query HTTP API` |
| 5 | Complete public Graph Query API evidence. | Runtime Service / runtime service | Public production EDT/Designer XML loopback matrix, bounds/errors/cleanup/repeated-run evidence, and synchronized current-state docs. | Task 4. | `Complete Sprint 18 Graph Query API evidence` |
| 6 | Review the integrated Sprint 18 baseline. | Review / review | Findings, complete validation, sprint decision, Sprint 17 suite retirement, and Sprint 19 hand-off. | Task 5 and all implementation validation. | `Complete Sprint 18 Graph Query API review` |

```text
Committed Sprint 18 planning baseline
    -> Task 1 Graph Query API investigation
    -> Task 2 accepted ADR-0040
    -> Task 3 transport-neutral Graph Query service
    -> Task 4 versioned Graph Query HTTP API and composition
    -> Task 5 public production evidence and current-state docs
    -> Task 6 integration review and conditional Sprint 17 suite retirement
    -> Sprint 19 planning eligibility
```

##### Task contracts

Task 1 creates only
`docs/architecture/graph-query-api-investigation.md`. It records exact public
query definitions and results, stable identifiers and value vocabularies,
snapshot/configuration selection, Runtime and HTTP ownership, current consumers,
dependency constraints, fixtures, CI constraints, compatibility-sensitive
health behavior, candidate bounded operations, public test seams, and every
decision ADR-0040 must make. It selects no architecture and changes no
production behavior.

Task 2 creates `docs/adr/0040-graph-query-api.md` and synchronizes only
planning-level architecture text required by the decision. It fixes semantic
authority and dependency direction, immutable snapshot selection, exact bounded
operation and limit policy, owned response projection, stable identifiers and
ordering, missing/invalid state, versioned route/method/status/schema matrix,
error mapping, compatibility, lifecycle, cancellation/shutdown, observability,
deterministic testing, first slice, migration, rejected alternatives, and
deferred scope. It implements no production behavior.

Task 3 implements only the ADR-0040 transport-neutral query boundary with
focused tests. It consumes immutable Workspace observation, selects exactly one
configuration, delegates to existing canonical graph queries, returns owned
deterministic results and typed failures, and enforces accepted bounds. It does
not add routes, change graph semantics, rebuild snapshots, or select a second
wire or protocol authority.

Task 4 maps the committed Task 3 boundary through the existing Runtime-owned
HTTP listener. It implements only the accepted exact versioned route, method,
status, JSON, and error matrix; wires Workspace observation at the composition
root; preserves health probes and lifecycle-derived readiness; and adds focused
handler and loopback evidence without a new listener, background task, semantic
query language, or external production dependency.

Task 5 exercises the public production boundary through the real filesystem
detector, EDT and Designer XML builders, immutable Workspace publication,
transport-neutral query service, and loopback HTTP server. It proves the
accepted success, multi-configuration selection, missing/invalid input,
operation bounds, deterministic ordering, lifecycle/shutdown, cleanup, and
fresh repetition matrix with provenance-backed cross-platform inputs and no
arbitrary sleeps. It synchronizes README, Architecture, and Semantic Model
current-state text but does not complete the sprint.

Task 6 reviews the planning-through-Task-5 range and creates
`docs/reviews/sprint-18-graph-query-api.md` only for `pass` or `pass with
non-blocking follow-ups` after all focused and full validation succeeds. That
decision transitions Sprint 18 to `completed`, makes Sprint 19 File Watching
the unique `next` target, and atomically retires the exact Sprint 17 suite in
the same review commit. An inventory mismatch, endangered untracked file, or
retained link dependency blocks retirement and the final commit. The review
never silently fixes code.

##### Sprint 18 state gates and completion criteria

Sprint 18 remains `next` during planning and becomes `active` only after the
planning commit and Task 1 start. `already_complete` requires current committed
evidence and successful required validation; no empty commit is created. Stop
after the first prerequisite, implementation, validation, staging, commit, or
review failure. Do not skip, reorder, combine, or partially commit tasks.

Completion requires committed or proven Tasks 1-5, accepted ADR-0040, one
transport-neutral selected-snapshot query authority, deterministic owned
results and limits, exact versioned HTTP compatibility and error mapping,
truthful lifecycle behavior, public production evidence for EDT and Designer
XML configurations, preserved canonical graph semantics and health routes,
explicit deferred scope, the complete workspace gate, and a non-blocking Task 6
decision. A blocked review keeps Sprint 18 incomplete, preserves the Sprint 17
suite, and leaves Sprint 19 ineligible.

The [Sprint 18 integration review](reviews/sprint-18-graph-query-api.md) records
`pass` against committed Task 5 head
`d7ba04bc6a4b0e18d46d051419809c2e0756fce7`. Sprint 18 is `completed`, Sprint
19 File Watching is the unique `next` target, and the exact verified Sprint 17
prompt suite is retired in the atomic review commit.

Planning validation covers Markdown links and structure, prompt numbering,
manifest/prerequisite/commit-message agreement, accepted-versus-deferred scope,
unchanged `next` state, verified previous-suite inventory, `git diff --check`,
and unrelated-change absence. Suggested planning commit message:

```text
Plan Sprint 18 Graph Query API
```

#### Sprint 19 File Watching execution plan

Sprint 19 is planned from committed readiness head
`ee0be7b4803651a6631c80493aa841b7b14e5e41`. The Sprint 18 integration
review records `pass`, Sprint 19 is the unique `next` target, the working tree
is clean, and the tracked Runtime fixture plus deterministic temporary copies
provide repository-owned sources for add, modify, remove, rename, burst,
invalid-build, recovery, shutdown, cleanup, and repeated-run evidence. No
accepted decision currently defines watcher technology, normalized change
semantics, coalescing, rebuild scheduling, failure publication, or recovery, so
investigation and ADR acceptance precede implementation. The existing Runtime
Service profile, workflow, and template already cover service ownership,
concurrency, cancellation, shutdown, health, observability, and public Runtime
integration; no Codex Framework change is required. A new production
dependency is not selected by planning and still requires explicit approval if
the accepted architecture proves one necessary.

##### Objective

Detect relevant changes under the configured Workspace root and connect them
to deterministic Runtime-owned rebuild orchestration so that complete valid
immutable snapshots replace the previous publication atomically, readers
never observe a partial build, lifecycle and health contracts remain truthful,
and failure, recovery, shutdown, cleanup, and repeated fresh runs have public
cross-platform evidence.

##### Included scope

- repository-backed investigation of filesystem observation, Workspace build,
  snapshot publication, Runtime service ordering, graph-query observation,
  dependencies, platform constraints, fixtures, and deterministic test seams;
- an accepted ADR defining watcher ownership, watched boundary, normalized
  change vocabulary, relevance, coalescing, scheduling, serialization,
  publication, failure and recovery, lifecycle, shutdown, observability,
  compatibility, dependency, first-slice, and deferred-scope contracts;
- one Runtime-owned file-change observation boundary with deterministic
  normalization, bounded coordination, cancellation, terminal cleanup, and
  focused positive/negative/burst evidence;
- serialized complete Workspace rebuild orchestration using the existing
  production detector and EDT/Designer XML builders, atomic valid publication,
  stable snapshot observation, failure retention or clearing exactly as
  accepted, recovery, and focused integration evidence;
- public production evidence for relevant and irrelevant changes, add/modify/
  remove/rename-equivalent source transitions, burst coalescing, invalid build
  and recovery, graph-query visibility, shutdown, cleanup, and equal fresh
  runs without arbitrary sleeps;
- current-state documentation, complete workspace validation, integration
  review, and conditional Sprint 18 prompt-suite retirement.

##### Excluded scope

- graph node, edge, identity, query, resolution, validation, Diff, Impact,
  report, provenance, source-parser, source-format, or Coverage semantics;
- incremental graph or Semantic Index mutation, partial snapshot publication,
  per-file semantic repair, persistent cache owned by Sprint 20, and supported
  CLI behavior owned by Sprint 21;
- Git change ingestion, remote/network workspaces, symlink policy expansion,
  OS-specific public APIs, filesystem writes, edit transactions, watch-control
  HTTP routes, streaming, subscriptions, progress APIs, dynamic configuration,
  retries outside the accepted change/recovery loop, restart, forced
  termination, authentication, authorization, metrics/tracing export,
  benchmarks, and unsupported performance or security claims.

##### Sprint 19 prerequisite and retirement gate

Task 1 requires one committed Sprint 19 planning baseline containing this plan
and the complete suite under
`docs/codex/prompts/sprint-19-file-watching/`. Every dependent task requires the
preceding committed outcome. Stored prompts do not authorize commits;
authorization comes only from the launching user instruction.

The immediately preceding suite is exactly
`docs/codex/prompts/sprint-18-graph-query-api/`, with these seven tracked files:
`00-sprint-18-execution-loop.md`,
`01-investigate-graph-query-api-boundary.md`,
`02-define-graph-query-api-contract.md`,
`03-implement-graph-query-service.md`,
`04-implement-graph-query-http-api.md`,
`05-complete-graph-query-api-evidence.md`, and
`06-sprint-18-integration-review.md`. The tracked and filesystem inventories
match and contain no untracked file at planning time. The suite remains
untouched through Task 5. Only Task 6 may retire this exact inventory after a
non-blocking review and successful complete validation.

##### Ordered task manifest

| Order | Task | Profile / template | Owned outcome | Required committed prerequisite | Suggested commit message |
|---:|---|---|---|---|---|
| 1 | Investigate the File Watching boundary. | Investigation / investigation | Current filesystem, Workspace build/publication, Runtime lifecycle, consumer, dependency, platform, fixture, and deterministic-test evidence with explicit ADR questions. | Accepted Sprint 19 planning baseline. | `Investigate Sprint 19 File Watching` |
| 2 | Define the File Watching contract. | Architecture / architecture | Accepted ADR-0041 watcher ownership, change, relevance, coalescing, rebuild, publication, failure/recovery, lifecycle, dependency, compatibility, first-slice, and deferred-scope contract. | Task 1. | `Define Sprint 19 File Watching contract` |
| 3 | Implement file-change watching. | Runtime Service / runtime service | Accepted Runtime-owned observation boundary, normalized relevant change signals, bounded coordination, cancellation, cleanup, and focused evidence. | Task 2. | `Implement Sprint 19 file change watching` |
| 4 | Integrate Workspace rebuild orchestration. | Runtime Service / runtime service | Serialized complete rebuilds, atomic valid snapshot replacement, accepted failure/recovery behavior, lifecycle preservation, and focused evidence. | Task 3. | `Integrate Sprint 19 Workspace rebuilds` |
| 5 | Complete public File Watching evidence. | Runtime Service / runtime service | Public production change/rebuild/query/failure/recovery/shutdown matrix and synchronized current-state docs. | Task 4. | `Complete Sprint 19 File Watching evidence` |
| 6 | Review the integrated Sprint 19 baseline. | Review / review | Findings, complete validation, sprint decision, Sprint 18 suite retirement, and Sprint 20 hand-off. | Task 5 and all implementation validation. | `Complete Sprint 19 File Watching review` |

```text
Committed Sprint 19 planning baseline
    -> Task 1 File Watching investigation
    -> Task 2 accepted ADR-0041
    -> Task 3 Runtime-owned file-change watching
    -> Task 4 serialized Workspace rebuild orchestration
    -> Task 5 public production evidence and current-state docs
    -> Task 6 integration review and conditional Sprint 18 suite retirement
    -> Sprint 20 planning eligibility
```

##### Task contracts

Task 1 creates only
`docs/architecture/file-watching-investigation.md`. It records exact current
filesystem discovery and read behavior, Workspace builder/publication and
observer contracts, Runtime service ordering and cancellation, graph-query
consumer visibility, dependency and platform constraints, fixture provenance,
deterministic test seams, relevant/irrelevant change candidates, and every
decision ADR-0041 must make. It selects no architecture, adds no dependency,
and changes no production behavior.

Task 2 creates `docs/adr/0041-file-watching.md` and synchronizes only
planning-level architecture text required by the decision. It fixes ownership
and dependency direction, watched roots and relevance, normalized change and
overflow/error vocabulary, event coalescing and scheduling, build
serialization, snapshot publication and reader consistency, failure retention
or clearing, recovery, lifecycle/readiness, cancellation/shutdown,
observability, deterministic testing, dependency policy, migration, first
slice, rejected alternatives, and deferred scope. It implements no production
behavior. If the accepted contract requires a new production dependency, Task
3 cannot change a manifest until explicit approval is available.

Task 3 implements only the ADR-0041 file-change observation boundary with
focused tests. It owns every watcher task, channel, timer, and native or
portable resource; emits only the accepted normalized relevant signal; applies
the accepted bounded coalescing policy; terminates through Runtime cancellation;
and proves ignored changes, burst behavior, failure propagation, cleanup, and
fresh repetition. It does not rebuild or publish semantic snapshots.

Task 4 connects the committed Task 3 boundary to the existing complete
`WorkspaceSnapshotBuilder`. It serializes rebuilds, prevents partial
publication, atomically replaces only complete valid immutable snapshots,
implements exactly the accepted failure/recovery policy, preserves graph-query
single-snapshot observation and lifecycle-derived health, and adds focused
orchestration evidence. It does not add watch-control routes, persistence,
incremental semantic mutation, or new graph facts.

Task 5 exercises the public production boundary through real filesystem
observation, production EDT and Designer XML builders, Workspace publication,
Graph Query API observation, and Runtime shutdown. It proves relevant and
irrelevant changes, add/modify/remove/rename-equivalent transitions, burst
coalescing, invalid build and recovery, atomic visibility, deterministic
ordering, cancellation, resource cleanup, and equal fresh runs with tracked
provenance-backed inputs and event acknowledgements rather than arbitrary
sleeps. It synchronizes README, Architecture, and Semantic Model current-state
text but does not complete the sprint.

Task 6 reviews the planning-through-Task-5 range and creates
`docs/reviews/sprint-19-file-watching.md` only for `pass` or `pass with
non-blocking follow-ups` after all focused and full validation succeeds. That
decision transitions Sprint 19 to `completed`, makes Sprint 20 Persistent Cache
the unique `next` target, and atomically retires the exact Sprint 18 suite in
the same review commit. An inventory mismatch, endangered untracked file, or
retained link dependency blocks retirement and the final commit. The review
never silently fixes code.

##### Sprint 19 state gates and completion criteria

Sprint 19 remains `next` during planning and becomes `active` only after the
planning commit and Task 1 start. `already_complete` requires current committed
evidence and successful required validation; no empty commit is created. Stop
after the first prerequisite, implementation, validation, staging, commit, or
review failure. Do not skip, reorder, combine, or partially commit tasks.

Completion requires committed or proven Tasks 1-5, accepted ADR-0041, one
owned change-observation authority, deterministic relevant-change coalescing,
serialized complete rebuilds, atomic valid snapshot replacement, explicit
failure and recovery behavior, truthful lifecycle and health behavior, stable
Graph Query observation, public cross-platform production evidence, preserved
canonical graph and adapter semantics, explicit deferred scope, the complete
workspace gate, and a non-blocking Task 6 decision. A blocked review keeps
Sprint 19 incomplete, preserves the Sprint 18 suite, and leaves Sprint 20
ineligible.

The [Sprint 19 integration review](reviews/sprint-19-file-watching.md) records
`pass` against corrective evidence head
`cadc4b9f3e20e4da94df3ec91223c98f60255385`. Sprint 19 is `completed`, Sprint
20 Persistent Cache is the unique `next` target, and the exact verified Sprint
18 prompt suite is retired in the atomic review commit.

Planning validation covers Markdown links and structure, prompt numbering,
manifest/prerequisite/commit-message agreement, accepted-versus-deferred scope,
unchanged `next` state, verified previous-suite inventory, `git diff --check`,
and unrelated-change absence. Suggested planning commit message:

```text
Plan Sprint 19 File Watching
```

#### Sprint 20 Persistent Cache execution plan

Sprint 20 is planned from committed readiness head
`9694d3e81f20376660ce67f1d64d002ffbabe92b`. The Sprint 19 integration
review records `pass`, Sprint 20 is the unique `next` target, the working tree
is clean, and the committed Persistent State Profile, Workflow, and Template
close the required Codex Framework readiness stage. The tracked Runtime
Workspace fixture, disposable temporary copies, complete-byte source state,
canonical graph validation and Diff, public Graph Query observations, and
macOS/Windows CI provide repository-owned codec, invalidation, corruption,
recovery, lifecycle, and clean-rebuild test oracles.

No accepted decision currently defines the persisted owner, complete payload,
schema/version vocabulary, cache identity, invalidation inputs, filesystem
location, replacement safety, compatibility, migration-by-rebuild behavior,
corruption classification, recovery, Runtime integration, or observability.
Investigation and ADR acceptance therefore precede implementation. Planning
selects no format, fingerprint, checksum, storage path, replacement primitive,
or new production dependency. Adding any production dependency remains gated
on explicit user approval.

##### Objective

Persist complete validated Runtime Workspace semantic snapshots behind one
versioned source-neutral cache boundary so an exact valid entry can be restored
without adapter rebuilding, every incompatible, stale, corrupt, partial, or
unverifiable entry is contained deterministically, source or semantic-contract
changes trigger a clean rebuild, successful builds replace cache state safely,
and cache hits, misses, recovery, file watching, Graph Query, lifecycle, health,
shutdown, and repeated fresh runs have public cross-platform evidence.

##### Included scope

- repository-backed investigation of canonical snapshot content, graph
  reconstruction and validation, source-state identity, Runtime configuration
  and lifecycle, file-watcher relevance, filesystem behavior, dependencies,
  fixtures, consumers, compatibility, and deterministic test seams;
- an accepted ADR defining semantic authority, cache ownership, complete
  persisted envelope/payload, schema and semantic-build versions, encoding,
  ordering, identity, invalidation, load/write/replacement, corruption,
  compatibility, migration-by-clean-rebuild, recovery, lifecycle,
  observability, dependency, first-slice, and deferred-scope contracts;
- a versioned source-neutral Workspace snapshot codec that round-trips every
  accepted graph/payload/provenance/diagnostic/reference/report observation,
  reconstructs through public invariants, validates before use, and rejects
  malformed, partial, incompatible, or semantically invalid bytes;
- deterministic cache storage and invalidation using the accepted complete
  source-state and semantic-contract inputs, contained paths, safe complete
  replacement, typed load/write outcomes, cleanup, and focused failure evidence;
- Runtime startup and rebuild integration that publishes only a validated exact
  hit or a complete clean build, persists successful initial and replacement
  snapshots, preserves watcher coalescing and failure recovery, and never makes
  cache state a second readiness or semantic authority;
- public production evidence over tracked EDT and Designer XML inputs for cold
  miss/write, warm hit without adapter rebuilding, source/contract invalidation,
  corruption/incompatibility, write failure, clean-rebuild recovery, Graph Query
  equivalence, file-change replacement, shutdown cleanup, and fresh repetition;
- current-state documentation, complete workspace validation, integration
  review, and conditional Sprint 19 prompt-suite retirement.

##### Excluded scope

- new graph facts, node/edge kinds, source parsing, adapter semantics, Coverage
  transitions, public Graph Query or health wire changes, and mutable canonical
  semantic authority;
- incremental graph/index persistence, partial configuration publication,
  per-file semantic repair, cache-based authorization, cross-process writers or
  locking, remote/shared caches, compression, encryption, eviction, size or age
  policy, user-facing cache management, supported CLI behavior owned by Sprint
  21, and automatic migration from a historical schema that never existed;
- stable cache-file compatibility beyond the accepted schema/version contract,
  native filesystem notifications, Git/network workspaces, edit transactions,
  dynamic configuration, restart, forced termination, authentication,
  authorization, metrics/tracing export, benchmarks, and unsupported performance
  or security claims.

##### Sprint 20 prerequisite and retirement gate

Task 1 requires one committed Sprint 20 planning baseline containing this plan
and the complete suite under
`docs/codex/prompts/sprint-20-persistent-cache/`. Every dependent task requires
the preceding committed outcome. Stored prompts do not authorize commits;
authorization comes only from the launching user instruction.

The immediately preceding suite is exactly
`docs/codex/prompts/sprint-19-file-watching/`, with these seven tracked files:
`00-sprint-19-execution-loop.md`,
`01-investigate-file-watching-boundary.md`,
`02-define-file-watching-contract.md`,
`03-implement-file-change-watching.md`,
`04-integrate-workspace-rebuilds.md`,
`05-complete-file-watching-evidence.md`, and
`06-sprint-19-integration-review.md`. The tracked and filesystem inventories
match and contain no untracked file at planning time. The suite remains
untouched through Task 6. Only Task 7 may retire this exact inventory after a
non-blocking review and successful complete validation.

##### Ordered task manifest

| Order | Task | Profile / template | Owned outcome | Required committed prerequisite | Suggested commit message |
|---:|---|---|---|---|---|
| 1 | Investigate the Persistent Cache boundary. | Investigation / investigation | Current snapshot, graph reconstruction/validation, source identity, Runtime lifecycle, filesystem, dependency, fixture, consumer, compatibility, and deterministic-test evidence with explicit ADR questions. | Accepted Sprint 20 planning baseline. | `Investigate Sprint 20 Persistent Cache` |
| 2 | Define the Persistent Cache contract. | Architecture / architecture | Accepted ADR-0042 authority, schema, payload, identity, invalidation, storage, compatibility, corruption, recovery, lifecycle, dependency, first-slice, and deferred-scope contract. | Task 1. | `Define Sprint 20 Persistent Cache contract` |
| 3 | Implement the Workspace snapshot cache codec. | Persistent State / persistent state | Accepted versioned complete snapshot encoding/decoding, invariant reconstruction, validation, deterministic bytes, and focused round-trip/rejection evidence. | Task 2. | `Implement Sprint 20 snapshot cache codec` |
| 4 | Implement cache storage and deterministic invalidation. | Persistent State / persistent state | Accepted source/semantic validity identity, contained storage, complete replacement, typed hit/miss/rejection/write behavior, cleanup, and focused failure/recovery evidence. | Task 3. | `Implement Sprint 20 cache storage and invalidation` |
| 5 | Integrate the Runtime cache lifecycle. | Persistent State plus Runtime Service / persistent state | Startup hit/miss/build/write orchestration, successful rebuild persistence, watcher compatibility, validated-only publication, lifecycle preservation, and focused integration evidence. | Task 4. | `Integrate Sprint 20 Runtime cache lifecycle` |
| 6 | Complete public Persistent Cache evidence. | Persistent State plus Runtime Service / persistent state | Public cold/warm/invalidation/corruption/write-failure/recovery/watch/query/shutdown/repetition matrix and synchronized current-state docs. | Task 5. | `Complete Sprint 20 Persistent Cache evidence` |
| 7 | Review the integrated Sprint 20 baseline. | Review / review | Findings, complete validation, sprint decision, Sprint 19 suite retirement, and Sprint 21 hand-off. | Task 6 and all implementation validation. | `Complete Sprint 20 Persistent Cache review` |

```text
Committed Sprint 20 planning baseline
    -> Task 1 Persistent Cache investigation
    -> Task 2 accepted ADR-0042
    -> Task 3 versioned complete snapshot codec
    -> Task 4 deterministic storage and invalidation
    -> Task 5 Runtime cache lifecycle integration
    -> Task 6 public production evidence and current-state docs
    -> Task 7 integration review and conditional Sprint 19 suite retirement
    -> Sprint 21 planning eligibility
```

##### Task contracts

Task 1 creates only
`docs/architecture/persistent-cache-investigation.md`. It records exact
canonical snapshot content and construction invariants, graph/payload/
provenance/diagnostic/request/report reconstruction surfaces, validation and
clean-rebuild equivalence, complete-byte source identity, Runtime configuration
and service ownership, watcher interactions, filesystem/dependency/platform
constraints, fixture provenance, consumers, deterministic failure seams, and
every decision ADR-0042 must make. It selects no format, identity algorithm,
storage path, migration policy, or production dependency and changes no
production behavior.

Task 2 creates `docs/adr/0042-persistent-cache.md` and synchronizes only
planning-level architecture text required by the decision. It fixes semantic
authority and ownership, exact complete persisted content, schema/build version
vocabulary, encoding and ordering, cache identity and invalidation inputs,
path/storage/replacement behavior, hit/miss and typed failure classifications,
compatibility, clean-rebuild migration, corruption containment, recovery,
Runtime startup/rebuild/lifecycle/health/observer behavior, deterministic
testing, dependency policy, first slice, rejected alternatives, and deferred
scope. It implements no production behavior. If the accepted contract requires
a new production dependency, Task 3 cannot change a manifest until explicit
approval is available.

Task 3 implements only the ADR-0042 complete Workspace snapshot codec with
focused tests. It preserves every accepted configuration record, graph node,
payload, edge, provenance, diagnostic, reference request/statistic, and report
observation; reconstructs canonical domain values through checked APIs; rejects
unknown versions, malformed/partial content, duplicate identities, invalid
payloads/edges, invalid graphs, and inconsistent reports; and proves stable
round-trip bytes and clean-build equivalence. It performs no filesystem I/O or
Runtime publication.

Task 4 implements only the accepted cache identity and filesystem store. It
derives validity from complete accepted source and semantic-contract inputs,
contains all paths under the accepted owner, reads only exact candidate entries,
classifies missing/incompatible/corrupt/unreadable state, writes a complete
validated entry through the accepted safe replacement procedure, handles
interruption/failure deterministically, and proves invalidation, cleanup,
recovery, and fresh repetition. It does not change Runtime startup, publication,
watching, health, Graph Query, or current-state documentation.

Task 5 connects the committed codec/store to the existing `WorkspaceService`.
It closes source-scan/load/build/write races exactly as ADR-0042 accepts,
publishes only a validated exact cache hit or complete clean build, writes after
successful initial and replacement builds, preserves the last valid snapshot
across recoverable cache/write/rebuild failures as accepted, keeps watcher
coalescing and Graph Query single-snapshot observation stable, and preserves
lifecycle-derived health and shutdown cleanup. It adds focused orchestration
evidence but no HTTP or CLI cache surface.

Task 6 exercises the public production boundary through tracked temporary EDT
and Designer XML inputs, production source observation, cache load/write,
Workspace publication, File Watching rebuilds, Graph Query API, and Runtime
shutdown. It proves cold miss/write, warm hit without semantic adapter rebuild,
source and semantic-version invalidation, incompatible/corrupt/partial entry
handling, write failure, clean-build recovery, exact graph/evidence/query
equivalence, cache replacement after change, cleanup, and equal fresh runs. It
synchronizes README, Architecture, and Semantic Model current-state text but
does not complete the sprint.

Task 7 reviews the planning-through-Task-6 range and creates
`docs/reviews/sprint-20-persistent-cache.md` only for `pass` or `pass with
non-blocking follow-ups` after all focused and full validation succeeds. That
decision transitions Sprint 20 to `completed`, makes Sprint 21 CLI Client the
unique `next` target, and atomically retires the exact Sprint 19 suite in the
same review commit. An inventory mismatch, endangered untracked file, or
retained link dependency blocks retirement and the final commit. The review
never silently fixes code.

##### Sprint 20 state gates and completion criteria

Sprint 20 remains `next` during planning and becomes `active` only after the
planning commit and Task 1 start. `already_complete` requires current committed
evidence and successful required validation; no empty commit is created. Stop
after the first prerequisite, implementation, validation, staging, commit, or
review failure. Do not skip, reorder, combine, or partially commit tasks.

Completion requires committed or proven Tasks 1-6, accepted ADR-0042, one
versioned source-neutral persisted-state authority below canonical
`WorkspaceSnapshot`, complete deterministic encoding and validated
reconstruction, exact validity and invalidation inputs, safe complete storage
replacement, typed incompatibility/corruption/recovery behavior, startup and
watch-rebuild integration, valid-hit and clean-build equivalence, public
cross-platform production evidence, preserved graph/adapter/query/health
semantics, explicit deferred scope, the complete workspace gate, and a
non-blocking Task 7 decision. A blocked review keeps Sprint 20 incomplete,
preserves the Sprint 19 suite, and leaves Sprint 21 ineligible.

Planning validation covers Markdown links and structure, prompt numbering,
manifest/prerequisite/commit-message agreement, accepted-versus-deferred scope,
unchanged `next` state, verified previous-suite inventory, `git diff --check`,
and unrelated-change absence. Suggested planning commit message:

```text
Plan Sprint 20 Persistent Cache
```

##### Sprint 20 completed state

Tasks 1-6 are committed in dependency order. The
[Sprint 20 integration review](reviews/sprint-20-persistent-cache.md) records
`pass` after the focused Runtime matrix and complete workspace gate. Sprint 20
is `completed`, Sprint 21 CLI Client is the unique `next` target, and the exact
verified Sprint 19 prompt suite is retired in the review commit. ADR-0042,
canonical semantic authority, complete validated snapshot caching, deterministic
invalidation, Runtime/File Watching/Graph Query/health compatibility, public
mixed EDT/Designer evidence, and deferred-scope boundaries remain authoritative.

#### Sprint 21 CLI Client execution plan

Sprint 21 is planned from committed readiness head
`45c4473365a026f2acb83b4a6e9db0d8b2dbe2fb`. The
[Sprint 20 integration review](reviews/sprint-20-persistent-cache.md) records
`pass`, Sprint 21 is the unique `next` target, and the working tree is clean.
The existing Runtime Service Profile, Workflow, and Template cover supported
client work; no Codex Framework change is required.

The tracked Runtime production fixture, exact health and Graph Query HTTP/1.1
contracts, raw-loopback test helpers, public Runtime construction seams, and
macOS/Windows CI provide repository-owned command, request, response, failure,
cleanup, and repetition oracles. The first slice can remain dependency-free in
production by treating accepted Runtime JSON as an opaque validated HTTP body;
planning does not approve a new production dependency.

No accepted decision currently defines CLI command syntax, endpoint discovery,
HTTP client ownership, response presentation, exit codes, local validation,
transport failures, help/version behavior, or public client test boundary.
Investigation and ADR acceptance therefore precede implementation.

##### Objective

Replace the `oneagent-cli` placeholder with the first supported, deterministic
client for the accepted Runtime health, Workspace configuration-listing, exact
node, direct-relation, and bounded-traversal operations, preserving every
Sprint 16-20 lifecycle, HTTP, graph, watcher, cache, and semantic contract.

##### Included scope

- repository-backed investigation of the CLI placeholder, Runtime endpoint and
  configuration contracts, HTTP/1.1 client feasibility, dependencies, command
  conventions, output/error streams, exit status, fixtures, public process and
  library entry points, supported platforms, and deterministic test seams;
- an accepted ADR defining CLI ownership, command and option grammar, endpoint
  selection, request encoding, response handling, output and error contracts,
  exit codes, failure classification, resource lifecycle, compatibility,
  dependency policy, first slice, and deferred scope;
- a dependency-free command/configuration boundary with deterministic parsing,
  validation, help/version output, request construction, diagnostics, and
  process exit behavior;
- a bounded synchronous HTTP/1.1 client for the accepted health and `/api/v1`
  GET routes, exact query encoding, response framing, JSON media-type checks,
  success/error passthrough, transport failures, connection cleanup, and
  repeated calls;
- public integration evidence through the real CLI boundary and production
  Runtime over tracked EDT and Designer XML inputs, including all commands,
  invalid invocations, server errors, unavailable transport, deterministic
  output, shutdown, listener release, and repeated fresh runs;
- current-state documentation, complete workspace validation, integration
  review, and conditional Sprint 20 prompt-suite retirement.

##### Excluded scope

- changes to Runtime health or Graph Query routes, methods, parameters, bounds,
  JSON schemas, statuses, readiness, snapshot consistency, watcher/cache
  behavior, graph facts, parsers, adapters, or Coverage state;
- starting, stopping, supervising, discovering, or configuring the Runtime
  process; Workspace mutation/open/edit commands; cache management; watch
  subscriptions; streaming; pagination; arbitrary queries; response reformatting
  or semantic interpretation; shell completion; configuration files or
  environment variables; and activation of `oneagent-protocol`;
- DNS, URLs, proxies, redirects, authentication, authorization, TLS, HTTP/2,
  retries, configurable timeouts, cancellation, packaging/installers, telemetry,
  benchmarks, and unsupported performance or security claims;
- any new production dependency without separate explicit user approval, and
  v0.4 release review or Sprint 22 implementation.

##### Sprint 21 prerequisite and retirement gate

Task 1 requires one committed Sprint 21 planning baseline containing this plan
and the complete suite under `docs/codex/prompts/sprint-21-cli-client/`. Every
dependent task requires the preceding committed outcome. Stored prompts do not
authorize commits; authorization comes only from the launching user instruction.

The immediately preceding suite is exactly
`docs/codex/prompts/sprint-20-persistent-cache/`, with these eight tracked files:
`00-sprint-20-execution-loop.md`,
`01-investigate-persistent-cache-boundary.md`,
`02-define-persistent-cache-contract.md`,
`03-implement-snapshot-cache-codec.md`,
`04-implement-cache-storage-invalidation.md`,
`05-integrate-runtime-cache-lifecycle.md`,
`06-complete-persistent-cache-evidence.md`, and
`07-sprint-20-integration-review.md`. The tracked and filesystem inventories
match and contain no untracked file at planning time. The suite remains
untouched through Task 5. Only Task 6 may retire this exact inventory after a
non-blocking review and successful complete validation.

##### Ordered task manifest

| Order | Task | Profile / template | Owned outcome | Required committed prerequisite | Suggested commit message |
|---:|---|---|---|---|---|
| 1 | Investigate the CLI Client boundary. | Investigation / investigation | Current CLI, Runtime routes/configuration, command/output/error/exit expectations, HTTP feasibility, dependency, fixture, platform, consumer, and deterministic-test evidence with explicit ADR questions. | Accepted Sprint 21 planning baseline. | `Investigate Sprint 21 CLI Client` |
| 2 | Define the CLI Client contract. | Architecture / architecture | Accepted ADR-0043 ownership, grammar, endpoint, request, response, output, error, exit, resource, compatibility, dependency, first-slice, and deferred-scope contract. | Task 1. | `Define Sprint 21 CLI Client contract` |
| 3 | Implement the CLI command boundary. | Runtime Service / runtime service | Accepted command/option parsing, local validation, help/version, request model, diagnostics, output routing, and exit classification with focused evidence. | Task 2. | `Implement Sprint 21 CLI command boundary` |
| 4 | Implement the Runtime HTTP client. | Runtime Service / runtime service | Accepted bounded HTTP/1.1 connection, request encoding, response framing/validation, JSON passthrough, transport/server failure behavior, cleanup, and focused client evidence. | Task 3. | `Implement Sprint 21 Runtime HTTP client` |
| 5 | Complete public CLI Client evidence. | Runtime Service / runtime service | Public CLI-to-production-Runtime health/configuration/node/relation/traversal, failure, output, exit, shutdown, cleanup, repetition matrix and synchronized current-state docs. | Task 4. | `Complete Sprint 21 CLI Client evidence` |
| 6 | Review the integrated Sprint 21 baseline. | Review / review | Findings, complete validation, sprint decision, Sprint 20 suite retirement, and v0.4 release-review hand-off. | Task 5 and all implementation validation. | `Complete Sprint 21 CLI Client review` |

```text
Committed Sprint 21 planning baseline
    -> Task 1 CLI Client investigation
    -> Task 2 accepted ADR-0043
    -> Task 3 CLI command boundary
    -> Task 4 Runtime HTTP client
    -> Task 5 public production evidence and current-state docs
    -> Task 6 integration review and conditional Sprint 20 suite retirement
    -> v0.4 release review eligibility
```

##### Task contracts

Task 1 creates only `docs/architecture/cli-client-investigation.md`. It records
the exact placeholder and binary boundary, accepted Runtime routes and wire
contracts, endpoint configuration evidence, feasible dependency-free HTTP/1.1
behavior, argument/stream/exit conventions, public fixtures, process/library
test seams, consumers, platform constraints, and every decision ADR-0043 must
make. It selects no production contract and changes no production behavior.

Task 2 creates `docs/adr/0043-cli-client.md` and synchronizes only planning-level
architecture text required by the decision. It fixes ownership, command and
option grammar, endpoint input, request encoding, response framing and media
validation, stdout/stderr and exact exit behavior, local versus server versus
transport failures, resource cleanup, compatibility, dependency policy, first
slice, rejected alternatives, tests, and deferred scope. It implements no Rust.

Task 3 replaces the placeholder with only the accepted reusable command
boundary. It implements deterministic argument parsing, duplicate/unknown/
missing/value validation, help/version, typed requests, stable local diagnostics,
stdout/stderr routing, and exit classification, with focused tests. It performs
no network I/O and changes no Runtime contract.

Task 4 implements only the accepted Runtime HTTP client and connects it to the
Task 3 command boundary. It opens bounded client-owned connections, percent-
encodes accepted query values, sends exact GET requests, parses accepted
HTTP/1.1 framing, validates response status/media/body boundaries, forwards
exact server JSON to the accepted stream, classifies transport/protocol/server
failures, closes resources, and proves repeated calls. It adds no Runtime route,
protocol authority, semantic interpretation, or production dependency.

Task 5 exercises the public CLI boundary against the real production Runtime
and tracked temporary EDT/Designer inputs. It proves every accepted health,
configuration, node, relation, traversal, invalid invocation, server-domain,
not-ready/unavailable, unreachable/malformed-response, output/exit, shutdown,
cleanup, ordering, and repeated-run row without arbitrary sleeps or external
services. It synchronizes README, Architecture, and Semantic Model current-state
text but does not complete the sprint.

Task 6 reviews the planning-through-Task-5 range and creates
`docs/reviews/sprint-21-cli-client.md` only for `pass` or `pass with non-blocking
follow-ups` after all focused and full validation succeeds. That decision
transitions Sprint 21 to `completed`, makes the v0.4 release integration review
eligible, and atomically retires the exact Sprint 20 suite in the same review
commit. The review never silently fixes code.

##### Sprint 21 state gates and completion criteria

Sprint 21 remains `next` during planning and becomes `active` only after the
planning commit and Task 1 start. `already_complete` requires committed live
evidence and successful required validation; no empty commit is created. Stop
after the first prerequisite, implementation, validation, staging, commit, or
review failure. Do not skip, reorder, combine, or partially commit tasks.

Completion requires committed or proven Tasks 1-5, accepted ADR-0043, one
supported deterministic CLI command boundary, exact client compatibility with
every accepted health and Graph Query operation, closed local/server/transport
failure and exit behavior, dependency and resource ownership, public mixed
EDT/Designer production evidence, preserved Runtime/Workspace/query/watcher/
cache/semantic behavior, explicit deferred scope, the complete workspace gate,
and a non-blocking Task 6 decision. A blocked review keeps Sprint 21 incomplete,
preserves the Sprint 20 suite, and leaves the v0.4 review ineligible.

Planning validation covers Markdown links and structure, prompt numbering,
manifest/prerequisite/commit-message agreement, accepted-versus-deferred scope,
unchanged `next` state, verified previous-suite inventory, `git diff --check`,
and unrelated-change absence. Suggested planning commit message:

```text
Plan Sprint 21 CLI Client
```

##### Sprint 21 completed state

Tasks 1-5 are committed in dependency order. The
[Sprint 21 integration review](reviews/sprint-21-cli-client.md) records `pass`
after the focused CLI/Runtime matrix and complete workspace gate. Sprint 21 is
`completed`, the v0.4 release integration review is the unique next gate, and
the exact verified Sprint 20 prompt suite is retired in the review commit.
ADR-0043, Runtime and semantic authority, exact CLI command/request/output/exit
contracts, public mixed EDT/Designer executable evidence, compatibility with
health, Graph Query, Workspace, File Watching, and Persistent Cache behavior,
and deferred-scope boundaries remain authoritative.

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
