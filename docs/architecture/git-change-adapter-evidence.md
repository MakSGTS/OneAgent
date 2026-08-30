# Sprint 38 Git Change Adapter Evidence

## Status and scope

This document records Task 6 evidence executed on 2026-08-30 from committed
Task 5 head `e2818c83` plus the subsequent bounded reader-cleanup remediation.
The remediation removes detached cleanup work, reserves bounded cleanup within
the complete read deadline, adds production-child drop/deadline evidence, and
completes the invalid status/path and UNC matrices. Sprint 38 remains active
until a fresh-context independent review, primary reconciliation, artifact-
consistency check, Sprint 39 hand-off, and conditional Sprint 37 prompt-suite
retirement. No Git endpoint, state-layer, Workspace, Graph, cache, protocol, or
consumer behavior changes.

The implemented boundary is governed by
[ADR-0060](../adr/0060-git-change-adapter.md). Git supplies bounded local
repository evidence only. Production filesystem discovery and EDT/Designer XML
adapters remain source authorities, complete validated `SemanticGraph`
snapshots remain semantic authority, and
`SemanticGraphDiff::between(previous, current)` remains the canonical change
derivation only after two complete graphs exist.

The committed implementation chain is:

| Slice | Commit |
| --- | --- |
| Reusable task framework | `7eac8515` |
| Planning | `89c79c69` |
| Investigation | `83650d2f` |
| ADR-0060 | `095ac719` |
| Normalized change-set domain | `926a5314` |
| Local Git repository reader | `7173dea9` |
| Workspace change input | `e2818c83` |

## Requirement-to-test matrix

