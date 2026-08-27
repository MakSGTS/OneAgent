# Complete Sprint 32 LSP Evidence

Continue OneAgent development.

## Reporting

- Prompt and repository artifacts: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/runtime-service-implementation.md`

## Template

`docs/codex/templates/runtime-service-task.md`

## Authoritative documents

- `docs/adr/0054-lsp-adapter.md`
- `docs/architecture/lsp-adapter-investigation.md`
- `docs/adr/0039-workspace-service.md`
- `docs/adr/0050-mcp-server.md`
- `docs/adr/0051-mcp-semantic-tools.md`
- `docs/adr/0052-vscode-extension-foundation.md`
- `docs/adr/0053-navigation-symbol-search.md`
- `docs/Architecture.md`
- `docs/architecture/semantic-model-2.md`
- `docs/Roadmap.md`
- `.github/workflows/ci.yml`

## Prerequisites / Required gate

Task 6 is committed and every focused protocol, lifecycle, semantic, diagnostic,
public-process, and compatibility check passes.

## Task

Complete public, cross-platform, compatibility, dependency, scope, and current-
state evidence for the accepted Sprint 32 boundary.

## Scope

### Included

Complete clean Rust and public `oneagent-lsp` matrices; exact lifecycle,
framing, capability/method, URI/position, symbol/navigation, diagnostic,
malformed/negative/reordered/repeated/cleanup evidence; macOS/Windows CI
integration; public binary inventory; API/capability/handler/dependency/lockfile/
generated-artifact/secret/path-leak/deferred-scope/link audits; coexistence with
MCP/HTTP/CLI/VS Code; and synchronized `docs/Architecture.md`, semantic model,
Roadmap, README/current-state documentation, and fixture provenance if changed.

### Excluded

New production behavior, mutable document synchronization, unsupported LSP
methods, IDE UI/provider migration, external-client compatibility, remote
transports, diagnostics rules/UI, edits/refactoring, telemetry, and broad
performance/security claims.

## Acceptance Criteria

- Every ADR-0054 claim has non-zero public evidence and CI coverage appropriate
  to the supported platforms.
- Capabilities, handlers, process behavior, tests, docs, binaries, and current-
  state inventories agree exactly.
- Existing MCP, HTTP, CLI, Graph, Workspace, adapters, extension, dependencies,
  lockfiles, and package behavior remain compatible.
- No unsupported capability, production dependency, generated artifact, secret,
  or absolute-path disclosure is claimed or tracked.

## Task-specific Validation

- Run the complete canonical Rust workspace gate and every public LSP process
  test with non-zero counts.
- Run protocol/capability/handler, dependency/lockfile, binary/CI, generated-
  artifact, secret/path-leak, deferred-scope, Markdown-link, prompt inventory,
  and `git diff --check` audits.

## Suggested commit message

`Complete Sprint 32 LSP evidence`

## Final report additions

Report exact commands/counts, CI platforms, binary and capability inventory,
supported method/diagnostic matrix, dependency/exclusion audits, documentation
transitions, compatibility evidence, and preserved behavior.
