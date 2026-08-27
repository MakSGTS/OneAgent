# Complete Sprint 33 Chat and Context Evidence

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
- `docs/adr/0044-context-engine.md`
- `docs/adr/0050-mcp-server.md`
- `docs/adr/0051-mcp-semantic-tools.md`
- `docs/adr/0052-vscode-extension-foundation.md`
- `docs/adr/0053-navigation-symbol-search.md`
- `docs/Architecture.md`
- `docs/architecture/semantic-model-2.md`
- `docs/Roadmap.md`
- `extensions/vscode/README.md`
- `.github/workflows/ci.yml`

## Prerequisites / Required gate

Task 6 is committed and every focused decoder, controller, panel, chat,
integration, process, Host, and lifecycle check passes.

## Task

Complete package, cross-platform CI, compatibility, scope, security, and
current-state evidence for the accepted Sprint 33 boundary.

## Scope

### Included

Frozen offline install; clean typecheck/build; complete unit, Runtime-process,
and pinned Extension Host matrices; exact package and two-build VSIX inventory;
macOS/Windows CI; API/manifest/registration/catalog agreement; Context/model
input and rendering security audits; dependency/license/lockfile/generated-
artifact/secret/path/deferred-scope audits; Rust Context/MCP/Workspace and full
workspace compatibility; and synchronized Architecture, semantic model,
Roadmap, extension README/changelog, audit, package, CI, and provenance docs.

### Excluded

New production behavior, Runtime LLM providers, source reads, implicit context,
model tools/edits, webview scripts, persistence, remote/web/multi-root/EDT or
diagnostics UI, Marketplace publication/signing, telemetry, and broad quality,
performance, or security claims.

## Acceptance Criteria

- Every ADR-0055 claim has non-zero public evidence and declared cross-platform
  CI coverage appropriate to the supported extension boundary.
- API, manifest, Runtime catalog, panel/chat behavior, tests, docs, package, and
  generated-artifact inventories agree exactly.
- Existing Rust Runtime/MCP/Context, extension navigation/lifecycle, dependency,
  lockfile, and package behavior remains compatible.
- No unsupported feature, production dependency, generated artifact, secret,
  personal path, unsafe HTML/script, or hidden prompt input is tracked or claimed.

## Task-specific Validation

- Run frozen install, clean, typecheck, compile, all unit/process/Host tests,
  package list/check, two clean VSIX builds/verifications, and extension audit.
- Run focused Rust Context/MCP/Workspace tests and the canonical Rust workspace
  gate with non-zero applicable filters.
- Run source/API/manifest/catalog/dependency/license/lockfile/generated/secret/
  path/prompt/rendering/deferred/link/prompt-inventory and `git diff --check`
  audits.

## Suggested commit message

`Complete Sprint 33 chat and context evidence`

## Final report additions

Report exact commands/counts, CI platforms, package inventory, supported user
flow, prompt/rendering security, dependencies, documentation transitions,
compatibility, and preserved behavior.
