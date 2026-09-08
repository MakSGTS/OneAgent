# Sprint 41 Safe Edit Transactions Execution Loop

## Reporting language

Russian user-facing reports; English repository artifacts and commit messages.
This file is only a dispatcher and durable ledger.

## Canonical authorities

Use `docs/codex/workflows/sequential-sprint-execution.md`, applicable AGENTS.md,
and each child's selected Profile/base/specialized Template. Read only each
child's admitted Context Manifest. Do not accumulate implementation transcripts.

## Sprint objective and current state

Sprint 41 is the unique next target at
`ceb3a91da70afde202cab84f7ea42846cdd734bd` on `codex/v0.7`.
Sprint 40.1 has a committed pass review. Add checked, reversible local edit
transactions for the existing complete BSL callable-rename plans, with production
semantic rebuild and explicit authorization. The bounded first surface is a
runtime Rust API; protocol and IDE edit surfaces are deferred. Exact transaction
mechanisms belong to Task 2, not this planning record.

## Starting-state requirements

Start on `codex/v0.7-sprint-41`, created from the version head above.
Require the committed `Plan Sprint 41 Safe Edit Transactions` baseline and
a clean task-owned tree. Resolve every future prerequisite subject to one exact
commit in this sprint ancestry and pass the full ID; do not accept an ambiguous
historical subject. The initial modified/staged/untracked inventory is empty.

## Ordered task manifest

| Order | Prompt | Prerequisite commit subject | Outcome | Validation additions | Commit message |
|---:|---|---|---|---|---|
| 1 | [01-investigate-safe-edit-transactions.md](01-investigate-safe-edit-transactions.md) | Plan Sprint 41 Safe Edit Transactions | Repository-backed transaction readiness and boundary investigation. | Non-mutating planner, paired-source, lifecycle, and policy evidence; documentation validation. | `Investigate Sprint 41 Safe Edit Transactions` |
| 2 | [02-define-safe-edit-transactions.md](02-define-safe-edit-transactions.md) | Investigate Sprint 41 Safe Edit Transactions | Accepted bounded transaction ADR and compatibility contract. | ADR/authority consistency, affected consumer inventory, Markdown links. | `Define Sprint 41 Safe Edit Transactions` |
| 3 | [03-map-safe-edit-transaction-invariants.md](03-map-safe-edit-transaction-invariants.md) | Define Sprint 41 Safe Edit Transactions | Complete accepted-ADR production invariant matrix. | Every applicable ADR invariant mapped to location, ordering, negative oracle, and focused command. | `Map Sprint 41 Safe Edit Transaction Invariants` |
| 4 | [04-review-safe-edit-transaction-design.md](04-review-safe-edit-transaction-design.md) | Define Sprint 41 producer-owned semantic projection | Independent targeted design gate, recorded only after pass. | Fresh read-only design review and primary documentation checks; no full production gate. | `Approve Sprint 41 producer-owned semantic projection design` |
| 5 | [05-implement-safe-edit-transactions.md](05-implement-safe-edit-transactions.md) | Approve Sprint 41 producer-owned semantic projection design | Checked apply/reversal with confined writes, recovery, authorization, and atomic semantic publication. | 12 focused checks defined by the corrected matrix; one stable canonical full workspace gate. | `Implement Sprint 41 Safe Edit Transactions` |
| 6 | [06-complete-safe-edit-transaction-evidence.md](06-complete-safe-edit-transaction-evidence.md) | Implement Sprint 41 Safe Edit Transactions | Exact final implementation evidence, consumer audit, and immutable review handoff. | Stable-head test/log reconciliation, API/dependency/Coverage audit, documentation checks. | `Document Sprint 41 Safe Edit Transaction Evidence` |
| 7 | [07-sprint-41-integration-review.md](07-sprint-41-integration-review.md) | Document Sprint 41 Safe Edit Transaction Evidence | Independent integration decision, Sprint 41 completion, and v0.7 release-review handoff. | Independent reviewer and primary focused/full gates, artifact consistency, exact retirement audit. | `Complete Sprint 41 Safe Edit Transactions Review` |

Every child uses Prompt Contract v2 and fresh_context: required. Planning must
validate the master and all seven children before accepting this baseline.

## Commit authorization mode

