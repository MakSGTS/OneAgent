# Complete Sprint 39 Change Impact Evidence

Continue OneAgent development.

## Reporting

- Prompt and repository artifacts: English.
- User-visible reports: Russian.

## Profile and template

- `docs/codex/profiles/diagnostics-engine-implementation.md`
- `docs/codex/templates/diagnostics-engine-task.md`

## Required workflows

- `docs/codex/workflows/implementation.md`
- `docs/codex/workflows/diagnostics-engine.md`

## Authoritative documents

- `docs/Roadmap.md`
- `docs/Architecture.md`
- `docs/architecture/semantic-model-2.md`
- `docs/architecture/change-impact-analysis-investigation.md`
- `docs/architecture/git-change-adapter-evidence.md`
- `docs/architecture/mcp-semantic-tools-investigation.md`
- `docs/adr/0017-depends-on-semantics.md`
- `docs/adr/0027-incremental-semantic-index-maintenance.md`
- `docs/adr/0039-workspace-service.md`
- `docs/adr/0041-file-watching.md`
- `docs/adr/0042-persistent-cache.md`
- `docs/adr/0049-tool-execution-policy.md`
- `docs/adr/0051-mcp-semantic-tools.md`
- `docs/adr/0060-git-change-adapter.md`
- `docs/adr/0061-change-impact-analysis.md`
- all committed Task 3–5 code, tests, fixtures, and validation evidence

## Prerequisite

Task 5 is committed, its full gate passes, and all accepted report, Workspace,
cache/lifecycle, filesystem/Git equivalence, MCP, Tool Policy, and public-process
behavior is implemented.

## Task

Complete repository-owned Change Impact Analysis evidence, compatibility and
scope audits, and current-state documentation. Introduce no new production
behavior.

## Required evidence

- Create `docs/architecture/change-impact-analysis-evidence.md` with a complete
  requirement-to-test matrix for every ADR-0061 authority, input,
  previous/current publication and Configuration identity, report identity,
  status, availability, reason, order, duplicate/conflict, completeness,
  summary, bound, failure, sensitive-data, Workspace publication, cache,
  lifecycle, filesystem/Git equivalence, MCP, policy, process, client, and
  compatibility rule.
- Add only missing repository-owned fixture or harness evidence required for
  empty/equal, positive, direct/transitive/removed, Configuration addition/
  removal/identity transition, reordered, repeated, duplicate/conflict,
  exact/over bound, inconsistent input, failure/recovery, warm/cold cache,
  filesystem/Git equivalence, cancellation/cleanup, public-process, and
  sensitive-data cases. Do not change accepted behavior.
- Run and record complete Graph/Analysis/Workspace/cache/watching/Git-input/
  Runtime/protocol/Tool Policy/MCP/client/public-process matrices with exact
  non-zero counts and zero failures.
- Reassert Graph authority for facts/diff/impact, complete source-adapter builds
  and validation, immutable Workspace publication, cache recovery,
  diagnostics/rules compatibility, protocol catalog/capability/policy truth,
  HTTP/CLI/LSP/VS Code/EDT compatibility, source confinement, and truthful
  Coverage state.
- Prove repository paths, statuses, baselines, completeness, and operation order
  do not enter impact identity, Configuration matching, seeds, reasons,
  summaries, snapshots, cache, protocols, or errors. Record filesystem/Git
  complete-end-state equivalence explicitly.
- Synchronize only README, Architecture, Semantic Model, and Roadmap current-
  state text required to describe the verified Change Impact Analysis boundary,
  limitations, and Sprint 40 hand-off.
- Audit public APIs, dependencies/features/licenses/unsafe surface, cache
  schema/compatibility, protocol schemas/capabilities, configuration and
  credentials, absolute/personal paths, raw source/repository content,
  generated artifacts, Git status, documentation links, scope, and absence of
  selective semantic mutation, new graph authority, risk scoring,
  refactoring/edit behavior, remote Git, telemetry, and unsupported
  performance/security claims.

## Excluded scope

New production behavior, new Graph impact policy, new source or repository
input, new protocol tool or IDE UI, diagnostics/rules, scoring/risk prediction,
selective semantic mutation, refactoring, code actions, source edits,
transactions, rollback, release review, Sprint 40 implementation, and
prompt-suite retirement.

## Validation

Run the complete focused and public-process matrix required by ADR-0061, all
dependency/API/cache/schema/sensitive-data/scope/documentation audits, then:

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
failed audit, or incomplete platform/client row blocks completion.

## Suggested commit message

`Complete Sprint 39 Change Impact evidence`

## Final report additions

Report the acceptance matrix, exact test counts and commands, current-state
docs, report/Workspace/cache/lifecycle/equivalence/MCP/policy evidence,
API/dependency/schema/client/sensitive-data/scope audits, preserved behavior,
limitations, and canonical gate results.
