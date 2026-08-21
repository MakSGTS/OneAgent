# Workspace Service Investigation

## Status and scope

This document records the read-only Sprint 17 investigation at committed
planning baseline `d8571c46df227e4b7b1f90ee1db0a753f8ac9416` on 2026-08-21.
It identifies live repository evidence and the decisions required from
ADR-0039. It does not select architecture or describe implemented Runtime
Workspace behavior.

Sprint 17 is the unique `next` Roadmap target after the Sprint 16 integration
review issued `pass`. Graph-query APIs, file watching, persistent cache,
supported CLI behavior, later transports, security, and performance remain
outside this investigation.

## Accepted constraints

- [ADR-0002](../adr/0002-runtime-composition-root.md) makes
  `oneagent-runtime` the dependency construction and lifecycle owner;
  `AppState` contains immutable shared state and `main.rs` contains no domain or
  infrastructure logic.
- [ADR-0004](../adr/0004-filesystem-workspace-discovery.md) puts the
  `WorkspaceDetector` port in `oneagent-workspace` and filesystem discovery in
  `oneagent-workspace-fs`.
- [ADR-0036](../adr/0036-designer-xml-adapter.md) keeps EDT and Designer XML
  adapters independent, makes an ordinary discovered Designer root a
  `DesignerXmlBuildScope::Complete` build, and preserves graph identity and
  provenance without converting one source layout to the other.
- [ADR-0037](../adr/0037-runtime-service-container.md) requires ordered service
  startup, named startup/runtime failures, Runtime-owned tasks, receiver-only
  cancellation, reverse cleanup, complete joins, and a fresh `App` for every
  run.
- [ADR-0038](../adr/0038-http-api-health.md) derives readiness only from the
  canonical Runtime lifecycle. `Initializing` is not ready, `Running` is ready,
  and `Stopping` or `Stopped` is not ready. It defers Workspace semantic-build
  readiness to Sprint 17 without authorizing a second mutable status authority.
- [Semantic Model 2.0](semantic-model-2.md) assigns orchestration, graph
  construction pipeline, snapshots, and API exposure to Runtime while graph
  facts and validation remain owned by `oneagent-graph` and source parsing
  remains adapter-owned.

## Confirmed repository evidence

### Workspace domain and discovery

`crates/workspace/src/lib.rs` currently defines:

- `WorkspaceFormat::{Edt, DesignerXml, Extension, Unknown}`;
- `DiscoveredConfiguration`, containing only `root_path: PathBuf` and
  `format: WorkspaceFormat`;
- synchronous `WorkspaceDetector::discover(&Path) ->
  Result<Vec<DiscoveredConfiguration>, Box<dyn Error + Send + Sync>>`;
- `Configuration`, whose stable identity and exact name are loaded by an
  adapter, not by the detector;
- `Workspace`, a mutable `Vec<Configuration>` container with insertion-order
  iteration and lookup by `EntityId`.

No current Runtime consumer constructs `Workspace`. The type has no graph,
diagnostics, lifecycle, atomic publication, or multi-configuration collision
contract. It therefore cannot be treated as the Sprint 17 snapshot without an
explicit ADR decision.

`FileSystemWorkspaceDetector` in `adapters/filesystem/src/lib.rs` is the only
production `WorkspaceDetector` implementation. It confirms:

- a default recursion limit of six and an explicit `new(max_depth)` override;
- EDT identity by the simultaneous presence of regular `.project` and
  `src/Configuration/Configuration.mdo` files;
- Designer identity through `is_designer_xml_project`, including marker,
  version, symlink, and compatibility validation;
- typed failures for a missing/non-directory root, unreadable directory or
  entry, unreadable file type, invalid Designer candidate, and conflicting EDT
  plus Designer markers;
- project roots as traversal boundaries and `.git`, `.idea`, `.vscode`,
  `target`, and `node_modules` as ignored directories;
- deduplication and deterministic ascending `PathBuf` result order through a
  `BTreeMap<PathBuf, WorkspaceFormat>`.

The production detector emits only `Edt` and `DesignerXml`; it cannot currently
emit `Extension` or `Unknown`. An empty readable directory succeeds with an
empty result. An incomplete EDT marker set is ignored rather than reported as a
project. The detector does not load configuration identity, so UUID/name
duplicates across different roots cannot be detected during discovery.

### Production semantic builders

The EDT production entry point is
`FileSystemEdtSemanticGraphBuilder` through the public
`EdtSemanticGraphBuilder` trait in `adapters/edt/src/lib.rs`:

```rust
fn build_graph_with_diagnostics(
    &self,
    project_root: &Path,
) -> Result<EdtSemanticGraphBuildResult, EdtGraphError>;
```

