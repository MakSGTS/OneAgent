# Git Change Adapter Task Template

## Purpose

Use this template for one accepted Git repository boundary, endpoint,
change-normalization, deterministic ordering, Workspace change-input, or
production-integration slice.

## Recommended profile

- `docs/codex/profiles/git-change-adapter-implementation.md`

## Required base template

- `docs/codex/templates/task-prompt.md`

## Required task-specific sections

- Repository and Workspace authority boundary
- Repository-owned evidence and production entry point
- Baseline and current endpoint contract
- Included Git state layers and completeness
- Change identity, status vocabulary, and path confinement
- Rename, copy, delete, type-change, conflict, and untracked policy
- Ordering, duplicate, bound, and failure contract
- Workspace change-input mapping and equivalence oracle
- Process, dependency, platform, lifecycle, and consumer impact

## Additional acceptance requirements

- Keep Git evidence separate from Workspace, source-adapter, Graph, Analysis,
  diagnostic, impact, refactoring, and edit authority.
- Define the exact repository boundary, baseline/current endpoints, included
  state layers, completeness, typed failures, and concurrent-mutation behavior.
- Define normalized identity, old/new path optionality, closed statuses,
  deterministic total order, duplicates, conflicts, and exact/over bounds.
- Constrain paths before publication and prove separator, traversal, absolute,
  non-text, symlink, nested-repository, worktree, submodule, and unsupported
  cases applicable to the slice.
- Treat rename/copy similarity as policy-owned evidence and prove ambiguity,
  tie, threshold, disabled, and operation-order behavior when included.
- Define and execute a non-empty equivalence oracle between Git-derived inputs
  and accepted filesystem or complete Workspace end-state behavior.
- Prove repository construction reorder, Git output reorder, repeated reads,
  fresh runs, cancellation, cleanup, platform compatibility, and recovery.
- Audit process/dependency behavior and every affected Workspace, cache,
  lifecycle, public API, protocol, client, and sensitive-data boundary.

## Additional report sections

- Repository and authority boundary
- Endpoint and included-state contract
- Normalized change identity and ordering
- Rename/copy/delete/conflict/untracked evidence
- Path confinement, bounds, failures, and sensitive-data evidence
- Workspace mapping and equivalence matrix
- Process/dependency/platform compatibility
- Deferred impact, refactoring, edit, remote, and UI scope

## Additional validation

- Run non-zero focused repository-boundary, endpoint, status, path,
  rename/copy/delete/conflict/untracked, ordering, duplicate, bound, failure,
  cancellation, repetition, and equivalence tests applicable to the slice.
- Run the production adapter entry point against repository-owned temporary Git
  repositories or provenance-backed fixtures; parser-only tests are
  insufficient for a production claim.
- Run affected Workspace, cache, Runtime, public-process, protocol, or client
  checks when their observable behavior changes.
- Run full workspace validation for production behavior, public APIs, Cargo,
  Workspace inputs, snapshots, cache, or protocol changes as required by
  `docs/codex/core/validation.md`.
