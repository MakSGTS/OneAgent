# Sprint 34 EDT Integration Prototype Review

Continue OneAgent development.

## Reporting

- Prompt and repository artifacts: English.
- User-visible reports: Russian.

## Profile and template

- `docs/codex/profiles/review.md`
- `docs/codex/templates/review-task.md`

## Authoritative documents

- `docs/Roadmap.md`
- `docs/Architecture.md`
- `docs/architecture/semantic-model-2.md`
- `docs/architecture/edt-integration-prototype-investigation.md`
- `docs/adr/0050-mcp-server.md`
- `docs/adr/0051-mcp-semantic-tools.md`
- `docs/adr/0052-vscode-extension-foundation.md`
- `docs/adr/0056-edt-integration-prototype.md`
- `docs/reviews/sprint-33-ai-chat-context-panel.md`
- every committed Sprint 34 prompt

## Prerequisites / required gate

Tasks 1-6 are committed in manifest order, every required validation succeeds,
and no task-created change is uncommitted. Resolve the exact planning-through-
Task-6 range and preserve unrelated pre-existing user changes.

The current user's launch explicitly authorizes exactly one fresh-context
read-only reviewer under `docs/codex/workflows/review.md`. Launch it
automatically without separate confirmation. No other delegation is authorized.

## Review target

Review the exact Sprint 34 planning-through-Task-6 range. Give the reviewer only
the repository root, exact range, authorities, objective, criteria, exclusions,
external access boundary, validation matrix, and required output contract. Do
not provide an expected decision or implementation-agent conclusions.

## Scope

Included: plan/order; pinned official and installed evidence; ADR-0056;
dependency-free protocol/process client; bounds/failures/cancellation/cleanup;
EDT project and configuration gate; command/UI/job/lifecycle; bundle/feature/p2
repository; disposable install/uninstall and EDT-host behavior; Maven/JDK/PDE/
CI/package/dependency/security/scope/docs evidence; Rust/MCP and IDE-client
compatibility; and external read-only compliance.

Excluded: new Rust/MCP semantics, Java source parsing or semantic authority,
proprietary EDT implementation API, semantic editor UI, persistent connection,
automatic startup, remote/multi-project support, external publication, signing,
telemetry, bundled Runtime/JRE, credentials, and broad unsupported claims.

## Review criteria

- Every Roadmap and ADR-0056 criterion has independent executed evidence.
- Protocol validation, process ownership, selection, configuration, threading,
  UI publication, cancellation, replacement and disposal are deterministic and
  fail closed.
- Bundle imports, Java environments, command registration, feature/repository,
  install/uninstall, Host, CI, package and docs agree exactly.
- No credential, personal path, generated package, unauthorized dependency,
  external write, semantic duplication, or deferred feature is tracked or
  claimed.

## Independent reviewer required output

Return the exact range and observed initial HEAD/status; one recommendation
(`pass`, `pass with non-blocking follow-ups`, or `blocked`); an acceptance-
evidence matrix; blocking/non-blocking findings with exact evidence; missing
evidence; commands and exact outcomes including zero/unexecuted checks; scope/
exclusion and external-access conformance; residual risks; next action; and
confirmation of fresh context, read-only operation, and no delegation.

The primary must independently inspect the same range and rerun the complete
Java unit/process/PDE/EDT-host/package/install/CI matrix, canonical Rust and IDE
compatibility gates, and API/manifest/catalog/dependency/license/scope/security/
secret/path/generated-artifact/link audits. Preserve both evidence sets and use
the more severe decision. Do not fix findings inside the review task.

## Authorized review outputs and state transition

Only after a non-blocking effective decision and successful validation:

1. Draft `docs/reviews/sprint-34-edt-integration-prototype.md`.
2. Ask the same reviewer to verify read-only that the artifact preserves every
   finding, missing-evidence item, result, decision, and risk.
3. After a passing consistency check, update `docs/Roadmap.md` to mark Sprint
   34 `completed` and Sprint 35 the unique `next` target.
4. Re-enumerate and retire exactly these tracked files from
   `docs/codex/prompts/sprint-33-ai-chat-context-panel/`:
   - `00-sprint-33-execution-loop.md`
   - `01-investigate-ai-chat-context-panel.md`
   - `02-define-ai-chat-context-panel.md`
   - `03-implement-context-runtime-client.md`
   - `04-implement-context-panel.md`
   - `05-implement-ai-chat-participant.md`
   - `06-integrate-chat-context-extension.md`
   - `07-complete-chat-context-evidence.md`
   - `08-sprint-33-integration-review.md`
5. Commit the review artifact, state transition, and exact deletions atomically.

Stop before mutation if the reviewer blocks, evidence disagrees, consistency
fails, the suite inventory differs, an untracked file would be endangered, or
external read-only compliance cannot be proven.

## Validation

Run the reviewer and primary matrices above, verify current Sprint 34 suite
completeness, exact Sprint 33 tracked/filesystem inventory, all links after
deletion, Roadmap state, staged paths, and `git diff --check`.

## Suggested commit message

`Complete Sprint 34 EDT integration prototype review`

## Final report additions

Report reviewer identity and fresh/read-only/no-delegation evidence, both
validation ledgers, reconciliation, artifact consistency, effective decision,
retired paths, external-access compliance, final state, and Sprint 35
eligibility.
