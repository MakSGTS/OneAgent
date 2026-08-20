# ADR-0032: Subsystem Hierarchy Semantics

## Status

Accepted

## Context

Sprint 10 extends the completed top-level Subsystem and direct content slice.
The bootstrap-selected repository-local EDT project contains 127 Subsystem
descriptors: 13 at the top level and 114 nested through repeated
`Subsystems/<Name>` directories. The root `.gitignore` excludes this live
reference tree, so implementation completion requires a tracked
provenance-backed reduced fixture. Each
nested descriptor has one UUID, one direct name, one qualified
`parentSubsystem`, and one matching declaration in its immediate parent's
`subsystems` collection. The complete evidence and compatibility inventory are
recorded in
[the source investigation](../architecture/subsystem-hierarchy-source-investigation.md).

[ADR-0020](0020-includes-semantics.md) accepts `Includes` as direct declared
composition membership but deliberately restricts production to top-level
Subsystem content and metadata-object targets. It does not authorize nested
discovery, Subsystem hierarchy endpoints, or a transitive membership query.
The current validator therefore rejects `Subsystem --Includes--> Subsystem`.

The repository needs a bounded hierarchy contract before recursive discovery
can create nodes or edges without conflating composition with configuration
ownership, inferring facts from directory layout alone, persisting transitive
closure, or resolving nested objects by non-unique local name.

## Decision

Nested EDT Subsystems preserve the same two UUID-derived semantic
representations as top-level Subsystems. Direct declared parent-child hierarchy
uses `EdgeKind::Includes` between flat `NodeKind::Subsystem` nodes. Direct
metadata membership continues to use the ADR-0020 endpoint. Transitive metadata
membership is computed by a source-independent query and is never stored as
additional graph edges.

## Canonical semantic statements

Direct hierarchy:

```text
Subsystem Parent --Includes--> Subsystem Child
```

means that the parent's direct `mdclass:Subsystem/subsystems` declaration, the
child's direct `mdclass:Subsystem/parentSubsystem` declaration, and the physical
immediate nested descriptor path agree on the same parent-child relation.

Direct content remains:

```text
Subsystem A --Includes--> Metadata B
```

and retains the exact meaning accepted by ADR-0020.

Transitive membership of Subsystem `A` is the deterministic set of metadata
nodes directly included by `A` or by any direct or indirect child Subsystem
reachable through hierarchy Includes edges. Descendant Subsystem nodes are
traversal intermediates, not transitive metadata-member results.

## Source contract

### Recursive discovery boundary

The accepted artifact family is:

```text
src/Subsystems/<Top>/[Subsystems/<Child>/...]/<LocalName>.mdo
```

Discovery begins only from the existing top-level `src/Subsystems` directory.
At every successfully parsed Subsystem descriptor, recursion follows only the
direct repeated `subsystems` names through the immediate `Subsystems`
directory. Directory enumeration order has no semantic meaning.

Each nested descriptor must satisfy all three projections:

1. its immediate parent directly declares its exact local name through
   `mdclass:Subsystem/subsystems`;
2. the descriptor directly declares exactly one qualified
   `mdclass:Subsystem/parentSubsystem` value naming the complete ancestor path;
3. its physical path is the corresponding immediate
   `Subsystems/<LocalName>` child directory.

Top-level Subsystems must have no direct `parentSubsystem`. A missing,
additional, duplicate, or contradictory projection is a typed fatal hierarchy
source error. The builder emits no partial facts from the inconsistent
Subsystem hierarchy. It does not choose one projection as an implicit repair.

The qualified parent grammar is an odd-length alternating sequence:

```text
Subsystem.<Top>[.Subsystem.<Nested>]...
```

Every prefix must be the exact case-sensitive token `Subsystem`; every name
must be non-empty and valid under the existing `EntityName` contract. Leading
or trailing whitespace, empty components, another prefix, or extra components
are malformed. Resolution uses the complete ancestor path, never global local
name. Duplicate local names under different parents remain valid.

Unreadable directories, absent or multiple `.mdo` descriptors, malformed XML,
wrong roots, invalid UUIDs/names, and project-root escapes reuse typed fatal
filesystem or descriptor errors. Symlink behavior must remain within existing
repository filesystem safety; recursion must not follow a path outside the
project root.

### Nested direct content

