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
     propagation. Its Runtime-owned Workspace service performs one configured
     production discovery/build, publishes separate immutable
     per-configuration semantic snapshots, and clears them during owned
     shutdown. Its Axum service still exposes only HTTP liveness and
     lifecycle-derived readiness probes; graph-query transport remains
     unimplemented.
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

- Graph-query Runtime APIs, file watching, persistence, and the supported CLI
  remain assigned to Sprints 18–21.
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
17 implementation and public production evidence are present, but Sprint 17
remains incomplete until its independent integration review records a decision.
