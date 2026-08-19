# Attribute and Tabular Section Source Investigation

## Status

Decision-ready investigation recorded for Sprint 6.

This document is evidence, not an architecture decision. It does not change
production behavior or Semantic Coverage.

## Objective

Identify the live EDT source boundary, implemented semantic behavior, consumer
surface, and smallest evidence-backed capability that can extend the completed
Attribute and TabularSection first slice without inventing source semantics.

The investigation was performed against repository HEAD `922ba70`. Mutable
repository facts must still be rechecked before implementation.

## Evidence classification

- **Confirmed** means directly present in repository code, tests, fixtures, or
  committed review evidence.
- **Accepted** means normative in an accepted ADR or authoritative architecture
  document but not necessarily implemented for member content.
- **Unknown** means the repository does not contain enough source evidence or an
  accepted semantic contract.

## Repository-owned source inventory

### Real-format EDT artifacts

| Artifact | Attribute and tabular-section evidence | Current production evidence |
|---|---|---|
| `adapters/edt/tests/fixtures/ownership_project/src/Documents/Sales/Sales.mdo` | One UUID-backed top-level Attribute; one UUID-backed TabularSection; two UUID-backed nested Attributes; one nested primitive Number type with qualifiers; one nested duplicate metadata type observation; a tabular-section `LineNumber` standard attribute; `lineNumberLength`. No member synonym. | `adapters/edt/tests/ownership.rs` builds it through `FileSystemEdtSemanticGraphBuilder` and proves immediate ownership, reference projection, provenance, Query, Validation, Coverage, repeated builds, and source-order independence. |
| `adapters/edt/tests/fixtures/grants_project/src/Documents/Sale/Sale.mdo` | One UUID-backed TabularSection with direct `<synonym><key>ru</key><value>...</value></synonym>`; four UUID-backed nested Attributes; three Attributes have the same direct synonym shape; one Catalog reference; three Number types with precision/scale; `dataHistory`, `fullTextSearch`, and `lineNumberLength`. | `adapters/edt/tests/grants.rs` builds the complete fixture through the production builder for Grants evidence. It proves that the artifact is accepted by the live builder, but its assertions do not verify member synonym or other member content. |

No other committed `.mdo` fixture contains an Attribute or TabularSection
element. The real-format owner matrix is therefore limited to Document.
Catalog, register, Business Process, Task, and other owner families are not
real-artifact evidence for Attribute or TabularSection source variants.

### Generated and inline source evidence

`adapters/edt/src/metadata_structure.rs` contains focused generated XML tests
for:

- multiple top-level Attributes and one TabularSection;
- top-level and nested Attributes with immediate parents;
- two UUID-less TabularSections containing equal UUID-less Attribute names;
- a UUID-less top-level Catalog Attribute;
- direct and composite metadata type observations;
- Dimensions, Resources, Forms, and Commands through the same reader.

`adapters/edt/tests/ownership.rs` also generates two equivalent Document
descriptors with the top-level Attribute and TabularSection in opposite source
orders. These generated forms prove current parser and graph behavior, but they
do not independently authorize new EDT fields.

## Confirmed implementation boundary

### Parser model

`EdtMetadataChildDescriptor` in
`adapters/edt/src/metadata_structure.rs` contains only:

- `EntityId`;
- `EntityName`;
- `EdtMetadataChildKind`;
- immediate `parent_id`;
- metadata type reference descriptors.

The reader recognizes `attributes`/`attribute`,
`tabularSections`/`tabularSection`, Dimensions, Resources, Forms, and Commands.
It reads a child UUID, the first child name, and every `types` text observed
while that child is pending. It does not retain synonym, type qualifiers,
standard attributes, produced types, data-history settings, full-text-search
settings, or line-number length.

A completed Attribute is buffered under the nearest still-pending
TabularSection. All other completed children are emitted under the metadata
object. `finish_child` emits the TabularSection before its buffered nested
Attributes.

### Identity and duplicate behavior

- A present source UUID is used unchanged as `EntityId`.
- A UUID-less top-level child uses
  `<metadata-object-id>:<child-kind>:<raw-name>`.
- A UUID-less nested Attribute is finished with its TabularSection identity as
  parent, so its fallback includes the immediate owner.
- Equal UUID-less Attribute names under different UUID-less TabularSections are
  proven not to collide by a focused parser test.
