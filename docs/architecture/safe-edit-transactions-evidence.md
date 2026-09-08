# Sprint 41 Safe Edit Transaction Evidence

## Status and immutable implementation boundary

Current implementation evidence qualifies
`a78568b250201fbab35bb36928d82b3b8fb9f414`
(`Cover Sprint 41 boundary and isolation oracles`) on
`codex/v0.7-sprint-41-remediation`. This fresh documentation follow-on starts at
that clean commit. Its exact new code range is
`fc146d8802bbb82e9557a63d6c535f584a270b04..a78568b250201fbab35bb36928d82b3b8fb9f414`:
three existing Runtime paths, +507/-9 = 516 churn. Test-only observations expose
the real comparator and OS collision routes; production behavior is unchanged.
Sprint 41 remains **active**. All closure claims require a fresh independent
and primary Task 7 gate; 1461 passing implementation tests are not a review pass.
Original Task 5/6 subjects remain unique; this distinct follow-on uses
`Update Sprint 41 boundary oracle evidence`.

Original unique Task 5 `f2813d2eff5fa78efe3f0d4a705e3bc51de13979` and
Task 6 `e99a6ac14494f9b00fc2f144b01b84e402c9f7d4` are verified ancestry
prerequisites. The previous integration review of
`ceb3a91da70afde202cab84f7ea42846cdd734bd..33922bea4cbd1e9b84e67fd01a8ebc7e6c81f5a0`
was **blocked**. Isolated baseline
`fa031100ac19a98b17e676687a498bcce4e7280e` has the same tree as that reviewed
endpoint. The remediation delta is exactly
`fa031100ac19a98b17e676687a498bcce4e7280e..aaeacbfa675bd1a321f5e5c160950c6c661052d0`.
Primary identified R3-R5 (P2) and ran documentation checks only; its Cargo
completion evidence is absent. Independent identified R1/R2 (P1), R3 (P2) and
M1-M4; it completed only F1-F10 on the old tree. F11/F12 and its canonical Cargo
gate were not run. Neither review inherits the implementation's old 1435 tests.
The original findings and final process-stop update remain in
`local-artifacts/codex-runs/sprint-41/remediation/review-handoff.md`.

### Second blocked integration gate

After the earlier remediation `aaeacbfa675bd1a321f5e5c160950c6c661052d0`
and evidence `9c98e2ce9abd205bb93a79f7177649ff1c7acbdf` were integrated,
both reviewers audited all T01-T35 at the immutable range
`ceb3a91da70afde202cab84f7ea42846cdd734bd..fc146d8802bbb82e9557a63d6c535f584a270b04`
and returned **blocked** for missing oracle evidence M1-M5: actual outside-Workspace
hard link, actual `create_new` collision, changed existing second Configuration,
original-document one-over bound and production Configuration under `.oneagent`.
Neither demonstrated a production protection bypass or another concrete defect.
These second-round M1-M5 identifiers are distinct from first-round M1-M4 above.

Independent review completed F1 (82 tests/two nonempty targets) and F2
(six tests/one nonempty target), both exit 0 with no failures, ignored or empty
targets. The dispatcher runner deliberately stopped with exit 75 at
`STOP_AFTER_CURRENT` before F3; no active Cargo process was signalled.
F3-F12/G1-G5 were unexecuted and runner G6 was not reached; separate range and
working diff checks passed. Primary completed its own all-35 source audit and
documentation/Git checks but ran **no Rust commands** after the known blocker.
Both trees and endpoints stayed clean/unchanged. Neither role produced a pass
artifact, completion transition or artifact-consistency gate. Independent's
three initial selector-checker failures were corrected without repository edits.
Primary checked 243 hashes, 52 names, 133 references, four budgets and retirement
targets; syntax and suite 8/repository 22 checks passed for both roles.

The full compact handoff and retained review records are under
`local-artifacts/codex-runs/sprint-41/oracle-remediation/review-handoff.md`
and `prior-review/`. The new implementation below addresses those five evidence
gaps; it does not convert either historical blocked review into a pass.

### Reproducible cumulative scope

Keep original baseline `93661837df8d63bfed10c9b70d1986c4e0d12aa5`.
The exact prerequisite documentation range ends at
`f3c1f8c087378b78c50f7fd97499b2c2d7e5f510`; it contains ten paths,
+738/-85 = 823 churn and no production change. Original Task 5's exact range
from that prerequisite has 25 paths, +11067/-192 = 11259 churn (seven created,
eighteen modified). Remediation adds an eight-path delta, +2257/-393 = 2650.
The first remediation subtotal was 25 paths/+12973/-234 = 13207.
The latest three-path oracle delta is +507/-9 = 516; the **net cumulative
implementation is 25 paths, +13471/-234 = 13705 churn**,
with no binaries. It exceeds the 12000 estimate and remains below the authorized
32-path/20000-churn hard caps. Overlapping diffs are not added together.

