# ADR-0022: Writes Semantics

## Status

Accepted

## Context

Semantic Model 2.0 declares `EdgeKind::Writes`, generic dependency and usage
navigation already classifies it as a dependency edge, and Impact Analysis
already traverses it through the existing dependency policy. The EDT adapter
does not emit Writes, however, and graph validation currently accepts every
Writes endpoint pair through a broad permissive branch. The repository needs a
source and endpoint contract before persistent mutation can be represented
without conflating a method spelling, receiver inference, authorization,
generic calls, references, reads, or normalized dependencies.

Repository-owned BSL contains persistent register operations in
`OneAgent_EDTproject/src/Documents/RefundOfPaymentByOrder/ObjectModule.bsl`:

```bsl
Procedure Posting(Cancel, PostingMode)
    // ...
    RegisterRecords.CashAccountBalance.Write();
    RegisterRecords.RefundBankPayment.Write();
    // ...
EndProcedure
```

The owning descriptor
`OneAgent_EDTproject/src/Documents/RefundOfPaymentByOrder/RefundOfPaymentByOrder.mdo`
declares both targets as document register records:

```xml
<registerRecords>AccumulationRegister.CashAccountBalance</registerRecords>
<registerRecords>AccumulationRegister.RefundBankPayment</registerRecords>
```

Both top-level Accumulation Register descriptors also exist. This combination
provides static evidence for a narrow platform-defined receiver shape, an
owning Document, an exact metadata namespace and name, and a persistent target.

The same repository proves that the final method name is not sufficient:

- `BinaryData.Write(TempFileName)` writes a file in
  `OneAgent_EDTproject/src/CommonForms/EditSpreadsheetDocument/Module.bsl`;
- `TextDocument.Write(ListFileName)` writes text in
  `OneAgent_EDTproject/src/CommonForms/CheckUpdateFile/Module.bsl`;
- `Archiver.Write()` writes an archive in
  `OneAgent_EDTproject/src/WebServices/Exchange/Module.bsl`;
- `Form.Write()` invokes UI behavior in
  `OneAgent_EDTproject/src/Reports/FinancialReport/Commands/CheckReportKind/CommandModule.bsl`;
- `ProductSale.Write(DocumentWriteMode.Posting)` in
  `OneAgent_EDTproject/src/DataProcessors/WorkplaceForSales/Forms/Form/Module.bsl`
  follows local object creation and requires value/type-flow evidence before a
  metadata target can be proven;
- metadata object variables such as `PredefinedItem.Write()` require similar
  local binding evidence.

The existing `BslCall` model retains the containing procedure or function, the
qualified target spelling, and a one-based source line. It does not preserve
receiver type, arguments, source ranges, or general local value flow. The
current EDT metadata domain model also does not preserve document
`<registerRecords>` declarations. Those limitations define implementation
prerequisites; they do not justify guessing.

ADR-0021 explicitly leaves Writes to a separate architecture task. ADR-0019
establishes that write authorization and write behavior are independent facts.

## Problem

Without a precise contract, a future producer could incorrectly:

- emit Writes for every qualified call ending in `.Write(...)`;
- infer a persistent target from a variable name or method name;
- use a role right as evidence of runtime mutation;
- derive Writes from Reads, References, Calls, or DependsOn;
- accept arbitrary procedure, function, module, Query, or metadata endpoints;
- resolve a register name without proving that the owning Document declares it;
- emit a partial or lower-confidence fact from malformed or unsupported syntax;
- create placeholder, Unknown, file, external, or dynamic graph targets;
- store duplicate edges or nondeterministically ordered provenance;
- close `semantic_edge.writes` using architecture documentation alone.

## Decision

`EdgeKind::Writes` represents a direct, resolved persistent-data mutation fact
from the semantic code entity that performs an accepted write operation to the
top-level metadata object whose stored records may be changed by that operation.

Canonical direction:

```text
writer --Writes--> persistent mutation target
```

The smallest production slice is:

```text
NodeKind::Procedure
    --Writes-->
NodeKind::Metadata(MetadataKind::AccumulationRegister)
```

The source must be an existing Procedure node declared in the Object Module of
an EDT Document. The source statement must be one complete, standalone,
zero-argument call with the exact first-slice shape:

