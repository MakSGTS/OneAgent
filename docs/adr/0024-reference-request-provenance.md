# ADR-0024: Reference Request Provenance

## Status

Accepted

## Context

ADR-0008 requires a detected semantic reference to retain its source node,
textual reference, category, resolution state, candidates when available, and
provenance even when no graph edge can be emitted. The graph domain currently
provides `SemanticReference`, `ResolutionError`, `SemanticDiagnostic`,
`SemanticReferenceStatistics`, and provenance-backed resolved edges, but it
does not expose the reference request that connects collection to those
outcomes.

The EDT adapter currently has several request-like representations with
different source contracts:

| Family | Collection or pending representation | Current terminal representation |
|---|---|---|
| Metadata member type | private `PendingMetadataReference` | `References` and `DependsOn`, or a typed diagnostic, plus statistics |
| BSL call | `BslCall` and private resolver results | `Calls`, or a typed diagnostic, plus statistics |
| Query source | `QuerySourceOccurrence` and private resolution outcomes | `Reads`, or a typed diagnostic, plus statistics |
| BSL write | private `EdtWritesCandidate` and resolution outcomes | `Writes`, or a typed diagnostic, plus statistics |
| Role protected resource | role-right declarations and private observations | `References` and `Grants`, or a typed diagnostic, plus statistics |
| Subsystem content | private `PendingSubsystemContentObservation` | `Includes`, or a typed diagnostic, plus statistics |
| Metadata extension | private `PendingMetadataExtension` | `Extends`, or no emitted fact when resolution is not accepted |

These representations are not interchangeable. They carry parser- and
adapter-specific fields, and some include rejected syntax or relation evidence
that is not a semantic reference request. They must not be moved wholesale
into `oneagent-graph`.

`PendingMetadataReference` preserves descriptor path, metadata owner, source
member, reference role, expected metadata kind, and target name. Provenance is
currently constructed only when the adapter emits a resolved edge or a typed
diagnostic. Consequently the graph-domain and EDT
`SemanticProvenanceCapability::ReferenceRequest` entries are both
`PartiallySupported`: the concept is modeled, but there is no public request
value with provenance attached at collection time.

Diagnostics retain many terminal fields, but they exist only for failed
outcomes. Resolved edges retain target identity and provenance, but not the
original typed request as one queryable value. Statistics retain counts only.
None of these is a complete substitute for the request lifecycle required by
ADR-0008.

## Decision

Introduce a public, source-independent `SemanticReferenceRequest` domain type
in `oneagent-graph` in a later implementation task.

The type is a build-observation value, not a graph node or graph edge. A
deterministically ordered request ledger belongs to the semantic build result
alongside the graph and diagnostics. Source adapters translate accepted
source-specific observations into this type at the point where parsing has
proved that a semantic reference exists.

The first EDT production slice is limited to metadata member type references
currently represented by `PendingMetadataReference`. Other request families
remain private until their own source contracts are mapped explicitly to the
public fields and tested. Acceptance of the public type does not force every
private candidate, rejection, or pending relation into one abstraction.

## Source-independent domain API

The implementation may refine constructor names, but it must preserve this
conceptual model:

```rust
pub struct SemanticReferenceRequest {
    id: SemanticReferenceRequestId,
    source_node: EntityId,
    category: SemanticReferenceCategory,
    reference: SemanticReference,
    expected_kinds: Vec<NodeKind>,
    candidates: Vec<EntityId>,
    state: ResolutionState,
    outcome: SemanticReferenceRequestOutcome,
    provenance: Vec<Provenance>,
}

pub enum SemanticReferenceCategory {
    MetadataType,
    Callable,
    QuerySource,
    WriteTarget,
    ProtectedResource,
    SubsystemMember,
    ExtensionTarget,
}

pub enum SemanticReferenceRequestOutcome {
    Collected,
    Resolved,
    MissingTarget,
    PartialWorkspace,
    AmbiguousTarget,
    IncompatibleTargetKind,
    InvalidOwnerReference,
}
```

The category vocabulary describes semantic intent and must not contain EDT
directory names, XML element names, filesystem paths, or parser-specific
variants. A category may be added only with a defined source contract,
resolution behavior, and projection policy.

`SemanticReference` remains the canonical typed target expression. Raw source
spelling may use `SemanticReference::Raw`; normalized names, node identities,
and owner-scoped forms use the existing typed variants. A parallel textual
target string must not be stored when `SemanticReference` already represents
it.

`source_node` is required. A producer that cannot identify a declared semantic
source node does not yet satisfy this request contract and must keep its
private representation or diagnostic path until that prerequisite exists.

