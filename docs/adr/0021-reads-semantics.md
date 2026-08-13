# ADR-0021: Reads Semantics

## Status

Accepted

## Context

Semantic Model 2.0 declares `EdgeKind::Reads`, but the EDT adapter does not
emit it and the graph schema currently accepts Reads through a broad permissive
rule. The repository needs a production contract before query data access can
be represented without conflating source extraction, query parsing, metadata
resolution, generic references, normalized dependencies, and impact traversal.

The existing production pipeline already provides the source entity needed by
the first slice:

- `crates/bsl/src/queries.rs` conservatively extracts complete static query
  text from a named local BSL Query binding inside a known procedure or
  function;
- `adapters/edt/src/bsl_graph.rs` emits a stable `NodeKind::Query` node and its
  `Contains` ownership edge;
- Query identity is independent from query text and must remain unchanged;
- the production Query-node test proves that this source slice currently emits
  no `Reads`, `Writes`, or query-derived `DependsOn` edges.

No 1C query-language parser, query AST, query-source model, or metadata
data-source resolver exists in `oneagent-bsl`. Real sources under
`OneAgent_EDTproject/src` demonstrate simple persistent sources such as
`Catalog.Products` and `InformationRegister.ObjectsToDelete`, but also joins,
unions, nested queries, temporary tables, virtual tables, parameter and
external tables, multiple statements, and dynamically replaced query text.
Those artifacts are evidence of the required boundary, not a complete grammar
specification.

ADR-0017 already reserves the future relation
`Query --Reads--> Metadata(...)` for resolved static query data-source access.
This ADR accepts that direction and defines its first production slice.

## Problem

Without a precise contract, a future producer could incorrectly:

- scan query text for metadata-looking substrings instead of parsing it;
- emit an edge from the owning procedure or module instead of the Query node;
- treat fields, parameters, temporary tables, or virtual tables as persistent
  metadata sources;
- emit a partial read set from a query whose complete source structure was not
  understood;
- create placeholder targets for missing, ambiguous, dynamic, or external
  sources;
- store duplicate edges for repeated source occurrences;
- conflate Reads with References, Writes, DependsOn, Contains, or Grants;
- persist transitive dependencies or reverse impact edges;
- close `semantic_edge.reads` using architecture documentation alone.

## Decision

`EdgeKind::Reads` represents a direct, resolved data-access fact from a semantic
entity that evaluates a data source to the persistent metadata object whose
stored records are read.

The first production slice is:

```text
NodeKind::Query --Reads--> NodeKind::Metadata(first-slice persistent kind)
```

The source must be an existing static BSL Query node. The target must be an
existing, uniquely resolved top-level metadata object admitted by the explicit
first-slice allowlist. The query-language parser must prove the complete
first-slice source structure before any Reads edge is emitted.

## Canonical semantic definition

`Query A --Reads--> Metadata B` means that the complete static query program
represented by Query `A` directly declares a supported persistent data source
that resolves exactly to metadata object `B`, and evaluation of that query
reads records from `B`.

Canonical direction:

```text
reader --Reads--> data source
```

For the first slice, the reader is always `NodeKind::Query` and the data source
is always an allowlisted `NodeKind::Metadata(kind)`.

Reads is a resolved direct fact. It is not a transitive dependency, reverse
impact relation, runtime execution event, field selection, textual mention, or
proof that the owning BSL declaration executed the query.

## Source and target matrix

### Canonical endpoint matrix

| Source fact | Source kind | Target kind | First slice |
|---|---|---|---|
| Supported static query source | `NodeKind::Query` | `NodeKind::Metadata(MetadataKind::Catalog)` | Yes |
| Supported static query source | `NodeKind::Query` | `NodeKind::Metadata(MetadataKind::InformationRegister)` | Yes |
| Supported static query source | `NodeKind::Query` | Any other `NodeKind::Metadata(kind)` | Deferred |
| BSL procedure, function, or module | `Procedure`, `Function`, or `Module` | Any metadata kind | No |
| Query source occurrence | `NodeKind::Query` | Metadata member, flat semantic, external, or unknown node | No |

The first-slice allowlist is intentionally narrower than the set of metadata
kinds discovered by EDT. It is grounded in the existing Query-node integration
example using `Catalog.Products` and the real static one-line Query declaration
using `InformationRegister.ObjectsToDelete`. Adding another persistent source
family requires focused real-source and resolver evidence, precise validator
coverage, and an explicit extension of this allowlist.

