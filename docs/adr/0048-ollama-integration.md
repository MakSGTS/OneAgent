# ADR-0048: Ollama Provider Integration

## Status

Accepted

Implementation remains gated on explicit user approval of every direct
dependency edge and feature set recorded in this ADR. Acceptance changes no
current support claim and does not authorize Cargo or production Rust changes.

## Context

Sprint 26 must add one bounded local Ollama provider behind ADR-0045's
provider-neutral `LlmProvider` seam without weakening the implemented and
reviewed ADR-0046 and ADR-0047 concrete adapters.

The [Ollama integration investigation](../architecture/ollama-integration-investigation.md)
confirms:

- `oneagent-llm` owns the complete provider-neutral identity, catalog, bounded
  text, usage, finish, error, timeout, cancellation, and substitution contract;
- the OpenAI-compatible and LM Studio adapters expose only their concrete
  provider types; their clients, execution helpers, DTOs, and test harnesses are
  private and their accepted contracts cannot be generalized for Ollama;
- official `/api/tags` and `/v1/models` schemas do not declare model
  capabilities, while official `POST /api/show` returns a capability list;
- official model summaries distinguish remote-backed entries with
  `remote_model` and `remote_host`; a signed-in local Ollama may transparently
  send a cloud-model generation request;
- the authorized Ollama 0.33.0 catalog supplemented `/api/tags` with
  capabilities and contained one remote-backed entry, but that mutable addition
  is not a cross-version repository oracle;
- native `/api/generate` can represent one non-streaming raw text request and
  terminal exact-model response without importing roles/messages, while
  provider template, thinking, context, timing, token, and model-lifecycle
  additions can be explicitly disabled or ignored;
- controlled loopback can prove the complete first slice without installed
  Ollama, a model, credential, cloud service, or external network.

ADR-0045 remains authoritative for every provider-neutral behavior. ADR-0046
and ADR-0047 remain authoritative for the existing concrete leaves. This
decision owns only the new local Ollama leaf, its bounded native Tags/Show
discovery, native raw generation mapping, and conformance evidence.

## Decision

### Canonical ownership and dependency direction

Create workspace library package `oneagent-ollama` at `adapters/ollama`. It
owns:

- `OllamaProvider`, the only public concrete adapter type;
- stable provider identity and deterministic numeric-loopback construction;
- one private native HTTP client, three endpoint URLs, and private wire DTOs;
- bounded Tags/Show discovery, native raw generation, execution races, redacted
  errors, and focused/public controlled-loopback evidence.

Dependency direction is strictly:

```text
future composition/callers -> oneagent-ollama -> oneagent-llm -> std
```

`oneagent-ollama` has no dependency on another concrete provider, Analysis,
Runtime, protocol, CLI, graph, workspace, or a source adapter. Existing crates
remain unchanged, and no current consumer gains an Ollama dependency in Sprint
26.

### Exact dependency set and approval gate

The package manifest must use exactly:

```toml
[dependencies]
oneagent-llm = { path = "../../crates/llm" }
reqwest = { version = "0.13.4", default-features = false }
serde = { version = "1.0.228", features = ["derive"] }
serde_json = "1.0.150"
tokio = { version = "1.53.0", features = ["macros", "time"] }

[dev-dependencies]
tokio = { version = "1.53.0", features = ["io-util", "macros", "net", "rt-multi-thread", "sync", "time"] }
```

No other direct normal or dev dependency is accepted. Reqwest defaults remain
disabled. The local-only provider enables no TLS backend, proxy, compression,
charset, cookie, form, multipart, blocking, HTTP/2, HTTP/3, SOCKS, DNS
replacement, JSON, or stream feature. `serde_json` owns private serialization
and parsing after byte bounds. Tokio production features own only the biased
operation/cancellation/total-timeout race; listener, I/O, runtime,
synchronization, and test macro features remain dev-only.

Every external version is already present in the workspace, but this block
creates new direct dependency edges and a new feature responsibility for
`oneagent-ollama`. Prior approvals for other packages do not authorize them.
Task 3 is prohibited until the user explicitly approves this exact normal and
dev dependency block. Cargo may then add the workspace member and only
mechanical lock changes; Task 3 must inspect manifest, feature graph, and lock
diffs.

