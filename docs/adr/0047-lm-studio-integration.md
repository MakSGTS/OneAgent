# ADR-0047: LM Studio Provider Integration

## Status

Accepted

Implementation remains gated on explicit user approval of every direct
dependency edge and feature set recorded in this ADR. Acceptance changes no
current support claim and does not authorize Cargo or production Rust changes.

## Context

Sprint 25 must add one bounded LM Studio provider behind ADR-0045's
provider-neutral `LlmProvider` seam without weakening the implemented and
reviewed ADR-0046 OpenAI-compatible adapter.

The [LM Studio integration investigation](../architecture/lm-studio-integration-investigation.md)
confirms:

- `oneagent-llm` already owns the complete provider-neutral identity, catalog,
  bounded text, usage, finish, error, timeout, cancellation, and substitution
  contract;
- `OpenAiCompatibleProvider` is a complete leaf whose public surface exposes
  only construction and `LlmProvider`; its client, URLs, headers, execution
  helpers, DTOs, and test support are private;
- LM Studio's OpenAI-compatible `/v1/models` response does not expose model
  type and can contain both language and embedding models, so using generic
  discovery would advertise an embedding-only model as `TextGeneration`;
- LM Studio's native `GET /api/v1/models` response distinguishes `llm` from
  `embedding` and identifies loaded instances through
  `loaded_instances[].id`;
- the public ADR-0045 APIs can translate one LM Studio request into a temporary
  `openai-compatible` request, delegate exactly once, and bind the validated
  result back to the original LM Studio request without changing ADR-0046;
- native `/api/v1/chat` lacks the accepted closed finish mapping, while
  `/v1/chat/completions` imports roles, messages, and prompt-template semantics;
- `/v1/completions` is the only investigated endpoint compatible with the
  existing raw-text request and closed terminal finish contract, but LM Studio
  documents it as a legacy base-model endpoint whose output can be unsuitable
  for chat-tuned models;
- controlled loopback can prove every accepted success, malformed, bound,
  identity, timeout, cancellation, redaction, and cleanup outcome without LM
  Studio, a downloaded model, a credential, or external network access.

ADR-0045 remains authoritative for all provider-neutral behavior. ADR-0046 and
the [Sprint 24 integration review](../reviews/sprint-24-openai-compatible-provider.md)
remain authoritative for the generic adapter. This decision owns only the new
LM Studio leaf, its native discovery mapping, its composition over the existing
generic generation operation, and its conformance evidence.

## Decision

### Canonical ownership and dependency direction

Create workspace library package `oneagent-lm-studio` at
`adapters/lm-studio`. It owns:

- `LmStudioProvider`, the only public concrete adapter type;
- stable LM Studio provider identity and deterministic construction;
- one private native discovery client, URL, authorization header, DTO set, and
  execution helpers;
- one private composed `OpenAiCompatibleProvider` used only for terminal text
  generation;
- native discovery projection, composition translation, redacted error policy,
  and focused/public controlled-loopback evidence.

Dependency direction is strictly:

```text
future composition/callers -> oneagent-lm-studio
oneagent-lm-studio -> oneagent-openai-compatible -> oneagent-llm -> std
oneagent-lm-studio -------------------------------> oneagent-llm
```

`oneagent-llm` and `oneagent-openai-compatible` remain unchanged. The new leaf
does not depend on analysis, Runtime, protocol, CLI, graph, workspace, source
adapters, or another concrete provider. No current consumer gains an LM Studio
dependency in Sprint 25.

The composed generic provider remains a private implementation detail. It
retains its exact `openai-compatible` identity and ADR-0046 behavior; it is not
reconfigured to claim `lm-studio`, and its private transport surface is not
exported.

### Exact dependency set and approval gate

The new adapter manifest must use exactly these direct dependencies:

```toml
[dependencies]
oneagent-llm = { path = "../../crates/llm" }
oneagent-openai-compatible = { path = "../openai-compatible" }
reqwest = { version = "0.13.4", default-features = false, features = ["rustls"] }
serde = { version = "1.0.228", features = ["derive"] }
serde_json = "1.0.150"
tokio = { version = "1.53.0", features = ["macros", "time"] }

[dev-dependencies]
tokio = { version = "1.53.0", features = ["io-util", "macros", "net", "rt-multi-thread", "sync", "time"] }
```

