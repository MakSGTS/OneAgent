# ADR-0046: OpenAI-Compatible Provider Adapter

## Status

Accepted

Implementation remains gated on explicit user approval of the exact dependency
set in this ADR. Acceptance does not claim that the adapter is implemented or
supported.

## Context

Sprint 24 must add the first concrete provider behind ADR-0045's std-only
`LlmProvider` seam. The
[OpenAI-compatible provider investigation](../architecture/openai-compatible-provider-investigation.md)
confirms:

- `oneagent-llm` owns the complete provider-neutral identity, catalog, bounded
  text, usage, finish, policy, cancellation, error, and substitution contract;
- no current Context Engine, Runtime, protocol, CLI, or configuration consumer
  depends on that crate;
- llama.cpp build 10485 at pinned commit
  `1511ce3bc3f087376c8526b4ad07100bfabb277f` exposes the required
  `/v1/models` and non-streaming `/v1/completions` operations;
- the authorized live service returned the expected list and completion shapes,
  but also accepted an unknown request model and silently used the loaded model;
- deterministic controlled-loopback evidence can reproduce every accepted
  success, malformed, bound, identity, timeout, cancellation, redaction, and
  cleanup case without a live provider or credential.

ADR-0045 is authoritative wherever provider-neutral behavior is involved. This
decision owns only concrete configuration, HTTP/TLS/JSON wire mapping, adapter
execution, and adapter conformance.

Official crates.io metadata checked on 2026-08-26 identifies `reqwest 0.13.4`
as the current stable release, with Rust 1.85 as its minimum supported version.
Its default features include system proxy discovery, so default features cannot
be enabled under the explicit-configuration contract. The repository toolchain
is Rust 1.97.1.

## Decision

### Canonical ownership and dependency direction

Create workspace library package `oneagent-openai-compatible` at
`adapters/openai-compatible`. It owns:

- `OpenAiCompatibleProvider`, the only public concrete adapter type;
- validated endpoint/client construction and the adapter provider identity;
- optional bearer-header construction and all concrete redaction policy;
- private OpenAI-compatible discovery and completion DTOs;
- HTTP request execution, bounded response reading, JSON parsing, wire mapping,
  timeout/cancellation races, and cleanup;
- focused and public controlled-loopback conformance evidence.

Dependency direction is strictly:

```text
future composition/callers -> oneagent-openai-compatible -> oneagent-llm -> std
```

`oneagent-llm` remains unchanged and std-only. The concrete adapter does not
depend on analysis, Runtime, protocol, CLI, graph, workspace, source adapters,
or another concrete provider. No current consumer gains an adapter dependency
in Sprint 24.

### Exact dependency set and approval gate

The adapter manifest must use exactly these direct dependencies:

```toml
[dependencies]
oneagent-llm = { path = "../../crates/llm" }
reqwest = { version = "0.13.4", default-features = false, features = ["rustls"] }
serde = { version = "1.0.228", features = ["derive"] }
serde_json = "1.0.150"
tokio = { version = "1.53.0", features = ["macros", "time"] }

[dev-dependencies]
tokio = { version = "1.53.0", features = ["io-util", "macros", "net", "rt-multi-thread", "sync", "time"] }
```

No other direct production or dev dependency is accepted. In particular, do
not enable reqwest defaults, `system-proxy`, compression, charset conversion,
cookies, forms, multipart, blocking, HTTP/2, HTTP/3, native TLS, SOCKS, DNS
replacement, or stream features. `serde_json` rather than reqwest's `json`
feature serializes and parses private DTOs so successful response bytes can be
bounded before decoding.

`rustls` selects reqwest's Rust TLS implementation and platform verifier. The
first slice accepts only platform trust roots; it provides no custom root,
client certificate, insecure-certificate, or TLS-version configuration.

