# Sprint 37 Rules Engine Evidence

## Status and scope

This document records Task 7 evidence executed on 2026-08-30 from committed
Task 6 head `ca054770`. Sprint 37 remains active until Task 8 completes the
fresh-context independent review, primary reconciliation, artifact-consistency
check, Sprint 38 hand-off, and conditional Sprint 36 prompt-suite retirement.
Task 7 changes documentation only and introduces no production behavior.

The implemented boundary is governed by
[ADR-0059](../adr/0059-rules-engine.md). Graph remains authoritative for facts,
validation, recoverable semantic diagnostics, provenance, locations, reports,
and diffs. The Diagnostics Engine remains authoritative for diagnostic
identity, normalization, exact suppression, ordering, checked summaries, and
complete bounded reports. `oneagent-analysis::rules` owns only validated rule
identity, registration, in-memory enable/disable configuration, deterministic
dependency planning, synchronous sequential execution, cooperative
cancellation, terminal results, and rule-produced diagnostic candidates.

Production Runtime uses an empty immutable registry, default configuration,
and `NeverCancelled`. It publishes a complete empty `RuleExecutionReport` with
each immutable Configuration snapshot and makes no claim that a product rule
exists. Repository-owned conformance rules exercise the non-empty boundary in
tests only.

The committed implementation chain is:

| Slice | Commit |
| --- | --- |
| Reusable task framework | `68045f0c` |
| Planning | `3c0222dc` |
| Investigation | `bab5cade` |
| ADR-0059 | `6518c6c1` |
| Rule registry | `f8589f2c` |
| Rule planning | `241b8350` |
| Rule execution and diagnostics | `84313b6e` |
| Workspace/cache/protocol composition | `ca054770` |

## Requirement-to-test matrix

