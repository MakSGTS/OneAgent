# Sprint 41 Safe Edit Transaction Design Review

## Current controlled-unwind design decision

Decision: **pass**, for design only, on the immutable full range
`ceb3a91da70afde202cab84f7ea42846cdd734bd..d328d8638bab12c5ebc8fe2591d21d615be115a8`.
The correction delta is
`38a9bde3407f151e2c17b380e8bd28252c5a39f9..d328d8638bab12c5ebc8fe2591d21d615be115a8`.
Primary `/root/unwind_design_primary` and independent reviewer
`/root/unwind_design_independent` used separate fresh contexts and observed
the same initial/final review HEAD
`d328d8638bab12c5ebc8fe2591d21d615be115a8`, clean on
`codex/v0.7-sprint-41-remediation`. Exactly one ancestry commit has subject
`Define Sprint 41 controlled unwind recovery`, at that endpoint. The correction
changes six documentation paths, +399/-17 = 416 text churn, with no binary,
production, manifest or reusable-framework change.

The independent result arrived before this sole-artifact edit: **pass**, no
blocking or non-blocking design findings, and no missing design-owner or
negative-oracle specification. The primary independently inspected the same
authorities and relevant owners and agrees without unresolved disagreement.
This records planned architecture, not implemented conformance or execution of
the seven new unwind oracles. The same independent reviewer
`/root/unwind_design_independent` subsequently returned **artifact consistency:
pass**, with no weakened-report discrepancy. It confirmed all 35 rows, both
ranges, unique prerequisite, accounting, seven missing-evidence/risk items,
separate checks, initial selector error/correction, explicit zero-match search,
unexecuted Cargo and all detector/recovery/commit/shutdown/redaction boundaries.
It verified the historical body byte-identical to HEAD, three new links/selectors
valid and `git diff --check` exit 0. HEAD and branch stayed unchanged; the sole
review artifact then contained +181/-0. Only this consistency metadata and next
action are updated after that check; implementation still requires the committed
pass.

