# ADR-0031: Conditional Direct Grants

## Status

Accepted

## Context

The completed Grants first slice models one direct EDT allow declaration as:

```text
Role --Grants--> AccessRight(resource, right)
AccessRight --References--> protected resource
```

`AccessRight` identity currently contains the resolved protected-resource ID and
the exact right token. EDT production accepts only explicit `true` declarations
for Configuration, Catalog, Document, Information Register, and Accumulation
Register resources. Deny, inheritance, access profiles, access groups, runtime
assignments, and effective authorization remain outside that contract.

Repository-owned EDT evidence also contains a narrower unmodeled fact. The
`BaseUser/Rights.rights` fixture stores `restrictionByCondition/condition` on
the `Read` and `Update` rights for `Catalog.Product`, with the exact condition
`WHERE NOT DeletionMark`. `EdtRoleRightsReader` already preserves that field as
`EdtRoleRowRestriction`, validates the surrounding XML structure, and rejects a
missing condition. The production Grants pipeline filters explicit allows and
then drops the preserved restriction before access-right identity, node
construction, provenance, and graph insertion.

The restriction qualifies the direct declaration. Treating a conditional and
an unconditional declaration as the same access-right node loses source
semantics. Attaching the text only to provenance would preserve audit data but
would not make conditionality a queryable semantic fact. Evaluating the
expression would require runtime data and authorization context that the
repository does not own.

## Decision

Sprint 9 adds opaque, source-preserving row-restriction content to the existing
direct Grants model. It does not interpret or execute the condition.

The canonical statements are:

```text
Role --Grants--> AccessRight(resource, right, no restriction)
Role --Grants--> AccessRight(resource, right, row restriction condition)
AccessRight --References--> protected resource
```

Both statements remain direct declared allow grants. A condition narrows the
declared right but does not prove an effective authorization result.

## Graph model

`GraphNodePayload` gains typed AccessRight content compatible only with
`NodeKind::AccessRight`. The payload records an optional row restriction whose
condition is opaque text. `GraphNode::access_right_payload()` exposes it without
requiring callers to decode entity IDs or provenance.

`AccessRight` continues to expose protected-resource ID and right ID. Its
existing constructor creates an unconditional right with the current identity
and display name unchanged. An additive constructor accepts an optional typed
row restriction.

The restriction constructor trims leading and trailing Unicode whitespace,
rejects a resulting empty value, and otherwise preserves the condition exactly,
including case, internal whitespace, identifiers, literals, and language. No
parser, normalizer, validator, or evaluator for the condition language is
introduced.

## Identity

Unconditional identity remains byte-for-byte compatible:

```text
access_right:resource#<length>:<resource-id>;right#<length>:<right-id>
```

A conditional identity appends one length-delimited component containing the
canonical trimmed condition:

```text
access_right:resource#<length>:<resource-id>;right#<length>:<right-id>;row_restriction#<length>:<condition>
```

Therefore:

- an existing unconditional graph has unchanged node and edge identities;
- equal resource/right/condition triples deduplicate deterministically across
  roles and duplicate declarations;
- conditional and unconditional declarations remain distinct;
- different conditions for the same resource/right remain distinct;
- provenance, role identity, source path, and traversal order do not affect
  access-right identity.

The display name for an unconditional right remains unchanged. A conditional
right uses a deterministic display suffix that distinguishes it without being
an identity input.

## Direction and endpoint compatibility

No `NodeKind`, `EdgeKind`, or endpoint matrix changes. The validator continues
to accept only:

```text
NodeKind::Role --Grants--> NodeKind::AccessRight
NodeKind::AccessRight --References--> accepted protected Metadata kind
```

Typed AccessRight payload is valid only on `NodeKind::AccessRight`. Legacy
payload-free AccessRight nodes remain constructible for public API and test
compatibility, but `SemanticGraph::insert_access_right` stores the typed payload
for domain-created rights.

## EDT production mapping

For every explicit `true` declaration on an accepted, uniquely resolved
protected resource, EDT production carries `EdtRoleRightDeclaration`'s optional
row restriction through the existing private resolution observation and graph
insertion pipeline.

The observation key, access-right aggregation key, companion References key,
and Grants target key include the optional canonical condition. Provenance for
the node and both edges records whether a row restriction is absent or present
and, when present, its canonical condition. Provenance is not part of semantic
identity.