`local-artifacts/codex-runs/sprint-41/oracle-remediation/scope.json` and
`local-artifacts/codex-runs/sprint-41/oracle-evidence/reconcile.py` reproduce the partition: net baseline-to-implementation on the
24 original source/test/fixture paths, plus the exact original Task 5 master
ledger delta. Never blanket-exclude the shared master path or drop carried work.
Original Task 6 is a separate five-document +418/-20 = 438 contribution. The
full unpartitioned baseline-to-`aaeacbfa` delta is 35 paths/+14118/-328 = 14446,
not the implementation subtotal. Prior evidence `9c98e2ce` is separately
five documents/+428/-125 = 553. At `a78568b2` the full unpartitioned range is
35/+14920/-329 = 15249; this five-document follow-on is separate again.
Earlier 10000-cap failures, incomplete attempts and original Task 5 success stay
historical; neither the design pass nor isolation resets accounting.

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
| T01 Local API, authorization and lifetime | W:686 with_edit_policy; W:749 service start; E:237 reserve_attempt | Default disabled, explicit cooperative ownership and immutable policy before startup; unavailable handle cannot reserve before readiness or after stop/poison. Platform eligibility precedes writes. | [`disabled_unready_stopped_and_foreign_services_reject`](../../apps/runtime/tests/safe_edit_transactions.rs) (`apps/runtime/tests/safe_edit_transactions.rs:372`): a second service at numeric publication 1 and unconfigured/new/stopped handles cannot reach I. | F2 |
| T02 Local API, authorization and lifetime; Complete publication baseline and bounds before retention | E:237 reserve_attempt; E:335 submit; E:1119 execute | Nonblocking sole prepared/queued/running slot before retaining input, plan construction or scan; checked attempt increment never wraps/reuses; release on drop/terminal path. | [`attempt_lifetime_and_bounds`](../../apps/runtime/src/workspace/edit.rs) (`apps/runtime/src/workspace/edit.rs:2023`): second request while prepared/queued/running returns `Busy`; injected exhausted counter never reuses; dropped challenge/authorization releases exactly one slot. | F3 |
| T03 Local API, authorization and lifetime | E:710 prepare; E:1259 run_attempt; A:265 compare_plan; AP:3071 owner-local test | Prepare and submit regenerate from same current Arc; compare complete request, target, preconditions, ordered operations (paths/ranges/tokens/versions/IDs/replacements), dependencies, completeness and summary before authorization retention/staging. Hash equality never substitutes for complete equality. Respect the reachability split below; no public forging API. | [`structured_plan_and_capability_tampering_reject`](../../apps/runtime/src/workspace/edit.rs) (`apps/runtime/src/workspace/edit.rs:3635`) passes every constructor-reachable same-ID structural difference (including duplicate summaries and LocalCall/QualifiedCall categories), whole-plan substitutions and Runtime-private capability mutants through the real coordinator; zero writes. [`complete_plan_comparison_rejects_each_representable_private_field`](../../crates/analysis/src/refactoring.rs) (`crates/analysis/src/refactoring.rs:3071`) preserves the plan ID while changing each representable private component and calls the real A comparator. Single-variant/type-unrepresentable states use explicit type/constructor evidence, not fabricated runtime tests. | F3, F1 |
| T04 Local API, authorization and lifetime | E:335 submit; E:435 challenge/authorization/receipt; E:861 policy_matches | Bind private Arc service identity, attempt, direction, predecessor Arc/ID, plan, baseline, frozen producer projection and immutable policy evaluation; irreversibly consume submission identity before revalidation. Capability fields private, non-cloneable, no constructor/deserializer; IDs/snapshots/receipts cannot confer authority. | [`structured_plan_and_capability_tampering_reject`](../../apps/runtime/src/workspace/edit.rs) (`apps/runtime/src/workspace/edit.rs:3635`): actor/request/service, direction/baseline/Arc substitution, double submission and replay each reject; [`disabled_unready_stopped_and_foreign_services_reject`](../../apps/runtime/tests/safe_edit_transactions.rs) (`apps/runtime/tests/safe_edit_transactions.rs:372`) submits foreign public capability. | F3, F2 |
| T05 Local API, authorization and lifetime | E:710 prepare; E:504 EditPolicyGate::execute; E:861 policy_matches; E:335 submit | Reserve bounded length-delimited arguments before allocation; exact apply/reverse ToolId, `LocalMutation`, actor/request/revision/effects/bytes. Only confirmed `RequireConfirmation` passes. Completed `execute_tool` precedes mutation queue/spawn/stage; executor performs none of those. | [`policy_gate_is_exact_confirmed_and_side_effect_free`](../../apps/runtime/src/workspace/edit.rs) (`apps/runtime/src/workspace/edit.rs:3838`): Deny, bare Allow, missing/changed/reused confirmation, changed arguments/revision/effects and gate cancellation; no I event or mutation worker. | F3, F9 |
| T06 Complete publication baseline and bounds before retention | W:857 prepare_edit_baseline; W:868 finish_edit_baseline; E:897 check_admission; I:196 capture | Before eligibility capture all directory/entry kinds and exact bytes before/after build or validated cache acceptance; require equal scans and full root/document agreement. Custom detector coverage must be provable. Never replace saved publication baseline with two later scans. | [`complete_baseline_staleness_rejects`](../../apps/runtime/tests/safe_edit_transactions.rs) (`apps/runtime/tests/safe_edit_transactions.rs:505`): alter untouched call/module, metadata, roots or unknown input after preview; two equal later scans still reject. [`publication_baseline_admission`](../../apps/runtime/src/workspace/edit.rs) (`apps/runtime/src/workspace/edit.rs:4081`) injects missing captured document/custom root. | F2, F3 |
| T07 Complete publication baseline and bounds before retention | C:284 prepare_edit_namespace; I:211 scan; E:897 check_admission; W:1014 run_workspace_updates | Finish namespace/cache maintenance before capture; directory markers remain inputs. Exclude only actual confined regular reserved cache file without source/discovery role and exact verified owned I files. Scan ignored directories/unknown `.oneagent`; serialize cache temps; leftover temp remains input. | `cache_namespace_and_scan_exclusions` (`apps/runtime/src/workspace/edit.rs:4792`) now inspects the prepared baseline for real EDT/Designer `.oneagent/configuration` roots, raw documents, descriptor and directory. `configuration_under_oneagent_survives_cache_and_transactions` (`apps/runtime/tests/safe_edit_transactions.rs:819`) exercises actual Missing/Failed cache compatibility, full rebuild, apply/undo and stale-source rejection. Existing namespace/load/write negatives remain. | F2, F3, F7 |
| T08 Complete publication baseline and bounds before retention | I:196 capture; I:211 scan; I:373 read_checked; I:162 entry_length; I:175 file admission | Before collecting/sorting each entry/path admit 16,384 entries, 4,096 bytes/path, 4,194,304 total path bytes. Before reading admit metadata/remaining allowance: 8,388,608 bytes/file and 67,108,864 total. Limited read detects one extra byte; enumeration scratch bounded. | [`scan_bounds_precede_retention`](../../apps/runtime/src/workspace/edit_io.rs) (`apps/runtime/src/workspace/edit_io.rs:827`): exact/one-over each dimension, growing read, overflow and wide/deep trees; observe read/allocation counters, no over-bound retention. | F4 |
| T09 Complete publication baseline and bounds before retention; order 1/3 | E:897 check_admission; A:283 replacement_bytes; I:186 buffers; I:460 EditIo::with_admission | Before copies/buffers admit 4,096 operations/64 files, one Configuration/target, 1,048,576 original/result bytes per edited document, 8,388,608 aggregate originals/results each; checked result length before allocation. Preserve existing planner/adapter bounds. | `attempt_lifetime_and_bounds` (`apps/runtime/src/workspace/edit.rs:2023`) and `buffer_and_disk_bounds_precede_allocation` (`apps/runtime/src/workspace/edit_io.rs:1041`) retain operation/file/result/aggregate checks. `original_one_over_document_bound_rejects_admissible_result` (`apps/runtime/src/workspace/edit_io.rs:995`) separately tests 1048576/1048577-byte originals and an admissible shorter result at the real I/O guard: exact Bounds before I/O, unchanged full baseline/bytes. Public over-bound construction is impossible, not a Runtime comparator test; public exact-limit positive remains. | F2, F3, F4, F10 |
| T10 Complete publication baseline and bounds before retention | A:74 SafeEditProjectionAdmission; E:958 freeze_projection; DP:27 projector; EP:26 projector; E:1259 attempt/undo transfer | Every baseline/verification/original/result/recovery copy plus new projection records, nested strings and scratch reserves within the same 268,435,456 bytes first through A/DP/EP count/reserve/emit before staging; share unchanged buffers, promptly release scans. Only one undo; expire before next apply/other successor/stop/poison. No rejected payload retained in error. | [`buffer_and_disk_bounds_precede_allocation`](../../apps/runtime/src/workspace/edit_io.rs) (`apps/runtime/src/workspace/edit_io.rs:1041`) covers bound arithmetic exact/one-over and overflow (real shared-lease peak and release are qualified separately under R1/R2/R4); [`attempt_lifetime_and_bounds`](../../apps/runtime/src/workspace/edit.rs) (`apps/runtime/src/workspace/edit.rs:2023`) asserts release on every terminal path including denied reversal/drop. | F4, F3 |
| T11 Filesystem ownership, staging and guarantees; order 1/2/4/5 | I:101 Unix identity; I:343 validate_path; I:373 read_checked; E:897 check_admission | Before each source read, stage, replace, restore and cleanup validate canonical Workspace/Configuration roots and all ancestors with `symlink_metadata`; reject lexical escapes, symlinks, wrong kind/outside canonical path, unprovable platform identity, nlink != 1, duplicate identities and non-bijective document/file mapping. | `confinement_rechecked_at_every_io_boundary` (`apps/runtime/src/workspace/edit_io.rs:1363`) retains internal alias/root/ancestor/target/owned-file swaps. `outside_workspace_hard_link_rejects_without_touching_alias` (`apps/runtime/tests/safe_edit_transactions.rs:748`) adds the actual sibling outside-Workspace alias and separate sentinel, both inside the repository; ConfinementUnverifiable/NotNeeded/0 preserves bytes/dev/inode/nlink, source evidence and current Arc. | F2, F4 |
| T12 Single coordinator and guard ordering, order 2/3 | A:283 replacement_bytes; E:1259 checked increment and regenerated plan | Check next publication increment before source mutation; validate document version, expected token, bounds/nonoverlap/canonical operation order; apply descending raw offsets preserving every other byte. | [`replacement_rejects_invalid_ranges_versions_and_tokens`](../../crates/analysis/tests/safe_edit.rs) (`crates/analysis/tests/safe_edit.rs:724`): stale version, wrong token, overlap/reordering/omission, invalid byte boundary and overflow reject; [`publication_barriers_and_overflow`](../../apps/runtime/src/workspace/edit.rs) (`apps/runtime/src/workspace/edit.rs:2206`) proves exhausted successor cannot stage/write. | F1, F3 |
| T13 Filesystem ownership, staging and guarantees; order 4 | I:598 stage_all; I:460 EditIo::with_admission; I:692 cleanup_owned | Before `create_new` reserve 16,777,216 disk bytes/128 files, bounded checked names without source extensions in same parent, private permissions; track exact created identity. Collisions never remove existing entries; no prefix sweep. Restore reuses backup slot. | `buffer_and_disk_bounds_precede_allocation` (`apps/runtime/src/workspace/edit_io.rs:1041`) retains exact/one-over disk admission. `staging_faults_preserve_sources` (`apps/runtime/src/workspace/edit_io.rs:1169`) captures occupied result/backup names in the baseline before staging, reaches actual create_new AlreadyExists exactly once and reports Io. One occupied-name attempt, no retries, create counts 1/2, owned retention 0/1 then zero after cleanup, no replacement and intact unowned bytes/identity/full baseline. | F4 |
| T14 Single coordinator and guard ordering, order 4 | I:598 stage_all; I:373 read_checked; I:692 cleanup_owned | Fully write results/backups, preserve standard permissions without broadening temp access, `sync_all`, close/read back exact bytes; verify all backups before first replace. Staging failure cleans only owned artifacts. | [`staging_faults_preserve_sources`](../../apps/runtime/src/workspace/edit_io.rs) (`apps/runtime/src/workspace/edit_io.rs:1169`): every create/write/permission/sync/close-observation/readback ordinal for results/backups, short/corrupt output; no replacement, successful cleanup or explicit cleanup recovery failure. | F4 |
| T15 Single coordinator and guard ordering, order 5 | I:631 replace_checked; I:314 verify_tree; I:715 restore_checked | Repeat complete original baseline and path guards immediately before first replace; recheck original bytes/kind/identity before each later file in canonical path order. Rename verified sibling over target, no pre-delete/truncate/copy fallback. Record attempt before call and classify observed result even on ambiguous error. | [`replacement_ordinals_classify_ambiguous_failure`](../../apps/runtime/src/workspace/edit_io.rs) (`apps/runtime/src/workspace/edit_io.rs:1243`): faults before/after each actual rename and later-target external edit; recovery reflects observed bytes, never false success or lost unrelated change. | F4 |
| T16 Single coordinator and guard ordering, order 6 | E:1259 run_attempt; W:1643 production build; I:314 verify_tree | Entire actual tree must equal baseline plus exact result bytes before rebuild, excluding only verified owned files. Build complete Workspace; repeat scan after build; evaluate A. No candidate cache write. | [`post_write_build_and_tree_failures_recover`](../../apps/runtime/src/workspace/edit.rs) (`apps/runtime/src/workspace/edit.rs:2432`): unrelated module/metadata mutation, added/removed root/file, edit during build; fail builder/Designer Complete/validation/rule/diagnostic composition; recover without candidate publication/cache write. | F3 |
| T17 Complete semantic oracle 1/6 | E:1064 compare_snapshot; E:1658 compare_exact_snapshot; A:1633 validate_postconditions | Before commit compare all Configuration IDs/roots/formats, Module identities/owners, document IDs/roles/paths/completeness/inventory and exact changed/untouched bytes. Other Configurations exactly equal; undo uses saved original projection. | `inventory_and_untouched_evidence_mismatch_rejects` (`crates/analysis/tests/safe_edit.rs:425`) and `semantic_candidate_faults_recover` (`apps/runtime/src/workspace/edit.rs:3553`) retain inventory/document/role/path/root and reachable candidate mutants. `existing_second_configuration_semantic_change_recovers` (`apps/runtime/src/workspace/edit.rs:3391`) adds two existing distinct Configurations, unchanged inventory and a constructor-valid role mutation after the real build. The actual unedited-Configuration comparator rejects with SemanticMismatch/Recovered/0 after replacement/restoration; positive control applies/increments once. Sole completeness variant remains type evidence. | F1, F3 |
| T18 Complete semantic oracle 2 | A:1262 SafeEditProducerProjection::new; A:1633 validate_postconditions; Q:822 bsl_query_id | BSL-owned expected target appears once and old target disappears; compare target name/kind/Module owner and all node payloads/provenance after the callable/directly-owned-Query closure and frozen DP/EP projection. Q owns Query IDs; exact binding/text and unchanged other-callable Query remain mandatory. Exact nonidentifier bytes also preserve export/async/parameters absent from Graph payload. No unrelated node change. | [`node_and_edge_projection_rejects_unrelated_changes`](../../crates/analysis/tests/safe_edit.rs) (`crates/analysis/tests/safe_edit.rs:497`): retained old/wrong new target, changed owner/kind/export source or unrelated equal-count node/payload change; [`semantic_candidate_faults_recover`](../../apps/runtime/src/workspace/edit.rs) (`apps/runtime/src/workspace/edit.rs:3553`) rejects same mutants. | F1, F3 |
| T19 Complete semantic oracle 2 | A:1262 SafeEditProducerProjection::new; A:1633 validate_postconditions | Exhaustive endpoints/kinds/full provenance after the callable/directly-owned-Query closure and frozen producer projection; Reads/DependsOn targets remain exact; Compare every input to Graph’s canonical derived edge identity without allocating duplicate NodeIds/EdgeIds; GraphEdge stores no independent ID. Explicit provenance comparison is mandatory: `GraphEdge::eq` omits it. Include incoming/outgoing calls inside target and ownership. | [`node_and_edge_projection_rejects_unrelated_changes`](../../crates/analysis/tests/safe_edit.rs) (`crates/analysis/tests/safe_edit.rs:497`): equal-count edge swap, lost internal/outgoing call, provenance-only change; [`semantic_candidate_faults_recover`](../../apps/runtime/src/workspace/edit.rs) (`apps/runtime/src/workspace/edit.rs:3553`) prevents publication. | F1, F3 |
| T20 Complete semantic oracle 3 | A:283 replacement_bytes; A:1633 validate_postconditions; AP:2627 raw_range_to_source_span | Every occurrence in canonical document/range/kind order: exact cumulative byte deltas, replacement lengths, tokens/kinds/lexical owners/unique resolutions. Only prescribed target/range changes. Designer mappings remain required without unsupported Graph Calls edges. | [`occurrence_projection_rejects_omission_and_ambiguity`](../../crates/analysis/tests/safe_edit.rs) (`crates/analysis/tests/safe_edit.rs:669`): missing/extra same-count call, constructor-valid differing range/token/lexical-owner evidence, ambiguous/unsupported or retargeted untouched occurrence, missing Designer mapping; constructor rejection of inconsistent raw bytes/version/range is separate evidence, never credited as Runtime comparator execution; [`semantic_candidate_faults_recover`](../../apps/runtime/src/workspace/edit.rs) (`apps/runtime/src/workspace/edit.rs:3553`) checks production rejection. | F1, F3, F11 |
| T21 Complete semantic oracle 4 | DP:27 projector; EP:26 projector; A:718 SafeEditProvenance; A:1633 validate_postconditions | Freeze DP whole-module/declaration and EP nested Query/request/diagnostic provenance before stage/candidate. A independently enumerates typed before keys and rejects incomplete/duplicate/conflicting/unconsumed maps. Map before/result raw coordinates with canonical helpers; preserve path/source kind/role, producer/origin/confidence/resolution. Respect producer line-start/file-only anchors; never replace with token spans. Compare every node/edge/reference provenance record. | [`anchor_and_reference_projection_rejects_loss`](../../crates/analysis/tests/safe_edit.rs) (`crates/analysis/tests/safe_edit.rs:796`): BOM/multibyte longer/shorter names, CRLF/LF, file-only anchors, changed producer/path/span with same count; [`semantic_candidate_faults_recover`](../../apps/runtime/src/workspace/edit.rs) (`apps/runtime/src/workspace/edit.rs:3553`) blocks acceptance. | F1, F3 |
| T22 Complete semantic oracle 4 | EP:26 projector; A:966 SafeEditRequest; A:1601 validate_equivalence | Consume frozen EP complete terminal requests with dependent Query IDs and unchanged metadata; map typed references/source IDs/names/anchors; reconstruct terminal IDs via `reconstruct_terminal`; compare every category/expected kind/candidate/state/outcome/provenance. Resolved/unresolved/unsupported dispositions and statistics agree, no omitted request. | [`anchor_and_reference_projection_rejects_loss`](../../crates/analysis/tests/safe_edit.rs) (`crates/analysis/tests/safe_edit.rs:796`): lost unresolved request, altered candidate/outcome at equal statistics, canonical identity changed through a structural reference substitution or provenance change; no isolated corruption of a private derived ID; [`semantic_candidate_faults_recover`](../../apps/runtime/src/workspace/edit.rs) (`apps/runtime/src/workspace/edit.rs:3553`) rejects. | F1, F3 |
| T23 Complete semantic oracle 5 | A:1040 SafeEditDiagnostic; A:1601 validate_equivalence; E:1064 compare_snapshot | Require complete validation/rule/diagnostic composition; compare every existing report/status/count and complete record collection, code/severity/parameters/related evidence/anchors; no absent incomplete marker or omission-counter field is invented. Use before-bound DiagnosticFact keys and frozen canonical producer diagnostics, including malformed/unsupported Query; reconstruct canonical findings/IDs from typed inputs; unchanged messages exact, derived messages use known producer. Unclassifiable mapping fails closed, no blanket replacement/clean-project prerequisite. | [`diagnostic_and_rule_projection_rejects_non_equivalence`](../../crates/analysis/tests/safe_edit.rs) (`crates/analysis/tests/safe_edit.rs:873`): lost unrelated pre-existing finding, changed severity/message/related evidence, lost producer evidence or equal-count producer-generated rule status; use canonical whole-record/report substitutions rather than private derived-field corruption; valid mapped pre-existing diagnostics pass. [`semantic_candidate_faults_recover`](../../apps/runtime/src/workspace/edit.rs) (`apps/runtime/src/workspace/edit.rs:3553`) checks all report mutants. | F1, F3 |
| T24 Complete semantic oracle 6; order 7 | W:1323 compose_change_impact; E:1646 predecessor_matches; W:1249 send_replace | Derive impact from actual adjacent pair after complete oracle; check current predecessor Arc and successor overflow. Undo receives new publication/impact IDs, never original ID. | [`publication_barriers_and_overflow`](../../apps/runtime/src/workspace/edit.rs) (`apps/runtime/src/workspace/edit.rs:2206`): stale Arc, wrong impact pair/overflow reject or recover; [`apply_reversal_preserve_exact_bytes_and_old_arcs`](../../apps/runtime/tests/safe_edit_transactions.rs) (`apps/runtime/tests/safe_edit_transactions.rs:284`) proves old readers immutable and one increment. | F3, F2 |
| T25 Single coordinator and guard ordering, order 7/8 | I:692 cleanup_owned; I:314 verify_tree; E:1259 run_attempt; W:1235 final commit guard | Check cancellation/predecessor/full state, remove all owned stage/backup while retaining pre-reserved originals; cleanup failure recovers. Final expected source/path scan and cancellation after cleanup precede sole commit `send_replace(Some(successor))`. Same turn installs baseline/outcome/undo and invalidates preparation. No fallible source I/O or cancellation rollback after commit. | [`final_cleanup_scan_and_commit_barrier`](../../apps/runtime/src/workspace/edit.rs) (`apps/runtime/src/workspace/edit.rs:2471`): cleanup failure, edit after cleanup, cancel at final guard/just after commit; only precommit cases recover, committed success remains. | F3 |
| T26 Single coordinator and guard ordering, order 9 | W:1249 semantic commit; W:1253 serialized cache write; C:41 schema version | Only accepted successor writes cache while serialized; success delivered after bounded cache attempt. Cache failure affects cache status, never source success/count. Existing schema/version; no baseline/capability/undo persistence. | [`cache_namespace_and_scan_exclusions`](../../apps/runtime/src/workspace/edit.rs) (`apps/runtime/src/workspace/edit.rs:4792`) blocks/fails postcommit cache write without rollback; [`edit_commit_cache_failure_preserves_success`](../../apps/runtime/tests/persistent_cache.rs) (`apps/runtime/tests/persistent_cache.rs:7`) checks public status/cold restart without edit authority. | F3, F7 |
| T27 Single coordinator and guard ordering | W:1014 run_workspace_updates; W:803 startup publication; W:1144 rebuild publication; W:1358 shutdown clearing | No admission during earlier build/cache write; no watcher/explicit-input build, cache write or clearing writer alongside transaction. Coalesce watcher noise; self-write-only baseline consumes no extra ID; true external change and bounded explicit input rebuild normally. | [`publication_barriers_and_overflow`](../../apps/runtime/src/workspace/edit.rs) (`apps/runtime/src/workspace/edit.rs:2206`) controls startup/build/cache/transaction/stop; [`edit_self_write_noise_and_external_change`](../../apps/runtime/tests/file_watching.rs) (`apps/runtime/tests/file_watching.rs:7`), [`edit_serializes_explicit_change_input`](../../apps/runtime/tests/git_change_workspace.rs) (`apps/runtime/tests/git_change_workspace.rs:8`) verify public ordering. | F3, F6, F8 |
| T28 Recovery, failure precedence and reversal | I:715 restore_checked; I:631 attempted replacement journal; E:1480 failure/recovery branch | All failures after any attempted replacement enter joined recovery in reverse attempt order; restore only exact expected result identity/bytes or prove already original. Observe ambiguous failures. Stage/verify originals through same confinement; never overwrite third-party bytes/delete unknown entry. | [`recovery_ordinals_preserve_unrelated_edits`](../../apps/runtime/src/workspace/edit_io.rs) (`apps/runtime/src/workspace/edit_io.rs:1283`): each restoration/check/stage/rename ordinal, third-state bytes/kind/alias swaps; unchanged unrelated sentinels and complete outcome record. | F4 |
| T29 Recovery, failure precedence and reversal | I:715 restore_checked; I:692 cleanup_owned; E:1480 failure/recovery branch | Recovery success requires full original bytes/tree/permissions and all owned artifacts removed; retain old publication/no increment, original closed cause plus `Recovered`, no undo. Restore exact originals, not semantic reconstruction. | [`recovery_outcomes_and_precedence`](../../apps/runtime/src/workspace/edit.rs) (`apps/runtime/src/workspace/edit.rs:2493`): failure after every replace then successful restore checks full state/old Arc; remaining temp/untouched-source change cannot report `Recovered`. | F3 |
| T30 Recovery, failure precedence and reversal | E:1480 failure/recovery branch; E:680 shutdown; W:1232 quarantine clearing | Restore/original verification/cleanup failure takes precedence as `RecoveryRequired`, trigger secondary. Clear current to None; invalidate capabilities/undo; disable edits/rebuild publication/cache writes; retain bounded recovery data. Stop preserves unremovable owned files/counts. Only operator repair and new cold validated service exits. | [`recovery_outcomes_and_precedence`](../../apps/runtime/src/workspace/edit.rs) (`apps/runtime/src/workspace/edit.rs:2493`): fail recovery then edit/watch/explicit-input/cache/stop; no new publication/write, old Arcs immutable, no force/retry, no source/path secrets. | F3 |
| T31 Recovery, failure precedence and reversal | E:296 prepare_reversal; E:710 prepare; E:1259 shared run_attempt; E:1658 compare_exact_snapshot | Receipt consumed even if preparation denied; same service/exact current applied successor Arc/ID/result baseline plus fresh confirmation. Saved originals/results/original semantics, symmetric bounds/guards/rebuild/cleanup; submitted reversal consumes undo on every outcome. Failure restores applied state or quarantines. | [`stale_reversal_and_reconfirmation_reject`](../../apps/runtime/tests/safe_edit_transactions.rs) (`apps/runtime/tests/safe_edit_transactions.rs:546`): foreign/stale successor, intervening apply/rebuild, denied/replayed confirmation, changed bytes; [`reversal_failures_restore_applied_state`](../../apps/runtime/src/workspace/edit.rs) (`apps/runtime/src/workspace/edit.rs:2651`) covers shared fault ordinals and no replay. | F2, F3 |
| T32 Cancellation, shutdown and redaction | E:144 cancellation; E:680 shutdown; W:1227 joined worker; E:1480 joined recovery | Before first replace cancel cleans staging; afterwards requests uninterruptible recovery. Dropped response never drops service ownership; terminal status retained until slot release. Stop closes admission/invalidates preparation/signals work/joins worker and recovery before clearing; committed success survives. | [`cancellation_drop_and_shutdown_join`](../../apps/runtime/src/workspace/edit.rs) (`apps/runtime/src/workspace/edit.rs:3925`): cancel/drop/stop every phase, block restore and prove shutdown waits; no detached writer/late rollback. | F3, F5 |
| T33 Recovery, failure precedence and reversal; Cancellation, shutdown and redaction | E:38 closed causes; E:461 redacted Debug; E:861 policy_matches; E:1119 execute; E:1480 recovery precedence | Precedence: availability, input/bounds, capability, policy, cancellation, publication/plan, source/path, staging; first error per phase except recovery override. Before output/log retention redact bytes/tokens/digests/arguments/absolute paths/nested errors; only bounded preview displays source projection. | [`closed_precedence_and_redaction`](../../apps/runtime/src/workspace/edit.rs) (`apps/runtime/src/workspace/edit.rs:4053`): multiple simultaneous failures and secret sentinels in source/path/arguments/build/I/O error; inspect Debug/errors/audit/outcome/captured logs, only closed status/counts/IDs and deterministic results. | F3, F9 |
| T34 Compatibility and affected consumers; Owners and dependency direction | U:45 additive exports; W:686 opt-in configuration; Q:822 canonical helper; DP:27 and EP:26 pure projectors | Additive opt-in Rust API only; no planner authority/new counter; preserve impact alias, eight-tool catalog, protocols/GraphQuery/diagnostics/adapter semantics/cache format. No product entry point enables edits. Pure A uses public G/L/D and admitted DP/EP/Q owners, no second planner/graph facts. Test canonical encoding and builder compatibility before/after helper extraction. | [`default_service_remains_read_only_with_edit_api`](../../apps/runtime/tests/workspace_service.rs) (`apps/runtime/tests/workspace_service.rs:7`) denies mutation; existing public consumer/paired planner suites reject unsupported/stale input and preserve observations. | F5, F10, F11, F12 |
| T35 Complete publication baseline and bounds before retention; Local API, authorization and lifetime | W:857 prepare_edit_baseline; W:868 finish_edit_baseline; W:1014 successor lifecycle; E:710 prepare | Edit-enabled initial instability fails startup; unstable successor retains predecessor/no increment. An unprovable or over-bound edit baseline never permits preparation; otherwise valid read-only evidence remains read-only. Preserve otherwise eligible Query-containing targets without blanket rejection. Every accepted nontransaction successor invalidates prepared challenge/authorization/undo; no time-based or cross-process credential exists. | [`publication_baseline_admission`](../../apps/runtime/src/workspace/edit.rs) (`apps/runtime/src/workspace/edit.rs:4081`): unstable before/after startup, rebuild and validated cache hit cannot become edit-eligible; over-bound baseline denies preparation; a watcher/explicit-input successor expires retained challenge and undo. Existing default read-only lifecycle remains valid. | F3, F5, F7 |