The normal Tokio features own `select!` and the total timer only. Loopback
listener, I/O, runtime, synchronization, and test macro support are dev-only.
Cargo may add or update transitive lock entries mechanically from these exact
requirements, but Task 3 must inspect that diff and must not broaden direct
features.

Task 3 is prohibited until the user explicitly approves this exact dependency
set. Sprint launch and ADR acceptance are not dependency approval.

### Stable provider identity and public construction

The adapter provider ID is exactly the case-sensitive string
`openai-compatible`. Construction is conceptually:

```rust
impl OpenAiCompatibleProvider {
    pub fn new(
        configuration: ProviderConfiguration,
        base_url: &str,
    ) -> Result<Self, LlmError>;
}
```

The constructor consumes `ProviderConfiguration`. Its provider ID must equal
`openai-compatible`; mismatch returns `InvalidConfiguration` before client
construction. `LlmProvider::id()` returns that accepted owned ID.

The public type is `Send + Sync` but does not implement `Clone`, `Debug`,
`Display`, serialization, equality, or hashing. It owns one reqwest client, the
provider ID, two parsed endpoint URLs, and an optional sensitive authorization
header. It does not expose its client, URLs, or header. Secret zeroization and
prevention of unavoidable transport-buffer copies remain unclaimed.

### Base URL contract

`base_url` denotes the server origin root, not an endpoint and not an already
versioned `/v1` base. Validation occurs before constructing a success value:

1. require `1..=2_048` UTF-8 bytes;
2. reject leading/trailing Unicode whitespace and every Unicode control
   character without echoing the value;
3. parse one absolute URL;
4. require scheme exactly `http` or `https` after URL parsing;
5. require a non-empty host and permit an explicit port;
6. reject username, password, query, and fragment;
7. require path to be empty or exactly `/`; reject every other path;
8. set the stored root path to `/` and join exact relative paths `v1/models`
   and `v1/completions`;
9. verify joined URLs retain the same scheme, host, and effective port.

Parsing may apply the URL library's normal scheme/host canonicalization. That
canonical form is private transport state, never provider/model identity and
never a displayed value. There is no path case conversion, arbitrary path
normalization, endpoint probing, or base discovery.

Construction failure maps to `InvalidConfiguration` with one of a closed set of
static diagnostics naming only the violated rule. No diagnostic contains the
input URL, host, port, parse error source, or credential.

### HTTP, redirect, proxy, TLS, and client policy

Build one reqwest client with:

- redirect policy `reqwest::redirect::Policy::none()`;
- `no_proxy()` even though the `system-proxy` feature is disabled;
- the exact static user agent `oneagent-openai-compatible/0.1.0`;
- no default authorization or content header;
- no client-level connect, read, or total timeout that could conflict with the
  provider-neutral total timeout;
- no cookie store, decompression, connection pool tuning, DNS override, local
  bind, interface selection, custom TLS roots, or invalid-certificate mode.

Both explicit HTTP and HTTPS roots are accepted. HTTP exists for local provider
endpoints and controlled loopback; the adapter makes no confidentiality claim
for HTTP. HTTPS uses the selected Rust TLS/platform-root policy.

A redirect response is terminal because redirects are disabled. The adapter
does not follow it, does not forward authorization, and does not retry another
location. Environment variables and files cannot change proxy, base URL,
credential, TLS, timeout, or retry behavior.

### Bearer authentication and redaction

An absent `ProviderSecret` sends no `Authorization` header. A present secret is
converted to exactly `Authorization: Bearer <secret>` during construction.
Failure to represent it as an HTTP header maps to `InvalidConfiguration` with a
static diagnostic and no raw value. The header is inserted explicitly into each
request and is never placed in the URL or JSON body.

The complete sensitive boundary includes:

- base and endpoint URLs, hostnames, user input, generated output;
- authorization and all request/response headers;
- request and response bodies, provider error objects, reqwest error sources;
- parsed DTOs containing prompt, output, model, or provider additions;
- credentials and every synthetic sentinel used by tests.

