# ADR-0029: Form and Command Navigation Semantics

## Status

Accepted

## Context

Sprint 3 completed the first Form and Command declaration slice. The EDT
adapter already emits:

- Common Forms as `NodeKind::Metadata(MetadataKind::CommonForm)`;
- Common Commands as `NodeKind::Metadata(MetadataKind::Command)`;
- subordinate Forms as `NodeKind::Form`;
- subordinate Commands as `NodeKind::Command`;
- canonical UUID or owner-scoped fallback identity;
- one provenance-backed `Contains` edge from the immediate metadata owner;
- deterministic Query, Validation, Diff, Impact, and Coverage behavior for
  those declaration facts.

The Sprint 7 source investigation in
`docs/architecture/form-command-source-investigation.md` confirms that this
behavior is implemented and `Supported`. Recreating the nodes or their existing
ownership would not expand the knowledge model.

The same investigation identifies three connected gaps with repository-owned
source evidence:

1. subordinate Form `Module.bsl` and Common/subordinate Command
   `CommandModule.bsl` artifacts do not reach the module and BSL graph pipeline;
2. mapped `commandParameterType` values do not enter the public semantic
   reference-request lifecycle;
3. complete static literal `OpenForm(...)` statements in Command modules do not
   produce a typed navigation fact.

Semantic Model 2.0 conceptually lists richer UI nodes and an `Opens` relation,
but the live public graph contains only flat `NodeKind` and `EdgeKind` enums.
An accepted endpoint, identity, resolution, provenance, validation, Query,
Impact, and Coverage contract is required before production code may add a new
navigation relation.

## Decision

Sprint 7 accepts a bounded Form and Command slice with three outcomes:

1. existing Form and Command entities own their accepted executable modules;
2. accepted command parameter metadata types produce precise `References` and
   `DependsOn` facts through the public request lifecycle;
3. a complete resolved static form-opening call produces a new
   `EdgeKind::Opens` fact from the containing Procedure to the canonical Form.

This decision does not accept the complete conceptual UI taxonomy. It adds no
internal Form element, Form attribute, Form command, event, binding, external,
unknown, or placeholder node.

## Canonical semantic statements

### Module ownership

```text
Form or Command --Contains--> Module
Module --Contains--> Procedure or Function
```

The first relation means that the module artifact is executable content owned
by the canonical Form or Command entity. The second is the existing BSL
declaration contract.

### Command parameter reference

```text
Command --References--> parameter metadata type
Command --DependsOn--> parameter metadata type
```

`References` records the resolved declared parameter-type reference.
`DependsOn` records the normalized direct dependency justified by that resolved
type fact. Neither relation is inferred from command placement, form opening,
or ownership.

### Form opening

```text
Procedure --Opens--> Form
```

`Opens` means that a complete accepted static call in the Procedure directly
requests opening the resolved canonical Form target.

It is not:

- ownership;
- generic mention or display text;
- a callable `Calls` relation;
- a command trigger or handler relation;
- proof that the form opens successfully at runtime;
- a reference to an unresolved, generated, default, or dynamic form;
- a persisted transitive navigation path.

## Canonical entity boundary

The existing entity split remains authoritative:

| Source concept | Canonical graph kind |
|---|---|
| Common Form descriptor | `NodeKind::Metadata(MetadataKind::CommonForm)` |
| Common Command descriptor | `NodeKind::Metadata(MetadataKind::Command)` |
| Subordinate Form declaration | `NodeKind::Form` |
| Subordinate Command declaration | `NodeKind::Command` |
| Form or Command module artifact | `NodeKind::Module` |
| BSL declaration in the module | `NodeKind::Procedure` or `NodeKind::Function` |

`MetadataKind::Form` remains not applicable to the EDT top-level discovery
path. No duplicate metadata node is created for a subordinate Form or Command.

## Identity

### Existing entities

Existing Common and subordinate Form and Command identities remain unchanged.
Source UUIDs remain canonical. The existing owner-scoped UUID-less fallback is
not redefined by this decision.

The existing Common Form `Module.bsl` identity and name emitted through the
generic top-level module path remain unchanged as a compatibility constraint.

### New module observations

New module identity is derived only from the canonical owner identity and a
stable role discriminator:

```text
<form-node-id>:form_module
<command-node-id>:command_module
```