```bsl
RegisterRecords.<RegisterName>.Write();
```

Whitespace and a terminal semicolon do not affect classification. Comments,
strings, additional expressions, aliases, localized spellings, chained
receivers, calls through another object, and non-empty argument lists are not
accepted by the first slice.

The owning Document descriptor must declare exactly
`AccumulationRegister.<RegisterName>` in `<registerRecords>`, and exactly one
existing top-level `NodeKind::Metadata(MetadataKind::AccumulationRegister)`
node with that normalized name must resolve. All requirements are conjunctive.
Failure of any requirement produces no Writes edge.

## Canonical semantic definition

`Procedure A --Writes--> Metadata B` means that the complete accepted source
statement in Procedure `A` invokes the platform Document register-records
collection for register `B`, the owning Document explicitly declares `B` as an
Accumulation Register record target, and semantic resolution identified exactly
one compatible persistent metadata node.

Writes is a direct resolved static behavior fact. It does not prove that the
procedure executed, that a transaction committed, that every possible control
flow reaches the statement, or that a particular record was inserted, updated,
or deleted. It is not a textual mention, authorization fact, reverse impact
relation, transitive dependency, or inferred receiver type.

The phrase “may be changed” covers the platform operation's persistent mutation
effect without attempting record-level operation classification. `Clear()` is
not a Writes source in this slice because the selected production fact is the
subsequent accepted `Write()` operation.

## Source and target matrix

### Canonical endpoint matrix

| Source contract | Source kind | Target kind | First slice |
|---|---|---|---|
| Accepted Document Object Module register-record write | `NodeKind::Procedure` | `NodeKind::Metadata(MetadataKind::AccumulationRegister)` | Yes |
| Same shape inside a function | `NodeKind::Function` | Accumulation Register metadata | Deferred |
| Same shape outside a Document Object Module | Procedure or Function | Accumulation Register metadata | No |
| Accepted-looking shape targeting another declared register family | `NodeKind::Procedure` | Information, Accounting, or Calculation Register metadata | Deferred |
| Object, manager, local variable, form, Query, or module write | Any other source kind | Any metadata kind | No |
| Any source | Any metadata member, flat semantic, Unknown, external, file, or dynamic node | No |

`NodeKind::Metadata(MetadataKind::Unknown)` is never a persistent mutation
target. The first-slice allowlist contains exactly
`MetadataKind::AccumulationRegister`; it is not shorthand for every metadata
kind accepted by EDT.

### Evidence-backed examples

The first production fixture may use either or both occurrences in
`OneAgent_EDTproject/src/Documents/RefundOfPaymentByOrder/ObjectModule.bsl:76-78`.
Their containing source is Procedure `Posting`; their owning Document descriptor
declares the matching Accumulation Registers; and the corresponding top-level
metadata descriptors exist.

Additional `RegisterRecords.<Name>.Write()` occurrences in Document Object
Modules confirm that the shape is not unique to one file. They do not expand
the accepted metadata-kind allowlist. For example,
`HistoryOfCustomsClearanceStatuses.Write(True)` targets an Information Register
and has an argument, so it remains deferred even though the owning descriptor
declares that register.

## Supported first-slice source contract

An occurrence is accepted only when all of the following are proven:

1. EDT module discovery identifies the source artifact as the Object Module of
   a discovered top-level Document metadata object.
2. Existing BSL declaration extraction identifies a containing Procedure and
   the corresponding `NodeKind::Procedure` node exists.
3. A Writes-specific extractor or parser consumes a complete standalone call
   statement rather than matching a substring or suffix.
4. The callee components are exactly `RegisterRecords`, one valid local
   register identifier, and `Write`, with no leading or trailing receiver
   components.
5. The call contains zero arguments and has no unparsed expression remainder.
6. The owning Document descriptor contains exactly one compatible typed
   `<registerRecords>AccumulationRegister.<RegisterName></registerRecords>`
   declaration after deterministic normalization.
7. Metadata resolution finds exactly one compatible existing top-level graph
   target of the allowlisted kind.

The exact `RegisterRecords` root is platform context, not inferred local type
flow. The descriptor declaration supplies the persistent metadata kind and
name. Neither fact is sufficient without the other.