None may appear through adapter/error `Debug`, `Display`, `source`, panic text,
diagnostics, logs, snapshots, or retained provider bodies. Public `LlmError`
formatting remains ADR-0045's generic output. Adapter diagnostics use only a
closed static vocabulary plus a decimal HTTP status when applicable and remain
within 512 bytes. The adapter does not attach reqwest or Serde errors as sources.

Explicit successful `input()` and `output()` access remains governed by
ADR-0045 and is not logging authorization.

### Request-body bounds

Discovery sends no body. Completion JSON is serialized to an owned byte vector
before transport. The exact maximum serialized body is 524,288 bytes (512 KiB),
which accommodates worst-case JSON escaping of the accepted 65,536-byte input
and 256-byte model ID plus the fixed envelope. Exceeding this adapter wire bound
returns `InvalidRequest` before HTTP I/O.

The serializer is infallible for the private string/bool/integer DTO in normal
operation; an unexpected serialization failure maps to `Internal` with a static
diagnostic. The serialized bytes are not retained after the operation.

### Model discovery request and response

Every `discover_models` call performs exactly one fresh:

```text
GET /v1/models
Accept: application/json
Authorization: Bearer ...   # only when configured
```

No request body, cache, conditional header, endpoint probe, retry, refresh,
fallback, or merge is allowed.

A successful response must be a JSON object matching this private logical DTO:

```text
object: required string, exactly "list"
data:   required array, zero through 1,024 entries
entry:  required object with required string id
```

Unknown fields at the top level and inside entries are ignored. Missing or
mistyped required fields, a non-`list` object, malformed/trailing JSON, or a
partial body maps to `Protocol`. An entry ID is passed unchanged to
`ModelId::new`; invalid IDs, more than 1,024 entries, or duplicate exact IDs map
to `InvalidModelCatalog`. Empty `data` is successful.

Every accepted entry receives the configured provider ID and exactly
`ModelCapability::TextGeneration`. All entries are built before
`ModelCatalog::new` performs canonical full-identity ordering and atomic
duplicate/provider validation. Provider ownership, creation, metadata, and
unknown capability fields are ignored and are not shared-domain values.

### Completion request

`generate` first rejects a request whose provider ID differs from
`openai-compatible` with `InvalidRequest` and no HTTP I/O. A valid request is
serialized with exactly these fields:

```json
{
  "model": "<exact request model id>",
  "prompt": "<exact request input>",
  "max_tokens": 1,
  "stream": false
}
```

The displayed `1` is replaced by the request's numeric `max_output_bytes`.
This is a conservative provider token ceiling, not token/byte equivalence. It
never exceeds 65,536. The originating local byte bound remains authoritative
after decoding. No temperature, sampling, seed, stop, suffix, log probability,
echo, grammar, cache, role, message, prompt template, or provider extension is
sent.

The request is exactly one:

```text
POST /v1/completions
Content-Type: application/json
Accept: application/json
Authorization: Bearer ...   # only when configured
```

### Completion response and fallback rejection

A successful response must be a JSON object with:

```text
object:         required string, exactly "text_completion"
model:          required string
choices:        required array with exactly one entry
choice.text:    required string
choice.index:   required integer, exactly 0
finish_reason:  required string, exactly "stop" or "length"
```

Unknown fields are ignored only after required fields validate. Malformed or
trailing JSON, missing/mistyped fields, a non-`text_completion` object, or a
partial body maps to `Protocol`. Zero/multiple choices, a nonzero index, an
unknown/null finish, invalid response model ID, response model mismatch, empty
output, or over-bound output maps to `InvalidResponse`.

The response model must equal the exact originating request model before
success construction. This rejects the observed llama.cpp behavior where an
unknown requested model silently falls back to the loaded model. There is no
alias, normalization, fallback, or use of a preceding catalog to rewrite it.

