# Form and Command Source Investigation

## Status

Decision-ready investigation recorded for Sprint 7.

This document is repository evidence, not an implementation change. It does
not change production behavior, public APIs, or Semantic Coverage.

## Objective

Identify the live EDT source boundary, implemented compatibility baseline,
consumer surface, and smallest evidence-backed Form and Command capability that
can be accepted and executed during Sprint 7 without treating the conceptual
Semantic Model 2.0 UI taxonomy as an implemented API.

The investigation was performed against repository HEAD `eab870ab1b3f`. Mutable
repository facts must be rechecked before implementation.

## Evidence classification

- **Confirmed** means directly present in repository code, tests, real source
  artifacts, or committed review evidence.
- **Accepted** means normative in an accepted ADR or authoritative architecture
  document but not necessarily implemented for Forms and Commands.
- **Unknown** means the repository does not contain enough evidence or an
  accepted contract to implement the behavior safely.

## Repository-owned source inventory

### Corpus summary

At the investigated baseline, `OneAgent_EDTproject` contains:

- 960 subordinate `Forms/<Name>/Form.form` artifacts;
- 946 subordinate `Forms/<Name>/Module.bsl` artifacts;
- 120 subordinate `Commands/<Name>/CommandModule.bsl` artifacts;
- 84 Common Form `.mdo` descriptors;
- 70 Common Command `.mdo` descriptors;
- 21 Command Group `.mdo` descriptors;
- 101 subordinate Command modules containing at least one `OpenForm(...)`
  spelling;
- 32 Common Command modules containing at least one `OpenForm(...)` spelling;
- 37 `.mdo` artifacts containing a non-empty `commandParameterType` block.

These counts describe the current repository corpus. They are not completeness
claims about EDT or the 1C platform.

### Representative artifacts

| Artifact | Confirmed evidence | Current production behavior |
|---|---|---|
| `OneAgent_EDTproject/src/Catalogs/CounterpartiesProducts/CounterpartiesProducts.mdo` | Three UUID-backed subordinate Forms, one UUID-backed subordinate Command, direct multilingual synonyms, use-purpose values, a command group value, and an empty command-parameter type. | The metadata-structure reader emits the Form and Command descriptors with UUID identity, canonical name, immediate Catalog owner, and declared provenance. Other content is discarded. |
| `OneAgent_EDTproject/src/Catalogs/CounterpartiesProducts/Forms/PriceImport/Form.form` | Form elements, data paths, form command bindings, one form event, and form attributes. | No production reader consumes `Form.form`. No internal UI nodes, bindings, or event facts are emitted. |
| `OneAgent_EDTproject/src/Catalogs/CounterpartiesProducts/Forms/PriceImport/Module.bsl` | Procedures and functions implementing form behavior. | The current module reader does not descend into subordinate Form directories, so this module is not analyzed or emitted. |
| `OneAgent_EDTproject/src/Catalogs/CounterpartiesProducts/Commands/CounterpartiesProductsPriceImport/CommandModule.bsl` | `CommandProcessing` contains the exact literal call `OpenForm("Catalog.CounterpartiesProducts.Form.PriceImport")`. | `CommandModule.bsl` is not a recognized module-reader candidate. The call, its containing Procedure, and its navigation target are not emitted. |
| `OneAgent_EDTproject/src/CommonForms/AccessRights/AccessRights.mdo` | UUID, name, multilingual synonym, use purposes, and standard-command policy. | The top-level descriptor is emitted as `NodeKind::Metadata(MetadataKind::CommonForm)` with the ADR-0023 common payload. |
| `OneAgent_EDTproject/src/CommonForms/AccessRights/Module.bsl` | Common Form executable behavior. | `Module.bsl` is discovered through the existing top-level module path and represented as the current common-module-shaped `NodeKind::Module` child. Its existing identity is a compatibility constraint. |
| `OneAgent_EDTproject/src/CommonCommands/AccessRights/AccessRights.mdo` | UUID, multilingual synonym, group, four Catalog reference parameter types, and representation. | The top-level descriptor is emitted as `NodeKind::Metadata(MetadataKind::Command)` with common payload; command parameter types are not retained as reference observations. |
| `OneAgent_EDTproject/src/CommonCommands/AccessRights/CommandModule.bsl` | Command entry-point code and form-opening behavior. | `CommandModule.bsl` is not recognized by the current module reader and contributes no Module, symbol, or navigation fact. |
| `OneAgent_EDTproject/src/CommandGroups/Information/Information.mdo` | UUID, name, multilingual synonym, representation, and command-group category. | `CommandGroups` has no EDT directory mapping or source-independent metadata kind and is ignored by discovery. |

