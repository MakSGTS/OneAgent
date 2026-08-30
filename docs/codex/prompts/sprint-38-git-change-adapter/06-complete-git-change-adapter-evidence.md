# Complete Sprint 38 Git Change Adapter Evidence

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
- `docs/adr/0039-workspace-service.md`
- `docs/adr/0041-file-watching.md`
- `docs/adr/0042-persistent-cache.md`
- `docs/adr/0060-git-change-adapter.md`
- all committed Task 3–5 code, tests, fixtures, and validation evidence

## Prerequisite

Task 5 is committed, its full gate passes, and all accepted domain, repository
reader, Workspace mapping, rebuild, cache, lifecycle, and compatibility
behavior is implemented.

## Task

Complete repository-owned Git Change Adapter evidence, compatibility and scope
audits, and current-state documentation. Introduce no new production behavior.

## Required evidence

- Create `docs/architecture/git-change-adapter-evidence.md` with a complete
  requirement-to-test matrix for every ADR-0060 authority, repository,
  endpoint, state-layer, status, path, identity, order, duplicate, conflict,
  rename/copy, bound, failure, sensitive-data, reader/process-or-library,
  Workspace mapping, rebuild, cache, lifecycle, platform, and consumer rule.
- Add only missing repository-owned fixtures or harness evidence required to
  cover empty, positive, deletion, type change, rename/copy, conflict,
  untracked/ignored, incompatible, path escape, exact/over bound, reordered,
  repeated, concurrent mutation, cancellation, cleanup, invalid build/recovery,
  Workspace equivalence, cache, and public-consumer cases. Do not change
  accepted behavior.
- Run and record complete domain/reader/Workspace/cache/watching/Runtime/
  Graph/Analysis/protocol/client/public-process matrices with exact non-zero
  counts and zero failures.
- Reassert production adapter and Graph authority, ADR-0027 canonical diff,
  complete rebuild and validation, immutable publication, cache recovery,
  diagnostic/rule behavior, protocol catalogs/capabilities/policy,
  HTTP/CLI/VS Code/EDT compatibility, and unchanged Coverage state.
- Synchronize only README, Architecture, Semantic Model, and Roadmap current-
  state text required to describe the verified Git Change Adapter boundary,
  limitations, and Sprint 39 hand-off.
- Audit public APIs, dependencies/features/licenses/unsafe surface, executable
  assumptions, supported platforms, schemas/capabilities, configuration and
  credentials, absolute/personal paths, raw output/source content, generated
  artifacts, repository status, documentation links, scope, and absence of
  remote Git, repository mutation, semantic impact, refactoring, edits,
  telemetry, and unsupported performance/security claims.

## Excluded scope

New production behavior, new Git state layers beyond ADR-0060, remote access,
repository mutation, new protocol/IDE capabilities, semantic impact,
diagnostics/rules from Git, selective Graph mutation, refactoring, safe edits,
release review, Sprint 39 implementation, and prompt-suite retirement.

## Validation

Run the complete focused and public-process matrix required by ADR-0060, all
platform/dependency/API/sensitive-data/scope/documentation audits, then:

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets --all-features -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
git diff --check
```

Record exact commands, counts, exits, skips, limitations, and artifact paths. A
zero-match filter, skipped required row, authority/schema/capability mismatch,
failed audit, or incomplete cross-platform row blocks completion.

## Suggested commit message

`Complete Sprint 38 Git Change Adapter evidence`

## Final report additions

Report the acceptance matrix, exact test counts and commands, current-state
docs, domain/reader/Workspace/cache/lifecycle evidence, API/dependency/
platform/configuration/sensitive-data/scope audits, preserved behavior,
limitations, and canonical gate results.