Current `BslCall` output may be reused as a candidate locator because it keeps
the containing symbol, qualified spelling, and source line. It is not complete
proof because it does not preserve the argument list or complete statement
shape. A later implementation must add a dedicated typed extraction result or
an equivalently conservative parser without changing the public `BslCall` API
unless a separate API task justifies that change.

## Extraction and parsing completeness

The first-slice parser need not be a full BSL parser, but it must be lexical and
structure-aware enough to prove the whole accepted statement. Regular-expression
or suffix-only matching is insufficient because comments, strings, nested
expressions, qualified identifiers, and multiple statements can contain the
same text.

The minimum typed candidate contains:

- source module identity and source artifact;
- containing procedure identity or unambiguous containing procedure name;
- raw receiver and method spelling;
- normalized register identifier;
- verified zero-argument classification;
- deterministic one-based source location, with a range if the parser provides
  one;
- complete-statement classification;
- owning Document identity.

Extraction must distinguish accepted candidates from recognized but rejected
write-like statements. A parser error, unsupported construct, incomplete
statement, absent containing scope, or unproven module ownership produces a
typed outcome and no candidate eligible for resolution.

The first slice is per-statement, not all-or-nothing per procedure. An
unsupported `.Write(...)` statement does not invalidate another independently
complete accepted `RegisterRecords.<Name>.Write()` statement in the same
procedure. Each accepted candidate must nevertheless be complete; no fragment
of one statement may produce an edge.

## Persistent and non-persistent boundaries

| Source category | First-slice classification | Writes result |
|---|---|---|
| Exact standalone `RegisterRecords.<Name>.Write()` in a Document Object Module Procedure | Persistent candidate | Resolve descriptor declaration and target; emit on unique success |
| Same shape with arguments | Unsupported first-slice arguments | Typed diagnostic; no edge |
| `Recorder.RegisterRecords.<Name>.Write()` or another chained root | Unsupported receiver shape | Typed diagnostic; no edge |
| Local metadata object such as `ProductSale.Write(...)` | Requires value/type flow | Typed unresolved or unsupported receiver diagnostic; no edge |
| File, binary, text, archive, stream, or temporary-data write | Non-persistent external effect | Typed non-persistent or unsupported receiver diagnostic; no edge |
| Form or UI write | UI behavior | Typed unsupported receiver diagnostic; no edge |
| External component or unknown receiver write | External or unresolved behavior | Typed unresolved receiver diagnostic; no edge |
| Bare `Write(...)` or localized/aliased spelling | Insufficient evidence | Typed unsupported shape diagnostic; no edge |
| String or comment containing `.Write(` | Not a call | No candidate and no edge |
| Dynamic member or computed receiver | Dynamic evidence | Typed dynamic-target diagnostic; no edge |

The method spelling `.Write(...)` alone is never evidence of persistent
metadata mutation. A target is not accepted merely because a same-named
metadata object exists.

## Document register declaration extraction

The later EDT implementation must preserve document `<registerRecords>` entries
as typed source evidence before Writes resolution. Each declaration needs at
least:

- owning Document identity;
- exact metadata namespace and local name;
- normalized `MetadataKind` when the namespace is supported;
- descriptor provenance and a deterministic member location or source context;
- typed malformed, duplicate, and unsupported-kind outcomes.

This information may remain private to the EDT contribution pipeline. This ADR
does not require a public metadata-domain API or a new graph node for a register
declaration. Duplicate equivalent declarations are deduplicated
deterministically; conflicting normalized declarations are ambiguous and do not
authorize a Writes edge.

## Target resolution rules

Resolution runs only after Document register declarations and all top-level
metadata nodes are available.

For each complete candidate:

1. Identify the owning Document from EDT module ownership, never from path text
   alone in the semantic contract.
2. Match the candidate register identifier against that Document's typed
   register-record declarations using the repository's established
   deterministic identifier normalization policy.
3. Require exactly one matching declaration of
   `MetadataKind::AccumulationRegister`.
4. Resolve among existing graph nodes of exactly
   `NodeKind::Metadata(MetadataKind::AccumulationRegister)` using the normalized
   local name.