`EdtSemanticGraphBuildResult` owns one `SemanticGraph`, ordered recoverable
`SemanticDiagnostic` values, a canonical reference-request ledger, reference
statistics, and public `report`, `diff`, `validate`, and Coverage projections.
`build_graph` is a compatibility method that consumes the result and discards
recoverable diagnostics. Fatal load, filesystem, parser, identity, payload,
graph, and semantic assembly failures return no result.

The Designer production entry point is
`FileSystemDesignerXmlSemanticGraphBuilder` through the public
`DesignerXmlSemanticGraphBuilder` trait in
`adapters/designer-xml/src/semantic_graph.rs`:

```rust
fn build_graph(
    &self,
    project_root: &Path,
    scope: DesignerXmlBuildScope,
) -> Result<SemanticGraph, DesignerXmlGraphError>;
```

An ordinary discovered root must use `DesignerXmlBuildScope::Complete` under
ADR-0036. `Partial` is explicit caller input for controlled reduced sources and
must not be inferred by Runtime. Fatal configuration, metadata, module,
provenance, payload, graph, BSL, duplicate-node, and duplicate-ownership errors
return no graph. The result has no adapter-level recoverable diagnostic ledger;
generic `SemanticGraph::{report, diff, validate}` and Designer-specific
Coverage remain available.

Both builders are synchronous and perform `std::fs` reads and parsing in the
calling thread. Both return canonical `SemanticGraph` values with deterministic
`BTreeMap` node and `BTreeSet` edge iteration. `SemanticGraph` is `Clone` but is
not internally synchronized or mutation-protected. Read-only publication can
be achieved by ownership and API boundaries, but no such Workspace snapshot
exists today.

The builders have deliberately different accepted semantic breadth. A Runtime
or Workspace layer must not normalize away EDT diagnostics, claim whole-graph
cross-format equality, or add missing Designer semantics.

### Runtime construction, state, lifecycle, and failures

`oneagent-runtime` currently has no dependency on `oneagent-workspace`,
`oneagent-workspace-fs`, `oneagent-edt`, `oneagent-designer-xml`, or
`oneagent-graph`. All five are local workspace packages already present in
`Cargo.lock`; adding the exact required path dependencies would introduce no
new external production package and creates no current dependency cycle.
`tempfile` is already locked and used by the adapter test targets, but Runtime
has no current dev-dependencies.

`RuntimeConfig` contains application name, environment, and one typed HTTP bind
address. It has no Workspace root, discovery depth, build scope, or startup
policy. `DefaultConfigurationProvider` returns `RuntimeConfig::default()`.
`main.rs` constructs the default configuration and registers only `http`.

`AppBuilder::build` constructs `Arc<AppState>` before it builds the service
container. `AppState` contains only `RuntimeConfig` and `RuntimeHealth`; it has
no extension registry or Workspace observer. `ServiceContext` gives every
service a clone of that immutable state plus receiver-only cancellation. A
service value may carry a separately constructed observer/sender pair, as
`HttpService` carries its bound-address watch, but no accepted contract chooses
between a service handle and `AppState` for Workspace snapshots.

`ServiceContainer` starts services sequentially in registration order. Each
`RuntimeService::start` runs in a Runtime-owned Tokio task and must return a
`ServiceTask` before the service is acknowledged. `App` stays
`LifecycleState::Initializing` until all registered services acknowledge
startup, then transitions to `Running`. Consequently:

- doing the initial Workspace build inside `start` would keep existing
  lifecycle-derived readiness false until that build acknowledged success;
- registering HTTP before Workspace would make the probe listener observable
  during the Workspace build, while registering Workspace first would delay
  listener startup;
- returning from `start` before the build completes would allow Runtime to
  become ready unless ADR-0039 adds another accepted readiness input;
- synchronous parsing directly inside the async start future would block a
  Tokio worker thread;
- `spawn_blocking` can isolate blocking work, but cancellation, join ownership,
  panic mapping, and publication atomicity require an explicit decision;
- the injected process shutdown future is not polled by `App::run` until all
  service startups complete, so shutdown during a long initial build has no
  current accepted behavior.

A startup error is wrapped as `RuntimeErrorKind::ServiceStartFailed` with the
registered service name and the original boxed error retained as `source()`.
Acknowledged service failures, unexpected success, join failure, shutdown-source
failure, cleanup failures, and reverse cancellation/join already have stable
classifications. The current taxonomy can preserve an adapter/discovery source
chain, but ADR-0039 must decide whether a Workspace-specific public domain error
is also required.

### Current consumers and compatibility surface

