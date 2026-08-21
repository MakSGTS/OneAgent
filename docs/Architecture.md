# Architecture

OneAgent uses a modular Rust workspace centered on a source-independent semantic
graph. The architecture distinguishes the current implementation from planned
product adapters so that roadmap intent is not mistaken for available behavior.

## Current implementation

1. **Shared and domain crates**
   - `oneagent-common` owns shared typed primitives.
   - `oneagent-metadata` owns the typed 1C metadata model.
   - `oneagent-workspace` owns workspace and project abstractions.
   - `oneagent-bsl` owns BSL lexical and syntax analysis.
2. **Semantic core**
   - `oneagent-graph` owns canonical semantic nodes, edges, provenance,
     validation, query, diff, impact, coverage, and resolution APIs.
   - `oneagent-analysis` contributes source-independent declaration and call
     analysis over the BSL and graph contracts.
3. **Source adapters**
   - `oneagent-edt` reads supported EDT artifacts and contributes facts to the
     canonical semantic graph.
   - `oneagent-designer-xml` reads accepted hierarchical Designer XML artifacts
     and contributes the same source-independent graph kinds without replacing
     EDT semantics.
   - `oneagent-workspace-fs` discovers supported workspaces through the
     filesystem boundary.
4. **Applications and protocol foundation**
   - `oneagent-runtime` exposes the long-running composition root as a reusable
     library. It owns ordered service startup, rollback, task handles,
     per-service cancellation, reverse shutdown, lifecycle, and terminal error
     propagation. Its Runtime-owned Workspace service performs configured
     production discovery/build, observes subsequent file changes, serializes
     complete rebuilds, publishes separate immutable per-configuration semantic
     snapshots, retains the last valid publication across failed rebuilds, and
     clears the snapshot during owned shutdown. It also owns one fixed
     Workspace-local complete-snapshot cache, exact validity checks, safe
     replacement, and typed cache observation without making persisted bytes a
     semantic authority. Public status observers report rebuild phase, attempts,
     publications, failures, and the latest cache load/write outcomes. Its sole
     Axum service exposes HTTP liveness and
     lifecycle-derived readiness probes plus the versioned read-only Graph
     Query route set. A transport-neutral observer-backed query component owns
     exact configuration, node, direct-relation, and bounded-traversal
     operations without becoming a background service.
   - `oneagent-cli` is a package placeholder and is not yet a supported client.
   - `oneagent-protocol` is a package foundation and does not yet expose HTTP,
     MCP, or LSP contracts.

`SemanticGraph` is the canonical semantic authority. Adapters may observe source
formats and contribute provenance-backed facts, but source-specific identities
and parser state must not become competing graph truth. Derived facilities such
as query, resolution, reports, diffs, impact analysis, and the Sprint 4 Semantic
Index remain read-only views over graph snapshots.

## Planned boundaries

The roadmap assigns future boundaries explicitly:

- Graph-query Runtime APIs are implemented in Sprint 18, and Sprint 19 File
  Watching and Sprint 20 Persistent Cache are completed with `pass` integration
  reviews. Sprint 21 CLI Client is the unique `next` target.
- MCP, VS Code, LSP, and EDT product integration arrive in Sprints 28–35.
- Git change ingestion arrives in Sprint 38 as an input adapter, not a semantic
  authority.

Detailed accepted decisions live in `docs/adr`. The dependency-ordered delivery
sequence and status live only in `docs/Roadmap.md`.

## Accepted Runtime service-container boundary

[ADR-0037](adr/0037-runtime-service-container.md) governs the implemented Sprint
15 boundary. `oneagent-runtime` remains the composition root and exposes a
transport-independent library boundary.
`AppBuilder` owns ordered, uniquely named service registration; `App` owns the
built container and lifecycle; the running container owns every service task
handle and per-service cancellation source until all handles terminate.

