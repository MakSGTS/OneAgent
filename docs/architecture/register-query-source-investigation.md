# Sprint 8 Register and Query Source Investigation

## Status and purpose

This document records the repository-owned evidence used to plan Sprint 8 —
Registers and Queries. It separates Query declarations, query-language data
sources, register virtual tables, register metadata, and semantic relations so
that one term does not silently broaden another.

This is planning evidence, not production support. It changes no parser,
resolver, graph, Coverage status, test, or fixture.

Evidence labels are:

- **Confirmed** — present in current code, tests, fixtures, Git history, or a
  cited real EDT artifact;
- **Accepted** — required by an accepted ADR or current architecture document;
- **Probable** — suggested by repository evidence but not sufficient for a
  production contract;
- **Unknown** — not established by repository-owned evidence.

## Planning baseline

The investigation was performed at
`7d5a0e6f46b8cee0737f679abf3b712de967ab8a`, after the passing Sprint 7
integration review. Sprint 8 is the next planning target. The only pre-existing
working-tree entry was the unrelated untracked
`docs/roadmap-calendar-forecast.md`; it is outside this planning change.

The current implementation is internally complete for its accepted first
slices:

- static named BSL Query bindings inside known Procedures or Functions produce
  stable `NodeKind::Query` nodes owned through `Contains`;
- the query-language parser accepts one completely consumed `SELECT` with one
  direct Catalog or Information Register source and optional supported alias;
- rejected joins, unions, nested queries, batches, temporary tables, virtual
  tables, parameter sources, malformed text, and unconsumed tails produce
  typed no-edge outcomes;
- the private EDT resolver performs deterministic case-normalized exact-kind
  resolution after top-level metadata collection;
- unique resolution emits provenance-backed `Reads`; failures emit typed
  diagnostics without placeholder targets;
- the first Document register-record `Writes` slice emits only
  `Procedure --Writes--> Metadata(AccumulationRegister)`;
- metadata type references and Command parameter types already use the public
  reference-request lifecycle and may emit normalized `DependsOn`;
- query sources still use private resolution outcomes and legacy statistics;
- query-derived and write-derived `DependsOn` are not emitted;
- graph-domain and EDT registries have no Critical, High, or Medium gaps, so
  Sprint 8 is an explicit supported-boundary expansion rather than gap closure.

## Terminology boundary

| Term | Meaning in this plan | Current state |
|---|---|---|
| Query declaration source | Artifact and local declaration that creates one stable Query entity | Static named BSL bindings only |
| Query-language data source | Qualified persistent source named inside a parsed query program | Direct Catalog and Information Register only |
| Register virtual table | Third-component or invoked source such as `AccumulationRegister.X.Balance(...)` | Recognized only for deterministic rejection |
| Register metadata object | Top-level Information, Accumulation, Accounting, or Calculation Register entity and its accepted members | All four kinds are modeled; real corpus coverage differs by family |
| `Reads` | Direct resolved access from Query to persistent metadata | Supported first slice |
| `Writes` | Direct resolved persistent mutation from an accepted Procedure | Supported first slice |
| `DependsOn` | Materialized normalized direct dependency retaining the proving lower-level fact | Metadata type and Command parameter origins implemented; Query origin deferred |

“Additional Query sources” is therefore not treated as permission to add new
Query declaration families, virtual-table semantics, or arbitrary query
grammar.

## Query declaration source evidence

### Confirmed current source

`crates/bsl/src/queries.rs` and `adapters/edt/src/bsl_graph.rs` accept a stable
named local Query binding with complete static text in a supported constructor
or `.Text` literal assignment. The nearest known Procedure or Function owns the
Query. Query identity is independent from query text.

The multiline decoder and its private decoded-to-BSL source map are implemented
and documented by `query-language-parser-investigation.md`. This source family
is sufficient for the selected Sprint 8 data-source expansion.

### Deferred declaration families

The following remain **Accepted future categories** in Semantic Model 2.0 but
are not implementation-ready here:

- module-scope BSL Query declarations;
- runtime-concatenated, reassigned, or externally loaded text;
- metadata-owned data-composition datasets;
- dynamic-list query settings;
- standalone query artifacts and external resources.

Their EDT representation, stable local identity, ownership, duplicate policy,
and provenance are either **Unknown** or owned by later Report/Form work.
Sprint 8 does not add a Query node source family.

## Direct register query-source evidence

### Accumulation Register

The real Common Module query at
`OneAgent_EDTproject/src/CommonModules/Accounting/Module.bsl:936-970` contains
a static `Query.Text` assignment inside Procedure `InventoryCostBeforeWrite`.
Its direct source is:

```text
AccumulationRegister.InventoryCost AS OldRecords
```

