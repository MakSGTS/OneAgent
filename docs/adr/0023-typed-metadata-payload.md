# ADR-0023: Typed Metadata Payload

## Status

Accepted

## Context

ADR-0003 assigns the source-independent metadata domain to
`oneagent-metadata`, while ADR-0006 and ADR-0008 make the semantic graph the
canonical queryable representation of indexed configuration facts. The current
implementation preserves stable metadata identity, canonical name, kind,
ownership, and provenance, but it does not preserve metadata descriptor content
as typed graph data.

The current source-independent models are intentionally small:

- `MetadataObject` contains identity, name, kind, and optional parent identity;
- `GraphNode` contains identity, name, kind, and provenance;
- payload does not participate in graph equality, graph diff, Query results, or
  a serialization contract because no payload field exists.

The EDT configuration reader parses configuration UUID, name, and optional
synonym. UUID and name reach the workspace and graph models, but synonym is
discarded. The generic top-level metadata reader parses UUID, name, optional
synonym, kind, extension information, descriptor path, and, for Document only,
typed register-record declaration outcomes. Its consumers currently use:

| Parsed fact | Current consumer | Semantic classification |
|---|---|---|
| UUID | metadata and graph identity | Identity, not payload |
| Name | metadata and graph canonical name | Identity-facing core data, not payload |
| Kind | metadata and graph taxonomy | Node kind, not payload |
| Synonym | reader tests only | Common metadata payload |
| Adopted-object extension target | `Extends` resolution and emission | Relation evidence, not payload |
| Descriptor path | readers, provenance, diagnostics, and resolution | Source/provenance, not payload |
| Document register-record declarations | Writes resolution | Document-specific metadata payload and relation evidence |

The same generic reader supplies Catalog, Document, Enumeration, Common Module,
Report, Data Processor, Information Register, Accumulation Register, Accounting
Register, Calculation Register, Business Process, Task, Role, Common Command,
Common Form, Common Template, HTTP Service, Web Service, XDTO Package, and
Subsystem objects. Configuration uses its dedicated reader. Repository-owned
code does not currently parse another top-level field that has a justified
source-independent payload meaning for these kinds.

Other EDT readers extract subordinate members, role rights, Subsystem content,
modules, and metadata references. Those facts already have or are intended to
have first-class nodes, edges, diagnostics, and provenance. Copying them into a
top-level property bag would create two competing semantic representations.

Every supported EDT metadata-entity capability remains
`PartiallySupported`. Its required evidence includes
`SemanticPayloadPreserved`, and the current graph has no typed payload. An
architecture contract is required before public types, graph diff, Query, EDT
conversion, tests, and Coverage can change consistently.

## Decision

`oneagent-metadata` owns the canonical source-independent typed metadata payload
model. The semantic graph stores that same domain value as typed content of a
metadata node; it does not redefine EDT fields or keep a graph-specific string
map.

Conceptually, the metadata-domain API is:

```rust
pub struct MetadataPayload {
    common: CommonMetadataPayload,
    specific: Option<MetadataSpecificPayload>,
}

pub struct CommonMetadataPayload {
    synonym: Option<String>,
}

pub enum MetadataSpecificPayload {
    Document(DocumentMetadataPayload),
}

pub struct DocumentMetadataPayload {
    register_records: Vec<MetadataRegisterRecord>,
}

pub struct MetadataRegisterRecord {
    target_kind: MetadataKind,
    target_name: EntityName,
}
```

The exact Rust names may be refined during implementation, but the ownership,
closed typed structure, and semantic boundaries are normative. A fully dynamic
property map is not an accepted replacement for these types.

`MetadataObject` owns one `MetadataPayload`. `GraphNode` gains a closed node
payload discriminator conceptually equivalent to:

```rust
pub enum GraphNodePayload {
    None,
    Metadata(MetadataPayload),
}
```