| ADR-0059 requirement | Repository-owned evidence | Result |
| --- | --- | --- |
| Canonical immutable inputs are Graph, caller-supplied complete validation, and the base Semantic/Validation diagnostic report | Public context tests and source/dependency audit of `oneagent-analysis::rules` | pass |
| Rules do not read source, parse, mutate Graph, invoke validation, access Runtime/cache/protocol state, or consume another rule result | Object-safe `Rule`/borrowed `RuleContext` API and public context conformance tests | pass |
| Rule IDs and local diagnostic/failure codes use exact validated grammar, byte bounds, equality, order, and redacted errors | 19 rule-domain unit tests plus public registry/execution redaction tests | pass |
| Rule definitions own canonical unique dependency IDs without behavior participation in equality | Domain and public registry identity tests | pass |
| Registry is immutable, bounded to 4,096 inputs, sorted by complete ID, and rejects duplicate/conflicting registrations without selecting by input order | Domain exact/over tests and 5 public registry tests | pass |
| Registration behavior is shared and source independent without comparing executable objects | Public `Arc<dyn Rule>` ownership and identity tests | pass |
| Configuration is in-memory, enable/disable only, default-enabled, bounded to 4,096 settings, canonical, and rejects duplicates/unknown rules | Domain exact/over tests and 6 public planning tests | pass |
| Missing, self, repeated, chain, diamond, cycle, independent, and aggregate dependency limits are deterministic | Domain and public planning topology/failure tests | pass |
| Dependencies mean required successful completion and the smallest complete ready `RuleId` is the total-order tie-breaker | Public planning and execution order tests | pass |
| Disabled rules remain observable; Disabled, NotApplicable, Blocked, Failed, Cancelled, and Completed are distinct terminal states | Public planning and execution outcome tests | pass |
| Execution is synchronous, sequential, one attempt, and independent rules continue after a failure | 15 public execution tests | pass |
| Cancellation is checked before and after evaluation; accepted output is discarded after late cancellation and remaining rules cancel | Public pre-existing/post-evaluation cancellation tests and Runtime-owned cleanup regressions | pass |
| Dependencies block on every non-Completed terminal outcome while independent branches remain executable | Public failure/diamond execution tests | pass |
| Invalid per-rule output fails only that rule; aggregate engine/domain errors return no partial report | Invalid-output, conflict, per-rule and aggregate exact/over public tests | pass |
| Per-rule and aggregate results reconcile exact terminal and diagnostic counts | Empty, mixed-outcome, repetition, and summary tests | pass |
| Rule diagnostics retain rule ID, local code, normalized severity/category/message, canonical Graph anchors, and Graph-derived provenance count | Public execution and diagnostic integration tests | pass |
| Rule diagnostic identity is rule ID, local code, and canonical anchors; exact duplicates collapse and conflicts fail closed | 6 public rule-diagnostic tests | pass |
| Rule evidence shares ADR-0058 suppression, ordering, filtering, summary, bounds, and mixed-family completeness | Public rule-diagnostic and MCP summary/filter tests | pass |
| Messages, anchors, observed provenance, per-rule output, aggregate output, and error detail enforce exact/one-over bounds | Domain, execution, diagnostics, Workspace, MCP, and LSP bound tests | pass |
| Errors are closed, bounded, and do not echo source, path, rule behavior, rejected identity/configuration, provenance, secret, or internal chain | Exhaustive rule error formatting plus protocol and process sensitive-data tests | pass |
| Workspace constructs base diagnostics, plan, execution, and final diagnostics before atomic publication | Runtime composition unit test, Workspace service atomic-failure tests, and immutable snapshot assertions | pass |
| Production empty registry/default configuration yields an equal empty report across repeated, rebuilt, watched, observed, and fresh service runs | Runtime unit, Workspace service, file-watching, and persistent-cache suites | pass |
| Cache schema remains 1; executable/configuration/plan/results are not serialized; semantic compatibility advances from 3 to 4 | Cache codec/version assertions and source/diff audit | pass |
| Version 3 and incompatible future entries invalidate; decode recomputes equal validation, empty rule report, and final diagnostic report | Public cold/warm/invalidation/corruption/write-failure/watched-replacement tests | pass |
| Runtime startup, readiness, watcher generation, replacement, failure recovery, cancellation, shutdown, and cleanup ownership remain unchanged | 99 Runtime unit tests and 12 public Workspace/cache/watching tests | pass |
| MCP keeps exactly seven lexicographic read-only Tool Policy-gated tools and adds no rule-management tool | MCP schema/catalog, Tool Policy, stdio, and process tests | pass |
| `oneagent.diagnostics` accepts family `rule`, projects local code/kind plus Rule-only `ruleId`, and retains complete unfiltered summaries | MCP unit and 7 public semantic-tool tests | pass |
| `ruleId` is absent for Semantic/Validation findings; projections expose no root, path, source, raw reference, producer, opaque provenance, failure code, rejected value, or internal chain | Controlled projection unit test and public serialized-output audit | pass |
| Modern and negotiated legacy MCP revisions retain equal payloads, lifecycle, bounds, malformed-input precedence, EOF, cancellation, and channel purity | 53 Protocol tests plus 8 stdio and 17 public process tests | pass |
| LSP keeps its exact 3.17 capability and wire shape and projects only active one-anchor Rule findings with one confined typed span | Controlled Rule projection unit test, 5 stdio tests, and 8 public process tests | pass |
| Missing, zero, multiple, span-less, escaping, suppressed, incompatible, or different-document evidence is omitted and the complete 100/101 bound remains fail-closed | Existing LSP unit/confinement/bound and generated public-process tests | pass |
| Graph validation/reports/diffs, adapters, HTTP, CLI, VS Code, EDT, and Coverage remain compatible | 86 focused Graph tests, complete Rust gate, exact-head CI consumer matrix, and unchanged Coverage audit | pass |
| No external rule configuration, dynamic plugin/script/remote rule, rule-management UI/protocol, mutable document, fix/edit, telemetry, or performance/security claim is introduced | Production diff, tracked-file, schema/capability, dependency, and scope audits | pass |

No required matrix row uses a zero-match filter and no required row is skipped.
The initial `cargo test -p oneagent-analysis rules::` command also invoked six
integration binaries with zero matching tests because Cargo propagated the
module filter. Those zero matches are not evidence; all four required public
rule integration targets were run separately without a filter.

## Focused Rust evidence

The following commands were executed sequentially from the repository root.
All exited zero.

