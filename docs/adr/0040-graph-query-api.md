# ADR-0040: Graph Query API

## Status

Accepted

## Context

Sprint 18 must expose stable graph and semantic queries through the Runtime API.
The repository already has all required lower-level authorities but no accepted
product boundary between them:

- `SemanticGraph` is the canonical immutable fact owner;
- `SemanticGraphQuery` provides deterministic borrowed lookup, relation, and
  bounded traversal results over one graph snapshot;
- ADR-0039 publishes separate immutable per-configuration graphs through a
  cloneable Workspace observer;
- ADR-0037 owns Runtime service lifetimes, cancellation, and reverse cleanup;
- ADR-0038 owns one Axum listener and exact lifecycle-derived health behavior.

The [Sprint 18 investigation](../architecture/graph-query-api-investigation.md)
confirms complete production and test oracles for a bounded slice. It also
confirms that graph-domain values are borrowed and do not implement Serde,
`NodeKind` and `EdgeKind` have no accepted wire vocabulary, `HttpService` cannot
currently observe Workspace state, and the placeholder protocol and CLI crates
do not establish ownership. Existing in-process query behavior is therefore
not itself a stable Runtime or HTTP contract.

This ADR must preserve current graph semantics while fixing exact ownership,
selection, operation, limit, result, route, schema, error, lifecycle, and
compatibility contracts before implementation.

## Decision

### Semantic authority and dependency direction

`SemanticGraph` remains the sole owner of graph facts, identity, payload,
provenance, validation, and ordering. `SemanticGraphQuery` remains the sole
source-independent query semantics facade. Sprint 18 adds no node kind, edge
kind, semantic inference, resolution rule, graph mutation, index policy, or
adapter behavior.

`oneagent-runtime` owns a transport-neutral Graph Query component. It receives a
clone of `WorkspaceSnapshotObserver`, selects one immutable configuration by
canonical `EntityId`, delegates accepted operations to that configuration's
`SemanticGraphQuery`, and projects borrowed results into closed owned Runtime
values. Projection copies accepted fields but does not create canonical graph
objects or a second index.

The dependency direction is:

```text
WorkspaceSnapshotObserver
    -> Runtime Graph Query component
        -> SemanticGraphQuery

Runtime Graph Query component
    -> Runtime HTTP adapter
```

Graph, metadata, common, Workspace domain, and source adapters do not depend on
Runtime or HTTP. The HTTP adapter never reads graph storage directly.

The first slice remains in the reusable `oneagent-runtime` library. It does not
activate `oneagent-protocol`: that crate has no current schema, dependency, or
consumer authority, and Sprint 21 owns the first supported client. A later
explicit compatibility task may move or re-export the accepted owned contracts
without changing this wire API.

No new external production dependency is accepted. Runtime already depends on
the graph and Workspace authorities, Axum, Serde, Serde JSON, and Tokio.

### Construction and ownership

The composition root constructs dependencies explicitly:

1. construct one `WorkspaceService`;
2. obtain one `WorkspaceSnapshotObserver` before registration;
3. construct one cloneable transport-neutral Graph Query component from that
   observer;
4. construct the production `HttpService` with that query component;
5. register `http` before `workspace`, preserving ADR-0039 startup and reverse
   shutdown order.

`HttpService::new()` remains source-compatible and health-only. It registers no
Graph Query routes and treats those paths as ordinary unknown paths. A new
explicit constructor or builder accepted by the implementation enables the
complete Graph Query route set with one Graph Query component. Partial route
registration is not supported.

The Graph Query component owns only an observer clone. It is not an
ADR-0037 `RuntimeService`, starts no task, owns no listener or cancellation
source, and requires no service registration. Request handlers own request
decoding and response allocations. The Workspace service owns publication; the
HTTP service owns router state, listener, connection work, and graceful
shutdown. No graph, snapshot sender, listener, task, registry, or mutable status
is global or detached.

### Snapshot and lifecycle consistency

