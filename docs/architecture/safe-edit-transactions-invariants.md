# Safe Edit Transaction Invariants

Accepted design: [ADR-0064](../adr/0064-safe-edit-transactions.md), committed at
`7de36516d283a810ec5ec01b09980b07b6e634dc` (`Define Sprint 41 Safe Edit Transactions`).
Status: complete architecture mapping for the independent Task 4 gate.
Production conformance and execution of planned tests are not claimed.
Every new symbol and test below is explicitly **planned** for Task 5. This
mapping refines locations and evidence, not the accepted algorithm or scope.

## Owner and test key

Keys below expand to exact paths and symbols in each matrix row. Planned
private helpers specify roles; they do not add public transaction surfaces.

| Key | Exact production path and existing or planned symbols/roles |
|---|---|
| A | **Planned** `crates/analysis/src/safe_edit.rs`: `compare_plan` compares complete borrowed `RefactoringPlan` values; `replacement_bytes` validates/transforms one borrowed document and operation slice; `SafeEditProjection` maps only the accepted target and raw ranges; `SafeEditEvidence<'a>` borrows Graph/source/reference/validation/rule/diagnostic evidence per Configuration; `validate_postconditions` exhaustively compares before/after evidence with closed `SafeEditError`. Export through existing `crates/analysis/src/lib.rs`; no disk, policy or Runtime dependency. |
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
| T04 Local API, authorization and lifetime | E `EditAttempt`, `submit`; P | Bind private Arc service identity, attempt, direction, predecessor Arc/ID, plan, baseline and immutable policy evaluation; irreversibly consume submission identity before revalidation. Capability fields private, non-cloneable, no constructor/deserializer; IDs/snapshots/receipts cannot confer authority. | `E::structured_plan_and_capability_tampering_reject`: actor/request/service, direction/baseline/Arc substitution, double submission and replay each reject; `R::disabled_unready_stopped_and_foreign_services_reject` submits foreign public capability. | F3, F2 |
| T05 Local API, authorization and lifetime | E `prepare`, `EditPolicyGate`, `submit`; P | Reserve bounded length-delimited arguments before allocation; exact apply/reverse ToolId, `LocalMutation`, actor/request/revision/effects/bytes. Only confirmed `RequireConfirmation` passes. Completed `execute_tool` precedes mutation queue/spawn/stage; executor performs none of those. | `E::policy_gate_is_exact_confirmed_and_side_effect_free`: Deny, bare Allow, missing/changed/reused confirmation, changed arguments/revision/effects and gate cancellation; no I event or mutation worker. | F3, F9 |
| T06 Complete publication baseline and bounds before retention | I `EditBaseline::capture`; E `prepare`; W `initialize_workspace`, `rebuild_workspace`; B | Before eligibility capture all directory/entry kinds and exact bytes before/after build or validated cache acceptance; require equal scans and full root/document agreement. Custom detector coverage must be provable. Never replace saved publication baseline with two later scans. | `R::complete_baseline_staleness_rejects`: alter untouched call/module, metadata, roots or unknown input after preview; two equal later scans still reject. `E::publication_baseline_admission` injects missing captured document/custom root. | F2, F3 |
| T07 Complete publication baseline and bounds before retention | C `prepare_edit_namespace`; I `capture`, `verify_tree`; W cache scheduling | Finish namespace/cache maintenance before capture; directory markers remain inputs. Exclude only actual confined regular reserved cache file without source/discovery role and exact verified owned I files. Scan ignored directories/unknown `.oneagent`; serialize cache temps; leftover temp remains input. | `E::cache_namespace_and_scan_exclusions`: `.oneagent` Configuration, ignored source, unknown temp, cache path as source/root, substituted kind/missing markers; no concealed input or eligible unstable publication. | F3, F7 |
| T08 Complete publication baseline and bounds before retention | I `EditIoBudget`, `capture`, `read_checked` | Before collecting/sorting each entry/path admit 16,384 entries, 4,096 bytes/path, 4,194,304 total path bytes. Before reading admit metadata/remaining allowance: 8,388,608 bytes/file and 67,108,864 total. Limited read detects one extra byte; enumeration scratch bounded. | `I::scan_bounds_precede_retention`: exact/one-over each dimension, growing read, overflow and wide/deep trees; observe read/allocation counters, no over-bound retention. | F4 |
| T09 Complete publication baseline and bounds before retention; order 1/3 | E `EditAdmission`; I `EditIoBudget`; A `replacement_bytes` | Before copies/buffers admit 4,096 operations/64 files, one Configuration/target, 1,048,576 original/result bytes per edited document, 8,388,608 aggregate originals/results each; checked result length before allocation. Preserve existing planner/adapter bounds. | `E::attempt_lifetime_and_bounds`, `I::buffer_and_disk_bounds_precede_allocation`: exact/one-over each dimension, oversized result and checked overflow; no allocation/stage beyond reservations. | F3, F4, F10 |
| T10 Complete publication baseline and bounds before retention | E `EditUndo`, `EditAdmission`; I `EditIoBudget` | Every baseline/verification/original/result/recovery copy reserves within 268,435,456 additional raw bytes first; share unchanged buffers, promptly release scans. Only one undo; expire before next apply/other successor/stop/poison. No rejected payload retained in error. | `I::buffer_and_disk_bounds_precede_allocation` covers simultaneous peak exact/one-over and overflow; `E::attempt_lifetime_and_bounds` asserts release on every terminal path including denied reversal/drop. | F4, F3 |
| T11 Filesystem ownership, staging and guarantees; order 1/2/4/5 | I `validate_path`, `read_checked`; E preparation/submission | Before each source read, stage, replace, restore and cleanup validate canonical Workspace/Configuration roots and all ancestors with `symlink_metadata`; reject lexical escapes, symlinks, wrong kind/outside canonical path, unprovable platform identity, nlink != 1, duplicate identities and non-bijective document/file mapping. | `I::confinement_rechecked_at_every_io_boundary`: traversal, root/ancestor/target symlinks, inside/outside hard links, duplicate alias and path/kind swaps at each seam; outside sentinel unchanged. | F4 |
| T12 Single coordinator and guard ordering, order 2/3 | A `replacement_bytes`, `SafeEditProjection`; E `run_attempt`; U | Check next publication increment before source mutation; validate document version, expected token, bounds/nonoverlap/canonical operation order; apply descending raw offsets preserving every other byte. | `A::replacement_rejects_invalid_ranges_versions_and_tokens`: stale version, wrong token, overlap/reordering/omission, invalid byte boundary and overflow reject; `E::publication_barriers_and_overflow` proves exhausted successor cannot stage/write. | F1, F3 |
| T13 Filesystem ownership, staging and guarantees; order 4 | I `stage_all`, `OwnedEditFile`, `EditIoBudget` | Before `create_new` reserve 16,777,216 disk bytes/128 files, bounded checked names without source extensions in same parent, private permissions; track exact created identity. Collisions never remove existing entries; no prefix sweep. Restore reuses backup slot. | `I::buffer_and_disk_bounds_precede_allocation`, `I::staging_faults_preserve_sources`: exact/one-over disk limits, name exhaustion/collision and owned-entry swap; unrelated entries survive, no source write. | F4 |
| T14 Single coordinator and guard ordering, order 4 | I `stage_all`, `read_checked`, `cleanup_owned` | Fully write results/backups, preserve standard permissions without broadening temp access, `sync_all`, close/read back exact bytes; verify all backups before first replace. Staging failure cleans only owned artifacts. | `I::staging_faults_preserve_sources`: every create/write/permission/sync/close-observation/readback ordinal for results/backups, short/corrupt output; no replacement, successful cleanup or explicit cleanup recovery failure. | F4 |
| T15 Single coordinator and guard ordering, order 5 | E `run_attempt`; I `verify_tree`, `replace_checked` | Repeat complete original baseline and path guards immediately before first replace; recheck original bytes/kind/identity before each later file in canonical path order. Rename verified sibling over target, no pre-delete/truncate/copy fallback. Record attempt before call and classify observed result even on ambiguous error. | `I::replacement_ordinals_classify_ambiguous_failure`: faults before/after each actual rename and later-target external edit; recovery reflects observed bytes, never false success or lost unrelated change. | F4 |
| T16 Single coordinator and guard ordering, order 6 | I `verify_tree`; E `run_attempt`; B | Entire actual tree must equal baseline plus exact result bytes before rebuild, excluding only verified owned files. Build complete Workspace; repeat scan after build; evaluate A. No candidate cache write. | `E::post_write_build_and_tree_failures_recover`: unrelated module/metadata mutation, added/removed root/file, edit during build; fail builder/Designer Complete/validation/rule/diagnostic composition; recover without candidate publication/cache write. | F3 |
| T17 Complete semantic oracle 1/6 | A `validate_postconditions`, `SafeEditEvidence`; B; I `verify_tree` | Before commit compare all Configuration IDs/roots/formats, Module identities/owners, document IDs/roles/paths/completeness/inventory and exact changed/untouched bytes. Other Configurations exactly equal; undo uses saved original projection. | `A::inventory_and_untouched_evidence_mismatch_rejects`: missing document/occurrence evidence, extra root, changed role/path or second Configuration; `E::semantic_candidate_faults_recover` submits every reachable candidate mutant through production comparison. The sole `SourceEvidenceCompleteness::BslCallableRenameV1` variant is closed-type evidence, not an executable changed-marker mutant. | F1, F3 |
| T18 Complete semantic oracle 2 | A projection/comparison; G | BSL-owned expected target appears once and old target disappears; compare target name/kind/Module owner and all node payloads/provenance after substitution. Exact nonidentifier bytes also preserve export/async/parameters absent from Graph payload. No unrelated node change. | `A::node_and_edge_projection_rejects_unrelated_changes`: retained old/wrong new target, changed owner/kind/export source or unrelated equal-count node/payload change; `E::semantic_candidate_faults_recover` rejects same mutants. | F1, F3 |
| T19 Complete semantic oracle 2 | A comparison; G | Exhaustive endpoints/kinds/full provenance after single target substitution; `SemanticGraphQuery::edge_id` derives IDs. Explicit provenance comparison is mandatory: `GraphEdge::eq` omits it. Include incoming/outgoing calls inside target and ownership. | `A::node_and_edge_projection_rejects_unrelated_changes`: equal-count edge swap, lost internal/outgoing call, provenance-only change; `E::semantic_candidate_faults_recover` prevents publication. | F1, F3 |
| T20 Complete semantic oracle 3 | A projection/comparison; AP occurrences; B capture | Every occurrence in canonical document/range/kind order: exact cumulative byte deltas, replacement lengths, tokens/kinds/lexical owners/unique resolutions. Only prescribed target/range changes. Designer mappings remain required without unsupported Graph Calls edges. | `A::occurrence_projection_rejects_omission_and_ambiguity`: missing/extra same-count call, constructor-valid differing range/token/lexical-owner evidence, ambiguous/unsupported or retargeted untouched occurrence, missing Designer mapping; constructor rejection of inconsistent raw bytes/version/range is separate evidence, never credited as Runtime comparator execution; `E::semantic_candidate_faults_recover` checks production rejection. | F1, F3, F11 |
| T21 Complete semantic oracle 4 | A projection; AP coordinate helpers; L; G provenance | Map before/result raw coordinates with canonical helpers; preserve path/source kind/role, producer/origin/confidence/resolution. Respect producer line-start/file-only anchors; never replace with token spans. Compare every node/edge/reference provenance record. | `A::anchor_and_reference_projection_rejects_loss`: BOM/multibyte longer/shorter names, CRLF/LF, file-only anchors, changed producer/path/span with same count; `E::semantic_candidate_faults_recover` blocks acceptance. | F1, F3 |
| T22 Complete semantic oracle 4 | A projection/comparison; L | Map typed references/source IDs/names/anchors; reconstruct terminal IDs via `reconstruct_terminal`; compare every category/expected kind/candidate/state/outcome/provenance. Resolved/unresolved/unsupported dispositions and statistics agree, no omitted request. | `A::anchor_and_reference_projection_rejects_loss`: lost unresolved request, altered candidate/outcome at equal statistics, canonical identity changed through a structural reference substitution or provenance change; no isolated corruption of a private derived ID; `E::semantic_candidate_faults_recover` rejects. | F1, F3 |
| T23 Complete semantic oracle 5 | A comparison; D; B `compose_rule_evidence` | Require complete validation/rule/diagnostic composition; compare every existing report/status/count and complete record collection, code/severity/parameters/related evidence/anchors; no absent incomplete marker or omission-counter field is invented. Reconstruct canonical findings/IDs from typed inputs; unchanged messages exact, derived messages use known producer. Unclassifiable mapping fails closed, no blanket replacement/clean-project prerequisite. | `A::diagnostic_and_rule_projection_rejects_non_equivalence`: lost unrelated pre-existing finding, changed severity/message/related evidence, lost producer evidence or equal-count producer-generated rule status; use canonical whole-record/report substitutions rather than private derived-field corruption; valid mapped pre-existing diagnostics pass. `E::semantic_candidate_faults_recover` checks all report mutants. | F1, F3 |
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
| T34 Compatibility and affected consumers; Owners and dependency direction | U; W/B/AP read-only owners; existing `apps/runtime/src/mcp_tools.rs`, `main.rs`, `bin/oneagent-mcp.rs`, `bin/oneagent-lsp.rs` | Additive opt-in Rust API only; no planner authority/new counter; preserve impact alias, eight-tool catalog, protocols/GraphQuery/diagnostics/adapter semantics/cache format. No product entry point enables edits. Pure A uses public G/L/D owners, no second planner/graph facts. | **Planned** `apps/runtime/tests/workspace_service.rs::default_service_remains_read_only_with_edit_api` denies mutation; existing public consumer/paired planner suites reject unsupported/stale input and preserve observations. | F5, F10, F11, F12 |
| T35 Complete publication baseline and bounds before retention; Local API, authorization and lifetime | W `initialize_workspace`, `rebuild_workspace`, `run_workspace_updates`; E `EditAttempt`, `EditUndo`; I `capture` | Edit-enabled initial instability fails startup; unstable successor retains predecessor/no increment. An unprovable or over-bound edit baseline never permits preparation; otherwise valid read-only evidence remains read-only. Every accepted nontransaction successor invalidates prepared challenge/authorization/undo; no time-based or cross-process credential exists. | `E::publication_baseline_admission`: unstable before/after startup, rebuild and validated cache hit cannot become edit-eligible; over-bound baseline denies preparation; a watcher/explicit-input successor expires retained challenge and undo. Existing default read-only lifecycle remains valid. | F3, F5, F7 |

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
16-path/5000-churn/no-binary baseline, F1's library target and 12 focused groups.
Require the separately committed complete-audit design pass before resuming
implementation. No test or production conformance is claimed by this document.

