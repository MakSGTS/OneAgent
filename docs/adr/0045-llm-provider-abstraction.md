# ADR-0045: Provider-Independent LLM Abstraction

## Status

Accepted

## Context

Sprint 23 must define and implement the provider-independent boundary used by
future OpenAI-compatible, LM Studio, and Ollama adapters. It must not import one
future provider's wire schema into the shared domain or require live services,
credentials, or network access for acceptance.

The [LLM Provider investigation](../architecture/llm-provider-investigation.md)
confirms that the repository has no provider/model/request/response/capability/
secret/error domain and no provider consumer. `oneagent-analysis` owns the
deterministic Context Engine under ADR-0044; `oneagent-runtime` owns application
lifecycle, Tokio tasks, transport, and receiver-only service cancellation;
`oneagent-protocol` contains no transport contract. Runtime configuration and
cancellation types are application-specific patterns rather than reusable LLM
domain authority.

The current Rust 1.97.1 toolchain can express owned domain values, a boxed
borrowed future, and object-safe provider substitution using only the standard
library. Constructed values and deterministic fake providers can prove the
complete first slice without an external production dependency, provider wire
fixture, async executor, live model, or credential.

## Decision

### Canonical ownership and dependency direction

Create the workspace library crate `crates/llm` with package name
`oneagent-llm`. It owns provider-independent LLM identities, model descriptors,
capabilities, text requests and responses, byte usage, finish reasons,
secret-safe construction values, execution policy, error taxonomy,
cancellation input, and provider substitution.

The crate has no production dependency, including no dependency on common,
analysis, Runtime, protocol, Tokio, Serde, an HTTP client, or a provider SDK.
Its public contract uses only `std`. The root workspace manifest adds the crate;
`Cargo.lock` may change only mechanically for the new local package.

Dependency direction is:

```text
Context Engine / future callers ── owned text ──▶ oneagent-llm
Runtime / protocol / concrete provider adapters ─────────▶ oneagent-llm
oneagent-llm ──X──▶ analysis / Runtime / protocol / adapters
```

`oneagent-llm` does not construct concrete providers, read configuration
sources, start tasks, select an executor, call a transport, mutate semantic
state, or retain Context Engine values. Future adapters own wire mapping and may
depend on this crate. Future Runtime composition owns adapter construction,
lifecycle, and request orchestration.

### Public module and value boundary

The crate may split implementation across local modules, but its public boundary
contains these concepts with nearby repository naming style:

- `ProviderId`, `ModelId`, and `ModelIdentity`;
- `ModelCapability`, `ModelDescriptor`, and `ModelCatalog`;
- `ProviderSecret` and `ProviderConfiguration`;
- `RetryPolicy` and `ProviderExecutionPolicy`;
- `TextGenerationRequest`;
- `TextGenerationResponse`, `TextUsage`, and `FinishReason`;
- `LlmErrorKind`, `LlmError`, and `ProviderDiagnostic`;
- `ProviderFuture`, `CancellationSignal`, `NeverCancelled`,
  `ProviderExecutionContext`, and `LlmProvider`.

Exact private helpers may differ. Public names above are normative unless Rust
language constraints require an equivalent name recorded in implementation
documentation. All successful domain values are owned and contain no borrowed
provider, Context, Runtime, transport, or configuration-source state.

No public value implements Serde in Sprint 23. There is no wire format or stable
serialization claim.

### Stable byte bounds

All string limits count UTF-8 bytes, not Unicode scalar values, display width,
or model tokens. Accepted inclusive maxima are:

| Value | Maximum UTF-8 bytes |
|---|---:|
| provider identifier | 128 |
| model identifier | 256 |
| request input text | 65,536 |
| requested output text | 65,536 |
| provider secret | 4,096 |
| retained provider diagnostic | 512 |

One model catalog contains at most 1,024 models. These are library safety and
testability bounds, not claims about a concrete provider, context window,
tokenizer, transport, or model capacity.

