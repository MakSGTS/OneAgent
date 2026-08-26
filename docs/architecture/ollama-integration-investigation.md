# Ollama Integration Investigation

## Status and purpose

This document records the Sprint 26 evidence required for ADR-0048. It does
not accept architecture, change provider support, authorize dependencies, or
make installed Ollama, a local or cloud model, credentials, or network access a
repository prerequisite.

The investigation separates:

- **confirmed repository evidence** from current code, manifests, tests, and
  Git history;
- **accepted constraints** from ADR-0045 through ADR-0047;
- **official Ollama evidence** from primary documentation read on 2026-08-26;
- **sanitized local observations** explicitly authorized for this sprint;
- **unresolved decisions** that ADR-0048 must select before implementation.

## Confirmed repository baseline

The investigation ran from committed Sprint 26 planning baseline `68d8574b`
(`Plan Sprint 26 Ollama Integration`) with a clean working tree.

### Provider-neutral ownership

`oneagent-llm` remains the std-only provider-neutral owner. Its public boundary
confirms:

- case-sensitive, bounded, owned, provider-scoped identities;
- one closed `ModelCapability::TextGeneration` value;
- `ModelCatalog` canonical ordering, exact duplicate rejection, provider-scope
  validation, and a zero-through-1,024 model bound;
- explicit optional non-cloneable `ProviderSecret` construction input;
- one validated text request with at most 65,536 UTF-8 input bytes and an exact
  `1..=65_536` output-byte bound;
- one request-bound non-empty terminal text response, local UTF-8 byte usage,
  and closed `Completed` or `OutputLimit` finish;
- closed error kinds, bounded redacted diagnostics, represented total timeout,
  `RetryPolicy::Never`, cooperative cancellation, and object-safe
  `LlmProvider` discovery/generation futures.

There is no endpoint, model-family, local/cloud, provider metadata, token,
prompt-template, chat/message, tool, streaming, model lifecycle, registry,
Runtime, or configuration-source authority in the shared crate.

### Existing concrete providers

`oneagent-openai-compatible` exports only `OpenAiCompatibleProvider`. Its
client, URLs, bearer header, DTOs, execution helpers, and loopback support are
private. ADR-0046 fixes exact `/v1/models` and `/v1/completions` behavior under
stable provider ID `openai-compatible`.

`oneagent-lm-studio` exports only `LmStudioProvider`. It owns private native
type-aware discovery and privately composes the unchanged generic provider for
generation. Its provider ID, URLs, authentication, dependency graph, wire
mapping, tests, and public behavior are fixed by ADR-0047 and cannot be
generalized for Ollama without reopening that accepted contract.

Both implementations prove that a concrete provider can:

- validate an explicit origin root and numeric-loopback default without I/O;
- disable redirects and proxy discovery and use Rust TLS explicitly;
- bound JSON request and response bodies before decoding;
- return closed static redacted errors without provider or library sources;
- race one operation against cancellation and an optional total timer;
- use controlled `127.0.0.1:0` servers with exact request counts and joins;
- preserve provider-neutral identity and terminal response construction.

Private clients, execution helpers, DTOs, and test support cannot be imported
by a new package. Public composition can reuse complete
`OpenAiCompatibleProvider` generation only by translating to and from its fixed
provider identity, as the LM Studio leaf already demonstrates.

### Dependency and consumer inventory

The current concrete transport dependency block is:

```text
oneagent-llm
reqwest 0.13.4, default features disabled, rustls only
serde 1.0.228, derive only
serde_json 1.0.150
tokio 1.53.0, macros and time only
```

Controlled-loopback tests add only Tokio `io-util`, `macros`, `net`,
`rt-multi-thread`, `sync`, and `time`. LM Studio additionally has a direct
normal dependency on `oneagent-openai-compatible`.

Reverse normal dependency inspection confirms that only the two concrete
provider leaves consume `oneagent-llm`; only LM Studio consumes the generic
adapter. Analysis, Runtime, protocol, CLI, graph, workspace, and source
adapters consume none of these packages. Sprint 26 therefore has no existing
provider consumer to migrate.

