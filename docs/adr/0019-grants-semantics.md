# ADR-0019: Grants Semantics

## Status

Accepted

## Context

Semantic Model 2.0 declares `EdgeKind::Grants` as a semantic graph edge. The
graph domain can store, query, diff, report, and broadly validate the edge kind,
but the repository does not define what a grant fact means, which endpoint
kinds it may connect, or which EDT source artifact may produce it.

The current graph model already emits flat `NodeKind::Role` nodes for EDT role
metadata objects while preserving the original
`NodeKind::Metadata(MetadataKind::Role)` object nodes. Role access-right
modeling is intentionally deferred. Current `NodeKind` variants do not include
a first-class access-right, permission, or object-right node.

EDT exposes role-right semantics through the role/rights model rather than
through BSL data access. The relevant domain fact is a role declaring an access
right value for a protected object or platform capability. This is a ternary
access fact:

```text
role + right/operation + protected resource
```

Without a precise contract, a future producer could incorrectly:

- use `Grants` as a synonym for `Includes`, `Contains`, `References`,
  `Reads`, `Writes`, or `DependsOn`;
- store `Role --Grants--> Metadata(...)` and lose the concrete right identity;
- store `Role --Grants--> Right` and lose the protected resource identity;
- treat access profiles, access groups, user assignments, or BSP data as
  platform role grants;
- persist effective runtime permissions or inherited/transitive access as
  direct source facts;
- encode allow/deny state only in provenance or diagnostics;
- emit edges from unresolved, unsupported, or external access facts.

## Problem

`EdgeKind::Grants` needs an implementation-ready architecture contract before
EDT can emit it. The contract must preserve both the access operation identity
and the protected resource identity while keeping the relation deterministic,
direct-only, provenance-backed, and distinct from data-access and membership
relations.

## Decision

`EdgeKind::Grants` represents an explicit, direct, declared allow grant from an
access subject to a scoped access-right entity.

The canonical stored direction is:

```text
access subject --Grants--> scoped access right
```

The first production slice is limited to EDT role object-right declarations:

```text
NodeKind::Role --Grants--> NodeKind::AccessRight
```

`NodeKind::AccessRight` is a required future graph-domain node kind. It
represents one stable scoped access capability: one right or operation applied
to one protected resource. The target node identity must preserve the protected
resource identity and the right identity. `Grants` must not be emitted until the
target node kind exists and has its own identity, provenance, query, coverage,
and validation contract.

The graph must not use edge payloads, provenance-only data, or a direct
`Role --Grants--> Metadata(...)` edge to represent the first slice because the
current `GraphEdge` identity is only `(source, target, kind)`. A direct edge to
the protected metadata object would collapse `Read`, `Insert`, `Update`,
`Delete`, `Posting`, and other rights into the same edge identity.

## Canonical semantic statement

`A --Grants--> B` means that access subject `A` has a direct source-declared
allowed grant for scoped access-right entity `B`.

For the first EDT slice:

```text
Role(SalesManager) --Grants--> AccessRight(Document.SalesInvoice, Read)
```

means that the EDT role descriptor explicitly declares the role as allowing the
`Read` right on the protected `Document.SalesInvoice` resource.

## Domain scope

The first slice belongs to the static 1C platform configuration model:

- source subject: EDT metadata object of kind `Role`;
- graph subject: existing flat `NodeKind::Role`;
- protected resource: resolved metadata object or another explicitly modeled
  protected semantic resource;
- right identity: stable platform/EDT right code or name within the protected
  resource scope;
- grant value: explicit allow only.

Runtime users, sessions, access group membership, access profile assignment,
BSP application data, database rows, and effective permission computation are
outside the first slice.

## Production source evidence

The accepted production source is the EDT role-right model associated with an
EDT `Role` metadata object.

Repository evidence:

- EDT role metadata objects are already discovered from `src/Roles/<Role>` and
  emitted as both `NodeKind::Metadata(MetadataKind::Role)` and flat
  `NodeKind::Role`;
- current fixtures contain role `.mdo` descriptors with role identity and name,
  but they do not contain object-right declarations;
- no current OneAgent parser reads role-right matrices, RLS conditions, or
  right values.

External EDT API evidence:

- EDT exposes rights-domain model types under `com._1c.g5.v8.dt.rights.model`;
- `BaseRightsDescription` stores object rights as object-id to
  right-name/value mappings and tracks top objects that have RLS;
