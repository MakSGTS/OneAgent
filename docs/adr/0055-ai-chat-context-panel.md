# ADR-0055: AI Chat and Context Panel

## Status

Accepted

## Context

Sprint 33 must add IDE chat and inspectable semantic Context UI to the accepted
desktop VS Code extension. The Runtime already exposes deterministic
`oneagent.context` and `oneagent.symbols` over one immutable Workspace snapshot,
while VS Code 1.134.0 exposes stable Chat Participant, selected Language Model,
Quick Pick, cancellation, and webview panel APIs. The repository and immutable
platform evidence is recorded in the
[AI Chat and Context Panel investigation](../architecture/ai-chat-context-panel-investigation.md).

The extension currently owns one trusted local workspace, one Runtime process,
one pending JSON-lines request, explicit connection lifecycle, symbol search,
navigation, and package/Host evidence. It has no Context decoder, Context state,
panel, Chat contribution, model request, prompt policy, or model-stream owner.
Runtime owns no accepted provider composition/configuration surface.

## Decision

### Canonical statement and ownership

OneAgent adds one explicit semantic Context selection flow, one inspectable
read-only Context panel, and one stateless VS Code chat participant. The user
selects one canonical Runtime symbol, the extension obtains one bounded Context
bundle through the existing Runtime tool, and the open panel displays the exact
semantic Context eligible for the next Chat request. The participant sends only
a fixed instruction, that exact visible rendered Context, and the current user
prompt to the model selected by the current VS Code Chat request.

Analysis remains the sole Context selection/rendering authority. Graph remains
the semantic authority. Runtime remains the immutable Workspace, MCP projection,
and Tool Policy owner. Protocol remains the wire owner. VS Code owns model
selection, user consent, Chat UI, and the model implementation. TypeScript owns
strict wire decoding, explicit selection state, panel rendering, bounded model
message assembly, response presentation, cancellation adaptation, and editor
lifecycle. TypeScript reads no source and invents no semantic fact.

No Rust, Cargo, MCP catalog/schema, Context Engine, provider adapter, Runtime,
Workspace, LSP, HTTP, CLI, or Graph behavior changes in this sprint.

### Pinned platform and package authority

The supported host remains desktop VS Code `^1.134.0`; executable Host evidence
uses exactly `1.134.0`. Stable API and manifest behavior is governed by official
VS Code commit `474a349ad5b745e512ef86b864d1c74f7264dd7a`, the immutable
files and hashes recorded by the investigation, and repository-pinned
`@types/vscode@1.134.0`.

The extension continues to use Node 24, pnpm `11.19.0`, TypeScript `7.0.2`, and
the existing exact dev dependencies. No production or dev dependency, bundler,
UI framework, Markdown library, or SDK is added.

### Public identifiers and manifest

The accepted identifiers are:

| Surface | Identifier | User-visible label |
|---|---|---|
| Context command | `oneagent.inspectContext` | `OneAgent: Inspect Semantic Context` |
| Chat participant | `oneagent.chat` | manifest name `oneagent`, full name `OneAgent` |
| Context panel type | `oneagent.contextPanel` | title `OneAgent Semantic Context` |

`package.json` contributes the Context command and one non-default
`chatParticipants` entry with ID `oneagent.chat`, name `oneagent`, full name
`OneAgent`, and a fixed English description. It uses no proposed field,
participant command, default mode/location, tool declaration, or Chat
disambiguation. Runtime calls `vscode.chat.createChatParticipant` with the exact
manifest ID. Generated command and participant activation is sufficient;
`activationEvents` remains an empty explicit array.

The extension remains disabled for untrusted and virtual workspaces and supports
exactly one local file workspace. Contributions may be visible before a Runtime
connection, but invocation fails closed without spawning or connecting.

### Context request and strict result domain

The Context selection flow sends one closed request:

