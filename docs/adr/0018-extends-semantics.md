# ADR-0018: Extends Semantics

## Status

Accepted

## Context

Semantic Model 2.0 declares `EdgeKind::Extends` as a semantic graph edge. The
graph domain can store, query, diff, validate, and report the edge kind, but the
EDT adapter does not emit it and the repository did not define a production
contract for it.

Without a precise contract, a producer could incorrectly:

- use `Extends` as a synonym for `Contains`, `References`, `DependsOn`, or
  `Calls`;
- infer extension from naming conventions or ownership;
- store reverse extension edges;
- persist transitive inheritance closure;
- emit edges for unresolved textual base names;
- conflate configuration extensions, metadata inheritance, form inheritance,
  BSL declaration overriding, and borrowed metadata into one unvalidated
  relation.

## Decision

`EdgeKind::Extends` represents an explicit, resolved, direct semantic extension
relation between two semantic entities where the source entity augments,
specializes, borrows from, or inherits from the target entity according to a
source-level extension fact.

The canonical stored direction is:

```text
extending entity --Extends--> directly extended base entity
```

Only explicit production facts may create `Extends` edges. A producer must not
derive `Extends` from ownership, references, dependencies, calls, naming
conventions, directory layout alone, or graph topology.

The first production slice is limited to metadata-object extension facts:

```text
derived metadata object --Extends--> base metadata object
```

Both endpoints must be existing `NodeKind::Metadata(kind)` nodes. The first
slice uses only resolved targets inside the current graph and does not create
placeholder nodes for external or missing bases.

## Semantic definition

`A --Extends--> B` means that semantic entity `A` directly extends `B` because a
source artifact explicitly declares `B` as the base, borrowed, original, or
extended entity for `A`.

The relation is direct-only. The graph must store only facts declared by source
artifacts or resolved from explicit source declarations. Transitive ancestors,
inherited member expansion, effective merged metadata, reverse descendants, and
computed closure are queried or analyzed separately and must not be persisted as
`Extends`.

## Non-meaning

`Extends` is not:

- ownership or containment;
- a generic reference;
- a dependency edge;
- an execution call;
- a data-access relation;
- a subsystem inclusion relation;
- an impact relation;
- an unresolved textual mention of a possible base;
- a statement that inherited members have already been materialized;
- a guarantee that the full effective object merge has been computed.

## Relationship to existing edge kinds

### Contains

`Contains` represents structural ownership from owner to child. Ownership does
not imply extension, and extension does not imply ownership.

```text
configuration --Contains--> metadata object
derived metadata object --Extends--> base metadata object
```

These facts may coexist but must be emitted from different production facts.

### References

`References` represents a resolved semantic reference. A reference to a base
entity is not automatically an extension. A producer may emit both `References`
and `Extends` only when the source contains both meaningful facts or when an
accepted producer contract requires preserving the lower-level resolved
reference that proved extension.

### DependsOn

`DependsOn` is a normalized direct dependency. `Extends` can imply impact or
dependency in some analyses, but this ADR does not automatically emit
`DependsOn` from `Extends`. Any such normalization requires a separate accepted
dependency contract.

### Calls

`Calls` is restricted to resolved execution between callable declarations.
Declaration overriding or implementation may be modeled by future declaration
semantics, but BSL calls must not emit `Extends`.

### Ownership

Ownership is queried through `Contains`. `Extends` has no ownership semantics
and must not satisfy single-owner validation.

## Candidate interpretations

| Candidate | Decision |
|---|---|
| Configuration extension object to base configuration object | Future. Requires a configuration-extension node/source model before production emission. |
| Derived metadata object to base metadata object | First production slice. Uses existing metadata object nodes and directly models explicit metadata extension facts. |
| Inherited form to base form | Future. Requires parser support for form inheritance facts and precise `Form` endpoint policy. |
| Inherited declaration to base declaration | Rejected for this edge until a declaration override/implementation contract exists. This may require a different edge kind. |
| Borrowed metadata to original metadata | Included in the first metadata-object slice when represented as an explicit resolved metadata extension fact. |
| Name-based or ownership-derived extension | Rejected. It is not an explicit source fact. |

## Allowed endpoint matrix

The first production slice allows only metadata object to metadata object
extension pairs with the same metadata kind unless a future ADR explicitly
allows cross-kind extension.

