# Execute Sprint 21 CLI Client

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

- `docs/Roadmap.md`, Sprint 21 execution plan
- `docs/Architecture.md`
- `docs/architecture/semantic-model-2.md`
- `docs/reviews/sprint-20-persistent-cache.md`
- `docs/adr/0037-runtime-service-container.md`
- `docs/adr/0038-http-api-health.md`
- `docs/adr/0039-workspace-service.md`
- `docs/adr/0040-graph-query-api.md`
- `docs/adr/0041-file-watching.md`
- `docs/adr/0042-persistent-cache.md`
- the Task 1 investigation and Task 2 ADR after they are committed

## Sprint objective and state

Sprint 21 is `next` at the committed planning baseline. Replace the CLI
placeholder with the first supported deterministic client for accepted Runtime
health, Workspace configuration listing, exact node lookup, direct relations,
and bounded traversal. Preserve every existing Runtime, HTTP, semantic, watcher,
cache, lifecycle, and source-adapter contract.

## Starting-state requirements

- Resolve mutable state from the live repository.
- Require the committed Sprint 21 planning baseline containing this suite and
  the matching Roadmap manifest.
- Preserve all pre-existing changes.
- Stop when Sprint 21 is not the unique eligible sprint or a committed
  prerequisite is absent.

The verified immediately preceding suite is
`docs/codex/prompts/sprint-20-persistent-cache/`, with exactly:

- `00-sprint-20-execution-loop.md`
- `01-investigate-persistent-cache-boundary.md`
- `02-define-persistent-cache-contract.md`
- `03-implement-snapshot-cache-codec.md`
- `04-implement-cache-storage-invalidation.md`
- `05-integrate-runtime-cache-lifecycle.md`
- `06-complete-persistent-cache-evidence.md`
- `07-sprint-20-integration-review.md`

Only Task 6 may conditionally retire that inventory.

## Commit authorization mode

Resolve commit authorization only from the current launching user instruction.
When it explicitly requests one commit per successful task, stage only
task-owned paths and create the manifest commit after validation. Stored prompt
text does not authorize commits.

## Ordered task manifest

| Order | Prompt | Required committed prerequisite | Task-owned outcome | Validation additions | Suggested commit message |
|---:|---|---|---|---|---|
| 1 | `01-investigate-cli-client-boundary.md` | Sprint 21 planning baseline | CLI, Runtime wire/configuration, HTTP feasibility, dependency, fixture, platform, consumer, command/output/error/exit, and deterministic-test investigation | Path/API/dependency/platform/fixture/oracle checks; `git diff --check` | `Investigate Sprint 21 CLI Client` |
| 2 | `02-define-cli-client-contract.md` | Task 1 | Accepted ADR-0043 ownership, grammar, endpoint, request, response, output, error, exit, resource, compatibility, dependency, and deferred-scope contract | Link/scope/decision consistency; `git diff --check` | `Define Sprint 21 CLI Client contract` |
| 3 | `03-implement-cli-command-boundary.md` | Task 2 | Command parsing, local validation, help/version, request model, diagnostics, output routing, exit classification, and focused evidence | Non-zero command/validation/output tests; full workspace gate | `Implement Sprint 21 CLI command boundary` |
| 4 | `04-implement-runtime-http-client.md` | Task 3 | Bounded HTTP/1.1 client, request encoding, response framing/validation, JSON passthrough, failure behavior, cleanup, and focused evidence | Non-zero client/protocol/failure/repetition tests; full workspace gate | `Implement Sprint 21 Runtime HTTP client` |
| 5 | `05-complete-cli-client-evidence.md` | Task 4 | Public CLI-to-production-Runtime operation/failure/output/exit/shutdown/repetition matrix and current-state docs | Non-zero public client/server matrix; full workspace gate | `Complete Sprint 21 CLI Client evidence` |
| 6 | `06-sprint-21-integration-review.md` | Task 5 and successful implementation validation | Review, transition, and conditional Sprint 20 suite retirement | Complete focused/full matrix and inventory checks | `Complete Sprint 21 CLI Client review` |

## Initial audit additions

- Record exact Sprint start time, `HEAD`, `git status --short`, relevant history,
  Roadmap state, and available token telemetry.
- Verify every prompt and authority path in the manifest.
- Verify Sprint 20 has a committed non-blocking review and Sprint 21 is unique.
- Re-enumerate the exact Sprint 20 tracked/filesystem prompt inventory and stop
  on ambiguity or an endangered untracked file.

## Task-loop additions

- Record timestamps, elapsed time, exact validation, commit, and final status
  for every task.
- Do not combine investigation, architecture, command boundary, HTTP client, or
  public production evidence across task boundaries.
- Preserve all accepted Runtime route/schema/lifecycle, Workspace publication,
  Graph Query ordering/snapshot, File Watching, cache, graph, and adapter
  behavior.
- Do not add a production dependency without explicit user approval.
- A zero-match filtered test is not evidence.

## Already-complete policy additions

Use `already_complete` only when committed live evidence plus exact validation
proves every acceptance criterion. Existing raw TCP test helpers and the CLI
placeholder do not prove a supported client. Do not create an empty commit.

## Failure and integration-review gates

Stop after the first missing prerequisite, implementation, validation, staging,
commit, or review failure. Run Task 6 only after Tasks 1-5 are committed or
proven `already_complete`. Only a non-blocking Task 6 decision plus successful
complete validation may complete Sprint 21, make the v0.4 release review
eligible, and retire the exact Sprint 20 suite.

## Final report additions

Report the ordered task table, exact commits and subjects, start/end/elapsed
times, available token telemetry, validation results, changed and preserved
paths, integration-review decision, Sprint 20 suite retirement result, v0.4
release-review eligibility, `.codex/` preservation, and final Git state.