Map `stop` to `FinishReason::Completed` and `length` to
`FinishReason::OutputLimit`. Construct success only through
`TextGenerationResponse::new`; local exact UTF-8 input/output byte usage is
authoritative. Provider token usage, IDs, timestamps, ownership, timings,
probabilities, and other additions are ignored.

### Response-body bounds and decoding

For successful status responses, inspect an available `Content-Length` before
reading and reject it if it exceeds the endpoint bound. Regardless of that
header, read `Response::chunk()` incrementally, use checked addition, and stop
before appending a chunk that would exceed:

| Endpoint | Maximum successful response body |
|---|---:|
| `/v1/models` | 1,048,576 bytes (1 MiB) |
| `/v1/completions` | 524,288 bytes (512 KiB) |

These bounds include the full encoded JSON envelope. The completion bound
accommodates worst-case JSON escaping of the maximum shared output. The
discovery bound accommodates 1,024 accepted maximum-length IDs plus framing and
ordinary escaping; an ID that decodes to a control character remains invalid.

An exceeded bound or checked-add overflow maps to `Protocol` with a static
diagnostic. The adapter never calls an unbounded whole-body helper. It parses
only after the complete bounded body arrives and rejects trailing non-whitespace
JSON through `serde_json`'s single-value parser.

For non-success HTTP status, do not read or parse the response body at all; drop
it and return the status mapping. This prevents provider error payload retention
and makes error-body size irrelevant to adapter memory.

### HTTP, transport, protocol, and semantic error mapping

The mapping is exact:

| Condition | `LlmErrorKind` |
|---|---|
| Invalid construction input, provider ID, URL, header, or client build | `InvalidConfiguration` |
| Request provider mismatch or serialized request over 512 KiB | `InvalidRequest` |
| HTTP 408, 429, or 500 through 599 | `ProviderUnavailable` |
| Every other non-success HTTP status, including redirects and other 3xx/4xx | `ProviderRejected` |
| DNS, connect, TLS, request write, response header/body read, or premature-close failure | `Transport` |
| Malformed/partial/trailing JSON, missing/mistyped wire fields, wrong wire object, or successful body over endpoint limit | `Protocol` |
| Invalid/duplicate/over-count discovery identities | `InvalidModelCatalog` |
| Completion model/choice/index/finish/output violates terminal shared semantics | `InvalidResponse` |
| Accepted total timeout wins | `Timeout` |
| Existing or winning in-flight cancellation | `Cancelled` |
| Unexpected private serialization/invariant failure | `Internal` |

Status diagnostics may contain only `provider returned HTTP NNN`. All other
diagnostics are closed static English strings. Provider error JSON and reqwest/
Serde source text are never copied into `LlmError`.

There is exactly one attempt for every condition. `is_retryable()` remains
classification only and cannot cause replay.

### Timeout, cancellation, terminal precedence, and cleanup

For both operations, terminal precedence is:

1. generation provider mismatch before cancellation;
2. already-requested cancellation before serialization or HTTP work;
3. start exactly one operation future;
4. race the complete operation against `CancellationSignal::cancelled()` and,
   when configured, one Tokio total timer;
5. when multiple branches are ready in the same poll, cancellation wins, then
   the operation result, then timeout;
6. otherwise the first observed branch wins and later events cannot replace it;
7. drop losing futures, bounded buffers, DTOs, request bytes, response state,
   and temporary header copies before returning one terminal result.

The total timeout covers serialization after the pre-check, request creation,
DNS, connect, TLS, write, headers, all body chunks, JSON decoding, mapping, and
terminal shared construction. An absent timeout creates no timer. There is no
separate connect/read timeout, retry delay, spawned adapter task, detached body
reader, or background refresh.

`tokio::select!` uses explicit biased branch order to make simultaneous-ready
precedence deterministic. Dropping reqwest futures is the cooperative abort
mechanism owned by this adapter. Conformance must prove no adapter-owned active
operation remains after success, every failure, timeout, or cancellation.

### Deterministic conformance