No other direct production or dev dependency is accepted. Reqwest defaults and
the ADR-0046-rejected proxy, compression, charset, cookie, form, multipart,
blocking, HTTP/2, HTTP/3, native-TLS, SOCKS, DNS-replacement, and stream
features remain disabled. Native discovery uses `serde_json`, not reqwest's
`json` feature, so response bytes are bounded before decoding. Tokio production
features own only the biased operation/cancellation/total-timeout race; socket,
I/O, runtime, synchronization, and test-macro support remain dev-only.

These versions and external packages are already present in the approved
ADR-0046 workspace graph, but every line above creates a new direct dependency
edge or feature responsibility for `oneagent-lm-studio`. The ADR-0046 approval
does not authorize those new edges. Task 3 is prohibited until the user
explicitly approves this exact normal and dev dependency block. Cargo may then
add the workspace member and only mechanical lock changes; Task 3 must inspect
the manifest, feature graph, and lock diff before proceeding.

### Stable provider identity and public construction

The adapter provider ID is exactly the case-sensitive string `lm-studio`. Its
public construction is conceptually:

```rust
impl LmStudioProvider {
    pub fn new(
        configuration: ProviderConfiguration,
        base_url: &str,
    ) -> Result<Self, LlmError>;

    pub fn new_local(
        configuration: ProviderConfiguration,
    ) -> Result<Self, LlmError>;

    pub fn id(&self) -> &ProviderId;
}
```

`new_local` delegates to `new` with the exact root
`http://127.0.0.1:1234`. Numeric IPv4 loopback avoids DNS and is the only
default. There is no `Default` implementation because provider configuration
and optional credential ownership remain explicit.

Both constructors consume `ProviderConfiguration`. Its provider ID must equal
`lm-studio`; mismatch returns `InvalidConfiguration` before URL/client work.
`LlmProvider::id()` returns the accepted owned LM Studio ID.

The public type is `Send + Sync` but does not implement `Clone`, `Debug`,
`Display`, serialization, equality, or hashing. It exposes no client, endpoint,
header, composed provider, wire value, or generic-provider identity. Construction
performs no network I/O, model discovery, endpoint probe, server start, or model
operation.

### Server-root and locality contract

`base_url` denotes one server origin root, not `/api`, `/api/v1`, `/v1`, or an
endpoint. `new` applies the exact ADR-0046 root validation contract:

1. require `1..=2_048` UTF-8 bytes;
2. reject leading/trailing Unicode whitespace and every Unicode control
   character without echoing the value;
3. parse one absolute URL;
4. require scheme exactly `http` or `https` after parsing;
5. require a non-empty host and permit an explicit port;
6. reject username, password, query, and fragment;
7. require path to be empty or exactly `/`;
8. normalize the private root path to `/`;
9. join exact relative path `api/v1/models` and verify the scheme, host, and
   effective port remain unchanged;
10. pass the same original root to `OpenAiCompatibleProvider::new`, which joins
    and retains its unchanged `/v1/models` and `/v1/completions` endpoints.

The explicit constructor accepts HTTP or HTTPS roots with any URL-parser-valid
host; it therefore supports a caller-selected local or network-exposed LM
Studio server but makes no locality, confidentiality, or security claim. Only
`new_local` fixes numeric loopback. The adapter does not resolve DNS to prove
locality, read LM Studio settings, inspect interfaces, or rewrite `localhost`.

URL-library scheme/host canonicalization remains private transport state and
never affects provider or model identity. Invalid roots map to
`InvalidConfiguration` with closed static diagnostics that contain no URL,
host, port, parser source, or credential.

### Construction composition and client policy

After provider-ID and root validation, construction:

1. borrows the optional non-cloneable secret only long enough to build one
   sensitive native Authorization header;
2. builds one native reqwest client with redirects disabled, `no_proxy()`, and
   static user agent `oneagent-lm-studio/0.1.0`;
3. creates a private `ProviderConfiguration` with static provider ID
   `openai-compatible` and moves the same secret into it;
