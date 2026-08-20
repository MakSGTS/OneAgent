# ADR-0034: Report Data Composition Semantics

## Status

Accepted

## Context

Sprint 12 must add the first evidence-backed Report and Data Composition System
semantic model. OneAgent already discovers top-level EDT Reports and emits
their UUID-backed metadata nodes, members, modules, ownership, and existing
references. It does not read Report template declarations or `.dcs` artifacts,
and the graph has no Data Composition Schema, Data Set, or Data Composition
Field node kinds.

The repository-owned investigation in
`docs/architecture/report-data-composition-source-investigation.md` proves 56
valid Report descriptors, 56 UUID-backed Data Composition Schema declarations,
70 uniquely named direct data sets, 970 uniquely named direct fields, and 38
direct Query data sets. It also proves two source limits:

- eight nested Union Query data sets have the same local name and no stable key
  independent from order or query text;
- none of the 46 direct-or-nested DCS queries satisfies the existing bounded
  complete-source `QueryLanguageParser` contract.

The first slice therefore has sufficient evidence for stable entities and
ownership, but not for nested Union child identity or Query data dependencies.

## Decision

Add source-independent graph entities for Data Composition Schema, direct Data
Set, and direct named Data Composition Field. Reuse the existing Query node for
the one complete query declaration owned by each accepted direct Query data
set. Connect only immediate ownership with `Contains`.

The canonical first-slice graph is:

```text
Metadata(Report)
    --Contains--> DataCompositionSchema
DataCompositionSchema
    --Contains--> DataSet
DataSet
    --Contains--> DataCompositionField
    --Contains--> Query        # only DataSetQuery
```

No reverse edge, stored closure, source-reference edge, or report-specific
second query API is added.

## Graph model

Add these public node kinds with stable machine-readable names:

```rust
NodeKind::DataCompositionSchema // "data_composition_schema"
NodeKind::DataSet               // "data_set"
NodeKind::DataCompositionField  // "data_composition_field"
```

Add closed source-independent payload concepts equivalent to:

```rust
pub struct DataCompositionSchemaPayload {
    main: bool,
}

pub enum DataSetKind {
    Query,
    Object,
    Union,
}

pub struct DataSetPayload {
    kind: DataSetKind,
    data_source: Option<EntityName>,
}

pub struct DataCompositionFieldPayload {
    data_path: EntityName,
}
```

The exact Rust module and accessor names may follow nearby conventions, but the
closed content and compatibility rules are normative:

- Schema payload is valid only for `DataCompositionSchema`;
- Data Set payload is valid only for `DataSet`;
- Field payload is valid only for `DataCompositionField`;
- Query and Object data sets require one non-empty local data-source name;
- Union data sets have no direct data-source name in the accepted slice;
- a Field has one non-empty decoded data path;
- payload changes are semantic modifications and never identity changes.

The `main` flag preserves whether the owning Report's exact optional
`mainDataCompositionSchema` selects this schema. Main selection is intrinsic
typed content of an owned schema in the first Report-specific slice; it is not
modeled as an imprecise generic `References` edge.

## Identity

### Data Composition Schema

The direct Report `<templates uuid="...">` UUID is the canonical schema
`EntityId`. Report name, template name, path, main selection, and `.dcs` content
do not participate.

### Direct Data Set

The canonical identity tuple is:

```text
(schema UUID, direct data-set name)
```

Use the repository's length-prefixed owner-scoped identity convention or an
equivalent collision-safe encoding. Data-set kind, data-source name, XML
position, fields, query text, and file traversal order do not participate.

### Direct Data Composition Field

The canonical identity tuple is:

```text
(data-set ID, direct field name)
```

Field data path, title, role, appearance, XML position, and traversal order do
not participate.

### Metadata-owned Query

The canonical identity tuple is:

```text
(data-set ID, fixed query declaration role)
```

Exactly one Query is owned by each accepted direct `DataSetQuery`. Query text
and source order do not participate. Changing complete query text preserves
the Query ID and is observable through changed deterministic source provenance,
consistent with the existing BSL Query compatibility boundary.

All IDs must be collision-tested against delimiter-containing owner and local
name components. Duplicate accepted direct data-set or field names under one
owner are structural errors rather than ordinal identities.

## Contains endpoint and ownership contract

Extend `Contains` with only these additive pairs:

```text
Metadata(Report)      --Contains--> DataCompositionSchema
DataCompositionSchema --Contains--> DataSet
DataSet               --Contains--> DataCompositionField
DataSet               --Contains--> Query
```

The existing Procedure/Function-to-Query ownership pairs remain valid. Every
reversed, transitive, unrelated, Unknown, other Metadata kind, other flat node,
or missing-endpoint pair is invalid.

