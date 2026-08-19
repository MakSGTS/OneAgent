# ADR-0027: Incremental Semantic Index Maintenance

## Status

Accepted

## Context

Sprint 4 completed the boundary defined by
[ADR-0026](0026-semantic-index-boundary.md). `SemanticGraph` remains the only
canonical owner of semantic facts, while the crate-internal `SemanticIndex`
provides deterministic complete-snapshot lookups for node identity, exact name,
node kind, stable edge identity, edge kind, adjacency, and containment.

The current index stores borrowed canonical nodes and edges. Those references
cannot be retained across distinct previous and current graph lifetimes.
`SemanticGraphDiff` already provides the canonical directional description of
an `old -> new` graph transition with owned old/new node and edge snapshots.
Sprint 5 must reuse that contract, retain unaffected derived lookup state, and
preserve Query and Resolution compatibility without creating source-specific
events, a mutable public index, or a second semantic model.

## Decision

Incremental maintenance is a crate-internal, functional transition between two
complete canonical `SemanticGraph` snapshots. It produces one complete derived
state for the current snapshot or a typed failure. It never mutates or repairs
either graph and never publishes a partially updated index.

### Canonical change input

The canonical input consists of:

1. one accepted previous `SemanticGraph` snapshot;
2. one accepted current `SemanticGraph` snapshot;
3. the directional `SemanticGraphDiff::between(previous, current)` result.

The normalization layer may accept a supplied diff only when it is exactly
equal to a newly derived diff for the same borrowed snapshot pair. A mismatched
diff, wrong snapshot pair, or manually authored operation stream is rejected.
Filesystem, Git, workspace, EDT, and producer events are not graph/index
boundary inputs.

The normalized batch borrows its previous and current graphs and owns only the
deterministic change records needed by the index transition. This pairing gives
the lifecycle an exact process-local freshness check with `ptr::eq`; pointer
identity is never part of semantic identity, ordering, serialization, or
observable output.

### Derived state representation

The retained portion of the index uses owned stable identifiers and lookup
keys, not references into the previous graph. Canonical nodes and edges are
always resolved from the graph snapshot paired with the state.

The owned derived state contains the following deterministic structures:

- node identities and exact-name and node-kind membership by `EntityId`;
- stable edge identity and edge-kind membership by `EdgeId`;
- incoming and outgoing adjacency, including kind-filtered membership;
- containment owner edges, all owners, children, child kinds, and
  owner-plus-exact-child-name membership.

Values within a lookup are ordered by `EntityId` or stable `EdgeId`, matching
Sprint 4. Duplicate exact names, multiple owners, self-loops, cycles, and other
canonical invalid states remain present. The state may clone or structurally
share unaffected owned entries, but this is an implementation detail and no
allocation or performance target is accepted.

A borrowed index view pairs:

```text
&SemanticGraph current snapshot
    + owned/borrowed SemanticIndexState for that same snapshot
```

The view returns only canonical `GraphNode` and `GraphEdge` references resolved
from the paired graph. A missing resolution is an internal stale-state error,
not a placeholder fact.

### Normalized operation model

Normalization produces unique typed operations in this total phase order:

1. remove edges;
2. remove old node projections;
3. add new node projections;
4. add edges;
5. refresh same-identity node or edge content that does not change lookup keys.

Operations within a phase are ordered by stable `NodeId` or `EdgeId`. A node
semantic-content modification is one typed replacement with old and new
snapshots; application projects it through phases 2 and 3. An edge endpoint or
kind change is already a removed edge plus an added edge under existing Diff
identity. Same-identity provenance-only edge changes remain typed refreshes and
do not change identity, kind, adjacency, or containment membership.

`SemanticGraphDiff` owns uniqueness and collision behavior. Repeated or
duplicate graph insertions are collapsed by canonical graph storage before the
diff. Changes that cancel between the two accepted snapshots produce no
operation. Contradictory raw operations are not a supported input; a supplied
diff that disagrees with the canonical snapshot transition is rejected.
Stable-id collisions with different canonical identity components are rejected.

### Invalidation rules

Node operations invalidate:

- identity membership for addition or removal;
- the old and new exact-name buckets when the name changes;
- the old and new node-kind buckets when the kind changes.

Payload, provenance, and other same-identity content changes are retained in
the normalized batch but do not change a lookup dimension. Returned objects
still expose the current canonical content because the view resolves through
the current graph.

Edge operations invalidate:

- stable edge identity and edge-kind membership;
- outgoing and incoming adjacency for both endpoints;
- both kind-filtered adjacency keys;
- every containment key when the edge kind is `Contains`.

Containment insertion and removal derive owner and child name/kind keys from
the canonical graph snapshot appropriate to that operation. A node name or
kind replacement also rekeys containment child-name or child-kind membership
for every retained incoming `Contains` edge.

A removed node requires removal of every incident old edge in the same
canonical transition. Normalization rejects a batch when an incident edge
would remain or an added edge endpoint is absent from the current graph. Edge
removal precedes node removal; node addition precedes edge addition. These
preparation phases are never externally visible.

