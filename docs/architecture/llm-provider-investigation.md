# LLM Provider Abstraction Investigation

## Status and scope

This document records repository evidence for Sprint 23 before ADR-0045 chooses
the provider-independent LLM contract. It does not accept architecture or
describe implemented provider support.

Investigated baseline:

- `HEAD`: `01f51a5045938a4369c768ae5cba5eacec95cc65`
- Sprint state: Sprint 22 is `completed`; Sprint 23 LLM Provider Abstraction is
  the unique `next` target.
- Working tree at task start: clean.
- Toolchain: `rustc 1.97.1` and `cargo 1.97.1`.
- CI: `.github/workflows/ci.yml` runs format, check, test, and Clippy on
  `macos-14` and `windows-latest`.

The investigation uses only tracked repository files, Cargo metadata, current
tests, and accepted architecture. It uses no provider documentation, provider
wire payload, live service, network request, credential, environment variable,
or developer-local model state.

## Accepted constraints

- The Roadmap limits Sprint 23 to provider-independent model, request, response,
  and capability contracts. Concrete OpenAI-compatible, LM Studio, and Ollama
  integrations belong to Sprints 24-26.
- ADR-0044 assigns deterministic semantic selection and rendering to
  `oneagent-analysis`. It explicitly excludes provider/model execution,
  streaming, conversations, tools, secrets, retries, cancellation, Runtime,
  protocol, MCP, and IDE behavior.
- `SemanticGraph` remains canonical semantic authority. A provider result cannot
  become a graph fact or Context Engine input merely because a provider returns
  it.
- The committed LLM Provider framework requires provider-neutral domain values,
  capability validation, secret safety, explicit timeout/retry/cancellation
  behavior, stable errors, and deterministic repository-owned contract evidence.
- No production dependency may be added without explicit user approval. A
  transitive package in `Cargo.lock` is not an available direct dependency.
- Tests cannot require live credentials, external network, arbitrary sleeps, or
  developer-local services.

## Current workspace and dependency evidence

The workspace has twelve members: Runtime and CLI applications; common,
workspace, metadata, protocol, graph, BSL, and analysis crates; and filesystem,
EDT, and Designer XML adapters. No path, manifest, crate, module, target, or
public symbol represents an LLM provider, model, provider request, provider
response, provider capability, provider secret, or provider error.

Relevant direct dependency facts are:

| Component | Direct production dependencies | Provider-boundary consequence |
|---|---|---|
| `oneagent-common` | none | It contains generic `EntityId` and `EntityName`, but no provider identity or secret type. |
| `oneagent-analysis` | BSL, common, graph | It owns Context Engine derived analysis and has no Runtime, async, provider, or serialization dependency. |
| `oneagent-protocol` | none | It contains only `component_name()` and no transport or provider domain contract. |
| `oneagent-runtime` | Axum, Tokio, Serde, tracing, source/graph/workspace crates | It owns application composition and transport, not source-independent provider semantics. |
| `oneagent-cli` | none | Runtime, Tokio, Serde JSON, and tempfile are dev dependencies used only by its integration tests. |

`Cargo.lock` contains futures, Hyper, Tower, and Tokio packages through existing
application dependencies. None is a direct dependency of a provider-neutral
library because that library does not exist. Reusing one would require an
explicit manifest change; using a new external production dependency would also
require user approval.

The current dependency direction provides no existing leaf library that owns
both LLM domain values and execution. A new workspace library can technically
use only `std` for owned values and a boxed `Future` type, but the exact crate,
package, module split, dependency on `oneagent-common`, and future/object-safety
shape remain ADR decisions.

## Ownership candidates and compatibility constraints

### `oneagent-analysis`

Confirmed facts:

- it owns source-independent semantic analysis and Context Engine;
- `ContextEngine::build` synchronously borrows one `SemanticGraph` and returns
  an owned `ContextBundle`;
- it has no production consumer outside its own unit and public integration
  tests;
- it has no async executor, Runtime, protocol, provider, or serialization
  dependency.

Constraint: placing provider execution here would mix canonical derived semantic
selection with non-deterministic external execution and would contradict the
ADR-0044 ownership boundary. The crate can remain a future caller input source
without depending on the provider abstraction.

