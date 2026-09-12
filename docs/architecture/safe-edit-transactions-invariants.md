# Safe Edit Transaction Invariants

Accepted design: [ADR-0064](../adr/0064-safe-edit-transactions.md), committed at
`7de36516d283a810ec5ec01b09980b07b6e634dc` (`Define Sprint 41 Safe Edit Transactions`).
Status: controlled-unwind architecture mapping; a new targeted design gate is
pending. Production conformance and execution of new unwind tests are not claimed.
The initial owner key/matrix below records the original planned design; current
implementation exists at `38a9bde3407f151e2c17b380e8bd28252c5a39f9`.
The complete controlled-unwind audit below reconciles that design with observed
owners and planned changes. The current prerequisite is the unique
`Define Sprint 41 controlled unwind recovery` commit, followed by the separate
`Approve Sprint 41 controlled unwind recovery design` pass. Producer and earlier
passes remain historical and do not approve this owner-lifetime revision.

## Owner and test key

Keys below expand to exact paths and symbols in each matrix row. Planned
private helpers specify roles; they do not add public transaction surfaces.

| Key | Exact production path and existing or planned symbols/roles |
|---|---|
| A | **Planned** `crates/analysis/src/safe_edit.rs`: `compare_plan` compares complete borrowed `RefactoringPlan` values; `replacement_bytes` validates/transforms one borrowed document and operation slice; `SafeEditProjection` maps only the callable/directly-owned-Query closure and raw ranges; `SafeEditProducerProjection`, `SafeEditProducerInput`, `SafeEditModuleInput`, `SafeEditProjectionAdmission` own complete typed keys, count/reserve/emit and immutable expected evidence; `SafeEditEvidence<'a>` borrows Graph/source/reference/validation/rule/diagnostic evidence per Configuration; `validate_postconditions` exhaustively compares before/after evidence with closed `SafeEditError`. Export through existing `crates/analysis/src/lib.rs`; no disk, policy or Runtime dependency. |
| AP | Existing `crates/analysis/src/refactoring.rs`: `RefactoringRequest`, `RefactoringPlan`, `RefactoringTarget`, `SourceEvidenceSet`, `SourceDocument`, `SourceOccurrence`, `SourceContentVersion`, `raw_range_to_source_span`, `raw_offset_to_source_position`. **Planned** production change only to the last two helpers' crate visibility so A reuses canonical BOM/CRLF/Unicode conversion. Also add module-local `#[cfg(test)] safe_edit_tests` for representable private-field mutants calling the real A comparator; expose no forge/reconstruction API or cross-crate test capability. |
| E | **Planned** `apps/runtime/src/workspace/edit.rs`: public `WorkspaceEditHandle::{prepare_apply,prepare_reversal,checked_apply,checked_reversal}`, `WorkspaceEditChallenge::confirm(self)`, `WorkspaceEditAuthorization`, `WorkspaceEditReceipt`, `WorkspaceEditCancellation::{new,request}`, `WorkspaceEditOwnership::ExclusiveCooperative`, `WorkspaceEditOutcome`; private `EditCoordinator::{reserve_attempt,prepare,submit,run_attempt,commit,recover,quarantine,shutdown}`, `EditAdmission`, `EditAttempt`, `EditUndo`, `EditPolicyGate`, `EditServiceIdentity`. Own service binding, bounded retention, policy admission and transaction state. |
| I | **Planned** `apps/runtime/src/workspace/edit_io.rs`: private `EditBaseline::capture` (bounded complete scan), `EditIo::{validate_path,read_checked,stage_all,replace_checked,restore_checked,cleanup_owned,verify_tree}`, `EditIoBudget` (checked preallocation reservations), `OwnedEditFile` (exact created path/identity). Standard-library I/O; deterministic test seams wrap these real operations, never an alternative transaction. |
| W | Existing `apps/runtime/src/workspace/mod.rs`: `WorkspaceService::start`, `initialize_workspace`, `run_workspace_updates`, `rebuild_workspace`, `compose_change_impact`, `finish_workspace_updates`, `WorkspaceSnapshot::{publication_id,plan_refactoring}`, `WorkspaceConfigurationSnapshot` getters and `WorkspaceSnapshotObserver`. **Planned** `WorkspaceService::{with_edit_policy,edit_handle}` and E/I integration into these lifecycle owners. Startup/update `send_replace(Some(..))` and shutdown `send_replace(None)` are all coordinated; E quarantine/commit add no independent sender owner. |
| B | Existing `apps/runtime/src/workspace/mod.rs`: `WorkspaceSnapshotBuilder::build`, `build_edt`, `build_designer_xml`, `validate_complete_build`, `snapshot_from_parts`, `compose_rule_evidence`. Existing `adapters/edt/src/lib.rs::FileSystemEdtSemanticGraphBuilder::build_graph_with_source_evidence`, `adapters/designer-xml/src/semantic_graph.rs::FileSystemDesignerXmlSemanticGraphBuilder::build_graph_with_source_evidence`, with `DesignerXmlBuildScope::Complete`; `adapters/edt/src/source_evidence.rs::build_source_evidence` and `adapters/designer-xml/src/source_evidence.rs::build_source_evidence` own occurrence capture. |
| C | Existing `apps/runtime/src/workspace/cache.rs`: `WorkspaceCacheStorage::{load,write}`, `WorkspaceCacheStore::{load,write,write_inner}`, `ensure_real_directory`. **Planned** private `WorkspaceCacheStorage::prepare_edit_namespace` and store implementation initialize/verify existing namespace before edit-baseline capture; controlled implementations follow the same ordering. Existing W `publish_cache_load` / `publish_cache_write` retain status ownership. No codec/schema or source-journal role. |
| P | Existing `crates/tool-policy/src/confirmation.rs::ToolConfirmationChallenge::confirm`, private `ConfirmationBinding` / `ToolConfirmation::matches`; `crates/tool-policy/src/execution.rs::execute_tool`. Runtime E `EditPolicyGate` is the **planned** immediate side-effect-free executor. |
| G | Existing `crates/graph/src/query.rs::SemanticGraphQuery::{nodes,edges,owner,edge_id}`, `crates/graph/src/node.rs::GraphNode::{id,name,kind,payload,provenance}`, `crates/graph/src/edge.rs::GraphEdge::{source,target,kind,provenance}`, `crates/graph/src/provenance.rs::Provenance`; `crates/bsl/src/lib.rs::bsl_callable_id`. Sole identity/fact owners. |
| L | Existing `crates/graph/src/reference_request.rs::SemanticReferenceRequest::{reconstruct_terminal,id,source_node,category,reference,expected_kinds,candidates,state,outcome,provenance}`, `SemanticReferenceRequestLedger::requests`; `crates/common/src/source.rs::{SourceLocation,SourcePosition,SourceSpan}`. Existing `adapters/edt/src/bsl_graph.rs::declaration_location` and `adapters/designer-xml/src/semantic_graph.rs::declaration_location` use line-start declaration anchors, not token spans. |
| D | Existing `crates/graph/src/diagnostic.rs::SemanticDiagnostic::{from_resolution_error_with_reference,new}`, `crates/graph/src/validation.rs::SemanticGraphValidator`, `crates/analysis/src/diagnostics/mod.rs::DiagnosticFinding::{from_semantic,from_validation,from_rule}`, `DiagnosticReport`, `crates/analysis/src/diagnostics/engine.rs::DiagnosticEngine`, `crates/analysis/src/rules/execution.rs::{RuleEngine,RuleExecutionReport,RuleDiagnostic}`. A compares full typed records and uses canonical constructors for derived fields. |
| U | Existing `crates/analysis/src/publication.rs::WorkspacePublicationId`; existing `apps/runtime/src/lib.rs` adds only **planned** local E exports. Existing `apps/runtime/src/workspace/change.rs::{WorkspaceFileState,WorkspaceChangeSource}` remains the normal observation owner, not I's complete edit scan. |
| DP | **Planned** `adapters/designer-xml/src/safe_edit.rs::project_safe_edit_provenance`, exported by `adapters/designer-xml/src/lib.rs`; shared `semantic_graph.rs::{module_source_id,declaration_provenance}` extracted from existing `emit_module_and_declarations`/`source_id`. Canonical whole-module SHA and every dependent declaration provenance stay with the producer. |
| EP | **Planned** `adapters/edt/src/safe_edit.rs::project_safe_edit_provenance`, exported by `adapters/edt/src/lib.rs`; shared pure `bsl_graph.rs::analyze_captured_module`, existing Query/request/diagnostic context helpers, and `query_source_resolution.rs` collection/resolver producers. No filesystem read in projection; existing captured/file APIs remain compatible. |
| Q | **Planned** `crates/bsl/src/queries.rs::bsl_query_id`, extracted from current `query_id` and exported by `crates/bsl/src/lib.rs`; extractor and projector share identical existing encoding. |

