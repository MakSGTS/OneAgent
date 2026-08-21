# Review Sprint 15 Runtime Service Container

Continue OneAgent development.

## Reporting

- Repository content and commit message: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/review.md`

## Template

`docs/codex/templates/review-task.md`

## Authoritative documents

- `docs/Roadmap.md`, Sprint 15 execution plan and live task records
- `docs/adr/0037-runtime-service-container.md`
- `docs/adr/0002-runtime-composition-root.md`
- `docs/architecture/runtime-service-container-investigation.md`
- `docs/architecture/semantic-model-2.md`
- `docs/Architecture.md`
- `docs/reviews/v0.3-release-review.md`
- the committed Sprint 15 prompt suite

## Prerequisites / Required gate

Require Tasks 1-5 committed or proven `already_complete`, every implementation
full workspace gate successful, no task-created uncommitted change, and an exact
planning-through-Task-5 commit range. Stop before outputs otherwise.

## Review target

Review the entire Sprint 15 range for composition ownership, service identity
and registration, lifecycle, startup order and rollback, task/resource
ownership, cancellation, requested shutdown, service and join failures,
terminal state, public Runtime boundary, App/main integration, deterministic
tests, cross-platform behavior, documentation, compatibility, and scope
containment.

## Scope

### Included

- Commit/path audit and acceptance evidence matrix for every ADR-0037 criterion.
- Exact focused and full validation rerun.
- One explicit `pass`, `pass with non-blocking follow-ups`, or `blocked` decision.
- For a non-blocking decision only: create
  `docs/reviews/sprint-15-runtime-service-container.md`, transition Sprint 15 to
  `completed`, and make Sprint 16 HTTP API and Health the unique `next` target.
- Conditional retirement of the exact Sprint 14 suite as the final bounded
  action, atomically included in the review commit.

### Excluded

- Silent implementation fixes, architecture reselection, new tests or behavior,
  HTTP routes/health schema, workspace or graph services, file watching,
  persistence, CLI work, Sprint 16 planning, and deletion outside the exact
  preceding suite.

## Review Criteria

- Every service, task, channel, cancellation source, and lifecycle transition
  has the exact ADR-0037 owner and terminal handling.
- Startup rollback, service failure, requested shutdown, join failure, cleanup,
  and repeated fresh runs are deterministic and independently tested.
- The App is genuinely long-running until shutdown or failure, while `main.rs`
  remains a composition boundary.
- Internal lifecycle evidence does not claim the Sprint 16 public health API.
- Tests use deterministic synchronization, match non-zero targets, and remain
  compatible with repository macOS and Windows CI.
- Existing configuration, v0.3 semantic behavior, unrelated packages, deferred
  scope, repository safety, prompt inventory, and documentation remain correct.

## Previous-suite retirement procedure

The verified preceding directory is
`docs/codex/prompts/sprint-14-designer-xml-adapter/` with exactly these tracked
files:

- `00-sprint-14-execution-loop.md`
- `01-investigate-designer-xml-source-contracts.md`
- `02-define-designer-xml-adapter-contract.md`
- `03-implement-designer-xml-discovery.md`
- `04-parse-designer-xml-metadata.md`
- `05-parse-designer-xml-modules.md`
- `06-emit-designer-xml-semantics.md`
- `07-complete-sprint-14-conformance-evidence.md`
- `08-sprint-14-integration-review.md`

Only after a non-blocking decision and all required validation succeeds,
re-enumerate and compare tracked inventory, verify no endangered untracked file
or retained link dependency, delete only those exact files using explicit safe
edits, and stage each deletion explicitly. Include the review artifact, Roadmap
transition, architecture current-state synchronization if required, and all
nine deletions in the single final review commit. Any mismatch blocks retirement
and the commit. Preserve this Sprint 15 suite and `run-next-sprint.md`.

## Task-specific Validation

- Run every focused command named by Tasks 3-5 against exact non-zero targets.
- `cargo test -p oneagent-runtime --no-fail-fast`
- Run the complete workspace validation gate.
- Validate review links, Roadmap state, exact retirement inventory, retained
  Sprint 15 suite, and `git diff --check` after authorized deletions.

## Suggested commit message

`Complete Sprint 15 Runtime service container review`

## Final report additions

Report reviewed range and commits, evidence matrix, findings by severity,
missing evidence, decision, validation, review artifact/state transition, every
retired path or blocker, Sprint 16 eligibility, commit, and final Git state.