### `oneagent-runtime`

Confirmed facts:

- it owns application/service lifecycle, Tokio tasks, receiver-only service
  cancellation, HTTP, workspace snapshots, and stable Runtime failures;
- `ConfigurationProvider` synchronously loads `RuntimeConfig` and returns a
  boxed error; it is not an LLM provider and has no secret contract;
- `RuntimeService` uses public boxed `Future` aliases, and the service container
  owns task handles and cancellation sources;
- `Cancellation` depends on `tokio::sync::watch`, has a crate-private source,
  and is supplied only through `ServiceContext`;
- Runtime does not depend on `oneagent-analysis` and exposes no Context or LLM
  service/route/configuration.

Constraint: the existing `ConfigurationProvider`, `RuntimeService`, and
`Cancellation` types are reusable patterns and compatibility evidence, not
provider-neutral domain owners. Making a provider library depend on the
application crate would reverse layering and prevent use outside Runtime.

### `oneagent-protocol`

Confirmed fact: the crate has no dependencies and exposes only a component-name
function. It has no accepted HTTP, MCP, LSP, LLM, or serialization authority.

Constraint: placing the source-independent provider domain here would conflate
provider semantics with future transport contracts. A protocol projection can
depend on a provider domain later; the reverse dependency is not evidenced.

### `oneagent-common`

Confirmed facts:

- `EntityId` and `EntityName` validate non-whitespace input and expose owned
  strings;
- both derive `Debug`, `Clone`, equality, order, and hash; `EntityId` also
  implements `Display` and parsing;
- no type has secret/redaction semantics.

Constraint: the generic identifiers are a possible primitive dependency for
non-secret provider/model identity, but their formatting and validation are not
an accepted provider identity contract. They are unsafe for secret values by
construction. ADR-0045 must choose reuse or local strong types from evidence,
not from naming similarity.

### New provider-neutral workspace library

Confirmed feasibility:

- workspace libraries follow a small `Cargo.toml` plus `src/lib.rs` pattern;
- Rust 1.97.1 and edition 2024 support `std::future::Future`, `Pin`, `Box`, and
  trait-based substitution without a runtime dependency;
- public integration tests can be placed under the new crate's `tests/` path;
- a standard-library-only production crate would add only a workspace member
  and local package to the lockfile.

Unknowns for ADR-0045: exact crate/package name, whether it depends on common,
module layout, public future lifetime, trait object strategy, Send/Sync bounds,
and whether cancellation is a provider input, caller-owned future, or a later
orchestration concern.

## Context Engine input boundary

`ContextBundle` is owned, cloneable, equality-comparable, and exposes:

- canonical seed IDs and admitted items;
- exact UTF-8 byte budget accounting and omission counts;
- provenance-backed item fragments and explanations;
- `rendered() -> &str`, the exact deterministic concatenated semantic rendering.

It has no serialization implementation and no provider, prompt, role, message,
conversation, tokenizer, model-token, source-text, or transport semantics. A
provider-neutral request can technically accept an owned text value supplied by
a caller without depending on `oneagent-analysis`. Whether the first request has
one input string or a closed message/role model, and whether `ContextBundle`
rendering is merely caller-supplied text or has a named integration, remain ADR
decisions.

No production crate currently consumes `ContextBundle`. Therefore Sprint 23 can
prove additive provider contracts without changing Context Engine, Runtime, or
protocol APIs. A direct Context-to-provider adapter, prompt construction, or
Runtime route lacks a current owner and belongs outside the first slice.

## Provider and model identity evidence

The repository contains no accepted provider identifier, model identifier,
endpoint identity, display name, alias, version, owner, or discovery record.
The Roadmap establishes only three future integration families:
OpenAI-compatible, LM Studio, and Ollama. It does not define serialized names or
wire identity.

Provider-neutral identity questions for ADR-0045 are therefore:

1. Is provider identity a non-empty stable value supplied by adapter
   construction, and is model identity scoped by that provider?
2. Which whitespace, length, case, normalization, display, ordering, and
   duplicate rules apply?