Numeric bounds use checked arithmetic. An invalid or overflowing value produces
one typed failure and no partial domain value.

### Provider and model identity

`ProviderId` and `ModelId` are separate local strong types. They do not reuse
`EntityId`, because provider identity needs its own length, whitespace, control-
character, and future compatibility contract without importing semantic-domain
formatting.

Each identifier:

- owns one case-sensitive string;
- rejects an empty or all-whitespace string;
- rejects leading or trailing Unicode whitespace;
- rejects Unicode control characters;
- rejects a value over its exact UTF-8 byte maximum;
- preserves every accepted byte without trimming, case conversion,
  normalization, aliasing, or provider-specific parsing;
- implements `Debug`, `Display`, `Clone`, equality, total order, and hash.

Validation precedence is empty/all-whitespace, byte maximum, boundary
whitespace, then control character. Public error diagnostics identify the field
and violation but never echo the rejected raw value.

`ModelIdentity` consists of one `ProviderId` and one `ModelId`. Model IDs are
scoped by provider ID. Equality and ordering compare provider ID first, then
model ID. There is no endpoint, URL, owner, organization, model family, alias,
version, price, or provider metadata in identity.

### Model capability vocabulary

The closed Sprint 23 capability enum has one variant:

- `TextGeneration` — accepts one bounded provider-neutral text request and
  produces one terminal bounded text response.

The explicit enum makes compatibility observable and provides an additive
extension point. It does not imply roles/messages, prompts, conversations,
streaming, tools, structured output, media, embeddings, token counts, or any
provider wire feature.

A `ModelDescriptor` contains exactly one `ModelIdentity` and a canonical ordered
set of capabilities. An empty capability set is valid and represents a known
model incompatible with the Sprint 23 text request. Duplicate capability input
is deduplicated by set semantics. No display label or opaque metadata is
accepted.

### Model discovery projection

`ModelCatalog` contains one provider ID and zero to 1,024 model descriptors.
Construction:

1. validates the count;
2. verifies every model identity has the catalog provider ID;
3. sorts models by full `ModelIdentity`;
4. rejects duplicate model identities atomically.

An empty catalog is a successful discovery result. Caller order never affects
the accepted catalog. A provider-specific unknown field or capability remains
inside its adapter; it is not retained as an opaque shared property and cannot
alter compatibility.

Discovery is a fresh provider call. Sprint 23 defines no cache, refresh,
staleness, background discovery, global registry, or merge across providers.

### Provider configuration and secrets

`ProviderConfiguration` contains one `ProviderId` and an optional
`ProviderSecret`. It is an adapter-construction input only. It has no endpoint,
header, organization, project, environment source, file source, CLI source,
precedence, keychain, reload, or transport setting.

`ProviderSecret` owns one non-empty value up to 4,096 UTF-8 bytes. It rejects
empty/all-whitespace and over-limit input without retaining or reporting the raw
value. Whitespace and control characters inside a non-empty accepted secret are
opaque and preserved.

Secret safety is:

- no `Clone`, `Copy`, `Display`, equality, order, hash, or serialization;
- `Debug` renders exactly `ProviderSecret([REDACTED])`;
- one explicit `expose()` accessor returns `&str` for concrete adapter
  construction;
- validation errors, `ProviderConfiguration` debug output, `LlmError`, and
  public diagnostics never contain the secret;
- no real credential appears in fixtures, tests, snapshots, or repository
  content.

`ProviderConfiguration` is not cloneable, comparable, hashable, or serializable.
Its `Debug` output shows provider identity and only whether a credential is
present. The standard-library implementation does not claim guaranteed memory
zeroization on drop; that requires separate dependency and security evidence.

### Text request contract

`TextGenerationRequest` is constructed from:

- one `ModelDescriptor`;
- one owned input string;
- one required `max_output_bytes` in `1..=65_536`.