### Additional confirmed variants

The real corpus contains both exact literal and non-literal form-opening calls.
Examples include:

- explicit subordinate form targets such as
  `Catalog.CounterpartiesProducts.Form.PriceImport`;
- Common Form targets such as `CommonForm.UserGroups`;
- default-form spellings such as `DataProcessor.WorkplaceForSales.Form`;
- shorthand list- or object-form spellings such as
  `Catalog.DataExchangesSessions.ListForm`;
- dynamic first arguments such as `OpenForm(Name, ...)`;
- calls whose arguments span several lines.

These are distinct source forms. The occurrence of the `OpenForm` spelling
alone is not sufficient evidence for one wildcard resolution rule.

Command parameter types include currently mapped metadata-reference spellings
such as `CatalogRef.Products`, `DocumentRef.CustomerOrder`, and
`TaskRef.PerformerTask`, as well as deferred spellings such as
`DefinedType.DocumentComment`. A first slice can reuse only the nine metadata
target kinds already accepted by ADR-0025.

## Confirmed implementation boundary

### Entity model and identity

The source-independent model already declares:

- `MetadataKind::CommonForm` for top-level Common Forms;
- `MetadataKind::Command` for top-level Common Commands;
- `MetadataKind::Form` as an explicit model variant that is not applicable to
  the EDT top-level discovery path;
- `NodeKind::Form` for subordinate Forms;
- `NodeKind::Command` for subordinate Commands.

The metadata-structure reader uses the source UUID unchanged when present. A
UUID-less child fallback is `<owner-id>:<child-kind>:<name>`. Forms and Commands
are completed under their metadata-object owner; no deeper Form or Command
nesting is currently recognized.

Common Form and Common Command top-level nodes use their descriptor UUIDs and
the generic metadata payload contract. This split is committed behavior and is
not a Sprint 7 modeling choice.

### Ownership and graph emission

`collect_metadata_child` emits subordinate Forms and Commands as flat nodes.
`collect_metadata_child_ownership` emits exactly one provenance-backed
`Contains` edge from the metadata object to each child. Node collection occurs
before ownership insertion.

Graph validation currently:

- accepts `Metadata(_) --Contains--> Form`;
- accepts `Metadata(_) --Contains--> Command`;
- requires one owner for every Form and Command;
- rejects missing, incompatible, and multiple owners;
- does not accept `Form --Contains--> Module` or
  `Command --Contains--> Module`.

Inline production-builder tests prove subordinate Form and Command identity,
ownership, Query navigation, provenance, graph validation, and repeated-build
stability. The EDT Coverage registry classifies the existing Form and Command
node capabilities as `Supported`. This evidence covers declaration and
ownership only, not internal Form semantics, executable modules, parameter
references, or navigation relations.

### Typed content

`MetadataMemberPayload` is compatible only with `NodeKind::Attribute` and
`NodeKind::TabularSection`. The metadata-structure reader intentionally parses
member synonym only for those two kinds. Form and Command synonyms, use
purposes, group values, representation, and other descriptor content are
discarded.

The real Form and Command corpus commonly contains multiple locale values.
Selecting one value or extending the payload to a localized collection would
require a separate content contract. Optional single-value member synonym from
ADR-0028 cannot be silently reused.

### Module discovery and BSL analysis

`FileSystemEdtModuleReader` checks only the supplied metadata-object directory
and recognizes:

- `ObjectModule.bsl` as `EdtModuleKind::Object`;
- `ManagerModule.bsl` as `EdtModuleKind::Manager`;
- `Module.bsl` as `EdtModuleKind::Common`.

It does not recurse into `Forms/<Name>` or `Commands/<Name>` and does not
recognize `CommandModule.bsl`. Consequently:

- top-level Common Form `Module.bsl` can use the existing module pipeline;
- subordinate Form modules are ignored;
- subordinate and Common Command modules are ignored;
- BSL declarations, queries, calls, and diagnostics inside ignored modules do
  not reach the graph.

The existing graph and BSL analysis can represent Module, Procedure, Function,
Query, and Calls facts once an accepted adapter path supplies a module
descriptor. No separate UI-specific callable kind is required for the first
module slice.

### References

The metadata-structure reader observes mapped `types` text while a child is
pending, but the production reference collection accepts only Attribute,
Dimension, and Resource sources. It does not distinguish command parameter
types from the existing generic `Type` role. The generic top-level descriptor
reader does not retain Common Command parameter types.

