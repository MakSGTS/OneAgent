# ADR-0035: XDTO and Service Semantics

## Status

Accepted

## Context

Sprint 13 must expand XDTO Package, HTTP Service, and Web Service semantics
beyond the existing top-level metadata-node coverage. The repository-owned EDT
corpus and exact inventory are recorded in
`docs/architecture/xdto-service-source-investigation.md`.

The current graph already preserves stable UUID identity, canonical name,
optional synonym, provenance, configuration ownership, service `Module.bsl`
nodes, and BSL Procedure/Function declarations. It does not preserve service
endpoints, Web operations/parameters, direct XDTO types, XDTO declarations, or
handler dispatch.

The real XDTO corpus is intentionally much larger than a safe first slice. It
contains 12,666 uniquely named direct Value/Object types but also 61,435 nested
properties, imports, restrictions, inline types, and repeated property names.
The latter require separate property identity, QName, inheritance, occurrence,
and schema-composition decisions. Modeling all visible XML as generic property
bags or ordinal nodes would create unstable and source-specific semantics.

The service corpus provides a smaller exact contract: stable UUIDs for every
HTTP URL Template/Method and Web Operation/Parameter; owner-local handler names
that resolve to existing service-module Functions for both HTTP and Web; exact
XDTO package references; exact `(namespace, type name)` declarations; and
deterministic negative or external outcomes. Source XML handler-field names do
not define the BSL declaration kind.

## Decision

Accept one additive source-independent first slice for:

- typed metadata content of XDTO Package, HTTP Service, and Web Service;
- direct XDTO Value/Object types;
- HTTP URL Templates and Methods;
- Web Service Operations and Parameters;
- exact internal XDTO package/type references;
- exact handler dispatch to existing service-module callables with
  source-proven family-specific kinds.

The semantic graph remains the only authority for nodes, ownership, references,
dispatch, provenance, validation, and generic query/index behavior. The EDT
adapter owns source parsing, artifact joins, diagnostics, and production
projection. No EDT XML type or path becomes a graph-domain API.

## Canonical node model

Add these source-independent node kinds:

```text
XdtoType
HttpServiceUrlTemplate
HttpServiceMethod
WebServiceOperation
WebServiceParameter
```

`XdtoType` uses a closed typed payload discriminator `Value` or `Object`.
Keeping one node kind allows exact namespace/name resolution without inventing
different reference grammars, while the payload preserves the direct source
family. The type discriminator is semantic content, not identity.

HTTP URL Template payload preserves exact decoded template text. HTTP Method
payload preserves the optional explicit `httpMethod` token; absence remains
distinct from a value and no default verb is inferred from the method name.

Web Operation payload preserves its return XDTO type declaration and optional
explicit nillability. Web Parameter payload preserves its XDTO type
declaration, optional explicit nillability, and typed transfer direction.
The first accepted transfer-direction values are the repository-proven `Out`
and `InOut`; absence is explicit. A different explicit token is a typed
unsupported source outcome until repository evidence accepts it.

The source-independent XDTO type declaration is the exact decoded pair:

```text
(namespace URI, local type name)
```

It is preserved even when the namespace is external and no graph target is
emitted. Empty namespace or local name is invalid source, not an external
reference.

## Metadata payload additions

Extend the closed `MetadataSpecificPayload` model with compatible variants:

- XDTO Package: exact descriptor namespace;
- HTTP Service: exact root URL;
- Web Service: exact namespace and zero-or-more canonical XDTO package
  declarations.

A Web Service package declaration is one of:

- a repository reference parsed from exact `XDTOPackage.<name>` syntax;
- an external namespace URI from a String value.

Declarations are deterministically ordered and deduplicated if the source
allows repeated equivalent values. Metadata UUID, name, kind, parent,
provenance, and module relations remain outside payload. Handler/procedure
bindings remain reference requests and edges rather than copied resolved IDs in
payload.