Repository search finds production builder consumers only in adapter-local
tests and conformance code. Runtime, HTTP, CLI, analysis, and graph-query code do
not currently depend on the Workspace detector or either builder. Therefore
Sprint 17 has no existing Workspace service API to preserve, but it must preserve:

- the public builder and detector APIs and their error/source contracts;
- EDT diagnostics, reference requests, reports, validation, and Coverage;
- Designer complete/partial caller semantics and exact provenance;
- graph identity, ordering, reports, diff, validation, and query behavior;
- Runtime service registration, lifecycle, health wire contract, error
  precedence, reverse cleanup, and fresh-run independence.

Graph Query API consumers are deferred to Sprint 18. The Sprint 17 snapshot
must provide a source-neutral transport-independent seam without preselecting
query endpoints or serialization.

## Data and testability evidence

### Repository-owned positive sources

`adapters/designer-xml/tests/fixtures/sprint14_conformance/` is the smallest
tracked paired EDT/Designer source slice with an explicit provenance and SHA-256
inventory. Its two child roots represent the same configuration and Common
Module identities. It proves cross-adapter canonical projection equality only
for the documented first slice. The Designer half is an official selective
export and existing conformance invokes it with explicit `Partial`; it is not
by itself evidence that Runtime may infer partial scope for a discovered root.

The Designer unit-test builders in
`adapters/designer-xml/src/semantic_graph.rs` construct tracked source-backed
accepted inputs and prove a public `Complete` build, exact graph counts,
fatal malformed input, deterministic query/report/diff/validation/provenance,
and source-path-independent identity. A bounded Runtime-owned complete fixture
may be derived from these exact accepted artifacts when public cross-package
evidence cannot stably reuse their private test helper.

Tracked EDT projects under `adapters/edt/tests/fixtures/` exercise the public
full builder, recoverable diagnostics, fatal malformed inputs, validation,
Coverage, diff, and repeated builds. The Sprint 14 paired EDT root is sufficient
for a small positive graph; richer fixture selection must follow the exact
assertion required and retain its README provenance.

Ignored `OneAgent_EDTproject/`, `OneAgent_DesignerXML/`, and
`Retail_edt_project/` corpora are investigation evidence only. They cannot be a
Runtime or CI prerequisite.

### Negative and lifecycle oracles

- `oneagent-workspace-fs` unit tests generate missing/incomplete/conflicting and
  depth-bound trees without platform-specific paths.
- Designer unit and conformance tests mutate temporary copies to prove malformed
  metadata/BSL returns no graph and partial absence creates no placeholder.
- EDT tests cover fatal load/parser failures and recoverable semantic
  diagnostics separately.
- `apps/runtime/tests/service_container.rs` uses Tokio channels and one-second
  hang guards to prove six public startup, rollback, service-failure, shutdown,
  cleanup, and fresh-run cases without arbitrary sleeps.
- `apps/runtime/tests/http_health.rs` proves four public loopback lifecycle,
  exact-wire, bind-failure, release, and repetition cases. It can observe
  `Initializing`, `Running`, and `Stopping` without changing the health schema.
- `.github/workflows/ci.yml` runs format, check, test, and Clippy on `macos-14`
  and `windows-latest`; fixtures and tests must avoid Unix-only signals,
  symlinks, permissions, and absolute-path assumptions.

Executed inventory checks at this baseline listed five filesystem discovery
tests, five Designer semantic-graph unit tests, one focused EDT builder test,
one focused EDT validation test, six public Runtime service-container tests,
and four public HTTP health tests. The filtered Designer command
`cargo test -p oneagent-designer-xml semantic_graph -- --list` matched zero
conformance tests, so that zero-match target is not counted as conformance
evidence; the dedicated conformance target must be listed or executed directly.

### Observable first-slice cases

The repository can safely test these cases without external input:

| Case | Production entry point | Observable oracle |
|---|---|---|
| Missing or non-directory root | `WorkspaceDetector::discover` | Typed discovery error and named Runtime startup failure source chain |
| Empty readable root | `WorkspaceDetector::discover` | Successful empty ordered discovery; ADR must decide whether service startup may publish it |
| One EDT root | detector plus EDT builder | Stable configuration ID, graph/report/diagnostics/validation |
| One complete Designer root | detector plus Designer builder with `Complete` | Stable configuration ID, graph/report/validation/provenance |
| Multiple and reordered roots | detector `BTreeMap` output | Stable path order and repeated snapshot observations |
| Conflicting markers | detector | `ConflictingFormatMarkers`, no builder call, no published snapshot |
| Fatal adapter input | selected builder | Typed source error, no graph from that builder |
| Recoverable EDT references | EDT build result | Graph plus ordered diagnostics/reference statistics |
| Duplicate configuration UUID across roots | both builders plus configuration-node inspection | Deterministic collision input; ADR must decide rejection versus isolated per-configuration storage |
| Startup, readiness, cancellation, cleanup | public Runtime service/lifecycle/health seams | Channel/watch coordination, named failure, terminal `Stopped`, no surviving owners |
| Repeated fresh run | fresh builder/App and the same repository-owned input | Equal ordered observations and independent resources |