- `Rls` exposes referenced fields and a condition expression;
- EDT contains task classes for adding and editing right values and RLS.

The future implementation must verify the exact serialized EDT artifact and XML
layout before parsing. This ADR fixes the semantic contract and source model,
but it does not invent concrete parser fields for role-right XML.

## Explicit non-meanings

`Grants` is not:

- structural ownership or containment;
- role membership;
- subsystem inclusion;
- access-profile inclusion;
- user or user-group assignment;
- a generic reference;
- a normalized dependency;
- query or BSL data-flow;
- `Reads` or `Writes`;
- effective runtime authorization;
- inherited or transitive access;
- denied access;
- absence of access;
- a runtime audit event;
- an RLS condition expression by itself;
- BSP `AccessProfile` or `AccessGroup` semantics.

## Alternatives considered

### Candidate A: `Role --Grants--> Right`

Rejected for the first slice. A right such as `Read` can be stable, but it does
not identify the protected object. This loses whether the role can read
`Catalog.Products`, `Document.SalesInvoice`, or another resource.

### Candidate B: `Role --Grants--> MetadataObject`

Rejected. This preserves the protected resource but loses the right identity.
The current edge identity cannot distinguish separate rights between the same
role and metadata object. Encoding the right only in provenance would make the
semantic fact unqueryable and would collapse duplicates.

### Candidate C: `Role --Grants--> Permission` or scoped access-right node

Accepted. A reified scoped access-right node preserves the ternary access fact
without edge payloads. The target identity contains both the protected resource
identity and the right identity, while the `Grants` edge connects the role to
that capability.

### Candidate D: `Role --Grants--> Operation`

Rejected for the first slice. An operation node such as `Read` or `Update`
loses the protected resource. It has the same problem as Candidate A unless a
second relation scopes the operation to a resource, which is equivalent to the
accepted reified scoped-right model.

### Candidate E: `AccessProfile --Grants--> Role`

Deferred and not part of platform role grants. Access profiles are typically
application/BSP or runtime assignment concepts rather than the platform EDT
role-right matrix. Relationship to roles is membership or inclusion, not a
declared object-right grant.

### Candidate F: `AccessGroup --Grants--> AccessProfile` or Role

Rejected for the initial static graph. Access groups are user-assignment or
application-domain concepts. They are not a direct EDT role-right declaration
and belong to a separate runtime/application access model.

### Candidate G: Role grants effective user access

Rejected. Effective access depends on runtime user assignments, role
combination, inherited defaults, deny/absence policy, RLS conditions, session
state, and application data. `Grants` stores direct declarations only.

## Ternary relation analysis

The required access fact has three semantic dimensions:

```text
subject + operation/right + protected resource
```

The current graph edge has no typed payload and uses only
`(source, target, EdgeKind)` as identity. Therefore the first slice must reify
the operation/resource pair as a node:

```text
Role --Grants--> AccessRight(resource, right)
AccessRight --References--> protected resource
```

The `References` edge from `AccessRight` to the protected resource is a
recommended companion relation for future implementation because it enables
resource-centric navigation without duplicating the grant relation. The
`AccessRight` node identity itself must still include the protected resource
identity so that the `Grants` edge remains unique and stable even if companion
edges are omitted from a partial query.

No production `Grants` edge may be emitted before this ternary representation is
available.

## Relationship with Includes

`Includes` represents membership, composition, or inclusion semantics when a
future producer defines such a contract. It is not an access grant.

Examples that must not emit `Grants` in the first slice:

```text
AccessProfile --Includes--> Role
Subsystem --Includes--> Metadata(...)
Role --Includes--> another role
```

If a future access profile source assigns a role to a profile, that fact must
use an accepted `Includes` or membership contract, not `Grants`. `Grants`
remains reserved for direct allow rights.

## Relationship with Reads

`Reads` describes code or query data-access behavior: a source expression,
query, procedure, or function reads data from a target resource. It does not
describe authorization.

A role may grant a read right and a query may read the same metadata object, but
these are separate facts:

```text
Role --Grants--> AccessRight(Catalog.Products, Read)
Query --Reads--> Metadata(Catalog.Products)
```

The existence of `Reads` must not imply `Grants`, and the existence of
`Grants(Read)` must not imply a `Reads` edge.

## Relationship with Writes

`Writes` describes code or query write behavior. It does not describe
authorization.