### Lifecycle, freshness, and atomic publication

An accepted incremental state is paired with exactly one graph snapshot. A
transition checks that:

- the previous state is paired with the batch's previous graph instance;
- the batch diff matches its previous/current snapshots;
- all old/new entities required by operations exist in the appropriate graph;
- incident-edge and endpoint dependencies are complete;
- the finished derived state is internally consistent with the current graph.

Application clones or prepares state privately, applies the complete normalized
batch, validates the result, and only then returns a new state paired with the
current graph. The previous state remains unchanged and reusable after any
failure.

Applying the same batch again to the same accepted previous state is a
deterministic retry and produces an equivalent current state. Applying it to an
already-current or unrelated state is a typed stale-base failure. A newly
normalized `current -> current` transition is the accepted idempotent empty
operation. Wrong-target and mismatched-diff inputs fail before mutation.

Callers may fall back to a clean complete-snapshot rebuild after any typed
incremental failure. Fallback does not convert an invalid graph into an accepted
one and does not alter the failed previous state.

### Query and Resolution compatibility

The existing public constructors and signatures remain unchanged:

- `SemanticGraph::query()` and `SemanticGraphQuery::new` remain `const`;
- `SemanticGraph::resolution_index()` and `SemanticResolutionIndex::new`
  remain available;
- borrowed return values, typed resolution errors, candidate ordering, Query
  traversal, and missing/invalid behavior remain compatible.

Clean constructors build an owned state from their graph and remain the full
rebuild oracle. Crate-internal lifecycle constructors may borrow an accepted
incremental state paired with the same graph. Shared references preserve Query
covariance; no lazy cell containing graph-borrowing values is introduced.
Query and Resolution delegate to the same lookup implementation in both modes.

Validation continues to inspect canonical graph facts directly. Diff and build
Diff remain directional complete-snapshot comparisons. Impact pairs the Query
view for each corresponding previous/current graph. Coverage and EDT do not own
index invalidation policy.

### Full-rebuild equivalence

For every supported transition and after every step of a supported sequence,
the incremental result must equal a clean state built from the same current
graph for every Sprint 4 lookup dimension. Query primitive and derived results,
Resolution successes and typed failures, ordering, and invalid-state visibility
must also match.

The oracle constructs the current graph independently and performs a clean
index build. It must not reuse incremental invalidation helpers to calculate
expected results. The comparison key universe includes keys from both previous
and current graphs plus explicit missing keys so stale entries are detectable.

Required evidence covers empty and no-op transitions; node add, remove, name,
kind, payload, and provenance modification; edge add, remove, provenance
refresh, endpoint/kind replacement; adjacency and containment changes; node
deletion with incident edges; duplicate names; multiple owners; invalid
ownership; self-loops; cycles; mixed batches; reversed construction; repeated
retry; multi-step sequences; stale failure; and failure followed by retry.

### Complexity expectations

`SemanticGraphDiff` remains a complete-snapshot comparison. Normalization is
proportional to the diff plus deterministic ordering. Application updates only
keys affected by normalized operations, with ordered-map/set costs, while the
implementation may clone retained state to preserve atomicity. Clean rebuild
remains available. Sprint 5 makes no latency, allocation, percentage, or
asymptotic improvement claim without a separate reproducible benchmark.

## Rejected alternatives

- **Retain borrowed previous nodes or edges.** Rejected because references from
  the previous graph cannot become canonical references into the current graph.
- **Make `SemanticGraph` or source adapters own invalidation.** Rejected because
  it mixes canonical fact ownership or source-specific events with derived
  index policy.
- **Expose a mutable public index or change Query construction.** Rejected
  because no public API change is required and Sprint 4 compatibility remains
  accepted.
- **Use manually ordered raw operations as authority.** Rejected because they
  duplicate Diff semantics and make duplicates and contradictions ambiguous.
- **Always rebuild and call it incremental.** Rejected as the implementation
  path because it does not retain unaffected derived entries; clean rebuild is
  retained only as the oracle and fallback.
- **Persist or serialize the derived state now.** Rejected because persistence,
  cache formats, invalidation across processes, and migration belong to later
  accepted work.

## Consequences

- `SemanticGraph` remains the only semantic authority.
- The incremental boundary is source-independent and reuses
  `SemanticGraphDiff` rather than adding a competing change vocabulary.
- Owned lookup membership can survive the previous graph lifetime, while
  canonical values always come from the current graph.
- Atomic functional transitions make failure and retry explicit and testable.
- Public Query and Resolution compatibility can be preserved.
- Full-rebuild equivalence is required for every supported change class and
  sequence.

## Deferred scope

- persistent or serialized caches and cross-process snapshot identity;
- Runtime, async publication, service containers, and orchestration;
- filesystem watching, Git adapters, workspace change adapters, and EDT-owned
  change generation;
- transport, CLI, HTTP, MCP, LSP, IDE, and editor integration;
- structural-sharing guarantees and benchmark-backed optimization;
- new semantic facts, inference, identity, Query, Resolution, Validation,
  Impact, or Coverage policy.