The canonical module names are `FormModule` and `CommandModule`. Source paths,
display synonyms, owner names, insertion order, and random identifiers do not
participate in identity.

The same Common or subordinate Command `CommandModule.bsl` role uses the same
`command_module` discriminator relative to its canonical owner. Equal Command
names under different owners therefore cannot collide.

### Edge identity

Every accepted relation uses the standard graph edge identity:

```text
(source_node_id, target_node_id, edge_kind)
```

Multiple equivalent observations for the same edge aggregate provenance in
deterministic order. Repeated parsing and repeated graph construction must not
create duplicate edge identities.

## Ownership endpoint matrix

The precise additive `Contains` matrix is:

| Owner | Child | Accepted |
|---|---|---:|
| `NodeKind::Form` | `NodeKind::Module` | Yes |
| `NodeKind::Command` | `NodeKind::Module` | Yes |
| `NodeKind::Metadata(MetadataKind::Command)` | `NodeKind::Module` | Yes |
| `NodeKind::Metadata(MetadataKind::CommonForm)` | `NodeKind::Module` | Existing compatibility |
| Any internal UI, placeholder, or unknown node | `NodeKind::Module` | No |
| `NodeKind::Form` or `NodeKind::Command` | Any metadata node | No |

The existing Metadata-to-Form and Metadata-to-Command ownership rules remain
unchanged. Every accepted module requires exactly one owner. The graph must
reject missing, incompatible, multiple, and self ownership.

Module nodes are collected before ownership edges are inserted. Filesystem or
XML traversal order must not affect graph validity or output ordering.

## Accepted module source contract

The accepted production artifacts are:

| Owner | Artifact | Module role |
|---|---|---|
| Subordinate Form | `Forms/<FormName>/Module.bsl` | `form_module` |
| Subordinate Command | `Commands/<CommandName>/CommandModule.bsl` | `command_module` |
| Common Command | `CommonCommands/<CommandName>/CommandModule.bsl` | `command_module` |
| Common Form | Existing `<CommonFormName>/Module.bsl` path | Existing compatibility behavior |

The reader must join a subordinate directory to an already parsed Form or
Command declaration by exact source name under the same metadata owner. It must
not synthesize a Form or Command node from an orphan directory.

Missing optional module artifacts do not invalidate the owner entity. Orphan,
duplicate, unreadable, wrong-kind, and name-mismatched module directories must
produce deterministic typed outcomes and no guessed ownership.

Accepted module descriptors enter the existing BSL declaration, query, call,
provenance, diagnostic, and deterministic graph-contribution pipeline. This
decision does not change the meaning of existing BSL `Calls`, Query, Reads, or
Writes facts.

## Command parameter reference contract

### Source forms

The first slice accepts only a direct `commandParameterType/types` observation
owned by a parsed Common or subordinate Command descriptor.

The parser must preserve a distinct semantic role equivalent to
`CommandParameterType`; it must not report the observation as an Attribute or
generic member type.

Multiple accepted type values produce multiple canonical request observations.
Duplicate equal values aggregate deterministically.

### Target allowlist

The first slice reuses exactly the nine ADR-0025 metadata mappings:

- Catalog;
- Document;
- Enumeration;
- Information Register;
- Accumulation Register;
- Accounting Register;
- Calculation Register;
- Business Process;
- Task.

Primitive types, `DefinedType`, platform types, unrecognized prefixes, and
every other metadata family remain unsupported or deferred. They do not produce
lower-confidence edges.

### Reference endpoint matrix

| Source | Target | `References` | Companion `DependsOn` |
|---|---|---:|---:|
| `NodeKind::Command` | Accepted `NodeKind::Metadata(...)` target | Yes | Yes |
| `NodeKind::Metadata(MetadataKind::Command)` | Accepted `NodeKind::Metadata(...)` target | Yes | Yes |
| Form, Module, Procedure, Function, or internal UI node | Any target | No | No |
| Command | Unsupported or unresolved target | No | No |

The graph validator must enumerate the exact target kinds. Wildcard
`Command --References--> Metadata(_)` and
`Command --DependsOn--> Metadata(_)` branches are not accepted.

### Request lifecycle

Command parameter observations use the ADR-0024 public request lifecycle:

- collection with source provenance;
- immutable canonical request identity;
- exact name-and-kind resolution;
- deterministic candidate ordering and provenance aggregation;
- resolved, missing, ambiguous, incompatible, partial, malformed, and
  unsupported terminal outcomes;