All matrix oracle identifiers are planned test functions unless marked existing.
`A::<name>` is in planned `crates/analysis/tests/safe_edit.rs` under F1;
`AP::<name>` is in planned `crates/analysis/src/refactoring.rs::safe_edit_tests`
under F1's library target, with access only through the owning module's privacy;
`R::<name>` in planned `apps/runtime/tests/safe_edit_transactions.rs` under F2;
`E::<name>` in planned `apps/runtime/src/workspace/edit.rs::tests` under F3;
`I::<name>` in planned `apps/runtime/src/workspace/edit_io.rs::tests` under F4.
These aliases are exact locations, not abstract verification suggestions.
Table-driven functions may cover multiple independently asserted cases.

## Sprint 41 ADR invariant matrix

Clause names are exact ADR-0064 Decision headings; order numbers refer to the
normative apply stages in `Single coordinator and guard ordering`. Each failure
oracle asserts the production closed cause, exact source/tree state, publication/
observer state and retained-material disposition as applicable. Each pre-write
rejection also asserts zero source replacement attempts.

| ID / accepted ADR clause | Exact production owner | Guarded operation or retention point and required order | Negative production-path oracle | Focused command |
|---|---|---|---|---|
| T01 Local API, authorization and lifetime | W configuration/start; E `reserve_attempt`, `EditServiceIdentity`; U exports | Default disabled, explicit cooperative ownership and immutable policy before startup; unavailable handle cannot reserve before readiness or after stop/poison. Platform eligibility precedes writes. | `R::disabled_unready_stopped_and_foreign_services_reject`: a second service at numeric publication 1 and unconfigured/new/stopped handles cannot reach I. | F2 |
| T02 Local API, authorization and lifetime; Complete publication baseline and bounds before retention | E `reserve_attempt`, `EditAdmission`, `EditAttempt` | Nonblocking sole prepared/queued/running slot before retaining input, plan construction or scan; checked attempt increment never wraps/reuses; release on drop/terminal path. | `E::attempt_lifetime_and_bounds`: second request while prepared/queued/running returns `Busy`; injected exhausted counter never reuses; dropped challenge/authorization releases exactly one slot. | F3 |
| T03 Local API, authorization and lifetime | E `prepare`, `submit`; W `plan_refactoring`; A `compare_plan`; AP owning-module tests and closed constructors/types | Prepare and submit regenerate from same current Arc; compare complete request, target, preconditions, ordered operations (paths/ranges/tokens/versions/IDs/replacements), dependencies, completeness and summary before authorization retention/staging. Hash equality never substitutes for complete equality. Respect the reachability split below; no public forging API. | `E::structured_plan_and_capability_tampering_reject` passes every constructor-reachable same-ID structural difference (including duplicate summaries and LocalCall/QualifiedCall categories), whole-plan substitutions and Runtime-private capability mutants through the real coordinator; zero writes. `AP::complete_plan_comparison_rejects_each_representable_private_field` preserves the plan ID while changing each representable private component and calls the real A comparator. Single-variant/type-unrepresentable states use explicit type/constructor evidence, not fabricated runtime tests. | F3, F1 |
| T04 Local API, authorization and lifetime | E `EditAttempt`, `submit`; P | Bind private Arc service identity, attempt, direction, predecessor Arc/ID, plan, baseline, frozen producer projection and immutable policy evaluation; irreversibly consume submission identity before revalidation. Capability fields private, non-cloneable, no constructor/deserializer; IDs/snapshots/receipts cannot confer authority. | `E::structured_plan_and_capability_tampering_reject`: actor/request/service, direction/baseline/Arc substitution, double submission and replay each reject; `R::disabled_unready_stopped_and_foreign_services_reject` submits foreign public capability. | F3, F2 |
| T05 Local API, authorization and lifetime | E `prepare`, `EditPolicyGate`, `submit`; P | Reserve bounded length-delimited arguments before allocation; exact apply/reverse ToolId, `LocalMutation`, actor/request/revision/effects/bytes. Only confirmed `RequireConfirmation` passes. Completed `execute_tool` precedes mutation queue/spawn/stage; executor performs none of those. | `E::policy_gate_is_exact_confirmed_and_side_effect_free`: Deny, bare Allow, missing/changed/reused confirmation, changed arguments/revision/effects and gate cancellation; no I event or mutation worker. | F3, F9 |
| T06 Complete publication baseline and bounds before retention | I `EditBaseline::capture`; E `prepare`; W `initialize_workspace`, `rebuild_workspace`; B | Before eligibility capture all directory/entry kinds and exact bytes before/after build or validated cache acceptance; require equal scans and full root/document agreement. Custom detector coverage must be provable. Never replace saved publication baseline with two later scans. | `R::complete_baseline_staleness_rejects`: alter untouched call/module, metadata, roots or unknown input after preview; two equal later scans still reject. `E::publication_baseline_admission` injects missing captured document/custom root. | F2, F3 |
| T07 Complete publication baseline and bounds before retention | C `prepare_edit_namespace`; I `capture`, `verify_tree`; W cache scheduling | Finish namespace/cache maintenance before capture; directory markers remain inputs. Exclude only actual confined regular reserved cache file without source/discovery role and exact verified owned I files. Scan ignored directories/unknown `.oneagent`; serialize cache temps; leftover temp remains input. | `E::cache_namespace_and_scan_exclusions`: `.oneagent` Configuration, ignored source, unknown temp, cache path as source/root, substituted kind/missing markers; no concealed input or eligible unstable publication. | F3, F7 |
| T08 Complete publication baseline and bounds before retention | I `EditIoBudget`, `capture`, `read_checked` | Before collecting/sorting each entry/path admit 16,384 entries, 4,096 bytes/path, 4,194,304 total path bytes. Before reading admit metadata/remaining allowance: 8,388,608 bytes/file and 67,108,864 total. Limited read detects one extra byte; enumeration scratch bounded. | `I::scan_bounds_precede_retention`: exact/one-over each dimension, growing read, overflow and wide/deep trees; observe read/allocation counters, no over-bound retention. | F4 |
| T09 Complete publication baseline and bounds before retention; order 1/3 | E `EditAdmission`; I `EditIoBudget`; A `replacement_bytes` | Before copies/buffers admit 4,096 operations/64 files, one Configuration/target, 1,048,576 original/result bytes per edited document, 8,388,608 aggregate originals/results each; checked result length before allocation. Preserve existing planner/adapter bounds. | `E::attempt_lifetime_and_bounds`, `I::buffer_and_disk_bounds_precede_allocation`: exact/one-over each dimension, oversized result and checked overflow; no allocation/stage beyond reservations. | F3, F4, F10 |
| T10 Complete publication baseline and bounds before retention | E `EditUndo`, `EditAdmission`; I `EditIoBudget` | Every baseline/verification/original/result/recovery copy plus new projection records, nested strings and scratch reserves within the same 268,435,456 bytes first through A/DP/EP count/reserve/emit before staging; share unchanged buffers, promptly release scans. Only one undo; expire before next apply/other successor/stop/poison. No rejected payload retained in error. | `I::buffer_and_disk_bounds_precede_allocation` covers simultaneous peak exact/one-over and overflow; `E::attempt_lifetime_and_bounds` asserts release on every terminal path including denied reversal/drop. | F4, F3 |
| T11 Filesystem ownership, staging and guarantees; order 1/2/4/5 | I `validate_path`, `read_checked`; E preparation/submission | Before each source read, stage, replace, restore and cleanup validate canonical Workspace/Configuration roots and all ancestors with `symlink_metadata`; reject lexical escapes, symlinks, wrong kind/outside canonical path, unprovable platform identity, nlink != 1, duplicate identities and non-bijective document/file mapping. | `I::confinement_rechecked_at_every_io_boundary`: traversal, root/ancestor/target symlinks, inside/outside hard links, duplicate alias and path/kind swaps at each seam; outside sentinel unchanged. | F4 |
| T12 Single coordinator and guard ordering, order 2/3 | A `replacement_bytes`, `SafeEditProjection`; E `run_attempt`; U | Check next publication increment before source mutation; validate document version, expected token, bounds/nonoverlap/canonical operation order; apply descending raw offsets preserving every other byte. | `A::replacement_rejects_invalid_ranges_versions_and_tokens`: stale version, wrong token, overlap/reordering/omission, invalid byte boundary and overflow reject; `E::publication_barriers_and_overflow` proves exhausted successor cannot stage/write. | F1, F3 |
| T13 Filesystem ownership, staging and guarantees; order 4 | I `stage_all`, `OwnedEditFile`, `EditIoBudget` | Before `create_new` reserve 16,777,216 disk bytes/128 files, bounded checked names without source extensions in same parent, private permissions; track exact created identity. Collisions never remove existing entries; no prefix sweep. Restore reuses backup slot. | `I::buffer_and_disk_bounds_precede_allocation`, `I::staging_faults_preserve_sources`: exact/one-over disk limits, name exhaustion/collision and owned-entry swap; unrelated entries survive, no source write. | F4 |
| T14 Single coordinator and guard ordering, order 4 | I `stage_all`, `read_checked`, `cleanup_owned` | Fully write results/backups, preserve standard permissions without broadening temp access, `sync_all`, close/read back exact bytes; verify all backups before first replace. Staging failure cleans only owned artifacts. | `I::staging_faults_preserve_sources`: every create/write/permission/sync/close-observation/readback ordinal for results/backups, short/corrupt output; no replacement, successful cleanup or explicit cleanup recovery failure. | F4 |
| T15 Single coordinator and guard ordering, order 5 | E `run_attempt`; I `verify_tree`, `replace_checked` | Repeat complete original baseline and path guards immediately before first replace; recheck original bytes/kind/identity before each later file in canonical path order. Rename verified sibling over target, no pre-delete/truncate/copy fallback. Record attempt before call and classify observed result even on ambiguous error. | `I::replacement_ordinals_classify_ambiguous_failure`: faults before/after each actual rename and later-target external edit; recovery reflects observed bytes, never false success or lost unrelated change. | F4 |
| T16 Single coordinator and guard ordering, order 6 | I `verify_tree`; E `run_attempt`; B | Entire actual tree must equal baseline plus exact result bytes before rebuild, excluding only verified owned files. Build complete Workspace; repeat scan after build; evaluate A. No candidate cache write. | `E::post_write_build_and_tree_failures_recover`: unrelated module/metadata mutation, added/removed root/file, edit during build; fail builder/Designer Complete/validation/rule/diagnostic composition; recover without candidate publication/cache write. | F3 |
| T17 Complete semantic oracle 1/6 | A `validate_postconditions`, `SafeEditEvidence`; B; I `verify_tree` | Before commit compare all Configuration IDs/roots/formats, Module identities/owners, document IDs/roles/paths/completeness/inventory and exact changed/untouched bytes. Other Configurations exactly equal; undo uses saved original projection. | `A::inventory_and_untouched_evidence_mismatch_rejects`: missing document/occurrence evidence, extra root, changed role/path or second Configuration; `E::semantic_candidate_faults_recover` submits every reachable candidate mutant through production comparison. The sole `SourceEvidenceCompleteness::BslCallableRenameV1` variant is closed-type evidence, not an executable changed-marker mutant. | F1, F3 |
| T18 Complete semantic oracle 2 | A projection/comparison; G; DP/EP/Q | BSL-owned expected target appears once and old target disappears; compare target name/kind/Module owner and all node payloads/provenance after the callable/directly-owned-Query closure and frozen DP/EP projection. Q owns Query IDs; exact binding/text and unchanged other-callable Query remain mandatory. Exact nonidentifier bytes also preserve export/async/parameters absent from Graph payload. No unrelated node change. | `A::node_and_edge_projection_rejects_unrelated_changes`: retained old/wrong new target, changed owner/kind/export source or unrelated equal-count node/payload change; `E::semantic_candidate_faults_recover` rejects same mutants. | F1, F3 |
| T19 Complete semantic oracle 2 | A comparison; G; DP/EP/Q | Exhaustive endpoints/kinds/full provenance after the callable/directly-owned-Query closure and frozen producer projection; Reads/DependsOn targets remain exact; `SemanticGraphQuery::edge_id` derives IDs. Explicit provenance comparison is mandatory: `GraphEdge::eq` omits it. Include incoming/outgoing calls inside target and ownership. | `A::node_and_edge_projection_rejects_unrelated_changes`: equal-count edge swap, lost internal/outgoing call, provenance-only change; `E::semantic_candidate_faults_recover` prevents publication. | F1, F3 |
| T20 Complete semantic oracle 3 | A projection/comparison; AP occurrences; B capture | Every occurrence in canonical document/range/kind order: exact cumulative byte deltas, replacement lengths, tokens/kinds/lexical owners/unique resolutions. Only prescribed target/range changes. Designer mappings remain required without unsupported Graph Calls edges. | `A::occurrence_projection_rejects_omission_and_ambiguity`: missing/extra same-count call, constructor-valid differing range/token/lexical-owner evidence, ambiguous/unsupported or retargeted untouched occurrence, missing Designer mapping; constructor rejection of inconsistent raw bytes/version/range is separate evidence, never credited as Runtime comparator execution; `E::semantic_candidate_faults_recover` checks production rejection. | F1, F3, F11 |
| T21 Complete semantic oracle 4 | A typed projection; DP/EP; AP coordinate helpers; L; G provenance | Freeze DP whole-module/declaration and EP nested Query/request/diagnostic provenance before stage/candidate. A independently enumerates typed before keys and rejects incomplete/duplicate/conflicting/unconsumed maps. Map before/result raw coordinates with canonical helpers; preserve path/source kind/role, producer/origin/confidence/resolution. Respect producer line-start/file-only anchors; never replace with token spans. Compare every node/edge/reference provenance record. | `A::anchor_and_reference_projection_rejects_loss`: BOM/multibyte longer/shorter names, CRLF/LF, file-only anchors, changed producer/path/span with same count; `E::semantic_candidate_faults_recover` blocks acceptance. | F1, F3 |
| T22 Complete semantic oracle 4 | A typed RequestFact/comparison; EP; L | Consume frozen EP complete terminal requests with dependent Query IDs and unchanged metadata; map typed references/source IDs/names/anchors; reconstruct terminal IDs via `reconstruct_terminal`; compare every category/expected kind/candidate/state/outcome/provenance. Resolved/unresolved/unsupported dispositions and statistics agree, no omitted request. | `A::anchor_and_reference_projection_rejects_loss`: lost unresolved request, altered candidate/outcome at equal statistics, canonical identity changed through a structural reference substitution or provenance change; no isolated corruption of a private derived ID; `E::semantic_candidate_faults_recover` rejects. | F1, F3 |
| T23 Complete semantic oracle 5 | A typed DiagnosticFact/comparison; DP/EP; D; B `compose_rule_evidence` | Require complete validation/rule/diagnostic composition; compare every existing report/status/count and complete record collection, code/severity/parameters/related evidence/anchors; no absent incomplete marker or omission-counter field is invented. Use before-bound DiagnosticFact keys and frozen canonical producer diagnostics, including malformed/unsupported Query; reconstruct canonical findings/IDs from typed inputs; unchanged messages exact, derived messages use known producer. Unclassifiable mapping fails closed, no blanket replacement/clean-project prerequisite. | `A::diagnostic_and_rule_projection_rejects_non_equivalence`: lost unrelated pre-existing finding, changed severity/message/related evidence, lost producer evidence or equal-count producer-generated rule status; use canonical whole-record/report substitutions rather than private derived-field corruption; valid mapped pre-existing diagnostics pass. `E::semantic_candidate_faults_recover` checks all report mutants. | F1, F3 |
| T24 Complete semantic oracle 6; order 7 | W `compose_change_impact`; E `run_attempt`, `commit`; U | Derive impact from actual adjacent pair after complete oracle; check current predecessor Arc and successor overflow. Undo receives new publication/impact IDs, never original ID. | `E::publication_barriers_and_overflow`: stale Arc, wrong impact pair/overflow reject or recover; `R::apply_reversal_preserve_exact_bytes_and_old_arcs` proves old readers immutable and one increment. | F3, F2 |
| T25 Single coordinator and guard ordering, order 7/8 | E `run_attempt`, `commit`; I `cleanup_owned`, `verify_tree`; W sender | Check cancellation/predecessor/full state, remove all owned stage/backup while retaining pre-reserved originals; cleanup failure recovers. Final expected source/path scan and cancellation after cleanup precede sole commit `send_replace(Some(successor))`. Same turn installs baseline/outcome/undo and invalidates preparation. No fallible source I/O or cancellation rollback after commit. | `E::final_cleanup_scan_and_commit_barrier`: cleanup failure, edit after cleanup, cancel at final guard/just after commit; only precommit cases recover, committed success remains. | F3 |
| T26 Single coordinator and guard ordering, order 9 | E `commit`; W update loop/cache status; C `write` | Only accepted successor writes cache while serialized; success delivered after bounded cache attempt. Cache failure affects cache status, never source success/count. Existing schema/version; no baseline/capability/undo persistence. | `E::cache_namespace_and_scan_exclusions` blocks/fails postcommit cache write without rollback; **planned** `apps/runtime/tests/persistent_cache.rs::edit_commit_cache_failure_preserves_success` checks public status/cold restart without edit authority. | F3, F7 |
| T27 Single coordinator and guard ordering | W all three writers; E coordinator; C; U change source | No admission during earlier build/cache write; no watcher/explicit-input build, cache write or clearing writer alongside transaction. Coalesce watcher noise; self-write-only baseline consumes no extra ID; true external change and bounded explicit input rebuild normally. | `E::publication_barriers_and_overflow` controls startup/build/cache/transaction/stop; **planned** `apps/runtime/tests/file_watching.rs::edit_self_write_noise_and_external_change`, `apps/runtime/tests/git_change_workspace.rs::edit_serializes_explicit_change_input` verify public ordering. | F3, F6, F8 |
| T28 Recovery, failure precedence and reversal | E `recover`; I `restore_checked`, `verify_tree`, `cleanup_owned` | All failures after any attempted replacement enter joined recovery in reverse attempt order; restore only exact expected result identity/bytes or prove already original. Observe ambiguous failures. Stage/verify originals through same confinement; never overwrite third-party bytes/delete unknown entry. | `I::recovery_ordinals_preserve_unrelated_edits`: each restoration/check/stage/rename ordinal, third-state bytes/kind/alias swaps; unchanged unrelated sentinels and complete outcome record. | F4 |
| T29 Recovery, failure precedence and reversal | E `recover`; I `verify_tree`, `cleanup_owned` | Recovery success requires full original bytes/tree/permissions and all owned artifacts removed; retain old publication/no increment, original closed cause plus `Recovered`, no undo. Restore exact originals, not semantic reconstruction. | `E::recovery_outcomes_and_precedence`: failure after every replace then successful restore checks full state/old Arc; remaining temp/untouched-source change cannot report `Recovered`. | F3 |
| T30 Recovery, failure precedence and reversal | E `quarantine`, `shutdown`; W writers; I retained material | Restore/original verification/cleanup failure takes precedence as `RecoveryRequired`, trigger secondary. Clear current to None; invalidate capabilities/undo; disable edits/rebuild publication/cache writes; retain bounded recovery data. Stop preserves unremovable owned files/counts. Only operator repair and new cold validated service exits. | `E::recovery_outcomes_and_precedence`: fail recovery then edit/watch/explicit-input/cache/stop; no new publication/write, old Arcs immutable, no force/retry, no source/path secrets. | F3 |
| T31 Recovery, failure precedence and reversal | E `prepare_reversal`, `EditUndo`, `submit`, `run_attempt`; A; I | Receipt consumed even if preparation denied; same service/exact current applied successor Arc/ID/result baseline plus fresh confirmation. Saved originals/results/original semantics, symmetric bounds/guards/rebuild/cleanup; submitted reversal consumes undo on every outcome. Failure restores applied state or quarantines. | `R::stale_reversal_and_reconfirmation_reject`: foreign/stale successor, intervening apply/rebuild, denied/replayed confirmation, changed bytes; `E::reversal_failures_restore_applied_state` covers shared fault ordinals and no replay. | F2, F3 |
| T32 Cancellation, shutdown and redaction | E cancellation/run/recovery/shutdown; W `finish_workspace_updates` | Before first replace cancel cleans staging; afterwards requests uninterruptible recovery. Dropped response never drops service ownership; terminal status retained until slot release. Stop closes admission/invalidates preparation/signals work/joins worker and recovery before clearing; committed success survives. | `E::cancellation_drop_and_shutdown_join`: cancel/drop/stop every phase, block restore and prove shutdown waits; no detached writer/late rollback. | F3, F5 |
| T33 Recovery, failure precedence and reversal; Cancellation, shutdown and redaction | E outcomes/attempt/undo/error/Debug; I errors; P audit | Precedence: availability, input/bounds, capability, policy, cancellation, publication/plan, source/path, staging; first error per phase except recovery override. Before output/log retention redact bytes/tokens/digests/arguments/absolute paths/nested errors; only bounded preview displays source projection. | `E::closed_precedence_and_redaction`: multiple simultaneous failures and secret sentinels in source/path/arguments/build/I/O error; inspect Debug/errors/audit/outcome/captured logs, only closed status/counts/IDs and deterministic results. | F3, F9 |
| T34 Compatibility and affected consumers; Owners and dependency direction | U; W/B/AP read-only owners; existing `apps/runtime/src/mcp_tools.rs`, `main.rs`, `bin/oneagent-mcp.rs`, `bin/oneagent-lsp.rs` | Additive opt-in Rust API only; no planner authority/new counter; preserve impact alias, eight-tool catalog, protocols/GraphQuery/diagnostics/adapter semantics/cache format. No product entry point enables edits. Pure A uses public G/L/D and admitted DP/EP/Q owners, no second planner/graph facts. Test canonical encoding and builder compatibility before/after helper extraction. | **Planned** `apps/runtime/tests/workspace_service.rs::default_service_remains_read_only_with_edit_api` denies mutation; existing public consumer/paired planner suites reject unsupported/stale input and preserve observations. | F5, F10, F11, F12 |
| T35 Complete publication baseline and bounds before retention; Local API, authorization and lifetime | W `initialize_workspace`, `rebuild_workspace`, `run_workspace_updates`; E `EditAttempt`, `EditUndo`; I `capture` | Edit-enabled initial instability fails startup; unstable successor retains predecessor/no increment. An unprovable or over-bound edit baseline never permits preparation; otherwise valid read-only evidence remains read-only. Preserve otherwise eligible Query-containing targets without blanket rejection. Every accepted nontransaction successor invalidates prepared challenge/authorization/undo; no time-based or cross-process credential exists. | `E::publication_baseline_admission`: unstable before/after startup, rebuild and validated cache hit cannot become edit-eligible; over-bound baseline denies preparation; a watcher/explicit-input successor expires retained challenge and undo. Existing default read-only lifecycle remains valid. | F3, F5, F7 |

