# OpenAI-Compatible Provider Investigation

## Status

- Investigation date: 2026-08-26 (`Asia/Novosibirsk`).
- Planning baseline: `f9124372` (`Plan Sprint 24 OpenAI-Compatible Provider`).
- Repository boundary: `/Users/maxim_tomshin/Development/oneagent` was the
  workspace used for repository evidence; no sibling project was inspected.
- Live evidence authority: user-authorized read-only SSH access to
  `maxim_adm@192.168.0.176` during the Sprint 24 launch investigation.
- Upstream source authority: llama.cpp build 10485 at commit
  `1511ce3bc3f087376c8526b4ad07100bfabb277f` dated 2026-08-18.
- Decision state: evidence is ready for ADR-0046; no adapter architecture or
  dependency set is accepted by this document.

## Purpose and evidence classes

This investigation supplies the repository, pinned-source, sanitized live-wire,
dependency, mapping, failure, consumer, and deterministic-oracle evidence needed
to decide the first concrete OpenAI-compatible provider adapter. It separates:

- **confirmed repository facts**, observed in the planning baseline;
- **accepted constraints**, already decided by ADR-0045;
- **confirmed pinned facts**, read from the exact llama.cpp commit;
- **sanitized live observations**, observed from the authorized service;
- **recommended decision candidates**, implementable choices for ADR-0046; and
- **unresolved decisions**, which this investigation does not accept.

Live availability is not reproducible acceptance evidence. No credential,
server configuration content, unrestricted request or response content, timing
payload, log, or private model artifact is retained here.

## Repository boundary and current implementation

### Workspace and ownership

The workspace root `Cargo.toml` has 13 members. The provider-neutral package is
`oneagent-llm` at `crates/llm`; there is no OpenAI-compatible adapter member and
no generic provider-adapter directory yet. Existing source adapters are owned
under `adapters/`, so `adapters/openai-compatible` is the consistent concrete
adapter location. The dependency direction required by ADR-0045 is:

```text
adapter/openai-compatible -> oneagent-llm -> std
```

The reverse dependency is invalid. `oneagent-llm` must not acquire HTTP, JSON,
TLS, executor, Runtime, Context Engine, or provider-specific wire dependencies.

### Provider-neutral contract inventory

The public contract is defined entirely by `crates/llm/src/`:

| Area | Confirmed contract |
|---|---|
| Identity | `ProviderId` is case-sensitive and bounded to 128 UTF-8 bytes. `ModelId` is case-sensitive and bounded to 256 bytes. Neither is normalized. `ModelIdentity` scopes a model to a provider. |
| Discovery | `ModelCatalog` accepts zero through 1,024 descriptors, sorts by full identity, and rejects duplicate or foreign-provider identities. The only capability is `TextGeneration`. |
| Construction | `ProviderConfiguration` owns a provider ID and optional `ProviderSecret`; it has no endpoint field or external configuration loading. The secret is non-empty, at most 4,096 bytes, explicitly exposed, and redacted by `Debug`. |
| Request | `TextGenerationRequest` owns one exact non-empty input of at most 65,536 UTF-8 bytes, one text-capable model identity, and `max_output_bytes` in `1..=65_536`. Its `Debug` omits input content. |
| Response | `TextGenerationResponse::new` binds success to the originating request, requires non-empty UTF-8 output within `max_output_bytes`, and derives usage from local input/output byte lengths. |
| Finish | Shared terminal values are only `Completed` and `OutputLimit`. |
| Policy | Timeout is absent or `1ns..=300s`; retry is fixed to `Never`; maximum attempts is exactly one. The neutral crate represents policy but owns no clock or executor. |
| Cancellation | `CancellationSignal` exposes an immediate check and an awaitable signal. `LlmProvider` requires pre-work and in-flight cooperative observation. |
| Errors | The closed kinds include configuration, request, catalog, response, rejection, unavailable, transport, protocol, timeout, cancellation, and internal failures. Diagnostics are optional, explicitly accessed, already redacted, and bounded to 512 bytes. |

`LlmProvider` is an asynchronous, object-safe seam with fresh
`discover_models` and one `generate` operation. Both return owned terminal
values. ADR-0045 requires provider mismatch and existing cancellation to be
rejected before provider work, no automatic retry, cooperative in-flight
cancellation, and validated cleanup.

### Consumers and compatibility