Any new Ollama package edge or feature responsibility is a new direct
dependency even when its external version is already locked. ADR-0048 must
record an exact dependency block, and Task 3 requires explicit user approval
before adding it.

### Executable baseline and deterministic oracle

The current baseline passed on 2026-08-26:

- `cargo test -p oneagent-llm --offline` — 22 unit and 7 public integration
  tests passed; its zero-test doc-test target is not behavioral evidence;
- `cargo test -p oneagent-openai-compatible --lib --offline` — 18 passed;
- `cargo test -p oneagent-openai-compatible --test conformance --offline` — 6
  passed;
- `cargo test -p oneagent-lm-studio --lib --offline` — 19 passed;
- `cargo test -p oneagent-lm-studio --test conformance --offline` — 6 passed.

The concrete-provider tests used only sandbox-authorized controlled loopback.
They establish reusable test patterns, not a public transport or test-support
API.

## Accepted constraints from ADR-0045 through ADR-0047

ADR-0045 remains authoritative for provider identity, capability/catalog,
bounded raw text, local byte usage, finish, secret, errors, no retry, optional
total timeout, cooperative cancellation, one terminal result, and public
substitution.

ADR-0046 remains authoritative for the generic provider. Sprint 26 cannot
change its fixed identity, public surface, endpoints, request/response rules,
dependencies, client policy, errors, or tests to simplify Ollama.

ADR-0047 remains authoritative for LM Studio. Sprint 26 cannot export or move
its private native client, execution helpers, DTOs, or test harness, and cannot
change its composition or support claims.

The accepted first slice excludes provider metadata in shared values, chat and
message semantics, tools, structured output, reasoning, vision, embeddings,
streaming, prompt policy, token authority, model selection/lifecycle, retry,
fallback, cache, Runtime integration, and live-provider acceptance.

## Official Ollama evidence