Services start sequentially in registration order and acknowledge startup by
returning their owned task. Partial startup rolls acknowledged services back in
reverse order. A requested shutdown, unexpected exit, service error, or task
join failure triggers reverse cooperative cancellation and complete joining;
the application reaches `Stopped` before returning its terminal result. The
first slice has no detached tasks, global registry, new dependency, or bounded
shutdown timeout.

The public Runtime lifecycle and deterministic in-memory service probes remain
the ownership foundation for the HTTP adapter; workspace, graph, watcher,
persistence, and CLI services remain Sprints 17-21.

## Accepted HTTP and health boundary

[ADR-0038](adr/0038-http-api-health.md) governs the implemented Sprint 16 HTTP
slice. One Runtime-owned Axum service binds during service startup, exposes only
`GET /health/live` and `GET /health/ready`, derives readiness exclusively from
the canonical Runtime lifecycle, and completes through ADR-0037 cancellation
and task ownership. The default address is `127.0.0.1:3000`; callers can supply
a typed override, including port zero, and observe the actual bound address
without controlling the listener.

Liveness returns `200` with `{"status":"alive"}` while the handler is
reachable. Readiness returns `200` with `{"status":"ready"}` only during
`Running`, and `503` with `{"status":"not_ready"}` during observable
`Initializing` and `Stopping` states. Only GET is supported; registered wrong
methods return `405` with `Allow: GET`, and unknown exact paths return `404`.
The listener binds before startup acknowledgement, bind errors remain named
service-start failures, and graceful shutdown releases the listener only after
the Runtime-owned HTTP task completes.

## Accepted Graph Query API boundary

[ADR-0040](adr/0040-graph-query-api.md) governs the implemented Sprint 18
boundary. Production composition constructs one Workspace observer, injects it
into one transport-neutral `GraphQueryService`, and gives that component to the
existing `HttpService`; `http` still starts before `workspace`, and
`HttpService::new()` remains a health-only compatible construction path.

The query-enabled listener registers exactly four GET routes:

- `/api/v1/configurations` lists separate published configurations;
- `/api/v1/graph/node` selects one exact node in one exact configuration;
- `/api/v1/graph/relations` returns direct incoming or outgoing edges with an
  optional one-kind filter;
- `/api/v1/graph/traverse` performs deterministic breadth-first traversal with
  mandatory depth and result bounds.

Each request observes one immutable Workspace snapshot. The HTTP adapter first
validates the closed query syntax and values, then requires canonical Runtime
readiness, and finally delegates to the transport-neutral component. Results
use owned payload-free projections, limits default to 50 and cannot exceed 100,
traversal depth cannot exceed 4, and truncation is explicit. Exact stable JSON
errors distinguish lifecycle, snapshot, selection, identifier, syntax,
vocabulary, boolean, and bound failures without exposing internal diagnostics.
Health routes remain the sole liveness/readiness authority and retain their
Sprint 16 wire contract.

### Sprint 18 public evidence matrix

The public `apps/runtime/tests/graph_query_api.rs` target uses raw Tokio
loopback HTTP and the tracked Sprint 17 provenance fixture through production
filesystem discovery and both production builders.

| Contract | Public evidence |
| --- | --- |
| Separate production graphs | Configuration listing preserves exact Designer XML and EDT identities, formats, counts, and canonical order; node queries select facts from each graph without merging. |
| Four accepted operations | Exact node, outgoing/incoming direct relation, filtered relation, empty relation, bounded traversal, included start, and empty depth-zero results are asserted through public HTTP. |
| Bounds and closed errors | Defaults, truncation, unknown configuration/node, invalid identifier/query/encoding, unsupported direction/edge kind, limit/depth bounds, invalid boolean, and unavailable snapshot map to exact status/code/message rows. |
| Route compatibility | Every registered route is GET-only; HEAD/POST return `405` with `Allow: GET`; unknown and trailing-slash paths retain empty `404`; JSON is returned independently of `Accept`. |
| Lifecycle authority | Published snapshots remain query-inaccessible during gated `Initializing` and `Stopping`, become available only in `Running`, and absent snapshots are distinct from lifecycle readiness. |
| Ownership and determinism | Two fresh production runs return equal wire observations, clear snapshot/address watches, join all owned work, release the listener, and permit immediate rebind. |