### Complete named-oracle reconciliation

The table below preserves the forty original named oracle functions and adds
twelve first-remediation and four boundary-remediation oracles: 56 unique functions. This follow-on checked each
current declaration and the reported focused group's exact
`test ... ok` record, independently of aggregate target success. Additional
producer and projection tests qualify T10/T16-T23; the one-MiB positive qualifies
T09/T10/T35. Group counts below include existing tests too and are not 52 new
independent suites. Runtime candidate substitutions execute after the real
production build and before the production comparator, with expected projection
already frozen before staging. They never supply their own expected evidence.

| Named oracle | Exact test location | Successful focused groups |
|---|---|---|
| `anchor_and_reference_projection_rejects_loss` | [`crates/analysis/tests/safe_edit.rs:796`](../../crates/analysis/tests/safe_edit.rs) | F1, F10 |
| `apply_reversal_preserve_exact_bytes_and_old_arcs` | [`apps/runtime/tests/safe_edit_transactions.rs:284`](../../apps/runtime/tests/safe_edit_transactions.rs) | F2, F3, F5, F6, F7, F8 |
| `attempt_lifetime_and_bounds` | [`apps/runtime/src/workspace/edit.rs:2023`](../../apps/runtime/src/workspace/edit.rs) | F3 |
| `buffer_and_disk_bounds_precede_allocation` | [`apps/runtime/src/workspace/edit_io.rs:1041`](../../apps/runtime/src/workspace/edit_io.rs) | F4 |
| `cache_namespace_and_scan_exclusions` | [`apps/runtime/src/workspace/edit.rs:4792`](../../apps/runtime/src/workspace/edit.rs) | F3 |
| `cancellation_drop_and_shutdown_join` | [`apps/runtime/src/workspace/edit.rs:3925`](../../apps/runtime/src/workspace/edit.rs) | F3 |
| `closed_precedence_and_redaction` | [`apps/runtime/src/workspace/edit.rs:4053`](../../apps/runtime/src/workspace/edit.rs) | F3 |
| `complete_baseline_staleness_rejects` | [`apps/runtime/tests/safe_edit_transactions.rs:505`](../../apps/runtime/tests/safe_edit_transactions.rs) | F2, F3, F5, F6, F7, F8 |
| `complete_plan_comparison_rejects_each_representable_private_field` | [`crates/analysis/src/refactoring.rs:3071`](../../crates/analysis/src/refactoring.rs) | F1, F10 |
| `confinement_rechecked_at_every_io_boundary` | [`apps/runtime/src/workspace/edit_io.rs:1363`](../../apps/runtime/src/workspace/edit_io.rs) | F4 |
| `default_service_remains_read_only_with_edit_api` | [`apps/runtime/tests/workspace_service.rs:7`](../../apps/runtime/tests/workspace_service.rs) | F5 |
| `designer_projection_preserves_canonical_provenance` | [`adapters/designer-xml/src/safe_edit.rs:305`](../../adapters/designer-xml/src/safe_edit.rs) | F11 |
| `diagnostic_and_rule_projection_rejects_non_equivalence` | [`crates/analysis/tests/safe_edit.rs:873`](../../crates/analysis/tests/safe_edit.rs) | F1, F10 |
| `disabled_unready_stopped_and_foreign_services_reject` | [`apps/runtime/tests/safe_edit_transactions.rs:372`](../../apps/runtime/tests/safe_edit_transactions.rs) | F2, F3, F5, F6, F7, F8 |
| `edit_commit_cache_failure_preserves_success` | [`apps/runtime/tests/persistent_cache.rs:7`](../../apps/runtime/tests/persistent_cache.rs) | F7 |
| `edit_self_write_noise_and_external_change` | [`apps/runtime/tests/file_watching.rs:7`](../../apps/runtime/tests/file_watching.rs) | F6 |
| `edit_serializes_explicit_change_input` | [`apps/runtime/tests/git_change_workspace.rs:8`](../../apps/runtime/tests/git_change_workspace.rs) | F8 |
| `edt_projection_preserves_nested_query_evidence` | [`adapters/edt/src/safe_edit.rs:512`](../../adapters/edt/src/safe_edit.rs) | F11 |
| `exact_one_mib_document_remains_eligible` | [`apps/runtime/tests/safe_edit_transactions.rs:715`](../../apps/runtime/tests/safe_edit_transactions.rs) | F2, F3, F5, F6, F7, F8 |
| `final_cleanup_scan_and_commit_barrier` | [`apps/runtime/src/workspace/edit.rs:2471`](../../apps/runtime/src/workspace/edit.rs) | F3 |
| `inventory_and_untouched_evidence_mismatch_rejects` | [`crates/analysis/tests/safe_edit.rs:425`](../../crates/analysis/tests/safe_edit.rs) | F1, F10 |
| `node_and_edge_projection_rejects_unrelated_changes` | [`crates/analysis/tests/safe_edit.rs:497`](../../crates/analysis/tests/safe_edit.rs) | F1, F10 |
| `occurrence_projection_rejects_omission_and_ambiguity` | [`crates/analysis/tests/safe_edit.rs:669`](../../crates/analysis/tests/safe_edit.rs) | F1, F10 |
| `policy_gate_is_exact_confirmed_and_side_effect_free` | [`apps/runtime/src/workspace/edit.rs:3838`](../../apps/runtime/src/workspace/edit.rs) | F3 |
| `post_write_build_and_tree_failures_recover` | [`apps/runtime/src/workspace/edit.rs:2432`](../../apps/runtime/src/workspace/edit.rs) | F3 |
| `producer_projection_completeness_and_bounds` | [`crates/analysis/tests/safe_edit.rs:927`](../../crates/analysis/tests/safe_edit.rs) | F1, F10 |
| `projection_freeze_precedes_io` | [`apps/runtime/src/workspace/edit.rs:2186`](../../apps/runtime/src/workspace/edit.rs) | F3 |
| `publication_barriers_and_overflow` | [`apps/runtime/src/workspace/edit.rs:2206`](../../apps/runtime/src/workspace/edit.rs) | F3 |
| `publication_baseline_admission` | [`apps/runtime/src/workspace/edit.rs:4081`](../../apps/runtime/src/workspace/edit.rs) | F3 |
| `query_containing_targets_preserve_complete_nested_evidence` | [`apps/runtime/tests/safe_edit_transactions.rs:444`](../../apps/runtime/tests/safe_edit_transactions.rs) | F2, F3, F5, F6, F7, F8 |
| `recovery_ordinals_preserve_unrelated_edits` | [`apps/runtime/src/workspace/edit_io.rs:1283`](../../apps/runtime/src/workspace/edit_io.rs) | F4 |
| `recovery_outcomes_and_precedence` | [`apps/runtime/src/workspace/edit.rs:2493`](../../apps/runtime/src/workspace/edit.rs) | F3 |
| `replacement_ordinals_classify_ambiguous_failure` | [`apps/runtime/src/workspace/edit_io.rs:1243`](../../apps/runtime/src/workspace/edit_io.rs) | F4 |
| `replacement_rejects_invalid_ranges_versions_and_tokens` | [`crates/analysis/tests/safe_edit.rs:724`](../../crates/analysis/tests/safe_edit.rs) | F1, F10 |
| `reversal_failures_restore_applied_state` | [`apps/runtime/src/workspace/edit.rs:2651`](../../apps/runtime/src/workspace/edit.rs) | F3 |
| `scan_bounds_precede_retention` | [`apps/runtime/src/workspace/edit_io.rs:827`](../../apps/runtime/src/workspace/edit_io.rs) | F4 |
| `semantic_candidate_faults_recover` | [`apps/runtime/src/workspace/edit.rs:3553`](../../apps/runtime/src/workspace/edit.rs) | F3 |
| `staging_faults_preserve_sources` | [`apps/runtime/src/workspace/edit_io.rs:1169`](../../apps/runtime/src/workspace/edit_io.rs) | F4 |
| `stale_reversal_and_reconfirmation_reject` | [`apps/runtime/tests/safe_edit_transactions.rs:546`](../../apps/runtime/tests/safe_edit_transactions.rs) | F2, F3, F5, F6, F7, F8 |
| `structured_plan_and_capability_tampering_reject` | [`apps/runtime/src/workspace/edit.rs:3635`](../../apps/runtime/src/workspace/edit.rs) | F3 |
| `every_nested_copy_allocation_is_prepaid_and_releases_on_failure` | [`crates/analysis/src/safe_edit.rs:814`](../../crates/analysis/src/safe_edit.rs) | F1, F10 |
| `every_read_ordinal_fails_through_real_io_routes` | [`apps/runtime/src/workspace/edit_io.rs:1108`](../../apps/runtime/src/workspace/edit_io.rs) | F4 |
| `nested_projection_allocations_are_prepaid_and_partial_failures_release` | [`adapters/edt/src/safe_edit.rs:441`](../../adapters/edt/src/safe_edit.rs) | F11 |
| `ordinary_rebuild_cache_keeps_admission_busy` | [`apps/runtime/src/workspace/edit.rs:4711`](../../apps/runtime/src/workspace/edit.rs) | F3 |
| `policy_rejections_never_enter_mutation_queue` | [`apps/runtime/src/workspace/edit.rs:4645`](../../apps/runtime/src/workspace/edit.rs) | F3 |
| `prepaid_resolution_preserves_unicode_contexts_and_all_terminal_outcomes` | [`adapters/edt/src/query_source_resolution.rs:784`](../../adapters/edt/src/query_source_resolution.rs) | F11 |
| `queued_cancellation_precedes_changed_predecessor` | [`apps/runtime/src/workspace/edit.rs:4373`](../../apps/runtime/src/workspace/edit.rs) | F3 |
| `queued_payload_remains_revocable_until_worker_claim` | [`apps/runtime/src/workspace/edit.rs:4501`](../../apps/runtime/src/workspace/edit.rs) | F3 |
| `retained_capabilities_release_payload_on_successor_and_stop` | [`apps/runtime/src/workspace/edit.rs:4274`](../../apps/runtime/src/workspace/edit.rs) | F3 |
| `reversal_read_ordinals_fail_closed` | [`apps/runtime/src/workspace/edit.rs:2524`](../../apps/runtime/src/workspace/edit.rs) | F3 |
| `shared_raw_projection_lease_survives_io_and_undo` | [`apps/runtime/src/workspace/edit.rs:4570`](../../apps/runtime/src/workspace/edit.rs) | F3 |
| `submit_availability_precedes_expired_or_malformed_capability` | [`apps/runtime/src/workspace/edit.rs:4441`](../../apps/runtime/src/workspace/edit.rs) | F3 |
| `existing_second_configuration_semantic_change_recovers` | [`apps/runtime/src/workspace/edit.rs:3391`](../../apps/runtime/src/workspace/edit.rs) | F3 |
| `original_one_over_document_bound_rejects_admissible_result` | [`apps/runtime/src/workspace/edit_io.rs:995`](../../apps/runtime/src/workspace/edit_io.rs) | F4 |
| `outside_workspace_hard_link_rejects_without_touching_alias` | [`apps/runtime/tests/safe_edit_transactions.rs:748`](../../apps/runtime/tests/safe_edit_transactions.rs) | F2, F3, F5, F6, F7, F8 |
| `configuration_under_oneagent_survives_cache_and_transactions` | [`apps/runtime/tests/safe_edit_transactions.rs:819`](../../apps/runtime/tests/safe_edit_transactions.rs) | F2, F3, F5, F6, F7, F8 |

