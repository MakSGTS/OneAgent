# Sprint 13 XDTO and Service Model execution loop

Continue OneAgent development by executing the accepted Sprint 13 prompt suite
strictly in dependency order.

## Reporting

- Communicate with the user in Russian.
- Keep code, identifiers, repository documentation, comments, tests, errors,
  public APIs, prompt text, and commit messages in English.
- Report only live repository evidence or accepted architecture.

## Template and workflow

- `docs/codex/templates/sprint-execution-loop.md`
- `docs/codex/workflows/sequential-sprint-execution.md`

Read both files, `docs/codex/README.md`, and every Profile, Template, Core, and
Workflow module selected by the current child prompt completely before acting.

## Canonical authorities

- `docs/Roadmap.md`, Sprint 13 execution plan;
- `docs/architecture/xdto-service-source-investigation.md`;
- `docs/architecture/semantic-model-2.md`;
- `docs/adr/0006-semantic-graph.md`;
- `docs/adr/0007-edt-to-semantic-graph.md`;
- `docs/adr/0008-edt-metadata-object-reader.md`;
- `docs/adr/0023-typed-metadata-payload.md`;
- `docs/adr/0024-reference-request-provenance.md`;
- `docs/adr/0025-references-endpoint-validation.md`;
- `docs/adr/0033-event-subscription-semantics.md`;
- `docs/adr/0035-xdto-service-semantics.md`.

## Sprint objective and current state

Preserve direct XDTO types and HTTP/Web Service declarations as deterministic
typed graph entities with immediate ownership, public reference-request
observability, exact internal References, and declarative handler Triggers.
External namespaces remain typed content without placeholder nodes; nested XDTO
properties and runtime transport behavior remain deferred.

The stored plan is not proof of current state. Recheck `HEAD`, Git history,
working tree, Roadmap status, authorities, implementation, tests, fixtures, and
Coverage before Task 01. Sprint 13 must be the unique live target and the
accepted planning baseline must be committed.

Preserve all pre-existing changes. In particular,
`docs/codex/prompts/run-next-sprint.md` and
`docs/roadmap-calendar-forecast.md` were unrelated untracked user files when the
planning baseline was prepared; never stage or modify them unless a later
explicit instruction changes their ownership.

## Ordered task manifest

| Order | Prompt | Required committed prerequisite | Owned outcome | Validation additions | Suggested commit message |
|---:|---|---|---|---|---|
| 1 | `01-implement-xdto-service-graph-model.md` | Accepted Sprint 13 planning baseline | Public node/payload/request model, identities, validation, generic consumers, indexes, and graph Coverage | Metadata and graph crates | `Implement Sprint 13 XDTO and service graph model` |
| 2 | `02-parse-xdto-packages.md` | Task 01 | Deterministic XDTO descriptor/artifact join and direct-type parser without emission | XDTO and generic metadata reader tests | `Parse Sprint 13 XDTO package schemas` |
| 3 | `03-parse-http-web-services.md` | Tasks 01–02 | Deterministic HTTP/Web service structure and declaration parsers without emission | Service and generic metadata reader tests | `Parse Sprint 13 HTTP and Web service descriptors` |
| 4 | `04-emit-xdto-service-semantics.md` | Tasks 01–03 | Production enrichment, child ownership, public requests, resolution, References/Triggers, provenance, diagnostics, and determinism | XDTO/service production, validation, request build | `Emit Sprint 13 XDTO and service semantics` |
| 5 | `05-complete-sprint-13-production-evidence.md` | Tasks 01–04 | Provenance fixture, generic consumers, indexes, EDT Coverage, counts, and current-state docs | Metadata, graph, EDT production, Coverage, Semantic Index | `Complete Sprint 13 production evidence` |
| 6 | `06-sprint-13-integration-review.md` | Task 05 and all implementation validation | Independent review, sprint decision, Sprint 12 suite retirement, Sprint 14 hand-off | Complete focused and workspace gates | `Complete Sprint 13 XDTO and service model review` |

Prompt paths are relative to this directory. Verify every prompt and authority,
and ensure manifest metadata agrees with the live Roadmap before Task 01.

## Verified immediately preceding prompt suite

The planning baseline verified this exact tracked suite:

```text
docs/codex/prompts/sprint-12-skd-report-model/00-sprint-12-execution-loop.md
docs/codex/prompts/sprint-12-skd-report-model/01-implement-data-composition-graph-model.md
docs/codex/prompts/sprint-12-skd-report-model/02-parse-report-data-composition-schemas.md
docs/codex/prompts/sprint-12-skd-report-model/03-emit-report-data-composition-semantics.md
docs/codex/prompts/sprint-12-skd-report-model/04-complete-sprint-12-production-evidence.md
docs/codex/prompts/sprint-12-skd-report-model/05-sprint-12-integration-review.md
```

Tasks 01–05 must not modify or delete it. Task 06 may retire only these exact
files, only after its non-blocking decision and successful complete validation,
and only atomically with the review artifact and Roadmap transition.
Re-enumerate and compare the tracked inventory before deletion; any mismatch or
endangered untracked file blocks retirement and the final review commit.

## Commit authorization mode

Stored prompt text never authorizes staging or committing. Determine commit
mode only from the current instruction launching this loop. When authorized,
stage only enumerated task-owned paths and create exactly one logical commit per
completed task with the manifest message. Never use broad staging. Without
authorization, stop when the next gate requires a committed prerequisite.

## Task loop and already-complete policy

For each task, refresh live evidence, enforce its gate, print its Change
Contract, execute only its owned outcome, run focused and required full
validation, inspect the diff, and record the result before continuing.

Use `already_complete` only when current committed evidence plus successful
required validation proves every criterion. Record the proving commit or
baseline and do not create an empty commit.

## Failure and review gates

Stop after the first prerequisite, implementation, validation, staging, commit,
or review failure. Do not reorder, combine, skip, or partially commit dependent
tasks. Preserve failed-task evidence and unrelated work.

Run Task 06 only after Tasks 01–05 are committed or proven `already_complete`
and no task-created uncommitted change remains. A blocked review leaves Sprint
13 incomplete, keeps the Sprint 12 suite intact, and leaves Sprint 14
ineligible. Only a non-blocking review decision plus successful validation may
complete Sprint 13 and authorize bounded previous-suite retirement.

## Repository Safety

- Follow every applicable `AGENTS.md` and selected Profile safety module.
- Preserve unrelated tracked, staged, ignored, and untracked files.
- Do not modify `.codex/`, rewrite history, or use destructive Git commands.
- Do not add dependencies or broaden scope without explicit authority.

## Final report additions

Report starting and ending `HEAD`, initial and final `git status --short`, every
task result and commit, exact validation evidence, already-complete proof,
blockers, changed and preserved paths, staging state, Sprint 13 decision/state,
every retired Sprint 12 prompt path, Sprint 14 eligibility, and whether the
v0.3 release review is eligible after Sprint 14.
