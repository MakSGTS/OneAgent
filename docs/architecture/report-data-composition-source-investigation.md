# Report Data Composition Source Investigation

## Purpose

This document records repository-owned EDT evidence for Sprint 12 SKD and
Report Model. It is a planning authority for source shape, identity, scope, and
testability. It does not change parser, graph, Coverage, or production behavior.

## Investigated baseline

The investigation was performed against committed Sprint 11 review head
`8b0d22ef955129d4bf6eb88549529a81baf9c466`. The real source corpus is under
`OneAgent_EDTproject/src/Reports/`. The repository root `.gitignore` excludes
`OneAgent_EDTproject/`, so the corpus is planning evidence rather than a tracked
test fixture. A Sprint 12 evidence task must derive a small tracked fixture with
explicit paths and hashes before Coverage changes.

The production EDT builder already discovers top-level Report metadata through
the universal `Reports` path and emits UUID-backed `MetadataKind::Report` nodes,
configuration ownership, common payload, members, modules, and existing
references. It does not preserve Report data-composition template declarations,
read `.dcs` artifacts, or emit Data Composition Schema, Data Set, Data
Composition Field, or metadata-owned Query nodes.

## Corpus inventory

The corpus contains 56 valid Report `.mdo` descriptors. All Report directory
names agree with the direct descriptor `name`, and all descriptor XML parses.

Fifty-two Reports declare 56 templates whose exact direct `templateType` is
`DataCompositionSchema`. Every declaration has a non-empty unique UUID and an
exact artifact at:

```text
Reports/<report>/Templates/<template-name>/Template.dcs
```

There are no missing or extra `.dcs` artifacts relative to those 56 accepted
declarations. Fifty-one Reports declare one exact
`mainDataCompositionSchema` value matching one of their declared schemas.
`FinancialReport` owns one non-main Data Composition Schema, while
`AnalyticalReportByCategories`, `SalesAnalytics`, `TransferOfProduct`, and
`WarehouseSchema` have no accepted schema declaration. Missing main-schema
selection is therefore valid and distinct from a missing declared artifact.

Every `.dcs` root is `DataCompositionSchema`. The 56 schemas contain this
first-slice direct inventory:

| Direct source construct | Count |
|---|---:|
| Data Composition Schema artifacts | 56 |
| Root `dataSource` declarations | 54 |
| Direct `dataSet` declarations | 70 |
| Direct `DataSetQuery` declarations | 38 |
| Direct `DataSetObject` declarations | 25 |
| Direct `DataSetUnion` declarations | 7 |
| Direct `DataSetFieldField` declarations | 970 |
| Direct `DataSetFieldFolder` declarations | 6 |
| Direct Query text elements | 38 |

The two schemas without a root data source or data set are the valid empty
schemas owned by `IncomeAndExpenseReportSpreadsheetDoc` and `UniversalReport`.
Every non-empty schema has one root `DataSource1` of exact type `Local`. Every
direct Query or Object data set references `DataSource1`; direct Union data sets
have no direct data-source reference.

Every direct data set has one non-empty name, and data-set names are unique
within their schema. Each accepted direct field has a non-empty `field` name
and `dataPath`; field names are unique within their direct owner data set.
Every direct Query data set has exactly one non-empty `query` element. These
facts provide stable owner-scoped identity inputs without using traversal order
or query text.

## Serialized source contract

Report template declaration shape:

```xml
<mdclass:Report uuid="9edf1ea5-8b8d-4359-b801-6fb5ca2f8009">
  <name>IncomeAndExpenseReport</name>
  <mainDataCompositionSchema>
    Report.IncomeAndExpenseReport.Template.MainDataCompositionSchema
  </mainDataCompositionSchema>
  <templates uuid="9dd3aa95-10e4-4729-bf1e-4eded30bfa15">
    <name>MainDataCompositionSchema</name>
    <templateType>DataCompositionSchema</templateType>
  </templates>
</mdclass:Report>
```

Representative direct schema content:

```xml
<DataCompositionSchema
    xmlns="http://v8.1c.ru/8.1/data-composition-system/schema"
    xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
  <dataSource>
    <name>DataSource1</name>
    <dataSourceType>Local</dataSourceType>
  </dataSource>
  <dataSet xsi:type="DataSetQuery">
    <name>DataSet1</name>
    <field xsi:type="DataSetFieldField">
      <dataPath>Product</dataPath>
      <field>Product</field>
    </field>
    <dataSource>DataSource1</dataSource>
    <query>SELECT ...</query>
  </dataSet>
</DataCompositionSchema>
```

The parser must validate namespaces and direct nesting rather than match local
names anywhere in the document. Other settings, parameters, calculated fields,
templates, appearances, roles, and localized presentation content exist but do
not participate in the first source-independent entity contract.

## Identity evidence

