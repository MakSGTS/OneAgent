# ADR-0030: Direct Register Query Sources and Data Dependencies

## Status

Accepted

## Context

The completed first Data Access Graph slice creates stable Query nodes from
static named BSL Query declarations and emits direct resolved `Reads` edges to
Catalog or Information Register metadata only when the minimum query-language
parser proves the complete source set. The current parser accepts one fully
consumed `SELECT` with one direct persistent source. Joins, unions, nested
queries, batches, temporary tables, virtual tables, external or parameter
sources, malformed input, and unconsumed grammar produce typed no-edge
outcomes.

The EDT adapter resolves Query sources through private outcomes. It does not
expose them through the public ADR-0024 reference-request ledger. Query-derived
`DependsOn` is also absent even though ADR-0017 reserves that origin and the
required production `Reads` evidence now exists.

The Sprint 8 source investigation confirms static BSL Query declarations with
direct Accumulation and Accounting Register sources and corresponding top-level
metadata targets:

- `AccumulationRegister.InventoryCost` in
  `OneAgent_EDTproject/src/CommonModules/Accounting/Module.bsl`;
- `AccountingRegister.FinancialAccounting` in
  `OneAgent_EDTproject/src/CommonModules/MonthEndTransactions/Module.bsl`.

The same corpus contains register virtual tables, complex query grammar, and
other declaration forms, but their semantics are not interchangeable with a
direct persistent source. No real Calculation Register target/source pair was
confirmed.

## Decision

Sprint 8 extends the existing direct Query data-source slice in three connected
ways:

1. add direct Accumulation and Accounting Register source categories to the
   current completely parsed one-source query-language contract;
2. represent accepted Query data-source observations through the existing
   public `SemanticReferenceCategory::QuerySource` request lifecycle;
3. preserve the direct `Reads` projection and add one normalized derived
   `DependsOn` projection for every uniquely resolved accepted Query source.

This decision does not add Query declaration sources, register virtual-table
semantics, broader query grammar, Calculation Register access, Query mutation,
or write-derived dependencies.

## Terminology

The following categories are normative and separate:

- **Query declaration source** — source artifact and stable local declaration
  that creates a Query node;
- **query-language persistent source** — qualified data source parsed from the
  complete query program;
- **register virtual table** — invoked or third-component register source whose
  semantics may differ from direct access to the register;
- **direct relation** — `Reads`, retaining the specific data-access fact;
- **normalized relation** — `DependsOn`, retaining a materialized direct
  dependency justified by the direct relation.

Adding one category never implicitly adds another.

## Canonical semantic statements

### Direct data access

```text
Query --Reads--> persistent metadata source
```

This means the complete accepted static query program directly names the
persistent source and exact resolution identified one canonical top-level
metadata target.

### Normalized query dependency

```text
Query --DependsOn--> persistent metadata source
```

This means the same terminal resolved QuerySource request proves that a
meaningful change to the persistent source may require re-analysis of the
Query. The dependency is direct and materialized; it is not transitive closure,
reverse impact, ownership, or a replacement for `Reads`.

## Query entity boundary

The existing Query entity contract remains unchanged:

- source is a static named BSL Query binding inside a known Procedure or
  Function;
- the nearest callable owns the Query through `Contains`;
- Query identity is derived from owner and stable local declaration identity,
  never query text or source traversal order;
- changing text without moving the stable declaration preserves Query identity;
- dynamic, reassigned, module-scope, metadata-owned, standalone, or external
  declaration sources remain deferred.

No new `NodeKind`, ownership edge, payload, or identity rule is introduced.

## Supported query-language source contract

The accepted parser boundary remains one completely consumed query program:

- exactly one `SELECT` statement;
- exactly one direct persistent source;
- current accepted projection and alias grammar only;
- no unconsumed input;
- no construct capable of contributing another source.

The additive persistent categories are:

| Parsed namespace | `QuerySourceCategory` | Exact target kind | Sprint 8 state |
|---|---|---|---|
| `Catalog` or accepted Russian spelling | `Catalog` | `Metadata(Catalog)` | Existing |
| `InformationRegister` | `InformationRegister` | `Metadata(InformationRegister)` | Existing |
| `AccumulationRegister` | `AccumulationRegister` | `Metadata(AccumulationRegister)` | Added |
| `AccountingRegister` | `AccountingRegister` | `Metadata(AccountingRegister)` | Added |
| `CalculationRegister` | none | none | Deferred |
| Any other namespace | none | none | Deferred or unsupported |