- Repeated reads of the same generated source return equal descriptors.
- The behavior of two UUID-less children with the same kind and name under the
  same owner is not proven. Their current fallback inputs are equal, so a future
  contract must classify this as duplicate or conflict rather than assume a
  distinct identity.
- Duplicate source UUIDs, conflicting duplicate observations, and renamed
  UUID-less members are not covered by real artifacts or focused negative
  tests.

### Ownership and graph emission

`collect_metadata_child` in `adapters/edt/src/lib.rs` emits the existing
`NodeKind::Attribute` or `NodeKind::TabularSection` node with name, identity,
and declared provenance. The node uses `GraphNodePayload::None`.

`collect_metadata_child_ownership` emits `Contains` from the descriptor's
immediate `parent_id`. Node collection precedes ownership-edge insertion, so
nested XML completion order does not require the owner edge to be inserted
before its target node exists.

The graph schema accepts Metadata-to-Attribute, Metadata-to-TabularSection, and
TabularSection-to-Attribute containment. Validation requires one owner and
rejects missing, incompatible, and multiple ownership. The real ownership
fixture proves that a nested Attribute has only the TabularSection owner and no
companion metadata-object owner.

### References

The parser maps metadata type text only for the nine target kinds accepted by
ADR-0025. Primitive Number and other non-mapped types do not become metadata
reference requests. Attribute, Dimension, and Resource nodes are the only
current metadata-type request sources.

Accepted observations enter the public `SemanticReferenceRequest` ledger with
collection provenance. Exact name-and-kind resolution produces `References`
and the ADR-0017 companion `DependsOn`, or typed missing, ambiguous,
incompatible, and partial outcomes. Duplicate equal observations aggregate in
the request lifecycle. These reference semantics and their Coverage evidence
are completed Sprint 3 behavior, not a new Sprint 6 gap.

### Errors and unsupported input

The structure reader exposes typed errors for read failures, malformed XML,
missing names, an empty child with neither identifier nor name, invalid
identifiers, invalid names, and invalid mapped reference target names.
Unsupported type prefixes and primitive types are ignored as metadata
references. The repository has generated tests for several positive forms, but
does not contain a complete member-specific negative matrix for malformed
synonym, duplicate synonym values, conflicting child identity, unsupported
nesting, or unexpected owner families.

## Consumer and compatibility inventory

| Surface | Current dependency |
|---|---|
| Metadata domain | `oneagent-metadata` owns top-level `MetadataPayload`; it has no subordinate member payload type. ADR-0023 explicitly keeps members out of top-level metadata payload. |
| Graph node | `GraphNodePayload` is `None` or top-level `Metadata(MetadataPayload)`. Attribute and TabularSection nodes currently use `None`. Payload participates in equality and node snapshots. |
| Validation | Payload-kind compatibility is enforced at controlled construction. `Contains`, `References`, and `DependsOn` use precise endpoint matrices. |
| Query and Semantic Index | Node lookup, exact names, kinds, owners, and children expose borrowed `GraphNode` values. A future typed node payload is already observable through the node returned by Query; no adapter-specific query surface is needed. |
| Diff and Incremental Index | `NodeSnapshot` includes `GraphNodePayload`; a payload-only change is `NodeModifiedAspect::SemanticContent` and retains node identity. Incremental index refresh already treats node snapshot content as canonical input. |
| Impact | Payload-only node changes are direct changes. Optional ownership propagation uses `Contains`; no new impact edge is needed for display content. |
| Resolution and references | Owner resolution uses the shared containment index. Metadata type requests use child identity and do not depend on member payload. |
| Reports and build validation | Reference reports and validation depend on request outcomes, projections, diagnostics, and statistics, not display payload. |
| Coverage | Graph node capabilities require modeling, kind, identity, provenance, Query, and positive tests. EDT Attribute ownership and current reference/provenance capabilities are already `Supported`. The current registries do not claim subordinate member payload preservation. |

Known public construction consumers include graph-domain tests, EDT child
emission, Query/Diff/Impact tests, Semantic Index and Incremental Index tests,
and any direct `GraphNode` construction. Compatibility constructors currently
create `GraphNodePayload::None` and must remain available unless an accepted ADR
explicitly migrates them.

## Implemented-versus-missing matrix

