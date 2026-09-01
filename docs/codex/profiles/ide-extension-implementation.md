# IDE Extension Implementation Profile

## Purpose

Use this profile for implementing one accepted editor-extension build,
packaging, activation, configuration, Runtime-connectivity, UI-state, or
extension-host lifecycle slice.

## Required Core modules

- `docs/codex/core/repository-safety.md`
- `docs/codex/core/repository-investigation.md`
- `docs/codex/core/context-management.md`
- `docs/codex/core/change-contract.md`
- `docs/codex/core/validation.md`
- `docs/codex/core/final-report.md`

## Required Workflow modules

- `docs/codex/workflows/implementation.md`
- `docs/codex/workflows/ide-extension.md`
- `docs/codex/workflows/runtime-service.md` when child-process ownership,
  connection lifecycle, cancellation, shutdown, or public Runtime behavior
  changes
- `docs/codex/workflows/mcp-protocol.md` when the extension implements or
  changes MCP wire behavior or compatibility claims

## Task-family expectations

- Pin authoritative editor, manifest, toolchain, packaging, and test contracts
  before implementation.
- Keep editor API adaptation, extension lifecycle, Runtime transport, protocol,
  and domain semantics in their accepted ownership layers.
- Prove build, activation, configuration, connectivity, failure, cleanup, and
  packaging claims through the narrowest applicable public boundaries.
- Make child processes, streams, listeners, commands, UI resources, and pending
  work explicitly owned and disposable.
- Do not combine unresolved editor or connectivity architecture with
  implementation; use a preceding investigation or architecture task.
