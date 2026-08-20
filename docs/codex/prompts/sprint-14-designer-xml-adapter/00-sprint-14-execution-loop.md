# Execute Sprint 14 Designer XML Adapter

Continue OneAgent development.

## Reporting

- User-visible reports: Russian.
- Repository content and commit messages: English.
- Report only live repository evidence and successful command results.

## Template and workflow

- `docs/codex/templates/sprint-execution-loop.md`
- `docs/codex/workflows/sequential-sprint-execution.md`

Read both files completely before execution, including every Profile, Template,
Core module, Workflow, ADR, and architecture document selected by each child
task.

## Canonical authorities

- `docs/Roadmap.md`, Sprint 14 execution plan
- `docs/architecture/semantic-model-2.md`
- `docs/architecture/designer-xml-source-corpus.md`
- `docs/reviews/sprint-13-xdto-service-model.md`
- `docs/adr/0004-filesystem-workspace-discovery.md`
- `docs/adr/0005-edt-configuration-loading.md`
- `docs/adr/0007-edt-to-semantic-graph.md`
- `docs/adr/0008-edt-metadata-object-reader.md`
- the Task 1 investigation and Task 2 ADR after they are committed

## Sprint objective and state

Sprint 14 is `next` at planning head
`5b8c57b44247ffed5b26a52877b3b333bbf64703`. Implement the accepted first
slice of a hierarchical Designer XML source adapter while preserving canonical
configuration, supported top-level metadata, module, and BSL declaration
semantics across EDT and Designer sources. Do not claim whole-graph or
field-for-field equivalence.

## Starting-state requirements

- Resolve mutable state from the live repository.
- Require the committed Sprint 14 planning baseline containing this complete
  suite and the matching Roadmap manifest.
- Preserve all pre-existing changes.
- Stop when Sprint 14 is not the unique eligible sprint or a committed
  prerequisite is absent.

The verified immediately preceding suite is
`docs/codex/prompts/sprint-13-xdto-service-model/`, with exactly:

- `00-sprint-13-execution-loop.md`
- `01-implement-xdto-service-graph-model.md`
- `02-parse-xdto-packages.md`
- `03-parse-http-web-services.md`
- `04-emit-xdto-service-semantics.md`
- `05-complete-sprint-13-production-evidence.md`
- `06-sprint-13-integration-review.md`

Only Task 8 may conditionally retire that inventory.

## Commit authorization mode

Resolve commit authorization only from the current user instruction launching
this loop. When it explicitly requests one commit per successful task, stage
only task-owned paths and create the manifest commit after validation. Stored
prompt text does not authorize commits.

## Ordered task manifest

| Order | Prompt | Required committed prerequisite | Task-owned outcome | Validation additions | Suggested commit message |
|---:|---|---|---|---|---|
| 1 | `01-investigate-designer-xml-source-contracts.md` | Sprint 14 planning baseline | Source-contract investigation | Hash/count/path/link checks; `git diff --check` | `Investigate Sprint 14 Designer XML source contracts` |
| 2 | `02-define-designer-xml-adapter-contract.md` | Task 1 | Accepted ADR-0036 contract | Link/scope/decision consistency; `git diff --check` | `Define Sprint 14 Designer XML adapter contract` |
| 3 | `03-implement-designer-xml-discovery.md` | Task 2 | Discovery and configuration loading | Focused workspace/filesystem/adapter tests; full workspace gate | `Implement Sprint 14 Designer XML discovery` |
| 4 | `04-parse-designer-xml-metadata.md` | Task 3 | Metadata artifact parser | Focused adapter parser tests; full workspace gate | `Parse Sprint 14 Designer XML metadata` |
| 5 | `05-parse-designer-xml-modules.md` | Task 4 | Module artifact parser | Focused adapter and BSL tests; full workspace gate | `Parse Sprint 14 Designer XML modules` |
| 6 | `06-emit-designer-xml-semantics.md` | Task 5 | Production graph contribution | Focused adapter/graph/BSL tests; full workspace gate | `Emit Sprint 14 Designer XML semantics` |
| 7 | `07-complete-sprint-14-conformance-evidence.md` | Task 6 | Paired conformance and completion evidence | Conformance/consumer/index tests; full workspace gate | `Complete Sprint 14 conformance evidence` |
| 8 | `08-sprint-14-integration-review.md` | Task 7 and successful implementation validation | Review, transition, and conditional Sprint 13 suite retirement | Complete focused review matrix and full workspace gate | `Complete Sprint 14 Designer XML adapter review` |

## Already-complete, failure, and review gates

- `already_complete` requires current committed evidence and successful required
  validation for every criterion; never create an empty commit.
- Stop at the first prerequisite, implementation, validation, staging, commit,
  or review failure. Do not skip, reorder, combine, or partially commit tasks.
- Run Task 8 only after Tasks 1-7 are committed or proven already complete.
- Only `pass` or `pass with non-blocking follow-ups` plus successful validation
  may complete Sprint 14, make the v0.3 release review eligible, and authorize
  the final review commit.
- Prompt retirement is Task 8's final bounded action and must be atomic with
  the review artifact and Roadmap transition.

## Final report additions

Report the ordered task outcomes, timestamps, elapsed durations, token telemetry
when available, exact commits and subjects, validation results, starting and
ending `HEAD`, initial and final status, changed and preserved paths, `.codex/`
state, review decision, current suite, every retired path, v0.3 release-review
eligibility, Sprint 15 eligibility, and remaining staged or uncommitted work.