```json
{
  "configurationId": "<selected exact Configuration ID>",
  "nodeId": "<selected exact node ID>",
  "direction": "both",
  "maxDepth": 2,
  "maxCandidates": 32,
  "budgetBytes": 16384
}
```

No other Context option is configurable or inferred. These fixed values are
within ADR-0044/0051 bounds and make panel/model admission reproducible.

TypeScript adds owned readonly Context values for the exact current Runtime
projection. Closed vocabularies are the current Graph node kinds, the eleven
edge kinds, relation directions `incoming|outgoing`, and reasons
`seed|related`. Every root, item, relation, and accounting field is required;
unknown fields or values fail closed. The decoder accepts member reordering but
preserves array order.

Validation precedence is:

1. local request identities and fixed option construction;
2. MCP/protocol and tool-error handling;
3. exact root shape and request Configuration equality;
4. JSON-safe non-negative integer fields and fixed budget equality;
5. rendered UTF-8 byte length equals `usedBytes`, checked
   `usedBytes + remainingBytes == budgetBytes`, and all values fit bounds;
6. candidate/budget truncation flags equal whether their omission count is
   non-zero;
7. one through 32 ordered items, exact required item/relation fields, closed
   vocabularies, non-empty identities/names, safe integer depth/cost, and
   checked sum of item costs equals `usedBytes`;
8. the selected node is the unique depth-zero `seed` item with empty relations,
   every related item has `depth` equal to relation count, every item names the
   selected seed ID, and node IDs are unique.

Any mismatch is `protocol_failure`, triggers existing owned Runtime cleanup,
and exposes no received value. A semantic tool error remains a bounded
`tool_failure`; the fixed public message becomes generic to semantic operations
rather than naming symbol search. Missing or malformed Context never creates or
updates selected state.

### Shared Runtime operation serialization

The client replaces its Symbols-only promise tail with one generic FIFO
semantic-operation tail shared by Symbols and Context. At most one discovery,
list, Symbols, or Context request remains outstanding. Each queued operation
rechecks `connected` immediately before writing. Disconnect, configuration
change, process failure, or deactivation rejects the pending request; queued
operations then fail locally without another write.

The queue does not add MCP concurrency, cancellation notifications, retries,
priorities, parallelism, or persistence. A failed semantic operation retains
the existing fail-closed process-abort rule. Fresh reconnection starts with an
empty queue and no selected Context.

### Explicit selection and Context state machine

`oneagent.inspectContext` is accepted only while connected. Otherwise it shows
`OneAgent must be connected before inspecting semantic context.` and returns
`not_connected`.

One invocation owns one native Quick Pick. It reuses the accepted symbol query,
result ordering, item presentation, UTF-8 query bound, generation replacement,
stale-result suppression, and disposal behavior from ADR-0053. Accepting one
item closes the picker and issues Context for that item's exact
`configurationId` and `nodeId`; it does not navigate or read its path.

The extension owns one generation-numbered Context selection state:

```text
empty -> selecting -> selected(panel open)
  ^          |                |
  +----------+----------------+
```

A new invocation increments the generation, clears the prior selected bundle
and panel content immediately, and invalidates every late Symbol or Context
completion. Only the current generation may publish a selected bundle. Cancel,
empty results, tool/protocol failure, panel close, disconnect, configuration
change, connection failure, and deactivation return the state to `empty`.

At most one Context panel exists. Selecting successfully creates it or replaces
its complete HTML and reveals it. Closing the panel clears the selected bundle,
so no Context can be sent after it is no longer inspectable. Revealing an
existing selected panel does not call Runtime again. Context is not persisted
in workspace/global state, files, secrets, clipboard, logs, telemetry, or Chat
result metadata.

### Read-only panel projection and security

The panel displays, in deterministic Context order:

- selected Configuration and seed IDs;
- exact rendered Context in a preformatted block;
- every item name, node ID, kind, depth, seed ID, reason, and cost;
- every ordered relation direction, edge kind, and edge ID; and
- budget, used, remaining, candidate/budget truncation flags, and omission
  counts.