| ADR-0060 requirement | Repository-owned evidence | Result |
| --- | --- | --- |
| Runtime owns the local repository domain, reader, and Workspace input without making Git a Workspace, adapter, Graph, Analysis, diagnostic, rule, impact, cache, protocol, or edit authority | Public API/source audit, 119 Runtime unit tests, 86 focused Graph tests, and 117 Analysis tests | pass |
| The reader is explicit-demand only; default `App`, `RuntimeConfig`, `WorkspaceService`, CLI, HTTP, MCP, LSP, VS Code, and EDT do not discover or start Git | Construction/source audit plus default Runtime, public process, VS Code, and EDT matrices | pass |
| One caller-supplied existing directory must be the exact canonical worktree root | Public missing/file/plain/mismatch/nested tests and real normal/linked worktree tests | pass |
| Normal, detached, linked, and exact-root nested worktrees are accepted; bare, unborn, missing, non-repository, and mismatched roots are typed failures | 7 public reader tests over disposable repositories | pass |
| `GitCommitId` accepts exactly 40- or 64-character lowercase hexadecimal identities and exposes no ref, branch, tag, root, or common-directory identity | Domain unit/public tests and local SHA-1/SHA-256 production-reader evidence | pass |
| The current endpoint is the final tracked worktree plus non-ignored untracked files; staged and unstaged layers are folded relative to pinned `HEAD` | Public staged, unstaged, combined, cancelled-to-HEAD, added, modified, deleted, and untracked repository states | pass |
| Ignored paths and empty untracked directories are deliberately outside Git completeness and remain filesystem-observation concerns | Public ignored/untracked test and explicit `TrackedAndUntrackedNonIgnored` domain assertion | pass |
| Any unmerged entry fails the complete read; a changed gitlink or unsupported mode is not treated as a file change | Public conflict/gitlink repositories and injected unsupported-mode parser evidence | pass |
| Exactly two complete observation passes must agree on top level, baseline, modes, paths, completeness, and normalized changes | Injected reordered-equal and first/second-pass drift tests | pass |
| Concurrent mutation has one closed `UnstableRepository` outcome with no third pass or partial result | Controlled pass-drift test and bounded-runner call assertions | pass |
| Paths are non-empty confined UTF-8 relative values using `/`, normal components, preserved case/code points, and at most 4,096 bytes | Domain exact/over, absolute, drive, UNC, slash, backslash, dot, traversal, NUL, Unicode, and redaction tests | pass |
| Non-UTF-8 process output is rejected before publication and never echoed | Injected reader/parser test; real filesystem construction limitation is recorded below | pass with recorded platform limitation |
| The closed status vocabulary is Added, Modified, Deleted, TypeChanged, and Untracked with the exact old/new path matrix | Exhaustive valid/invalid domain unit matrix and 6 public domain tests | pass |
| Rename and copy similarity are disabled; moves and copies remain deterministic delete/add or ordinary addition evidence | Fixed `--no-renames` process audit and public move/copy repository test | pass |
| Change identity and total order use effective path, kind, previous path, and current path independently from encounter order | Domain reorder, public duplicate/reorder, injected output reorder, and real operation-order tests | pass |
| Exact duplicates collapse; non-identical records on one effective path fail atomically without selecting by encounter order | Domain unit/public duplicate and conflict tests | pass |
| Empty, exact 10,000-change, and one-over sets have deterministic complete outcomes | Domain unit/public count-bound tests | pass |
| Stdout is bounded to 16 MiB, stderr to 64 KiB, path and change counts are checked, and outputs are drained incrementally without detached reader tasks | Injected exact/over reader tests and production source audit | pass |
| One read owns at most one child at a time, one 30-second complete deadline with a reserved cleanup interval, exactly two passes, and no retry | Injected call-order/cancellation tests plus real-child drop/deadline/reap evidence | pass |
| Production invokes only `git` with fixed non-shell, NUL-oriented, non-mutating local argument vectors | Production command-construction audit and real disposable-repository tests | pass |
| Stdin is closed; stdout/stderr are concurrently drained; pager, color, quoting, rename detection, fsmonitor, untracked cache, credentials, and ambient repository override variables are disabled or removed | Production command/environment audit and injected bounded-runner tests | pass |
| Spawn, exit, read, malformed output, incompatible mode, output limit, timeout, cancellation, and contextual repository failures are closed and redacted | Runtime Git unit tests and public error-kind assertions | pass |
| Cancellation, timeout, and future drop terminate and reap owned work without a detached process, pipe-reader task, result, or partial change set | Gated runner cancellation/timeout tests, real child-process drop/deadline/reap test, and source audit | pass |
| No Rust Git package, native library, Cargo feature, manifest, lockfile, license inventory, or unsafe surface is added | Task 3-5 manifest/lockfile diff, workspace `unsafe_code = "forbid"`, Clippy, and CI audits | pass |
| The public reader supports capability-based system Git rather than a hard-coded version and requires no installer, network, credential, or user repository | Real temporary-repository tests with isolated local identity plus process/scope audit | pass |
| A cloneable pre-registration Workspace handle maps only ordered non-empty path/status records into a private source-neutral request | Runtime handle unit test, public Git-to-Workspace tests, and source/API audit | pass |
| Empty, Accepted, Backpressure, and Closed submission outcomes are exact; empty takes precedence and submission never blocks | Runtime outcome/redaction test and startup/shutdown tests | pass |
| Capacity is one; one accepted request can remain pending during a build; later input observes backpressure; accepted submissions remain separate | Gated Workspace rebuild/follow-up test | pass |
| Filesystem latest-revision observation remains active and no accepted source is selected as semantic priority | Runtime coalescing/watching tests and public Git/Workspace composition | pass |
| Every accepted non-empty input triggers complete scan, discovery, EDT/Designer build, validation, stable rescan, cache policy, and atomic publication | 2 public Git Workspace tests over copied mixed-format fixtures | pass |
| Git paths never select configurations, parsers, nodes, invalidation, diagnostics, rules, impact seeds, or partial Graph mutation | Source audit, equal complete snapshots, Graph Query comparison, and unchanged Graph/Analysis suites | pass |
| Build failure retains the last valid snapshot and a later explicit input can recover atomically | Public invalid EDT build/repair/recovery sequence | pass |
| Cache schema remains 1, semantic compatibility remains 4, and no Git identity, path, status, process data, or queue is serialized | Cache constants/diff audit, 4 public cache tests, and cold/warm/change Git Workspace test | pass |
| Cancellation joins an active complete rebuild and watcher, closes the input receiver and observers, clears the snapshot, and publishes Stopped | Gated cancellation unit test, public fresh-service/shutdown test, and Runtime lifecycle matrix | pass |
| Graph validation, reports, canonical diffs, impact behavior, diagnostics, rules, and Coverage remain unchanged | 86 Graph tests, 117 Analysis tests, Runtime composition tests, and unchanged Coverage source audit | pass |
| HTTP and CLI retain their exact routes, schemas, exit behavior, and lifecycle gating | 4 HTTP public tests and 2 CLI real-process tests | pass |
| MCP retains seven lexicographically ordered read-only Tool Policy-gated tools and all three negotiated revisions | 53 Protocol, 33 Tool Policy, 7 semantic-tool, 8 stdio, and 17 process tests | pass |
| LSP retains its exact 3.17 capabilities and immutable startup-snapshot behavior | 53 Protocol tests plus 5 LSP stdio and 8 LSP process tests | pass |
| VS Code and EDT retain their existing commands, package boundaries, process behavior, and no Git capability | Local VS Code checks and exact-head macOS/Windows VS Code/EDT CI jobs | pass |
| Public and debug errors expose no absolute/personal path, repository configuration, credentials, source content, raw command output, environment, rejected value, or internal chain | Exhaustive error formatting, controlled secret-path fixtures, source scans, and process tests | pass |
| No remote Git, repository mutation, semantic impact workflow, refactoring, edits, telemetry, benchmark, or broad performance/security claim is introduced | Production command/diff, tracked-file, docs, and scope audits | pass |

