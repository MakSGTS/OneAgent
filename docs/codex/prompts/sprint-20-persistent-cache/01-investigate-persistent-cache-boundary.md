# Investigate Sprint 20 Persistent Cache Boundary

Continue OneAgent development.

## Reporting

- Prompt and repository content: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/investigation.md`

## Template

`docs/codex/templates/investigation-task.md`

## Authoritative documents

- `docs/Roadmap.md`, Sprint 20 execution plan
- `docs/Architecture.md`
- `docs/architecture/semantic-model-2.md`
- `docs/reviews/sprint-19-file-watching.md`
- `docs/adr/0039-workspace-service.md`
- `docs/adr/0040-graph-query-api.md`
- `docs/adr/0041-file-watching.md`

## Prerequisites / Required gate

Require the committed Sprint 20 planning baseline containing this complete
prompt suite and matching Roadmap manifest. Require Sprint 20 to be the unique
eligible target and preserve a clean task-owned state.

## Investigation objective

Create `docs/architecture/persistent-cache-investigation.md` with verified
evidence for the smallest complete, testable Persistent Cache slice and the
exact questions ADR-0042 must decide. Do not select architecture, add a
dependency, or modify production behavior.

## Questions to answer

- Which current types own every `WorkspaceSnapshot` field, graph node/payload/
  edge/provenance value, diagnostic, reference request/statistic, report,
  validation invariant, configuration identity/order, and clean-build result?
- Which public checked constructors, getters, iterators, insertion APIs, and
  validators can reconstruct complete source-neutral state without serializing
  incidental private indexes or creating a second semantic authority?
- Which exact source-state inputs, ignored paths, semantic-builder/schema
  compatibility inputs, Workspace identity inputs, and race boundaries can
  determine cache validity without timestamps, platform paths, or process state?
- Which format/encoding options are already available through approved Runtime
  dependencies, and would any viable complete codec require explicit approval
  for a new production dependency or changes to lower-level crate APIs?
- Where can cache configuration, load, write, replacement, and cleanup be owned
  without watcher feedback loops, hidden global state, cross-platform rename
  assumptions, path escape, partial publication, or unowned blocking I/O?
- How must missing, incompatible, newer, older, malformed, truncated, duplicate,
  semantically invalid, unreadable, write-failed, and interrupted entries be
  classified and recovered through a clean build?
- How can Runtime startup and post-watch rebuilds close scan/load/build/write
  races, preserve last-valid snapshot behavior, and keep health, Graph Query,
  update status, cancellation, shutdown, and fresh repetition compatible?
- Which tracked fixtures and deterministic temporary/failure seams prove cold
  miss/write, warm hit without adapter rebuilding, invalidation, corruption,
  compatibility, clean-build recovery, exact snapshot/query equivalence, watcher
  replacement, cleanup, and supported-platform behavior?

## Evidence scope

- `apps/runtime/` configuration, Workspace snapshots/builders/service/change
  source, Graph Query, production composition, manifests, tests, and fixtures.
- `crates/common/`, `crates/metadata/`, and `crates/graph/` identity, payload,
  graph construction, diagnostics, reference ledger, reports, validation, Diff,
  and public reconstruction surfaces.
- `adapters/filesystem/`, `adapters/edt/`, and `adapters/designer-xml/` production
  discovery/build inputs and errors relevant to validity and clean rebuilds.
- Cargo manifests/lockfile/tree, CI platforms, consumers, Git history, Roadmap,
  reviews, ADRs, current-state documents, and existing filesystem patterns.

## Evidence sources / fixtures

At minimum inspect:

- `apps/runtime/src/workspace/mod.rs`
- `apps/runtime/src/workspace/change.rs`
- `apps/runtime/src/workspace/graph_query.rs`
- `apps/runtime/src/config/mod.rs`
- `apps/runtime/src/main.rs`
- `apps/runtime/tests/workspace_service.rs`
- `apps/runtime/tests/file_watching.rs`
- `apps/runtime/tests/graph_query_api.rs`
- `apps/runtime/tests/fixtures/workspace_service/`
- relevant public types under `crates/common/src/`, `crates/metadata/src/`, and
  `crates/graph/src/`

Record exact provenance for every proposed public integration oracle. Do not
make ignored local corpora, network services, arbitrary sleeps, host-global
cache state, or unapproved dependencies prerequisites.

## Excluded

ADR acceptance, production Rust changes, Cargo changes, public API changes,
fixture changes, graph/parser/adapter semantic changes, codec/storage/runtime
implementation, supported CLI behavior, prompt retirement, Roadmap transition,
performance/security claims, dependency additions, and external research.

## Completion Criteria

- The investigation separates confirmed repository evidence, accepted
  constraints, compatibility-sensitive behavior, unsupported cases, unknowns,
  and decision questions.
- It inventories complete persisted content candidates, checked reconstruction
  and validation surfaces, consumers, dependencies, platforms, source validity
  inputs, filesystem constraints, fixtures, and non-zero test oracles.
- Every accepted candidate has observable valid-hit, miss, invalidation,
  incompatible/corrupt/partial, write failure, recovery, lifecycle, watcher,
  cancellation, cleanup, equivalence, and repeated-run evidence where
  applicable.
- It defines the minimum ADR matrix for authority, schema/payload, versions,
  encoding, ordering, identity, invalidation, location, replacement,
  compatibility, corruption, migration/recovery, lifecycle, observability,
  dependencies, testing, and deferred scope.
- It states explicitly whether implementation can remain on approved existing
  dependencies or which exact new production dependency would require approval.
- Missing or conflicting evidence blocks Task 2 instead of being replaced with
  invented formats, identities, filesystem guarantees, migration, or recovery.
- No production, manifest, fixture, Roadmap-state, current-state, or prompt-suite
  file is changed.

## Repository Safety

Create only `docs/architecture/persistent-cache-investigation.md`. Preserve
`.codex/`, production code, manifests, fixtures, prompt suites, Roadmap state,
and unrelated files. Stage only the investigation document when commit mode is
authorized.

## Task-specific Validation

- Verify every cited path, type, API, fixture, dependency, platform, test, and
  consumer from the live repository.
- Run non-mutating focused `--list` or existing tests only when needed to prove
  an oracle; report zero matches separately.
- Validate document links and `git diff --check`.
- `git status --short`

## Suggested commit message

`Investigate Sprint 20 Persistent Cache`

## Final report additions

Report confirmed persisted-state and ownership boundaries, reconstruction and
validation constraints, validity inputs, filesystem/dependency findings,
fixture/test oracles, unresolved ADR questions, decision readiness, changed
path, validation, commit, and final Git state.
