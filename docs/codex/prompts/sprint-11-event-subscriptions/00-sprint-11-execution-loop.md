# Sprint 11 Event Subscriptions execution loop

Continue OneAgent development by executing the accepted Sprint 11 prompt suite
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

- `docs/Roadmap.md`, Sprint 11 execution plan;
- `docs/architecture/event-subscription-source-investigation.md`;
- `docs/architecture/semantic-model-2.md`;
- `docs/adr/0006-semantic-graph.md`;
- `docs/adr/0007-edt-to-semantic-graph.md`;
- `docs/adr/0008-edt-metadata-object-reader.md`;
- `docs/adr/0012-bsl-symbols-in-semantic-graph.md`;
- `docs/adr/0016-cross-module-bsl-call-resolution.md`;
- `docs/adr/0023-typed-metadata-payload.md`;
- `docs/adr/0024-reference-request-provenance.md`;
- `docs/adr/0025-references-endpoint-validation.md`;
- `docs/adr/0033-event-subscription-semantics.md`.

## Sprint objective and current state

Discover repository-proven EDT Event Subscriptions, preserve UUID identity and
typed event content, resolve supported source selectors and Common Module
handler procedures, emit direct provenance-backed References and Triggers, and
retain deterministic unrelated behavior without inventing unsupported metadata
families or runtime dispatch semantics.

The stored plan is not proof of current state. Recheck `HEAD`, Git history,
working tree, Roadmap status, authorities, implementation, tests, fixtures, and
Coverage before Task 01. Sprint 11 must be the unique live target and the
accepted planning baseline must be committed.

Preserve all pre-existing changes. In particular,
`docs/codex/prompts/run-next-sprint.md` and
`docs/roadmap-calendar-forecast.md` were unrelated untracked user files when the
planning baseline was prepared; never stage or modify them unless a later
explicit instruction changes their ownership.

## Ordered task manifest

| Order | Prompt | Required committed prerequisite | Owned outcome | Validation additions | Suggested commit message |
|---:|---|---|---|---|---|
| 1 | `01-implement-event-subscription-graph-model.md` | Accepted Sprint 11 planning baseline | Metadata/payload model, References/Triggers endpoints, exhaustive public consumers, generic graph evidence | Metadata and graph crates | `Implement Sprint 11 event subscription graph model` |
| 2 | `02-parse-event-subscription-descriptors.md` | Task 01 | Typed deterministic EDT descriptor, selector, event, and handler parsing without graph emission | Event Subscription and generic metadata reader tests | `Parse Sprint 11 event subscription descriptors` |
| 3 | `03-resolve-event-subscription-targets.md` | Tasks 01–02 | Exact/family source and owned-handler resolution outcomes without production integration | Event Subscription resolution and graph resolution tests | `Resolve Sprint 11 event subscription targets` |
| 4 | `04-emit-event-subscription-semantics.md` | Tasks 01–03 | Production discovery, nodes, ownership, References, Triggers, provenance, diagnostics, statistics | Event Subscription production and graph validation tests | `Emit Sprint 11 event subscription semantics` |
| 5 | `05-complete-sprint-11-production-evidence.md` | Tasks 01–04 | Provenance fixture, consumers, indexes, Coverage, aggregate counts, current-state docs | Metadata, graph, EDT production, Coverage, Semantic Index | `Complete Sprint 11 production evidence` |
| 6 | `06-sprint-11-integration-review.md` | Task 05 and all implementation validation | Independent review, sprint decision, Sprint 10 suite retirement, Sprint 12 hand-off | Complete focused and workspace gates | `Complete Sprint 11 event subscriptions review` |

Prompt paths are relative to this directory. Verify every prompt and authority,
and ensure manifest metadata agrees with the live Roadmap before Task 01.

## Verified immediately preceding prompt suite

The planning baseline verified this exact tracked suite:

```text
docs/codex/prompts/sprint-10-subsystems-composition/00-sprint-10-execution-loop.md
docs/codex/prompts/sprint-10-subsystems-composition/01-implement-subsystem-hierarchy-graph-rules.md
docs/codex/prompts/sprint-10-subsystems-composition/02-parse-nested-subsystem-hierarchy.md
docs/codex/prompts/sprint-10-subsystems-composition/03-emit-nested-subsystem-composition.md
docs/codex/prompts/sprint-10-subsystems-composition/04-complete-sprint-10-production-evidence.md
docs/codex/prompts/sprint-10-subsystems-composition/05-sprint-10-integration-review.md
```

Tasks 01–05 must not modify or delete it. Task 06 may retire only these exact
files, only after its non-blocking decision and successful complete validation,
and only atomically with the review artifact and Roadmap transition. Re-enumerate
and compare the tracked inventory before deletion; any mismatch or endangered
untracked file blocks retirement and the final review commit.

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
11 incomplete, keeps the Sprint 10 suite intact, and leaves Sprint 12
ineligible. Only a non-blocking review decision plus successful validation may
complete Sprint 11 and authorize bounded previous-suite retirement.

## Repository Safety

- Follow every applicable `AGENTS.md` and selected Profile safety module.
- Preserve unrelated tracked, staged, ignored, and untracked files.
- Do not modify `.codex/`, rewrite history, or use destructive Git commands.
- Do not add dependencies or broaden scope without explicit authority.

## Final report additions

Report starting and ending `HEAD`, initial and final `git status --short`, every
task result and commit, exact validation evidence, already-complete proof,
blockers, changed and preserved paths, staging state, Sprint 11 decision/state,
every retired Sprint 10 prompt path, and Sprint 12 eligibility.