The [current continuation gate](../codex/prompts/sprint-41-safe-edit-transactions/00-sprint-41-execution-loop.md#current-controlled-unwind-continuation-gate)
overrides historical child prerequisite routing only. Only the separately
committed `Approve Sprint 41 controlled unwind recovery design` pass admits the
three-file remediation. Producer pass `f25388cd8073bcd228c8eaa951ef1c0178907431`
and every earlier decision below remain historical. Renewed user agreement
supersedes awaiting-agreement text, not failures, missing evidence or reviewer
disagreement. In particular the fifth blocked review retains the primary's P2
contract/evidence recommendation and the independent reviewer's conditional
owner-local concern without a confirmed P2. Historical partial checks and
watcher/startup timeout causes are not reexecuted or diagnosed by this gate.
The rejected exploratory probe was never compiled/executed and was not retried.

### Complete controlled-unwind acceptance and reconciliation

Both roles accepted all 35 rows of the
[controlled-unwind audit](../architecture/safe-edit-transactions-invariants.md#controlled-unwind-ownership-and-complete-audit),
preserving each original R/L/C/T classification and reachable negative oracle.
The original matrix and amendment each contain T01-T35 exactly once in order.
Constructor/type evidence never substitutes for an executable reachable case.

| Rows | Independently reconciled design acceptance |
|---|---|
| T01-T05 | Opt-in/liveness/identity, sole retained reservation and terminal release, complete plan/type split, consumed capabilities and confirmation before the mutation queue remain ordered before retained mutation. |
| T06-T10 | Original publication baseline, exact cache/owned-entry exclusions, admission before allocation, all source/result bounds and shared projection/undo leases survive partial failure without extra unreserved retention. |
| T11-T15 | Confinement at every boundary; overflow/version/token/order checks; prepaid immediate created-entry registration; all staging/backups verified before replacement; attempted marker before rename and observed transition before the next injectable boundary. Unknown identity grants no cleanup or exclusion authority. |
| T16-T19 | Retained I/O survives the actual post-write builder/comparator boundary; complete inventory and closed completeness-type evidence remain; frozen node/directly-owned-Query projection and full edge provenance comparison are unchanged. |
| T20-T23 | Complete occurrence/raw/lexical evidence, before-bound canonical producer mappings, whole terminal requests/Query IDs and full diagnostic/rule reports retain constructor-valid substitutions and every original negative case. |
| T24-T27 | Actual adjacent publication pair, fallible material/cleanup/final scan before commit, backup recreation on abandonment, committed success through cache failure and rejoined envelope with serialized writers. |
| T28-T30 | Shared checked reverse finalizer and separate recovery catch; complete original bytes/tree/permissions and zero artifacts before recovery success; quarantine retains bounded material/exact counts, closed secondary cause and None observation on recovery error/unwind. |
| T31-T35 | Symmetric reversal and receipt consumption, retained/dropped responses with joined stop, closed phase mapping and transaction-only redaction, unchanged APIs/dependencies and lifecycle expiry without blanket Query rejection. |

Source inspection confirmed the correction's premise: `edit.rs::run_attempt`
has local optional I/O at line 1292; `execute` and `abandon_commit` use Result-only
finalization; `mod.rs:1227` moves the coordinator into a joined worker whose
join-error mapping loses it. `mod.rs:1249` publishes before `edit.rs:1559`
constructs undo/outcome. These are source observations, not an executed
post-write panic reproduction. The actual I/O owners are `edit_io.rs` functions
`create_owned`, `replace_checked`, `cleanup_owned`, `restore_checked` and
`restore_one` at lines 547, 637, 698, 721 and 746 of the reviewed endpoint.

The [ADR amendment](../adr/0064-safe-edit-transactions.md#controlled-transaction-owner-unwind)
retains coordinator, attempt/reservation, response, phase, optional I/O and
prepared terminal/commit material outside narrow synchronous catches. Workers
return that envelope; the lifecycle loop alone owns the snapshot sender.
Preparation, consumed submission, mutation/build/comparison and precommit
material construction enter shared checked finalization. Recovery borrows the
retained owner in a separate catch and quarantines without retry on error or
unwind. Removed backups can be recreated from prepaid originals. Apply,
reversal and abandoned commit share those rules.

Preparation/admission unwind maps to `Unavailable`, build/comparison/impact to
`SemanticMismatch`, confined I/O/precommit material construction to `IoFailed`.
Recovery failure overrides with `RecoveryRequired`/`Required` and the original
closed trigger as secondary; later cancellation cannot replace that trigger.
All fallible outcome/receipt/undo preparation precedes the final guard and
`send_replace`. The same non-awaiting coordinator turn transfers prepared
baseline/undo/status; postcommit cache failure, response drop or stop cannot
cause rollback. Shutdown joins mutation and recovery before owner release or
healthy observation clearing, including when recovery is blocked.

UP/US/UM/UR/UL/UC in `edit.rs::tests` and UI in `edit_io.rs::tests` have exact
planned names and F3/F4 owners in the matrix. They must trace actual named
operations with earlier guards executed, paired EDT/Designer apply/reversal
controls and actual post-write builder faults; an earlier synthetic checkpoint
cannot qualify a later boundary. Private `WorkspaceService::with_builder`
means detector injection is L evidence even though the builder's `with_detector`
is public. It establishes neither public-service injection nor a default-input
panic trigger. All 30 original unique A/AP/E/I/R oracle names were located by
the independent reviewer; existence alone does not prove execution.

Frozen typed producer inventories, exactly-once consumption, canonical DP/EP/Q
ownership, parser/output lifetime separation, preallocation arithmetic within
268435456 and the exact 1MiB positive remain intact. No producer, public API,
dependency, protocol, cache-schema or Coverage change is admitted. The correction
fits the three planned Runtime owners: `edit.rs`, `edit_io.rs` and `mod.rs`.

Accounting retains baseline `93661837df8d63bfed10c9b70d1986c4e0d12aa5`, 24
source/fixture paths and only original Task 5 shared-master delta
`f3c1f8c087378b78c50f7fd97499b2c2d7e5f510..f2813d2eff5fa78efe3f0d4a705e3bc51de13979`
(+1/-1). The independent audit reproduced 25 paths, +14238/-234 = 14472,
no binaries, leaving 5528 below the 20000 hard cap. The current 25-path estimate
sums to 12000; historical 8500/10000 figures below are superseded. The additional
1000-1800 source and separate 300-700 seven-document estimates do not reset
accounting. Caps remain 32 paths/20000 churn/no binaries, with exact-range
documentation exclusions and no blanket shared-master subtraction.

### Separate controlled-unwind documentation validation

All commands ran from the repository root. Each role performed its own checks;
primary local artifacts are under
`local-artifacts/codex-runs/sprint-41/unwind-remediation/design-primary/`.
The independent reviewer created no files/logs; the dispatcher retained its
compact returned report separately. The table distinguishes differing scopes.

| Check | Independent reviewer | Primary |
|---|---|---|
| Root/status/branch/HEAD and unique prerequisite ancestry | exit 0; exact clean endpoint, one match | exit 0; same |
| `git diff --check` for full immutable range, correction range and working tree | each exit 0 | each exit 0 |
| `bash -n scripts/validate-codex-prompts.sh` | exit 0 | exit 0 |
| `scripts/validate-codex-prompts.sh docs/codex/prompts/sprint-41-safe-edit-transactions/*.md` (eight explicitly expanded paths) | exit 0; 8 files | exit 0; 8 files |
| `scripts/validate-codex-prompts.sh` | exit 0; 22 files | exit 0; 22 files |
| Read-only Markdown/link/selector audit | 466 local references; zero flagged; four efficiency records match | nine added correction links/anchors resolve across six changed files; prompt validator verifies efficiency selectors |
| Matrix/allocation/preserved suites | both 35-row inventories; exact accounting above; 11/4/8 prompts | both ordered 35-row inventories; 25 allocations sum to 12000; 11/4/8 tracked prompts |

The independent initial ad hoc efficiency selector stopped Roadmap at a deeper
heading and printed four false comparisons; the corrected delimiter produced
four matches. This is an audit-selector mistake, not a repository defect.
Its `rg -n 'fn controlled_unwind_' apps/runtime/src/workspace/edit.rs apps/runtime/src/workspace/edit_io.rs`
and containing batch exited 1 with zero matches: all seven new functions remain
absent/planned/unexecuted. No test pass follows. Changed-path production,
manifest and reusable-framework queries found zero paths. No dedicated Markdown
checker was discovered in scoped scripts; explicit link/selector checks apply.
Neither role ran F1-F12, canonical Cargo validation, probes or panic reproduction.

Primary exact command arguments and outcomes are retained in `validation.json`.
SHA-256 of `syntax.log` and `diff.log` is
`e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`;
`suite.log`: `97e195843b8803ec5485b13b02819112a47e3d06e1175b14b139f27175e61124`;
`repository.log`: `bef4042bc4e99cbf365437427ebd45ed6d28716d61ab17b76a7e7d0d1dbc7c47`;
`documentation-audit.log`: `b9d946b29c38ad41993af9e51df81f2132fd5a7e66953e7cc2cc200390d6ad8a`.
Effective context window and measured telemetry are unavailable; both roles
used bounded selectors. Primary preflight admitted the bounded manifest.

### Missing implementation evidence and residual risks

These are nonblocking for design only and remain required before completion:

1. All seven new named functions remain absent/unexecuted. Require actual
   boundary traces, paired format/direction controls, recovery unwind and
   retained/dropped-response shutdown proof.
2. Current source does not implement the envelope, complete containment or
   prepared commit transfer. This design pass is not source acceptance.
3. No default-input trigger or executed post-write reproduction exists. The
   private detector oracle remains owner-local and must reach the real builder.
4. Historical partial validations do not qualify new source. Watcher/startup
   timeout causes remain unresolved; this gate performs no runtime diagnosis.
5. Every `AssertUnwindSafe` needs owner-specific justification. Partial state
   must remain usable only by finalization; callbacks or mutex guards cannot be
   blindly asserted safe at uncontained transitions.
6. No abort, secondary panic during unwinding/destruction, kill, crash, power-loss,
   hostile-writer or durable-recovery guarantee is added. Global panic-hook
   output/stderr is outside transaction-controlled redaction. Blocking OS I/O
   can delay joined stop; multi-file disk atomicity and richer metadata remain
   excluded as before.
7. Remeasure final churn and every named oracle after implementation; estimates
   and existing oracle names do not qualify the new source. Run one stable
   F1-F12/G1-G6 remediation cycle, then separate independent/primary full gates.

No source/tests, staging, commit, ref or push changed in this gate; `.codex/`
was untouched. Only this review artifact is edited after reconciliation;
ignored primary validation artifacts are local evidence. All 11 Sprint 40,
four Sprint 40.1 and eight Sprint 41 prompts remain. Sprint 41 stays active,
v0.7 release-ineligible; no retirement, release, main merge, tag or Sprint 42.
Next action: following the same-reviewer consistency pass and final primary
documentation check, the dispatcher may create the separate design-pass commit and admit
the coherent three-file source remediation, followed by separate Task 6 evidence
and Task 7 reviews. All historical decisions below retain their original scope.

## Current producer-owned projection decision

Current decision: **pass** for the targeted design gate on the exact immutable
range `ceb3a91da70afde202cab84f7ea42846cdd734bd..cb1a25ea70e395dbfa87eb51923686b28d09d94e`.
Primary `/root/s41_task4` and independent reviewer `/root/s41_design_reviewer`
each observed initial/final review HEAD
`cb1a25ea70e395dbfa87eb51923686b28d09d94e`, clean branch
`codex/v0.7-sprint-41`. Both independently returned pass, with no blocking or
non-blocking findings and no missing design-owner evidence. The dispatcher
delivered the complete independent result before these documentation updates.
Neither review role mutated files, created logs, staged, committed, delegated,
inspected the preserved stash/floating implementation, or ran Cargo during review.

The unique prerequisite is `d849ebb50d83393b7aad6f34a1e6a43775a22af4`
(`Define Sprint 41 producer-owned semantic projection`). The correction used
no-ff merges `b2f1bdcfb9591ba24a1da9ef79c4a388083bac7f`,
`5b1fc2e301201cf48d63909d5ab9666df9a44ff7`, and the current reviewed endpoint.
The exact delta from `93661837df8d63bfed10c9b70d1986c4e0d12aa5` contains
nine documentation paths, 543 additions and 70 deletions. Production code,
manifests, dependencies and reusable framework are unchanged. The accepted
producer/identity correction is now part of ADR-0064; earlier passes below
remain historical and do not themselves approve this revised mechanism.

### Incomplete Task 5 attempt and current authorization

The prior complete-audit pass was committed as
`93661837df8d63bfed10c9b70d1986c4e0d12aa5`. Its Task 5 attempt remains
incomplete: nine task-owned paths and 1593 text additions/deletions, no binaries
and no implementation commit/push. Designer whole-module digest/declaration
provenance and EDT directly owned Query identities exposed the producer-boundary
blocker. The user explicitly approved the resulting producer-owned correction.
All earlier EOF, T03 and T17 findings and decisions are retained below.

Historical executed results come only from the committed Task 5 ledger:
Analysis package checking passed; Runtime all-targets checking passed before
later uncovered edits; the integration run exited 101 with one passed and one
failed test (`SemanticMismatch`, `Recovered`, zero retained files). Remaining
focused groups and the stable full gate were not run. Existing evidence paths
are `local-artifacts/codex-runs/sprint-41/task-5/analysis-check.log`,
`runtime-check.log`, and `F2-development.log` in that same directory. Neither
design-review role reexecuted those checks or inspected the preserved floating
implementation/stash. These historical results do not qualify the new design's
future implementation.

### Complete matrix, ownership and allocation assessment

Both roles evaluated the complete T01-T35 matrix, its 35 exactly-once R/L/C/T
classifications, and the [producer correction disposition](../architecture/safe-edit-transactions-invariants.md#producer-projection-correction-and-complete-audit-disposition).
Every original owner, ordering guard and reachable negative oracle remains
required. T03's owner-local/constructor split and T17's exact closed-type
evidence are preserved; neither constructor rejection nor an impossible state
is credited as a Runtime comparator test.

| Review domain | Independent and primary assessment |
|---|---|
| Typed expected evidence | pass: NodeIdentity, NodeFact, EdgeFact, RequestFact, DiagnosticFact and QueryFact use complete independently before-derived inventories, checked full publication/plan/document binding and exactly-once consumption. Diagnostic ordinals bind the exact full before record. Duplicate, missing, extra, conflicting, reused or unconsumed mappings reject. |
| Identity closure and complete semantics | pass: only the selected callable and its directly owned format-supported Query identities may change. Canonical BSL IDs, Contains ownership, exact Query binding/text and all unaffected facts remain mandatory. EDT includes malformed/unsupported Query; Designer emits no Query facts and gains no new Query semantics. |
| Canonical producer ownership | pass: DP owns whole-module SHA and all dependent declaration provenance; EP owns captured-byte analysis and nested Query/collection/resolver/request/diagnostic contexts; Q extracts the existing canonical Query-ID helper. Shared producer helpers preserve builder encodings without copying them into Analysis or Runtime. |
| Freeze and independent oracle | pass: before evidence is reproduced and checked, then expected projection freezes before staging and candidate build. Candidate evidence cannot fill missing keys or become its own expected value. Undo retains exact original evidence. All reachable candidate mutants traverse the real coordinator and comparator after production rebuild. |
| Allocation and lifetime | pass: borrowed counting neither allocates nor invokes another parser. Nested output/string/storage, key inventory, sorting, bitsets and simultaneous old/new growth reserve before emission/retention within the same 268435456-byte allowance. Existing scoped canonical parser internals remain excluded; no uncharged parser result escapes into attempt/undo. The exact 1MiB simple-rename positive remains required without a hidden smaller document cap. |
| Complete original protections | pass: full source freshness, policy binding, confinement, staging, recovery, quarantine, publication/cache serialization, cancellation, shutdown, redaction and old-snapshot compatibility remain intact across all 35 rows. |

Independent source inspection confirmed Designer digest/declaration production
in `adapters/designer-xml/src/semantic_graph.rs`, EDT Query/Contains emission
before query-language failure handling and shared nested contexts in
`adapters/edt/src/bsl_graph.rs`/`query_source_resolution.rs`, and canonical
`query_id` in `crates/bsl/src/queries.rs`. The captured-byte analyzer can be
separated from the existing disk fallback. Existing dependency directions and
nonallocating Graph iterators support the planned owners/counting boundary.
New producer test owners are exact DP/EP/Q, A and E locations in the matrix.

The allocation is exactly 25 unique paths totaling 8500 estimated text churn,
no binaries, 12 focused groups and one stable full gate. Only F10 broadens to
Analysis plus BSL all-targets and F11 to Designer plus EDT all-targets; F1's
library target and other groups remain intact. Stop before further work above
32 paths, 10000 text additions/deletions or any binary, overriding the revised
estimate's general 2x calculation. Resume retains implementation baseline
`93661837df8d63bfed10c9b70d1986c4e0d12aa5` and the preserved nine paths/1593
churn. Exclude separately committed prerequisite documentation/review deltas
by exact range only; do not reset the budget, subtract entire shared paths or
double-count overlapping snapshots.

### Independently executed producer-gate checks

Each role executed its own checks on the reviewed committed range.

| Check | Independent reviewer | Primary |
|---|---|---|
| Full-range and `93661837df8d63bfed10c9b70d1986c4e0d12aa5..cb1a25ea70e395dbfa87eb51923686b28d09d94e` correction `git diff --check` | exit 0 | exit 0 |
| Unchanged production/manifests/framework, prerequisite and final clean state | pass | pass |
| `bash -n scripts/validate-codex-prompts.sh` | exit 0 | exit 0 |
| `scripts/validate-codex-prompts.sh docs/codex/prompts/sprint-41-safe-edit-transactions/*.md` | exit 0; 8 files | exit 0; 8 files |
| `scripts/validate-codex-prompts.sh` | exit 0; 22 files | exit 0; 22 files |
| Markdown path/anchor checks | 15 files; 458 references; 0 errors | 15 files; 458 references; 0 errors |
| Matrix/classification/allocation/efficiency/accounting | pass: 35/35, 25 paths/8500, 12 groups and explicit caps | pass: 35/35, 25 paths/8500, 12 groups and four matching efficiency records |

Both initial enumeration probes included supplemental T rows and asserted;
corrected selectors limited to the primary matrix passed. This was a probe
error, not a documentation defect. The reviewer's new production-projector
symbol search returned zero matches (exit 1), reported separately as absence
evidence. No F1-F12 or full production gate ran in this design review.

No design-owner evidence is missing. Future allocation instrumentation,
exact/one-over limits and 1MiB positive, shared-helper/canonical producer
regressions, all candidate mutants, complete paired multi-file fixtures and
lifecycle/I/O evidence remain mandatory. The 8500 allocation is an estimate;
capacity and compatibility claims require those implementation tests. Parser
heap exclusion does not imply a whole-process memory bound. Original cooperative
ownership, hostile-writer, multi-file disk atomicity, crash/durable-undo, richer
metadata and stalled-I/O shutdown limitations below remain unchanged.

The same reviewer subsequently returned **artifact consistency: pass** on the
two-document draft with no required correction. It confirmed the exact range
and prerequisite, all 35 obligations, separate checks, zero-match/probe outcomes,
complete historical/incomplete-attempt evidence, pending production evidence
and risks without weakening. HEAD remained the reviewed endpoint, the index
was empty, and the draft diff and target Markdown anchor checks exited 0.

Effective context window and measured token telemetry are unavailable; bounded
selectors were used. No new logs were retained. Sprint 41 remains active and
push is deferred. Only the separately committed current pass with subject
`Approve Sprint 41 producer-owned semantic projection design` admits Task 5
resume; the dispatcher owns restoration of the preserved implementation.

## Complete-audit decision (historical)

Historical decision: **pass**, independently reconciled for the exact immutable
range `ceb3a91da70afde202cab84f7ea42846cdd734bd..ca385c5bcb0f5eb4ce4b26da32b41bfa54f0b28f`.
Primary `/root/s41_task4` and the same independent reviewer
`/root/s41_design_reviewer` both observed initial/final review HEAD
`ca385c5bcb0f5eb4ce4b26da32b41bfa54f0b28f`, branch
`codex/v0.7-sprint-41`, and a clean tree. Neither role mutated files, created
logs, staged, committed, delegated, or moved endpoints during this review.
The dispatcher supplied the complete independent result before these authorized
artifact/ledger updates. No remaining blocking or non-blocking finding and no
missing design-owner evidence remain on this range.

The new prerequisite is `3cdd67f7df47311f2b7a9936ab2afc5eb6515f06`
(`Clarify Sprint 41 complete invariant evidence placement`). The associated
no-ff merges are `22e38a3b65e800c866139a33f7515529c51e7119`,
`4ecf05972c13da237d8dffa8fc057fba33c75431`, and the current reviewed endpoint.
The delta from `d2dc6fdb772f04cb15cc38029b7494d262cae9af` is exactly five
documentation paths, 116 additions and 17 deletions. Production, ADR-0064,
architecture mechanism, dependencies and reusable framework remain unchanged.

### Preserved intervening blockers and renewed agreement

The first pass below was committed as
`b2f89c86012e71190afed077f42b5af82d552b42`. Task 5 admission at that same
HEAD then stopped with zero production edits or tests: original T03 required
Runtime and an integration test to forge Analysis-private fields and a
type-unrepresentable `RefactoringCompleteness` alternative. That was an
evidence-placement defect, not a demonstrated production vulnerability.

Correction `b918ed50ee556c6a235a36fb2268785941c3a801`
(`Refine Sprint 41 constructor-boundary test ownership`) used no-ff merges
`37a608fa351617bc1159b1347cac4d9cc30e0809`,
`34cc85682110b028670452c2cc37952e54417e8c`, and
`d2dc6fdb772f04cb15cc38029b7494d262cae9af`. The complete independent
T01-T35 audit of `ceb3a91da70afde202cab84f7ea42846cdd734bd..d2dc6fdb772f04cb15cc38029b7494d262cae9af`
accepted corrected T03 but returned **blocked** for one confirmed P2: matrix
line 66 still demanded a changed `SourceEvidenceCompleteness` marker and lines
133-138 propagated that impossible value through F3. The only variant at
`crates/analysis/src/refactoring.rs:652` is `BslCallableRenameV1`;
`SourceDocument`'s private field at line 667, constructor at line 681 and getter
at line 793 cannot provide an alternative in safe Rust. Primary reconciliation
accepted this finding; no pass artifact or commit was created for that range.
There were no other confirmed defects in that full audit.

After that complete audit, the user explicitly agreed to the consolidated
evidence-placement correction. The current review evaluates that correction;
neither the earlier pass nor agreement alone substitutes for this new gate.
All previous ranges, findings and decisions remain historical evidence.

### Current matrix coverage and reconciliation

Both roles independently evaluated all T01-T35 owner, guard, retention,
ordering and negative-oracle obligations and the complete R/L/C/T audit in the
[current matrix](../architecture/safe-edit-transactions-invariants.md#complete-matrix-representability-audit).
R denotes reachable production evidence, L owner-local private tests, C
constructor rejection and T unavailable safe-Rust states. Every row is classified
exactly once; C/T evidence never receives Runtime comparator-test credit.

| Rows | Current evidence placement and accepted result |
|---|---|
| T03 | pass: Runtime exercises all constructor-reachable same-ID structural differences, including duplicate summaries and LocalCall/QualifiedCall categories. AP owner-local unit tests call the real comparator for safely representable private mutants; closed constructors/types separately prove impossible states. No public forge API, unsafe value or weakened equality. |
| T17 | pass: the exact sole completeness variant is type evidence. Comparison of the actual field and every reachable missing document/occurrence, inventory, role/path/root/Configuration difference remains mandatory through the real production path. |
| T20-T21 | pass: source-constructor-valid semantic differences and canonical coordinate/provenance mappings reach the comparator. Invalid raw-byte/version/range combinations are constructor evidence only. |
| T22 | pass: canonical `reconstruct_terminal` whole-record substitutions cover structural/identity/provenance changes; no isolated private derived-ID corruption is required. |
| T23 | pass: canonical diagnostic/validation/rule producers supply actual records, fields, statuses, counts and complete collections; no absent incomplete marker or omission-counter field is invented. All real evidence losses remain negative production cases. |
| T01-T02, T04-T16, T18-T19, T24-T35 | pass: the remaining 29 rows retain their exact owners, ordered guards and real production/private-owner oracles. No accepted transaction invariant or executable negative case is weakened. |

The mapping retains 35 obligations, 12 focused command groups, F1's
`cargo test -p oneagent-analysis --lib --test safe_edit`, and the estimated
16-path/5000-text-churn/no-binary scope. For T16-T23 every reachable safely
constructed candidate still traverses the real coordinator after production
build and before the real comparator/commit. Source constructors cannot be
bypassed by a test-only validator or publication writer.

### Current independently executed documentation checks

Each role ran its own checks; all results below completed successfully.

| Check | Independent reviewer | Primary |
|---|---|---|
| `git diff --check ceb3a91da70afde202cab84f7ea42846cdd734bd..ca385c5bcb0f5eb4ce4b26da32b41bfa54f0b28f` | exit 0 | exit 0 |
| `git diff --check d2dc6fdb772f04cb15cc38029b7494d262cae9af..ca385c5bcb0f5eb4ce4b26da32b41bfa54f0b28f` | exit 0 | exit 0 |
| Unchanged production/ADR/framework and exact correction delta | pass | pass |
| `bash -n scripts/validate-codex-prompts.sh` | exit 0 | exit 0 |
| `scripts/validate-codex-prompts.sh docs/codex/prompts/sprint-41-safe-edit-transactions/*.md` | exit 0; 8 files | exit 0; 8 files |
| `scripts/validate-codex-prompts.sh` | exit 0; 22 files | exit 0; 22 files |
| Markdown paths/anchors | 15 files; 455 references; 0 errors | 15 files; 455 references; 0 errors |
| Matrix/classification/scope/efficiency/prerequisite/state | pass; 35 rows exactly once, 12 groups | pass; 35 rows exactly once, 12 groups, 4 matching efficiency records, 7 contiguous manifest tasks |
| Working diff and final review state | exit 0; unchanged clean HEAD | exit 0; unchanged clean HEAD |

The same reviewer subsequently returned **artifact consistency: pass** on the
two-document draft, with no required correction. It confirmed the exact new
range/prerequisite, all T01-T35 evidence placements, separate validation,
historical decisions and pending production evidence/risks. HEAD remained
`ca385c5bcb0f5eb4ce4b26da32b41bfa54f0b28f`; only the authorized primary-owned
artifact and Task 4 ledger changes were present, and draft diff-check exited 0.

Missing future production evidence remains separate from design evidence:
F1-F12 and full Rust validation were not run in this documentation gate. Complete
paired multi-file Runtime acceptance, fault ordinals, exact bounds and lifecycle
barriers remain mandatory implementation evidence. No zero-match search is
credited as a passing test. No production test or capability is implemented.

All residual risks below remain applicable: estimated scope, cooperative source
ownership without hostile-writer exclusion, non-atomic multi-file disk changes,
no crash recovery/durable undo, richer metadata deferred, and potentially
unbounded shutdown delay for stalled OS I/O. Effective context window and token
telemetry are unavailable; bounded selectors and the exact correction delta
were used. No logs were created. Sprint 41 remains active and push is deferred.

Recommended next action: commit this current pass and the Task 4 ledger with the
unique subject `Approve Sprint 41 complete invariant evidence design`; only that
committed prerequisite admits a fresh Task 5 attempt. The earlier zero-change
Task 5 blocker remains historical evidence.

## First pass decision and reviewed baseline (historical)

Historical decision: **pass** for the targeted pre-implementation design gate only.
Sprint 41 remains active. Production transaction conformance and integration
completion are not claimed; Task 5 requires this separately committed artifact.

- Repository branch: `codex/v0.7-sprint-41`.
- Exact replacement review range:
  `ceb3a91da70afde202cab84f7ea42846cdd734bd..f8b3009a5cab05c40158ee16d9d10b263def1dcd`.
- Primary and independent review initial/final observed HEAD:
  `f8b3009a5cab05c40158ee16d9d10b263def1dcd`; both observed a clean tree.
- Unchanged invariant prerequisite:
  `e54734c433ef449b8ca04eda7b7cc8eef8af2e35`
  (`Map Sprint 41 Safe Edit Transaction Invariants`).
- Accepted authorities: [ADR-0064](../adr/0064-safe-edit-transactions.md),
  the complete [T01-T35 matrix](../architecture/safe-edit-transactions-invariants.md#sprint-41-adr-invariant-matrix),
  [investigation](../architecture/safe-edit-transactions-investigation.md),
  and the Sprint 41 execution plan in [Roadmap](../Roadmap.md).

Independent reviewer: `/root/s41_design_reviewer`, dispatched by `/root` with
fresh context and read-only authority. Primary: `/root/s41_task4`. Neither
review role delegated, mutated files, created logs, staged, committed, or moved
the endpoints during either read-only review. The primary created this artifact
only after the dispatcher supplied the complete independent repeat result.

## Original blocked gate and corrected provenance

The original immutable range
`ceb3a91da70afde202cab84f7ea42846cdd734bd..e54734c433ef449b8ca04eda7b7cc8eef8af2e35`
remains a **blocked** historical result. Both roles found no architecture-level
blocker, but `git diff --check` against that range exited 2: `new blank line at
EOF` in the seven Sprint 41 child prompts, at their original lines
`01:116`, `02:94`, `03:94`, `04:126`, `05:140`, `06:94`, and `07:127`.
No pass artifact or Task 4 commit was created for that range.

After both reviews ended, the dispatcher separately corrected those seven EOF
blank lines in `8bbd5d43287b0cd968f16bf2aeaca042b9d879d4`
(`Fix Sprint 41 prompt trailing blank lines`). The version-derived remediation
workflow used no-ff merges `6c00f84d03e9796be956d040d2ba883bbaa68298`,
`68fee5469158b2dc5956bb39aeca6ac6d8377068`, and
`f8b3009a5cab05c40158ee16d9d10b263def1dcd`.
Both roles independently verified the replacement delta: exactly seven deleted
EOF blank lines in seven prompts, with no ADR, matrix, Roadmap, architecture,
or production change. The replacement endpoint passed the repeated design and
documentation checks; it does not rewrite the original blocked result.

## Acceptance evidence matrix

Every production obligation T01-T35 was evaluated independently against its
accepted owner, ordered guard/retention point, negative production oracle and
focused command. All 35 sequential rows and all 12 command mappings are present.
The grouped record below preserves complete row coverage; planned symbols and
tests remain planned, not executed evidence.

| Matrix rows | Evaluated design obligations | Independent and primary result |
|---|---|---|
| T01-T05 | Opt-in API, service identity, one outstanding attempt, exact structured plan/capability binding, one-use confirmation and side-effect-free policy gate | pass |
| T06-T10 | Publication-bound complete source baseline, cache/scan exclusions, bounded enumeration/read/allocation and reversal retention before retaining payload | pass |
| T11-T15 | Confinement and alias rechecks, exact replacements, preallocation/disk admission, verified staging/backups, ordered per-file replacement and ambiguous failures | pass |
| T16-T17 | Complete before/after production rebuild and tree checks; exhaustive Configuration/document inventory and untouched evidence | pass |
| T18-T19 | Canonical renamed target, every node/edge/payload and explicit provenance comparison; Graph edge equality alone is insufficient | pass |
| T20-T22 | Complete occurrence projection, Unicode/BOM/line-ending coordinates, canonical anchors and every reference-ledger disposition | pass |
| T23-T24 | Complete diagnostic/rule equivalence through canonical producers and actual adjacent publication/impact identity | pass |
| T25-T27 | Final cleanup/scan/cancellation before sole commit, serialized postcommit cache and all publication writers, watcher/explicit-input ordering | pass |
| T28-T30 | Checked reverse-order recovery, exact full restoration/cleanup, quarantine and recovery-failure precedence | pass |
| T31-T33 | Separately confirmed reversal, cancellation/drop/shutdown joining, deterministic closed failures and redaction | pass |
| T34-T35 | Additive read-only compatibility, unchanged consumers, stable publication eligibility and successor invalidation | pass |

The primary also located the three existing snapshot writers at
`apps/runtime/src/workspace/mod.rs:767`, `:1036`, and `:1187`, checked the
canonical public diagnostic/reference constructors, and confirmed the matrix's
planned crate-visibility change for the existing coordinate helpers. The planned
API provides a usable bound authorization path; no intermediate candidate is
exposed as a current publication. No owner, guard order, negative oracle or
accepted boundary contradiction remains unresolved at design level.

## Independent result and primary reconciliation

The same reviewer returned **pass** for the exact replacement range. There are
no remaining blocking or non-blocking findings and no missing design-owner
evidence. The original positive T01-T35 assessments remain valid because the
correction did not change those authorities. The primary independently repeated
the replacement-delta and documentation checks and agrees without weakening
the reviewer decision, findings, deferred evidence or residual risks.

The same reviewer subsequently checked this draft and the sole Task 4 ledger
row read-only and returned **artifact consistency: pass**. It confirmed the
original blocked result, all seven EOF locations, correction commit/three
merges, exact replacement pass range, matrix coverage, separate validation,
future production evidence and all material risks without required correction
or capability overstatement. HEAD remained the reviewed endpoint; only the two
authorized documentation paths were present as primary-owned changes.

## Executed validation

Commands ran from the repository root. Independent and primary evidence are
separate runs; neither role substitutes the other's successful command result.

| Check | Independent reviewer result | Primary result |
|---|---|---|
| `git diff --check ceb3a91da70afde202cab84f7ea42846cdd734bd..f8b3009a5cab05c40158ee16d9d10b263def1dcd` | exit 0 | exit 0 |
| Correction delta from `e54734c433ef449b8ca04eda7b7cc8eef8af2e35` to `f8b3009a5cab05c40158ee16d9d10b263def1dcd` | exact seven EOF deletions; diff-check exit 0 | exact seven EOF deletions; no authority/production changes |
| `bash -n scripts/validate-codex-prompts.sh` | exit 0 | exit 0 |
| `scripts/validate-codex-prompts.sh docs/codex/prompts/sprint-41-safe-edit-transactions/*.md` | exit 0; 8 files | exit 0; 8 files |
| `scripts/validate-codex-prompts.sh` | exit 0; 22 files | exit 0; 22 files |
| Markdown path/anchor inspection of the 14 reviewed files | 451 references; 0 problems | 451 references; 0 problems |
| Matrix enumeration | 35 sequential T rows; 12 F commands | 35 sequential T rows; 12 F commands |
| `git diff --check` and read-only final Git state | exit 0; clean tree and unchanged HEAD | exit 0; clean tree and unchanged HEAD |

The primary's initial incorrect `docs/codex/templates/task.md` lookup failed and
was corrected to the declared `task-prompt.md`; it is not validation evidence.
The bounded Markdown-linter/link-checker discovery in `scripts` and `.github`
returned zero matches (exit 1), so manual path/anchor checks and the existing
prompt validator provide the documentation gate. The original expected
zero-match production-symbol search is absence evidence, not passing tests.

After drafting, the primary verified all 11 Markdown references in the two
authorized files, six required review sections, exact EOF termination, four
matching Sprint 41 efficiency records in Roadmap/master, and seven contiguous
manifest tasks. An initial custom efficiency probe incorrectly expected records
in child prompts and failed; the corrected probe checked their actual owners
and passed. No repository defect or production-test claim follows from that
probe failure. Prompt syntax, explicit-suite/repository validation and both the
working and immutable-range diff checks were independently repeated successfully.

## Missing production evidence, scope and residual risks

No design-owner evidence is missing. F1-F12 and the full Rust production gate
were deliberately not run: this is a documentation-only design gate, the new
transaction modules/tests are not implemented, and zero matched tests would not
prove conformance. Complete multi-file paired Runtime fixture acceptance, every
semantic mutant and I/O fault ordinal, exact/one-over retention limits and
lifecycle barriers remain mandatory implementation evidence.

The 16-path/5000-text-churn allocation is an estimate, not measured implementation
scope or permission to narrow the oracle. Cooperative exclusive source ownership
does not exclude hostile path-swap races. Semantic publication can be atomic
while multi-file disk changes are not. No crash recovery, durable/cross-process
undo, richer metadata preservation or bounded shutdown during stalled OS I/O is
claimed. These accepted limitations remain unchanged by the pass.

The review adds no dependencies, production behavior, Supported/Coverage claims,
protocol or UI edit surface, new refactoring family, metadata/path rename,
cross-Configuration/Workspace mutation, remote/Git edit operation, Sprint 42 or
release execution. Old snapshots and existing read-only consumers retain their
contracts. No production tests were added or changed.

Effective context window and measured token telemetry are unavailable. Both
roles used the bounded manifest and exact correction delta; primary admission
was warning with narrowed selectors. No large logs were retained for this gate.

Recommended next action: commit this pass record and the Task 4 ledger together,
then dispatch Task 5 in fresh context against that exact committed prerequisite.
Push remains deferred to sprint end. Sprint completion still requires the later
implementation evidence and independent integration review.