4. constructs one unchanged `OpenAiCompatibleProvider` from the same root;
5. retains the LM Studio ID, native client, native discovery URL, native header,
   and composed provider only after every step succeeds.

An impossible failure to construct the accepted static internal provider ID
maps to `Internal`; root, header, or either client-construction failure maps to
`InvalidConfiguration`. No partial successful provider escapes.

The native client has no client-level connect, read, or total timeout; no
cookies, decompression, pool tuning, DNS override, local bind, interface
selection, custom TLS root, client certificate, or invalid-certificate mode.
Redirects are terminal, authorization is never forwarded to another location,
and environment/files cannot configure proxy, root, credential, TLS, timeout,
or retry. HTTP is an explicit caller choice; HTTPS uses the same reqwest Rust
TLS and platform-verifier policy as ADR-0046.

Generation uses the composed provider's unchanged client and exact user agent
`oneagent-openai-compatible/0.1.0`. The two clients do not share cookies,
mutable configuration, caches, tasks, or endpoint state.

### Bearer authentication and sensitive-data boundary

An absent `ProviderSecret` sends no Authorization header on either endpoint. A
present secret becomes exactly `Authorization: Bearer <secret>` in both the
native discovery client and composed generic provider. Header conversion occurs
during construction; an invalid HTTP header value maps to
`InvalidConfiguration` without exposing the raw value. Header values are marked
sensitive and inserted explicitly into each request, never into URLs or bodies.

The complete sensitive boundary includes:

- server and endpoint URLs, hostnames, authorization, all headers, and secret
  copies;
- native catalog bodies and DTOs, including model keys, loaded instance IDs,
  paths, metadata, and additions;
- request input, generated output, completion bodies/DTOs, provider error
  objects, usage, stats, timing, and library error sources;
- every synthetic sentinel used by tests.

None may appear through provider/error `Debug`, `Display`, `source`, panic text,
diagnostics, logs, fixtures containing real data, or snapshots. Public errors
retain ADR-0045 generic formatting. Diagnostics use only a closed static English
vocabulary plus decimal HTTP status where accepted and remain within 512 bytes.
Explicit successful `input()` and `output()` access remains ADR-0045 behavior,
not logging authorization.

Construction and generation create unavoidable bounded in-memory secret,
header, input, and output copies. They are dropped before the operation returns;
memory zeroization and prevention of transport-buffer copies are not claimed.

### Native model-discovery request

Every `discover_models` call ultimately performs exactly one fresh request:

```text
GET /api/v1/models
Accept: application/json
Authorization: Bearer ...   # only when configured
User-Agent: oneagent-lm-studio/0.1.0
```

It sends no body, content header, cache/conditional header, endpoint probe,
retry, fallback, merge, implicit load, or preceding generic discovery. There is
no catalog cache, refresh task, staleness value, or global registry.

### Native discovery wire contract

A successful response must be one JSON object with this private logical shape:

```text
models: required array

each model entry:
  type:             required string, exactly "llm" or "embedding"
  key:              required string
  loaded_instances: required array

each loaded instance:
  id: required string
```

Unknown fields at the top level, model-entry level, and loaded-instance level
are ignored only after all required fields and types validate. `key` is required
wire evidence for the downloaded model but is not a shared identity and is not
validated as `ModelId`. Each loaded-instance `id` must be a string for both
documented model types; only IDs belonging to `llm` entries undergo shared
`ModelId` validation and projection.

Missing or mistyped required fields, a non-object entry/instance, malformed or
trailing JSON, a partial body, or a type other than exact lowercase `llm` or
`embedding` maps to `Protocol`. An unknown future type rejects the complete
catalog rather than being silently omitted or reinterpreted. Unknown additional
fields remain ignored and cannot become provider-neutral metadata.

The source model-entry and embedding-instance counts have no independent shared
limit beyond the 1 MiB encoded response bound. Only projected LLM instances
participate in the ADR-0045 catalog maximum.

### Discovery projection, identity, and loaded state

Projection is all-or-nothing:

1. validate the complete required wire shape;
2. ignore every well-formed `embedding` entry without assigning a capability;
3. let an `llm` entry with zero loaded instances contribute no model;
4. preserve each `llm` `loaded_instances[].id` byte-for-byte through
   `ModelId::new`;
