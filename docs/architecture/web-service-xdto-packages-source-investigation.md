# Web Service XDTO Package Declarations Source Investigation

## Status

Accepted corrective architecture evidence after Sprint 13. Observed on
2026-08-20 at committed baseline
`2a073990114809563e11eb7a523423df68d4e4fa`.

## Scope and method

This investigation compares direct `xdtoPackages` declarations in both
repository-local ignored EDT corpora registered by
[Local EDT Source Corpora](edt-source-corpora.md):

- `OneAgent_EDTproject/src/WebServices` and its `XDTOPackages` siblings;
- `Retail_edt_project/Розница_базовая/src/WebServices` and its
  `XDTOPackages` siblings.

The audit parsed XML namespace-aware, inspected only direct Web Service
`xdtoPackages` children, decoded their `xsi:type` wrapper and direct `value`,
joined repository declarations by exact `XDTOPackage.<name>`, and compared
descriptor `namespace` with `Package.xdto` `targetNamespace`. Operation return
and Parameter value declarations were counted from their direct `nsUri`
fields. Inventories were sorted by path or typed value; source order was
retained only as evidence and was not treated as semantic order.

Counts are point-in-time corpus evidence, not runtime constants. The original
[XDTO and Service Source Investigation](xdto-service-source-investigation.md)
remains accurate for its explicitly named OneAgent planning corpus and Sprint
13 baseline. This document records the later contradictory Retail evidence
without rewriting that historical record.

## Implemented compatibility boundary

The source-independent `WebServiceMetadataPayload` already owns a
`Vec<WebServiceXdtoPackage>` and its constructor sorts and deduplicates the
typed declarations. The public payload therefore represents zero-or-more
repository or external declarations without using source position as identity.

The EDT parser now stores a canonical `Vec<EdtWebServiceXdtoPackage>`, exposes
`xdto_packages()`, parses every direct declaration independently, and sorts and
deduplicates the typed values. Metadata projection consumes the complete
collection, and package-request collection emits one request for every unique
repository declaration while leaving external namespaces payload-only.

Repository XDTO type resolution is independent of the declared package list.
Production builds a complete namespace-to-package-owner index from all parsed
repository XDTO Packages. A type declaration creates an owner-scoped
`XdtoType` request when its namespace occurs in that complete index. All
candidate owners for an equal namespace are retained, sorted, and deduplicated;
declaration or filesystem order cannot select one silently.

An `XdtoPackage` request is different: it is an unscoped exact-name request
from the Web Service metadata node to `Metadata(XdtoPackage)`. Its reachable
terminal outcomes are `Resolved`, `MissingTarget`, `AmbiguousTarget`, and
`IncompatibleTargetKind`. `InvalidOwnerReference` applies only to owner-scoped
`XdtoType` and `Callable` requests.

## Corpus-separated inventory

| Fact | OneAgent | Retail |
|---|---:|---:|
| Web Services | 8 | 18 |
| Direct declaration cardinality | 0: 1; 1: 7 | 0: 3; 1: 13; 2: 1; 4: 1 |
| `core:ReferenceValue` occurrences | 2 | 12 |
| `core:StringValue` occurrences | 5 | 7 |
| Services mixing repository and external declarations | 0 | 1 |
| Services with equivalent repeated declarations | 0 | 0 |
| Operation/Parameter type occurrences | 479 | 667 |
| Type occurrences in any repository package namespace | 1 | 24 |
| Type occurrences in a namespace declared by that service | 1 | 20 |
| Type occurrences in another repository package namespace | 0 | 4 |
| XDTO descriptor/artifact pairs | 20 | 411 |
| Unique package names | 20 | 411 |
| Unique package namespaces | 20 | 410 |
| Descriptor/artifact namespace mismatches | 0 | 0 |

No service in either corpus repeats an equivalent typed declaration within the
same descriptor. Repeated values across different services remain different
semantic requests because the source Web Service node participates in request
identity. Acceptance of within-service deduplication is therefore a canonical
compatibility decision aligned with the implemented metadata payload and
ADR-0024 aggregation, not a claim that the live corpora contain an equivalent
duplicate.

### OneAgent declarations

The eight-service corpus preserves the original Sprint 13 observation:

- two repository declaration occurrences, both
  `XDTOPackage.EnterpriseDataExchange_1_0_1_1`;
- five external declarations, all
  `http://v8.1c.ru/8.1/data/core`;
- one service with no declaration;
- no mixed or multi-declaration service.

The repository package namespace is
`http://v8.1c.ru/SSL/Exchange/EnterpriseDataExchange`; its descriptor and
artifact namespaces agree. One of 479 type occurrences uses that namespace.

### Retail declarations

The 18-service Retail corpus contains 19 declaration occurrences: 12
repository references and seven external namespace values. The repository
references name 11 distinct existing XDTO Packages:

| Exact package name | Occurrences | Descriptor/artifact namespace |
|---|---:|---|
| `CommerceML205a` | 1 | `urn:1C.ru:commerceml_205` |
| `CommerceML210` | 1 | `urn:1C.ru:commerceml_210` |
| `DMIL` | 1 | `http://www.1c.ru/dmil` |
| `EnterpriseDataExchange_1_0_1_1` | 2 | `http://v8.1c.ru/SSL/Exchange/EnterpriseDataExchange` |
| `EquipmentService` | 1 | `http://www.1c.ru/EquipmentService` |
| `EquipmentService_1_0_0_6` | 1 | `http://www.1c.ru/EquipmentService/1.0.0.6` |
| `EquipmentService_1_0_0_7` | 1 | `http://www.1c.ru/EquipmentService/1.0.0.7` |
| `EquipmentService_2_0_0_3` | 1 | `http://www.1c.ru/EquipmentService/2.0.0.3` |
| `MobileClientIntegration` | 1 | `http://www.1c.ru/SB/MobileExchange` |
| `АдминистрированиеОбменаДанными` | 1 | `http://www.1c.ru/SaaS/ExchangeAdministration/Common` |
| `АдминистрированиеОбменаДанными_2_4_5_1` | 1 | `http://www.1c.ru/SaaS/ExchangeAdministration/Common/2.4.5.1` |

All seven external occurrences use
`http://v8.1c.ru/8.1/data/core`. Every repository reference resolves by exact
name to an XDTO Package descriptor, and every selected descriptor namespace
equals its `Package.xdto` target namespace.

Retail has one duplicate repository namespace across the complete 411-package
index: `Envelope` and `SOAP_Envelope_1_1` both declare
`http://schemas.xmlsoap.org/soap/envelope/`. No Web Service type declaration in
either inspected corpus uses that namespace. The duplicate still confirms that
type resolution must retain all owners and preserve the existing ambiguous
terminal behavior rather than select the first package.

## Multi-declaration evidence

### EquipmentService

`Retail_edt_project/Розница_базовая/src/WebServices/EquipmentService/EquipmentService.mdo`
contains four direct repository declarations in this source order:

1. `XDTOPackage.EquipmentService_2_0_0_3`;
2. `XDTOPackage.EquipmentService`;
3. `XDTOPackage.EquipmentService_1_0_0_7`;
4. `XDTOPackage.EquipmentService_1_0_0_6`.

All four packages exist and have distinct matching descriptor/artifact
namespaces. The service contains 28 type occurrences; 11 use
`http://www.1c.ru/EquipmentService/1.0.0.6`, one of its declared repository
package namespaces. The source order is neither lexical nor semantic version
order and provides no safe priority or identity contract.

### MobileService

`Retail_edt_project/Розница_базовая/src/WebServices/MobileService/MobileService.mdo`
contains one repository declaration,
`XDTOPackage.MobileClientIntegration`, followed by the external namespace
`http://v8.1c.ru/8.1/data/core`. This proves that repository and external
variants may coexist in one valid declaration collection. The repository
package exists and its descriptor/artifact namespace is
`http://www.1c.ru/SB/MobileExchange`.

### Type resolution outside the declaration list

`Retail_edt_project/Розница_базовая/src/WebServices/SiteExchange2/SiteExchange2.mdo`
declares `XDTOPackage.CommerceML210`, but four of its type occurrences use
`urn:1C.ru:commerceml_205`, the namespace of the repository package
`CommerceML205a`. The package list therefore cannot be a scope, filter, or
disambiguation priority for XDTO type requests. Exact namespace/name lookup
must continue against the complete repository snapshot.

## Representative artifact hashes

Hashes are SHA-256 over the exact ignored source bytes at the observed
baseline. They are provenance inputs for a later minimal tracked reduction;
the ignored files themselves are not test dependencies.

