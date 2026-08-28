# Sprint 35 External AI Client Compatibility Execution Loop

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
- `docs/adr/0049-tool-execution-policy.md`
- `docs/adr/0050-mcp-server.md`
- `docs/adr/0051-mcp-semantic-tools.md`
- `docs/reviews/sprint-34-edt-integration-prototype.md`
- the pinned official client and MCP sources recorded by Task 1
- `docs/adr/0057-external-ai-client-compatibility.md` after Task 2

## Sprint objective and state

Make the repository-owned `oneagent-mcp` stdio server usable by the exact
supported Codex and Cursor clients through standards-conforming initialization,
tool discovery, tool calls, and shutdown while preserving the accepted modern
OneAgent protocol, immutable seven-tool catalog, semantic results, bounded
resources, deterministic failures, and existing clients.

Execution requires the committed planning baseline with subject
`Plan Sprint 35 External AI Client Compatibility`. Sprint 34 must remain
completed and Sprint 35 must be the unique eligible target. Resolve commit mode
and external access only from the current user instruction; this stored prompt
authorizes neither.

The current user's launch authorizes exactly one mandatory fresh-context
read-only reviewer for Task 6. Launch it automatically at the review gate
without separate confirmation. No other delegation is authorized.

## Ordered task manifest

| Order | Prompt | Required committed prerequisite | Task-owned outcome | Validation additions | Suggested commit message |
|---:|---|---|---|---|---|
| 1 | `01-investigate-external-ai-client-compatibility.md` | Sprint 35 planning commit | Pinned client, wire, protocol, lifecycle, response, and oracle investigation | Client/version/request/protocol/source/implementation/link gates | `Investigate Sprint 35 external AI client compatibility` |
| 2 | `02-define-external-ai-client-compatibility.md` | Task 1 | Accepted ADR-0057 | ADR/evidence/protocol/lifecycle/dependency/scope/link consistency | `Define Sprint 35 external AI client compatibility` |
| 3 | `03-implement-legacy-mcp-protocol.md` | Task 2 | Version negotiation, session state, legacy projection, and modern preservation | Protocol version/order/shape/error/isolation/regression matrices | `Implement Sprint 35 legacy MCP protocol` |
| 4 | `04-integrate-mcp-client-lifecycle.md` | Task 3 | Production stdio connection lifecycle | Public process initialize/list/call/EOF/shutdown/repetition matrices | `Integrate Sprint 35 MCP client lifecycle` |
| 5 | `05-complete-external-client-evidence.md` | Task 4 | Real-client and synthetic conformance evidence plus current-state docs | Codex/Cursor/public-process/conformance/audit/full workspace gates | `Complete Sprint 35 external client evidence` |
| 6 | `06-sprint-35-integration-review.md` | Task 5 and all validation | Independent review, reconciliation, artifact consistency, release-review hand-off, and conditional Sprint 34 suite retirement | Reviewer and primary complete matrices | `Complete Sprint 35 external AI client compatibility review` |

Execute strictly in order. Read every child prompt and selected framework module
completely before acting. Stop after the first blocking failure. Do not combine
tasks or treat a zero-match filter as passing evidence.

## External-access boundary

The current user authorized read/run access to the exact locally installed
Codex CLI supplied at runtime as `<ONEAGENT_CODEX_CLI>`, and authorized reading
the official Cursor installer plus downloading/running the exact official
Cursor client artifact supplied as `<ONEAGENT_CURSOR_CLI>`. The current live
planning evidence used Codex CLI `0.150.0-alpha.8` and Cursor Agent
`2026.08.25-3e8eec8`; Task 1 must verify and pin the executable hashes and
official sources before relying on them.

Run both clients only against the repository-local `oneagent-mcp`. Do not alter
global Codex or Cursor configuration. Keep downloaded binaries, disposable
workspaces, project-local client configs, wrappers, traces, and logs under
`local-artifacts/sprint-35/`. Never commit personal absolute paths,
credentials, tokens, client caches, or generated logs. Any additional external
client executable requires new explicit authorization; protocol fixtures do
not.

## Previous-suite retirement

The verified immediately preceding suite is exactly
`docs/codex/prompts/sprint-34-edt-integration-prototype/` with these eight
tracked files and no untracked additions at planning time:

- `00-sprint-34-execution-loop.md`
- `01-investigate-edt-integration-prototype.md`
- `02-define-edt-integration-prototype.md`
- `03-implement-edt-runtime-client.md`
- `04-implement-edt-command-lifecycle.md`
- `05-package-edt-plugin.md`
- `06-complete-edt-integration-evidence.md`
- `07-sprint-34-integration-review.md`

Only Task 6 may retire those exact files after a non-blocking review, complete
primary validation, and the same reviewer's artifact-consistency confirmation.

## Final report additions

Report every task outcome, exact commit and subject, focused and full
validation, exact client versions and executable evidence, external-access and
global-configuration compliance, protocol/version/lifecycle compatibility,
preserved seven-tool semantics, dependency/API impact, reviewer identity and
fresh/read-only evidence, reconciliation, previous-suite retirement, final
state, and v0.6 release-review eligibility.
