# XDTO and Service Source Investigation

## Status

Accepted planning evidence for Sprint 13.

## Scope and method

This investigation records repository-owned EDT evidence for the bounded
Sprint 13 XDTO, HTTP Service, and Web Service slice. It does not define the
source-independent semantic contract; ADR-0035 owns that decision.

The inspected production corpus is the ignored project under
`OneAgent_EDTproject/src/`:

- `XDTOPackages/*/*.mdo` and `XDTOPackages/*/Package.xdto`;
- `HTTPServices/*/*.mdo` and `HTTPServices/*/Module.bsl`;
- `WebServices/*/*.mdo` and `WebServices/*/Module.bsl`.

The audit used XML namespace-local element and attribute names, direct-child
ordering only where the format makes hierarchy significant, case-insensitive
BSL declaration matching consistent with the existing BSL pipeline, and
deterministic sorted inventories. Counts are planning evidence from the live
repository at planning head `cf59854baebc6fe88add0de5a0e5b6858b755a19`,
not permanent runtime constants.

## Existing compatibility baseline

- `MetadataKind::HttpService`, `MetadataKind::WebService`, and
  `MetadataKind::XdtoPackage` already exist with stable machine codes.
- The EDT adapter discovers `HTTPServices`, `WebServices`, and `XDTOPackages`
  through the universal top-level descriptor reader and emits UUID-backed
  metadata nodes, optional synonym payload, provenance, and configuration
  ownership.
- `Module.bsl` in an HTTP or Web Service directory is already discovered as the
  metadata object's Common module. Existing BSL extraction emits its Procedure
  and Function nodes with stable owner-scoped identities.
- Generic Query, Diff, reports, Validation, complete Semantic Index, and
  incremental clean-rebuild behavior already apply to every emitted graph fact.
- HTTP endpoints, Web operations and parameters, XDTO schema types, service
  handler dispatch, and service-to-XDTO relations have no current first-class
  graph representation.
- ADR-0023 explicitly keeps endpoints, operations, modules, and XDTO schema
  internals separate from generic top-level metadata payload bags.

These are compatibility constraints. Sprint 13 must enrich the existing nodes
and modules rather than create a second top-level identity authority.

## XDTO package descriptors and schema artifacts

The corpus contains 20 XDTO Package object directories. Every directory has
exactly one `.mdo` descriptor and exactly one `Package.xdto` artifact.

All 20 descriptors have:

- root local name `XDTOPackage` in
  `http://g5.1c.ru/v8/dt/metadata/mdclass`;
- one UUID;
- one direct `name`;
- one direct `namespace`;
- one or two localized `synonym` entries.

All 20 schema artifacts have root local name `package` in
`http://v8.1c.ru/8.1/xdto` and one `targetNamespace`. Every artifact
`targetNamespace` exactly equals its descriptor `namespace`. All package UUIDs,
names, descriptor namespaces, and schema target namespaces are unique.

The direct schema-child inventory is:

| Direct child | Count | Identity evidence |
|---|---:|---|
| `import` | 17 | Namespace only; no repository-owned imported package identity is proven. |
| `valueType` | 3,421 | Required direct `name`; names are unique across both direct type families within one package. |
| `objectType` | 9,245 | Required direct `name`; names are unique across both direct type families within one package. |

The smallest collision-safe direct-type identity can therefore use the owning
package identity plus the exact decoded local name. Type family remains typed
semantic content and does not need an ordinal or source position.

The schema corpus also contains 5,493 `enumeration`, 61,435 `property`, 16
`pattern`, and 1,667 `typeDef` nested elements. Properties repeat names within
the same package and require their immediate declaring type, form, bounds,
QName, inline type, inheritance, ordering, and occurrence contracts. Imports,
base types, restrictions, enumerations, properties, inline type definitions,
and cross-package schema dependency resolution are not safe first-slice
entities merely because their XML is present.

### Representative XDTO evidence

- `XDTOPackages/CurrencyRates/CurrencyRates.mdo` proves descriptor namespace.
- `XDTOPackages/CurrencyRates/Package.xdto` proves one direct Object type and
  nested Attribute-form properties.
- `XDTOPackages/EnterpriseDataExchange_1_0_1_1/Package.xdto` proves the one
  repository-internal type referenced by a Web Service declaration.
- `XDTOPackages/ApplicationExtensionsManifest_1_0_0_1/Package.xdto` proves
  mixed direct Value/Object types and repeated nested property names.
- `XDTOPackages/EnterpriseData_1_17_3/Package.xdto` proves a large schema with
  328 direct Value types and 867 direct Object types.

## HTTP Service descriptors

The corpus contains two HTTP Services, `Site` and `wms_mobile`, with exactly one
descriptor and one `Module.bsl` each.

Every descriptor has:

- root local name `HTTPService` in the metadata namespace;
- one service UUID and name;
- one `rootURL`, `reuseSessions`, and `sessionMaxAge`;
- direct `urlTemplates` children, each with UUID, name, and `template`;
- direct `methods` children under a URL template, each with UUID, name, and
  `handler`, plus optional `httpMethod`.

Observed values and cardinalities are:

| Fact | Count / vocabulary |
|---|---|
| Services | 2 |
| URL templates | 35; every UUID and owner-local name is unique |
| Methods | 35; every UUID is unique; method names repeat under different URL templates |
| `rootURL` | one non-empty value per service |
| `reuseSessions` | `AutoUse` for both services |
| `sessionMaxAge` | `20` for both services |
| explicit `httpMethod` | `POST` in 11 methods; absent in 24 methods |
| handler | one non-empty handler name per method |

