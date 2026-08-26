# LM Studio Integration Investigation

## Status and purpose

This document records the Sprint 25 evidence required for ADR-0047. It does
not accept architecture, change provider support, authorize dependencies, or
make live LM Studio a repository prerequisite.

The investigation separates:

- **confirmed repository evidence** from current code, manifests, tests, and
  Git history;
- **accepted constraints** from ADR-0045 and ADR-0046;
- **official LM Studio evidence** from documentation read on 2026-08-26;
- **sanitized local observations** explicitly authorized for the current macOS
  sprint launch;
- **unresolved decisions** that ADR-0047 must select before implementation.

## Confirmed repository baseline

The investigation ran from committed Sprint 25 planning baseline `da5cdbd5`
(`Plan Sprint 25 LM Studio Integration`) with a clean working tree.

### Provider-neutral ownership

`oneagent-llm` is the std-only provider-neutral owner. Its current public
boundary confirms:

- provider/model identities are case-sensitive, bounded, owned, cloneable, and
  ordered;
- `ModelCapability` contains only `TextGeneration`;
- `ModelCatalog` accepts zero through 1,024 provider-scoped descriptors, sorts
  them, and rejects duplicate identities atomically;
- `ProviderConfiguration::into_parts()` transfers one provider ID and optional
  non-cloneable `ProviderSecret` without exposing an implicit source;
- `TextGenerationRequest::new()` can construct a bounded request from a public
  descriptor and exact owned input;
- `TextGenerationResponse::new()` can bind bounded output and a copied
  `FinishReason` to the originating public request;
- `LlmProvider` exposes only fresh discovery and one terminal text-generation
  future with provider-neutral timeout/cancellation input.

There is no provider registry, endpoint value, model lifecycle, chat/message
domain, prompt policy, tokenizer, streaming value, provider metadata escape
hatch, or Runtime consumer.

### Existing OpenAI-compatible adapter

`oneagent-openai-compatible` is the only concrete provider package. Its public
surface exports only `OpenAiCompatibleProvider`; all client, URL, header, DTO,
execution, and test-support values are private.

Construction and execution are fixed by code and ADR-0046:

- provider ID must be exactly `openai-compatible`;
- construction consumes one explicit root URL and optional credential;
- the root is joined privately to `/v1/models` and `/v1/completions`;
- reqwest redirects and proxies are disabled;
- every discovery entry receives `TextGeneration` because the accepted generic
  `/v1/models` DTO contains only `id`;
- generation rejects another provider identity before I/O;
- one non-streaming completion maps exact model, prompt, output bound, response
  model, one choice, `stop`/`length`, output, errors, timeout, cancellation, and
  cleanup;
- public response construction and exact output/finish accessors are available
  after the generic operation succeeds.

The hard-coded provider ID means an LM Studio request cannot be passed directly
to the generic adapter. The generic discovery result also cannot support native
LM Studio type filtering because the provider-specific type was discarded
before the wrapper could inspect it.

### Current dependency and consumer inventory

The exact direct normal dependencies of `oneagent-openai-compatible` are:

```text
oneagent-llm
reqwest 0.13.4, default features disabled, rustls only
serde 1.0.228, derive only
serde_json 1.0.150
tokio 1.53.0, macros and time only
```

Its dev-only Tokio features are `io-util`, `macros`, `net`,
`rt-multi-thread`, `sync`, and `time`.

Current reverse normal dependency inspection confirms:

- no package depends on `oneagent-openai-compatible`;
- only `oneagent-openai-compatible` depends on `oneagent-llm`;
- `oneagent-analysis`, `oneagent-runtime`, `oneagent-protocol`, and
  `oneagent-cli` do not consume either provider package.

No existing public consumer therefore requires migration in Sprint 25. The
existing adapter remains an independent compatibility target.

### Baseline validation

The required focused commands passed on 2026-08-26 with permission limited to
controlled loopback binds:

- `cargo test -p oneagent-openai-compatible --lib --offline` — 18 passed;
- `cargo test -p oneagent-openai-compatible --test conformance --offline` — 6
  passed;
- `cargo test -p oneagent-llm --offline` — 22 unit and 7 public integration
  tests passed; its doc-test target contained zero tests and is not counted as
  behavioral evidence.

## Accepted constraints from ADR-0045 and ADR-0046

ADR-0045 remains authoritative for:

- provider-scoped identities and canonical catalogs;
- one closed `TextGeneration` capability;
- one bounded raw text input and terminal bounded text output;
- local UTF-8 byte usage and closed `Completed`/`OutputLimit` finish reasons;
- explicit secret-safe construction inputs;
- closed typed errors and bounded redacted diagnostics;
- exactly one attempt, no automatic retry, optional total timeout, cooperative
  cancellation, and no partial result;
- no chat/history, roles/messages, prompt policy, provider token authority,
  model lifecycle, Runtime ownership, or live-provider acceptance.

ADR-0046 remains authoritative for the generic adapter. Sprint 25 cannot change
its stable ID, endpoints, public type, accepted request/response mapping,
dependency features, error policy, or conformance claims merely to simplify LM
Studio integration.

## Official LM Studio evidence

The following primary documentation was read on 2026-08-26:

- [REST API overview](https://lmstudio.ai/docs/developer/rest) — LM Studio 0.4.0
  introduced and recommends native `/api/v1/*` endpoints;
- [native model list](https://lmstudio.ai/docs/developer/rest/list) —
  `GET /api/v1/models` returns available LLM and embedding models, including
  `type`, model `key`, and `loaded_instances[].id`;
- [OpenAI-compatible model list](https://lmstudio.ai/docs/developer/openai-compat/models)
  — `GET /v1/models` returns models visible to the server and may include all
  downloaded models when Just-In-Time loading is enabled;
- [native chat](https://lmstudio.ai/docs/developer/rest/chat) —
  `POST /api/v1/chat` accepts string or item input, defaults `stream` to false
  and `store` to true, and returns a model-instance ID plus a heterogeneous
  output-item array and provider stats;
- [OpenAI-compatible chat completions](https://lmstudio.ai/docs/developer/openai-compat/chat-completions)
  — `POST /v1/chat/completions` accepts role-bearing messages and applies a
  prompt template for chat-tuned models;
- [legacy completions](https://lmstudio.ai/docs/developer/openai-compat/completions)
  — `POST /v1/completions` remains supported for base-model text completion,
  applies no prompt template, and may produce unexpected tokens with chat-tuned
  models;
- [server quickstart](https://lmstudio.ai/docs/developer/rest/quickstart) — the
  default root is `http://localhost:1234`, authentication is optional by
  default, and native chat may load a requested model automatically;
- [server boundary](https://lmstudio.ai/docs/developer/core/server) — LM Studio
  may serve on localhost or an explicitly enabled network interface;
- [authentication](https://lmstudio.ai/docs/developer/core/authentication) — API
  tokens require LM Studio 0.4.0 or newer and use a Bearer header when enabled;
- [CLI model loading](https://lmstudio.ai/docs/cli/local-models/load) — a loaded
  instance may receive a custom `--identifier`, and that identifier is then
  used by API `model` parameters;
- [CLI loaded-model list](https://lmstudio.ai/docs/cli/local-models/ps) —
  `lms ps` and its JSON form enumerate loaded instances.

The official material does not provide a stable server-version response field
for the inspected endpoints. API-v1 support proves an API family, not the exact
installed application build.

## Sanitized authorized local observations

The user authorized local LM Studio use on macOS for Sprint 25. No LM Studio
filesystem, model path, credential, configuration file, unrestricted model
content, or process log was inspected or stored.

The live audit recorded:

```text
lms CLI commit: 71bd99c
server: ON, 127.0.0.1:1234
loaded LLM: qwen/qwen3-4b
```

Only bounded structural projections were retained:

| Operation | Sanitized observation |
| --- | --- |
| `GET /v1/models` | `object=list`; two IDs; neither entry exposed a type discriminator. |
| `GET /api/v1/models` | One `type=llm`, `key=qwen/qwen3-4b`, one loaded instance; one `type=embedding` with zero loaded instances. |
| `POST /v1/completions` | Exact requested model, `object=text_completion`, one non-empty choice, `finish_reason=length`, usage fields, and provider-specific `stats`. |
| `POST /v1/chat/completions` | Exact requested model, one choice, `finish_reason=length`, assistant role, empty content at the eight-token bound, usage, and `stats`. |
| `POST /api/v1/chat` | With `stream=false`, `store=false`, `reasoning=off`, and an eight-token bound: exact model-instance ID, one non-empty `message` output item, stats, and no response ID. |

The local completion/chat input was a short synthetic instruction. Output text
was not retained in the document or command transcript projection. The
observations prove reachable wire shapes only. They do not prove stable output,
quality, performance, deterministic sampling, model compatibility, JIT
behavior, error mapping, or repository acceptance.

## Discovery evidence and mapping choices

### Confirmed generic incompatibility

The local OpenAI-compatible catalog contained both the loaded LLM and an
embedding model without a type field. Current generic code assigns
`TextGeneration` to every entry. Configuring that adapter directly for LM
Studio would therefore advertise the embedding model as text-capable. This is
the concrete Sprint 25 discovery gap.

### Native v1 source vocabulary

The smallest fields needed by the provider-neutral projection are:

```text
models: required array
entry.type: required discriminator, documented "llm" or "embedding"
entry.key: required downloaded-model identity
entry.loaded_instances: required array
loaded_instance.id: required API model identity for that loaded instance
```

All display, publisher, architecture, quantization, size, parameter, context,
format, vision, tool, reasoning, description, and variant values are
provider-specific and have no ADR-0045 destination.

### Evidence-supported conservative first slice

The strongest current evidence supports discovering **loaded LLM instance
IDs**:

1. accept only entries whose exact type is `llm`;
2. ignore well-formed `embedding` entries as unsupported by the closed shared
   capability vocabulary;
3. project each `loaded_instances[].id`, not the parent model `key`, into a
   provider-scoped `ModelId` with `TextGeneration`;
4. allow an LLM with zero loaded instances to contribute no descriptor;
5. build the complete result through `ModelCatalog::new` so ordering and exact
   duplicate rejection remain canonical.

This boundary avoids claiming model download, load, JIT, TTL, auto-evict, or
selection ownership. It also preserves custom instance identifiers documented
as the API `model` value. The live instance ID equaled its model key, but the
official custom-identifier contract proves that equality is not universal.

ADR-0047 must still decide whether an unknown future `type` rejects the entire
catalog as `Protocol` or is ignored as an unsupported addition. Rejecting it is
the stricter first-slice choice because silently ignoring a new type could hide
an incompatible provider vocabulary change.

### Required negative and boundary cases

Repository-owned fixtures can deterministically cover:

- empty `models` and only well-formed embedding/unloaded-LLM entries;
- one and multiple loaded LLM instances, reordered parents/instances, and
  exact canonical output;
- custom instance IDs, invalid IDs, duplicate IDs within/across parents, and
  exactly 1,024 versus 1,025 projected instances;
- missing/mistyped `models`, `type`, `loaded_instances`, or instance `id`;
- unknown type, nulls, malformed/trailing JSON, partial bodies, unknown fields,
  and large ignored metadata;
- successful body bounds, non-success status, redirect, transport, timeout,
  pre-cancellation, in-flight cancellation, and repeated fresh calls.

No real downloaded or loaded model is needed for these oracles.

## Generation candidate comparison

| Candidate | Evidence and advantages | Contract conflict or risk | Decision readiness |
| --- | --- | --- | --- |
| `POST /v1/completions` | Exact raw `prompt` shape matches ADR-0045; response supplies exact model and `stop`/`length`; current generic adapter already proves bounds, errors, redaction, timeout, cancellation, and cleanup; live structural success observed. | Officially legacy, base-model-oriented, no prompt template, and chat-tuned output may be unexpected. Support cannot imply chat-model quality. | Fully testable as a narrow compatibility slice; ADR-0047 must explicitly accept the legacy/base-model limitation. |
| `POST /v1/chat/completions` | Stateless request and response includes exact model and finish reason; official prompt template supports chat-tuned models. | Requires constructing role-bearing messages and therefore imports roles/message/prompt-template semantics explicitly excluded from the planned first slice; live eight-token output was empty. | Not eligible without reopening scope and accepting new message semantics. |
| `POST /api/v1/chat` | Current recommended native API; accepts a plain string; `store=false` avoids retained response state; live response contained one message item and no response ID. | Response is a heterogeneous item array, model identity is an instance ID, documented output has no finish reason, default storage must be overridden, and reasoning/tool/invalid-item behavior needs new mappings. Inferring `OutputLimit` from token counts would be ambiguous. | Native follow-up candidate; not sufficient for the current closed finish/output contract without additional architecture. |

The investigation does not accept a wire contract. It establishes that
`/v1/completions` is the only candidate currently compatible with the planned
raw-text/no-role/no-state first slice and existing finish semantics. ADR-0047
must either accept that bounded legacy compatibility or stop and change the
sprint scope before implementation.

## Feasible reuse boundary

### Hybrid composition is implementable through current public APIs

If ADR-0047 accepts native loaded-instance discovery plus legacy completion
generation, a new leaf package can preserve `OpenAiCompatibleProvider`
unchanged:

```text
future caller
  -> oneagent-lm-studio
       -> oneagent-openai-compatible
            -> oneagent-llm
       -> oneagent-llm
```

Construction can:

1. consume `ProviderConfiguration` whose ID is exactly `lm-studio`;
2. retain that provider ID;
3. construct a sensitive native-discovery Authorization header from the
   borrowed secret content;
4. move the same non-cloneable secret into a new internal
   `ProviderConfiguration` with ID `openai-compatible`;
5. construct the unchanged generic provider from the same explicit root;
6. construct a separate native-discovery reqwest client and
   `/api/v1/models` URL under ADR-0047 rules.

Generation can:

1. reject a non-`lm-studio` request before delegation;
2. reject pre-cancellation;
3. create a temporary public descriptor with provider
   `openai-compatible`, the exact same cloned `ModelId`, and
   `TextGeneration`;
4. create a temporary generic request from the exact copied input and output
   byte bound;
5. call `OpenAiCompatibleProvider::generate` exactly once with the same
   execution context;
6. let the generic adapter validate response model, finish, bounds, status,
   timeout, cancellation, redaction, and cleanup;
7. construct `TextGenerationResponse::new` against the original LM Studio
   request using the validated output and copied finish reason.

The public APIs needed for every step exist. This design introduces bounded
temporary copies of sensitive input/output and an unavoidable header copy; they
must be dropped before return and never formatted. It introduces no public
change to the generic adapter and no second attempt. The generic provider owns
the in-flight timer/cancellation race; the LM Studio wrapper performs only
bounded synchronous translation before and after it.

### Alternatives requiring broader change

- Generalizing `OpenAiCompatibleProvider` to accept arbitrary provider IDs
  would weaken its accepted stable identity and affect public behavior.
- Exporting its private client/execution helpers would create a new public
  transport API and couple LM Studio discovery to ADR-0046 internals.
- Extracting a new shared HTTP-provider crate would add a broader abstraction,
  migrate the completed generic adapter, and expand Task 3 risk.
- A fully independent LM Studio adapter would duplicate the complete generation
  transport/error/cancellation implementation that already passes public
  conformance.
- Wrapping generic discovery cannot recover the missing model type and remains
  invalid.

The hybrid composition is therefore the smallest evidence-backed reuse
candidate. ADR-0047, not this investigation, must accept or reject it.

## Construction, locality, authentication, and version evidence

- Official default server root is `http://localhost:1234`; the live server used
  explicit numeric loopback `http://127.0.0.1:1234`.
- LM Studio may also serve on a network interface. Repository evidence does not
  define DNS resolution or a robust loopback-only URL validator.
- Existing ADR-0046 root validation already covers explicit HTTP/HTTPS,
  host/port, empty-or-root path, no userinfo/query/fragment, no redirects, no
  proxies, Rust TLS, and redacted errors.
- Authentication is absent by default and may be enabled with an API token;
  optional Bearer behavior matches existing `ProviderSecret` semantics.
- Native REST API v1 and token authentication require LM Studio 0.4.0 or newer
  according to official documentation. The exact live application version is
  unknown; the CLI exposes commit `71bd99c`, not a verified server release.

ADR-0047 must decide whether to expose only an explicit root constructor, add a
deterministic local-default constructor using numeric loopback, or support both.
It must not read environment, files, GUI settings, CLI state, or DNS-derived
configuration. A broad security or remote-LM-Studio compatibility claim is not
supported.

## Error, timeout, cancellation, and redaction evidence

No LM Studio error-body schema needs to become shared authority. The existing
ADR-0046 mapping is reusable for both native discovery and delegated generation:

- 408, 429, and 5xx -> `ProviderUnavailable`;
- other non-success statuses, including redirects -> `ProviderRejected`;
- connection/request/body transport failures -> `Transport`;
- malformed, partial, mistyped, trailing, or over-bound successful JSON ->
  `Protocol`;
- invalid/duplicate/over-count projected instance identities ->
  `InvalidModelCatalog`;
- provider mismatch -> `InvalidRequest`;
- response identity/choice/finish/output violation -> `InvalidResponse`;
- total timer -> `Timeout`;
- existing or winning in-flight cancellation -> `Cancelled`.

Provider error bodies should not be read on non-success status. Diagnostics can
reuse static text plus decimal status only. URLs, credentials, headers, native
catalog bodies, request input, generated output, provider stats, model paths,
and Serde/reqwest sources remain sensitive or unrestricted and must not enter
implicit formatting, errors, fixtures, snapshots, or review artifacts.

Native discovery can reuse the 1 MiB successful-body bound and incremental
checked body reading from ADR-0046. Delegated generation retains the existing
512 KiB request/response wire bounds and ADR-0045 output-byte validation.

## Deterministic repository oracle

The existing Tokio controlled-loopback pattern can be reproduced inside the new
package without making the generic crate's private test harness public. The LM
Studio harness should bind only `127.0.0.1:0`, acknowledge readiness without
sleeps, capture a bounded exact request count, write complete or deliberately
partial synthetic responses, join deterministically, and prove zero surviving
state.

The complete first-slice matrix is:

| Area | Required deterministic cases |
| --- | --- |
| Construction | Exact provider ID, root/default decision, scheme/host/port/path, auth absent/present/invalid, user agent, no proxy/redirect, no I/O, Send/Sync, redaction. |
| Discovery wire | Exact `GET /api/v1/models`, Accept/auth headers, no body, fresh repeated request. |
| Discovery projection | Loaded LLM instance, multiple/custom instances, embedding and unloaded exclusion, empty, reorder, unknown additions, duplicate/invalid/over-count IDs. |
| Discovery failures | Missing/mistyped/unknown required fields, malformed/trailing/partial JSON, status, redirect, body bounds, transport, timeout, cancellation, cleanup. |
| Generation bridge | Exact translated model/input/bound, `stop`/`length`, Unicode, output bound, local usage, repeated calls, one generic request. |
| Generation failures | Provider mismatch before I/O, response fallback/mismatch, malformed/choice/index/finish/output/status/body/transport/timeout/cancellation, redaction, cleanup. |
| Compatibility | Complete existing generic adapter unit/public targets, complete `oneagent-llm` targets, provider substitution, analysis and Runtime library tests, dependency direction. |

No case requires an installed LM Studio application, downloaded model, API
token, local filesystem state, external network, ignored fixture, timing
threshold, or model-quality oracle.

## Unresolved ADR-0047 decisions

ADR-0047 has sufficient evidence to decide, but must explicitly select:

1. the new package/path/type names and exact dependency direction;
2. hybrid generic-generation composition versus another implementation;
3. `/v1/completions` as a deliberately legacy/base-model first slice or a
   scope stop before chat/native semantics;
4. loaded LLM instance IDs as discovery output and exact unknown-type behavior;
5. explicit root, numeric local default, or both, including any remote-root
   support claim;
6. exact direct dependency/version/features and the required user approval;
7. native discovery URL, request/response body limits, required/ignored fields,
   status/error diagnostics, and redaction vocabulary;
8. wrapper/generic timeout and cancellation precedence and bounded sensitive
   translation copies;
9. the exact public conformance and compatibility matrix.

## Deferred and unsupported scope

- LM Studio installation, daemon, GUI, server startup/shutdown, model download,
  load/unload, JIT ownership, TTL, and auto-evict;
- unloaded-model advertisement, model selection, aliasing, fallback, registry,
  cache, refresh, persistence, and retry/backoff;
- `/api/v1/chat`, `/v1/chat/completions`, Responses, Anthropic compatibility,
  roles/messages, prompt templates/policy, history/state, streaming, reasoning,
  tools, MCP, structured output, vision, and embeddings;
- provider metadata, token/statistics authority, model paths, context windows,
  sampling controls, cost, performance, quality, or security certification;
- Runtime registration/configuration/routes, Context orchestration, protocol,
  CLI, MCP, LSP, IDE, UI, graph mutation, and Coverage changes;
- live LM Studio availability or output as CI/review acceptance.

## Decision readiness and recommended next action

The data and testability gate passes for ADR-0047. Repository APIs prove that a
new leaf adapter can combine native loaded-instance discovery with one delegated
generic legacy completion attempt without changing ADR-0045 or ADR-0046. The
official and sanitized live evidence provides exact positive wire vocabulary;
controlled loopback can produce every negative and boundary oracle.

The next task should create ADR-0047, accept or reject the hybrid first slice,
record the exact dependency approval gate, and keep implementation blocked until
that ADR is committed and every new direct dependency/feature is explicitly
approved by the user.