- diagnostics, statistics, report, validation, and build-diff participation.

Unresolved, ambiguous, incompatible, partial, malformed, and unsupported
observations emit no `References` or `DependsOn` edge and create no placeholder
node.

## Static form-opening source contract

### Accepted call boundary

The first slice accepts a call only when every condition holds:

1. the source artifact is an accepted Common or subordinate
   `CommandModule.bsl`;
2. the call is contained directly in a parsed `Procedure`;
3. the callee spelling is exactly `OpenForm`;
4. the first argument is one complete static string literal;
5. the literal matches one accepted explicit target grammar;
6. the target resolves uniquely to a compatible canonical Form node.

The initial target grammar is:

```text
CommonForm.<FormName>
<SupportedMetadataPrefix>.<OwnerName>.Form.<FormName>
```

The supported metadata-prefix allowlist is limited to existing EDT metadata
kinds that can own an explicit subordinate `NodeKind::Form` in the current
model:

- `Catalog`;
- `Document`;
- `Report`;
- `DataProcessor`;
- `InformationRegister`;
- `AccumulationRegister`;
- `AccountingRegister`;
- `CalculationRegister`;
- `BusinessProcess`;
- `Task`.

Every prefix maps to one exact `MetadataKind`. The parser must not use a
wildcard prefix or infer a missing kind from a globally unique name.

### Resolution

`CommonForm.<FormName>` resolves by exact name and
`NodeKind::Metadata(MetadataKind::CommonForm)`.

An explicit subordinate target resolves in two stages:

1. resolve exactly one metadata owner by canonical name and mapped kind;
2. resolve exactly one `NodeKind::Form` child with the requested name under
   that owner.

Global Form-name resolution is not an accepted replacement for owner-scoped
resolution. A same-named Form under another owner is unrelated.

### Opens endpoint matrix

| Source | Target | Accepted |
|---|---|---:|
| `NodeKind::Procedure` in an accepted Command module | `NodeKind::Form` | Yes |
| `NodeKind::Procedure` in an accepted Command module | `NodeKind::Metadata(MetadataKind::CommonForm)` | Yes |
| Procedure outside an accepted Command module | Any Form | No |
| Function, Module, Form, or Command node | Any Form | No |
| Procedure | Any non-Form node | No |

The first implementation must not use a broad
`Procedure --Opens--> Metadata(_)` rule.

### Excluded call forms

The first slice emits no `Opens` edge for:

- dynamic, computed, concatenated, localized, or variable first arguments;
- default-form spellings such as `DataProcessor.Name.Form`;
- shorthand spellings such as `Catalog.Name.ListForm` or
  `Document.Name.ObjectForm`;
- platform-generated or undeclared forms;
- unsupported metadata prefixes;
- calls inside a Function;
- calls from Form modules, ordinary metadata modules, or other source families;
- malformed or incomplete calls;
- missing, ambiguous, incompatible, partial-workspace, or external targets.

Rejected candidates produce the typed diagnostic or unsupported classification
appropriate to their established evidence. They never produce placeholder or
lower-confidence edges.

## Provenance

Every new Module node, ownership edge, Command reference request, resolved
reference edge, dependency edge, diagnostic, and `Opens` edge carries
deterministic provenance.

Module provenance identifies:

- source artifact path;
- canonical owner identity and kind;
- accepted module role;
- producer;
- declared or parsed origin as appropriate.

Command parameter provenance identifies:

- descriptor artifact;
- Command identity;
- parameter-type role and raw token;
- mapped target kind and canonical target identity when resolved;
- collection or resolution stage.

`Opens` provenance identifies:

- `CommandModule.bsl` artifact;
- canonical Command, Module, and containing Procedure identities;
- complete accepted literal;
- resolved Form target identity;
- producer;
- `FactOrigin::Resolved`;
- exact confidence;
- `ResolutionState::Resolved`.

Multiple evidence records for one edge sort and deduplicate through the existing
canonical provenance rules.

## Diagnostics and partial workspaces

The implementation must preserve typed deterministic outcomes for:

- unreadable, orphaned, duplicated, or mismatched Form/Command module files;
- malformed or unsupported command parameter types;
- missing, ambiguous, incompatible, and partial parameter targets;
- malformed, unsupported, or dynamic form-opening calls;
- missing, ambiguous, incompatible, and partial Form targets;
- wrong source module or callable kind.