## Constructor-boundary evidence correction

Task 5 admission at `b2f89c86012e71190afed077f42b5af82d552b42`
found that the original T03 oracle crossed Analysis-private fields from Runtime
and an integration test. `RefactoringPlan::new` validates relationships and
recomputes the ID; there is no public reconstruction/deserialization seam.
`RefactoringCompleteness` has only `Complete`, so an incomplete enum value is
not representable in safe Rust. This is an evidence-placement defect, not a
production vulnerability or a change to ADR-0064's transaction mechanism.

Runtime still proves all reachable same-ID differences through production
admission, particularly constructor-produced duplicate-summary differences and
LocalCall/QualifiedCall category differences. Analysis owner-local unit tests
exercise full structural comparison for every safely representable private
component, preserving the old ID without making those values public. Record
the closed enum/constructor definitions for unrepresentable states separately;
do not use unsafe values, weaken constructor invariants, add a public forge
API, or count an impossible mutation as an executed test. Type evidence does
not replace reachable negative tests. All other T rows and semantic/failure
oracles remain unchanged. The corrected mapping requires a new committed
targeted design pass before Task 5 resumes.

## Complete matrix representability audit

The user explicitly agreed to this consolidated evidence-placement plan after
the independent full T01-T35 audit of
`ceb3a91da70afde202cab84f7ea42846cdd734bd..d2dc6fdb772f04cb15cc38029b7494d262cae9af`.
That review accepted the T03 correction but blocked the still-impossible T17
changed-completeness marker. This section records the complete audit, not another
isolated exception. It changes no production invariant or transaction mechanism.