`SourceEvidenceCompleteness::BslCallableRenameV1` (`AP:652`) and
`RefactoringCompleteness::Complete` (`AP:1720`) have no alternative safe values.
These are closed-type evidence. Invalid raw UTF-8/ranges/tokens/overlaps and
forbidden Graph payload combinations are constructor evidence; they are never
counted as Runtime comparator executions. T03's representable Analysis-private
mutants call the real comparator at their owning module, with plan ID preserved.
Private Runtime slot/identity/policy mutants and public foreign-service use
exercise the reachable capability boundary without a public forge API.

Parameter coverage is separate from Cargo test totals: 64 post-build semantic
candidate mutations, 18 plan/capability cases, 110 original reversal phase/I/O ordinal
cases across both formats, plus 66 real read/read_filled failure ordinals and
360 normal/recovery reversal read cases, and 16 public Query format/disposition/Unicode
combinations. Test data derives from the tracked paired Sprint 14 corpus plus
complete Workspace descriptors and Query fixtures; the exact derivations are in
the [Workspace fixture README](../../apps/runtime/tests/fixtures/workspace_service/README.md).
Both complete production builders accept the temporary multi-file fixtures.
EDT LF and Designer BOM/CRLF byte oracles cover longer/shorter Unicode names,
declaration/local/qualified calls, unaffected bytes, exact reversal and old Arcs.

