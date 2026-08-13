# ADR-0020: Includes Semantics

## Status

Accepted

## Context

Semantic Model 2.0 declares `EdgeKind::Includes`, but the EDT adapter does not
emit it and the graph schema still accepts it through a broad fallback rule.
The repository therefore needs a production contract before subsystem
composition can be represented without conflating membership, ownership,
reference, authorization, and dependency semantics.

The graph already provides the two representations required by the first
slice:

- a flat `NodeKind::Subsystem` semantic node derived from each discovered EDT
  Subsystem object;
- a `NodeKind::Metadata(kind)` node for every supported top-level metadata
  object, including metadata Role and Subsystem objects.

These representations are intentionally distinct. The flat Subsystem node is
the composition subject. A metadata object node is the included member.

## Problem

Without a precise contract, a future producer could incorrectly:

- use `Contains` for a many-to-many membership relation;
- use the metadata Subsystem object rather than the flat Subsystem node as the
  source;
- resolve `Role.<Name>` to the flat access-control Role node;
- emit a generic `References`, `Grants`, or `DependsOn` edge instead of direct
  membership;
- infer membership from configuration inventory or command-interface data;
- persist transitive membership closure;
- invent placeholder targets for unsupported or partial workspaces;
- make edge identity or provenance depend on XML order.

## Confirmed source facts

### EDT artifact family

The accepted source family is the EDT Subsystem metadata descriptor:

```text
src/Subsystems/<SubsystemName>/<SubsystemName>.mdo
```

The descriptor root is `mdclass:Subsystem`. Direct membership is serialized as
the repeated direct child field:

```xml
<mdclass:Subsystem ...>
  ...
  <content>Document.EmptyDocument</content>
  <content>Role.Document_EmptyDocument_Posting</content>
  <content>Document.TestDocument</content>
</mdclass:Subsystem>
```

The exact XML field is `mdclass:Subsystem/content`. The value is a qualified
metadata reference in the form `<Kind>.<Name>`.

Subsystem hierarchy is serialized separately through nested `Subsystems`
directories, repeated `<subsystems>` declarations, and `<parentSubsystem>`
declarations. Those sources do not prove direct `<content>` membership and are
not part of this decision's first production slice.

### Inspected source evidence

A read-only inspection of a representative EDT 8.3.27 project, whose
configuration compatibility mode is 8.3.21, confirmed:

- 127 Subsystem descriptors: 13 top-level and 114 nested;
- 125 descriptors with at least one `<content>` declaration;
- 3,971 `<content>` observations and 3,510 unique qualified targets;
- the same qualified target in as many as four different Subsystem
  descriptors;
- no exact duplicate `<content>` token inside one inspected descriptor.

The runtime version and configuration compatibility mode are separate facts and
must not be conflated. The observed many-to-many membership is evidence against
modeling subsystem membership as exclusive ownership. The absence of
source-local duplicates in this sample does not prove that EDT forbids them.

The representative `TestObject` descriptor has stable identity and directly
declares two Document targets and one Role target. Repository history for that
source changes `Role.PostingEmptyDocument` to
`Role.Document_EmptyDocument_Posting`, supporting exact qualified-name
resolution rather than display-name or historical-alias matching.

## Decision

`EdgeKind::Includes` represents a direct, explicit, source-declared membership
of one semantic entity in a composition subject.

The first accepted producer is direct EDT Subsystem `<content>` membership.

## Canonical semantic definition

`Subsystem A --Includes--> Metadata B` means that Subsystem `A` directly and
explicitly declares metadata object `B` as a member through one
`mdclass:Subsystem/content` value.

Canonical direction:

```text
NodeKind::Subsystem --Includes--> NodeKind::Metadata(kind)
```

## Source contract

The first production slice has the following source contract:

- artifact: `src/Subsystems/<Name>/<Name>.mdo` discovered by the existing
  top-level EDT Subsystem path;
- root: `mdclass:Subsystem`;
- field: each direct repeated `mdclass:Subsystem/content` child;
- relation: direct and declared, not inferred or derived;
- token: exact `<Kind>.<Name>` text;
- ordering: source order has no semantic meaning and must not affect graph or
  provenance ordering;