R means reachable through a public API/constructor/producer and exercised through
the admitted production path. L means a safely representable private state tested
inside its owning module. C means constructor rejection precedes publication of
the invalid combination. T means an alternative value/field does not exist in
safe Rust. Constructor/type evidence never replaces a reachable negative test
and must not be counted as an executed Runtime comparator test.

| Classification | Complete audited row inventory | Evidence placement |
|---|---|---|
| R | T01, T19, T34 | Existing public alternatives/constructors and production consumer paths. |
| L | T10 | Private E/I reservation and lifecycle seams inside their owners. |
| R + C | T12, T18, T20, T21, T22, T23 | Constructor-valid but semantically wrong whole values/records reach the real comparator; rejected combinations are documented separately. |
| R + L | T02, T04, T06, T07, T08, T09, T11, T13, T14, T15, T16, T24, T25, T26, T27, T28, T29, T30, T31, T32, T33, T35 | Real operations plus already admitted private E/I/W/C seams; no new forging surface. |
| R + L + C | T05 | Real policy alternatives plus Runtime-private binding tests and canonical policy rejection. |
| R + L + C + T | T03 | The constructor-boundary split above, including both reachable same-ID differences and AP owner-local comparator tests. |
| R + T | T17 | Executable missing/extra documents, occurrences, inventory, role/path/root/Configuration substitutions; the sole source-completeness variant is type evidence only. |

