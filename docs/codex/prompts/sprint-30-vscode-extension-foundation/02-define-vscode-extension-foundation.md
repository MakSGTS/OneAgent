# Define Sprint 30 VS Code Extension Foundation

Continue OneAgent development.

## Reporting

- Prompt and repository artifacts: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/architecture.md`

## Template

`docs/codex/templates/architecture-task.md`

## Authoritative ADRs / architecture documents

- `docs/architecture/vscode-extension-foundation-investigation.md`
- `docs/adr/0002-runtime-composition-root.md`
- `docs/adr/0050-mcp-server.md`
- `docs/adr/0051-mcp-semantic-tools.md`
- `docs/Roadmap.md`

## Prerequisites / Required gate

Task 1 is committed and its investigation has no blocking unknown.

## Task

Create and accept `docs/adr/0052-vscode-extension-foundation.md`.

## Scope

### Included

Define the extension identity and supported desktop/workspace-host matrix;
pinned Node/package/TypeScript/build/test/package toolchain; manifest and VSIX
inventory; activation/deactivation; commands; workspace-scoped configuration
and precedence; executable/workspace resolution; environment policy; one owned
`oneagent-mcp` child; MCP initialization and sequential request correlation;
frame/result/diagnostic bounds; connection states and status UI; failures,
configuration replacement, unexpected exit, restart if justified, shutdown,
cleanup, and deterministic unit/extension-host/real-process/CI evidence.

### Excluded

Production code, navigation/search, LSP, diagnostics engine/UI, chat/context
panel, EDT, remote/web hosts, multi-root fan-out, concurrent requests, workspace
watching/reload, runtime download/install, Marketplace publication, telemetry,
authentication, and changes to accepted semantic or MCP server behavior.

## Acceptance Criteria

- ADR-0052 records confirmed evidence, canonical decisions, rejected
  alternatives, exact owners and state transitions, bounds, sensitive-data
  rules, compatibility, dependencies, packaging, tests, prerequisites, and
  deferred scope.
- The extension remains an MCP client and does not duplicate domain authority.
- No unapproved production dependency is selected.
- Every implementation task has an executable oracle.

## Task-specific Validation

- Validate all links, source pins, accepted/deferred consistency, and ADR index
  conventions.
- Run `git diff --check`.

## Suggested commit message

`Define Sprint 30 VS Code extension foundation`

## Final report additions

Report the accepted host/toolchain/package/lifecycle contracts, rejected
alternatives, dependencies, and implementation gates.
