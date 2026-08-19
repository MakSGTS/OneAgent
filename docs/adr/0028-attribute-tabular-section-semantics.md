# ADR-0028: Attribute and Tabular Section Member Content

## Status

Accepted

## Context

Sprint 3 completed the first Attribute and TabularSection semantic slice. The
EDT adapter emits stable `Attribute` and `TabularSection` nodes, preserves the
nearest owner for nested Attributes, emits deterministic provenance-backed
`Contains`, and converts accepted Attribute metadata types through the public
reference-request lifecycle into precise `References` and `DependsOn`
projections or typed failures.

The Sprint 6 source investigation in
`docs/architecture/attribute-tabular-section-source-investigation.md` confirms
that this behavior is already implemented and Supported. Repeating it would not
expand the knowledge model.

The investigation also identifies one smallest unmodeled source fact with
repository-owned real-format evidence. The committed
`adapters/edt/tests/fixtures/grants_project/src/Documents/Sale/Sale.mdo`
contains:

- one TabularSection with one direct
  `<synonym><key>ru</key><value>...</value></synonym>`;
- three nested Attributes with the same single direct synonym value shape;
- one nested Attribute with no synonym.

The committed ownership fixture contains a TabularSection, one top-level
Attribute, and two nested Attributes without member synonym. The live
`EdtMetadataChildDescriptor` discards all member synonym content, and
`GraphNodePayload` supports only no content or top-level `MetadataPayload`.

ADR-0023 defines synonym as optional display content distinct from identity,
canonical name, ownership, source path, provenance, members, and relations. It
also explicitly prevents subordinate member facts from being copied into a
top-level metadata-object payload. A separate typed member-content contract is
therefore required.

Other observed EDT fields do not have enough accepted source-independent
semantics for this slice. They include TabularSection standard attributes,
Number qualifiers, `dataHistory`, `fullTextSearch`, produced types, and
line-number length. The repository also lacks real Attribute or TabularSection
artifacts for non-Document owners and lacks evidence for deeper nesting,
same-owner UUID-less duplicates, multiple locale values, or alternative member
synonym encodings.

## Decision

Introduce a closed source-independent `MetadataMemberPayload` in
`oneagent-metadata`. Its first and only accepted field is an optional synonym.
Store that value in a distinct `GraphNodePayload::MetadataMember` variant that
is compatible only with `NodeKind::Attribute` and
`NodeKind::TabularSection` in this slice.

The conceptual domain API is:

```rust
pub struct MetadataMemberPayload {
    synonym: Option<String>,
}

impl MetadataMemberPayload {
    pub const fn new(synonym: Option<String>) -> Self;
    pub const fn empty() -> Self;
    pub fn synonym(&self) -> Option<&str>;
}
```

The exact accessor names may be refined during implementation, but the owning
crate, closed field set, optional-value semantics, and graph compatibility are
normative. A string map, EDT-specific payload type, or reuse of top-level
`MetadataPayload` is not an accepted substitute.

## Member synonym contract

Member synonym is explicit localized display text. It is distinct from:

- the canonical `EntityName` used by exact name lookup and resolution;
- member `EntityId` and source UUID;
- owner identity and containment path;
- metadata type expressions and resolved targets;
- source artifact path and provenance;
- form captions, UI labels, and other presentation models.

The domain preserves the decoded string supplied by the accepted reader. It
must not synthesize synonym from name, case-fold it, localize it, use it as a
fallback name, or replace absence with an empty string. Absence and a present
value are distinct. No additional whitespace normalization is performed after
the accepted XML reader behavior.

The first compatibility set is exactly:

- `NodeKind::Attribute`;
- `NodeKind::TabularSection`.

Dimension, Resource, Measure, StandardAttribute, Form, Command, top-level
Metadata nodes, and every other node kind remain incompatible with
`MetadataMemberPayload` until their own accepted evidence exists.

## EDT source contract

The first EDT source slice accepts a recognized Attribute or TabularSection
child containing exactly one direct synonym `value` text under its own direct
`synonym` container:

```xml
<attributes uuid="...">
  <name>Price</name>
  <synonym>
    <key>ru</key>
    <value>Price</value>
  </synonym>
</attributes>
```

The `key` is source locale context and is not stored in the first
source-independent payload. The first slice does not select between locales or
publish locale identity. It is accepted only for the repository-proven single
direct value form.

Parser behavior is:

- no direct synonym container produces `MetadataMemberPayload::empty()`;
- exactly one direct non-empty value produces `Some(decoded_value)`;
- an accepted synonym container with no non-empty direct value is a typed
  member-synonym parser error;
- a second direct synonym container or second direct value for the same member
  is a typed duplicate member-synonym parser error;
- nested member synonym belongs to the innermost pending recognized child and
  must not leak into its TabularSection or metadata-object owner;
- unsupported alternative encodings, including member synonym `content`, are
  not silently treated as the accepted value form.

The implementation may use one error enum variant with structured context or
separate missing/duplicate variants. Errors must retain descriptor path and
child kind and must have deterministic equality/display behavior consistent
with the existing parser error model. The parser must not emit a partial child
descriptor after an invalid accepted synonym container.

Generated parser tests may exercise non-ASCII and malformed text using the
confirmed XML shape. Real-format production evidence must include both the
present grants fixture and the absent ownership fixture.

## Identity, equality, and ordering

Payload is semantic content, not identity.

- A source UUID remains the member `EntityId` unchanged.
- UUID-less identity remains
  `<immediate-owner-id>:<child-kind>:<raw-name>`.
- Synonym does not participate in fallback identity, request identity, edge
  identity, owner resolution, or name lookup.
- Changing only synonym preserves node and edge identities.
- `MetadataMemberPayload` derives structural `PartialEq` and `Eq`.
- The payload contains no collection in this slice, so it introduces no new
  collection ordering.
- Repeated reads and builds over unchanged source must produce equal payload,
  graph, build result, validation, and report state.

Same-owner UUID-less duplicates and duplicate UUIDs are not redefined by this
ADR. The parser must not use synonym to distinguish them. Their broader source
policy remains deferred.

## Graph contract

`GraphNodePayload` gains a closed variant conceptually equivalent to:

```rust
pub enum GraphNodePayload {
    None,
    Metadata(MetadataPayload),
    MetadataMember(MetadataMemberPayload),
}
```

Controlled graph-node construction must reject:

- member payload on any node other than Attribute or TabularSection;
- top-level Metadata payload on a member node;
- any attempt to repair a mismatch by changing node kind, identity, or
  dropping content.

`GraphNode` exposes a borrowed convenience accessor for member payload. Existing
`new` and `new_with_provenance` compatibility constructors continue to create
`GraphNodePayload::None`; this avoids a repository-wide mechanical migration of
tests and producers. Compatibility construction is not EDT payload evidence.
The EDT Attribute and TabularSection production path must use explicit member
payload construction for both present and absent synonym values before Coverage
can change.

No new NodeKind, EdgeKind, ownership endpoint, reference endpoint, graph index,
or placeholder node is introduced.

## Ownership and reference compatibility

This decision preserves the completed first slice unchanged:

- top-level Attributes and TabularSections remain owned by their metadata
  object;
- a nested Attribute remains owned only by the nearest TabularSection;
- a nested Attribute receives no companion metadata-object `Contains` edge;
- source UUID and owner-scoped UUID-less fallback rules remain unchanged;
- metadata type requests still originate only from Attribute, Dimension, and
  Resource under the current contract;
- the nine ADR-0025 target mappings remain exact;
- resolved requests may emit the current `References` and ADR-0017
  `DependsOn`; failures emit no resolved or placeholder edge;
- synonym does not affect request identity, candidates, terminal state,
  diagnostics, statistics, or provenance aggregation.

No Task 06 production reference change is required by this ADR. After member
payload emission is implemented, the reference task must be closed as
`already_complete` by focused regression evidence and the existing accepted
commits rather than an empty commit.

## Provenance

Member payload contains no adapter path, locale key, producer, confidence, or
resolution state. The existing member node provenance identifies the `.mdo`
descriptor, semantic member identity, producer, parsed/declared origin, and
confidence. That node provenance applies to its accepted payload.