No acceptance test may contact a live service. Tests use synthetic data and a
controlled server bound to `127.0.0.1:0`. The harness exposes readiness without
sleeps, accepts an exact finite request count, bounds captured input, writes one
exact or deliberately partial response, joins deterministically, and proves
address rebind/zero active state after completion.

Focused evidence must cover:

- provider ID, URL bound/scheme/host/port/userinfo/query/fragment/path and
  endpoint joining;
- client no-proxy/no-redirect behavior and exact static user agent;
- absent/present/invalid bearer behavior and complete formatting redaction;
- exact methods, paths, headers, JSON field sets, input/model preservation, and
  request-body bound;
- discovery empty/positive/reordered/unknown/missing/mistyped/malformed/
  duplicate/over-count/over-body/status/transport/timeout/cancel cases;
- generation `stop`/`length`/Unicode/exact-bound/unknown-field/usage-ignore and
  wrong-model/choice/index/finish/empty/over-output/over-body/status/transport/
  timeout/cancel cases;
- redirect not followed, exactly one attempt, repeated fresh operations,
  dropped partial work, sentinel absence, and zero active state.

The public non-zero conformance target must use `OpenAiCompatibleProvider`
through `&dyn LlmProvider`. Private tests may inspect captured wire data and
construction internals but cannot replace public seam evidence.

### Compatibility and documentation completion

Sprint 24 does not change `oneagent-llm`, `oneagent-analysis`,
`oneagent-runtime`, `oneagent-protocol`, CLI, Runtime configuration, semantic
graph, source adapters, or current Coverage Registries. Existing LLM, analysis,
and Runtime library tests remain compatibility gates.

Architecture acceptance alone changes no support claim. After Tasks 3-5
implement the adapter, Task 6 adds the public conformance matrix and synchronizes
only truthful current-state text in `README.md`, `docs/Architecture.md`, and
`docs/architecture/semantic-model-2.md`. Sprint 24 remains incomplete until an
independent review records a non-blocking decision after the full workspace
gate.

## Rejected alternatives

### Put OpenAI wire types in `oneagent-llm`

Rejected because ADR-0045 deliberately keeps the shared crate std-only and
provider-neutral. Wire DTOs, HTTP/TLS, JSON, URLs, and bearer headers belong to
the concrete adapter.

### Put the adapter in Runtime or Context Engine

Rejected because Runtime is a future composition owner and Context Engine is a
deterministic semantic consumer. A reusable concrete adapter must not depend on
either application layer.

### Reuse the CLI HTTP/1.1 client

Rejected because it is Runtime-specific, has no HTTPS/TLS client contract, and
does not own the required JSON, redirects, proxy, timeout, cancellation, or
provider redaction behavior.

### Use reqwest default features or native TLS

Rejected because reqwest 0.13.4 defaults include implicit system proxy behavior
and unrelated charset/HTTP2 features. Native TLS creates platform-specific
backend behavior. The exact Rust TLS feature with defaults disabled is smaller
and explicit.

### Use low-level Hyper directly

Rejected for the first slice because it requires additional URI, body,
connector, and TLS policy ownership without improving the accepted contract.
The selected reqwest surface already exposes no-proxy, no-redirect, explicit
headers, status, and incremental chunks.

### Accept arbitrary base paths or both root and `/v1` bases

Rejected because endpoint joining becomes ambiguous and configuration errors
can silently target `/v1/v1` or replace a path segment. The first slice accepts
one root-origin meaning only.

### Follow redirects or honor environment proxies

Rejected because either can change the destination and credential path through
implicit external configuration. The first slice requires deterministic direct
destination behavior.

### Copy provider usage or accept unknown finish reasons

Rejected because ADR-0045 defines local UTF-8 byte usage and a closed finish
vocabulary. Provider tokens and unknown wire terminals cannot become shared
truth implicitly.

### Trust the requested model when the response differs