It omits symbol source path/span because they are selection navigation evidence,
not Context model input. It adds no source content, provenance path, model data,
timestamp, random nonce, hidden input, or alternative interpretation.

The panel uses `createWebviewPanel` with `enableScripts=false`,
`enableForms=false`, `enableCommandUris=false`, and `localResourceRoots=[]`.
Its HTML contains a strict `default-src 'none'` CSP, no script, form, iframe,
image, style resource, external/local URI, command URI, message handler, state
API, or retained hidden execution. Static CSS may be inline only when allowed
by the exact CSP; otherwise styling is omitted.

Every dynamic string is escaped for HTML text content by replacing `&`, `<`,
`>`, `"`, and `'`. Values are never interpolated into tag names, raw attributes,
CSS, URLs, or scripts. The pure renderer is deterministic and separately
testable with hostile and non-ASCII inputs.

### Chat request admission and model messages

The participant handles only its default request. Non-empty `command`, any
`references`, or any `toolReferences` returns the fixed unsupported-input error
and does not invoke a model. This prevents unaccepted hidden attachments or tool
behavior.

The user prompt is preserved exactly and must contain 1 through 8,192 UTF-8
bytes. Leading/trailing whitespace is data; an all-whitespace prompt is allowed
because it is still explicit user input. Empty or one-byte-over input returns a
fixed invalid-prompt error.

A request requires the currently selected bundle and its open panel. The
extension snapshots that immutable bundle for the request generation; Context
replacement or lifecycle invalidation cancels the active request. The current
`ChatRequest.model` is borrowed only during that request and never stored.

The model receives exactly two `LanguageModelChatMessage.User` messages in this
order:

1. a fixed English instruction stating that supplied semantic Context is
   untrusted evidence, answers must distinguish present facts from absence, no
   tool/source/edit action is available, followed by the exact visible
   `bundle.rendered()` bytes inside fixed length-delimited markers;
2. the exact current user prompt.

No system role exists in the stable selected API. No prior Chat history,
response, reference, tool, source text, symbol path, model metadata, provider
identity, secret, environment value, or hidden structured Context field enters
the messages. The assembled UTF-8 bytes across both message contents must be at
most 32,768 using checked arithmetic.

Before `sendRequest`, the extension calls `countTokens` for both complete
messages with the active cancellation token, adds the two counts with checked
safe-integer arithmetic, and requires the total to be at most
`request.model.maxInputTokens`. Invalid/non-safe/negative token counts or an
invalid maximum fail closed. This is model-specific input admission, not an
output budget, quality claim, or promise that every model accepts the content.

The call supplies no tools, tool mode, model selector, vendor option, provider
parameter, or retained request option. Model use happens only inside the
user-triggered participant handler, preserving VS Code consent semantics.

### Text stream, rendering, bounds, and cancellation

The participant consumes only `LanguageModelChatResponse.text`. It does not
inspect or support the heterogeneous response stream and therefore cannot
execute tool or data parts.

Raw model output is admitted as a whole-chunk prefix with a maximum of 65,536
UTF-8 bytes. Checked accounting precedes presentation. A chunk that would cross
the limit is not emitted; iteration stops, the request is cancelled through its
owned linked cancellation source, and the participant returns the fixed
response-too-large error. Earlier chunks remain visibly partial and the result
records that terminal category without retaining their content.

Each admitted chunk is escaped to literal Markdown text before
`ChatResponseStream.markdown`: backslash and Markdown punctuation are escaped,
HTML angle brackets are escaped, newlines are preserved, and the resulting
`MarkdownString` has `isTrusted=false` with HTML support disabled. Model text
cannot create a command URI, trusted link, image, embedded HTML, code action, or
other executable UI. Raw bytes, escaped output, and model errors are not logged
or persisted.