- absence: no `<content>` values means no membership edges;
- duplicate observations: equivalent observations are deterministically
  deduplicated and do not create duplicate edges or duplicate provenance.

The parser must not recursively reinterpret descendant fields as direct content
of the top-level descriptor. Nested Subsystem discovery is a separate source
slice.

## Endpoint contract

### Canonical endpoint matrix

| Source fact | Source kind | Target kind | Direction |
|---|---|---|---|
| Direct EDT Subsystem `<content>` declaration | `NodeKind::Subsystem` | `NodeKind::Metadata(kind)` | subsystem to directly declared member |

`NodeKind::Metadata(MetadataKind::Unknown)` is not a valid concrete target for
this contract. The endpoint family does not authorize arbitrary source kinds or
flat semantic target kinds.

### Source representation

The source is the existing flat `NodeKind::Subsystem` node whose identity is
derived from the Subsystem metadata UUID. The corresponding
`NodeKind::Metadata(MetadataKind::Subsystem)` object remains the configuration-
owned metadata object and is not the membership source.

### Target representation

The target is the existing metadata object node selected by the parsed metadata
kind and exact local name.

`Role.<Name>` resolves only to
`NodeKind::Metadata(MetadataKind::Role)`. It must never fall back to the flat
`NodeKind::Role` access-control subject used by `Grants`.

`Subsystem.<Name>` would refer to
`NodeKind::Metadata(MetadataKind::Subsystem)`, not to a flat Subsystem node.
Subsystem-to-subsystem membership is excluded from the first slice, so this
prefix produces no edge until a separate slice defines its interaction with
hierarchy sources and semantic self-membership.

### First-slice target allowlist

The first production slice accepts only prefixes whose top-level metadata
kinds are already discovered and emitted by the EDT adapter, excluding
Subsystem itself:

| Serialized prefix | Target metadata kind |
|---|---|
| `Catalog` | `MetadataKind::Catalog` |
| `Document` | `MetadataKind::Document` |
| `Enum` | `MetadataKind::Enumeration` |
| `CommonModule` | `MetadataKind::CommonModule` |
| `Report` | `MetadataKind::Report` |
| `DataProcessor` | `MetadataKind::DataProcessor` |
| `InformationRegister` | `MetadataKind::InformationRegister` |
| `AccumulationRegister` | `MetadataKind::AccumulationRegister` |
| `AccountingRegister` | `MetadataKind::AccountingRegister` |
| `CalculationRegister` | `MetadataKind::CalculationRegister` |
| `BusinessProcess` | `MetadataKind::BusinessProcess` |
| `Task` | `MetadataKind::Task` |
| `Role` | `MetadataKind::Role` |
| `CommonCommand` | `MetadataKind::Command` |
| `CommonForm` | `MetadataKind::CommonForm` |
| `CommonTemplate` | `MetadataKind::Template` |
| `HTTPService` | `MetadataKind::HttpService` |
| `WebService` | `MetadataKind::WebService` |
| `XDTOPackage` | `MetadataKind::XdtoPackage` |

This allowlist is fixed for the first slice. A new EDT directory mapping does
not silently extend Includes production support. Configuration, subordinate
Form, Unknown, and Subsystem targets are not in the first-slice allowlist.

## Direction, directness, identity, and duplicates

The stored direction is from the declaring flat Subsystem node to the directly
declared metadata member. Reverse membership navigation uses the incoming edge
index; no reverse edge is stored.

Only direct membership is stored. Transitive membership must be computed by a
future query over accepted hierarchy and membership relations. The graph must
not persist transitive closure or infer membership from a parent Subsystem.

Includes uses the standard graph edge identity:

```text
(source_node_id, target_node_id, EdgeKind::Includes)
```

XML position, source path, token spelling, provenance, and insertion order are
not part of semantic identity.

Repeated observations that resolve to the same source and target support one
canonical edge. Distinct provenance observations are sorted and deduplicated
before insertion. Repeating the identical token in one descriptor yields one
edge and one equivalent provenance record. Different Subsystems including the
same metadata object produce distinct edges because their source identities
differ.