It retains the selected `ModelIdentity`, exact input bytes, and output bound. It
does not retain the whole descriptor or provider configuration.

Request validation precedence is:

1. empty or all-whitespace input;
2. input over 65,536 UTF-8 bytes;
3. zero output bound;
4. output bound over 65,536 bytes;
5. missing `TextGeneration` capability.

Accepted input preserves leading/trailing whitespace, line endings, Unicode,
and byte order exactly after the non-empty test. There is no trim, prompt
prefix, role, message list, history, template, source extraction, tokenization,
or model-specific parameter. There is no default output bound; the caller must
supply it explicitly.

Only the public validated request type reaches `LlmProvider::generate`.
Incompatible input therefore fails before provider execution and produces no
partial request.

### Terminal text response, usage, and finish

`TextGenerationResponse` is constructed against the originating validated
request and contains:

- the same `ModelIdentity` as the request;
- one non-empty owned output string no larger than the request's
  `max_output_bytes`;
- one `TextUsage` containing exact request-input and response-output UTF-8 byte
  counts;
- one closed `FinishReason`.

`TextUsage` is computed locally with checked `usize` arithmetic. It is not a
provider token count and does not retain provider-reported usage. The accepted
finish variants are:

- `Completed` — the provider mapped a normal terminal completion;
- `OutputLimit` — the provider mapped its bounded-output terminal condition.

An empty output, over-bound output, model mismatch, arithmetic overflow,
provider partial/chunk value, malformed terminal value, or unknown provider
finish reason maps to `InvalidResponse`. No partial response is returned.
Adapters may reject unsupported provider finish values; the shared enum has no
`Unknown(String)` escape hatch.

Responses implement `Debug`, `Clone`, and equality for deterministic contract
tests. Request and response text are sensitive content: `Debug` for both types
must report identity, byte counts, bounds, usage, and finish only, never input or
output text. Explicit `input()` and `output()` accessors provide content.

Task 3 may introduce the public response/usage/finish shapes with only a crate-
private checked constructor because `TextGenerationRequest` does not exist until
Task 4. Task 4 adds the public request-bound response constructor. There is no
public constructor that can supply arbitrary usage or bypass request identity
and output-bound validation.

### Provider-neutral error contract

The closed stable error-kind enum is:

- `InvalidProviderId`;
- `InvalidModelId`;
- `InvalidModelCatalog`;
- `InvalidConfiguration`;
- `InvalidRequest`;
- `IncompatibleModel`;
- `InvalidResponse`;
- `ProviderUnavailable`;
- `ProviderRejected`;
- `Transport`;
- `Protocol`;
- `Timeout`;
- `Cancelled`;
- `Internal`.

`LlmError` owns one kind and at most one `ProviderDiagnostic`. `Display` renders
only one stable generic English message for the kind. Custom `Debug` renders the
kind plus diagnostic presence and byte length, never diagnostic content.
`std::error::Error::source()` returns `None` in Sprint 23 so an unrestricted
provider error cannot bypass redaction.

`ProviderDiagnostic` accepts one non-empty string up to 512 UTF-8 bytes. It does
not implement `Debug`, `Display`, `Clone`, comparison, hash, serialization, or
`Error`. Its explicit `as_str()` accessor is the only content access. Concrete
adapters are responsible for supplying already redacted text that excludes
credentials, headers, URLs containing credentials, request/response bodies, and
unbounded provider payloads. Sprint 23 synthetic tests assert implicit error
formatting cannot expose a sentinel.

`LlmErrorKind::is_retryable()` returns `true` only for
`ProviderUnavailable`, `Transport`, and `Timeout`. This is classification for a
future caller; it does not authorize or perform retry. `Cancelled`, validation,
compatibility, rejection, protocol, invalid-response, and internal failures are
not retryable.

### Execution and retry policy

`ProviderExecutionPolicy` contains:

- optional total timeout represented as `Option<Duration>`;
- `RetryPolicy`, whose only Sprint 23 variant is `Never`.

A present timeout must be greater than zero and no more than 300 seconds. It is
an adapter input and stable policy value only. `oneagent-llm` owns no clock,
timer, executor, delay, abort, or forced termination and does not enforce the
timeout. A concrete adapter may later enforce it and return `Timeout` under its
accepted transport contract.

`RetryPolicy::Never` means exactly one provider invocation, attempt number one,
no replay, no delay/backoff, and no retry after any error even when
`is_retryable()` is true. Sprint 23 contains no automatic executor wrapper that
could make another attempt. A future retry policy requires a separate accepted
orchestration decision covering clock, delay, idempotence, replay safety,
attempt accounting, cancellation, and cleanup.

### Cancellation contract

Cancellation is provider-neutral cooperative input, not a Runtime cancellation
source. The object-safe public trait is conceptually:

```rust
pub trait CancellationSignal: Send + Sync {
    fn is_cancelled(&self) -> bool;
    fn cancelled(&self) -> ProviderFuture<'_, ()>;
}
```

`NeverCancelled` is the stateless default implementation: `is_cancelled()` is
false and `cancelled()` remains pending. The provider crate creates no public
cancellation source and owns no task or global state. Runtime or another caller
may adapt its own receiver-only source later without changing this trait.

The terminal precedence is:

1. if cancellation is already requested when discovery or generation begins,
   return `Cancelled` without provider work;
2. during a pending provider operation, the implementation must observe the
   supplied cancellation future cooperatively;
3. the first observed provider terminal result or cancellation result wins;
4. after a terminal result, later cancellation cannot replace it;
5. cancellation returns no partial catalog or response and the provider future
   must release its owned temporary resources before completion.

The shared crate cannot forcibly stop a provider future or enforce cooperation.
Every concrete adapter must pass the public conformance contract before it is
claimed supported.

### Asynchronous provider seam

The standard-library future alias is:

```rust
pub type ProviderFuture<'a, T> =
    Pin<Box<dyn Future<Output = T> + Send + 'a>>;
```

`ProviderExecutionContext<'a>` borrows one `ProviderExecutionPolicy` and one
`dyn CancellationSignal`. It is copyable and owns no executor or task.

The object-safe provider trait is conceptually:

```rust
pub trait LlmProvider: Send + Sync {
    fn id(&self) -> &ProviderId;

    fn discover_models<'a>(
        &'a self,
        context: ProviderExecutionContext<'a>,
    ) -> ProviderFuture<'a, Result<ModelCatalog, LlmError>>;

    fn generate<'a>(
        &'a self,
        request: &'a TextGenerationRequest,
        context: ProviderExecutionContext<'a>,
    ) -> ProviderFuture<'a, Result<TextGenerationResponse, LlmError>>;
}
```

Each call borrows the provider for the future lifetime and returns one owned
terminal result. The provider must return its own identity from `id()`. A
successful catalog must use that provider ID; a request must use that provider
ID; and a response must use the request model identity. Identity mismatch is an
atomic typed failure.

The trait does not construct providers, store credentials, spawn tasks, retain
requests after its future completes, stream chunks, cache catalogs, retry,
select models, or expose provider-specific extensions. Callers poll the future
on their chosen executor. Two independent fake implementations must work
through `&dyn LlmProvider` to prove substitution.

### Validation and terminal precedence

The all-or-nothing order across the public boundary is:

1. construct and validate identities, descriptor/catalog, configuration, secret,
   execution policy, and request before a provider call;
2. reject a provider-ID mismatch before delegation;
3. reject already-requested cancellation before provider work;
4. delegate exactly once under `RetryPolicy::Never`;
5. while pending, accept the first observed provider terminal result or
   cancellation;
6. validate catalog/response identity and terminal shape before returning
   success;