## Concrete focused validation commands

Task 5 runs these **12 checks**, sequentially, after implementing planned tests.
F1-F4 currently name new tests/modules: running them now would yield a missing
target or zero matches, not evidence. No Rust tests were run by Task 3. At the
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
| F10 | `cargo test -p oneagent-analysis --test refactoring_plan --test refactoring_source_evidence` |
| F11 | `cargo test -p oneagent-designer-xml --test conformance` |
| F12 | `cargo test -p oneagent-runtime --test mcp_process --test mcp_semantic_tools --test lsp_stdio --test graph_query_api` |

F1 covers each representable pure comparator mutant at its admitted owner.
Every row follows the complete representability audit above. T03 and T17 do not
require Runtime to forge Analysis-private or type-unrepresentable values. For
T16-T23 semantic candidate evidence, F3 must pass every reachable, safely
constructed candidate mutant
through `EditCoordinator::run_attempt` after the real production build and before
the real comparator/commit. That seam changes candidate evidence only; it cannot
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

The accepted design fits the committed baseline at mapping level: **16 unique
paths, 5000 textual additions plus deletions, no binaries, 12 focused checks,
one stable canonical full gate**. This allocation is an estimate, not measured
implementation churn or permission to weaken an invariant. Existing dependency
edges/public queries/constructors suffice; no package/manifest, Graph payload,
adapter semantic rule or protocol change is required.

