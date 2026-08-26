# Integrate Sprint 30 VS Code Runtime Lifecycle

Continue OneAgent development.

## Reporting

- Prompt and repository artifacts: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/ide-extension-implementation.md`

## Template

`docs/codex/templates/ide-extension-task.md`

## Authoritative documents

- `docs/adr/0052-vscode-extension-foundation.md`
- `docs/architecture/vscode-extension-foundation-investigation.md`
- `docs/adr/0050-mcp-server.md`
- `docs/adr/0051-mcp-semantic-tools.md`

## Prerequisites / Required gate

Task 4 is committed and the client/package validation gates pass.

## Task

Integrate the MCP client into the public VS Code extension lifecycle.

## Scope

### Included

Accepted workspace-folder selection; bounded workspace-scoped configuration
reads and validation; explicit connect/disconnect commands and activation
ownership; one context-owned client; deterministic connecting/connected/
disconnecting/disconnected/failed status UI; command enablement/context state if
accepted; configuration replacement; startup/protocol/exit failures; user-
initiated reconnect only when accepted; deactivation shutdown; disposables; pure
unit and pinned extension-host tests.

### Excluded

Navigation/search, LSP, diagnostics engine/UI, chat/context panel, EDT,
background eager connection, multi-root fan-out, automatic binary discovery or
download beyond ADR-0052, watcher/reload, concurrent calls, Marketplace
publication, telemetry, and semantic changes.

## Acceptance Criteria

- Activation is demand-driven and every command, status object, listener,
  configuration subscription, client, and pending operation has one owner.
- Invalid/missing workspace or configuration produces the accepted bounded
  status and no child process.
- Connect/disconnect, replacement, failure, repeated activation, and
  deactivation have exact states and leave no orphan or duplicate registration.
- Pinned extension-host tests exercise public activation, contributions,
  configuration, commands, state, and cleanup with non-zero test counts.

## Task-specific Validation

- Run non-zero unit and pinned extension-host tests.
- Run the complete client real-process and package gates.
- Inspect manifest contribution/implementation agreement and packaged inventory.
- Run `git diff --check`.

## Suggested commit message

`Integrate Sprint 30 VS Code runtime lifecycle`

## Final report additions

Report activation triggers, configuration contract, UI states, resource
ownership, failure/replacement behavior, extension-host counts, and cleanup.
