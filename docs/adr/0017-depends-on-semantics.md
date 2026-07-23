# ADR-0017: DependsOn Semantics

## Status

Accepted

## Context

Semantic Model 2.0 declares `EdgeKind::DependsOn` as a semantic graph edge.
The graph domain can store, query, diff, validate, and propagate impact through
the edge kind, but the EDT adapter currently emits only `Contains`,
`References`, and `Calls`.

`DependsOn` therefore needs a production contract before EDT can emit it
without conflating dependency, reference, call, ownership, and impact semantics.

## Problem

OneAgent needs a normalized dependency relation for consumers that do not want
to interpret every lower-level fact independently. At the same time, the graph
must preserve the original facts that prove the dependency.

Without a precise contract, a producer could incorrectly:

- duplicate every `Calls` or `References` edge as `DependsOn`;
- store transitive dependency closure;
- store reverse impact edges;
- emit dependencies for unresolved or textual-only mentions;
- lose provenance for the source fact that proves the dependency.

## Decision

`EdgeKind::DependsOn` is a materialized normalized direct semantic dependency.

The stored direction is:

```text
dependent --DependsOn--> dependency
```

Only resolved production facts may create `DependsOn` edges. Producers must keep
the original lower-level fact, such as `Calls` or `References`, when that fact
has its own semantic meaning.

The first EDT implementation slice is limited to resolved metadata type
dependencies already discovered by the metadata reference pipeline.

## Semantic definition

`A --DependsOn--> B` means that semantic entity `A` has a direct semantic
dependency on semantic entity `B`, proven by a resolved production fact, such
that a meaningful change to `B` may require re-analysis of `A`.

A stored `DependsOn` edge is not:

- a transitive dependency;
- an unresolved reference;
- an ownership relation;
- a call relation;
- a generic reference;
- an impact relation;
- an incidental textual mention.

## Direction

The canonical stored direction is from dependent to dependency.

Impact traversal uses the reverse direction:

```text
changed dependency
    <- reverse DependsOn traversal -
potentially impacted dependents
```

The graph must not store a separate reverse edge for impact.

## Stored versus derived

`DependsOn` is stored as a materialized normalized relation derived from
resolved lower-level production facts during graph construction.

This keeps dependency queries simple and deterministic while preserving the
lower-level facts that explain why the dependency exists. `Calls` remains the
execution relation between resolved callable declarations. `References`
remains the resolved semantic reference relation. `DependsOn` does not replace
either edge kind.

## Direct versus transitive

Stored `DependsOn` edges represent direct dependencies only.

Transitive dependencies are computed by graph traversal. The graph must not
persist transitive closure. Cycles are allowed when domain semantics permit
mutual dependencies. Traversal and impact analysis must deduplicate visited
nodes deterministically.

## Relationship to Calls

`A --Calls--> B` is logically a dependency candidate, but the first EDT
`DependsOn` implementation must not automatically duplicate every `Calls` edge.

`Calls` is restricted to resolved `Procedure` and `Function` endpoints and is
already part of dependency and impact queries. A later BSL dependency slice may
materialize selected resolved calls as `DependsOn` after it defines whether the
dependency source is the calling declaration, owning module, or owning metadata
object.

## Relationship to References

`A --References--> B` is not automatically equivalent to `A --DependsOn--> B`.

Only resolved references with dependency semantics may produce `DependsOn`.
Unresolved references, ambiguous references, incompatible-kind references,
external symbols, and incidental textual mentions must not emit `DependsOn`.

The first EDT implementation may use resolved metadata type references from
metadata members because they represent a direct type dependency from the member
to the referenced metadata object.

## Relationship to Contains

Ownership alone does not imply dependency.

`owner --Contains--> member` must not automatically become
`owner --DependsOn--> member` or `member --DependsOn--> owner`. Ownership is
represented by `Contains` and queried through ownership navigation.

## Initial implementation slice

The first implementation slice that may close `semantic_edge.depends_on` later
is:

- source: the existing EDT metadata reference extraction pipeline;
- origin: resolved metadata member type reference;
- lower-level fact: existing `References` edge;
- normalized fact: additional `DependsOn` edge from the member to the referenced
  metadata object;
- producer stage: EDT semantic graph builder after successful metadata
  reference resolution;
- no BSL call duplication;
- no query data-access dependency;
- no ownership dependency.

Candidate origin classification:

| Candidate origin | Classification |
|---|---|
| Resolved module call | Planned for later BSL dependency phase |
| Resolved semantic metadata reference | Included in the first implementation |
| Resolved query data-source access | Planned for future Data Access Graph |
| Form or command binding | Planned for later UI/binding phase |
| Type dependency | Included when represented by resolved metadata member type references |
| Subsystem membership | Represented by `Contains`, not by `DependsOn` |
| Ownership | Represented only by `Contains` |

## Allowed source/target matrix

The first implementation slice uses only existing `NodeKind` variants.

| Origin | Source kind | Target kind | Stored as `DependsOn` |
|---|---|---|---|
| EDT attribute type reference | `Attribute` | `Metadata(...)` | Yes |
| EDT register dimension type reference | `Dimension` | `Metadata(...)` | Yes |
| EDT register resource type reference | `Resource` | `Metadata(...)` | Yes |
| Resolved BSL call | `Procedure` or `Function` | `Procedure` or `Function` | Future |
| Static query data-source access | `Query` | `Metadata(...)` | Future |
| Form or command binding | `Form` or `Command` | Binding target kind | Future |
| Subsystem membership | `Subsystem` or `Metadata(...)` | `Metadata(...)` | No |
| Ownership | Any owner kind | Any owned kind | No |

The first implementation must not use wildcard source or target rules.