Resolve authorization from the current user instruction. This launch explicitly
requests one commit after each completed task and push at sprint end. Its push
timing overrides the repository default of immediate push. Record intermediate
commits as local/push deferred; do not silently push early. No empty commits.

Keep the required no-ff branch sequence. After Tasks 1-6 and validation, merge
implementation into `codex/v0.7`, then create
`codex/v0.7-sprint-41-review` for Task 7. Merge a successful review back into
`codex/v0.7`, then push only that current branch to origin. That push publishes
all reachable sprint and review commits. Do not push other branches, main, or
tags; do not execute the release review. Stop on a push failure.

## Fresh-context child-runner authorization and unavailable-runtime fallback

Each child requires a fresh runner without inherited turns and without further
delegation. Resolve runner authorization against the current user instruction
and higher-priority runtime rules before dispatch. A stored prompt cannot
override a runtime prohibition. If authorization or guaranteed fresh context is
unavailable, stop at the exact child and report its prompt and committed
prerequisite instead of executing it in this accumulated dispatcher context.

## Constructor-boundary corrective prerequisite

Task 5 admission at `b2f89c86012e71190afed077f42b5af82d552b42` stopped
with zero source changes: the original T03 required Runtime and an integration
test to forge Analysis-private fields and a type-unrepresentable completeness
value. The corrective commit `Refine Sprint 41 constructor-boundary test ownership`
changes only evidence placement. Runtime tests all reachable same-ID structural
differences; Analysis owner-local unit tests call the real comparator for
representable private mutants; closed-type/constructor evidence covers
unrepresentable states. No public forge API, unsafe code or transaction change.
F1 gains the Analysis library target; 16 paths/5000 churn/12 groups remain.

The original Task 4 pass and ledger row remain historical evidence for their
reviewed range, but do not admit the corrected matrix. Require the new gate and
unique commit subject below before resuming Task 5. Preserve both decisions in
the same design artifact when recording the new pass. Resolve the resumed Task 5
start to that new committed pass; its initial blocked attempt had zero churn.

## Complete audit and renewed agreement

The independent review of the constructor-boundary correction at
`d2dc6fdb772f04cb15cc38029b7494d262cae9af` accepted T03 but blocked
T17's impossible changed `SourceEvidenceCompleteness` marker. A complete
T01-T35 representability/constructor/privacy audit then distinguished reachable
Runtime cases, owner-local mutants, constructor rejection and closed-type
evidence. No other confirmed defect remained. The user explicitly agreed to
the consolidated revised plan before this further remediation.

The new prerequisite `Clarify Sprint 41 complete invariant evidence placement`
records that full audit, replaces only the impossible T17 marker with exact
type evidence, preserves every reachable negative case, and clarifies valid
producer-based T20-T23 substitutions. The transaction mechanism, ADR and
16-path/5000-churn/12-group budget remain unchanged. The new gate below must
pass and be committed before Task 5 resumes; prior passes remain historical.

## Producer projection correction and current blocked attempt

The user explicitly approved the producer-owned projection revision after the
Task 5 attempt starting at `93661837df8d63bfed10c9b70d1986c4e0d12aa5`.
Earlier T03/T17 agreements and design passes remain historical and do not approve
this new identity/provenance mechanism. Task 5 is incomplete: 9 task-owned paths,
1593 text additions/deletions, no binary or implementation commit/push. The
original zero-change T03 blocker remains recorded above.

Analysis package checking passed; Runtime all-targets checking passed before
subsequent uncovered edits. The integration run exited 101 with 1 passed and
1 failed test; apply returned SemanticMismatch/Recovered and zero retained
files. Remaining focused groups and the stable full gate have not run. Evidence
is retained in `local-artifacts/codex-runs/sprint-41/task-5/`:
`analysis-check.log`, `runtime-check.log`, `F2-development.log`.

The dispatcher preserved the incomplete implementation in stash
`ff2a1d29683197438c304496804ff430bec7c682` and prepared clean documentation
remediation at `b2f1bdcfb9591ba24a1da9ef79c4a388083bac7f` on
`codex/v0.7-sprint-41-remediation`. This documentation task must not inspect,
restore or change that stash. Its unique correction subject is
`Define Sprint 41 producer-owned semantic projection`; its review is pending.
Only a separately committed `Approve Sprint 41 producer-owned semantic projection design`
pass for the corrected immutable range permits Task 5 resume. The design artifact
and historical ledger pass are not edited by the correction task.