5. Emit Writes only when exactly one compatible graph node exists.

Display synonyms, historical names, UUID guesses, localization aliases, and
case-sensitive filesystem assumptions must not participate in resolution.
Case-insensitive language matching must not hide a normalization collision.

Failure policy:

| Condition | Required result |
|---|---|
| Owning Document is missing or ambiguous | Typed owner diagnostic; no edge |
| No matching `<registerRecords>` declaration | Typed undeclared-register diagnostic; no edge |
| Declaration has a deferred metadata kind | Typed unsupported-target-kind diagnostic; no edge |
| Multiple normalized compatible declarations | Typed ambiguous-declaration diagnostic; no edge |
| Missing metadata graph target | Typed missing-target diagnostic; no edge |
| Multiple compatible metadata targets | Typed ambiguous-target diagnostic with deterministically ordered candidates; no edge |
| Same name exists only at an incompatible kind | Typed incompatible-target-kind diagnostic; no edge |
| External, partial-workspace, or dynamically selected target | Typed unresolved-target diagnostic; no edge |
| Missing Procedure source node | Graph-construction invariant failure; no edge |

Missing, ambiguous, incompatible, external, unsupported, dynamic, or malformed
candidates must not create placeholder, Unknown, file, or external graph nodes.

## Direct versus derived semantics

Writes stores the direct resolved mutation fact supported by the accepted call
and Document declaration. The emitted edge uses:

```text
FactOrigin::Resolved
ResolutionState::Resolved
Confidence::Exact
```

The edge is not derived from the owner's metadata type, another edge, an access
right, or a reverse index. A future normalized `DependsOn` edge may use Writes
as evidence only under a separately accepted dependency-source contract. It
must not replace the Writes edge.

## Identity, provenance, duplicates, and determinism

Writes uses the standard graph edge identity:

```text
(source_node_id, target_node_id, EdgeKind::Writes)
```

Receiver spelling, source position, argument formatting, declaration order,
provenance, and insertion order are not part of edge identity.

Every emitted Writes edge must carry deterministic provenance sufficient to
explain both syntactic classification and persistent target resolution. The
minimum context is:

- source Procedure node ID and owning module ID;
- owning Document node ID;
- Object Module source artifact and exact candidate source location;
- raw and normalized register identifier;
- owning descriptor source and matching typed `<registerRecords>` declaration;
- resolved target node ID and `MetadataKind::AccumulationRegister`;
- stable extractor/parser, declaration-reader, resolver, and graph-contributor
  producer stages;
- resolved origin, resolved state, and exact confidence.

The current provenance model may encode source-specific locations and
resolution context in deterministic source identifiers. This ADR does not
require a new public source-range API.

Repeated accepted statements that resolve to the same Procedure and target
support one canonical edge. Every distinct occurrence and matching declaration
may contribute provenance, but equivalent records are sorted and deduplicated
before insertion. The producer must aggregate provenance before inserting the
edge; duplicate edge identity alone is not a provenance merge mechanism.

Given identical inputs, candidate ordering, declaration ordering, diagnostics,
resolution outcomes, edge identity, provenance ordering, graph snapshots, and
build reports must be identical. Filesystem order, XML traversal order, source
discovery order, hash-map order, and candidate encounter order must not affect
the result. Repeated builds must produce empty graph and build-result diffs.

## Validation constraints

The later validation task must replace the broad Writes branch with exactly:

```text
NodeKind::Procedure
    --Writes-->
NodeKind::Metadata(MetadataKind::AccumulationRegister)
```

The graph validator validates the canonical endpoint contract, not the EDT
source syntax. The EDT producer remains responsible for proving Object Module
ownership, exact call shape, descriptor membership, and resolution.

The validator must reject:

- every non-Procedure source kind;
- metadata targets outside `MetadataKind::AccumulationRegister`;
- metadata members, flat semantic, Unknown, file, external, or placeholder
  targets;
- missing endpoints;
- edges without provenance under the existing provenance invariant.

A physical self-loop is impossible for valid endpoint kinds. Existing missing
endpoint, provenance, and general graph invariants remain authoritative.

## Typed diagnostics

