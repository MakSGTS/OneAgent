# ADR-0039: Runtime Workspace Service

## Status

Accepted

## Context

Sprint 17 must add the first Runtime-owned Workspace lifecycle and semantic-build
orchestration service. The
[Workspace Service investigation](../architecture/workspace-service-investigation.md)
confirms that the repository already has:

- the source-independent `WorkspaceDetector` port and deterministic
  `FileSystemWorkspaceDetector` for EDT and Designer XML roots;
- production EDT and Designer XML semantic graph builders;
- canonical deterministic `SemanticGraph` values, validation, reports,
  diagnostics, reference-request evidence, and provenance;
- an ordered Runtime service container with named failures, complete task
  ownership, lifecycle-derived readiness, cancellation, reverse cleanup, and
  public deterministic integration seams.

The accepted components do not define Runtime Workspace root configuration,
multi-configuration snapshot shape, build dispatch, publication atomicity,
configuration collisions, blocking-task ownership, startup acknowledgement,
or Workspace observation. The EDT builder returns a graph plus recoverable
diagnostics and reference evidence; the Designer builder returns a graph and
requires the caller to choose `Complete` or `Partial`. Runtime currently depends
on neither adapter and publishes no Workspace state.

This ADR defines the bounded initial-build service only. Graph-query APIs, file
watching, persistent cache, and supported CLI behavior remain later sprints.

## Decision

### Ownership and dependency direction

`oneagent-runtime` owns Workspace orchestration and is the composition root.
It may add path dependencies on these existing workspace members:

- `oneagent-workspace` for discovery and format contracts;
- `oneagent-workspace-fs` for production filesystem discovery;
- `oneagent-edt` and `oneagent-designer-xml` for production semantic builds;
- `oneagent-graph`, `oneagent-metadata`, and `oneagent-common` only for their
  source-independent graph, metadata-kind, identity, diagnostic, request, and
  report types.

These are local path dependencies already present in the locked workspace. No
new external production dependency is accepted.

Runtime selects and invokes adapters but does not parse source, convert one
source layout to another, change graph facts, or create an adapter-independent
replacement for `SemanticGraph`. The detector remains replaceable through its
domain port. EDT and Designer XML remain independent producers.

### Runtime configuration

`RuntimeConfig` gains one Workspace root `PathBuf`:

- `RuntimeConfig::new` and `Default` preserve their existing signatures and use
  `.` as the backward-compatible development root;
- a builder-style `with_workspace_root(impl Into<PathBuf>)` override supplies
  explicit test and application roots;
- the configured path is passed unchanged to `WorkspaceDetector::discover`;
  Runtime does not canonicalize, rewrite, create, or traverse it independently;
- the production detector's default recursion limit of six is the accepted
  first-slice policy; depth is not a new Runtime configuration field;
- detector validation owns missing, non-directory, unreadable, symlink-sensitive
  Designer, conflicting-marker, boundary, and ignored-directory behavior.

Relative-path interpretation therefore remains the process/filesystem boundary
already used by `std::path`; the path is observation and error context, not
semantic identity. Path-alias and symlink equivalence beyond existing detector
behavior are deferred.

### Discovery and format dispatch

One initial build performs exactly one production discovery for the configured
root. Discovered configurations are consumed in the detector's deterministic
ascending root-path order.

Dispatch is total over `WorkspaceFormat`:

- `Edt` invokes `FileSystemEdtSemanticGraphBuilder::build_graph_with_diagnostics`;
- `DesignerXml` invokes
  `FileSystemDesignerXmlSemanticGraphBuilder::build_graph` with
  `DesignerXmlBuildScope::Complete`;
- `Extension` and `Unknown` are rejected as unsupported if an injected or future
  detector yields them.

Runtime must never infer `DesignerXmlBuildScope::Partial`. Controlled tests or
tools may continue to request `Partial` directly from the adapter, outside the
production Workspace service.

Builds execute sequentially in discovered root-path order in the first slice.
This gives deterministic first-failure behavior and does not invent a
performance-driven concurrency policy.

### Canonical immutable snapshot

The published value is one source-neutral immutable `WorkspaceSnapshot`
containing zero or more `WorkspaceConfigurationSnapshot` records. It is not one
merged graph.

Each configuration record owns or immutably shares:

- the discovered root path and `WorkspaceFormat`;
- the canonical configuration `EntityId` and exact `EntityName`, extracted from
  the single `MetadataKind::Configuration` node in the built graph;
- one canonical `SemanticGraph` behind an immutable shared reference;
- ordered source-independent `SemanticDiagnostic` values;
- a canonical `SemanticReferenceRequestLedger` and total
  `SemanticReferenceStatistics`;
- a `SemanticGraphReport` representing the same graph, diagnostics, requests,
  and accepted legacy observations.

