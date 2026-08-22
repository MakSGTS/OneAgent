# Review Sprint 21 CLI Client Integration

Continue OneAgent development.

## Reporting

- Prompt and repository content: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/review.md`

## Template

`docs/codex/templates/review-task.md`

## Authoritative documents

- `docs/Roadmap.md`, Sprint 21 execution plan
- `docs/architecture/cli-client-investigation.md`
- `docs/adr/0043-cli-client.md`
- `docs/reviews/sprint-20-persistent-cache.md`
- `docs/adr/0037-runtime-service-container.md`
- `docs/adr/0038-http-api-health.md`
- `docs/adr/0039-workspace-service.md`
- `docs/adr/0040-graph-query-api.md`
- `docs/adr/0041-file-watching.md`
- `docs/adr/0042-persistent-cache.md`
- exact committed Sprint 21 planning-through-Task-5 range

## Prerequisites / Required gate

Require Tasks 1-5 committed or proven `already_complete`, every implementation
validation successful, clean task-owned state, and exact commit/path inventory.
Stop with a blocked review if any prerequisite or acceptance evidence is absent.

## Review task

Independently review the integrated Sprint 21 baseline. Do not fix findings.
Issue `pass`, `pass with non-blocking follow-ups`, or `blocked`. Only after a
non-blocking decision and complete successful validation may this task create
the review artifact, transition Roadmap state, and retire the previous suite.

## Scope

### Included

- Exact planning-through-Task-5 commit and path range.
- Investigation quality, ADR-0043 completeness, command grammar, endpoint and
  request mapping, response framing/limits, JSON/output/stream behavior, local/
  server/transport/protocol failures, exits, resource ownership, dependency and
  compatibility boundaries, focused/public evidence, docs, and deferred scope.
- Complete validation, findings, missing evidence, risk assessment, Roadmap
  transition, v0.4 release-review hand-off, and conditional Sprint 20 suite
  retirement.

### Excluded

Implementation fixes, architecture reselection, new tests to repair evidence,
new commands/dependencies/routes/schemas, protocol activation, semantic or
adapter changes, v0.4 review itself, Sprint 22 work, packaging, benchmarks, and
unrelated cleanup.

## Review Criteria

- Every committed task owns exactly its planned outcome and satisfies its
  acceptance criteria without excluded scope.
- ADR-0043 is implemented exactly; every supported command maps to one existing
  accepted Runtime route without semantic or protocol authority drift.
- Parsing, validation precedence, endpoint/request encoding, response
  framing/bounds/media/body handling, outputs, errors, streams, and exit codes
  are closed, deterministic, compatible, and fully evidenced.
- Every connection and process-owned resource terminates on success and failure;
  no detached tasks, listeners, pools, globals, retries, or leaks exist.
- Public tracked EDT/Designer evidence proves the complete command/client/server
  matrix on macOS/Windows-compatible paths without arbitrary sleeps or external
  services.
- Runtime lifecycle/readiness, health and Graph Query wires, Workspace snapshot
  consistency, File Watching, Persistent Cache, graph semantics, adapters,
  Coverage, and deferred scope remain truthful and unchanged.
- Complete focused and workspace validation succeeds.

## Acceptance evidence matrix

Record one evidence/result row for planning readiness, investigation, accepted
architecture, command grammar, local validation, help/version, endpoint and
query encoding, exact requests, response framing and body bound, JSON/output
preservation, error/stream/exit behavior, connection ownership/cleanup,
dependency impact, every supported command, both production formats, Runtime
domain/unavailable/transport/protocol failures, ordering, lifecycle/shutdown,
repetition, platform behavior, docs, compatibility, and scope containment.

## Authorized review outputs and state transition

Only after issuing `pass` or `pass with non-blocking follow-ups` and completing
all required validation:

- create `docs/reviews/sprint-21-cli-client.md`;
- transition Sprint 21 to `completed` in `docs/Roadmap.md` and make the v0.4
  release integration review the unique next gate;
- synchronize only minimal current-state hand-off text in `README.md`,
  `docs/Architecture.md`, and `docs/architecture/semantic-model-2.md` if needed;
- conditionally retire the exact verified previous suite
  `docs/codex/prompts/sprint-20-persistent-cache/` in the same review commit.

The previous suite has exactly these planned tracked files:

- `00-sprint-20-execution-loop.md`
- `01-investigate-persistent-cache-boundary.md`
- `02-define-persistent-cache-contract.md`
- `03-implement-snapshot-cache-codec.md`
- `04-implement-cache-storage-invalidation.md`
- `05-integrate-runtime-cache-lifecycle.md`
- `06-complete-persistent-cache-evidence.md`
- `07-sprint-20-integration-review.md`

Before deletion, re-enumerate tracked, filesystem, and untracked inventory and
stop on mismatch or danger. Delete only exact files through explicit patches;
never use recursive deletion, globs, `git clean`, or broad staging. Verify no
retained Markdown link depends on a deleted prompt. Keep the Sprint 21 suite,
`run-next-sprint.md`, non-adjacent suites, and `.codex/` untouched.

## Task-specific Validation

- List and run exact focused command/client and public client/server targets with
  non-zero test counts.
- `cargo test -p oneagent-cli`
- Run relevant Runtime health and Graph Query targets.
- Run the canonical complete workspace validation from
  `docs/codex/core/validation.md`.
- Verify commit/path ownership, docs links, Roadmap state, prompt-retirement
  inventory, `git diff --check`, and final `git status --short`.

## Suggested commit message

`Complete Sprint 21 CLI Client review`

## Final report additions

Report reviewed range and commits, evidence matrix, findings, missing evidence,
decision, validation, review/state/doc outputs, every retired Sprint 20 path or
`already_retired` evidence, v0.4 review eligibility, deferred scope, residual
risk, commit, and final Git state.