For T17 the exact type owner is
`crates/analysis/src/refactoring.rs::SourceEvidenceCompleteness`, whose only
variant is `BslCallableRenameV1`. `SourceDocument::new` and
`SourceDocument::completeness` carry that same closed type. A different marker
cannot be manufactured even in an owner-local safe test. Keep comparison of the
actual field and all reachable incompleteness (lost documents/occurrences/source
evidence); do not add a variant, unsafe value, or public forge API for testing.

T20 uses `SourceDocument::new`-valid semantic differences, preserving its raw
byte/version/Unicode/lexical consistency checks. T22 uses
`SemanticReferenceRequest::reconstruct_terminal` for structural identity and
complete-record changes; a private ID alone is not separately forgeable. T23
uses canonical diagnostic constructors, `SemanticGraphValidator`,
`RuleEngine`, and report/finding producers. Compare all real fields/records;
do not invent a report incomplete marker, omission counter, or direct mutation
of private derived identity/status fields. All actual omissions and unrelated
finding/request losses remain negative production-path cases.

The revised plan keeps 35 requirements, all reachable negative oracles, the
then-current 16-path/5000-churn/no-binary baseline, F1's library target and
12 focused groups. That full audit and pass remain historical evidence. The
producer correction below updates the mechanism/scope without deleting any
classification or reachable oracle. No test or production conformance is claimed.

## Producer projection correction and complete audit disposition

The user approved this correction after the incomplete Task 5 attempt at
`93661837df8d63bfed10c9b70d1986c4e0d12aa5` exposed Designer digest and EDT
owned-Query identity dependencies. The original full 35-row classification
remains unchanged; this additional complete disposition preserves every reachable
negative case. Exact typed keys, complete consumption and allocation arithmetic
are normative in ADR-0064, not choices left to implementation.

| Audited rows | Correction disposition and required evidence |
|---|---|
| T01-T05 | Preserve authorization, full plan, private owner and sole reservation; bind expected projection to that same attempt (F1/F2/F3/F9). |
| T06-T10 | Full source coverage/retention unchanged. A `producer_projection_completeness_and_bounds` and E `projection_freeze_precedes_io` test count/reserve/emit for all nested strings, mapping/record storage, Query text, inventory/sort/consumption scratch within 268435456; exact/one-over, overflow, partial-construction release and no I/O. DP/EP tests exercise shared counting encoders (F1/F3/F4/F10/F11). |
| T11-T16 | Existing path/stage/write/build guards unchanged; E `projection_freeze_precedes_io` injects producer failure before stage and proves zero I events. Freeze expected before rebuild/candidate injection (F3/F4). |
| T17 | Exact inventory/other-Configuration comparison and closed completeness-type evidence unchanged; mappings cannot hide missing facts (F1/F3). |
| T18-T19 | Only callable-to-directly-owned-Query closure via Q; preserve binding/text/payload and full edge/provenance. Extra/removed Query, altered binding/text/payload, another callable with equal binding and Reads/DependsOn mutants reach A and E (F1/F3/F10/F11). |
| T20 | Exact bytes, complete occurrence/token/range and constructor-valid substitutions preserved (F1/F3/F11). |
| T21 | DP `designer_projection_preserves_canonical_provenance`, EP `edt_projection_preserves_nested_query_evidence`: before reproduction, longer/shorter Unicode, LF/BOM-CRLF, changed-module unchanged declarations, untouched metadata and undo. Source-ID/digest/producer/path/role/location mutants reach A and E after rebuild (F1/F3/F11). |
| T22 | EP complete ledger uses canonical dependent Query IDs and all collection/resolver/projection provenance. Request/candidate/outcome mutants reach real A/E, never private ID forgery (F1/F3/F11). |
| T23 | Complete canonical diagnostics/rules with mapped anchors/reference IDs; preserve resolved/unresolved/malformed/unsupported Query and pre-existing parser diagnostics. Full-record mutants reach A/E (F1/F3/F10/F11). |
| T24-T27 | Publication, impact, cache and watcher ordering unchanged (F2/F3/F6/F7/F8). |
| T28-T30 | Sole filesystem owner, exact recovery and quarantine unchanged (F3/F4). |
| T31 | Exact original semantic evidence and producer IDs remain within the same undo reservation; shared reversal-failure tests include producer projection (F2/F3/F11). |
| T32 | Cancellation/drop/shutdown joining and lease release/transfer preserved; lifecycle evidence remains pending (F3/F5). |
| T33 | Closed redacted failures exclude source bytes, paths, digests and producer contexts (F3/F9/F11). |
| T34 | Compatible canonical DP/EP/Q helpers; builder/cache/protocol encodings unchanged before/after extraction (F5/F10/F11/F12). |
| T35 | Full eligibility, including Query-containing targets, preserved; no blanket Query restriction (F2/F3/F5/F7/F11). |

Allocation tests distinguish existing scoped canonical parser internals from new
projection-owned retention. The borrowed count pass itself allocates nothing and
calls no additional parser. All emitted/retained strings, records, parser-result
copies, nested maps, sorting and growth-overlap buffers are leased beforehand.
Keep the exact 1MiB simple-rename positive; no universal parser reserve or hidden
size cap. QueryFact inventory is independently reconciled with format-supported
Graph Query facts: EDT includes malformed/unsupported Query, Designer has none.