A write-related role right and a write data-flow fact may coexist but must be
emitted from different producers and source facts:

```text
Role --Grants--> AccessRight(Document.SalesInvoice, Update)
Procedure --Writes--> Metadata(Document.SalesInvoice)
```

`Writes` must not be derived from role rights, and `Grants` must not be derived
from query or BSL write analysis.

## Relationship with containment and ownership

`Contains` represents structural ownership. A role metadata object is contained
by the configuration, and a scoped access-right node may have a structural
owner only if a future node ownership contract requires it.

Ownership does not imply authorization. Authorization does not imply ownership.
`Grants` must not satisfy single-owner validation and must not be used for
owner navigation.

## Direction

The stored direction is from the access subject to the scoped access-right
entity:

```text
Role --Grants--> AccessRight
```

The graph must not store reverse edges such as `GrantedTo`. Reverse traversal
from an access right to roles is a query concern.

## Endpoint matrix

The first production slice allows:

| Origin | Source kind | Target kind | Direction | Status |
|---|---|---|---|---|
| EDT role object-right allow declaration | `NodeKind::Role` | future `NodeKind::AccessRight` | role grants scoped right | First slice |

The future `NodeKind::AccessRight` must be compatible with one resolved
protected resource. The first slice should support resolved
`NodeKind::Metadata(kind)` protected resources when the EDT right applies to a
metadata object. Other resource scopes require a separate slice.

Representative forbidden pairs:

- `NodeKind::Role --Grants--> NodeKind::Metadata(...)`;
- `NodeKind::Metadata(MetadataKind::Role) --Grants--> NodeKind::Metadata(...)`;
- `NodeKind::Role --Grants--> NodeKind::Role`;
- `NodeKind::Role --Grants--> NodeKind::Subsystem`;
- `NodeKind::Procedure --Grants--> NodeKind::AccessRight`;
- `NodeKind::Query --Grants--> NodeKind::AccessRight`;
- any source or target involving `NodeKind::Unknown`;
- missing, unresolved, ambiguous, or external protected-resource targets in the
  first slice.

## Identity

`Grants` uses the standard graph edge identity:

```text
(source_node_id, target_node_id, EdgeKind::Grants)
```

For the first slice:

- source node identity is the existing flat role node identity:
  `<role_metadata_object_id>:role`;
- target node identity is the future scoped access-right identity;
- scoped access-right identity must include the protected resource identity and
  the right identity;
- edge identity must not include provenance, source path, parsing order, or
  textual display names.

A future scoped access-right identity should follow this canonical shape or an
equivalent deterministic encoding:

```text
access_right:resource#<resource-id>;right#<right-code>
```

The identity must not use localized right names, collection indexes, filesystem
enumeration order, or role-specific data.

## Provenance

Every emitted `Grants` edge and every supporting scoped access-right node must
carry provenance.

For the first EDT slice, provenance must identify:

- role EDT source artifact;
- role metadata object id;
- flat role node id;
- protected resource declaration from the rights source;
- resolved protected resource node id;
- right code or stable right name;
- declared right value;
- whether the value was accepted as an explicit allow;
- producer;
- `FactOrigin::Declared` or `FactOrigin::Resolved` according to the producer
  path;
- `ResolutionState::Resolved` for emitted grant edges.

Provenance must not be part of semantic identity. It may include source path,
XML path, source range, role descriptor path, or rights descriptor path when
available.

## Resolution

The first slice may emit `Grants` only when:

1. the role source resolves to an existing flat `NodeKind::Role`;
2. the protected resource resolves to exactly one existing graph node of an
   accepted resource kind;
3. the right code is supported and has stable identity;
4. the declared value is an explicit allow;
5. the scoped access-right target node exists or can be created deterministically
   by the same accepted production slice.

Missing protected resources must not emit `Grants` or placeholder nodes in the
first slice. Ambiguous protected resources must not emit `Grants`. External
protected resources must not emit `Grants` until an external-node contract is
accepted. Unsupported rights must be skipped or diagnosed according to the
producer diagnostics policy. Malformed source values are parser errors or typed
diagnostics, not graph edges. Partial workspaces may produce fewer grant facts
but must remain deterministic.

## Allow/deny policy

The first slice emits only explicit allow grants.

Accepted allow values must come from the EDT rights model, for example a value
equivalent to `Set` or a boolean true object-right entry. The future
implementation must verify the exact serialized representation before parsing.