For EDT, those observations come from `EdtSemanticGraphBuildResult` without
discarding its public diagnostics, requests, statistics, or report. For
Designer XML, diagnostics and requests are empty, reference statistics are
empty, and the report is the graph report. This empty evidence means that the
Designer builder produced none; it does not claim EDT-equivalent semantic
breadth.

Every graph must pass the appropriate public validation before publication:
EDT uses build-result validation and Designer uses graph validation. A build
whose validation contains errors is a failed configuration build even when the
adapter returned a value.

Records are normalized into ascending canonical configuration-ID order after
all builds succeed. The snapshot provides read-only iteration and lookup by
configuration ID. It exposes no mutable graph or collection reference.
Configuration ID, not root path or insertion order, is snapshot identity.

Separate configuration graphs remain separate. Equal non-configuration node
IDs in different graphs do not overwrite or merge each other. Sprint 18 must
select a configuration before querying or accept a later explicit aggregate
contract.

### Identity, duplicates, and graph shape

Each successful adapter graph must contain exactly one node of
`NodeKind::Metadata(MetadataKind::Configuration)`. Zero or multiple
configuration nodes are a source-neutral Workspace build error.

Two discovered roots that produce the same configuration `EntityId` make the
entire initial build fail. Runtime reports both ordered root paths with the
duplicate identity and publishes no snapshot. Equal exact names with different
IDs are allowed. Duplicate discovered root paths remain detector-owned and are
already deduplicated by the production `BTreeMap`.

No graph node is replaced, merged, renamed, or re-identified by Runtime.
Configuration-root location is retained for observation and errors but does not
enter graph identity.

### Empty Workspace and publication atomicity

A valid readable configured root with zero discovered configurations is a
successful empty `WorkspaceSnapshot`. It represents confirmed absence under the
accepted discovery depth, not a missing root. Missing or invalid roots remain
errors.

The initial snapshot is all-or-nothing:

1. discovery and every supported configuration build complete into private
   local values;
2. validation, configuration-node cardinality, and duplicate-ID checks complete;
3. only then is one immutable snapshot published;
4. any discovery, dispatch, adapter, validation, identity, join, or panic
   failure publishes nothing and fails Workspace service startup.

No earlier successful configuration is published after a later failure. There
is no partial-success snapshot or per-configuration failure record in the first
slice.

### Snapshot observation and state ownership

The composition root constructs `WorkspaceService` and obtains a cloneable
transport-neutral `WorkspaceSnapshotObserver` before registering the service.
This follows the existing `HttpService` observer pattern without putting
adapter construction in `AppState` or introducing a global registry.

The observer exposes the current `Option<Arc<WorkspaceSnapshot>>` and change
subscription through an owned watch receiver. The service owns the only sender:

- initial state is `None`;
- one complete initial snapshot is published before startup acknowledgement;
- requested cancellation publishes `None` before the service task returns;
- dropping or failing startup closes the sender with no published snapshot;
- a later Runtime service startup failure triggers ADR-0037 rollback and clears
  the already published Workspace snapshot during reverse cleanup.

The snapshot object and every graph it contains are immutable. The watch value
may change only between absent and one complete initial snapshot in Sprint 17.
File-watcher replacement snapshots belong to Sprint 19.

Future services, including Sprint 18 query adapters, receive an observer clone
from the composition root. The observer is not an HTTP, CLI, MCP, or LSP API.

### Service startup and blocking-work ownership

`WorkspaceService` is a normal ADR-0037 `RuntimeService` registered under the
stable name `workspace`.

Its `start` implementation moves the configured root and the complete builder
into exactly one `tokio::task::spawn_blocking` operation. The Runtime-owned
start task awaits that blocking handle. The operation performs discovery,
sequential dispatch, validation, normalization, and complete snapshot
construction. No filesystem or parser work runs directly on an async executor
worker.

The blocking handle is never detached. Completion, returned errors, and panic or
join failure are observed before `start` returns. On success, `start` publishes
the snapshot and returns one service task that owns the sender and waits only for
receiver-side Runtime cancellation. On cancellation it clears the snapshot and
returns successfully.

The initial blocking build is deliberately non-interruptible in this first
slice. The current `App::run` does not poll the injected shutdown future until
all services acknowledge startup, and Rust blocking work cannot be safely
force-cancelled. A shutdown request arriving during the initial build is
observed after the build joins and startup completes; Runtime then follows its
normal `Running -> Stopping` path and clears the snapshot. This bounded policy
is explicit and testable; no detached or aborted blocking task is allowed.

### Lifecycle, registration order, and readiness

Production composition registers `http` before `workspace`.

- HTTP can expose liveness and not-ready responses while the Workspace initial
  build holds Runtime in `Initializing`.