| Origin | Source kind | Target kind | Direction | Status |
|---|---|---|---|---|
| EDT metadata object declares a resolved base/original metadata object | `NodeKind::Metadata(kind)` | `NodeKind::Metadata(kind)` | source extends target | First slice |
| Configuration extension declares a base configuration | Future configuration-extension node | `NodeKind::Metadata(MetadataKind::Configuration)` | source extends target | Future |
| Form declares a base form | `NodeKind::Form` | `NodeKind::Form` | source extends target | Future |
| BSL declaration overrides or implements another declaration | `NodeKind::Procedure` or `NodeKind::Function` | `NodeKind::Procedure` or `NodeKind::Function` | not decided | Future ADR |

## Forbidden endpoint matrix

The first production slice forbids:

- `NodeKind::Metadata(a) --Extends--> NodeKind::Metadata(b)` when `a != b`;
- any metadata member source such as `Attribute`, `TabularSection`,
  `Dimension`, `Resource`, `Measure`, `Form`, or `Command`;
- `Procedure` or `Function` endpoints;
- `Module` endpoints;
- `Query` endpoints;
- `Role` or `Subsystem` flat nodes unless a future producer contract defines
  such semantics;
- `Unknown` endpoints;
- missing, unresolved, ambiguous, or external placeholder targets.

## Direction

The stored direction is always from the extending entity to the directly
extended base entity.

The graph must not store reverse edges such as `ExtendedBy`. Reverse traversal
is a query concern.

## Directness and cycles

Stored `Extends` edges are direct-only.

For a chain:

```text
A --Extends--> B
B --Extends--> C
```

the graph must not automatically store:

```text
A --Extends--> C
```

Self-loops are invalid for the first metadata-object slice. Cycles between
distinct metadata objects are invalid when the producer can detect them through
resolved extension edges. Detection may be implemented by graph validation once
the first production slice emits `Extends`.

## Identity and uniqueness

`Extends` uses the standard graph edge identity:

```text
(source_node_id, target_node_id, EdgeKind::Extends)
```

The identity must be:

- deterministic;
- stable across repeated builds;
- independent of discovery order;
- unique per source-target-kind tuple;
- independent from provenance details.

Multiple equivalent source facts for the same canonical extension relation must
produce one edge with deterministic provenance aggregation or one
deterministically selected provenance record until aggregation is explicitly
implemented.

## Provenance

Every emitted `Extends` edge must carry provenance.

For the first metadata-object slice, provenance must identify:

- source EDT descriptor artifact;
- extending metadata object;
- declared base/original metadata object identifier or name;
- extension fact category;
- resolved target metadata object identity;
- producer;
- `FactOrigin::Resolved` or `FactOrigin::Derived`, according to whether the
  producer stores the resolved source fact directly or normalizes it from a
  lower-level parsed fact;
- `ResolutionState::Resolved`;
- deterministic confidence.

The current `Provenance` model can represent this through a stable source
identifier with a path fragment that encodes the extension context. No new
graph-domain provenance type is required for the first slice.

## Resolution policy

Emit `Extends` only when the target resolves to exactly one existing graph node
with the allowed target kind.

For missing targets, ambiguous targets, incompatible target kinds, unsupported
external targets, malformed source values, unsupported parser inputs, and
partial workspaces, the producer must not create an `Extends` edge or a
placeholder node. These cases should remain recoverable diagnostics or skipped
unsupported facts according to the producer's diagnostics policy.

## Validation policy

The graph validator must replace broad acceptance for `EdgeKind::Extends` with
precise endpoint rules when the first production slice is implemented.

The first slice validator rule is:

```text
NodeKind::Metadata(kind) --Extends--> NodeKind::Metadata(kind)
```

with no self-loop. Missing endpoints remain structural errors. Edge provenance
remains required by the existing provenance validation.

Cycle validation for `Extends` is required before the first production slice can
be marked complete if the source domain allows declaring cycles. If cycle
detection is not feasible at extraction time, graph validation must own the
cycle invariant.

## Coverage Registry completion criteria

`semantic_edge.extends` may transition to `Supported` only after all of the
following evidence exists:

- `EdgeKind::Extends` is declared;
- this ADR is accepted and referenced by Semantic Model 2.0 documentation;
- production parser exposes an explicit metadata-object extension fact;
- production builder resolves the target through the existing graph resolution
  path or an accepted equivalent;