Namespace matching remains exact for the evidence-backed spellings. Russian
Accumulation and Accounting Register spellings are not accepted without
repository-owned evidence.

The parser preserves raw spelling, normalized category, namespace, local name,
optional accepted alias, and deterministic query-text location. It does not
resolve metadata or emit graph facts.

## All-or-nothing completeness

The entire accepted source set must be proven before collection or resolution
can emit a request projection.

The following continue to make the Query ineligible for `Reads` and
query-derived `DependsOn`:

- JOIN or another top-level source;
- UNION or multiple branches;
- nested query;
- multiple statements or batch;
- temporary table declaration or source;
- external or parameter data source;
- register virtual table;
- dynamic or incomplete Query text;
- malformed or unconsumed query-language input.

Finding an accepted direct source before a rejected remainder never authorizes
a partial edge set.

## Register virtual-table boundary

An invoked or third-component register source, including Balance, Turnovers, or
BalanceAndTurnovers forms, is not a direct source under this ADR. The parser
continues to produce the typed `VirtualTableSource` outcome and no source
request, `Reads`, or `DependsOn` edge.

This ADR makes no statement that a virtual table is equivalent to its base
register. Parameters, virtual fields, totals, and base-register dependency need
a separate evidence-backed contract.

## Public QuerySource request lifecycle

Accepted parsed occurrences use the existing public request model. No new
public category or lifecycle state is required.

### Collection

One accepted occurrence creates or aggregates one collected request with:

- `source_node`: canonical Query node ID;
- `category`: `SemanticReferenceCategory::QuerySource`;
- canonical target expression containing the parsed metadata category and local
  name through existing source-independent reference vocabulary;
- exactly one expected `NodeKind` from the endpoint table;
- no candidates;
- `Collected` outcome and `Unresolved` state;
- collection provenance.

Collection provenance identifies the Query, owner, BSL module artifact, local
binding, raw source spelling, parsed category, deterministic query-text
location, and collector producer. Adapter paths and parser structs do not enter
the public request type.

### Resolution

Resolution occurs after all top-level metadata nodes are present. It preserves
the current query-specific deterministic policy:

1. normalize the local identifier with the accepted locale-independent Rust
   lowercase key and no NFC/NFKC pass;
2. partition candidates by the one exact expected kind;
3. prefer ambiguity among two or more compatible candidates;
4. resolve exactly one compatible candidate;
5. report incompatible kind when only differently typed candidates exist;
6. distinguish explicitly partial from complete workspace absence.

Candidate IDs are sorted and deduplicated. Display synonyms, aliases,
filesystem names, historical names, and localization guesses do not resolve a
target.

Terminal mapping is:

| Resolver outcome | Request outcome/state | Projection |
|---|---|---|
| Unique compatible target | `Resolved` / `Resolved` | `Reads` and derived `DependsOn` |
| No target in complete workspace | `MissingTarget` / `Unresolved` | typed diagnostic only |
| Explicit partial-workspace absence | `PartialWorkspace` / `Partial` | typed diagnostic or current partial projection only |
| Multiple compatible targets | `AmbiguousTarget` / `Ambiguous` | typed diagnostic only |
| Same name only at wrong kind | `IncompatibleTargetKind` / `Unresolved` | typed diagnostic only |

No failure creates a placeholder, Unknown, external, or lower-confidence target.

## Request identity and duplicates

Request identity follows ADR-0024 and uses the canonical tuple:

```text
(source Query, QuerySource category, target reference, expected kinds)
```

State, candidates, occurrence position, and provenance do not participate.
Equivalent occurrences aggregate collection and resolver provenance in sorted,
deduplicated order. Conflicting terminal content for one identity is a build
invariant error, not last-writer-wins behavior.

Statistics are derived once from terminal canonical requests. Parser
rejections that do not form an accepted request remain separately named legacy
rejected observations until a broader reporting contract is accepted. They
must not be counted as both request and rejection.

## Endpoint matrices

### Reads

The precise additive matrix is:

```text
NodeKind::Query
    --Reads-->
NodeKind::Metadata(
    Catalog
    | InformationRegister
    | AccumulationRegister
    | AccountingRegister
)
```

Every other source kind, Metadata kind, metadata member, flat semantic node,
Unknown, external, placeholder, reversed pair, or missing endpoint is invalid.

### DependsOn

