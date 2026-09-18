# Sprint 39 Change Impact Analysis Evidence

## Status and scope

This document records Task 6 evidence executed on 2026-09-01 from committed
Task 5 head `9ea95dbf293831e0740733ee8f73a3b2a0bf198f`. The implemented boundary is
governed by [ADR-0061](../adr/0061-change-impact-analysis.md). Sprint 39 remains
active until the mandatory fresh-context Task 7 review, primary
reconciliation, artifact-consistency check, Sprint 40 hand-off, and conditional
Sprint 38 prompt-suite retirement.

The first slice adds one complete bounded source-independent product report for
the latest adjacent successful Workspace publications. Graph remains the sole
authority for semantic facts, diff, dependency classification, propagation,
impact seeds, statuses, availability, reasons, and traversal completeness.
Analysis owns report identity, Configuration matching, admission, ordering, and
reconciliation. Workspace owns publication and lifecycle. MCP owns projection,
protocol validation, and the existing Tool Policy gate.

The committed implementation chain is:

| Slice | Commit |
| --- | --- |
| Planning | `6d9fd0ff` |
| Investigation | `4522a6d9` |
| ADR-0061 | `706c2665` |
| Analysis report | `c1ea37fb` |
| Workspace publication | `483a0865` |
| MCP and public process | `9ea95dbf` |

## Requirement-to-test matrix

