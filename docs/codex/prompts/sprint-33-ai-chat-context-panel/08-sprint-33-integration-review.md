# Sprint 33 AI Chat and Context Panel Integration Review

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
- `docs/architecture/ai-chat-context-panel-investigation.md`
- `docs/adr/0044-context-engine.md`
- `docs/adr/0049-tool-execution-policy.md`
- `docs/adr/0050-mcp-server.md`
- `docs/adr/0051-mcp-semantic-tools.md`
- `docs/adr/0052-vscode-extension-foundation.md`
- `docs/adr/0053-navigation-symbol-search.md`
- `docs/adr/0055-ai-chat-context-panel.md`
- `docs/reviews/sprint-32-lsp-adapter.md`
- every committed Sprint 33 prompt

## Prerequisites / Required gate

Tasks 1-7 are committed in manifest order, every required validation succeeds,
and no task-created change is uncommitted. Resolve the exact planning-through-
Task-7 range and preserve unrelated pre-existing user changes.

The current user's launch explicitly authorizes exactly one fresh-context
read-only reviewer under `docs/codex/workflows/review.md`. Launch it
automatically without separate confirmation. No other delegation is authorized.

## Review target

Review the exact Sprint 33 planning-through-Task-7 range. Give the reviewer only
the repository root, exact range, authorities, objective, criteria, exclusions,
validation matrix, and required output contract. Do not provide an expected
decision or implementation-agent conclusions.

## Scope

### Included

Plan/order; pinned platform investigation; ADR-0055; Context client decoding
and serialization; explicit semantic selection and read-only panel; Chat/model
messages, streaming, cancellation and failures; manifest/activation/lifecycle;
Runtime/MCP/Context/navigation compatibility; unit/process/Host/package/VSIX/
CI/dependency/security/scope/documentation evidence.

### Excluded

New Rust/MCP semantics, Runtime LLM providers, source reads, implicit context,
model tools/edits, panel scripts, persistence/history, diagnostics UI,
remote/web/multi-root/EDT, Marketplace, telemetry, and broad quality,
performance, or security claims.

## Review Criteria

- Every Roadmap and ADR-0055 criterion has independent executed evidence.
- Context decoding, selection, prompt assembly, streaming, cancellation,
  rendering, escaping, bounds, failures, invalidation, and disposal are
  deterministic and fail closed.
- Every model input is explicit and inspectable; no semantic, source, secret,
  tool, edit, history, or filesystem authority is invented in TypeScript.
- Manifest/API/registration, Runtime catalog, unit/process/Host/package/CI,
  dependency, security, scope, and docs agree with non-zero required tests.

## Independent reviewer required output

Return the exact range and observed initial HEAD/status; one recommendation
(`pass`, `pass with non-blocking follow-ups`, or `blocked`); an acceptance-
evidence matrix; blocking/non-blocking findings with exact evidence; missing
evidence; commands and exact outcomes including zero/unexecuted checks; scope/
exclusion conformance; residual risks; next action; and confirmation of fresh
context, read-only operation, and no delegation.

The primary must independently inspect the same range and rerun the complete
Context/client/panel/chat/lifecycle/unit/process/Host/package/VSIX/CI matrix,
canonical Rust compatibility gate, and API/manifest/catalog/dependency/license/
scope/security/secret/path/generated-artifact/link audits. Preserve both
evidence sets and use the more severe decision. Do not fix findings inside the
review task.

## Authorized review outputs and state transition

Only after a non-blocking effective decision and successful validation:

1. Draft `docs/reviews/sprint-33-ai-chat-context-panel.md`.
2. Ask the same reviewer to verify read-only that the artifact preserves every
   finding, missing-evidence item, result, decision, and risk.
3. After a passing consistency check, update `docs/Roadmap.md` to mark Sprint
   33 `completed` and Sprint 34 the unique `next` target.
4. Re-enumerate and retire exactly these tracked files from
   `docs/codex/prompts/sprint-32-lsp-adapter/`:
   - `00-sprint-32-execution-loop.md`
   - `01-investigate-lsp-adapter.md`
   - `02-define-lsp-adapter.md`
   - `03-implement-lsp-protocol-core.md`
   - `04-implement-lsp-runtime-lifecycle.md`
   - `05-implement-lsp-navigation-symbols.md`
   - `06-implement-lsp-diagnostics.md`
   - `07-complete-lsp-evidence.md`
   - `08-sprint-32-integration-review.md`
5. Commit the review artifact, state transition, and exact deletions atomically.

Stop before mutation if the reviewer blocks, evidence disagrees, consistency
fails, the suite inventory differs, or an untracked file would be endangered.

## Task-specific Validation

Run the reviewer and primary matrices above, verify current Sprint 33 suite
completeness, exact Sprint 32 tracked/filesystem inventory, all links after
deletion, Roadmap state, staged paths, and `git diff --check`.

## Suggested commit message

`Complete Sprint 33 AI chat and context panel review`

## Final report additions

Report reviewer identity and fresh/read-only/no-delegation evidence, both
validation ledgers, reconciliation, artifact consistency, effective decision,
retired paths, final state, and Sprint 34 eligibility.
