# ADR-0033: Event Subscription Semantics

## Status

Accepted

## Context

Sprint 11 must model EDT Event Subscriptions, their event identity, declared
source objects, Common Module handler procedures, and resulting semantic
relations. The repository currently ignores `src/EventSubscriptions`, has no
Event Subscription metadata kind or payload, and has no execution edge for a
declarative event dispatch.

The repository-owned source investigation in
`docs/architecture/event-subscription-source-investigation.md` proves 99 real
descriptors with stable UUIDs, canonical names, non-empty event values, one or
more source selectors, and exactly one three-component Common Module handler.
It also proves that source selectors have two distinct meanings:

- `Family.MetadataName` selects one exact metadata object;
- `Family` selects every metadata object in that source family.

The current graph can represent 162 observed selector occurrences across
Catalog, Document, Information Register, Accumulation Register, Accounting
Register, Calculation Register, Business Process, and Task metadata. The live
corpus also contains Constants, Defined Types, Exchange Plans, and Chart
families that OneAgent does not yet model.

All 93 unique live handlers resolve to one exported Procedure owned by the
named Common Module's Module node. Four multiline declarations were initially
misclassified by a line-oriented audit because `Export` appears on their final
declaration line. Event Subscription handler binding is nevertheless an
ownership relation rather than a BSL cross-module call, so it does not reuse
the exported-only call policy from ADR-0016. The reduced production fixture
exercises this boundary by recomposing an exact live non-exported owned
Procedure as a handler target.

ADR-0023 requires new intrinsic metadata content to use a closed typed payload.
ADR-0025 requires every new References family to have an explicit endpoint
matrix. ADR-0024's public request lifecycle currently requires exactly one
resolved candidate and therefore cannot represent an intentionally
multi-target bare source-family selector without a separate contract.

## Decision

Add Event Subscription as a top-level metadata entity with stable EDT UUID
identity, typed event payload, configuration ownership, direct resolved source
and handler References, and one specialized execution relation.

The canonical first-slice graph is:

```text
Metadata(Configuration)
    --Contains-->
Metadata(EventSubscription)
    --References--> Metadata(supported source kind)
    --References--> Procedure
    --Triggers----> Procedure
```

`References` records the two explicit declarations in the descriptor: selected
source objects and the handler procedure. `Triggers` records the specialized
execution meaning of the resolved handler declaration. No reverse edge or
derived closure is stored.

## Metadata model and identity

Add `MetadataKind::EventSubscription` with stable machine code
`event_subscription`. The universal top-level EDT descriptor reader and
production discovery path may use the existing source UUID as the canonical
`EntityId`; directory name, source-selector order, event value, handler path,
and payload do not participate in identity.

Add a closed kind-specific metadata payload conceptually equivalent to:

```rust
pub struct EventSubscriptionMetadataPayload {
    event: EntityName,
}

pub enum MetadataSpecificPayload {
    Document(DocumentMetadataPayload),
    EventSubscription(EventSubscriptionMetadataPayload),
}
```

The exact Rust names may be refined, but ownership and compatibility are
normative. `event` is the non-empty decoded direct `<event>` value. It remains
an exact typed name rather than a closed platform enum because the repository
has no authoritative complete event vocabulary. Changing the event modifies
semantic content without changing node identity.

Source selectors and handler paths are relation evidence and must not be copied
into payload. Descriptor path remains provenance. Optional synonym continues
to use `CommonMetadataPayload`; optional comment remains deferred because no
source-independent comment contract exists.

## Source parser contract

The dedicated EDT Event Subscription parser accepts exactly one direct:

- root Event Subscription UUID;
- `name`;
- `source` container with at least one `types` value;
- non-empty `event`;
- non-empty `handler`.

Each source selector has one or two non-empty dot-separated components. The
handler has exactly three non-empty components and the first component must be
`CommonModule`.

Missing or duplicate required direct fields, an empty source list, invalid UUID
or name, malformed XML, wrong root, unreadable file, or multiple descriptor
files is a fatal descriptor error. No Event Subscription node or partial
relations are emitted for that descriptor.

Malformed individual `types` values and unsupported prefixes are typed
rejected observations after the descriptor itself is structurally valid. They
do not prevent accepted selectors or a valid handler from being processed.
Source occurrences retain their ordinal context for provenance. Canonical
semantic ordering is independent of XML occurrence order.

## Supported source selectors

The first slice maps these serialized prefixes:

| Prefix | Target metadata kind |
|---|---|
| `CatalogObject`, `CatalogManager` | Catalog |
| `DocumentObject`, `DocumentManager` | Document |
| `InformationRegisterRecordSet` | Information Register |
| `AccumulationRegisterRecordSet` | Accumulation Register |
| `AccountingRegisterRecordSet` | Accounting Register |
| `CalculationRegisterRecordSet` | Calculation Register |
| `BusinessProcessObject`, `BusinessProcessManager` | Business Process |
| `TaskObject` | Task |