The ADR fixes typed before-fact keys, complete exactly-once consumption, canonical
adapter APIs and checked preallocation before producer scratch/retention. Expected
projection freezes before staging/candidate; the only identity closure adds
format-supported directly owned Query. The full T01-T35 audit and eligibility
remain; write/recovery/undo/coordinator, dependencies and wire/UI stay unchanged.
At that producer correction the matrix allocated 25 paths/8500 estimated churn,
expanded only F10/F11,
and retains 12 focused groups/one stable full gate.

Resume must retain original implementation accounting baseline
`93661837df8d63bfed10c9b70d1986c4e0d12aa5`; the new design-pass commit is
an execution prerequisite, never a budget reset. Restore and verify the original
9-file inventory only under the dispatcher's authorized resume operation; count
its 1593 churn as task-owned, not unrelated pre-existing work. Reconcile the
cumulative implementation diff, including all task-owned untracked text,
formatting and Task 5 ledger, against that baseline. Exclude separately committed
prerequisite documentation corrections/review deltas by exact commit/range only;
never subtract entire shared paths or double-count overlapping snapshots.
The producer correction originally capped 32 paths/10000 additions plus
deletions/no binaries. The explicit user override below supersedes only its
text estimate/cap; the cumulative accounting rule remains unchanged.

## User-authorized completion budget increase

On 2026-09-08 the user explicitly authorized raising the completion hard cap to
20000 text additions plus deletions and continuing. The complete audited
remaining-work estimate sets the final estimate at 12000. The user chose 20000,
not the proposed 14000; the previous 10000 cap and historical 25/8500 estimate
are superseded. Keep exactly 25 planned paths, a hard path cap of 32, no binary
paths, all T01-T35 requirements, 12 focused groups and one stable full gate.
This is a numeric execution-budget override, not architecture/remediation scope.
The existing targeted design pass f25388cd8073bcd228c8eaa951ef1c0178907431
remains the mechanism gate; no new gate or pass claim is required.

The pause inventory is 24 implementation paths/9369 churn/no binaries, including
all carried work, with no Task 5 completion or stable full-gate claim. Evidence:
`local-artifacts/codex-runs/sprint-41/task-5/producer-resume/completion-budget-estimate.md`,
`budget-pause-summary.md` and `budget-pause-state.json` in that same directory.
Keep original cumulative baseline 93661837df8d63bfed10c9b70d1986c4e0d12aa5;
f25388cd8073bcd228c8eaa951ef1c0178907431 is a resume/mechanism checkpoint,
not a reset. Exclude only exact separately committed prerequisite documentation
ranges, including this four-document budget commit; count all implementation
work and the eventual Task 5 ledger delta, even in the shared master path.
Historical Task 4 passes and incomplete Task 5 ledger state are preserved.
Stop before further work above 32 paths, 20000 churn or any binary; do not
substitute the general 2x baseline rule. Commit completed tasks and push only
at sprint end under the current user instruction.

## Initial audit additions

Validate this master plus all children with
`scripts/validate-codex-prompts.sh`. Check unique eligibility, clean tree,
prerequisite and manifest ordering, exact messages, matrix selector, matching
Roadmap efficiency records, preserved previous suite, and Context preflight.
Effective context window/telemetry may be unknown; estimates are admission-only.

## Sprint efficiency contract

```text
sprint_efficiency_contract: v1
adr_invariant_matrix: docs/architecture/safe-edit-transactions-invariants.md::Sprint 41 ADR invariant matrix
design_review_gate: docs/codex/prompts/sprint-41-safe-edit-transactions/04-review-safe-edit-transaction-design.md|Define Sprint 41 producer-owned semantic projection|docs/reviews/sprint-41-safe-edit-transactions-design.md|Approve Sprint 41 producer-owned semantic projection design
implementation_baseline: docs/codex/prompts/sprint-41-safe-edit-transactions/05-implement-safe-edit-transactions.md|25|12000|none|12|1
```