## Remediation closure and new direct evidence

The following complements every T01-T35 row above; it does not replace the
accepted R/L/C/T split or count constructor rejection as Runtime execution.
All source and test positions in this document qualify `a78568b2`; the first-round corrections remain present.

| Finding and affected requirements | Current production correction and directly executed oracle |
|---|---|
| R1 / T02,T10,T31,T32,T35 | `E:673 expire_capability`, `E:660 publish_baseline`, `E:680 shutdown`: service-revocable `Arc<Mutex<Option<EditAttempt>>>` stays with caller and queued command until worker claim; a Weak coordinator handle takes/drops payload on successor/stop. `E:4274 retained_capabilities_release_payload_on_successor_and_stop` checks actual Weak raw-buffer release across repeated successors and reversal stop; `E:4501 queued_payload_remains_revocable_until_worker_claim` checks the real unserviced queue and empty payload. Logical invalidation alone is not credited. |
| R2 / T10,T18-T23 | `adapters/edt/src/bsl_graph.rs:707 capture_query_evidence` retains prepaid typed requests/diagnostics/edge-provenance vectors; duplicate merging reserves old/new overlap. `adapters/edt/src/query_source_resolution.rs:502 canonical_context` shares nonallocating count and fixed-capacity canonical emission; `:288 resolve_name_with_admission`, `:350 reserve_request_clone` and `bsl_graph.rs:1444 reserve_unresolved_call_inputs` prepay new producer arguments. `A:814 every_nested_copy_allocation_is_prepaid_and_releases_on_failure` covers 15 request + 12 diagnostic owner allocations. `EP:441 nested_projection_allocations_are_prepaid_and_partial_failures_release` covers 15 distinct quota attempts; `query_source_resolution.rs:784 prepaid_resolution_preserves_unicode_contexts_and_all_terminal_outcomes` compares legacy/admitted canonical records. |
| R3 / T27 | `E:660 publish_baseline` preserves writer Busy; `W:1014 run_workspace_updates` remains owner through ordinary rebuild cache completion. `E:4711 ordinary_rebuild_cache_keeps_admission_busy` blocks the actual cache write, checks immediate Busy and later successful apply. `apps/runtime/tests/git_change_workspace.rs:8 edit_serializes_explicit_change_input` now waits for Watching at the expected identity and asserts exact AuthorizationMismatch, preserving the controlled cache regression. |
| R4 / T10,T19 | `I:440 admission`, `I:460 with_admission`, `E:1259 run_attempt`: one lease includes baseline + two scans + two edited-byte allowances + read scratch, then transfers through frozen projection, I/O/recovery and undo. `A:1633 validate_postconditions` compares mapped source/target/kind and explicit complete provenance without duplicate owning identity scratch. `E:4570 shared_raw_projection_lease_survives_io_and_undo` observes real Query apply/reversal retained and peak charges and transfer to undo. |
| R5 / T05,T33 | `E:335 submit` completes side-effect-free `execute_tool` before `try_send`; `E:1119 execute` rechecks availability/cancellation before claim/predecessor without executing policy again. `E:4645 policy_rejections_never_enter_mutation_queue` proves denial/cancel never queues; `E:4441 submit_availability_precedes_expired_or_malformed_capability` proves stopped/poisoned/unavailable/Busy precedence; `E:4373 queued_cancellation_precedes_changed_predecessor` proves Cancelled before PublicationMismatch with no transaction I/O. |
| M1 / T03 | `AP:3071 complete_plan_comparison_rejects_each_representable_private_field` now also independently changes nested declaration document/module/configuration/version/kind/lexical owner, precondition document/version, operation configuration and both range endpoints, retaining the plan ID and calling the actual comparator. Absent fields and sole completeness variants remain explicit type evidence. |
| M2 / T08,T14-T16,T25,T28,T29,T31 | `I:1108 every_read_ordinal_fails_through_real_io_routes` injects all 66 reached read/read_filled positions; `E:2524 reversal_read_ordinals_fail_closed` replays 360 EDT/Designer normal and recovery read positions, checking exact causes/recovery/tree/observer outcomes. `I:827 scan_bounds_precede_retention` adds a real 128-level tree and constrained scanner rejection before retaining the next path, preserving wide/exact/one-over checks. R2/R4 provide separate allocation/lease evidence. |
| M3 / T33 | `E:1781 rejected` and `E:2651 reversal_failures_restore_applied_state` assert exact primary/secondary causes and RecoveryRequired override. `E:4053 closed_precedence_and_redaction` captures actual selected-worker tracing after positive `E:521 emit_worker_entry_calibration`; real nested detector failure and `I:88 io` conversion receive secret-bearing errors. Captured outcome/trace excludes secret tokens, names and absolute paths. Audit/Debug remain separate negative routes; no claim covers every process stdout/stderr stream. |
| M4 | Direct real capability, queue, producer, allocation, ordinary-cache and I/O regressions for R1-R5 above are present and executed. Closure remains subject to independent review. |