## Accepted Workspace service boundary

[ADR-0039](adr/0039-workspace-service.md) governs the implemented Sprint 17
initial-build slice. `RuntimeConfig` owns one Workspace root, and production
composition starts HTTP before one uniquely named `workspace` service. That
service moves the configured path and complete snapshot builder into exactly one
owned blocking task, runs filesystem discovery once, dispatches EDT and
Designer XML builds sequentially, validates every graph, rejects unsupported or
colliding configurations, and publishes only one complete immutable snapshot.

The snapshot keeps configurations as separate graphs ordered by canonical
Configuration identity. Each record preserves its detected root and format,
exact Configuration name and ID, canonical graph, diagnostics, reference
ledger/statistics, and report. A valid empty root publishes an empty snapshot;
any discovery, adapter, validation, cardinality, duplicate-identity, or blocking
task failure publishes nothing and becomes a named Workspace startup failure.
Cancellation clears the snapshot before the owned service task returns. Runtime
readiness remains derived only from lifecycle, so snapshot presence is not an
independently mutable health label.

### Sprint 17 public evidence matrix

The public `apps/runtime/tests/workspace_service.rs` target uses only production
discovery/build paths and public Runtime observation. Its bounded tracked EDT
and Designer inputs have an explicit provenance and SHA-256 inventory.

| Contract | Public evidence |
| --- | --- |
| Both production formats | One mixed root builds exact EDT and complete Designer graphs, preserves their distinct evidence, and orders them by Configuration ID. |
| Determinism and fresh ownership | Repeated fresh applications publish equal observations and close every snapshot sender after `App::run`. |
| Empty and invalid roots | Empty readable roots publish an empty snapshot; missing and non-directory roots return named startup failures without publication. |
| Discovery and atomic failure | Conflicting markers, duplicate Configuration identity, and a later fatal adapter input reject the entire snapshot. |
| Readiness authority | With a deterministic later startup/cleanup gate, real health requests remain not-ready in `Initializing` and `Stopping`, become ready only in `Running`, and retain the Sprint 16 wire vocabulary. |
| Shutdown cleanup | Reverse cancellation keeps the complete snapshot available until the Workspace service is reached, then clears it and closes observation before terminal `Stopped`. |

## Accepted File Watching boundary

[ADR-0041](adr/0041-file-watching.md) governs the implemented Sprint 19
boundary. After the startup build, one Runtime-owned source recursively scans
the configured Workspace root every 250 milliseconds using normalized relative
paths, entry kinds, and complete regular-file bytes. Descendants of `.git`,
`.idea`, `.vscode`, `target`, and `node_modules` are excluded; source extensions
are not filtered. The source emits only the latest opaque revision through a
private single-value channel.

The Workspace service remains the sole rebuild owner. It closes the startup
scan/build race with before/build/after scans, serializes complete rebuilds,
coalesces changes that arrive during a build, and atomically replaces the
published `Arc` only after a valid all-or-nothing build. A post-start observation
or semantic-build failure retains the last valid snapshot and becomes public
update status instead of terminating Runtime; a later change can recover.
Health/readiness and Graph Query wire contracts remain unchanged. Shutdown
cancels and joins the change source and any in-flight build, prevents a
post-cancellation publication, clears the snapshot, and publishes terminal
`Stopped` update status.

### Sprint 19 production and deterministic evidence matrix

The public `apps/runtime/tests/file_watching.rs` target imports only the
`oneagent_runtime` library surface, copies the tracked Sprint 17 fixture into
fresh temporary roots, uses production polling/discovery/build paths, and
queries the existing Graph Query API over raw Tokio loopback HTTP. Event watches
are the asserted synchronization mechanism; five-second timeouts are hang
guards rather than timing evidence. Negative ignored-change and exact active-
build concurrency assertions use the focused controlled-tick/gated-builder
tests because ADR-0041 explicitly forbids treating the production polling period
as a test oracle; those tests retain the real scanner or complete builder as the
authority.