Diagnostics identify the source artifact, containing semantic owner, raw
observation, expected kind where known, ordered candidates, and supporting
provenance. A rejected observation does not invalidate another independently
complete observation from the same artifact.

## Determinism

Given identical source artifacts, module discovery, descriptor observations,
call candidates, resolution candidates, diagnostics, requests, nodes, edges,
provenance, Query results, reports, and Coverage output must be equal regardless
of filesystem order, XML order, insertion order, or repeated execution.

Equivalent observations aggregate in ordered collections. Source-order
reversal and repeated-build tests are required for every new production path.

## Query, dependency, usage, and Impact policy

The existing generic Query API remains the public source-independent facade.
No adapter-specific Form or Command query API is added.

`Opens` participates in generic edge-kind filtering and direct navigation:

- outgoing `Opens` from a Procedure is a direct dependency on the opened Form;
- incoming `Opens` to a Form is direct usage by that Procedure;
- reverse traversal participates in Impact propagation from a changed Form to
  Procedures that directly open it;
- transitive behavior is computed by bounded graph traversal and is not stored.

The first slice emits no companion `References` or `DependsOn` edge for an
`Opens` fact. `Opens` retains the specific navigation meaning and participates
directly in dependency, usage, and Impact classification.

Command parameter `References` and `DependsOn` use the existing Query and
Impact policies for those edge kinds. Their original and normalized facts are
both retained.

Ownership remains policy-controlled through `Contains`. Form or Command
ownership does not automatically become a dependency.

## Diff and incremental-index policy

Existing canonical node and edge snapshots remain authoritative. New module
nodes, ownership edges, Command reference facts, diagnostics, and `Opens` edges
participate in graph and build-result Diff through their stable identities and
content.

The shared Semantic Index and incremental update path must produce results
equivalent to a clean full rebuild. This decision does not introduce a second
adapter-owned index or cache.

## Coverage Registry completion criteria

Architecture acceptance changes no current status or aggregate count.

Existing Form and Command declaration and ownership capabilities remain
`Supported`; they must not be reopened or counted as new Sprint 7 evidence.

New or expanded capabilities may transition only after the production path
proves all applicable evidence:

- source-independent model declaration;
- precise validator endpoint rules;
- real source discovery and parsing;
- stable identity;
- canonical graph emission;
- public Query behavior;
- Diff and incremental full-rebuild equivalence;
- Impact policy where applicable;
- complete provenance;
- positive, negative, malformed, unsupported, missing, ambiguous,
  incompatible, partial, duplicate, reordered-source, and repeated-build tests;
- representative `FileSystemEdtSemanticGraphBuilder` integration evidence;
- deterministic diagnostics, request ledger, statistics, and report output;
- independent graph-domain and EDT Coverage transitions where required.

Coverage must distinguish at least:

- Form module discovery and ownership;
- Command module discovery and ownership;
- Command parameter reference production;
- `semantic_edge.opens` production and consumer support.

Only completed capabilities transition. Deferred Form internals, Command
Groups, dynamic/default targets, localized payload, and execution relations do
not become required evidence for the bounded slice.

## Relationship with existing ADRs

ADR-0006 remains authoritative for the typed deterministic graph. This ADR adds
one precise edge kind; it does not replace the flat current graph model with the
conceptual nested taxonomy.

ADR-0007 remains authoritative for EDT contribution and canonical identities.
This ADR implements the deferred enrichment path for accepted Form and Command
modules without changing existing entity IDs.

ADR-0017 remains authoritative for `DependsOn`. This ADR accepts only resolved
Command parameter metadata types as a new normalized dependency origin.
`Opens` participates directly in dependency queries and does not emit a
companion `DependsOn`.

ADR-0023 remains authoritative for top-level metadata payload. Form internals
and subordinate relations are not copied into that payload.

ADR-0024 remains authoritative for semantic reference requests. Command
parameter types use the same public lifecycle with a new precise role and
source family.

ADR-0025 remains authoritative for `References` validation. This ADR adds only
the exact Command endpoint matrix defined above and does not reopen rejected UI
wildcards.

ADR-0028 remains authoritative for Attribute and TabularSection payload.
Form/Command multilingual synonyms do not silently extend that payload.

