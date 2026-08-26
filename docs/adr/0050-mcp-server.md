# ADR-0050: MCP Server

## Status

Accepted

## Context

Sprint 28 introduces the first public Model Context Protocol boundary before
semantic MCP tools exist. The boundary must be useful to Sprint 29 without
advertising future behavior, combining incompatible protocol eras, placing
business semantics in a wire crate, or contaminating a newline-delimited stdio
channel with Runtime output.

The [MCP Server investigation](../architecture/mcp-server-investigation.md)
confirms that `crates/protocol` is an inactive placeholder, while
`apps/runtime` owns Tokio and process composition. It also identifies two
integration constraints: `App::run` currently prints a banner to standard
output, and an ADR-0037 Runtime service that returns successfully is classified
as an unexpected exit. Reusing the current Runtime executable or service
container unchanged would therefore violate either stdio purity or EOF
shutdown semantics.

This decision follows the official MCP `2026-07-28` [base
protocol](https://modelcontextprotocol.io/specification/2026-07-28/basic),
[stdio transport](https://modelcontextprotocol.io/specification/2026-07-28/basic/transports/stdio),
and [schema](https://modelcontextprotocol.io/specification/2026-07-28/schema).
The versioned [TypeScript
schema](https://github.com/modelcontextprotocol/modelcontextprotocol/blob/main/schema/2026-07-28/schema.ts)
is the source of truth and its generated [JSON
Schema](https://github.com/modelcontextprotocol/modelcontextprotocol/blob/main/schema/2026-07-28/schema.json)
is a cross-check. These sources were retrieved on 2026-08-26.

## Decision

### Canonical statement

OneAgent implements one stateless MCP revision, `2026-07-28`, over a dedicated
`oneagent-mcp` newline-framed stdio process. `oneagent-protocol` owns bounded
JSON-RPC/MCP values, validation, encoding, discovery, and transport-independent
dispatch. Runtime owns async stream adaptation and the real process streams.
The first server exposes only `server/discover`, advertises `{}` capabilities,
and exits successfully and promptly on stdin EOF. It never uses the legacy
initialization/session protocol and never writes a non-protocol byte to stdout.

### Authority and compatibility

`PROTOCOL_VERSION` is exactly `"2026-07-28"`. The supported version list is
exactly `["2026-07-28"]` in that order. Every request is independent and must
carry its own required protocol version and client capabilities. No connection,
process, request history, prior discovery, client identity, or repeated ID
creates a session.

The server does not implement `initialize`, `initialized`, a session ID, an
older revision, or fallback probing. A request for another version receives
`UnsupportedProtocolVersion` rather than legacy behavior. Supporting another
revision requires a later ADR and conformance matrix.

### Ownership and dependency direction

`crates/protocol` owns:

- supported revision and resource-bound constants;
- request identifiers, metadata, client capability values, request and
  notification envelopes, result and error envelopes;
- parse, duplicate-key detection, validation, and compact serialization;
- server identity, discovery result, server capability values;
- deterministic method registration and transport-independent dispatch.

It owns no Tokio runtime, stream, file descriptor, process, Workspace state,
graph query, Context Engine, Tool Policy, provider, credential, logger, global
mutable state, background task, or semantic handler.

`apps/runtime` owns:

- the injected asynchronous line-stream adapter over Tokio `AsyncRead` and
  `AsyncWrite`;
- bounded reads, frame delimiters, flushes, cancellation selection, EOF, and
  I/O failure classification;
- the dedicated `oneagent-mcp` binary and its stdin, stdout, stderr, signal
  future, exit status, and final diagnostic;
- public in-memory and process composition surfaces needed by conformance
  tests.

The dedicated MCP process does not construct `App`, `ServiceContainer`, HTTP,
Workspace, or Graph Query in Sprint 28. Existing `oneagent-runtime` composition
and its banner remain unchanged. This is an explicit process boundary, not a
new exception inside ADR-0037 service completion. Sprint 29 may compose
existing semantic owners into the MCP process but must not reverse dependency
direction or move their behavior into `oneagent-protocol`.

Approved implementation requires these direct production dependency edges:

```text
oneagent-protocol ──▶ serde
oneagent-protocol ──▶ serde_json
oneagent-runtime  ──▶ oneagent-protocol
```

The exact `serde` and `serde_json` versions are already locked and directly
used by Runtime; `oneagent-protocol` is already a workspace member. No new
third-party package, version, MCP SDK, or protocol-to-Runtime edge is accepted.
Cargo changes remain blocked until the user explicitly approves all three
edges.

### Public protocol values

The protocol crate exposes strongly distinguished inbound request and
notification values and outbound result and error values. Callers cannot turn
a notification into a response-producing request. Public values may expose
owned JSON object content where method-specific adapters need it, but they do
not expose deserializer internals, transport state, or source errors.

`RequestId` is either:

- an owned UTF-8 string of at most 256 bytes; or
- a JSON integer representable by `serde_json` as `i64` or `u64`.

Empty request-ID strings are accepted because the selected schema does not
forbid them. String bytes and integer values are echoed exactly; integer lexical
spelling need not be preserved. `null`, boolean, fraction, exponent-form
number, array, object, and out-of-representation IDs are invalid. A request ID
must not be reused while outstanding according to MCP, but the sequential
first slice has at most one active request and retains no completed-ID registry.

Method names are non-empty UTF-8 strings of at most 256 bytes. The complete
message bound below also bounds metadata, params, unknown extension values,
and error data. JSON arrays and objects may nest at most 128 levels. Unknown
well-formed top-level, params, and `_meta` extension members are ignored or
retained only as needed by the selected handler; their presence does not alter
dispatch.

Known request metadata remains schema-validated even when Sprint 28 does not
consume it. `progressToken` is a string or number; the deprecated versioned
log-level value is one of the eight schema values; client information requires
string `name` and `version`, string-valued known optional text fields, and
well-typed icon objects. `_meta` keys follow the versioned prefix/name grammar.
Known client capabilities retain the selected schema shapes: `roots` is an
object; `sampling` and `elicitation` are objects whose known options are
objects; `experimental` and `extensions` map names to objects; extension names
are valid prefixed metadata keys. Unknown top-level client capability members
remain accepted with arbitrary JSON values because the schema is explicitly
open.

### Parse and validation precedence

For one size-bounded UTF-8 frame, validation is deterministic:

1. parse exactly one JSON value while rejecting a duplicate key at any object
   depth;
2. require a top-level object;
3. require `jsonrpc` to be the string `"2.0"`;
4. classify a message with an `id` member as a request and require a valid
   `RequestId`; otherwise require a valid method and classify it as a
   notification;
5. for a request, require `params` to be an object and `params._meta` to be an
   object;
6. require string `_meta["io.modelcontextprotocol/protocolVersion"]`, a
   schema-conformant object at
   `_meta["io.modelcontextprotocol/clientCapabilities"]`, valid metadata key
   names, and correct known optional metadata/client-information shapes;
7. reject an unsupported protocol version;
8. find the method, validate any required client capability, then validate
   method-specific params;
9. invoke the handler only after every preceding check succeeds.

Invalid JSON is `ParseError`. A duplicate key is syntactically parseable but
ambiguous and is `InvalidRequest`. A non-object, wrong JSON-RPC marker, invalid
or oversized ID, absent/invalid method, or inbound result/error object is
`InvalidRequest`. Missing or wrong-typed params, `_meta`, required metadata, or
method fields is `InvalidParams`. An unknown compatible request is
`MethodNotFound`. A well-typed unsupported version is
`UnsupportedProtocolVersion`. A missing capability is evaluated only for a
registered method that declares it and is `MissingRequiredClientCapability`.

Once a valid request ID is known, every subsequent error echoes it. Parse
errors and errors before a valid ID is established omit the `id` member, as the
selected MCP schema permits. Raw input, unknown values, params, client
identity, and parser source chains never enter protocol error data, `Display`,
or `Debug` output.

A valid notification has `jsonrpc: "2.0"`, no `id`, a valid method, and absent
or object params. No notification receives any response, including an unknown
method or invalid method-specific params. `server/discover` as a notification
is not executed because discovery is request-only. Malformed JSON cannot yet be
classified as a notification and produces the normal ID-less parse error.

### Closed error contract

The first slice emits only these errors:

| Name | Code | Message | Data |
| --- | ---: | --- | --- |
| Parse error | `-32700` | `Parse error` | omitted |
| Invalid request | `-32600` | `Invalid Request` | omitted |
| Method not found | `-32601` | `Method not found` | omitted |
| Invalid params | `-32602` | `Invalid params` | omitted |
| Internal error | `-32603` | `Internal error` | omitted |
| Missing required client capability | `-32021` | `Missing required client capability` | `{"requiredCapabilities": ...}` |
| Unsupported protocol version | `-32022` | `Unsupported protocol version` | `{"supported":["2026-07-28"],"requested":...}` |

The missing-capability data contains only the deterministic required capability
object registered for that method. Unsupported-version data contains the exact
accepted requested version string and exact supported list. `HeaderMismatch`
`-32020` is reserved by MCP for HTTP header/body mismatch and is never emitted
by this stdio-only slice.

Unexpected handler failure maps to the closed internal error without source
detail. Serialization failure is a terminal internal transport failure rather
than a second protocol response. No application-defined code in the MCP
reserved `-32020..=-32099` range is introduced.

Public constructors are fallible and preserve the same closed contract.
Standard errors cannot be constructed with an MCP-specific code that requires
data; the two MCP-specific constructors supply and validate their exact data.
Complete result construction checks the fully wrapped response. Construction
and encoding both reject a response above 1,048,576 bytes or above the accepted
128-level JSON nesting bound, so an unbounded handler value cannot reach the
transport writer. A handler construction failure becomes the small closed
internal error.

### Discovery and truthful capabilities

`server/discover` is the only registered public method. It requires exactly a
`RequestParams` object: the required `_meta` plus any unknown extension fields,
and no method-specific required value. Its deterministic result is:

```json
{
  "resultType": "complete",
  "supportedVersions": ["2026-07-28"],
  "capabilities": {},
  "_meta": {
    "io.modelcontextprotocol/serverInfo": {
      "name": "oneagent",
      "version": "<workspace package version>"
    }
  },
  "ttlMs": 0,
  "cacheScope": "public"
}
```

`ttlMs: 0` makes discovery immediately stale and avoids a promise about future
capability stability. `cacheScope: "public"` is truthful because every field is
static and independent of client identity, authorization context, Workspace,
or user data. Instructions, title, description, website, icons, experimental,
logging, completions, prompts, resources, tools, extensions, and all list-
changed flags are omitted.

Every successful result contains `resultType: "complete"`. Discovery includes
server information as shown. Other future results should include the same
server information unless a later accepted decision documents why not.

### Method registration and dispatch

The server is built from a registry ordered canonically by exact method name,
not caller insertion order. `server/discover` is reserved and registered by the
server constructor. Registering an empty, oversized, duplicate, or reserved
method fails construction before serving input. Each registration identifies:

- one exact method name;
- whether it accepts requests, notifications, or both;
- an optional exact required-client-capability object;
- one transport-independent handler.

One message invokes at most one handler. Dispatch is synchronous and
single-request in Sprint 28; a handler returns a complete result or closed
internal error before the next frame is dispatched. The registry performs no
I/O and starts no task. Reordered registration produces identical behavior.
Fresh server instances share no mutable state.

The public Sprint 28 constructor registers only discovery. Dummy registrations
may exist only in tests to prove duplicate, ordering, capability, params,
notification, and handler-failure behavior. Production cannot advertise a
capability unless the corresponding public methods and conformance evidence
exist.

### Framing and resource bounds

The Runtime adapter reads and writes a reliable byte stream using these rules:

- a frame ends at LF (`0x0A`);
- one optional CR immediately before LF is removed;
- the maximum frame payload is 1,048,576 bytes, excluding LF and the optional
  CR;
- the reader uses one fixed 8,192-byte scratch buffer and never accumulates more
  than the maximum frame plus that scratch buffer and delimiter state;
- an empty frame is passed to the decoder and yields an ID-less parse error;
- a non-empty final payload without LF is an incomplete frame and is not
  dispatched;
- invalid UTF-8 and an oversized or incomplete frame are terminal framing
  failures with no protocol response for that frame;
- malformed but size-bounded UTF-8 JSON yields one parse-error response and the
  adapter continues with the next frame;
- a literal embedded LF terminates the current frame; malformed fragments may
  produce parse errors but no partial fragment reaches a method handler;
- an escaped JSON newline remains ordinary content and is valid.

The 1,048,576-byte and 128-level limits apply independently to each fully
encoded outbound JSON-RPC response before the framing LF is added. Exact-bound
responses are accepted; a one-byte-over or over-depth response fails before
any writer call.

The adapter processes frames sequentially. For each response-producing request
it serializes one compact JSON value, writes exactly one LF, and flushes before
reading the next frame. Notifications produce no output and require no flush.
There is no response reordering, request queue, concurrent handler, detached
task, or retry. The protocol crate serializer itself emits no LF; framing owns
the delimiter.

No banner, log, tracing event, diagnostic, panic text, source error, startup
acknowledgement, or unrelated byte is written through the protocol writer.
The injected writer is its sole owner for the duration of the adapter call.

### EOF, cancellation, failure, and cleanup

The adapter has one structured owner and returns one terminal outcome:

- `EndOfInput` after LF-terminated frames when the next read returns EOF;
- `Cancelled` when the injected cancellation future completes while waiting
  for more input;
- a bounded typed failure for read, invalid UTF-8, oversized frame, incomplete
  EOF frame, encode, write, flush, or cancellation-source failure.

Cancellation is observed between frames and while blocked reading. Once a
response write begins it is completed and flushed or reported as a write/flush
failure; cancellation cannot intentionally leave a partial valid response. EOF
and cancellation are successful process outcomes. Every failure is terminal.
The adapter spawns no task, so returning proves that all adapter work and
borrowed streams are released.

The public `oneagent-mcp` binary constructs a fresh discovery-only server,
binds process stdin/stdout, and supplies `tokio::signal::ctrl_c()` as an
additional cancellation source. It is externally ready when construction has
succeeded and the input loop is polling; there is no separate wire or stderr
startup acknowledgement. A successful response to the first piped discovery
request is the deterministic readiness oracle.

Closing stdin produces `EndOfInput`, exit code 0, and no diagnostic. Ctrl-C
cancellation also exits 0 but is not required acceptance evidence. A terminal
transport failure exits non-zero and writes exactly one bounded English
diagnostic line to stderr with prefix `oneagent-mcp:` and a stable failure
category, never raw input or a source chain. Stdout remains protocol-only on
all terminal paths.

The process constructs no `App`, so its stdout cannot reach the existing
Runtime banner. The existing Runtime process continues to use ADR-0037
startup, cancellation, failure precedence, and cleanup unchanged.

### Deterministic acceptance evidence

Protocol-domain tests must cover:

- string, empty-string, signed, zero, and unsigned integer IDs and rejected ID
  kinds/bounds;
- valid discovery metadata, reordered/unknown members, and repeated requests;
- every known client capability shape, namespaced extension keys, optional
  progress/log/client-information/icon shape, and malformed counterparts;
- malformed JSON, non-object input, duplicate keys at every relevant depth,
  wrong marker, missing/wrong method, params, `_meta`, version, and capability;
- unsupported version and exact error data;
- unknown methods, method params, required capabilities, handler failure, and
  registration conflicts;
- valid and invalid notifications with no response;
- deterministic compact serialization, closed MCP-specific error data, exact
  and over-limit outbound size/depth behavior, and safe error formatting.

Injected-stream tests must cover positive, malformed, repeated, multi-frame,
CRLF, empty, boundary, one-byte-over-limit, invalid UTF-8, incomplete EOF,
embedded-LF fragments, escaped-newline content, partial reads, notification,
ordering, flush, read failure, write failure, cancellation of a pending reader,
EOF, stream release, cleanup, no-extra-output, and fresh-instance cases.

Platform-neutral child-process pipe tests must cover exact discovery,
unknown-method and malformed-capability envelopes, notification no-output,
exact stdout JSON lines, empty-or-accepted stderr, stdin-close exit 0, terminal
malformed-frame exit non-zero where applicable, bounded hang guards, and
repeated fresh processes. No real signal, fixed port, credential, remote
service, live MCP client, platform-specific pipe API, or tool action is
required.

The canonical full workspace gate and focused Runtime HTTP, Workspace, Graph
Query, lifecycle, and CLI compatibility targets must still pass.

## Consequences

### Positive

- The first public MCP boundary follows one current, pinned stateless revision.
- Discovery is conforming and truthful before semantic tools exist.
- Wire/dispatch semantics remain independent of Tokio and application state.
- Runtime reuses its existing async dependency without changing its existing
  HTTP process or service completion rules.
- Strict framing, bounds, channel ownership, and process tests make hangs,
  contamination, and unbounded input deterministic failures.
- Sprint 29 can register semantic handlers without changing JSON-RPC framing.

### Negative

- The server is intentionally useful only for discovery until Sprint 29.
- Strict duplicate-key, ID-size, frame-size, and final-newline policies reject
  some inputs that a more permissive implementation could accept.
- The dedicated binary creates a second Runtime-package entry point whose
  future semantic composition must remain synchronized with shared owners.
- Synchronous dispatch cannot host long-running concurrent methods without a
  later lifecycle and ordering decision.
- Only the latest selected revision is supported; legacy clients require later
  compatibility work.

## Rejected alternatives

### Use an MCP SDK

Rejected for Sprint 28. The slice is small, the selected revision is new, and
an SDK would add an unapproved dependency and its own lifecycle/compatibility
surface without evidence that it matches OneAgent ownership or bounds.

### Implement legacy initialization and sessions

Rejected. It combines incompatible eras with the stateless `2026-07-28`
request metadata model and creates state the selected protocol forbids the
server from inferring.

### Advertise tools, resources, prompts, or logging early

Rejected. No corresponding production handler or conformance evidence exists.
Empty capabilities are the only truthful Sprint 28 result.

### Put Tokio or semantic services in `oneagent-protocol`

Rejected. It reverses dependency direction and makes a wire/dispatch library
own transport runtime or business behavior.

### Add MCP stdio to the existing Runtime executable

Rejected for this slice. Its stdout banner, HTTP/Workspace lifecycle, external
Ctrl-C shutdown, and service unexpected-exit rule conflict with a pure,
EOF-owned stdio process. Changing those established contracts is unnecessary.

### Treat MCP as an ADR-0037 Runtime service

Rejected for Sprint 28. Normal stdin EOF is success for MCP but a successful
service return is an unexpected Runtime failure. Expanding the container's
completion vocabulary without another service consumer is not the minimal
change.

### Concurrent dispatch

Rejected for the first slice. It adds ID tracking, response ordering, bounded
queues, cancellation routing, and task ownership without a method that needs
concurrency.

### Recover from invalid UTF-8, oversized, or incomplete frames

Rejected. No reliable request ID is available, recovery complicates bounded
draining, and terminating is deterministic and fail-closed. Recoverable valid
UTF-8 JSON syntax errors remain line-local.

### Buffer an unterminated final JSON value at EOF as a message

Rejected. The selected stdio binding requires newline-delimited messages and
EOF is the shutdown signal. Dispatching an unterminated value would blur those
contracts.

## Implementation prerequisites

1. Record explicit user approval for the three production dependency edges.
2. Implement and commit the protocol domain before registry/dispatch.
3. Implement and commit discovery/dispatch before stdio adaptation.
4. Implement and commit injected stdio before the real process binding.
5. Keep each task within its prompt-owned files and pass the full workspace
   gate before the next task.
6. Do not synchronize public current-state documentation until public and real
   process conformance passes.

## Deferred scope

- semantic graph, context, and tool methods and capability advertisement;
- tool execution, confirmation interaction, and Tool Policy adapters;
- asynchronous/concurrent handlers and `notifications/cancelled` execution;
- MCP revisions other than `2026-07-28` and legacy fallback;
- Streamable HTTP, SSE, socket, and other transports;
- auth, TLS, remote exposure, and credentials;
- resources, prompts, sampling, elicitation, tasks, subscriptions, logging,
  completions, extensions, and server-initiated interaction;
- external MCP client compatibility claims;
- editor integration, packaging, installers, and release UX.