5. create one descriptor scoped to provider `lm-studio` with exactly
   `ModelCapability::TextGeneration` for each accepted loaded instance;
6. reject more than 1,024 projected instances before success;
7. construct the complete result through `ModelCatalog::new` for canonical
   full-identity ordering and exact duplicate rejection.

An invalid projected ID, duplicate projected ID within or across parent
entries, or projected count over 1,024 maps to `InvalidModelCatalog`. Parent
`key` equality is not a shared ambiguity and receives no uniqueness rule;
projected loaded-instance identity is authoritative. An empty native list, an
embedding-only list, and a list of only unloaded LLMs each produce a successful
empty catalog.

The adapter never advertises a downloaded-but-unloaded model, parent model key,
embedding model, unknown type, or invalid/ambiguous instance as text-capable.
It owns no download, load, unload, JIT, TTL, auto-evict, or alias policy. A model
may be unloaded after discovery; a later generation failure is mapped from that
single request and does not trigger discovery, loading, fallback, or retry.

### Accepted generation endpoint and compatibility limit

LM Studio generation deliberately uses the composed adapter's exact
non-streaming:

```text
POST /v1/completions
```

ADR-0046's request/response/status/body/timeout/cancellation contract applies
unchanged after translation. This accepts only a narrow wire-compatible raw-text
slice. It does not claim chat-model suitability, prompt-template application,
response quality, or broad LM Studio compatibility. A discovered loaded LLM is
wire-eligible, but the provider-neutral domain has no base-versus-chat model
capability and the adapter does not guess one.

Native `/api/v1/chat` and OpenAI-compatible `/v1/chat/completions` are not
fallbacks. Failure of `/v1/completions` is one terminal failure.

### Generation bridge and terminal mapping

`generate` first rejects a request whose provider ID differs from `lm-studio`
with `InvalidRequest` before cancellation, allocation, or I/O. For an accepted
request, the bridge:

1. rejects already-requested cancellation before translation;
2. creates a temporary descriptor with provider `openai-compatible`, the exact
   cloned request `ModelId`, and exactly `TextGeneration`;
3. creates one temporary `TextGenerationRequest` containing the exact copied
   input and identical `max_output_bytes`;
4. calls the composed `OpenAiCompatibleProvider::generate` exactly once with
   the same `ProviderExecutionContext`;
5. receives only a generic result already validated against its temporary
   request;
6. on success, copies the validated output and `FinishReason` into
   `TextGenerationResponse::new` against the original LM Studio request;
7. returns the generic failure unchanged or one newly bound LM Studio success.

The delegated request therefore serializes exactly ADR-0046's fields:

```json
{
  "model": "<exact loaded LM Studio instance id>",
  "prompt": "<exact request input>",
  "max_tokens": 1,
  "stream": false
}
```

The displayed `1` is replaced by the exact numeric `max_output_bytes`. It is a
conservative provider token ceiling, not token/byte equivalence. The local byte
bound remains authoritative.

The successful wire response must remain exact ADR-0046
`object=text_completion`, exact response model, one choice at index zero,
non-empty bounded text, and finish `stop` or `length`. The composed provider
maps `stop` to `Completed`, `length` to `OutputLimit`, rejects every model/
choice/index/finish/output violation, ignores provider usage/stats/additions,
and computes no provider token authority. Rebinding preserves the original
`lm-studio` identity and recomputes exact local input/output UTF-8 byte usage.

There is no catalog lookup, implicit discovery, model rewrite, alias, fallback,
prompt prefix/template, role, message, history, sampling field, provider
extension, or second request in the bridge.

### Body and allocation bounds

Native discovery reuses ADR-0046's exact successful response limit of
1,048,576 bytes (1 MiB). An available `Content-Length` over the limit is
rejected before body reading. Regardless of that header, chunks are read with
checked addition and the operation stops before appending bytes over the limit.
An exceeded limit or checked-add overflow maps to `Protocol`.