## Rejected alternatives

1. Recreate Form and Command nodes during Sprint 7. Rejected because current
   production evidence already proves declaration, identity, ownership,
   provenance, Query, Validation, and Coverage.
2. Implement the complete conceptual UI taxonomy in one sprint. Rejected
   because `Form.form` hierarchy, identity, bindings, events, and endpoint
   contracts remain unresolved.
3. Treat every `OpenForm` spelling as a resolved navigation fact. Rejected
   because the corpus contains dynamic, shorthand, default, unsupported, and
   malformed forms.
4. Store a form-opening call as generic `References`. Rejected because opening
   is a distinct direct behavior fact and generic reference loses that meaning.
5. Store both `Opens` and a companion `References` or `DependsOn` edge.
   Rejected for the first slice because Query and Impact can classify `Opens`
   directly without duplicating the fact.
6. Emit `Command --Opens--> Form`. Rejected because the complete source fact is
   performed by a containing Procedure; Command ownership remains available by
   reverse `Contains` navigation.
7. Add `Command --Executes--> Procedure` now. Rejected because module ownership
   and handler naming do not yet prove a complete runtime-trigger contract.
8. Resolve Forms globally by name. Rejected because equal Form names under
   different metadata owners are valid and owner scope is canonical.
9. Create placeholder Forms for unresolved, generated, or default targets.
   Rejected because no accepted placeholder identity or lifecycle exists.
10. Reuse Attribute/TabularSection optional synonym payload for Forms and
    Commands. Rejected because the real corpus contains multilingual values and
    no locale collection or selection contract is accepted.
11. Model every `<group>` value as a Command Group reference. Rejected because
    the source also contains platform placement tokens and the current model
    has no complete Command Group contract.
12. Parse all `Form.form` elements during this slice. Rejected because the
    smallest module/reference/navigation capability does not require internal
    UI identity or bindings.

## Deferred scope

- `Form.form` elements, attributes, commands, events, data paths, captions, and
  action bindings;
- Command Group entities, hierarchy, categories, and placement references;
- multilingual subordinate Form and Command payload;
- default, list, object, selection, generated, dynamic, and expression-based
  Form targets;
- form opening outside recognized Command-module Procedures;
- `Executes`, `Handles`, `Displays`, `BindsTo`, `InvokesCommand`, or other
  conceptual UI relations;
- unresolved/external Form nodes;
- new parameter target families including Defined Types;
- form-derived generic References or DependsOn;
- Designer XML ingestion and cross-adapter conformance;
- runtime UI state or proof of actual execution.

## Ordered implementation prerequisites

1. Extend the source-independent graph model with `Opens`, exact endpoint
   validation, dependency/usage/Impact classification, module-owner additions,
   and precise Command reference/dependency endpoints.
2. Extend EDT source parsing for subordinate Form modules, Common/subordinate
   Command modules, and typed command parameter observations without graph
   emission.
3. Emit accepted Module nodes, ownership, BSL declarations, and existing BSL
   semantic facts through the production builder.
4. Convert accepted Command parameter observations through the public request
   lifecycle and emit resolved `References` and `DependsOn` projections.
5. Implement a typed complete-statement static `OpenForm` candidate extractor
   for accepted Command-module Procedures without graph emission.
6. Resolve accepted Form targets and emit canonical provenance-backed `Opens`
   edges with typed failure outcomes.
7. Complete representative production evidence, Diff, incremental equivalence,
   Query, Impact, diagnostics, reports, determinism, and independent Coverage
   transitions; synchronize current-state documentation only after the tests
   pass.
8. Perform an independent Sprint 7 integration review before changing Sprint
   status or making Sprint 8 eligible for planning.

## Consequences

- Existing Form and Command identities and ownership remain stable.
- Form and Command executable code can join the existing BSL semantic graph
  without a parallel UI-specific symbol model.
- Command parameter metadata types gain precise request, reference, dependency,
  diagnostic, and provenance behavior.
- Consumers can query direct resolved form-opening navigation without parsing
  source strings or conflating it with Calls or References.
- The first slice remains small enough to validate deterministically through a
  real EDT production path.
- Rich Form internals, Command Groups, dynamic targets, and runtime execution
  semantics remain explicit later decisions.
- Architecture acceptance alone changes neither production behavior nor
  Coverage status.
