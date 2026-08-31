# Implement Sprint 38 Git Repository Reader

Continue OneAgent development.

## Reporting

- Prompt and repository artifacts: English.
- User-visible reports: Russian.

## Profile and template

- `docs/codex/profiles/git-change-adapter-implementation.md`
- `docs/codex/templates/git-change-adapter-task.md`

## Required workflows

- `docs/codex/workflows/implementation.md`
- `docs/codex/workflows/git-change-adapter.md`

## Authoritative documents

- `docs/Roadmap.md`
- `docs/Architecture.md`
- `docs/architecture/semantic-model-2.md`
- `docs/architecture/git-change-adapter-investigation.md`
- `docs/adr/0037-runtime-service-container.md`
- `docs/adr/0041-file-watching.md`
- `docs/adr/0060-git-change-adapter.md`

## Prerequisite

Task 3 is committed, its complete validation passes, and the normalized domain
matches ADR-0060.

## Task

Implement the accepted bounded local Git repository reader boundary and map
only raw repository evidence into the Task 3 normalized change-set domain. Do
not integrate Runtime Workspace rebuild orchestration yet.

## Required behavior and evidence

- Implement only the ADR-selected library, process, or injected-reader
  boundary with exact repository root, endpoint resolution, included state
  layers, completeness, stability/concurrent-mutation, and platform behavior.
- Convert every accepted repository status and old/new path into the normalized
  domain without leaking raw output order, implementation-local object IDs,
  absolute paths, environment, config, credentials, source content, stderr, or
  internal error chains.
- Apply accepted rename/copy/delete/type-change/conflict/untracked/ignored,
  nested/worktree/submodule/bare/missing/incompatible, duplicate, ambiguity,
  bound, encoding, and confinement policies exactly. Unsupported cases must
  fail or defer as typed, not disappear silently.
- If a process is accepted, own executable resolution, arguments, working
  directory, bounded stdout/stderr, environment, exit/signal mapping,
  cancellation, timeout, cleanup, and deterministic injection. If a library is
  accepted, preserve the ADR dependency/version/features/license/unsafe and
  cancellation boundaries.
- Use repository-owned temporary Git repositories to prove accepted empty,
  added, modified, deleted, type-change, rename/copy, conflict, untracked,
  ignored, reordered-operation, endpoint, branch-independent, repeated fresh,
  malformed/incompatible, exact/over-bound, cancellation, cleanup, and
  sensitive-data cases applicable to the first slice.
- Keep tests deterministic through explicit process or repository completion;
  do not use arbitrary sleeps, ambient global Git configuration, network,
  credentials, user repositories, or source mutation outside temporary roots.
- Preserve production watcher, Workspace/cache/lifecycle, Graph/Analysis,
  protocols, IDEs, adapters, and Coverage outside the accepted reader API.

## Excluded scope

Workspace change-input mapping or service integration, complete rebuild
scheduling, cache changes, public Git protocol/UI, remote operations,
credentials, repository mutation outside disposable tests, semantic impact,
selective parsing/Graph mutation, diagnostics, rules, refactoring, edits,
current-state documentation, and Sprint completion.

## Validation

Run non-zero focused repository-boundary/endpoint/state-layer/status/path/
rename-copy-delete-conflict-untracked/ordering/bound/failure/cancellation/
cleanup/repetition tests through the accepted production reader. Run process,
dependency, license, unsafe, platform, and sensitive-data audits applicable to
the implementation, then the canonical full Rust workspace gate and
`git diff --check`.

## Suggested commit message

`Implement Sprint 38 Git repository reader`

## Final report additions

Report reader ownership and entry point, implementation family, repository and
endpoint behavior, included layers and status/path mapping, rename/conflict/
untracked behavior, bounds/errors/confinement, temporary-repository evidence,
cancellation/cleanup, dependency/platform impact, exact tests/counts, and full
validation.
