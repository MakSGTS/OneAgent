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
| v0.2 — Semantic Core | Typed semantic graph, semantic index, and deterministic incremental indexing | Sprints 2–5 | in progress |
| v0.3 — 1C Knowledge Model | Broader metadata semantics and Designer XML ingestion | Sprints 6–14 | planned |
| v0.4 — Runtime API | Long-running services, APIs, cache, and a usable CLI client | Sprints 15–21 | planned |
| v0.5 — AI Integration | Context engine and local or OpenAI-compatible LLM providers | Sprints 22–27 | planned |
| v0.6 — MCP and IDE | MCP, VS Code, LSP, EDT, and external AI client integrations | Sprints 28–35 | planned |
| v0.7 — Intelligence | Diagnostics, Git-aware change ingestion, impact, refactoring, and safe edits | Sprints 36–41 | planned |
| v1.0 — Stable Platform | Stable APIs, plugin SDK, hardening, documentation, and release | Sprints 42–46 | planned |

Calendar forecasts are intentionally kept outside this document until capacity,
scope, and release criteria are baselined. Adding a forecast must not duplicate
or override the dependency order recorded in the sprint tables.

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

### Current planning focus

| Sprint | Version | Goal | Status |
|---|---|---|---|
| Sprint 4 — Semantic Index | v0.2 | Build the derived semantic index defined by [ADR-0026](adr/0026-semantic-index-boundary.md) over the completed graph and EDT semantic model. | next |

Sprint 4 is the next dependency-ordered target. It becomes `active` only after
its architecture boundary, task decomposition, acceptance criteria, and
validation plan are approved.

### Planned sprints

#### v0.2 — Semantic Core

| Sprint | Goal | Status |
|---|---|---|
| Sprint 5 — Incremental Indexing | Update the semantic index deterministically from workspace changes without rebuilding unaffected state. | planned |

The v0.2 release integration review follows Sprint 5.

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