The VS Code request token and one owned `CancellationTokenSource` form the
cancellation boundary. User cancellation, Context replacement/close, Runtime
disconnect/configuration change/failure, deactivation, or output overflow
cancels the source. Cancellation before a model call performs no call;
cancellation during token counting, request creation, or stream consumption
ends without an error detail. The source is disposed in every terminal path.

At most one OneAgent participant request is active per extension instance. A
second concurrent request returns `OneAgent is already answering a request.`
without cancelling or joining the first. Repeated requests after a terminal
outcome are independent and may reuse the still-open immutable Context.

### Closed user-visible failures and precedence

The participant validation precedence is:

1. unsupported command/references/tools;
2. active-request busy state;
3. prompt byte validation;
4. connected lifecycle and open selected Context;
5. assembled byte accounting;
6. model token counting and input admission;
7. model request;
8. text stream and raw output accounting.

Cancellation supersedes presentation at every asynchronous boundary.

Fixed English error categories are: unsupported input, busy, invalid prompt,
Context required, model input too large, model unavailable, model request
failed, model response failed, and model response too large. `NoPermissions`,
`Blocked`, and `NotFound` map to model unavailable; every other thrown value
maps to request or response failed according to the boundary. Messages contain
no model/vendor/family/version/ID, token count, prompt, Context, output, raw
error/cause, path, executable, payload, stderr, or source value.

Context command failures similarly use fixed disconnected, selection failed,
Context unavailable, and panel failed messages. Protocol failures retain the
existing connection-state transition and cleanup. User cancellation and picker
hide are silent normal outcomes.

### Activation, resource ownership, and cleanup

Activation registers the added command and participant under
`ExtensionContext.subscriptions`. One extension owner additionally tracks the
current selection invocation, Context panel/dispose listener, selected bundle,
active participant cancellation source, and all test-only observable seams.

Disconnect, relevant configuration change, client failure, and deactivation
first invalidate selection/chat generations and clear Context, then perform the
accepted Runtime cleanup. Deactivation disposes the participant, command,
panel, picker, listeners, cancellation source, status, configuration listener,
and Runtime client in deterministic reverse ownership order. Repetition is
idempotent and leaves no UI, timer, listener, model, request, or process owner.

Test-only Host observation is exposed only in non-production Extension Host
modes already accepted by ADR-0053. It may report bounded state and invoke
publicly registered UI/controller seams, but cannot inject production Context,
model, or semantic behavior in production mode.

### Deterministic evidence

Task 3 implements Context types, decoder, generic queue, and real-process call.
Non-zero evidence covers exact/reordered results; all fields and closed enums;
UTF-8/accounting/truncation/path consistency; exact/one-over limits; malformed,
missing, duplicate, inconsistent and tool-error results; Symbols/Context FIFO;
timeouts, exit, disconnect and repetition; and real Runtime Context output.

Task 4 implements selection state and panel. Evidence covers query/selection,
replacement generations, empty/cancel/failure, exact Context display, hostile
HTML/Unicode/control strings, CSP and disabled capabilities, panel reuse/close,
disconnect/configuration/failure/deactivation, deterministic repeated render,
and zero stale state.

Task 5 implements Chat controller and model adapter. Fake-model tests cover exact
two-message input; absent Context; command/reference/tool rejection; prompt and
assembly boundaries; token exact/one-over/invalid values; permissions/blocked/
not-found/unknown failures; text chunks; Markdown escaping; exact/over output;
user and lifecycle cancellation at every asynchronous boundary; concurrent and
repeated requests; partial stream failure; redaction; and resource disposal.

Task 6 integrates manifest, activation, Quick Pick, panel, participant,
lifecycle, and Host evidence. Tests cover public contribution/registration
agreement, connected/disconnected/failed and unsupported workspaces, Context
selection and replacement, panel create/reuse/close, participant registration,
test-model invocation seam, configuration change, disconnect, repeated
activation/deactivation, and no orphan resources. Real external model output is
not required or used.