7. return one owned success or one `LlmError`, never both and never partial
   provider state.

The provider trait contract and public fake evidence make this order observable.
No insertion order, hash order, clock, random source, environment, network, or
global state participates.

### Deterministic conformance evidence

Tasks 3-5 add focused unit evidence. Task 6 adds a non-zero public integration
target under `crates/llm/tests/` that uses only public APIs and repository-owned
Rust fakes.

The complete matrix includes, as applicable:

- every identity validation outcome and exact boundary;
- model scoping, capability ordering/deduplication, catalog empty/maximum/
  over-limit/mismatch/duplicate/reordered behavior;
- secret empty/maximum/over-limit construction and exact redacted formatting;
- diagnostic maximum and implicit formatting redaction with synthetic sentinels;
- request empty/maximum/over-limit/output-bound/validation-precedence and
  incompatible-model behavior;
- response model, empty/maximum/over-limit, usage, finish, and malformed/unknown
  rejection;
- execution-policy zero/maximum/over-limit timeout and `RetryPolicy::Never`;
- independent provider substitution, discovery, generation, provider failure,
  already-cancelled and in-flight cancellation, one invocation, cleanup,
  reordered input, and repeated fresh equality;
- unchanged public Context Engine and Runtime tests and dependency direction.

Tests use immediate futures or a small standard-library deterministic poll
harness with explicit controllable state and wakers. They use no Tokio direct
dependency in `oneagent-llm`, no arbitrary sleep, filesystem, socket, external
network, live provider, environment configuration, credential, or ignored data.

### Context Engine and Runtime compatibility

`oneagent-analysis` remains unchanged and does not depend on `oneagent-llm`.
Callers may copy `ContextBundle::rendered()` into a request input, but Sprint 23
does not define prompt semantics, roles, source text, tokenization, or direct
integration.

`oneagent-runtime` remains unchanged and does not depend on `oneagent-llm` in
Sprint 23. It gains no provider configuration, service, route, health state,
protocol, CLI operation, cancellation adaptation, or task. Existing analysis
and Runtime package tests remain compatibility gates.

The graph, metadata, BSL, workspace, source adapters, protocol, and CLI remain
unchanged. Provider outputs do not create semantic graph facts.

### Coverage and documentation completion

Current Coverage Registries describe semantic graph and source-ingestion
capabilities and have no LLM-provider category. Sprint 23 does not add or
transition a Coverage capability.

Architecture acceptance alone does not change current product support. After
Tasks 3-5 implement the complete first slice, Task 6 adds public conformance
evidence and synchronizes `README.md`, `docs/Architecture.md`, and
`docs/architecture/semantic-model-2.md`. Sprint 23 remains incomplete until an
independent review records a non-blocking decision after the full workspace
gate.

## Rejected alternatives

### Put provider execution in `oneagent-analysis`

Rejected because ADR-0044 keeps deterministic semantic selection independent
from external model execution and provider state.

### Put the shared domain in `oneagent-runtime`

Rejected because Runtime is the application composition root and transport
owner. A reusable adapter and future non-Runtime consumer must not depend on the
application crate.

### Put the shared domain in `oneagent-protocol`

Rejected because the provider domain is transport-independent and the protocol
crate has no accepted provider or serialization authority.

### Reuse `EntityId` and `EntityName` for provider/model identity

Rejected because their validation and formatting contract does not include the
provider-specific safety bounds and because provider identity is not semantic-
graph entity identity.

### Add Tokio, `async-trait`, futures, Serde, an HTTP client, or a provider SDK

Rejected for Sprint 23 because standard-library futures and constructed values
cover the abstraction. Concrete transport and serialization begin in Sprint 24
and require their own evidence and dependency decision.

### Use generic strings, JSON maps, or provider-specific enums in the shared API

Rejected because they weaken validation, leak wire schemas, and make capability,
finish, error, identity, and compatibility behavior non-exhaustive.

