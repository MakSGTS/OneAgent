# Execute Sprint 28 MCP Server

Continue OneAgent development.

## Reporting

- Prompt and repository changes: English.
- User-visible reports: Russian.
- Stop after the first blocking failure.

## Template and workflow

- `docs/codex/templates/sprint-execution-loop.md`
- `docs/codex/workflows/sequential-sprint-execution.md`

Read both files and every Profile, Template, Core module, Workflow, ADR,
specification source, and architecture document selected by each child prompt
before acting.

## Canonical authorities

- `docs/Roadmap.md`, Sprint 28 execution plan
- `docs/adr/0037-runtime-service-container.md`
- `docs/adr/0038-http-api-health.md`
- `docs/adr/0040-graph-query-api.md`
- `docs/adr/0043-cli-client.md`
- `docs/adr/0049-tool-execution-policy.md`
- `docs/architecture/semantic-model-2.md`
- `docs/reviews/sprint-27-tool-execution-policy.md`
- official MCP specification and schema revision selected by Task 2

## Sprint objective and current state

Define and implement one bounded MCP server foundation with an explicit
protocol revision, strict JSON-RPC validation, truthful first-slice
capabilities, deterministic dispatch, newline-framed stdio, structured
Runtime/process lifecycle ownership, channel purity, and repository-owned
conformance. The accepted planning state is `next`; it becomes `active` only
when Task 1 starts from the committed planning baseline.

## Starting-state requirements

- The Sprint 28 planning commit is the current committed prerequisite.
- The working tree has no conflicting task-created change.
- `docs/codex/prompts/sprint-28-mcp-server/` owns this complete suite.
- The verified previous suite is exactly
  `docs/codex/prompts/sprint-27-tool-execution-policy/`, with eight tracked
  files and no untracked addition at planning time.
- The current launch explicitly authorizes one fresh-context read-only reviewer
  agent for Task 8 and one separate commit per successfully completed task.
- Production dependency edges require explicit current-user approval before
  Task 3.
- No task may require a live MCP client, external network, credential, remote
  transport, real signal, or real tool effect for acceptance.

## Ordered task manifest

| Order | Prompt | Required committed prerequisite | Task-owned outcome | Validation additions | Suggested commit message |
|---:|---|---|---|---|---|
| 1 | `01-investigate-mcp-server.md` | Sprint 28 planning commit | Verified official revision/schema, ownership, dependency, message/error/bounds, capability/dispatch, stdio, lifecycle, consumer, platform, and oracle evidence. | Source-provenance and repository inventory, link consistency, `git diff --check`. | `Investigate Sprint 28 MCP server` |
| 2 | `02-define-mcp-server.md` | Task 1 | Accepted ADR-0050 for the bounded MCP server contract. | ADR/investigation/specification consistency, link and decision audit, `git diff --check`. | `Define Sprint 28 MCP server` |
| 3 | `03-implement-mcp-protocol-domain.md` | Task 2 and explicit approval for planned production dependency edges | Bounded JSON-RPC/MCP domain and codec foundation. | Non-zero protocol-domain tests, dependency/public-surface/redaction audits, full workspace gate. | `Implement Sprint 28 MCP protocol domain` |
| 4 | `04-implement-mcp-server-dispatch.md` | Task 3 | Deterministic discovery, version handling, dispatch, notifications, and closed errors. | Non-zero dispatch/discovery/version/error tests, protocol regressions, full workspace gate. | `Implement Sprint 28 MCP server dispatch` |
| 5 | `05-implement-mcp-stdio-transport.md` | Task 4 | Bounded injected stdio framing, channel purity, EOF/cancellation/failure, and cleanup. | Non-zero framing/transport/cleanup tests, dispatch regressions, full workspace gate. | `Implement Sprint 28 MCP stdio transport` |
| 6 | `06-integrate-mcp-server-lifecycle.md` | Task 5 | Public MCP process and Runtime lifecycle composition. | Non-zero lifecycle/process tests, existing Runtime regressions, full workspace gate. | `Integrate Sprint 28 MCP server lifecycle` |
| 7 | `07-complete-mcp-server-evidence.md` | Task 6 | Public library/executable conformance, compatibility audits, and current-state docs. | Non-zero public matrix, capability/method/channel audits, Runtime compatibility, full workspace gate. | `Complete Sprint 28 MCP server evidence` |
| 8 | `08-sprint-28-integration-review.md` | Task 7 and all implementation validation | Independent and primary review, consistency check, decision, state transition, Sprint 29 hand-off, and conditional Sprint 27 suite retirement. | Independent and primary focused/public/full matrices, range/scope/link/retirement audits. | `Complete Sprint 28 MCP server review` |

## Commit authorization mode

The current launch requests one separate commit after every successfully
completed task. Stage only task-owned paths and create exactly one logical
commit per completed task. Do not create empty commits for `already_complete`.

## Initial audit additions

- Record sprint start time, exact `HEAD`, `git status --short`, relevant history,
  and available runtime token telemetry.
- Verify every manifest path, authority, prerequisite, and commit message agrees
  with the committed Roadmap plan.
- Recheck official specification mutability and the previous-suite tracked,
  filesystem, and untracked inventories.
- Preserve `.codex/`, unrelated files, and all pre-existing work.

## Task loop additions

- Read each child prompt completely and print its Change Contract before edits.
- Treat specification/schema mismatch, an unapproved dependency edge, false
  capability advertisement, wrong JSON-RPC error precedence, protocol-output
  contamination, detached I/O/task state, or external-client-dependent
  acceptance as blocking.
- Treat zero matched tests as missing evidence.
- Continue only after the task commit is verified and task-created state is
  clean.

## Already-complete, failure, and review gates

`already_complete` requires current committed evidence and successful required
validation for every acceptance criterion. Stop at the first failed
prerequisite, implementation, validation, staging, commit, or review gate.

Run Task 8 only after Tasks 1-7 are committed or proven `already_complete`.
Task 8 must use one separate fresh-context read-only reviewer under
`docs/codex/workflows/review.md`, preserve its report independently, reconcile
it with primary evidence, and obtain the same reviewer's final artifact-
consistency check. Only a non-blocking effective decision plus successful
validation may mark Sprint 28 `completed`, make Sprint 29 `next`, and retire
the verified Sprint 27 suite atomically with the review commit.

## Final report additions

Report ordered task outcomes, timestamps and elapsed times, available token
telemetry, exact commits and subjects, validation, independent reviewer identity
and read-only/fresh-context confirmation, reviewer recommendation, evidence
discrepancies and consistency result, changed/preserved paths, `.codex/` status,
Sprint 27 retirement and deletions, Sprint 29 eligibility, final `HEAD`, and
final `git status --short`.
