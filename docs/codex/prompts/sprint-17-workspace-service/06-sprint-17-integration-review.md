# Review Sprint 17 Workspace Service

Continue OneAgent development.

## Reporting

- Prompt and repository content: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/review.md`

## Template

`docs/codex/templates/review-task.md`

## Authoritative documents

- `docs/Roadmap.md`, Sprint 17 execution plan
- `docs/architecture/workspace-service-investigation.md`
- `docs/architecture/semantic-model-2.md`
- `docs/Architecture.md`
- `README.md`
- `docs/reviews/sprint-16-http-api-health.md`
- `docs/adr/0002-runtime-composition-root.md`
- `docs/adr/0004-filesystem-workspace-discovery.md`
- `docs/adr/0036-designer-xml-adapter.md`
- `docs/adr/0037-runtime-service-container.md`
- `docs/adr/0038-http-api-health.md`
- `docs/adr/0039-workspace-service.md`

## Prerequisites / Required gate

Require committed Tasks 1-5 in manifest order, successful required focused and
full validation, clean task-owned state, and an exact planning-through-Task-5
commit range. Stop when any task is incomplete or evidence is missing.

## Review target

Review the integrated Sprint 17 planning and Task 1-5 baseline. Do not repair
implementation findings in this task.

## Reviewed baseline / commit range

Resolve and record the exact committed Sprint 17 planning commit through Task 5
range from live history. Treat prompt hashes and historical counts only as
context.

## Scope

### Included

- Planning/framework readiness, investigation completeness, ADR-0039,
  dependency direction, Workspace configuration/discovery/build dispatch,
  immutable snapshot authority, lifecycle/readiness, ownership, failures,
  cancellation/shutdown, public EDT/Designer XML evidence, documentation truth,
  validation, and exclusions.
- One review artifact, Roadmap/current-state transition, README/Architecture
  transition text when required, and conditional exact Sprint 16 suite
  retirement after a non-blocking decision.

### Excluded

- Silent implementation fixes, new graph-query or Workspace HTTP APIs, watcher,
  cache, CLI, dependency or semantic changes, v0.4 release review, Sprint 18
  planning, current Sprint 17 suite retirement, older non-adjacent suite cleanup,
  and `.codex/`.

## Review Criteria

- Every Task 1-5 acceptance criterion is proven by committed code, documents,
  focused tests, repository-owned fixtures, and current full validation.
- Runtime remains the composition root; adapters retain parsing/discovery
  ownership; published semantic graphs remain canonical immutable authorities.
- Root validation, stable ordering, supported format dispatch, duplicate or
  collision policy, diagnostics, and atomic snapshot publication match
  ADR-0039.
- Startup acknowledgement and readiness occur only from accepted owned evidence;
  failure, cancellation, reverse shutdown, cleanup, and repeated runs preserve
  ADR-0037/0038.
- Public tests traverse both production format paths and cover the accepted
  positive, empty/multiple, failing, readiness, cleanup, and repeated-run matrix
  without fake-only, arbitrary-sleep, external, ignored-corpus, or
  platform-specific substitutes.
- Sprint 18-21 and later concerns remain absent, and current docs contain no
  unsupported claim.
- Complete validation succeeds with no blocking findings.

## Acceptance evidence matrix

Record evidence for planning readiness, accepted architecture, configuration,
discovery, both builder paths, snapshot identity/order/atomicity, diagnostics,
ownership/concurrency, lifecycle/readiness, failures, cancellation/shutdown,
cleanup/repetition, public fixture provenance, documentation, exclusions, and
every focused/full validation command. Issue exactly `pass`, `pass with
non-blocking follow-ups`, or `blocked`.

## Authorized review outputs and state transition

Only after `pass` or `pass with non-blocking follow-ups` and successful complete
validation:

- create `docs/reviews/sprint-17-workspace-service.md`;
- transition Sprint 17 to `completed` and Sprint 18 Graph Query API to the unique
  `next` target in `docs/Roadmap.md`;
- synchronize only completion-level current-state text in `README.md`,
  `docs/Architecture.md`, and `docs/architecture/semantic-model-2.md` when
  needed;
- conditionally retire the verified immediately preceding suite exactly as
  defined below;
- commit all review-owned outputs atomically with the suggested message.

The verified preceding suite is
`docs/codex/prompts/sprint-16-http-api-health/`, with exactly these tracked
deletion targets:

- `00-sprint-16-execution-loop.md`
- `01-investigate-http-api-health-boundary.md`
- `02-define-http-api-health-contract.md`
- `03-implement-runtime-health-state.md`
- `04-implement-http-service.md`
- `05-complete-http-api-health-evidence.md`
- `06-sprint-16-integration-review.md`

Re-enumerate and compare the tracked inventory before deletion. Stop if the
directory differs, contains an endangered untracked file, or a retained current
link requires it. Delete only those explicitly enumerated tracked files through
the normal file-editing mechanism. Preserve the complete Sprint 17 suite,
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

`Complete Sprint 17 Workspace service review`

## Final report additions

Report the reviewed range, findings by severity, missing evidence, acceptance
matrix, exact test counts and validation outcomes, decision, review artifact,
state transition, every retired path or `already_retired`, Sprint 18 eligibility,
commit, preserved paths, and final Git state.
