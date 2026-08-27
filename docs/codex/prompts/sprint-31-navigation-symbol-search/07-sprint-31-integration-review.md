# Sprint 31 Navigation and Symbol Search Integration Review

Continue OneAgent development.

## Reporting

- Prompt and repository artifacts: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/review.md`

## Template

`docs/codex/templates/review-task.md`

## Authoritative documents

- `docs/Roadmap.md`
- `docs/Architecture.md`
- `docs/architecture/semantic-model-2.md`
- `docs/architecture/navigation-symbol-search-investigation.md`
- `docs/adr/0039-workspace-service.md`
- `docs/adr/0040-graph-query-api.md`
- `docs/adr/0049-tool-execution-policy.md`
- `docs/adr/0050-mcp-server.md`
- `docs/adr/0051-mcp-semantic-tools.md`
- `docs/adr/0052-vscode-extension-foundation.md`
- `docs/adr/0053-navigation-symbol-search.md`
- `docs/reviews/sprint-30-vscode-extension-foundation.md`
- every committed Sprint 31 prompt

## Prerequisites / Required gate

Tasks 1-6 are committed in manifest order, every required validation succeeds,
and no task-created change is uncommitted. Resolve the exact planning-through-
Task-6 commit range and preserve unrelated pre-existing user changes.

The current user's launch explicitly authorizes exactly one fresh-context
read-only reviewer under `docs/codex/workflows/review.md`. Launch it
automatically without separate confirmation. No other reviewer or delegation is
authorized.

## Review target

Review the exact Sprint 31 planning-through-Task-6 range. Give the reviewer only
the repository root, exact range, authorities, objective, criteria, exclusions,
validation matrix, and required output contract. Do not provide an expected
decision or implementation-agent conclusions.

## Scope

### Included

Plan/order; pinned investigation; ADR-0053; source path/span model and producers;
Graph/query/validation/compatibility behavior; Workspace confinement; MCP
catalog/schema/Tool Policy/dispatch/bounds/errors; public process behavior; VS
Code commands, client validation, Quick Pick, document opening, selection,
cancellation, failures, activation and cleanup; real EDT/Designer fixtures;
unit, integration, Extension Host, real-process, packaging, cross-platform CI,
dependency, scope, and documentation evidence.

### Excluded

Source content disclosure, fuzzy ranking beyond accepted behavior, filesystem
search, LSP and provider APIs, reference UI, diagnostics, chat/context UI,
workspace reload/watch changes, remote/web/multi-root, external clients,
Marketplace work, telemetry, Runtime installation, edits/refactoring, and broad
performance/security claims.

## Review Criteria

- Every Roadmap and ADR-0053 criterion has independent executed evidence.
- Source locations are typed, source-derived, deterministic, workspace-confined,
  coordinate-correct, and optional where unsupported; consumers do not decode
  opaque provenance.
- Search matching, ordering, ambiguity, limits, errors, and Tool Policy are
  bounded and stable.
- Protocol discovery, schemas, handlers, public process, TypeScript client,
  manifest, UI, tests, CI, and package contents agree exactly.
- Existing graph identities, six-tool behavior, Runtime/HTTP/CLI, Sprint 30
  lifecycle, channel purity, failure cleanup, and deferred scope are preserved.
- Required tests are non-zero; clean extension and canonical Rust gates pass.

## Independent reviewer required output

Return the exact range and observed initial HEAD/status; one recommendation
(`pass`, `pass with non-blocking follow-ups`, or `blocked`); an
acceptance-evidence matrix; blocking/non-blocking findings with exact evidence;
missing evidence; commands and exact outcomes including zero/unexecuted checks;
scope/exclusion conformance; residual risks; next action; and confirmation of
fresh context, read-only operation, and no delegation.

The primary must independently inspect the same range and rerun the complete
source-location/Graph/adapter/Workspace/Tool Policy/protocol/Runtime/public-
process/extension/Extension Host/VSIX inventory matrix, canonical Rust workspace
gate, and API/catalog/schema/handler/policy/manifest/dependency/scope/ignored/
secret/path/generated-artifact/link audits. Preserve both evidence sets and use
the more severe decision. Do not fix findings inside the review task.

## Authorized review outputs and state transition

Only after a non-blocking effective decision and successful validation:

1. Draft `docs/reviews/sprint-31-navigation-symbol-search.md`.
2. Ask the same reviewer to verify read-only that the artifact preserves every
   finding, missing-evidence item, result, decision, and risk.
3. After a passing consistency check, update `docs/Roadmap.md` to mark Sprint
   31 `completed` and Sprint 32 the unique `next` target.
4. Re-enumerate and retire exactly these tracked files from
   `docs/codex/prompts/sprint-30-vscode-extension-foundation/`:
   - `00-sprint-30-execution-loop.md`
   - `01-investigate-vscode-extension-foundation.md`
   - `02-define-vscode-extension-foundation.md`
   - `03-establish-vscode-extension-package.md`
   - `04-implement-mcp-runtime-client.md`
   - `05-integrate-vscode-runtime-lifecycle.md`
   - `06-complete-vscode-extension-evidence.md`
   - `07-sprint-30-integration-review.md`
5. Commit the review artifact, state transition, and exact deletions atomically.

Stop before mutation if the reviewer blocks, evidence disagrees, consistency
fails, the suite inventory differs, or an untracked file would be endangered.

## Task-specific Validation

Run the reviewer and primary matrices above, verify current Sprint 31 suite
completeness, exact Sprint 30 tracked/filesystem inventory, all links after
deletion, Roadmap state, staged paths, and `git diff --check`.

## Suggested commit message

`Complete Sprint 31 navigation and symbol search review`

## Final report additions

Report reviewer identity and fresh/read-only/no-delegation evidence, both
validation ledgers, reconciliation, artifact consistency, effective decision,
retired paths, final state, and Sprint 32 eligibility.
