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
The matrix allocates exactly 25 paths/8500 estimated churn, expands only F10/F11,
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
Stop before further work above 32 unique paths, 10000 additions/deletions or any
binary path, overriding the general 2x calculation for the new baseline.

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
implementation_baseline: docs/codex/prompts/sprint-41-safe-edit-transactions/05-implement-safe-edit-transactions.md|25|8500|none|12|1
```

The matrix is provisional during planning. Task 3 maps all applicable accepted
production invariants after ADR-0064 is committed. Task 4 is a pending design
gate, not a claimed pass. Before Task 5, supply the exact matrix commit, design
pass artifact/commit, task-start commit, and initial untracked inventory.
The revised numerical baseline admits 25 paths and 8500 text additions/deletions, no
binary paths, 12 focused checks and one stable full implementation gate.
The focused commands are fixed by the accepted matrix before design review.
Independent reviewer and primary completion gates are separate evidence.
Use the cumulative implementation accounting below. Explicitly stop at more than
32 paths, more than 10000 text churn or any binary path; these tighter caps
override twice the new 25/8500 baseline. The canonical full gate remains one.

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

## Ledger

| Order | Prompt | Status | Start HEAD | End HEAD | Validation | Commit/push | Tokens | Logs |
|---:|---|---|---|---|---|---|---|---|
| 1 | 01-investigate-safe-edit-transactions.md | completed | 078b258e150842da0a79da96ea09395887080cfc | commit subject: Investigate Sprint 41 Safe Edit Transactions (resolve in sprint ancestry) | Analysis plan 17; paired conformance 4; Runtime public 9; Runtime Workspace unit 75 (49 filtered); Tool Policy 26+7; 0 policy doc-tests separately; prompt syntax pass, explicit suite 8 and repository 22 pass; Markdown/selector/efficiency checks and diff-check pass | Investigate Sprint 41 Safe Edit Transactions; local, push deferred to sprint end | unavailable; preflight warning | local-artifacts/codex-runs/sprint-41/task-1/ |
| 2 | 02-define-safe-edit-transactions.md | completed | 9471c34e2b151ca4fd52609eb73c24b036930230 | commit subject: Define Sprint 41 Safe Edit Transactions (resolve in sprint ancestry) | ADR/authority/consumer/oracle review pass; 5 changed Markdown links and 11 ADR sections pass; 4 efficiency records equal; 7 manifest tasks contiguous; prompt syntax pass, explicit suite 8 and repository 22 pass; diff-check pass; Rust gates not applicable; zero matched test filters not applicable | Define Sprint 41 Safe Edit Transactions; local, push deferred to sprint end | unavailable; effective window unknown; estimated admission pass | local-artifacts/codex-runs/sprint-41/task-2/ |
| 3 | 03-map-safe-edit-transaction-invariants.md | completed | 7de36516d283a810ec5ec01b09980b07b6e634dc | commit subject: Map Sprint 41 Safe Edit Transaction Invariants (resolve in sprint ancestry) | 35 production obligations, 12 focused commands, 16 paths/5000 estimated churn/no binaries mapped; 38 paths, 36 owner selectors, 20 Markdown links/selectors pass; 4 efficiency records equal and 7 manifest tasks contiguous; prompt syntax, explicit suite 8 and repository 22 pass; diff-check pass; Rust gates and zero matched test filters not applicable | Map Sprint 41 Safe Edit Transaction Invariants; local, push deferred to sprint end | unavailable; effective window unknown; preflight warning with narrowed selectors | local-artifacts/codex-runs/sprint-41/task-3/ |
| 4 | 04-review-safe-edit-transaction-design.md | completed | ca385c5bcb0f5eb4ce4b26da32b41bfa54f0b28f | commit subject: Approve Sprint 41 complete invariant evidence design (resolve in sprint ancestry) | Independent and primary complete-audit pass; historical EOF blocked/pass, zero-change Task 5 blocker, T17 blocked range and renewed agreement preserved in design artifact; T01-T35 classified exactly once/12 groups, 455 baseline Markdown references pass; prompt syntax, explicit suite 8 and repository 22 pass; full/correction/working diff-check pass; production gates deferred | Approve Sprint 41 complete invariant evidence design; prior pass b2f89c86012e71190afed077f42b5af82d552b42 remains historical; local, push deferred to sprint end | unavailable; effective window unknown; preflight warning with narrowed selectors | none |
| 5 | 05-implement-safe-edit-transactions.md | blocked | 93661837df8d63bfed10c9b70d1986c4e0d12aa5 | 93661837df8d63bfed10c9b70d1986c4e0d12aa5 | Producer-owned Designer provenance / EDT dependent Query identity blocker; Analysis package check passed; Runtime all-targets check passed before later uncovered edits; Runtime integration exit 101, 1 passed/1 failed (SemanticMismatch, Recovered, 0 retained files); remaining focused/full gates not run | No implementation commit; 9 files/1593 churn preserved; new producer design pass required; initial zero-change T03 attempt retained in history above; push deferred | unavailable | local-artifacts/codex-runs/sprint-41/task-5/ |
| 6 | 06-complete-safe-edit-transaction-evidence.md | not_started | pending | pending | pending | pending | unavailable | none |
| 7 | 07-sprint-41-integration-review.md | not_started | pending | pending | pending | pending | unavailable | none |

## Final report additions

Report starting/ending state, exact HEAD and branch, verified task outcomes and
commits, end-of-sprint push, review identities/reconciliation/consistency,
retirement, measured telemetry or unavailable, logs, remaining changes, and
the exact next action. Never describe planning as implemented transactions.
