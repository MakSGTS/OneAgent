# ADR-0036: Designer XML Source Adapter

## Status

Accepted

## Context

Sprint 14 introduces the first source adapter for hierarchical 1C:Enterprise
Designer XML dumps. The graph and metadata domains are source-format
independent, while the existing production source path is specific to EDT.

The registered paired corpus and
[source investigation](../architecture/designer-xml-source-investigation.md)
prove real Designer markers, configuration and top-level metadata shapes,
generic module layouts, canonical UUID/name agreement, and normalized BSL
agreement. They also prove different XML vocabularies, paths, encodings, and a
known EDT-to-Designer form-event loss boundary. The evidence does not justify
field-for-field or whole-graph equality.

## Decision

Add a dedicated `oneagent-designer-xml` source adapter for one bounded first
slice. It owns Designer-specific detection, artifact assembly, XML parsing,
module source loading, build orchestration, diagnostics, provenance, and
adapter-specific Coverage evidence. It maps into existing source-independent
workspace, metadata, BSL, and graph APIs and does not depend on `oneagent-edt`.

### Detection and project boundary

A Designer project root requires both regular, non-symlink root files:

- `ConfigDumpInfo.xml` with root namespace
  `http://v8.1c.ru/8.3/xcf/dumpinfo`, local root `ConfigDumpInfo`, exact
  `format="Hierarchical"`, and a supported format version;
- `Configuration.xml` with root namespace
  `http://v8.1c.ru/8.3/MDClasses`, local root `MetaDataObject`, and one direct
  `Configuration` child.

The first slice supports exact XML format version `2.20`. A different non-empty
version is `UnsupportedVersion`, not a best-effort parse. The two roots must
declare the same supported version. Missing or malformed marker content does not
form a Designer root.

A directory that satisfies both the accepted EDT marker set and the Designer
marker set is a typed `ConflictingFormatMarkers` discovery error. Detection does
not select by rule order. An accepted Designer root is a traversal boundary.
Directory entries and results are canonical path ordered before further work.

Discovery does not follow symlink directories. Marker and accepted artifact
files must be regular non-symlink files. An accepted artifact that resolves
outside the project boundary, overlaps another canonical artifact identity, or
cannot be inspected/read is a typed error. Unknown directories are not
traversed as nested projects after an accepted root is established.

### Complete and partial input

The adapter exposes an explicit closed build scope:

```rust
pub enum DesignerXmlBuildScope {
    Complete,
    Partial,
}
```

The ordinary discovered-root builder uses `Complete`. Tests, controlled tools,
and future callers may request `Partial` explicitly. Scope is caller input and
is never inferred from missing artifacts.

Both scopes require valid root markers and configuration content. In `Complete`,
any malformed accepted descriptor or module is fatal for the configuration and
no successful partial graph is returned. A supported family directory may be
absent or empty because a complete 1C configuration may contain zero objects of
that family. Completeness does not require at least one object per family.

In `Partial`, the caller deliberately supplies a subset of otherwise valid
artifacts. Absence outside that supplied subset produces neither a missing
diagnostic nor a placeholder fact. Every supplied accepted artifact must still
be structurally valid; `Partial` does not downgrade malformed, duplicate,
ambiguous, conflicting, or unreadable supplied input.

`ConfigDumpInfo.xml` is validated as a project marker and provenance source in
the first slice. Its 15,649 nested `Metadata` records are not treated as the
semantic entity inventory or as a complete-artifact oracle. Deeper manifest
consistency belongs to a later slice.

### Configuration assembly and mapping

The configuration loader accepts exactly:

```text
MetaDataObject(version="2.20")
  Configuration(uuid)
    Properties
      Name
      Synonym / v8:item / v8:lang
      Synonym / v8:item / v8:content
```

UUID and a non-empty exact name are required. Identity is the declared UUID.
The accepted payload is the existing `MetadataPayload` common synonym only.
For compatibility with the current EDT first slice, the adapter preserves the
first direct serialized synonym item content when present and non-empty. The
item's language and order are source observations, not identity. Reordering
unrelated elements is equivalent; reordering distinct localized synonym items
may change the compatibility payload and is not treated as formatting-only.