| ADR-0061 requirement | Repository-owned evidence | Result |
| --- | --- | --- |
| Graph remains the only fact, diff, impact, seed, reason, status, availability, and traversal authority | No Graph production file changed in the Task 3–5 range; 18 Graph Impact tests and 86 Graph validation/report/build-diff/reference/Coverage tests | pass |
| Analysis accepts only non-zero adjacent publication IDs, canonical Configuration IDs, complete borrowed graphs, and cancellation | `crates/analysis/src/change_impact.rs` API/source audit, 2 internal bound/overflow tests, and 10 public Change Impact tests | pass |
| Fixed policy is depth 4, all default dependency kinds, ownership disabled, and provenance direct-only | Analysis source audit plus direct/transitive/reason and Graph Impact policy tests | pass |
| Publication identity is process-local checked non-zero `u64`; report identity is the ordered adjacent pair | Analysis publication overflow test; Runtime initial, replacement, failure/recovery, cache, and fresh-service tests | pass |
| Configuration identity uses only canonical `EntityId`; names, roots, formats, paths, order, and Git evidence do not match transitions | Analysis reorder/duplicate tests; Workspace name/format transition unit test; Git/filesystem end-state equivalence test | pass |
| Same-ID inputs compare; previous-only is Removed; current-only is Added; an ID transition is removal plus addition with no rename inference | Public Analysis added/removed test and Runtime watching/MCP live-publication tests | pass |
| Exact duplicate endpoint inputs collapse; same-ID different-graph input fails without selecting by input order | Public Analysis duplicate/reorder/repetition and conflict/redaction tests | pass |
| Empty endpoints and equal graphs produce complete empty reports; equal successful rebuilds create distinct adjacent publications | Public Analysis empty/equal tests and Git-input equal-rebuild publication test | pass |
| Every Graph status, availability, direct/transitive/removed case, and typed reason remains owned and ordered | Public Analysis positive report test and 18 Graph Impact tests | pass |
| Configuration, node, and reason order is canonical and independent from insertion, discovery, trigger, and operation order | Analysis reorder/repetition, Graph insertion-order, Runtime fresh-run, and Git opposite-order tests | pass |
| Report and transition summaries reconcile checked Configuration, seed, status, availability, affected-node, and depth counts | Analysis positive/empty/equal/bound tests and MCP depth/truncation projection test | pass |
| Complete in-memory admission is exact at 4,096 Configurations, 4,096 identifier bytes, 65,536 affected nodes, 256 reasons per node, 262,144 reasons total, and depth 4 | Analysis exact/one-over public and internal tests, including Graph-owned canonical `EdgeId` values at 4,096 and 4,097 bytes on equal graphs with no reasons; no partial result is returned | pass |
| Closed failures cover conflicts, identifier/count bounds, inconsistent Graph evidence, summary/publication overflow, and cancellation without rejected values | Analysis error-kind/display tests and Runtime error mapping tests | pass |
| Cancellation is checked around normalization and every Graph call; no partial report or detached task survives | Analysis cancellation test and Runtime active-build cancellation/shutdown unit tests | pass |
| A snapshot atomically embeds either initial no-predecessor availability or one complete adjacent report | Runtime unit tests plus File Watching, Git-input, cache, and MCP observer tests | pass |
| Cold, warm, and standalone initial snapshots use publication 1 with `NoPreviousPublication` and never invent history | Workspace builder/unit and 4 public cache tests | pass |
| Successful update assigns exactly one next ID and atomically publishes graph and report | Runtime unit tests, 2 File Watching tests, 3 Git-input tests, and live MCP observer/process tests | pass |
| Failed, cancelled, stale, over-bound, or invalid builds consume no ID, retain the last valid snapshot, and later recover against that snapshot | Runtime unit tests and public File Watching/Git-input failure-recovery tests | pass |
| Coalesced follow-up work remains single-writer and compares successive accepted publications | Runtime 123-test unit suite, including controlled follow-up/cancellation/observer tests | pass |
| Shutdown joins owned source/build work, clears the snapshot, closes observers, and fresh service runs restart at publication 1 | Runtime unit, Workspace, File Watching, Git-input, MCP EOF, and VS Code process tests | pass |
| External `Arc` clones stay immutable while Workspace retains no predecessor/history after report construction | Runtime observer/unit tests and source ownership audit | pass |
| Cache schema remains 1, semantic compatibility is 5, and publication IDs/reports are not serialized | Cache constants/source audit and explicit JSON-envelope assertions in 4 public cache tests | pass |
| Current, old/new, stale-source, corrupt/checksum-invalid, unavailable, and write-failed cache states preserve clean-build recovery | 4 public cache tests and Runtime cache unit matrix | pass |
| Cold and warm semantic content is equal; both start with no predecessor; later watched updates build impact from the actually published endpoint | Public cold/warm/watched cache equality and publication assertions | pass |
| Filesystem and Git triggers with equal complete semantic endpoints produce equal reports apart from process-local IDs | Public Git/filesystem complete-end-state equivalence and opposite-operation-order tests | pass |
| Repository root, paths, statuses, baseline, completeness, staged/untracked origin, and operation order never enter impact identity, seeds, reasons, summaries, cache, wire output, or errors | Analysis input/API audit, Git-input mapping audit, cache envelope assertions, equality tests, MCP redaction tests, and sensitive-data scans | pass |
| MCP stays at revision `2026-07-28`, retains legacy negotiated revisions, truthful `capabilities.tools={}`, and exactly seven ordered tools | 53 Protocol, 9 semantic-tool, 8 stdio, 18 process, VS Code, EDT, and external-client-fixture tests | pass |
| `oneagent.impact` advertises exact exclusive legacy and publication selectors; missing, partial, mixed, extra, malformed, reordered, and out-of-range arguments use deterministic existing precedence | Semantic catalog/schema and selector-matrix tests plus process compatibility calls | pass |
| Legacy mode preserves same-snapshot two-Configuration request, computation, result shape, defaults, order, errors, and bounds | Existing semantic-tool/public-process responses and 18 Graph Impact tests | pass |
| Publication mode supports available/unavailable, compared/added/removed/equal transitions and keeps removed IDs queryable for the latest report | In-memory initial/live observer tests and live public-process test | pass |
| Depth filtering never reruns Graph; complete requested-depth summary precedes item/reason limits; truncation and omitted reasons reconcile | Runtime internal projection test and public observer-backed limit/reason assertions | pass |
| Depth is `0..=4`; item and reason limits are `1..=100`; a result above 65,536 bytes fails closed without shortening strings | Catalog/argument exact/one-over tests and oversized-result tests | pass |
| Every known call clones exactly one immutable snapshot and traverses the unchanged read-only Tool Policy identity/effect/decision/execution/audit path | Observer-backed server source audit, 33 Tool Policy tests, Runtime policy unit tests, and semantic repetition tests | pass |
| Denial remains `policy_denied`; execution/output failures remain closed; schema annotations do not authorize a call | Runtime policy-denial/failure/oversized-output tests | pass |
| The public MCP process waits for an initial complete publication, observes later atomic publications between calls, keeps stdout pure, and joins Runtime on EOF/failure/cancellation | 18 public process tests, including live mutation, repeated sessions, redacted startup failure, channel purity, and EOF | pass |
| Public process tests never write persistent cache into shared fixtures | Rust and VS Code process tests copy fixtures into disposable directories; post-run fixture-clean audit | pass |
| Static `semantic_server(WorkspaceSnapshot)` remains supported while observer-backed construction is additive | Runtime public API/Rustdoc audit and static/observer semantic tests | pass |
| HTTP, CLI, LSP, VS Code production client, EDT integration, diagnostics, and rules gain no new impact operation or UI | 3 Graph Query, 4 HTTP, 2 CLI process, 5 LSP stdio, 8 LSP process, 62 VS Code unit, 18 Extension Host, 2 VS Code process, 41 EDT extension, 339 Rust EDT, and 128 Analysis tests | pass |
| Existing Codex/Cursor discovery/catalog fixtures remain compatible and the `{}` impact negative request remains invalid | `public_mcp_process_runs_exact_codex_and_cursor_lifecycles_repeatably`; no new live external-client matrix is claimed | pass |
| No Cargo dependency, feature, license, unsafe, protocol revision, Coverage capability, configuration/credential, or reverse dependency is added | Manifest/lock/Coverage/unsafe/sensitive-value diff audits, strict Clippy, Rustdoc, VS Code/EDT package audits | pass |
| No selective semantic mutation, new Graph authority, scoring, risk prediction, refactoring/edit behavior, remote Git, telemetry, benchmark, or broad security/performance claim is introduced | Task-range production diff and deferred-scope scans | pass |