The typed outcome preserves distinctions required by diagnostics and
`SemanticReferenceStatistics`; `ResolutionState` alone cannot distinguish a
missing target from an incompatible target. Constructors derive or validate
the state/outcome pairing, so callers cannot create contradictory values.
`expected_kinds`, `candidates`, and `provenance` are sorted and deduplicated by
constructors or a controlled builder. Callers do not receive mutable access
that could violate deterministic ordering or lifecycle invariants.

## Identity and ordering

`SemanticReferenceRequestId` is derived from the canonical tuple:

```text
(source_node, category, reference, expected_kinds)
```

`expected_kinds` participates after sorting and deduplication. Candidates,
resolution state, typed outcome, and provenance do not participate because they
are mutable semantic content of the same request across resolution or workspace
snapshots.
The identifier must use centralized deterministic encoding and must not depend
on insertion order, collection indexes, process state, filesystem traversal
order, or a target selected by resolution.

Canonical request ordering is by request identifier. Candidate identifiers and
provenance records use their existing deterministic domain order. The complete
value derives or implements equality and ordering consistently with these
rules.

Multiple identical observations for the same identity aggregate into one
request when their state, outcome, expected kinds, and candidates agree. Their
provenance is sorted and deduplicated. Conflicting terminal content for one
identity is a build invariant error, not a last-writer-wins merge. Distinct
source nodes, categories, target expressions, or expected-kind sets remain
distinct requests. Repeated builds over the same workspace must produce
identical request values and order.

## Provenance ownership

The request owns provenance from collection time. The adapter must create the
first `Provenance` value when it converts an accepted source observation into a
`SemanticReferenceRequest`, before resolution begins.

Collection provenance must identify:

- the source artifact or a deterministic source-context identity;
- the source semantic node;
- the producer responsible for reference collection;
- `FactOrigin::Parsed` or `FactOrigin::Declared`, according to the accepted
  source contract;
- confidence justified by that source contract;
- the collection-time resolution state.

Resolution may append resolver provenance. A resolver entry identifies the
resolver producer, uses `FactOrigin::Resolved`, and records the state produced
by that stage. Historical collection provenance is not rewritten. The
request's top-level `state` is the canonical current state; the resolution
value inside each provenance record describes the stage-specific evidence and
must not be interpreted as a competing current state.

The current public `Provenance` type has no source-span field. The metadata
first slice therefore uses its existing deterministic descriptor-and-member
source-context identity without adding an EDT path or span to the graph-domain
request. Before an occurrence-sensitive family such as BSL calls or query
sources migrates, a separate source-independent span/location contract is a
prerequisite. Encoding an adapter path directly in the public request is not an
acceptable substitute.

Resolved edges and diagnostics retain their own provenance because they are
separate emitted facts. They copy or derive the applicable request evidence and
add the emitting producer. They must not manufacture unrelated source context.

## Lifecycle

The canonical lifecycle is:

```text
source-specific observation
    -> accepted public request with collection provenance
    -> deterministic resolution
    -> terminal request state and candidates
    -> zero or one direct edge projection, or zero or one diagnostic projection
    -> statistics derived from terminal requests
    -> optional normalized derived edges
```

Collection creates an `Unresolved` request with no candidates. A source
contract may create `Partial` only when incompleteness is already proven at
collection time. Malformed syntax, unsupported prefixes, ignored textual
mentions, and parser rejections are diagnostics or parser outcomes, not
reference requests, unless a later contract proves a valid semantic target
expression and category.

Resolution produces one terminal state:

| Typed outcome | Request state | Candidate contract | Direct projection |
|---|---|---|---|
| `Resolved` | `Resolved` | exactly the resolved target | the category's resolved edge |
| `AmbiguousTarget` | `Ambiguous` | all compatible targets, sorted and unique | typed ambiguous diagnostic |
| `MissingTarget` | `Unresolved` | empty | typed unresolved diagnostic |
| `PartialWorkspace` | `Partial` | empty or known partial candidates | producer policy may emit an informational diagnostic; no resolved edge |
| `IncompatibleTargetKind` | `Unresolved` | all inspected incompatible targets, sorted and unique | typed incompatible-kind diagnostic with actual-kind evidence |
| `InvalidOwnerReference` | `Unresolved` | the known child or owner candidates required by the category | typed invalid-owner diagnostic |

`Collected` pairs with `Unresolved` and empty candidates before resolution and
is not a terminal outcome.

`ResolutionState::NotApplicable` is not a valid state for an accepted request.
It remains valid for graph facts to which resolution does not apply.