3. Does a model descriptor contain only identity and capabilities in Sprint 23,
   or also an opaque human-readable label?
4. Does discovery return an owned ordered collection, and how are duplicate
   model identities classified?
5. Are empty discovery results successful, unsupported, or errors?
6. Are provider-specific metadata and unknown future capabilities rejected,
   hidden behind an adapter boundary, or represented opaquely without affecting
   shared semantics?

No repository evidence supports endpoint URLs, organization/project IDs,
context-window sizes, prices, model families, version strings, or provider wire
metadata in the shared first slice.

## Capability and compatibility evidence

The provider framework requires explicit capability checks, but no capability
vocabulary is implemented. The Roadmap's provider-independent first slice and
Sprint 23 exclusions prove only bounded text generation as a candidate common
denominator. Streaming, tools, structured output, images, audio, embeddings,
tokenizers, and conversations are explicitly deferred.

ADR-0045 must decide:

- whether the first closed capability enum has only text generation or whether
  the text contract itself is implicit;
- whether discovery can advertise an empty capability set;
- canonical set ordering and duplicate handling;
- validation precedence between invalid request fields, unknown model identity,
  and incompatible capability;
- whether compatibility is checked by a request constructor, a separate
  validator, or the provider seam before delegation;
- how a future unknown capability is rejected without using a wildcard string
  as accepted semantic meaning.

A capability assertion cannot be derived from a provider name, model string, or
successful fake response.

## Request and response evidence

There is no provider request or response type, field vocabulary, bound, default,
usage unit, or finish reason. The only exact current text input is
`ContextBundle::rendered()`, whose budget is UTF-8 bytes and explicitly not model
tokens.

Repository-owned deterministic cases can safely cover:

- empty, whitespace-only, minimum, exact-maximum, and over-maximum text;
- invalid provider/model identity and duplicate/reordered inputs where order is
  declared irrelevant;
- accepted versus incompatible capability combinations;
- exact successful owned text output;
- empty or partial output classification;
- known and unknown finish classifications;
- absent, zero, boundary, inconsistent, or overflow-prone usage counters;
- error precedence and absence of a partial success on failure;
- repeated construction and execution equality for identical fake input.

ADR-0045 must choose the actual request fields, bounds, defaults, validation
precedence, meaningful ordering, output representation, usage units, finish
vocabulary, and partial/unknown behavior. No provider wire field or token count
can be copied into the shared contract without new evidence.

## Configuration and secret evidence

The repository contains no tracked credential/secret file and no secret-bearing
domain type. `RuntimeConfig` derives `Debug` and `Clone`; it contains application,
environment, bind-address, and workspace-root values only. Its
`ConfigurationProvider` returns unrestricted boxed errors. These contracts do
not establish safe LLM credentials.

ADR-0045 must decide:

- whether Sprint 23 needs a secret value only as an adapter-construction input
  or no configuration object at all;
- whether a secret is non-empty and bounded;
- whether cloning is forbidden or explicitly allowed;
- exact redacted `Debug` behavior and whether `Display`, equality, hash,
  serialization, or error embedding are absent;
- whether consumers may borrow the secret contents through one explicit method;
- whether endpoint/base URL is configuration but not secret, and what diagnostic
  redaction applies to URLs, headers, request bodies, response bodies, and
  provider messages;
- the maximum retained diagnostic size and whether request/response text is
  categorically excluded.

Environment, file, CLI, OS keychain, hot reload, and precedence between
configuration sources have no repository evidence and remain outside Sprint 23
unless ADR-0045 explicitly keeps them deferred.

## Async execution, timeout, retry, and cancellation evidence

`RuntimeService` demonstrates one object-safe pattern:

```text
Pin<Box<dyn Future<Output = Result<..., ...>> + Send + 'static>>
```

The application owns those futures and Tokio task handles. This proves the
workspace can expose boxed asynchronous work; it does not determine provider
future lifetime, ownership, or executor.

Runtime `Cancellation` is receiver-only and idempotent, but its source is private
and its implementation depends on Tokio. It cannot be constructed or owned by a
new independent library through the current public API. Reusing it would couple
provider semantics to Runtime. A provider-neutral cancellation input, a
caller-owned cancellation future, cooperative observation only, or deferral are
all still open decisions.

