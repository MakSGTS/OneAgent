# Investigate Sprint 19 File Watching Boundary

Continue OneAgent development.

## Reporting

- Prompt and repository content: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/investigation.md`

## Template

`docs/codex/templates/investigation-task.md`

## Authoritative documents

- `docs/Roadmap.md`, Sprint 19 execution plan
- `docs/Architecture.md`
- `docs/architecture/semantic-model-2.md`
- `docs/reviews/sprint-18-graph-query-api.md`
- `docs/adr/0002-runtime-composition-root.md`
- `docs/adr/0037-runtime-service-container.md`
- `docs/adr/0038-http-api-health.md`
- `docs/adr/0039-workspace-service.md`
- `docs/adr/0040-graph-query-api.md`

## Prerequisites / Required gate

Require the committed Sprint 19 planning baseline containing this complete
prompt suite and matching Roadmap manifest. Require Sprint 19 to be the unique
eligible target and preserve a clean task-owned state.

## Investigation objective

Create `docs/architecture/file-watching-investigation.md` with verified
evidence for the smallest testable File Watching and Workspace rebuild slice
and the exact questions ADR-0041 must decide. Do not select architecture, add a
dependency, or modify production behavior.

## Questions to answer

- Which current types own configured Workspace roots, production discovery,
  EDT/Designer XML complete builds, validation, immutable publication,
  observation, Graph Query selection, lifecycle, service ordering,
  cancellation, shutdown, and failures?
- Which tracked files and directories are semantically relevant to each
  accepted source format, and which editor, VCS, build-output, temporary,
  unknown, metadata-only, directory, symlink, or outside-root changes are
  confirmed relevant, irrelevant, or unresolved?
- Which add, modify, remove, rename, directory, burst, duplicate, reordered,
  overflow, watch-root loss, permission, invalid-build, and recovery cases have
  repository-owned positive or negative test oracles?
- Which watcher technologies are already locked or available through the
  standard library and Tokio, which cross-platform behavior do they prove, and
  would any viable first slice require explicit approval for a new production
  dependency?
- Where can one owner normalize raw observations and apply bounded coalescing
  without relying on arbitrary sleeps, losing cancellation, spawning detached
  tasks, or making platform-specific events a public API?
- How can complete rebuilds be serialized and atomically published while an
  older immutable snapshot remains observable, and which failure-retention,
  clearing, retry, and recovery decisions remain open?
- How must initial startup, `Running`, `Stopping`, cancellation, service
  failure, graph-query requests, health readiness, listener compatibility, and
  snapshot sender closure constrain the first slice?
- Which public fixtures and deterministic temporary mutations prove production
  EDT and Designer XML changes, irrelevant changes, atomic visibility, failure
  and recovery, shutdown cleanup, and repeated fresh runs on supported CI
  platforms?

## Evidence scope

- `apps/runtime/` configuration, service container, Workspace snapshot builder
  and observer, Graph Query service and HTTP API, production composition,
  manifests, tests, and tracked fixtures.
- `adapters/filesystem/`, `adapters/edt/`, and `adapters/designer-xml/`
  discovery/read boundaries and error behavior relevant to watched inputs.
- `Cargo.toml`, affected crate manifests, `Cargo.lock`, current dependency graph,
  CI platforms, consumers, Git history, Roadmap, reviews, ADRs, and current-state
  documents.

## Evidence sources / fixtures

At minimum inspect:

- `apps/runtime/src/workspace/mod.rs`
- `apps/runtime/src/workspace/graph_query.rs`
- `apps/runtime/src/service/`
- `apps/runtime/src/main.rs`
- `apps/runtime/tests/workspace_service.rs`
- `apps/runtime/tests/graph_query_api.rs`
- `apps/runtime/tests/fixtures/workspace_service/`
- `adapters/filesystem/src/lib.rs`
- relevant production readers under `adapters/edt/src/` and
  `adapters/designer-xml/src/`

Record exact provenance for every proposed public integration oracle. Do not
make ignored local corpora, network filesystems, external services, arbitrary
sleep timing, or unapproved dependencies prerequisites.

## Excluded

ADR acceptance, production Rust changes, Cargo changes, public API changes,
fixture changes, graph or parser semantics, watcher implementation, rebuild
orchestration, persistence, supported CLI behavior, prompt retirement, Roadmap
transition, performance claims, dependency additions, and external research.

## Completion Criteria

- The investigation separates confirmed repository evidence, accepted
  constraints, compatibility-sensitive behavior, unsupported cases, unknowns,
  and decision questions.
- It inventories exact public types, ownership, consumers, dependency choices,
  platform constraints, relevant input families, fixtures, and non-zero test
  oracles for the first slice.
- Every candidate capability has an observable positive, negative, burst,
  invalid, recovery, lifecycle, cancellation, cleanup, and repeated-run oracle
  where applicable.
- It defines the minimum ADR matrix for ownership, watched boundary, normalized
  changes, relevance, coalescing, scheduling, rebuild serialization,
  publication, failure/recovery, lifecycle, shutdown, observability,
  compatibility, dependencies, testing, and deferred scope.
- It states explicitly whether implementation can remain on approved existing
  dependencies or which exact new production dependency would require approval.
- Missing or conflicting evidence blocks Task 2 instead of being replaced with
  invented event semantics, timing, failure behavior, or APIs.
- No production, manifest, fixture, Roadmap-state, current-state, or
  prompt-suite file is changed.

## Repository Safety

Create only `docs/architecture/file-watching-investigation.md`. Preserve
`.codex/`, production code, manifests, fixtures, current prompt suites, Roadmap
state, and unrelated files. Stage only the investigation document when commit
mode is authorized.

## Task-specific Validation

- Verify every cited path, type, API, fixture, dependency, platform, test, and
  consumer from the live repository.
- Run non-mutating focused `--list` or existing tests only when needed to prove
  an oracle; report zero matches separately.
- Validate document links and `git diff --check`.
- `git status --short`

## Suggested commit message

`Investigate Sprint 19 File Watching`

## Final report additions

Report confirmed ownership and input boundaries, compatibility constraints,
dependency/approval findings, platform and fixture/test oracles, unresolved ADR
questions, decision readiness, changed path, validation, commit, and final Git
state.