A resolved request must have exactly one candidate compatible with
`expected_kinds`. An ambiguous request must have at least two candidates. A
request in any non-resolved state emits no resolved edge and no placeholder,
Unknown, external, or synthetic target node.

## Incomplete workspaces

Workspace completeness is explicit resolution input. Absence in a complete
workspace is `Unresolved`; absence in an explicitly partial workspace is
`Partial`. Producers must not infer completeness from a missing filesystem
path or silently convert partial absence to success.

Partial requests remain in the ledger for later incremental resolution. Their
identity is stable when a later complete snapshot resolves them. This ADR does
not introduce incremental mutation or persistence; build snapshots remain
immutable and diffs compare the same request identity across snapshots.

## Edges, diagnostics, and statistics

The request ledger is the canonical record of processed semantic references.
Edges, diagnostics, and statistics are projections and must not become
independent competing sources of request truth.

- A `Resolved` metadata type request emits one `References` edge for its
  canonical source-target pair.
- A companion `DependsOn` edge remains a separate derived fact governed by
  ADR-0017. It is not another processed request and does not increment
  reference statistics.
- A failed terminal request may emit one typed diagnostic. The diagnostic
  retains user-facing code, severity, actual kind when applicable, and a copy
  of the request identity fields needed by existing consumers.
- Diagnostic aggregation follows request identity. Duplicate observations add
  provenance to the canonical request and its diagnostic projection rather
  than incrementing the processed-request total.
- `SemanticReferenceStatistics` is derived once from canonical terminal
  request outcomes. Edge insertion and diagnostic insertion do not
  independently increment it. `PartialWorkspace` maps to the existing
  unresolved counter until a separate public statistics outcome is accepted;
  this compatibility mapping is explicit and tested.
- Malformed and unsupported parser outcomes that are not accepted requests may
  continue to contribute their existing statistics through a separately named
  rejected-observation path until a broader reporting contract is designed.
  They must not be counted both as requests and rejections.

The later migration must preserve current observable totals for the accepted
first slice unless repository evidence identifies an existing duplicate-count
bug and a separate behavior change is approved.

## Public exposure and dependency direction

`SemanticReferenceRequest`, its identifier and category belong to
`oneagent-graph`. They may depend on existing source-independent types from
`oneagent-common`, `oneagent-metadata`, and `oneagent-graph`; they must not
contain `PathBuf`, EDT descriptors, XML roles, BSL parser structs, or runtime
types.

The intended dependency direction remains:

```text
oneagent-graph <- oneagent-edt <- oneagent-runtime
```

The graph-domain crate neither reads source files nor performs EDT conversion.

The later public API adds an ordered `reference_requests()` view to
`EdtSemanticGraphBuildResult`. The request ledger is build-level state and is
not inserted into `SemanticGraph`, so `SemanticGraphQuery` continues to query
nodes and edges only. If consumers need request filtering, `oneagent-graph`
provides a separate immutable request-ledger query/view rather than making
requests synthetic graph entities.

`SemanticGraphReport` continues to expose aggregate resolution statistics, but
the request-aware constructor derives them from the ledger. Existing
statistics-based constructors may remain as compatibility APIs during
migration. `SemanticGraphBuildDiff` must later compare added, removed, and
modified requests by stable request identity; a state, candidate, or provenance
change is a modification.

Known future consumers that require migration are:

- `EdtSemanticGraphBuildResult` construction, accessors, report, validation,
  and diff;
- metadata-reference collection, resolution, edge emission, diagnostics, and
  statistics;
- `SemanticGraphReport` provenance and resolution summaries;
- `SemanticGraphBuildDiff` and its summary;
- build-level validation of statistics, diagnostic projections, and request
  invariants;
- graph-domain and EDT Coverage Registry evidence and representative tests.

Existing graph node, edge, diagnostic, report, diff, and query APIs remain
source-compatible until a dedicated implementation task changes them.

## First implementation slice

The first slice converts `PendingMetadataReference` into the public request
ledger without broadening metadata parsing or resolution semantics.

It must:

1. add graph-domain request identity, category, value, invariant validation,
   deterministic aggregation, and immutable query access;
2. extend build reports and diffs without changing graph equality or graph
   node/edge identity;
3. attach collection provenance while the EDT adapter extracts an accepted
   metadata member type reference;
4. resolve requests through the existing exact name-and-kind behavior;
5. derive existing `References`, `DependsOn`, diagnostics, and statistics from
   the terminal request exactly once;
6. expose ordered requests through `EdtSemanticGraphBuildResult`;
7. preserve complete- and partial-workspace behavior without placeholder nodes;
8. add graph-domain unit tests and production-builder EDT integration tests for
   positive, missing, ambiguous, incompatible, duplicate, and repeated-build
   cases;
