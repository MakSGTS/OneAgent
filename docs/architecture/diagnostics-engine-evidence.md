# Sprint 36 Diagnostics Engine Evidence

## Status and scope

This document records Task 7 evidence executed on 2026-08-29 from committed
Task 6 head `49e0c647`. Sprint 36 remains `active` until the mandatory Task 8
fresh-context integration review, primary reconciliation, and artifact-
consistency check complete. Task 7 changes documentation only and introduces no
production behavior.

The implemented boundary is governed by
[ADR-0058](../adr/0058-diagnostics-engine.md). Graph remains the authority for
recoverable semantic diagnostics, validation, facts, provenance, locations,
reports, and build diffs. `oneagent-analysis::diagnostics` owns only immutable
normalization, exact suppression, deterministic ordering, checked summaries,
and the complete bounded `DiagnosticReport`. Runtime composes that report into
each immutable Configuration snapshot before publication. MCP and LSP project
the same snapshot result without rerunning validation or reading source.

The committed implementation chain is:

| Slice | Commit |
| --- | --- |
| Planning | `cc890879` |
| Investigation | `7fa34274` |
| Reusable task framework | `3b28dcdd` |
| ADR-0058 | `7ceec272` |
| Diagnostic domain | `1cab3293` |
| Orchestration | `0b0c5d60` |
| Workspace/cache snapshots | `4f9ffeeb` |
| MCP/LSP reporting | `49e0c647` |

## Requirement-to-test matrix

| ADR-0058 requirement | Repository-owned evidence | Result |
| --- | --- | --- |
| Exactly two canonical immutable input families and caller-supplied validation | `crates/analysis/src/diagnostics/engine.rs`; Analysis engine unit and public integration tests | pass |
| Graph and producer authority; no parsing, source read, graph mutation, or validator invocation in the engine | dependency/source audit; engine API accepts `&[SemanticDiagnostic]` and `&SemanticGraphValidationResult` | pass |
| Closed family, severity, category, code, kind, disposition, and typed identity vocabularies | `crates/analysis/src/diagnostics/mod.rs` exhaustive vocabulary tests | pass |
| Semantic and validation identity fields remain exact and family tagged | public identity tests plus cross-family mixed-report tests | pass |
| Exact duplicates collapse and same-identity/different-content fails closed independently of input order | report/domain and engine conflict/reorder tests | pass |
| Active before suppressed, Error before Warning, then category/family/identity/content order | report ordering and repeated/reordered engine tests | pass |
| Exact in-memory identity suppression only; default policy suppresses nothing and retained counters reconcile | domain, engine, MCP projection, and LSP active/suppressed unit tests | pass |
| Complete non-truncating engine with exact/one-over input, finding, suppression, message, anchor, and provenance bounds | domain/engine bound tests and Workspace atomic publication boundary test | pass |
| Closed redacted errors with no rejected content, identity, path, reference, provenance, or internal chain | exhaustive error-kind/debug/display tests and protocol sensitive-data audit | pass |
| Checked complete summary and read-only filtering without summary reconstruction | report reconciliation tests and MCP filtered/truncated summary tests | pass |
| Workspace publishes raw diagnostics, exact complete validation, and equal complete report atomically | Runtime unit, Workspace service, and watching tests | pass |
| Cache schema and canonical evidence fields remain unchanged, semantic compatibility advances from version 2 to 3, and derived validation/report are recomputed equally after decode | cache unit and public cold/warm/corruption/write-recovery tests | pass |
| Seven lexicographically ordered read-only MCP tools and unchanged Tool Policy execution | semantic catalog/schema tests, direct denied-policy unit test, protocol dispatch, stdio, and process suites | pass |
| `oneagent.diagnostics` schema, filters, default suppression visibility, normalized fields, complete summary, limit, ordering, and redaction | MCP unit, semantic-tool, exact/over-bound, malformed-argument, repetition, and public-process tests | pass |
| Stateless `2026-07-28` and negotiated `2025-06-18`/`2025-11-25` envelopes expose the same tool payload | modern semantic-tool suite, protocol session suite, and public two-revision payload equality test | pass |
| LSP 3.17 pull-only full reports project active findings with exactly one confined node span and preserve code/severity/message | Runtime LSP unit and public process tests | pass |
| Suppressed, missing, multi-anchor, conflicting, span-less, escaping, incompatible, and different-document evidence is omitted rather than guessed | LSP location/confinement unit matrix and public URI negative matrix | pass |
| LSP complete-result bound accepts 100 and returns `RequestFailed` for 101 without a prefix | unit boundary test and generated public-process exact/one-over workspace test | pass |
| MCP/LSP lifecycle, malformed input, EOF, channel purity, exit, repetition, cancellation, and cleanup remain unchanged | protocol, MCP/LSP stdio, and MCP/LSP public-process suites | pass |
| Graph reports/diffs/validation, adapters, HTTP, CLI, VS Code, EDT, and Coverage remain compatible | focused Graph/Runtime tests, complete Rust gate, VS Code compile/unit/process suite, EDT Tycho/PDE suite, and unchanged Coverage registry audit | pass |
| No Rules Engine, configurable suppression, new producer, UI, mutable document, fix/edit, remote transport, telemetry, or performance/security claim | production diff and scope audit | pass |

