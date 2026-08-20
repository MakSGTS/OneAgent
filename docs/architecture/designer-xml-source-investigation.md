# Designer XML Source Adapter Investigation

## Status and scope

Completed repository investigation for Sprint 14 at committed planning baseline
`73751ebc5390b1705fa53ad97af5331bde0ceb06` on 2026-08-21.

This document records source evidence and decision inputs. It does not accept a
Designer XML adapter architecture, define a public compatibility promise,
implement production behavior, or mark a capability supported. ADR-0036 owns
the architecture decision that follows this investigation.

## Evidence sources

- registered ignored Designer corpus: `OneAgent_DesignerXML/`;
- paired ignored EDT source: `OneAgent_EDTproject/src/`;
- [Designer XML corpus registration](designer-xml-source-corpus.md);
- [Semantic Model 2.0](semantic-model-2.md);
- [ADR-0004](../adr/0004-filesystem-workspace-discovery.md),
  [ADR-0005](../adr/0005-edt-configuration-loading.md),
  [ADR-0007](../adr/0007-edt-to-semantic-graph.md), and
  [ADR-0008](../adr/0008-edt-metadata-object-reader.md);
- `crates/workspace/src/lib.rs`;
- `adapters/filesystem/src/lib.rs`;
- `adapters/edt/src/lib.rs`, `metadata_object.rs`, `module_reader.rs`, and
  `bsl_graph.rs`;
- current workspace, metadata, BSL, graph, Coverage, fixture, and integration
  test consumers.

The complete ignored corpora are local research inputs, not runtime or CI
prerequisites. All tracked tests must use provenance-backed reductions.

## Confirmed corpus identity and inventory

The registered corpus is a Designer 8.3.27.2214 hierarchical dump with XML
format version `2.20`. Its configuration identity is paired with the EDT source:

| Dimension | Designer XML | EDT | Result |
|---|---|---|---|
| Configuration UUID | `408a41e7-907a-4fb3-8999-83d1e8b6e093` | same | equal |
| Name | `DNSWorldEdition` | same | equal |
| Version | `1.22.1.1` | same | equal |
| Designer format | `Hierarchical` | not applicable | source-specific |
| XML vocabulary | `http://v8.1c.ru/8.3/MDClasses` | `http://g5.1c.ru/v8/dt/metadata/mdclass` | deliberately different |

Recomputed live inventory:

| Dimension | Count |
|---|---:|
| Files | 10,964 |
| XML files | 7,585 |
| BSL modules | 2,717 |
| Other files | 662 |
| Top-level directories | 40 |
| File-content bytes | 237,458,744 |
| `<Metadata ...>` entries in `ConfigDumpInfo.xml` | 15,649 |

The manifest count is not a supported-object count: it includes metadata
members and module records as well as top-level objects. An implementation must
not infer semantic entities by treating every manifest entry as a top-level
descriptor.

## Project detection evidence

The real root has both of these regular files:

| Path | Confirmed root vocabulary |
|---|---|
| `ConfigDumpInfo.xml` | `{http://v8.1c.ru/8.3/xcf/dumpinfo}ConfigDumpInfo`, `format="Hierarchical"`, `version="2.20"` |
| `Configuration.xml` | `{http://v8.1c.ru/8.3/MDClasses}MetaDataObject`, direct `{...}Configuration`, `version="2.20"` |

`ConfigDumpInfo.xml` and `Configuration.xml` both begin with a UTF-8 BOM in the
registered corpus. BOM presence is therefore a decoder concern, not a detector
marker or identity component. The root has neither `.project` nor
`src/Configuration/Configuration.mdo`, so the live paired formats do not
conflict.

ADR-0004 already reserves `WorkspaceFormat::DesignerXml` and explicitly allows
a separate Designer detector. The current filesystem detector recognizes only
the EDT pair `.project` plus `src/Configuration/Configuration.mdo`, sorts
accepted roots through a `BTreeSet`, and stops recursion at an accepted EDT
root. There is no current Designer detector, marker validation, format conflict
policy, or Designer project boundary.