The loader returns the existing source-independent `Configuration` with
`WorkspaceFormat::DesignerXml`. Paths, XML wrapper/namespace, version, hashes,
and adapter name do not participate in configuration identity or payload.

### Top-level metadata assembly and mapping

The first slice accepts the 20 source-independent top-level families for which
the paired corpus contains at least one direct Designer descriptor:

| Directory | Direct child | `MetadataKind` |
|---|---|---|
| `Catalogs` | `Catalog` | `Catalog` |
| `Documents` | `Document` | `Document` |
| `Enums` | `Enum` | `Enumeration` |
| `CommonModules` | `CommonModule` | `CommonModule` |
| `Reports` | `Report` | `Report` |
| `DataProcessors` | `DataProcessor` | `DataProcessor` |
| `InformationRegisters` | `InformationRegister` | `InformationRegister` |
| `AccumulationRegisters` | `AccumulationRegister` | `AccumulationRegister` |
| `AccountingRegisters` | `AccountingRegister` | `AccountingRegister` |
| `BusinessProcesses` | `BusinessProcess` | `BusinessProcess` |
| `Tasks` | `Task` | `Task` |
| `Roles` | `Role` | `Role` |
| `CommonCommands` | `CommonCommand` | `Command` |
| `CommonForms` | `CommonForm` | `CommonForm` |
| `CommonTemplates` | `CommonTemplate` | `Template` |
| `HTTPServices` | `HTTPService` | `HttpService` |
| `WebServices` | `WebService` | `WebService` |
| `XDTOPackages` | `XDTOPackage` | `XdtoPackage` |
| `Subsystems` | `Subsystem` | `Subsystem` |
| `EventSubscriptions` | `EventSubscription` | `EventSubscription` |

The current paired configuration has zero Calculation Registers and no
`CalculationRegisters/` Designer directory. `MetadataKind::CalculationRegister`
exists and EDT supports the family, but repository evidence does not prove the
Designer direct root/path shape. Designer Calculation Registers are therefore
deferred instead of inferred from EDT or plural naming convention.

One accepted descriptor is `<Directory>/<Name>.xml`. Its
`{MDClasses}MetaDataObject` has exactly one compatible direct kind child with a
required UUID and one direct `Properties` container containing required exact
`Name` and optional direct common `Synonym` items. The filename stem, declared
name, family, and root kind must agree exactly. Accepted descriptors are keyed
by `(MetadataKind, exact name)` and canonicalized by kind, name, UUID, then path;
duplicates or conflicting identities are fatal rather than first-match wins.

The canonical graph node reuses the declared UUID, existing
`NodeKind::Metadata(kind)`, exact name, and existing common synonym payload. It
is owned directly by the configuration through `Contains`. This first slice
does not parse or emit kind-specific payloads, metadata members, specialized
flat Role/Subsystem nodes, or any nested semantic content. For Document,
HTTP/Web Service, XDTO Package, and Event Subscription, the source-independent
payload therefore remains the accepted common part only; no empty kind-specific
payload is invented.

Unknown top-level directories and unsupported nested artifacts create no
`MetadataKind::Unknown`, `NodeKind::Unknown`, placeholder, or partial semantic
fact. They remain outside the accepted slice and may be counted only in an
adapter-specific ignored/deferred observation report.

### Module assembly and BSL declarations

The first slice accepts three optional module roles for an already accepted
top-level metadata owner:

| Role | Designer artifact | Stable ID suffix | Module name |
|---|---|---|---|
| Object | `<Directory>/<Name>/Ext/ObjectModule.bsl` | `object_module` | `ObjectModule` |
| Manager | `<Directory>/<Name>/Ext/ManagerModule.bsl` | `manager_module` | `ManagerModule` |
| Common | `CommonModules/<Name>/Ext/Module.bsl` | `common_module` | owner name |