The precise query-origin matrix is identical in endpoints but distinct in
meaning and edge kind:

```text
NodeKind::Query
    --DependsOn-->
NodeKind::Metadata(
    Catalog
    | InformationRegister
    | AccumulationRegister
    | AccountingRegister
)
```

This matrix is additive to the existing Attribute, Dimension, Resource, and
Command dependency matrices. It is not a wildcard
`Query --DependsOn--> Metadata(_)` rule.

Graph validation checks endpoint kinds, not parser syntax. The EDT producer is
responsible for the complete source and resolution contract.

## Edge identity, provenance, and aggregation

Both projections use standard edge identity:

```text
(source_node_id, target_node_id, edge_kind)
```

The `Reads` edge remains:

- `FactOrigin::Resolved`;
- `ResolutionState::Resolved`;
- exact confidence.

The companion `DependsOn` edge is:

- `FactOrigin::Derived`;
- `ResolutionState::Resolved`;
- exact confidence.

Both identify the terminal request, Query and owner, source artifact, raw and
normalized occurrence, location, expected target kind, and resolved target.
The dependency provenance additionally identifies the retained `Reads`
projection as the proving lower-level fact and the normalization producer.

Multiple occurrences for one Query-target pair support one canonical edge of
each kind. Producers aggregate sorted, deduplicated provenance before insertion;
duplicate edge identity alone is not a provenance merge mechanism.

## Relationship to other relations

| Relation | Sprint 8 boundary |
|---|---|
| `Reads` | Retained direct data-access fact |
| `DependsOn` | Added normalized direct dependency from the same resolved request |
| `Writes` | Unchanged Procedure mutation fact; does not create a companion dependency here |
| `References` | Not emitted for Query data sources |
| `Contains` | Query ownership remains unchanged and implies no data dependency |
| `Calls` | Query evaluation or platform methods are not modeled by this source contract |
| `Grants` | Authorization remains independent from observed data access |

## Query, Diff, Impact, reports, and indexes

The existing generic Query API remains authoritative. It exposes both stored
relations through edge-kind filtering and exposes both as direct dependencies.
Consumers that ask for all dependencies observe two distinct relations to the
same target; consumers filtering `Reads` or `DependsOn` observe only the named
fact.

Incoming usage navigation similarly exposes both edge kinds. This is deliberate
normalized-fact observability, not duplicate edge identity.

Graph Diff observes added, removed, or modified `Reads` and `DependsOn` edges
independently. Build Diff observes stable QuerySource requests and their state,
candidate, or provenance modifications.

Impact reverse traversal retains one unique affected Query node while allowing
deterministically ordered reasons for both direct and normalized relations. No
reverse edge or transitive closure is stored.

Reports derive request statistics once and count stored edge kinds normally.
Validation reconciles request terminal outcomes, diagnostics, projections, and
statistics without double counting.

The complete Semantic Index and incremental index add no new dimension. Their
node, edge, adjacency, request-independent graph, Query, Diff, and Impact
results must remain equivalent to a clean rebuild.

## Diagnostics

Existing typed query-language and resolution diagnostics remain stable. The
implementation must continue to distinguish at least:

- malformed query syntax;
- unsupported query structure;
- unsupported persistent namespace;
- virtual table source;
- temporary table source;
- external or parameter source;
- missing metadata target;
- ambiguous metadata target;
- incompatible metadata target kind;
- explicit partial-workspace absence.

New accepted namespaces do not turn previously rejected complex grammar into a
partially accepted program. Diagnostics retain Query identity, source context,
expected kind when known, ordered candidates, and deterministic provenance.

## Coverage completion criteria

Architecture acceptance changes no capability status, priority, required
evidence, or aggregate count. Existing `semantic_edge.reads`,
`semantic_edge.depends_on`, and graph-domain/EDT ReferenceRequest capabilities
remain `Supported` for their committed slices.

The expanded boundary may be recorded as current production only after:

- exact graph validation covers both added `Reads` targets and all four
  Query-origin `DependsOn` targets with exhaustive negative evidence;
- the parser exposes typed Accumulation and Accounting categories from
  provenance-backed fixtures while preserving all current rejections;
- public QuerySource requests carry collection and resolver provenance;
- terminal outcomes project diagnostics and statistics exactly once;
- unique success emits canonical `Reads` plus derived `DependsOn` without
  changing Query identity or ownership;
- production fixtures prove both added register families and existing Catalog
  and Information Register compatibility;