Later extraction and resolution tasks must expose deterministic typed outcomes.
At minimum they must distinguish:

- malformed or incomplete write statement;
- unsupported write statement shape;
- unsupported or unresolved receiver;
- non-persistent receiver category when statically known;
- unsupported non-empty arguments;
- missing containing symbol;
- unsupported containing symbol kind;
- unsupported module or metadata owner kind;
- missing or ambiguous owning Document;
- missing matching document register declaration;
- ambiguous document register declaration;
- unsupported declared metadata kind;
- dynamic target;
- missing metadata target;
- ambiguous metadata target;
- incompatible metadata target kind.

Diagnostics must identify the source artifact, source location, containing
symbol when known, raw candidate, and relevant declaration or target context.
Ambiguous candidates and diagnostics sort deterministically. Diagnostics do not
create graph endpoints or lower-confidence Writes edges.

The implementation may classify obviously non-persistent receivers separately
from unresolved receivers when static lexical evidence supports that category.
It must not invent receiver types merely to improve diagnostic wording.

## Relationship to other edges

| Edge | Meaning | Boundary from Writes |
|---|---|---|
| `Writes` | Direct resolved persistent mutation from the accepted Procedure statement | Canonical edge for this fact |
| `Reads` | Direct resolved persistent data access | A read does not imply a write; Writes is not derived from Query sources |
| `Grants` | Explicit authorization from a Role to a scoped AccessRight | Role rights never prove mutation behavior and Writes never proves authorization |
| `References` | Generic resolved semantic reference | Does not express mutation and is not emitted as a companion by the first slice |
| `DependsOn` | Materialized normalized direct semantic dependency | No companion edge until a separate source contract is accepted |
| `Contains` | Structural ownership | Document/module/procedure ownership remains unchanged and does not imply mutation |
| `Calls` | Resolved invocation between callable declarations | The platform write operation is not a Procedure/Function target and no Calls edge is implied |

The current call extractor may observe the qualified spelling, but a resolved or
unresolved generic call is not Writes evidence. The first Writes producer emits
no companion Calls, References, or DependsOn edge and does not change existing
call-resolution behavior.

## Reconciliation with existing architecture

ADR-0006 remains unchanged. Writes is a typed directed edge, both endpoints
must exist, provenance is required, and graph construction is deterministic.

ADR-0008 remains unchanged. BSL and EDT extraction contribute controlled facts
to the source-independent graph; the graph crate does not parse BSL or EDT; and
unresolved knowledge remains explicit rather than becoming a resolved edge.

ADR-0013 remains authoritative for generic BSL call extraction. Its current
model is a possible candidate locator, not complete Writes evidence. This ADR
does not change `BslCall` or generic Calls semantics.

ADR-0017 remains authoritative for normalized DependsOn semantics. This ADR
does not authorize call-derived, write-derived, or query-derived DependsOn.

ADR-0019 remains authoritative for Grants. Write-related access rights and
write behavior remain separate facts emitted from separate source evidence.

ADR-0021 remains authoritative for Reads. Writes is not inferred by negating,
mirroring, or extending the accepted Query Reads slice. Query-language mutation
and query-derived dependency semantics remain deferred.

## Dependency-query and Impact Analysis policy

The existing Query API and Impact Analysis policy remain unchanged.

Writes continues to participate in dependency and usage classification:

- outgoing Writes from a Procedure is a direct dependency on the persistent
  metadata target;
- incoming Writes to metadata is direct mutation usage by the Procedure;
- generic edge-kind queries expose the stored edge without a dedicated API.

Impact Analysis continues to traverse dependency edges in reverse from a
changed Accumulation Register to directly affected Procedure nodes. Existing
bounded traversal and optional ownership propagation determine transitive
results. No reverse impact edge, transitive Writes edge, new Query method,
weight, score, execution probability, or risk rank is stored by this slice.

## Coverage Registry completion criteria

`semantic_edge.writes` remains `DeclaredOnly` after this architecture task.
Architecture acceptance adds no EDT production evidence and does not change
Coverage counts or priorities.

The capability may transition to `Supported` only after production evidence
proves the accepted first slice:

- `EdgeKind::Writes` remains declared;
- this accepted contract exists;
- repository-owned positive and negative source fixtures exist;
- Document `<registerRecords>` declarations are preserved as typed evidence
  with deterministic provenance and diagnostics;
- a typed Writes-specific extractor or parser proves complete first-slice
  statements and rejects unsupported forms;
- exact owning-Document declaration matching exists;
- exact metadata kind-and-name resolution exists without placeholders;
- the broad Writes validation branch is replaced by the precise endpoint rule;
- the EDT production graph path emits resolved Writes edges from existing
  Procedure nodes without changing declaration identity or ownership;
- duplicate provenance aggregation and deterministic diagnostic ordering are
  proven;
- focused positive tests cover both repository-owned selected targets;
- negative tests cover non-persistent receivers, object writes that need type
  flow, arguments, wrong module/owner/source kinds, undeclared registers,
  unsupported target kinds, and missing, ambiguous, or incompatible targets;
- Query tests prove outgoing dependency, incoming usage, and generic edge-kind
  navigation;
- Impact tests prove the existing reverse dependency policy without storing
  reverse or transitive edges;
- full EDT builder integration proves emission, provenance, validation, and
  absence of companion Writes for rejected calls;
- repeated-build tests prove stable nodes, edges, provenance, diagnostics,
  graph diff, and build-result diff;
- a final registry-only change adds all required production evidence, changes
  only `semantic_edge.writes` to `Supported`, removes exactly its High gap, and
  updates aggregate expectations deterministically.

Until every item is complete, the capability remains `DeclaredOnly`. Sprint 3
Integration Review remains blocked by this High gap.

## Rejected alternatives

1. Every `.Write(...)` call emits Writes. Rejected because repository-owned
   file, text, archive, UI, external, and unresolved object calls share that
   spelling.
2. Every qualified call ending in `.Write` emits Writes. Rejected because a
   qualified name does not provide receiver type or persistent target identity.
3. Local variable names or nearby constructors are enough to infer the target.
   Rejected because the current model has no general value/type-flow contract.
4. `ProductSale.Write(DocumentWriteMode.Posting)` is the first slice. Rejected
   because it requires local creation/binding propagation and Document target
   resolution not preserved by `BslCall`.
5. Every `RegisterRecords.<Name>.Write(...)` shape is accepted. Rejected because
   arguments and register families have different unaccepted semantics, and
   the smallest evidence-backed slice uses zero arguments and Accumulation
   Registers.
6. Syntax alone resolves `RegisterRecords.<Name>`. Rejected because the owning
   Document descriptor provides the authoritative register family and
   membership evidence.
7. Document declaration alone emits Writes. Rejected because a possible record
   target does not prove that a Procedure performs a write.
8. Functions, modules, Queries, and arbitrary metadata nodes are accepted as
   source endpoints immediately. Rejected because the selected evidence is a
   Procedure and broad endpoints would weaken validation.
9. Information, Accounting, and Calculation Registers are included now.
   Rejected because they are not required for the smallest selected fixture and
   require their own positive source and argument-semantics evidence.
10. Writes is inferred from Reads, References, Calls, or DependsOn. Rejected
    because those edges encode different direct facts.
11. Writes is inferred from Grants or role rights. Rejected because
    authorization does not prove code behavior.
12. Placeholder or Unknown targets preserve unresolved writes. Rejected because
    the graph has no accepted placeholder contract and unresolved diagnostics
    preserve uncertainty without a false resolved edge.
13. Architecture acceptance changes Coverage to `Supported`. Rejected because
    no EDT producer, precise validation, integration test, or production
    provenance exists yet.

## Deferred scope and unknowns

The following remain deferred:

- local value/type-flow analysis for `Object.Write(...)` and object variables;
- Document, Catalog, Business Process, Task, and other object persistence;
- Information, Accounting, and Calculation Register record-set writes;
- `Write(True)` and other argument-bearing register operations;
- `Recorder.RegisterRecords...`, aliases, returned receivers, chained
  expressions, and dynamic members;
- localized receiver and method spellings until repository-owned lexical
  evidence and normalization rules exist;
