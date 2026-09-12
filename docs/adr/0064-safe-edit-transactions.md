# ADR-0064: Safe Edit Transactions

## Status

Accepted for Sprint 41 architecture; controlled-unwind implementation evidence
qualifies `45147cf1649e9ca8315feb52c02a9936df0fa1f9` after separate design pass
`19f9f23b3851e5b24f781e6c00b160fc706fe8d3` on architecture
`d328d8638bab12c5ebc8fe2591d21d615be115a8`. Fresh independent and primary
integration gates remain pending. This decision does not claim a supported edit
capability. It extends [ADR-0063](0063-refactoring-planner.md) only at the local
Runtime mutation boundary. The [investigation](../architecture/safe-edit-transactions-investigation.md)
records the original prerequisite; the [current evidence](../architecture/safe-edit-transactions-evidence.md#current-controlled-unwind-evidence-and-review-handoff)
reconciles all 35 invariants and executed owner-local oracles.

The controlled-unwind amendment below is accepted as the renewed architecture
decision. Its prerequisite was the separate committed
`Approve Sprint 41 controlled unwind recovery design` gate on the exact unique
`Define Sprint 41 controlled unwind recovery` commit; the IDs above satisfy it.
Earlier design decisions remain valid only for their recorded ranges.

## Context and accepted scope

A complete plan is immutable semantic evidence, not edit authorization. Current
Runtime startup, successor rebuild and shutdown are the three snapshot writers;
cache stability checks alone do not guard publication. Operation versions omit
untouched modules, metadata and discovery inputs. Generic Tool Policy evidence
does not authenticate a Workspace service lifetime, and its cancellation path
may drop executor work. Those gaps must be closed together.

Accept exactly `bsl_callable_rename_v1`, one existing callable in one complete
Configuration in one local Workspace, for EDT and Designer XML. Apply replaces
every planned declaration/local-call/qualified-call identifier. Reversal restores
the exact previous bytes, after separate confirmation. Both use a full production
Workspace rebuild and one checked successor publication. No source format,
Graph semantics, wire endpoint, UI, dependency, or existing API removal is added.

## Decision

### Owners and dependency direction

| Owner | Accepted responsibility |
|---|---|
| `crates/analysis/src/refactoring.rs` | Existing immutable source/request/plan contracts and pure planner remain canonical. |
| New `crates/analysis/src/safe_edit.rs`, exported by `crates/analysis/src/lib.rs` | Additive pure exact-plan comparison, replacement/range transformation, typed `SafeEditProducerProjection` admission and complete semantic postcondition evaluation over borrowed before/after evidence. Owns complete mapping consumption and bounds; does not encode adapter provenance. No disk, policy, service, cache or publication writer. |
| Graph query and BSL identity owners already used by Analysis | Sole identity, ownership, edge, callable-name and resolution authority. Validators use public queries and existing constructors, not new graph facts or duplicated ID formats. |
| EDT/Designer adapters and `WorkspaceSnapshotBuilder` | Complete production discovery/capture/build/validation, including reference ledger, rules and diagnostics. Adapter semantics stay unchanged. |
| New `apps/runtime/src/workspace/edit.rs` | Service-bound API, bounded attempt/undo state, policy gate, ordered transaction state machine and results; integrates into `workspace/mod.rs`. |
| New `apps/runtime/src/workspace/edit_io.rs` | Private confined observation, staged replacement, original-byte backups, checked restoration and cleanup. Standard library I/O only; injected test seams use this same production algorithm. |
| `apps/runtime/src/workspace/mod.rs` | Single lifecycle coordinator for startup, all rebuilds, transactions, cache publication decisions, recovery quarantine and shutdown. Owns the snapshot sender. |
| `crates/tool-policy/src/confirmation.rs` and `execution.rs` | Existing exact binding, one-use confirmation and policy execution gate. Runtime composes these unchanged; no transaction/domain dependency enters Tool Policy. |
| `apps/runtime/src/workspace/cache.rs` | Existing rebuildable cache owner, never a source journal or backup store. |
| New `adapters/designer-xml/src/safe_edit.rs` and `adapters/edt/src/safe_edit.rs`, exported by their existing `lib.rs` | Pure `project_safe_edit_provenance` APIs produce expected typed evidence from the complete before publication and exact replacement bytes, before staging or candidate build. Encoding stays with shared adapter producers. |
| `adapters/designer-xml/src/semantic_graph.rs` | Shared canonical module/declaration provenance helpers used by both builder and projector; recompute whole-module SHA and all declarations in changed modules. |
| `adapters/edt/src/bsl_graph.rs` and `query_source_resolution.rs` | Shared captured-byte analysis and canonical Query, request, edge and diagnostic context producers; unchanged public builder entry points delegate to them. |
| `crates/bsl/src/queries.rs`, exported by `crates/bsl/src/lib.rs` | Extract public `bsl_query_id(owner_id: &EntityId, binding_name: &str) -> Result<EntityId, BslQueryError>` from private `query_id`; both extractor and projector reuse exactly the existing encoding. |

Direction remains Graph/BSL/common -> Analysis -> adapters -> Runtime. Tool
Policy is consumed by Runtime. No new production dependency edge or package is
required. Runtime borrows before/after evidence into Analysis and orchestrates
I/O; it must not implement a second rename planner or semantic validator.

### Local API, authorization and lifetime

The additive public surface is exported only through `apps/runtime/src/lib.rs`:

- `WorkspaceService::with_edit_policy(ToolPolicy, WorkspaceEditOwnership)`
  enables edits before service start. Default services remain edit-disabled.
  `WorkspaceEditOwnership::ExclusiveCooperative` is the caller's explicit
  commitment to exclude other writers, linked aliases and path swaps throughout
  an attempt/recovery; it is not an acquired OS lock. The policy is immutable
  for this service lifetime; policy replacement requires a new service.
- `WorkspaceService::edit_handle()` returns a cloneable
  `WorkspaceEditHandle`, with no authority before successful startup.
- `prepare_apply(RefactoringRequest, ActorId, ToolRequestId)` obtains the exact
  current production plan and returns a non-cloneable `WorkspaceEditChallenge`
  plus the existing bounded structured preview. No caller-supplied edits or
  path/source arguments are accepted.
- `prepare_reversal(WorkspaceEditReceipt, ActorId, ToolRequestId)` prepares the
  one retained successful apply's exact inverse. The receipt is opaque,
  non-cloneable and not authorization. Preparation consumes it even if denied.
- `WorkspaceEditChallenge::confirm(self)` records explicit caller confirmation
  and yields a non-cloneable `WorkspaceEditAuthorization`. Confirmation does
  not write files. Dropping either value releases its reservation.
- `checked_apply(WorkspaceEditAuthorization, cancellation)` and
  `checked_reversal(WorkspaceEditAuthorization, cancellation)` submit to the
  owning service and await `WorkspaceEditOutcome`; direction mismatch fails.
  Success carries old/new canonical publication IDs, plan ID, counts and, for
  apply only, the opaque reversal receipt. No automatic redo/history API exists.

These are async handle operations except configuration, handle acquisition and
confirmation. The new cloneable `WorkspaceEditCancellation::new()` /
`request()` token uses existing Tokio watch primitives, independently of the
response future's lifetime; the coordinator also observes service `Cancellation`.
The existing service cancellation observer has no public request constructor
and is not repurposed. Capability types have private fields; no public
constructor, deserializer, clone, integer or snapshot can forge a capability.

Every challenge/authorization binds a private `Arc` service identity checked by
pointer identity, a checked monotonic attempt number, direction, exact current
snapshot `Arc`, publication ID, complete structured plan, complete source
baseline and immutable policy evaluation. A live foreign service at publication
1 cannot match. The attempt number is not another publication sequence. It
never wraps or resets inside one service; exhaustion disables new edits.
One service retains at most one prepared, queued or running attempt. Accepting
submission irreversibly consumes its slot identity before revalidation; failure
cannot replay it. Releasing a slot never makes its attempt number reusable.

Runtime constructs a bounded `ToolRequest` with exactly `LocalMutation` and
internal tool identity `oneagent.workspace.edit.apply` or
`oneagent.workspace.edit.reverse`. These identities are policy inputs, not MCP
catalog additions. Canonical length-delimited arguments bind the attempt,
direction, publication, Configuration, target and full plan ID; structured
equality is also checked privately, so hashes are not authentication. Actor,
request ID, policy revision, effects and exact argument bytes are bound by
existing Tool Policy. Only `RequireConfirmation` is eligible; `Deny` and bare
`Allow` fail closed (`PolicyDenied` / `ConfirmationRequired`).

The challenge wraps the original `ToolAuthorization` and its unique
`ToolConfirmationChallenge`. Submission consumes the matching confirmation via
`execute_tool`. Its executor is an immediate, side-effect-free admission gate;
it never stages, writes, queues or spawns mutation. Only a completed, confirmed
gate allows the coordinator to start the transaction. Generic cancellation may
drop this gate safely. The service-owned transaction is subsequently joined
independently; it is never the generic executor future.

Preparation regenerates through `WorkspaceSnapshot::plan_refactoring`; submission
regenerates again from that same still-current snapshot. Compare complete request,
target, preconditions, ordered operations (including paths, raw ranges, tokens,
versions, IDs and replacements), dependencies, completeness and summary. Equal
IDs with unequal structures are `PlanMismatch`, including injected forged plans.
Neither a preview nor an old snapshot can authorize edits. Challenges expire on
drop, submission, any successor publication, recovery quarantine or service
stop; there is no time-based or cross-process credential.

### Complete publication baseline and bounds before retention

Runtime privately associates each transaction-eligible publication with a
complete stable source baseline. It records canonical root/Configuration roots,
format/discovery identity, every directory and entry kind, and exact regular-file
bytes beneath the Workspace. This stronger transaction scan must not reuse
`WorkspaceFileState::scan` unchanged: that scan skips root `.oneagent` and ignored
directory descendants, while discovery does not exclude `.oneagent`.

For transaction eligibility, scan all descendants, including ordinarily ignored
directories. Exclude only the exact verified Runtime cache file
`.oneagent/cache/workspace-v1.json` and the exact
current attempt's owned temporary files; never exclude a whole directory by
name. A cache-file exclusion is allowed only for the actual reserved cache path
with a regular confined entry and no source/discovery role. Unknown `.oneagent`
contents remain inputs. Initialize the existing cache namespace before capturing
an edit-eligible baseline; its directory markers then remain ordinary baseline
entries. Cache initialization/maintenance must finish before the before/after
capture, and later cache writes must not create an unobserved directory. The
coordinator excludes concurrent cache temporary-file activity during source
scans, and a leftover cache temporary file is an ordinary observed input, not
silently erased transaction evidence. A source document or discovered root missing from the
baseline makes the publication ineligible. Conservative extra inputs may cause
staleness; hidden semantic inputs may not be ignored. Custom detector results
must satisfy the same coverage and confinement proof or edits are unavailable.

Capture before and after complete build (or validated cache acceptance), require
equal inventories/bytes and agreement with all captured documents and discovered
roots. Retain that baseline with the publication, never substitute a later scan
of equal touched files. Existing read-only services/builders retain their APIs;
a publication with unprovable/over-bound edit baseline is still read-only and
cannot prepare a transaction. Unstable builds must not publish a candidate as
transaction-eligible. For an edit-enabled service, all accepted successor builds
must pass stable before/after observation; failure retains the previous Arc and
consumes no publication ID. Initial instability fails edit-enabled startup.

Transaction-specific inclusive bounds are deliberately narrower than planning:

| Resource | Maximum and admission point |
|---|---|
| Concurrent outstanding attempts | 1 total prepared/queued/running; reserve with nonblocking admission before plan construction, scan or retained payload. Further requests return `Busy`, no unbounded waiting queue. |
| Retained reversal | 1 successful apply, shared with its original/result buffers; expire before admitting a subsequent apply and on any other successor publication, stop or quarantine. |
| Operations / touched regular files | 4,096 / 64; checked before constructing replacement buffers or staging. One Configuration and target remain mandatory. |
| Each original and resulting edited document | 1,048,576 bytes; calculate checked output length before allocation. |
| Aggregate original / aggregate resulting edited bytes | 8,388,608 bytes each, checked before retaining copies; rollback reserve is mandatory. |
| Complete baseline entries / each relative path / aggregate path bytes | 16,384 / 4,096 / 4,194,304; admit each directory entry before collecting/sorting or retaining a path. |
| Each baseline regular file / all baseline regular-file bytes | 8,388,608 / 67,108,864; check metadata and remaining allowance before reading, use a limited read and detect one extra byte without retaining an unbounded allocation. |
| Additional transaction-owned raw/projection buffers | 268,435,456 bytes across baseline, verification scans, original/result/recovery copies and new projection-owned records/strings/scratch; reserve checked capacities before allocation/retention, share immutable unchanged bytes, release scans promptly. |
| Temporary disk bytes / files | 16,777,216 / 128 total for result and backup files; reserve before `create_new`. Restoration reuses each backup slot. |

These are per-service transaction retention limits, not a bound on arbitrary
reader-held snapshots, the existing builder/Graph heap, or total process memory.
Every existing planner/adapter limit still applies. No rejected payload is
retained in an error. A complete rebuild remains subject to existing production
admission; no larger corpus or performance claim follows. Exact-limit and
one-over tests cover each row, checked arithmetic and release on every terminal
path. Inspection scratch space and directory enumeration must also be bounded.

### Single coordinator and guard ordering

The `WorkspaceService` lifecycle/update loop is the sole coordinator; an I/O
worker may run blocking work but cannot publish. Startup readiness, watcher
rebuild, explicit Git change-input rebuild, transaction and shutdown all use it.
No transaction begins while a previously admitted build/cache write is running.
No build, cache write or clearing writer races the transaction. Watcher signals
coalesce while busy; explicit input retains its existing bounded admission.

The normative order for apply is:

1. Reserve the sole attempt slot before retaining preparation input; validate
   edit enablement, owner/liveness, identifiers, current publication and bounds.
   Generate exact complete production plan, acquire/verify its complete stable
   publication baseline and path eligibility, reserve all future byte/operation
   budgets, then issue the exact policy confirmation challenge. No disk writes.
2. On submission consume the attempt capability. Under the coordinator recheck
   live owner, direction, policy confirmation, cancellation, current predecessor
   Arc/ID, regenerated complete plan equality, full baseline equality and path
   ownership. Check the next publication increment before any file mutation.
3. Construct exact result bytes by applying each document's operations in
   descending raw-offset order, checking versions, bounds, nonoverlap and expected
   tokens. Reserve/check result and recovery budgets before allocation. Freeze
   the complete producer-owned expected projection below before staging, source
   writes or candidate rebuild; projection errors are pre-write rejections. Preserve
   every byte outside those ranges, including BOM, CRLF/LF and trailing newline.
4. Validate roots/ancestors/targets, permissions and aliases; create all private
   staged result and backup files with `create_new`, bounded names and no source
   extensions. Write fully, preserve standard permissions, `sync_all`, close and
   read back exact bytes. All original backups must be verified before the first
   replacement. Any staging failure cleans owned artifacts, writes no source.
5. Immediately before the first replacement repeat complete baseline and all
   source/path guards. For each document in canonical confined-path order,
   recheck that original bytes, kind and alias identity match, then rename the
   verified same-parent result over the target. Never pre-delete/truncate the
   target or fall back to copy-on-error. Record each attempted replacement and
   classify its observed result, including ambiguous I/O failure, before recovery.
6. Scan the complete actual tree: its only difference must be the exact result
   byte set (excluding owned temporary files). Build the complete Workspace with
   `WorkspaceSnapshotBuilder`, Designer `Complete`, graph validation, reference
   ledger, rules and diagnostics. Repeat complete scan after build and evaluate
   every semantic postcondition below. No transaction cache write occurs yet.
7. Check cancellation, predecessor Arc, checked successor ID and expected full
   source state again. Compose adjacent impact with the existing owner. Remove
   all owned staged/backup files before commit; the pre-reserved in-memory
   originals still permit recovery. Cleanup failure triggers recovery. Perform
   the final expected-source/path scan after cleanup and check cancellation.
8. Prepare all fallible outcome/undo construction before publication, retaining
   the attempt and I/O owner until the last precommit guard. The single
   `snapshot.send_replace(Some(successor))` is the commit point.
   Publish the new private baseline and outcome in the same coordinator turn,
   increment the existing publication counter once, retain one reversal record,
   and invalidate other preparation evidence. There is no fallible source I/O
   or cancellation-to-rollback transition after this point.
9. Write the cache only for this accepted successor while still serialized;
   cache failure is an observable cache-status failure, never rollback of a
   committed source transaction. Deliver success after this bounded cache
   attempt. Process coalesced observations by comparing actual current source
   to the new baseline: self-write-only noise consumes no extra publication;
   real external changes or explicit change input follow normal rebuild rules.

The old snapshot remains immutable and available to retained readers throughout.
During an active transaction observers may still obtain the previous snapshot;
it is historical analysis evidence, not a claim that intermediate disk bytes
match. No reader sees an intermediate candidate. Multiple Configuration roots
may be observed, but only one may be mutated; all others must remain exactly
equivalent and participate in complete freshness and rebuild validation.

### Filesystem ownership, staging and guarantees

The supported first mutation platform has standard-library same-directory
replacement and Unix file identity/link-count inspection (macOS/Linux). Other
platforms remain read-only until equivalent guards are proven; fail
`ConfinementUnverifiable` before writes. No platform package is added.

`edit_io` validates the canonical Workspace and Configuration roots and each
component through the target with `symlink_metadata`, rejecting symlinks,
nonregular targets, lexical escapes and a canonical path outside its root.
Distinct target paths must have distinct file identities; hard-linked targets
(`nlink != 1`) fail, including aliases outside the Workspace. Source evidence
paths must map one-to-one to these files. Recheck identities and path kinds at
each read/replacement/restoration/cleanup, not only during planning. The root
and directory hierarchy are covered by the cooperative ownership assumption.

Create bounded sibling names owned by a checked attempt/file ordinal, using
`create_new`; collisions fail without deleting the existing entry. Track exact
created paths and identities. Do not sweep a prefix or reuse arbitrary files.
No blanket ignored-directory rule may conceal these artifacts: only this
attempt's verified entries are excluded from expected-tree comparison.
Backup and staged bytes are private; temporary permissions must not broaden
access to source bytes. Standard `Permissions` are retained on replacement and
restoration. Exact timestamps, inode identity, ACLs, extended attributes, file
flags and other richer metadata preservation are not promised; deployments
requiring them must keep edits disabled.

The caller excludes external writers for the entire operation and recovery.
Portable path checks cannot close a hostile path-swap TOCTOU race or acquire
cross-process exclusion. Observed external changes fail closed, but a write
after the last check is outside the guarantee. Each successful rename replaces
one file; readers of several files may observe a mixed set. The semantic
publication is atomic; the multi-file filesystem operation is not. `sync_all`
and read-back are checked I/O steps, not a power-loss durability promise.
Process kill, OS crash and power loss can leave mixed sources or temporary
files; no durable journal, automatic restart recovery or cross-process undo is
claimed. Never interpret leftover files as authorization on restart.

### Complete semantic oracle

Source equality and parsing success alone are insufficient. Analysis evaluates
all Configuration graphs and complete source evidence using public semantic
queries; Runtime also requires successful production reference/rule/diagnostic
composition. Comparison is exhaustive, not limited to planned operations or
node/edge counts. Define the expected transformation from the original plan:

1. File inventory, Configuration IDs/roots/formats, Module identities/owners,
   document IDs/roles/paths/completeness and all untouched bytes remain equal.
   Changed document bytes are exactly the raw-range replacements. No metadata
   edit, root addition/removal or source-evidence omission is permitted.
2. The old target disappears and exactly the BSL-owned expected post-rename
   Node ID appears with the same kind, Module owner, export/async/parameter and
   other payload attributes except the accepted name/identity. The sole dependent
   identity closure is the selected callable's directly owned Query nodes,
   derived by `bsl_query_id` with unchanged exact binding. Query text and payload
   remain equal. No other node is added, removed or changes payload. For every
   Graph edge compare its kind, endpoints and complete provenance after this
   bounded identity closure and the producer projection below; derive
   edge IDs through the Graph owner. This covers incoming/outgoing ownership
   and calls, including calls originating inside the renamed callable.
3. Compare every source occurrence in canonical document/range/kind order.
   Apply the exact cumulative byte-length delta map to all later ranges; the
   replaced range takes replacement length. Planned occurrences use the exact
   desired token and resolve uniquely to the new target. Every other token,
   kind, lexical owner and resolution remains equal except references to the
   renamed target ID and mechanically shifted source positions. No extra,
   missing, ambiguous or unsupported occurrence is accepted as a renamed call.
   Designer does not gain unsupported Graph `Calls` edges: its complete source
   occurrence mappings are mandatory call evidence independently of edges.
4. Source locations/provenance and reference-ledger anchors are recomputed from
   exact before/result bytes using the same source-location owners (including
   BOM and Unicode scalar columns). Compare full records after this explicit
   mapping, using canonical producer-owned expected source identities/contexts,
   preserving source kind/role and unaffected fields. All reference
   requests and their resolved/unresolved/unsupported dispositions must match
   after the admitted callable/Query identity and name/range transformation; equal aggregate statistics alone
   are insufficient. No unresolved record may silently disappear.
5. Graph validation, built-in rules and diagnostic composition must complete.
   Compare the full diagnostic/rule evidence after the same target and anchor
   mapping: rule/code, severity, parameters, related evidence, completeness and
   omission counts. Identity or rendered message fields derived from those
   inputs must be regenerated with their canonical owners, never discarded or
   normalized by blanket string replacement. Any unclassifiable difference,
   new/lost unrelated diagnostic or incomplete report rejects the candidate.
   Pre-existing diagnostics are allowed only under this exact equivalence;
   requiring a previously clean project is not a substitute for the oracle.
6. Other Configurations are exactly unchanged. Change impact is freshly derived
   from the actual adjacent pair through `compose_change_impact`; its previous
   and current IDs must be this committed pair. Undo compares against the saved
   original semantic projection; publication and adjacent-impact IDs are the
   only lifecycle fields intentionally different from that old snapshot.

The range/identity transformation is a pure comparison projection, not a new
Graph, inference engine or producer of facts. A field that cannot be proven
equivalent fails closed; implementation must not weaken this definition to
parsing, selected lookups, full-graph counts or “no extra errors”. Task 3 must
map each complete-oracle component to its actual owner and negative test before
design review permits implementation.

### Producer-owned expected projection

The user-approved producer-projection correction supersedes target-only identity
and mechanical provenance substitution. It preserves eligibility, including
callables containing resolved, unresolved, malformed or unsupported Query text
already admitted by the production builder. It does not change write, recovery,
undo, coordinator or authorization mechanisms. Earlier complete T01-T35
representability/constructor/type evidence remains required.

Both adapters expose the same additive pure signature:
`project_safe_edit_provenance(input: SafeEditProducerInput<'_>, admission: &mut SafeEditProjectionAdmission) -> Result<SafeEditProducerProjection, SafeEditError>`.
The new Analysis types have private storage and checked public construction.
`SafeEditProducerInput` borrows the complete before Configuration evidence,
complete plan and the complete ordered document replacement list, including
unchanged documents by borrow. Each `SafeEditModuleInput` binds document/module/
owner IDs, module name, format/role, canonical captured `SourcePath`, original
bytes and exact replacement bytes. Runtime supplies paths from the retained
confined baseline, not caller input. Analysis validates input completeness,
path/document correspondence and exact plan replacement before adapter dispatch.
The input has no candidate, filesystem handle, callback to a candidate, transport
argument or authority to reread source. One projector handles one Configuration;
Runtime combines results for the complete Workspace, retaining exact unchanged
evidence for other Configurations.

`SafeEditProducerProjection` binds the exact before publication/Configuration,
plan structure and original/result document versions, not hashes alone. It
contains these typed mappings, canonicalized by the **before** keys:

| Mapping / key (all scoped by Configuration ID) | Required value and complete domain |
|---|---|
| `NodeIdentity { before: EntityId }` | Expected `EntityId` for exactly the selected callable and each directly owned Query. No unrelated or identity-equal entry is allowed. Both domains/ranges must be unique and disjoint from all unchanged node IDs. |
| `NodeFact { before: EntityId }` | Full ordered expected provenance list for every before node, including unchanged nodes and all declarations in a changed Designer module. Payload/name/kind comparison remains independently constrained by the semantic oracle. |
| `EdgeFact { before_source: EntityId, kind: EdgeKind, before_target: EntityId }` | Full expected provenance list for every before edge. Expected endpoints use only `NodeIdentity`; Graph derives the edge ID. No opaque EdgeId decoding. |
| `RequestFact { before: SemanticReferenceRequestId }` | Full canonical expected terminal request, including source, category, typed reference, expected kinds, candidates, state, outcome and complete provenance. Expected request IDs are derived by the canonical producer/`reconstruct_terminal`. |
| `DiagnosticFact { before_ordinal: usize }` | Full expected `SemanticDiagnostic` for every record in the before snapshot's canonical diagnostic order; the key also binds that exact full before record by immutable borrow. An ordinal alone cannot identify a diagnostic in another publication. |
| `QueryFact { before: EntityId }` | Captured before/expected owner, exact binding and exact query text plus canonical source projection for every format-supported producer Query, independently reconciled with the complete before Graph Query inventory. EDT includes malformed/unsupported Query text because its Query/Contains emission precedes query-language parsing. Designer's current producer emits no Query facts: its QueryFact and dependent Query closure are empty, even when unchanged BSL text contains Query. This adds no Designer Graph/query semantics; exact bytes still prove unchanged text. |

Fact keys are an enum with these variants, not concatenated strings or hashes.
Provenance values compare every source ID, optional location, producer, origin,
confidence and resolution field. Each value is typed `Unchanged` or
`Expected(T)`: `Unchanged` explicitly resolves its checked fact key against the
same retained before publication when compared, while `Expected(T)` owns its
prepaid result. The frozen projection stores no borrowing pointer into a scoped
parser result and needs no self-referential Runtime structure. The coordinator
retains the before Arc separately and Analysis checks its binding on use. Duplicate
keys, duplicate expected identities, conflicting mappings, omitted identities
or facts, extra facts and version/publication/plan mismatch reject the projection.
Canonical sorting must not deduplicate an invalid input into apparent success.
Analysis derives the complete key inventory from before evidence independently
of projector output, and requires equality with that inventory before staging.
Each mapping is consumed exactly once while assembling expected collections;
unused or reused entries reject. Candidate collections are then compared in
their own canonical order against those frozen expected collections, including
full provenance explicitly rather than `GraphEdge::eq`. Candidate ordering or
counts cannot supply missing keys, align diagnostics or repair a projection.

For the identity closure, Query membership is proven through Graph `Contains`
with exactly one owner equal to the selected callable and the before canonical
BSL extraction. Re-run the same BSL producer on replacement bytes; pair Query
by before owner and exact binding, never by a string prefix or encounter order.
Verify identical Query count/binding/text, no changed unrelated Query, and no
new collision. Call `bsl_query_id` for old and expected IDs and require old ID
agreement too. The allowed closure has no recursive descendants, metadata or
other-callable nodes. For every edge/request/diagnostic use these exact mapped
IDs; Reads/DependsOn targets, candidate sets and terminal outcomes remain equal.
Extra/removed Query, changed text/binding/payload or another callable's equal
binding is a negative case, not a reason to drop Query-containing targets.

Designer shares `module_source_id` and `declaration_provenance` helpers within
`semantic_graph.rs`, extracted from `emit_module_and_declarations`/`source_id`.
They take captured path, exact raw bytes and existing typed module/declaration
facts. Builder and projector call the same helpers. The whole-module digest
changes every dependent declaration provenance in each changed module; unchanged
declarations therefore receive their canonical new source IDs too. Unrelated
metadata/Configuration provenance is exact. Analysis never formats this encoding.

EDT extracts pure `analyze_captured_module` in `bsl_graph.rs` with typed module
ID/name/path, borrowed raw bytes and the existing occurrence limit; the current
file-reading `analyze_module` wrappers delegate after capture. The projector
uses this same pure analyzer for before and exact result bytes. Share existing
`query_context`, `query_provenance`, `query_request_projection_provenance`,
`query_diagnostic_provenance`, and the collection/resolver context builders in
`query_source_resolution.rs`; no new context encoder exists in Analysis or
Runtime. Query resolution uses the complete before metadata index, whose exact
unchanged inventory is checked, and production Complete scope. It preserves
all collected/terminal ledger records and pre-existing parser diagnostics,
including unsupported/malformed Query cases with no successful Reads edge.
Do not recover typed inputs by parsing private formatted provenance strings.

The projector reproduces the before-side producer evidence and checks it against
the captured publication before accepting expected mappings. This prevents
silently trusting an incompatible baseline. Expected diagnostics use canonical
producer constructors with the mapped typed fields, then existing Analysis
validation/rule/diagnostic owners derive complete expected reports from these
expected facts. This pure computation is a comparison projection, never a new
published Graph or inference rule. Outcomes, codes, severities, unsupported
dispositions, unchanged messages/fields and report statuses remain independently
constrained; only canonically dependent IDs/anchors/messages may transform.
Neither the candidate nor its rebuilt diagnostics are their own expected value.

### Projection admission arithmetic and freeze point

`SafeEditProjectionAdmission` shares the attempt's existing 268,435,456-byte
additional-buffer reservation; it does not add another independent allowance.
Charge newly owned projection strings, mapping/record storage and scratch as
well as raw bytes. Existing borrowed snapshots are not copied or charged again;
the existing exclusion of canonical builder/parser/Graph internal heap from a
total-process-memory guarantee remains. Any parser result retained by the new
projection is charged before copying/retaining it, never retrospectively.

Let `R` be already reserved attempt bytes. For every new vector of type `T`
and admitted capacity `n`, reserve `n.checked_mul(size_of::<T>())`; for every
owned string/raw slice reserve its UTF-8/raw capacity, including duplicated
copies; for every nested vector/boxed allocation add its separate storage.
Enum/record inline fields are included once in `size_of`, nested allocations
are not. `P = sum(checked capacities)` covers all concurrent projection output,
canonical key inventory, consumption bitset (`(fact_count + 7) / 8` with checked
addition), sorting/index arrays, context-formatting scratch and before/result
producer records. Require checked `R + P <= 268435456` before allocation or
retention. An allocation failure is a closed pre-write error. Do not use hidden
`collect`, `format!`, clone, implicit Vec growth or growing maps outside this
reservation. Reserve vector capacity explicitly; account for granted capacity
before retaining/using it, failing and releasing an unexpected over-allocation.

The budget remains a transaction-owned retention bound, not a new bound on
existing parser internals. The pure captured analyzer may run the existing
canonical extractor/parser work under its existing admission and scoped internal
working lifetime. Its existing internal token/scope/AST/source-map allocations
remain in the previously excluded parser heap; do not create extra copies,
retain them in the attempt, or silently reclassify new projection allocations as
parser internals. In particular this correction introduces no universal
raw-byte-times-identity scratch reserve or smaller effective document-size cap.
The existing exact 1,048,576-byte simple-rename positive must remain admissible
when the actual transaction-owned reservations fit.

There are three separate phases. First, canonical parsing owns its scoped
internal result; it is not yet retained producer output or a transaction buffer.
Second, the projection count pass visits borrowed before facts, exact result
bytes and those scoped parser results without allocation. It must not invoke
another parser/extractor, build formatted contexts, clone records, collect or
sort. It counts output elements and every nested byte length with the shared
canonical encoder's counting sink. Third, only after A/E grants the exact
checked capacity lease may the producer emit/retain new output, key arrays,
strings, Query text/maps or projection-specific scratch. Release the parser
working result when its borrowed visit/emission finishes. Anything moved or
copied out of parser internals into retained producer output is charged before
that retention; no uncharged parser result may escape in EditAttempt/EditUndo.

Count inventory and consumed-bitset sizes directly from existing complete
before collections. Use two borrowed passes instead of a temporary collecting
map; sort only the pre-reserved emitted arrays. All new producer context encoders
write first to a nonallocating counting `fmt::Write` sink and then to the
already leased buffer, using
the exact same canonical encoder, so the expected context length is computed
without allocating its string. In particular length-prefix digit counts and
UTF-8 bytes come from that producer, not an Analysis copy of the encoding.
Keep at most one module's new raw/context scratch concurrently; release its
lease before processing the next. Every additional output buffer owned by the
projector uses this counting/emitting reservation path. If a vector/string must
grow, reserve the full new capacity while the old capacity remains charged;
release the old lease only after replacement/deallocation. No implicit growth
may borrow space already reserved for originals or frozen expected evidence.
Checked arithmetic overflow,
one-over capacity, missing counts or output exceeding the measured reservation
reject before staging. Tests instrument reservation before each new allocation
and retention boundary, including nested provenance/request/diagnostic strings,
sorting scratch, Query text and partial-construction error cleanup.

Freeze the validated complete projection in `EditAttempt` after exact result
bytes exist and before `stage_all`. Bind it to the same consumed attempt and
predecessor; no candidate access or post-write regeneration can replace it.
F3 candidate fault injection runs only after the production rebuild with this
expected projection already immutable. Reversal retains exact original evidence
in the existing `EditUndo` reservation and compares a rebuilt original state to
it; no candidate-derived expected undo. Drop, denial, cancellation, commit,
undo expiry and quarantine release or transfer each lease exactly once, while
recovery retains its previously reserved originals. New failures stay redacted.

### Recovery, failure precedence and reversal

Before source mutation, failure returns a closed cause with zero source writes
and no publication increment. After any attempted replacement, every failure
before commit (including cancellation, cleanup, build, oracle or final scan)
enters recovery owned by the coordinator. Restore attempted files in reverse
replacement order only if current bytes/identity match this attempt's exact
expected result, or prove they already equal the original. Never overwrite an
unrelated change, remove an unknown entry or treat an I/O error as proof that a
rename did nothing. Stage/verify original bytes and replace using the same
confined algorithm; record all restoration and cleanup outcomes.

Successful recovery requires exact complete original source-state equality,
original permissions and removal of every owned temporary file. It retains the
old publication, consumes no ID, returns the original failure with
`Recovered`, and creates no undo receipt. Rollback is byte restoration rather
than semantic guessing; the old full validated snapshot remains the oracle.

If restoration, final original-state verification or cleanup fails, return
`RecoveryRequired` ahead of the triggering error, with that error retained only
as a closed secondary cause. Enter a poisoned state, clear current observation
to `None`, invalidate all capabilities/undo, disable all subsequent edits,
rebuild publications and cache writes, and retain bounded recovery material.
Retained old Arcs stay immutable but are not current. No automatic retry or
“last good matches disk” claim is allowed. The only exit is service stop plus
operator repair and a new cold/validated service; there is no force-accept or
automatic recovery API in this slice. Preserve unremovable owned artifacts on
stop and report their count/closed status; in-memory evidence ends with process
lifetime. Do not log bytes or absolute recovery paths.

Closed pre-write precedence is: unavailable/stopped/poisoned/busy; input/bound;
owner/capability/direction; policy/confirmation; cancellation; publication/plan;
complete source/path; staging. Within an ordered phase the first failure wins;
recovery failure always overrides it. Useful closed causes include
`Unavailable`, `Stopped`, `Busy`, `BoundsExceeded`, `AuthorizationMismatch`,
`PolicyDenied`, `ConfirmationRequired`, `Cancelled`, `PublicationMismatch`,
`PlanMismatch`, `SourceChanged`, `ConfinementUnverifiable`, `IoFailed`,
`SemanticMismatch` and `RecoveryRequired`. This is an additive Runtime result
vocabulary, not a change to existing planner or MCP failures.

Reversal requires the same service and exactly the successful apply's current
successor Arc/ID, complete result baseline and fresh policy confirmation. It
uses saved originals/results and the original complete semantic evidence; it
does not reverse-engineer an undo from current text. All apply stages, limits,
path/alias guards, cleanup and complete production rebuild apply symmetrically.
Success publishes one new successor containing exact original bytes/semantics;
it never reuses the original publication ID. Failure recovers to the applied
state, or poisons the service on failed recovery. A submitted reversal consumes
the undo record on every outcome; there is no replay or automatic retry.

### Controlled transaction-owner unwind

This correction was planned against the historical implementation at
`38a9bde3407f151e2c17b380e8bd28252c5a39f9`. At that input,
`EditCoordinator::run_attempt` retains its optional `EditIo` only in a local
variable and finalizes `Result::Err`; `execute` and `abandon_commit` do not
contain unwind. `run_workspace_updates` moves the coordinator into a joined
worker and maps join failure to `SnapshotUnavailable`, which cannot recover
the lost owner. It also calls `commit` to construct undo/outcome after
`send_replace`. These source facts justify the new retained-owner design;
they are not an executed post-write panic reproduction.

The minimum correction stays private to Runtime `workspace/edit.rs`,
`edit_io.rs` and `mod.rs`. A service-owned transaction envelope retains the
coordinator, attempt/reservation, response sender, explicit phase, optional
I/O owner and prepared terminal/commit material outside each unwind boundary.
Workers borrow that state for narrow synchronous operations. Ownership may move
into a joined worker only when its normal return carries the envelope and
terminal/commit state back; no ordinary controlled unwind may escape to a
join-error mapping that drops them. The snapshot sender stays in the lifecycle
loop. Neither response drop nor cancellation owns or destroys this envelope.

Contain preparation (including reversal receipt consumption), submission
revalidation, precommit staging/replacement, complete build/comparison and
fallible outcome/undo preparation in explicit synchronous boundaries. Retain
the optional I/O owner before its first creating operation. A boundary returns
either its normal result or a closed phase failure; both enter one shared
finalizer. Do not wrap the whole async lifecycle, continue a partially executed
phase or retry the panicking operation. Each `AssertUnwindSafe`, if required,
must identify its captured mutable owners and explain why their partial state
is usable only by finalization: exclusive access, prepaid retained buffers,
immediate created-entry registration, marker-before-replace ordering, immutable
predecessor/projection and no publication inside the boundary. No user callback,
mutex guard or inferred poisoned invariant may be blindly asserted safe.

On successful `create_new`, register a present entry immediately, before any
fallible identity check or test callback; reserve registration capacity before
the create. Unknown identity remains counted but never grants exclusion,
cleanup, restoration or replacement authority. Mark attempted replacement
before calling rename; record successful/ambiguous stage, restore and removal
observations before any subsequent injectable boundary. Recovery interprets
partial state through checked observation, never by assuming a failed or
unwound call performed no mutation. Exact retained counts must reconcile known
successful removal/rename observations; unknown present entries stay retained.

With no attempted replacement, the finalizer verifies original source/tree
state and owned staging cleanup before reporting `NotNeeded` and zero retained
files. After any attempted replacement it invokes checked reverse-order recovery,
including backup recreation after precommit cleanup removed the old backup.
`abandon_commit`, apply and reversal use this same finalizer. Recovery borrows
the retained I/O owner inside its own narrow unwind boundary; an error or
unwind there stops recovery without retry and quarantines. Return
`RecoveryRequired`/`Required`, the original closed trigger as secondary and the
exact retained artifact count; expire preparation/undo, clear observation and
forbid edit/rebuild/cache publication. Preserve bounded material through stop.
Successful recovery requires the full original tree/bytes/permissions and no
owned artifacts, retains the old publication and returns `Recovered`. Reversal
uses its applied predecessor as the restoration oracle, with identical rules.

Reuse existing closed causes without adding public variants: preparation or
admission unwind maps to `Unavailable`; production build/comparison/impact
unwind to `SemanticMismatch`; confined I/O or precommit material-construction
unwind to `IoFailed`. These phase mappings describe containment failure, not
an asserted OS error. Normal `Err` precedence stays unchanged; a contained
unwind fixes its phase cause before recovery, and later cancellation cannot
replace it. Recovery failure always takes precedence. Opaque panic payloads
are never formatted or retained in outcomes, Debug, audit or transaction logs.
No process-global panic hook is changed: the Rust hook can run before catch,
so this contract does not promise redaction of hook output or stderr.

Finish allocating/cloning outcome, receipt and undo material before the last
precommit guard. Publication remains exactly `send_replace(Some(successor))`;
the same non-awaiting coordinator turn installs prepared baseline/undo/status
by ownership transfer. After publication there is no rollback path, fallible
source I/O or reconstructive outcome step. The retained success survives
postcommit cache failure/unwind, response drop and stop. Cache status alone
records cache failure; terminal delivery follows its bounded attempt. Shutdown
closes admission and joins both mutation and recovery before releasing the
slot/owner or clearing a healthy observation. A blocked recovery therefore
blocks completion of stop, even with a dropped response.

Controlled Rust unwind is an in-process guarantee. `panic=abort`, a secondary
panic during unwinding/destruction, process kill, OS crash and power loss remain
excluded; no durable recovery follows. Existing confinement, cooperative-writer,
memory/disk bounds and complete semantic obligations are unchanged.

### Cancellation, shutdown and redaction

Cancellation before the first replacement cleans staging and writes no source;
after a replacement it requests recovery and cannot interrupt recovery. A
dropped response future does not cancel ownership: the service completes or
recovers and retains terminal status until the attempt slot is released.
Shutdown closes admission, invalidates preparation, signals active work, joins
the blocking worker and recovery, and only then performs the clearing writer.
If commit already occurred, success remains success and shutdown only stops the
service. A stalled OS I/O call may delay shutdown; no bounded wall-clock or
hard-kill recovery guarantee is made. No detached mutation task survives the
coordinator.

Debug, errors, audit and logs contain closed causes, phase, counts and safe
canonical IDs only. Raw original/result/staged/backup bytes, expected tokens,
content digests, policy argument payloads and absolute paths are redacted,
including nested I/O/build errors. The existing intentionally bounded preview
is the only display projection. Recovery status must not dump sensitive data.
No new telemetry collection or persisted source-bearing history is added.
This redaction promise applies to transaction-controlled output; process-global
panic-hook output is outside it, as specified in the controlled-unwind boundary.

## Compatibility and affected consumers

The existing `WorkspacePublicationId` remains canonical;
`ChangeImpactPublicationId` remains its compatible alias. No new publication
counter, cache schema or semantic-cache version is needed: the private edit
baseline/capabilities/undo never persist. Fresh service IDs still start at 1.
Old snapshots/builders remain read-only and immutable; existing planner output
continues to have no edit authority.

Affected composition/verification consumers are `apps/runtime/src/main.rs`,
`apps/runtime/src/bin/oneagent-mcp.rs`, `apps/runtime/src/bin/oneagent-lsp.rs`,
`apps/runtime/src/mcp_tools.rs`, Workspace GraphQuery observers, Runtime
`workspace_service`, `file_watching`, `persistent_cache`, `git_change_workspace`,
`mcp_process`, `mcp_semantic_tools`, `lsp_stdio` tests and Designer paired
conformance tests. Existing consumers need no source migration: configuration
is additive and defaults edit-disabled; no product entry point enables it in
this sprint. New direct Rust tests are the first callers. The eight-tool MCP
catalog, all protocol revisions, CLI/HTTP/LSP/IDE surfaces, diagnostics/rule
semantics and source Coverage Registry claims remain unchanged.

## Rejected alternatives

- A catch only around `spawn_blocking`/the whole moved coordinator loses
  recovery and response ownership before classification; `Result`-only cleanup
  omits controlled unwind. Both fail the retained-envelope requirement.
- Catching each I/O call and blindly continuing/retrying can overwrite an
  unrelated change or reuse partially invalid state. Shared checked finalization
  and quarantine are required; cleanup in `Drop` is not a recovery protocol.
- Constructing undo/outcome after publication leaves a fallible committed tail;
  rollback after publication would contradict the accepted successor. Prepare
  material first and retain committed success through cache/stop handling.
- Global panic-hook replacement, public detector injection, a new public panic
  variant, weaker semantic comparisons or a new recovery dependency expand this
  correction without resolving transaction ownership.
- Snapshot/client writes or a separate write-only mutex fail to serialize
  startup, watcher/rebuild, cache and shutdown publication paths.
- Trusting a PlanId, numeric publication ID, caller-supplied operations or
  generic confirmation alone fails structured equivalence and service binding.
- Operation-only freshness or two new equal scans misses changes since the
  publication and semantically relevant untouched sources.
- In-place truncate/write, pre-deleting source before rename, or borrowing the
  cache replacement routine exposes avoidable partial files and unsafe recovery.
- Parsing-only, count-only or planned-occurrence-only checks cannot prove
  unaffected semantic evidence or detect newly omitted references.
- Running mutation inside a cancellable generic ToolExecutor can abandon
  restoration; publishing before validation makes invalid disk changes visible
  as accepted semantic state.
- Durable journals, cross-process locks, OS-specific directory-handle APIs and
  third-party filesystem packages expand the approved first slice. Cooperative
  ownership and honest failure/crash limits are accepted instead.

## Implementation prerequisites and deferred scope

The new unique architecture subject and separately committed targeted pass in
Status supersede historical producer-only admission for this correction.
Remap all T01-T35 under the retained envelope, preserve their R/L/C/T split and
require owner-local tests at actual production boundaries. A custom detector
is L evidence: `WorkspaceService::with_builder` and its builder field are private;
public `WorkspaceSnapshotBuilder::with_detector` does not expose service
injection. Require paired EDT/Designer positive apply/reversal controls and a
post-write build unwind, recovery unwind, backup recreation, retained/dropped
response, stop while recovery is blocked, replay/lease expiration, quarantine
and postcommit-success oracles. Earlier guards must execute before fault seams.
No default-input panic trigger is established. Current approved owner-local
post-write unwind oracles are qualified by the evidence above; the historical
safety-rejected exploratory probe was never compiled or executed.

Task 3 must map every applicable guard and retention point above to planned
production symbols and deterministic negative oracles. Task 4 must independently
accept that committed design before Task 5 changes code. Tests must include EDT
LF and Designer BOM/CRLF exact bytes, a production-buildable multi-file variant,
every I/O ordinal (including restoration/cleanup), full baseline staleness,
foreign/replayed authorization, exact/one-over bounds, all semantic comparison
components, watcher/cache/publication/shutdown barriers and immutable old Arcs.
The existing paired corpus alone is not multi-file or complete Runtime proof.

No Supported claim or Coverage Registry update is authorized from this ADR.
Completion requires real production guards, tests, integration review and the
canonical validation matrix. Other families, metadata/path/file renames,
multi-Configuration/cross-Workspace edits, Git/remote mutation, model-generated
edits, new protocol/UI surfaces, cross-process history/recovery, broader platform
guarantees, benchmarks, Sprint 42 and release execution remain deferred.