The graph reuses `oneagent_metadata::MetadataPayload`; it does not duplicate the
field schema. `GraphNodePayload::Metadata` is valid only for
`NodeKind::Metadata(_)`, and its kind-specific variant must match the node's
`MetadataKind`. Non-metadata nodes use `GraphNodePayload::None` until their own
separate typed-content contracts are accepted.

Payload is semantic content, not identity. Changing synonym or register-record
content preserves the metadata object ID and graph node ID. The changed node is
reported as modified rather than removed and added.

## Common payload contract

The minimum common payload contains only `synonym`.

`synonym` is the optional localized display text explicitly parsed from the
descriptor's direct synonym content. It is distinct from:

- canonical `EntityName`, which remains the stable programmatic name;
- metadata identity and UUID;
- qualified name or ownership path;
- source path and provenance;
- UI captions nested in forms or other subordinate artifacts.

The adapter preserves the XML-decoded value produced by the accepted reader.
It must not synthesize a synonym from `name`, localize it, case-fold it, or
replace an absent value with an empty string. Absence and an explicitly parsed
value remain distinguishable. Any future whitespace-normalization change
requires reader evidence and tests rather than an implicit payload rule.

Configuration and every supported generic top-level metadata kind use this
same common contract. A kind with no repository-owned kind-specific field has
`specific: None`; this is an explicit statement that no additional accepted
payload exists today, not an invitation to insert arbitrary keys.

## Kind-specific payload contract

The first accepted kind-specific variant is Document register-record content.
Each well-formed declaration whose namespace maps to a repository-owned
`MetadataKind` preserves the source-independent target components:

- target `MetadataKind`;
- target `EntityName`.

The payload contains declarations, not resolved graph targets and not Writes
edges. Target node identity, resolution state, diagnostics, occurrence
provenance, and emitted relations remain owned by their existing resolution and
graph models.

The payload vector is sorted deterministically by `(target_kind, target_name)`
and equivalent declarations are deduplicated. The current mapped namespaces are
Information Register, Accumulation Register, Accounting Register, and
Calculation Register. ADR-0022 accepts only Accumulation Register for its first
Writes slice, but that behavioral allowlist does not erase other well-formed
Document content from payload. Malformed declarations, unknown namespaces, and
ambiguous normalized declarations do not become valid payload entries. They
remain typed EDT outcomes and diagnostics. A declaration alone does not
authorize a Writes edge.

No other kind-specific top-level payload field is accepted by current
repository evidence. New fields require:

1. a repository-owned EDT or other adapter fixture;
2. a source-independent semantic definition;
3. a new closed `MetadataSpecificPayload` variant or a typed field in an
   existing variant;
4. deterministic equality, ordering, diff, Query, and provenance behavior;
5. positive, absent-value, malformed-value, and repeated-build evidence as
   applicable.

## Supported-kind inventory

The accepted payload inventory is:

| Metadata kind | Common payload | Specific payload | Explicitly separate facts |
|---|---|---|---|
| Configuration | Optional synonym | None | Workspace root path, format, provenance |
| Catalog | Optional synonym | None | Members, modules, references, extension relation |
| Document | Optional synonym | Register-record targets | Members, modules, Writes, references, extension relation |
| Enumeration | Optional synonym | None | Members, references, extension relation |
| Common Module | Optional synonym | None | Module and BSL symbol nodes |
| Report | Optional synonym | None | Members, modules, references, extension relation |
| Data Processor | Optional synonym | None | Members, modules, references, extension relation |
| Information Register | Optional synonym | None | Members, modules, references, extension relation |
| Accumulation Register | Optional synonym | None | Members, modules, references, extension relation |
| Accounting Register | Optional synonym | None | Dimensions, resources mapped to measures, modules, references, extension relation |
| Calculation Register | Optional synonym | None | Members, modules, references, extension relation |
| Business Process | Optional synonym | None | Members, modules, references, extension relation |
| Task | Optional synonym | None | Members, modules, references, extension relation |
| Role | Optional synonym | None | Flat Role node, rights, Grants, extension relation |
| Common Command | Optional synonym | None | Subordinate command data remains deferred |
| Common Form | Optional synonym | None | Form internals remain separate UI/member facts |
| Common Template | Optional synonym | None | Template content and binary artifacts remain deferred |
| HTTP Service | Optional synonym | None | Endpoints and modules remain separate facts |
| Web Service | Optional synonym | None | Operations and modules remain separate facts |
| XDTO Package | Optional synonym | None | Schema/type internals remain separate facts |
| Subsystem | Optional synonym | None | Flat Subsystem node and Includes relations |