| Contract | Evidence |
| --- | --- |
| Both production formats | Exact EDT and Designer XML name changes trigger complete production rebuilds and become visible in separate snapshots and Graph Query responses. |
| Atomic immutable replacement | A held pre-change `Arc` remains unchanged while later observations receive one valid replacement; Graph Query requests observe only complete published snapshots. |
| Add/remove/rename-equivalent changes | Moving a Designer root outside the watched Workspace and back under a different root name proves removal and addition detection without a native rename event contract. |
| Relevance and ignored state | Focused real-filesystem scans prove complete bytes, paths, entry kinds, and all five ignored-directory exclusions; a controlled production-service scan proves an ignored mutation leaves public update status unchanged. |
| Burst and in-flight coalescing | Public status proves a mutation accepted after `Rebuilding` causes exactly one follow-up publication and a multi-entry project-tree addition causes one attempt; the focused gated builder proves one active build and one bounded latest-state follow-up. |
| Failure retention and recovery | Corrupt EDT input reports a semantic-build failure while the last valid snapshot and query result remain available; a later repair publishes a recovered snapshot. |
| Observation failure and readiness | Removing the watched root reports `Observation`, retains the queryable snapshot and exact ready health response, and publishes one recovered rebuild when the root returns. |
| Status and ownership | Public update status proves attempts, publications, phases, failure classification, recovery, terminal `Stopped`, closed snapshot/update receivers, listener release, and equal fresh-run observations. |

## Accepted Persistent Cache boundary

[ADR-0042](adr/0042-persistent-cache.md) governs the implemented Sprint 20
baseline without changing `SemanticGraph`, source adapters, Runtime lifecycle,
health, or Graph Query authority. `WorkspaceService` owns source observation,
cache orchestration, complete clean builds, immutable publication, cancellation,
and cleanup. The cache is a private versioned representation of one complete
validated `WorkspaceSnapshot`; decoded content is reconstructed through checked
domain APIs and passes complete build validation before it can be published.

The fixed entry is `.oneagent/cache/workspace-v1.json` under the configured
Workspace root, with one bounded temporary replacement file. Exact complete
source state plus explicit schema and semantic-build versions determine validity.
Startup performs scan/load/scan before accepting a hit, otherwise runs one clean
build, closes the build race with a final scan, and writes only stable state.
File Watching rebuilds use the same pre/build/post stability rule and finish
cache work before atomically publishing a valid replacement. Cache-owned paths
are excluded from source observation, so replacement cannot create a watcher
feedback loop.

Missing, changed, incompatible, corrupt, or unavailable entries clean-build
instead of becoming semantic authority. Failed writes and unstable-source skips
do not reject a valid snapshot. Public cloneable cache observation exposes only
the closed latest load and write outcomes; it does not add HTTP, CLI, readiness,
or protocol state. Shutdown joins current blocking cache/build work, closes cache
observation with the other Workspace observers, and preserves the complete cache
entry for a fresh process.

### Sprint 20 public evidence matrix

The public `apps/runtime/tests/persistent_cache.rs` target imports only the
`oneagent_runtime` library surface, copies the tracked Sprint 17 mixed EDT and
Designer XML provenance fixture into disposable roots, and exercises production
source scans, cache storage, both clean builders, Workspace/File Watching,
Graph Query, health, cancellation, and shutdown. Watches are synchronization;
five-second timeouts are hang guards, not polling-duration evidence.