Payload changes preserve node identity and appear as
`NodeModifiedAspect::SemanticContent`. No payload field participates in graph
or edge identity.

## Identity

Use source UUID verbatim for:

- HTTP URL Template;
- HTTP Method;
- Web Service Operation;
- Web Service Parameter.

Existing metadata UUIDs remain unchanged.

A direct XDTO type has no source UUID. Its identity is a collision-safe
length-prefixed tuple of:

```text
(owner XDTO Package node ID, exact local type name)
```

The encoding must not collide when delimiters occur in either component. It
must not use namespace alone, type family, source position, filesystem order,
content, import order, property content, or an ordinal. The live corpus proves
that direct Value/Object names are unique across both families within one
package. Duplicate direct names are a deterministic source error, not separate
ordinal nodes.

All existing node and edge stable codes and identity encodings remain
unchanged.

## Ownership

The exact immediate `Contains` chains are:

```text
Metadata(XdtoPackage) --Contains--> XdtoType

Metadata(HttpService) --Contains--> HttpServiceUrlTemplate
HttpServiceUrlTemplate --Contains--> HttpServiceMethod

Metadata(WebService) --Contains--> WebServiceOperation
WebServiceOperation --Contains--> WebServiceParameter
```

Every accepted child requires exactly one immediate owner. Configuration still
owns the top-level metadata object, and HTTP/Web metadata objects still own
their existing Module nodes. Do not add transitive shortcut ownership, reverse
ownership, placeholder owners, or a second service/module node.

`Contains` remains structural and does not become an Impact dependency.

## Reference requests

Use the public source-independent `SemanticReferenceRequest` ledger from
ADR-0024 for every accepted repository-resolvable declaration.

Add two reference categories:

```text
XdtoPackage
XdtoType
```

Existing `Callable` owns service handler resolution.

Canonical requests are:

| Source | Category | Reference | Expected target |
|---|---|---|---|
| Web Service metadata node | `XdtoPackage` | exact package name | `Metadata(XdtoPackage)` |
| Web Operation or Parameter | `XdtoType` | exact child under package resolved by namespace | `XdtoType` |
| HTTP Method | `Callable` | exact child Function under the owning service Module | `Function` |
| Web Operation | `Callable` | exact child Function under the owning service Module | `Function` |

The adapter may first map an XDTO namespace to exactly one repository package,
then create the child request against that package. A missing namespace/package,
ambiguous package, missing child, ambiguous child, incompatible kind, or wrong
owner uses the typed ADR-0024 terminal lifecycle and deterministic candidate
order.

An external package namespace or external XDTO type namespace is a valid typed
external source observation, not a repository reference request. It emits no
candidate, placeholder, diagnostic claiming a missing local target, or graph
edge. Malformed package-reference grammar is a typed parser outcome and not a
request. Complete-workspace missing internal targets remain requests and typed
diagnostics.

Collection provenance is created before resolution and identifies the source
artifact, source semantic node, declaration role, exact reference expression,
producer, parsed/declared origin, exact confidence, and unresolved state.
Resolution appends resolved-stage provenance without rewriting collection
evidence. Requests, edges, diagnostics, and statistics follow ADR-0024
aggregation and consistency rules.

## References relations

Extend the precise `References` endpoint matrix only with:

```text
Metadata(WebService) --References--> Metadata(XdtoPackage)
WebServiceOperation --References--> XdtoType
WebServiceParameter --References--> XdtoType
HttpServiceMethod --References--> Function
WebServiceOperation --References--> Function
```

Direction is always referencing declaration to resolved target. No external,
missing, ambiguous, incompatible, Unknown, or placeholder target emits a
References edge. A Web Operation may have both one return-type reference and
one handler reference when both resolve; their different targets and request
categories keep them distinct.

This additive matrix does not broaden any existing metadata-member,
AccessRight, command, or Event Subscription reference family.

