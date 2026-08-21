# Review Sprint 18 Graph Query API

Continue OneAgent development.

## Reporting

- Prompt and repository content: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/review.md`

## Template

`docs/codex/templates/review-task.md`

## Authoritative documents

- `docs/Roadmap.md`, Sprint 18 execution plan
- `docs/architecture/graph-query-api-investigation.md`
- `docs/architecture/semantic-model-2.md`
- `docs/Architecture.md`
- `README.md`
- `docs/reviews/sprint-17-workspace-service.md`
- `docs/adr/0002-runtime-composition-root.md`
- `docs/adr/0008-semantic-model-2-knowledge-graph.md`
- `docs/adr/0026-semantic-index-boundary.md`
- `docs/adr/0037-runtime-service-container.md`
- `docs/adr/0038-http-api-health.md`
- `docs/adr/0039-workspace-service.md`
- `docs/adr/0040-graph-query-api.md`

## Prerequisites / Required gate

Require committed Tasks 1-5 in manifest order, successful required focused and
full validation, clean task-owned state, and an exact planning-through-Task-5
commit range. Stop when any task is incomplete or evidence is missing.

## Review target

Review the integrated Sprint 18 planning and Task 1-5 baseline. Do not repair
implementation findings in this task.

## Reviewed baseline / commit range

Resolve and record the exact committed Sprint 18 planning commit through Task 5
range from live history. Treat prompt hashes and historical counts only as
context.

## Scope

### Included

- Planning/framework readiness, investigation completeness, ADR-0040, semantic
  and dependency authority, immutable snapshot/configuration selection,
  accepted graph operations and bounds, owned results/errors, HTTP route and
  schema compatibility, composition, lifecycle/readiness, cancellation and
  shutdown, public EDT/Designer XML evidence, documentation truth, validation,
  and exclusions.
- One review artifact, Roadmap/current-state transition, README/Architecture
  transition text when required, and conditional exact Sprint 17 suite
  retirement after a non-blocking decision.

### Excluded

- Silent implementation fixes, graph semantic changes, watcher/invalidation,
  cache, supported CLI, aggregate graph behavior, new dependency, v0.4 release
  review, Sprint 19 planning, current Sprint 18 suite retirement, older
  non-adjacent suite cleanup, and `.codex/`.

## Review Criteria

- Every Task 1-5 acceptance criterion is proven by committed code, documents,
  focused tests, repository-owned fixtures, and current full validation.
- `SemanticGraph` remains canonical; immutable per-configuration Workspace
  snapshots remain separate; Runtime query behavior only selects, delegates,
  bounds, and projects accepted current semantics.
- Operations, limits, identities, ordering, results, errors, routes, methods,
  statuses, media types, JSON fields/vocabularies, and negative fallback behavior
  match ADR-0040 exactly.
- HTTP uses the existing listener and Runtime composition boundary; health,
  readiness, startup, cancellation, reverse shutdown, address observation, and
  resource cleanup preserve ADR-0037/0038/0039.
- Public tests traverse both production format paths and cover the complete
  accepted success, selection, missing/invalid, bounded, lifecycle, cleanup,
  and repeated-run matrix without fake-only, handler-only, arbitrary-sleep,
  external, ignored-corpus, or platform-specific substitutes.
- Sprints 19-21 and later concerns remain absent, and current docs contain no
  unsupported claim.
- Complete validation succeeds with no blocking findings.

## Acceptance evidence matrix

Record evidence for planning readiness, accepted architecture, semantic and
snapshot authority, configuration/node selection, every operation and bound,
owned result/error projection, route/schema compatibility, composition,
ownership/concurrency, lifecycle/readiness, failures, cancellation/shutdown,
cleanup/repetition, public fixture provenance, documentation, exclusions, and
every focused/full validation command. Issue exactly `pass`, `pass with
non-blocking follow-ups`, or `blocked`.

## Authorized review outputs and state transition

Only after `pass` or `pass with non-blocking follow-ups` and successful complete
validation:

- create `docs/reviews/sprint-18-graph-query-api.md`;
- transition Sprint 18 to `completed` and Sprint 19 File Watching to the unique
  `next` target in `docs/Roadmap.md`;
- synchronize only completion-level current-state text in `README.md`,
  `docs/Architecture.md`, and `docs/architecture/semantic-model-2.md` when
  needed;
- conditionally retire the verified immediately preceding suite exactly as
  defined below;
- commit all review-owned outputs atomically with the suggested message.

The verified preceding suite is
`docs/codex/prompts/sprint-17-workspace-service/`, with exactly these tracked
deletion targets:

- `00-sprint-17-execution-loop.md`
- `01-investigate-workspace-service-boundary.md`
- `02-define-workspace-service-contract.md`
- `03-implement-workspace-snapshot.md`
- `04-implement-workspace-service.md`
- `05-complete-workspace-service-evidence.md`
- `06-sprint-17-integration-review.md`

Re-enumerate and compare the tracked inventory before deletion. Stop if the
directory differs, contains an endangered untracked file, or a retained current
link requires it. Delete only those explicitly enumerated tracked files through
the normal file-editing mechanism. Preserve the complete Sprint 18 suite,
`run-next-sprint.md`, all older non-adjacent suites, and every path outside this
directory. Include the exact deletions with the review artifact and state
transition in one final review commit. If already absent from committed state,
record `already_retired` and keep the ordinary review commit.

## Task-specific Validation

- Re-run the complete non-zero focused matrix recorded by Tasks 3-5.
- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps`
- `git diff --check`
- Verify prompt inventory, retained links, exact commit range, diff scope,
  Roadmap state, next-sprint eligibility, fixture provenance, and final Git
  state.

## Suggested commit message

`Complete Sprint 18 Graph Query API review`

## Final report additions

Report the reviewed range, findings by severity, missing evidence, acceptance
matrix, exact test counts and validation outcomes, decision, review artifact,
state transition, every retired path or `already_retired`, Sprint 19 eligibility,
commit, preserved paths, and final Git state.