| Command or exact suite | Tests passed | Failed / ignored |
| --- | ---: | --- |
| `cargo test -p oneagent-analysis rules::` library target | 19 | 0 / 0 |
| `cargo test -p oneagent-analysis --test rule_registry` | 5 | 0 / 0 |
| `cargo test -p oneagent-analysis --test rule_planning` | 6 | 0 / 0 |
| `cargo test -p oneagent-analysis --test rule_execution` | 15 | 0 / 0 |
| `cargo test -p oneagent-analysis --test rule_diagnostics` | 6 | 0 / 0 |
| `cargo test -p oneagent-graph --test validation --test report --test build_diff --test reference_request_build --test coverage` | 86 | 0 / 0 |
| `cargo test -p oneagent-runtime --lib` | 99 | 0 / 0 |
| `cargo test -p oneagent-runtime --test workspace_service` | 6 | 0 / 0 |
| `cargo test -p oneagent-runtime --test persistent_cache` | 4 | 0 / 0 |
| `cargo test -p oneagent-runtime --test file_watching` | 2 | 0 / 0 |
| `cargo test -p oneagent-protocol` | 53 | 0 / 0 |
| `cargo test -p oneagent-tool-policy` | 33 | 0 / 0 |
| `cargo test -p oneagent-runtime --test mcp_semantic_tools` | 7 | 0 / 0 |
| `cargo test -p oneagent-runtime --test mcp_stdio` | 8 | 0 / 0 |
| `cargo test -p oneagent-runtime --test mcp_process` | 17 | 0 / 0 |
| `cargo test -p oneagent-runtime --test lsp_stdio` | 5 | 0 / 0 |
| `cargo test -p oneagent-runtime --test lsp_process` | 8 | 0 / 0 |
| `cargo test -p oneagent-runtime --test http_health` | 4 | 0 / 0 |
| `cargo test -p oneagent-cli --test runtime_client` | 2 | 0 / 0 |

The Graph total is the exact sum of `validation` (55), `report` (3),
`build_diff` (2), `reference_request_build` (7), and `coverage` (19). Protocol
is library (7), LSP domain (12), MCP dispatch (6), MCP domain (15), and MCP
session (13). Tool Policy is library (26) plus conformance (7).

## Public product compatibility

The unchanged VS Code consumer was checked locally with the repository-owned
pinned VS Code 1.134.0 runtime in Node mode. Production and test TypeScript
type checks and compilation passed, followed by 62 unit tests and 2 public
`oneagent-mcp` process tests with zero failures, cancellations, skips, or todos.
From `extensions/vscode`, the exact commands used
`ELECTRON_RUN_AS_NODE=1 ./.vscode-test/review-node-bin/node` to invoke both
TypeScript projects with and without `--noEmit`, then:

```bash
REPOSITORY="$(git rev-parse --show-toplevel)"
ELECTRON_RUN_AS_NODE=1 ./.vscode-test/review-node-bin/node \
  --test dist-test/test/unit/*.test.js
ONEAGENT_MCP_BIN="$REPOSITORY/target/debug/oneagent-mcp" \
ELECTRON_RUN_AS_NODE=1 ./.vscode-test/review-node-bin/node \
  --test dist-test/test/integration/*.test.js
```

