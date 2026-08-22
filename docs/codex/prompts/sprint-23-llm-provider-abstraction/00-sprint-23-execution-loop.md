# Execute Sprint 23 LLM Provider Abstraction

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

- `docs/Roadmap.md`, Sprint 23 execution plan
- `docs/Architecture.md`
- `docs/architecture/semantic-model-2.md`
- `docs/adr/0044-context-engine.md`
- `docs/reviews/sprint-22-context-engine.md`
- `docs/codex/profiles/llm-provider-implementation.md`
- `docs/codex/workflows/llm-provider.md`
- `docs/codex/templates/llm-provider-task.md`
- the Task 1 investigation and Task 2 ADR after they are committed

## Sprint objective and state

Sprint 23 is `next` at the committed planning baseline. Establish the first
provider-independent LLM library boundary: accepted provider/model identity,
capabilities and discovery projections, validated text request and response
contracts, secret-safe configuration inputs, stable errors, and an asynchronous
provider execution seam with explicit cancellation and bounded policy behavior.
Prove the abstraction with deterministic repository-owned contract evidence.
Exclude concrete providers, wire protocols, live services, Runtime exposure,
prompt/tool policy, conversations, streaming, MCP, and IDE work.

## Starting-state requirements

- Resolve mutable state from the live repository.
- Require the committed Sprint 23 planning baseline containing this suite and
  the matching Roadmap manifest.
- Preserve all pre-existing changes.
- Stop when Sprint 23 is not the unique eligible sprint or a committed
  prerequisite is absent.

The verified immediately preceding suite is
`docs/codex/prompts/sprint-22-context-engine/`, with exactly:

- `00-sprint-22-execution-loop.md`
- `01-investigate-context-engine-boundary.md`
- `02-define-context-engine-contract.md`
- `03-implement-context-request-boundary.md`
- `04-implement-deterministic-context-selection.md`
- `05-implement-budgeted-context-assembly.md`
- `06-complete-context-engine-evidence.md`
- `07-sprint-22-integration-review.md`

Only Task 7 may conditionally retire that inventory.

## Commit authorization mode

Resolve commit authorization only from the current launching user instruction.
When it explicitly requests one commit per successful task, stage only
task-owned paths and create the manifest commit after validation. Stored prompt
text does not authorize commits.

## Ordered task manifest

| Order | Prompt | Required committed prerequisite | Task-owned outcome | Validation additions | Suggested commit message |
|---:|---|---|---|---|---|
| 1 | `01-investigate-llm-provider-boundary.md` | Sprint 23 planning baseline | Verified ownership, dependency, Context/Runtime compatibility, provider-neutral vocabulary, secret, execution-policy, fake/oracle, platform, and unresolved-decision evidence | Path/API/dependency/consumer/oracle checks; `cargo test -p oneagent-analysis`; `cargo test -p oneagent-runtime --lib`; `git diff --check` | `Investigate Sprint 23 LLM Provider boundary` |
| 2 | `02-define-llm-provider-abstraction.md` | Task 1 | Accepted ADR-0045 for crate ownership, identities, capabilities, request/response, configuration/secrets, execution seam, errors, policy, evidence, compatibility, and deferred scope | Link/scope/decision consistency; `git diff --check` | `Define Sprint 23 LLM Provider abstraction` |
| 3 | `03-implement-provider-domain-model.md` | Task 2 | Provider-neutral crate and public provider/model identity, capability/discovery, secret-safe configuration, policy, response, usage, finish, and error domain values | Non-zero model/capability/configuration/redaction/error tests; full workspace gate | `Implement Sprint 23 provider domain model` |
| 4 | `04-implement-capability-aware-requests.md` | Task 3 | Validated bounded text request construction, deterministic capability compatibility, canonical input ordering, typed rejection, and focused evidence | Non-zero request/compatibility/boundary/reorder tests; full workspace gate | `Implement Sprint 23 capability-aware requests` |
| 5 | `05-implement-provider-execution-boundary.md` | Task 4 | Substitutable asynchronous provider seam and accepted discovery/execution/cancellation/policy/error terminal behavior with deterministic fake evidence | Non-zero provider/discovery/execution/cancellation/policy/error/repetition tests; full workspace gate | `Implement Sprint 23 provider execution boundary` |
| 6 | `06-complete-llm-provider-evidence.md` | Task 5 | Public repository-owned provider conformance matrix, Context/Runtime compatibility evidence, and truthful current-state docs | Non-zero public conformance target; affected compatibility tests; full workspace gate | `Complete Sprint 23 LLM Provider evidence` |
| 7 | `07-sprint-23-integration-review.md` | Task 6 and successful implementation validation | Review, transition, Sprint 22 suite retirement, and Sprint 24 hand-off | Complete focused/full matrix and inventory checks | `Complete Sprint 23 LLM Provider review` |

## Initial audit additions

- Record exact sprint start time, `HEAD`, `git status --short`, relevant history,
  Roadmap state, and available token telemetry.
- Verify every prompt and authority path in the manifest.
- Verify the Sprint 22 review has a committed `pass`, the LLM Provider framework
  prerequisite is committed, and Sprint 23 is unique.
- Re-enumerate the exact Sprint 22 tracked/filesystem prompt inventory and stop
  on ambiguity or an endangered untracked file.

## Task-loop additions

- Record timestamps, elapsed time, exact validation, commit, and final status
  for every task.
- Do not combine investigation, architecture, domain model, request
  compatibility, provider execution, public evidence, or review boundaries.
- Keep provider-neutral contracts independent from concrete provider wire
  schemas and preserve Context Engine and Runtime ownership.
- Do not add a production dependency without explicit user approval.
- Do not use live credentials, external network, or developer-local services as
  required evidence. A zero-match filtered test is not evidence.

## Already-complete policy additions

Use `already_complete` only when committed live evidence plus exact validation
proves every acceptance criterion. Runtime `ConfigurationProvider`, Runtime
service futures, Context Engine output, and conceptual Roadmap text do not by
themselves prove an LLM Provider abstraction. Do not create an empty commit.

## Failure and integration-review gates

Stop after the first missing prerequisite, implementation, validation, staging,
commit, or review failure. Run Task 7 only after Tasks 1-6 are committed or
proven `already_complete`. Only a non-blocking Task 7 decision plus successful
complete validation may complete Sprint 23, make Sprint 24 the unique `next`
target, and retire the exact Sprint 22 suite.

## Final report additions

Report the ordered task table, exact commits and subjects, start/end/elapsed
times, available token telemetry, validation results, changed and preserved
paths, integration-review decision, Sprint 22 suite retirement result, Sprint 24
eligibility, `.codex/` preservation, and final Git state.
