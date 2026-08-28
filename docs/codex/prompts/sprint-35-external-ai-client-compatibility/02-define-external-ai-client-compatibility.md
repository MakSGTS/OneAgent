# Define Sprint 35 External AI Client Compatibility

Continue OneAgent development.

## Reporting

- Prompt and repository artifacts: English.
- User-visible reports: Russian.

## Profile and template

- `docs/codex/profiles/architecture.md`
- `docs/codex/templates/architecture-task.md`

## Required workflow

`docs/codex/workflows/architecture.md`

## Authoritative documents

- `docs/Roadmap.md`
- `docs/Architecture.md`
- `docs/architecture/semantic-model-2.md`
- `docs/architecture/external-ai-client-compatibility-investigation.md`
- `docs/adr/0049-tool-execution-policy.md`
- `docs/adr/0050-mcp-server.md`
- `docs/adr/0051-mcp-semantic-tools.md`
- `docs/reviews/sprint-34-edt-integration-prototype.md`

## Prerequisite

Task 1 is committed and its investigation contains no blocking unknown or
unapproved production dependency.

## Task

Create `docs/adr/0057-external-ai-client-compatibility.md` and synchronize only
planning-level architecture text required by the accepted decision. Implement
no production behavior.

## Required decisions

- Fix the exact accepted MCP protocol revisions and deterministic negotiation,
  downgrade, unsupported-version, and advertised-version rules.
- Fix connection/session ownership, state transitions, initialize and
  initialized ordering, method availability, duplicate initialization,
  notification behavior, shutdown/exit or EOF semantics, and isolation across
  sequential or concurrent connections.
- Define how pure protocol dispatch receives negotiated context without global
  mutable state, and assign ownership between `oneagent-protocol` and Runtime
  stdio transport.
- Fix capability interpretation, client-info handling, request IDs, error
  precedence/codes/data, malformed input behavior, cancellation boundaries,
  framing/size/time/resource limits, stderr behavior, and cleanup.
- Define exact version-specific initialize, tools/list, and tools/call response
  projections. Preserve the immutable seven-tool catalog, Tool Policy,
  OneAgent semantic result authority, ordering, and accepted modern fields.
- Fix public API compatibility and migration impact for every protocol/Runtime,
  VS Code, LSP, EDT, test, and fixture consumer found by Task 1.
- Fix dependency policy, security/configuration boundaries, exact first slice,
  synthetic conformance matrix, real Codex/Cursor acceptance matrix, CI versus
  authorized local-host evidence, and zero-skip/zero-match handling.
- Record rejected alternatives and defer unsupported revisions, extra clients,
  HTTP/SSE or remote transport, authentication, client installation,
  publication, tool-catalog expansion, and semantic changes.

## Acceptance evidence

ADR-0057 is `Accepted`, maps every investigation question to one explicit
decision or deferral, assigns each behavior to Tasks 3-5, identifies all public
consumer effects and migrations, introduces no production dependency without
approval, and agrees with Roadmap task boundaries and existing ADR authority.

## Excluded scope

Rust implementation, tests or fixtures that encode new behavior, client
downloads or execution, global configuration, prompt-suite retirement, Sprint
completion, release review, and future-client compatibility claims.

## Validation

Run ADR/investigation question coverage, protocol-version and response-schema
consistency, consumer/migration ownership, dependency and scope audits,
Markdown link checks, `git diff --check`, and unrelated-change inspection.

## Suggested commit message

`Define Sprint 35 external AI client compatibility`

## Final report additions

Report the accepted revisions, state machine, response projection, ownership,
compatibility/migration, dependency, evidence, rejected-alternative, and
deferred-scope decisions plus unchanged production behavior.
