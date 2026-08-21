# Sprint 18 Graph Query API Investigation

## Status and purpose

This document records repository evidence for Sprint 18 Graph Query API at
committed planning baseline `9e9ab1c062c7e934bc3f7a933ad6215fac1d9ff6`.
It does not select routes, wire schemas, result fields, limits, error codes, or
implementation ownership. Those decisions belong to ADR-0040.

The investigation found sufficient repository-owned production paths and test
oracles for a bounded Runtime API. No external source, service, ignored corpus,
new semantic fact, or speculative graph operation is required.

## Accepted constraints

- [ADR-0008](../adr/0008-semantic-model-2-knowledge-graph.md) requires a
  storage-independent graph query boundary and prohibits exposing internal
  collections directly.
- [ADR-0026](../adr/0026-semantic-index-boundary.md) keeps `SemanticGraph` as
  canonical fact authority and `SemanticGraphQuery` as the source-independent
  read facade. Public query compatibility cannot change without a separate
  decision and consumer audit.
- [ADR-0027](../adr/0027-incremental-semantic-index-maintenance.md) preserves
  complete-snapshot query behavior and keeps index state crate-internal.
- [ADR-0037](../adr/0037-runtime-service-container.md) assigns composition,
  service task ownership, cancellation, and complete reverse cleanup to
  `oneagent-runtime`.
- [ADR-0038](../adr/0038-http-api-health.md) fixes one Runtime-owned Axum
  listener and the exact unversioned health contract. Domain APIs must not turn
  health routes into a versioned API or create a second readiness authority.
- [ADR-0039](../adr/0039-workspace-service.md) publishes separate immutable
  per-configuration graphs through a cloneable observer. Consumers must select
  a configuration; merged and cross-configuration semantics are deferred.
- The [Sprint 17 review](../reviews/sprint-17-workspace-service.md) records
  `pass` and leaves Graph Query API behavior explicitly unimplemented.

## Confirmed graph query boundary

### Canonical storage and derived lookup

`oneagent-graph` owns all graph facts. `SemanticGraph` stores nodes and edges,
returns a `SemanticGraphQuery<'_>` through `query()`, and retains direct
`node`, `nodes`, `edges`, validation, report, diff, resolution, and Impact
surfaces. `SemanticGraphQuery` borrows exactly one graph snapshot. Its public
methods build or use the crate-internal deterministic `SemanticIndex`; neither
the query facade nor the index creates or changes facts.

The public query facade returns borrowed `GraphNode` and `GraphEdge` values for
most operations. `SemanticGraphRelation` also borrows both values. Only
`SemanticGraphTraversalNode` is already an owned query result record. Therefore
none of those borrowed types can be retained as a transport response after the
selected graph borrow ends. A Runtime boundary must project accepted fields
into owned values without copying graph authority.

### Implemented operation inventory

All methods below exist in `crates/graph/src/query.rs` and are covered by the
non-zero `crates/graph/tests/query.rs` target.

| Family | Current operations | Confirmed behavior |
| --- | --- | --- |
| Identity | `node`, `node_by_entity_id`, `contains_node`, `edge`, `contains_edge`, `SemanticGraphQuery::edge_id` | Exact stable lookup; missing values return `None` or `false`. |
| Node collections | `nodes`, `nodes_by_kind`, `nodes_by_name`, `nodes_by_name_and_kind` | Results use deterministic node-identity order. Name matching uses exact `GraphNode::name`, not fuzzy, tokenized, or ranked search. |
| Containment | `owners`, `owner`, `owner_edges`, `children`, `children_by_kind` | Only `Contains` participates. All owners remain observable; `owner` returns `None` unless exactly one exists. |
| Edge collections | `edges`, `edges_by_kind`, `outgoing_edges`, `incoming_edges`, and kind-filtered variants | Results use deterministic stable edge-identity order. Unknown nodes return empty collections. |
| Neighbors | downstream/upstream helpers and `neighbors` with `SemanticGraphEdgeFilter` | Results are unique by node identity and sorted. `All` or an exact set of existing edge kinds is supported. |
| Dependency navigation | direct dependencies/usages and kind/set filtered variants | The closed dependency policy is `Calls`, `References`, `Reads`, `Writes`, `DependsOn`, and `Opens`. `Contains`, `Grants`, `Includes`, `Extends`, and `Triggers` are excluded. Relations are ordered by related node identity and stable edge identity. |
| Traversal | `traverse` with direction, edge filter, mandatory `max_depth`, and `include_start` | Deterministic breadth-first traversal, one result per node, cycle/self-loop safe. Unknown starts return an empty result. The API has a depth bound but no result-count budget. |
| Specialized composition | `transitive_subsystem_members` | Derives nested Subsystem membership through `Includes`; it creates no closure edge and returns an empty result for unknown or non-Subsystem starts. |

