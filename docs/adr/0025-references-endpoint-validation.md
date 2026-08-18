# ADR-0025: References Endpoint Validation

## Status

Accepted

## Context

`SemanticGraphSchema::allows()` delegates every current `EdgeKind` to an
endpoint rule, but the rules were introduced at different stages. The schema
Rustdoc still says that broad dependency-like relations are intentionally
unrestricted. That description is stale: accepted ADRs and implementation work
have narrowed Calls, DependsOn, Extends, Grants, Includes, Reads, and Writes,
while Contains uses its ownership matrix.

`References` is the remaining exception. Its current rule accepts an edge when:

- the source is `NodeKind::Unknown`;
- the target is `NodeKind::Unknown` or any `NodeKind::Metadata(_)`; or
- both endpoints belong to a broad participant set containing Metadata,
  Module, Procedure, Function, Query, Form, Command, Attribute,
  StandardAttribute, TabularSection, Dimension, Resource, and Measure.

This permits many combinations that no current producer emits and explicitly
permits Unknown endpoints that current production contracts reject.

Two production paths currently emit References:

1. metadata member type resolution emits a reference from an Attribute,
   Dimension, or Resource to one of nine mapped metadata target kinds;
2. role-right resolution emits a companion reference from a scoped
   `AccessRight` node to its resolved protected metadata resource.

The second path is required by ADR-0019's reified ternary grant model. The
current broad rule happens to allow it because every Metadata target is
accepted, even though `AccessRight` is absent from the broad participant list.
No single accepted contract currently states both production endpoint slices
or distinguishes them from future BSL, UI, external, unresolved, or placeholder
references.

## Decision

Adopt a precise additive endpoint contract for the currently emitted
`EdgeKind::References` slices. A later implementation task must replace only
the broad References rule with this contract and add exhaustive graph-domain
validator evidence.

The canonical direction remains:

```text
referencing semantic entity --References--> resolved referenced entity
```

References is a direct resolved semantic fact. It does not authorize a
dependency, call, data access, mutation, ownership, grant, inclusion, or
extension fact. Both endpoints must already exist. Unresolved, ambiguous,
incompatible, external, Unknown, and placeholder targets emit no References
edge.

## Complete current EdgeKind audit

The audit uses production insertion sites, accepted ADRs, focused validator
tests, and production integration tests. A taxonomy-level rule is not
classified as broad merely because it covers several accepted node kinds;
"broad" means that the rule authorizes endpoint semantics without a current
accepted contract.

| EdgeKind | Current endpoint rule | Production producers and emitted pairs | Authority and evidence | Classification |
|---|---|---|---|---|
| `Contains` | Ownership matrix: Metadata owners for top-level or metadata children; TabularSection may own Attribute; Module owns Procedure/Function; Procedure/Function owns Query | EDT configuration and metadata contributors, EDT BSL contributor, and `oneagent-analysis`; accepted pairs are configuration-to-metadata, metadata-to-module/member/form/command, tabular-section-to-attribute, module-to-callable, and callable-to-query | ADR-0007, ADR-0012, ownership inventory; `procedure_can_own_query_node`, `tabular_section_can_own_attribute`, `module_cannot_own_attribute`, `child_without_owner_is_error`, and production ownership tests | Precise taxonomy-level ownership policy; not a fallback |
| `Calls` | Procedure or Function to Procedure or Function | EDT local and cross-module call emission and `oneagent-analysis` call graph | ADR-0015 and ADR-0016; `invalid_edge_endpoint_combination_is_error`, EDT resolved/unresolved call tests, and analysis call tests | Precise endpoint family |
| `References` | Unknown wildcard, any Metadata target, or any two broad reference participants | EDT metadata member types: Attribute/Dimension/Resource to mapped Metadata; EDT role rights: AccessRight to protected Metadata | Metadata reference production tests and grants production tests; ADR-0019 covers the companion AccessRight relation, but no complete endpoint contract or focused validator matrix existed before this ADR | Stale permissive implementation; this ADR supplies the missing complete contract |
| `Reads` | Query to Catalog or Information Register Metadata | EDT query-language Reads contributor | ADR-0021; exhaustive `reads_schema_*`, graph validation, and production Reads tests | Precise first-slice matrix |
| `Writes` | Procedure to Accumulation Register Metadata | EDT Writes contributor | ADR-0022; exhaustive `writes_schema_*`, graph validation, and production Writes tests | Precise first-slice matrix |
| `Grants` | Role to AccessRight | EDT role-right contributor | ADR-0019; positive/negative validator and production Grants tests | Precise first-slice matrix |
| `Includes` | Subsystem to the explicit 19-kind Metadata allowlist | EDT Subsystem content contributor | ADR-0020; allowlist, source rejection, self-loop, and production Includes tests | Precise first-slice matrix |
| `Extends` | Metadata(kind) to Metadata(same kind) | EDT adopted metadata-object extension contributor | ADR-0018; same-kind positive, cross-kind/unrelated negative, self-loop, and production Extends tests | Precise first-slice matrix |
| `DependsOn` | Attribute, Dimension, or Resource to Metadata | EDT metadata type-reference resolution emits the normalized companion edge | ADR-0017; positive member matrix, unrelated negative pairs, and metadata reference production tests | Precise first-slice matrix |