Object and Manager roles apply only where the accepted source contains the
exact artifact. Common applies only to `MetadataKind::CommonModule`. Each role
is optional; absence emits no module. A duplicate role, orphan directory,
owner/name mismatch, wrong owner kind, symlink/outside-root artifact, or
unreadable/invalid UTF-8 source is fatal for supplied input.

Module identity is the existing `<owner-uuid>:<role-suffix>`. Modules use the
existing `NodeKind::Module`, exact compatible name, and one owner-to-module
`Contains`. Source text is decoded as UTF-8, one leading BOM is removed, and
CRLF or bare CR is normalized to LF before the public `oneagent-bsl` declaration
extractor runs. Raw bytes and path remain provenance evidence. Normalized text
and its hash are content/conformance inputs, not stable identity.

The first slice emits existing Procedure and Function nodes and the existing
module ownership/declaration facts produced by the BSL declaration contract. It
does not emit Calls, Query, Reads, Writes, References, DependsOn, or other
resolution/derived relations. Those require source-family conformance beyond
module declaration equality.

### Failure scope

The adapter uses deterministic typed errors for:

- missing, malformed, unsupported, or conflicting root markers;
- version or root-namespace incompatibility;
- unreadable directories/files and symlink/outside-root artifacts;
- missing/empty/duplicate configuration UUID, name, Properties, or incompatible
  direct root structure;
- missing, duplicate, mismatched, malformed, incompatible, or ambiguous supplied
  metadata descriptors;
- duplicate/orphan/wrong-owner/unreadable supplied modules;
- BSL declaration parse failures.

All structural failures above are configuration-fatal in both scopes. The build
returns no successful graph with a silently skipped accepted sibling. Unknown
and explicitly deferred artifact families are not failures and contribute no
facts. The adapter may report their deterministic counts separately.

### Provenance

Every emitted fact carries exact adapter-local provenance with a stable
`oneagent.designer-xml.*` producer stage and project-relative source path. Node,
ownership, module, and declaration provenance distinguish raw source artifact,
semantic owner, role, and normalized parser input where applicable.

Provenance does not participate in canonical node or edge identity and is not a
cross-adapter equality dimension. Within one adapter it is ordered,
deduplicated, deterministic, visible through reports and diffs, and retained for
validation and future invalidation.

The public production result is adapter-specific orchestration, not a new
semantic authority. It owns the canonical `SemanticGraph`, ordered typed
Designer diagnostics/deferred observations, declared `DesignerXmlBuildScope`,
and the dedicated adapter Coverage report. It exposes graph facts through the
existing graph APIs and does not expose parser-local XML structures through
metadata or graph domain types.

### Cross-adapter conformance oracle

Conformance is tested through the public production builders over a tracked
provenance-backed paired EDT/Designer fixture. A complete fixture claimed as a
Designer dump must be produced by official 1C tooling. Parser-local partial
fixtures may exact-copy registered artifacts and mutations but must declare
`Partial` and record every source/reduced hash.

The canonical projection compares:

- configuration ID, kind, exact name, and accepted common payload;
- accepted top-level metadata ID, kind, exact name, and accepted common payload;
- configuration-to-metadata `Contains` identity;
- accepted module ID, kind, exact name, and metadata ownership;
- Procedure and Function ID, name, export status, owner, and declaration facts;
- terminal build success/failure category for the declared test scope;
- Query, Diff, report, Validation, complete-index, and incremental-index results
  over those canonical facts.

Both sides assert non-zero configuration, metadata, module, Procedure/Function,
and ownership/declaration counts. A controlled UUID, name, synonym, module body,
or declaration change must create the exact expected inequality.

Excluded equality dimensions are source path, producer ID, raw provenance,
adapter-specific diagnostic wording/code, XML wrapper/namespace/tag spelling,
format metadata, serialization order outside content-bearing synonym items, BOM,
line endings, raw bytes/hashes, and every explicitly deferred artifact.