### Detector decisions still required

The repository supports the following decision candidates but has not accepted
one yet:

- require both root files, rather than accept a lone `Configuration.xml`;
- validate the manifest root namespace, `format="Hierarchical"`, and the root
  configuration wrapper before accepting the boundary;
- reject a directory that simultaneously satisfies EDT and Designer marker
  sets instead of selecting the first format;
- keep the accepted root as a traversal boundary and preserve canonical path
  ordering;
- reject or explicitly constrain symlink escape and overlapping roots rather
  than silently traverse external content.

Flat/monolithic XML dumps, extension projects, other format versions, and
directories containing only one marker have no accepted source contract.

## Configuration artifact contract evidence

`Configuration.xml` has this direct shape:

```text
MetaDataObject(version="2.20")
  Configuration(uuid="408a41e7-907a-4fb3-8999-83d1e8b6e093")
    Properties
      Name = DNSWorldEdition
      Synonym
        v8:item / v8:lang
        v8:item / v8:content
```

The paired EDT descriptor uses lower-case direct `name` and repeated `synonym`
entries with `key` and `value`. The UUID, name, and first Russian synonym value
match. Namespaces, element casing, wrapper depth, and synonym encoding differ.
The current EDT loader extracts the first accepted synonym content and maps UUID,
name, and `MetadataPayload(CommonMetadataPayload)` into a source-independent
`Configuration` with `WorkspaceFormat::Edt`.

The Designer root and paired EDT configuration raw hashes differ because their
serializations differ:

| Artifact | SHA-256 |
|---|---|
| `OneAgent_DesignerXML/Configuration.xml` | `b7eed83a154d0f68c858f10d991ee985fb6d7df878f7abb328c1e441d57a2bdd` |
| `OneAgent_EDTproject/src/Configuration/Configuration.mdo` | corpus-specific raw content; not an equivalence dimension |

Configuration identity must therefore use the declared UUID and semantic name,
not a file hash, path, wrapper, namespace spelling, or format version.

## Top-level metadata artifact contract evidence

For each supported family, Designer stores a direct descriptor at
`<PluralFamily>/<Name>.xml` and optional/nested artifacts below
`<PluralFamily>/<Name>/`. The direct descriptor is a
`{MDClasses}MetaDataObject` wrapper with one kind-specific child whose `uuid`
attribute is the stable metadata identity. Accepted common content is under
`Properties/Name` and repeated `Properties/Synonym/v8:item` values.

Example paired Catalog evidence:

| Dimension | Designer | EDT |
|---|---|---|
| Descriptor | `Catalogs/Products.xml` | `src/Catalogs/Products/Products.mdo` |
| Root child | `Catalog` | `mdclass:Catalog` |
| UUID | `92bcb692-56c4-4199-bf7e-e33cdd76a310` | same |
| Name | `Products` | same |
| Russian synonym | `Номенклатура` | same |
| Raw SHA-256 | `2ec9ce8e4b9b84893655f60bc00c99ba506dd906fc6f560fa13e02f5dfe9f149` | `917d9aeb244e71660849cb83bb4b27c1934d1d642561ce01ef8cc245c3742228` |

The following current EDT top-level families have direct paired Designer files.
Counts were recomputed from direct `*.xml` files and paired depth-two EDT
`*.mdo` descriptors:

| Designer / EDT family | Designer | EDT |
|---|---:|---:|
| Catalogs | 116 | 116 |
| Documents | 81 | 81 |
| Enums | 154 | 154 |
| CommonModules | 509 | 509 |
| Reports | 56 | 56 |
| DataProcessors | 89 | 89 |
| InformationRegisters | 174 | 174 |
| AccumulationRegisters | 21 | 21 |
| AccountingRegisters | 1 | 1 |
| CalculationRegisters | 0 | 0 |
| BusinessProcesses | 2 | 2 |
| Tasks | 1 | 1 |
| Roles | 1,004 | 1,004 |
| CommonCommands | 70 | 70 |
| CommonForms | 84 | 84 |
| CommonTemplates | 26 | 26 |
| HTTPServices | 2 | 2 |
| WebServices | 8 | 8 |
| XDTOPackages | 20 | 20 |
| Subsystems | 13 | 13 |
| EventSubscriptions | 99 | 99 |
| **Total** | **2,530** | **2,530** |

Equal family counts prove a paired non-empty inventory, not field equality. The
smallest source-independent metadata slice supported by current APIs is UUID,
`MetadataKind`, exact name, optional common synonym payload, configuration
ownership, and source provenance. Members, extension facts, register records,
command parameters, row rights, subsystem content/hierarchy, event selectors,
DCS content, XDTO types, and service children use distinct serialized shapes
and require separate source contracts.

### Artifact assembly and error evidence

The direct filename, direct kind child, declared `Properties/Name`, and UUID are
four independent observations available for deterministic agreement checks.
The repository can safely derive negative tests from exact source copies for:

- missing direct descriptor or family directory;
- duplicate candidate files for one canonical family/name key;
- filename versus declared name mismatch;
- family versus root-kind mismatch;
- missing, empty, duplicate, or malformed UUID/name/common synonym values;
- malformed XML, wrong namespace/root, unreadable file, reordered equivalent
  XML, and repeated parsing;
- unknown top-level directories and nested artifacts outside the accepted slice.

No checked-in malformed Designer corpus exists. Generated negative fixtures
must mutate provenance-backed exact copies and state the mutation; they must not
be presented as real valid source vocabulary.

## Module artifact contract evidence

The current EDT generic top-level module reader accepts Object, Manager, and
Common module roles with stable identities `<owner-id>:<role>`. Designer stores
their paired files under the top-level object directory:

| Role | Designer path | EDT path | Live count evidence |
|---|---|---|---:|
| Object | `<Family>/<Name>/Ext/ObjectModule.bsl` | `src/<Family>/<Name>/ObjectModule.bsl` | 310 in each corpus |
| Manager | `<Family>/<Name>/Ext/ManagerModule.bsl` | `src/<Family>/<Name>/ManagerModule.bsl` | 513 in each corpus |
| Common | `CommonModules/<Name>/Ext/Module.bsl` | `src/CommonModules/<Name>/Module.bsl` | 509 accepted owners |

Designer also contains `RecordSetModule.bsl`, `ValueManagerModule.bsl`,
configuration modules under `Ext/`, Form modules, and Command modules. Those
are not part of the smallest generic top-level module slice. Form and Command
modules depend on member parsing and owner-aware layouts; configuration,
RecordSet, and ValueManager roles need separate accepted graph ownership and
identity evidence.

Representative paired source after removing one leading UTF-8 BOM and
normalizing CRLF or CR to LF is byte-equal:

| Designer / EDT pair | Normalized SHA-256 | Equal |
|---|---|---|
| `CommonModules/AccessManagement/Ext/Module.bsl` / `src/CommonModules/AccessManagement/Module.bsl` | `5f3a55ae5d96509e5d6a16fa20c5a37275a57cd1b5111742ac374f345bf4d33b` | yes |
| `WebServices/Exchange/Ext/Module.bsl` / `src/WebServices/Exchange/Module.bsl` | `27acb605034d7a6d5fdf7663d35b2da1b74e2ac61d814cdc0432b83f7364d191` | yes |
| `HTTPServices/Site/Ext/Module.bsl` / `src/HTTPServices/Site/Module.bsl` | `c6d73a7da06f3e0b2c9509b5170004c0f5f5873e4a6a5aef6a4ac859e058f0d6` | yes |