Rejected by live evidence: llama.cpp can silently fall back to its loaded model.
Exact response identity is mandatory before success.

### Buffer whole bodies before enforcing limits

Rejected because `Content-Length` can be absent or false and a whole-body helper
can allocate beyond the accepted bound. Incremental checked reading is required.

### Add retries, cache, model selection, or provider probing

Rejected because ADR-0045 fixes one attempt and fresh discovery, and Sprint 24
has no replay, freshness, alias, selection, or endpoint-negotiation authority.

### Require live llama.cpp acceptance

Rejected because availability, model state, credentials, responses, latency,
and upstream behavior are mutable. Live observations are supplementary evidence
only.

## Consequences and risks

- One new leaf crate and a bounded external dependency graph are added after
  explicit approval; `oneagent-llm` and current consumers remain unchanged.
- HTTP permits unencrypted local transport by explicit caller choice; HTTPS
  uses platform trust and no custom-root escape hatch.
- The byte bound used as `max_tokens` can allow more provider work than the
  final byte output accepts. Local response validation prevents oversized
  success but cannot make token cost equal bytes.
- Ignoring provider usage and unknown fields preserves shared semantics but
  deliberately exposes no provider metadata.
- Disabling redirects/proxies may reject environments that require them; that is
  preferred to implicit destination changes in the first slice.
- Cooperative cancellation relies on dropping reqwest futures and cannot claim
  server-side generation termination after a connection closes.
- Credentials can have unavoidable in-memory copies in HTTP buffers; no
  zeroization or security certification is claimed.

## Implementation prerequisites and order

1. Obtain explicit user approval for the exact direct dependency/version/
   feature set recorded above.
2. Task 3 creates the workspace member, manifest/lock changes, public constructor,
   private constants/DTO foundation, URL/client/auth/redaction policy, and
   focused construction tests without performing discovery or generation.
3. Task 4 implements fresh bounded model discovery, exact mapping, errors,
   timeout/cancellation, cleanup, and focused controlled-loopback evidence.
4. Task 5 implements the exact non-streaming completion request/response,
   identity fallback rejection, local output validation, errors, timeout/
   cancellation, cleanup, and focused controlled-loopback evidence.
5. Task 6 adds the public conformance target, dependency/redaction audits,
   consumer compatibility, full validation, and truthful current-state docs.
6. Task 7 independently reviews the complete range before any Sprint state
   transition or previous-suite retirement.

If the approved dependency graph cannot provide the exact no-proxy,
no-redirect, Rust-TLS, bounded-chunk, timeout, cancellation, or redaction
contract, stop before substituting dependencies or weakening this ADR.

## Deferred scope

- Chat completions, Responses API, streaming/SSE, tools, structured output,
  reasoning fields, images, audio, embeddings, reranking, and extensions;
- messages/roles, prompt templates or policy, conversation history, sampling,
  tokenization, provider usage authority, cost, quality, and performance;
- retry/backoff, rate limiting, concurrency pools, caching, refresh, aliases,
  fallback, model selection, health probing, and endpoint negotiation;
- environment/file/CLI/keychain configuration, proxy configuration, custom TLS
  roots, client certificates, insecure TLS, reload, and secret zeroization;
- Runtime registration/routes/lifecycle, Context orchestration, protocol/CLI,
  persistence, MCP, LSP, IDE, UI, and graph mutation;
- live-service acceptance and broad OpenAI, llama.cpp, or third-party
  compatibility/security claims;
- LM Studio, Ollama, Tool Execution Policy, and later sprint implementation.

## Completion criteria

Sprint 24 can complete only after dependency approval is recorded, Tasks 3-6
implement and prove every accepted construction, wire, bound, identity,
redaction, status, transport, protocol, timeout, cancellation, cleanup, and
compatibility rule without external state, the canonical full workspace gate
succeeds, current-state documentation preserves all deferrals, and the Sprint
24 integration review records `pass` or `pass with non-blocking follow-ups`.
