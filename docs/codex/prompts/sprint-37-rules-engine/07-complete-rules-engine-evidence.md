# Complete Sprint 37 Rules Engine Evidence

Continue OneAgent development.

## Reporting

- Prompt and repository artifacts: English.
- User-visible reports: Russian.

## Profile and template

- `docs/codex/profiles/rules-engine-implementation.md`
- `docs/codex/templates/rules-engine-task.md`

## Required workflows

- `docs/codex/workflows/implementation.md`
- `docs/codex/workflows/rules-engine.md`
- `docs/codex/workflows/diagnostics-engine.md`

## Authoritative documents

- `docs/Roadmap.md`
- `docs/Architecture.md`
- `docs/architecture/semantic-model-2.md`
- `docs/architecture/rules-engine-investigation.md`
- `docs/adr/0039-workspace-service.md`
- `docs/adr/0042-persistent-cache.md`
- `docs/adr/0058-diagnostics-engine.md`
- `docs/adr/0059-rules-engine.md`
- all committed Task 3–6 code, tests, fixtures, and validation evidence

## Prerequisite

Task 6 is committed, its full gate passes, and all accepted registry, planning,
execution, diagnostic, Workspace, cache, lifecycle, and compatibility behavior
is implemented.

## Task

Complete repository-owned Rules Engine evidence, compatibility and scope
audits, and current-state documentation. Introduce no new production behavior.

## Required evidence

- Complete a requirement-to-test matrix for every ADR-0059 input, owner,
  identity, registration, dependency, ordering, configuration, applicability,
  execution, cancellation, failure, result, diagnostic integration, bound,
  error, sensitive-data, snapshot/cache, lifecycle, projection, and
  compatibility rule.
- Add only missing repository-owned fixtures or harness evidence required to
  cover empty, positive, duplicate/conflict, dependency topology and failures,
  configuration, disabled/inapplicable, rule failure, cancellation, result
  collision, exact/over-bound, reordered, repeated, cache/rebuild/invalidation,
  watcher replacement, policy/confinement, lifecycle, and cleanup cases. Do not
  change accepted behavior.
- Run and record complete Graph/Analysis/Diagnostics/Workspace/cache/watching/
  Runtime/MCP/LSP/public-process matrices with exact non-zero test counts and
  zero failures.
- Reassert canonical Graph, validation, provenance, location, and Diagnostics
  Engine authority; immutable snapshot equality; cache recovery; seven-tool
  catalog; Tool Policy; MCP revision projections; LSP capabilities and
  confinement; HTTP/CLI/VS Code/EDT behavior; and unchanged Coverage state.
- Synchronize only README, Architecture, Semantic Model, and Roadmap current-
  state text required to describe the verified Rules Engine boundary,
  limitations, and Sprint 38 hand-off.
- Audit public APIs, dependencies/licenses, schemas/capabilities, configuration
  and secrets, personal/absolute paths, source content, ignored/generated
  artifacts, repository status, documentation links, scope, and absence of
  dynamic plugins, scripts, remote rules, rule-management UI/protocol, mutable
  documents, fixes, source edits, telemetry, and unsupported performance or
  security claims.

## Excluded scope

New production behavior, new rule families beyond the accepted conformance
slice, external configuration, new protocol/IDE capabilities, plugins, scripts,
remote rules, mutable documents, fixes/refactoring, Git Change Adapter,
release review, Sprint 38 implementation, and prompt-suite retirement.

## Validation

Run the complete focused and public-process matrix required by ADR-0059, all
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
zero-match filter, skipped required row, authority/schema/capability mismatch,
failed audit, or incomplete compatibility row blocks completion.

## Suggested commit message

`Complete Sprint 37 rules engine evidence`

## Final report additions

Report the acceptance matrix, exact test counts and commands, current-state
docs, rule-domain/diagnostic/snapshot/cache/lifecycle evidence, API/dependency/
catalog/capability/configuration/sensitive-data/scope audits, preserved
behavior, limitations, and canonical gate results.