The module is declared by
`CommonModule.Accounting` in
`OneAgent_EDTproject/src/Configuration/Configuration.mdo`. The persistent
target is declared by
`AccumulationRegister.InventoryCost`, and its descriptor is
`OneAgent_EDTproject/src/AccumulationRegisters/InventoryCost/InventoryCost.mdo`.

This is **Confirmed** evidence that a static Query declaration reads a direct
top-level Accumulation Register source with a stable target metadata object.

### Accounting Register

The real Common Module query at
`OneAgent_EDTproject/src/CommonModules/MonthEndTransactions/Module.bsl:1514-1541`
contains a static `Query.Text` assignment inside Procedure
`ARAPUpdateExecute`. Its direct source is:

```text
AccountingRegister.FinancialAccounting AS FinancialAccounting
```

The module and target are declared in the Configuration descriptor. The target
descriptor is
`OneAgent_EDTproject/src/AccountingRegisters/FinancialAccounting/FinancialAccounting.mdo`.

This is **Confirmed** evidence for a direct top-level Accounting Register
source with a stable target.

### Calculation Register

The inspected Configuration descriptor contains no Calculation Register
declaration, and focused searches found no repository-owned BSL query source
that can be joined to a top-level Calculation Register target. The generic EDT
kind exists, but parser taxonomy alone is not production evidence.

Calculation Register query access is therefore **Unknown for the current real
corpus** and deferred.

### Information Register

Information Register direct access is already implemented and Supported. It is
a compatibility baseline, not a Sprint 8 deliverable.

## Grammar and fixture boundary

The confirmed Accumulation and Accounting examples contain projections and
tails beyond the current minimum query-language grammar. Their direct source
spelling and persistent target relationship are real evidence, but their whole
programs are not evidence that the current parser can safely consume arbitrary
projection, `WHERE`, or `ORDER BY` grammar.

The selected slice therefore extends only the persistent namespace/category
allowlist. Production fixtures may reduce the confirmed source declarations to
the already accepted complete form:

```text
SELECT Alias.Field FROM Namespace.Name AS Alias
```

Every reduction must be labeled generated or reduced scaffolding and retain a
manifest mapping to the exact real source path, range, qualified name, and
target descriptor. It must not be described as a verbatim complete production
query.

The following remain deferred:

- projection lists and general expression grammar;
- `WHERE`, `GROUP BY`, `ORDER BY`, totals, and query-language functions beyond
  an independently proven complete grammar;
- JOIN, UNION, nesting, batches, and multiple statements;
- temporary, external, and parameter tables;
- comments and string-literal shielding beyond current evidence;
- Russian Accumulation or Accounting Register namespace spellings.

The all-or-nothing rule remains authoritative: an unconsumed or unsupported
program emits no `Reads` or query-derived `DependsOn`, even if one supported
source-looking fragment was seen.

## Register virtual tables

The corpus contains many examples such as:

```text
AccumulationRegister.RevenueAndCostOfProductSales.Turnovers(...)
AccumulationRegister.QuantitativeAccounting.BalanceAndTurnovers(...)
AccumulationRegister.ProductsInStorageBins.Balance(...)
```

These are **Confirmed source shapes** but not direct top-level source
equivalents. Parameters, virtual fields, totals semantics, and whether one
virtual table justifies a direct edge to its base register require a separate
contract. ADR-0021 explicitly rejected silently mapping virtual tables to base
registers.

Sprint 8 continues to classify them as typed `VirtualTableSource` no-edge
outcomes. No base-register `Reads` or `DependsOn` is inferred.

## Register metadata semantics

The live EDT pipeline already models top-level Information, Accumulation,
Accounting, and Calculation Register metadata kinds. It emits Dimension and
Resource children, maps Accounting Register resources to `NodeKind::Measure`,
preserves immediate ownership, and processes accepted member type references.

The two selected Sprint 8 additions use only the existing top-level metadata
nodes as query targets. They do not add:

- new register member nodes;
- register standard attributes;
- member payload, qualifiers, produced types, periodicity, or totals settings;
- field-level Reads or Writes;
- register-record runtime entities;
- new metadata identity or ownership rules.

Observed fields in register descriptors remain **Probable future knowledge
model inputs**, not accepted Sprint 8 semantics.

## Query-source request lifecycle

`SemanticReferenceCategory::QuerySource` already exists in the public graph
domain, and build reports, validation, and build Diff already understand the
generic request ledger. Current query-source parsing and resolution instead use
private `QuerySourceResolutionOutcome` values and independently updated legacy
statistics.

ADR-0024 requires each deferred family to define source node, target
representation, collection/resolver provenance, terminal projections,
duplicates, statistics compatibility, and production evidence before
migration. The current Query node and parsed source occurrence provide those
prerequisites:

- source node: existing `NodeKind::Query` identity;
- category: existing `SemanticReferenceCategory::QuerySource`;
- target: typed metadata name with one exact expected `NodeKind`;
- collection evidence: query artifact, owner, raw source spelling, category,
  and deterministic query-text location;
- resolver evidence: exact kind, normalized local name, workspace scope, and
  ordered candidates;
- direct projection: `Reads` on unique success;
- failure projection: current typed query-source diagnostic;
- statistics: derived once from terminal canonical requests.

Migration is therefore **decision-ready** without a new public category or
placeholder model.

## Query-derived dependency decision readiness

ADR-0017 reserves `Query --DependsOn--> Metadata(...)` for resolved static
query data-source access. ADR-0021 requires production `Reads` evidence before
accepting that normalized origin. That prerequisite is now complete.

The proving fact is one terminal resolved QuerySource request and its retained
`Reads` projection. The normalized direction is:

```text
Query --DependsOn--> resolved persistent metadata source
```

The edge is `FactOrigin::Derived`, `ResolutionState::Resolved`, and exact. It
uses the same canonical source and target identities as the proving `Reads`
edge but has independent edge identity through `EdgeKind::DependsOn`.
Equivalent evidence is sorted and deduplicated before insertion.

Missing, ambiguous, incompatible, partial, parser-rejected, virtual,
temporary, external, dynamic, or incomplete sources emit neither edge.
`Reads` is retained and remains the direct data-access fact.

Generic dependency queries will expose both direct `Reads` and normalized
`DependsOn` relations. Reverse Impact still returns unique affected Query nodes
but may retain deterministic reasons for both stored edges. This additive
observability is accepted and must be tested.

Write-derived `DependsOn` remains deferred. It has separate Procedure source
semantics and is not needed to complete the selected Query-focused dependency
slice.

## Consumer and compatibility inventory

| Surface | Required compatibility |
|---|---|
| Query identity and ownership | No change |
| Query-language parser | Add only two direct persistent categories; preserve current all-or-nothing grammar and diagnostics |
| Query-source resolver | Preserve normalization, collision precedence, workspace scope, and deterministic candidates while producing public requests |
| Validation | Add exact `Reads` and `DependsOn` target pairs; no wildcard Metadata target |
| Provenance | Preserve direct resolved Reads evidence; add collection/resolver request evidence and derived dependency evidence |
| Diagnostics/statistics | Preserve typed outcomes; derive terminal counts once without double counting |
| Query API | Existing generic relation APIs expose both stored facts; no dedicated adapter API |
| Diff and build Diff | Stable requests and edges participate through existing identity rules |
| Impact | Changed register reaches the Query through direct and normalized dependency reasons without duplicate affected nodes |
| Semantic Index | No new index dimension; complete and incremental results must equal a clean rebuild |
| Coverage | Existing Reads, DependsOn, and ReferenceRequest capabilities remain Supported; evidence is expanded without a status/count transition |

## Selected Sprint 8 slice

The narrowest coherent evidence-backed slice is:

1. extend direct query-language source classification and exact resolution to
   Accumulation and Accounting Registers;
2. migrate accepted query-source observations to the public QuerySource request
   lifecycle;
3. retain `Reads` and emit a normalized companion `DependsOn` for every uniquely
   resolved accepted Query source, including existing Catalog and Information
   Register targets;
4. prove the complete path through reduced provenance-backed EDT fixtures,
   negative outcomes, Query, Diff, Impact, reports, validation, request ledger,
   repeated builds, and complete/incremental index equivalence.

No new `NodeKind`, `EdgeKind`, metadata payload, Query identity, declaration
source, or public query API is required.

## Deferred and rejected scope

- Calculation Register query sources without real target/source evidence.
- Direct metadata families outside Catalog and the accepted register kinds.
- Register virtual tables and base-register inference.
- JOIN, UNION, nested queries, batches, multiple statements, temporary tables,
  and general expression grammar.
- New Query declaration source families.
- Query fields, parameters, result fields, and field-level edges.
- Query-language mutation and Query `Writes`.
- Expanded BSL object or register write forms.
- Write-derived `DependsOn`.
- New register metadata payload or member semantics.
- Placeholder, Unknown, external, or lower-confidence graph targets.
- New dependencies, serialization, Runtime, API transport, Designer XML, or
  later-sprint integrations.

## Codex Framework readiness

The existing graph implementation, parser implementation, general
implementation, and review profiles plus graph-model, parser, implementation,
graph-emission, review, sprint-planning, and sprint-execution templates cover
the selected work. No reusable Framework gap was found. No post-sprint
Framework audit task is justified by this readiness review.

## Readiness conclusion

The evidence is sufficient to accept the bounded ADR-0030 contract and plan a
strictly sequential six-task implementation and review. Architecture planning
does not change production behavior, Coverage status, aggregate counts, or
Sprint completion state.