Every successfully discovered nested descriptor is passed to the existing
direct content parser. Its direct `<content>` observations use ADR-0020's exact
qualified-token grammar, allowlist, resolution, diagnostics, statistics,
deduplication, and provenance rules.

`Subsystem.<...>` content remains outside the direct content allowlist. The
four repository-observed self-content tokens do not create hierarchy,
metadata-membership, self-loop, placeholder, or inferred facts. They retain the
recognized-but-deferred diagnostic outcome until a separate decision proves a
distinct semantic meaning.

## Identity and representation

Every nested descriptor produces:

- one `NodeKind::Metadata(MetadataKind::Subsystem)` node with the source UUID;
- one flat `NodeKind::Subsystem` node with identity
  `<metadata UUID>:subsystem`.

This is byte-compatible with the existing top-level convention. Hierarchy path,
parent identity, local name, depth, XML order, and provenance do not enter node
identity. Reparenting a descriptor with the same UUID therefore preserves both
node IDs and changes only direct hierarchy edge/provenance state.

Nested metadata Subsystem objects retain the existing configuration-to-metadata
`Contains` relation. The flat Subsystem node has no ownership edge. Hierarchy
uses only Includes and never changes canonical ownership.

Direct hierarchy edge identity is the existing stable tuple:

```text
(parent_flat_subsystem_id, child_flat_subsystem_id, EdgeKind::Includes)
```

Equivalent source observations aggregate deterministic, sorted, deduplicated
provenance and never duplicate a graph edge.

## Endpoint and validation contract

ADR-0020's Includes matrix is extended additively:

| Source | Target | Meaning |
|---|---|---|
| `NodeKind::Subsystem` | `NodeKind::Subsystem` | Direct declared parent-child hierarchy |
| `NodeKind::Subsystem` | `NodeKind::Metadata(ADR-0020 allowlisted kind)` | Direct declared metadata membership |

All other Includes endpoints remain rejected. In particular, metadata
Subsystem objects cannot be hierarchy sources or targets, flat Role nodes
cannot be members, and Unknown nodes remain invalid.

Validation must reject missing endpoints, physical self-loops, and any directed
cycle composed only of Subsystem-to-Subsystem Includes edges. Cycle detection
must be deterministic and must report stable involved node/edge identities.
Metadata membership edges cannot continue hierarchy traversal.

## Provenance

Every nested metadata node, flat Subsystem node, configuration Contains edge,
hierarchy Includes edge, and direct content fact retains deterministic source
evidence.

A hierarchy edge's provenance must identify at minimum:

- project-relative parent descriptor path and UUID;
- project-relative child descriptor path and UUID;
- exact parent field `mdclass:Subsystem/subsystems` and raw child name;
- exact child field `mdclass:Subsystem/parentSubsystem` and raw qualified path;
- resolved parent and child flat Subsystem IDs;
- a stable EDT subsystem-hierarchy producer stage;
- `FactOrigin::Resolved`, `ResolutionState::Resolved`, and exact confidence.

Physical directory location is corroborating source context, not semantic
identity. Provenance ordering is independent from filesystem or XML order.

## Query contract

Existing generic edge queries continue to expose direct hierarchy and content.
Sprint 10 adds one source-independent read-only query operation for transitive
metadata membership. The implementation may choose a name consistent with the
live Query API, but its observable contract is fixed:

- input is one `NodeId` expected to identify a flat Subsystem;
- an unknown or wrong-kind input returns an empty result without mutating the
  graph or producing diagnostics;
- traversal follows only outgoing Subsystem-to-Subsystem Includes edges;
- results include direct metadata members of the input and all reachable
  descendants;
- results exclude the start Subsystem, descendant Subsystems, metadata
  Subsystem objects, and unsupported endpoint kinds;
- duplicate metadata members reachable through multiple descendants appear
  once;
- ordering is by stable node identity;
- traversal is cycle-safe even for an invalid unvalidated graph;
- no transitive edge, provenance, diagnostic, or cache authority is created.

The complete and incremental Semantic Index must produce query-equivalent
results to a clean rebuild after add, remove, reparent, and content changes.

## Diff, Impact, reports, and dependency behavior

Canonical Diff observes added and removed direct hierarchy edges, provenance
changes, and nested node/content changes through existing identities. It does
not report computed transitive edges because none exist.