Each transport-neutral call obtains one `Arc<WorkspaceSnapshot>` from the
observer and holds that exact value for the whole call. It selects and queries
one `WorkspaceConfigurationSnapshot` inside that value. A single result can
never mix two snapshot observations. Snapshot pointer identity is not exposed
or serialized.

The transport-neutral component is snapshot-gated, not lifecycle-gated:

- `None` produces typed `WorkspaceUnavailable`;
- `Some(empty snapshot)` is a valid empty Workspace;
- an exact known Configuration ID selects one immutable graph;
- an unknown Configuration ID produces typed `ConfigurationNotFound`.

The HTTP adapter additionally requires canonical Runtime readiness before every
Graph Query call. When lifecycle is not `Running`, a registered Graph Query
route returns `runtime_not_ready` even if a snapshot is already present during
`Initializing` or remains present during early `Stopping`. Once `Running`, an
unexpected absent snapshot returns `workspace_unavailable`. This preserves
ADR-0038 as the only readiness authority and makes public query availability
total across observable lifecycle states.

One request retains its selected immutable snapshot even if Sprint 19 later
publishes a replacement. A later request observes the then-current value. This
is ordinary per-request snapshot consistency, not a stale-read, generation,
rebuild, invalidation, or cache policy. Sprint 19 must decide those later
semantics without changing successful Sprint 18 response schemas silently.

During shutdown, lifecycle enters `Stopping` before reverse service cleanup, so
new Graph Query requests return `runtime_not_ready` while HTTP remains reachable.
Workspace cancellation then clears the observer before the earlier-registered
HTTP service is cancelled. The listener still shuts down through ADR-0038 and
all admitted connection work remains structurally owned.

### Accepted first-slice operations

The first slice accepts exactly four operations:

1. list published Workspace configurations;
2. look up one node by exact Configuration ID and exact node ID;
3. list direct incoming or outgoing relations for one known node, optionally
   restricted to one exact edge kind;
4. traverse from one known node with exact direction, mandatory bounded depth,
   optional one-kind edge filter, and optional inclusion of the start node.

All operations are read-only. They use current `SemanticGraphQuery` behavior:

- configuration order is canonical Configuration `EntityId` order;
- node lookup is exact;
- direct edges use stable edge identity order;
- relations retain the stable edge-identity order of the current incoming or
  outgoing adjacency query;
- traversal is deterministic breadth-first traversal, visits each node once,
  and handles cycles and self-loops;
- an omitted edge kind means all graph edge kinds, not only dependency kinds.

Direct relation direction is literal graph direction:

- `outgoing` follows source to target;
- `incoming` follows target to source.

This is not the specialized dependency/usage policy. Clients can select one
edge kind but cannot submit a set in the first slice.

Known nodes with no matching relation return a successful empty list. Unknown
nodes are rejected before calling a collection method, so they do not collapse
into the same empty result. Empty Workspaces and unknown configurations are also
distinct.

Exact-name/kind search, ownership-specific routes, dependency/usage aliases,
transitive Subsystem membership, shortest path, declarations, references,
callers/callees, source definitions, Impact, diagnostics, validation, reports,
and reference ledgers are not first-slice operations. Their in-process APIs and
future product schemas remain unchanged and unclaimed.

### Request bounds

List, relation, and traversal operations accept one `limit`:

- omitted: `50`;
- minimum: `1`;
- maximum: `100`;
- syntax: non-empty ASCII decimal digits only, with leading zeroes permitted;
- zero, a sign, whitespace, non-decimal text, overflow, or a value above `100`
  is rejected.

The result is the first `limit` records in canonical deterministic order. The
response contains `truncated: true` exactly when more records existed and were
not returned. It contains no cursor, total count, next page, or continuation
token. Node lookup has no `limit`.

Traversal additionally requires `max_depth`:

- accepted range: `0..=4`;
- syntax: the same unsigned ASCII decimal rule;
- depth counts edges from the start node;
- `include_start` defaults to `false` and accepts exactly `true` or `false`;
- when included, the start record has depth `0` and no reason edge;
- with `max_depth=0` and `include_start=false`, the result is successfully empty.