| # | Exact Task 5 path | Planned purpose | Estimated text churn |
|---:|---|---|---:|
| 1 | `crates/analysis/src/safe_edit.rs` | Pure plan/range/semantic comparison | 800 |
| 2 | `crates/analysis/src/lib.rs` | Additive export | 10 |
| 3 | `crates/analysis/src/refactoring.rs` | Internal coordinate helper reuse and private-field comparator unit tests | 160 |
| 4 | `apps/runtime/src/workspace/edit.rs` | API/coordinator/policy/state and F3 tests | 1250 |
| 5 | `apps/runtime/src/workspace/edit_io.rs` | Confined bounded I/O/shared seams and F4 tests | 1000 |
| 6 | `apps/runtime/src/workspace/mod.rs` | Writer/baseline/builder integration | 430 |
| 7 | `apps/runtime/src/workspace/cache.rs` | Private namespace preparation | 60 |
| 8 | `apps/runtime/src/lib.rs` | Additive local exports | 20 |
| 9 | `crates/analysis/tests/safe_edit.rs` | F1 public/semantic comparison mutants; private mutants move to owner-local unit tests | 350 |
| 10 | `apps/runtime/tests/safe_edit_transactions.rs` | F2 public paired/multi-file evidence | 630 |
| 11 | `apps/runtime/tests/workspace_service.rs` | F5 default/readiness/stop compatibility | 60 |
| 12 | `apps/runtime/tests/file_watching.rs` | F6 self-write/external-change evidence | 60 |
| 13 | `apps/runtime/tests/persistent_cache.rs` | F7 cache failure/restart evidence | 60 |
| 14 | `apps/runtime/tests/git_change_workspace.rs` | F8 explicit-input serialization | 60 |
| 15 | `apps/runtime/tests/fixtures/workspace_service/README.md` | Exact fixture derivations/byte oracle | 45 |
| 16 | `docs/codex/prompts/sprint-41-safe-edit-transactions/00-sprint-41-execution-loop.md` | Task 5 ledger | 5 |

This estimate depends on table-driven faults/projections reusing production
owners, not a promise of final line count. Task 5 measures paths/churn from its
exact task-start commit, including its ledger, and explains growth before review;
the committed scope/binary stop-loss remains binding. If sufficient producer
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