False declarations continue to emit no grant fact. Missing, ambiguous,
incompatible, unsupported, and malformed inputs retain their existing typed
outcomes. Condition content does not change protected-resource resolution or
reference statistics.

## Consumer behavior

Query, Diff, Impact, reports, complete Semantic Index construction, and
incremental index maintenance already consume graph node IDs, kinds, payloads,
and edge identities generically. Sprint 9 must prove that they:

- expose conditional and unconditional rights as distinct nodes;
- retain typed restriction payload in node lookup and node diffs;
- preserve deterministic ordering;
- keep companion References and Grants navigation exact;
- produce incremental results equivalent to a clean rebuild.

No condition-specific evaluator or authorization query API is added.

## Provenance

Every conditional AccessRight node, Grants edge, and companion References edge
retains resolved provenance identifying the role artifact, role IDs, declared
and resolved resource, exact right token, explicit `true` value, accepted direct
allow status, restriction presence, canonical condition, and fact kind.

Multiple identical declarations aggregate provenance deterministically. A
conditional and an unconditional observation never aggregate into the same
semantic node or edge.

## Coverage and completion criteria

The existing AccessRight and Grants capabilities remain `Supported`; Sprint 9
adds evidence to their accepted production boundary and does not create a new
Coverage capability or change aggregate counts.

Sprint 9 is complete only when:

- typed AccessRight payload and conditional identity are implemented with
  unconditional compatibility;
- wrong-kind payload construction is rejected;
- real EDT row-restriction evidence reaches production graph nodes and edges;
- absent, present, duplicate, reordered, conditional-versus-unconditional, and
  distinct-condition cases are deterministic;
- false, missing, malformed, unsupported, unresolved, ambiguous, and
  incompatible behavior remains typed and non-emitting where required;
- Query, Diff, Impact, reports, complete index, incremental index, validation,
  provenance, and repeated-build evidence passes;
- graph and EDT Coverage status and aggregate counts remain unchanged;
- current-state architecture and Roadmap documentation match implementation;
- the complete workspace validation succeeds;
- the Sprint 9 integration review records a non-blocking decision.

## Deferred scope

This decision does not define or implement:

- parsing, normalization, type checking, compilation, or execution of RLS
  expressions;
- effective row filtering or authorization decisions;
- explicit deny semantics or negative graph facts;
- inherited, defaulted, or transitive rights;
- `setForNewObjects`, `setForAttributesByDefault`, or
  `independentRightsOfChildObjects` semantics;
- access profiles, access groups, BSP policy objects, runtime users, or role
  assignments;
- non-metadata or unsupported protected-resource families;
- direct Role-to-Metadata Grants edges;
- a new condition node, edge kind, authorization service, or persistence
  format.

## Rejected alternatives

1. **Keep the condition only in provenance.** Rejected because consumers could
   not distinguish conditional and unconditional semantic rights.
2. **Reuse the unconditional AccessRight identity.** Rejected because one node
   would merge declarations with different restriction semantics.
3. **Include the role ID in AccessRight identity.** Rejected because the node
   represents a scoped capability and identical declarations may be shared;
   the Grants edge already carries the subject dimension.
4. **Add condition text to Grants edge identity.** Rejected because the graph's
   canonical edge identity has no payload and changing it would affect every
   edge consumer.
5. **Introduce a condition node or new relation.** Rejected for this slice
   because repository evidence requires preservation, not an expression AST or
   evaluation graph.
6. **Model false values as deny.** Rejected because the source meaning relative
   to defaults and inheritance is not accepted and ADR-0019 explicitly defers
   deny semantics.
7. **Evaluate `WHERE NOT DeletionMark`.** Rejected because effective evaluation
   needs runtime data and context beyond the repository-owned source oracle.

## Migration and rollback

The change is additive for conditional declarations and preserves every
unconditional identity and public construction path. Existing graphs without
conditional payload remain valid.

If production evidence fails, conditional projection can be removed while
retaining the typed parser observation and accepted architecture. Coverage
counts must remain unchanged, documentation must state that production support
is pending, and no partially supported conditional graph fact may remain.

## Consequences

- Direct conditional allow declarations become source-independent, queryable
  graph facts without claiming effective authorization.
- Existing unconditional Grants behavior and identities remain compatible.
- Conditional identity can be longer because it preserves opaque condition
  content.
- Semantically equivalent but textually different conditions remain distinct;
  equivalence requires a future accepted language contract.
- Deny, inheritance, defaults, profiles, groups, users, and evaluation remain
  explicitly deferred.