Delegated generation retains ADR-0046's exact 524,288-byte (512 KiB) serialized
request and successful response bounds. The bridge adds only ADR-0045-bounded
copies: one model ID, at most 65,536 input bytes, and at most 65,536 output
bytes. Output remains limited by the original request's inclusive bound.

No unbounded whole-body helper is allowed. Successful JSON is parsed only after
complete bounded reading. For non-success status, neither client reads or parses
the response body; it is dropped immediately.

### Error and status mapping

The new leaf uses the ADR-0045 closed error vocabulary. Native discovery maps:

| Condition | `LlmErrorKind` |
|---|---|
| Invalid provider ID, URL, header, or client construction | `InvalidConfiguration` |
| HTTP 408, 429, or 500 through 599 | `ProviderUnavailable` |
| Every other non-success status, including redirects and other 3xx/4xx | `ProviderRejected` |
| DNS, connect, TLS, request write, header/body read, or premature close | `Transport` |
| Malformed/partial/trailing JSON, missing/mistyped fields, unknown type, or successful body over 1 MiB | `Protocol` |
| Invalid, duplicate, or over-count projected LLM instance identities | `InvalidModelCatalog` |
| Total timeout wins | `Timeout` |
| Existing or in-flight cancellation wins | `Cancelled` |
| Impossible static identity or private invariant failure | `Internal` |

Generation provider mismatch maps to `InvalidRequest`. Every other generation
condition preserves the exact ADR-0046 mapping, including request wire bound,
status, redirect, transport, protocol, terminal response, timeout,
cancellation, and internal failures. The final rebind can return only
`InvalidResponse` for an unexpected output-bound/accounting invariant.

Status diagnostics may contain only `provider returned HTTP NNN`. All other
diagnostics are closed static English strings. Native/generic bodies, Serde and
reqwest sources, URLs, model values, and sensitive content are never copied
into `LlmError`.

There is exactly one network attempt per operation and no automatic retry for
any error. `LlmErrorKind::is_retryable()` remains caller-visible classification
only.

### Timeout, cancellation, precedence, and cleanup

Native discovery uses the same execution semantics as ADR-0046:

1. reject already-requested cancellation before request work;
2. start exactly one operation future;
3. race it against `CancellationSignal::cancelled()` and, when configured, one
   Tokio total timer;
4. when branches are ready in the same poll, cancellation wins, then operation
   result, then timeout;
5. otherwise the first observed branch wins and later events cannot replace it;
6. drop losing futures, request/response state, buffers, DTOs, and header copies
   before returning one terminal result.

The native total timeout covers request creation, DNS, connect, TLS, write,
headers, all body chunks, JSON decoding, projection, and catalog construction.
An absent timeout creates no timer. There is no separate connect/read timeout,
spawned task, detached reader, or background work.

Generation preserves provider mismatch before cancellation. Its bounded
synchronous descriptor/request translation occurs after pre-cancellation and
before the delegated operation; the composed provider owns the single
in-flight cancellation and total-timer race. Post-success rebinding is also
bounded and synchronous. Cancellation becoming observable during translation is
seen by the composed provider before its HTTP work. The delegated total timeout
covers the complete ADR-0046 operation but does not claim to preempt the
non-yielding bounded pre/post translation.

Dropping reqwest futures is cooperative client-side abort. Neither adapter
claims server-side generation termination after connection closure. Every
terminal path returns no partial catalog/response and leaves no adapter-owned
active operation.

### Deterministic conformance and compatibility

No acceptance test contacts LM Studio or external network. Synthetic tests use
controlled servers bound only to `127.0.0.1:0`. The harness announces readiness
without sleeps, accepts an exact finite request count, bounds captured input,
writes complete or deliberately partial responses, joins deterministically,
and proves zero surviving adapter state.

Focused and public evidence must cover:

- exact ID, explicit/default root, URL bounds/components, local default,
  HTTP/HTTPS, optional/invalid bearer, endpoint joining, no-I/O construction,
  Send/Sync, client policy, dependency features, and redaction;
- exact native discovery method/path/headers/no-body, empty/mixed/reordered/
  maximum catalogs, multiple/custom loaded instances, embedding and unloaded
  exclusion, unknown fields, unknown type, required-field/type failures,
  malformed/trailing/partial JSON, invalid/duplicate/over-count IDs, status,
  redirect, advertised/streamed body bounds, transport, timeout, cancellation,
  one request, cleanup, and repeated fresh calls;
