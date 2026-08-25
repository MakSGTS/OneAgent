# Execute Sprint 24 OpenAI-Compatible Provider

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

- `docs/Roadmap.md`, Sprint 24 execution plan
- `docs/Architecture.md`
- `docs/architecture/semantic-model-2.md`
- `docs/adr/0045-llm-provider-abstraction.md`
- `docs/reviews/sprint-23-llm-provider-abstraction.md`
- `docs/codex/profiles/llm-provider-implementation.md`
- `docs/codex/workflows/llm-provider.md`
- `docs/codex/templates/llm-provider-task.md`
- Task 1 investigation and ADR-0046 after they are committed

## Sprint objective and state

Sprint 24 is `next` at the committed planning baseline. Implement one concrete
OpenAI-compatible provider for fresh `/v1/models` discovery and one
non-streaming `/v1/completions` text-generation attempt through ADR-0045's
provider-neutral seam. Require explicit validated configuration, strict
identity and terminal validation, bounded redacted failures, timeout and
cooperative cancellation, and repository-owned controlled-loopback evidence.

Exclude chat/Responses APIs, streaming, tools, prompt policy, automatic retry,
implicit configuration sources, Runtime exposure, live-provider acceptance,
and broad compatibility claims.

## Starting-state requirements

- Resolve mutable state from the live repository and enforce `AGENTS.md`.
- Require the committed Sprint 24 planning baseline and clean task-owned state.
- Resolve commit authorization only from the current user instruction.
- Preserve unrelated and ignored local artifacts.
- Do not access `192.168.0.176` after Task 1 evidence is committed unless the
  current user instruction explicitly authorizes that exact access again.
- Stop before Task 3 unless the user explicitly approves the exact production
  dependency set accepted by ADR-0046.

## Ordered task manifest

| Order | Prompt | Required committed prerequisite | Task-owned outcome | Validation additions | Suggested commit message |
|---:|---|---|---|---|---|
| 1 | `01-investigate-openai-compatible-provider.md` | Sprint 24 planning baseline | Verified pinned/live/repository wire, transport, dependency, mapping, error, policy, consumer, and oracle evidence. | Read-only source consistency and `git diff --check`. | `Investigate Sprint 24 OpenAI-compatible provider` |
| 2 | `02-define-openai-compatible-provider.md` | Task 1 evidence | Accepted ADR-0046 for the complete bounded adapter contract. | Decision/link consistency and `git diff --check`. | `Define Sprint 24 OpenAI-compatible provider` |
| 3 | `03-implement-openai-compatible-client.md` | ADR-0046 plus explicit dependency approval | Concrete crate, approved dependencies, safe construction/client policy, and wire foundation. | Non-zero construction/URL/auth/redaction tests and full workspace gate. | `Implement Sprint 24 OpenAI-compatible client` |
| 4 | `04-implement-openai-compatible-discovery.md` | Task 3 | Fresh strict `/v1/models` mapping and terminal policy. | Non-zero controlled-loopback discovery tests and full workspace gate. | `Implement Sprint 24 model discovery` |
| 5 | `05-implement-openai-compatible-generation.md` | Task 4 | One strict non-streaming `/v1/completions` attempt. | Non-zero controlled-loopback generation tests and full workspace gate. | `Implement Sprint 24 text generation` |
| 6 | `06-complete-openai-compatible-evidence.md` | Task 5 | Public conformance matrix, compatibility evidence, and current-state docs. | Public adapter target, compatibility matrix, and full workspace gate. | `Complete Sprint 24 OpenAI-compatible evidence` |
| 7 | `07-sprint-24-integration-review.md` | Task 6 and all validation | Review decision, state transition, Sprint 25 hand-off, and conditional Sprint 23 suite retirement. | Exact focused/public/full review matrix. | `Complete Sprint 24 OpenAI-compatible review` |

Do not skip, reorder, combine, or partially commit dependent tasks. Read each
child prompt and all selected modules completely before its task.

## Commit authorization mode

The current launch instruction explicitly authorizes one commit per
successfully completed task. Stage only task-owned paths, use the exact manifest
message, verify the commit and paths, and continue only from a clean task-owned
state. `already_complete` requires committed evidence and successful validation;
never create an empty commit.

## Failure and integration-review gates

Stop after the first prerequisite, dependency-approval, implementation,
validation, staging, commit, or review failure. Preserve evidence and leave
later tasks `not_started`. Run Task 7 only after Tasks 1-6 are committed or
proven `already_complete` and all required validation succeeds.

Only a Task 7 `pass` or `pass with non-blocking follow-ups` plus successful
validation may complete Sprint 24, make Sprint 25 eligible, and retire the
previous suite.

## Previous-suite retirement

The exact immediately preceding suite is
`docs/codex/prompts/sprint-23-llm-provider-abstraction/` with these tracked files:

- `00-sprint-23-execution-loop.md`
- `01-investigate-llm-provider-boundary.md`
- `02-define-llm-provider-abstraction.md`
- `03-implement-provider-domain-model.md`
- `04-implement-capability-aware-requests.md`
- `05-implement-provider-execution-boundary.md`
- `06-complete-llm-provider-evidence.md`
- `07-sprint-23-integration-review.md`

Only Task 7 may delete those exact files, after revalidating tracked,
filesystem, and untracked inventory and issuing a non-blocking decision. Keep
this Sprint 24 suite, `run-next-sprint.md`, non-adjacent suites, and `.codex/`.

## Final report additions

Report start/end HEAD and status, ordered task outcomes, timestamps and elapsed
time, exact commits and validation, dependency approval, changed/preserved
paths, review decision, previous-suite retirement, Sprint 25 eligibility,
tokens when exposed or `недоступно`, and final repository state.