- Workspace publishes one complete snapshot before acknowledging startup.
- `App` transitions to `Running` only after HTTP and Workspace have both
  acknowledged startup.
- ADR-0038 readiness therefore becomes true only after a complete Workspace
  snapshot exists, without a second readiness label or health schema change.
- `Stopping` makes readiness false through the existing lifecycle before reverse
  cleanup clears the Workspace snapshot.

If Workspace startup fails, Runtime enters cleanup, cancels and joins HTTP,
reaches `Stopped`, publishes no Workspace snapshot, and returns a named
`ServiceStartFailed` error. If a later service is added and fails startup,
reverse rollback clears the previously published Workspace snapshot.

Snapshot presence is a Workspace observation, not the health authority. An
observer may see a complete snapshot during the small interval after Workspace
publication and before `App` enters `Running`; clients must still use Runtime
readiness to decide whether the complete application can serve work.

### Errors and observability

Runtime adds a stable source-neutral Workspace build error type with these
semantic categories:

- discovery failed for the configured root;
- unsupported discovered format at a root;
- EDT or Designer semantic build failed at a root;
- graph validation failed at a root;
- configuration-node cardinality is not exactly one;
- duplicate configuration identity across ordered roots;
- blocking build task failed to join or panicked.

Exact Rust variant names may follow local conventions, but public accessors
must expose the stable category and applicable root, format, configuration ID,
and validation summary without parsing display text. Source errors remain in
the `Error::source` chain. Diagnostic text may include local root paths but must
not include source file contents, graph payload dumps, unbounded diagnostics,
or serialized graph state.

At the Runtime boundary, every initial-build error is wrapped by the existing
`RuntimeErrorKind::ServiceStartFailed` with service name `workspace`. No new
top-level Runtime error kind is required. Normal cancellation and snapshot
clearing are not failures.

The immutable snapshot, observer state, configuration identities/order,
diagnostic/request/report counts, lifecycle watch, and existing error accessors
are the accepted test and future observability surface. Logs are not acceptance
evidence.

### Determinism and public test contract

Focused and public tests must prove:

- missing and non-directory configured roots;
- successful empty discovery and empty snapshot;
- one EDT and one complete Designer XML production build;
- deterministic multiple-root discovery, build, and configuration-ID ordering;
- `Extension` and `Unknown` rejection through a controlled detector seam;
- conflicting markers, fatal adapter input, invalid graph/cardinality, and
  duplicate configuration identity;
- recoverable EDT diagnostics, requests, statistics, and report preservation;
- Designer empty diagnostic/request evidence without false equivalence claims;
- no publication after any failed build and clearing after rollback/shutdown;
- HTTP liveness/not-ready while Workspace startup is controlled, readiness only
  after complete publication, and unchanged exact health wire responses;
- non-interruptible but joined initial blocking work, named startup failure,
  reverse cleanup, terminal `Stopped`, and no surviving sender/task;
- equal observations from repeated fresh applications and independent resources.

Tests use repository-owned provenance-backed source inputs or a bounded Runtime
fixture derived from them with an explicit README/hash inventory. Ignored local
corpora, external services, symlinks, Unix-only behavior, real process signals,
and arbitrary sleeps are not acceptance evidence. Bounded channels, watches,
and hang timeouts provide deterministic coordination on macOS and Windows.

## First production slice

Sprint 17 implements only:

1. Workspace root configuration with the preserved constructor surface;
2. source-neutral immutable per-configuration snapshot/result types and
   deterministic lookup;
3. production filesystem discovery and total EDT/Designer dispatch;
4. sequential all-or-nothing initial builds on one owned blocking task;
5. one snapshot observer and one Runtime-owned `workspace` service;
6. HTTP-before-Workspace composition and lifecycle-derived readiness;
7. public production evidence and current-state documentation.

The service performs no rebuild after startup. Its acknowledged task only owns
snapshot lifetime and cancellation cleanup.

## Compatibility and migration

- Existing `RuntimeConfig::new`, `Default`, application name, environment, HTTP
  bind address, and `with_http_bind_address` behavior remain compatible.
- Existing `AppBuilder`, generic service registration, `AppState`, lifecycle,
  Runtime error kinds, HTTP paths/methods/status/schema, and shutdown behavior
  remain compatible.
- Existing Workspace detector, EDT builder, Designer builder, graph, diagnostic,
  request, report, validation, and Coverage APIs retain their meanings.
- Runtime adds only local path dependencies and bounded public Workspace types.
- The production binary begins discovering/building the configured default `.`
  root before becoming ready. Applications and tests that construct `App`
  manually remain free to omit Workspace registration; readiness continues to
  mean that all services registered in that application acknowledged startup.