## Resolution rules

A stored `DependsOn` edge must have a resolved target node.

Unresolved references, ambiguous resolution, incompatible target kinds, external
platform symbols, built-in functions, missing metadata objects, and partial
workspaces do not create placeholder dependency targets. They remain represented
by existing diagnostics, reference statistics, or future external-symbol
modeling.

## Identity

`DependsOn` uses the standard graph edge identity:

```text
(source_node_id, target_node_id, EdgeKind::DependsOn)
```

Multiple evidence facts for the same canonical dependency support the same edge
identity. Distinct source-target pairs remain distinct edges.

## Provenance

Every emitted `DependsOn` edge must carry provenance.

For the first implementation slice, provenance must identify:

- the source EDT descriptor artifact;
- the source metadata object;
- the source metadata member;
- the resolved reference role and target identity;
- the producer that normalized the dependency;
- `FactOrigin::Derived`;
- `ResolutionState::Resolved`.

The current graph model stores provenance as a vector on a canonical edge.
Therefore multiple evidence facts for one canonical dependency must be
represented as one edge with deterministically ordered aggregated provenance.
If a future source cannot aggregate provenance deterministically, that source
must define a prerequisite evidence model before emitting `DependsOn`.

## Duplicate handling

Producers must ensure:

- repeated parsing does not duplicate a dependency edge;
- repeated builds produce identical edge identities;
- traversal order does not affect output;
- multiple equivalent facts produce deterministic provenance ordering;
- distinct source-target pairs remain distinct.

## Validation constraints

The later implementation task should replace the current broad acceptance for
the first slice with additive endpoint validation:

- `Attribute --DependsOn--> Metadata(...)`;
- `Dimension --DependsOn--> Metadata(...)`;
- `Resource --DependsOn--> Metadata(...)`.

Missing endpoint nodes remain rejected by graph insertion and validation.
`DependsOn` self-dependencies are not valid for the first slice because a
metadata member cannot have the same node identity as a metadata object target.
Cycles between distinct nodes are allowed. Emitted edges must have provenance.
Duplicate behavior follows the standard edge identity.

## Coverage completion criteria

`semantic_edge.depends_on` may transition to `Supported` only after the later
implementation task proves the first slice through the real EDT pipeline.

Required evidence:

- `EdgeKind` declaration exists;
- modeled semantics is documented by this ADR;
- endpoint validation rule exists for the first slice;
- EDT production extraction source exists;
- EDT production graph builder emits `EdgeKind::DependsOn`;
- stable identity uses the standard edge identity;
- provenance is attached;
- positive focused test exists;
- negative test exists for unresolved or non-dependency sources;
- integration test exists through `FileSystemEdtSemanticGraphBuilder`;
- duplicate and repeated-build determinism test exists;
- Coverage Registry remains deterministic and transitions only this capability.

The architecture task must not change the current capability status.

## Impact analysis policy

`DependsOn` participates in dependency and impact traversal as a dependency edge.

Direct impact follows reverse traversal from changed dependency to direct
dependents. Transitive impact is computed by bounded traversal over dependency
edges and must deduplicate cycles deterministically.

`Calls` and `References` continue to participate directly in first-version
dependency and impact queries. They are not normalized away through
`DependsOn`. Risk ranking, dependency strength, weights, and impact scores are
outside this edge contract.

## Query/Data Access Graph boundary

Query dependencies are excluded from the first implementation.

The existing Query slice preserves static query text but intentionally does not
emit `Reads`, `Writes`, or `DependsOn`. Future Data Access Graph work will parse
query-language sources and may produce dependencies involving `Query` and
`Metadata(...)` nodes. This ADR does not design the Data Access Graph.

## Consequences

- `DependsOn` gains a precise implementation-ready contract.
- The first production slice can close exactly one Coverage Registry capability.
- Existing `Calls`, `References`, and `Contains` behavior remains unchanged.
- Consumers can distinguish original facts from normalized dependencies.
- Future dependency origins can be added without changing the meaning of
  existing `DependsOn` edges.

## Rejected alternatives

1. `DependsOn` as a synonym for `References` is rejected because not every
   reference is a dependency.
2. `DependsOn` as a synonym for `Calls` is rejected because calls are execution
   facts with callable endpoint constraints.
3. Automatic duplication of every `Calls` edge is rejected for the first slice
   because the owning dependency level is not yet defined.
4. Automatic duplication of every `References` edge is rejected because
   reference strength differs by source and role.
5. `DependsOn` as a wildcard relation between arbitrary nodes is rejected
   because it cannot be validated precisely.
6. Persisting transitive closure is rejected because traversal already computes
   closure and persisted closure would be nondeterministic under incremental
   updates.
7. Storing reverse `Impacts` edges is rejected because reverse traversal over
   dependency edges already represents impact direction.
8. Emitting unresolved `DependsOn` edges to placeholder nodes is rejected
   because the current graph has no external-symbol placeholder contract.
9. Delaying the entire contract until the full Data Access Graph exists is
   rejected because metadata type dependencies can be modeled now without query
   semantics.
10. Removing `EdgeKind::DependsOn` is rejected because the graph, query, impact,
    and coverage models already reserve it for normalized dependency facts.

## Follow-up implementation tasks

1. Completed: implement the first EDT slice for resolved metadata member type
   references.
2. Completed: add precise validator rules for the first slice.
3. Completed: add focused and integration tests for emission, provenance, deduplication,
   repeated-build determinism, and Coverage Registry transition.
4. Define a separate BSL dependency contract before materializing call-derived
   `DependsOn` edges.
5. Define the Data Access Graph contract before emitting query-derived
   `DependsOn` edges.