### Coverage completion

Graph-domain Coverage does not change because the first slice reuses existing
node and edge kinds. EDT Coverage remains EDT-specific and must not be relabeled
as Designer support.

The Designer adapter may expose a dedicated static registry/report covering:

- project discovery and configuration loading;
- each accepted top-level metadata family;
- Object, Manager, and Common module roles;
- Procedure/Function declaration contribution;
- configuration/metadata/module/symbol ownership and provenance;
- explicit partial scope, fatal structural outcomes, determinism, production
  entry point, and paired conformance.

A capability becomes `Supported` only after positive, applicable negative,
reordered, repeated, public consumer, production fixture, and paired conformance
evidence passes. Architecture documentation alone changes no status.

## Rejected alternatives

### Convert Designer XML to an EDT directory tree

Rejected. It would create writable intermediate source, make EDT layout a
semantic authority, obscure original provenance, and require runtime 1C tooling
or speculative conversion rules.

### Depend on `oneagent-edt` or reuse EDT-local public types

Rejected. Adapter-local descriptors and orchestration are not source-independent
contracts. Both adapters must meet shared metadata, BSL, graph, and workspace
interfaces independently.

### Treat `ConfigDumpInfo.xml` as the semantic entity model

Rejected for the first slice. Its records include modules and members, and
repository evidence does not prove a complete semantic mapping for every entry.

### Accept a lone `Configuration.xml` or select a conflicting format by order

Rejected. Both-marker validation and typed conflict behavior prevent false
positive roots and nondeterministic adapter selection.

### Claim whole-graph or field-for-field equivalence

Rejected. Serialization differs, later EDT semantics have no Designer parser,
and the corpus has a documented bridge loss boundary.

### Emit unknown or placeholder facts for unsupported artifacts

Rejected. The graph already uses typed accepted concepts; absence or unsupported
source does not justify synthetic semantic identity.

## Implementation prerequisites

1. Add the Designer adapter crate and accepted detector/configuration boundary.
2. Implement independently testable metadata assembly/parser.
3. Implement independently testable module assembly/normalization/parser.
4. Orchestrate existing graph and BSL APIs without graph-model expansion.
5. Add official-tool or explicitly partial paired fixture provenance.
6. Prove the canonical conformance projection and controlled change.
7. Add truthful Designer-specific Coverage/current-state evidence.

Every implementation task must run focused non-zero tests and the full workspace
validation gate because parser, public workspace behavior, graph emission, or
Cargo membership changes.

## Deferred scope

- flat/monolithic dumps, other XML format versions, extensions, parent
  configurations, binary artifacts, and deep manifest consistency;
- multilingual payload policy beyond first-item compatibility;
- Attributes, Tabular Sections, Standard Attributes, Dimensions, Resources,
  Measures, Forms, Commands, Templates, and other metadata members;
- specialized Role/AccessRight, Subsystem, Event Subscription, report DCS,
  XDTO, HTTP, and Web Service semantics;
- Form, Command, configuration, RecordSet, and ValueManager module roles;
- Calls, Query/Reads/Writes, References, Grants, Includes, Extends, Triggers,
  DependsOn, public request lifecycles, and resolution diagnostics;
- persistence, Runtime/API/CLI, MCP/LSP/IDE, packaging, and performance.

The documented four invalid form-event bindings remain outside any conformance
claim until new source evidence and architecture explicitly accept them.

## Consequences

- Sprint 14 can implement a non-empty production Designer adapter without a new
  graph model or a second semantic authority.
- Canonical identity and accepted payload/ownership/declaration facts can be
  compared across adapters while provenance stays exact and source-specific.
- The first slice is intentionally narrower than the current EDT builder; later
  semantic families require their own evidence-backed Designer mappings.
- Strict structural failures favor truthful complete knowledge over silently
  incomplete graphs.
- Exact version support and first-item synonym compatibility are bounded
  compatibility decisions that later ADRs may expand without changing identity.