Existing timeouts are transport- or test-specific:

- the CLI uses fixed socket connect/read/write timeouts under ADR-0043;
- Runtime tests use Tokio timeouts as hang guards;
- Runtime lifecycle ADRs deliberately reject a general forced shutdown timeout.

No shared clock, deadline, retry policy, attempt counter, backoff, rate-limit,
or replay contract exists. ADR-0045 can define stable timeout/retry/cancel error
classifications and representation-only policy values without implementing
transport behavior. If it accepts automatic timeouts or retries, it must also
establish clock ownership, delay, replay safety, cancellation precedence,
attempt accounting, cleanup, and a deterministic oracle; current evidence does
not supply those contracts.

Streaming, interrupted streams, partial chunks, connection reuse, concurrency
limits, rate limiting, shutdown, and detached task ownership are unsupported in
Sprint 23. A first-slice provider future can be caller-polled and return one
terminal owned result without spawning work, but ADR-0045 must accept or reject
that boundary.

## Error taxonomy evidence

Nearby domain and Runtime APIs use closed error enums, stable kind enums,
`Display`, and `std::error::Error`. Runtime retains original sources and bounded
structured cleanup failures, while Context Engine returns typed all-or-nothing
domain failures. Neither contract contains secrets or provider diagnostics.

The provider-neutral error matrix must distinguish only cases whose owner is
known. Candidate categories for ADR review are:

- invalid provider/model/configuration/request input;
- incompatible model capability;
- discovery or provider failure;
- transport/protocol failure placeholders for future adapters;
- timeout, cancellation, and retry exhaustion only if policy accepts them;
- invalid, empty, partial, malformed, or unsupported provider response;
- internal contract violation without leaking sensitive content.

ADR-0045 must define exact variants, precedence, retry eligibility ownership,
provider diagnostic retention/redaction/size, and whether error sources are
stored. Provider-specific status codes or response bodies have no Sprint 23
oracle and cannot enter the shared taxonomy yet.

## Consumers and compatibility inventory

| Consumer or boundary | Current evidence | Sprint 23 compatibility requirement |
|---|---|---|
| Context Engine | Only `oneagent-analysis` tests consume its public API. | Keep behavior and dependencies unchanged; caller-owned rendered text may be future input without direct coupling. |
| Runtime | No dependency on analysis and no LLM service/config/route. | Keep service lifecycle, cancellation, health, Workspace, HTTP, and Graph Query behavior unchanged. |
| Protocol | Empty domain placeholder with no consumers. | Do not add a transport projection in Sprint 23. |
| CLI | No production dependencies and no LLM command. | Keep current command/wire behavior unchanged. |
| Source adapters and graph | No provider use. | No dependency or semantic/Coverage change. |
| Future Sprint 24 adapter | Not implemented. | ADR-0045 must make substitution possible without importing a future wire schema now. |

The graph/adapters Coverage Registries describe semantic nodes, edges,
provenance, and source ingestion. They contain no AI-provider category. Sprint
23 has no truthful Coverage transition; public conformance tests and current-
state documentation are its acceptance evidence.

## Deterministic contract-test oracles

The repository can provide all required first-slice evidence with constructed
Rust values and fake providers:

1. Domain unit cases validate identifiers, model ordering/deduplication,
   capabilities, bounds, usage, finish, and errors with exact equality.
2. Secret sentinel cases use a synthetic unique value and assert it is absent
   from every permitted debug/error/diagnostic output. No fixture stores a real
   credential.
3. Request cases record whether a fake was invoked, proving invalid or
   incompatible requests fail before provider delegation.
4. Independent immediate-success and immediate-failure fakes return exact owned
   discovery/response/error values through the public seam.
5. A controllable pending fake and explicit test synchronization can prove
   cancellation precedence and cleanup if ADR-0045 includes in-flight
   cancellation. Arbitrary sleeps are unnecessary.
6. Reordered model/capability inputs and fresh repeated fake instances can prove
   canonical equality and no retained global state.
7. Existing non-zero `oneagent-analysis` and `oneagent-runtime` package tests
   prove compatibility. Runtime loopback tests need sandbox bind permission but
   no external network.