`MetadataKind::Form` and `MetadataKind::Unknown` remain not applicable to the
EDT top-level payload path under the existing Coverage classification. This ADR
does not create synthetic payload for either kind.

## Non-overlapping responsibilities

The semantic responsibilities are:

- `EntityId` identifies the metadata object and remains stable across payload
  changes;
- `EntityName` is the canonical programmatic name used by existing name lookup
  and resolution;
- `MetadataKind` and `NodeKind` classify the entity;
- `parent_id` and `Contains` express ownership;
- `Provenance` identifies the source artifact, producer, and fact origin;
- source paths remain in workspace, descriptor, diagnostic, and provenance
  models;
- `MetadataPayload` stores source-independent, non-identity metadata content;
- first-class members and relations remain nodes and edges rather than copied
  payload collections.

Adopted-object extension information is excluded from payload because
ADR-0018 defines it as `Extends`. Role rights are excluded because ADR-0019
defines scoped AccessRight nodes and Grants/References edges. Subsystem content
is excluded because ADR-0020 defines Includes. Metadata children and type
references remain nodes, Contains edges, References edges, and DependsOn edges.
Document register records are the narrow exception: their declared target
components are intrinsic Document content, while their resolution and
behavioral consequences remain relations.

## Equality, ordering, and graph diff

`MetadataPayload` and every nested type implement structural `PartialEq` and
`Eq`. Any collection stored in payload must expose deterministic canonical
ordering; the first Document collection is sorted and deduplicated before graph
insertion. Hash-map iteration order, filesystem order, parser occurrence order,
and insertion order must not affect payload equality.

`MetadataObject` equality includes payload. `GraphNode` equality includes
`GraphNodePayload`. Node identity continues to use only `EntityId`.

`NodeSnapshot` includes payload. A payload-only change is
`NodeModifiedAspect::SemanticContent` under the existing public diff taxonomy.
The first implementation does not add a new public modified-aspect variant;
this avoids an unnecessary exhaustive-match migration while making the existing
semantic-content label accurate for name, kind, and payload.

Payload is not included in edge identity, resolution keys, or graph node
indexes unless a later query contract explicitly adds a payload index.

## Query contract

The existing Query API returns borrowed `GraphNode` values. `GraphNode` exposes
its typed payload and a convenience metadata-payload accessor, so node lookup,
kind lookup, name lookup, ownership navigation, and traversal can inspect
payload without a second adapter-specific query surface.

The first implementation does not add synonym search, payload predicates, or
register-record navigation methods. Those operations require a separate public
query and indexing contract. Existing exact canonical-name behavior remains
unchanged.

## Serialization and compatibility

The repository has no public graph serialization format today. Future
serialization must represent payload as a versioned, tagged structure with
stable field names and deterministic collection ordering. It must distinguish
absent common values, `specific: None`, and a present typed specific variant.
An unversioned flattened map is prohibited because it would erase variant and
compatibility boundaries.

Existing public constructors remain available during migration:

- `MetadataObject::new` creates an object with empty common payload and no
  specific payload;
- `GraphNode::new` and `GraphNode::new_with_provenance` create nodes with
  `GraphNodePayload::None`;
- new explicit constructors or builders accept payload without changing
  identity or provenance parameters.

Keeping compatibility constructors prevents a repository-wide mechanical
rewrite of non-metadata tests and producers. It does not constitute EDT payload
evidence. The EDT metadata contributor must use the explicit payload path for
every supported top-level metadata node before the corresponding Coverage
evidence can change.

