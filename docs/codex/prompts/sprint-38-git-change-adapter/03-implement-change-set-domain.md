# Implement Sprint 38 Change-Set Domain

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
- `docs/adr/0027-incremental-semantic-index-maintenance.md`
- `docs/adr/0041-file-watching.md`
- `docs/adr/0060-git-change-adapter.md`

## Prerequisite

Task 2 is committed and ADR-0060 is accepted with no blocking domain,
identity, path, dependency, or migration question.

## Task

Implement only the ADR-0060 source-independent typed normalized repository
change-set domain. Add no Git repository I/O or Workspace orchestration yet.

## Required behavior and evidence

- Place every public or crate-visible type and invariant in the exact
  ADR-selected owner while preserving Workspace, adapter, Graph, Analysis,
  diagnostics, and edit authority.
- Implement accepted repository and endpoint identities, normalized path
  representation and confinement, change identity, old/new path optionality,
  closed status vocabulary, deterministic equality and total order, bounds,
  and closed redacted errors.
- Enforce accepted addition, modification, deletion, type-change,
  rename/copy-candidate, conflict/unmerged, untracked, ignored, duplicate, and
  incompatible behavior only as represented by the ADR. Do not infer semantic
  identity from path or status.
- Normalize equivalent input independently from insertion, iterator, hash,
  filesystem, Git output, or platform order. Reject contradictions and
  out-of-bound or escaping inputs before returning a complete value.
- Add focused tests for empty/single/mixed sets, every accepted status and
  endpoint form, path separators/traversal/absolute/non-text cases, old/new
  path rules, duplicates/conflicts, reordered construction, exact/over count
  and component bounds, stable order, error redaction, and repeated creation.
- Update only exports and Rustdoc required by the domain. Preserve existing
  Git governance behavior, production filesystem watcher, Workspace/cache,
  Graph/Analysis, Runtime, protocols, adapters, and Coverage.

## Excluded scope

Git executable or library calls, repository discovery or reading, environment
or process handling, Workspace mapping, rebuilds, cache/lifecycle changes,
semantic impact, diagnostics, rules, protocols, IDE UI, repository mutation,
new production dependency, current-state documentation, and Sprint completion.

## Validation

Run non-zero focused endpoint/change/status/path/order/duplicate/conflict/
bound/redaction/reorder/repetition tests and affected package/API/Rustdoc
checks, then:

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets --all-features -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
git diff --check
```

## Suggested commit message

`Implement Sprint 38 change-set domain`

## Final report additions

Report domain owner and API, repository/endpoint/change/status/path identity,
ordering, bounds, failures and confinement, exact focused tests/counts,
dependency impact, full-gate results, and deferred repository I/O/Workspace
integration.