No required row uses a zero-match test filter and no required row is skipped.
The four zero-test all-target entries are the expected public binary entry
points `oneagent-cli`, `oneagent-runtime`, `oneagent-mcp`, and `oneagent-lsp`;
they are inventory, not acceptance evidence.

## Focused local evidence

The following commands ran sequentially from the repository root and exited
zero unless a different working directory is shown:

| Command or exact suite | Tests passed | Failed / ignored / filtered |
| --- | ---: | --- |
| `cargo test -p oneagent-graph --test impact --quiet` | 18 | 0 / 0 / 0 |
| `cargo test -p oneagent-graph --test validation --test report --test build_diff --test reference_request_build --test coverage --quiet` | 86 | 0 / 0 / 0 |
| `cargo test -p oneagent-analysis --quiet` | 128 | 0 / 0 / 0 |
| `cargo test -p oneagent-analysis --test change_impact --quiet` | 9 | 0 / 0 / 0 |
| `cargo test -p oneagent-runtime --lib --quiet` | 123 | 0 / 0 / 0 |
| `cargo test -p oneagent-runtime --test workspace_service --quiet` | 6 | 0 / 0 / 0 |
| `cargo test -p oneagent-runtime --test file_watching --quiet` | 2 | 0 / 0 / 0 |
| `cargo test -p oneagent-runtime --test git_change_workspace --quiet` | 3 | 0 / 0 / 0 |
| `cargo test -p oneagent-runtime --test persistent_cache --quiet` | 4 | 0 / 0 / 0 |
| `cargo test -p oneagent-runtime --test graph_query_api --quiet` | 3 | 0 / 0 / 0 |
| `cargo test -p oneagent-protocol --quiet` | 53 | 0 / 0 / 0 |
| `cargo test -p oneagent-tool-policy --quiet` | 33 | 0 / 0 / 0 |
| `cargo test -p oneagent-runtime --test mcp_semantic_tools --quiet` | 9 | 0 / 0 / 0 |
| `cargo test -p oneagent-runtime --test mcp_stdio --quiet` | 8 | 0 / 0 / 0 |
| `cargo test -p oneagent-runtime --test mcp_process --quiet` | 18 | 0 / 0 / 0 |
| `cargo test -p oneagent-runtime --test http_health --quiet` | 4 | 0 / 0 / 0 |
| `cargo test -p oneagent-runtime --test lsp_stdio --quiet` | 5 | 0 / 0 / 0 |
| `cargo test -p oneagent-runtime --test lsp_process --quiet` | 8 | 0 / 0 / 0 |
| `cargo test -p oneagent-cli --test runtime_client --quiet` | 2 | 0 / 0 / 0 |
| `cargo test -p oneagent-edt --quiet` | 339 | 0 / 0 / 0 |

The Analysis total is 73 library tests plus public targets with 9 Change
Impact, 11 Context, 3 Diagnostics, 6 Rule diagnostics, 15 Rule execution, 6 Rule
planning, and 5 Rule registry tests. The two internal Change Impact tests are
included in the 73 library tests. The 86 Graph compatibility tests are build
diff (2), Coverage (19), reference requests (7), report (3), and validation
(55).

The workspace inventory command enumerated 81 all-target test executables: 77
non-zero targets, the four expected zero-test binaries, and 1,286 tests. The
inventory was derived from `cargo test --workspace --all-targets -- --list
--format terse`; no filtered test contributes to the total.