Includes remains excluded from dependency queries and Impact propagation.
Sprint 10 does not infer `DependsOn`, `Contains`, `References`, `Opens`, or
another relation from hierarchy or transitive membership. Generic reports may
count the additional direct nodes and Includes edges without adding a second
semantic authority.

## Failure and determinism policy

- A structurally inconsistent hierarchy is fatal and produces no partial graph
  from that build.
- A valid nested descriptor with malformed or unresolved direct content follows
  ADR-0020's recoverable diagnostic policy and may still contribute its nodes
  and hierarchy edge.
- Equivalent descriptors, declaration order, and directory enumeration order
  produce identical graph, diagnostics, statistics, provenance, and query
  results.
- Repeated child declarations or duplicate physical candidates are errors, not
  implicit deduplication, because they make the source hierarchy ambiguous.
- A cycle, self-parent, or path escape is rejected before successful graph
  completion.

## Public API and compatibility impact

No new `NodeKind` or `EdgeKind` is introduced. Existing top-level node and
direct metadata Includes identities remain unchanged. The validator's accepted
Includes endpoint matrix expands additively to the flat Subsystem target, and a
new read-only Query method is additive public API. External exhaustive matches
on existing enums require no change.

Production graphs for projects with nested Subsystems gain their previously
missing metadata/flat nodes, configuration ownership edges, direct hierarchy
Includes, and nested direct content Includes. Projects containing only
top-level Subsystems remain byte-compatible apart from documentation or
expanded test evidence.

## Coverage completion criteria

Coverage status and aggregate counts change only if the live registry proves a
new capability row is necessary. With the current coarse capabilities, Sprint
10 is expected to expand evidence for `semantic_node.subsystem`, metadata
Subsystem nodes, configuration ownership, and `semantic_edge.includes` without
changing their existing `Supported` status or registry totals.

Completion requires executable evidence for:

- recursive source discovery through five live-source-proven depths and a
  tracked provenance-backed fixture;
- duplicate local names under different parents;
- all hierarchy source-agreement failures;
- stable nested node and direct edge identity/provenance;
- precise endpoint and cycle validation;
- nested ADR-0020 content emission and negative outcomes;
- deterministic transitive membership results and no persisted closure;
- repeated/reordered builds and add/remove/reparent/content transitions;
- Query, Diff, reports, complete-index, incremental-index, and clean-rebuild
  equivalence;
- unchanged dependency and Impact traversal;
- full workspace validation and truthful current-state documentation.

## Rejected alternatives

1. `Contains` for hierarchy is rejected because flat Subsystems are composition
   subjects, not canonically owned child entities.
2. A new edge kind is rejected because Includes already means direct declared
   composition membership and can use a precise additive endpoint.
3. Metadata Subsystem endpoints are rejected because they would conflate
   configuration inventory with the existing flat semantic subject.
4. Directory-only inference is rejected because the repository provides two
   explicit XML declarations that must agree with the path.
5. One-projection precedence or silent repair is rejected because it would hide
   contradictory source.
6. Global local-name resolution is rejected because repository-owned sources
   contain duplicate names under different parents.
7. Persisted transitive Includes edges are rejected because they are derived,
   duplicate source truth, complicate provenance, and destabilize Diff.
8. Dependency or Impact propagation through hierarchy is rejected because no
   accepted source meaning proves change dependency.
9. Treating `Subsystem.<...>` content as hierarchy is rejected because observed
   values include self-content and do not replace the dedicated hierarchy
   fields.

## Deferred scope

- command-interface files and navigation;
- configuration inventory lists as another hierarchy authority;
- Subsystem aliases, localized or case-insensitive path resolution;
- cross-project, extension, external, or placeholder Subsystems;
- recovery or partial graph output from contradictory hierarchy;
- semantic meaning for `Subsystem.<...>` content tokens;
- persisted transitive closure or cached independent membership authority;
- hierarchy-aware dependency, Impact, authorization, UI, Runtime, API, CLI,
  MCP, LSP, persistence, and serialization surfaces;
- unsupported metadata content prefixes and later-sprint entity families.

## Implementation order

1. Extend the graph endpoint, validation, cycle, and transitive query contract.
2. Implement deterministic recursive hierarchy parsing without graph emission.
3. Integrate nested node, ownership, hierarchy, provenance, and content emission.
4. Complete representative production, consumer, index, Coverage, and current-
   state documentation evidence.
5. Run the Sprint 10 integration review.
