# Investigate Sprint 30 VS Code Extension Foundation

Continue OneAgent development.

## Reporting

- Prompt and repository artifacts: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/investigation.md`

## Template

`docs/codex/templates/investigation-task.md`

## Authoritative documents

- `docs/Roadmap.md`
- `docs/Architecture.md`
- `docs/adr/0002-runtime-composition-root.md`
- `docs/adr/0050-mcp-server.md`
- `docs/adr/0051-mcp-semantic-tools.md`
- `docs/reviews/sprint-29-mcp-semantic-tools.md`
- Official stable VS Code Extension API manifest, anatomy, activation,
  extension-host, testing, continuous-integration, bundling, and publishing
  documentation
- Official Microsoft VS Code extension samples and packaging-tool sources used
  only with pinned provenance

## Prerequisites / Required gate

- The committed Sprint 30 planning baseline is HEAD.
- The framework prerequisite `90695c74` is an ancestor.
- Sprint 30 is the unique `next` target.

## Investigation objective

Create
`docs/architecture/vscode-extension-foundation-investigation.md` and update
only the Sprint 30 Roadmap state needed to record Task 1 start. Produce enough
verified evidence for ADR-0052 without implementation.

## Questions to answer

- Which stable VS Code, Node.js, TypeScript, package-manager, test-runner, and
  packaging versions form one reproducible supported matrix?
- Which manifest identity, desktop workspace extension host, entry point,
  activation events, commands, configuration keys/scopes/defaults, and package
  contents are valid?
- How can an extension resolve and spawn `oneagent-mcp`, select one workspace
  root, initialize MCP, correlate sequential requests, bound frames and
  diagnostics, and stop without changing accepted Runtime behavior?
- Which connection states and user-visible status are observable without
  navigation, LSP, diagnostics, chat, or semantic duplication?
- Which pure unit, extension-host, real-process, package-inventory, repeated,
  negative, and cross-platform CI cases provide deterministic oracles?
- Which development dependencies are sufficient? If any production dependency
  is required, stop and identify the explicit approval needed.

## Evidence scope

Inspect `extensions/`, root manifests and CI, Runtime MCP binary/transport,
protocol bounds and messages, real-process tests, accepted ADRs, official
upstream sources, and available pinned local/CI runtime paths. Record confirmed
facts, candidates, rejected candidates, unresolved choices, exact source URLs
or immutable revisions, and dependency/license implications.

## Excluded

Production implementation, Node dependency installation, ADR acceptance,
navigation/search, LSP, diagnostics, chat/context UI, EDT, remote/web extension
hosts, Marketplace publication, telemetry, workspace reload, concurrent MCP,
and Runtime/protocol changes.

## Completion Criteria

- Every architecture choice required by Task 2 is decision-ready.
- Tool and platform sources are pinned or the task stops with an exact blocker.
- The test matrix covers positive, invalid, missing, incompatible, failure,
  repeated, reordered, cleanup, and package inventory cases as applicable.
- No unsupported external behavior is claimed.

## Task-specific Validation

- Verify every local path and upstream source recorded in the investigation.
- Verify the nine-file Sprint 29 prompt inventory remains unchanged.
- Run `git diff --check`.

## Suggested commit message

`Investigate Sprint 30 VS Code extension foundation`

## Final report additions

Report source provenance, toolchain candidates, repository owners, dependency
impact, remaining unknowns, and ADR readiness.