The matrix is provisional during planning. Task 3 maps all applicable accepted
production invariants after ADR-0064 is committed. Task 4 is a pending design
gate, not a claimed pass. Before Task 5, supply the exact matrix commit, design
pass artifact/commit, task-start commit, and initial untracked inventory.
The revised numerical baseline admits 25 paths and 12000 text additions/deletions, no
binary paths, 12 focused checks and one stable full implementation gate.
The focused commands are fixed by the accepted matrix before design review.
Independent reviewer and primary completion gates are separate evidence.
Use the cumulative implementation accounting below. Explicitly stop at more than
32 paths, more than 20000 text churn or any binary path; these tighter caps
override twice the new 25/12000 baseline. The canonical full gate remains one.

## Task-loop additions

Verify each child result independently, including committed paths, validation,
clean state and deferred push, before recording one durable ledger row. Record
a completed row in the child's own commit when possible using its known start
ID and unique commit subject as the self-reference; resolve its end ID live.
Do not create unrelated ledger-only commits. Count the master ledger path in
each child's scope and any implementation churn calculation.

## Automatic mandatory reviewer authorization

Tasks 4 and 7 require separate fresh read-only reviewers as defined by the
Review workflow. The dispatcher coordinates their handoff; child runners never
delegate. Task 7 requires the same reviewer for final artifact consistency.
Resolve runtime authorization before dispatch; never substitute self-review.
Keep useful primary verification independent and avoid concurrent Cargo use.

## Already-complete policy additions

Only current committed evidence and successful required checks can establish
already_complete. Never skip an unmet prerequisite or create an empty commit.

## Failure and integration-review gates

Stop at the first blocking failure with the exact command, result, state, diff,
and recovery action. A failed design review creates no pass artifact or commit.
Do not implement review fixes inside a review task. Follow repository remediation
branches when separately authorized continuation resolves a review blocker.
The second architecture-level blocked review triggers the sequential workflow
stop-loss and renewed plan agreement.

Only a non-blocking Task 7 result with full independent/primary validation and
same-reviewer artifact consistency completes Sprint 41. The next eligible gate
is the v0.7 release integration review; Sprint 42 stays planned.

## Immediately preceding suite inventory

Conditional retirement targets exactly these four tracked files:

- `docs/codex/prompts/sprint-40-1-refactoring-planner-remediation/00-sprint-40-1-execution-loop.md`
- `docs/codex/prompts/sprint-40-1-refactoring-planner-remediation/01-review-refactoring-planner-remediation-design.md`
- `docs/codex/prompts/sprint-40-1-refactoring-planner-remediation/02-remediate-refactoring-planner-contract.md`
- `docs/codex/prompts/sprint-40-1-refactoring-planner-remediation/03-sprint-40-1-integration-review.md`

Re-enumerate immediately before retirement; refuse extra/untracked targets.
Preserve every Sprint 40 file and this entire Sprint 41 suite. Use explicit
file edits only, and include retirement in the single Task 7 review commit.

## Blocked integration and remediation evidence follow-on

Original Task 5 `f2813d2eff5fa78efe3f0d4a705e3bc51de13979` and Task 6
`e99a6ac14494f9b00fc2f144b01b84e402c9f7d4` remain unique committed
prerequisites. Task 7 at `33922bea4cbd1e9b84e67fd01a8ebc7e6c81f5a0` was
blocked. Primary reported R3-R5 (P2), with documentation checks but no Cargo
gate. Independent reported R1/R2 (P1), R3 (P2), M1-M4; only its F1-F10
completed on the old tree. No old implementation count supplies missing review
validation. Original findings and final runner-stop evidence are retained in
`local-artifacts/codex-runs/sprint-41/remediation/review-handoff.md`.

Isolated `fa031100ac19a98b17e676687a498bcce4e7280e` has the reviewed tree.
Remediation `aaeacbfa675bd1a321f5e5c160950c6c661052d0` implements closure
claims for R1-R5/M1-M4 within the accepted owners, ADR and R/L/C/T split.
All 18 stable commands exited 0, F1-F12 counts
82/6/28/7/16/9/11/10/33/222/401/37, canonical test 1447/83 nonzero targets
plus four separate empty harnesses. All 243 input hashes and 50 retained log
hashes are reconciled; old 1435, attempt-1 F8 9/1 and development failures remain
historical. These are implementation results, not an independent review pass.