## VS Code, EDT, and platform evidence

Local VS Code production and test TypeScript type checks and compilation ran
with the bundled Node executable, followed by 62 unit tests and 2 public
`oneagent-mcp` integration tests. All passed with zero failures,
cancellations, skips, or todos. The first convenience invocation, `pnpm run
typecheck`, did not start TypeScript because `node` was absent from the process
`PATH`; it exited 1 with `node: not found` and is not acceptance evidence. The
same two type checks were then executed directly with the bundled Node path and
exited zero. The public-process test now copies the shared fixture to a
temporary directory, so production cache writes do not modify tracked fixtures.

Exact-head [CI run 33431687151](https://github.com/MakSGTS/OneAgent/actions/runs/33431687151)
completed successfully at `9ea95dbf293831e0740733ee8f73a3b2a0bf198f`.
All six jobs passed: Rust, VS Code, and EDT on both `macos-14` and
`windows-latest`.

- Each Rust job ran formatting, all-target checking, both public Runtime
  process builds, the complete workspace tests, focused Context/MCP
  compatibility, strict Clippy, and warning-denied Rustdoc.
- Each VS Code job passed typecheck, 62 unit tests, 18 Extension Host scenarios,
  2 public Runtime process tests, the 12-file package inventory, two equal
  14-file VSIX builds, and the scope/dependency audit of 43 tracked extension
  files, 18 license groups, and 3 documents.
- Each EDT job passed the JDK/host boundary, public Runtime build, 41 tests with
  zero failures/errors/skips including the real Runtime process, and the p2
  package audit. The macOS package contains exactly 7 repository files and 4
  content units; the Windows job passed the same checks.

No local GUI launch is claimed. The exact-head CI jobs provide the required
cross-platform Extension Host and Eclipse/Tycho evidence. Task 6 did not run a
live Codex or Cursor executable and does not claim a new external-client
matrix. The repository-owned exact initialize/catalog fixtures and current
public-process compatibility test preserve the accepted Sprint 35 evidence.

## Post-review remediation evidence

The first fresh-context Task 7 review blocked completion because Analysis
validated edge endpoints but did not validate the byte length of Graph's
composite canonical `EdgeId` on equal graphs where no impact reason retained
that identity. Remediation commit
`eee6b615571f18beb61811ea2752119b93949e9c` closes that gap without adding a
second identity authority: Analysis calls the existing Graph-owned
`SemanticGraphQuery::edge_id` constructor for every input edge before report
admission.

The public Change Impact boundary test now proves both inclusive cases on equal
graphs with an empty diff and no reasons: a canonical edge identifier of
exactly 4,096 bytes is accepted, while 4,097 bytes fails closed with
`IdentifierTooLarge`, `actual=4097`, and `maximum=4096`. The focused remediation
matrix passed 10 public Change Impact tests, 129 total Analysis tests, 18 Graph
Impact tests, 86 Graph compatibility tests, 123 Runtime unit tests, and the
unchanged Workspace, watching, Git-input, cache, query, protocol, Tool Policy,
MCP, HTTP, LSP, CLI, and Rust EDT suites with zero failures, ignored tests, or
filtered tests.

The complete remediation inventory contains 81 all-target executables, 77
non-zero targets, the same four expected zero-test binaries, and 1,287 tests.
`cargo fmt --all -- --check`, `cargo check --workspace --all-targets`,
`cargo test --workspace --all-targets`, strict all-feature Clippy,
warning-denied workspace Rustdoc, and `git diff --check` all exited zero.

Exact-code-head [CI run 33457248893](https://github.com/MakSGTS/OneAgent/actions/runs/33457248893)
completed successfully at the remediation commit. All six Rust, VS Code, and
EDT jobs passed on `macos-14` and `windows-latest`, including the public Runtime
processes, Extension Host and real Runtime integration, package and dependency
audits, Clippy, and Rustdoc. No live Codex or Cursor executable run is newly
claimed; the unchanged repository-owned client lifecycle fixture remains the
accepted compatibility evidence.

## Canonical gate

The Task 6 final validation cycle is:

| Command | Exact outcome |
| --- | --- |
| `cargo fmt --all -- --check` | exit 0 |
| `cargo check --workspace --all-targets` | exit 0 |
| `cargo test --workspace --all-targets` | exit 0; 81 targets, 1,286 passed, 0 failed/ignored/measured/filtered |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | exit 0 |
| `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps` | exit 0 |
| `git diff --check` | exit 0 |

## API, dependency, cache, and sensitive-data audits

- `oneagent-analysis::change_impact` is an additive public module with typed
  publication/configuration/report/summary/completeness/error/cancellation
  values and fixed bound constants. Runtime additively exposes snapshot
  publication/impact accessors, `WorkspaceChangeImpact`, the observer-backed
  semantic-server constructor, and `App::run_without_banner`. Existing public
  items and signatures remain available.
- No root or crate `Cargo.toml`, `Cargo.lock`, production dependency, feature,
  third-party package, native library, license inventory, workspace member, or
  reverse dependency changed in Tasks 3–5. Workspace crates continue to forbid
  unsafe Rust; the changed production sources contain no `unsafe` block.
- No Graph production or Coverage registry file changed. Analysis invokes
  `SemanticGraph::diff` and `SemanticImpactAnalyzer`; Runtime and MCP do not
  implement a competing traversal.
- Cache schema remains `1`; semantic compatibility is exactly `5`. The cache
  DTO contains neither publication ID nor change-impact report. Version-4 and
  otherwise incompatible entries use the existing clean-rebuild path.
- MCP remains pinned to modern revision `2026-07-28`, retains legacy
  `2025-06-18` and `2025-11-25`, advertises the same seven tools and empty tools
  capability object, and preserves the existing 1 MiB frame and 65,536-byte
  Tool Policy argument/result boundaries.
- Task-range searches found no absolute/personal path, username, private-key
  marker, common token prefix, credential, or bearer value in changed code or
  tests. Public impact projections exclude roots, provenance, source content,
  formats, repository values, policy internals, and raw error chains.
- Repository-owned process/client tests use disposable directories. The shared
  `apps/runtime/tests/fixtures/workspace_service` tree contained no generated
  `.oneagent` cache after the final focused, client, inventory, and workspace
  runs.
- The Task 3–5 range contains only Analysis, Runtime, tests, and one VS Code
  process-test isolation change. It contains no generated binary, cache,
  package, credential, local log, or unrelated source change.
- Coverage state is unchanged because the report creates no node, edge, parser
  result, source fact, dependency kind, or diagnostic producer.

The principal read-only audit commands returned the expected empty changes or
zero matches:

```text
git diff --name-only 706c2665..9ea95dbf -- Cargo.toml Cargo.lock apps/runtime/Cargo.toml crates/analysis/Cargo.toml crates/graph/Cargo.toml
git diff --name-only 706c2665..9ea95dbf -- crates/graph/src/coverage.rs adapters/edt/src/coverage.rs adapters/designer-xml/src/coverage.rs
rg -n '/Users/|maxim_tomshin|BEGIN [A-Z ]*PRIVATE KEY|ghp_|github_pat_|AKIA|Bearer |password|credential' <Task 3-5 changed code and tests>
rg -n '\bunsafe\b' <Task 3-5 changed production sources>
git diff -U0 706c2665..9ea95dbf -- crates/analysis/src apps/runtime/src extensions/vscode/src | rg -i 'risk|scor|refactor|source edit|code action|telemetr|remote git|history query|selective semantic|incremental graph'
```

The exact-head CI query returned `completed` / `success` and six successful
jobs. No configuration source, credential mechanism, network protocol, remote
Git operation, or user repository is introduced by the implementation.

## Current boundary and Sprint 40 hand-off

The report describes only the latest adjacent successful publications in one
running Workspace service. Publication IDs reset across processes and are not
history, timestamps, commits, or persistent identities. Startup and warm cache
hits correctly expose no predecessor. Reports are complete only through fixed
Graph depth four; MCP may project depth zero through four and independently
omit items or reasons with explicit counts. A whole in-memory or protocol
result that exceeds its bound fails closed.

The first slice does not infer semantic change from repository paths or
statuses, retain arbitrary endpoints, persist transition history, select
changed sources, mutate a graph incrementally, score or predict risk, produce a
refactoring plan, edit source, offer transactions or rollback, add HTTP/CLI/LSP
or IDE impact UI, access remote Git, emit telemetry, or establish a performance
or security benchmark.

Sprint 40 Refactoring Planner may consume only a separately accepted immutable
semantic precondition and plan contract. It must not reinterpret this impact
report as authorization to edit, as a path-to-node mapping, as unbounded
closure, or as risk/scoring evidence. Before Sprint 40 can become active, Task 7
must independently review this Sprint 39 implementation and evidence, reconcile
findings, pass artifact consistency and the full gate, complete Sprint 39, and
perform the roadmap transition.
