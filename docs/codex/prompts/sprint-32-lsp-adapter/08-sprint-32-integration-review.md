# Sprint 32 LSP Adapter Integration Review

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
- `docs/architecture/lsp-adapter-investigation.md`
- `docs/adr/0039-workspace-service.md`
- `docs/adr/0050-mcp-server.md`
- `docs/adr/0051-mcp-semantic-tools.md`
- `docs/adr/0052-vscode-extension-foundation.md`
- `docs/adr/0053-navigation-symbol-search.md`
- `docs/adr/0054-lsp-adapter.md`
- `docs/reviews/sprint-31-navigation-symbol-search.md`
- every committed Sprint 32 prompt

## Prerequisites / Required gate

Tasks 1-7 are committed in manifest order, every required validation succeeds,
and no task-created change is uncommitted. Resolve the exact planning-through-
Task-7 range and preserve unrelated pre-existing user changes.

The current user's launch explicitly authorizes exactly one fresh-context
read-only reviewer under `docs/codex/workflows/review.md`. Launch it
automatically without separate confirmation. No other delegation is authorized.

## Review target

Review the exact Sprint 32 planning-through-Task-7 range. Give the reviewer only
the repository root, exact range, authorities, objective, criteria, exclusions,
validation matrix, and required output contract. Do not provide an expected
decision or implementation-agent conclusions.

## Scope

### Included

Plan/order; pinned LSP investigation; ADR-0054; protocol messages, lifecycle,
capabilities, dispatch and errors; Content-Length framing and public process;
Workspace/root/URI/position handling; accepted navigation/symbol/diagnostic
projection; Graph/adapter/MCP compatibility; real fixtures; unit/integration/
raw-process/cross-platform CI/dependency/scope/documentation evidence.

### Excluded

Mutable document synchronization and source analysis, unsupported LSP methods,
IDE-specific UI/provider migration, MCP changes, remote transports, external
client claims, diagnostic rules/configuration, edits/refactoring, telemetry,
and broad performance/security claims.

## Review Criteria

- Every Roadmap and ADR-0054 criterion has independent executed evidence.
- Wire/lifecycle/framing/capabilities/errors are bounded, deterministic,
  specification-conformant, channel-pure, and resource-owned.
- URIs/paths/positions are confined and correct; navigation/symbol/diagnostic
  results derive only from immutable canonical facts and advertise no deferred
  behavior.
- Public process, protocol, Runtime, Graph, adapters, MCP/HTTP/CLI, extension,
  CI, dependencies, tests, and docs agree; required tests are non-zero.

## Independent reviewer required output

Return the exact range and observed initial HEAD/status; one recommendation
(`pass`, `pass with non-blocking follow-ups`, or `blocked`); an acceptance-
evidence matrix; blocking/non-blocking findings with exact evidence; missing
evidence; commands and exact outcomes including zero/unexecuted checks; scope/
exclusion conformance; residual risks; next action; and confirmation of fresh
context, read-only operation, and no delegation.

The primary must independently inspect the same range and rerun the complete
protocol/Runtime/public-process/semantic/diagnostic/compatibility matrix,
canonical Rust workspace gate, and API/capability/handler/dependency/CI/scope/
secret/path/generated-artifact/link audits. Preserve both evidence sets and use
the more severe decision. Do not fix findings inside the review task.

## Authorized review outputs and state transition

Only after a non-blocking effective decision and successful validation:

1. Draft `docs/reviews/sprint-32-lsp-adapter.md`.
2. Ask the same reviewer to verify read-only that the artifact preserves every
   finding, missing-evidence item, result, decision, and risk.
3. After a passing consistency check, update `docs/Roadmap.md` to mark Sprint
   32 `completed` and Sprint 33 the unique `next` target.
4. Re-enumerate and retire exactly these tracked files from
   `docs/codex/prompts/sprint-31-navigation-symbol-search/`:
   - `00-sprint-31-execution-loop.md`
   - `01-investigate-navigation-symbol-search.md`
   - `02-define-navigation-symbol-search.md`
   - `03-implement-source-location-model.md`
   - `04-implement-navigation-mcp-tools.md`
   - `05-integrate-vscode-navigation-search.md`
   - `06-complete-navigation-search-evidence.md`
   - `07-sprint-31-integration-review.md`
5. Commit the review artifact, state transition, and exact deletions atomically.

Stop before mutation if the reviewer blocks, evidence disagrees, consistency
fails, the suite inventory differs, or an untracked file would be endangered.

## Task-specific Validation

Run the reviewer and primary matrices above, verify current Sprint 32 suite
completeness, exact Sprint 31 tracked/filesystem inventory, all links after
deletion, Roadmap state, staged paths, and `git diff --check`.

## Suggested commit message

`Complete Sprint 32 LSP adapter review`

## Final report additions

Report reviewer identity and fresh/read-only/no-delegation evidence, both
validation ledgers, reconciliation, artifact consistency, effective decision,
retired paths, final state, and Sprint 33 eligibility.
