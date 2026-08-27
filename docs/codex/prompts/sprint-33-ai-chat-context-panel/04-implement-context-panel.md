# Implement Sprint 33 Context Panel

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
- `docs/adr/0051-mcp-semantic-tools.md`
- `docs/adr/0052-vscode-extension-foundation.md`
- `docs/adr/0053-navigation-symbol-search.md`

## Prerequisites / Required gate

Task 3 is committed and the strict Context client/process matrix passes.

## Task

Implement bounded semantic Context selection state and the accepted read-only
Context panel presentation.

## Scope

### Included

Accepted explicit seed-selection flow over canonical Symbols results; Context
loading/replacement generations; immutable selected bundle state; deterministic
panel view model; complete visible provenance/relation/accounting/truncation
facts required by ADR-0055; HTML escaping; script-free CSP/content; panel reuse,
reveal, replacement, close, disconnect, and disposal behavior; pure controller
and renderer APIs; and focused unit tests.

### Excluded

Chat participant or model request, hidden/automatic context, source reads,
semantic inference, webview scripts/messages, persistence, Runtime/MCP changes,
manifest/activation integration, new dependencies, and final docs/package work.

## Acceptance Criteria

- A user-selected canonical node produces exactly one bounded inspectable
  Context bundle, and replacement cannot present stale results.
- Every untrusted string is escaped, no script executes, and the panel exposes
  only decoded canonical fields with exact truncation/accounting state.
- Failure, cancellation, disconnect, close, and deactivation leave no stale
  selected context or owned resource.
- Unit tests cover positive, empty, malformed-state, injection, boundary,
  replacement, reordered, repeated, failure, and disposal cases.

## Task-specific Validation

- Run non-zero Context selection/controller/rendering/security unit tests.
- Run existing symbol/client regression tests, typecheck, and compile.
- Audit script/CSP/escaping and generated-artifact scope.
- Run `git diff --check`.

## Suggested commit message

`Implement Sprint 33 context panel`

## Final report additions

Report selection/state behavior, visible fields, rendering/CSP security,
lifecycle/disposal, compatibility, and exact test counts.
