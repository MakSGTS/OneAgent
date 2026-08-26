# Execute Sprint 29 MCP Semantic Tools

Continue OneAgent development.

## Reporting and execution

- Repository changes and commit messages: English.
- User-visible reports: Russian.
- Follow `docs/codex/templates/sprint-execution-loop.md` and
  `docs/codex/workflows/sequential-sprint-execution.md`.
- Stop after the first blocking failure.
- Read each child prompt and every selected authority completely before acting.

## Canonical authorities

- `docs/Roadmap.md`, Sprint 29 execution plan
- ADR-0040, ADR-0044, ADR-0049, and ADR-0050
- `docs/architecture/semantic-model-2.md`
- `docs/reviews/sprint-28-mcp-server.md`
- official MCP revision `2026-07-28` tools and schema sources selected by Task 2

## Ordered manifest

| Order | Prompt | Prerequisite | Commit |
|---:|---|---|---|
| 1 | `01-investigate-mcp-semantic-tools.md` | Planning commit | `Investigate Sprint 29 MCP semantic tools` |
| 2 | `02-define-mcp-semantic-tools.md` | Task 1 | `Define Sprint 29 MCP semantic tools` |
| 3 | `03-implement-mcp-tool-protocol.md` | Task 2 | `Implement Sprint 29 MCP tool protocol` |
| 4 | `04-implement-semantic-graph-tools.md` | Task 3 | `Implement Sprint 29 semantic graph tools` |
| 5 | `05-implement-impact-context-tools.md` | Task 4 | `Implement Sprint 29 impact and context tools` |
| 6 | `06-integrate-mcp-semantic-tools.md` | Task 5 | `Integrate Sprint 29 MCP semantic tools` |
| 7 | `07-complete-mcp-semantic-tool-evidence.md` | Task 6 | `Complete Sprint 29 MCP semantic tool evidence` |
| 8 | `08-sprint-29-integration-review.md` | Task 7 | `Complete Sprint 29 MCP semantic tools review` |

The current launch authorizes one separate commit after every successful task.
Stage task-owned paths explicitly. Do not create empty commits. Record exact
start/end times, HEAD/status, validations, outcomes, and available telemetry.

For every task print its Change Contract before edits. Treat false capability
or schema claims, Tool Policy bypass, raw path/source leakage, mutable snapshot
behavior, unbounded data, zero matched tests, failed validation, dirty task
handoff, or commit failure as blocking.

Task 8 uses exactly one fresh-context read-only reviewer under
`docs/codex/workflows/review.md`. The primary independently validates and asks
the same reviewer to check the draft artifact before any state transition or
Sprint 28 prompt deletion.
