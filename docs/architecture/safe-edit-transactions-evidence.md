# Sprint 41 Safe Edit Transaction Evidence

## Status and immutable implementation boundary

Task 5 is implemented and its retained validation is reconciled. Task 6 records
the evidence for integration review; Sprint 41 remains **active**. This document
does not supply Task 7's independent review, primary completion gate, completion
transition, previous-suite retirement or release approval.

The unique implementation commit is
`f2813d2eff5fa78efe3f0d4a705e3bc51de13979`
(`Implement Sprint 41 Safe Edit Transactions`) on `codex/v0.7-sprint-41`.
Task 6 started at that exact clean tracked/index/untracked boundary.
The exact implementation delta is
`f3c1f8c087378b78c50f7fd97499b2c2d7e5f510..f2813d2eff5fa78efe3f0d4a705e3bc51de13979`.
The original cumulative accounting baseline remains
`93661837df8d63bfed10c9b70d1986c4e0d12aa5`; the producer design pass
`f25388cd8073bcd228c8eaa951ef1c0178907431` is a mechanism prerequisite,
not an accounting reset. The exact separately committed prerequisite range
`93661837df8d63bfed10c9b70d1986c4e0d12aa5..f3c1f8c087378b78c50f7fd97499b2c2d7e5f510`
contains only the ten documentation paths enumerated by the retained exclusion
audit. Its production tree is unchanged. Subtracting only that exact delta
includes all carried implementation and the Task 5 ledger change in its shared
path; no whole path or overlapping intermediate total is subtracted.

The resulting inventory is **25 paths, +11067/-192 = 11259 text churn, seven
created and eighteen modified files, no binaries**. This fits the user-authorized
25-path/12000-churn estimate and 32-path/20000-churn hard caps from
`f3c1f8c087378b78c50f7fd97499b2c2d7e5f510`. The earlier 10000 cap and
failed attempts remain historical evidence. Task 6 changes only documentation.

