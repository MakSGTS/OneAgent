# Complete Sprint 36 Diagnostics Engine Evidence

Continue OneAgent development.

## Reporting

- Prompt and repository artifacts: English.
- User-visible reports: Russian.

## Profile and template

- `docs/codex/profiles/diagnostics-engine-implementation.md`
- `docs/codex/templates/diagnostics-engine-task.md`

## Required workflow

- `docs/codex/workflows/implementation.md`
- `docs/codex/workflows/diagnostics-engine.md`

## Authoritative documents

- `docs/Roadmap.md`
- `docs/Architecture.md`
- `docs/architecture/semantic-model-2.md`
- `docs/architecture/diagnostics-engine-investigation.md`
- `docs/adr/0039-workspace-service.md`
- `docs/adr/0051-mcp-semantic-tools.md`
- `docs/adr/0054-lsp-adapter.md`
- `docs/adr/0058-diagnostics-engine.md`
- all committed Task 3–6 code, tests, fixtures, and validation evidence

## Prerequisite

Task 6 is committed, its full gate passes, and all accepted domain, engine,
Workspace, cache, MCP, and LSP behavior is implemented.

## Task

Complete repository-owned Diagnostics Engine evidence, compatibility and scope
audits, and current-state documentation. Introduce no new production behavior.

## Required evidence

- Complete a requirement-to-test matrix for every ADR-0058 input, identity,
  vocabulary, suppression, ordering, bound, summary, report, failure,
  sensitive-data, location, snapshot/cache, MCP, LSP, and compatibility rule.
- Add only missing repository-owned fixtures or harness evidence required to
  cover empty, positive, negative, mixed-family, duplicate/collision,
  suppressed, exact/over-bound, reordered, repeated, cache/rebuild,
  missing/invalid location, malformed request, policy denial, lifecycle,
  channel-purity, and cleanup cases. Do not change accepted behavior.
- Run and record complete Graph/Analysis/Workspace/cache/watching/Runtime/MCP/
  LSP/public-process matrices with exact non-zero test counts and zero failures.
- Reassert Graph and producer authority, raw diagnostic and validation
  compatibility, immutable snapshot equality, seven-tool catalog, Tool Policy,
  MCP revision projections, LSP truthful capabilities, source confinement,
  HTTP/CLI/VS Code/EDT behavior, and existing Coverage state.
- Synchronize only README, Architecture, Semantic Model, and Roadmap current-
  state text required to describe the verified Diagnostics Engine boundary,
  public reporting, limitations, and Sprint 37 hand-off.
- Audit public APIs, dependencies/licenses, schemas/capabilities, secrets,
  credentials, personal/absolute paths, source content, ignored/generated
  artifacts, repository status, documentation links, scope, and absence of
  Rules Engine, UI, mutable-document, edit, remote, telemetry, and unsupported
  performance/security claims.

## Excluded scope

New production behavior, configurable rules, new diagnostics or graph facts,
new protocol capabilities, UI, mutable documents, edits/refactoring, release
review, Sprint 37 implementation, and prompt-suite retirement.

## Validation

Run the complete focused and public-process matrix required by ADR-0058, all
compatibility/audit/documentation checks, then:

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets --all-features -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
git diff --check
```

Record exact commands, counts, exits, skips, limitations, and artifact paths. A
zero-match filter, skipped required row, schema/capability mismatch, failed
audit, or incomplete compatibility row blocks completion.

## Suggested commit message

`Complete Sprint 36 diagnostics evidence`

## Final report additions

Report the acceptance matrix, exact test counts and commands, current-state
docs, API/dependency/catalog/capability/sensitive-data/scope audits, preserved
behavior, limitations, and canonical gate results.