## Resolution and failure policy

Resolution is deterministic and case-sensitive:

1. Preserve the raw `<content>` token.
2. Require exactly one `.` separator with non-empty `<Kind>` and `<Name>`
   components. Do not trim, case-fold, localize, or apply aliases.
3. Map `<Kind>` through the explicit first-slice prefix allowlist.
4. Convert `<Name>` to the existing exact local-name representation.
5. Resolve by exact local name and exact
   `NodeKind::Metadata(parsed_metadata_kind)` through the existing semantic
   resolution index.
6. Emit Includes only when exactly one compatible in-graph target exists and
   the flat source Subsystem node exists.

Failure policy:

| Condition | Required result |
|---|---|
| Malformed descriptor XML | Return the existing typed reader/build error; emit no partial edge from that descriptor |
| Malformed qualified token | Record a deterministic typed format diagnostic and emit no edge |
| Unsupported prefix | Record a deterministic unsupported-prefix outcome and emit no edge |
| Recognized but deferred `Subsystem` prefix | Record a deterministic deferred/unsupported outcome and emit no edge |
| Missing target | Record a missing-target resolution outcome and emit no edge |
| Ambiguous exact-kind target | Record all candidate identities in deterministic order and emit no edge |
| Name exists only with an incompatible kind | Record an incompatible-kind outcome and emit no edge |
| `Role.<Name>` has only a flat Role candidate | Treat it as missing or incompatible for the required metadata Role kind; emit no edge |
| External target or partial workspace | Treat the absent in-graph metadata endpoint as missing; emit no placeholder and no edge |
| Missing flat Subsystem source | Report a graph-construction invariant failure and emit no edge |

No placeholder, `Unknown`, external, or flat semantic target node may be
invented by this slice. Diagnostics and reference statistics may reuse existing
resolution facilities, but they must preserve the Includes source context and
must not change the edge contract.

## Provenance requirements

Every emitted Includes edge must carry deterministic provenance that is
sufficient to explain the resolved membership. The minimum source context is:

- project-relative Subsystem descriptor path;
- Subsystem metadata UUID;
- flat Subsystem semantic node ID;
- exact XML field `mdclass:Subsystem/content`;
- raw qualified target token;
- parsed target metadata kind and local name;
- resolved target node ID;
- stable EDT subsystem-content resolution producer stage;
- `FactOrigin::Resolved`;
- `ResolutionState::Resolved`;
- exact confidence.

The parsed `<content>` observation precedes resolution, but the emitted semantic
edge is classified as `Resolved` because its target identity is established by
the resolution stage. The deterministic source identifier may encode the
required context using canonical escaping. Provenance is not part of edge
identity.

The current graph provenance model has no source-specific span field. Exact XML
source spans are therefore deferred. The descriptor path and exact field/token
context are mandatory even while spans are unavailable, and a future span
addition must not change semantic identity.

## Validation contract

The later implementation must replace broad Includes acceptance with an
explicit rule for the first slice:

```text
NodeKind::Subsystem
    --Includes-->
NodeKind::Metadata(first-slice allowlisted kind)
```

The validator must reject:

- any non-Subsystem source kind, including metadata Subsystem objects;
- flat `NodeKind::Role` targets;
- flat `NodeKind::Subsystem` targets;
- `NodeKind::Unknown` and `NodeKind::Metadata(MetadataKind::Unknown)` targets;
- metadata kinds outside the first-slice allowlist;
- missing endpoints;
- physical self-loops;
- edges without provenance under existing provenance validation.

The endpoint kinds make a physical `source_node_id == target_node_id` loop
structurally impossible in a valid graph, but the validator must still reject
one defensively. Semantic self-membership through a Subsystem's corresponding
metadata object is also excluded because metadata Subsystem targets are not in
the first-slice allowlist.

## Relationship to other edges

| Edge | Meaning | Why it does not replace Includes |
|---|---|---|
| `Contains` | Structural ownership from owner to owned child | Subsystem content is many-to-many membership and does not establish a canonical owner |
| `References` | Generic resolved semantic reference | The `<content>` field declares membership, not merely mention or linkage |
| `Includes` | Direct declared composition membership | Canonical relation for this source fact |
| `Grants` | Direct allowed access from a flat Role to a scoped AccessRight | Membership does not authorize access, and metadata Role members are not access subjects here |
| `DependsOn` | Direct normalized semantic dependency | Membership alone does not prove change dependency or impact propagation |

