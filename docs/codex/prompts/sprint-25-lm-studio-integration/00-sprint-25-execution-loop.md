# Execute Sprint 25 LM Studio Integration

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

- `docs/Roadmap.md`, Sprint 25 execution plan
- `docs/Architecture.md`
- `docs/architecture/semantic-model-2.md`
- `docs/adr/0045-llm-provider-abstraction.md`
- `docs/adr/0046-openai-compatible-provider.md`
- `docs/architecture/openai-compatible-provider-investigation.md`
- `docs/reviews/sprint-24-openai-compatible-provider.md`
- `docs/codex/profiles/llm-provider-implementation.md`
- `docs/codex/workflows/llm-provider.md`
- `docs/codex/templates/llm-provider-task.md`
- Task 1 investigation and ADR-0047 after they are committed

## Sprint objective and state

Sprint 25 is `next` at the committed planning baseline. Add one bounded LM
Studio provider behind ADR-0045 with stable provider identity, explicit local
server construction, fresh type-aware model discovery, one accepted terminal
text-generation path, strict identity and response validation, bounded redacted
failures, timeout/cancellation cleanup, and repository-owned controlled-loopback
evidence.

Exclude LM Studio installation or lifecycle management, live-state acceptance,
model download/load/unload, chat history, streaming, tools, MCP, embeddings,
prompt policy, retries, Runtime exposure, and broad compatibility or quality
claims.

## Starting-state requirements

- Resolve mutable state from the live repository and enforce `AGENTS.md`.
- Require the committed Sprint 25 planning baseline and clean task-owned state.
- This execution suite explicitly requires one separate commit after every
  successfully validated task. Do not require another user message to stage
  and commit task-owned paths.
- Preserve unrelated and ignored local artifacts.
- Treat the planning-time macOS observations at `127.0.0.1:1234` as sanitized,
  mutable investigation context, not acceptance evidence.
- Do not access a live LM Studio process, use `lms`, start or stop its server,
  or send model input unless the current user instruction authorizes that local
  access in the active execution context.
- Stop before Task 3 unless the user explicitly approves every new direct
  dependency or dependency feature accepted by ADR-0047. Reuse of the already
  approved ADR-0046 dependency set does not authorize a broader feature set.

## Ordered task manifest

| Order | Prompt | Required committed prerequisite | Task-owned outcome | Validation additions | Suggested commit message |
|---:|---|---|---|---|---|
| 1 | `01-investigate-lm-studio-integration.md` | Sprint 25 planning baseline | Verified repository/official/local wire, model-type, generation, reuse, dependency, error, consumer, and oracle evidence. | Read-only evidence consistency and `git diff --check`. | `Investigate Sprint 25 LM Studio integration` |
| 2 | `02-define-lm-studio-integration.md` | Task 1 evidence | Accepted ADR-0047 for the complete bounded LM Studio provider contract. | Decision/link consistency and `git diff --check`. | `Define Sprint 25 LM Studio integration` |
| 3 | `03-implement-lm-studio-client.md` | ADR-0047 plus approval for any new direct dependency or feature | Concrete adapter/composition foundation, safe construction/client policy, and private wire foundation. | Non-zero construction/identity/redaction tests and full workspace gate. | `Implement Sprint 25 LM Studio client` |
| 4 | `04-implement-lm-studio-discovery.md` | Task 3 | Fresh type-aware discovery that exposes only accepted LLM entries. | Non-zero controlled-loopback discovery and generic-adapter regression tests plus full workspace gate. | `Implement Sprint 25 LM Studio discovery` |
| 5 | `05-implement-lm-studio-generation.md` | Task 4 | One strict non-streaming terminal text-generation attempt. | Non-zero controlled-loopback generation and generic-adapter regression tests plus full workspace gate. | `Implement Sprint 25 LM Studio generation` |
| 6 | `06-complete-lm-studio-evidence.md` | Task 5 | Public conformance matrix, compatibility evidence, and current-state docs. | Public provider targets, compatibility matrix, and full workspace gate. | `Complete Sprint 25 LM Studio evidence` |
| 7 | `07-sprint-25-integration-review.md` | Task 6 and all validation | Review decision, state transition, Sprint 26 hand-off, and conditional Sprint 24 suite retirement. | Exact focused/public/full review matrix. | `Complete Sprint 25 LM Studio review` |

Do not skip, reorder, combine, or partially commit dependent tasks. Read each
child prompt and all selected modules completely before its task.

## Required per-task commit

After every successfully validated task, stage only its exact task-owned paths,
create one commit with the exact manifest message, verify the committed paths
and resulting `HEAD`, and continue only from clean task-owned state. This prompt
is the explicit commit instruction for all eight Sprint 25 task prompts.

If validation fails, the task outcome is partial, or unrelated changes cannot
be excluded from staging, do not commit and stop the execution loop.

`already_complete` requires committed evidence and successful validation;
never create an empty commit.

## Failure and integration-review gates

Stop after the first prerequisite, authorization, dependency-approval,
implementation, validation, staging, commit, or review failure. Preserve
evidence and leave later tasks `not_started`. Run Task 7 only after Tasks 1-6
are committed or proven `already_complete` and all required validation succeeds.

Only a Task 7 `pass` or `pass with non-blocking follow-ups` plus successful
validation may complete Sprint 25, make Sprint 26 eligible, and retire the
previous suite.

## Previous-suite retirement

The exact immediately preceding suite is
`docs/codex/prompts/sprint-24-openai-compatible-provider/` with these tracked
files:

- `00-sprint-24-execution-loop.md`
- `01-investigate-openai-compatible-provider.md`
- `02-define-openai-compatible-provider.md`
- `03-implement-openai-compatible-client.md`
- `04-implement-openai-compatible-discovery.md`
- `05-implement-openai-compatible-generation.md`
- `06-complete-openai-compatible-evidence.md`
- `07-sprint-24-integration-review.md`

Only Task 7 may delete those exact files, after revalidating tracked,
filesystem, and untracked inventory and issuing a non-blocking decision. Keep
this Sprint 25 suite, `run-next-sprint.md`, non-adjacent suites, and `.codex/`.

## Final report additions

Report start/end HEAD and status, ordered task outcomes, timestamps and elapsed
time, exact commits and validation, live-access and dependency approvals,
changed/preserved paths, review decision, previous-suite retirement, Sprint 26
eligibility, tokens when exposed or `недоступно`, and final repository state.
