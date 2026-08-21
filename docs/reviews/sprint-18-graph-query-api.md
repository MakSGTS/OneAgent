# Sprint 18 Graph Query API Integration Review

## Decision

`pass`

Sprint 18 satisfies ADR-0040 and the Roadmap completion gate. No blocking or
non-blocking findings and no missing acceptance evidence remain. Sprint 19 File
Watching may become the unique `next` target.

## Reviewed baseline

- Planning parent: `dac86be41eed3356e230079ab2607503a85f5b87`
- Reviewed range: `9e9ab1c0^..d7ba04bc`
- Task 5 head: `d7ba04bc6a4b0e18d46d051419809c2e0756fce7`
- Review date: 2026-08-21

| Commit | Outcome |
| --- | --- |
| `9e9ab1c062c7e934bc3f7a933ad6215fac1d9ff6` | Sprint 18 execution plan and ordered prompt suite |
| `cd10f7d262e7ae55269891b4f7f595624a00cb4c` | Live graph-query, Workspace snapshot, Runtime HTTP, dependency, fixture, and testability investigation |
| `9cbd15e9e0baae7b19832342dc4772805c46d820` | Accepted ADR-0040 Graph Query API contract |
| `1e352f2233f7ce8e8331a153c89e2f473de74dee` | Transport-neutral observer-backed query component, owned projections, bounds, errors, and focused evidence |
| `69a1d3ddc9ffc6c6b605d8e687362d23520d74e7` | Exact versioned HTTP adapter, production composition, strict request parsing, and loopback evidence |
| `d7ba04bc6a4b0e18d46d051419809c2e0756fce7` | Public EDT/Designer XML API evidence and current-state documentation |

The range changes only the planned Runtime query and HTTP surface, production
composition, public Runtime tests, ADR/investigation/current-state documents,
Roadmap planning, and the Sprint 18 prompt suite. It adds no external
dependency and changes no canonical graph facts, source-adapter parser,
Workspace build policy, health schema, file-watching, persistence, supported
CLI, protocol crate, or later-sprint behavior.

## Acceptance evidence matrix

| Criterion | Evidence | Result |
| --- | --- | --- |
| Planning readiness | The committed plan resolves Sprint 18 as the unique eligible target, proves the existing Runtime Service framework sufficient, orders six prerequisite-gated tasks, and preserves the exact previous-suite retirement gate. | pass |
| Investigation completeness | The investigation records live Graph Query behavior, immutable snapshot/configuration selection, Runtime HTTP ownership, consumers, dependencies, compatibility constraints, production fixtures, loopback restrictions, and every ADR decision question before implementation. | pass |
| Accepted architecture | ADR-0040 fixes semantic authority, ownership, dependency direction, exact operations, bounds, projections, errors, routes, schemas, lifecycle, compatibility, deterministic evidence, first slice, and deferred scope. | pass |
| Semantic authority | `SemanticGraph` and `SemanticGraphQuery` remain canonical. Runtime only selects one immutable configuration, delegates existing exact adjacency/traversal behavior, bounds the owned projection, and never repairs or reinterprets graph facts. | pass |
| Snapshot consistency | Every transport-neutral call obtains one `Arc<WorkspaceSnapshot>`, distinguishes unavailable and valid empty snapshots, selects one exact configuration, and retains owned result values independent of later observations. | pass |
| Configuration and node selection | Configuration listing preserves canonical identity order. Exact IDs select independent EDT or Designer XML graphs; invalid, missing configuration, missing node, and known-empty outcomes remain distinct. | pass |
| Configuration operation and bounds | Listing defaults to 50, accepts `1..=100`, returns only the canonical prefix, and sets `truncated` exactly when additional configurations exist. | pass |
| Node operation | Exact lookup returns an owned payload-free node projection with exhaustive stable node and nested metadata-kind vocabulary and no result limit. | pass |
| Relation operation | Direct literal incoming/outgoing adjacency preserves stable edge identity order, accepts one optional exact edge-kind filter, returns the reached node projection, bounds results, and distinguishes an unknown node from a known empty relation set. | pass |
| Traversal operation | Deterministic breadth-first traversal is direction-aware, cycle/self-loop safe, depth-bounded to `0..=4`, optionally includes the start node, accepts one edge-kind filter, bounds results, and preserves first-discovery edge evidence. | pass |
| Owned projections and errors | Configuration, node, relation, and traversal results own only accepted fields. Typed errors distinguish availability, identifiers, selection, limit, and depth without exposing payloads, provenance, source paths, diagnostics, or `Display` prose. | pass |
| Request parsing | The HTTP adapter decodes UTF-8 query components once and rejects unknown, duplicate, missing, empty required, malformed percent, invalid UTF-8, and structurally invalid parameters. Closed direction, edge, boolean, unsigned-decimal, overflow, and range rules map to exact stable errors. | pass |
| Route and method compatibility | Exactly four `/api/v1` GET routes are registered only by the query-enabled constructor. Registered HEAD/POST return `405`, `Allow: GET`, empty body; unknown and trailing-slash paths retain empty `404`; `HttpService::new()` remains health-only. | pass |
| Success and error schemas | Every success and domain error is JSON with the exact ADR-0040 fields, enum values, status, code, and message. Omitted optional response values serialize as `null`; arrays retain deterministic order; `Accept` does not alter JSON. | pass |
| Runtime composition | Production creates Workspace service, observer, query component, and query-enabled HTTP service explicitly, then registers `http` before `workspace`. The existing HTTP service remains the sole listener and connection/task owner. | pass |
| Lifecycle and readiness | Structurally valid query requests require canonical Runtime `Running` before snapshot access. Public gates prove `runtime_not_ready` in observable `Initializing` and `Stopping`, success in `Running`, and distinct `workspace_unavailable` for an absent snapshot while Running. | pass |
| Cancellation and cleanup | Reverse cancellation preserves ADR-0037/0038/0039 ordering. Workspace clears publication before completion, HTTP shuts down gracefully, all tasks join, observers close, the bound-address watch clears, and the listener can immediately rebind. | pass |
| Public production evidence | The three-test `graph_query_api` target uses raw Tokio loopback requests and the tracked Sprint 17 provenance fixture through real filesystem discovery and both production builders. It asserts exact separate identities, counts, nodes, relations, traversal, errors, lifecycle, cleanup, and equal fresh runs. | pass |
| Deterministic and cross-platform tests | Focused synthetic tests cover cycles, self-loops, truncation, exhaustive mappings, and replacement snapshots; public tests use port zero, channels/watches, and bounded hang guards with no arbitrary sleep, fixed port, real signal, external service, ignored corpus, or absolute platform path. | pass |
| Documentation truth | README, Architecture, Semantic Model, Roadmap, investigation, and ADR agree on the implemented bounded API and preserve Sprint 19 rebuild, Sprint 20 persistence, Sprint 21 supported CLI, and later exclusions. | pass |
| Scope containment | The range adds no watcher, rebuild generation, cache, aggregate graph, mutation, batch/streaming API, arbitrary query language, search, pagination, authentication, TLS, OpenAPI, external production dependency, supported client, or protocol-crate authority. | pass |