### Stable identity and public construction

The provider ID is exactly case-sensitive `ollama`. Public construction is:

```rust
impl OllamaProvider {
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

`new_local` delegates to `new` with exactly
`http://127.0.0.1:11434`. There is no `Default`; provider configuration remains
explicit. The configuration provider ID must equal `ollama`. A mismatch
returns `InvalidConfiguration` before secret, URL, or client work.

The local first slice accepts no credential. A present `ProviderSecret` returns
`InvalidConfiguration` with a static diagnostic before URL/client work and is
dropped without exposure. Local Ollama requires no authentication; direct
cloud API keys and reverse-proxy authentication remain deferred.

The public type is `Send + Sync` and implements no `Clone`, `Debug`, `Display`,
serialization, equality, or hashing. It exposes no client, URL, header, DTO,
model metadata, or execution helper. Construction performs no I/O, endpoint
probe, daemon start, discovery, model operation, or cloud request.

### Numeric-loopback root contract

`base_url` denotes the local Ollama server origin root, not `/api` or an
endpoint. Validation is exact:

1. require `1..=2_048` UTF-8 bytes;
2. reject leading/trailing Unicode whitespace and every Unicode control
   character without echoing the value;
3. parse one absolute URL;
4. require scheme exactly `http`;
5. require host exactly the numeric IPv4 literal `127.0.0.1` and permit one
   explicit port;
6. reject username, password, query, and fragment;
7. require path empty or exactly `/`;
8. normalize the private root path to `/`;
9. join exact relative paths `api/tags`, `api/show`, and `api/generate`;
10. verify every joined URL retains scheme, host, and effective port.

`localhost`, IPv6, wildcard, DNS, Unix sockets, arbitrary local interfaces,
HTTPS, remote origins, and direct `ollama.com` are rejected. Callers that need
another address require a later architecture extension. This narrow contract
makes destination policy deterministic and testable; it does not inspect DNS,
interfaces, environment, server settings, sign-in state, or model storage.

URL canonicalization is private transport state and never identity. Invalid
roots map to `InvalidConfiguration` with closed static diagnostics containing
no input URL, host, port, parser source, or secret.

### HTTP client policy

Build one reqwest client with:

- redirect policy `reqwest::redirect::Policy::none()`;
- `no_proxy()`;
- exact static user agent `oneagent-ollama/0.1.0`;
- no default authorization or content header;
- no client-level connect/read/total timeout;
- no cookies, decompression, pool tuning, DNS override, local bind, interface
  selection, or TLS configuration.

All requests remain numeric-loopback HTTP. Redirects are terminal and cannot
change destination. Environment and files cannot configure destination, proxy,
credential, timeout, retry, or any provider behavior.

### Sensitive-data and redaction boundary

The sensitive or unrestricted boundary includes:

- input and endpoint URLs, all headers, and rejected secrets;
- Tags and Show bodies/DTOs, names, model IDs, remote fields, digests, details,
  templates, licenses, parameters, and capabilities;
- generation request input, output, thinking, context, timing, token, log-prob,
  model, and error payloads;
- reqwest/Serde sources and every test sentinel.

None may appear through provider/error `Debug`, `Display`, `source`, panic text,
diagnostics, logs, real-data fixtures, or snapshots. Public errors preserve
ADR-0045 generic formatting. Diagnostics use only a closed static English
vocabulary plus decimal HTTP status and remain within 512 bytes. Provider error
bodies are never read on non-success status. Explicit successful `input()` and
`output()` access remains ADR-0045 behavior, not logging authorization.

Bounded request and response copies are dropped on terminal return. Memory
zeroization and prevention of unavoidable HTTP-buffer copies are not claimed.

### Native Tags request and response

Every discovery call begins with exactly one fresh:

```text
GET /api/tags
Accept: application/json
User-Agent: oneagent-ollama/0.1.0
```

It sends no body, Authorization, content header, cache/conditional header,
probe, retry, fallback, or merge.

A successful response is one JSON object with required `models` array. Each
entry requires:

```text
name:        required string
model:       required string
remote_model: optional string
remote_host:  optional string
```