`NodeKind::Metadata(MetadataKind::Unknown)` is never a concrete target. The
source contract does not authorize arbitrary metadata kinds, child nodes, or
flat semantic nodes.

## Supported first-slice source contract

The first slice accepts only a Query node produced by the existing static BSL
Query extraction contract:

- the Query has a known procedure or function owner;
- the local binding identity is stable;
- the complete query text is statically available from the supported
  constructor or `.Text` literal assignment form;
- the Query node and its ownership edge already exist with provenance;
- the complete text is accepted by the future query-language parser without a
  syntax or unsupported-structure diagnostic;
- the parsed program contains exactly one statement;
- that statement is one top-level `SELECT`;
- the statement has exactly one top-level data source;
- the source is a direct persistent metadata name from the first-slice
  allowlist, with an optional source alias;
- no other construct can contribute an additional data source.

The source may have projections, filtering, ordering, and scalar query
parameters only when the parser can consume them while proving that they do not
introduce another query or data source. A query-language implementation must not
treat an unparsed expression tail as proof that the source set is complete.

The existing Query node identity, owner, name, and `Contains` edge remain
unchanged. Parsing attaches analysis to that entity; it does not replace or
split the Query node.

## Query-language parsing boundary

Reads emission requires a real query-language parser. Regular-expression or
substring extraction of `FROM`-like text is not accepted because identifiers,
comments, string literals, nested queries, batches, localized keywords, and
source parameters make textual matching unsound.

Repository evidence is insufficient to declare a complete grammar. Therefore
the first ordered follow-up is a focused parser investigation. It must establish
repository-owned fixtures and a source contract for at least:

- lexical rules and statement boundaries;
- English and Russian query-language keyword handling;
- identifiers, qualified names, aliases, and case behavior;
- comments and string literals;
- parameter tokens and their positions;
- top-level versus nested source clauses;
- syntax-error recovery and deterministic source locations;
- detection of joins, unions, nested queries, batches, temporary tables,
  virtual tables, and external or parameter data sources.

The later parser implementation must produce a typed AST or an equivalently
typed parsed source model. For the first slice, the minimum result is:

- one parsed query program associated with the existing Query identity;
- one statement classification;
- a deterministically ordered collection of data-source occurrences;
- for each occurrence, its raw spelling, normalized source category,
  namespace, local metadata name, and source location;
- typed syntax and unsupported-structure diagnostics.

The parser must report whether the entire source set is complete for the
accepted slice. Reads emission is forbidden when that proof is absent.

## Data-source boundaries

| Source category | First-slice classification | Reads result |
|---|---|---|
| Direct top-level `Catalog.<Name>` source | Persistent metadata source | Resolve exact Catalog target and emit on unique success |
| Direct top-level `InformationRegister.<Name>` source | Persistent metadata source | Resolve exact Information Register target and emit on unique success |
| Other persistent metadata namespace | Deferred source family | Typed unsupported-source diagnostic; no edge |
| Register virtual table such as a third component or invocation | Virtual table | Typed unsupported-source diagnostic; no edge, including no edge to the base register |
| Temporary table declaration or source | Temporary table | Typed unsupported-structure diagnostic; no edge for the Query |
| Parameter used as a data source | Dynamic or external source | Typed unsupported-source diagnostic; no edge for the Query |
| External table supplied by the caller | External source | Typed unsupported-source diagnostic; no edge for the Query |
| Nested query | Nested source scope | Typed unsupported-structure diagnostic; no edge for the Query |
| `UNION` or `UNION ALL` | Multiple query branches | Typed unsupported-structure diagnostic; no edge for the Query |
| `JOIN` of any kind | Multiple top-level sources | Typed unsupported-structure diagnostic; no edge for the Query |
| Multiple statements or a query batch | Multiple statement scopes | Typed unsupported-structure diagnostic; no edge for the Query |
| Dynamically assembled, replaced, reassigned, or incomplete text | Incomplete static evidence | No parser input accepted for Reads; no edge |
| Malformed query text | Syntax failure | Typed syntax diagnostic; no edge |