| Capability | Evidence state | Sprint 6 classification |
|---|---|---|
| Attribute and TabularSection node kinds | Implemented and Supported | Compatibility baseline |
| UUID identity and owner-scoped UUID-less fallback | Implemented; real UUID and generated fallback evidence | Compatibility baseline; same-owner duplicate conflict still needs explicit policy |
| Immediate top-level and nested ownership | Implemented and Supported for real Document fixture | Compatibility baseline |
| Exact metadata type references and request lifecycle | Implemented and Supported | Compatibility baseline |
| Deterministic provenance, Query, Validation, Diff, Impact, repeated builds | Implemented for the accepted first slice | Compatibility baseline |
| Optional member synonym source | Present and absent cases exist in real Document artifacts; current parser discards it | Decision-ready smallest gap |
| TabularSection `LineNumber` standard attribute | One real artifact contains it; current model has no `LineNumber` standard-attribute kind or TabularSection ownership contract | Not decision-ready from the single existing artifact under the Roadmap gate |
| Number precision and scale | Present in two real artifacts; current reader intentionally ignores primitive type content | Deferred pending a typed member-type/value-domain contract |
| `dataHistory` and `fullTextSearch` | Present in one real artifact | Deferred; no accepted source-independent semantics |
| `producedTypes` and `lineNumberLength` | Present in real artifacts | Deferred; no accepted semantic contract |
| Deeper nesting and non-Document owners | No real repository artifact | Unknown; investigation prerequisite |
| Same-owner duplicate names/UUIDs and multiple synonym values | No real artifact or accepted policy | Unknown; negative policy prerequisite |

## Smallest decision-ready Sprint 6 slice

The smallest additional evidence-backed slice is optional display synonym
content for recognized Attribute and TabularSection nodes.

The source evidence is bounded:

- the direct child form is
  `<synonym><key>ru</key><value>text</value></synonym>`;
- one real TabularSection and three real nested Attributes contain exactly one
  direct value;
- the ownership fixture proves absence;
- current member identity, ownership, references, and provenance are already
  independent from this content.

Task 02 is decision-ready to define a closed source-independent member payload
whose first field is an optional synonym. The decision must keep synonym out of
identity, canonical name, ownership, reference keys, and top-level
`MetadataPayload`. It must define compatibility for Attribute and
TabularSection node kinds, controlled construction, absent versus present
values, payload-only Diff behavior, direct-value source parsing, duplicate or
multiple-value rejection, provenance, Query observability, and Coverage
criteria.

Only the single direct `<value>` source form shown by the real fixture is proven
for member synonym. Reusing the top-level metadata reader's `<content>` form,
selecting among several locales, normalizing whitespace, synthesizing from
name, or accepting multiple values would require separate evidence and must not
be inferred by Task 02.

This slice requires no new node kind, edge kind, containment rule, reference
category, target-kind mapping, placeholder node, or dependency semantics.
Consequently the later reference-integration task is expected to be
`already_complete` unless Task 02 finds contradictory live evidence; it must
not create an empty commit.

## Accepted constraints

- ADR-0003 keeps source-independent semantic content outside EDT.
- ADR-0006 and ADR-0007 keep the semantic graph canonical and adapters as fact
  producers.
- ADR-0023 prohibits copying subordinate members into top-level metadata
  payload and establishes synonym as display content rather than identity.
- ADR-0024 keeps reference requests as build observations with deterministic
  provenance and lifecycle.
- ADR-0025 permits only the current precise reference endpoint matrices.
- Roadmap requires unknown member forms to return to investigation before
  architecture or implementation.

## Unknowns and deferred evidence

- Member synonym forms using `<content>` or multiple locale entries.
- Locale selection, locale identity, and localized synonym collections.
- Present synonym on a real top-level Attribute.
- Attribute and TabularSection artifacts owned by non-Document metadata kinds.
- Same-owner UUID-less duplicates and duplicate UUID behavior.
- `LineNumber`, other kind-specific standard attributes, primitive type
  qualifiers, history/search flags, produced types, and line-number length.
- Deeper or otherwise unsupported ownership nesting.

These unknowns do not block the bounded optional-synonym slice, but they remain
outside its Coverage claim.

## Codex Framework readiness

The existing architecture, graph-model, parser, graph-emission, and review
profiles and templates express every required Task 2–8 boundary. No reusable
Framework gap was found, so `docs/codex/` must remain unchanged.

## Decision readiness and next action

Task 02 may proceed. It should accept or reject only the optional member-synonym
slice described above, define its typed domain and compatibility contract, and
leave every other observed field or source variant deferred. Architecture
acceptance alone must not change production behavior or Coverage.