The output limit is a response and allocation boundary at the Runtime result
projection layer. Current `SemanticGraphQuery::traverse` computes its complete
depth-bounded vector before projection; Sprint 18 makes no CPU, intermediate
allocation, denial-of-service, or performance guarantee. A truly work-budgeted
iterator requires a separate graph compatibility decision and benchmark or
security evidence. The mandatory depth ceiling, result ceiling, loopback
default, and absence of arbitrary queries are the accepted first-slice bounds.

### Stable owned projections

The Runtime defines owned values for Configuration, Node, Relation, and
Traversal records. Exact Rust type names may vary, but their semantic fields
and JSON projections are fixed below.

Configuration projection fields:

- `id`: canonical Configuration ID string;
- `name`: exact Configuration name;
- `format`: `edt` or `designer_xml`;
- `node_count`: canonical graph node count;
- `edge_count`: canonical graph edge count.

Node projection fields:

- `id`: canonical node ID string;
- `name`: exact canonical node name;
- `kind`: closed node-kind string;
- `metadata_kind`: closed metadata-kind string for `kind=metadata`, otherwise
  JSON `null`.

Relation projection fields:

- `edge_id`: canonical stable edge ID;
- `edge_kind`: closed edge-kind string;
- `source_node_id`: canonical source node ID;
- `target_node_id`: canonical target node ID;
- `related_node`: complete Node projection for the endpoint reached in the
  requested direction.

Traversal projection fields:

- `node`: complete Node projection;
- `depth`: unsigned edge depth;
- `via_edge_id`: stable first-discovery edge ID or JSON `null` for the included
  start node.

Graph payloads and provenance are excluded. Diagnostic messages, source roots,
reference ledgers, reports, validation details, Rust type names, pointer values,
and `Debug` or `Display` prose are also excluded. Adding any of them later is an
additive compatibility decision with complete variant coverage; fixture-local
partial payload serialization is prohibited.

### Closed value vocabularies

Direction values are exactly:

- `outgoing`;
- `incoming`.

Workspace format values are exactly:

- `edt`;
- `designer_xml`.

Edge kind values are exactly:

- `contains`;
- `calls`;
- `references`;
- `reads`;
- `writes`;
- `grants`;
- `includes`;
- `extends`;
- `depends_on`;
- `opens`;
- `triggers`.

Flat node kind values are exactly:

- `module`;
- `procedure`;
- `function`;
- `query`;
- `data_composition_schema`;
- `data_set`;
- `data_composition_field`;
- `xdto_type`;
- `http_service_url_template`;
- `http_service_method`;
- `web_service_operation`;
- `web_service_parameter`;
- `form`;
- `command`;
- `attribute`;
- `standard_attribute`;
- `tabular_section`;
- `dimension`;
- `resource`;
- `measure`;
- `role`;
- `access_right`;
- `subsystem`;
- `unknown`.

`NodeKind::Metadata(_)` uses `kind: "metadata"` and one exact
`metadata_kind` value:

- `configuration`;
- `subsystem`;
- `catalog`;
- `document`;
- `enumeration`;
- `common_module`;
- `report`;
- `data_processor`;
- `information_register`;
- `accumulation_register`;
- `accounting_register`;
- `calculation_register`;
- `business_process`;
- `task`;
- `role`;
- `common_form`;
- `form`;
- `command`;
- `template`;
- `http_service`;
- `web_service`;
- `xdto_package`;
- `event_subscription`;
- `unknown`.

These mappings are total over current `WorkspaceFormat`, `NodeKind`,
`MetadataKind`, and `EdgeKind`. Implementation must use exhaustive Rust matches
without a fallback that silently maps future enum variants to `unknown`.

### Versioned route and method matrix

The exact registered production routes are:

| Operation | Request |
| --- | --- |
| List configurations | `GET /api/v1/configurations?limit=<limit>` |
| Node lookup | `GET /api/v1/graph/node?configuration_id=<id>&node_id=<id>` |
| Direct relations | `GET /api/v1/graph/relations?configuration_id=<id>&node_id=<id>&direction=<direction>&edge_kind=<kind>&limit=<limit>` |
| Bounded traversal | `GET /api/v1/graph/traverse?configuration_id=<id>&node_id=<id>&direction=<direction>&max_depth=<depth>&edge_kind=<kind>&include_start=<bool>&limit=<limit>` |

Optional parameters may be omitted exactly where defaults are defined. The
path is exact and has no trailing-slash alias or redirect. Only GET is accepted.
Registered wrong methods, including HEAD and POST, return `405`, `Allow: GET`,
and an empty body through the existing explicit method filter. Unknown paths,
including an unknown `/api/v1` path and every trailing-slash variant, return the
existing `404` with an empty body.

Query parameter names are exact lowercase snake case. Parameters are decoded as
UTF-8 once according to URL query encoding. Unknown, duplicate, missing
required, empty required, invalid percent-encoding, or invalid UTF-8 parameters
are rejected as `invalid_query`. Values are case-sensitive. IDs are accepted
when the decoded value is non-empty and not whitespace-only, then preserved
exactly for canonical lookup; no trim, case folding, path normalization, or
UUID-only restriction is applied.

`configuration_id` and `node_id` are query parameters rather than path segments
because current canonical identity permits arbitrary non-empty strings and has
no path-segment compatibility rule.

The first slice ignores `Accept` and supports JSON only. Successful and domain
error responses use `content-type: application/json`. Content negotiation,
request bodies, POST queries, compression, cache headers, and HTTP/2 guarantees
are not accepted.

### Success schemas

Object member order is not a compatibility guarantee. Arrays retain the
deterministic order defined above.

List configurations:

```json
{
  "configurations": [
    {
      "id": "408a41e7-907a-4fb3-8999-83d1e8b6e093",
      "name": "DNSWorldEdition",
      "format": "designer_xml",
      "node_count": 4,
      "edge_count": 3
    }
  ],
  "truncated": false
}
```

Node lookup:

```json
{
  "configuration_id": "408a41e7-907a-4fb3-8999-83d1e8b6e093",
  "node": {
    "id": "408a41e7-907a-4fb3-8999-83d1e8b6e093",
    "name": "DNSWorldEdition",
    "kind": "metadata",
    "metadata_kind": "configuration"
  }
}
```

Direct relations:

```json
{
  "configuration_id": "408a41e7-907a-4fb3-8999-83d1e8b6e093",
  "node_id": "408a41e7-907a-4fb3-8999-83d1e8b6e093",
  "direction": "outgoing",
  "edge_kind": "contains",
  "relations": [
    {
      "edge_id": "<stable-edge-id>",
      "edge_kind": "contains",
      "source_node_id": "408a41e7-907a-4fb3-8999-83d1e8b6e093",
      "target_node_id": "dc24575c-a787-411d-93bd-494271291d73",
      "related_node": {
        "id": "dc24575c-a787-411d-93bd-494271291d73",
        "name": "DynamicSecurityOverridable",
        "kind": "metadata",
        "metadata_kind": "common_module"
      }
    }
  ],
  "truncated": false
}
```

Omitted `edge_kind` is serialized as JSON `null` in relation and traversal
responses. It is never omitted from a successful response.

Bounded traversal:

```json
{
  "configuration_id": "408a41e7-907a-4fb3-8999-83d1e8b6e093",
  "start_node_id": "408a41e7-907a-4fb3-8999-83d1e8b6e093",
  "direction": "outgoing",
  "edge_kind": "contains",
  "max_depth": 1,
  "include_start": false,
  "nodes": [
    {
      "node": {
        "id": "dc24575c-a787-411d-93bd-494271291d73",
        "name": "DynamicSecurityOverridable",
        "kind": "metadata",
        "metadata_kind": "common_module"
      },
      "depth": 1,
      "via_edge_id": "<stable-edge-id>"
    }
  ],
  "truncated": false
}
```