Scalar query parameters outside data-source positions are not graph targets and
do not themselves create Reads edges. They may be present only when the parser
still proves the complete source set. A parameter or external table in a source
position makes the whole Query unsupported for first-slice emission.

The first slice is all-or-nothing per Query. It must not emit a supported
persistent source found before an unsupported join, union, nested query,
temporary-table statement, or malformed remainder. This prevents a partial
edge set from being mistaken for complete data-access evidence.

## Metadata resolution rules

Parsed metadata sources are resolved after all top-level metadata nodes exist.
Resolution uses the normalized metadata namespace and local metadata identifier,
never the query alias or display synonym.

For the first slice:

1. Preserve the raw source spelling for diagnostics and provenance.
2. Map the parsed persistent namespace only through the explicit allowlist.
3. Normalize identifier case according to the parser-investigation contract;
   do not apply display-name, synonym, historical-name, or localization aliases.
4. Resolve among existing graph nodes of the exact required
   `NodeKind::Metadata(kind)`.
5. Emit Reads only when exactly one compatible node exists.

Case-insensitive language matching must not make resolution nondeterministic.
If multiple in-graph names normalize to the same identifier, resolution is
ambiguous and emits no edge. A future shared case-normalized resolution API is
optional; the first implementation may keep this policy inside the query-source
resolver without changing the public Query API.

Failure policy:

| Condition | Required result |
|---|---|
| Missing metadata target | Typed missing-target diagnostic; no edge |
| Multiple compatible targets | Typed ambiguous-target diagnostic with deterministically ordered candidates; no edge |
| Name exists only at an incompatible kind | Typed incompatible-kind diagnostic; no edge |
| Unsupported metadata namespace | Typed unsupported-source diagnostic; no edge |
| External or partial workspace target | Missing or external-source diagnostic; no placeholder and no edge |
| Missing Query source node | Graph-construction invariant failure; no edge |

Missing, ambiguous, incompatible, external, unsupported, dynamic, malformed,
or incomplete sources must not create placeholder, Unknown, or external graph
targets.

## Direct versus derived semantics

Reads stores only direct, resolved data-access facts found in the Query source.
It does not store transitive closure, ownership projection, reverse usage, or
reverse impact edges.

The parsed source occurrence is direct source evidence. The emitted edge is
classified as resolved because its metadata endpoint is established by semantic
resolution:

```text
FactOrigin::Resolved
ResolutionState::Resolved
Confidence::Exact
```

Reads is not a derived normalized dependency. A future query-derived DependsOn
edge may use a resolved Reads fact as evidence, but that task must preserve the
Reads edge and follow a separate accepted dependency contract.

## Identity, provenance, duplicates, and determinism

Reads uses the standard graph edge identity:

```text
(source_node_id, target_node_id, EdgeKind::Reads)
```

Query text, alias, source position, metadata spelling, parser state,
provenance, and insertion order are not part of edge identity.

Every emitted Reads edge must carry deterministic provenance sufficient to
explain the resolved data access. The minimum context is:

- the existing Query node ID and owner ID;
- the BSL module source path and Query declaration context already used by the
  Query node;
- the raw query-source occurrence and deterministic query-text location;
- the parsed source category, namespace, and local metadata name;
- the resolved target node ID and target metadata kind;
- stable parser, query-source resolver, and graph-contributor producer stages;
- `FactOrigin::Resolved`, `ResolutionState::Resolved`, and exact confidence.

The current provenance model may encode source-specific location and resolution
context in a deterministic source identifier. A public source-range API is not
required by this ADR and must not be added implicitly.

Repeated source occurrences that resolve to the same Query and target support
one canonical edge. Every distinct source occurrence may contribute evidence to
that edge, but equivalent provenance records are sorted and deduplicated before
insertion. The producer must aggregate provenance before inserting the edge;
the graph's duplicate edge identity alone is not a provenance merge mechanism.

Given identical inputs, parser output, diagnostics, resolution outcomes, edge
identity, provenance ordering, graph snapshots, and build reports must be
identical. Filesystem order, AST traversal order, map iteration, and query
source order must not change the result.

## Validation constraints

The implementation task must replace broad Reads acceptance with only:

```text
NodeKind::Query
    --Reads-->
NodeKind::Metadata(MetadataKind::Catalog)

NodeKind::Query
    --Reads-->
NodeKind::Metadata(MetadataKind::InformationRegister)
```

