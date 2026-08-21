# Review Sprint 16 HTTP API and Health

Continue OneAgent development.

## Reporting

- Prompt and repository content: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/review.md`

## Template

`docs/codex/templates/review-task.md`

## Authoritative documents

- `docs/Roadmap.md`, Sprint 16 execution plan
- `docs/architecture/http-api-health-investigation.md`
- `docs/architecture/semantic-model-2.md`
- `docs/Architecture.md`
- `README.md`
- `docs/reviews/sprint-15-runtime-service-container.md`
- `docs/adr/0002-runtime-composition-root.md`
- `docs/adr/0037-runtime-service-container.md`
- `docs/adr/0038-http-api-health.md`

## Prerequisites / Required gate

Require committed Tasks 1-5 in manifest order, successful required focused and
full validation, clean task-owned state, and an exact planning-through-Task-5
commit range. Stop when any task is incomplete or evidence is missing.

## Review target

Review the integrated Sprint 16 planning and Task 1-5 baseline. Do not repair
implementation findings in this task.

## Reviewed baseline / commit range

Resolve and record the exact committed Sprint 16 planning commit through Task 5
range from live history. Treat prompt hashes and historical counts only as
context.

## Scope

### Included

- Planning and framework readiness, investigation completeness, ADR-0038,
  lifecycle-derived health, public API/configuration, listener/task ownership,
  HTTP wire compatibility, failures, cancellation, shutdown, public loopback
  evidence, documentation truth, validation, and exclusions.
- One review artifact, Roadmap/current-state transition, README/Architecture
  transition text when required, and conditional exact Sprint 15 suite
  retirement after a non-blocking decision.

### Excluded

- Silent implementation fixes, new routes or schemas, dependency changes,
  workspace/graph/CLI behavior, release review, Sprint 17 planning, current
  Sprint 16 suite retirement, older non-adjacent suite cleanup, and `.codex/`.

## Review Criteria

- Every Task 1-5 acceptance criterion is proven by committed code, documents,
  focused tests, and current full validation.
- Health/readiness is derived only from owned lifecycle evidence and follows the
  exact ADR-0038 state/status/schema matrix.
- Listener acquisition, startup failure, service task ownership, cancellation,
  graceful shutdown, release/rebind, and repeated runs preserve ADR-0037.
- Public loopback tests cover the accepted success and negative wire contract;
  no handler-only, arbitrary-sleep, external-service, or platform-specific
  evidence substitutes for it.
- Later Sprint 17-21 and security/operability scope remains absent and current
  docs contain no unsupported claims.
- Complete validation succeeds with no blocking findings.

## Acceptance evidence matrix

Record evidence for planning readiness, accepted architecture, health state,
HTTP listener and wire contract, bind failure, cancellation/shutdown, resource
cleanup, repeated runs, documentation, exclusions, and every focused/full
validation command. Issue exactly `pass`, `pass with non-blocking follow-ups`,
or `blocked`.

## Authorized review outputs and state transition

Only after `pass` or `pass with non-blocking follow-ups` and successful complete
validation:

- create `docs/reviews/sprint-16-http-api-health.md`;
- transition Sprint 16 to `completed` and Sprint 17 Workspace Service to the
  unique `next` target in `docs/Roadmap.md`;
- synchronize only the completion-level current-state text in `README.md` and
  `docs/Architecture.md` when needed;
- conditionally retire the verified immediately preceding suite exactly as
  defined below;
- commit all review-owned outputs atomically with the suggested message.

The verified preceding suite is
`docs/codex/prompts/sprint-15-runtime-service-container/`, with exactly these
tracked deletion targets:

- `00-sprint-15-execution-loop.md`
- `01-investigate-runtime-service-container.md`
- `02-define-runtime-service-container-contract.md`
- `03-implement-runtime-service-container.md`
- `04-integrate-runtime-application-lifecycle.md`
- `05-complete-runtime-service-container-evidence.md`
- `06-sprint-15-integration-review.md`

Re-enumerate and compare the tracked inventory before deletion. Stop if the
directory differs, contains an endangered untracked file, or a retained current
link requires it. Delete only those explicitly enumerated tracked files through
the normal file-editing mechanism. Preserve the complete Sprint 16 suite,
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
  Roadmap state, next-sprint eligibility, and final Git state.

## Suggested commit message

`Complete Sprint 16 HTTP API and health review`

## Final report additions

Report the reviewed range, findings by severity, missing evidence, acceptance
matrix, exact test counts and validation outcomes, decision, review artifact,
state transition, every retired path or `already_retired`, Sprint 17 eligibility,
commit, preserved paths, and final Git state.