The future first-slice producer must emit Includes for this source fact. It does
not require a companion References or DependsOn edge. Any future lower-level
reference fact must have its own accepted producer contract and must not replace
Includes.

## Reconciliation with existing architecture

ADR-0007 remains unchanged: configuration-to-metadata-object structural
ownership is represented by `Contains`.

ADR-0017 remains authoritative for `DependsOn` direction, identity, provenance,
resolution, traversal, and its first metadata-member type-reference slice.
This ADR partially supersedes only ADR-0017's older statements that classify
"Subsystem membership" as `Contains`, including the corresponding candidate-
origin and endpoint-matrix rows. Direct EDT Subsystem `<content>` evidence shows
many-to-many composition membership rather than exclusive ownership, so that
fact is represented by Includes. The historical ADR is not rewritten.

ADR-0019 remains unchanged: access authorization is represented by Grants, and
its examples of `Subsystem --Includes--> Metadata(...)` are consistent with
this decision. Metadata Role membership and flat Role authorization remain
separate facts.

## Query and Impact classification

Includes is excluded from dependency traversal. It is not added to the Query
API dependency or usage edge classification.

Includes is excluded from Impact Analysis in the first slice. A future
subsystem-aware impact policy may use membership as an optional projection, but
that requires a separate accepted decision and must not change the direct edge
meaning.

The existing generic Query API is sufficient for first-slice navigation:

- `edges_by_kind(EdgeKind::Includes)` lists all stored memberships;
- `outgoing_edges_by_kind(subsystem, EdgeKind::Includes)` lists direct members;
- `incoming_edges_by_kind(member, EdgeKind::Includes)` lists direct containing
  Subsystems;
- generic all-edge, outgoing-edge, and incoming-edge queries remain available.

No dedicated membership query API is required by this slice.

## Rejected alternatives

1. `Contains` is rejected because direct Subsystem content is many-to-many
   membership, not canonical structural ownership.
2. `References` as the only edge is rejected because it loses the field's
   composition meaning.
3. `Grants` is rejected because inclusion does not authorize access.
4. `DependsOn` is rejected because membership alone is not a dependency or
   impact fact.
5. The metadata Subsystem object as source is rejected because the existing
   flat Subsystem node is the semantic composition subject.
6. Flat Role targets are rejected because they are access-control subjects,
   while `<content>` names a metadata object.
7. Persisting transitive closure is rejected because only direct declarations
   are source facts and traversal can compute closure later.
8. Placeholder or Unknown targets are rejected because the first slice has no
   external-node identity or membership contract.
9. Inferring membership from configuration inventory, command-interface
   navigation, or directory location is rejected because those sources do not
   prove direct `<content>` membership.
10. Parsing all nested Subsystems in the first slice is rejected because the
    current adapter discovers only top-level Subsystem descriptors and hierarchy
    semantics remain unresolved.
11. A dedicated membership query API is rejected for the first slice because
    generic typed edge queries already provide direct navigation.

## Minimal first production slice

The narrowest implementation-ready slice is:

1. Reuse existing top-level `src/Subsystems/<Name>/<Name>.mdo` discovery.
2. Parse only direct repeated `mdclass:Subsystem/content` fields.
3. Preserve deterministic pending observations containing the source descriptor,
   Subsystem UUID and flat node ID, raw token, parsed kind, and parsed name.
4. Accept only the explicit first-slice prefix allowlist.
5. Resolve each target by exact local name and exact metadata kind through the
   existing semantic resolution index.
6. Emit one provenance-backed
   `NodeKind::Subsystem --Includes--> NodeKind::Metadata(kind)` edge per unique
   resolved source-target pair.
7. Add the precise validator allowlist and negative endpoint checks.
8. Preserve generic direct edge queries and leave dependency and Impact
   traversal unchanged.
9. Transition Coverage only after all completion evidence exists.