Every emitted Schema, Data Set, Field, and metadata-owned Query has exactly one
immediate owner. Graph validation must detect missing or multiple owners and
ownership cycles through existing deterministic issue ordering. `Contains`
does not become a data dependency or Impact propagation edge.

## EDT parser contract

The dedicated Report Data Composition reader joins two source artifacts without
emitting graph facts.

From the Report `.mdo`, it accepts:

- the existing Report UUID and canonical name as the owner contract;
- every direct `templates` element with a unique valid UUID, non-empty unique
  name, and exact `templateType` value `DataCompositionSchema`;
- zero or one direct `mainDataCompositionSchema` value in exact form
  `Report.<report-name>.Template.<template-name>` selecting a declared schema.

Every accepted declaration requires exactly one file at
`Templates/<template-name>/Template.dcs`. Missing or ambiguous artifacts,
duplicate UUID/name, malformed main selection, a main selection targeting an
undeclared schema, wrong root, malformed XML, unreadable input, or invalid
required names are fatal for that Report's composition descriptor and therefore
for the current complete production build. Existing non-DCS templates remain
outside this reader and retain universal metadata-reader behavior.

From the `.dcs` root, the parser accepts:

- exact Data Composition Schema namespace/root;
- zero or one direct local `dataSource` named `DataSource1` with type `Local`
  for the evidence-backed first slice;
- zero or more direct `dataSet` elements of exact `xsi:type`
  `DataSetQuery`, `DataSetObject`, or `DataSetUnion`;
- one non-empty unique direct name for each accepted data set;
- one exact `dataSource` reference for Query/Object and none for Union;
- zero or more direct `DataSetFieldField` elements with non-empty unique
  `field` name and non-empty `dataPath`;
- exactly one non-empty direct `query` for `DataSetQuery` and none for Object
  or Union.

The parser preserves exact decoded names, data paths, data-set kind, query text,
main selection, artifact path, and deterministic occurrence context. Returned
accepted entities are canonicalized by semantic identity rather than XML or
filesystem order.

## Deferred and malformed source outcomes

Nested data sets, including the eight duplicate-name Union children, produce a
typed deferred observation and no node. `DataSetFieldFolder` produces a distinct
typed deferred observation and no empty-name Field. Unknown data-set or field
`xsi:type` values are typed unsupported observations.

Deferred or unsupported child observations are recoverable after the Report,
Schema, and independently accepted direct children are structurally valid.
Production integration projects them through deterministic diagnostics and
legacy rejected-observation statistics until a broader public request or
reporting family is accepted. They never create Unknown, placeholder, guessed,
ordinal, or content-hash identities.

Malformed required content of an otherwise selected accepted direct entity is
fatal to avoid a partially declared owner subtree. Generated tests must cover
missing, empty, duplicate, mismatched, wrong-root, malformed, unreadable,
reordered, and repeated-read cases.

## Production emission

Report discovery remains the existing universal top-level `Reports` path. The
dedicated reader augments only successfully parsed Report descriptors. The
producer inserts, in dependency order:

1. the existing Report metadata node and configuration ownership;
2. one Schema node per accepted UUID and Report-to-Schema `Contains`;
3. direct Data Set nodes and Schema-to-DataSet `Contains`;
4. direct named Field nodes and DataSet-to-Field `Contains`;
5. one Query node and DataSet-to-Query `Contains` for each direct Query data
   set;
6. typed recoverable diagnostics/statistics for deferred or unsupported
   observations.

Every node and edge has non-empty deterministic provenance containing the
Report/schema/data-set/field/query identity, artifact-relative path, semantic
role, accepted source content required for Diff, stable producer, origin,
confidence, and resolution. Provenance is evidence, not identity.

Reordered descriptor entries, direct data sets, fields, and filesystem
traversal must produce equal canonical graph, diagnostics, statistics, report,
and indexes. Repeated builds are identical.

## Query and data-dependency boundary

Metadata-owned Query nodes are source declarations and reuse `NodeKind::Query`,
generic Query navigation, Diff, reports, validation, and indexes. They do not
use the BSL binding extractor and are owned by Data Set rather than Procedure
or Function.

No real DCS query passes the current complete-source query parser. Sprint 12
must not submit these queries to the public QuerySource request ledger, emit
query-language diagnostics based on partial parsing, or project `Reads`,
`DependsOn`, `References`, or data-source target candidates. Query text remains
opaque complete declaration content until a later evidence-backed query grammar
accepts the entire source set.

This exclusion preserves ADR-0030 all-or-nothing completeness and leaves every
existing BSL Query identity, request, diagnostic, Reads, DependsOn, and
statistics behavior unchanged.

## Generic consumers

Existing generic APIs remain authoritative:

- Query exposes new node kinds and immediate ownership in deterministic order;
- Diff reports stable add/remove/modify behavior for payload, provenance, and
  `Contains` facts;
- Impact does not infer dependencies from Data Composition ownership;
- reports count the stored node and edge kinds normally;
- validation enforces payload compatibility, precise endpoints, unique
  ownership, provenance, and report consistency;
- complete and incremental Semantic Index state must match clean rebuilds for
  every accepted node/payload/ownership transition;
- no Report-, Schema-, DataSet-, Field-, or DCS-specific second Query service is
  added.

## Coverage completion criteria

Architecture acceptance alone changes no Coverage status or aggregate count.

Graph-domain support requires:

- all three node kinds and closed compatible payloads;
- collision-safe identity and payload-only modification evidence;
- precise positive and exhaustive negative Contains matrices;
- unique ownership, provenance, Query, Diff, Impact-policy, report, Validation,
  complete-index, and incremental-index evidence;
- unchanged existing Query ownership and every unrelated graph behavior.

EDT support additionally requires:

- production Report descriptor and `.dcs` joining;
- exact UUID/name/main/data-source/data-set/field/query preservation;
- valid empty, main, non-main, Query, Object, and Union schemas;
- typed missing, duplicate, mismatched, unsupported, nested, folder, malformed,
  reordered, and repeated-build outcomes;
- a tracked provenance-backed reduced production fixture;
- complete generic consumer/index transitions and unchanged unrelated EDT
  behavior.

Graph-domain Coverage may transition after the complete graph-model task.
EDT capabilities and current-state aggregate counts transition only in the
final production-evidence task after all applicable evidence passes.

## Compatibility impact

Adding three `NodeKind` variants and three `GraphNodePayload` variants expands
public exhaustive enums. Adding `DataSetKind` and the typed payload structs adds
public APIs. Repository consumers must be updated in the graph-model task.
Existing variants, machine codes, identities, constructors, Query ownership,
edge kinds, endpoint matrices, and producer behavior remain unchanged.

There is no graph serialization or persistence contract today. A future
serialized representation must version the new tagged variants. No production
dependency or Cargo manifest change is required.

## Rejected alternatives

1. Represent schemas, data sets, and fields as `MetadataKind::Template` or
   generic Metadata nodes. Rejected because they are subordinate data entities
   with distinct ownership and content, not top-level metadata objects.
2. Store the DCS tree only as Report payload. Rejected because schemas, data
   sets, fields, and queries must remain independently addressable.
3. Use `References` to mark the main schema. Rejected because generic
   References would not preserve the main role and would introduce an
   imprecise dependency-like relation.
4. Use query text or XML ordinal in data-set identity. Rejected because content
   changes or reordering would create false entity replacement.
5. Emit nested duplicate-name Union children by ordinal. Rejected because the
   real source supplies no identity that survives reordering.
6. Hash nested query text into identity. Rejected because a query edit would
   replace the node rather than modify content.
7. Flatten nested Union children into their direct parent. Rejected because it
   would erase declared structure and conflate independent query programs.
8. Treat field folders as empty-name Data Composition Fields. Rejected because
   their source kind and structure are distinct.
9. Run the current query parser and emit whatever sources it recognizes before
   failure. Rejected by ADR-0030 all-or-nothing completeness and the zero-match
   corpus audit.
10. Expand query grammar in the same sprint. Rejected because complete DCS
    query-language semantics, virtual tables, batches, temporary tables, and
    nested sources form an independent capability.
11. Add specialized Report query APIs. Rejected because generic graph Query
    already exposes typed nodes and ownership.

## Deferred scope

- nested Union child data sets and their fields or queries;
- field folders, parameters, calculated fields outside accepted direct field
  declarations, roles, appearances, templates, settings, variants, layouts,
  filters, totals, and runtime composition behavior;
- complete DCS query-language parsing, virtual tables, batches, temporary
  tables, QuerySource requests, Reads, DependsOn, References, result schemas,
  and field-level lineage;
- non-Report Data Composition Schemas, Common Templates, external resources,
  extensions, partial workspaces, Designer XML, Runtime, API, CLI, MCP, LSP,
  IDE, persistence, and serialization.

## Implementation order

1. Implement the source-independent node, payload, identity, Contains,
   validation, generic consumer, index, and graph Coverage model.
2. Parse Report Data Composition declarations and `.dcs` artifacts into typed
   accepted/deferred outcomes without graph emission.
3. Integrate production node, ownership, provenance, diagnostics, statistics,
   determinism, and generated builder evidence.
4. Complete tracked production evidence, generic consumers, indexes, EDT
   Coverage, aggregate counts, and current-state documentation.
5. Run the Sprint 12 integration review.
