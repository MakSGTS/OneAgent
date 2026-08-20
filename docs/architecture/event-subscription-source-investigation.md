# Event Subscription Source Investigation

## Purpose

This document records repository-owned EDT evidence for Sprint 11 Event
Subscriptions. It is a planning authority for source shape and testability; it
does not itself change parser, graph, Coverage, or production behavior.

## Investigated baseline

The investigation was performed against committed Sprint 10 head
`62d22c53d0e0c7f077d477398fe899c311dd5cc3`. The real source corpus is under
`OneAgent_EDTproject/src/EventSubscriptions/`. The repository root `.gitignore`
excludes `OneAgent_EDTproject/`, so this corpus is planning evidence rather than
a committed fixture. A Sprint 11 evidence task must derive a small tracked
fixture with explicit provenance before Coverage changes.

The production EDT builder currently ignores `EventSubscriptions` because
`supported_metadata_directories()` has no mapping for that directory.
`oneagent-metadata` has no `MetadataKind::EventSubscription`, the graph has no
event-subscription execution edge, and the generic EDT descriptor reader does
not preserve source selectors, event, or handler.

## Corpus inventory

The corpus contains 99 `.mdo` descriptors in 99 object directories. Every
descriptor has exactly one root `mdclass:EventSubscription` with a unique UUID,
one direct `name`, one `source` container, one direct `event`, and one direct
`handler`. No duplicate UUID or name was observed.

Observed direct-field counts are:

| Field | Evidence |
|---|---:|
| Event Subscription descriptors | 99 |
| Source `types` occurrences | 314 |
| Unique source selector values | 210 |
| Bare family selectors | 91 occurrences / 15 unique |
| Qualified family-and-name selectors | 223 occurrences / 195 unique |
| Unique event values | 18 |
| Unique handler paths | 93 |
| Optional synonym entries | 171 |
| Optional comment entries | 4 |

Source-list sizes are not fixed. Eighty-one descriptors contain one selector;
the remaining descriptors contain 2, 3, 4, 5, 6, 7, 8, 30, 41, or 94
selectors. Source ordering is therefore input order, not semantic identity.

## Serialized source contract

Representative source shape:

```xml
<mdclass:EventSubscription
    xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass"
    uuid="7fa2863c-662c-4893-b42e-01a883bffc54">
  <name>Catalogs_BeforeWrite</name>
  <synonym>
    <key>en</key>
    <value>Catalogs before write</value>
  </synonym>
  <source>
    <types>CatalogObject</types>
  </source>
  <event>BeforeWrite</event>
  <handler>CommonModule.ObjectEvents.Catalogs_BeforeWrite</handler>
</mdclass:EventSubscription>
```

The `types` vocabulary has exactly one or two dot-separated components in the
live corpus:

```text
Family
Family.MetadataName
```

No empty, three-component, or deeper value was observed. The parser still
needs deterministic typed failures for those malformed forms because they are
representable source corruption cases.

The handler vocabulary has exactly three components in all 99 descriptors:

```text
CommonModule.ModuleName.ProcedureName
```

All 93 unique module directories and procedure declarations exist in the real
corpus. Eighty-nine handler procedures are exported and four are not exported.
No handler resolves to a Function. Event Subscription handler resolution must
therefore use declared Common Module procedure ownership and must not reuse the
cross-module call rule that rejects non-exported targets.

## Event vocabulary

The corpus contains these exact event values:

| Event | Count |
|---|---:|
| `BeforeWrite` | 36 |
| `OnWrite` | 22 |
| `BeforeDelete` | 12 |
| `Posting` | 5 |
| `FillCheckProcessing` | 5 |
| `Filling` | 4 |
| `UndoPosting` | 3 |
| `OnCopy` | 2 |
| `PresentationGetProcessing` | 1 |
| `PresentationFieldsGetProcessing` | 1 |
| `OnSetNewNumber` | 1 |
| `OnSetNewCode` | 1 |
| `OnSendNodeDataToSlave` | 1 |
| `OnSendDataToSlave` | 1 |
| `OnSendDataToMaster` | 1 |
| `OnReceiveDataFromSlave` | 1 |
| `OnReceiveDataFromMaster` | 1 |
| `AfterWriteDataHistoryVersionsProcessing` | 1 |

The repository does not contain an accepted closed platform event enum. The
first slice can safely preserve a non-empty decoded event name as typed payload
without claiming this observed corpus is the complete platform vocabulary.

## Source selector vocabulary

The observed selector prefixes and occurrence counts are:

| Prefix | Occurrences | Unique values | Current semantic target |
|---|---:|---:|---|
| `CatalogObject` | 44 | 25 | `MetadataKind::Catalog` |
| `CatalogManager` | 1 | 1 | `MetadataKind::Catalog` |
| `DocumentObject` | 52 | 33 | `MetadataKind::Document` |
| `DocumentManager` | 1 | 1 | `MetadataKind::Document` |
| `InformationRegisterRecordSet` | 44 | 43 | `MetadataKind::InformationRegister` |
| `AccumulationRegisterRecordSet` | 4 | 2 | `MetadataKind::AccumulationRegister` |
| `AccountingRegisterRecordSet` | 3 | 2 | `MetadataKind::AccountingRegister` |
| `CalculationRegisterRecordSet` | 1 | 1 | `MetadataKind::CalculationRegister` |
| `BusinessProcessObject` | 5 | 1 | `MetadataKind::BusinessProcess` |
| `BusinessProcessManager` | 2 | 1 | `MetadataKind::BusinessProcess` |
| `TaskObject` | 5 | 1 | `MetadataKind::Task` |
| `ConstantValueManager` | 77 | 76 | unsupported metadata family |
| `DefinedType` | 35 | 18 | unsupported union/type family |
| `ExchangePlanObject` | 18 | 1 | unsupported metadata family |
| `ChartOfAccountsObject` | 7 | 1 | unsupported metadata family |
| `ChartOfCalculationTypesObject` | 6 | 1 | unsupported metadata family |
| `ChartOfCharacteristicTypesObject` | 9 | 2 | unsupported metadata family |

The existing metadata model supports 162 selector occurrences and 111 unique
values across the first eleven prefixes. This includes 50 bare-family
occurrences and 112 qualified occurrences. Every one of the 101 unique
qualified supported targets has a matching real metadata object directory.

Bare selectors denote a family set, not an ambiguous single-name request. They
must resolve deterministically to all graph metadata nodes of the mapped kind.
Qualified selectors resolve by exact canonical metadata name and mapped kind.
Equivalent object/manager prefixes may select the same metadata object and
must aggregate into one graph edge with all deterministic provenance records.

Unsupported families must remain typed observations and diagnostics. They do
not authorize new metadata kinds, placeholder nodes, `Unknown` targets, or
silent omission. In particular, `DefinedType` is not equivalent to one
metadata object and cannot be flattened into the existing nine-kind metadata
type-reference contract.

## Handler resolution oracle

Every live handler path begins with `CommonModule`, names an existing
`MetadataKind::CommonModule` object, and names exactly one Procedure owned by
that object's `Module` node. The existing graph ownership chain provides an
executable resolution oracle:

```text
Metadata(CommonModule)
    --Contains--> Module
    --Contains--> Procedure
```

Resolution is exact and source-declared. Missing module, missing procedure,
wrong target kind, duplicate candidates, and malformed path must emit no
resolved handler edge and must produce deterministic typed diagnostics. A
non-exported Procedure remains valid because the real corpus proves that Event
Subscription handlers are not governed by the exported cross-module call
restriction.

## Testability gate

The repository evidence is sufficient to plan and test the first slice:

- UUID, name, synonym, source selectors, event, handler, nesting, and value
  vocabulary are directly observable;
- positive exact, positive family, duplicate-target, unsupported-family,
  missing-target, malformed, reordered, and repeated-build cases have an
  observable graph or diagnostic result;
- all handler targets and all qualified supported source targets have live
  existence oracles;
- the universal metadata reader, BSL declaration pipeline, semantic resolution
  index, graph validation, Query, Diff, Impact, reports, complete index,
  incremental index, and Coverage registries are discoverable consumers;
- the canonical full workspace validation matrix is known.

A tracked reduced fixture must include at least one exact selector, one bare
family selector, two prefixes selecting the same metadata target, one
unsupported selector, one exported handler, and one non-exported handler. Its
README must identify live paths and source/derived hashes so the ignored corpus
is not treated as hidden test input.

## Architecture questions resolved by ADR-0033

The accepted architecture must define:

- Event Subscription metadata identity and typed event payload;
- configuration ownership;
- exact and family source selection;
- handler resolution through Common Module ownership;
- `References` and execution-edge directions and endpoint matrices;
- provenance aggregation, diagnostics, statistics, validation, Query, Diff,
  Impact, index, and Coverage behavior;
- the boundary between adapter-private multi-target selector observations and
  the public ADR-0024 single-target request lifecycle.

## Deferred evidence

The corpus does not justify first-slice semantics for Constants, Defined Types,
Exchange Plans, Charts of Accounts, Charts of Calculation Types, or Charts of
Characteristic Types because those metadata kinds are not modeled or produced
by OneAgent. It also does not justify extensions, cross-project targets,
Designer XML, platform-wide event enumeration, event ordering, handler
signatures, runtime dispatch simulation, or effective execution frequency.
