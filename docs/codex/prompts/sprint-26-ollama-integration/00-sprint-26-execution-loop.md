# Execute Sprint 26 Ollama Integration

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

- `docs/Roadmap.md`, Sprint 26 execution plan
- `docs/adr/0045-llm-provider-abstraction.md`
- `docs/adr/0046-openai-compatible-provider.md`
- `docs/adr/0047-lm-studio-integration.md`
- `docs/architecture/semantic-model-2.md`
- `docs/reviews/sprint-25-lm-studio-integration.md`

## Sprint objective and current state

Add one bounded Ollama provider behind the provider-neutral `LlmProvider` seam
with capability-aware fresh discovery, one strict terminal text-generation
path, typed redacted failures, bounded execution, and repository-owned
conformance evidence. The accepted planning state is `next`; it becomes
`active` only when Task 1 starts from the committed planning baseline.

## Starting-state requirements

- The Sprint 26 planning commit is the current committed prerequisite.
- The working tree has no conflicting task-created change.
- `docs/codex/prompts/sprint-26-ollama-integration/` owns this complete suite.
- The verified previous suite is exactly
  `docs/codex/prompts/sprint-25-lm-studio-integration/`, with eight tracked files
  and no untracked addition at planning time.
- Live Ollama calls require current user authorization and are supplementary;
  no task may require a daemon, model, credential, cloud request, or external
  network for acceptance.

## Ordered task manifest

| Order | Prompt | Required committed prerequisite | Task-owned outcome | Validation additions | Suggested commit message |
|---:|---|---|---|---|---|
| 1 | `01-investigate-ollama-integration.md` | Sprint 26 planning commit | Verified repository, official, sanitized local version/catalog, capability, local/cloud, generation, reuse, dependency, error, consumer, and deterministic-oracle evidence. | Documentation consistency, source links, sanitization audit, `git diff --check`. | `Investigate Sprint 26 Ollama integration` |
| 2 | `02-define-ollama-integration.md` | Task 1 | Accepted ADR-0048 for the bounded provider contract. | ADR/investigation consistency, link and decision audit, `git diff --check`. | `Define Sprint 26 Ollama integration` |
| 3 | `03-implement-ollama-client.md` | Task 2 and explicit approval for any new repository production dependency or feature | Concrete adapter foundation with accepted identity, construction, policy, and private wire seam. | Non-zero focused foundation tests, dependency/public-surface audits, full workspace gate. | `Implement Sprint 26 Ollama client` |
| 4 | `04-implement-ollama-discovery.md` | Task 3 | Fresh strict capability-aware model discovery. | Non-zero focused discovery tests, provider-neutral regression, full workspace gate. | `Implement Sprint 26 Ollama discovery` |
| 5 | `05-implement-ollama-generation.md` | Task 4 | One strict terminal text-generation attempt. | Non-zero focused generation tests, concrete-provider regressions, full workspace gate. | `Implement Sprint 26 Ollama generation` |
| 6 | `06-complete-ollama-evidence.md` | Task 5 | Public provider conformance, compatibility evidence, and current-state docs. | Non-zero public/focused/regression targets, compatibility checks, full workspace gate. | `Complete Sprint 26 Ollama evidence` |
| 7 | `07-sprint-26-integration-review.md` | Task 6 and all implementation validation | Review decision, Sprint state transition, Sprint 27 hand-off, and conditional Sprint 25 suite retirement. | Exact focused/public/full review matrix, range/scope/link/retirement audits. | `Complete Sprint 26 Ollama review` |

## Commit authorization mode

Resolve authorization only from the current user instruction launching this
workflow. The current launch requests one separate commit after every
successfully completed task, so stage only task-owned paths and create exactly
one logical commit per completed task. Do not create empty commits for
`already_complete` outcomes.

## Initial audit additions

- Record Sprint start time, exact `HEAD`, `git status --short`, relevant history,
  and available runtime token telemetry.
- Verify every manifest path, authority, prerequisite, and commit message agrees
  with the committed Roadmap plan.
- Recheck the previous-suite tracked, filesystem, and untracked inventories.
- Preserve `.codex/`, unrelated files, and all pre-existing work.

## Task loop additions

- Read each child prompt completely and print its Change Contract before edits.
- Refresh official or live evidence only inside the task's authority boundary.
- Never send generation to a local or cloud Ollama model for acceptance.
- Treat zero matched tests, cloud-dependent evidence, sensitive leakage, or an
  unapproved new dependency/feature as a blocking failure.
- Continue only after the task commit is verified and task-created state is
  clean.

## Already-complete, failure, and review gates

`already_complete` requires current committed evidence and successful required
validation for every acceptance criterion. Stop at the first failed
prerequisite, implementation, validation, staging, commit, or review gate.

Run Task 7 only after Tasks 1-6 are committed or proven `already_complete`.
Only `pass` or `pass with non-blocking follow-ups` plus successful complete
validation may mark Sprint 26 `completed`, make Sprint 27 `next`, and retire the
verified Sprint 25 suite atomically with the review commit.

## Final report additions

Report the ordered task outcomes, timestamps and elapsed times, available token
telemetry, exact commits and subjects, validation results, changed and preserved
paths, `.codex/` status, review decision, Sprint 25 retirement result and exact
deletions, Sprint 27 eligibility, final `HEAD`, and final `git status --short`.