Repository search finds `oneagent-llm` only in its own manifest, implementation,
unit tests, and public conformance test. No other Rust package depends on it.
In particular:

- `oneagent-analysis` depends on BSL, common, and graph only;
- `oneagent-runtime` depends on source, graph, metadata, workspace, HTTP server,
  JSON, Tokio, and tracing packages, but not `oneagent-llm`;
- `oneagent-cli` has no production dependencies and uses Runtime only for tests;
- no protocol, configuration, Context Engine, Runtime, CLI, or persistence type
  currently owns provider construction or execution.

Sprint 24 can therefore add a leaf adapter without changing these consumers.
Runtime registration, configuration loading, Context-to-prompt orchestration,
and protocol exposure remain deferred.

### Existing transport and test patterns

The workspace already contains `http` and `hyper` transitively and uses Axum,
Serde, `serde_json`, and Tokio in Runtime. It contains no locked `reqwest`,
`url`, `rustls`, `hyper-rustls`, or `tokio-rustls` package. Existing controlled
HTTP tests bind an OS-assigned loopback port with `TcpListener`, exchange exact
bounded requests and responses, await explicit readiness/termination, and prove
that the address can be rebound after shutdown. These are reusable oracle
patterns, not permission to couple the adapter to Runtime.

Confirmed versions already selected elsewhere in the workspace are:

- `serde` 1.0.228 with derive where used;
- `serde_json` 1.0.150;
- `tokio` 1.53.0 with package-specific feature sets;
- `http` 1.4.2 and `hyper` 1.10.1 as transitive packages.

Their presence in `Cargo.lock` is not approval to add them as direct adapter
dependencies.

## Pinned llama.cpp evidence

### Source identity and relevant paths

The authorized host reported llama.cpp build 10485, source commit
`1511ce3bc3f087376c8526b4ad07100bfabb277f`, and a running `llama-server` built
from that source. The relevant pinned definitions are:

- `tools/server/README.md`: OpenAI-compatible `/v1/models` and
  `/v1/completions` endpoint documentation;
- `tools/server/server.cpp`: endpoint route registration and request dispatch;
- `tools/server/server-task.cpp`: completion result and error response mapping.

These paths and the full commit identify the evidence. The mutable upstream
branch, current upstream documentation, and other OpenAI-compatible servers are
not authorities for this slice.

### Required first-slice wire vocabulary

Pinned source and the live observations agree on these operations:

```text
GET  /v1/models
POST /v1/completions
Content-Type: application/json
Authorization: Bearer <credential>   # only when explicitly configured
```

The generation request fields needed by the shared contract are:

```json
{
  "model": "<exact requested model id>",
  "prompt": "<exact validated input>",
  "max_tokens": 1,
  "stream": false
}
```

The value `1` above is illustrative only. ADR-0046 must decide the safe mapping
from a byte bound to llama.cpp's token ceiling; this investigation does not
claim that bytes equal tokens. No temperature, sampling, seed, stop sequence,
logit, grammar, cache, or provider extension is required by ADR-0045.

The minimum discovery response shape is:

```json
{
  "object": "list",
  "data": [
    {
      "id": "<model id>"
    }
  ]
}
```

The minimum terminal completion response shape is:

```json
{
  "object": "text_completion",
  "model": "<actual response model id>",
  "choices": [
    {
      "text": "<bounded synthetic output>",
      "index": 0,
      "finish_reason": "stop"
    }
  ]
}
```

Pinned llama.cpp may include additional fields such as IDs, creation values,
ownership, metadata, usage, probabilities, and timings. They are not shared
domain authority. Unknown response fields can be ignored only after the
required envelope and terminal fields pass strict validation.

## Sanitized live observations

The user-authorized service was reachable through SSH as `maxim_adm` at
`192.168.0.176`. The system service was active and enabled and listened only on
`127.0.0.1:8080`; it was not directly exposed on the LAN address. No service
setting was changed.