Ollama's API documentation is not strictly versioned. The official
[API introduction](https://docs.ollama.com/api/introduction), read on
2026-08-26, states that the API is expected to remain backward compatible but
does not provide a semantic API version. The following exact primary URLs were
read on that date:

- [API introduction](https://docs.ollama.com/api/introduction) — installed
  Ollama serves the native API under `http://localhost:11434/api`; direct cloud
  service uses `https://ollama.com/api`;
- [get version](https://docs.ollama.com/api-reference/get-version) —
  `GET /api/version` returns a required `version` string;
- [list models](https://docs.ollama.com/api/tags) — `GET /api/tags` returns a
  `models` array with model names and file/detail metadata;
- [show model details](https://docs.ollama.com/api-reference/show-model-details)
  — `POST /api/show` requires `model` and returns a `capabilities` string array;
- [generate](https://docs.ollama.com/api/generate) — `POST /api/generate`
  accepts `model`, raw `prompt`, `stream`, `raw`, `think`, `keep_alive`, and
  runtime `options`; terminal response fields include `model`, `response`,
  `done`, `done_reason`, and provider timing/token additions;
- [streaming](https://docs.ollama.com/api/streaming) — generation streams NDJSON
  by default and returns one JSON response when `stream=false`;
- [errors](https://docs.ollama.com/api/errors) — common failures use HTTP 400,
  404, 429, 500, or 502 and a JSON `error` body; a non-streaming adapter can
  classify status without retaining that body;
- [authentication](https://docs.ollama.com/api/authentication) — local
  `http://localhost:11434` requires no authentication, while direct cloud API
  access uses a bearer API key; a local signed-in Ollama can transparently
  authenticate a cloud-model request;
- [OpenAI compatibility](https://docs.ollama.com/api/openai-compatibility) —
  Ollama supports `/v1/models` and `/v1/completions` in addition to chat and
  Responses endpoints; the compatibility API accepts a placeholder API key
  locally.

The current official
[OpenAPI document](https://github.com/ollama/ollama/blob/main/docs/openapi.yaml),
also read on 2026-08-26, adds two important details not shown in the rendered
Tags example:

- a model summary may include `remote_model` and `remote_host`, which identify
  a remote-backed entry;
- `ModelSummary` does not declare `capabilities`; the declared capability source
  is `ShowResponse` from `/api/show`.

Consequently, one `/api/tags` response alone is not a stable documented oracle
for `TextGeneration`. Capability-safe native discovery requires either bounded
`/api/show` calls for candidate models or another accepted documented source.
The OpenAI-compatible `/v1/models` schema likewise does not prove text versus
embedding capability.

### Native generation vocabulary

The smallest documented native request compatible with the shared raw-text
shape can use:

```json
{
  "model": "<exact model id>",
  "prompt": "<exact input>",
  "stream": false,
  "raw": true,
  "think": false,
  "options": { "num_predict": 1 }
}
```

The displayed `1` can be replaced by the provider-neutral output-byte bound as
a conservative token ceiling, but bytes and tokens are not equivalent. Setting
`raw=true` disables provider prompt templating; setting `think=false` prevents a
separate thinking output. `keep_alive` is deliberately omitted because setting
it would claim model load/unload duration policy.

The documented terminal response provides exact model and response strings,
`done`, and a string `done_reason`. Official examples show `stop`; official
schema does not define a closed enum. Upstream documentation and issue evidence
show `length`, but ADR-0048 must decide whether repository evidence is
sufficient to map only `stop` and `length` and reject every other value. Timing,
token counts, context, thinking, and log-probability additions have no shared
destination.

### Compatibility generation candidate

Ollama also documents non-streaming `/v1/completions` with `model`, string
`prompt`, `max_tokens`, and `stream`. The existing generic adapter already
implements that exact bounded operation and closed `stop`/`length` mapping.
Public composition is technically feasible without changing ADR-0046, but it
would retain the generic compatibility limitations and would not solve
capability-safe discovery.

ADR-0048 must compare native `/api/generate` against composed
`/v1/completions`. Native generation avoids a compatibility bridge and can set
`raw=true`, but requires a new private request/response mapping and an accepted
finish oracle. Composition reuses proven transport and finish behavior, but
requires bounded identity translation and preserves a compatibility endpoint
instead of the native API.

## Sanitized authorized local observations

The current user authorized bounded local Ollama use. No Ollama filesystem,
model path, server configuration, environment, credential, logs, unrestricted
model content, or cloud endpoint was inspected. No model was pulled, created,
copied, deleted, loaded, stopped, or invoked for generation.

Only this structural evidence was retained:

```text
client version: 0.33.0
server version: 0.33.0
catalog: one remote-backed entry
entry identity: non-empty name and model strings with equal values
entry remote marker: remote_model and remote_host present
entry capabilities: completion, tools, thinking, vision
```

The digest, actual model name, timestamps, sizes, model details, remote URL,
paths, and other payload were deliberately not retained. The observation proves
that version 0.33.0 may add `capabilities` directly to `/api/tags` and may list a
cloud-backed model. It does not override the official OpenAPI schema, establish
all-version behavior, prove local-model shape, authorize a cloud call, or serve
as repository acceptance evidence.

## Discovery evidence and unresolved mapping choices

### Confirmed capability gap

The shared catalog may advertise `TextGeneration` only from exact provider
evidence. Official `/api/tags` and `/v1/models` do not provide a declared
capability field. Inferring capability from a model name, family, format,
template, size, local presence, or absence of an embedding marker would be
unsafe.

Official `/api/show` provides the required capability list. A conservative
native candidate is:

1. perform one bounded fresh `GET /api/tags`;
2. validate and canonicalize at most 1,024 candidate identities before any
   model result escapes;
3. classify explicit remote-backed entries according to an ADR-0048 policy
   before any `/api/show` or generation request could cause cloud traffic;
4. perform one bounded `POST /api/show` for each accepted candidate with only
   its exact model ID and optional `verbose=false`;
5. project a descriptor with exactly `TextGeneration` only when capabilities
   contain exact lowercase `completion`;
6. omit or reject embedding-only and unknown capability combinations according
   to an explicit all-or-nothing ADR rule;
7. construct the complete result through `ModelCatalog::new`.

This is bounded N+1 discovery, not one-request discovery. ADR-0045 requires a
fresh provider call but does not require one network request. ADR-0048 must
define a maximum candidate/request count, request order, duplicate handling,
remote-entry policy, atomic failure behavior, and cancellation/timeout coverage
across the complete sequence.

### Identity choices

Official model summaries expose both `name` and `model` as model-name strings.
The local observation found equal values but does not prove universal equality.
ADR-0048 must select one canonical wire field or require exact equality; it
must not silently prefer one on conflict. `remote_model`, digest, family,
parameter size, quantization, and upstream host are provider metadata rather
than shared identity.

### Required discovery cases

Repository-owned fixtures can cover:

- empty, one, maximum, reordered, duplicate, and over-count tag catalogs;
- local, remote-backed, mixed, and ambiguous remote-marker entries;
- equal and conflicting name/model fields and invalid shared identities;
- `/api/show` completion, embedding-only, empty, duplicate, unknown, missing,
  and mistyped capability arrays;
- unknown additions, nulls, malformed/trailing JSON, partial bodies, advertised
  and streamed body bounds;
- tag/show status, redirect, transport, total timeout, pre-cancellation,
  in-flight cancellation at each sequence position, exact request count,
  deterministic order, repeated fresh calls, and cleanup.

No case requires a live daemon, model, credential, or cloud service.

## Construction, locality, authentication, and cloud boundary

Official local default is `http://localhost:11434`; the deterministic default
candidate is `http://127.0.0.1:11434` to avoid DNS. The existing origin-root
validation contract supports explicit HTTP/HTTPS scheme, host/port, root-only
path, no userinfo/query/fragment, no redirect, no proxy, Rust TLS, and redacted
errors without claiming locality.

Local Ollama itself requires no credential. Direct `https://ollama.com/api`
uses bearer authentication, while a signed-in local daemon may transparently
send cloud requests when a cloud model is selected. A provider cannot infer
from loopback alone that generation stays local.

ADR-0048 must decide:

- explicit root plus numeric-loopback default or a narrower local-only surface;
- whether `ProviderSecret` is rejected for the local first slice, accepted as
  explicit bearer for direct/proxied roots, or moved to deferred scope;
- whether remote-backed catalog entries are rejected atomically, filtered, or
  advertised with an explicit unsupported-live-acceptance limitation;
- whether an exact direct cloud origin is accepted or excluded.

The adapter must not read environment, files, sign-in state, server settings,
DNS/interface state, or model storage to answer these questions.

## Error, bounds, timeout, cancellation, and redaction evidence

The existing concrete mapping is reusable unless ADR-0048 records an
Ollama-specific exception:

- 408, 429, and 5xx -> `ProviderUnavailable`;
- every other non-success status, including redirects -> `ProviderRejected`;
- connect/TLS/request/body/premature-close failure -> `Transport`;
- malformed, partial, mistyped, trailing, or over-bound successful JSON ->
  `Protocol`;
- invalid/duplicate/over-count catalog identities -> `InvalidModelCatalog`;
- request provider mismatch -> `InvalidRequest`;
- response identity, terminal, or output violation -> `InvalidResponse`;
- winning total timer -> `Timeout`;
- existing or winning in-flight cancellation -> `Cancelled`.

Provider error bodies need not be read or retained on non-success status even
though Ollama documents an `error` field. Diagnostics can remain closed static
text plus decimal HTTP status. Root/endpoint URLs, credentials, headers, tag and
show bodies, remote fields, request input, output, thinking, timing/token/context
data, library sources, and every sentinel remain sensitive or unrestricted and
must not enter implicit formatting, errors, fixtures with real data, snapshots,
or review artifacts.

The existing 1 MiB discovery-body and 512 KiB generation request/response
bounds are feasible starting points. Multi-request show discovery requires an
explicit per-response bound and a checked aggregate-work/request-count bound.
Native generation must serialize before I/O and keep the local response output
bound authoritative after decoding.

One total timeout must cover the complete logical discovery or generation call.
There is exactly one attempt per planned wire step and no retry, fallback,
probing, detached task, or background refresh. Cancellation wins before work
when already requested and participates in deterministic biased races while an
operation is pending. Every terminal path drops request/response buffers,
futures, DTOs, header copies, and bounded identity copies before return.

## Deterministic repository oracle

A new package can reproduce the existing controlled-loopback pattern without
exporting another provider's private test support. The harness binds only
`127.0.0.1:0`, announces readiness without sleep, captures a bounded finite
request sequence, writes complete or deliberately partial synthetic responses,
joins deterministically, and proves zero surviving adapter state.

The complete first-slice oracle must include:

| Area | Required deterministic cases |
| --- | --- |
| Construction | Exact ID, explicit/default root, URL components, accepted auth choice, no-I/O, Send/Sync, dependency features, public surface, redaction. |
| Discovery wire | Exact Tags request plus bounded ordered Show requests if accepted, headers/body fields, fresh repetition, and no unintended remote request. |
| Discovery projection | Exact identity rule, completion capability, embedding/unknown capability behavior, local/cloud policy, empty/maximum/reordered/duplicate cases. |
| Discovery failures | Missing/mistyped/ambiguous fields, malformed/trailing/partial JSON, status, redirect, body/work bounds, transport, timeout, cancellation at every step, one attempt per step, cleanup. |
| Generation wire | Exact native or composed endpoint, model/input/output ceiling, non-streaming/raw/thinking/template policy, response identity, `stop`/`length`, local byte usage. |
| Generation failures | Provider/response mismatch, malformed terminal shapes, empty/over-bound output, status, redirect, body bounds, transport, timeout, cancellation, redaction, one attempt, cleanup. |
| Compatibility | Complete Ollama unit/public targets, both existing concrete-provider targets, `oneagent-llm`, provider substitution, Analysis and Runtime library tests, dependency direction. |

No acceptance case may use installed/running Ollama, local or cloud models,
credentials, environment or filesystem state, external network, ignored tests,
timing thresholds, provider token counts, response text, quality, performance,
or cost.

## Unresolved ADR-0048 decisions

ADR-0048 has sufficient evidence to decide, but must explicitly select:

1. package/path/type names, public surface, stable `ollama` identity, and exact
   dependency direction;
2. exact normal/dev dependency and feature block plus the approval gate;
3. explicit origin root, numeric-loopback default, locality, authentication,
   and direct/cloud-root behavior;
4. Tags/Show versus another documented discovery sequence, identity field,
   request ordering/count, remote-entry policy, capability vocabulary, unknown
   behavior, body/work bounds, and atomicity;
5. native `/api/generate` versus composed `/v1/completions`, including exact
   raw/template/thinking, output-ceiling, response-model, `done`, finish, and
   provider-addition mapping;
6. error/status diagnostics, redaction, timeout/cancellation precedence,
   exactly-once-per-step semantics, and cleanup;
7. public conformance, existing-provider regressions, consumer compatibility,
   documentation completion, and deferred scope.

## Deferred and unsupported scope

- Ollama installation, daemon/server lifecycle, upgrade, pull/create/copy/
  delete/push, storage, preload/load/unload, keep-alive ownership, and model
  selection;
- live local or cloud models, credentials, direct cloud traffic, output,
  latency, throughput, cost, quality, or broad compatibility/security claims as
  acceptance;
- chat/history, roles/messages, streaming, tools, MCP, structured output,
  thinking/reasoning, vision/images, embeddings, prompt templates/policy,
  provider metadata, token authority, and sampling controls;
- fallback, retry/backoff, rate limiting, concurrency policy, cache/refresh,
  registry, persistence, and endpoint negotiation;
- environment/file/CLI/keychain configuration, sign-in discovery, proxy
  configuration, custom TLS roots, client certificates, insecure TLS, reload,
  and secret zeroization;
- Runtime registration/routes/lifecycle, Context orchestration, protocol/CLI,
  MCP, LSP, IDE, UI, graph mutation, source adapters, and Coverage changes.

## Decision readiness and recommended next action

The data and testability gate passes for ADR-0048. Official documentation,
sanitized local structure, current public APIs, and executable controlled-
loopback patterns provide enough evidence to choose a bounded Ollama leaf.
Capability-safe discovery cannot rely on Tags alone under the official schema;
the architecture must resolve bounded Tags/Show behavior and the cloud boundary.
Generation has two feasible candidates whose trade-offs are explicit.

The next task should create ADR-0048, select one implementable first slice,
record the exact dependency approval gate, and keep implementation blocked until
that ADR is committed and every new direct dependency/feature is explicitly
approved by the user.