- production builder emits `EdgeKind::Extends`;
- emitted edge uses the standard edge identity;
- emitted edge carries deterministic provenance;
- graph validation has precise endpoint rules for the first slice;
- positive integration test proves source node, target node, edge direction,
  identity, provenance, and coexistence with neighboring edges;
- negative test proves unresolved, ambiguous, unsupported, or incompatible
  source facts do not emit placeholder edges;
- duplicate and repeated-build tests prove deterministic output;
- Coverage Registry transition changes only `semantic_edge.extends`;
- High gap count decreases by exactly one and Medium gap count remains
  unchanged, unless the registry-calculated values differ for a documented
  reason.

This architecture task must not change the current capability status or
coverage counters.

## Minimal production slice

The smallest implementation-ready slice after this ADR is:

| Production fact | Source `NodeKind` | Target `NodeKind` | Direction |
|---|---|---|---|
| Explicit EDT metadata object base/original/borrowed-object relation with a resolved in-graph target | `NodeKind::Metadata(kind)` | `NodeKind::Metadata(kind)` | derived object --Extends--> base object |

The slice requires parser support before graph emission.

## Implementation prerequisites

1. Add a typed EDT descriptor field for explicit metadata object extension
   facts.
   - Reason: current descriptor readers extract UUID, name, synonym, children,
     modules, and type references, but not base/original metadata facts.
   - Affected subsystem: `oneagent-edt` metadata object reader.
   - Follow-up task: parse the exact EDT XML element or attribute representing
     the base/original metadata object.
2. Add realistic EDT fixtures containing a valid metadata-object extension
   relation and at least one unresolved or unsupported relation.
   - Reason: Coverage cannot close from synthetic graph-only mutation.
   - Affected subsystem: `oneagent-edt` tests.
   - Follow-up task: add minimal production fixtures or inline realistic test
     projects.
3. Add target resolution for metadata-object extension facts.
   - Reason: `Extends` requires resolved target nodes and must not create
     placeholders.
   - Affected subsystem: EDT semantic graph builder.
   - Follow-up task: collect pending extension facts and resolve them after
     metadata object nodes exist.
4. Add precise graph validation for the first endpoint matrix.
   - Reason: broad `Extends` acceptance is insufficient once production emits
     the edge.
   - Affected subsystem: `oneagent-graph` validation.
   - Follow-up task: allow only same-kind metadata object extension pairs and
     reject unrelated endpoints.
5. Transition Coverage Registry only after production evidence exists.
   - Reason: architecture alone is not production support.
   - Affected subsystem: `oneagent-edt` coverage registry.
   - Follow-up task: mark `semantic_edge.extends` supported with representative
     tests after implementation.

## Consequences

- `Extends` gains a precise architecture contract.
- Future implementation can close exactly one capability without redesigning
  semantics during coding.
- Existing `Contains`, `References`, `DependsOn`, and `Calls` behavior remains
  unchanged.
- Coverage counters remain unchanged until production support exists.
- The first implementation must add parser support before emitting graph edges.

## Rejected alternatives

1. Treating `Extends` as ownership is rejected because ownership is already
   represented by `Contains`.
2. Treating any resolved reference as `Extends` is rejected because references
   have broader semantics.
3. Treating `Extends` as `DependsOn` is rejected because dependency is a
   normalized impact-oriented relation, while extension is a source-declared
   specialization relation.
4. Treating calls or declaration references as `Extends` is rejected until a
   separate declaration override contract exists.
5. Emitting reverse `ExtendedBy` edges is rejected because reverse traversal is
   provided by graph queries.
6. Persisting transitive extension closure is rejected because it duplicates
   traversal results and complicates incremental updates.
7. Creating placeholder nodes for missing bases is rejected because the current
   first slice requires resolved in-graph targets.
8. Using configuration-extension semantics as the first slice is deferred
   because the current model lacks a dedicated configuration-extension node and
   EDT parser contract.

## Future work

- Define configuration-extension nodes and extension-to-base configuration
  semantics.
- Parse EDT metadata object extension/base/original metadata facts.
- Add form inheritance semantics for `NodeKind::Form` when form parser support
  exists.
- Decide whether BSL declaration overriding belongs to `Extends`, a future
  `Overrides` relation, or another declaration edge.
- Decide whether `Extends` should participate in Impact Analysis after
  production evidence exists.
- Add precise validation and production integration tests for each future slice.
