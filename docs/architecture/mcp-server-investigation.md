# MCP Server Investigation

## Status

- Sprint: 28 — MCP Server
- Evidence captured: 2026-08-26
- Decision target: ADR-0050
- Scope: investigation only; this document does not accept an architecture

## Purpose

This investigation records the repository and protocol evidence needed to add
the first OneAgent MCP server without inventing a legacy session model, leaking
non-protocol output onto standard output, or coupling later semantic tools to
the transport. Confirmed facts, candidate decisions, and unresolved decisions
are separated deliberately.

## Authoritative protocol sources

The current released MCP revision at the time of investigation is
`2026-07-28`. The versioned specification and schema for that revision are the
authoritative inputs for Sprint 28:

- [Basic protocol](https://modelcontextprotocol.io/specification/2026-07-28/basic)
- [stdio transport](https://modelcontextprotocol.io/specification/2026-07-28/basic/transports/stdio)
- [schema reference](https://modelcontextprotocol.io/specification/2026-07-28/schema)
- [TypeScript source-of-truth schema](https://github.com/modelcontextprotocol/modelcontextprotocol/blob/main/schema/2026-07-28/schema.ts)
- [generated JSON Schema](https://github.com/modelcontextprotocol/modelcontextprotocol/blob/main/schema/2026-07-28/schema.json)

The version segment, rather than a floating specification page, pins the
normative protocol revision. The TypeScript schema is identified by the MCP
project as the source of truth; the JSON Schema is useful as a generated
cross-check. These sources were reopened on 2026-08-26.

### Normative facts for the selected revision

- Messages use JSON-RPC 2.0 request, notification, response, and error shapes.
- A request identifier is a non-null string or integer. A notification has no
  identifier and receives no response.
- Each request carries `_meta` entries for
  `io.modelcontextprotocol/protocolVersion` and
  `io.modelcontextprotocol/clientCapabilities`. Client information is
  optional.
- Known `ClientCapabilities` members have schema-defined object shapes:
  `roots` is an object; `sampling` and `elicitation` contain only object-valued
  known options; `experimental` and namespaced `extensions` map to objects.
  Known optional request metadata, client-information, and icon fields also
  retain their schema types even when the server does not use their values.
- `_meta` names follow the versioned prefix/name grammar. Extension capability
  identifiers require the prefixed form; unknown capability members remain
  open for forward-compatible JSON values.
- A successful response identifies its `resultType`. The current base result
  types include `complete` and `input_required`.
- `server/discover` is the discovery method for this revision. Its complete
  result reports `supportedVersions`, `capabilities`, `ttlMs`, and
  `cacheScope`, with optional instructions and server information.
- Protocol-specific JSON-RPC errors include header mismatch (`-32020`), missing
  required client capability (`-32021`), and unsupported protocol version
  (`-32022`), in addition to standard JSON-RPC errors.
- The `2026-07-28` protocol is stateless. It does not use the legacy
  `initialize`/`initialized` handshake or session identifiers.
- Under stdio, each line is exactly one JSON-RPC message. Messages must not
  contain embedded newlines. Standard output is reserved for protocol
  messages; diagnostic text may use standard error.
- End of input is the primary graceful stdio shutdown signal and the server is
  expected to exit promptly.

The specification does not choose OneAgent-specific input size, concurrency,
queueing, server identity, discovery TTL, or process-composition limits. Those
are ADR decisions, not protocol facts.

### Compatibility boundary

Older MCP material describes initialization, negotiated sessions, and earlier
capability shapes. Implementing those concepts in the first slice would create
an incompatible hybrid. Sprint 28 should either implement the selected
`2026-07-28` revision coherently or stop. Supporting an additional revision is
a separate compatibility feature.

## Repository baseline

### Protocol crate

`crates/protocol` is a workspace member and currently contains only
`component_name()`. It has no dependencies, wire types, validation, codec,
dispatcher, transport, or tests. Current architecture documentation describes
it as an inactive placeholder. Its name makes it the natural candidate for
transport-neutral JSON-RPC/MCP ownership, but the repository has not yet made
that decision.

### Runtime application

`apps/runtime` is the existing process and service-composition boundary:

- `AppBuilder` registers named `RuntimeService` values.
- `ServiceContainer` starts registered services, treats unexpected service
  completion as failure, propagates cancellation, and joins cleanup.
- `App::run` builds the container and waits for an external shutdown future.
- `apps/runtime/src/main.rs` currently registers Workspace and HTTP services
  and supplies `tokio::signal::ctrl_c()` as shutdown.
- `App::run` prints a startup banner with `println!`, so reusing that process
  unchanged for stdio would violate the protocol-only standard-output rule.
- A running service has cancellation input but no existing public mechanism to
  request process-wide shutdown when it observes EOF.

ADR-0037 makes Runtime the service composition root and requires coordinated
cancellation and cleanup. ADR-0038 and ADR-0040 place HTTP health and graph
query behavior in the same application without making HTTP a semantic owner.
ADR-0043 keeps the CLI a separate HTTP client process. Sprint 28 must preserve
those boundaries and existing behavior.

### Semantic and policy owners

Workspace snapshots and graph-query semantics already belong to Runtime and
the graph/workspace layers. The Sprint 27 tool policy is transport-neutral and
does not authorize execution by itself. Sprint 29 owns semantic MCP tools.
Therefore Sprint 28 has no evidence for moving semantic queries, tool
authorization, provider integration, or side effects into a protocol crate.

### Existing test patterns

The repository already demonstrates:

- deterministic Tokio service lifecycle and cancellation tests in Runtime;
- loopback HTTP integration without external credentials;
- child-process testing and separated standard output/error in CLI tests;
- controlled service start, failure, unexpected exit, and cleanup oracles;
- explicit JSON serialization dependencies in Runtime.

No live MCP client, remote transport, signal delivery, credential, or semantic
tool side effect is required to prove the Sprint 28 boundary.

## Dependency evidence and gate

The lockfile already contains the exact workspace versions of `serde`,
`serde_json`, and `tokio`. Runtime already declares all three. The protocol
crate declares none of them, and Runtime does not currently depend on
`oneagent-protocol`.

The smallest anticipated production dependency changes are:

1. add workspace `serde` and `serde_json` dependencies to `crates/protocol`;
2. add the existing path dependency `oneagent-protocol` to `apps/runtime`.

No MCP SDK or new third-party package is supported by current evidence. Adding
the three direct dependency edges still requires the explicit dependency
approval gate recorded by the sprint plan. A transport design that puts Tokio
inside the protocol crate would introduce another direct edge and therefore
requires separate evidence and approval; the existing Runtime Tokio edge makes
that unnecessary for the first candidate design.

## Ownership candidates

The following split is the leading candidate, not an accepted decision:

| Concern | Candidate owner | Evidence and constraint |
| --- | --- | --- |
| JSON-RPC identifiers, requests, notifications, responses, and errors | `crates/protocol` | Transport-neutral wire contract; currently missing |
| MCP revision metadata and discovery values | `crates/protocol` | Versioned protocol contract; no semantic dependency required |
| Decode, validation, and canonical one-line encoding | `crates/protocol` | Must be reusable and independently testable |
| Method registry and protocol dispatch | `crates/protocol` | Empty Sprint 28 registry can remain semantic-free |
| Async line reading/writing and actual stdin/stdout handles | `apps/runtime` | Runtime already owns Tokio and process lifecycle |
| Service cancellation and cleanup | `apps/runtime` | Required by ADR-0037 |
| Semantic tool handlers | deferred Runtime/application layer | Sprint 29; protocol crate must not own Workspace semantics |
| Public executable/mode | unresolved | Must avoid changing the existing HTTP process contract accidentally |

Alternatives requiring ADR treatment include a dedicated MCP binary within
`apps/runtime`, an explicit runtime mode, or adding MCP stdio to the default
runtime process. A new package is not justified by current evidence. Adding
stdio to the default process is risky because it changes shutdown ownership,
standard-output behavior, and current HTTP/Workspace startup semantics.

## First-slice protocol questions for ADR-0050

### Input and envelope

The ADR must fix:

- accepted JSON-RPC version and the precedence of parse, envelope, metadata,
  protocol-version, capability, and method validation;
- preservation and echo of every valid string or integer identifier;
- behavior for `null`, boolean, fractional, object, array, duplicate, and
  otherwise invalid identifiers;
- request versus notification classification when an `id` member is absent;
- handling of unknown top-level and `_meta` members;
- schema validation for known capability, progress-token, log-level,
  client-information, icon, and namespaced-extension shapes before dispatch;
- rejection policy for duplicate JSON object keys, invalid UTF-8, blank lines,
  embedded newlines, and oversized lines;
- fixed maximum line length and whether the reader can recover at the next
  delimiter after an oversized input;
- exact error data shape and whether validation exposes deterministic field
  paths without echoing sensitive or unbounded input.

Duplicate-key behavior cannot be assumed from `serde_json::Value`, because
ordinary deserialization may discard the distinction. If duplicates must be
rejected, decoding needs an explicit mechanism and tests.

### Discovery and dispatch

For a conforming semantic-free first slice, the ADR must define:

- exactly one supported version: `2026-07-28`;
- truthful empty server capabilities until Sprint 29 registers semantic tools;
- deterministic server identity and optional metadata fields;
- whether optional discovery instructions are omitted or populated, and the
  exact required `ttlMs` and `cacheScope` values, with no invented caching
  promise;
- `server/discover` success for compatible metadata;
- unsupported-version and missing-capability precedence;
- standard method-not-found behavior for unknown requests;
- notification validation with no response, including unknown notifications;
- stable registration rules that Sprint 29 can extend without modifying the
  codec or stdio loop.

The dispatcher must not advertise a tool capability, tool list, or executable
method before its semantic owner and evidence exist.

### stdio transport

The ADR must define:

- newline-delimited UTF-8 input with one JSON-RPC message per line;
- one compact JSON response plus one newline for each response-producing
  request;
- no standard-output bytes other than encoded protocol messages;
- diagnostic output, if any, on standard error only;
- sequential processing as the simplest deterministic first-slice ordering,
  unless bounded concurrency is explicitly justified;
- flushing after a response so pipe clients do not deadlock;
- notification processing without a response line;
- whether malformed input is recoverable per line or terminates the service;
- distinct read, dispatch, serialize, write, flush, cancellation, and EOF
  outcomes;
- EOF as graceful completion and a composition mechanism that does not let the
  service container misclassify it as an unexpected service exit;
- prompt cancellation of blocked reads and joined writer cleanup;
- deterministic response to broken output pipes, without retry loops or
  protocol text on standard error.

The existing service contract makes EOF ownership a real integration issue.
Either the process shutdown future must observe MCP EOF, the container must gain
an accepted graceful-completion concept, or a dedicated process loop must own
stdio outside a long-running `RuntimeService`. The ADR must select one and
prove that HTTP and Workspace behavior remain unchanged.

## Deterministic validation matrix

### Protocol-domain tests

- minimal valid `server/discover` request with string and integer IDs;
- compatible metadata with optional client information and unknown extension
  fields;
- missing, null, wrong-type, duplicate, and reordered envelope fields;
- missing/wrong JSON-RPC marker, method, params, and required `_meta` entries;
- supported versus unsupported protocol revision;
- missing required capability where a registered method requires one;
- standard parse, invalid-request, invalid-params, and method-not-found errors;
- MCP-specific `-32020`, `-32021`, and `-32022` errors where applicable;
- notification versions of valid, invalid, and unknown methods with no result;
- deterministic compact encoding and identifier round trips;
- closed public error construction, mandatory MCP-specific error data, and
  exact/over-limit outbound response bounds;
- boundary-size, one-byte-over-limit, invalid UTF-8, embedded-newline, and
  duplicate-key fixtures after the ADR fixes those policies.

### In-memory transport tests

Platform-neutral in-memory streams should prove:

- multiple requests in one stream and repeated discovery;
- request/notification mixtures and response ordering;
- partial reads and a final line followed by EOF;
- escaped JSON newlines that remain within one frame;
- explicit flush and exactly one trailing newline per response;
- cancellation while the reader is blocked;
- controlled reader, writer, serialization, and flush failures;
- EOF, failure, and cancellation cleanup with no detached task;
- drop/release evidence after partial input and cancellation of a blocked
  reader;
- empty input and no-extra-output behavior.

These tests can remain in Runtime and use its existing Tokio dependency, so the
protocol crate does not need an async runtime dependency.

### Real process tests

A real child-process pipe test must:

1. spawn the selected MCP entry point with piped stdin, stdout, and stderr;
2. send a discovery request and assert the exact JSON-RPC result semantically;
3. send an unknown request and assert the exact error and echoed identifier;
4. send malformed known capability metadata and assert the exact invalid-params
   envelope;
5. send a notification and prove no response is produced;
6. close stdin, enforce a bounded wait, and require successful prompt exit;
7. assert stdout contains only expected newline-delimited JSON-RPC messages;
8. assert stderr is empty or contains only an explicitly documented diagnostic;
9. repeat enough of the sequence to detect stale session state or leaked
   process-global state.

Malformed and oversized cases may use separate child processes if the accepted
policy terminates the transport. Tests must use timeout guards so a broken EOF
or flush contract fails instead of hanging the suite.

## Preserved behavior and consumers

The accepted design must preserve:

- current Runtime HTTP health, configuration, and graph-query endpoints;
- Workspace loading, publication, cache behavior, and readiness transitions;
- CLI HTTP client behavior and exit-code/stdout/stderr contracts;
- Context Assembly, Tool Policy, provider, metadata, parser, graph, analysis,
  observability, and logging behavior;
- the absence of global mutable semantic state and transport-owned business
  logic.

Known future consumers are Sprint 29 semantic MCP tools and Sprint 35 external
client compatibility evidence. They need a stable registration and transport
boundary, not speculative capabilities in Sprint 28.

## Explicitly deferred scope

- semantic graph, context, or tool methods;
- tool execution, authorization interaction, or side effects;
- legacy MCP revisions and multi-version negotiation beyond reporting the one
  selected supported version;
- Streamable HTTP or other remote transports;
- authentication, authorization transport, TLS, and remote exposure;
- external MCP client execution and compatibility claims;
- distribution packaging, installers, editor configuration, and release UX;
- resources, prompts, sampling, elicitation, tasks, subscriptions, and other
  MCP features not required for discovery and dispatch extensibility.

## Unsupported assumptions

Current evidence does not support assuming that:

- an MCP SDK is necessary or preferable to the small first slice;
- an earlier initialization/session protocol remains compatible;
- empty capabilities may be replaced by an advertised future tool capability;
- newline framing alone imposes a safe memory bound;
- `serde_json` rejects duplicate keys by default;
- a normal Runtime service may return successfully on EOF without container
  changes;
- the current Runtime banner is safe on a protocol stdout channel;
- closing stdin is equivalent to delivering an operating-system signal;
- a live client or network service is needed for acceptance.

## Decisions still required in ADR-0050

ADR-0050 must accept or reject the ownership candidate and specify:

1. authoritative revision and exact supported-version behavior;
2. concrete wire types, validation order, result/error shapes, and identifiers;
3. resource limits, duplicate-key policy, recovery policy, and safe diagnostics;
4. discovery identity, optional fields, empty capabilities, and registration;
5. sequential or bounded-concurrent dispatch and response ordering;
6. stdio adapter ownership, flushing, EOF, cancellation, and failure behavior;
7. public entry point and its composition with the existing Runtime lifecycle;
8. exact dependency edges and the recorded approval gate;
9. unit, in-memory, lifecycle, and real-process acceptance oracles;
10. deferred compatibility and semantic-tool boundaries.

## Readiness conclusion

The repository and official sources provide enough evidence to write ADR-0050.
They do not yet authorize implementation. The remaining questions are design
choices that can be made deterministically in the ADR. Production dependency
edits remain blocked until the explicit approval gate is satisfied.