### Define messages, roles, token usage, streaming, or tools now

Rejected because the repository has no accepted prompt, conversation, tokenizer,
stream, or tool-policy authority. The first slice is one bounded text input and
one terminal text output.

### Make secrets cloneable/serializable or include diagnostics in error formatting

Rejected because implicit copies and formatting make credential or sensitive
content leakage difficult to detect and contain.

### Implement automatic timeout and retry orchestration

Rejected because the repository has no provider transport, clock, replay-safety,
delay, or retry-ownership evidence. Sprint 23 represents a bounded optional
timeout, classifies retryable errors, and fixes retries to `Never`.

### Require a live provider as the contract oracle

Rejected because it would make results depend on network, credentials, model
availability, provider drift, latency, and developer-local state.

## Consequences

- Future provider adapters receive one closed source-independent domain and
  object-safe asynchronous seam without importing Runtime or Context Engine.
- Text request/response behavior, byte usage, model compatibility, secret
  formatting, errors, cancellation, and no-retry behavior are deterministic and
  testable before any provider wire integration.
- The first slice adds a new public crate and API but no external production
  dependency.
- UTF-8 byte bounds are exact safety bounds and not model-token or provider-
  capacity claims.
- Timeout is represented for adapters but not enforced centrally; retryable
  classification does not cause retry.
- Cooperative cancellation requires each concrete provider to demonstrate
  conformance. The shared crate cannot forcibly abort opaque futures.
- Secret memory zeroization, external credential storage, and security
  certification remain unclaimed.

## Implementation prerequisites and order

1. Add `oneagent-llm` and implement identities, capabilities, catalog,
   configuration/secret, policy, public response/usage/finish shapes with a
   crate-private checked constructor, errors, Rustdoc, and focused domain tests
   with no production dependency.
2. Implement validated capability-aware text requests, the public request-bound
   response constructor, and focused boundary, precedence, redaction,
   reordering, and repetition tests.
3. Implement `ProviderFuture`, cancellation values, execution context, and
   `LlmProvider`; add deterministic fake discovery/generation/cancellation/
   no-retry/cleanup tests.
4. Add the public conformance target, rerun Context Engine and Runtime
   compatibility, and synchronize truthful current-state documentation.
5. Complete an independent integration review and full workspace gate.

If Rust object-safety or lifetime evidence makes the exact conceptual trait
unimplementable, stop before changing the contract and report the concrete
language constraint. Do not add a dependency or silently substitute a different
ownership model.

## Deferred scope

- OpenAI-compatible, LM Studio, and Ollama adapters and wire schemas;
- HTTP, JSON, SSE, TLS, DNS, proxies, endpoint discovery, authentication
  headers, organization/project configuration, and live model discovery;
- environment, file, CLI, keychain, credential precedence, reload, and secret
  memory zeroization;
- provider metadata, model aliases/families/versions/context windows/prices;
- model tokens, tokenizers, provider-reported token usage, cost, latency, quality,
  rate limits, and performance/security certification;
- prompt templates/policy, system/user roles, messages, conversations/history,
  source text, structured output, tools, images, audio, embeddings, and streaming;
- automatic timeout enforcement, retries/backoff, concurrency limits, pooling,
  global provider registry, caching, refresh, persistence, and shutdown;
- direct Context Engine integration, Runtime services/routes/configuration,
  protocol/CLI, MCP, LSP, IDE, and UI behavior;
- graph mutation, semantic/provider Coverage registry, and provider output as
  semantic truth.

## Completion criteria

Sprint 23 is complete only when the implementation order is committed, public
contract evidence proves every accepted value and terminal path without network
or credentials, existing Context Engine and Runtime behavior remains compatible,
the canonical full workspace validation succeeds, current-state documentation
preserves every deferral, and the Sprint 23 integration review records `pass` or
`pass with non-blocking follow-ups`.