The concrete repository evidence is:

- endpoint rules and shared invariants in
  `crates/graph/src/validation.rs`;
- focused endpoint tests in `crates/graph/tests/validation.rs`;
- EDT structural, metadata-reference, Includes, Extends, Grants, and
  DependsOn production in `adapters/edt/src/lib.rs`;
- EDT Calls and Reads production in `adapters/edt/src/bsl_graph.rs`;
- EDT Writes production in `adapters/edt/src/writes_emission.rs`;
- the additional declaration/call graph contributor in
  `crates/analysis/src/lib.rs`;
- production integration tests in `adapters/edt/src/lib.rs`,
  `adapters/edt/tests/ownership.rs`, `adapters/edt/tests/reads.rs`,
  `adapters/edt/tests/writes.rs`, `adapters/edt/tests/grants.rs`, and
  `adapters/edt/tests/includes.rs`;
- accepted contracts in `docs/adr/0007-edt-to-semantic-graph.md`,
  `docs/adr/0012-bsl-symbols-in-semantic-graph.md`,
  `docs/adr/0015-local-calls-in-semantic-graph.md`,
  `docs/adr/0016-cross-module-bsl-call-resolution.md`, and
  `docs/adr/0017-depends-on-semantics.md` through
  `docs/adr/0022-writes-semantics.md`.

The exact remaining permissive endpoint surface is therefore only
`EdgeKind::References`. No emitted dependency, access, composition, or
extension edge other than References lacks a precise endpoint family.

This audit is limited to endpoint-kind policy. Separate graph invariants such
as cycle policy, self-loop policy, ownership cardinality, provenance warnings,
and build-report consistency are not redefined here.

## Current References endpoint matrix

### Metadata member type references

The metadata structure reader recognizes reference targets only from
Attribute, Dimension, and Resource type declarations. The accepted matrix is
the full cross-product of these source kinds and the nine currently mapped
target kinds:

```text
source:
    NodeKind::Attribute
    NodeKind::Dimension
    NodeKind::Resource

target:
    NodeKind::Metadata(MetadataKind::Catalog)
    NodeKind::Metadata(MetadataKind::Document)
    NodeKind::Metadata(MetadataKind::Enumeration)
    NodeKind::Metadata(MetadataKind::InformationRegister)
    NodeKind::Metadata(MetadataKind::AccumulationRegister)
    NodeKind::Metadata(MetadataKind::AccountingRegister)
    NodeKind::Metadata(MetadataKind::CalculationRegister)
    NodeKind::Metadata(MetadataKind::BusinessProcess)
    NodeKind::Metadata(MetadataKind::Task)
```