`SemanticGraphEdgeFilter::any` deduplicates kinds through `BTreeSet`. An empty
accepted set is currently a valid filter that matches no edge. There is no
string parser or public transport vocabulary for direction, node kind, edge
kind, or filters.

### Planned operations without implementation evidence

Semantic Model 2.0 mentions qualified-name lookup, declaration and source
definition search, references, callers, callees, shortest path, dependency
closure, and reverse dependency closure. The live graph facade has no public
methods with those contracts, except that callers/callees may sometimes be
expressed through existing filtered relation operations and dependency closure
may sometimes be expressed through bounded traversal. Treating those
equivalences as a stable public API is an architecture decision, not confirmed
repository behavior.

The graph has no pagination, cursor, result-count budget, serialized snapshot,
arbitrary query language, fuzzy/full-text search, cross-configuration query,
mutation command, streaming result, or request cancellation API. None may be
claimed by Sprint 18 without additional accepted evidence and scope.

### Identity and value vocabulary evidence

- `EntityId` is the canonical graph storage identity. Construction rejects an
  empty or whitespace-only string and otherwise preserves the supplied string.
- `NodeId` and `EdgeId` are stable public graph wrappers with string accessors.
  `NodeId::new` itself performs no validation. Query conversion to `EntityId`
  makes an empty or whitespace-only `NodeId` behave as absent.
- Stable edge identity is centralized over source identity, target identity,
  and `EdgeKind`; it is already shared by query, diff, validation, and Impact.
- `GraphNode` exposes ID, exact name, `NodeKind`, one closed `GraphNodePayload`,
  and ordered provenance. `GraphEdge` exposes source, target, `EdgeKind`, and
  ordered provenance.
- `NodeKind` contains `Metadata(MetadataKind)` plus 24 flat semantic variants.
  `EdgeKind` contains 11 variants. `MetadataKind` has an accepted
  machine-readable `as_str`; `NodeKind` and `EdgeKind` do not.
- Provenance exposes optional source ID, producer ID, `FactOrigin`,
  `Confidence`, and `ResolutionState`. These enum types also have no accepted
  wire vocabulary.
- Graph, common, metadata, identity, node, edge, payload, and provenance types
  do not derive Serde serialization. The only production Serde response in
  `oneagent-runtime` is the private single-field HTTP health response.

Consequently, Rust `Debug` text, enum variant spelling, `Display` prose, or
automatic serialization of graph-domain types is not a stable API. ADR-0040
must explicitly select every exposed field and closed string vocabulary. It
must also decide whether the bounded first slice excludes typed payload and
provenance detail or defines a complete owned projection for them. Partial
serialization of only whichever payload variant a fixture happens to contain
would be unsupported.

## Confirmed Workspace selection and lifetime boundary

### Published state

`WorkspaceService` owns a Tokio watch sender with value
`Option<Arc<WorkspaceSnapshot>>`. Before registration, the composition root can
obtain a `WorkspaceSnapshotObserver`. The observer can return the current value
or create another receiver. It cannot publish, mutate, select, rebuild, or
cancel.

The service performs one complete initial build before startup acknowledgement,
publishes exactly one immutable snapshot on success, and retains it while the
service task waits for receiver-only Runtime cancellation. Cancellation replaces
the value with `None`; dropping the final sender closes subscriptions. Startup
failure publishes nothing. ADR-0039 permits Sprint 19 to replace snapshots in
the future, but the current implementation changes only `None -> Some -> None`.

Runtime lifecycle is the only readiness authority. A complete snapshot can be
visible briefly while the application remains `Initializing`, and it remains
visible during reverse cleanup until the Workspace service is cancelled.
Transport behavior must therefore decide whether snapshot presence alone is
query availability or whether lifecycle additionally gates requests. Existing
ADRs intentionally do not decide that question.

### Configuration records