Explicit deny is deferred. If EDT distinguishes deny from unset/inherited
values, deny must not be encoded as a missing `Grants` edge once deny modeling
is required. A future deny contract may introduce a dedicated relation, a
dedicated access-right state node, or a typed diagnostic, but this ADR does not
define it.

Absent or unset rights do not emit `Grants`. Inherited or provided rights do not
emit `Grants` in the first slice unless they are explicitly stored as direct
role declarations and the implementation contract accepts their semantics.

## Direct/effective policy

Stored `Grants` edges are direct-only.

The graph must not persist:

- effective user permissions;
- transitive role/profile/group permissions;
- inherited permission closure;
- role aggregation results;
- runtime authorization decisions;
- RLS-evaluated row-level outcomes.

Effective access analysis may later traverse role assignments, memberships,
direct grants, deny rules, and runtime context, but that analysis must not
change the meaning of direct `Grants`.

## Validation contract

The future validator must replace broad `Grants` acceptance with the first
slice endpoint rule once production emission exists:

```text
NodeKind::Role --Grants--> NodeKind::AccessRight
```

The validator must reject:

- self-loops;
- missing endpoints;
- direct role-to-metadata grants;
- metadata role object nodes as sources;
- unsupported source kinds;
- unsupported target kinds;
- `Unknown` endpoints;
- scoped access-right nodes whose identity or companion resource relation is
  inconsistent with the accepted access-right node contract.

Graph insertion already rejects missing endpoints. Edge provenance remains
covered by existing provenance validation.

## Minimal first production slice

The minimal implementation-ready slice is:

| Production fact | Source node | Target node | Direction |
|---|---|---|---|
| EDT role object declares an explicit allowed object right for a resolved metadata object | `NodeKind::Role` | future `NodeKind::AccessRight(resource=Metadata(...), right=<right-code>)` | role --Grants--> scoped right |

Non-goals for the first slice:

- no effective access computation;
- no user assignments;
- no access profiles;
- no access groups;
- no BSP-specific semantics;
- no RLS expression parsing beyond preserving or deferring the source fact;
- no deny modeling;
- no inherited/transitive grants;
- no direct `Role --Grants--> Metadata(...)` edges;
- no production emission without a scoped access-right node contract.

## Deferred slices

Future slices may cover:

- explicit deny modeling;
- inherited/provided role-right values;
- global or configuration-level rights;
- non-metadata protected resources;
- RLS condition nodes and diagnostics;
- access profile and access group membership;
- BSP access-control metadata;
- runtime user-to-role assignments;
- effective access analysis;
- query/code access compared against role grants.

## Implementation prerequisites

1. Define the graph-domain scoped access-right node kind and identity.
   - Affected subsystem: `oneagent-graph`.
   - Reason: `Grants` needs a target preserving both right and resource.
2. Define ownership/query/report/coverage behavior for scoped access-right
   nodes.
   - Affected subsystem: graph validation, query, coverage, report.
   - Reason: target nodes must be real semantic entities.
3. Verify and parse the real EDT role-right source artifact.
   - Affected subsystem: `oneagent-edt` role/rights reader.
   - Reason: current EDT parser reads role descriptors but not rights.
4. Resolve protected resources against existing graph nodes.
   - Affected subsystem: EDT semantic graph builder.
   - Reason: first slice forbids unresolved/external placeholder targets.
5. Create scoped access-right nodes with provenance.
   - Affected subsystem: graph builder and provenance source identifiers.
   - Reason: target identity and provenance are required before grants.
6. Emit `Role --Grants--> AccessRight` edges.
   - Affected subsystem: EDT semantic graph builder.
   - Reason: capability closure requires production graph emission.
7. Add precise validator rules and negative tests.
   - Affected subsystem: `oneagent-graph` validation.
   - Reason: broad `Grants` acceptance is insufficient.
8. Add focused and integration fixtures.
   - Affected subsystem: `oneagent-edt` tests.
   - Reason: Coverage closure must use real production evidence.
9. Transition Coverage Registry only after all criteria are satisfied.
   - Affected subsystem: EDT Coverage Registry.
   - Reason: architecture alone is not production support.

## Coverage Registry completion criteria

`semantic_edge.grants` may transition to `Supported` only after all of the
following evidence exists:

- real EDT source is parsed;
- source `NodeKind::Role` is emitted as a real node;
- target scoped access-right node is emitted as a real node;
- right identity is preserved in target identity;
- protected resource identity is preserved in target identity and/or companion
  relation;
