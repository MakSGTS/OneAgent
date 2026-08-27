# Integrate Sprint 33 Chat and Context Extension

Continue OneAgent development.

## Reporting

- Prompt and repository artifacts: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/ide-extension-implementation.md`

## Template

`docs/codex/templates/ide-extension-task.md`

## Authoritative documents

- `docs/adr/0055-ai-chat-context-panel.md`
- `docs/architecture/ai-chat-context-panel-investigation.md`
- `docs/adr/0050-mcp-server.md`
- `docs/adr/0051-mcp-semantic-tools.md`
- `docs/adr/0052-vscode-extension-foundation.md`
- `docs/adr/0053-navigation-symbol-search.md`

## Prerequisites / Required gate

Task 5 is committed and all Context-panel and chat-controller evidence passes.

## Task

Integrate the accepted Context command/panel and chat participant through the
public VS Code manifest, activation, Runtime lifecycle, and Extension Host.

## Scope

### Included

Exact manifest command and chat-participant contributions; stable public API
registration; explicit context/panel/chat/disposable ownership; trusted local
single-workspace and connected-state gates; selection Quick Pick; panel create/
reuse/close; participant handler adaptation; disconnect/configuration-change/
failed-connect/deactivation invalidation and cleanup; test-only observable Host
seams; manifest/audit/package inventory updates; unit, real-process, and pinned
Extension Host tests; and existing command/lifecycle compatibility.

### Excluded

New semantic or protocol behavior, Runtime provider wiring, source reads,
remote/web/multi-root/EDT support, diagnostics UI, model tools/edits, persistence,
Marketplace publication, telemetry, new dependencies, and final documentation.

## Acceptance Criteria

- Manifest contributions and runtime registrations agree exactly and use only
  stable pinned APIs.
- All commands, participants, panels, Quick Picks, listeners, pending work, and
  model activity are owned, invalidated, and disposed across every lifecycle.
- Public Host evidence covers positive, disconnected, failed, unsupported,
  repeated, replacement, cancellation, configuration-change, and deactivation
  behavior without timing-dependent sleeps.
- Existing connect/disconnect/search, seven-tool compatibility, packaging, and
  unsupported-workspace behavior remain unchanged.

## Task-specific Validation

- Run complete non-zero extension unit and real Runtime-process tests.
- Run the pinned Extension Host matrix, including repeated activation and all
  unsupported-workspace cases.
- Run manifest/audit, typecheck, clean compile, package inventory, and
  generated-artifact checks.
- Run `git diff --check`.

## Suggested commit message

`Integrate Sprint 33 chat and context extension`

## Final report additions

Report manifest/API registrations, activation/disposal ownership, Host and
process cases, compatibility, package impact, and exact test counts.