`WorkspaceSnapshot::configurations()` returns separate records sorted by
canonical Configuration `EntityId`. `configuration(&EntityId)` performs exact
binary-search lookup. An empty successfully discovered Workspace has a
published empty snapshot. `WorkspaceConfigurationSnapshot` exposes:

- source root and detected `WorkspaceFormat`;
- exact Configuration ID and name;
- `Arc<SemanticGraph>` through a borrowed `graph()` accessor;
- ordered semantic diagnostics;
- the canonical reference-request ledger and statistics;
- the deterministic graph report.

No method selects by root path, source format, name, ordinal position, or a
default configuration. Equal names with different IDs are not collisions.
Unknown configuration identity returns `None`. ADR-0039 requires explicit
configuration selection and rejects merged cross-configuration authority.

The Runtime currently exports the snapshot, configuration record, and observer
types. No Runtime query service, stable query request/result, graph HTTP route,
or query error type exists.

## Confirmed HTTP and composition boundary

`HttpService` is the sole public HTTP service and listener owner. It binds the
configured address before startup acknowledgement, publishes the actual address
through a read-only watch receiver, builds a private Axum router, and returns one
Runtime-owned server task. Runtime cancellation drives graceful shutdown and
address clearing. There is no second listener, router registry, or global
state.

The router currently receives only `Arc<AppState>`. `AppState` contains immutable
Runtime configuration and a read-only lifecycle-derived health view. It does
not contain Workspace observation or a query dependency. `main.rs` creates a
`WorkspaceService`, obtains an observer, assigns it to an unused local, then
registers `http` before `workspace`. No request handler can currently reach a
Workspace snapshot.

The only accepted routes are:

| Request | Result |
| --- | --- |
| `GET /health/live` | `200`, JSON media type, exact `{"status":"alive"}` |
| `GET /health/ready` while `Running` | `200`, exact `{"status":"ready"}` |
| `GET /health/ready` otherwise while reachable | `503`, exact `{"status":"not_ready"}` |
| Wrong method on a health route | `405`, `Allow: GET`, empty body |
| Unknown exact path or trailing slash | `404`, empty body |

Health paths are deliberately unversioned. Graph paths, versioning, methods,
parameters, success statuses, schemas, error media types, stable error codes,
malformed encoding, duplicate parameter, trailing-slash, and unavailable-state
behavior are all unresolved. Axum 0.8.9, Serde, Serde JSON, and Tokio are already
direct Runtime dependencies, so a bounded API can be implemented and tested
without a new external production dependency.

The construction seam is also unresolved. Repository-feasible choices include
passing an observer-derived query dependency to `HttpService`, extending
immutable `AppState`, or constructing a transport-neutral query object shared by
composition. The current evidence does not select among them. Any accepted
choice must preserve HTTP-before-Workspace startup, immutable ownership, no
hidden service construction, no second listener, and complete reverse cleanup.

## Consumer and dependency inventory

Current `SemanticGraphQuery` consumers are inside `oneagent-graph`, EDT
production resolution/emission code, adapter and graph tests, and Impact. They
use borrowed graph values and current deterministic semantics. No Runtime,
protocol, CLI, MCP, LSP, IDE, or external supported client consumes it.

`oneagent-runtime` already depends on the graph, Workspace, both production
adapters, common/metadata/workspace domain crates, Axum, Serde, Serde JSON, and
Tokio. `oneagent-graph` depends only on common and metadata. The intended
dependency direction forbids graph from depending on Runtime or transport.

`oneagent-protocol` contains only `component_name()` and has no dependencies.
`oneagent-cli` prints a placeholder message and also has no dependencies.
Architecture documents describe both as foundations, not supported protocol or
client implementations. Sprint 21 owns the supported Runtime client. Whether
Sprint 18 introduces shared protocol types in `oneagent-protocol` or keeps the
first stable contracts in the reusable Runtime library is unresolved. Selecting
the protocol crate would also require an explicit dependency and serialization
ownership decision; the placeholder itself is not proof.

No repository evidence requires a new external production dependency for the
first slice. Any proposed dependency addition must be justified separately and
must not be inferred from this investigation.

## Repository-owned fixtures and executable oracles

### Production fixture

`apps/runtime/tests/fixtures/workspace_service/` is a tracked, LF-normalized,
SHA-256 inventoried public Runtime fixture. Its README records source provenance
for every file. Production discovery finds exactly two independent
configurations:

| Format | Configuration ID | Name | Confirmed graph observation |
| --- | --- | --- | --- |
| Designer XML | `408a41e7-907a-4fb3-8999-83d1e8b6e093` | `DNSWorldEdition` | 4 nodes, 3 edges, no diagnostics or requests |
| EDT | `50000000-0000-0000-0000-000000000000` | `WritesFixture` | 13 nodes, 14 edges, 3 diagnostics, 1 request, and preserved Reads/Writes-related evidence |

The two IDs sort in the order above and are stable positive configuration
selection oracles. Designer contributes a Configuration, Common Module, Module,
Procedure, and their ownership chain. EDT contributes Configuration,
Documents, Accumulation Registers, modules, procedures, Query, Reads, Writes,
and dependency-related relations. Existing public tests independently validate
both graphs before observing counts.

Negative tests copy the fixture into temporary directories and deterministically
cover missing and non-directory roots, conflicting format markers, duplicate
Configuration identity, and fatal later-adapter input without modifying tracked
bytes. A temporary empty root proves a valid published empty snapshot.

### Non-zero existing test targets

The following live `--list` commands completed successfully:

| Command | Non-zero evidence |
| --- | ---: |
| `cargo test -p oneagent-graph --test query -- --list` | 19 query tests |
| `cargo test -p oneagent-runtime workspace::tests -- --list` | 9 Workspace unit tests in the library target |
| `cargo test -p oneagent-runtime --test workspace_service -- --list` | 6 public production Workspace tests |
| `cargo test -p oneagent-runtime --test http_health -- --list` | 4 public loopback HTTP tests |

The `workspace::tests` filter reports zero tests in the Runtime binary and
integration binaries; only its 9 matching library tests are evidence. No
filtered zero count is treated as a pass.

Graph query tests prove exact identity and name lookup, kind queries, edge
identity and adjacency order, multiple containment owners, filtered neighbors,
dependency and usage policy, Reads versus DependsOn distinction, Opens and
Triggers classification, cycle-safe bounded traversal, payload visibility, and
deterministic transitive Subsystem membership. They do not prove any wire
schema or Runtime integration.

Workspace tests prove both production builders, empty and multiple
configurations, invalid roots, duplicate identity, later failure atomicity,
snapshot publication and clearing, health across `Initializing`, `Running`, and
`Stopping`, closed observation, and equal fresh runs. HTTP tests prove raw
loopback request parsing, exact health success and negative wire matrices, bind
failure, graceful release/rebind, and equal fresh runs. Tokio watch/oneshot
channels provide event synchronization; one-second timeouts are hang guards,
not timing assertions.

CI runs the complete workspace gate on `macos-14` and `windows-latest`.
Temporary directories and loopback TCP are established cross-platform seams.
No symlink, Unix socket, fixed port, real process signal, arbitrary sleep,
external service, or ignored local corpus is needed.

The active managed Codex filesystem/network sandbox rejects local
`TcpListener::bind(127.0.0.1:0)` with `PermissionDenied` and `Operation not
permitted`. The public Workspace and HTTP targets therefore initially closed
their address observers before publication when run inside that sandbox. The
same unchanged targets passed outside the network sandbox with all 6 Workspace
tests and all 4 HTTP tests successful. This is an execution-environment
constraint, not Runtime failure evidence; future loopback validation in this
environment must request the same bounded local-bind permission.

## Candidate bounded evidence matrix for ADR-0040

This table identifies testable repository evidence; it does not accept the
operation or wire contract.

| Candidate concern | Positive oracle | Negative or boundary oracle |
| --- | --- | --- |
| Configuration discovery/listing | Two fixture configurations in stable ID order | Published empty snapshot |
| Configuration selection | Both exact fixture IDs | Unknown, empty/whitespace, or malformed identity as separately decided outcomes |
| Node identity/detail | Exact Configuration/Common Module/Module/Procedure and EDT Query/Register nodes | Unknown or invalid node ID |
| Direct relations | Designer ownership chain; EDT Contains/Reads/Writes/DependsOn relations | Unknown node, empty edge filter, unsupported vocabulary |
| Bounded traversal | Existing graph cycle/depth tests plus production ownership/dependency paths | Depth zero, accepted maximum, over-limit depth, cycle/self-loop synthetic graph evidence |
| Deterministic order | Existing graph reversed-insertion tests and repeated fresh Runtime builds | Duplicate names and multiple relations in graph tests |
| Snapshot availability | Published snapshot before/through `Running` | No publication after startup failure; valid empty snapshot; clearing during shutdown |
| HTTP compatibility | Existing raw loopback parser and exact health matrix | Wrong method, unknown path, trailing slash, malformed encoding/parameters as ADR-defined cases |
| Cleanup and repetition | Existing observer closure, address clearing, listener rebind, two fresh runs | Startup failure with no published snapshot and no listener leak |