| Observation | Sanitized result | Consequence |
|---|---|---|
| `GET /health` | HTTP success with `{"status":"ok"}`. | Confirms the observed server was ready; health is not part of the Sprint 24 adapter contract. |
| `GET /v1/models` | HTTP success; top-level `object` was `list`; `data` contained one model whose ID was `qwen3.8-27b-q8_0`. | Discovery must accept the exact ID and map every accepted entry to `TextGeneration`. Live cardinality is not a fixed contract. |
| Valid `POST /v1/completions` | HTTP success; top-level object was `text_completion`; response model matched the loaded model; exactly one choice had `index: 0`, bounded text, and `finish_reason: "stop"`; usage/timing data was also present. | The first slice can map the required terminal fields and ignore provider usage/timing fields. |
| Malformed JSON | HTTP 500 with a JSON error object. | Error classification cannot rely on every client-invalid payload producing 4xx. The adapter sends locally generated valid JSON, but must bound and redact any non-success body. |
| Missing `prompt` | HTTP 400 with a JSON error object whose type was `invalid_request_error`. | A valid shared request always supplies `prompt`; unexpected rejection maps by accepted status policy without retaining the provider body. |
| Unknown request model | The single-model server returned HTTP 200, performed generation with the loaded model, and reported `qwen3.8-27b-q8_0` in the response. | The adapter must compare the response model with the request model and reject this silent fallback atomically. |

Only a synthetic bounded prompt and output were used for the successful shape
check. Their contents, provider timing values, generated IDs, and full response
bodies are deliberately not retained.

## Mapping candidates for ADR-0046

### Construction and URL policy

Recommended implementable candidate:

1. Create `adapters/openai-compatible` as a leaf crate depending inward on
   `oneagent-llm`.
2. Construct it from one `ProviderConfiguration` plus one explicit base URL;
   never read environment variables, files, proxy variables, keychains, or
   global client state.
3. Accept only absolute `http` or `https` URLs with a non-empty host. Reject
   userinfo, query, fragment, non-root endpoint ambiguity, and unsupported
   schemes without echoing the input.
4. Normalize only the trailing slash needed for deterministic joining and append
   exact relative paths `v1/models` and `v1/completions`. Do not normalize host,
   path case, or percent-encoded identity.
5. Disable redirects and implicit proxies. Use a fixed adapter-owned user agent
   only if ADR-0046 accepts its exact non-sensitive value.
6. Send `Authorization: Bearer ...` only when a credential is present. Never
   place credentials in the URL, query, diagnostics, `Debug`, or retained errors.
7. Support TLS through an explicitly selected Rust TLS backend with no custom
   roots or invalid-certificate mode in the first slice.

ADR-0046 must decide whether the accepted base denotes the server root or an
already-versioned `/v1/` root. Accepting both creates ambiguous path joining and
is not recommended.

### Discovery mapping

Recommended candidate:

- require HTTP success, JSON object, exact top-level `object: "list"`, and an
  array `data`;
- require every entry to be an object with one valid exact string `id`;
- map every accepted ID to the configured provider and
  `ModelCapability::TextGeneration`;
- allow an empty array because `ModelCatalog` explicitly allows it;
- ignore unknown fields but reject missing, mistyped, over-limit, duplicate, or
  foreign-scope identities;
- build the complete vector first, then call `ModelCatalog::new` so sorting and
  rejection are atomic and provider-neutral;
- perform one fresh GET per call with no cache, refresh loop, retry, aliasing,
  or model selection.

ADR-0046 must decide a wire-entry count/body-byte limit at or below a bounded
transport envelope. The shared catalog limit remains exactly 1,024 models.

### Generation mapping

Recommended candidate:

- reject a request whose provider ID differs from the adapter before transport;
- copy exact model ID and input to `model` and `prompt`;
- always send `stream: false` and never accept a streaming response;
- send no sampling or provider-extension fields;
- require top-level `object: "text_completion"` and exact response `model`;
- require exactly one choice with numeric `index: 0`, string `text`, and a known
  finish reason;
- map `stop` to `FinishReason::Completed` and `length` to
  `FinishReason::OutputLimit`; reject null or all other finish values;
- require response model equality with the originating request to close the
  observed llama.cpp fallback behavior;
- construct success only through `TextGenerationResponse::new`, which enforces
  non-empty output, the local byte bound, model binding, and local byte usage;
- ignore provider-reported token usage, IDs, timestamps, and timings.

`max_output_bytes` cannot be represented exactly by `max_tokens`. A conservative
candidate is to use the accepted byte bound as the numeric token ceiling while
retaining the local byte limit as the authoritative terminal check. This avoids
under-requesting ASCII output, caps the provider request at 65,536 tokens, and
does not claim token/byte equivalence. ADR-0046 must explicitly accept or reject
this candidate.

### Errors, bounds, timeout, and cancellation

Recommended classification candidates:

| Condition | Candidate shared kind |
|---|---|
| Invalid base URL/client construction | `InvalidConfiguration` |
| Request belongs to another provider | `InvalidRequest` |
| Existing or winning in-flight cancellation | `Cancelled` |
| Accepted total timeout wins | `Timeout` |
| DNS/connect/TLS/write/read transport failure | `Transport` or a narrower accepted split with `ProviderUnavailable` |
| HTTP 408, 429, or 5xx | `ProviderUnavailable` candidate |
| Other non-success HTTP status | `ProviderRejected` candidate |
| Invalid JSON or invalid required wire envelope/field | `Protocol` |
| Valid wire envelope violates terminal shared semantics | `InvalidResponse` |
| Discovery violates catalog identity/count invariants | `InvalidModelCatalog` |

Status codes may be retained as bounded non-sensitive static diagnostics, but
provider response bodies, headers, URLs, credentials, prompts, and completions
must not be retained or formatted. Transport-library error sources can contain
URLs and must not be attached blindly to `LlmError`.

The transport must enforce a cumulative response-body byte limit while reading,
not only inspect `Content-Length` and not buffer an unbounded body first. One
small limit may cover both endpoints, or ADR-0046 may select separate discovery
and generation envelopes. The limit must leave room for the maximum accepted
65,536-byte output plus JSON framing while remaining explicitly bounded.

One total timeout covers request setup, connect, write, response headers, body
read, parse, and mapping. There is exactly one attempt. Existing cancellation is
checked before work; in-flight cancellation races the whole operation; the
losing transport future is dropped; all response/body state must be released.
ADR-0046 must decide deterministic timeout-versus-cancellation precedence when
both are ready.

## Dependency candidates and approval gate

The smallest practical async implementation candidate is:

| Dependency | Role | Repository state | Approval consequence |
|---|---|---|---|
| `oneagent-llm` path dependency | Shared identities, requests, responses, policy, cancellation, and errors. | Existing std-only workspace member. | New inward adapter edge; exact manifest entry must be accepted. |
| `reqwest` with default features disabled and an explicit Rust TLS/JSON feature set | Async HTTP client, URL parsing/joining, headers, redirect/proxy controls, bounded incremental response reading. | Not present in `Cargo.lock`. | New production dependency and transitive graph require explicit user approval. |
| `serde` with derive | Private strict wire DTOs. | Version 1.0.228 exists elsewhere. | New direct adapter production dependency requires explicit approval. |
| `serde_json` | Exact JSON encoding/decoding. | Version 1.0.150 exists elsewhere. | New direct adapter production dependency requires explicit approval. |
| `tokio` with only required time/synchronization/test features | Total timeout and deterministic cancellation race; loopback tests may additionally need net/runtime/macros as dev features. | Version 1.53.0 exists elsewhere. | Any production feature set and dev feature set must be accepted explicitly. |

A lower-level `hyper` implementation could avoid introducing `reqwest` as a
named dependency but would require selecting and maintaining more HTTP, body,
URI, connector, and TLS policy directly. The existing CLI HTTP client is a
minimal HTTP/1.1 Runtime client without TLS and does not satisfy the explicit
HTTP/HTTPS provider requirement. Reusing or copying it is not recommended.

ADR-0046 must select exact versions and features after checking the resulting
lock diff. Task 3 cannot change a production manifest or lockfile until the user
explicitly approves that exact set.

## Deterministic repository-owned oracle

Acceptance must use controlled loopback servers and synthetic fixtures only.
The server must bind `127.0.0.1:0`, expose its selected address explicitly,
accept a known finite connection count, capture bounded request metadata/body,
write one exact bounded response or controlled partial response, and join before
the test completes. No test may use the live host, credentials, environment
configuration, ignored files, fixed ports, sleeps, or mutable upstream state.

### Construction and request-capture matrix

- accepted HTTP and HTTPS URL structure without performing external I/O;
- rejected scheme, missing host, userinfo, query, fragment, ambiguous base path,
  boundary/control input, and over-limit input without echoing values;
- deterministic endpoint joining and exactly one request per operation;
- exact method, path, content type, and four generation fields;
- optional bearer header present exactly when configured;
- redirect response is terminal and never followed;
- no proxy/environment-derived destination;
- secret, URL, prompt, and response sentinels absent from all implicit formatting
  and error values.

### Discovery matrix