## Declarative dispatch

Reuse `EdgeKind::Triggers` for the generic declarative-dispatch statement:

```text
declarative dispatch source --Triggers--> resolved callable
```

Extend its accepted endpoints with:

```text
HttpServiceMethod --Triggers--> Function
WebServiceOperation --Triggers--> Function
```

The existing Event Subscription endpoint remains valid. `Triggers` states that
the declaration dispatches to the exact handler callable; it does not assert a
BSL body `Calls`, transport execution, authorization, dependency propagation,
or runtime availability. A resolved handler emits both `References` and
`Triggers`, each with deterministic relation-specific provenance. An
unresolved handler emits neither.

The current non-propagating Impact policy for `Triggers` remains unchanged.

## Parsing and artifact joins

### XDTO packages

The dedicated parser joins one already discovered XDTO Package descriptor to
exactly one `Package.xdto` in the same object directory. It validates exact
roots/namespaces, required descriptor UUID/name/namespace, required schema
`targetNamespace`, and equality between descriptor and schema namespaces.

It accepts direct `valueType` and `objectType` names, canonicalizes them by
exact name, rejects duplicates across both direct families, and records direct
`import` plus nested schema constructs as typed deferred observations. It does
not emit graph facts.

Missing, extra, ambiguous, unreadable, malformed, wrong-root, wrong-namespace,
namespace-mismatch, missing-name, empty-name, and duplicate-name conditions are
deterministic typed errors. Filesystem/XML order does not affect output.

### HTTP and Web Services

Dedicated service parsers enrich already discovered metadata descriptors. They
validate exact roots/namespaces and declared UUID/name ownership hierarchy.

The HTTP parser accepts service root URL, UUID-backed direct URL Templates,
UUID-backed nested Methods, template text, optional explicit HTTP method, and
required handler name.

The Web parser accepts service namespace, optional typed XDTO package
declaration, UUID-backed direct Operations, UUID-backed nested Parameters,
required return/value type declarations, optional Boolean nillability, optional
accepted transfer direction, and required Procedure binding.

Duplicate UUID/name conflicts inside their semantic owner, missing required
values, invalid reference grammar, invalid Boolean, unsupported explicit
direction, wrong hierarchy, unreadable/malformed XML, and exact source
cardinality violations are typed deterministic errors. Parsing is separate
from graph emission and handler/XDTO resolution.

## Production emission

Production discovery continues through the existing top-level directory map.
After generic metadata and BSL module/symbol insertion, the EDT builder:

1. joins and parses XDTO package artifacts and both service descriptor families;
2. enriches existing metadata payloads without changing identity/name/kind;
3. emits accepted child nodes and immediate Contains ownership;
4. collects package, type, and callable requests with provenance;
5. resolves against the complete graph snapshot;
6. projects terminal requests to References/Triggers or typed diagnostics;
7. derives legacy reference statistics exactly once from terminal requests.

Fatal structural parser/join failures fail the complete build and return no
successful partial graph result. Valid external declarations and explicitly
deferred XDTO constructs preserve accepted siblings and do not emit synthetic
facts. Equivalent observations aggregate deterministically. Reordered source
files/elements and repeated builds produce equal graph, requests, provenance,
diagnostics, statistics, report, and validation results.

## Validation, Query, Diff, report, and index contracts

- `GraphNodePayload` compatibility is closed for every new node and metadata
  payload variant; wrong kind/payload pairs are rejected.
- `SemanticGraphSchema` accepts only the new immediate Contains, additive
  References, and additive Triggers pairs above.
- Every accepted child has exactly one owner; multiple/missing/wrong owners,
  reversed edges, forbidden self-loops, missing endpoints, and unrelated pairs
  remain observable validation failures.
- Generic Query APIs navigate new kinds, payloads, ownership, References, and
  Triggers without an EDT-specific query authority.