- exact bridge model/input/output bound, generic request field set, Unicode,
  maximum escaping, response identity, `stop`/`length`, local byte usage,
  unknown additions, provider usage/stats ignore, malformed/missing/mistyped/
  partial bodies, choice/index/finish/output violations, status, redirect,
  request/response bounds, transport, timeout, cancellation, one request,
  cleanup, and repeated fresh generation;
- sentinel absence from every implicit format and diagnostic;
- complete existing `oneagent-openai-compatible` unit/public conformance and
  `oneagent-llm` tests, provider substitution through `&dyn LlmProvider`,
  dependency direction, and analysis/Runtime library compatibility.

The public non-zero Sprint 25 conformance target uses only exported
`LmStudioProvider` and `oneagent-llm` APIs. Live availability, installed models,
credentials, local paths, timing thresholds, output text, and quality are not
acceptance evidence.

### Compatibility, documentation, and Coverage

Sprint 25 does not change `oneagent-llm`, `oneagent-openai-compatible`,
`oneagent-analysis`, `oneagent-runtime`, `oneagent-protocol`, CLI, semantic
graph, source adapters, or current Coverage Registries. The generic adapter
must retain its exact public type, provider ID, URLs, wire fields, feature set,
errors, tests, and independent support claim.

Architecture acceptance alone does not mark LM Studio supported. After Tasks
3-5 implement the complete provider, Task 6 adds public conformance and updates
only truthful current-state text in `README.md`, `docs/Architecture.md`, and
`docs/architecture/semantic-model-2.md`. No semantic Coverage capability is
added or transitioned. Sprint 25 remains incomplete until independent review
passes the complete workspace gate.

## Rejected alternatives

### Configure the generic adapter directly for LM Studio

Rejected because `/v1/models` lacks the type discriminator and can expose an
embedding model. ADR-0046 would assign `TextGeneration` to every entry.

### Generalize `OpenAiCompatibleProvider` to arbitrary provider IDs

Rejected because its stable identity and response/catalog scoping are accepted,
implemented public behavior. Weakening them would reopen ADR-0046 and its
consumer contract.

### Export generic private client, execution, DTO, or test helpers

Rejected because it would create a new reusable transport API, couple LM Studio
to generic wire internals, and expand the completed adapter's public surface.
Current public composition is sufficient.

### Extract a shared HTTP-provider transport crate

Rejected because it would add a broader abstraction and migrate a completed
adapter without a second proven consumer contract. The small native discovery
surface does not justify that change.

### Implement an entirely independent LM Studio adapter

Rejected because it would duplicate the complete accepted completion request,
response, identity, body, status, timeout, cancellation, redaction, and cleanup
implementation. Public composition reuses that behavior without changing it.

### Discover through `/v1/models`

Rejected because missing type information makes embedding exclusion impossible
and downloaded/JIT-visible models do not establish the selected loaded-instance
boundary.

### Project parent model keys or unloaded models

Rejected because custom loaded-instance identifiers are the documented API
`model` value and parent keys would import download/JIT/model-lifecycle policy.
Only loaded LLM instance IDs are projected.

### Ignore an unknown native model type

Rejected because silent omission could hide an incompatible vocabulary change.
The strict first slice rejects the complete wire response as `Protocol`.

### Use native chat or OpenAI-compatible chat completions

Rejected because native chat has heterogeneous output/state and no accepted
closed finish mapping, while chat completions imports roles, messages, and
prompt-template semantics excluded from ADR-0045's raw-text first slice.

### Add a prompt template for chat-tuned models

Rejected because OneAgent has no accepted prompt, role, tokenizer, or
model-family policy. The legacy completion limitation is explicit rather than
hidden behind guessed transformation.

### Restrict every explicit root to verified loopback

Rejected because correct DNS/IP locality validation has no repository evidence
and LM Studio officially permits network serving. `new_local` supplies one
deterministic numeric-loopback choice; `new` remains explicit caller authority.

### Add probing, fallback, retry, discovery cache, or model loading

