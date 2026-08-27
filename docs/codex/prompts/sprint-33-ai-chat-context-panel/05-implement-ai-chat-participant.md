# Implement Sprint 33 AI Chat Participant

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
- `docs/adr/0049-tool-execution-policy.md`
- `docs/adr/0052-vscode-extension-foundation.md`

## Prerequisites / Required gate

Task 4 is committed and selected Context state/panel evidence passes. The exact
Chat, model-message, bounds, stream, cancellation, and failure contract is fixed.

## Task

Implement the bounded selected-Context VS Code chat participant controller and
model streaming behavior accepted by ADR-0055.

## Scope

### Included

Accepted prompt/context validation; deterministic messages containing only the
explicit selected Context and current prompt; current-request selected model;
text-only response stream; output bounds; cancellation; missing-context,
missing-model/capability, model rejection, unsupported part, filtered response,
and partial-stream behavior; stable redacted user messages; no retained model
or secret; pure controller/adaptation seam; and focused tests with fake models,
streams, tokens, and response sinks.

### Excluded

Manifest/activation registration, Runtime provider adapters, model selection or
secrets, model tools/edits, implicit source/context, custom history/persistence,
prompt quality claims, telemetry, new dependencies, and final package/docs.

## Acceptance Criteria

- Model input is deterministic, bounded, inspectable, and restricted to the
  accepted selected Context plus current user prompt.
- No tool call, edit, source read, secret, arbitrary history, or provider detail
  enters the first slice.
- Streaming, bounds, cancellation, failures, repeated requests, and disposal
  have deterministic non-zero evidence without leaking raw model errors.
- Existing Context panel, Runtime client, and extension lifecycle remain intact.

## Task-specific Validation

- Run non-zero chat controller/message/stream/cancellation/failure unit tests.
- Run Context/client regression tests, typecheck, and compile.
- Audit prompt inputs, model retention, tools/edits, error redaction, and bounds.
- Run `git diff --check`.

## Suggested commit message

`Implement Sprint 33 AI chat participant`

## Final report additions

Report model ownership, exact input components, bounds, streaming/cancellation,
error behavior, exclusions, compatibility, and test counts.
