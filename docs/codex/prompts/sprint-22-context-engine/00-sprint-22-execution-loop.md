# Execute Sprint 22 Context Engine

Continue OneAgent development.

## Reporting

- User-visible reports: Russian.
- Repository content and commit messages: English.
- Report only live repository evidence and successful command results.

## Template and workflow

- `docs/codex/templates/sprint-execution-loop.md`
- `docs/codex/workflows/sequential-sprint-execution.md`

Read both completely before execution, including every Profile, Template, Core
module, Workflow, ADR, and architecture document selected by each child task.

## Canonical authorities

- `docs/Roadmap.md`, Sprint 22 execution plan
- `docs/Architecture.md`
- `docs/architecture/semantic-model-2.md`
- `docs/reviews/v0.4-release-review.md`
- `docs/codex/profiles/context-engine-implementation.md`
- `docs/codex/workflows/context-engine.md`
- `docs/codex/templates/context-engine-task.md`
- the Task 1 investigation and Task 2 ADR after they are committed

## Sprint objective and state

Sprint 22 is `next` at the committed planning baseline. Build the first
source-independent deterministic Context Engine over one immutable canonical
graph snapshot: accepted seed resolution, bounded relevant candidate selection,
exact budget admission, provenance-backed explanations, stable semantic
rendering, and reproducible evaluation. Preserve canonical graph/query authority
and exclude source text, providers, models, transports, tools, MCP, and IDE work.

## Starting-state requirements

- Resolve mutable state from the live repository.
- Require the committed Sprint 22 planning baseline containing this suite and
  the matching Roadmap manifest.
- Preserve all pre-existing changes.
- Stop when Sprint 22 is not the unique eligible sprint or a committed
  prerequisite is absent.

The verified immediately preceding suite is
`docs/codex/prompts/sprint-21-cli-client/`, with exactly:

- `00-sprint-21-execution-loop.md`
- `01-investigate-cli-client-boundary.md`
- `02-define-cli-client-contract.md`
- `03-implement-cli-command-boundary.md`
- `04-implement-runtime-http-client.md`
- `05-complete-cli-client-evidence.md`
- `06-sprint-21-integration-review.md`

Only Task 7 may conditionally retire that inventory.

## Commit authorization mode

Resolve commit authorization only from the current launching user instruction.
When it explicitly requests one commit per successful task, stage only
task-owned paths and create the manifest commit after validation. Stored prompt
text does not authorize commits.

## Ordered task manifest

| Order | Prompt | Required committed prerequisite | Task-owned outcome | Validation additions | Suggested commit message |
|---:|---|---|---|---|---|
| 1 | `01-investigate-context-engine-boundary.md` | Sprint 22 planning baseline | Graph/query/provenance, analysis, consumer, dependency, request/seed, selection, budget, rendering, evaluation, fixture, platform, and compatibility investigation | Path/API/dependency/fixture/oracle checks; `cargo test -p oneagent-analysis`; `git diff --check` | `Investigate Sprint 22 Context Engine` |
| 2 | `02-define-context-engine-contract.md` | Task 1 | Accepted ADR-0044 canonical authority, request/seed/policy, selection, budget, bundle, provenance, explanation, rendering, evaluation, compatibility, and deferred-scope contract | Link/scope/decision consistency; `git diff --check` | `Define Sprint 22 Context Engine contract` |
| 3 | `03-implement-context-request-boundary.md` | Task 2 | Public request/policy/budget/bundle domain boundary, validation, accepted seed resolution, typed failures, and focused deterministic evidence | Non-zero request/validation/seed tests; full workspace gate | `Implement Sprint 22 context request boundary` |
| 4 | `04-implement-deterministic-context-selection.md` | Task 3 | Accepted traversal/filtering, relevance ordering, ties, bounds, deduplication, provenance paths, and focused evidence | Non-zero selection/order/bound/repetition tests; full workspace gate | `Implement Sprint 22 deterministic context selection` |
| 5 | `05-implement-budgeted-context-assembly.md` | Task 4 | Accepted cost/admission, explicit omission/truncation, bundle assembly, provenance/explanations, stable rendering, and focused evidence | Non-zero budget/assembly/rendering tests; full workspace gate | `Implement Sprint 22 budgeted context assembly` |
| 6 | `06-complete-context-engine-evidence.md` | Task 5 | Public repository-owned evaluation matrix, compatibility/regression evidence, and current-state docs | Non-zero public evaluation target; full workspace gate | `Complete Sprint 22 Context Engine evidence` |
| 7 | `07-sprint-22-integration-review.md` | Task 6 and successful implementation validation | Review, transition, Sprint 21 suite retirement, and Sprint 23 hand-off | Complete focused/full matrix and inventory checks | `Complete Sprint 22 Context Engine review` |

## Initial audit additions

- Record exact Sprint start time, `HEAD`, `git status --short`, relevant history,
  Roadmap state, and available token telemetry.
- Verify every prompt and authority path in the manifest.
- Verify the v0.4 release has a committed `pass`, the Context Engine framework
  prerequisite is committed, and Sprint 22 is unique.
- Re-enumerate the exact Sprint 21 tracked/filesystem prompt inventory and stop
  on ambiguity or an endangered untracked file.

## Task-loop additions

- Record timestamps, elapsed time, exact validation, commit, and final status
  for every task.
- Do not combine investigation, architecture, request/seed boundary, selection,
  budgeted assembly/rendering, public evidence, or review boundaries.
- Keep `SemanticGraph` and accepted queries authoritative and read-only.
- Do not add a production dependency without explicit user approval.
- A zero-match filtered test is not evidence.

## Already-complete policy additions

Use `already_complete` only when committed live evidence plus exact validation
proves every acceptance criterion. Conceptual Context Engine text, generic graph
traversal, and Impact APIs do not prove the Sprint 22 capability. Do not create
an empty commit.

## Failure and integration-review gates

Stop after the first missing prerequisite, implementation, validation, staging,
commit, or review failure. Run Task 7 only after Tasks 1-6 are committed or
proven `already_complete`. Only a non-blocking Task 7 decision plus successful
complete validation may complete Sprint 22, make Sprint 23 the unique `next`
target, and retire the exact Sprint 21 suite.

## Final report additions

Report the ordered task table, exact commits and subjects, start/end/elapsed
times, available token telemetry, validation results, changed and preserved
paths, integration-review decision, Sprint 21 suite retirement result, Sprint 23
eligibility, `.codex/` preservation, and final Git state.