The raw AccessManagement module hashes differ
(`3a057ffb...` Designer and `5f3a55ae...` EDT) because the Designer file uses
BOM/CRLF. Raw bytes and source paths remain provenance; normalized source is the
BSL parser input and conformance dimension. Normalization is not identity.

The current public `oneagent-bsl` crate exposes declaration, call, and Query
extractors. The EDT `AnalyzedBslModule` and `EdtModuleDescriptor` orchestration
types are adapter-local, so a Designer adapter must either call source-independent
BSL extractors directly or first introduce an independently justified shared
source-independent orchestration type. Depending on `oneagent-edt` or exposing
EDT types from the Designer adapter would violate the adapter boundary.

## Completeness and failure boundary evidence

The full registered Designer corpus is a complete official Designer dump: it
has both markers and passed import/CF round trips recorded in the corpus
registration. A reduced tracked fixture does not become complete merely by
copying those markers; absence alone cannot distinguish a deliberately partial
fixture from a corrupt complete dump.

The source-adapter framework therefore requires the caller or accepted entry
point to classify input as complete or explicitly partial. Repository evidence
supports two distinct production behaviors for ADR-0036 to decide:

- normal discovered roots are complete and required accepted artifacts fail the
  whole configuration;
- explicitly partial fixture/API builds accept a declared subset, retain missing
  as partial rather than missing/error, and never emit placeholders for absent
  entities.

`ConfigDumpInfo.xml` contains artifact identities and versions but has not been
proven to be a sufficient completeness oracle for every accepted semantic
family. The first slice may validate marker agreement and declared accepted
artifacts without claiming the manifest enumerates all required runtime facts.

Malformed required marker/configuration input must be fatal before a graph is
returned. For metadata or module siblings, ADR-0036 must choose and test whether
one malformed accepted artifact fails the configuration or is retained as a
typed entity-local outcome; the current EDT production builder generally uses
fatal structural errors and recoverable semantic-resolution diagnostics. No
Designer recovery rule is currently accepted.

## Canonical mapping and conformance candidate

The graph is source-format independent. The current public metadata, graph,
Query, Diff, report, Validation, complete-index, incremental-index, and BSL
declaration APIs are sufficient for a first slice without new node or edge
kinds.

The smallest coherent non-empty cross-adapter projection is:

1. configuration node ID, kind, exact name, accepted common payload;
2. accepted top-level metadata node ID, `MetadataKind`, exact name, accepted
   common payload;
3. configuration-to-metadata `Contains` identities;
4. accepted Object/Manager/Common module node ID, role kind, exact name;
5. metadata-to-module `Contains` identities;
6. Procedure and Function IDs, names, export status, module ownership, and
   declaration relations produced from normalized paired BSL source;
7. consumer-visible Query, Diff, report, Validation, and supported complete or
   incremental index projections over those facts;
8. terminal build success/failure category for the declared complete or partial
   scope.

The oracle must assert non-zero configuration, metadata, module, and declaration
counts for its selected fixture. A controlled UUID, name, synonym, module text,
or declaration change must produce the exact expected unequal projection so
filtering everything away cannot pass.

### Deliberate adapter-specific differences

These dimensions must remain outside canonical equality while still being
checked for deterministic exact provenance within each build:

- source paths and directory layout;
- producer identifiers and adapter-specific diagnostics;
- XML namespaces, wrappers, tag casing, format metadata, and serialization order;
- raw file bytes, UTF-8 BOM, CRLF/LF spelling, and source hashes;
- unsupported/deferred artifacts that are not part of the accepted slice.

The four invalid form-event bindings documented by the corpus registration are
outside the proposed first slice. Their loss prevents any later form-event
payload oracle from claiming losslessness without new evidence.

## Fixture and testability gate

The data gate passes for investigation, architecture, and the proposed first
slice:

- the complete real paired corpus is locally available and registered;
- exact markers, configuration fields, 21 paired family inventories, three
  generic top-level module roles, identities, payload candidates, and normalized
  BSL content are observable;
- positive, missing, duplicate, mismatch, malformed, unsupported, unreadable,
  reordered, repeated, partial, conflict, and controlled-change cases can be
  created from exact copies without inventing valid vocabulary;
- public production and consumer APIs provide observable acceptance results;
- focused crate tests and the canonical full workspace gate are known.

A tracked reduced fixture must follow the corpus policy. Parser-local reductions
may exact-copy the smallest artifacts and record every hash and mutation. A
fixture claimed as a complete Designer project must be produced with official
1C tooling; hand-authored `ConfigDumpInfo.xml` is prohibited. Exact-copying the
2,382,139-byte full manifest into a small partial fixture is possible but does
not make the fixture complete and should be avoided unless its manifest content
is directly under test.

## Consumer and implementation inventory

| Area | Current state | Sprint 14 effect candidate |
|---|---|---|
| Workspace | `WorkspaceFormat::DesignerXml` already exists | no enum expansion required |
| Filesystem detector | EDT-only markers and boundary | add accepted Designer rule and conflict tests |
| Cargo workspace | no Designer adapter crate | add one bounded adapter member |
| EDT adapter | monolithic EDT-specific builder and coverage | preserve behavior; do not reuse EDT-local types as authority |
| Metadata | source-independent kinds and payloads exist | reuse without public expansion |
| BSL | public line extractors exist | reuse normalized source input |
| Graph | public node/edge/provenance/query/diff/report/validation APIs exist | reuse existing kinds and identities |
| Semantic indexes | complete and incremental generic consumers exist | add conformance transitions, not new index dimensions |
| Coverage | graph-domain and EDT registries exist | decide a truthful Designer-specific registry/evidence boundary; do not relabel EDT coverage |
| Runtime/CLI | no semantic source orchestration product path | deferred to later sprints |

No new third-party production dependency is required by observed source shapes:
the workspace already uses `quick-xml` in the EDT adapter, and standard
filesystem/ordered collections cover discovery and deterministic assembly.

## Decision readiness and recommended first slice

Repository evidence is sufficient for ADR-0036. The smallest coherent slice is:

- both-marker hierarchical detection and exact configuration loading;
- the 21 existing EDT top-level family mappings, including zero live Calculation
  Registers, limited to UUID/name/common synonym/configuration ownership;
- existing generic top-level Object, Manager, and Common module roles;
- Procedure and Function declarations and their existing ownership relations;
- complete and explicit partial input modes;
- the bounded conformance projection above.

The architecture decision must still choose exact version compatibility,
symlink/overlap behavior, manifest validation depth, synonym locale selection,
entity-local versus configuration-fatal structural errors, public build-result
shape, and Designer-specific Coverage representation. These are decision inputs,
not missing external data.

Metadata members, Form/Command/configuration/RecordSet/ValueManager modules,
references and non-ownership edges, Subsystem/Event/DCS/XDTO/service internals,
extensions, flat dumps, binary artifacts, parent configurations, and whole-graph
equivalence remain deferred.

## Validation evidence

Investigation commands successfully:

- recomputed 10,964 files, 7,585 XML, 2,717 BSL, 40 top-level directories,
  237,458,744 bytes, and 15,649 manifest metadata entries;
- recomputed every family count in the paired 2,530-object table;
- recomputed representative raw SHA-256 values for markers, configuration,
  Products, AccessManagement descriptor, and modules;
- proved normalized byte equality and hashes for AccessManagement, Exchange,
  and Site modules;
- inspected current detector, workspace format, EDT descriptor/module/builder,
  BSL extractor, graph consumer, fixture, Coverage, and Cargo surfaces.

No production command, 1C tooling command, ignored-corpus mutation, or Rust test
was required for this documentation-only evidence task.
