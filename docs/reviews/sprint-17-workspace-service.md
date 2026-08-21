# Sprint 17 Workspace Service Integration Review

## Decision

`pass`

Sprint 17 satisfies ADR-0039 and the Roadmap completion gate. No blocking or
non-blocking findings and no missing acceptance evidence remain. Sprint 18 Graph
Query API may become the unique `next` target.

## Reviewed baseline

- Planning parent: `dd08923a54f5eacf5aad5a3cbc1a16267dadaa21`
- Reviewed range: `d8571c46^..7626c46a`
- Task 5 head: `7626c46a11f683b11fc8fc007738a85e5e65ea86`
- Review date: 2026-08-21

| Commit | Outcome |
| --- | --- |
| `d8571c46df227e4b7b1f90ee1db0a753f8ac9416` | Sprint 17 execution plan and ordered prompt suite |
| `f5b245c731cc31b0006b1fe7e89bb2d04ce09002` | Live Workspace, adapter, Runtime, dependency, fixture, and testability investigation |
| `48ee42a037a1ae84a2bffac898dfb411cc162a36` | Accepted ADR-0039 contract |
| `52a43a4148b7291ea8054c2fe376211e4dfb4927` | Immutable source-neutral snapshot and production build dispatch |
| `15989277845d73e46388ada43ab18af42515c2f2` | Runtime-owned Workspace service, observer, configuration, lifecycle, and composition |
| `7626c46a11f683b11fc8fc007738a85e5e65ea86` | Public production evidence, bounded fixture, and current-state documentation |

The range changes only the planned Runtime composition/configuration/snapshot
surface, local path dependencies, public tests and fixtures, and bounded
architecture, Roadmap, prompt, and current-state documents. It adds no external
dependency and changes no graph-domain semantics, adapter parser behavior,
protocol, Workspace/graph HTTP route, watcher, persistence, supported CLI, or
later-sprint implementation.

## Acceptance evidence matrix

| Criterion | Evidence | Result |
| --- | --- | --- |
| Planning readiness | The live investigation proves the existing Runtime Service framework, production detector/builders, canonical graph contracts, repository fixtures, and deterministic seams are sufficient without framework changes. | pass |
| Accepted architecture | ADR-0039 fixes Runtime ownership, root configuration, build dispatch, immutable per-configuration snapshots, atomicity, ordering, identity, lifecycle, failure, cancellation, and deferred scope before implementation. | pass |
| Dependency direction | Runtime depends only on existing local workspace/domain/adapter crates; adapters retain filesystem and source parsing, while `SemanticGraph` remains canonical authority. | pass |
| Configuration and discovery | `RuntimeConfig` preserves constructors/defaults, adds `.` plus an explicit root override, and passes the path unchanged through one production discovery with detector-owned validation/depth. | pass |
| Both production builders | EDT uses the diagnostics-preserving full build; Designer XML uses explicit `Complete`. The public mixed fixture produces exact independently validated graphs through both paths. | pass |
| Snapshot identity and order | Separate immutable configuration records retain canonical IDs/names, roots/formats, graphs, diagnostics, request evidence/statistics, and reports in Configuration-ID order with read-only lookup. | pass |
| Atomicity and collisions | Private construction publishes only after every build and validation; unit/public evidence rejects unsupported format, duplicate identity, conflicting markers, and a later fatal adapter input without partial publication. | pass |
| Blocking work ownership | One awaited `spawn_blocking` owns discovery and sequential builds; join/panic classification is tested, and no detached build handle is introduced. | pass |
| Lifecycle and readiness | Snapshot publication precedes service startup acknowledgement; `Running` still follows all acknowledgements. Public real HTTP requests prove lifecycle remains the sole readiness authority in `Initializing`, `Running`, and `Stopping`. | pass |
| Failure classification | Missing/non-directory roots and semantic failures become named `ServiceStartFailed` errors for `workspace` with preserved typed source categories and terminal `Stopped`. | pass |
| Cancellation and cleanup | Receiver-only cancellation clears the snapshot before Workspace task completion; ADR-0037 reverse cleanup and complete join behavior remain unchanged. | pass |
| Repetition and resource ownership | Fresh mixed and empty applications repeat equal observations; snapshot channel closure and terminal lifecycle prove no sender or service task survives `App::run`. | pass |
| Public fixture provenance | The bounded Runtime fixture documents exact EDT/Designer origins, reductions, generated query evidence, LF policy, and SHA-256 for every tracked source artifact; tests never require ignored corpora. | pass |
| Cross-platform deterministic tests | Public tests use temporary directories, Tokio channels/watches, loopback TCP, and one-second hang guards; they use no arbitrary sleep, symlink, real signal, external service, or platform-specific absolute path. | pass |
| Documentation truth | README, Architecture, Semantic Model, Roadmap, investigation, and ADR agree on implemented initial build and retain graph query, watcher, cache, and CLI exclusions. | pass |
| Scope containment | No Workspace/graph route, merged graph, mutable readiness flag, rebuild, watcher, cache, retry, restart, forced abort, new timeout, external dependency, parser change, or Coverage inflation appears in the range. | pass |

## Findings

No blocking or non-blocking findings.

## Missing evidence

None.

Zero-match filtered targets were not counted. Controlled detector and blocking
task cases are established by the nine non-zero Workspace unit tests; production
format, failure, readiness, and cleanup cases are established independently by
the six-test public target.

## Validation

The review reran the complete focused matrix:

- `cargo test -p oneagent-runtime workspace::tests -- --list` — exactly 9 tests.
- `cargo test -p oneagent-runtime workspace::tests` — 9 passed.
- `cargo test -p oneagent-runtime config::tests -- --list` — exactly 3 tests.
- `cargo test -p oneagent-runtime config::tests` — 3 passed.
- `cargo test -p oneagent-runtime --test workspace_service -- --list` — exactly 6 tests.
- `cargo test -p oneagent-runtime --test workspace_service` — 6 passed.
- `cargo test -p oneagent-runtime --test service_container` — 6 passed.
- `cargo test -p oneagent-runtime --test http_health` — 4 passed.
- `cargo test -p oneagent-workspace` — 1 passed.
- `cargo test -p oneagent-workspace-fs` — 5 passed.
- `cargo test -p oneagent-designer-xml` — 31 unit and 3 conformance tests passed.
- Complete `oneagent-edt` and `oneagent-graph` package test suites passed.

The complete gate also passed:

- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps`
- `git diff --check`

## Deferred scope

Graph and semantic query endpoints remain Sprint 18; file watching and rebuild
triggers remain Sprint 19; persistent cache remains Sprint 20; supported CLI
behavior remains Sprint 21. Aggregate/merged graph semantics, HTTP Workspace
routes, MCP/LSP/IDE/AI surfaces, authentication, authorization, TLS, metrics,
streaming, retry/restart, forced termination, and performance claims remain
outside the accepted Sprint 17 slice. These are explicit boundaries, not
missing evidence.

## Risk assessment

Residual risk is low and bounded to the deliberately deferred initial-build
limitations: blocking builds are joined but non-interruptible, snapshots do not
rebuild, and consumers must select one configuration graph. Each limitation has
an explicit later-sprint owner or accepted first-slice policy.

## Previous-suite retirement

Before review outputs, both `git ls-files` and the filesystem contained exactly
the seven planned Sprint 16 prompt files and no additional or untracked file.
Repository search found no retained Markdown link dependency on an individual
file. The exact suite is retired atomically with this review; the complete
Sprint 17 suite and `docs/codex/prompts/run-next-sprint.md` remain tracked.