DP/EP test names resolve to owner-local tests in each new adapter `safe_edit.rs`;
Q tests reside in `crates/bsl/src/queries.rs`. A names resolve to
`crates/analysis/tests/safe_edit.rs`; E names resolve to `workspace/edit.rs::tests`.
Missing/extra/duplicate/colliding typed maps and attempts to use candidate-derived
expected values are separate pre-write projection admission cases. Constructor
rejection is never credited as executed Runtime comparison. Every safely
representable candidate mutant still passes the real production A comparator
and E `run_attempt` after a complete rebuild against independently frozen expected
values. All original T03/T17 owner-local/type distinctions remain in force.

## Controlled-unwind ownership and complete audit

Normative mechanism: [controlled transaction-owner unwind](../adr/0064-safe-edit-transactions.md#controlled-transaction-owner-unwind).
This amendment is planned, with no executed post-write unwind reproduction.
Observed E `execute`, `prepare`, `run_attempt`, `commit`, `abandon_commit` and
W `run_workspace_updates` currently use Result-only transaction finalization;
W's joined-worker error loses the moved coordinator, and E constructs undo after
publication. I already registers a successful create before identity acquisition,
but registration capacity and all partial transitions must also withstand unwind.
Earlier owner-local constructor/type corrections and all negative cases remain.

Planned private E roles `EditEnvelope`, `finalize_failure` and `contain_recovery`
retain coordinator, reservation/attempt, response, phase, optional I owner and
prepared terminal/commit material outside narrow synchronous catches. These are
implementation roles, not new public signatures. W owns the sender and rejoins
the returned envelope; I remains the sole confined algorithm. Boundary order is
P preparation, S consumed/confirmed submission, M mutation/build/comparison,
F precommit material/cleanup/final guard, R checked recovery, C publication, then
K cache/delivery. R is the recovery phase here; R/L/C/T classifications below
retain their separate representability meaning. No M/F catch may publish.

New named oracles are **planned**, all inside admitted source files: E test
functions `controlled_unwind_preparation_retains_owner` (UP),
`controlled_unwind_before_replace_cleans_staging` (US),
`controlled_unwind_after_replace_recovers` (UM),
`controlled_unwind_recovery_quarantines` (UR),
`controlled_unwind_stop_and_response_join` (UL),
`controlled_unwind_commit_preserves_success` (UC) in
`apps/runtime/src/workspace/edit.rs::tests` under F3, and
`controlled_unwind_io_ordinals_preserve_ownership` (UI) in
`apps/runtime/src/workspace/edit_io.rs::tests` under F4. Existing matrix oracles
remain required; aliases below supplement, never replace, their exact names.
The R/L/C/T column covers the existing requirement: no new public semantic
forging surface is inferred from these private containment tests.

| Row | R/L/C/T | Actual owner / ordered retained-boundary obligation | Meaningful oracle / command |
|---|---|---|---|
| T01 | R | E `reserve_attempt`, W service startup: opt-in identity/liveness precedes P; no edit-enabled production entry point. | Existing disabled/foreign-service oracle asserts zero I events (F2); UP separately exercises private P containment (F3). |
| T02 | R + L | E reservation and envelope retain the sole slot through P/S/M/F/R and terminal transfer; never reuse IDs. | Existing attempt bounds plus UP/UL assert Busy, non-replay and one terminal release (F3). |
| T03 | R + L + C + T | E `prepare`/`run_attempt` regenerate and A `compare_plan` checks full structured plan before M; retained predecessor remains immutable across catch. | Existing E/AP complete-plan mutants and constructor/type evidence remain separate; UP verifies preparation failure cannot yield capability or I event (F1/F3). |
| T04 | R + L | E `execute` consumes the private capability before S revalidation; envelope keeps reservation/attempt through failure. | Existing private/foreign binding mutants plus UP/UL assert consumed submit/reversal cannot replay after unwind (F2/F3). |
| T05 | R + L + C | E handle `submit` completes `EditPolicyGate` before mutation queue; containment cannot skip confirmation or move mutation into executor. | Existing exact policy/no-queue oracle and UP/US trace policy before S/M with paired confirmed controls (F3/F9). |
| T06 | R + L | E `prepare`, I `EditBaseline::capture`, W build retain the saved publication baseline; neither catch nor recovery replaces it with two later scans. | Existing staleness/admission mutants plus UM verify original complete baseline after restore (F2/F3). |
| T07 | R + L | W/C namespace completion precedes P capture; I scan excludes only verified owned entries/exact safe cache file, including during R. | Existing scan exclusions plus US/UI unknown-created-identity sentinel case forbid cleanup or exclusion authority (F3/F4/F7). |
| T08 | R + L | I capture/read/budget admission precedes allocation in P/M/F/R; error/unwind retains bounded state only. | Existing exact/one-over scan cases and UI observe read/allocation ordering and outside sentinels (F4). |
| T09 | R + L | E `check_admission`, I `with_admission`, A replacement preserve all operation/file/result bounds before M; envelope adds no unreserved copy. | Existing attempt/buffer bounds plus US observe no stage before prepaid result/recovery buffers (F3/F4/F10). |
| T10 | L | E attempt/projection, I admission and undo transfer the same lease through M/F/R/C; stage/terminal failure expires it without an extra retained copy. | Existing shared-lease/attempt bounds plus UP/UR/UC assert retained/peak bytes and release or transfer at every terminal path (F3/F4). |
| T11 | R + L | I path/read/create/replace/restore/cleanup recheck confinement each time; catching unwind never authorizes an unknown path/identity. | Existing confinement negatives and UI exercise aliases/kind/path swaps with outside sentinel unchanged (F4). |
| T12 | R + C | E `run_attempt`, A replacements check overflow/version/token/order before M; planned F packaging also precedes C. | Existing range/overflow oracles prove zero writes for rejected input; UC checks one canonical successor in valid control (F1/F3). |
| T13 | R + L | I `create_owned` reserves registration capacity before create, then immediately records present/unknown identity before metadata checks/callbacks. | UI/US observe each result/backup create and unknown-identity boundary, exact retained count, collisions and no removal of unrelated entry (F3/F4). |
| T14 | R + L | I `stage_all` fully writes/verifies all results/backups before replacement; retained I survives create/write/permissions/sync/close/readback unwind. | UI/US enumerate real kind/ordinal in both directions; no source replacement, verified cleanup or Required quarantine (F3/F4). |
| T15 | R + L | I `replace_checked` validates baseline/target, records attempt before rename and records observed result before next callback; M failures go to R. | UI/UM exercise before/after each rename and ambiguous results, reverse attempt order and third-state sentinel preservation (F3/F4). |
| T16 | R + L | E `run_attempt` retains I outside complete builder/tree/compare catch after real replacement; expected projection was frozen earlier. | UM uses owner-local custom detector that accepts initial build and panics on post-write build; paired EDT/Designer apply/reversal controls and exact restored tree/no candidate cache or publication (F3). |
| T17 | R + T | A inventory/complete source comparison after real B build and before F; all Configurations/documents remain exhaustive. | Existing A/E inventory mutants reach real comparator; sole completeness variant remains type evidence; UM verifies comparator boundary reached (F1/F3). |
| T18 | R + C | A node/closure comparison with frozen DP/EP/Q evidence stays inside M, without new node or provenance ownership. | Existing constructor-valid node/Query mutants and E semantic-candidate recovery, then UM comparison unwind with valid positive control (F1/F3). |
| T19 | R | A compares every edge endpoint/kind and explicit provenance inside M; containment changes no semantic input reachability. | Existing reachable complete edge/provenance mutants still reject without publication; UM independently covers owner-local catch, not private edge forgery (F1/F3). |
| T20 | R + C | A/AP complete occurrence/range/lexical evidence follows real build inside M; constructor restrictions unchanged. | Existing canonical occurrence mutants and E candidate recovery, UI/UM preserve exact Unicode/BOM/line-ending bytes on recovery (F1/F3/F4/F11). |
| T21 | R + C | A/DP/EP before-bound complete provenance projection freezes before M; all mapping consumption precedes F. | Existing canonical anchor/reference/producer mutants and UM full comparison boundary; no candidate-derived expected evidence (F1/F3/F11). |
| T22 | R + C | A/EP/L compare canonical whole requests, dependent Query IDs and every disposition inside M. | Existing reconstruct_terminal whole-record mutants reach A/E; no invented private ID corruption; UM preserves original ledger after restore (F1/F3/F11). |
| T23 | R + C | B composition and A compare full diagnostic/rule reports inside M before F; preserve pre-existing findings. | Existing producer-valid report mutants plus UM actual build/comparison unwind; no invented completeness/count field (F1/F3/F10/F11). |
| T24 | R + L | W `compose_change_impact`, E predecessor guard and prepared commit bind actual adjacent pair before C; worker cannot publish. | Existing publication barrier/overflow and UC reject stale pair before C, prove old Arc immutable and exactly one valid increment (F2/F3). |
| T25 | R + L | E F material construction/cleanup/final scan precede W `send_replace`; envelope keeps I through `abandon_commit` and last guard. | US/UM/UC cover cleanup/final-guard/undo-preparation unwind and removed-backup recreation; only precommit failure restores (F3/F4). |
| T26 | R + L | W postcommit K cache task cannot own source recovery; prepared success is retained independently through cache result/unwind. | UC and existing public cache-failure oracle prove accepted ID/bytes/receipt persist on cache failure, cancellation or response drop (F3/F7). |
| T27 | R + L | W `run_workspace_updates` rejoins envelope/terminal state before writer release; all startup/watch/input/cache/clear writers remain serialized. | UL plus existing barriers/watch/input tests block R, assert Busy/no competing publication/cache and clean join (F3/F5/F6/F8). |
| T28 | R + L | E shared `finalize_failure`/`contain_recovery`, I `restore_checked`/`restore_one` cover Err/unwind and `abandon_commit` in reverse attempted order. | UI/UM cover actual restoration and recreated-backup ordinals; UR injects recovery unwind without retry/third-party overwrite (F3/F4). |
| T29 | R + L | I final full baseline/permission comparison and owned cleanup must succeed before E reports Recovered; no-write cleanup also verifies original tree. | UM/US compare all original bytes/tree/modes/artifacts; remaining temp or unrelated change fails verified success (F3/F4). |
| T30 | R + L | E retained recovery owner/quarantine, W observation clear: recovery Err/unwind overrides trigger and prohibits later writers; stop preserves material. | UR covers restore/verification/cleanup unwind, exact count including unknown identity, closed secondary, None observation, no later publication/cache or sentinel change (F3/F4). |
| T31 | R + L | E `prepare_reversal`/`execute` share envelope/finalizer and all M/F/R stages; applied predecessor is reversal recovery oracle. | UP/UM/UI paired reversal at every shared ordinal restores applied state or quarantines; receipt/undo consumed with no replay (F2/F3/F4). |
| T32 | R + L | E response sender/terminal/reservation remain retained across catches; W joins mutation and R before stop clearing or owner release. | UL keeps/drops response, cancels/stops each phase and blocks restore to prove stop waits; UC proves committed success survives (F3/F5). |
| T33 | R + L | E maps unwind by ADR phase without payload formatting/retention; R failure overrides; transaction-controlled outputs remain closed. | UP/UM/UR closed cause/disposition/count and secret-sentinel assertions inspect outcome/Debug/audit/transaction logs, without global hook/stderr redaction claim (F3/F9). |
| T34 | R | U public API, W builder composition and A/DP/EP/Q contracts unchanged; only three Runtime source paths admitted. | Existing default read-only consumers/paired planner suites plus source/API/dependency audit; private custom detector is not public service injection (F5/F10/F11/F12). |
| T35 | R + L | W baseline eligibility and E `expire_capability`/shutdown retain every successor/stop/quarantine expiration rule through envelope terminal transfer. | Existing baseline and retained-capability oracles plus UP/UR/UL assert no replay, released payload/lease, quarantine persists and no Query blanket rejection (F2/F3/F5/F7). |

UP/US/UM/UI must observe actual named production operations and record that all
earlier admission/confirmation/build guards ran. UI covers create, stage, replace,
read/check/compare-adjacent I/O, cleanup, restore and backup recreation ordinals;
UM covers actual build/semantic comparison and final-guard transitions. A fault
at an earlier synthetic checkpoint is not proof of a later boundary. Positive
controls must reach commit for both paired formats and directions without hooks.
The private detector crosses W's real builder only after a source replacement:
`WorkspaceSnapshotBuilder::with_detector` is public, but
`WorkspaceService::with_builder` and `builder` are private. This is L, never a
public-service injection finding. No default-input panic trigger is established.
Startup panic tests are not transaction/post-write evidence. The rejected old
exploratory probe was never compiled/executed and must not be retried or counted.

## Concrete focused validation commands

Task 5 runs these **12 checks**, sequentially, after implementing planned tests.
F1-F4 originally named new tests/modules; those implementation targets now exist.
The seven controlled-unwind functions above are still planned and unexecuted.
No Rust tests were run by Task 3 or this architecture amendment. At the
exact implementation head each command must execute nonzero tests and enumerate
every assigned named oracle; a nonzero group cannot hide missing oracle names.
Record per-target counts and zero-match filters separately.

| Key | Concrete command |
|---|---|
| F1 | `cargo test -p oneagent-analysis --lib --test safe_edit` |
| F2 | `cargo test -p oneagent-runtime --test safe_edit_transactions` |
| F3 | `cargo test -p oneagent-runtime --lib workspace::edit::tests::` |
| F4 | `cargo test -p oneagent-runtime --lib workspace::edit_io::tests::` |
| F5 | `cargo test -p oneagent-runtime --test workspace_service` |
| F6 | `cargo test -p oneagent-runtime --test file_watching` |
| F7 | `cargo test -p oneagent-runtime --test persistent_cache` |
| F8 | `cargo test -p oneagent-runtime --test git_change_workspace` |
| F9 | `cargo test -p oneagent-tool-policy --all-targets` |
| F10 | `cargo test -p oneagent-analysis -p oneagent-bsl --all-targets` |
| F11 | `cargo test -p oneagent-designer-xml -p oneagent-edt --all-targets` |
| F12 | `cargo test -p oneagent-runtime --test mcp_process --test mcp_semantic_tools --test lsp_stdio --test graph_query_api` |

F1 covers each representable pure comparator mutant at its admitted owner.
Every row follows the complete representability audit above. T03 and T17 do not
require Runtime to forge Analysis-private or type-unrepresentable values. For
T16-T23 semantic candidate evidence, F3 must pass every reachable, safely
constructed candidate mutant
through `EditCoordinator::run_attempt` after the real production build and before
the real comparator/commit. The expected projection is already frozen before staging and candidate build. That seam changes candidate evidence only; it cannot
bypass production admission, use a test-only validator or directly publish a
forged candidate. F4 enumerates every I/O kind/ordinal in both directions,
including before/after ambiguous replacement, restoration and cleanup. F3
crosses that shared fault table with transaction outcomes/lifecycle barriers.
Assertions inspect retention/allocation admission and forbidden-I/O absence,
not only errors. Small injected limits can prove order, but production exact
maximum/one-over cases remain required for every numeric bound.

F2 copies the tracked paired corpus at
`adapters/designer-xml/tests/fixtures/sprint14_conformance/` into a temporary
Workspace; its README owns EDT LF and Designer BOM/CRLF provenance. Existing
partial conformance is not Complete Runtime proof. The planned helper in
`safe_edit_transactions.rs` composes existing complete Runtime fixture descriptors,
adjusts the paired manifest to retained members and creates a second Common
Module descriptor/source with a qualified call. Record exact text derivations in
`apps/runtime/tests/fixtures/workspace_service/README.md`; preserve tracked source
hashes and existing grammar. Both production `WorkspaceSnapshotBuilder::build`
paths must accept the temporary multi-file fixture before transaction assertions.
F2 checks longer/shorter Unicode names, declaration/local/qualified/internal
calls, exact BOM/line endings/trailing bytes, apply/reversal, full semantic output
and old Arcs. Missing complete-build acceptance blocks implementation completion.

## Implementation scope and validation budget

The user-approved revised allocation is **25 unique
paths, 12000 textual additions plus deletions, no binaries, 12 focused checks,
one stable canonical full gate**. This allocation is an estimate, not measured
implementation churn or permission to weaken an invariant. Existing dependency
edges/public queries/constructors suffice; no package/manifest, Graph payload,
adapter semantic rule or protocol change is required. Compatible canonical
producer helper extraction is now expressly included.

| # | Exact Task 5 path | Planned purpose | Estimated text churn |
|---:|---|---|---:|
| 1 | `crates/analysis/src/safe_edit.rs` | Pure plan/range/semantic comparison | 1800 |
| 2 | `crates/analysis/src/lib.rs` | Additive export | 10 |
| 3 | `crates/analysis/src/refactoring.rs` | Internal coordinate helper reuse and private-field comparator unit tests | 300 |
| 4 | `apps/runtime/src/workspace/edit.rs` | API/coordinator/policy/state and F3 tests | 3600 |
| 5 | `apps/runtime/src/workspace/edit_io.rs` | Confined bounded I/O/shared seams and F4 tests | 1550 |
| 6 | `apps/runtime/src/workspace/mod.rs` | Writer/baseline/builder integration | 450 |
| 7 | `apps/runtime/src/workspace/cache.rs` | Private namespace preparation | 60 |
| 8 | `apps/runtime/src/lib.rs` | Additive local exports | 20 |
| 9 | `crates/analysis/tests/safe_edit.rs` | F1 public/semantic comparison mutants; private mutants move to owner-local unit tests | 850 |
| 10 | `apps/runtime/tests/safe_edit_transactions.rs` | F2 public paired/multi-file evidence | 650 |
| 11 | `apps/runtime/tests/workspace_service.rs` | F5 default/readiness/stop compatibility | 100 |
| 12 | `apps/runtime/tests/file_watching.rs` | F6 self-write/external-change evidence | 100 |
| 13 | `apps/runtime/tests/persistent_cache.rs` | F7 cache failure/restart evidence | 100 |
| 14 | `apps/runtime/tests/git_change_workspace.rs` | F8 explicit-input serialization | 100 |
| 15 | `apps/runtime/tests/fixtures/workspace_service/README.md` | Exact fixture derivations/byte oracle | 45 |
| 16 | `docs/codex/prompts/sprint-41-safe-edit-transactions/00-sprint-41-execution-loop.md` | Task 5 ledger | 15 |
| 17 | `adapters/designer-xml/src/safe_edit.rs` | Pure complete Designer expected projection, count/emit and tests | 650 |
| 18 | `adapters/designer-xml/src/semantic_graph.rs` | Shared canonical provenance helpers and compatibility tests | 120 |
| 19 | `adapters/designer-xml/src/lib.rs` | Additive projection exports | 10 |
| 20 | `adapters/edt/src/safe_edit.rs` | Pure complete EDT expected projection, count/emit and tests | 800 |
| 21 | `adapters/edt/src/bsl_graph.rs` | Captured-byte analyzer/shared context producers and regression tests | 350 |
| 22 | `adapters/edt/src/query_source_resolution.rs` | Canonical context reuse and request regression tests | 220 |
| 23 | `adapters/edt/src/lib.rs` | Additive projection exports | 10 |
| 24 | `crates/bsl/src/queries.rs` | Canonical public Query ID helper, shared use and tests | 80 |
| 25 | `crates/bsl/src/lib.rs` | Query helper export | 10 |

This estimate depends on table-driven faults/projections reusing production
owners, not a promise of final line count. Historically, the original 16
allocations totalled 5000 and the nine producer additions 3500: 25 paths/8500.
The current 25-path allocation above totals 12000, using the complete audited
remaining-work estimate and the later 24-path/9369-churn pause inventory. It
redistributes space to actual implementation and the remaining constructor,
producer, lifecycle, I/O, public, formatting and ledger evidence; no row or
requirement is removed. On 2026-09-08 the user explicitly authorized a 20000
hard cap, superseding the proposed 14000 and previous 10000. This changes only
numeric budget; the mechanism gate at f25388cd8073bcd228c8eaa951ef1c0178907431
remained valid for that numeric-only change. The later controlled-unwind
amendment now requires its own separate targeted design pass.
Stop **before further work** at more than 32 unique paths, more than 20000 text
additions plus deletions, or any binary path. This explicitly tighter cap
overrides the general 2x rule on the new estimate; do not derive 50/24000 caps.

Implementation accounting retains original Task 5 baseline
`93661837df8d63bfed10c9b70d1986c4e0d12aa5`. The new design pass is a resume
prerequisite, not a reset of implementation accounting. The preserved incomplete
9 files/1593 churn belong to Task 5 and must not be subtracted as unrelated
pre-existing work. Count the cumulative task-owned implementation diff,
including task-owned untracked text, formatting and its ledger update; exclude
only separately committed prerequisite documentation corrections/review records.
Maintain an explicit per-path reconciliation against the original baseline,
removing those separately committed documentation deltas by their commit/range,
not a blanket subtraction for every pre-existing path. Do not sum overlapping
snapshots of the same changes twice. At resume verify all nine restored files
against the preserved inventory before additional edits. If sufficient producer
evidence cannot be exposed or full comparison requires an unadmitted owner or
dependency, stop with the concrete blocker, never narrow the oracle. Task 4
independently evaluates this scope/mapping; this document does not record its pass.

After focused checks pass on the stable complete diff, run the canonical full
gate once: `cargo fmt --all -- --check`, `cargo check --workspace --all-targets`,
`cargo test --workspace --all-targets`,
`cargo clippy --workspace --all-targets --all-features -- -D warnings`,
`RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps`, and
`git diff --check`, owned by [Validation](../codex/core/validation.md).
Independent reviewer and primary integration gates are separate required evidence.

## Separate non-production evidence and deferred guarantees

For controlled-unwind remediation, the approved additional source envelope is
only E/I/W: `apps/runtime/src/workspace/edit.rs`, `edit_io.rs` and `mod.rs`.
Estimated additional source churn is 1000-1800; seven authority/evidence paths
have a separate 300-700 estimate. These remain estimates. Initial measured
implementation subtotal at `38a9bde3407f151e2c17b380e8bd28252c5a39f9` is
25 paths, +14238/-234 = 14472, no binaries. Preserve original baseline
`93661837df8d63bfed10c9b70d1986c4e0d12aa5`, its 24 source/fixture paths and
only original Task 5 shared-master delta
`f3c1f8c087378b78c50f7fd97499b2c2d7e5f510..f2813d2eff5fa78efe3f0d4a705e3bc51de13979`.
Keep caps 32 paths/20000 churn/no binaries; no reset or blanket master exclusion.
The new committed architecture/pass only admits the later coherent source task,
then separate Task 6 evidence and fresh independent/primary Task 7 full gates.
One stable remediation F1-F12/G1-G6 is required; historical partial checks do not
qualify new source. Sprint 41 stays active and v0.7 release-ineligible.

Documentation links/selectors, exact-head oracle enumeration/counts, dependency/
consumer audits, unchanged Coverage Registry, scope/churn accounting, independent
review, master ledger and prompt retirement are governance evidence. Task 3 runs
documentation gates only; Task 6 records implementation evidence and Task 7 owns
integration/retirement. No Supported claim follows from this map.

ADR-0064's excluded families, metadata/path/file renames, cross-Configuration or
cross-Workspace mutation, Git/remote/model-driven edits, MCP/HTTP/CLI/LSP/IDE edit
endpoints, durable journals/restart recovery, cross-process undo, richer metadata
preservation, broader platforms, hostile-writer TOCTOU exclusion, multi-file disk
atomicity, power-loss durability, benchmarks, Sprint 42 and release remain
deferred. Cooperative ownership covers root/directory hierarchy through recovery.
A stalled OS call may delay joined shutdown; implementation must not claim to
prevent these limitations. Standard permissions/exact bytes are covered above;
timestamps/inodes/ACLs/xattrs and other richer metadata are not promised.
