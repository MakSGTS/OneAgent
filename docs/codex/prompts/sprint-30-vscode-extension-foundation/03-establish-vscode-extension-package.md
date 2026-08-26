# Establish Sprint 30 VS Code Extension Package

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
- `docs/codex/workflows/ide-extension.md`

## Prerequisites / Required gate

Task 2 and accepted ADR-0052 are committed. Exact tool versions and dependency
policy are fixed. Stop before installation if a production dependency lacks
explicit approval.

## Task

Establish the accepted reproducible package under `extensions/vscode/`.

## Scope

### Included

Locked package-manager manifest, pinned development toolchain, TypeScript build
configuration, extension manifest, license/readme/changelog as required for a
valid VSIX, package-exclusion policy, minimal public activation/deactivation
entry point, accepted command/configuration contributions without Runtime
connection behavior, pure unit evidence, deterministic build output, and exact
VSIX inventory checks.

### Excluded

MCP client, child process, connection status behavior, extension-host lifecycle
integration beyond minimal activation ownership, CI changes, navigation, LSP,
diagnostics, chat, EDT, publication, and Runtime changes.

## Acceptance Criteria

- Clean install, typecheck, build, unit tests, and VSIX packaging are
  reproducible from the lockfile.
- Manifest identity, engine, host, activation, entry point, commands, and
  configuration match ADR-0052.
- Activation owns all registrations and deactivation leaves no resource.
- The VSIX contains only accepted runtime/user documentation files and excludes
  sources, tests, caches, secrets, local workspaces, and unrelated repository
  content.

## Task-specific Validation

- Run the exact non-zero install/typecheck/build/unit/package commands accepted
  by ADR-0052.
- Inspect and compare the complete VSIX inventory.
- Run `git diff --check`.

## Suggested commit message

`Establish Sprint 30 VS Code extension package`

## Final report additions

Report exact tool versions, dependency classes, build outputs, test counts, and
packaged file inventory.