ADR-0024 provides the public immutable request ledger, provenance lifecycle,
terminal outcomes, diagnostics, statistics, and deterministic aggregation.
ADR-0025 currently allows `References` only for its completed metadata-member
and access-right endpoint matrices. ADR-0017 permits form or command dependency
origins only after a separate UI/binding decision.

No production Command reference fact is currently emitted.

### Navigation

The live `EdgeKind` enum has no `Opens` or structured UI relation. The
conceptual Semantic Model 2.0 taxonomy lists `UiEdgeKind::Opens`, but that
conceptual example is not an implemented public API or endpoint contract.

Current BSL call extraction and resolution do not preserve a static
`OpenForm(...)` target as a typed navigation observation. No graph consumer can
distinguish an exact form-opening fact from another platform call.

The existing Semantic Index, Query, Diff, Impact, Validation, Coverage, report,
and incremental-index consumers enumerate `EdgeKind`. A new navigation edge
therefore requires an explicit source-independent graph task before adapter
emission.

### Form internals and Command Groups

No production reader consumes `Form.form`. Form elements, form attributes,
form commands, events, action bindings, data paths, captions, and nested UI
identity are unmodeled.

`CommandGroups` is absent from `MetadataKind`, `NodeKind`, supported EDT
directory mappings, validation, and Coverage. A `<group>` value may name a
standard placement category or a declared Command Group depending on source
form. The current evidence has not been normalized into a complete typed
contract.

## Consumer and compatibility inventory

| Surface | Current constraint for Sprint 7 |
|---|---|
| Metadata domain | Existing top-level payload and member identity remain authoritative. Form/Command descriptors must not be copied into an untyped property map. |
| Graph node model | Existing Form, Command, Module, Procedure, and Function nodes are sufficient for the bounded module slice. New internal UI nodes are not required. |
| Graph edge model | `Contains`, `References`, and `DependsOn` have accepted meanings. A direct form-opening fact requires a new precise relation rather than overloading one of them. |
| Validation | Ownership and reference endpoint matrices are closed. Any new owner or relation endpoint must be added explicitly. |
| Query and Semantic Index | Generic node, edge, ownership, dependency, usage, and traversal APIs already consume typed graph facts. A new edge kind must preserve deterministic ordering and compatibility of existing methods. |
| Diff and Incremental Index | Node and edge identity changes must remain canonical and source-order independent. New facts must participate in complete and incremental snapshots equivalently. |
| Impact | Ownership remains policy-controlled. A resolved form-opening relation needs an explicit dependency/usage and reverse-impact decision. |
| Resolution | Owner-scoped child lookup can resolve a named subordinate Form under an exact metadata owner. Common Form targets use exact name-and-kind resolution. |
| Reference requests | Command parameter types can reuse the public lifecycle only after adding a distinct role and precise source endpoint. |
| Reports and diagnostics | Unsupported, malformed, dynamic, missing, ambiguous, incompatible, and partial navigation outcomes must remain observable without placeholder nodes. |
| Coverage | Existing Form/Command declaration capabilities remain `Supported`. Modules, Command references, and navigation need independent completion evidence; architecture alone changes no status. |

## Implemented-versus-missing matrix

| Capability | Evidence state | Sprint 7 classification |
|---|---|---|
| Top-level Common Form and Common Command discovery | Implemented and `Supported` | Compatibility baseline |
| Subordinate Form and Command declaration | Implemented and `Supported` | Compatibility baseline |
| UUID and owner-scoped UUID-less identity | Implemented | Compatibility baseline |
| Metadata-object ownership and Query navigation | Implemented and validated | Compatibility baseline |
| Deterministic node/edge provenance and repeated builds | Implemented for declaration slice | Compatibility baseline |
| Existing Common Form `Module.bsl` path | Implemented through generic module discovery | Compatibility baseline; preserve identity |
| Subordinate Form module discovery | Real corpus present; no reader path | Decision-ready gap |
| Common and subordinate Command module discovery | Real corpus present; no reader path | Decision-ready gap |
| BSL symbols inside Form and Command modules | Existing analyzer available; source modules not supplied | Decision-ready after module discovery |
| Command parameter metadata references | Real mapped and deferred tokens present; no production observations | Decision-ready for the existing nine-kind allowlist only |
| Exact literal `OpenForm` to explicit subordinate or Common Form | Many real examples; no typed extractor or edge | Decision-ready bounded navigation gap |
| Default-form and shorthand targets | Real examples; target may not have an explicit Form node | Deferred |
| Dynamic `OpenForm` first argument | Real examples; no exact target identity | Diagnostic-only or deferred; no edge |
| Form internal elements, attributes, commands, events, and data paths | Large real corpus; no parser, identity, or endpoint decisions | Deferred beyond the first slice |
| Command Group entities and custom group references | Real descriptors present; no model or complete value classification | Deferred pending separate investigation |
| Explicit Command execution relation | Command modules and conventional handlers exist; no accepted runtime-trigger contract | Deferred; ownership does not imply execution |
| Form/Command multilingual display payload | Real values present; no locale collection contract | Deferred |