| Contract | Public evidence |
| --- | --- |
| Cold and warm completeness | A cold missing entry clean-builds both production formats and writes once; a fresh exact hit performs no write and restores equal graphs, payloads, provenance, diagnostics, reference evidence, statistics, reports, transport-neutral queries, and HTTP results. |
| Identity and compatibility | Complete source changes produce `SourceChanged`; older and newer schema and semantic-build versions produce `Incompatible`; every case clean-builds and replaces current state. |
| Corruption containment | Malformed, truncated, partial, checksum-invalid, and checksum-valid semantically invalid entries produce `Corrupt`, publish no persisted partial state, and recover through equal complete clean builds. |
| Storage failure and repair | A publicly constructible wrong-kind cache owner produces `Unavailable`/`Failed` while the valid Workspace remains ready and queryable; removing the obstacle permits missing/write recovery followed by a warm hit. |
| Watched replacement and reuse | Production EDT and Designer changes publish complete immutable replacements, preserve held older snapshots, replace cache bytes, ignore cache-owned probe state in the source identity, and restore the latest replacement on a fresh hit. |
| Lifecycle and cleanup | Cache work completes before `Running` or replacement publication; health and Graph Query contracts remain exact; shutdown clears snapshots, publishes terminal update state, closes snapshot/update/cache watches, releases listeners, leaves no temporary file, and preserves only reusable complete cache state. |

### Sprint 16 public evidence matrix

The public `apps/runtime/tests/http_health.rs` target imports only the
`oneagent_runtime` library surface and uses raw Tokio loopback TCP. Lifecycle
watches and one-shot channels define asserted events; one-second timeouts are
hang guards rather than timing evidence.

| Contract | Public evidence |
| --- | --- |
| Lifecycle-derived readiness | Real requests return not-ready during gated `Initializing`, ready during `Running`, and not-ready during gated reverse cleanup in `Stopping`. |
| Stable probe wire format | Liveness and readiness assert exact status, JSON media type, and closed single-field bodies. |
| Exact negative matrix | HEAD and POST on both routes return `405`, `Allow: GET`, and empty bodies; unknown and trailing-slash paths return `404` with empty bodies. |
| Startup failure | An occupied loopback address becomes named `ServiceStartFailed` for `http`, with no published address and terminal `Stopped`. |
| Graceful shutdown and ownership | Requested shutdown retains the HTTP service until earlier reverse cleanup completes, then joins it, clears address observation, and permits rebind. |
| Fresh repetition | Two separately built port-zero apps return equal wire responses and independently release every listener. |

### Sprint 15 public evidence matrix

The public `apps/runtime/tests/service_container.rs` target imports only the
`oneagent_runtime` library surface. Its deterministic in-memory probes use
channels as acknowledgements and timeouts only as hang guards.

| Contract | Public evidence |
| --- | --- |
| Genuinely long-running execution | The App remains pending after ordered startup until injected shutdown is released. |
| Requested shutdown | Services observe receiver-only cancellation and terminate in reverse registration order before `Stopped`. |
| Partial startup failure | A later named start error rolls the earlier acknowledged task back and closes every probe sender. |
| Running-service failure | The named error reaches the App caller after reverse sibling cleanup. |
| Unexpected exit and join panic | Early `Ok` and task panic retain distinct `RuntimeErrorKind` classifications. |
| Shutdown-source error | The source failure remains primary while the worker is cancelled and joined. |
| Fresh repetition and no detached work | Two separately built apps produce equal start/stop behavior; event-channel closure proves no probe task survives `App::run`. |

The [Sprint 15 integration review](reviews/sprint-15-runtime-service-container.md)
records `pass` after the focused and complete workspace gates. Sprint 15 is
completed. The [Sprint 16 integration review](reviews/sprint-16-http-api-health.md)
records `pass` for the owned HTTP and public health/readiness boundary; Sprint
17 implementation and public production evidence are completed with a `pass`
decision in the
[Sprint 17 integration review](reviews/sprint-17-workspace-service.md). Sprint
18 Graph Query API is completed with a `pass` decision in the
[Sprint 18 integration review](reviews/sprint-18-graph-query-api.md). Sprint 19
File Watching is completed with a `pass` decision in the
[Sprint 19 integration review](reviews/sprint-19-file-watching.md). Sprint 20
Persistent Cache is completed with a `pass` decision in the
[Sprint 20 integration review](reviews/sprint-20-persistent-cache.md). Sprint 21
CLI Client is the unique `next` target.