## Coverage impact

Graph-domain, EDT, and Designer Coverage registries do not change status. Sprint
17 orchestrates already accepted producers and records Runtime integration
evidence; architecture text or Runtime publication does not expand supported
semantic kinds or source formats.

## Rejected alternatives

### Merge every configuration into one graph

Rejected. Equal node identities across independent configurations could replace
facts, graph mutation would need new merge/conflict semantics, and Sprint 18 can
query explicit per-configuration graphs without creating a second authority.

### Publish adapter-local result enums

Rejected. It would make Runtime consumers depend on EDT/Designer orchestration
types and obstruct a source-neutral query boundary. The accepted snapshot keeps
source-independent graph, diagnostic, request, statistics, and report evidence.

### Discard EDT diagnostics and request evidence

Rejected. Those are accepted recoverable semantic observations and are required
for truthful reports and future consumers.

### Publish configurations as each build completes

Rejected. A later fatal input would leave an unsupported partial initial state
and make readiness and repeated runs dependent on failure position.

### Treat an empty discovered Workspace as failure

Rejected. Discovery already distinguishes a valid empty root from a missing or
invalid root; confirmed absence is a deterministic snapshot.

### Run synchronous builders directly in the async start task

Rejected. Filesystem parsing is blocking work and could stall executor workers.

### Detach, abort, or pretend to cancel blocking initial builds

Rejected. `spawn_blocking` work is not safely force-cancellable. The bounded
first slice joins it and documents shutdown timing.

### Acknowledge startup before the initial snapshot exists

Rejected. Runtime could become ready without semantic state or would require a
second mutable readiness authority.

### Put adapter construction or mutable Workspace state in `AppState`

Rejected. Adapter construction belongs to the composition root, and a dedicated
cloneable observer follows the existing bounded service pattern. `AppState`
remains immutable and transport-neutral.

### Register Workspace before HTTP

Rejected for the first slice. It would prevent the accepted liveness/not-ready
probe from observing a controlled initial build and provides no ownership
benefit.

### Infer Designer partial scope from a selective or reduced fixture

Rejected. ADR-0036 makes scope explicit and ordinary discovered roots complete.

### Add watcher, cache, query endpoint, or CLI behavior now

Rejected. Those capabilities require their own accepted lifecycle,
compatibility, invalidation, persistence, and transport contracts in Sprints
18-21.

## Deferred scope

- graph and semantic query services and HTTP endpoints: Sprint 18;
- file watching, change coalescing, rebuild replacement, and invalidation:
  Sprint 19;
- persistent snapshots, cache format, validation, and deterministic
  invalidation: Sprint 20;
- supported CLI Workspace and graph-query client behavior: Sprint 21;
- aggregate cross-configuration graph semantics, cross-configuration
  references, extension projects, `Unknown` formats, explicit partial Runtime
  workspaces, configurable discovery depth, path alias/canonicalization policy,
  live add/remove, concurrent builds, retries, restart, progress streaming,
  forced termination, and a build timeout;
- Workspace or graph HTTP routes, request/response error mapping, MCP, LSP, IDE,
  AI, authentication, authorization, TLS, CORS, metrics, tracing export,
  OpenAPI, packaging, benchmarks, and performance claims.

## Implementation prerequisites

1. Implement the source-neutral immutable snapshot records, observer, stable
   error categories, configuration-node extraction, total format dispatch,
   report/request preservation, validation, ordering, duplicate rejection, and
   all-or-nothing builder with focused tests.
2. Add only the required local Runtime path dependencies and no external
   production dependency.
3. Implement the `workspace` Runtime service with one owned `spawn_blocking`
   handle, publication before acknowledgement, cancellation clearing, and no
   detached work.
4. Add Workspace root configuration and register HTTP before Workspace in the
   production composition root.
5. Prove both real production builder paths, failures, readiness, rollback,
   shutdown, cleanup, and fresh repetition through non-zero public tests over
   provenance-backed repository inputs.
6. Synchronize current-state documentation only after implementation evidence
   exists; do not change Coverage support status.

Every production Rust, public API, or Cargo change runs focused affected tests
and the complete workspace validation gate.

## Consequences

- Runtime gains one deterministic semantic-build authority without absorbing
  source parsing or graph meaning.
- Ready means every registered production service, including the initial
  Workspace build, acknowledged startup.
- A Workspace may be empty or contain multiple isolated configuration graphs;
  consumers select by stable configuration ID.
- Initial failures are atomic and preserve their adapter source chains beneath
  the stable named Runtime failure.
- Initial blocking work can delay shutdown but cannot outlive Runtime startup or
  become detached.
- Sprint 18 receives an immutable transport-neutral observation seam while query
  semantics and endpoints remain explicitly undecided.
