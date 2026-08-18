# OneAgent Roadmap

## Vision

**OneAgent** — кроссплатформенная платформа интеллектуальной разработки для `1C:Enterprise`, основанная на семантической модели конфигурации, графе знаний и интеграции с локальными и облачными LLM.

## Product roadmap

```mermaid
gantt
    title OneAgent Roadmap
    dateFormat  YYYY-MM-DD
    axisFormat  %b %Y

    section v0.1 Foundation
    Cargo workspace and quality gates       :done, f1, 2026-07-01, 7d
    Runtime foundation                      :done, f2, after f1, 5d
    Workspace discovery                     :done, f3, after f2, 5d
    EDT configuration reader                :done, f4, after f3, 5d
    Metadata domain model                   :done, f5, after f4, 5d

    section v0.2 Semantic Core
    Typed semantic graph                    :done, s1, 2026-07-20, 7d
    EDT metadata object descriptors         :done, s2, after s1, 5d
    EDT module nodes                        :done, s3, after s2, 5d
    BSL declaration extraction              :done, s4, after s3, 5d
    Local call extraction and resolution    :done, s5, after s4, 8d
    Cross-module call resolution            :done, s6, after s5, 8d
    Semantic index                          :s7, after s6, 12d
    Incremental indexing                    :s8, after s7, 12d

    section v0.3 1C Knowledge Model
    Attributes and tabular sections         :k1, 2026-09-01, 10d
    Forms and commands                      :k2, after k1, 12d
    Registers and queries                   :k3, after k2, 15d
    Roles and access rights                 :k4, after k3, 12d
    Subsystems and composition              :k5, after k4, 8d
    Event subscriptions                     :k6, after k5, 8d
    SKD and report model                    :k7, after k6, 15d
    XDTO and service model                  :k8, after k7, 12d

    section v0.4 Runtime API
    Runtime service container               :r1, 2026-11-15, 8d
    HTTP API and health endpoint            :r2, after r1, 8d
    Workspace service                       :r3, after r2, 10d
    Graph query API                         :r4, after r3, 12d
    File watching                           :r5, after r4, 10d
    Persistent cache                        :r6, after r5, 12d

    section v0.5 AI Integration
    Context engine                          :a1, 2027-01-15, 15d
    LLM provider abstraction                :a2, after a1, 10d
    OpenAI-compatible provider              :a3, after a2, 8d
    LM Studio integration                   :a4, after a3, 8d
    Ollama integration                      :a5, after a4, 8d
    Tool execution policy                   :a6, after a5, 12d

    section v0.6 MCP and IDE
    MCP server                              :m1, 2027-03-15, 15d
    MCP semantic tools                      :m2, after m1, 15d
    VS Code extension foundation            :m3, after m2, 15d
    Navigation and symbol search            :m4, after m3, 12d
    AI chat and context panel               :m5, after m4, 12d
    EDT integration prototype               :m6, after m5, 20d
    External AI client compatibility        :m7, after m6, 12d

    section v0.7 Intelligence
    Diagnostics engine                      :i1, 2027-06-01, 15d
    Rules engine                            :i2, after i1, 15d
    Change impact analysis                  :i3, after i2, 15d
    Refactoring planner                     :i4, after i3, 20d
    Safe edit transactions                  :i5, after i4, 15d

    section v1.0
    Public API stabilization                :v1, 2027-09-01, 20d
    Plugin SDK                              :v2, after v1, 20d
    Performance and security hardening      :v3, after v2, 20d
    Documentation and examples              :v4, after v3, 15d
    OneAgent 1.0 release                    :milestone, v5, after v4, 1d
```

## Release goals

### v0.1 — Foundation
Stable Rust workspace, Runtime composition root, cross-platform CI, EDT workspace discovery, base metadata model.

### v0.2 — Semantic Core
Typed semantic graph, real EDT UUIDs, module/procedure/function nodes, local and cross-module call graph, semantic and incremental indexes.

### v0.3 — 1C Knowledge Model
Attributes, tabular sections, forms, commands, registers, queries, roles, access rights, subsystems, SKD, XDTO and services.

### v0.4 — Runtime API
Long-running runtime, workspace lifecycle, file watching, graph-query API and persistent cache.

### v0.5 — AI Integration
Context engine, LLM abstraction, LM Studio, Ollama and OpenAI-compatible endpoints.

### v0.6 — MCP and IDE
MCP server, VS Code extension, EDT integration, navigation and semantic tools.
OneAgent should expose its semantic graph, query, validation, diagnostics,
impact, and context capabilities through MCP so external AI clients such as
Codex, Cursor, and cloud agents can consume OneAgent without product-specific
core integrations.

### v0.7 — Intelligence
Diagnostics, impact analysis, planning, refactoring and safe edit transactions.

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

### Completed sprints