The matrix is deliberately aligned with the implemented reference-prefix map
and the nine production-builder fixtures. It is not shorthand for every
`MetadataKind`. Adding a parser mapping does not silently expand graph
validation; parser, production, validation, and test evidence must move
together.

### Scoped access-right resource references

ADR-0019 requires one companion relation from the reified scoped right to its
protected resource:

```text
source:
    NodeKind::AccessRight

target:
    NodeKind::Metadata(MetadataKind::Configuration)
    NodeKind::Metadata(MetadataKind::Catalog)
    NodeKind::Metadata(MetadataKind::Document)
    NodeKind::Metadata(MetadataKind::InformationRegister)
    NodeKind::Metadata(MetadataKind::AccumulationRegister)
```

These are exactly the resource kinds accepted by the current
`protected_resource_reference` production mapping and proven by the Grants
integration fixture. The companion edge preserves resource navigation; the
right identity remains in the AccessRight node, and the Role-to-AccessRight
authorization fact remains `Grants`.

### Forbidden pairs

The current contract rejects every pair outside the two matrices, including:

- either endpoint as `NodeKind::Unknown`;
- `NodeKind::Metadata(MetadataKind::Unknown)` as target;
- Module, Procedure, Function, Query, Form, Command, StandardAttribute,
  TabularSection, Measure, Role, or Subsystem as a References source;
- metadata members, callables, flat semantic nodes, or AccessRight as targets;
- metadata-member targets outside the nine-kind type map;
- AccessRight targets outside the five protected-resource kinds;
- unresolved, ambiguous, incompatible, external, or placeholder targets;
- missing endpoints.

A physical self-loop is impossible for the accepted endpoint families. This
ADR does not add a separate References self-loop rule.

## Production and provenance contract

The validator checks node kinds, not source syntax. Producers remain
responsible for proving their accepted source facts and exact resolution.

Every emitted References edge uses the standard identity:

```text
(source_node_id, target_node_id, EdgeKind::References)
```

Every production edge must retain non-empty deterministic resolved provenance.
For metadata type references it identifies the descriptor, metadata owner,
source member, reference role, mapped target kind/name, and resolved target.
For AccessRight resource references it identifies the role-right artifact,
role, protected resource, right, explicit allow value, scoped-right identity,
and resolved resource. Both use `ResolutionState::Resolved`; provenance does
not participate in edge identity.

Equivalent observations aggregate deterministically into one edge with sorted,
deduplicated provenance. Distinct source-target pairs remain distinct edges.
The endpoint validator must not inspect encoded provenance to decide whether a
pair is allowed.

## Deferred reference families

The following are not authorized by this ADR:

- BSL symbol or call references beyond resolved `Calls`;
- Procedure, Function, Module, or Query references to metadata;
- Query fields, parameters, or data sources beyond accepted `Reads`;
- UI Form, Command, binding, or event references;
- metadata member-to-member references;
- reference types for metadata kinds outside the nine-kind map;
- non-metadata protected resources;
- external platform symbols or cross-workspace entities;
- unresolved, ambiguous, partial, Unknown, or placeholder nodes;
- future request-ledger observations from ADR-0024 before they resolve.

Each future family requires an accepted source and endpoint extension,
production evidence, provenance, negative behavior, and an additive validator
change. A generic phrase such as "semantic reference" is not sufficient
authority to reopen the broad participant matrix.

## Compatibility impact

`SemanticGraph::insert_edge` currently enforces endpoint existence but does not
apply `SemanticGraphSchema`; the later narrowing changes `validate()` results,
not edge storage or identity. A manually constructed graph that uses a
currently broad-only References pair will remain constructible but will become
semantically invalid.

Repository generic Query tests currently construct storage-only references
such as Procedure-to-Metadata and Procedure-to-Function to exercise relation
filtering. They are not production evidence and do not authorize those endpoint
pairs. The implementation task must keep such fixtures isolated from schema
validation or replace them with accepted pairs when validation is part of the
test.

