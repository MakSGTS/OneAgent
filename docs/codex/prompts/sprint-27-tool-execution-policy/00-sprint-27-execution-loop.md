# Execute Sprint 27 Tool Execution Policy

Continue OneAgent development.

## Reporting

- Prompt and repository changes: English.
- User-visible reports: Russian.
- Stop after the first blocking failure.

## Template and workflow

- `docs/codex/templates/sprint-execution-loop.md`
- `docs/codex/workflows/sequential-sprint-execution.md`

Read both files and every Profile, Template, Core module, Workflow, ADR, and
architecture document selected by each child prompt before acting.

## Canonical authorities

- `docs/Roadmap.md`, Sprint 27 execution plan
- `docs/adr/0037-runtime-service-container.md`
- `docs/adr/0044-context-engine.md`
- `docs/adr/0045-llm-provider-abstraction.md`
- `docs/architecture/semantic-model-2.md`
- `docs/reviews/sprint-26-ollama-integration.md`

## Sprint objective and current state

Define and implement one source-independent, fail-closed AI tool execution
policy boundary with conservative side effects, deterministic authorization,
exact confirmation binding, gated one-attempt execution, typed outcomes, and
bounded redacted audit evidence. The accepted planning state is `next`; it
becomes `active` only when Task 1 starts from the committed planning baseline.

## Starting-state requirements

- The Sprint 27 planning commit is the current committed prerequisite.
- The working tree has no conflicting task-created change.
- `docs/codex/prompts/sprint-27-tool-execution-policy/` owns this complete suite.
- The verified previous suite is exactly
  `docs/codex/prompts/sprint-26-ollama-integration/`, with eight tracked files
  and no untracked addition at planning time.
- The current launch explicitly authorizes one fresh-context read-only reviewer
  agent for Task 7 and one separate commit per successfully completed task.
- No task may require a live provider, external service, credential, privileged
  operation, or real destructive or externally visible action for acceptance.

## Ordered task manifest

| Order | Prompt | Required committed prerequisite | Task-owned outcome | Validation additions | Suggested commit message |
|---:|---|---|---|---|---|
| 1 | `01-investigate-tool-execution-policy.md` | Sprint 27 planning commit | Verified ownership, dependency, identity, sensitivity, effects, policy, confirmation, execution, outcome, audit, consumer, platform, and oracle evidence. | Documentation/evidence consistency, repository inventory, `git diff --check`. | `Investigate Sprint 27 tool execution policy` |
| 2 | `02-define-tool-execution-policy.md` | Task 1 | Accepted ADR-0049 for the bounded source-independent policy contract. | ADR/investigation consistency, link and decision audit, `git diff --check`. | `Define Sprint 27 tool execution policy` |
| 3 | `03-implement-tool-request-domain.md` | Task 2 and approval for any new external production dependency or feature | Additive crate foundation with bounded request/effect/error domain. | Non-zero focused domain tests, dependency/public-surface/redaction audits, full workspace gate. | `Implement Sprint 27 tool request domain` |
| 4 | `04-implement-authorization-policy.md` | Task 3 | Fail-closed deterministic rule evaluation and request-bound decisions. | Non-zero focused policy tests, domain regressions, full workspace gate. | `Implement Sprint 27 authorization policy` |
| 5 | `05-implement-confirmed-execution.md` | Task 4 | Exact confirmation binding, one-attempt execution gate, terminal outcomes, and audit evidence. | Non-zero execution/fake tests, policy regressions, full workspace gate. | `Implement Sprint 27 confirmed execution` |
| 6 | `06-complete-tool-policy-evidence.md` | Task 5 | Public fake-executor conformance, compatibility evidence, and current-state docs. | Non-zero public/focused/compatibility targets, audits, full workspace gate. | `Complete Sprint 27 tool policy evidence` |
| 7 | `07-sprint-27-integration-review.md` | Task 6 and all implementation validation | Independent and primary review evidence, consistency check, decision, state transition, Sprint 28 hand-off, and conditional Sprint 26 suite retirement. | Independent and primary focused/public/full matrices, range/scope/link/retirement audits. | `Complete Sprint 27 tool execution policy review` |

## Commit authorization mode

The current launch requests one separate commit after every successfully
completed task. Stage only task-owned paths and create exactly one logical
commit per completed task. Do not create empty commits for `already_complete`
outcomes.

## Initial audit additions

- Record Sprint start time, exact `HEAD`, `git status --short`, relevant history,
  and available runtime token telemetry.
- Verify every manifest path, authority, prerequisite, and commit message agrees
  with the committed Roadmap plan.
- Recheck the previous-suite tracked, filesystem, and untracked inventories.
- Preserve `.codex/`, unrelated files, and all pre-existing work.

## Task loop additions

- Read each child prompt completely and print its Change Contract before edits.
- Never use real side effects, credentials, external network, or privileged
  execution as acceptance evidence.
- Treat zero matched tests, a policy/confirmation bypass, sensitive leakage, or
  an unapproved new external dependency/feature as a blocking failure.
- Continue only after the task commit is verified and task-created state is
  clean.

## Already-complete, failure, and review gates

`already_complete` requires current committed evidence and successful required
validation for every acceptance criterion. Stop at the first failed
prerequisite, implementation, validation, staging, commit, or review gate.

Run Task 7 only after Tasks 1-6 are committed or proven `already_complete`.
Task 7 must use one separate fresh-context read-only reviewer under
`docs/codex/workflows/review.md`, preserve its report independently, reconcile
it with primary evidence, and obtain the same reviewer's final artifact-
consistency check. Only a non-blocking effective decision plus successful
validation may mark Sprint 27 `completed`, make Sprint 28 `next`, and retire the
verified Sprint 26 suite atomically with the review commit.

## Final report additions

Report ordered task outcomes, timestamps and elapsed times, available token
telemetry, exact commits and subjects, validation, independent reviewer identity
and read-only/fresh-context confirmation, reviewer recommendation, evidence
discrepancies and consistency result, changed/preserved paths, `.codex/` status,
Sprint 26 retirement and deletions, Sprint 28 eligibility, final `HEAD`, and
final `git status --short`.