- canonical direction is implemented;
- direct-only policy is implemented;
- allow-only first-slice policy is implemented;
- missing protected resources are handled deterministically;
- ambiguous protected resources are handled deterministically;
- external protected resources are handled according to the accepted policy;
- unsupported rights are skipped or diagnosed deterministically;
- edge identity is deterministic;
- duplicate observations deduplicate deterministically;
- node and edge provenance is attached;
- exact validator matrix exists;
- positive production tests exist;
- negative production tests exist;
- regression tests prove existing role and metadata behavior remains unchanged;
- full workspace validation passes;
- documentation is updated from planned to implemented;
- High count decreases exactly once;
- Medium count remains unchanged unless registry-calculated values differ for a
  documented reason.

This architecture task must not change the current capability status or
coverage counters.

## Implementation readiness checklist

| Question | Answer |
|---|---|
| Is the domain meaning fixed? | Yes. Direct declared allow grant. |
| Is the direction fixed? | Yes. Access subject to scoped access right. |
| Are source and target node kinds fixed? | Yes, after the required `NodeKind::AccessRight` prerequisite. |
| Is the ternary-fact problem resolved? | Yes. Reified scoped access-right target node. |
| Is right identity fixed? | Yes. It belongs to the scoped access-right node identity. |
| Is protected-resource identity fixed? | Yes. It belongs to the scoped access-right node identity and should also be navigable by a companion resource relation. |
| Is allow/deny policy fixed? | Yes. First slice emits explicit allow only; deny is deferred and must not be encoded as `Grants`. |
| Is direct/effective policy fixed? | Yes. Direct declarations only; effective access is deferred. |
| Is the production source known? | Yes at the EDT role-right model level; exact serialized XML layout remains a parser prerequisite. |
| Are resolution rules fixed? | Yes. Emit only resolved in-graph protected resources for the first slice. |
| Is provenance fixed? | Yes. Role source, right, resource, value, resolved identities, producer, origin, and resolution are mandatory. |
| Is validator policy fixed? | Yes. `NodeKind::Role --Grants--> NodeKind::AccessRight`, no self-loop, no direct role-to-metadata edge. |
| Is edge identity sufficient? | Yes, because the target node carries right and resource identity. |
| Is the first slice selected? | Yes. EDT role object-right allow declarations for resolved metadata resources. |
| Are fixtures available? | No. Realistic role-right fixtures must be added by the implementation task. |
| Are prerequisites ordered? | Yes. Access-right node, parser, resolution, emission, validator, tests, coverage. |
| Are Coverage Registry criteria fixed? | Yes. This ADR defines them. |

Final readiness status: `Ready after listed prerequisites`.

## Migration strategy

The future implementation can be introduced without breaking existing graphs by
adding the scoped access-right node kind first, then adding parser extraction,
then graph emission, then validator tightening, and finally the Coverage
Registry transition.

Existing serialized graph data, if present, remains compatible because no
existing node or edge identity changes. Graphs built before the implementation
will simply lack `AccessRight` nodes and `Grants` edges. Direct role nodes and
metadata role nodes retain their current identities.

## Rollback strategy

If production evidence proves incomplete, the implementation can be rolled back
by disabling grant emission while keeping the architecture contract accepted.
Parser fields may remain internal if unused; scoped access-right nodes must not
be emitted without grants unless their own node capability has been accepted.
Coverage status and counters must revert to non-supported values if they had
changed. Documentation must state that production support is pending.

## Consequences

- `Grants` gains a precise architecture contract.
- The ternary access-fact problem is resolved without edge payloads.
- Right identity and protected-resource identity are preserved.
- Existing `Contains`, `Includes`, `References`, `DependsOn`, `Reads`, `Writes`,
  `Calls`, and `Extends` behavior remains unchanged.
- A future implementation requires a scoped access-right node prerequisite
  before it can close `semantic_edge.grants`.
- Coverage counters remain unchanged until production support exists.

## Future work

- Define `NodeKind::AccessRight` or an equivalent scoped permission node.
- Verify the exact EDT role-right XML/source layout and value vocabulary.
- Add EDT role-right parser support.
- Add representative role-right fixtures.
- Define explicit deny semantics.
- Define access profile and access group semantics separately from `Grants`.
- Decide whether access-right nodes participate in Impact Analysis.
