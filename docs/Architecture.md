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
   - `oneagent-workspace-fs` discovers supported workspaces through the
     filesystem boundary.
4. **Applications and protocol foundation**
   - `oneagent-runtime` exposes the long-running composition root as a reusable
     library. It owns ordered service startup, rollback, task handles,
     per-service cancellation, reverse shutdown, lifecycle, and terminal error
     propagation. It does not yet expose an HTTP or workspace/graph service.
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

- HTTP, workspace/graph Runtime services, file watching, persistence, and the
  supported CLI arrive in Sprints 16–21.
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

The public Runtime lifecycle and deterministic in-memory probe boundary are not
an HTTP health/readiness contract. HTTP routes, schemas, status mapping, and
client compatibility remain Sprint 16 work; workspace, graph, watcher,
persistence, and CLI services remain Sprints 17-21.

## Accepted HTTP and health boundary

[ADR-0038](adr/0038-http-api-health.md) accepts the planned Sprint 16 boundary:
one Runtime-owned Axum service will bind during service startup, expose only
`GET /health/live` and `GET /health/ready`, derive readiness exclusively from
the canonical Runtime lifecycle, and complete through ADR-0037 cancellation and
task ownership. The accepted wire schema, status matrix, loopback-default bind
configuration, and public client/server evidence requirements are fixed before
implementation. This section records an accepted plan, not current HTTP
support; the current implementation remains the Sprint 15 service container.

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
completed; Sprint 16 owns the next HTTP and public health/readiness boundary.