- manager methods and common-module wrappers that may mutate registers;
- query-language mutation, data modification statements, and Query Writes;
- record-level insert/update/delete classification and transaction outcomes;
- field-level or register-member write edges;
- inferred, probabilistic, external, runtime-traced, or cross-workspace targets;
- write-derived References or DependsOn companion edges;
- changes to Calls diagnostics for platform operations;
- a public receiver-type, source-range, register-declaration, or Writes-candidate
  API.

Remaining unknown behavior includes the complete set of BSL write forms, the
semantics of argument-bearing record-set writes across register families,
localized spellings, aliasing, and the minimum reusable value-flow model for
object persistence. None of these unknowns weakens the selected first slice;
they must remain unresolved instead of being guessed.

## Implementation prerequisites and ordered follow-up tasks

1. Create a focused repository-owned Writes source corpus that preserves the
   selected `RefundOfPaymentByOrder` positive statements, their Document
   register declarations, both target descriptors, and representative binary,
   text, archive, UI, object-variable, argument-bearing, wrong-owner, malformed,
   missing, ambiguous, and incompatible negative forms. Record expected typed
   outcomes; do not emit graph edges.
2. Extend the private EDT Document descriptor path to preserve typed
   `<registerRecords>` declarations, provenance, normalization, duplicates, and
   malformed/unsupported-kind diagnostics. Keep graph behavior and Coverage
   unchanged.
3. Implement a typed Writes candidate extractor or conservative parser for the
   exact complete standalone zero-argument statement. Preserve containing
   Procedure and source location; reject strings, comments, partial statements,
   extra receiver components, arguments, dynamic forms, and unsupported
   spellings. Do not resolve targets or emit edges.
4. Implement private resolution that joins a candidate to its owning Document,
   matches exactly one allowlisted register declaration, then resolves exactly
   one existing Accumulation Register metadata node. Add deterministic typed
   missing, ambiguous, incompatible, unsupported, and unresolved outcomes; do
   not emit edges.
5. Replace only the broad Writes validator branch with the precise
   Procedure-to-Accumulation-Register matrix and focused positive/negative
   validator tests. Do not emit Writes and do not change Coverage.
6. Emit canonical provenance-backed Writes edges through the EDT production
   graph path, aggregating duplicate occurrence and declaration evidence before
   insertion. Add focused producer tests while keeping Coverage
   `DeclaredOnly`.
7. Add full-builder integration, Query, Impact, negative, diagnostic-ordering,
   duplicate, and repeated-build evidence using repository-owned fixtures.
   Confirm that other `.Write(...)` calls emit no Writes edge and that existing
   Calls, Reads, References, DependsOn, Contains, and Grants behavior is
   unchanged.
8. Transition only `semantic_edge.writes` to `Supported` in a final
   registry-only task after all required evidence exists. Update exact EDT and
   combined Coverage counts and unblock Sprint 3 Integration Review only then.
9. Define each deferred register family, object persistence, value/type-flow,
   query mutation, or write-derived DependsOn origin through a separate
   evidence-backed architecture extension before implementation.

## Consequences

### Positive

- Writes has one canonical meaning, direction, and precise first endpoint
  matrix.
- The first implementation can be deterministic without general receiver type
  inference.
- Persistent mutation is proven by independent source syntax, owning Document
  declaration, and exact graph resolution evidence.
- Non-persistent and unresolved `.Write(...)` calls remain explicit negative
  outcomes rather than false graph facts.
- Architecture, validation, production evidence, and Coverage transition remain
  independently reviewable.

### Negative

- The first slice intentionally omits many real persistent object and register
  writes.
- EDT must preserve a new private typed Document declaration source before
  emission is possible.
- A Writes-specific complete-statement extractor is required even though the
  generic call extractor sees the qualified spelling.
- Procedure-only validation will require an explicit architecture extension
  before Functions or Query sources can emit Writes.

### Neutral

- `EdgeKind`, `NodeKind`, `MetadataKind`, graph identity, Query APIs, Impact
  policy, and existing edge behavior do not change.
- `semantic_edge.writes` remains `DeclaredOnly`; EDT Coverage remains 1 High and
  43 Medium gaps, combined Coverage remains 0 Critical, 1 High, and 44 Medium
  gaps, and Sprint 3 Integration Review remains blocked.
