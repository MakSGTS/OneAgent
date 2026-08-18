# ADR-0026: Semantic Index Boundary

## Status

Accepted

## Context

Sprint 4 introduces the Semantic Index after the typed graph and Semantic
Coverage work completed in Sprints 2 and 3. The repository already has three
related surfaces:

- `SemanticGraph` owns canonical nodes and edges in deterministic collections;
- `SemanticGraphQuery` provides read-only graph queries, but name, kind,
  adjacency, and edge-identity queries currently scan graph storage;
- `SemanticResolutionIndex` derives exact-name and ownership lookup structures
  for semantic reference resolution.

Creating an unrelated second graph, embedding source-specific EDT state in the
index, or adding a competing resolution authority would split semantic truth.
Conversely, treating the existing scan-based query facade as the completed
index would leave the incoming and outgoing index requirement from Semantic
Model 2.0 unresolved.

Sprint 5 owns incremental index maintenance. Runtime services, persistence,
protocol transports, and IDE integration belong to later versions. Sprint 4
therefore needs a source-independent snapshot boundary that can be implemented
and validated without pulling later concerns forward.

## Decision

The Semantic Index is a deterministic, read-only, derived view of one immutable
`SemanticGraph` snapshot.

The graph remains the canonical owner of semantic facts, identities,
provenance, and validation. Building or querying the index must not add, remove,
normalize, or reinterpret graph facts. An index can retain references into its
source snapshot or deterministic identifiers that resolve back to that
snapshot; it must not create a second canonical node or edge model.

### Required first slice

Sprint 4 must provide deterministic lookup structures for:

- node identity;
- exact canonical node name;
- node kind;
- stable edge identity;
- edge kind;
- outgoing adjacency by node and by node plus edge kind;
- incoming adjacency by node and by node plus edge kind;
- containment ownership and owned-child lookup required by current resolution.

Results must preserve the deterministic ordering already promised by
`SemanticGraphQuery`. Missing keys return empty results or the existing typed
resolution errors, as appropriate; they must not produce placeholder facts.

### Existing API relationship

Sprint 4 must reuse or compose the lookup logic currently owned by
`SemanticResolutionIndex`. It must not introduce an independent name or
ownership resolution policy. The implementation may make
`SemanticResolutionIndex` a compatibility facade over shared index data, or
have both public facades delegate to one internal index representation.

`SemanticGraphQuery` remains the source-independent read API. Its supported
results and traversal semantics must remain compatible while eligible scan
operations are backed by the index. Any public API change requires a separate
compatibility decision and a complete consumer audit.

### Lifecycle

The first slice is built from a complete graph snapshot. A graph mutation makes
that derived index stale; callers must build a new snapshot index. Incremental
updates, change invalidation, and structural sharing are explicitly deferred to
Sprint 5.

### Excluded from Sprint 4

- EDT, Designer, filesystem, Git, BSL-parser, or workspace-specific state;
- mutation APIs and incremental invalidation;
- on-disk serialization, cache formats, and cache migration;
- HTTP, CLI, MCP, LSP, VS Code, or Runtime service contracts;
- fuzzy, tokenized, ranked, or full-text search;
- new node kinds, edge kinds, resolution rules, or semantic inference;
- a performance percentage or latency target without a reproducible benchmark
  baseline.

## Acceptance evidence

Sprint 4 is complete only when focused tests prove:

1. indexed results are equivalent to the current public graph-query and
   resolution results for representative and empty graphs;
2. all required lookup dimensions are covered, including incoming and outgoing
   adjacency;
3. duplicate names and invalid ownership states retain current observable and
   typed behavior;
4. construction and result ordering are deterministic across insertion orders;
5. the existing Semantic Coverage, Query, Validation, Diff, Impact, and EDT
   integration test suites remain green;
6. a Sprint 4 integration review records the implemented boundary and any
   deliberately deferred optimization.

## Consequences

- There is one semantic authority: `SemanticGraph`.
- Query and resolution can share deterministic indexes without sharing
  source-adapter concerns.
- Sprint 4 can close the Semantic Model 2.0 adjacency-index gap without
  prematurely implementing Sprint 5 or Runtime persistence.
- Snapshot rebuild cost remains visible and accepted until incremental indexing
  is designed in Sprint 5.