| Sprint | Version | Goal | Status |
|---|---|---|---|
| Sprint 1 — Foundation | v0.1 | Establish the Cargo workspace, quality gates, Runtime foundation, workspace discovery, EDT configuration reader, and metadata domain model. | completed |
| Sprint 2 — Semantic Core Foundation | v0.2 | Establish the typed semantic graph, EDT metadata and module nodes, BSL declaration extraction, and local and cross-module call resolution. | completed |
| Sprint 3 — Semantic Coverage | v0.2 | Audit and complete graph-domain and EDT semantic coverage, close all Critical, High, and Medium gaps, and complete the integration review. | completed |

### Current planning focus

| Sprint | Version | Goal | Status |
|---|---|---|---|
| Sprint 4 — Semantic Index | v0.2 | Build the semantic index over the completed graph and EDT semantic model. | next |

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

The v0.3 release integration review follows Sprint 13.

#### v0.4 — Runtime API

| Sprint | Goal | Status |
|---|---|---|
| Sprint 14 — Runtime Service Container | Establish the long-running runtime composition and service lifecycle. | planned |
| Sprint 15 — HTTP API and Health | Expose the runtime through an HTTP API with health and readiness behavior. | planned |
| Sprint 16 — Workspace Service | Add workspace lifecycle and semantic-build orchestration services. | planned |
| Sprint 17 — Graph Query API | Expose stable graph and semantic query capabilities through the runtime API. | planned |
| Sprint 18 — File Watching | Detect workspace changes and connect them to runtime update orchestration. | planned |
| Sprint 19 — Persistent Cache | Persist validated semantic state with deterministic invalidation. | planned |

The v0.4 release integration review follows Sprint 19.

#### v0.5 — AI Integration

| Sprint | Goal | Status |
|---|---|---|
| Sprint 20 — Context Engine | Build deterministic semantic context selection and assembly. | planned |
| Sprint 21 — LLM Provider Abstraction | Define provider-independent model, request, response, and capability contracts. | planned |
| Sprint 22 — OpenAI-Compatible Provider | Implement the first OpenAI-compatible provider integration. | planned |
| Sprint 23 — LM Studio Integration | Add local LM Studio discovery and execution support. | planned |
| Sprint 24 — Ollama Integration | Add local Ollama discovery and execution support. | planned |
| Sprint 25 — Tool Execution Policy | Define and enforce safe AI tool execution boundaries. | planned |

The v0.5 release integration review follows Sprint 25.

#### v0.6 — MCP and IDE

| Sprint | Goal | Status |
|---|---|---|
| Sprint 26 — MCP Server | Establish the MCP server, lifecycle, and transport boundary. | planned |
| Sprint 27 — MCP Semantic Tools | Expose graph, query, validation, diagnostics, impact, and context capabilities through MCP. | planned |
| Sprint 28 — VS Code Extension Foundation | Establish extension packaging, activation, configuration, and runtime connectivity. | planned |
| Sprint 29 — Navigation and Symbol Search | Add semantic navigation and symbol-search experiences. | planned |
| Sprint 30 — AI Chat and Context Panel | Add IDE chat and inspectable semantic context UI. | planned |
| Sprint 31 — EDT Integration Prototype | Prove the EDT integration boundary and user workflow. | planned |
| Sprint 32 — External AI Client Compatibility | Validate Codex, Cursor, and other MCP-capable client workflows. | planned |

The v0.6 release integration review follows Sprint 32.

#### v0.7 — Intelligence

| Sprint | Goal | Status |
|---|---|---|
| Sprint 33 — Diagnostics Engine | Build semantic diagnostic orchestration and reporting. | planned |
| Sprint 34 — Rules Engine | Define deterministic rule registration, execution, and result contracts. | planned |
| Sprint 35 — Change Impact Analysis | Expand impact analysis into a product-facing workflow. | planned |
| Sprint 36 — Refactoring Planner | Produce validated semantic refactoring plans. | planned |
| Sprint 37 — Safe Edit Transactions | Apply planned edits through checked, reversible transactions. | planned |

The v0.7 release integration review follows Sprint 37.

#### v1.0 — Stable Platform

| Sprint | Goal | Status |
|---|---|---|
| Sprint 38 — Public API Stabilization | Stabilize supported public APIs and compatibility policy. | planned |
| Sprint 39 — Plugin SDK | Define and publish the supported extension SDK. | planned |
| Sprint 40 — Performance and Security Hardening | Complete profiling, performance, threat-model, and security hardening work. | planned |
| Sprint 41 — Documentation and Examples | Complete user, operator, contributor, and API documentation with examples. | planned |
| Sprint 42 — OneAgent 1.0 Release | Complete final release validation, packaging, and publication. | planned |

The v1.0 release integration review and release decision are part of Sprint 42.

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