No required matrix row uses a zero-match filter and no required row is skipped.
The four zero-test all-target entries are expected executable entry points and
are not acceptance evidence.

The first remediation matrix attempt stopped at `cargo fmt --all -- --check`
because one new public UNC assertion required rustfmt line wrapping. `cargo
fmt --all` applied only that mechanical formatting and the complete matrix was
rerun from the beginning. An earlier focused strict-Clippy run rejected the
initial direct in-future readers as large futures and one test duration style;
the readers were boxed without spawning, the duration was corrected, and the
same strict-Clippy command then passed. Neither failed attempt is acceptance
evidence.

## Focused Rust evidence

The following commands were executed sequentially from the repository root.
All exited zero.

| Command or exact suite | Tests passed | Failed / ignored |
| --- | ---: | --- |
| `cargo test -p oneagent-runtime --lib` | 119 | 0 / 0 |
| `cargo test -p oneagent-runtime --test repository_change_domain` | 6 | 0 / 0 |
| `cargo test -p oneagent-runtime --test git_change_reader` | 7 | 0 / 0 |
| `cargo test -p oneagent-runtime --test git_change_workspace` | 2 | 0 / 0 |
| `cargo test -p oneagent-runtime --test workspace_service` | 6 | 0 / 0 |
| `cargo test -p oneagent-runtime --test file_watching` | 2 | 0 / 0 |
| `cargo test -p oneagent-runtime --test persistent_cache` | 4 | 0 / 0 |
| `cargo test -p oneagent-runtime --test graph_query_api` | 3 | 0 / 0 |
| `cargo test -p oneagent-graph --test validation --test report --test build_diff --test reference_request_build --test coverage` | 86 | 0 / 0 |
| `cargo test -p oneagent-analysis` | 117 | 0 / 0 |
| `cargo test -p oneagent-protocol` | 53 | 0 / 0 |
| `cargo test -p oneagent-tool-policy` | 33 | 0 / 0 |
| `cargo test -p oneagent-runtime --test mcp_semantic_tools` | 7 | 0 / 0 |
| `cargo test -p oneagent-runtime --test mcp_stdio` | 8 | 0 / 0 |
| `cargo test -p oneagent-runtime --test mcp_process` | 17 | 0 / 0 |
| `cargo test -p oneagent-runtime --test lsp_stdio` | 5 | 0 / 0 |
| `cargo test -p oneagent-runtime --test lsp_process` | 8 | 0 / 0 |
| `cargo test -p oneagent-runtime --test http_health` | 4 | 0 / 0 |
| `cargo test -p oneagent-cli --test runtime_client` | 2 | 0 / 0 |