A standard-library-only production crate is feasible for domain values and a
boxed-future provider seam. Test execution can use immediately ready futures or
a small repository-owned deterministic poll harness. Adding Tokio or another
executor even as a dev dependency is a manifest choice that ADR-0045 and the
implementation Change Contract must make explicit; no production runtime
dependency is required by the evidence above.

No real-source fixture is applicable because Sprint 23 parses no external
serialization. Provider wire fixtures first become necessary when Sprint 24
selects an OpenAI-compatible source contract.

## Unsupported behavior and claims

The current repository cannot support claims about:

- concrete OpenAI-compatible, LM Studio, or Ollama request/response compatibility;
- endpoint discovery, authentication, TLS, proxies, DNS, HTTP, JSON, or SSE;
- model availability, model context windows, tokens, pricing, latency, quality,
  rate limits, or production reliability;
- prompt design, system/user roles, conversations, tool calls, structured
  output, streaming, images, audio, or embeddings;
- automatic retry/backoff, transport timeout enforcement, concurrent request
  limiting, pooling, persistence, or Runtime shutdown integration;
- secret storage, environment/file precedence, keychain integration, or threat-
  model completeness;
- Runtime, protocol, CLI, MCP, LSP, IDE, or UI exposure.

These are deferred, not negative test outcomes for the provider-neutral domain.

## ADR-0045 decision matrix

ADR-0045 is ready only if it answers every row below without external-data
assumptions.

| Area | Required decision | Repository oracle |
|---|---|---|
| Ownership | Crate/package/module owner and dependency direction | Cargo metadata/tree and compile checks |
| Identity | Provider/model validation, scoping, order, duplicate, display behavior | Exact constructed values |
| Discovery | Async or sync seam, empty/duplicate/order/error behavior, no cache/refresh claim | Deterministic fake discovery |
| Capabilities | Closed first-slice vocabulary and unknown/incompatible behavior | Capability matrices |
| Request | Text fields, bounds, defaults, ordering, validation precedence, sensitivity | Constructor and preflight cases |
| Response | Text, empty/partial/malformed behavior and ownership | Exact fake terminal results |
| Usage | Presence, units, bounds, consistency, overflow behavior | Boundary counters |
| Finish | Closed known reasons and unknown-provider behavior | Exhaustive exact values |
| Configuration | Provider construction inputs and absent environment/file precedence | Constructed configuration only |
| Secrets | Access, clone/format/serialization/error restrictions and redaction | Synthetic sentinel absence |
| Errors | Closed kinds, precedence, sources, diagnostics, size and redaction | Exhaustive error cases |
| Async seam | Future lifetime, Send/Sync, substitution, task/global-state absence | Multiple public fake implementations |
| Timeout | Representation/enforcement owner and classification | Explicit enabled/disabled oracle |
| Retry | Disabled or exact eligibility/attempt/replay/delay owner | Fake invocation counts |
| Cancellation | Input/source owner, observation, precedence, cleanup | Controllable pending fake |
| Context compatibility | Text hand-off without dependency or prompt claims | Existing `ContextBundle::rendered()` tests |
| Runtime compatibility | No service/route/config/lifecycle change | Existing Runtime tests |
| Conformance | Public non-zero test target, cases, exact outcomes, repetition | Repository-owned Rust fakes |
| Dependencies | Standard library or separately approved additions | Manifest/lock/tree diff |
| Deferred scope | Concrete providers, wire protocols, live configuration, tools, streaming, Runtime/MCP/IDE | Scope audit |

## Decision readiness

Repository evidence is sufficient for ADR-0045 to select a bounded provider-
neutral text first slice and for later tasks to test it without external data.
The evidence supports a standard-library-only production implementation and
deterministic fake-provider oracles. It does not select exact API names, crate
identity, field vocabulary, bounds, secret access, async trait shape, timeout/
retry/cancellation policy, or error variants; those are the Task 2 decisions.

Task 2 is unblocked if it preserves all unsupported and deferred behavior above.
It must stop if it requires a concrete provider wire contract, live credential,
external service, or unapproved production dependency to define Sprint 23.