The allocation evidence has distinct scopes. Analysis instruments the actual
owner after reservation/before `try_reserve_exact`, observes granted capacity,
and injects failure at every one of the selected 15 request/12 diagnostic nested
boundaries with return to the initial lease. EDT runs three production Query
cases at five quotas: 15 quota attempts, **not 15 allocation boundaries**.
The real shared-lease test proves transfer and accounted peak, not whole-process
heap measurement. Existing canonical parsing and Graph/helper internal work stay
within ADR-0064's accepted scoped exclusion; newly retained outputs and new
producer-created constructor arguments are prepaid and are not reclassified as
excluded internals. Count passes borrow scoped results without parsing or
allocating output. Full semantic records and canonical encodings remain intact.

The local `remediation/lowercase-capacity.rs` probe compiled/executed with exit 0
and records U+0130 input 2 bytes/output 3/capacity 8/reservation 8 in
`lowercase-capacity.log`. The retained lookup output is prepaid and checked.
The arithmetic is **not proof of every internal reallocation overlap** of the
unchanged canonical lowercase helper; those scoped internals remain under the
accepted exclusion. Greek final sigma and all terminal resolution outcomes are
covered separately by the production-builder equality test.

## Boundary oracle closure and reachability

All 35 rows retain the accepted R/L/C/T split. The current local
`oracle-remediation/35-row-nonvacuity-audit.{md,json}` records each guard,
fixture/mutant, positive control and observable outcome; `35-row-mapping.json`
is only a selector index. The five second-review closures are implementation
claims verified against committed source and exact focused log records here.

- **M1 / T11:** the public transaction is prepared before a hard link is created
  in a sibling repository-owned temporary directory. Explicit assertions prove
  the alias is outside Workspace but inside repository; source/alias share
  device/inode and nlink=2. The real confinement guard (`I:101`) rejects before
  mutation. Original source evidence/current Arc and both linked byte/identity
  observations plus a separate external sentinel remain unchanged. The former
  I/O fixture's internally named `outside` was not this external-alias evidence.
- **M2 / T13:** occupied result and backup names now belong to the captured
  baseline, so `I:598 stage_all` passes tree equality before `I:545 create_owned`
  reaches `OpenOptions::create_new`. The `I:567 create_already_exists` marker
  observes the actual OS error, not an injected checkpoint error. Both negatives
  have exact Io, one collision marker, create counts 1/2, no replacement and
  owned counts 0/1 before cleanup, zero afterwards. The algorithm makes one
  occupied-name attempt with no retry/alternate name; unowned bytes/device/inode
  and the complete baseline survive. Synthetic creation failures remain separate.
- **M3 / T17:** the complete production EDT pair has two distinct existing
  Configurations and constant identities/count. After the actual candidate build,
  the test confirms the renamed target exists and the second Configuration was
  equivalent before changing only a constructor-valid document role. The real
  `E:1064 compare_snapshot` records entry at `E:1085` into `validate_equivalence`
  and completion at `E:1089`. Entry occurs once for each positive/negative;
  completion is absent only for the negative. SemanticMismatch/Recovered/0
  follows actual replacement/restoration; original disk/source evidence and old
  Arc survive. The unmutated pair applies with one publication increment.
- **M4 / T09:** original 1048577 bytes is public-constructor-unreachable:
  `SourceDocument::new` and `SourceEvidenceAdmission` already reject it. The real
  `I:460 with_admission` original-byte guard is independently executable using
  1048576/1048577-byte originals, one file, a shorter result below 1 MiB and
  admissible aggregates/baseline/raw lease. Exact limit stages and cleans two
  owned files; one-over returns Bounds before any I/O event with unchanged full
  baseline/bytes. This is I/O-owner evidence, not an impossible Runtime semantic
  publication or forged plan. Existing public exact-one-MiB positive remains.