The positive real-format fixture should use the representative `TestObject`
shape with `Document.EmptyDocument`,
`Role.Document_EmptyDocument_Posting`, and `Document.TestDocument`. This is
slightly richer than the single `DataProcessor.PurchaseOrderBalances` example,
but it proves the highest-risk distinction: a `Role.<Name>` content token
targets the metadata Role node and never the flat Role node. The fixture remains
small and uses stable UUID-bearing target descriptors.

Negative fixtures must cover malformed, unsupported, missing, ambiguous,
incompatible-kind, external/partial-workspace, invalid-source, invalid-target,
and duplicate observation cases.

## Implementation prerequisites

Before production support is claimed, the later task must:

1. Add a focused Subsystem content reader or extend the existing descriptor
   reader without recursively loading nested Subsystems.
2. Define the explicit serialized-prefix-to-`MetadataKind` mapping above.
3. Add a deterministic pending membership observation type with complete source
   context.
4. Resolve observations only after all top-level metadata nodes and flat
   Subsystem nodes exist.
5. Add canonical source identifiers and resolved edge provenance.
6. Emit and deterministically deduplicate Includes edges.
7. Replace broad Includes validation with the first-slice endpoint rule.
8. Add focused parser, resolution, graph, validation, query, determinism, and
   integration tests.
9. Prove dependency and Impact traversal remain unchanged.
10. Update architecture and Coverage status only after production evidence
    passes full workspace validation.

## Coverage Registry completion criteria

`semantic_edge.includes` may transition from `DeclaredOnly` to `Supported` only
after the later implementation provides all of the following evidence:

- `EdgeKind::Includes` remains declared;
- this accepted semantic contract is referenced by current architecture;
- top-level Subsystem `<content>` extraction exists in the production EDT path;
- the exact first-slice prefix allowlist is implemented;
- exact kind-and-name resolution is implemented;
- the canonical endpoint direction is emitted;
- standard edge identity and deterministic deduplication are proven;
- deterministic resolved provenance contains every required context field;
- precise endpoint validation and negative endpoint tests exist;
- the real-format `TestObject` fixture proves Document and metadata Role targets;
- negative resolution tests cover every required failure class;
- repeated-build and duplicate-observation tests prove stable graph and
  provenance output;
- `FileSystemEdtSemanticGraphBuilder` integration proves production emission;
- generic outgoing, incoming, and all-edge queries expose the emitted edge;
- dependency traversal and Impact Analysis remain unchanged and exclude
  Includes;
- full workspace validation passes;
- Coverage changes only this capability and recalculates counters
  deterministically.

Architecture documentation alone is not production evidence. This task leaves
`semantic_edge.includes` as `DeclaredOnly`. Current counts remain EDT 3 High and
44 Medium gaps, and combined graph 3 High and 45 Medium gaps. If no other
registry state changes, the later transition should reduce the EDT and combined
High counts by exactly one and leave Medium counts unchanged; the implementation
task must verify rather than assume those calculated values.

## Deferred scope

The following remain explicitly deferred:

- nested Subsystem discovery;
- `<subsystems>` and `<parentSubsystem>` hierarchy semantics;
- transitive Subsystem membership;
- `<content>` targets whose prefix is not in the first-slice allowlist;
- Subsystem-to-Subsystem membership;
- access-profile-to-role membership;
- role hierarchy or role composition;
- runtime `AccessGroupProfiles` catalog data;
- configuration inventory lists;
- command-interface navigation;
- broad metadata coverage expansion;
- source spans until source-specific span support exists;
- production parsing, resolution, graph emission, validation, fixtures, tests,
  and Coverage transition.

## Consequences

- Includes has one canonical direct-membership meaning for the accepted EDT
  source.
- Subsystem membership is distinguishable from ownership, generic references,
  access grants, and dependencies.
- Metadata Role targets cannot be confused with flat access-control Role nodes.
- The existing graph identity, resolution index, provenance aggregation, and
  generic Query API can support the later implementation.
- Existing production graphs, validation, dependency traversal, Impact
  Analysis, Coverage status, and Coverage counts remain unchanged by this ADR.