No required row used a zero-match test filter. No required row was skipped.

## Focused Rust evidence

The following commands were executed sequentially from the repository root.
All exited zero.

| Command or exact suite | Tests passed | Failed / ignored |
| --- | ---: | --- |
| `cargo test -p oneagent-analysis diagnostics::` | 25 | 0 / 0 |
| `cargo test -p oneagent-analysis --test diagnostics_engine` | 3 | 0 / 0 |
| Graph validation/report/build-diff/reference-request/Coverage integration suites | 86 | 0 / 0 |
| `cargo test -p oneagent-runtime --lib` | 95 | 0 / 0 |
| `cargo test -p oneagent-runtime --test workspace_service` | 6 | 0 / 0 |
| `cargo test -p oneagent-runtime --test persistent_cache` | 4 | 0 / 0 |
| `cargo test -p oneagent-runtime --test file_watching` | 2 | 0 / 0 |
| `cargo test -p oneagent-protocol` | 53 | 0 / 0 |
| `cargo test -p oneagent-tool-policy` | 33 | 0 / 0 |
| `cargo test -p oneagent-runtime --test mcp_semantic_tools` | 7 | 0 / 0 |
| `cargo test -p oneagent-runtime --test mcp_stdio` | 8 | 0 / 0 |
| `cargo test -p oneagent-runtime --test mcp_process` | 17 | 0 / 0 |
| `cargo test -p oneagent-protocol --test lsp_domain` | 12 | 0 / 0 |
| `cargo test -p oneagent-runtime --test lsp_stdio` | 5 | 0 / 0 |
| `cargo test -p oneagent-runtime --test lsp_process` | 8 | 0 / 0 |
| `cargo test -p oneagent-runtime --test http_health` | 4 | 0 / 0 |
| `cargo test -p oneagent-cli --test runtime_client` | 2 | 0 / 0 |

The Graph integration total is the exact sum of `validation` (55), `report`
(3), `build_diff` (2), `reference_request_build` (7), and `coverage` (19).
The protocol total is the exact sum of library (7), LSP domain (12), MCP
dispatch (6), MCP domain (15), and MCP session (13). Tool Policy is library
(26) plus conformance (7).

## Public product compatibility

The existing VS Code consumer does not call `oneagent.diagnostics`, and no
extension file changed in Sprint 36. Current repository-owned compatibility was
nevertheless rerun through the repository-local pinned VS Code 1.134.0 runtime:

- production and test TypeScript type checks passed;
- production and test TypeScript compilation passed;
- 62 unit tests passed with zero failures, cancellations, skips, or todos;
- 2 public `oneagent-mcp` process tests passed with every negative counter zero.

The first convenience `pnpm run typecheck` attempt did not reach TypeScript
because this environment had no `node` command in `PATH`. It changed no file
and is not acceptance evidence. The accepted commands invoked the repository-
local pinned runtime directly in Node mode under the required macOS host-
execution approval.

The unchanged native EDT consumer was rerun sequentially with the current
`oneagent-mcp` binary and mixed Workspace fixture:

```bash
ONEAGENT_MCP_EXECUTABLE="$ONEAGENT_REPOSITORY/target/debug/oneagent-mcp" \
ONEAGENT_MCP_FIXTURE="$ONEAGENT_REPOSITORY/apps/runtime/tests/fixtures/workspace_service" \
  ./mvnw --batch-mode --no-transfer-progress clean verify
```