Rejected because those behaviors add requests, implicit mutable state, replay,
freshness, selection, and lifecycle ownership absent from ADR-0045 and Sprint
25.

### Require live LM Studio acceptance

Rejected because installed versions, server state, credentials, model state,
outputs, latency, and quality are mutable developer-local data. The sanitized
investigation observations are supplementary evidence only.

## Consequences and risks

- One new leaf crate and new direct dependency edges are added only after
  explicit approval; the external package versions/features do not broaden the
  existing workspace graph.
- Native discovery cannot expose embedding-only or unloaded models as
  `TextGeneration`, and custom loaded-instance identifiers remain exact.
- Generic generation behavior and conformance are reused without changing the
  generic adapter, at the cost of two private HTTP clients and bounded
  translation copies.
- `/v1/completions` is legacy and may produce unsuitable output for chat-tuned
  models. Sprint 25 supports a wire contract, not model quality or prompt
  compatibility.
- The byte bound used as `max_tokens` can permit more provider work than the
  final byte response accepts. Local output validation remains authoritative.
- Explicit HTTP may be unencrypted; explicit network roots may be non-local;
  HTTPS uses platform trust only.
- Strict unknown-type rejection can make discovery fail when LM Studio adds a
  new model category. This is preferred to silently changing capability claims.
- Cancellation is cooperative and does not prove server-side termination.
- Secret/header/request/response copies can exist in memory without guaranteed
  zeroization; no security certification is claimed.

## Implementation prerequisites and order

1. Obtain explicit user approval for the exact new normal and dev dependency
   block recorded above.
2. Task 3 creates the workspace member, manifest/lock changes, public provider
   type and constructors, native URL/client/header policy, private DTO and
   execution foundation, composed provider, Rustdoc, and focused construction
   evidence. It performs no discovery or generation and does not implement a
   partial `LlmProvider` contract.
3. Task 4 implements private native discovery execution and focused
   controlled-loopback evidence. It adds no generation and no temporary public
   discovery API.
4. Task 5 implements the generation bridge and the complete public
   `LlmProvider` implementation, exposing the already-tested discovery through
   the trait and adding focused generation/provider-substitution evidence.
5. Task 6 adds the public conformance target, dependency/redaction audits,
   complete generic/provider-neutral/consumer compatibility, full workspace
   validation, and truthful current-state documentation.
6. Task 7 independently reviews the complete planning-through-Task-6 range
   before any Sprint state transition or previous-suite retirement.

If public composition cannot preserve exact request/response identity,
timeout/cancellation precedence, no-retry behavior, redaction, or cleanup, stop
before exporting generic internals, adding a dependency, or weakening ADR-0045
or ADR-0046.

## Deferred scope

- LM Studio installation, GUI/daemon/server lifecycle, model download,
  load/unload, JIT, TTL, auto-evict, and local configuration discovery;
- native chat, chat completions, Responses, Anthropic compatibility, streaming,
  tools, MCP, structured output, reasoning, vision, embeddings, reranking, and
  shared provider metadata;
- messages/roles, prompt templates/policy, conversations/history, sampling,
  tokenization, provider usage/stats authority, cost, quality, latency,
  performance, and broad compatibility/security claims;
- model selection, aliases, fallback, retry/backoff, rate limiting, concurrency
  policy, cache/refresh, persistence, registry, and endpoint negotiation;
- environment/file/CLI/keychain configuration, proxy configuration, custom TLS
  roots, client certificates, insecure TLS, reload, and secret zeroization;
- Runtime registration/routes/lifecycle, Context orchestration, protocol/CLI,
  MCP, LSP, IDE, UI, graph mutation, source adapters, and Coverage changes;
- live LM Studio, downloaded models, credentials, local paths, or generated
  output as CI/review acceptance.

## Completion criteria

Sprint 25 can complete only after dependency approval is recorded; Tasks 3-6
implement and prove every accepted construction, discovery, composition,
identity, body/output bound, status, protocol, redaction, timeout, cancellation,
cleanup, compatibility, and documentation rule without external state; the
canonical full workspace gate succeeds; and the Sprint 25 integration review
records `pass` or `pass with non-blocking follow-ups`.