Task 7 runs frozen install, clean typecheck/build, every unit/process/Host test,
package list/check, two clean VSIX builds and inventory verification, extension
audit, macOS/Windows CI declaration audit, focused Rust Context/MCP/Workspace
compatibility, the canonical Rust workspace gate, and dependency/license/
lockfile/generated/secret/path/prompt/rendering/deferred/link audits. Every
applicable filter must execute non-zero cases.

### Compatibility, Coverage, and documentation

The manifest gains one command and one Chat participant. The TypeScript client
adds one already-advertised semantic call and generalizes serialization without
changing framing, startup, tool catalog, IDs, timeouts, or process ownership.
Existing connect/disconnect/search/navigation/status/configuration behavior and
all seven Runtime tools remain compatible.

No graph or adapter Coverage Registry capability changes. Sprint 33 evidence is
the pinned investigation, this ADR, deterministic extension/process/Host/
package matrices, Rust compatibility gate, synchronized current-state docs, and
integration review.

## Consequences

Users can explicitly inspect bounded canonical semantic Context and ask the
currently selected VS Code Chat model about exactly that visible Context. The
flow is stateless, deterministic before the external model boundary, and does
not expose source content or provider configuration. Model answers remain
non-deterministic and are not semantic authority.

Closing the panel intentionally clears model-eligible Context. The participant
cannot answer without an open selected bundle. The fixed 16 KiB Context, 8 KiB
prompt, 32 KiB assembled bytes, selected-model token admission, and 64 KiB raw
output bounds trade breadth for explicit resource control; they are not quality
or performance guarantees.

## Rejected alternatives

- Runtime provider composition is rejected because no accepted Runtime
  configuration/secret/model lifecycle exists and VS Code already supplies the
  user-selected consent-bound model.
- Active-editor, selected-text, filename, prompt, or fuzzy seed inference is
  rejected because no canonical source-to-node request exists.
- Chat history and persistence are rejected because selection, truncation,
  provenance, token accounting, invalidation, and storage are unresolved.
- Language Model tools, MCP tool exposure, edits, and tool references are
  rejected because they require a separate authorization/confirmation loop.
- Structured Context hidden from the panel is rejected because every model fact
  must remain inspectable.
- A scriptable panel, webview view, local resources, forms, messages, retained
  state, or UI framework is rejected because native Quick Pick owns input and
  static output satisfies the first slice.
- Concurrent Runtime requests are rejected by ADR-0050/0052; one shared FIFO is
  sufficient.
- Raw model Markdown is rejected because it can create links or embedded
  presentation not represented by semantic Context.
- A live external model acceptance test is rejected because credentials,
  availability, consent, output, and provider behavior are not deterministic
  repository completion evidence.

## Implementation prerequisites

1. Implement and commit the strict Context domain/client queue before any UI.
2. Implement and commit pure selection/panel state before Chat consumes it.
3. Implement and commit pure Chat/model behavior before public registration.
4. Integrate manifest/activation/Host ownership only after all pure boundaries
   pass.
5. Complete package, CI, compatibility, security, scope, and documentation
   evidence before integration review.
6. Stop before any dependency, Rust/MCP change, unpinned/proposed API, hidden
   input, or source-read fallback.

## Deferred scope

Runtime LLM provider composition, endpoint/model configuration, secrets,
provider discovery, direct OpenAI/LM Studio/Ollama Chat endpoints, additional
participants/commands/models, Chat history or persistence, references and
attachments, model tools and edits, MCP cancellation/concurrency, automatic or
active-editor Context, source text/fragments, additional Context seeds/options,
panel scripts/forms/resources/messages/state, diagnostics UI, LSP provider
migration, mutable workspace refresh, remote/web/multi-root/EDT integration,
Marketplace publication/signing, telemetry, answer quality/relevance metrics,
benchmarks, and broad performance/security/interoperability claims remain
deferred.