Canonical decision: [ADR-0064](../adr/0064-safe-edit-transactions.md).
The accepted [T01-T35 matrix](safe-edit-transactions-invariants.md#sprint-41-adr-invariant-matrix)
and [producer design gate](../reviews/sprint-41-safe-edit-transactions-design.md#current-producer-owned-projection-decision)
remain the authorities; the line references below describe actual committed
owners rather than treating planned symbol names as implemented APIs.

## Committed production and oracle locations

Every `Key:line` below is a one-based source line at the implementation commit
above. Owner keys expand to these exact repository paths. Test references use
the full path and function name; source line numbers are immutable evidence for
that commit, not a promise about future revisions.

| Key | Exact committed owner |
|---|---|
| A | [`crates/analysis/src/safe_edit.rs`](../../crates/analysis/src/safe_edit.rs) |
| AP | [`crates/analysis/src/refactoring.rs`](../../crates/analysis/src/refactoring.rs) |
| E | [`apps/runtime/src/workspace/edit.rs`](../../apps/runtime/src/workspace/edit.rs) |
| I | [`apps/runtime/src/workspace/edit_io.rs`](../../apps/runtime/src/workspace/edit_io.rs) |
| W | [`apps/runtime/src/workspace/mod.rs`](../../apps/runtime/src/workspace/mod.rs) |
| C | [`apps/runtime/src/workspace/cache.rs`](../../apps/runtime/src/workspace/cache.rs) |
| DP | [`adapters/designer-xml/src/safe_edit.rs`](../../adapters/designer-xml/src/safe_edit.rs) |
| EP | [`adapters/edt/src/safe_edit.rs`](../../adapters/edt/src/safe_edit.rs) |
| Q | [`crates/bsl/src/queries.rs`](../../crates/bsl/src/queries.rs) |
| U | [`apps/runtime/src/lib.rs`](../../apps/runtime/src/lib.rs) |
| P | [`crates/tool-policy/src/execution.rs`](../../crates/tool-policy/src/execution.rs) |

The actual recovery/quarantine branch is inside `E::run_attempt`; publication
occurs in W's existing lifecycle loop. There is no claim that every private
helper name proposed by the design became a separately named function.

| Invariant / ADR clause | Actual production location | Guard, order and boundary qualified by the tests | Executed negative oracle and outcome | Command |
|---|---|---|---|---|
| T01 Local API, authorization and lifetime | W:686 with_edit_policy; W:749 service start; E:220 reserve_attempt | Default disabled, explicit cooperative ownership and immutable policy before startup; unavailable handle cannot reserve before readiness or after stop/poison. Platform eligibility precedes writes. | [`disabled_unready_stopped_and_foreign_services_reject`](../../apps/runtime/tests/safe_edit_transactions.rs) (`apps/runtime/tests/safe_edit_transactions.rs:359`): a second service at numeric publication 1 and unconfigured/new/stopped handles cannot reach I. | F2 |
| T02 Local API, authorization and lifetime; Complete publication baseline and bounds before retention | E:220 reserve_attempt; E:318 submit; E:994 execute | Nonblocking sole prepared/queued/running slot before retaining input, plan construction or scan; checked attempt increment never wraps/reuses; release on drop/terminal path. | [`attempt_lifetime_and_bounds`](../../apps/runtime/src/workspace/edit.rs) (`apps/runtime/src/workspace/edit.rs:1824`): second request while prepared/queued/running returns `Busy`; injected exhausted counter never reuses; dropped challenge/authorization releases exactly one slot. | F3 |
| T03 Local API, authorization and lifetime | E:586 prepare; E:1136 run_attempt; A:237 compare_plan; AP:3071 owner-local test | Prepare and submit regenerate from same current Arc; compare complete request, target, preconditions, ordered operations (paths/ranges/tokens/versions/IDs/replacements), dependencies, completeness and summary before authorization retention/staging. Hash equality never substitutes for complete equality. Respect the reachability split below; no public forging API. | [`structured_plan_and_capability_tampering_reject`](../../apps/runtime/src/workspace/edit.rs) (`apps/runtime/src/workspace/edit.rs:3125`) passes every constructor-reachable same-ID structural difference (including duplicate summaries and LocalCall/QualifiedCall categories), whole-plan substitutions and Runtime-private capability mutants through the real coordinator; zero writes. [`complete_plan_comparison_rejects_each_representable_private_field`](../../crates/analysis/src/refactoring.rs) (`crates/analysis/src/refactoring.rs:3071`) preserves the plan ID while changing each representable private component and calls the real A comparator. Single-variant/type-unrepresentable states use explicit type/constructor evidence, not fabricated runtime tests. | F3, F1 |
| T04 Local API, authorization and lifetime | E:318 submit; E:364 challenge/authorization/receipt; E:735 policy_matches | Bind private Arc service identity, attempt, direction, predecessor Arc/ID, plan, baseline, frozen producer projection and immutable policy evaluation; irreversibly consume submission identity before revalidation. Capability fields private, non-cloneable, no constructor/deserializer; IDs/snapshots/receipts cannot confer authority. | [`structured_plan_and_capability_tampering_reject`](../../apps/runtime/src/workspace/edit.rs) (`apps/runtime/src/workspace/edit.rs:3125`): actor/request/service, direction/baseline/Arc substitution, double submission and replay each reject; [`disabled_unready_stopped_and_foreign_services_reject`](../../apps/runtime/tests/safe_edit_transactions.rs) (`apps/runtime/tests/safe_edit_transactions.rs:359`) submits foreign public capability. | F3, F2 |
| T05 Local API, authorization and lifetime | E:586 prepare; E:432 EditPolicyGate::execute; E:735 policy_matches; E:994 execute | Reserve bounded length-delimited arguments before allocation; exact apply/reverse ToolId, `LocalMutation`, actor/request/revision/effects/bytes. Only confirmed `RequireConfirmation` passes. Completed `execute_tool` precedes mutation queue/spawn/stage; executor performs none of those. | [`policy_gate_is_exact_confirmed_and_side_effect_free`](../../apps/runtime/src/workspace/edit.rs) (`apps/runtime/src/workspace/edit.rs:3323`): Deny, bare Allow, missing/changed/reused confirmation, changed arguments/revision/effects and gate cancellation; no I event or mutation worker. | F3, F9 |
| T06 Complete publication baseline and bounds before retention | W:857 prepare_edit_baseline; W:868 finish_edit_baseline; E:771 check_admission; I:190 capture | Before eligibility capture all directory/entry kinds and exact bytes before/after build or validated cache acceptance; require equal scans and full root/document agreement. Custom detector coverage must be provable. Never replace saved publication baseline with two later scans. | [`complete_baseline_staleness_rejects`](../../apps/runtime/tests/safe_edit_transactions.rs) (`apps/runtime/tests/safe_edit_transactions.rs:492`): alter untouched call/module, metadata, roots or unknown input after preview; two equal later scans still reject. [`publication_baseline_admission`](../../apps/runtime/src/workspace/edit.rs) (`apps/runtime/src/workspace/edit.rs:3566`) injects missing captured document/custom root. | F2, F3 |
| T07 Complete publication baseline and bounds before retention | C:284 prepare_edit_namespace; I:205 scan; E:771 check_admission; W:1014 run_workspace_updates | Finish namespace/cache maintenance before capture; directory markers remain inputs. Exclude only actual confined regular reserved cache file without source/discovery role and exact verified owned I files. Scan ignored directories/unknown `.oneagent`; serialize cache temps; leftover temp remains input. | [`cache_namespace_and_scan_exclusions`](../../apps/runtime/src/workspace/edit.rs) (`apps/runtime/src/workspace/edit.rs:3759`): `.oneagent` Configuration, ignored source, unknown temp, cache path as source/root, substituted kind/missing markers; no concealed input or eligible unstable publication. | F3, F7 |
| T08 Complete publication baseline and bounds before retention | I:190 capture; I:205 scan; I:367 read_checked; I:156 entry_length; I:169 file admission | Before collecting/sorting each entry/path admit 16,384 entries, 4,096 bytes/path, 4,194,304 total path bytes. Before reading admit metadata/remaining allowance: 8,388,608 bytes/file and 67,108,864 total. Limited read detects one extra byte; enumeration scratch bounded. | [`scan_bounds_precede_retention`](../../apps/runtime/src/workspace/edit_io.rs) (`apps/runtime/src/workspace/edit_io.rs:791`): exact/one-over each dimension, growing read, overflow and wide/deep trees; observe read/allocation counters, no over-bound retention. | F4 |
| T09 Complete publication baseline and bounds before retention; order 1/3 | E:771 check_admission; A:255 replacement_bytes; I:180 buffers; I:433 EditIo::new | Before copies/buffers admit 4,096 operations/64 files, one Configuration/target, 1,048,576 original/result bytes per edited document, 8,388,608 aggregate originals/results each; checked result length before allocation. Preserve existing planner/adapter bounds. | [`attempt_lifetime_and_bounds`](../../apps/runtime/src/workspace/edit.rs) (`apps/runtime/src/workspace/edit.rs:1824`), [`buffer_and_disk_bounds_precede_allocation`](../../apps/runtime/src/workspace/edit_io.rs) (`apps/runtime/src/workspace/edit_io.rs:924`): exact/one-over each dimension, oversized result and checked overflow; no allocation/stage beyond reservations. | F3, F4, F10 |
| T10 Complete publication baseline and bounds before retention | A:74 SafeEditProjectionAdmission; E:832 freeze_projection; DP:27 projector; EP:27 projector; E:1136 attempt/undo transfer | Every baseline/verification/original/result/recovery copy plus new projection records, nested strings and scratch reserves within the same 268,435,456 bytes first through A/DP/EP count/reserve/emit before staging; share unchanged buffers, promptly release scans. Only one undo; expire before next apply/other successor/stop/poison. No rejected payload retained in error. | [`buffer_and_disk_bounds_precede_allocation`](../../apps/runtime/src/workspace/edit_io.rs) (`apps/runtime/src/workspace/edit_io.rs:924`) covers simultaneous peak exact/one-over and overflow; [`attempt_lifetime_and_bounds`](../../apps/runtime/src/workspace/edit.rs) (`apps/runtime/src/workspace/edit.rs:1824`) asserts release on every terminal path including denied reversal/drop. | F4, F3 |
| T11 Filesystem ownership, staging and guarantees; order 1/2/4/5 | I:95 Unix identity; I:337 validate_path; I:367 read_checked; E:771 check_admission | Before each source read, stage, replace, restore and cleanup validate canonical Workspace/Configuration roots and all ancestors with `symlink_metadata`; reject lexical escapes, symlinks, wrong kind/outside canonical path, unprovable platform identity, nlink != 1, duplicate identities and non-bijective document/file mapping. | [`confinement_rechecked_at_every_io_boundary`](../../apps/runtime/src/workspace/edit_io.rs) (`apps/runtime/src/workspace/edit_io.rs:1157`): traversal, root/ancestor/target symlinks, inside/outside hard links, duplicate alias and path/kind swaps at each seam; outside sentinel unchanged. | F4 |
| T12 Single coordinator and guard ordering, order 2/3 | A:255 replacement_bytes; E:1136 checked increment and regenerated plan | Check next publication increment before source mutation; validate document version, expected token, bounds/nonoverlap/canonical operation order; apply descending raw offsets preserving every other byte. | [`replacement_rejects_invalid_ranges_versions_and_tokens`](../../crates/analysis/tests/safe_edit.rs) (`crates/analysis/tests/safe_edit.rs:724`): stale version, wrong token, overlap/reordering/omission, invalid byte boundary and overflow reject; [`publication_barriers_and_overflow`](../../apps/runtime/src/workspace/edit.rs) (`apps/runtime/src/workspace/edit.rs:1996`) proves exhausted successor cannot stage/write. | F1, F3 |
| T13 Filesystem ownership, staging and guarantees; order 4 | I:562 stage_all; I:433 EditIo::new; I:656 cleanup_owned | Before `create_new` reserve 16,777,216 disk bytes/128 files, bounded checked names without source extensions in same parent, private permissions; track exact created identity. Collisions never remove existing entries; no prefix sweep. Restore reuses backup slot. | [`buffer_and_disk_bounds_precede_allocation`](../../apps/runtime/src/workspace/edit_io.rs) (`apps/runtime/src/workspace/edit_io.rs:924`), [`staging_faults_preserve_sources`](../../apps/runtime/src/workspace/edit_io.rs) (`apps/runtime/src/workspace/edit_io.rs:991`): exact/one-over disk limits, name exhaustion/collision and owned-entry swap; unrelated entries survive, no source write. | F4 |
| T14 Single coordinator and guard ordering, order 4 | I:562 stage_all; I:367 read_checked; I:656 cleanup_owned | Fully write results/backups, preserve standard permissions without broadening temp access, `sync_all`, close/read back exact bytes; verify all backups before first replace. Staging failure cleans only owned artifacts. | [`staging_faults_preserve_sources`](../../apps/runtime/src/workspace/edit_io.rs) (`apps/runtime/src/workspace/edit_io.rs:991`): every create/write/permission/sync/close-observation/readback ordinal for results/backups, short/corrupt output; no replacement, successful cleanup or explicit cleanup recovery failure. | F4 |
| T15 Single coordinator and guard ordering, order 5 | I:595 replace_checked; I:308 verify_tree; I:679 restore_checked | Repeat complete original baseline and path guards immediately before first replace; recheck original bytes/kind/identity before each later file in canonical path order. Rename verified sibling over target, no pre-delete/truncate/copy fallback. Record attempt before call and classify observed result even on ambiguous error. | [`replacement_ordinals_classify_ambiguous_failure`](../../apps/runtime/src/workspace/edit_io.rs) (`apps/runtime/src/workspace/edit_io.rs:1037`): faults before/after each actual rename and later-target external edit; recovery reflects observed bytes, never false success or lost unrelated change. | F4 |
| T16 Single coordinator and guard ordering, order 6 | E:1136 run_attempt; W:1643 production build; I:308 verify_tree | Entire actual tree must equal baseline plus exact result bytes before rebuild, excluding only verified owned files. Build complete Workspace; repeat scan after build; evaluate A. No candidate cache write. | [`post_write_build_and_tree_failures_recover`](../../apps/runtime/src/workspace/edit.rs) (`apps/runtime/src/workspace/edit.rs:2222`): unrelated module/metadata mutation, added/removed root/file, edit during build; fail builder/Designer Complete/validation/rule/diagnostic composition; recover without candidate publication/cache write. | F3 |
| T17 Complete semantic oracle 1/6 | E:938 compare_snapshot; E:1512 compare_exact_snapshot; A:1464 validate_postconditions | Before commit compare all Configuration IDs/roots/formats, Module identities/owners, document IDs/roles/paths/completeness/inventory and exact changed/untouched bytes. Other Configurations exactly equal; undo uses saved original projection. | [`inventory_and_untouched_evidence_mismatch_rejects`](../../crates/analysis/tests/safe_edit.rs) (`crates/analysis/tests/safe_edit.rs:425`): missing document/occurrence evidence, extra root, changed role/path or second Configuration; [`semantic_candidate_faults_recover`](../../apps/runtime/src/workspace/edit.rs) (`apps/runtime/src/workspace/edit.rs:3043`) submits every reachable candidate mutant through production comparison. The sole `SourceEvidenceCompleteness::BslCallableRenameV1` variant is closed-type evidence, not an executable changed-marker mutant. | F1, F3 |
| T18 Complete semantic oracle 2 | A:1093 SafeEditProducerProjection::new; A:1464 validate_postconditions; Q:822 bsl_query_id | BSL-owned expected target appears once and old target disappears; compare target name/kind/Module owner and all node payloads/provenance after the callable/directly-owned-Query closure and frozen DP/EP projection. Q owns Query IDs; exact binding/text and unchanged other-callable Query remain mandatory. Exact nonidentifier bytes also preserve export/async/parameters absent from Graph payload. No unrelated node change. | [`node_and_edge_projection_rejects_unrelated_changes`](../../crates/analysis/tests/safe_edit.rs) (`crates/analysis/tests/safe_edit.rs:497`): retained old/wrong new target, changed owner/kind/export source or unrelated equal-count node/payload change; [`semantic_candidate_faults_recover`](../../apps/runtime/src/workspace/edit.rs) (`apps/runtime/src/workspace/edit.rs:3043`) rejects same mutants. | F1, F3 |
| T19 Complete semantic oracle 2 | A:1093 SafeEditProducerProjection::new; A:1464 validate_postconditions | Exhaustive endpoints/kinds/full provenance after the callable/directly-owned-Query closure and frozen producer projection; Reads/DependsOn targets remain exact; `SemanticGraphQuery::edge_id` derives IDs. Explicit provenance comparison is mandatory: `GraphEdge::eq` omits it. Include incoming/outgoing calls inside target and ownership. | [`node_and_edge_projection_rejects_unrelated_changes`](../../crates/analysis/tests/safe_edit.rs) (`crates/analysis/tests/safe_edit.rs:497`): equal-count edge swap, lost internal/outgoing call, provenance-only change; [`semantic_candidate_faults_recover`](../../apps/runtime/src/workspace/edit.rs) (`apps/runtime/src/workspace/edit.rs:3043`) prevents publication. | F1, F3 |
| T20 Complete semantic oracle 3 | A:255 replacement_bytes; A:1464 validate_postconditions; AP:2627 raw_range_to_source_span | Every occurrence in canonical document/range/kind order: exact cumulative byte deltas, replacement lengths, tokens/kinds/lexical owners/unique resolutions. Only prescribed target/range changes. Designer mappings remain required without unsupported Graph Calls edges. | [`occurrence_projection_rejects_omission_and_ambiguity`](../../crates/analysis/tests/safe_edit.rs) (`crates/analysis/tests/safe_edit.rs:669`): missing/extra same-count call, constructor-valid differing range/token/lexical-owner evidence, ambiguous/unsupported or retargeted untouched occurrence, missing Designer mapping; constructor rejection of inconsistent raw bytes/version/range is separate evidence, never credited as Runtime comparator execution; [`semantic_candidate_faults_recover`](../../apps/runtime/src/workspace/edit.rs) (`apps/runtime/src/workspace/edit.rs:3043`) checks production rejection. | F1, F3, F11 |
| T21 Complete semantic oracle 4 | DP:27 projector; EP:27 projector; A:690 SafeEditProvenance; A:1464 validate_postconditions | Freeze DP whole-module/declaration and EP nested Query/request/diagnostic provenance before stage/candidate. A independently enumerates typed before keys and rejects incomplete/duplicate/conflicting/unconsumed maps. Map before/result raw coordinates with canonical helpers; preserve path/source kind/role, producer/origin/confidence/resolution. Respect producer line-start/file-only anchors; never replace with token spans. Compare every node/edge/reference provenance record. | [`anchor_and_reference_projection_rejects_loss`](../../crates/analysis/tests/safe_edit.rs) (`crates/analysis/tests/safe_edit.rs:796`): BOM/multibyte longer/shorter names, CRLF/LF, file-only anchors, changed producer/path/span with same count; [`semantic_candidate_faults_recover`](../../apps/runtime/src/workspace/edit.rs) (`apps/runtime/src/workspace/edit.rs:3043`) blocks acceptance. | F1, F3 |
| T22 Complete semantic oracle 4 | EP:27 projector; A:825 SafeEditRequest; A:1432 validate_equivalence | Consume frozen EP complete terminal requests with dependent Query IDs and unchanged metadata; map typed references/source IDs/names/anchors; reconstruct terminal IDs via `reconstruct_terminal`; compare every category/expected kind/candidate/state/outcome/provenance. Resolved/unresolved/unsupported dispositions and statistics agree, no omitted request. | [`anchor_and_reference_projection_rejects_loss`](../../crates/analysis/tests/safe_edit.rs) (`crates/analysis/tests/safe_edit.rs:796`): lost unresolved request, altered candidate/outcome at equal statistics, canonical identity changed through a structural reference substitution or provenance change; no isolated corruption of a private derived ID; [`semantic_candidate_faults_recover`](../../apps/runtime/src/workspace/edit.rs) (`apps/runtime/src/workspace/edit.rs:3043`) rejects. | F1, F3 |
| T23 Complete semantic oracle 5 | A:883 SafeEditDiagnostic; A:1432 validate_equivalence; E:938 compare_snapshot | Require complete validation/rule/diagnostic composition; compare every existing report/status/count and complete record collection, code/severity/parameters/related evidence/anchors; no absent incomplete marker or omission-counter field is invented. Use before-bound DiagnosticFact keys and frozen canonical producer diagnostics, including malformed/unsupported Query; reconstruct canonical findings/IDs from typed inputs; unchanged messages exact, derived messages use known producer. Unclassifiable mapping fails closed, no blanket replacement/clean-project prerequisite. | [`diagnostic_and_rule_projection_rejects_non_equivalence`](../../crates/analysis/tests/safe_edit.rs) (`crates/analysis/tests/safe_edit.rs:873`): lost unrelated pre-existing finding, changed severity/message/related evidence, lost producer evidence or equal-count producer-generated rule status; use canonical whole-record/report substitutions rather than private derived-field corruption; valid mapped pre-existing diagnostics pass. [`semantic_candidate_faults_recover`](../../apps/runtime/src/workspace/edit.rs) (`apps/runtime/src/workspace/edit.rs:3043`) checks all report mutants. | F1, F3 |
| T24 Complete semantic oracle 6; order 7 | W:1323 compose_change_impact; E:1500 predecessor_matches; W:1249 send_replace | Derive impact from actual adjacent pair after complete oracle; check current predecessor Arc and successor overflow. Undo receives new publication/impact IDs, never original ID. | [`publication_barriers_and_overflow`](../../apps/runtime/src/workspace/edit.rs) (`apps/runtime/src/workspace/edit.rs:1996`): stale Arc, wrong impact pair/overflow reject or recover; [`apply_reversal_preserve_exact_bytes_and_old_arcs`](../../apps/runtime/tests/safe_edit_transactions.rs) (`apps/runtime/tests/safe_edit_transactions.rs:271`) proves old readers immutable and one increment. | F3, F2 |
| T25 Single coordinator and guard ordering, order 7/8 | I:656 cleanup_owned; I:308 verify_tree; E:1136 run_attempt; W:1235 final commit guard | Check cancellation/predecessor/full state, remove all owned stage/backup while retaining pre-reserved originals; cleanup failure recovers. Final expected source/path scan and cancellation after cleanup precede sole commit `send_replace(Some(successor))`. Same turn installs baseline/outcome/undo and invalidates preparation. No fallible source I/O or cancellation rollback after commit. | [`final_cleanup_scan_and_commit_barrier`](../../apps/runtime/src/workspace/edit.rs) (`apps/runtime/src/workspace/edit.rs:2261`): cleanup failure, edit after cleanup, cancel at final guard/just after commit; only precommit cases recover, committed success remains. | F3 |
| T26 Single coordinator and guard ordering, order 9 | W:1249 semantic commit; W:1253 serialized cache write; C:41 schema version | Only accepted successor writes cache while serialized; success delivered after bounded cache attempt. Cache failure affects cache status, never source success/count. Existing schema/version; no baseline/capability/undo persistence. | [`cache_namespace_and_scan_exclusions`](../../apps/runtime/src/workspace/edit.rs) (`apps/runtime/src/workspace/edit.rs:3759`) blocks/fails postcommit cache write without rollback; [`edit_commit_cache_failure_preserves_success`](../../apps/runtime/tests/persistent_cache.rs) (`apps/runtime/tests/persistent_cache.rs:7`) checks public status/cold restart without edit authority. | F3, F7 |
| T27 Single coordinator and guard ordering | W:1014 run_workspace_updates; W:803 startup publication; W:1144 rebuild publication; W:1358 shutdown clearing | No admission during earlier build/cache write; no watcher/explicit-input build, cache write or clearing writer alongside transaction. Coalesce watcher noise; self-write-only baseline consumes no extra ID; true external change and bounded explicit input rebuild normally. | [`publication_barriers_and_overflow`](../../apps/runtime/src/workspace/edit.rs) (`apps/runtime/src/workspace/edit.rs:1996`) controls startup/build/cache/transaction/stop; [`edit_self_write_noise_and_external_change`](../../apps/runtime/tests/file_watching.rs) (`apps/runtime/tests/file_watching.rs:7`), [`edit_serializes_explicit_change_input`](../../apps/runtime/tests/git_change_workspace.rs) (`apps/runtime/tests/git_change_workspace.rs:8`) verify public ordering. | F3, F6, F8 |
| T28 Recovery, failure precedence and reversal | I:679 restore_checked; I:595 attempted replacement journal; E:1336 failure/recovery branch | All failures after any attempted replacement enter joined recovery in reverse attempt order; restore only exact expected result identity/bytes or prove already original. Observe ambiguous failures. Stage/verify originals through same confinement; never overwrite third-party bytes/delete unknown entry. | [`recovery_ordinals_preserve_unrelated_edits`](../../apps/runtime/src/workspace/edit_io.rs) (`apps/runtime/src/workspace/edit_io.rs:1077`): each restoration/check/stage/rename ordinal, third-state bytes/kind/alias swaps; unchanged unrelated sentinels and complete outcome record. | F4 |
| T29 Recovery, failure precedence and reversal | I:679 restore_checked; I:656 cleanup_owned; E:1336 failure/recovery branch | Recovery success requires full original bytes/tree/permissions and all owned artifacts removed; retain old publication/no increment, original closed cause plus `Recovered`, no undo. Restore exact originals, not semantic reconstruction. | [`recovery_outcomes_and_precedence`](../../apps/runtime/src/workspace/edit.rs) (`apps/runtime/src/workspace/edit.rs:2283`): failure after every replace then successful restore checks full state/old Arc; remaining temp/untouched-source change cannot report `Recovered`. | F3 |
| T30 Recovery, failure precedence and reversal | E:1336 failure/recovery branch; E:557 shutdown; W:1232 quarantine clearing | Restore/original verification/cleanup failure takes precedence as `RecoveryRequired`, trigger secondary. Clear current to None; invalidate capabilities/undo; disable edits/rebuild publication/cache writes; retain bounded recovery data. Stop preserves unremovable owned files/counts. Only operator repair and new cold validated service exits. | [`recovery_outcomes_and_precedence`](../../apps/runtime/src/workspace/edit.rs) (`apps/runtime/src/workspace/edit.rs:2283`): fail recovery then edit/watch/explicit-input/cache/stop; no new publication/write, old Arcs immutable, no force/retry, no source/path secrets. | F3 |
| T31 Recovery, failure precedence and reversal | E:279 prepare_reversal; E:586 prepare; E:1136 shared run_attempt; E:1512 compare_exact_snapshot | Receipt consumed even if preparation denied; same service/exact current applied successor Arc/ID/result baseline plus fresh confirmation. Saved originals/results/original semantics, symmetric bounds/guards/rebuild/cleanup; submitted reversal consumes undo on every outcome. Failure restores applied state or quarantines. | [`stale_reversal_and_reconfirmation_reject`](../../apps/runtime/tests/safe_edit_transactions.rs) (`apps/runtime/tests/safe_edit_transactions.rs:533`): foreign/stale successor, intervening apply/rebuild, denied/replayed confirmation, changed bytes; [`reversal_failures_restore_applied_state`](../../apps/runtime/src/workspace/edit.rs) (`apps/runtime/src/workspace/edit.rs:2314`) covers shared fault ordinals and no replay. | F2, F3 |
| T32 Cancellation, shutdown and redaction | E:144 cancellation; E:557 shutdown; W:1227 joined worker; E:1336 joined recovery | Before first replace cancel cleans staging; afterwards requests uninterruptible recovery. Dropped response never drops service ownership; terminal status retained until slot release. Stop closes admission/invalidates preparation/signals work/joins worker and recovery before clearing; committed success survives. | [`cancellation_drop_and_shutdown_join`](../../apps/runtime/src/workspace/edit.rs) (`apps/runtime/src/workspace/edit.rs:3410`): cancel/drop/stop every phase, block restore and prove shutdown waits; no detached writer/late rollback. | F3, F5 |
| T33 Recovery, failure precedence and reversal; Cancellation, shutdown and redaction | E:38 closed causes; E:390 redacted Debug; E:735 policy_matches; E:994 execute; E:1336 recovery precedence | Precedence: availability, input/bounds, capability, policy, cancellation, publication/plan, source/path, staging; first error per phase except recovery override. Before output/log retention redact bytes/tokens/digests/arguments/absolute paths/nested errors; only bounded preview displays source projection. | [`closed_precedence_and_redaction`](../../apps/runtime/src/workspace/edit.rs) (`apps/runtime/src/workspace/edit.rs:3538`): multiple simultaneous failures and secret sentinels in source/path/arguments/build/I/O error; inspect Debug/errors/audit/outcome/captured logs, only closed status/counts/IDs and deterministic results. | F3, F9 |
| T34 Compatibility and affected consumers; Owners and dependency direction | U:45 additive exports; W:686 opt-in configuration; Q:822 canonical helper; DP:27 and EP:27 pure projectors | Additive opt-in Rust API only; no planner authority/new counter; preserve impact alias, eight-tool catalog, protocols/GraphQuery/diagnostics/adapter semantics/cache format. No product entry point enables edits. Pure A uses public G/L/D and admitted DP/EP/Q owners, no second planner/graph facts. Test canonical encoding and builder compatibility before/after helper extraction. | [`default_service_remains_read_only_with_edit_api`](../../apps/runtime/tests/workspace_service.rs) (`apps/runtime/tests/workspace_service.rs:7`) denies mutation; existing public consumer/paired planner suites reject unsupported/stale input and preserve observations. | F5, F10, F11, F12 |
| T35 Complete publication baseline and bounds before retention; Local API, authorization and lifetime | W:857 prepare_edit_baseline; W:868 finish_edit_baseline; W:1014 successor lifecycle; E:586 prepare | Edit-enabled initial instability fails startup; unstable successor retains predecessor/no increment. An unprovable or over-bound edit baseline never permits preparation; otherwise valid read-only evidence remains read-only. Preserve otherwise eligible Query-containing targets without blanket rejection. Every accepted nontransaction successor invalidates prepared challenge/authorization/undo; no time-based or cross-process credential exists. | [`publication_baseline_admission`](../../apps/runtime/src/workspace/edit.rs) (`apps/runtime/src/workspace/edit.rs:3566`): unstable before/after startup, rebuild and validated cache hit cannot become edit-eligible; over-bound baseline denies preparation; a watcher/explicit-input successor expires retained challenge and undo. Existing default read-only lifecycle remains valid. | F3, F5, F7 |

### Complete named-oracle reconciliation

The table below is the complete set of forty required named oracle functions.
Task 6 checked the declared source line and each reported group's exact
`test ... ok` record, independently of aggregate target success. Additional
producer and projection tests qualify T10/T16-T23; the one-MiB positive qualifies
T09/T10/T35. Group counts below include existing tests too and are not forty new
independent suites. Runtime candidate substitutions execute after the real
production build and before the production comparator, with expected projection
already frozen before staging. They never supply their own expected evidence.

| Named oracle | Exact test location | Successful focused groups |
|---|---|---|
| `anchor_and_reference_projection_rejects_loss` | [`crates/analysis/tests/safe_edit.rs:796`](../../crates/analysis/tests/safe_edit.rs) | F1, F10 |
| `apply_reversal_preserve_exact_bytes_and_old_arcs` | [`apps/runtime/tests/safe_edit_transactions.rs:271`](../../apps/runtime/tests/safe_edit_transactions.rs) | F2, F3, F5, F6, F7, F8 |
| `attempt_lifetime_and_bounds` | [`apps/runtime/src/workspace/edit.rs:1824`](../../apps/runtime/src/workspace/edit.rs) | F3 |
| `buffer_and_disk_bounds_precede_allocation` | [`apps/runtime/src/workspace/edit_io.rs:924`](../../apps/runtime/src/workspace/edit_io.rs) | F4 |
| `cache_namespace_and_scan_exclusions` | [`apps/runtime/src/workspace/edit.rs:3759`](../../apps/runtime/src/workspace/edit.rs) | F3 |
| `cancellation_drop_and_shutdown_join` | [`apps/runtime/src/workspace/edit.rs:3410`](../../apps/runtime/src/workspace/edit.rs) | F3 |
| `closed_precedence_and_redaction` | [`apps/runtime/src/workspace/edit.rs:3538`](../../apps/runtime/src/workspace/edit.rs) | F3 |
| `complete_baseline_staleness_rejects` | [`apps/runtime/tests/safe_edit_transactions.rs:492`](../../apps/runtime/tests/safe_edit_transactions.rs) | F2, F3, F5, F6, F7, F8 |
| `complete_plan_comparison_rejects_each_representable_private_field` | [`crates/analysis/src/refactoring.rs:3071`](../../crates/analysis/src/refactoring.rs) | F1, F10 |
| `confinement_rechecked_at_every_io_boundary` | [`apps/runtime/src/workspace/edit_io.rs:1157`](../../apps/runtime/src/workspace/edit_io.rs) | F4 |
| `default_service_remains_read_only_with_edit_api` | [`apps/runtime/tests/workspace_service.rs:7`](../../apps/runtime/tests/workspace_service.rs) | F5 |
| `designer_projection_preserves_canonical_provenance` | [`adapters/designer-xml/src/safe_edit.rs:305`](../../adapters/designer-xml/src/safe_edit.rs) | F11 |
| `diagnostic_and_rule_projection_rejects_non_equivalence` | [`crates/analysis/tests/safe_edit.rs:873`](../../crates/analysis/tests/safe_edit.rs) | F1, F10 |
| `disabled_unready_stopped_and_foreign_services_reject` | [`apps/runtime/tests/safe_edit_transactions.rs:359`](../../apps/runtime/tests/safe_edit_transactions.rs) | F2, F3, F5, F6, F7, F8 |
| `edit_commit_cache_failure_preserves_success` | [`apps/runtime/tests/persistent_cache.rs:7`](../../apps/runtime/tests/persistent_cache.rs) | F7 |
| `edit_self_write_noise_and_external_change` | [`apps/runtime/tests/file_watching.rs:7`](../../apps/runtime/tests/file_watching.rs) | F6 |
| `edit_serializes_explicit_change_input` | [`apps/runtime/tests/git_change_workspace.rs:8`](../../apps/runtime/tests/git_change_workspace.rs) | F8 |
| `edt_projection_preserves_nested_query_evidence` | [`adapters/edt/src/safe_edit.rs:421`](../../adapters/edt/src/safe_edit.rs) | F11 |
| `exact_one_mib_document_remains_eligible` | [`apps/runtime/tests/safe_edit_transactions.rs:702`](../../apps/runtime/tests/safe_edit_transactions.rs) | F2, F3, F5, F6, F7, F8 |
| `final_cleanup_scan_and_commit_barrier` | [`apps/runtime/src/workspace/edit.rs:2261`](../../apps/runtime/src/workspace/edit.rs) | F3 |
| `inventory_and_untouched_evidence_mismatch_rejects` | [`crates/analysis/tests/safe_edit.rs:425`](../../crates/analysis/tests/safe_edit.rs) | F1, F10 |
| `node_and_edge_projection_rejects_unrelated_changes` | [`crates/analysis/tests/safe_edit.rs:497`](../../crates/analysis/tests/safe_edit.rs) | F1, F10 |
| `occurrence_projection_rejects_omission_and_ambiguity` | [`crates/analysis/tests/safe_edit.rs:669`](../../crates/analysis/tests/safe_edit.rs) | F1, F10 |
| `policy_gate_is_exact_confirmed_and_side_effect_free` | [`apps/runtime/src/workspace/edit.rs:3323`](../../apps/runtime/src/workspace/edit.rs) | F3 |
| `post_write_build_and_tree_failures_recover` | [`apps/runtime/src/workspace/edit.rs:2222`](../../apps/runtime/src/workspace/edit.rs) | F3 |
| `producer_projection_completeness_and_bounds` | [`crates/analysis/tests/safe_edit.rs:927`](../../crates/analysis/tests/safe_edit.rs) | F1, F10 |
| `projection_freeze_precedes_io` | [`apps/runtime/src/workspace/edit.rs:1976`](../../apps/runtime/src/workspace/edit.rs) | F3 |
| `publication_barriers_and_overflow` | [`apps/runtime/src/workspace/edit.rs:1996`](../../apps/runtime/src/workspace/edit.rs) | F3 |
| `publication_baseline_admission` | [`apps/runtime/src/workspace/edit.rs:3566`](../../apps/runtime/src/workspace/edit.rs) | F3 |
| `query_containing_targets_preserve_complete_nested_evidence` | [`apps/runtime/tests/safe_edit_transactions.rs:431`](../../apps/runtime/tests/safe_edit_transactions.rs) | F2, F3, F5, F6, F7, F8 |
| `recovery_ordinals_preserve_unrelated_edits` | [`apps/runtime/src/workspace/edit_io.rs:1077`](../../apps/runtime/src/workspace/edit_io.rs) | F4 |
| `recovery_outcomes_and_precedence` | [`apps/runtime/src/workspace/edit.rs:2283`](../../apps/runtime/src/workspace/edit.rs) | F3 |
| `replacement_ordinals_classify_ambiguous_failure` | [`apps/runtime/src/workspace/edit_io.rs:1037`](../../apps/runtime/src/workspace/edit_io.rs) | F4 |
| `replacement_rejects_invalid_ranges_versions_and_tokens` | [`crates/analysis/tests/safe_edit.rs:724`](../../crates/analysis/tests/safe_edit.rs) | F1, F10 |
| `reversal_failures_restore_applied_state` | [`apps/runtime/src/workspace/edit.rs:2314`](../../apps/runtime/src/workspace/edit.rs) | F3 |
| `scan_bounds_precede_retention` | [`apps/runtime/src/workspace/edit_io.rs:791`](../../apps/runtime/src/workspace/edit_io.rs) | F4 |
| `semantic_candidate_faults_recover` | [`apps/runtime/src/workspace/edit.rs:3043`](../../apps/runtime/src/workspace/edit.rs) | F3 |
| `staging_faults_preserve_sources` | [`apps/runtime/src/workspace/edit_io.rs:991`](../../apps/runtime/src/workspace/edit_io.rs) | F4 |
| `stale_reversal_and_reconfirmation_reject` | [`apps/runtime/tests/safe_edit_transactions.rs:533`](../../apps/runtime/tests/safe_edit_transactions.rs) | F2, F3, F5, F6, F7, F8 |
| `structured_plan_and_capability_tampering_reject` | [`apps/runtime/src/workspace/edit.rs:3125`](../../apps/runtime/src/workspace/edit.rs) | F3 |

`SourceEvidenceCompleteness::BslCallableRenameV1` (`AP:652`) and
`RefactoringCompleteness::Complete` (`AP:1720`) have no alternative safe values.
These are closed-type evidence. Invalid raw UTF-8/ranges/tokens/overlaps and
forbidden Graph payload combinations are constructor evidence; they are never
counted as Runtime comparator executions. T03's representable Analysis-private
mutants call the real comparator at their owning module, with plan ID preserved.
Private Runtime slot/identity/policy mutants and public foreign-service use
exercise the reachable capability boundary without a public forge API.

Parameter coverage is separate from Cargo test totals: 64 post-build semantic
candidate mutations, 18 plan/capability cases, 110 reversal phase/I/O ordinal
cases across both formats, and 16 public Query format/disposition/Unicode
combinations. Test data derives from the tracked paired Sprint 14 corpus plus
complete Workspace descriptors and Query fixtures; the exact derivations are in
the [Workspace fixture README](../../apps/runtime/tests/fixtures/workspace_service/README.md).
Both complete production builders accept the temporary multi-file fixtures.
EDT LF and Designer BOM/CRLF byte oracles cover longer/shorter Unicode names,
declaration/local/qualified calls, unaffected bytes, exact reversal and old Arcs.

## Stable validation commands and retained logs

All paths in the command table are relative to this repository. The complete
Task 5 log directory is
`local-artifacts/codex-runs/sprint-41/task-5/producer-resume/`.
The log filename in each row expands beneath that exact directory. Full command
strings, exit statuses, per-target identities and counts are retained in
`stable-focused-results.json`, `stable-full-results.json`, and Task 6's
`local-artifacts/codex-runs/sprint-41/task-6/reconciliation.json`.
The latter was independently derived from the actual log summary/name records.

The 24 source/test/fixture paths in `stable-source-manifest.json` match their
committed blobs; its recorded SHA256 is
`402badacaa29e2e357ed44441535b3a457dd62dae250ab3874319420583668f8`.
Only the Runtime library cfg(test) candidate substitutions `query_other_callable`
and `query_owner` were finalized after F2. F1 Analysis and F2 public integration
inputs and all production files remained identical; F3 onward and the canonical
gate cover the final test source. Earlier development passes are not added to
the stable totals.

Every Cargo reproduction must use the repository root and this environment:

```bash
export TMPDIR="$PWD/local-artifacts/codex-runs/sprint-41/task-5/producer-resume/tmp"
export GIT_CEILING_DIRECTORIES="$TMPDIR"
```

| Key | Exact command | Exit | Passed / nonzero targets / zero harnesses | Full log filename |
|---|---|---:|---|---|
| F1 | `cargo test -p oneagent-analysis --lib --test safe_edit` | 0 | 81 / 2 / 0 | `stable-F1.log` |
| F2 | `cargo test -p oneagent-runtime --test safe_edit_transactions` | 0 | 6 / 1 / 0 | `stable-F2.log` |
| F3 | `cargo test -p oneagent-runtime --lib workspace::edit::tests::` | 0 | 20 / 1 / 0 | `stable-F3.log` |
| F4 | `cargo test -p oneagent-runtime --lib workspace::edit_io::tests::` | 0 | 6 / 1 / 0 | `stable-F4.log` |
| F5 | `cargo test -p oneagent-runtime --test workspace_service` | 0 | 16 / 1 / 0 | `stable-F5.log` |
| F6 | `cargo test -p oneagent-runtime --test file_watching` | 0 | 9 / 1 / 0 | `stable-F6.log` |
| F7 | `cargo test -p oneagent-runtime --test persistent_cache` | 0 | 11 / 1 / 0 | `stable-F7.log` |
| F8 | `cargo test -p oneagent-runtime --test git_change_workspace` | 0 | 10 / 1 / 0 | `stable-F8.log` |
| F9 | `cargo test -p oneagent-tool-policy --all-targets` | 0 | 33 / 2 / 0 | `stable-F9.log` |
| F10 | `cargo test -p oneagent-analysis -p oneagent-bsl --all-targets` | 0 | 221 / 12 / 0 | `stable-F10.log` |
| F11 | `cargo test -p oneagent-designer-xml -p oneagent-edt --all-targets` | 0 | 399 / 23 / 0 | `stable-F11.log` |
| F12 | `cargo test -p oneagent-runtime --test mcp_process --test mcp_semantic_tools --test lsp_stdio --test graph_query_api` | 0 | 37 / 4 / 0 | `stable-F12.log` |
| G1 | `cargo fmt --all -- --check` | 0 | not a test command | `stable-G1.log` |
| G2 | `cargo check --workspace --all-targets` | 0 | not a test command | `stable-G2.log` |
| G3 | `cargo test --workspace --all-targets` | 0 | 1435 / 83 / 4 | `stable-G3.log` |
| G4 | `cargo clippy --workspace --all-targets --all-features -- -D warnings` | 0 | not a test command | `stable-G4.log` |
| G5 | `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps` | 0 | not a test command | `stable-G5.log` |
| G6 | `git diff --check` | 0 | not a test command | `stable-G6.log` |

All focused groups executed nonzero tests and have zero ignored tests. F1 is
74 library + 7 integration tests; F9 is 26 + 7. F3 excludes 130 unrelated
library tests and F4 excludes 144; neither is a zero-match filter. F10/F11/F12
contain 12/23/4 nonzero targets. Counts overlap between commands and must not be
summed as unique coverage. G3 is 1435 passed tests in 83 nonzero targets plus
four zero-test harnesses; no tests were ignored or failed in the successful run.
The empty CLI main, Runtime main, `oneagent-mcp` and `oneagent-lsp` unit harnesses
provide no capability evidence; F12's process integration targets are nonempty.
Their exact target records and every nonzero per-target count are retained in
Task 6's reconciliation JSON.

The one planned canonical gate had an initial environment failure: G1/G2 passed,
then G3 exited 101 at unchanged
`apps/runtime/tests/git_change_reader.rs:309`, expecting `NotRepository` but
observing `WorktreeRootMismatch` because repository-local TMPDIR allowed the
non-repository fixture to discover the enclosing OneAgent Git repository.
The same eight-test target passed 8/8 with the Git ceiling above, with no source
or oracle change, followed by the full G1-G6 repeat on the unchanged source
manifest. This is not a first-attempt clean pass. The retained full failure logs
are `initial-environment-stable-G1.log`, `initial-environment-stable-G2.log`,
`initial-environment-stable-G3.log`; statuses are in
`initial-environment-stable-full-results.json`. The focused reproduction is
`git-reader-environment-repeat.log` with `environment-repeat-result.json`.
The ceiling isolates only repository-local temporary fixtures; actual fixture
repositories, subdirectory mismatch, bare/unborn/detached/linked-worktree
behavior still run. See `validation-environment.md` in the same log directory.

Task 6 reconciles these retained results; it does not run another redundant
production gate for documentation-only edits. Task 7's independent and primary
full gates remain separate required results.

## Compatibility, consumers and preserved boundaries

| Area | Audited implementation evidence and impact |
|---|---|
| Runtime API | `apps/runtime/src/lib.rs:45` additively exports `WorkspaceEditHandle`, `WorkspaceEditOwnership`, `WorkspaceEditCancellation`, `WorkspaceEditChallenge`, `WorkspaceEditAuthorization`, `WorkspaceEditReceipt`, `WorkspaceEditCause`, `WorkspaceEditRecovery`, `WorkspaceEditOutcome`. `W:686/697` adds `with_edit_policy`/`edit_handle`; `E:254/279/301/310/371` owns prepare/apply/reversal/confirmation. No existing consumer migration is required. All enabling callers are tests, including owner-local cfg(test); product main/MCP/LSP entry points remain byte-identical. F5/F12 and the canonical gate qualify existing consumers. |
| Analysis and producers | The additive `oneagent_analysis::safe_edit` module owns borrowed evidence, checked typed projections, retained lossless values, complete equality, replacement bytes and admission. Runtime consumes it and both adapters' `project_safe_edit_provenance`; adapters consume its pure admission/projection types. BSL exposes the existing canonical `bsl_query_id` encoding; extractor, Analysis and EDT projection reuse it. Shared Designer and EDT helpers preserve builder encodings; F1/F10/F11 and canonical regression tests qualify the extraction. No copied ID/provenance formatter or second planner enters Analysis/Runtime. |
| Existing planner and public observations | Existing immutable plan APIs remain intact; full structured equality and crate-local canonical coordinate/full-plan-copy helpers are shared. A plan, preview, snapshot, numeric publication ID or receipt is not authorization. `WorkspacePublicationId` and compatible `ChangeImpactPublicationId` alias remain canonical. Old Arcs remain immutable; no intermediate candidate is published. |
| Dependency direction | Exact diff contains no Cargo manifest/lockfile, Common, Graph or Tool Policy production changes, no added dependency edge/package and no unsafe block. Graph/BSL/common feed Analysis; adapters own provenance; Runtime coordinates I/O and consumes unchanged Tool Policy. Proposed extra Common/Graph APIs were unnecessary and were not added. |
| Graph and Coverage Registry | No Graph fact, semantic schema, registry entry or Supported claim changes. `adapters/edt/src/coverage.rs`, `adapters/designer-xml/src/coverage.rs` and `crates/graph/src/coverage.rs` are byte-identical. Only the selected callable and directly owned format-supported Query identities transform. Designer gains neither Query Graph facts nor unsupported Calls edges. Query binding/text, unrelated nodes/edges, terminal dispositions and diagnostics remain fully compared. This transaction is not a source-format Coverage upgrade. |
| Cache | `apps/runtime/src/workspace/cache.rs:41` keeps schema 1 and existing identity/codec. Namespace preparation is private and precedes baseline capture; exact reserved regular cache file is excluded, other entries remain inputs. Capabilities, publication baselines and undo are not persisted. Cache writes occur after semantic commit (`W:1249/1253`); F7 proves cache failure preserves committed success and cold restart has no edit authority. |
| Protocol and client catalog | `apps/runtime/src/mcp_tools.rs`, `main.rs`, `bin/oneagent-mcp.rs`, `bin/oneagent-lsp.rs` and protocol/client sources are unchanged in the exact diff. The existing eight-tool MCP catalog is unchanged; internal policy tool IDs are not catalog entries. No MCP/HTTP/CLI/LSP/IDE mutation endpoint or model-generated edit is added. F12 exercises MCP process/semantic tools, LSP stdio and GraphQuery consumers. |
| Sensitive data | Closed cause/recovery enums and redacted capability Debug (`E:38/390`) avoid retaining raw source, expected tokens, content digests, absolute paths, policy argument bytes or nested I/O/build errors in outputs. F3 redaction/precedence tests inspect sentinels across outcome, Debug, audit and captured logs; F9 exercises unchanged policy binding. Only the existing bounded preview intentionally displays source. No new telemetry or persisted source-bearing history is added. |

## Recovery outcomes, limitations and review handoff

Pre-write rejection leaves source bytes and publication unchanged, with no source
replacement. Post-write failures before semantic commit run joined recovery.
`Recovered` requires exact original complete tree/bytes/permissions and cleanup,
retains the predecessor and consumes no publication ID. If recovery, verification
or cleanup fails, `RecoveryRequired` overrides the trigger, clears current
observation to None, invalidates capabilities/undo, prevents further edit/rebuild/
cache publication, and retains bounded recovery material. The tests check
unrelated sentinels, owned artifacts, retained old Arcs and stop behavior.
Operator repair and a new cold validated service are required; there is no
force-accept or automatic retry API.

One successful apply retains one in-process reversal record. Reversal consumes
the receipt, needs the exact current applied successor and fresh confirmation,
uses saved original bytes/semantics and the same guarded rebuild/recovery path,
and publishes a new canonical ID on success. An intervening publication/apply,
stop or quarantine expires it. Failed reversal restores the applied state or
quarantines; submitted reversal has no replay or automatic retry. Cancellation
after replacement requests uninterruptible recovery. A dropped response never
drops service ownership; shutdown joins the worker/recovery. A committed result
is not rolled back by later cancellation or cache failure.

The retained execution is from this macOS host. The Unix identity branch
(`I:95`, `I:1157` test) checks device/inode/link count; non-Unix identity
(`I:107`) fails confinement before mutation. The ADR's macOS/Linux scope is not
evidence of an executed Linux, Windows or broader Unix qualification: none was
run here. No GUI-dependent validation was required by this Rust/documentation
surface. No ignored test is substituted for missing platform evidence.

The caller explicitly promises cooperative exclusive source ownership; this is
not an OS lock. Detected external changes reject, but hostile path swaps or a
write after the final check are outside the guarantee. Per-file same-directory
rename permits mixed multi-file disk visibility. No process-crash, power-loss,
hard-kill recovery, durable journal, cross-process undo or bounded shutdown-time
guarantee exists. Stalled OS I/O can delay joined shutdown. Standard permissions
are preserved; exact inode/timestamps/ACLs/xattrs/file flags are not promised.
The 268435456-byte reservation bounds new transaction-owned raw/projection
retention, not existing parser/builder/Graph internals, reader-held Arcs or total
process memory. Exact and one-over tests do not establish broad performance or
security guarantees.

Other refactoring families, metadata/file/path renames, cross-Workspace or
multi-Configuration mutation, Git/remote mutation, new wire/UI edit surfaces,
automatic model edits, durable history and Sprint 42/release execution remain
deferred. Accepted alternatives and their rejection remain in ADR-0064; this
evidence task does not reopen the mechanism.

Task 6 retained checks live under
`local-artifacts/codex-runs/sprint-41/task-6/`: `reconciliation.json`,
`reconciliation.log`, `documentation-checks.log`, `prompt-syntax.log`,
`prompt-suite.log`, `prompt-repository.log`, and `diff-check.log`.
Task 6 checks passed: the bounded documentation checker verified five approved
paths, 112 added/new Markdown links, all matrix/oracle/command entries, four
matching efficiency records and seven ordered tasks. Prompt-validator syntax,
the explicit eight-file Sprint 41 suite, the repository's 22 selected prompts,
and working/staged whitespace checks exited 0. No standalone repository
Markdown linter was found; the added/new references and sections were checked
directly. No Rust source or test changed and no production test was rerun here.
These are local artifacts, not tracked release evidence bundles. Source and
test locations and compact command outcomes are committed here; the dispatcher
must retain local logs for the fresh reviewer. The implementation manifest,
`invariant-evidence-map.md`, `stable-named-oracle-results.json`,
`prerequisite-exclusion-audit.json`, `compatibility-audit.md` and
`completion-summary.md` remain in the exact Task 5 log directory above.

The next action is the authorized no-ff implementation integration and fresh
Task 7 review, with this documentation commit as its unique prerequisite. No
production defect or missing required Task 5 result was found by this bounded
evidence reconciliation. Independent review can still discover defects; it has
not passed through this document. Push remains deferred to sprint end.
Effective context window and measured token telemetry are unavailable;
Task 6 preflight used bounded authorities with warning/narrowed selectors,
not claimed measured percentages.