Exact Task 6 head `ca054770613f5b3feb6af8b52cb4ec3527ca02a5`
also passed [CI run 33299869526](https://github.com/MakSGTS/OneAgent/actions/runs/33299869526)
on macOS and Windows. Both VS Code jobs passed typecheck, the same 62 unit
tests, 18 Extension Host scenarios, 2 real-process tests, the 12-file package
inventory, two equal 14-file VSIX builds, and the scope/dependency audit.

The first local EDT Maven attempt used the ambient Java 17 runtime and stopped
before tests because Tycho 5.0.2 requires newer class-file support. It changed
no tracked file and is not acceptance evidence. No external JDK path was
guessed or inspected. From `extensions/edt`, the failed command was:

```bash
REPOSITORY="$(git rev-parse --show-toplevel)"
ONEAGENT_MCP_EXECUTABLE="$REPOSITORY/target/debug/oneagent-mcp" \
ONEAGENT_MCP_FIXTURE="$REPOSITORY/apps/runtime/tests/fixtures/workspace_service" \
  ./mvnw --batch-mode --no-transfer-progress clean verify
```

The exact-head macOS and Windows CI jobs supplied the
repository-required Temurin JDK 25 and both passed host-boundary validation,
Runtime build, Tycho/PDE/real-process verification, and p2 package audit. The
macOS reactor reported `BUILD SUCCESS`; all 41 tests passed with zero failures,
errors, or skips, and the package auditor confirmed the same totals.

ADR-0059 changes no VS Code or EDT source and advertises no new IDE capability.
The exact-head cross-platform results prove current consumer compatibility
without claiming a rule-management or diagnostics UI.

## Canonical gate and inventory

The accepted Task 7 cycle is:

| Command | Exact outcome |
| --- | --- |
| `cargo fmt --all -- --check` | exit 0 |
| `cargo check --workspace --all-targets` | exit 0 |
| `cargo test --workspace --all-targets` | exit 0; 77 test targets, 1,231 passed, 0 failed/ignored/measured/filtered |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | exit 0 |
| `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps` | exit 0 |
| `git diff --check` | exit 0 |

The four zero-test targets are expected binary entry points: `oneagent-cli`,
`oneagent-runtime`, `oneagent-mcp`, and `oneagent-lsp`. The other 73 targets
contain all 1,231 tests. The inventory was recomputed from the compiled
`--all-targets` executables using `--list --format terse`; no filtered test is
included in that total.

## API, dependency, compatibility, and sensitive-data audits

- Sprint 37 adds public typed rule identity, definition, registry,
  configuration, plan, context, execution, result, diagnostic evidence, bounds,
  and errors in `oneagent-analysis`, plus the additive read-only Runtime
  `rule_execution_report()` accessor. Existing APIs remain available; no
  consumer requires migration.
- No Cargo manifest, `Cargo.lock`, production dependency, feature flag,
  third-party package, or license inventory changes. Analysis already depends
  on Graph and Runtime already depends on Analysis.
- Graph public types, validation, reports, diffs, producer behavior, adapters,
  and Coverage are unchanged. The empty production registry emits no semantic
  fact and causes no Coverage transition.
- Cache schema remains `1`; canonical serialized evidence fields are unchanged.
  Private semantic compatibility advances from `3` to `4`, intentionally
  invalidating version 3. Rule objects, configuration, plans, reports, and Rule
  findings are recomputed rather than serialized.
- MCP retains exactly seven lexicographically ordered tools, read-only
  annotations, Tool Policy, request isolation, limits, and all three revision
  envelopes. The diagnostic family vocabulary adds `rule`; only Rule items may
  add `ruleId`. No catalog or management surface is added.
- LSP retains its exact 3.17 capability object, UTF-16 positions, pull-only full
  reports, no result ID, no synchronization, and the complete limit of 100.
- Public Rule/MCP/LSP errors and values expose no credential, environment,
  cache bytes, filesystem root, absolute/personal path, source content, raw
  reference, producer, opaque provenance, rejected configuration, executable
  behavior, internal chain, or raw process trace.
- The Sprint diff and tracked files contain no generated artifact, client
  binary, cache, credential, token, or personal absolute path. Repository-local
  build products remain ignored under established directories.
- No external configuration grammar, plugin loading, scripting, remote rule,
  hot reload, mutable registration, rule SDK, rule-management protocol/UI,
  mutable-document analysis, automatic fix, code action, source edit, telemetry,
  or performance/security claim is implemented or advertised.

## Current limitations and Sprint 38 hand-off

Production has no product rule. The empty registry and default configuration
exercise the complete composition boundary but always publish an empty rule
execution report and no Rule findings. Non-empty conformance rules exist only
in repository tests. There is no external configuration, persistence,
discovery, plugin, script, remote rule, hot reload, UI, protocol management,
fix, or edit surface.

Rules are trusted synchronous in-process components. Cooperative cancellation
is observed at engine checkpoints; it cannot preempt a non-returning or
panicking body. Runtime uses the existing blocking-build containment boundary,
and production uses `NeverCancelled` for the empty registry. The engine does not
parallelize rules or expose intermediate results.

MCP can truthfully filter the empty Rule family and can project future accepted
Rule findings through the existing bounded diagnostic tool. LSP can project
only active single-anchor findings with one confined span and does not expose
rule identity or family. Neither adapter offers rule management.

Sprint 38 Git Change Adapter must remain an input adapter rather than a rule,
Graph, validation, diagnostic, Runtime, or protocol authority. Task 8 must
independently review the exact Sprint 37 range and evidence before it may mark
Sprint 37 completed, hand off Sprint 38, or retire the Sprint 36 prompt suite.