- negative, partial, ambiguous, incompatible, duplicate, reordered-source,
  repeated-build, Query, Diff, Impact, report, validation, and index-equivalence
  evidence passes;
- registry representative evidence and limitations are synchronized without a
  status or aggregate-count transition.

Architecture, parser-only tests, or manually inserted graph edges are not
production evidence.

## Compatibility constraints

- Existing `NodeKind`, `EdgeKind`, `MetadataKind`, Query IDs, Query ownership,
  BSL extraction, and public Query APIs remain source-compatible.
- Existing Catalog and Information Register `Reads` identities remain unchanged.
- Existing direct Reads now gain an explicit normalized companion dependency;
  this additive graph fact is the intended behavior change.
- Existing metadata type, Command, Writes, Grants, Includes, Extends, Opens,
  and reference-request behavior remains unchanged.
- No new crate or production dependency is required.
- Serialization, persistence, Runtime, API transport, and Designer XML are
  unaffected.

## Rejected alternatives

1. Treat every register kind as accepted. Rejected because Calculation Register
   source/target evidence is absent.
2. Map virtual tables directly to their base registers. Rejected because their
   parameters and semantics require a separate contract.
3. Parse the confirmed complete complex real queries in the same task.
   Rejected because general projection and tail grammar is independent from
   adding two persistent categories.
4. Emit partial Reads or dependencies before an unsupported remainder.
   Rejected because incomplete source sets would be exposed as complete facts.
5. Create query dependencies without retaining Reads. Rejected because the
   lower-level proving fact must remain observable.
6. Emit Query `References` as another companion edge. Rejected because direct
   access and normalized dependency already preserve the required meanings.
7. Derive `DependsOn` from every existing Reads edge after graph construction.
   Rejected because the canonical source is the terminal resolved production
   request with provenance, not an unqualified graph rewrite.
8. Add write-derived dependencies in the same slice. Rejected because Writes
   has a distinct Procedure source and separate evidence contract.
9. Keep Query source outcomes permanently private. Rejected because ADR-0024
   already provides the public lifecycle and current Query evidence satisfies
   its family prerequisites.
10. Change Coverage to a new status during planning. Rejected because planning
    creates no production evidence.

## Deferred scope

- Calculation Register and other persistent namespace families.
- Register virtual tables and base-register relations.
- JOIN, UNION, nesting, batches, multiple statements, temporary tables, and
  general query expression grammar.
- New Query declaration sources and metadata-owned Query entities.
- Query fields, parameters, result schemas, and field-level data access.
- Query-language mutation and Query `Writes`.
- Write-derived dependencies and broader BSL write forms.
- Register metadata payload, standard attributes, qualifiers, totals, and
  runtime record entities.
- Russian namespace spellings without repository evidence.
- Placeholder, Unknown, external, probabilistic, or cross-workspace targets.

## Ordered implementation prerequisites

1. Extend exact graph validation for the added Reads targets and Query-origin
   DependsOn matrix; prove generic Query, Diff, Impact, reports, and index
   compatibility without changing producers.
2. Add the two direct source categories and provenance-backed reduced parser
   fixtures; keep parsing separate from resolution and emission.
3. Convert parsed source observations and current resolver outcomes into the
   public QuerySource request lifecycle without changing production projections.
4. Integrate terminal requests into production diagnostics, statistics,
   `Reads`, and derived `DependsOn` emission.
5. Add representative full-builder evidence, synchronize Coverage evidence and
   current-state documentation, and keep statuses/counts unchanged.
6. Complete an independent Sprint 8 integration review before changing Sprint
   status or making Sprint 9 eligible.

## Consequences

### Positive

- Two real register families become exact persistent Query targets.
- Query-source uncertainty becomes observable through the existing public
  request lifecycle.
- Consumers receive normalized query dependencies without losing direct Reads.
- The slice reuses existing identity, graph, resolver, diagnostic, report,
  Diff, Impact, and index contracts.

### Negative

- The accepted grammar remains intentionally narrow and uses reduced fixtures
  rather than claiming full complex-query support.
- Consumers requesting all dependency relations observe both Reads and
  DependsOn for one resolved source.
- Calculation Registers and virtual tables remain unavailable.

### Neutral

- No new node or edge kind, Query declaration source, metadata payload,
  dependency, persistence format, or Runtime surface is introduced.
- Coverage statuses and aggregate counts do not change during planning or after
  the supported-boundary evidence expansion.