Unknown fields, including a Tags-level `capabilities` addition, are ignored
only after required fields validate. Both identity strings must pass
`ModelId::new` and be byte-for-byte equal. Conflict maps to `Protocol`; invalid
shared identity maps to `InvalidModelCatalog`.

Remote markers must be both absent or both present as non-empty strings.
Partial, null, empty, or mistyped markers map to `Protocol`. A well-formed
remote-backed entry is validated but omitted before Show; no request uses its
model ID. This ensures the local first slice does not intentionally trigger
transparent Ollama cloud access.

The source `models` array contains at most 1,024 entries, including remote
entries. Over-count maps to `InvalidModelCatalog` before any Show call. Local
IDs are sorted byte-for-byte and exact duplicates are rejected before Show.
An empty Tags list and a remote-only list produce a successful empty catalog.

### Capability-safe Show sequence

For every canonical local candidate, discovery sends one sequential exact:

```text
POST /api/show
Content-Type: application/json
Accept: application/json
User-Agent: oneagent-ollama/0.1.0

{"model":"<exact model id>","verbose":false}
```

The private serializer preserves the exact model bytes and field set. Show
order is canonical model-ID order, independent from Tags order. At most 1,024
Show requests follow one Tags request. There is no concurrency, cache, retry,
fallback, alternate endpoint, or request for a remote entry.

A successful Show response requires a `capabilities` array of strings. Unknown
top-level fields are ignored. Capability values are treated as an unordered
set: duplicate values do not change meaning. Exact lowercase `completion`
contributes one `ModelDescriptor` with exactly `TextGeneration`; a well-formed
array without it contributes no descriptor. Unknown capability strings,
`embedding`, `tools`, `thinking`, and `vision` are ignored because they have no
shared first-slice destination. Missing, null, or mistyped `capabilities` or a
non-string entry maps to `Protocol` and rejects the whole discovery call.

All Tags candidates and all required Show responses validate before
`ModelCatalog::new` constructs one canonical all-or-nothing result. A later
Show failure returns no partial catalog. Tags capabilities never replace Show.

### Discovery bounds and aggregate work

Exact inclusive bounds are:

| Value | Maximum bytes/count |
|---|---:|
| Tags successful response body | 1,048,576 bytes |
| one Show serialized request body | 4,096 bytes |
| one Show successful response body | 1,048,576 bytes |
| Tags entries / Show requests / projected models | 1,024 |

Available `Content-Length` is rejected before successful body reading when it
exceeds the endpoint limit. Regardless of the header, chunks use checked
addition and stop before appending bytes over the limit. Bodies are parsed only
after complete bounded reading and trailing non-whitespace JSON is rejected.

Only one Tags or Show response body and one decoded DTO are retained at a time.
Accepted descriptors are bounded by ADR-0045. The one provider-neutral total
timeout bounds the entire Tags plus sequential Show operation; there is no
independent per-request retry or timer.

### Native generation request

`generate` rejects a request whose provider differs from `ollama` with
`InvalidRequest` before cancellation, serialization, or I/O. It then rejects
already-requested cancellation before request work.

The private serialized body contains exactly:

```json
{
  "model": "<exact request model id>",
  "prompt": "<exact request input>",
  "stream": false,
  "raw": true,
  "think": false,
  "options": {
    "num_predict": 1
  }
}
```

The displayed `1` is replaced by exact `max_output_bytes`. It is a conservative
token ceiling, not token/byte equivalence; local output-byte validation remains
authoritative. `raw=true` prevents a provider prompt template, `think=false`
prevents requested thinking output, and `stream=false` requires one JSON body.

No system, template, suffix, images, format, keep-alive, context, sampling,
seed, stop, log-probability, tool, chat, or provider extension is sent. Omitting
`keep_alive` leaves any implicit server load/retention behavior under server
policy; the adapter does not own or claim model lifecycle.

The operation is exactly one:

```text
POST /api/generate
Content-Type: application/json
Accept: application/json
User-Agent: oneagent-ollama/0.1.0
```

### Native terminal response

A successful response requires:

```text
model:       required string, exact originating request model
response:    required string
done:        required boolean, exactly true
done_reason: required string, exactly "stop" or "length"
thinking:    optional string, absent or empty
```