Known implementation consumers are:

- `oneagent-metadata` constructors, tree tests, and equality;
- `oneagent-workspace::Configuration`, whose dedicated EDT reader currently
  discards configuration synonym;
- `oneagent-graph::GraphNode`, semantic graph insertion helpers, validation,
  diff snapshots, Query, Impact's diff consumption, reports, resolution, and
  their tests;
- `oneagent-edt` configuration and metadata readers, graph builder, extension,
  Subsystem, metadata-structure, module, role-right, reference, and Writes
  contribution paths;
- EDT and graph integration tests that construct GraphNode directly.

No dependency-direction change is required because `oneagent-graph` already
depends on `oneagent-metadata`, and `oneagent-edt` already depends on both.

## Provenance

Payload does not contain descriptor paths or adapter producer identifiers. The
metadata node's existing provenance identifies the descriptor that supplied the
payload. Document register-record parsing retains its current typed occurrence
context for diagnostics and Writes provenance; the canonical payload value does
not replace that source evidence.

A graph implementation must reject or diagnose a metadata payload attached to
a non-metadata node or a kind-specific payload that conflicts with the node's
`MetadataKind`. It must not repair a mismatch by changing node identity or
silently dropping the payload.

## Coverage completion criteria

Architecture acceptance does not add `SemanticPayloadPreserved` evidence and
does not change a capability status or aggregate count.

For one EDT `MetadataEntity(kind)` capability, the registry may add
`SemanticPayloadPreserved` only after all applicable evidence below exists in
production and tests:

1. the production configuration or generic top-level reader preserves optional
   synonym through the source-independent payload into the emitted metadata
   GraphNode;
2. present, absent, and non-ASCII synonym cases prove exact accepted behavior
   without fallback to canonical name;
3. the kind's accepted specific payload is preserved, or repository evidence
   proves that the inventory explicitly defines no specific field;
4. Document additionally proves sorted, deduplicated typed register-record
   payload for every mapped namespace plus malformed, unknown-namespace,
   ambiguous, duplicate, and empty outcomes;
5. extension, source path, ownership, members, rights, content membership, and
   resolved relations are not duplicated into payload;
6. graph validation rejects payload-kind mismatch and accepts the canonical
   metadata payload shape;
7. Query node lookup exposes the typed payload without changing exact name or
   kind lookup;
8. a payload-only change preserves node identity and appears as one modified
   semantic-content node in graph and build-result diff;
9. repeated builds of unchanged source produce equal payload, graph, diff,
   report, validation, and deterministic ordering;
10. focused metadata-domain, graph-domain, EDT reader, and full-builder tests
    pass, including a representative production integration test for that
    capability.

Capability transitions are independent. A capability becomes `Supported` only
when its complete existing required-evidence set is present; adding payload
evidence does not compensate for a missing representative test or integration
test. Registry aggregate counts must be recomputed from the verified state, not
copied from this ADR.

## Ordered implementation prerequisites

1. Add typed payload types to `oneagent-metadata`, keep compatibility
   constructors, and test common/specific equality and deterministic Document
   ordering.
2. Add typed node payload to `oneagent-graph`, enforce kind compatibility, and
   update equality, diff snapshots, public accessors, Query evidence, reports
   only where payload observability is explicitly required, and focused tests.
3. Preserve configuration synonym through the dedicated EDT reader and root
   graph-node contribution path.
4. Convert the generic top-level EDT descriptor into the common payload for all
   supported directory kinds without moving descriptor path or extension state
   into payload.
5. Convert accepted Document register-record declarations into the canonical
   specific payload while retaining private parse outcomes and existing Writes
   resolution behavior.
6. Add representative production fixtures and negative evidence for each
   supported metadata kind that still lacks the tests required by its Coverage
   capability.