Original implementation remains 25 paths/11259 churn; remediation delta is
8/+2257/-393 = 2650. Net cumulative implementation from
`93661837df8d63bfed10c9b70d1986c4e0d12aa5` is 25/+12973/-234 = 13207,
no binaries: above estimate 12000, below hard caps 32/20000. Exact prerequisite
documentation through `f3c1f8c087378b78c50f7fd97499b2c2d7e5f510` is 10 paths/
823 churn; original Task 6 is separately 5/438. Count the original Task 5 master
delta, never blanket-exclude it. Full unpartitioned 35/14446 at `aaeacbfa` is
reported separately in `remediation/scope.json` and `audit-scope.py`.

This fresh Task 6 follow-on starts at clean `aaeacbfa` and uses the distinct
commit subject `Update Sprint 41 remediation evidence`. It reconciles all 35
requirements/52 named oracles, consumer/API/dependency compatibility, exact
retention/oracle limits and Markdown/prompt checks; no Cargo rerun. Its artifacts
are `local-artifacts/codex-runs/sprint-41/remediation-evidence/`; measured context
telemetry is unavailable. The [current evidence](../../../architecture/safe-edit-transactions-evidence.md)
distinguishes 15 request + 12 diagnostic owner allocation boundaries, 15 EDT
quota attempts, shared lease transfer, and the Unicode capacity probe; none
proves total-process heap or every internal canonical-helper reallocation overlap.

After this documentation commit, the dispatcher integrates the remediation and
launches a fresh independent/primary Task 7 gate on an immutable endpoint.
Keep Sprint 41 active, all previous/current prompts, push deferred to sprint end,
and release review ineligible until that gate passes. No merge/push/tag/main or
completion operation is part of this documentation follow-on.

## Ledger