## Findings

No blocking or non-blocking findings.

## Missing evidence

None.

Zero-match filtered targets were not counted. The focused completion matrix
contains 46 distinct non-zero tests: 7 transport-neutral query tests, 3 HTTP
parser/error tests, 4 listener/router tests, 3 public Graph Query API tests, 4
public health tests, 6 public Workspace tests, and 19 canonical graph-query
tests. The complete workspace inventory contains 802 tests.

## Validation

The review reran the complete focused matrix:

- `cargo test -p oneagent-runtime workspace::graph_query::tests -- --list` — exactly 7 tests.
- `cargo test -p oneagent-runtime workspace::graph_query::tests` — 7 passed.
- `cargo test -p oneagent-runtime http::graph_query::tests -- --list` — exactly 3 tests.
- `cargo test -p oneagent-runtime http::graph_query::tests` — 3 passed.
- `cargo test -p oneagent-runtime http::tests -- --list` — exactly 4 tests.
- `cargo test -p oneagent-runtime http::tests` — 4 passed.
- `cargo test -p oneagent-runtime --test graph_query_api -- --list` — exactly 3 tests.
- `cargo test -p oneagent-runtime --test graph_query_api` — 3 passed.
- `cargo test -p oneagent-runtime --test http_health -- --list` — exactly 4 tests.
- `cargo test -p oneagent-runtime --test http_health` — 4 passed.
- `cargo test -p oneagent-runtime --test workspace_service -- --list` — exactly 6 tests.
- `cargo test -p oneagent-runtime --test workspace_service` — 6 passed.
- `cargo test -p oneagent-graph --test query -- --list` — exactly 19 tests.
- `cargo test -p oneagent-graph --test query` — 19 passed.

The complete gate also passed:

- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo test --workspace` — all 802 listed tests passed.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps`
- `git diff --check`

The managed sandbox denies loopback bind with `PermissionDenied`; all focused
and complete targets containing local TCP evidence were rerun with bounded
local-network permission. No external network or service was used.

## Deferred scope

File watching, rebuild publication, generation identity, stale-read and
invalidation policy remain Sprint 19. Persistent graph/snapshot cache,
versioning, migration, corruption, and clean-rebuild equivalence remain Sprint
20. Supported CLI discovery and Graph Query client behavior remain Sprint 21.
Payload/provenance projection, aggregate cross-configuration graphs, mutation,
additional query operations, pagination, streaming, protocol migration,
MCP/LSP/IDE/AI surfaces, authentication, authorization, TLS, OpenAPI, metrics,
rate limiting, benchmarks, and performance/security guarantees remain outside
the accepted Sprint 18 slice.

## Risk assessment

Residual risk is low and bounded to explicit first-slice limits. HTTP result
count and traversal depth are bounded, but the current canonical traversal
materializes its complete depth-bounded vector before Runtime projection and
makes no CPU or intermediate-allocation guarantee. Snapshots are immutable and
initial-build-only until Sprint 19 defines replacement semantics. Public access
has no authentication because deployment/security scope remains explicitly
deferred. None of these accepted limitations blocks the Sprint 18 contract.

## Previous-suite retirement

Before review outputs, both `git ls-files` and the filesystem contained exactly
the seven planned Sprint 17 prompt files and no additional or untracked file.
Repository search found no retained Markdown link dependency on an individual
file. The exact suite is retired atomically with this review; the complete
Sprint 18 suite, `docs/codex/prompts/run-next-sprint.md`, all older non-adjacent
suites, and `.codex/` remain unchanged.