Method identity must use its declared UUID, not repeated method names, HTTP
verb, handler name, ordinal, or URL text. URL Template identity likewise uses
its declared UUID. `rootURL`, template text, explicit HTTP method value, and
handler binding are mutable content or relation evidence rather than identity.

All 35 handler names resolve case-insensitively to exactly one Function in the
owning HTTP Service module. Zero resolve to Procedure, and no missing or
ambiguous live binding was found.

## Web Service descriptors

The corpus contains eight Web Services with exactly one descriptor and one
`Module.bsl` each.

Every descriptor has:

- root local name `WebService` in the metadata namespace;
- one service UUID and name;
- one `namespace`, `descriptorFileName`, and `sessionMaxAge`;
- zero or one direct `xdtoPackages` declaration;
- direct `operations`, each with UUID, name, `xdtoReturningValueType`,
  `procedureName`, and `dataLockControlMode`;
- zero or more direct `parameters` per operation, each with UUID, name, and
  `xdtoValueType`, plus optional `nillable` and `transferDirection`.

Observed values and cardinalities are:

| Fact | Count / vocabulary |
|---|---|
| Services | 8 |
| Operations | 119; every UUID and owner-local name is unique |
| Parameters | 360; every UUID is unique; names repeat across operations |
| Operation return type declarations | 119 |
| Parameter type declarations | 360 |
| `dataLockControlMode` | `Managed` for all 119 operations |
| explicit `nillable` | `true` in 180 declarations; otherwise absent |
| explicit `transferDirection` | `Out` in 41, `InOut` in 49, otherwise absent |
| handler binding | one non-empty `procedureName` value per operation |

All 119 handler names resolve case-insensitively to exactly one Function in the
owning Web Service module. Zero resolve to Procedure, and no missing or
ambiguous live binding was found. The EDT XML field name `procedureName` does
not determine the BSL declaration kind.

Operation and Parameter identities must use their declared UUIDs. Return/value
type, nillability, transfer direction, and procedure binding are content or
relation evidence rather than identity.

## Web Service XDTO declarations and type references

Seven Web Services declare one `xdtoPackages` value:

- two `core:ReferenceValue` declarations use the exact grammar
  `XDTOPackage.<name>` and both resolve to the repository-owned
  `EnterpriseDataExchange_1_0_1_1` package;
- five `core:StringValue` declarations name
  `http://v8.1c.ru/8.1/data/core`, which is not a repository-owned package;
- one Web Service has no `xdtoPackages` declaration.

The 479 operation-return and parameter type declarations use one direct `name`
and one direct `nsUri`. They contain 16 distinct `(namespace, name)` pairs. Of
the 479 occurrences, 478 use external platform or XML Schema namespaces. One
uses repository package namespace
`http://v8.1c.ru/SSL/Exchange/EnterpriseDataExchange` and resolves exactly to
the direct XDTO Object type `PrepareDataOperationResult`.

An internal XDTO type reference is therefore resolvable only by exact package
namespace plus exact direct type name. External namespaces are valid preserved
declarations but must not create placeholder packages/types or resolved graph
edges. Missing, ambiguous, and incompatible internal candidates require typed
terminal outcomes.

## Determinism and negative oracle

The repository provides a reliable acceptance oracle:

- XML parsing can require exact root and namespace, required UUID/name fields,
  the XDTO descriptor/artifact namespace join, and the observed direct
  hierarchy;
- stable UUIDs cover services, URL templates, methods, operations, and
  parameters;
- package-scoped direct XDTO names are unique without ordinals;
- existing graph ownership, validation, Query, Diff, report, complete index,
  and incremental rebuild APIs can observe additions, removals, content
  changes, and ownership changes;
- existing BSL Module/Function nodes provide exact handler resolution targets;
- malformed XML, missing/extra/ambiguous artifacts, wrong roots/namespaces,
  missing/duplicate UUIDs or names, namespace mismatch, duplicate direct type
  names, invalid Boolean/direction/package reference values, missing/ambiguous
  handlers, external type namespaces, reordered filesystem/XML inputs, and
  repeated builds are executable generated negative or determinism cases.

The positive production oracle must use a tracked reduced fixture derived from
the live paths above with recorded source hashes, reduction treatment, and
reduced hashes. Generated fixtures remain appropriate for negative cases but
cannot replace provenance-backed positive evidence.

## Accepted planning boundary

The repository evidence is sufficient to plan and test this bounded slice:

- preserve HTTP root URL, Web Service namespace/package declarations, and XDTO
  Package namespace as closed typed metadata content;
- model direct XDTO Value/Object types, HTTP URL Templates/Methods, and Web
  Service Operations/Parameters as independently addressable owned nodes;
- resolve repository XDTO package and direct-type declarations without creating
  external placeholders;
- resolve HTTP Method and Web Service Operation handlers to existing owned
  Function nodes and project declarative dispatch;
- preserve exact UUID or collision-safe owner/name identity, provenance,
  diagnostics, statistics, requests, determinism, generic consumers, complete
  and incremental indexes, and Coverage evidence.

## Explicitly deferred source scope

- XDTO imports, properties, enumerations, patterns, inline type definitions,
  bases, restrictions, bounds, inheritance, ordering, and cross-package schema
  dependency closure;
- external namespace/type nodes and platform schema catalogs;
- HTTP route grammar, placeholder parameters, matching precedence, security,
  sessions, publication, transport, request/response schemas, and runtime calls;
- Web Service WSDL generation, descriptor files, SOAP transport, runtime
  invocation, data-lock behavior, and external type resolution;
- BSL body behavior beyond exact declared handler/procedure binding;
- Designer XML ingestion, partial workspaces, persistence, Runtime/API/CLI,
  MCP/LSP/IDE, performance claims, and serialization.