The Tycho reactor completed with `BUILD SUCCESS`; all 41 PDE tests passed with
zero failures, errors, or skips, and the public Runtime probe ran twice. The
known platform-shutdown warning about an Eclipse URI-scheme job remained non-
fatal. Generated TypeScript, Tycho, PDE, feature, and p2 outputs are ignored and
left no tracked change.

ADR-0058 does not require another Codex or Cursor executable run when
repository-owned evidence finds no incompatibility. Their negotiated revisions,
catalog, lifecycle, and payload compatibility are covered by the current
protocol/public-process matrix; the exact prior client evidence remains in the
[Sprint 35 compatibility evidence](external-ai-client-compatibility-evidence.md).
No new external client behavior is claimed.

## Canonical gate and inventory

The complete post-implementation gate passed twice after the only development
failure, a strict Clippy report containing four mechanical warnings. Those
warnings were fixed, the failed command passed, and the complete cycle was
rerun. The accepted final cycle is:

| Command | Exact outcome |
| --- | --- |
| `cargo fmt --all -- --check` | exit 0 |
| `cargo check --workspace --all-targets` | exit 0 |
| `cargo test --workspace --all-targets` | exit 0; 73 test targets, 1,176 passed, 0 failed/ignored/measured/filtered |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | exit 0 |
| `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps` | exit 0 |
| `git diff --check` | exit 0 |

The four zero-test targets are the expected binary entry points
`oneagent-cli`, `oneagent-runtime`, `oneagent-mcp`, and `oneagent-lsp`; none is
an acceptance filter. The 69 non-zero targets contain all 1,176 tests. The
inventory was independently recomputed from the compiled `--all-targets`
executables using their `--list --format terse` output.

## Compatibility, dependency, and sensitive-data audits

- Sprint 36 changes no Cargo manifest, `Cargo.lock`, production dependency,
  third-party package, feature flag, or license inventory. Analysis already
  depended on Graph; Runtime already depended on Analysis.
- Graph public types and producer behavior are unchanged. Raw recoverable
  diagnostics, validation semantics, graph reports, build diffs, and Coverage
  remain available with their existing meanings. Coverage stays at its
  pre-Sprint-36 status and count because the engine adds no fact capability.
- Cache schema remains `1` and serialized canonical evidence fields are
  unchanged. The private semantic compatibility version advances from `2` to
  `3`, so earlier entries are intentionally rejected and cache bytes are not
  claimed to be compatible. Validation/report values are recomputed and are
  not serialized.
- MCP retains exactly seven lexicographically ordered tools, read-only
  annotations, Tool Policy authorization/execution, immutable startup state,
  request isolation, limits, and revision envelopes. Other tool payloads remain
  compatible; Graph summary and `oneagent.validation` now consume the same
  published validation result rather than rerunning a validator.
- LSP retains its exact 3.17 capability object, UTF-16 positions, pull-only
  full reports, no result ID, no synchronization, and a complete limit of 100.
- MCP findings expose no root, path, source content, raw reference, producer,
  opaque provenance, hash, credential, rejected input, or internal chain. LSP
  emits a file URI only after the existing Workspace and Configuration
  confinement checks produce one exact typed span.
- The Sprint diff and tracked files contain no credential, token, generated
  build artifact, personal absolute path, client binary, cache, or raw process
  trace. Build products remain ignored under their established directories.
- No schema or capability advertises Rules Engine registration/execution,
  persisted or configurable suppression, diagnostics UI, push/workspace
  diagnostics, mutable documents, fixes, edits, remote access, authentication,
  telemetry, or unsupported performance/security behavior.

## Current limitations and Sprint 37 hand-off

Production Workspace uses the empty suppression policy, so published reports
normally contain no suppressed findings. Suppression has exact typed in-memory
semantics only; it has no file grammar, patterns, baselines, directives, user
configuration, persistence, or UI. The engine consumes only existing Graph
evidence and cannot invent new findings.

MCP remains a bounded lossy projection with a default limit of 50 and maximum
100, while its summary describes the complete unfiltered report. LSP omits any
finding without exactly one resolvable confined span and fails the complete
request above 100. Neither adapter reads source or infers an alternate
location.

Sprint 37 must independently define deterministic rule registration,
dependencies, execution, configuration, ownership, result production, and
their relationship to the accepted diagnostic identity/report boundary. This
evidence does not preselect or implement that architecture.