7. Add payload-only graph/build diff and repeated-build determinism evidence.
8. In a final registry-only task, add `SemanticPayloadPreserved` per proven
   capability, update representative tests and limitations, recompute aggregate
   counts, and synchronize Roadmap and Semantic Model 2 current-state text.

Each slice must preserve semantic identity, existing edges, diagnostics,
resolution statistics, Query name behavior, and public compatibility unless a
separate task explicitly changes them.

## Rejected alternatives

### Store payload only in the EDT descriptor

Rejected because graph and metadata-domain consumers would still lose payload,
Query could not expose it, and the source adapter would remain the semantic
owner.

### Store an untyped property map in GraphNode

Rejected because string keys and values would mix source vocabulary with domain
semantics, weaken compatibility, and make per-kind validation and diff behavior
implicit. ADR-0008 already requires typed core properties.

### Add synonym directly to GraphNode without a metadata payload type

Rejected because it would make one metadata field a generic graph concern and
provide no typed per-kind extension path.

### Put descriptor path and extension target into payload

Rejected because source location belongs to provenance and extension is already
a first-class Extends relation. Duplicating either would permit contradictory
facts.

### Model every parsed EDT outcome as public payload

Rejected because malformed, unknown-namespace, duplicate-occurrence, and
ambiguous parse states are adapter diagnostics and resolution evidence. Only
well-formed source-independent semantic values belong in canonical payload.

### Change node identity when payload changes

Rejected because synonym and other semantic content may change while the same
metadata object UUID remains stable. Identity churn would turn a content change
into misleading removal/addition diffs.

### Implement every future metadata field in the first slice

Rejected because the repository does not yet parse or prove a complete field
vocabulary for every metadata family. Typed variants are added incrementally
from repository-owned evidence.

## Deferred scope

The following remain outside this decision's first implementation sequence:

- arbitrary EDT XML properties not currently parsed by repository code;
- subordinate member payload already represented by nodes and edges;
- form/UI internals, template content, service operations, XDTO schemas, report
  data-composition internals, and other unimplemented metadata families;
- payload search indexes and synonym-specific Query methods;
- public graph serialization and persistence formats;
- source-range or per-field public provenance types;
- Designer XML payload mapping;
- payload for non-metadata graph node kinds;
- Coverage status or aggregate-count changes without production evidence.

## Risks

- Compatibility constructors could leave EDT metadata nodes with empty payload
  while appearing migrated. Coverage therefore requires explicit production
  construction and per-kind integration evidence.
- Copying members or relation evidence into payload could create contradictory
  semantic facts. The closed inventory and payload-kind validation prevent
  arbitrary duplication.
- Payload fields could accidentally enter identity or resolution keys. Focused
  payload-only diff tests must prove stable IDs and unchanged resolution.
- A growing specific-payload enum could become expensive to evolve. New
  variants remain evidence-driven and serialization must be explicitly
  versioned before persistence is added.
- Canonicalizing Document declarations could discard diagnostic evidence. EDT
  parse outcomes and occurrence provenance remain available independently from
  the deduplicated semantic payload.

## Consequences

### Positive

- Metadata payload has one source-independent typed owner.
- Graph consumers can observe semantic content without depending on EDT.
- Identity remains stable while payload changes become visible in diff.
- Existing first-class nodes, edges, and provenance are not duplicated.
- New kind-specific fields have a closed typed migration path.
- Coverage transitions have explicit per-capability evidence requirements.

### Negative

- Metadata and graph public APIs gain new types and explicit payload
  construction paths.
- The EDT configuration and generic descriptor paths must be migrated
  separately.
- Complete Coverage requires representative fixtures for generic kinds that do
  not yet have them.
- Future serialization must version the tagged payload structure.

## Decision outcome

Typed metadata payload is owned by `oneagent-metadata` and stored as typed
semantic content on metadata GraphNode values. The accepted current payload is
optional common synonym plus deterministic Document register-record targets.
Identity, ownership, provenance, source location, members, and relations remain
separate. Production behavior and Semantic Coverage remain unchanged until the
ordered implementation evidence is complete.