The validator must reject:

- every non-Query source kind;
- metadata targets outside the first-slice allowlist;
- metadata member, flat semantic, Unknown, or external targets;
- missing endpoints;
- edges without provenance under the existing provenance invariant.

A physical self-loop is impossible for valid endpoint kinds. Existing missing
endpoint and provenance validation remains authoritative. The Writes branch
must remain unchanged until a separate Writes contract defines its endpoint
matrix.

## Diagnostics and unsupported-source behavior

Parser and resolver failures are typed, deterministic analysis outcomes. At a
minimum, later implementation must distinguish:

- malformed query syntax;
- unsupported query structure;
- unsupported persistent namespace;
- virtual table;
- temporary table;
- external or parameter data source;
- dynamic or incomplete source text;
- missing metadata target;
- ambiguous metadata target;
- incompatible metadata target kind.

Diagnostics must identify the Query node and source context and must sort
deterministically. They do not create graph endpoints or lower-confidence Reads
edges. Unsupported syntax must not be silently ignored.

## Relationship to other edges

| Edge | Meaning | Boundary from Reads |
|---|---|---|
| `Reads` | Direct resolved data access from Query to persistent metadata source | Canonical edge for this source fact |
| `Writes` | Direct resolved mutation of persistent data | Separate capability and contract; a query read does not imply a write |
| `References` | Generic resolved semantic reference | Does not express data-access direction or persistent read semantics |
| `DependsOn` | Materialized normalized direct semantic dependency | Deferred for queries until Reads production evidence exists |
| `Contains` | Structural ownership | Query ownership remains owner-to-Query and does not imply data access |
| `Grants` | Explicit access authorization | Authorization does not prove runtime data access |

The first Reads producer emits no companion References or DependsOn edge. A
future task may add a separately accepted fact without replacing Reads.

## Reconciliation with existing architecture

ADR-0006 remains unchanged. Reads is a typed directed graph edge, both endpoint
nodes must exist, and deterministic graph construction remains mandatory.

ADR-0008 remains unchanged. Query parsing and resolution contribute controlled
facts to the source-independent graph, every emitted fact retains provenance,
reverse navigation uses graph indexes, and unresolved analysis remains explicit
rather than becoming a resolved edge.

ADR-0017 remains authoritative for normalized DependsOn semantics. This ADR
adopts its reserved `Query --Reads--> Metadata(...)` data-access direction but
does not authorize query-derived DependsOn. That later dependency slice remains
deferred until Reads production evidence and a separate source contract exist.

## Dependency-query and Impact Analysis policy

The existing Query API and Impact Analysis policy remain unchanged.

Reads continues to participate in dependency and usage classification:

- outgoing Reads from a Query is a direct dependency on the metadata source;
- incoming Reads to metadata is direct usage by the Query;
- generic edge-kind queries expose the stored edge without a dedicated API.

Impact Analysis continues to traverse dependency edges in reverse from a
changed metadata source to directly affected Query nodes. Transitive impact is
computed by the existing bounded traversal and optional ownership policy. No
reverse impact edge, transitive Reads edge, new query method, weight, score, or
risk rank is stored by this slice.

## Coverage Registry completion criteria

`semantic_edge.reads` must remain `DeclaredOnly` while this architecture task is
the only completed work. Counts and gap priorities must not change.

The capability may transition to `Supported` only after production evidence
proves the accepted first slice:

- `EdgeKind::Reads` remains declared;
- this accepted semantic contract exists;
- repository-owned query-language investigation fixtures and findings exist;
- a typed parser or parsed source model proves complete first-slice source sets;
- typed syntax and unsupported-source diagnostics exist;
- exact metadata kind-and-name resolution exists;
- precise Reads endpoint validation exists;
- the EDT production graph path emits resolved Reads edges from existing Query
  nodes without changing Query identity or ownership;
- every edge carries deterministic resolved provenance;
- focused positive tests cover both allowlisted target kinds;
- negative tests cover unsupported structure, malformed text, virtual,
  temporary, external or parameter sources, and failed metadata resolution;
- duplicate occurrence and repeated-build tests prove deterministic identity,
  provenance, diagnostics, graph diff, and build-result diff;
- an integration test exercises the full `FileSystemEdtSemanticGraphBuilder`
  path from static BSL Query source through metadata resolution and emission;