There is no current supported oracle for live rebuild, watcher invalidation,
cache persistence, query transport, or forced cancellation of arbitrary
filesystem parsing. Those behaviors remain deferred rather than missing Sprint
17 data.

## Compatibility-sensitive unknowns

The repository does not currently answer the following. ADR-0039 must make each
decision before implementation:

1. **Root configuration:** whether exactly one required root is stored in
   `RuntimeConfig`, its default/provider behavior, path normalization boundary,
   discovery depth ownership, and missing-root failure timing.
2. **Empty discovery:** whether a valid root with zero configurations is a
   successful empty Workspace or a startup failure.
3. **Snapshot shape:** whether the canonical published unit is an ordered set of
   per-configuration build records, an aggregate graph, or another source-neutral
   immutable value. Aggregation cannot silently replace nodes with equal IDs.
4. **Adapter-result preservation:** how EDT diagnostics/reference evidence and
   the Designer graph-only result are represented without making adapter-local
   types the shared semantic authority or discarding accepted evidence.
5. **Identity and collisions:** the ordering key and exact behavior for duplicate
   configuration UUIDs, duplicate roots, path aliases, and graph node identity
   overlap across separate configurations.
6. **Publication atomicity:** whether any configuration failure rejects the
   entire initial snapshot, and what observers retain or see on failed startup.
7. **Build execution:** the owner of format dispatch and blocking work, use of
   `spawn_blocking`, join/panic/error mapping, and whether builds are sequential
   or bounded-parallel. No performance requirement justifies parallelism now.
8. **Startup and readiness:** whether initial build completion is part of
   `RuntimeService::start`, service registration order relative to HTTP, and
   whether existing lifecycle-derived readiness is sufficient.
9. **Cancellation:** what cancellation can mean before startup acknowledgement,
   given synchronous adapters and the current App startup loop, and whether the
   first slice explicitly treats initial builds as non-interruptible but joined.
10. **State ownership:** whether the immutable observer belongs in `AppState`, a
    dedicated Workspace handle constructed before registration, or another
    accepted Runtime-owned object.
11. **Errors and observability:** the source-neutral Workspace error/result
    vocabulary, stable fields, bounded path exposure, and whether existing named
    Runtime error classification is sufficient.
12. **Fixture boundary:** whether the public Runtime target may reuse the
    cross-adapter fixture by repository-relative path or must own a copied,
    provenance-linked complete fixture under `apps/runtime/tests/fixtures/`.

## Alternatives requiring an ADR decision

These are viable questions, not accepted outcomes:

- publish ordered per-configuration graphs versus merge into one graph;
- expose Workspace observation through `AppState` versus a dedicated pre-built
  service handle;
- perform all initial builds before service acknowledgement versus acknowledge
  a long-lived task and add a separate readiness input;
- fail the entire initial snapshot on one configuration versus preserve explicit
  per-configuration failure records;
- run deterministic sequential blocking builds versus bounded concurrent builds;
- use the existing Runtime error taxonomy alone versus add a stable
  source-neutral Workspace error classification beneath it.

Rejected or unsupported shortcuts already follow from accepted evidence:
Runtime must not infer Designer `Partial`, convert Designer input to EDT, discard
EDT diagnostics without a contract, expose mutable graphs, select a builder by
filesystem iteration order, silently overwrite duplicate identities, block the
async executor with unbounded synchronous parsing, detach blocking work, or add
Workspace/graph HTTP routes in Sprint 17.

## Decision readiness

The evidence gate passes. Every bounded initial-build capability has a current
production entry point, repository-owned positive and negative data, a public
or derivable deterministic test oracle, and a known focused plus full workspace
validation path. No external artifact, service, ignored corpus, new production
dependency, or speculative source format is required.

ADR-0039 is required before implementation because root configuration, result
shape, collision behavior, publication atomicity, blocking-task ownership,
startup/readiness, cancellation, state ownership, and public error/observation
contracts remain genuinely unresolved. Task 2 may proceed once this document is
committed; production work must stop if ADR-0039 cannot resolve the listed
questions without contradicting the accepted constraints.
