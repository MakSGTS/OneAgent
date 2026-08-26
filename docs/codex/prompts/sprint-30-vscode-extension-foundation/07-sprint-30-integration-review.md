# Sprint 30 VS Code Extension Foundation Integration Review

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
- `docs/architecture/vscode-extension-foundation-investigation.md`
- `docs/adr/0002-runtime-composition-root.md`
- `docs/adr/0050-mcp-server.md`
- `docs/adr/0051-mcp-semantic-tools.md`
- `docs/adr/0052-vscode-extension-foundation.md`
- `docs/reviews/sprint-29-mcp-semantic-tools.md`
- every committed Sprint 30 prompt

## Prerequisites / Required gate

Tasks 1–6 are committed in manifest order, every required validation succeeds,
and no task-created change is uncommitted. Resolve the exact planning-through-
Task-6 commit range and preserve unrelated pre-existing user changes.

The current user's launch explicitly authorizes exactly one fresh-context
read-only reviewer under `docs/codex/workflows/review.md`. Launch it
automatically without separate confirmation. No other reviewer or delegation is
authorized.

## Review target

Review the exact Sprint 30 planning-through-Task-6 range. Give the reviewer only
the repository root, exact range, authorities, objective, criteria, exclusions,
validation matrix, and required output contract. Do not provide an expected
decision or implementation-agent conclusions.

## Scope

### Included

Framework prerequisite ancestry; plan/order; pinned investigation; ADR-0052;
package/lock/toolchain; manifest and activation; configuration; MCP client;
process and extension lifecycle; status UI; bounds/redaction; failure and
cleanup; unit, extension-host, real-process, packaging, cross-platform CI, docs,
dependency, and compatibility evidence.

### Excluded

Navigation/search, LSP, diagnostics, chat/context UI, EDT, remote/web hosts,
multi-root fan-out, concurrent MCP, workspace watching/reload, Runtime download,
Marketplace publication/signing, telemetry, authentication, semantic changes,
and broad performance/security claims.

## Review Criteria

- Every Roadmap and ADR-0052 criterion has independent executed evidence.
- Extension/Runtime/protocol/domain ownership remains separated.
- Manifest claims, source behavior, tests, lockfile, CI, and VSIX inventory
  agree exactly.
- Activation is bounded and demand-driven; configuration and diagnostics are
  validated and redacted.
- Process/request/resource lifecycle is deterministic and leak-free across
  success, failure, repetition, replacement, and deactivation.
- Required tests are non-zero and pinned; clean package and full Rust gates pass.
- Deferred scope remains absent.

## Independent reviewer required output

Return the exact range and observed initial HEAD/status; one recommendation
(`pass`, `pass with non-blocking follow-ups`, or `blocked`); an
acceptance-evidence matrix; blocking/non-blocking findings with exact evidence;
missing evidence; commands and exact outcomes including zero/unexecuted checks;
scope/exclusion conformance; residual risks; next action; and confirmation of
fresh context, read-only operation, and no delegation.

The primary must independently inspect the same range and rerun the complete
extension clean-install/typecheck/build/unit/extension-host/real-process/VSIX
inventory gate, canonical Rust workspace gate, link/dependency/scope/ignored/
secret/path/generated-artifact audits, and `git diff --check`. Preserve both
evidence sets and use the more severe decision. Do not fix findings inside the
review task.

## Authorized review outputs and state transition

Only after a non-blocking effective decision and successful validation:

1. Draft `docs/reviews/sprint-30-vscode-extension-foundation.md`.
2. Ask the same reviewer to verify read-only that the artifact preserves every
   finding, missing-evidence item, result, decision, and risk.
3. After a passing consistency check, update `docs/Roadmap.md` to mark Sprint
   30 `completed` and Sprint 31 the unique `next` target.
4. Re-enumerate and retire exactly these tracked files from
   `docs/codex/prompts/sprint-29-mcp-semantic-tools/`:
   - `00-sprint-29-execution-loop.md`
   - `01-investigate-mcp-semantic-tools.md`
   - `02-define-mcp-semantic-tools.md`
   - `03-implement-mcp-tool-protocol.md`
   - `04-implement-semantic-graph-tools.md`
   - `05-implement-impact-context-tools.md`
   - `06-integrate-mcp-semantic-tools.md`
   - `07-complete-mcp-semantic-tool-evidence.md`
   - `08-sprint-29-integration-review.md`
5. Commit the review artifact, state transition, and exact deletions atomically.

Stop before mutation if the reviewer blocks, evidence disagrees, consistency
fails, the suite inventory differs, or an untracked file would be endangered.

## Task-specific Validation

Run the reviewer and primary matrices above, verify current Sprint 30 suite
completeness, previous-suite inventory, all links after deletion, Roadmap state,
staged paths, and `git diff --check`.

## Suggested commit message

`Complete Sprint 30 VS Code extension review`

## Final report additions

Report reviewer identity and fresh/read-only/no-delegation evidence, both
validation ledgers, reconciliation, artifact consistency, effective decision,
retired paths, final state, and Sprint 31 eligibility.