Unknown additions are ignored only after required fields validate. Malformed,
trailing, partial, missing, or mistyped fields map to `Protocol`. Invalid model
ID, response model mismatch, `done=false`, non-empty thinking, unknown finish,
empty output, or output beyond the original byte bound maps to
`InvalidResponse`.

Map `stop` to `FinishReason::Completed` and `length` to
`FinishReason::OutputLimit`. Construct success only through
`TextGenerationResponse::new` against the original request. Local exact UTF-8
input/output byte usage is authoritative. Provider context, timing, token
counts, log probabilities, and unknown additions are ignored and never become
shared metadata.

There is no implicit discovery, catalog lookup, remote-marker lookup, model
rewrite, alias, fallback, retry, prompt prefix/template, role/message, second
request, or cloud-state check during generation. Callers must use a model from
the latest catalog if they require the local-only discovery policy; the
provider seam itself cannot prove that an arbitrary validated request model is
still local.

### Generation bounds

The maximum serialized generation request is 524,288 bytes and the maximum
successful response body is 524,288 bytes. Request serialization occurs before
I/O and exceeding the request bound maps to `InvalidRequest`. Successful body
reading uses the same advertised and incremental checked-bound policy as
discovery. Over-bound success maps to `Protocol`.

The response bound accommodates worst-case JSON escaping of the shared maximum
output plus the fixed envelope. No unbounded whole-body helper is allowed.

### Error and status mapping

The mapping is exact for Tags, Show, and Generate:

| Condition | `LlmErrorKind` |
|---|---|
| Invalid provider ID, present secret, URL, endpoint, or client construction | `InvalidConfiguration` |
| Request provider mismatch or serialized generation/Show request over its bound | `InvalidRequest` |
| HTTP 408, 429, or 500 through 599 | `ProviderUnavailable` |
| Every other non-success status, including redirects and other 3xx/4xx | `ProviderRejected` |
| Connect, request write, response header/body read, or premature close | `Transport` |
| Malformed/partial/trailing JSON, missing/mistyped wire fields, name/model conflict, remote-marker conflict, or successful body over its endpoint limit | `Protocol` |
| Invalid/duplicate/over-count Tags identities | `InvalidModelCatalog` |
| Generate model/done/finish/thinking/output violates terminal semantics | `InvalidResponse` |
| Total timeout wins | `Timeout` |
| Existing or in-flight cancellation wins | `Cancelled` |
| Impossible static identity or private serialization/invariant failure | `Internal` |

Status diagnostics may contain only `provider returned HTTP NNN`. All other
diagnostics are closed static English strings. Provider bodies, URLs, model
values, remote fields, and library sources are never copied into `LlmError`.
There is exactly one attempt per planned wire step. Retryable classification
never causes replay.

### Timeout, cancellation, precedence, and cleanup

Discovery and generation use one logical operation race:

1. generation provider mismatch precedes cancellation;
2. already-requested cancellation returns before serialization or HTTP work;
3. start one complete operation future;
4. race it against `CancellationSignal::cancelled()` and, when configured, one
   Tokio total timer;
5. if branches are ready together, cancellation wins, then operation result,
   then timeout;
6. otherwise the first observed branch wins and later events cannot replace it;
7. drop the losing futures and all bounded temporary state before return.

The discovery operation future owns Tags plus every sequential Show request,
decoding step, projection, and final catalog construction. The generation
operation owns serialization, request creation, connect/write/headers/body,
decoding, mapping, and shared terminal construction. An absent timeout creates
no timer.

There is no spawned adapter task, detached reader, concurrent Show fan-out,
background catalog work, retry delay, or separate connect/read timeout.
Dropping reqwest futures is cooperative client-side abort. The adapter does not
claim server-side generation termination or model unload after connection
closure. Every terminal path returns no partial catalog/response and leaves no
adapter-owned active operation.

### Deterministic conformance and compatibility

No acceptance test contacts installed Ollama or external network. Synthetic
tests use a controlled server bound only to `127.0.0.1:0`; it announces
readiness without sleeps, accepts an exact finite request sequence, bounds
captured input, writes complete or deliberately partial responses, joins
deterministically, and proves zero surviving state.

Focused and public evidence must cover:

- exact ID, explicit/default numeric-loopback root, URL bounds/components,
  rejected credentials/remote roots, endpoint joins, no-I/O construction,
  Send/Sync, client policy, dependency features, and redaction;
