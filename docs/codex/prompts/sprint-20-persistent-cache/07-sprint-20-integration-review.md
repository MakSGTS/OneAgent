# Review Sprint 20 Persistent Cache Integration

Continue OneAgent development.

## Reporting

- Prompt and repository content: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/review.md`

## Template

`docs/codex/templates/review-task.md`

## Authoritative documents

- `docs/Roadmap.md`, Sprint 20 execution plan
- `docs/architecture/persistent-cache-investigation.md`
- `docs/Architecture.md`
- `docs/architecture/semantic-model-2.md`
- `docs/reviews/sprint-19-file-watching.md`
- `docs/adr/0037-runtime-service-container.md`
- `docs/adr/0038-http-api-health.md`
- `docs/adr/0039-workspace-service.md`
- `docs/adr/0040-graph-query-api.md`
- `docs/adr/0041-file-watching.md`
- `docs/adr/0042-persistent-cache.md`

## Prerequisites / Required gate

Require committed or proven `already_complete` Tasks 1-6, successful required
focused and complete validation, a clean task-owned tree, and an exact planning-
through-Task-6 range. Stop before review output if any prerequisite or acceptance
evidence is absent.

## Review target

Review the integrated Sprint 20 Persistent Cache baseline against the committed
plan, investigation, ADR-0042, accepted Runtime/Workspace/File Watching/Graph
Query compatibility, task exclusions, public production evidence, and complete
validation. Do not fix implementation findings in this task.

## Reviewed baseline / commit or diff range

- Planning parent: resolve from the live repository.
- Reviewed range: planning commit through committed Task 6 head.
- Record every commit hash, exact subject, owned paths, and validation outcome.

## Scope

- Canonical authority, persisted envelope/payload, schema/build versions,
  deterministic encoding, checked reconstruction, and complete validation.
- Cache identity/invalidation, path containment, load/write/replacement,
  compatibility, corruption containment, clean-rebuild recovery, and cleanup.
- Runtime cold/warm startup, File Watching replacement, publication, failure
  behavior, lifecycle/health, Graph Query compatibility, cancellation, shutdown,
  and repeated fresh ownership.
- Dependency approval, public EDT/Designer evidence, platform coverage,
  deterministic tests, fixture provenance, docs, and deferred-scope conformance.

## Excluded

Implementation fixes, new tests to repair missing evidence, architecture
reselection, new dependencies, graph/parser/adapter semantic changes,
incremental persistence, cache management APIs, supported CLI, cross-process or
remote cache, compression/encryption/eviction, later integrations, benchmarks,
and unrelated cleanup.

## Review Criteria

- Every committed task owns exactly its planned outcome and satisfies every
  acceptance criterion without pulling excluded scope forward.
- ADR-0042 is implemented exactly; canonical semantic authority is preserved and
  every accepted state is reconstructed and validated before use.
- Encoding is complete and deterministic as promised; current production graph
  variants and build evidence round-trip and remain clean-build equivalent.
- Exact validity inputs prevent stale hits; missing/incompatible/corrupt/partial/
  invalid/unreadable state and failed writes follow typed containment/recovery.
- Storage paths remain contained, complete replacement cannot expose a partial
  current entry, temporary state is cleaned, and cache-owned writes do not cause
  watcher feedback loops.
- Cold/warm startup and post-change writes preserve complete immutable
  publication, last-valid failure behavior, watcher coalescing, lifecycle-derived
  readiness, Graph Query single-snapshot/wire compatibility, shutdown, and
  resource ownership.
- Public tracked EDT/Designer evidence covers the complete required matrix on
  macOS/Windows-compatible paths without arbitrary sleeps or host-global state.
- Docs, Coverage state, compatibility, and every deferred capability remain
  truthful; complete focused and workspace validation succeeds.

## Acceptance evidence matrix

Record one evidence/result row for planning readiness, investigation, accepted
architecture, authority/schema, variant completeness, checked reconstruction,
validation, deterministic bytes, identity/invalidation, path containment,
replacement, compatibility/corruption, recovery, Runtime cold/warm behavior,
File Watching integration, publication atomicity, query/health compatibility,
failure handling, cancellation/cleanup, public EDT/Designer evidence, platform
behavior, dependency approval, fixture provenance, repetition, docs, and scope.

## Authorized review outputs and state transition

Only after issuing `pass` or `pass with non-blocking follow-ups` and completing
all required validation:

- create `docs/reviews/sprint-20-persistent-cache.md`;
- transition Sprint 20 to `completed` in `docs/Roadmap.md` and make Sprint 21 CLI
  Client the unique `next` target;
- synchronize only the minimal current-state hand-off text in `README.md`,
  `docs/Architecture.md`, and `docs/architecture/semantic-model-2.md` when needed;
- conditionally retire the exact verified previous suite
  `docs/codex/prompts/sprint-19-file-watching/` in the same review commit.

The previous suite has exactly these planned tracked files:

- `00-sprint-19-execution-loop.md`
- `01-investigate-file-watching-boundary.md`
- `02-define-file-watching-contract.md`
- `03-implement-file-change-watching.md`
- `04-integrate-workspace-rebuilds.md`
- `05-complete-file-watching-evidence.md`
- `06-sprint-19-integration-review.md`

Before deletion, re-enumerate tracked, filesystem, and untracked inventory and
stop on mismatch or danger. Delete only those exact files through explicit
patches; never use recursive deletion, globs, `git clean`, or broad staging.
Verify no retained Markdown link depends on an individual deleted prompt. Keep
the complete Sprint 20 suite, `run-next-sprint.md`, non-adjacent suites, and
`.codex/` untouched.

## Task-specific Validation

- List and run the exact focused codec, store/invalidation, Runtime integration,
  public Persistent Cache, Workspace/File Watching, Graph Query, and health
  matrix with non-zero test counts.
- Run `cargo test -p oneagent-runtime`.
- Run the canonical complete workspace validation from
  `docs/codex/core/validation.md`.
- Verify commit/path ownership, docs links, Roadmap state, prompt retirement
  inventory, `git diff --check`, and final `git status --short`.

## Suggested commit message

`Complete Sprint 20 Persistent Cache review`

## Final report additions

Report reviewed range and commits, evidence matrix, findings, missing evidence,
decision, validation, review/state/current-doc outputs, every retired Sprint 19
path or `already_retired` evidence, Sprint 21 eligibility, deferred scope,
residual risk, commit, and final Git state.