The first slice does not add field-level source spans or per-locale provenance.
Changing payload must not change provenance identity or source-context encoding
unless the underlying source observation changes.

## Query, Diff, Impact, and index behavior

The existing Query API returns borrowed GraphNode values. The member payload
accessor therefore makes synonym observable through identity, kind, exact-name,
owner, child, and traversal queries without a second query surface.

The first slice does not add synonym search, payload predicates, locale filters,
or a synonym index. Exact canonical-name lookup remains unchanged.

`NodeSnapshot` already contains `GraphNodePayload`. A synonym-only change:

- preserves node identity;
- appears as one modified node with
  `NodeModifiedAspect::SemanticContent`;
- is not represented as node removal and addition;
- leaves containment and reference edges unchanged.

Impact sees the member as directly changed. It may propagate through existing
explicit ownership options, but this ADR adds no new propagation rule.
Semantic Index and Incremental Index continue indexing identity, name, kind,
adjacency, and containment. They retain the canonical node snapshot content and
must remain equivalent to a clean rebuild; synonym does not add an index
dimension.

Reference reports, diagnostics, and statistics remain unchanged because member
synonym is not a reference fact.

## Serialization and public compatibility

The repository has no public graph serialization contract. A future serialized
form must use a versioned tagged `metadata_member` payload and distinguish absent
synonym from a present string. This ADR does not add persistence or wire format.

Public additions are limited to the source-independent member payload type,
the GraphNodePayload variant, controlled construction compatibility, and a
borrowed accessor. Existing node constructors and existing EDT child accessors
remain source-compatible unless implementation evidence proves a necessary
additive constructor migration.

Known consumers to audit are:

- metadata-domain tests and public re-exports;
- GraphNode payload compatibility and validation tests;
- Query, Diff, Impact, Semantic Index, and Incremental Index tests;
- EDT metadata-structure descriptors and parser tests;
- EDT child emission and real ownership/grants builders;
- graph-domain and EDT Coverage registries and representative tests.

## Coverage completion criteria

Architecture acceptance adds no evidence, changes no status, and changes no
aggregate count.

For graph-domain `SemanticNode(Attribute)` and
`SemanticNode(TabularSection)`, `SemanticPayloadPreserved` may be added to
required and present evidence only after:

1. the source-independent type and graph variant exist;
2. compatible Attribute/TabularSection shapes are accepted and every unrelated
   node kind is rejected;
3. public access through GraphNode and Query is tested;
4. absent, present, non-ASCII, equality, and payload-only Diff behavior is
   proven;
5. Semantic Index and Incremental Index retain clean-rebuild equivalence;
6. existing identity, ownership, validation, and reference regressions pass.

For EDT `SemanticNode(Attribute)` and `SemanticNode(TabularSection)`, the same
evidence may transition independently only after:

1. the parser preserves the accepted direct single-value form and absence;
2. missing, empty, duplicate, unsupported encoding, and nested-owner leakage
   behavior has focused tests;
3. the production builder explicitly emits payload for present and absent
   values;
4. the grants real fixture proves present TabularSection and nested Attribute
   values, while the ownership fixture proves absence;
5. Query, Validation, graph/build Diff, Impact, provenance, references, and
   repeated-build results remain deterministic;
6. representative test names and registry aggregates are recomputed from the
   live registry.

The capabilities are already `Supported` for their existing required evidence.
The registry task adds the new required evidence and proof atomically; it must
not temporarily claim Supported with missing evidence. Status and aggregate
status counts are therefore expected to remain unchanged.

## Ordered implementation prerequisites

1. Add `MetadataMemberPayload` to `oneagent-metadata` and
   `GraphNodePayload::MetadataMember` plus compatibility/accessor tests to
   `oneagent-graph`.
2. Prove validation, Query, payload-only Diff, Impact, Semantic Index, and
   Incremental Index compatibility without changing canonical lookup behavior.
3. Extend the EDT child descriptor and parser for the direct single `value`
   member synonym form, with typed missing/duplicate/unsupported behavior and
   focused tests.
