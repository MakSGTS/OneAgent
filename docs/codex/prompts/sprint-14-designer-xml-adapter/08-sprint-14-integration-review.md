# Review Sprint 14 Designer XML Adapter

Continue OneAgent development.

## Reporting

- Repository content and commit message: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/review.md`

## Template

`docs/codex/templates/review-task.md`

## Authoritative documents

- `docs/Roadmap.md`, Sprint 14 execution plan and live task records
- `docs/adr/0036-designer-xml-adapter.md`
- `docs/architecture/designer-xml-source-investigation.md`
- `docs/architecture/designer-xml-source-corpus.md`
- `docs/architecture/semantic-model-2.md`
- `docs/reviews/sprint-13-xdto-service-model.md`
- the committed Sprint 14 prompt suite

## Prerequisites / Required gate

Require Tasks 1-7 committed or proven `already_complete`, every implementation
full workspace gate successful, no task-created uncommitted change, and an exact
planning-through-Task-7 commit range. Stop before outputs otherwise.

## Review target

Review the entire Sprint 14 range for discovery, completeness, artifact
assembly, parsing, canonical mapping, identity, provenance, production entry
point, cross-adapter projection, controlled-change oracle, typed failures,
determinism, public consumers, complete/incremental indexes, Coverage,
documentation, EDT compatibility, and scope containment.

## Scope

### Included

- Commit/path audit and acceptance evidence matrix for every ADR-0036 criterion.
- Exact focused and full validation rerun.
- One explicit `pass`, `pass with non-blocking follow-ups`, or `blocked` decision.
- For a non-blocking decision only: create
  `docs/reviews/sprint-14-designer-xml-adapter.md`, transition Sprint 14 to
  `completed`, make the v0.3 release review eligible, and record Sprint 15 as
  the next planning target after that release gate.
- Conditional retirement of the exact Sprint 13 suite as the final bounded
  action, atomically included in the review commit.

### Excluded

- Silent implementation fixes, architecture reselection, new tests or behavior,
  broader Designer support, release-review execution, Sprint 15 planning, and
  deletion outside the exact preceding suite.

## Review Criteria

- Every included source field/path/failure is backed by investigation evidence
  and matches ADR-0036.
- Complete/partial input, project boundaries, ordering, and failure scopes are
  deterministic and preserve valid siblings only where accepted.
- Canonical identities/content are source independent; provenance remains exact.
- The paired oracle is non-empty, excludes only deliberate differences, and
  fails for its controlled semantic change.
- Query, Diff, report, Validation, Coverage, complete index, and incremental
  index evidence is complete and repeatable.
- EDT behavior, public API compatibility, deferred scope, repository safety,
  prompt inventory, and documentation remain correct.

## Previous-suite retirement procedure

The verified preceding directory is
`docs/codex/prompts/sprint-13-xdto-service-model/` with exactly these tracked
files:

- `00-sprint-13-execution-loop.md`
- `01-implement-xdto-service-graph-model.md`
- `02-parse-xdto-packages.md`
- `03-parse-http-web-services.md`
- `04-emit-xdto-service-semantics.md`
- `05-complete-sprint-13-production-evidence.md`
- `06-sprint-13-integration-review.md`

Only after a non-blocking decision and all required validation succeeds,
re-enumerate and compare tracked inventory, verify no endangered untracked file
or retained link dependency, delete only those exact files using explicit safe
edits, and stage each deletion explicitly. Include the review artifact, Roadmap
transition, Semantic Model review synchronization if required, and all seven
deletions in the single final review commit. Any mismatch blocks retirement and
the commit. Preserve this Sprint 14 suite and `run-next-sprint.md`.

## Task-specific Validation

- Run every focused command named by Tasks 3-7 against exact non-zero targets.
- Run applicable package tests for workspace, filesystem, Designer adapter, EDT,
  BSL, metadata, graph, Coverage, conformance, and indexes.
- Run the complete workspace validation gate.
- Validate review links, Roadmap state, exact retirement inventory, retained
  Sprint 14 suite, and `git diff --check` after authorized deletions.

## Suggested commit message

`Complete Sprint 14 Designer XML adapter review`

## Final report additions

Report reviewed range and commits, evidence matrix, findings by severity,
missing evidence, decision, validation, review artifact/state transition, every
retired path or blocker, v0.3 release-review and Sprint 15 eligibility, commit,
and final Git state.