| Order | Prompt | Status | Start HEAD | End HEAD | Validation | Commit/push | Tokens | Logs |
|---:|---|---|---|---|---|---|---|---|
| 1 | 01-investigate-safe-edit-transactions.md | completed | 078b258e150842da0a79da96ea09395887080cfc | commit subject: Investigate Sprint 41 Safe Edit Transactions (resolve in sprint ancestry) | Analysis plan 17; paired conformance 4; Runtime public 9; Runtime Workspace unit 75 (49 filtered); Tool Policy 26+7; 0 policy doc-tests separately; prompt syntax pass, explicit suite 8 and repository 22 pass; Markdown/selector/efficiency checks and diff-check pass | Investigate Sprint 41 Safe Edit Transactions; local, push deferred to sprint end | unavailable; preflight warning | local-artifacts/codex-runs/sprint-41/task-1/ |
| 2 | 02-define-safe-edit-transactions.md | completed | 9471c34e2b151ca4fd52609eb73c24b036930230 | commit subject: Define Sprint 41 Safe Edit Transactions (resolve in sprint ancestry) | ADR/authority/consumer/oracle review pass; 5 changed Markdown links and 11 ADR sections pass; 4 efficiency records equal; 7 manifest tasks contiguous; prompt syntax pass, explicit suite 8 and repository 22 pass; diff-check pass; Rust gates not applicable; zero matched test filters not applicable | Define Sprint 41 Safe Edit Transactions; local, push deferred to sprint end | unavailable; effective window unknown; estimated admission pass | local-artifacts/codex-runs/sprint-41/task-2/ |
| 3 | 03-map-safe-edit-transaction-invariants.md | completed | 7de36516d283a810ec5ec01b09980b07b6e634dc | commit subject: Map Sprint 41 Safe Edit Transaction Invariants (resolve in sprint ancestry) | 35 production obligations, 12 focused commands, 16 paths/5000 estimated churn/no binaries mapped; 38 paths, 36 owner selectors, 20 Markdown links/selectors pass; 4 efficiency records equal and 7 manifest tasks contiguous; prompt syntax, explicit suite 8 and repository 22 pass; diff-check pass; Rust gates and zero matched test filters not applicable | Map Sprint 41 Safe Edit Transaction Invariants; local, push deferred to sprint end | unavailable; effective window unknown; preflight warning with narrowed selectors | local-artifacts/codex-runs/sprint-41/task-3/ |
| 4 | 04-review-safe-edit-transaction-design.md | completed | cb1a25ea70e395dbfa87eb51923686b28d09d94e | commit subject: Approve Sprint 41 producer-owned semantic projection design (resolve in sprint ancestry) | Independent and primary producer-gate pass; full35/classification35, typed complete frozen projection and allocation admission, exact25/8500 and caps32/10000, F12; 458 baseline Markdown references, syntax, explicit suite8/repository22 and full/correction/working diff-check pass; all historical gates and incomplete Task5 evidence preserved; production checks pending | Approve Sprint 41 producer-owned semantic projection design; prior passes b2f89c86012e71190afed077f42b5af82d552b42 and 93661837df8d63bfed10c9b70d1986c4e0d12aa5 remain historical; local, push deferred to sprint end | unavailable; effective window unknown; preflight warning with bounded selectors | none |
| 5 | 05-implement-safe-edit-transactions.md | completed | 93661837df8d63bfed10c9b70d1986c4e0d12aa5 | commit subject: Implement Sprint 41 Safe Edit Transactions (resolve in sprint ancestry) | F1 74+7; F2 6; F3 20 (130 filtered); F4 6 (144 filtered); F5 16; F6 9; F7 11; F8 10; F9 26+7; F10 221/12 targets; F11 399/23 targets; F12 37/4 targets; all command exits 0, 40 named oracles/full35 mapping; canonical fmt/check/test/clippy/doc/diff pass, 1435 tests across 83 nonzero targets and 4 separate zero-test harnesses; initial G3 repo-local TMPDIR/Git-discovery failure and 8/8 focused environment retry retained, canonical repeat passed with GIT_CEILING_DIRECTORIES=$TMPDIR on unchanged source manifest | Implement Sprint 41 Safe Edit Transactions; all carried work included, cumulative 25 paths/11259 churn/no binaries; local, push deferred to sprint end | unavailable; effective window unknown | local-artifacts/codex-runs/sprint-41/task-5/producer-resume/ |
| 6 | 06-complete-safe-edit-transaction-evidence.md | completed | f2813d2eff5fa78efe3f0d4a705e3bc51de13979 | commit subject: Document Sprint 41 Safe Edit Transaction Evidence (resolve in sprint ancestry) | Exact 25-path/11259-churn range and prerequisite exclusion, 24 committed source hashes, 35 invariant rows/40 named oracles and 18 retained command logs reconciled; G3 1435 tests/83 nonzero targets/4 separate zero harnesses, initial environment failure and 8/8 retry preserved; API/dependency/Graph/Coverage/cache/catalog/redaction audit; changed/new Markdown links, four efficiency records/seven tasks, prompt syntax/explicit suite8/repository22 and diff-check pass; documentation-only, no new Rust gate | Document Sprint 41 Safe Edit Transaction Evidence; local, push deferred to sprint end; Task 7 pending, Sprint 41 active | unavailable; effective window unknown; preflight warning with bounded selectors | local-artifacts/codex-runs/sprint-41/task-6/; Task 5 producer-resume logs retained |
| 7 | 07-sprint-41-integration-review.md | blocked historical attempt; fresh gate pending | 33922bea4cbd1e9b84e67fd01a8ebc7e6c81f5a0 | no completion commit | Primary R3-R5; independent R1-R3/M1-M4; primary documentation checks only, independent F1-F10 only; no successful review or inherited full gate | no review commit/push/retirement; restart after remediation evidence integration | unavailable | local-artifacts/codex-runs/sprint-41/remediation/review-handoff.md |

The original Task 5/6 ledger rows above are historical completed boundaries.
The remediation-evidence follow-on starts at
`aaeacbfa675bd1a321f5e5c160950c6c661052d0`; its end is the unique subsequent
`Update Sprint 41 remediation evidence` commit. Its required documentation
checks and exact resulting commit are retained in the local follow-on summary;
the seven-task manifest and unique original task subjects are preserved.
Follow-on documentation validation passed 275 references/five anchors,
35 rows/123 production references/52 named oracles, four consistent efficiency
records, prompt syntax, explicit suite 8, repository 22 and whitespace checks.
The local checker syntax-assumption failure is retained before its corrected
pass. No production check was rerun and no review/completion decision was made.

## Final report additions

Report starting/ending state, exact HEAD and branch, verified task outcomes and
commits, end-of-sprint push, review identities/reconciliation/consistency,
retirement, measured telemetry or unavailable, logs, remaining changes, and
the exact next action. Never describe planning as implemented transactions.