4. Emit explicit present or empty member payload through the production child
   contributor while preserving node identity, provenance, ownership, and
   source-order independence.
5. Close the reference-integration task as already complete after focused
   request, endpoint, statistics, and repeated-build regressions pass.
6. Add real-fixture present/absent production evidence, transition the two
   graph-domain and two EDT payload evidence entries atomically, recompute live
   aggregates, and synchronize current-state documentation.
7. Complete the Sprint 6 integration review and status transition only after
   focused and full workspace validation passes.

## Rejected alternatives

### Reuse top-level MetadataPayload on member nodes

Rejected because MetadataPayload is a metadata-object contract with
kind-specific top-level variants. Reusing it would blur independently
addressable members with their owner and contradict ADR-0023 boundaries.

### Store synonym directly on GraphNode

Rejected because a generic graph field would make metadata-specific display
content part of every node and provide no closed extension path for member
content.

### Store an EDT string map

Rejected because source element names would leak into the source-independent
domain, weaken compatibility, and make validation implicit.

### Put synonym in node identity or canonical name

Rejected because display text may change while the same UUID-backed or
owner-scoped member remains. It must produce semantic modification, not identity
churn or name-resolution changes.

### Implement all observed member fields

Rejected because the repository has no accepted source-independent meaning or
complete evidence for qualifiers, history/search settings, produced types,
line-number length, or TabularSection standard attributes.

### Accept multiple locales or member synonym content encoding

Rejected for the first slice because real member evidence contains one direct
`value` only. Locale selection, locale collections, and `content` encoding need
separate investigation.

### Add or change reference semantics

Rejected because synonym is not a reference and the current request lifecycle
and precise endpoint matrices already cover all accepted member types.

## Deferred scope

- Member synonyms with `content`, several locale values, locale identity, or
  locale selection.
- Present synonym on a real top-level Attribute and non-Document owner fixture
  families beyond the generic parser contract.
- Same-owner UUID-less duplicates, duplicate UUIDs, and rename migration.
- TabularSection `LineNumber` and other standard attributes.
- Primitive type payload, precision, scale, length, allowed values, or
  qualifiers.
- `dataHistory`, `fullTextSearch`, produced types, and line-number length.
- Deeper ownership nesting and new Contains endpoints.
- New reference categories, target mappings, edges, diagnostics, or request
  families.
- Synonym search/indexing, serialization, persistence, Runtime, API transport,
  Forms, Commands, and later sprint work.

## Risks

- A generic XML text handler could assign a nested Attribute synonym to its
  TabularSection. Depth-specific parser tests are required.
- Compatibility constructors could leave EDT nodes with `None` payload while
  appearing migrated. Production must use explicit member payload for both
  present and absent values before Coverage changes.
- Adding a payload variant affects exhaustive matches. The consumer audit and
  full workspace validation are mandatory.
- Treating the locale key as selected semantics would overstate the first
  slice. The domain stores only the single proven display value and defers
  locale identity.
- Registry requirements could change without evidence. Required and present
  evidence must move atomically in the final Coverage task.

## Consequences

### Positive

- Attribute and TabularSection display content becomes observable without EDT
  dependency.
- Identity, ownership, references, and canonical names remain stable.
- Query and Diff reuse existing typed-node and snapshot behavior.
- The first slice is bounded by two committed real-format fixtures.
- Other EDT fields remain explicitly deferred instead of silently generalized.

### Negative

- Metadata and graph public APIs gain a new typed payload and exhaustive enum
  variant.
- The EDT reader needs explicit nested-path and duplicate-value handling.
- Coverage requires real builder evidence for both present and absent values.
- The first slice intentionally does not model locale identity or provide
  synonym search.

## Decision outcome

Sprint 6 accepts optional Attribute and TabularSection synonym as closed typed
member content. The graph stores it in a member-only payload variant; EDT reads
only the repository-proven direct single `value` form. Synonym never changes
identity, ownership, references, or canonical-name lookup. Production and
Coverage remain unchanged until the ordered implementation evidence is
complete.