- **M5 / T07:** real EDT/Designer roots under `.oneagent/configuration` are built
  by production discovery. `E:4792 cache_namespace_and_scan_exclusions` directly
  checks the bound prepared baseline contains directory, all raw documents and
  descriptor. The public `:819` oracle observes read-only default rejection,
  Missing cache load and Failed writes: ordinary cache scanning excludes
  `.oneagent` and its strict codec rejects incomplete source coverage, so startup
  uses complete rebuild. Apply/undo succeed despite Failed writes. A later
  `.oneagent` source mutation returns SourceChanged/NotNeeded/0 without semantic
  publication or overwriting external bytes. No successful cache Hit/write or
  cache schema change is claimed for this Configuration case.

## Current stable validation and immutable handoff

Current log root: `local-artifacts/codex-runs/sprint-41/oracle-remediation/`.
The stable source manifest has **243** inputs, SHA256
`d82154fcfd7540a376fa8bec1638ee727fc390c0d715316dfdeda373ecb8c5f5`.
This documentation task independently matched every input to its working file
and committed `a78568b2` blob, verified all **44** current/historical log hashes,
parsed every current raw test summary and matched all **56** named oracles to
current declarations and successful focused log records. Counts overlap between
commands and public fixture inclusions; no historical/focused total is added to
the canonical count. `final-git.json` binds the stable source to the clean code
commit. `validation-commands.json` and `validation-results.json` give exact
commands/statuses/target identities; `log-sha256.json` binds retained logs.

For reproduction, use the repository root and these exact paths:

```bash
export TMPDIR="$PWD/local-artifacts/codex-runs/sprint-41/oracle-remediation/tmp"
export GIT_CEILING_DIRECTORIES="$TMPDIR"
export CARGO_TARGET_DIR="$PWD/local-artifacts/codex-runs/sprint-41/remediation/target"
export RUSTDOCFLAGS="-D warnings"
```

Stable attempt 2 completed all 18 commands with exit 0:

| Key | Exact command | Passed / nonempty targets / empty targets | Full log |
|---|---|---|---|
| F1 | `cargo test -p oneagent-analysis --lib --test safe_edit` | 82 / 2 / 0 | `logs/F1.log` |
| F2 | `cargo test -p oneagent-runtime --test safe_edit_transactions` | 8 / 1 / 0 | `logs/F2.log` |
| F3 | `cargo test -p oneagent-runtime --lib workspace::edit::tests::` | 31 / 1 / 0 | `logs/F3.log` |
| F4 | `cargo test -p oneagent-runtime --lib workspace::edit_io::tests::` | 8 / 1 / 0 | `logs/F4.log` |
| F5 | `cargo test -p oneagent-runtime --test workspace_service` | 18 / 1 / 0 | `logs/F5.log` |
| F6 | `cargo test -p oneagent-runtime --test file_watching` | 11 / 1 / 0 | `logs/F6.log` |
| F7 | `cargo test -p oneagent-runtime --test persistent_cache` | 13 / 1 / 0 | `logs/F7.log` |
| F8 | `cargo test -p oneagent-runtime --test git_change_workspace` | 12 / 1 / 0 | `logs/F8.log` |
| F9 | `cargo test -p oneagent-tool-policy --all-targets` | 33 / 2 / 0 | `logs/F9.log` |
| F10 | `cargo test -p oneagent-analysis -p oneagent-bsl --all-targets` | 222 / 12 / 0 | `logs/F10.log` |
| F11 | `cargo test -p oneagent-designer-xml -p oneagent-edt --all-targets` | 401 / 23 / 0 | `logs/F11.log` |
| F12 | `cargo test -p oneagent-runtime --test mcp_process --test mcp_semantic_tools --test lsp_stdio --test graph_query_api` | 37 / 4 / 0 | `logs/F12.log` |
| fmt | `cargo fmt --all -- --check` | not a test command | `logs/fmt.log` |
| check | `cargo check --workspace --all-targets` | not a test command | `logs/check.log` |
| test | `cargo test --workspace --all-targets` | 1461 / 83 / 4 | `logs/test.log` |
| clippy | `cargo clippy --workspace --all-targets --all-features -- -D warnings` | not a test command | `logs/clippy.log` |
| doc | `cargo doc --workspace --no-deps` | not a test command | `logs/doc.log` |
| diff | `git diff --check` | not a test command | `logs/diff.log` |

Canonical test: **1461 passed / 83 nonempty targets / four empty unit targets**,
zero failed/ignored. The empty targets are CLI `src/main.rs`, Runtime
`src/main.rs`, `src/bin/oneagent-mcp.rs` and `src/bin/oneagent-lsp.rs`; exact
harness identities are retained in `validation-summary.md` and reconciliation.
They supply no capability credit. Every focused group is nonempty. No Rust
command is rerun for this five-document follow-on.

**Retained latest failures:** `logs/dev-public-1.log` exited 101 (7 passed,
2 failed): the oversized public fixture was rejected by the source constructor,
and the `.oneagent` cache success/Hit expectation contradicted the existing
codec. Corrected owner/cache assertions passed (8 public tests); no production
protection bypass or relaxed constructor was introduced. `logs/dev-clippy-1.log`
retains the redundant-closure lint (exit 101); the `SourceDocument::occurrences`
replacement and focused retry passed. `development-history.json` preserves all
nine development/retry commands.

The first stable cycle passed F1-F12/fmt/check, then canonical test exited 101
(1293 passed, one failed, zero ignored before Cargo stopped) in the unchanged
watcher test's generic five-second update wait at
`apps/runtime/tests/file_watching.rs:256`. Exact caller/cause is unknown because
no backtrace was captured; a watcher coalescing race is only a hypothesis.
Clippy/doc/diff were not reached. `stable-attempt-1/` retains its commands,
results, logs and identical manifest. The exact focused watcher retry passed,
then the complete second cycle above passed on unchanged source; no timeout,
assertion or source adjustment was made. This is not first-attempt success.
Older failures below and both blocked review gates remain separate histories.

The latest three-path diff changes no production API, dependency, Graph/Common,
Coverage registry, cache schema or wire/client catalog. It adds test-only event
observations and four named tests, extending the existing baseline/collision
oracles; the same OpenOptions result still receives the same I/O conversion.
The earlier additive public compatibility audit below remains applicable.
Current consumers are qualified by F5/F7/F10-F12 and this implementation cycle,
not by the incomplete historical reviewer runs. Only this macOS host ran.

Compact documentation-task outputs are under
`local-artifacts/codex-runs/sprint-41/oracle-evidence/`: `reconciliation.json`,
`reconciliation.log`, `documentation-checks.json`, `documentation-checks.log`,
`prompt-syntax.log`, `prompt-suite.log`, `prompt-repository.log`, `diff-check.log`
and `final-git.json`. The first local reconciliation attempt incorrectly looked
for a new `.oneagent` test name in E; the baseline assertions extend an existing
oracle. Correcting that selector yielded 56 unique names, with no source change;
`reconciliation-attempt-1.log` preserves the failure. Context window/telemetry
are unknown/unavailable; narrowed-selector admission is warning. Documentation
checks cover exact paths/anchors/locations, all 35 rows, four matching budget
records and seven manifest tasks; prompt syntax, explicit suite 8, repository 22
and working/staged/committed whitespace checks qualify only these documents.
A fresh Task 7 must audit an immutable integrated endpoint including this
separate documentation commit, with independent and primary validation plus
artifact consistency. Sprint 41 stays active; no retirement/release/merge/push
or completion action occurs here.

## Historical first-remediation validation and handoff

The following historical subsection qualifies `aaeacbfa`, not the current head.
Its 1447 tests and 52 names remain separate first-remediation results; the
current 1461/56 evidence above supersedes their role as current implementation
inputs without rewriting either failed review or prior validation history.

The retained current log root is
`local-artifacts/codex-runs/sprint-41/remediation/`. `validation-commands.json`
and `validation-results.json` contain full commands, statuses and target counts;
`log-sha256.json` verifies 50 retained current/historical logs. All 243 Rust/Cargo
inputs match both current source and `stable-source-manifest.json`, SHA256
`dcd2e411c611e7fc8bef959b5e46fb18c398d595c503038ec7493b7ffb1a017e`.
The runner checked the manifest before every command. `final-git.json` binds
these inputs to committed `aaeacbfa` and its eight changed source blob hashes.
The follow-on independently reconciled raw log summaries and 52 exact named
`test ... ok` records in the focused groups; no historical count was added.

Every Cargo reproduction uses the repository root and the exact environment:

```bash
export TMPDIR="$PWD/local-artifacts/codex-runs/sprint-41/remediation/tmp"
export GIT_CEILING_DIRECTORIES="$TMPDIR"
export CARGO_TARGET_DIR="$PWD/local-artifacts/codex-runs/sprint-41/remediation/target"
```

All 18 commands below exited 0. The test columns are passed / nonzero targets /
zero harnesses; non-test commands have no test claim. Log filenames expand
under the current log root, not the historical Task 5 directory.