- Content-only changes preserve IDs and appear as modified node facts. Target
  changes update requests and remove/add relation facts without changing source
  node identity.
- Reports include new node-kind distributions and existing relation/request
  totals deterministically.
- Complete and incremental indexes cover add/remove/modify/ownership/reference/
  dispatch transitions and remain equal to clean full rebuilds.
- `Contains` and `Triggers` remain excluded from default dependency Impact
  propagation. `References` keeps its existing explicit-filter semantics and
  does not silently become a dependency.

There is no serialization or persisted-format contract in this sprint.

## Coverage completion criteria

Graph Domain Coverage may transition new semantic-node and ownership
capabilities only after public model, identity, payload, validation, Query,
Diff, report, Impact policy, complete index, and incremental rebuild evidence
passes.

EDT Coverage may transition the XDTO/service first slice only after a tracked
provenance-backed reduced fixture proves descriptor/artifact parsing, metadata
enrichment, child emission, ownership, internal/external resolution policy,
requests, References, Triggers, provenance, diagnostics, statistics,
determinism, generic consumers, and both index lifecycles. Aggregate counts are
derived from executable registries rather than copied from planning estimates.

Existing top-level metadata capabilities remain compatible; their status alone
does not prove the new subordinate slice.

## Rejected alternatives

### Generic XML/property nodes

Rejected because source-local element names, ordinals, and dynamic property
maps would become an unstable second semantic model without property identity,
QName, inheritance, or occurrence contracts.

### Model all XDTO properties in Sprint 13

Rejected because the corpus proves repeated property names, nested inline
types, bounds, forms, restrictions, and imports whose identities and semantics
are not resolved by direct-type evidence.

### Use service name, route, handler, or ordinal as child identity

Rejected because UUIDs exist for service children and mutable content or
position would create rename/reorder instability.

### Create placeholder nodes for external XDTO namespaces

Rejected because no repository identity, lifecycle, provenance authority, or
cross-workspace contract exists for platform schemas.

### Emit Calls for declared service handlers

Rejected because descriptor dispatch is not a BSL call-site fact. References
plus Triggers preserve navigation and invocation intent without inventing code
execution.

### Infer Procedure from a service XML field name

Rejected because all 35 live HTTP and 119 live Web handler names resolve to
owned BSL Functions and zero resolve to Procedures. Serialized `handler` and
`procedureName` fields are source-format labels, not semantic declaration-kind
contracts.

### Accept both Procedure and Function for every service family

Rejected because the complete live corpus proves the narrower exact target
kind `Function` for all 154 handlers. Accepting both kinds would hide
incompatible declarations and broaden endpoint validation without source
evidence.

### Add a service-specific query or index

Rejected because canonical Graph Query and Semantic Index dimensions already
cover typed nodes, ownership, adjacency, and exact name/kind lookup.

## Deferred scope

- XDTO imports, properties, enum values, patterns, inline definitions, bases,
  restrictions, bounds, inheritance, ordering, schema dependency closure, and
  external platform type catalogs;
- HTTP route parsing/matching, URL parameters, verbs inferred from names,
  sessions, security, publication, transport, request/response schemas, and
  runtime invocation;
- Web descriptor/WSDL generation, SOAP, data-lock behavior, publication,
  runtime execution, and external type resolution;
- dynamic handler names or BSL body semantics beyond existing Procedure and
  Function nodes;
- Designer XML and cross-adapter identity equivalence, which remain Sprint 14;
- persistence, Runtime services, HTTP API, CLI, MCP, LSP, IDE, serialization,
  benchmarks, and performance claims.

## Consequences

Sprint 13 gains independently addressable XDTO types and service declarations,
precise ownership, request-ledger observability, internal schema navigation,
and declarative handler dispatch while preserving existing top-level metadata
and BSL behavior. The bounded first slice is testable from repository-owned
evidence and leaves the large unresolved schema/property surface explicit
rather than silently partial.