- exact Tags and Show methods/paths/headers/body fields, canonical Show order,
  empty/local/remote/mixed/reordered/maximum catalogs, completion and
  non-completion capabilities, unknown additions/values, name/model conflicts,
  remote-marker conflicts, invalid/duplicate/over-count IDs, malformed/
  trailing/partial bodies, status, redirect, body/work bounds, transport,
  cancellation at every sequence position, total timeout, request count,
  cleanup, and repeated fresh discovery;
- exact native Generate model/prompt/stream/raw/think/num_predict field set,
  Unicode and maximum escaping, model identity, `done`, `stop`/`length`, empty
  thinking, local byte usage, ignored provider additions, malformed/missing/
  mistyped/partial bodies, mismatch/done/finish/thinking/output violations,
  status, redirect, request/response bounds, transport, timeout, cancellation,
  one request, cleanup, and repeated fresh generation;
- sentinel absence from every implicit format and diagnostic;
- complete OpenAI-compatible, LM Studio, and provider-neutral tests, provider
  substitution through `&dyn LlmProvider`, dependency direction, and Analysis/
  Runtime library compatibility.

The public non-zero Ollama conformance target uses only exported
`OllamaProvider` and `oneagent-llm` APIs. Live availability, model state,
credentials, cloud traffic, local files, timing thresholds, output content,
quality, and performance are not acceptance evidence.

### Documentation and Coverage completion

Sprint 26 changes no provider-neutral, existing-provider, Analysis, Runtime,
protocol, CLI, graph, workspace, source-adapter, or Coverage behavior. Current
semantic Coverage Registries have no LLM-provider capability and receive no
new entry or transition.

Architecture acceptance alone does not mark Ollama supported. After Tasks 3-5
implement the complete leaf, Task 6 adds public conformance and updates only
truthful current-state text in `README.md`, `docs/Architecture.md`, and
`docs/architecture/semantic-model-2.md`. Sprint 26 remains incomplete until an
independent review records a non-blocking decision after the full workspace
gate.

## Rejected alternatives

### Discover from `/api/tags` capabilities

Rejected because the current official ModelSummary schema does not declare
that field. The local 0.33.0 addition is mutable supplementary evidence.

### Discover from `/v1/models`

Rejected because the compatibility catalog does not prove completion versus
embedding capability and would silently inflate the shared capability claim.

### Infer capability from name, family, format, template, or metadata

Rejected because none is an exact completion-capability oracle. Official Show
provides the explicit value.

### Send Show for remote-backed entries

Rejected because a signed-in local daemon may transparently access cloud state.
The local first slice validates and excludes remote entries before Show.

### Run Show requests concurrently

Rejected because it introduces scheduling, concurrency bounds, cancellation
fan-out, ordering, and cleanup complexity without a first-slice requirement.
Canonical sequential Show requests are bounded and deterministic.

### Use composed `/v1/completions` generation

Rejected because native `/api/generate` directly represents a raw prompt,
allows prompt templating and thinking to be explicitly disabled, and avoids a
provider-identity bridge. The required native mapping is bounded and fully
testable; existing adapters remain regression targets rather than dependencies.

### Use chat or Responses endpoints

Rejected because they import roles/messages, prompt templates, heterogeneous
output, history/state, tools, reasoning, or other values outside ADR-0045.

### Accept arbitrary or HTTPS roots and optional bearer authentication

Rejected for the local first slice because it broadens destination, TLS, proxy
authentication, and direct-cloud behavior. Numeric IPv4 loopback HTTP is the
only proven destination needed for installed Ollama and controlled tests.

### Use a provider SDK or export another adapter's private transport

Rejected because either adds a broader dependency/API contract. The existing
bounded reqwest/Serde/Tokio pattern is sufficient without changing completed
providers.

### Treat model generation as proof that a model is local

Rejected because a local signed-in daemon can transparently invoke a cloud
model. Discovery filters declared remote markers; arbitrary direct generation
remains caller authority under the provider-neutral seam.

### Add retry, fallback, cache, lifecycle, or endpoint probing

Rejected because those add replay, mutable state, selection, additional
requests, and ownership absent from ADR-0045 and Sprint 26.