| Corpus | Artifact | Bytes | SHA-256 |
|---|---|---:|---|
| OneAgent | `OneAgent_EDTproject/src/WebServices/EnterpriseDataExchange_1_0_1_1/EnterpriseDataExchange_1_0_1_1.mdo` | 18,856 | `6bb5c9b64aeb23816206652524ad44b5355fc0f86a144bfb114dbf88d40f5777` |
| OneAgent | `OneAgent_EDTproject/src/WebServices/InterfaceVersion/InterfaceVersion.mdo` | 1,778 | `3649666fc86aa0d73b65e8d5c4bc36e2f6eae07ec3f428da3663724041c9c221` |
| OneAgent | `OneAgent_EDTproject/src/XDTOPackages/EnterpriseDataExchange_1_0_1_1/EnterpriseDataExchange_1_0_1_1.mdo` | 496 | `dbb883f1be02bd75f2812e88ccd674ce53f2ef74f298472e38011741c5bc86dc` |
| OneAgent | `OneAgent_EDTproject/src/XDTOPackages/EnterpriseDataExchange_1_0_1_1/Package.xdto` | 559 | `70d5efefad5fad76b65c5ddc5c126b970d1808e1651e4a8ca62daea79845921a` |
| Retail | `Retail_edt_project/Розница_базовая/src/WebServices/EquipmentService/EquipmentService.mdo` | 11,749 | `0e426a416aed002c068140ccfde6f227a0a4a7ddf45bd746b9efe14f3127c59a` |
| Retail | `Retail_edt_project/Розница_базовая/src/WebServices/MobileService/MobileService.mdo` | 32,360 | `6b94177cae8c35d006534fd396004d4e606e8c1f53fca4802873c2d1d650dc7b` |
| Retail | `Retail_edt_project/Розница_базовая/src/WebServices/SiteExchange2/SiteExchange2.mdo` | 8,487 | `e0f74bceca89c3ab09230ed4bdf04628c35c4c8b5b2e62144bf4f7f10f781909` |
| Retail | `Retail_edt_project/Розница_базовая/src/XDTOPackages/EquipmentService_2_0_0_3/EquipmentService_2_0_0_3.mdo` | 383 | `5f674d844074d01f24d9d51965304310ba36332737547bbd0d59a154cf8ef50a` |
| Retail | `Retail_edt_project/Розница_базовая/src/XDTOPackages/EquipmentService_2_0_0_3/Package.xdto` | 42,663 | `3861458625890d8b08396a94571ac7e720016671878839bb079244deaa787bcd` |
| Retail | `Retail_edt_project/Розница_базовая/src/XDTOPackages/MobileClientIntegration/MobileClientIntegration.mdo` | 379 | `f08a3d5db897e59502f3d016fa41e2f071d1f2cd1a0b3fb48c8f6c82686e9857` |
| Retail | `Retail_edt_project/Розница_базовая/src/XDTOPackages/MobileClientIntegration/Package.xdto` | 22,756 | `cc9d3b0f6d3b06e589056531ddfa685396839806d5332a938048a1d9bb693a8e` |
| Retail | `Retail_edt_project/Розница_базовая/src/XDTOPackages/CommerceML205a/CommerceML205a.mdo` | 350 | `584f882a9101a450ca7988390f1de07d01d1f5f2a94f8e73965ef56f94bc9105` |
| Retail | `Retail_edt_project/Розница_базовая/src/XDTOPackages/CommerceML205a/Package.xdto` | 39,736 | `e7120d4e877b4e61bc7dd706db7a24379f165b3208f4c31f6b09dacc8d2b70ff` |
| Retail | `Retail_edt_project/Розница_базовая/src/XDTOPackages/CommerceML210/CommerceML210.mdo` | 349 | `0e491bba44dd9e50c34f785c95c557deff7c84d1d5b1a7af43af21e45bf747b6` |
| Retail | `Retail_edt_project/Розница_базовая/src/XDTOPackages/CommerceML210/Package.xdto` | 53,160 | `6c89dc9d415da175e6b30815610779e6e6df8c8b455d7fc20e6dad849d643da7` |

## Accepted corrective boundary

The evidence is sufficient to accept these source and implementation
contracts:

- direct Web Service `xdtoPackages` cardinality is zero-or-more;
- every direct child is parsed independently as the existing repository or
  external typed variant;
- the parser returns a typed collection sorted by declaration variant and
  exact value, consistent with the existing source-independent ordering, and
  deduplicates equivalent declarations within one service;
- every unique repository declaration creates one unscoped exact-name
  `XdtoPackage` request, while every external declaration remains payload-only;
- package request failures are independent of valid siblings and can terminate
  as missing, ambiguous, or incompatible, but never invalid-owner;
- structural failure in any declaration remains fatal for the complete service
  descriptor and therefore for the complete build;
- XDTO type resolution remains exact by `(namespace URI, local type name)` over
  the complete repository namespace index, independent of the service package
  declaration list;
- multiple resolved package requests may create multiple precise
  `Metadata(WebService) --References--> Metadata(XdtoPackage)` edges without a
  graph-domain or endpoint-matrix change;
- source reordering and repeated builds must preserve equal payloads, requests,
  candidates, provenance, diagnostics, statistics, reports, validation, and
  index results.

The bounded implementation is complete without a Coverage or registry-count
change. Generated tests and the tracked
`adapters/edt/tests/fixtures/multiple_xdto_packages_project/` reduction prove
canonical parsing, independent requests, terminal outcomes, exact References,
global namespace/type resolution, index visibility, source reordering, and
repeated builds. The optional whole-Retail probe passes both
multi-declaration services and reaches the later unrelated Role Rights parser
boundary.