Every successful operation returns `200 OK`, including empty configuration,
relation, or traversal arrays.

### Typed failures and HTTP error schema

The transport-neutral component exposes distinguishable error categories for:

- Workspace unavailable;
- invalid identifier;
- Configuration not found;
- node not found;
- limit out of range;
- traversal depth out of range.

Transport parsing owns invalid query syntax, unsupported direction, unsupported
edge kind, and invalid boolean text. HTTP readiness owns runtime-not-ready.
Errors use one closed schema:

```json
{
  "error": {
    "code": "configuration_not_found",
    "message": "configuration was not found"
  }
}
```

The exact mapping is:

| Condition | Status | Code | Exact message |
| --- | ---: | --- | --- |
| Lifecycle is not `Running` | `503` | `runtime_not_ready` | `runtime is not ready` |
| Graph Query component observes no snapshot | `503` | `workspace_unavailable` | `workspace snapshot is unavailable` |
| Unknown Configuration ID | `404` | `configuration_not_found` | `configuration was not found` |
| Unknown node in a known configuration | `404` | `node_not_found` | `node was not found` |
| Empty or whitespace-only Configuration or node ID | `400` | `invalid_identifier` | `identifier must not be empty` |
| Unknown, duplicate, missing, empty, malformed, invalidly encoded, or otherwise structurally invalid query parameter | `400` | `invalid_query` | `query parameters are invalid` |
| Direction outside the closed vocabulary | `400` | `unsupported_direction` | `direction is unsupported` |
| Edge kind outside the closed vocabulary | `400` | `unsupported_edge_kind` | `edge kind is unsupported` |
| `limit` syntax or range failure | `400` | `limit_out_of_range` | `limit must be between 1 and 100` |
| `max_depth` syntax or range failure | `400` | `max_depth_out_of_range` | `max_depth must be between 0 and 4` |
| `include_start` is not exact `true` or `false` | `400` | `invalid_boolean` | `include_start must be true or false` |

Specific recognized-value errors take precedence over generic `invalid_query`
after the complete parameter set passes unknown/duplicate/missing checks. HTTP
readiness is checked after successful structural parsing but before snapshot or
graph lookup. Within the component, identifier validation precedes snapshot
lookup, Configuration selection precedes node selection, and bound validation
precedes graph operation execution.

No source-chain text, Rust error name, Axum rejection body, diagnostic message,
filesystem path, adapter error, graph payload, or backtrace is serialized.
Serialization of the closed static error schema is infallible for the accepted
surface. Unexpected internal failures remain Runtime diagnostics or task
failures; the first slice defines no reproducible `500` row and must not invent
one from `Display` prose.

### Compatibility contract

The `/api/v1` paths, GET-only method surface, parameter names, default and
maximum bounds, success fields, array ordering, enum strings, status codes,
error codes, and exact error messages are stable Sprint 18 compatibility
surface. Existing health paths, methods, statuses, fields, exact bodies, and
fallback behavior remain unchanged.

Within v1:

- adding an optional response field, enum value, operation, or route requires a
  compatibility review and complete producer/consumer evidence;
- removing or renaming a field, enum value, error code, or route; changing a
  status, default, bound, ordering, or semantic meaning; or making an optional
  parameter required is breaking and prohibited without version migration;
- adding a new `NodeKind`, `MetadataKind`, `EdgeKind`, or `WorkspaceFormat` to
  the domain must make the exhaustive Runtime projection fail compilation until
  an explicit wire decision is accepted;
- JSON object member order and incidental transport headers are not stable;
- diagnostic prose and internal Rust APIs remain outside wire compatibility.

The supported CLI does not exist until Sprint 21. Sprint 18 public loopback
tests are protocol conformance evidence, not a supported client implementation.

### Deterministic evidence

Focused transport-neutral tests must prove:

- absent, empty, and multiple-configuration snapshots;
- exact configuration and node selection;
- total exhaustive value mappings;
- outgoing and incoming direct relations, one-kind filtering, known empty
  results, and unknown node distinction;
- traversal directions, depths `0` and `4`, over-limit depth, include-start,
  cycles/self-loops through existing graph evidence, result truncation, and
  equal repeated calls;
- limits `1`, default `50`, `100`, zero, `101`, malformed, and overflow;
- immutable per-call snapshot retention without mutation or mixed results.

Focused HTTP and public loopback tests must prove every registered route,
required/optional/default parameter, success schema, closed enum vocabulary,
error row, wrong method, unknown path, trailing slash, malformed encoding,
duplicate/unknown parameter, readiness state, content type, and deterministic
ordering.

Public production evidence must use the tracked Sprint 17 fixture through real
filesystem discovery and both production builders. It must query both exact
Configuration IDs, node/detail and relation facts from both formats, bounded
traversal, invalid and missing selection, `Initializing`, `Running`, and
`Stopping`, snapshot clearing, listener release, and equal fresh runs. Synthetic
graph tests remain the oracle for cycles, self-loops, multiple owners, duplicate
names, and maximum-result truncation absent from the production fixture.

Tests use channels and watches as event evidence, loopback port zero, temporary
directories, and bounded hang guards. They use no arbitrary sleep, fixed port,
real signal, external service, ignored corpus, or platform-specific socket.
Managed environments that deny local bind must run loopback targets with
bounded local-network permission; `PermissionDenied` is not product evidence.

Every production Rust, public API, or manifest change runs focused affected
tests and the complete workspace validation gate.

## First production slice

Sprint 18 implements only:

1. exhaustive owned Configuration, Node, Relation, Traversal, direction, kind,
   and typed error projections in Runtime;
2. one cloneable transport-neutral Graph Query component over one Workspace
   observer;
3. exact selected-configuration node, direct-relation, and bounded-traversal
   operations plus configuration listing;
4. one query-enabled construction path for the existing HTTP service while
   retaining health-only `HttpService::new()`;
5. the four exact GET routes, request parser, limits, closed success/error JSON,
   and lifecycle gating defined above;
6. production composition with HTTP before Workspace;
7. focused and public production loopback evidence and truthful current-state
   documentation.

## Rejected alternatives

### Serialize `GraphNode`, `GraphEdge`, and payload enums directly

Rejected. It would expose borrowed domain internals, couple graph evolution to
wire compatibility, require incomplete payload/provenance decisions, and make
Rust enum spelling a protocol.

### Put HTTP schema and routing in `oneagent-graph`

Rejected. Graph is transport-independent and cannot depend on Runtime, Axum, or
Serde wire policy.

### Activate `oneagent-protocol` because it already exists

Rejected for the first slice. The placeholder has no accepted schema,
dependency, or client consumer. Moving the already accepted Runtime contract may
be evaluated with Sprint 21 client evidence.

### Store Workspace observation in global state

Rejected. ADR-0037/0039 require explicit composition and independent fresh
applications. A global registry would split ownership and leak state across
runs.

### Put a mutable query-ready flag in `AppState`

Rejected. Runtime lifecycle is the sole readiness authority and Workspace
observation already owns snapshot availability.

### Make the Graph Query component an ADR-0037 background service

Rejected. Querying is synchronous read-only work over an existing observer and
needs no task, cancellation source, or startup acknowledgement. The Workspace
and HTTP services already own the relevant lifetimes.

### Merge all configuration graphs or choose the first configuration

Rejected. ADR-0039 preserves separate canonical graphs and requires explicit
selection by stable Configuration ID.

### Use node and configuration IDs as path segments

Rejected. Current identity accepts arbitrary non-empty strings and has no stable
path-segment encoding restriction. Exact query parameters preserve the accepted
identity domain.

### Expose every current query method

