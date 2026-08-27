# Investigate Sprint 33 AI Chat and Context Panel

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
- `docs/architecture/semantic-model-2.md`
- `docs/adr/0044-context-engine.md`
- `docs/adr/0049-tool-execution-policy.md`
- `docs/adr/0050-mcp-server.md`
- `docs/adr/0051-mcp-semantic-tools.md`
- `docs/adr/0052-vscode-extension-foundation.md`
- `docs/adr/0053-navigation-symbol-search.md`
- `docs/reviews/sprint-32-lsp-adapter.md`
- pinned VS Code 1.134.0 API, manifest, Chat, Language Model, webview, and test-runtime sources

## Prerequisites / Required gate

- The committed Sprint 33 planning baseline is HEAD.
- Sprint 32 is completed and Sprint 33 is the unique eligible target.
- The existing Context and Symbols tools, Runtime process fixture, pinned types,
  and Extension Host remain discoverable.

## Investigation objective

Create `docs/architecture/ai-chat-context-panel-investigation.md` and update
only the Sprint 33 Roadmap state needed to record Task 1 start. Produce
decision-ready evidence for ADR-0055 without production implementation.

## Questions to answer

- Which immutable VS Code 1.134.0 sources govern chat-participant contribution,
  `ChatRequest`, selected language models, message/response streaming,
  cancellation, errors, webview panels, HTML/CSP, activation, and disposal?
- What bounded user flow selects one canonical semantic node, invokes existing
  `oneagent.context`, displays every sent fact, and makes that exact bundle
  available to one chat request without implicit source reads or hidden context?
- Which prompt, context, result, stream, history, concurrency, cancellation,
  reconnection, invalidation, escaping, and failure limits are safe and testable?
- Which behavior belongs to pure TypeScript controllers, VS Code adaptation,
  the MCP client, Runtime, Context Engine, and the selected VS Code model?
- Can the slice use only current pinned dev dependencies and public APIs?

## Evidence scope

Inspect Context Engine and MCP Context contracts/tests, Symbols selection,
Runtime client framing and single-request ownership, extension lifecycle,
manifest/audit/package rules, current unit/process/Host fixtures, CI, accepted
ADRs, local pinned types/runtime, and immutable official sources. Record exact
fields, bounds, ordering, messages, permissions, lifecycle, security,
compatibility, dependency impact, and executable positive/negative oracles.

## Excluded

Production code, ADR acceptance, new Rust capability, Runtime provider wiring,
provider secrets, source reads, implicit editor-to-node inference, model tools
or edits, webview scripts, persistence, remote/web/multi-root/EDT integration,
Marketplace publication, telemetry, and broad quality/performance/security
claims.

## Completion Criteria

- Every architecture choice required by Task 2 is decision-ready.
- The smallest truthful Chat/Context slice has exact public and negative oracles.
- Ownership, bounds, security, lifecycle, failure, dependency, package, and
  compatibility behavior are fully inventoried.
- No production dependency is required, or execution stops for approval.

## Task-specific Validation

- Verify every local path and pinned official source recorded.
- Run selected non-zero Context/MCP, TypeScript client, process, Host, manifest,
  and package baselines.
- Verify the exact nine-file Sprint 32 prompt inventory.
- Run `git diff --check`.

## Suggested commit message

`Investigate Sprint 33 AI chat and context panel`

## Final report additions

Report API authority, user-flow candidates, ownership/security map, bounds and
failure matrix, dependency impact, exact baseline checks, unknowns, and ADR
readiness.