| Key | Exact command | Passed / targets / zero | Full log |
|---|---|---|---|
| F1 | `cargo test -p oneagent-analysis --lib --test safe_edit` | 82 / 2 / 0 | `logs/F1.log` |
| F2 | `cargo test -p oneagent-runtime --test safe_edit_transactions` | 6 / 1 / 0 | `logs/F2.log` |
| F3 | `cargo test -p oneagent-runtime --lib workspace::edit::tests::` | 28 / 1 / 0 | `logs/F3.log` |
| F4 | `cargo test -p oneagent-runtime --lib workspace::edit_io::tests::` | 7 / 1 / 0 | `logs/F4.log` |
| F5 | `cargo test -p oneagent-runtime --test workspace_service` | 16 / 1 / 0 | `logs/F5.log` |
| F6 | `cargo test -p oneagent-runtime --test file_watching` | 9 / 1 / 0 | `logs/F6.log` |
| F7 | `cargo test -p oneagent-runtime --test persistent_cache` | 11 / 1 / 0 | `logs/F7.log` |
| F8 | `cargo test -p oneagent-runtime --test git_change_workspace` | 10 / 1 / 0 | `logs/F8.log` |
| F9 | `cargo test -p oneagent-tool-policy --all-targets` | 33 / 2 / 0 | `logs/F9.log` |
| F10 | `cargo test -p oneagent-analysis -p oneagent-bsl --all-targets` | 222 / 12 / 0 | `logs/F10.log` |
| F11 | `cargo test -p oneagent-designer-xml -p oneagent-edt --all-targets` | 401 / 23 / 0 | `logs/F11.log` |
| F12 | `cargo test -p oneagent-runtime --test mcp_process --test mcp_semantic_tools --test lsp_stdio --test graph_query_api` | 37 / 4 / 0 | `logs/F12.log` |
| fmt | `cargo fmt --all -- --check` | not a test command | `logs/fmt.log` |
| check | `cargo check --workspace --all-targets` | not a test command | `logs/check.log` |
| test | `cargo test --workspace --all-targets` | 1447 / 83 / 4 | `logs/test.log` |
| clippy | `cargo clippy --workspace --all-targets --all-features -- -D warnings` | not a test command | `logs/clippy.log` |
| doc | `cargo doc --workspace --no-deps` | not a test command | `logs/doc.log` |
| diff | `git diff --check` | not a test command | `logs/diff.log` |

F1 is 75 library + 7 integration tests; F9 is 26 + 7. All focused groups are
nonempty. Canonical test is **1447 passed / 83 nonzero targets / 4 zero
harnesses**, zero failed/ignored. The empty CLI main, Runtime main, MCP binary
and LSP binary harnesses are not capability evidence. Focused groups overlap;
never add them to the workspace count. No Cargo command was rerun for this
five-document follow-on.

Historical stable attempt 1 remains in `stable-attempt-1/`: F1-F7 passed,
F8 failed 9/1 because its test treated publication as ordinary update completion.
The subsequent lifecycle-aware test change is explicit; final attempt 2 above
qualifies the corrected source. `development-history.json`, `logs/dev-*.log`
and `closure.md` preserve earlier harness calibration, type/lint corrections
and canonical-constructor test rejection. The original Task 5 Git-fixture
failure and 1435-test successful repeat remain below as historical results.
Only macOS was executed; no Linux/Windows or GUI validation claim is made.

Current consumer audit compares exact original/cumulative/remediation diffs:
Runtime exports and signatures, public planner/source types, Graph/Common/Tool
Policy production, Cargo manifests/lockfile, Coverage registries, protocol and
client catalog, product entry points and cache schema remain unchanged by the
eight-path remediation. Analysis additively exposes `SafeEditProvenance::source`,
`SafeEditRequest::{source_node,same_resolution_as}` and
`SafeEditDiagnostic::same_payload_as`; these borrowed access/comparison helpers
serve the admitted EDT producer/projector. Existing counting, admission and
typed-copy APIs remain compatible, with no API removal or migration. Graph/BSL/common -> Analysis -> adapters -> Runtime stays intact.
Only tests enable the local Rust edit API. Existing cache schema 1/eight MCP
tools/read-only defaults and exact canonical Graph semantics remain unchanged;
F5/F7/F10-F12 and the current canonical gate qualify those consumers.

The local follow-on evidence root is
`local-artifacts/codex-runs/sprint-41/remediation-evidence/`: the compact summary,
source/log/named-oracle reconciliation, scope reproduction, API/consumer audit
and documentation checks identify this start commit and its eventual documentation
commit. Measured context telemetry/effective window are unavailable; preflight
used a warning and bounded selectors, not a numerical headroom claim.
The follow-on documentation checker passed five allowed paths, 275 Markdown
references (five anchors), 35 rows/123 current production references/52 named
oracles, four consistent efficiency records and seven manifest tasks.
`bash -n scripts/validate-codex-prompts.sh`, the explicitly selected eight-file
Sprint 41 suite, repository validator (22 files) and `git diff --check` exited 0.
Its first local checker attempt failed because it expected the matrix's prose
budget in machine-record syntax; the corrected checker passed without changing
the accepted budget. That local failure log is preserved separately.
Task 7 must review the new immutable integrated endpoint independently and run
its own primary/completion validation and artifact consistency. No completion,
previous-suite retirement, merge, push, tag or release eligibility follows here.

## Historical Task 5 validation commands and retained logs

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
| Runtime API | `apps/runtime/src/lib.rs:45` additively exports `WorkspaceEditHandle`, `WorkspaceEditOwnership`, `WorkspaceEditCancellation`, `WorkspaceEditChallenge`, `WorkspaceEditAuthorization`, `WorkspaceEditReceipt`, `WorkspaceEditCause`, `WorkspaceEditRecovery`, `WorkspaceEditOutcome`. `W:686/697` adds `with_edit_policy`/`edit_handle`; `WorkspaceEditHandle::{prepare_apply,prepare_reversal,checked_apply,checked_reversal}` and `WorkspaceEditChallenge::confirm` own preparation/submission/confirmation. No existing consumer migration is required. All enabling callers are tests, including owner-local cfg(test); product main/MCP/LSP entry points remain byte-identical. F5/F12 and the canonical gate qualify existing consumers. |
| Analysis and producers | The additive `oneagent_analysis::safe_edit` module owns borrowed evidence, checked typed projections, retained lossless values, complete equality, replacement bytes and admission. Runtime consumes it and both adapters' `project_safe_edit_provenance`; adapters consume its pure admission/projection types. BSL exposes the existing canonical `bsl_query_id` encoding; extractor, Analysis and EDT projection reuse it. Shared Designer and EDT helpers preserve builder encodings; F1/F10/F11 and canonical regression tests qualify the extraction. No copied ID/provenance formatter or second planner enters Analysis/Runtime. |
| Existing planner and public observations | Existing immutable plan APIs remain intact; full structured equality and crate-local canonical coordinate/full-plan-copy helpers are shared. A plan, preview, snapshot, numeric publication ID or receipt is not authorization. `WorkspacePublicationId` and compatible `ChangeImpactPublicationId` alias remain canonical. Old Arcs remain immutable; no intermediate candidate is published. |
| Dependency direction | Exact diff contains no Cargo manifest/lockfile, Common, Graph or Tool Policy production changes, no added dependency edge/package and no unsafe block. Graph/BSL/common feed Analysis; adapters own provenance; Runtime coordinates I/O and consumes unchanged Tool Policy. Proposed extra Common/Graph APIs were unnecessary and were not added. |
| Graph and Coverage Registry | No Graph fact, semantic schema, registry entry or Supported claim changes. `adapters/edt/src/coverage.rs`, `adapters/designer-xml/src/coverage.rs` and `crates/graph/src/coverage.rs` are byte-identical. Only the selected callable and directly owned format-supported Query identities transform. Designer gains neither Query Graph facts nor unsupported Calls edges. Query binding/text, unrelated nodes/edges, terminal dispositions and diagnostics remain fully compared. This transaction is not a source-format Coverage upgrade. |
| Cache | `apps/runtime/src/workspace/cache.rs:41` keeps schema 1 and existing identity/codec. Namespace preparation is private and precedes baseline capture; exact reserved regular cache file is excluded, other entries remain inputs. Capabilities, publication baselines and undo are not persisted. Cache writes occur after semantic commit (`W:1249/1253`); F7 proves cache failure preserves committed success and cold restart has no edit authority. |
| Protocol and client catalog | `apps/runtime/src/mcp_tools.rs`, `main.rs`, `bin/oneagent-mcp.rs`, `bin/oneagent-lsp.rs` and protocol/client sources are unchanged in the exact diff. The existing eight-tool MCP catalog is unchanged; internal policy tool IDs are not catalog entries. No MCP/HTTP/CLI/LSP/IDE mutation endpoint or model-generated edit is added. F12 exercises MCP process/semantic tools, LSP stdio and GraphQuery consumers. |
| Sensitive data | Closed cause/recovery enums and redacted capability Debug (`E:38`/`E:461`) avoid retaining raw source, expected tokens, content digests, absolute paths, policy argument bytes or nested I/O/build errors in outputs. F3 redaction/precedence tests inspect sentinels across outcome, Debug, audit and actual selected-worker tracing (not all process stdout/stderr); F9 exercises unchanged policy binding. Only the existing bounded preview intentionally displays source. No new telemetry or persisted source-bearing history is added. |

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
(`I:101`, `I:1363` test) checks device/inode/link count; non-Unix identity
(`I:113`) fails confinement before mutation. The ADR's macOS/Linux scope is not
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

Historical original Task 6 retained checks live under
`local-artifacts/codex-runs/sprint-41/task-6/`: `reconciliation.json`,
`reconciliation.log`, `documentation-checks.log`, `prompt-syntax.log`,
`prompt-suite.log`, `prompt-repository.log`, and `diff-check.log`.
Original Task 6 checks passed: the bounded documentation checker verified five approved
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

The next action is dispatcher-owned remediation integration and a fresh Task 7
review of the immutable endpoint including this follow-on documentation commit.
Original Task 5/6 prerequisites remain unique. Both prior blocked gates are not
superseded by an implementation or documentation pass: closure claims must be
independently reviewed. Push remains deferred to sprint end. Effective context
window and measured token telemetry are unavailable; bounded-selector admission
is a warning, not a measured percentage.
