# Sprint 34 EDT Integration Prototype Execution Loop

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
- `docs/adr/0050-mcp-server.md`
- `docs/adr/0051-mcp-semantic-tools.md`
- `docs/adr/0052-vscode-extension-foundation.md`
- `docs/reviews/sprint-33-ai-chat-context-panel.md`
- the pinned official Eclipse and 1C plug-in-development sources recorded by Task 1

## Sprint objective and state

Prove one bounded native EDT user workflow: recognize one local EDT
configuration project, invoke a configured `oneagent-mcp` process in that
project root, validate one compatibility probe, report a stable visible result,
and clean up every owned resource without moving semantic authority into Java.

Execution requires the committed planning baseline with subject
`Plan Sprint 34 EDT Integration Prototype`. Sprint 33 must remain completed and
Sprint 34 must be the unique eligible target. Resolve commit mode only from the
current user instruction; this stored prompt does not authorize commits.

The current user's launch authorizes exactly one mandatory fresh-context
read-only reviewer for Task 7. Launch it automatically at the review gate
without separate confirmation. No other delegation is authorized.

## Ordered task manifest

| Order | Prompt | Required committed prerequisite | Task-owned outcome | Validation additions | Suggested commit message |
|---:|---|---|---|---|---|
| 1 | `01-investigate-edt-integration-prototype.md` | Sprint 34 planning commit | Pinned EDT/PDE/p2/JDK/Maven/API, workflow, compatibility, and executable-oracle investigation | Provenance/bundle/API/nature/process/toolchain/host/inventory/link gates | `Investigate Sprint 34 EDT integration prototype` |
| 2 | `02-define-edt-integration-prototype.md` | Task 1 | Accepted ADR-0056 | ADR/evidence/API/dependency/scope/link consistency | `Define Sprint 34 EDT integration prototype` |
| 3 | `03-implement-edt-runtime-client.md` | Task 2 | Dependency-free bounded Java Runtime probe and real-process evidence | Protocol/parser/bounds/timeout/cancellation/process/repetition matrices | `Implement Sprint 34 EDT Runtime client` |
| 4 | `04-implement-edt-command-lifecycle.md` | Task 3 | Native command, EDT project gate, configuration, UI result, job, and lifecycle ownership | Command/selection/configuration/job/UI/error/cancellation/disposal/PDE-host matrices | `Implement Sprint 34 EDT command lifecycle` |
| 5 | `05-package-edt-plugin.md` | Task 4 | Installable bundle, feature, p2 repository, and authorized EDT 2026.1 host proof | Clean Tycho/package/inventory/install/uninstall/host/repetition gates | `Package Sprint 34 EDT integration prototype` |
| 6 | `06-complete-edt-integration-evidence.md` | Task 5 | Complete CI, compatibility, package, security, scope, and current-state evidence | Maven/Java/Host/package/CI/Rust/dependency/license/secret/path/generated/link gates | `Complete Sprint 34 EDT integration evidence` |
| 7 | `07-sprint-34-integration-review.md` | Task 6 and all validation | Independent review, reconciliation, artifact consistency, Sprint 35 hand-off, and conditional Sprint 33 suite retirement | Reviewer and primary complete matrices | `Complete Sprint 34 EDT integration prototype review` |

Execute strictly in order. Read every child prompt and selected framework module
completely before acting. Stop after the first blocking failure. Do not combine
tasks or treat a zero-match filter as passing evidence.

## External-access boundary

The current user authorized read/run access only to the exact application
bundles supplied at runtime as these untracked values:

- `<ONEAGENT_ECLIPSE_APP>`: the authorized Eclipse plug-in-development bundle;
- `<ONEAGENT_EDT_APP>`: the authorized 1C:EDT 2026.1 bundle.

The current user authorized read-only access, with writes and deletion
forbidden, to the exact installed pool supplied at runtime as
`<ONEAGENT_P2_POOL>`. Resolve these placeholders only from the current user
instruction or execution environment; never commit their personal absolute
values. Keep temporary configurations, workspaces, toolchains, repositories,
and logs inside `local-artifacts/` or a disposable temporary directory. Never
store or print ITS credentials.

## Previous-suite retirement

The verified immediately preceding suite is exactly
`docs/codex/prompts/sprint-33-ai-chat-context-panel/` with these nine tracked
files and no untracked additions at planning time:

- `00-sprint-33-execution-loop.md`
- `01-investigate-ai-chat-context-panel.md`
- `02-define-ai-chat-context-panel.md`
- `03-implement-context-runtime-client.md`
- `04-implement-context-panel.md`
- `05-implement-ai-chat-participant.md`
- `06-integrate-chat-context-extension.md`
- `07-complete-chat-context-evidence.md`
- `08-sprint-33-integration-review.md`

Only Task 7 may retire those exact files after a non-blocking review, complete
validation, and the same reviewer's artifact-consistency confirmation.

## Final report additions

Report every task outcome, exact commit/subject, validation, preserved
pre-existing paths, external read/run and read-only compliance, JDK architecture
and product compatibility, API/dependency impact, reviewer identity and
fresh/read-only evidence, reconciliation, previous-suite retirement, final
state, and Sprint 35 eligibility.