No current EDT or analysis production producer emits a pair rejected by the
new matrix. The implementation must prove this by running the complete
workspace tests and the metadata-reference and Grants production builders.

## Ordered implementation follow-up

1. Replace `allows_reference` with two explicit source/target matrices matching
   this ADR. Remove Unknown and broad-participant fallback acceptance.
2. Update the stale `SemanticGraphSchema::allows()` Rustdoc so it describes
   every current edge as delegated to an explicit accepted endpoint policy.
3. Add a deterministic exhaustive positive schema test for all 27 metadata
   member pairs and all five AccessRight resource pairs.
4. Add exhaustive negative schema tests over every current `NodeKind`, every
   current `MetadataKind`, both Unknown representations, wrong member targets,
   wrong AccessRight targets, and reversed directions.
5. Add graph-level validation tests proving exact
   `InvalidEdgeEndpoints` code, severity, edge kind, endpoint kinds, node IDs,
   edge identity, provenance context, deterministic issue ordering, and
   repeated validation.
6. Prove accepted provenance-backed References edges remain valid and missing
   provenance remains governed by the existing provenance invariant.
7. Run metadata-reference and Grants production integration tests to prove no
   current producer regresses, then run full workspace validation.
8. Synchronize current-state documentation and mark Roadmap item 25 complete
   only after the implementation and tests pass.

The implementation is one focused graph-validation change. It must not modify
reference extraction, resolution, emission, identity, Query, Impact, graph
storage, diagnostics, request lifecycle, or metadata mappings.

## Coverage impact and completion criteria

Architecture acceptance changes no Coverage evidence, status, priority, or
aggregate count. `semantic_edge.references` is already `Supported` because
production emission, provenance, Query, validation presence, and integration
evidence exist. This ADR does not claim that the current validation presence is
precise.

Roadmap item 25 may be completed only when:

- the broad References branch is removed;
- every accepted pair and every forbidden family has focused deterministic
  graph-domain validator evidence;
- metadata-reference and Grants production tests remain green;
- no current producer emits an invalid pair;
- the stale schema comment and current-state documentation are updated;
- full workspace validation passes.

No automatic registry transition or count change is expected from the
narrowing because it completes the semantic precision of existing
`ValidationRuleExists` evidence rather than adding a new edge capability. If a
later audit changes registry evidence, that change must be isolated and based
on passing implementation tests; it must not be inferred from this ADR.

## Rejected alternatives

1. Keep the participant fallback as an intentionally extensible reference
   policy. Rejected because it authorizes unimplemented semantics and Unknown
   endpoints without source or provenance contracts.
2. Allow every source to any Metadata target. Rejected because target taxonomy
   alone does not prove reference meaning or source intent.
3. Narrow only to metadata member type references. Rejected because the
   production Grants slice already emits required AccessRight resource
   references.
4. Treat AccessRight resource navigation as Grants only. Rejected because
   ADR-0019 reifies the right/resource pair and requires a companion relation
   for direct resource navigation.
5. Allow Procedure-to-Metadata or Procedure-to-Function because generic Query
   tests construct those pairs. Rejected because storage/query fixtures are not
   production contracts; Calls, Reads, Writes, and DependsOn already model
   accepted specialized meanings.
6. Preserve unresolved references with Unknown or placeholder target nodes.
   Rejected because ADR-0024 uses a build-level request ledger and diagnostics,
   not false resolved edges.
7. Change production emission during validator narrowing. Rejected because all
   current production pairs already fit the accepted matrix.
8. Change Coverage counts during this architecture task. Rejected because no
   implementation evidence changed.

## Consequences

- Every current EdgeKind has an accepted endpoint policy.
- References remains broad in production code until the ordered implementation
  task is completed, so Roadmap item 25 remains open.
- The future narrowing preserves all current production References edges and
  rejects only unsupported manual or future pairs.
- New reference families require explicit additive contracts rather than
  silently inheriting a participant wildcard.
- Production validation and Coverage remain unchanged by this ADR.
