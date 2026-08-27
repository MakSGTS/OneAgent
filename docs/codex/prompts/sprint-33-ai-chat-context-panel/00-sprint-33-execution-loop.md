# Sprint 33 AI Chat and Context Panel Execution Loop

Continue OneAgent development.

## Reporting

- Prompt and repository artifacts: English.
- User-visible reports: Russian.

## Profile

Use the profile selected by each child prompt.

## Template

`docs/codex/templates/sprint-execution-loop.md`

## Required workflow

`docs/codex/workflows/sequential-sprint-execution.md`

## Canonical authorities

- `AGENTS.md`
- `docs/Roadmap.md`
- `docs/Architecture.md`
- `docs/architecture/semantic-model-2.md`
- `docs/adr/0044-context-engine.md`
- `docs/adr/0049-tool-execution-policy.md`
- `docs/adr/0050-mcp-server.md`
- `docs/adr/0051-mcp-semantic-tools.md`
- `docs/adr/0052-vscode-extension-foundation.md`
- `docs/adr/0053-navigation-symbol-search.md`
- `docs/reviews/sprint-32-lsp-adapter.md`

## Sprint objective and state

Add one bounded VS Code chat participant and one inspectable read-only semantic
Context panel that consume canonical Runtime facts without moving semantic
authority, source access, or provider secrets into the extension.

Execution requires the committed planning baseline with subject
`Plan Sprint 33 AI Chat and Context Panel`. Sprint 32 must remain completed and
Sprint 33 must be the unique eligible target. Resolve commit mode only from the
current user instruction; this stored prompt does not authorize commits.

The current user's launch authorizes exactly one mandatory fresh-context
read-only reviewer for Task 8. Launch it automatically at the review gate
without separate confirmation. No other delegation is authorized.

## Ordered task manifest

| Order | Prompt | Required committed prerequisite | Task-owned outcome | Validation additions | Suggested commit message |
|---:|---|---|---|---|---|
| 1 | `01-investigate-ai-chat-context-panel.md` | Sprint 33 planning commit | Pinned API, ownership, security, compatibility, and executable-oracle investigation | Source/API/manifest/baseline/inventory/link gates | `Investigate Sprint 33 AI chat and context panel` |
| 2 | `02-define-ai-chat-context-panel.md` | Task 1 | Accepted ADR-0055 | ADR/source/link/scope consistency | `Define Sprint 33 AI chat and context panel` |
| 3 | `03-implement-context-runtime-client.md` | Task 2 | Strict Context domain/decoder and serialized Runtime operation | Client/decoder/process Context matrices | `Implement Sprint 33 context Runtime client` |
| 4 | `04-implement-context-panel.md` | Task 3 | Bounded semantic selection and read-only Context panel | Controller/rendering/security/lifecycle matrices | `Implement Sprint 33 context panel` |
| 5 | `05-implement-ai-chat-participant.md` | Task 4 | Bounded selected-Context chat participant | Model-message/stream/failure/cancellation matrices | `Implement Sprint 33 AI chat participant` |
| 6 | `06-integrate-chat-context-extension.md` | Task 5 | Manifest, command, activation, disposal, Runtime, Chat, panel, and Host integration | Manifest/audit/unit/process/Host/lifecycle matrices | `Integrate Sprint 33 chat and context extension` |
| 7 | `07-complete-chat-context-evidence.md` | Task 6 | Package, CI, compatibility, scope, security, and current-state evidence | Complete extension/package/VSIX/Rust/link audits | `Complete Sprint 33 chat and context evidence` |
| 8 | `08-sprint-33-integration-review.md` | Task 7 and all validation | Independent review, reconciliation, artifact consistency, Sprint 34 hand-off, and conditional Sprint 32 suite retirement | Reviewer and primary complete matrices | `Complete Sprint 33 AI chat and context panel review` |

Execute strictly in order. Read every child prompt and selected framework module
completely before acting. Stop after the first blocking failure. Do not combine
tasks or treat a zero-match filter as passing evidence.

## Previous-suite retirement

The verified immediately preceding suite is exactly
`docs/codex/prompts/sprint-32-lsp-adapter/` with these nine tracked files and no
untracked additions at planning time:

- `00-sprint-32-execution-loop.md`
- `01-investigate-lsp-adapter.md`
- `02-define-lsp-adapter.md`
- `03-implement-lsp-protocol-core.md`
- `04-implement-lsp-runtime-lifecycle.md`
- `05-implement-lsp-navigation-symbols.md`
- `06-implement-lsp-diagnostics.md`
- `07-complete-lsp-evidence.md`
- `08-sprint-32-integration-review.md`

Only Task 8 may retire those exact files after a non-blocking review, complete
validation, and the same reviewer's artifact-consistency confirmation.

## Final report additions

Report every task outcome, timestamp, elapsed time, exact commit/subject,
validation, available token telemetry, preserved pre-existing paths, API and
compatibility impact, reviewer identity and fresh/read-only evidence,
reconciliation, previous-suite retirement, final state, and Sprint 34
eligibility.