The production fixture is sufficient for public selected-configuration node and
relation evidence. Synthetic graph tests remain necessary for duplicate names,
multiple owners, cycles, self-loops, and exact maximum-bound behavior not
present in the production fixture. A new fixture is not currently justified.

## ADR-0040 decision matrix

ADR-0040 must answer all of the following before implementation:

1. Which layer owns the transport-neutral request, owned result, and typed
   error types, and what dependency direction keeps graph and adapters free of
   Runtime or HTTP concerns?
2. How is the current Workspace observer supplied to the query boundary and
   HTTP service without hidden construction, a global registry, or a second
   state authority?
3. Does query availability require only a published snapshot, canonical
   `Running`, or another already accepted combination during `Initializing` and
   `Stopping`?
4. Which exact configuration identity syntax is accepted, and how are absent
   snapshot, empty Workspace, unknown configuration, invalid identity, and
   cleared snapshot distinguished?
5. Which existing `SemanticGraphQuery` operations form the first slice? For
   each operation, which parameters are required, optional, mutually exclusive,
   or unsupported?
6. What mandatory depth and result-count limits prevent unbounded request work?
   What are the exact inclusive boundaries and over-limit outcomes?
7. Which node, edge, relation, traversal, configuration, and error fields are
   exposed as owned results? Are payload and provenance excluded or completely
   projected?
8. What closed machine-readable vocabularies represent node kind, nested
   metadata kind, edge kind, direction, origin, confidence, resolution, and
   every other exposed enum?
9. Which exact versioned paths, methods, path/query parameters, success statuses,
   media types, JSON bodies, and stable error codes are accepted?
10. How do wrong methods, unknown paths, trailing slashes, missing, duplicate,
    malformed, percent-encoded, unsupported, and over-limit parameters map to
    responses without exposing implementation prose?
11. Is the existing `oneagent-protocol` placeholder in scope, or does the first
    reusable contract remain in `oneagent-runtime`? What explicit compatibility
    and dependency impact follows?
12. Which request work, observer clones, router state, and response allocations
    are owned per request or service, and what terminates them during graceful
    shutdown?
13. Which public loopback and focused synthetic tests prove every accepted row,
    health compatibility, deterministic order, cleanup, and fresh repetition?
14. Which graph operations, aggregate semantics, watcher/invalidation, cache,
    CLI, alternate transports, security, and performance concerns remain
    deferred?

## Unsupported assumptions and hard boundaries

- Existing in-process query methods are not an HTTP contract.
- Current Rust enum names are not accepted wire values.
- `Debug` and `Display` output is not a stable schema or error body.
- Snapshot presence does not independently redefine Runtime readiness.
- Empty query results do not by themselves distinguish unknown input from a
  known node with no relations; any public distinction requires an explicit
  pre-check and accepted error policy.
- `oneagent-protocol` being a workspace member does not assign it Sprint 18
  ownership.
- The Sprint 17 fixture does not justify every graph payload variant or every
  node/edge enum variant as a serialized compatibility claim.
- Depth-bounded traversal is not result-count bounded; an API must not expose an
  unreviewed resource surface.
- Sprint 18 does not define snapshot replacement, stale reads, or request
  consistency across a watcher update. Sprint 19 owns rebuild orchestration.
- No repository evidence supports merged graphs, cross-configuration traversal,
  mutation, arbitrary query languages, streaming, authentication,
  authorization, rate limiting, OpenAPI, or benchmark claims in this slice.

## Decision readiness

The repository is ready for ADR-0040. Canonical semantics, immutable source
state, public Runtime ownership, a real listener, both production builders,
stable fixture identities, negative cases, deterministic ordering, lifecycle
observation, cleanup, repeated runs, and cross-platform validation are all
available locally.

The unresolved items are architecture choices with complete observable oracles,
not missing external data. ADR-0040 can therefore select one bounded first slice
without inventing source artifacts or semantic behavior. If it cannot close
every operation, vocabulary, limit, route, schema, error, lifecycle, ownership,
and compatibility row above, production implementation must stop.