The Report descriptor supplies a unique UUID for each declared Data Composition
Schema. That UUID is the canonical schema identity and remains stable when the
schema file content, main-schema selection, data sets, or fields change.

Direct data sets and fields have no UUID. The corpus provides a stable local
name unique within the immediate accepted owner, so the first slice can derive:

```text
data-set id = (schema UUID, direct data-set name)
field id    = (data-set id, direct field name)
query id    = (data-set id, fixed query role)
```

Data-set kind, field data path, query text, source order, and filesystem order
are semantic content or provenance and never identity. Reordering unique direct
data sets or fields therefore preserves IDs and equal canonical output.

## Query compatibility audit

The complete corpus has 46 `DataSetQuery` declarations when nested Union
children are included. The existing `QueryLanguageParser` was run against the
exact query text of all 46 declarations. Zero queries satisfy its current
complete-source contract:

| Existing parser outcome | Count |
|---|---:|
| `VirtualTableSource` | 29 |
| `UnsupportedStructure` | 8 |
| `MalformedSyntax` for the current bounded grammar | 8 |
| `TemporaryTableSource` | 1 |

The first slice can safely emit metadata-owned Query entities for the 38 direct
Query data sets, preserving complete query text in deterministic provenance,
but it cannot emit `Reads`, query-origin `DependsOn`, QuerySource requests, or
partial source diagnostics from the current minimum parser. Doing so would
misrepresent incomplete source sets as complete semantic relations.

## Deferred source constructs

`ControlOfProductsAccounting` contains eight nested Query data sets inside one
direct Union. All eight use the same local name `DataSet1`, the same local data
source name, and have distinct query content. The source provides no UUID or
other stable local declaration key that survives child reordering and query
content changes. They cannot receive canonical first-slice identity without
using traversal order or semantic content. Nested Union child data sets are
therefore typed deferred observations, not emitted entities.

Six direct `DataSetFieldFolder` elements occur in two accounting-report Union
schemas. They are structural folders with no direct `field` name, while the
accepted entity is a named `DataSetFieldField`. Folder semantics are typed
deferred observations and do not create empty-name or guessed field nodes.

Other deferred constructs include nested data sources, parameters, calculated
fields outside the accepted direct field element, settings variants, field
folders, field roles, layouts, templates, expressions, totals, filters, runtime
settings, and general query-language analysis.

## Representative production evidence

The tracked reduced fixture should cover at least these live shapes:

| Live source | Required evidence |
|---|---|
| `Reports/AccessGroupsMembers/` | main Query schema, direct Query data set, named fields, query text |
| `Reports/VolumeIntegrityCheck/` | main Object schema and fields |
| `Reports/AccountCardFinancialAccounting/` | direct Union schema and fields |
| `Reports/ControlOfProductsAccounting/` | typed deferred nested duplicate-name Union children |
| `Reports/UniversalReport/` | valid empty main schema |
| `Reports/FinancialReport/` | valid non-main schema without a main selection |

The fixture README must record exact source paths, source hashes, reduction
treatment, and reduced-artifact hashes. Generated mutations may cover missing,
duplicate, mismatched, malformed, reordered, and repeated-build cases but must
not replace the positive provenance-backed corpus evidence.

## Testability gate

The repository evidence is sufficient to plan and test the bounded first slice:

- descriptor UUID, template name/type, optional main selection, artifact path,
  DCS root, data-source vocabulary, data-set kinds/names, direct field names and
  paths, query presence, and deferred nested shapes are directly observable;
- canonical identity inputs exist for every accepted entity;
- missing artifact, extra artifact, UUID/name mismatch, duplicate direct name,
  unsupported kind, folder, nested Union child, malformed XML, wrong root,
  empty value, reordered declaration, and repeated-build outcomes are testable;
- the universal metadata reader, graph validation, Query, Diff, Impact policy,
  reports, complete index, incremental index, and Coverage registries are
  discoverable consumers;
- the canonical full workspace validation matrix is known.

## Architecture questions resolved by ADR-0034

The accepted architecture must define:

- Schema, Data Set, Data Composition Field, and metadata-owned Query node kinds;
- UUID and owner-scoped identities;
- Report, Schema, Data Set, Field, and Query ownership directions;
- typed Schema/Data Set/Field content and main-schema selection;
- fatal descriptor/artifact failures versus typed deferred constructs;
- provenance, deterministic ordering, Query, Diff, Impact, reports, validation,
  complete/incremental index, and Coverage behavior;
- the explicit boundary excluding query-source resolution and nested Union
  child identities.

## Framework readiness decision

The existing graph-model, parser, graph-emission, review, sprint-planning, and
sequential-execution contracts express the required source, identity,
validation, safety, Coverage, and reporting gates. The Roadmap forecasts no new
task-template family before Sprint 14, and this investigation found no concrete
Sprint 12 framework gap. No Codex Framework change or post-sprint framework
audit is justified.