A qualified selector resolves by exact canonical metadata name and mapped
kind. A bare selector resolves to the complete deterministic set of metadata
nodes of the mapped kind in the current graph snapshot. An empty resolved set
is missing, not successful. Family results are ordered by stable node identity.

Manager and Object selectors are distinct source observations but may select
the same metadata target. Equivalent observations aggregate into one
`References` edge with sorted, deduplicated provenance. The edge identity
remains `(subscription_id, target_id, EdgeKind::References)`.

Every other prefix, including `ConstantValueManager`, `DefinedType`,
`ExchangePlanObject`, `ChartOfAccountsObject`,
`ChartOfCalculationTypesObject`, and `ChartOfCharacteristicTypesObject`, is
unsupported in the first slice. Unsupported observations emit deterministic
typed diagnostics and legacy rejected-observation statistics without creating
Unknown, external, placeholder, or guessed metadata nodes.

## Handler resolution

The handler path is:

```text
CommonModule.<module-name>.<procedure-name>
```

Resolution must prove this ownership chain in the built graph:

```text
Metadata(CommonModule, module-name)
    --Contains--> Module
    --Contains--> Procedure(procedure-name)
```

The resolver uses exact decoded names in the first slice and requires exactly
one candidate at each step. The handler target must be a Procedure. Function,
wrong-kind, missing, ambiguous, or invalid-owner outcomes emit a typed
diagnostic and no handler `References` or `Triggers` edge. Export status is not
part of this contract; a non-exported owned Procedure is valid.

One accepted handler observation records one resolved reference outcome even
though it projects both `References` and `Triggers`. Edge insertion does not
double-count statistics.

## Edge endpoint contracts

Extend ADR-0025 additively with this References matrix:

```text
source:
    NodeKind::Metadata(MetadataKind::EventSubscription)

target:
    NodeKind::Metadata(MetadataKind::Catalog)
    NodeKind::Metadata(MetadataKind::Document)
    NodeKind::Metadata(MetadataKind::InformationRegister)
    NodeKind::Metadata(MetadataKind::AccumulationRegister)
    NodeKind::Metadata(MetadataKind::AccountingRegister)
    NodeKind::Metadata(MetadataKind::CalculationRegister)
    NodeKind::Metadata(MetadataKind::BusinessProcess)
    NodeKind::Metadata(MetadataKind::Task)
    NodeKind::Procedure
```

Add `EdgeKind::Triggers` with exactly this first endpoint matrix:

```text
NodeKind::Metadata(MetadataKind::EventSubscription)
    --Triggers-->
NodeKind::Procedure
```

Every reversed or unrelated pair is invalid. Unknown, Function, Module,
unsupported MetadataKind, missing, unresolved, ambiguous, and placeholder
targets are forbidden. `Triggers` uses the standard edge identity and is
visible through generic Query, Diff, reports, validation, complete index, and
incremental index facilities.

`Triggers` is not independently added to the first dependency or Impact edge
classification. The accepted handler `References` relation already supplies
the existing dependency-navigation behavior; counting both relations as
dependencies would duplicate one declaration's propagation path. A future
execution-specific traversal policy requires separate evidence.

## Resolution lifecycle boundary

Event Subscription observations remain typed and adapter-private in this
sprint. They use existing source-independent diagnostics, provenance,
resolution errors, and statistics, but they are not inserted into the public
ADR-0024 `SemanticReferenceRequestLedger`.

This is required because a valid bare family selector intentionally resolves
to zero or more targets, while ADR-0024 currently defines `Resolved` as exactly
one candidate and `AmbiguousTarget` as multiple candidates. Treating a family
selector as ambiguous would be false, and splitting it into target-specific
requests would make request identity depend on resolution output.

The existing public ledger and its counts remain unchanged. A future
multi-target selector request contract must define identity, candidate-set
changes, partial-workspace behavior, diff, and statistics before this family
can migrate.

## Provenance, diagnostics, and determinism

Every emitted node and edge has non-empty deterministic provenance derived
from the descriptor path, subscription UUID, field role, source occurrence,
normalized selector or handler, resolved target, and producer stage. Provenance
does not participate in node or edge identity.

Resolved equivalent observations aggregate provenance. Diagnostics are sorted
and deduplicated through the existing graph-domain types. The first slice must
distinguish at least malformed selector, unsupported selector, missing target,
ambiguous target, incompatible target kind, invalid owner, and malformed
handler outcomes.

Reordered selectors and filesystem traversal must produce equal nodes, payload,
edges, diagnostics, statistics, reports, and indexes. Repeated builds of the
same source must be identical.

## Failure and partial behavior

Descriptor-structure errors are fatal to the complete production build, which
matches the current universal top-level metadata reader policy and prevents a
partially declared metadata object.

Resolution failures after a valid descriptor are recoverable observations:

- the Event Subscription node and configuration ownership remain;
- every independently resolved source edge remains;
- an unresolved handler emits no handler edges;
- an unsupported or unresolved source emits no source edge;
- all failures remain observable through diagnostics and statistics.

No unresolved observation creates a placeholder graph fact.

## Query, Diff, Impact, index, and report behavior

Existing generic APIs remain the public surface:

- Query exposes the Event Subscription node, ownership, References, and
  Triggers edges in deterministic order;
- Diff reports add/remove/modify for payload and direct edges through existing
  stable identities;
- Impact continues to classify References as dependency-like and does not
  independently propagate Triggers;
- complete and incremental Semantic Index state must match clean rebuilds for
  metadata kind, payload, adjacency, edge kind, provenance, resolution, and
  reports;
- no event-specific Query method or persisted derived closure is added.

## Coverage completion criteria

Architecture acceptance changes no Coverage status. Final Sprint 11 evidence
must recompute live aggregate counts after the new MetadataKind and EdgeKind
are added.

Graph-domain support requires:

- Event Subscription metadata kind and compatible typed payload;
- complete positive and negative References and Triggers endpoint matrices;
- deterministic Query, Validation, Diff, Impact-policy, report, complete-index,
  and incremental-index evidence;
- stable identity and payload-only modification evidence.

EDT support additionally requires:

- production discovery and parsing from `src/EventSubscriptions`;
- exact UUID/name/synonym/event preservation;
- exact and family source resolution;
- handler ownership resolution including a non-exported Procedure;
- positive, missing, ambiguous, incompatible, malformed, unsupported,
  duplicate, reordered, and repeated-build evidence;
- provenance-backed tracked production fixture and unchanged unrelated
  capabilities.

Coverage transitions occur only in the final production-evidence task after
all applicable evidence passes.

## Compatibility impact

Adding `MetadataKind::EventSubscription`,
`MetadataSpecificPayload::EventSubscription`, and `EdgeKind::Triggers` expands
public exhaustive enums. Repository consumers must be updated in the graph
model task. Existing variant names, codes, identities, constructors, endpoint
matrices, query results, and edge policies remain unchanged.

There is no graph serialization or persistence contract today. A future
serialized representation must version the new tagged variants. Cargo
workspace validation is required for every implementation task because the
enums and graph behavior are public.

## Rejected alternatives

1. Represent Event Subscriptions as flat `Unknown` nodes. Rejected because the
   source has stable metadata UUID identity and a dedicated top-level family.
2. Store source selectors and handler paths only as payload strings. Rejected
   because resolved semantic relations would remain unavailable and payload
   would duplicate relation evidence.
3. Use `Calls` for subscription dispatch. Rejected because Calls requires a
   callable source and represents code-level invocation.
4. Emit only `References` for the handler. Rejected because the descriptor has
   accepted execution meaning that warrants a precise specialized relation.
5. Emit only `Triggers` for the handler. Rejected because the handler remains
   an explicit resolved reference and existing dependency navigation is based
   on References.
6. Treat a bare family selector as ambiguous. Rejected because multiple
   selected sources are the declared meaning, not a resolution failure.
7. Split one family selector into public target-specific ADR-0024 requests.
   Rejected because target candidates are resolution content and must not enter
   request identity.
8. Add every observed unsupported metadata family now. Rejected because those
   entities lack source-independent identity, production, and Coverage
   contracts in OneAgent.
9. Require handlers to be exported. Rejected because export visibility governs
   BSL cross-module calls, while the accepted Event Subscription contract is an
   exact platform-owned Procedure binding. The reduced fixture proves the
   export-agnostic ownership rule without misrepresenting the live descriptor
   inventory.
10. Add Triggers to dependency and Impact policy immediately. Rejected because
    the companion References relation already provides the first-slice
    dependency path and duplicate propagation is unjustified.

## Deferred scope

- Constant, Defined Type, Exchange Plan, and Chart metadata families;
- public multi-target reference-request lifecycle;
- partial-workspace source-family resolution;
- case-insensitive aliases, extensions, and cross-project handlers or sources;
- handler signature validation and runtime dispatch simulation;
- platform-wide closed event enumeration and event-specific argument models;
- comments, subscription ordering, activation conditions, priorities, or
  effective runtime execution;
- event-specific Query APIs, execution reachability, dependency-policy changes,
  persistence, Runtime, API, CLI, MCP, LSP, IDE, Designer XML, and later-sprint
  entity families.

## Implementation order

1. Implement Event Subscription metadata/payload and graph endpoint rules.
2. Parse real EDT Event Subscription descriptors without graph emission.
3. Implement deterministic source and handler resolution outcomes without
   production edge emission.
4. Integrate discovery, node, ownership, References, Triggers, provenance,
   diagnostics, and statistics emission.
5. Complete tracked production evidence, consumers, indexes, Coverage, and
   current-state documentation.
6. Run the Sprint 11 integration review.