Rejected. Several Semantic Model operations remain unimplemented; payload and
provenance schemas are unresolved; specialized and unbounded product behavior
would exceed the testable first slice.

### Provide an arbitrary query language or POST body

Rejected. No grammar, validation, complexity, compatibility, or security oracle
exists. Four closed GET operations are sufficient for the first slice.

### Return full results without limits

Rejected. Production graphs can have unbounded breadth from the HTTP adapter's
perspective. The accepted response cap keeps the first wire surface bounded and
truthfully reports truncation.

### Add cursor pagination now

Rejected. Snapshot generation, continuation identity across Sprint 19 rebuilds,
and cache consistency are undecided. `truncated` is explicit first-slice
behavior, not hidden pagination.

### Treat snapshot presence as public readiness

Rejected. A snapshot can appear during `Initializing` and remain during early
`Stopping`; ADR-0038 makes lifecycle the only readiness authority.

### Return Axum rejection text or domain `Display` messages

Rejected. Those strings are implementation diagnostics, not stable error
schemas, and may expose paths or source details.

## Deferred scope

- file watching, rebuild publication, generation identity, stale reads,
  invalidation, and request consistency across replacement: Sprint 19;
- persistent graph/snapshot cache, schema versioning, corruption, migration,
  and clean-rebuild equivalence: Sprint 20;
- supported CLI configuration discovery and Graph Query client: Sprint 21;
- exact-name/kind search, ownership aliases, dependency/usage aliases,
  transitive Subsystem membership, shortest path, declaration/reference/source
  queries, diagnostics, validation, reports, Impact, and reference ledger APIs;
- payload and provenance projection, source fragments, aggregate or
  cross-configuration graphs, mutation, batch requests, cursors, streaming,
  subscriptions, arbitrary query languages, and request cancellation;
- `oneagent-protocol` migration, MCP, LSP, IDE, AI/context, OpenAPI, general
  version negotiation, and alternate transports;
- authentication, authorization, TLS, CORS, compression, rate limiting, request
  IDs, metrics/tracing export, proxy policy, HTTP/2 compatibility, cache
  headers, retries, restart, forced termination, and timeouts;
- work-budgeted traversal, benchmarks, latency/allocation targets, denial-of-
  service claims, packaging, and performance or security certification.

## Implementation prerequisites

1. Implement exhaustive owned value mappings, projections, request option
   newtypes, bounds, typed errors, and the observer-backed transport-neutral
   component with focused synthetic tests.
2. Preserve `HttpService::new()` and add one explicit query-enabled construction
   path; inject the component into private router state without changing the
   listener or service-container contracts.
3. Implement strict query parsing with unknown/duplicate rejection and exact
   GET route, success, and error matrices.
4. Wire Workspace observer -> Graph Query component -> HTTP service in `main.rs`
   while retaining `http` before `workspace` registration.
5. Add non-zero raw-loopback public evidence over both production fixture
   formats and every accepted lifecycle, error, bound, cleanup, and repeated-run
   case.
6. Synchronize current-state documentation only after production and public
   evidence exists. Do not change graph or adapter Coverage support.
7. Run focused affected tests and the complete workspace validation gate.

## Coverage Registry impact

None. Sprint 18 exposes already accepted graph facts and query behavior through
Runtime. It adds no semantic graph or source-adapter capability and must not
transition either Coverage Registry.

## Consequences

- Runtime gains one stable bounded query surface without copying semantic
  authority or exposing graph internals.
- Clients select configurations explicitly and receive deterministic owned
  values with closed vocabularies.
- Public query availability follows canonical Runtime readiness and immutable
  per-request snapshot selection.
- The existing listener, health routes, service ownership, Workspace build, and
  reverse cleanup remain authoritative.
- Result truncation is explicit, while pagination and work-budget guarantees
  remain visible limitations.
- Domain enum growth cannot silently enter the v1 wire surface.
- Sprint 19 can add snapshot replacement without redefining successful Sprint
  18 payloads, but it must accept generation and stale-read semantics separately.