- Query dependency/usage navigation and reverse Impact propagation are proven;
- the deterministic registry update changes only `semantic_edge.reads` and its
  resulting aggregate counts.

Architecture acceptance, parser-only tests, or manually inserted graph edges
are not sufficient Coverage evidence.

## Consequences

- Reads has a canonical meaning, direction, endpoint matrix, and no-edge policy.
- The existing static Query entity becomes the stable source for the first
  Data Access Graph edge without identity or ownership changes.
- The first slice is deliberately small enough to validate parser, resolution,
  provenance, Query, Impact, and Coverage behavior end to end.
- Unsupported query structures cannot produce misleading partial read sets.
- Parser investigation and implementation are required before graph emission.
- Broader metadata families and query constructs can be added through explicit
  evidence-backed extensions without changing existing edge identities.

## Rejected alternatives

1. Text search or regular expressions for metadata-looking names are rejected
   because they cannot prove source scope or exclude strings, comments, nested
   queries, dynamic fragments, and unsupported constructs.
2. Procedure or Function as the first-slice source is rejected because the
   existing Query node is the stable entity that owns the complete query text.
3. Module or metadata owner as source is rejected because it collapses distinct
   queries and their provenance.
4. All discovered metadata kinds as first-slice targets are rejected because
   the repository lacks equivalent real-source, parser, and resolver evidence
   for every family.
5. Mapping virtual tables directly to their base register is rejected because
   virtual-table semantics and parameters require a separate source contract.
6. Emitting Reads for supported fragments of an otherwise unsupported query is
   rejected because consumers could mistake a partial read set for complete
   data-access evidence.
7. Temporary, external, or parameter tables as metadata targets are rejected
   because no stable persistent metadata endpoint is proven.
8. Placeholder, Unknown, or external target nodes are rejected because the
   first slice requires exact in-graph metadata resolution.
9. Reads as a synonym for References is rejected because generic semantic use
   does not prove persistent data access.
10. Reads as a synonym for DependsOn is rejected because Reads is a direct
    resolved source fact while DependsOn is a normalized dependency relation.
11. Persisting transitive closure or reverse impact edges is rejected because
    existing traversal derives those views.
12. Designing Writes in the same task is rejected because mutation semantics
    require a separate production source and endpoint contract.

## Explicitly deferred scope

- query-language parser and AST implementation;
- parser diagnostics implementation;
- metadata data-source resolution code;
- Reads graph emission;
- metadata source families beyond Catalog and Information Register;
- joins, nested queries, unions, batches, and multiple statements;
- virtual tables, temporary tables, external tables, and data-source
  parameters;
- field-level data-access nodes or edges;
- Writes semantics or emission;
- query-derived DependsOn;
- metadata-owned Query source families;
- new node or edge kinds;
- public API changes;
- Coverage Registry evidence, status, or count changes;
- production fixtures and Rust tests;
- changes under `OneAgent_EDTproject`;
- new dependencies.

## Ordered follow-up implementation tasks

1. Perform a focused query-language parser investigation using repository-owned
   real sources and fixtures. Record the minimum lexical and grammar contract,
   bilingual behavior, deterministic locations, and typed diagnostic taxonomy
   without changing graph behavior.
2. Implement the accepted first-slice query-language parser or typed parsed
   source model in `oneagent-bsl`, including diagnostics and all-or-nothing
   completeness classification. Do not resolve metadata or emit graph edges.
3. Implement deterministic query-source resolution for the two allowlisted
   metadata kinds after all top-level metadata nodes exist. Cover exact kind,
   normalized name, missing, ambiguous, incompatible, and partial-workspace
   outcomes. Do not emit Reads yet.
4. Replace only the broad Reads validator branch with the precise first-slice
   endpoint matrix and focused positive and negative graph-domain tests. Leave
   Writes unchanged.
5. Emit canonical provenance-backed Reads edges through the EDT production
   builder, aggregate duplicate evidence before insertion, and add focused and
   full-pipeline tests for positive, negative, Query, Impact, and determinism
   behavior.
6. Transition only `semantic_edge.reads` to `Supported` and update aggregate
   counts after every completion criterion is proven by deterministic
   production evidence.
7. Define Writes in a separate architecture task. Define query-derived
   DependsOn only after Reads production evidence exists.
