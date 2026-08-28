# Investigate Sprint 35 External AI Client Compatibility

Continue OneAgent development.

## Reporting

- Prompt and repository artifacts: English.
- User-visible reports: Russian.

## Profile and template

- `docs/codex/profiles/investigation.md`
- `docs/codex/templates/investigation-task.md`

## Authoritative documents and sources

- `docs/Roadmap.md`
- `docs/Architecture.md`
- `docs/architecture/semantic-model-2.md`
- `docs/adr/0049-tool-execution-policy.md`
- `docs/adr/0050-mcp-server.md`
- `docs/adr/0051-mcp-semantic-tools.md`
- `docs/reviews/sprint-34-edt-integration-prototype.md`
- repository MCP protocol, Runtime stdio, process tests, fixtures, and consumers
- official immutable Codex, Cursor, and Model Context Protocol sources
- exact user-authorized client executables and repository-local traces

## Prerequisites / required gate

- The committed Sprint 35 planning baseline is HEAD.
- Sprint 34 is completed and Sprint 35 is the unique eligible target.
- External access remains limited exactly as recorded by the master prompt.

## Task

Create
`docs/architecture/external-ai-client-compatibility-investigation.md` and
update only the Sprint 35 Roadmap state needed to record Task 1 start. Produce
decision-ready evidence for ADR-0057 without production implementation.

## Required evidence

- Pin the exact Codex and Cursor executable versions, hashes, official source or
  download URLs, retrieval date, invocation modes, config scope, working
  directories, and redacted reproduction commands. Never track a personal path.
- Capture and explain each client's exact first `initialize` request, protocol
  version, capabilities, client info, subsequent expected lifecycle, list/call
  behavior, success oracle, and current exact server failure.
- Pin official MCP lifecycle, initialization, version negotiation, capability,
  notification, tools/list, tools/call, error, cancellation, shutdown, and
  stdio requirements for every candidate supported revision. Distinguish
  stable client practice from newer opt-in specifications.
- Inventory current `oneagent-protocol` and Runtime ownership, public APIs,
  request metadata rules, stateless dispatch, response fields, error
  precedence, framing, limits, EOF behavior, tests, and all IDE/EDT consumers.
- Compare bounded compatibility candidates: connection-owned negotiated state,
  explicit session input to pure dispatch, version-specific response
  projection, compatibility adapter, and any evidence-supported alternative.
  Record migration and concurrency implications without selecting one.
- Build a version/method/ordering/response matrix covering initialize,
  initialized, pre-initialize request, duplicate initialize, tools/list,
  tools/call success/domain failure, unknown methods, malformed input,
  notifications, request IDs, EOF, shutdown/exit where applicable, repetition,
  two-session isolation, modern regression, and clean termination.
- Identify repository-owned synthetic fixtures and exact public-client process
  seams for macOS evidence plus platform-neutral CI. Record how other
  MCP-capable clients are represented without claiming unexecuted compatibility.
- State every decision ADR-0057 must make, dependency approval gates, security
  and configuration constraints, rejected candidates supported by evidence,
  exact first slice, and deferred transports, authentication, remote clients,
  publication, and unsupported protocol revisions.

## Excluded scope

Architecture acceptance, Rust changes, Cargo changes, new dependencies,
semantic-tool or catalog changes, client installation outside
`local-artifacts/`, global client configuration, credentials, release review,
and compatibility claims unsupported by executable evidence.

## Validation

Run client version/hash/source/request-trace audits, protocol-source and schema
audits, current implementation and consumer inventory, reproduction checks for
both baseline failures, Markdown link checks, applicable existing MCP protocol
and public-process tests, `git diff --check`, and an unrelated-change audit.
Zero-match evidence is not sufficient.

## Suggested commit message

`Investigate Sprint 35 external AI client compatibility`

## Final report additions

Report pinned clients and official sources, exact current failures and wire
requests, protocol differences, implementation ownership, candidate boundaries,
test oracles, unresolved ADR questions, external-access compliance, and
unchanged production behavior.