## Smallest decision-ready Sprint 7 slice

The smallest coherent slice that advances Forms, Commands, references, and
navigation without opening the full UI model consists of three additions:

1. discover and analyze subordinate Form modules and Common/subordinate Command
   modules as ordinary graph Module nodes owned by the existing canonical Form
   or Command entity;
2. preserve mapped command parameter types as typed Command reference requests,
   resolving only the existing ADR-0025 nine-kind metadata allowlist and
   projecting precise `References` and justified `DependsOn` facts;
3. extract complete static literal `OpenForm(...)` calls from a Command module,
   resolve only explicit subordinate Form and Common Form targets, and emit one
   dedicated provenance-backed `Opens` relation from the containing Procedure
   to the resolved Form.

This slice requires one new edge kind and precise additions to existing
ownership, reference, dependency, Query, Impact, Coverage, and diagnostic
contracts. It requires no new Form, Command, internal UI, external, or
placeholder node kind.

The first navigation grammar is intentionally narrow:

- `CommonForm.<Name>` resolves to
  `NodeKind::Metadata(MetadataKind::CommonForm)`;
- `<SupportedKind>.<Owner>.Form.<Name>` resolves to an explicit
  `NodeKind::Form` child of the exact resolved metadata owner;
- the first argument must be a complete static string literal in a complete
  call statement inside a Procedure owned by a recognized Command module.

Default-form aliases, shorthand `ListForm` or `ObjectForm` spellings, dynamic
expressions, Functions, Form-module calls, and unrepresented metadata kinds are
not edge-producing forms in the first slice.

## Accepted constraints

- ADR-0006 keeps the semantic graph typed and deterministic.
- ADR-0007 permits later readers to enrich existing Form, Command, and Module
  facts without replacing their canonical identities.
- ADR-0017 requires a separate decision before form or command origins can
  produce `DependsOn`.
- ADR-0023 keeps nested UI and subordinate member facts out of top-level
  metadata payload.
- ADR-0024 requires public request lifecycle, deterministic provenance, typed
  terminal outcomes, diagnostics, and statistics for new reference families.
- ADR-0025 rejects wildcard UI reference endpoints.
- ADR-0028 keeps its optional member payload compatible only with Attribute and
  TabularSection until another accepted decision changes that boundary.
- Sprint 6 completion and current Coverage status are compatibility constraints,
  not Sprint 7 deliverables.

## Unknowns and deferred evidence

- Complete `Form.form` identity, hierarchy, binding, event, and data-path
  semantics.
- Command Group identity and the distinction between platform placement tokens
  and custom group references.
- Multilingual display-content representation for subordinate Forms and
  Commands.
- Default-form aliases, shorthand form names, dynamic names, expressions,
  variables, localization, and platform-generated forms.
- Form-opening calls outside a recognized Command module or inside a Function.
- An explicit `Command --Executes--> Procedure` contract.
- Form-command actions and event-handler execution relations.
- Unresolved external or placeholder Form nodes.
- Designer XML equivalence and cross-adapter identity.

These unknowns do not block the bounded module, parameter-reference, and static
navigation slice. They remain outside its Coverage claims.

## Codex Framework readiness

The existing investigation, architecture, graph-model, parser,
graph-emission, implementation, and review profiles and templates express the
required task boundaries. The current sprint-planning and sequential-execution
contracts provide the required prerequisite and state gates. No reusable
Framework gap was found, so `docs/codex/` must remain unchanged.

## Decision readiness and next action

The bounded slice is decision-ready. ADR-0029 may define the canonical module,
Command parameter-reference, and static form-opening contracts while leaving
Form internals, Command Groups, default/dynamic targets, localized payload, and
explicit command execution deferred. Architecture acceptance alone must not
change production behavior or Coverage.
