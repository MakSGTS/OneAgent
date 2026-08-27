# AI Chat and Context Panel Investigation

## Status and scope

This document records the live repository and pinned platform evidence used to
prepare ADR-0055 for Sprint 33 AI Chat and Context Panel. The investigation
baseline is committed Sprint 33 planning head `39c229aa2db4`.

The investigation covers one bounded VS Code workspace-extension slice: an
explicit semantic seed selected through the existing Runtime Symbols tool, an
inspectable read-only Context panel backed by the existing Runtime Context
tool, and one chat participant that may send only that selected Context plus
the current user prompt to the language model selected by the current VS Code
Chat request.

This document does not accept architecture and does not implement production
behavior. It separates confirmed evidence, accepted constraints, candidate
decisions for ADR-0055, rejected first-slice candidates, and unresolved items.

## Investigation baseline

- Repository: OneAgent workspace at committed HEAD `39c229aa2db4`.
- Completed prerequisite: Sprint 32 review commit `8e33c95c` with `pass with
  non-blocking follow-ups`.
- Current sprint state at Task 1 start: Sprint 33 is `active` and is the only
  eligible sprint.
- Extension engine: VS Code `^1.134.0`.
- Pinned API declarations: `@types/vscode@1.134.0`.
- Pinned TypeScript and test runtime: TypeScript `7.0.2`, Node `24.18.1`, pnpm
  `11.19.0`, and VS Code/Electron `1.134.0`.
- Immutable official VS Code tag commit:
  `474a349ad5b745e512ef86b864d1c74f7264dd7a`.
- Existing MCP revision: `2026-07-28`.
- Existing Runtime tool catalog: exactly seven tools, including
  `oneagent.context` and `oneagent.symbols`.
- Existing user changes in `.codex/config.toml`, `docs/Roadmap.md`,
  `docs/architecture/mcp-semantic-tools-investigation.md`, and
  `docs/reviews/sprint-29-mcp-semantic-tools.md` are unrelated and remain
  outside this task.

## Pinned platform authority

The official VS Code `1.134.0` tag resolves to commit
`474a349ad5b745e512ef86b864d1c74f7264dd7a`. The field-level sources used by
this investigation are immutable raw files at that commit:

| Authority | Immutable source | SHA-256 observed |
|---|---|---|
| Stable extension API | [vscode.d.ts](https://raw.githubusercontent.com/microsoft/vscode/474a349ad5b745e512ef86b864d1c74f7264dd7a/src/vscode-dts/vscode.d.ts) | `b13911f90d44d0bb4885cb72dbd5487e56d982095ab821c1202f244b99f560bc` |
| Chat participant manifest extension point | [chatParticipant.contribution.ts](https://raw.githubusercontent.com/microsoft/vscode/474a349ad5b745e512ef86b864d1c74f7264dd7a/src/vs/workbench/contrib/chat/browser/chatParticipant.contribution.ts) | `f77c8e0d95ff5e0b15f0ea5135f3b454ee6b12dbf07c793ae3e053ead954e47a` |
| Extension-host Chat adapter | [extHostChatAgents2.ts](https://raw.githubusercontent.com/microsoft/vscode/474a349ad5b745e512ef86b864d1c74f7264dd7a/src/vs/workbench/api/common/extHostChatAgents2.ts) | `7b01539b7175cee10d7221e5124c3739be5e99ac597d250c2ea5f7cef1b53aa4` |

The installed repository-local declaration
`extensions/vscode/node_modules/@types/vscode/index.d.ts` has observed SHA-256
`727083f9c38ee8cc815f4cb8bcb66a496e271e29fdd20119dcb342d6670be38f`.
It is generated and ignored local installation evidence, not a tracked
authority. `extensions/vscode/package.json` and `pnpm-lock.yaml` pin the
package name and version used for reproducible typechecking.

## Confirmed VS Code API and manifest facts

### Chat participant registration

- `contributes.chatParticipants` is a stable manifest extension point.
- Each participant entry requires `id` and `name`; `name` must match
  `^[\\w-]+$`. `description`, `fullName`, commands, sample request, and
  disambiguation are optional.
- A contributed participant creates an `onChatParticipant:<id>` activation
  event. The current extension's empty explicit `activationEvents` array can
  remain unchanged because generated activation is owned by the contribution.
- `vscode.chat.createChatParticipant(id, handler)` returns a disposable
  `ChatParticipant`. The manifest ID and runtime registration ID must match.
- OneAgent can contribute a non-default participant without proposed APIs.
  Default participant modes and additional locations are proposed-only and are
  not eligible for this sprint.

### Chat request and selected model

- `ChatRequest.prompt` is the current user-entered prompt; participant and
  command names are not part of it.
- `ChatRequest.references` and `toolReferences` are separate inputs. Supporting
  either would require an explicit accepted decoding and authorization
  contract. They are not required for the smallest slice.
- `ChatRequest.model` is the model currently selected by the user in Chat and
  must not be retained beyond the request lifetime.
- `LanguageModelChat.sendRequest` accepts a message array, optional request
  options, and a cancellation token. First use may require user consent and
  therefore may be invoked only from an explicit user action; the participant
  handler satisfies that event boundary.
- `LanguageModelChat.maxInputTokens` is model-specific. The model also exposes
  `countTokens`, so byte bounds alone cannot truthfully claim model admission.
  ADR-0055 must define both a deterministic repository-owned byte limit and a
  selected-model token admission check before `sendRequest`.
- `LanguageModelChatResponse.text` is a text-only async stream. The broader
  `stream` may contain text, tool calls, data, or unknown future parts. The
  smallest closed slice can consume only `.text` and pass no tools.
- A request or stream can reject. `LanguageModelError` identifies
  `NoPermissions`, `Blocked`, `NotFound`, or `Unknown`; raw causes and model
  details are not suitable user-visible diagnostics.
- Breaking from response iteration or cancelling the supplied token cancels
  consumption. The extension must own any linked cancellation source it adds.

### Response presentation

- `ChatResponseStream.markdown` accepts strings or `MarkdownString` values.
  Passing plain streamed text as Markdown would allow model-generated Markdown
  syntax but not command trust by itself. ADR-0055 must choose whether to emit
  plain text fragments through a non-trusted `MarkdownString` or escaped text.
- Progress, anchors, references, buttons, and file trees are available, but
  they add separate trust and navigation behavior. They are not required for a
  text-only first slice.
- Returning a `ChatResult.errorDetails` value gives a bounded user-visible
  failure without exposing the underlying exception.

### Webview panel boundary

- `vscode.window.createWebviewPanel` creates a disposable editor panel.
- `WebviewOptions.enableScripts` defaults to false. `enableForms` defaults to
  false when scripts are disabled. Command URIs default to false.
- `localResourceRoots` defaults to workspace and extension roots if omitted.
  The Context panel needs no resources, so an explicit empty array is the
  fail-closed candidate.
- A panel owns `webview.html`, visibility/reveal state, and a dispose event.
  Reusing one owned panel avoids duplicate Context state and additional
  serialization.
- Static HTML still treats interpolated data as markup. Every Context string
  must be escaped for text and attribute positions, and a strict
  `default-src 'none'` Content Security Policy should be present even when
  scripts and resources are disabled.

## Confirmed repository ownership

### Context authority

ADR-0044 and `oneagent-analysis` own deterministic Context selection,
provenance, reasons, paths, rendering, and UTF-8-byte budget accounting over one
immutable `SemanticGraph`. The Context Engine reads no source files, calls no
model, retains no graph, and returns an owned bundle.

The Runtime `oneagent.context` tool already accepts:

- required `configurationId` and `nodeId` strings;
- optional `direction`: `incoming`, `outgoing`, or `both`;
- optional `maxDepth` in `0..=4`;
- optional `maxCandidates` in `1..=128`;
- optional `budgetBytes` in `1..=32_768`.

Runtime defaults are `both`, depth `2`, candidates `50`, and budget `16_384`.
The result contains exact `configurationId`, `rendered`, ordered `items`,
budget/used/remaining counts, candidate and budget truncation flags, and exact
omission counts. Each item contains `nodeId`, `name`, closed graph `kind`,
`depth`, `seedId`, `reason`, ordered `relations`, and `costBytes`; each relation
contains closed direction, edge kind, and edge ID.

The Runtime maps missing seeds to `not_found`, validation and insufficient
budget outcomes to `invalid_arguments`, and internal invariant/cost failures to
`execution_failed`. It exposes only the current immutable Workspace snapshot.

### Seed selection and source navigation

The existing `oneagent.symbols` tool and TypeScript symbol decoder expose
canonical node IDs for Module, Procedure, Function, and EDT Query results.
The existing Quick Pick controller already proves bounded Unicode-lowercase
search, generation-based replacement, stale-result suppression, and disposal.
The Sprint 33 first slice can reuse that public operation as explicit seed
selection. It must not infer a graph node from active editor text, paths,
positions, opaque provenance, or arbitrary prompt text.

### Runtime client boundary

`extensions/vscode/src/mcp-client.ts` owns exactly one child process and one
pending JSON-lines request. Startup performs discovery and exact seven-tool
catalog validation. Current Symbols calls use a promise tail to serialize
repeated Symbols operations. A Context operation cannot safely add a separate
tail: Symbols and Context must share one generic semantic-operation queue or
another single-owner scheduler so cross-operation calls cannot collide.

The client already bounds frames to 1 MiB, JSON depth to 128, retained stderr
to 4 KiB, request timeout to five seconds, shutdown timeout to two seconds, and
request IDs to safe integers. Protocol, process, timeout, malformed-frame,
unexpected-exit, and cleanup behavior is reusable.

### Extension lifecycle and package

The extension supports exactly one trusted local file workspace, one Runtime
process, explicit connect/disconnect, configuration replacement without
automatic reconnect, one active Symbol Quick Pick, and deterministic
deactivation. Commands, status, configuration listener, Quick Pick, client,
streams, timers, and child process are explicitly owned.

The package has zero production dependencies. Generated `dist`, `dist-test`,
`.vscode-test`, coverage, and VSIX artifacts are ignored. The audit currently
forbids `createWebviewPanel` because Chat/Context UI was deferred; Sprint 33
must replace that historical prohibition with exact positive inventory and
security assertions rather than simply delete the check.

## Accepted compatibility constraints

The following accepted behavior must remain unchanged:

- Context Engine semantics, ordering, provenance, and rendering;
- exact seven-tool MCP catalog, JSON-lines framing, protocol revision, Tool
  Policy gating, and immutable Workspace snapshot;
- one child process and one pending request at a time;
- explicit connect/disconnect and no automatic Runtime restart;
- trusted, local, single-workspace requirement;
- existing symbol search/navigation and stale-result suppression;
- zero production Node dependencies and frozen dev toolchain;
- package/generated-artifact, secret/path redaction, and macOS/Windows CI
  policies;
- no Graph, Workspace, Runtime, provider-adapter, LSP, HTTP, or CLI behavior
  change.

## Decision-ready first-slice candidates

### User flow

The evidence supports this bounded candidate for ADR-0055:

1. A new explicit `oneagent.inspectContext` command is available only when the
   existing extension lifecycle is connected.
2. The command opens one bounded Symbols Quick Pick and requires the user to
   choose one canonical result.
3. The extension calls `oneagent.context` for that exact configuration/node,
   stores one immutable decoded bundle in memory, and opens or replaces one
   read-only Context panel.
4. The panel displays the exact selected identity, ordered items and relations,
   rendered Context, byte accounting, and truncation state. No hidden Context
   field is sent to a model.
5. The contributed `@oneagent` participant requires a currently selected
   Context bundle. If absent, it returns a stable instruction to run the
   inspect command and does not call a model.
6. A participant request performs selected-model token admission, sends only a
   fixed instruction containing the exact inspected `rendered` Context plus the
   current bounded user prompt, and streams only model text.
7. Replacing Context invalidates earlier state. Disconnect, configuration
   change, failure, panel close when accepted, and deactivation cancel active
   selection/chat work and clear the selected bundle according to one explicit
   ADR state machine.

This flow is explicit, inspectable, source-independent, and testable with the
current real Runtime and pinned Extension Host. It requires no Rust, protocol,
provider, dependency, or lockfile change.

### Candidate bounds for ADR-0055

Repository evidence supports fixed Context inputs at direction `both`, depth
`2`, at most `32` candidates, and `16_384` rendered bytes. These values fit
within existing Runtime/Context bounds and reduce the extension decoder and UI
surface. ADR-0055 must choose exact values and treat deviation as invalid.

The user prompt, assembled byte input, model-token admission, streamed output,
and queue limits still require architecture values. Candidate values can be
selected conservatively and proved with exact/one-over tests, but they must not
be described as model quality or universal context-window thresholds.

### State and concurrency candidate

One generic FIFO Runtime operation queue should serialize Symbols and Context
over the existing pending request. UI generation identifiers should suppress
stale selection or Context completion. Chat should permit at most one active
participant request owned by its handler; the VS Code cancellation token and an
owned linked source should end request/stream work. No model object, history,
or response content should survive the request.

## Rejected first-slice candidates

### Wire Runtime LLM adapters into the extension

Rejected because Runtime owns no accepted provider composition/configuration
surface, while VS Code already supplies the user-selected request model and
consent boundary. Adding provider selection, endpoints, secrets, or model
lifecycle would cross accepted Sprint 23-26 deferrals and require a separate
Runtime architecture slice.

### Infer Context from the active editor or prompt

Rejected because there is no accepted source-position-to-node request, selected
text seed, arbitrary filename seed, or prompt entity resolver. Such inference
would create TypeScript semantic authority or require a new Runtime contract.

### Let the model invoke OneAgent tools

Rejected because tool references, invocation tokens, model tool-call parts,
approval presentation, and edit/side-effect behavior require a separate
accepted tool authorization and loop contract. The existing MCP Tool Policy is
not automatically a VS Code Language Model tool policy.

### Use model or chat history

Rejected because history selection, truncation, provenance, token accounting,
and persistence are unresolved. The first slice can be stateless per current
request while VS Code continues to own Chat UI history presentation.

### Put interactive scripts in the Context panel

Rejected because selection happens through native Quick Pick and the panel is
inspectable output only. Scripts, messages, forms, local resources, persistence,
and retained hidden state add attack surface without first-slice value.

### Send structured Context fields that the panel does not show

Rejected because the sprint requires inspectable semantic Context. Model input
must be derivable byte-for-byte from visible selected state; hidden payload or
source enrichment would weaken that property.

## Testability matrix

| Boundary | Positive oracle | Negative and boundary oracle |
|---|---|---|
| Context decoder | Exact real Runtime result with all ordered items, relations, accounting, and truncation fields | Missing/extra/wrong-type fields; duplicate or unknown enum values; invalid UTF-8 accounting; non-integers; negative, unsafe, inconsistent totals; exact/one-over bounds |
| Runtime queue | Symbols and Context calls execute in FIFO order and keep `connected` | Cross-operation replacement, tool error, timeout, malformed frame, disconnect, exit, queued rejection, repeated calls |
| Selection controller | One selected Symbol produces one Context request and current immutable bundle | Empty query, cancelled pick, stale search/context completion, replacement, failure, dispose, disconnect |
| Panel renderer | Exact deterministic static HTML for reordered-equivalent decoded input | `<`, `>`, `&`, quotes, Unicode, controls, fake tags/attributes; CSP, no scripts/forms/resources/commands |
| Panel owner | Create, reveal, replace, close, reconnect, and deactivate have exact state | Duplicate panel, late update after dispose, stale bundle after lifecycle transition |
| Chat controller | Current selected Context plus current prompt creates exact messages and streams exact text | Empty/over-bound prompt, absent/stale Context, token count overflow, permissions/blocked/not-found/unknown error, rejected request, stream error, cancellation, output overflow, repeat |
| Manifest/API | Contributed ID equals runtime registration and generated activation works | Missing/mismatched contribution, proposed fields, unsupported workspace, duplicate disposal |
| Public process | Real `oneagent-mcp` returns exact Context used by TypeScript | Invalid seed, insufficient budget, malformed result, process exit, redacted startup failure |
| Extension Host | Public command selects Context, panel is created/reused, participant is registered, lifecycle cleanup is observable | disconnected/failed/untrusted/empty/virtual/multi-root, replacement, cancellation, configuration change, repeated activation/deactivation |
| Package/CI | Clean compile/tests, exact inventory, two reproducible VSIX builds, macOS/Windows declarations | generated/test/cache/secret inclusion, dependency or lockfile drift, unsupported API or platform claim |

Pure controller tests should use injected model, token, response, panel, and
Runtime seams. Extension Host tests must exercise public commands and
activation/disposal, but do not need a live external language model: a
test-only participant seam or model adapter can prove host registration while
model behavior remains covered deterministically in unit tests.

## Executed baseline evidence

The investigation ran these non-zero checks against `39c229aa`:

- `cargo test -p oneagent-analysis --test context_engine` — 11 passed;
- `cargo test -p oneagent-runtime --test mcp_semantic_tools` — 6 passed;
- `cargo test -p oneagent-runtime --test mcp_process` — 9 passed;
- extension TypeScript typecheck — passed for production and test configs;
- extension compile — passed for production and test configs;
- extension unit tests — 38 passed, zero failed/skipped/todo;
- extension real Runtime-process tests — 2 passed;
- pinned VS Code 1.134.0 Extension Host — trusted 6 twice, empty 1,
  virtual 1, multi-root 1, and Restricted Mode 1: 16 passed total;
- exact nine-file Sprint 32 prompt inventory — present and tracked;
- immutable upstream tag and three source hashes — resolved successfully.

The first typecheck attempt used an incorrect repository-relative PATH and
failed before TypeScript execution with `node: not found`. The corrected
extension-relative pinned-Node invocation passed both typechecks and all later
Node commands. The repeated trusted Host run printed the already documented
VS Code bootstrap `Unexpected SIGPIPE` diagnostic while all six assertions and
the Host process exited zero; this remains execution noise, not hidden pass
evidence.

## Dependency, compatibility, and security impact

No production or dev dependency is required by the candidate slice. All needed
Chat, model, Quick Pick, webview, cancellation, and lifecycle types exist in
the pinned stable VS Code API. The current TypeScript/Host/package toolchain can
test them.

The highest-risk inputs are model text, user prompt, graph names/IDs, relation
IDs, and Runtime strings rendered into Markdown or HTML. The safe boundary is
closed decoding, exact byte/token admission, raw-error redaction, non-trusted
Chat Markdown, complete HTML escaping, scripts/forms/command URIs disabled,
empty local resource roots, strict CSP, one owned panel, and lifecycle-driven
invalidation. This is a bounded threat argument, not a broad security claim.

## Unresolved architecture decisions for ADR-0055

No evidence blocker remains. ADR-0055 must still choose and record:

1. exact participant/command/panel identifiers and user-visible English text;
2. exact fixed Context request values;
3. exact user-prompt byte limit, token admission rule and reserved model-output
   allowance using `maxInputTokens` and `countTokens`;
4. exact output byte/chunk limit and partial-stream result behavior;
5. exact selected-Context state invalidation on panel close versus only Runtime
   lifecycle changes;
6. exact non-trusted Chat rendering strategy;
7. exact stable failure categories and validation precedence;
8. exact generic Runtime queue and concurrent Chat ownership rules;
9. exact test-only Host observability seam without production behavior change;
10. exact docs, package, CI, and completion evidence.

These are bounded design choices with repository-owned oracles, not missing
external data. Task 2 is decision-ready.

## Decision readiness

ADR-0055 can accept an extension-only first slice with no new dependency and no
Rust, MCP, Context Engine, provider-adapter, Workspace, LSP, HTTP, or CLI change.
Every included behavior has a stable pinned API and a deterministic unit,
process, Host, package, or compatibility oracle. Every deferred behavior has an
explicit ownership reason.