- positive empty, one-model, reordered multi-model, and unknown-field cases;
- missing/mistyped top-level object or data, malformed JSON, non-object entries,
  missing/mistyped/empty/over-limit IDs, duplicates, 1,024/excess model counts,
  over-limit body, relevant status classes, partial body, timeout, cancellation,
  and premature close;
- canonical ordering, provider scope, text capability, atomic rejection, one
  attempt, repeated fresh calls, cleanup, and address rebind.

### Generation matrix

- `stop` and `length` success, Unicode local byte accounting, exact output bound,
  tolerated unknown fields, and ignored provider usage;
- wrong provider before I/O, observed wrong-response-model fallback, missing or
  wrong envelope/model/choices/text/index/finish, zero or multiple choices,
  empty/over-limit output, malformed JSON, over-limit body, status classes,
  partial body, timeout, cancellation before/during work, and premature close;
- no retry, no redirect, no fallback, one terminal result, zero active operation
  state after success/failure/cancellation, repeated calls, and address rebind.

Public conformance must exercise the adapter through `&dyn LlmProvider`, while
focused private tests may inspect exact wire capture and constructor policy.

## Decision register for ADR-0046

| Question | Investigation status |
|---|---|
| Adapter owner and dependency direction | **Recommended:** `adapters/openai-compatible` depends inward on `oneagent-llm`; reverse coupling is prohibited. |
| Exact dependency versions/features | **Unresolved:** select the minimum `reqwest`/Rust-TLS, Serde/JSON, and Tokio sets, then obtain explicit user approval before Task 3. |
| Provider ID | **Unresolved:** choose one stable configured or adapter-fixed ID and define construction mismatch behavior. |
| Base URL meaning and normalization | **Unresolved:** root-base-only policy is recommended; exact path/trailing-slash rules must be accepted. |
| Redirect, proxy, and TLS policy | **Unresolved:** no redirects, no implicit proxies, and explicit Rust TLS are recommended. |
| Authentication | **Recommended:** optional explicit bearer only; no implicit credential source. Exact invalid-header behavior remains to decide. |
| Discovery shape and unknown fields | **Recommended:** strict required envelope/IDs, ignored unknown fields, shared canonicalization, empty catalog allowed. Exact wire body limit remains unresolved. |
| Generation fields | **Confirmed minimum:** `model`, `prompt`, `max_tokens`, `stream=false`; no sampling field is justified. |
| Byte-to-token mapping | **Unresolved:** numeric byte bound as conservative token ceiling plus authoritative local byte validation is the candidate. |
| Response identity and choices | **Recommended:** exact model match, exactly one choice at index zero, strict `stop`/`length`, local response constructor. |
| Error/status mapping | **Unresolved:** accept the precise 4xx/408/429/5xx and transport split plus static diagnostic vocabulary. |
| Body limits | **Unresolved:** choose explicit cumulative discovery and completion response bounds before buffering. |
| Timeout/cancellation precedence | **Unresolved:** one total timeout, pre-check, one in-flight race, and no retry are fixed; simultaneous-ready precedence must be accepted. |
| Compatibility | **Confirmed:** Context, Runtime, protocol, CLI, configuration, persistence, and `oneagent-llm` dependencies/API can remain unchanged. |
| Acceptance oracle | **Confirmed implementable:** deterministic synthetic controlled loopback plus public `LlmProvider` conformance; live service is excluded. |

## Deferred scope

Chat completions, Responses API, streaming/SSE, tools, structured output,
reasoning fields, images, audio, embeddings, reranking, prompt templates,
messages/roles, conversation state, tokenization, sampling policy, provider usage
authority, retry/backoff, rate limiting, concurrency pools, caching, fallback,
aliases, automatic model selection, implicit configuration, Runtime or protocol
registration, Context orchestration, persistence, MCP, LSP, IDE, UI, live
acceptance, performance, cost, quality, and broad OpenAI/third-party
compatibility remain outside Sprint 24.

## ADR-0046 readiness

The data gate passes. Repository types and consumers are known; the pinned
llama.cpp operation and required wire shapes are bounded; the live identity
fallback incompatibility is reproduced; production dependency candidates and
their approval gate are explicit; deterministic positive, negative, timeout,
cancellation, redaction, and cleanup oracles are implementable without external
state. ADR-0046 can now decide the exact first slice without inventing provider
fields or treating live availability as acceptance evidence.