9. migrate Coverage only after all required evidence exists.

## Deferred request families

BSL calls, query sources, Writes targets, role protected resources, Subsystem
content, and metadata extension targets are deferred. Each family must first
define:

- its public semantic category and target representation;
- a guaranteed source node;
- collection and resolver producers;
- complete versus partial workspace behavior;
- terminal edge and diagnostic projection;
- duplicate aggregation and statistics compatibility;
- focused and production integration evidence.

Private parser candidates and rejected observations remain private. A future
family may use the public request lifecycle without exposing its source-specific
candidate type.

## Coverage completion criteria

Architecture acceptance does not change Coverage status or aggregate counts.

The graph-domain
`SemanticProvenanceCapability::ReferenceRequest` entry may transition to
`Supported` only after:

- the public source-independent type and stable identity exist;
- constructors enforce state, candidate, ordering, and provenance invariants;
- request-ledger query, report, diff, and build validation integration exist;
- positive, negative, duplicate, transition, and determinism tests exist;
- representative tests prove provenance is attached before resolution.

The EDT entry may transition independently only after:

- the production metadata-reference pipeline creates public requests at
  extraction time;
- collection and resolver provenance are observable;
- existing resolved edges, derived dependencies, diagnostics, and statistics
  are produced without double counting;
- missing, ambiguous, incompatible, duplicate, and repeated-build fixtures pass
  through `FileSystemEdtSemanticGraphBuilder`;
- complete and partial workspace outcomes are deterministic;
- the registry transition changes only the EDT ReferenceRequest capability.

The graph-domain entry must not claim EDT production evidence, and the EDT
entry must not become `Supported` merely because the graph-domain type exists.

## Compatibility and migration impact

The implementation is additive at the domain level but changes build-result
construction and the internal source of statistics. Constructors should be
migrated in an ordered change so callers cannot provide contradictory request
ledgers and counters. Temporary compatibility constructors must validate or
clearly document that caller-supplied statistics are legacy input.

No serialization contract currently exists. A future serialized request form
must version category and identity encoding before external persistence is
introduced.

## Rejected alternatives

1. Keep every pending representation private and use diagnostics as the
   canonical unresolved model. Rejected because diagnostics do not represent
   resolved requests and cannot satisfy ADR-0008 lifecycle traceability.
2. Treat resolved edges as the canonical request record. Rejected because
   unresolved, ambiguous, partial, and incompatible requests emit no edge.
3. Infer requests from edges and diagnostics when reporting. Rejected because
   the original category, expected kinds, duplicates, and collection-time
   provenance cannot be reconstructed reliably.
4. Store requests as graph nodes or placeholder target nodes. Rejected because
   requests are build observations, not semantic entities, and placeholder
   nodes would change graph and Query semantics.
5. Move `PendingMetadataReference` into `oneagent-graph`. Rejected because its
   descriptor path and EDT reference role violate the source-independent
   dependency boundary.
6. Generalize all private candidates immediately. Rejected because Calls,
   Reads, Writes, Grants, Includes, and Extends have distinct parsing,
   completeness, and projection contracts.
7. Include candidates, state, or provenance in request identity. Rejected
   because resolution would replace one identity with another instead of
   modifying the same semantic request.
8. Continue incrementing statistics independently in every edge and diagnostic
   branch. Rejected because it permits double counting and contradictory build
   results.
9. Reclassify Coverage during this architecture task. Rejected because no
   production request value or new test evidence exists yet.

## Consequences

- One source-independent value connects collection, resolution, diagnostics,
  resolved edges, reports, and diffs.
- Collection-time provenance becomes observable without importing EDT details
  into the graph domain.
- Unresolved and partial references remain traceable without synthetic graph
  nodes.
- Build-result and report implementations become more explicit, but their
  migration must be coordinated to preserve statistics and compatibility.
- Other request families have a reusable lifecycle while retaining private
  source-specific parsing types.

## Ordered follow-up work

1. Implement graph-domain request identity, category, invariants, ledger query,
   report, diff, and validation integration.
2. Add complete graph-domain unit and determinism evidence.
3. Convert EDT metadata member type collection to public requests with
   collection-time provenance.
4. Derive metadata resolution projections and statistics from terminal
   requests without observable behavior changes.
5. Add complete, partial, failed, duplicate, and repeated-build production
   fixtures.
6. Transition the graph-domain and EDT Coverage entries independently after
   their respective evidence is complete.
7. Evaluate each deferred request family through a separate scoped
   implementation or architecture task.