Runtime's 119 unit tests include 9 normalized-domain tests, 8 reader and
production-process-boundary tests, 5 complete-file observation tests, 3 explicit-input tests, and the
existing cache, lifecycle, protocol-projection, Graph Query, diagnostics, and
rules coverage. The Graph total is validation (55), report (3), build diff
(2), reference-request build (7), and Coverage (19). Analysis is library (71)
plus 46 public tests. Protocol is library (7), LSP domain (12), MCP dispatch
(6), MCP domain (15), and MCP session (13). Tool Policy is library (26) plus
conformance (7).

## Production reader and platform evidence

Local production-entry tests used Git `2.50.1 (Apple Git-155)` and disposable
temporary repositories only. They proved SHA-1 and locally supported SHA-256
baselines, normal/detached/linked/nested exact roots, staged and unstaged
folding, additions, modifications, deletions, type change, untracked and
ignored policy, move/copy decomposition, conflict, gitlink rejection, operation
order, and fresh repeated reads.

Exact Task 5 head `e2818c83646aded18f485b3fbe0b47ce80a0261a`
passed [CI run 33308295359](https://github.com/MakSGTS/OneAgent/actions/runs/33308295359).
All six jobs completed successfully: Rust, VS Code, and EDT on both `macos-14`
and `windows-latest`. The Rust jobs built both public Runtime processes and ran
the complete workspace tests, focused Context/MCP compatibility, Clippy, and
Rustdoc. The Windows reader target executed its six applicable public tests;
the Unix-only type-change/non-UTF-8 test executed on macOS.

That Task 5 run is historical platform evidence and is not presented as
validation of the later cleanup remediation. The fresh integration review must
resolve and authenticate exact remediation-head macOS/Windows CI before any
Sprint 38 state transition.

The local APFS environment rejected construction of the deliberately invalid
non-UTF-8 filename with `Operation not permitted`. The test reported that
environment limitation and still proved the real regular-file-to-symlink type
change. Unsupported path bytes are covered independently by the injected
production parser and closed-error test. This evidence does not claim that a
real non-UTF-8 repository path was constructed locally or on Windows.

## Public product compatibility

The unchanged VS Code consumer was checked locally with the repository-owned
pinned VS Code 1.134.0 runtime in Node mode. Production and test TypeScript
type checks and compilation passed, followed by 62 unit tests and 2 public
`oneagent-mcp` process tests with zero failures, cancellations, skips, or todos.
The Node-mode commands emitted non-fatal macOS Electron code-sign inspection
warnings and still exited zero.

From `extensions/vscode`, the local commands were:

```bash
REPOSITORY="$(git rev-parse --show-toplevel)"
ELECTRON_RUN_AS_NODE=1 ./.vscode-test/review-node-bin/node \
  ./node_modules/typescript/bin/tsc -p tsconfig.json --noEmit
ELECTRON_RUN_AS_NODE=1 ./.vscode-test/review-node-bin/node \
  ./node_modules/typescript/bin/tsc -p tsconfig.test.json --noEmit
ELECTRON_RUN_AS_NODE=1 ./.vscode-test/review-node-bin/node \
  ./node_modules/typescript/bin/tsc -p tsconfig.json
ELECTRON_RUN_AS_NODE=1 ./.vscode-test/review-node-bin/node \
  ./node_modules/typescript/bin/tsc -p tsconfig.test.json
ELECTRON_RUN_AS_NODE=1 ./.vscode-test/review-node-bin/node \
  --test dist-test/test/unit/*.test.js
ONEAGENT_MCP_BIN="$REPOSITORY/target/debug/oneagent-mcp" \
ELECTRON_RUN_AS_NODE=1 ./.vscode-test/review-node-bin/node \
  --test dist-test/test/integration/*.test.js
```

Exact-head macOS and Windows VS Code CI also passed typecheck, the same 62 unit
tests, 18 Extension Host scenarios, 2 real-process tests, the 12-file package
inventory, two equal 14-file VSIX builds, and the scope/dependency audit of 43
tracked extension files, 18 license groups, and 3 documents.

Local EDT launch was unnecessary because the exact-head jobs supply the
repository-required Temurin JDK 25 and supported host boundary. Both macOS and
Windows reported `BUILD SUCCESS`; all 41 tests passed with zero failures,
errors, or skips, including the real Runtime process, and both p2 package audits
confirmed the same totals. No VS Code or EDT source, manifest, lockfile,
capability, package, or command changed in Sprint 38.

## Canonical gate and inventory

The accepted remediation cycle is:

| Command | Exact outcome |
| --- | --- |
| `cargo fmt --all -- --check` | exit 0 |
| `cargo check --workspace --all-targets` | exit 0 |
| `cargo test --workspace --all-targets` | exit 0; 80 test targets, 1,265 passed, 0 failed/ignored/measured/filtered |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | exit 0 |
| `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps` | exit 0 |
| `git diff --check` | exit 0 |

The four zero-test targets are expected binary entry points:
`oneagent-cli`, `oneagent-runtime`, `oneagent-mcp`, and `oneagent-lsp`. The
other 76 targets contain all 1,265 tests. The inventory was recomputed from the
compiled `--all-targets` executables using `--list --format terse`; no filtered
test is included in that total.

## API, dependency, executable, and sensitive-data audits

- Sprint 38 adds only additive public Runtime library types: validated commit,
  endpoint, completeness, path, change, change-set, reader, error, limit,
  submission-handle, and submission-outcome values. Existing public items and
  signatures remain available and no consumer requires migration.
- No Cargo manifest, `Cargo.lock`, production dependency, feature flag,
  third-party package, native library, license inventory, or workspace member
  changes. The workspace continues to forbid unsafe Rust.
- The only new production executable assumption is a `git` executable on the
  process path when a caller explicitly invokes `GitRepositoryReader`.
  Capability is validated by closed command outcomes; ordinary Runtime startup
  and every existing consumer remain independent from Git availability.
- The production adapter invokes no shell and only fixed `rev-parse`,
  `ls-files`, and `diff` operations. It has no fetch, pull, push, clone,
  checkout, branch, worktree-management, staging, commit, merge, rebase, reset,
  clean, hook, remote, or credential operation.
- Tests use only repository-owned fixtures copied into `tempfile` directories.
  They set local commit identity and signing policy and do not require ambient
  user repositories, global identity, credentials, network, or a remote.
- Cache schema remains `1` and semantic compatibility remains `4`. Git
  baseline, endpoint, completeness, paths, statuses, errors, process data, and
  queued requests do not enter cache bytes or snapshot equality.
- MCP retains exactly seven tools and all existing read-only Tool Policy
  declarations. LSP retains UTF-16 positions, no document synchronization,
  workspace symbols, and pull diagnostics. HTTP routes and CLI commands are
  unchanged. No wire schema or capability advertises Git.
- Coverage Registry files and counts are unchanged. Git evidence creates no
  semantic fact and therefore transitions no capability.
- Public errors and `Debug` output are closed and bounded. Source and diff
  scans found no credential, token, generated artifact, absolute/personal path,
  source content, raw Git output, branch/ref, repository configuration,
  environment value, or internal chain added to public snapshots, cache,
  protocols, or docs.
- The Task 3-5 implementation commits contain only the Runtime
  domain/reader/input sources and their tests. They contain no generated binary,
  cache, packaged client, editor artifact, or unrelated source change.

The exact read-only audit commands all exited zero:

```text
git diff --name-only 095ac719..e2818c83 -- Cargo.toml Cargo.lock apps/runtime/Cargo.toml
git diff --name-only 095ac719..e2818c83 -- crates/graph/src/coverage.rs adapters/edt/src/coverage.rs adapters/designer-xml/src/coverage.rs
git diff --name-only 095ac719..e2818c83 -- crates/protocol apps/cli extensions/vscode extensions/edt apps/runtime/src/http apps/runtime/src/mcp.rs apps/runtime/src/mcp_tools.rs apps/runtime/src/lsp.rs
! git grep -n unsafe -- apps/runtime/src apps/runtime/tests
! git grep -nE '/Users/|maxim_tomshin|BEGIN [A-Z ]*PRIVATE KEY|ghp_|github_pat_|AKIA' -- apps/runtime/src apps/runtime/tests docs/adr/0060-git-change-adapter.md
rg -n 'fetch|pull|push|clone|checkout|merge|rebase|reset|clean|add|commit|credential' apps/runtime/src/workspace/git.rs
rg -n 'Command::new|\.args\(' apps/runtime/src/workspace/git.rs
gh run view 33308295359 --json status,conclusion,url,jobs
```

The first three diff audits returned zero changed paths. The unsafe and
sensitive-value searches returned zero matches. The command/scope searches
returned only the fixed reader operations, the credential-helper disablement,
domain prose, and ordinary collection operations; they exposed no forbidden
Git operation. The exact-head CI query returned `completed` / `success` and six
successful jobs.

The repository-owned evidence artifacts are:

- `apps/runtime/src/workspace/repository_change.rs` — normalized domain;
- `apps/runtime/src/workspace/git.rs` — production and injected reader;
- `apps/runtime/src/workspace/mod.rs` — source-neutral Workspace input;
- `apps/runtime/tests/repository_change_domain.rs` — public domain evidence;
- `apps/runtime/tests/git_change_reader.rs` — disposable real-repository
  evidence;
- `apps/runtime/tests/git_change_workspace.rs` — mixed EDT/Designer complete
  rebuild equivalence; and
- this document — acceptance matrix, audits, limitations, and hand-off.

No additional fixture, generated artifact, local log, executable, package,
cache, or CI download is tracked by Task 6.

## Current limitations and Sprint 39 hand-off

The first slice is local and explicit-demand. It does not automatically poll or
discover Git, replace filesystem watching, expose Git through Runtime
configuration or protocols, or support a repository root different from the
Workspace root. System Git is required only for explicit reader calls and no
version is pinned.

The current endpoint folds tracked final-worktree state with non-ignored
untracked files relative to pinned `HEAD`. It does not expose index-only or
separate staged/unstaged endpoints, ignored completeness, empty untracked
directories, sparse-checkout completeness, submodule content, arbitrary
commit/tree comparisons, or multiple repositories. Conflicts and changed
gitlinks fail atomically. Rename/copy similarity is disabled, so moves and
copies do not claim identity.

An accepted non-empty input requests a complete rebuild and may publish an
equal complete result. It does not select a semantic object or implement
incremental Graph mutation. Backpressure is explicit and caller-owned; the
Runtime does not retry. No Git evidence is persisted or published.

Sprint 39 may consume the accepted complete repository evidence only through a
separately accepted product-facing Change Impact Analysis workflow. It must
derive semantic change from complete previous/current graphs and their
canonical diff rather than from repository paths or statuses. Refactoring,
source edits, transactions, rollback, remote Git, credentials, repository
mutation, protocol/IDE Git UI, telemetry, benchmarks, and broad performance or
security claims remain deferred.