### Require live Ollama acceptance

Rejected because daemon/version/model/sign-in/cloud/output/latency state is
mutable. Sanitized local observations are supplementary only.

## Consequences and risks

- One independent native leaf and new direct dependency edges are added only
  after explicit approval; no new external version enters the workspace.
- Official Show evidence prevents embedding-only or ambiguous models from being
  advertised as text generation.
- Remote-backed Tags entries cannot trigger a Show request and are not exposed
  by the local catalog.
- Discovery may issue up to 1,025 sequential local requests. Count, per-body
  memory, and total elapsed time are bounded, but large local catalogs can be
  expensive.
- Numeric-loopback-only HTTP deliberately excludes Docker/VM/network/HTTPS and
  direct cloud configurations.
- Native `done_reason` is a provider string; the first slice accepts only
  evidence-backed `stop` and `length` and rejects future additions.
- `num_predict` is a token ceiling derived conservatively from a byte bound;
  token cost is not byte usage and oversized decoded output still fails.
- Omitting keep-alive leaves server-side model retention to Ollama; invoking a
  local model may implicitly load it, but the adapter owns no lifecycle policy.
- Cancellation is cooperative and does not prove server-side termination.
- Sensitive request/response copies are bounded but not zeroized.

## Implementation prerequisites and order

1. Obtain explicit user approval for the exact normal and dev dependency block
   in this ADR.
2. Task 3 creates the workspace member, manifest/lock changes, public provider
   type and constructors, numeric-loopback URL/client policy, private DTO and
   execution foundation, Rustdoc, and focused construction evidence. It
   performs no Tags, Show, or Generate operation and does not implement a
   partial public `LlmProvider`.
3. Task 4 implements the complete Tags/Show discovery operation and focused
   controlled-loopback evidence without generation.
4. Task 5 implements native Generate and the complete public `LlmProvider`,
   exposing the already-tested discovery through the trait.
5. Task 6 adds public conformance, complete existing-provider/provider-neutral/
   consumer compatibility, dependency/redaction audits, full validation, and
   truthful current-state documentation.
6. Task 7 independently reviews the planning-through-Task-6 range before any
   Sprint state transition or previous-suite retirement.

If reqwest without a TLS feature cannot provide exact numeric-loopback HTTP,
bounded chunks, redirects, no proxy, timeout/cancellation, or redaction, stop
before adding a feature or dependency. If native Ollama wire evidence cannot
preserve capability, identity, finish, bounds, or local/cloud policy, stop
before weakening ADR-0045 or silently switching endpoints.

## Deferred scope

- non-loopback, IPv6, localhost/DNS, Docker/VM/network, HTTPS, direct cloud, API
  keys, reverse-proxy authentication, custom TLS, and proxy configuration;
- Ollama installation, daemon/server lifecycle, upgrade, pull/create/copy/
  delete/push, storage, preload/load/unload, keep-alive policy, and model
  selection;
- remote-backed/cloud model discovery and execution, sign-in state, direct
  cloud traffic, live credentials, and broad version/model compatibility;
- chat/history, roles/messages, streaming, tools, MCP, structured output,
  thinking/reasoning, vision/images, embeddings, templates/prompt policy,
  provider metadata, token authority, sampling, cost, quality, and performance;
- retry/backoff, fallback, rate limiting, concurrency, cache/refresh, registry,
  persistence, health probing, and endpoint negotiation;
- environment/file/CLI/keychain configuration, reload, secret zeroization, and
  security certification;
- Runtime registration/routes/lifecycle, Context orchestration, protocol/CLI,
  MCP, LSP, IDE, UI, graph mutation, source adapters, and Coverage changes;
- installed/running Ollama, local/cloud models, credentials, external network,
  generated output, or response quality as CI/review acceptance.

## Completion criteria

Sprint 26 can complete only after dependency approval is recorded; Tasks 3-6
implement and prove every accepted construction, local/cloud boundary,
Tags/Show capability mapping, native generation, identity, bound, status,
protocol, redaction, timeout, cancellation, exactly-once-per-step, cleanup,
compatibility, and documentation rule without external state; the canonical
full workspace gate succeeds; and the Sprint 26 integration review records
`pass` or `pass with non-blocking follow-ups`.
