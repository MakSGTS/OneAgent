# Sprint 22 Context Engine Integration Review

## Decision

`pass`

Sprint 22 satisfies ADR-0044 and the Roadmap completion gate. No blocking or
non-blocking findings and no missing acceptance evidence remain. Sprint 23 LLM
Provider Abstraction is the unique next planning target.

## Reviewed baseline

- Release parent: `b47e6ff493a5db2b1188761bc7b7bab362e511ca`
- Reviewed range: `b364805a^..fdc190da`
- Committed Task 6 head: `fdc190da7a3560c194fed33aee8979afd8cbdcde`
- Review date: 2026-08-22

| Commit | Subject | Owned paths |
| --- | --- | --- |
| `b364805a857d076ab8e7a42e435d3d2c0017d687` | `Add Context Engine task framework` | `docs/Roadmap.md`, `docs/codex/README.md`, and the Context Engine profile/workflow/template |
| `7c58f3d559fc23db3539c7e7c1f2606cc51b910b` | `Plan Sprint 22 Context Engine` | `docs/Roadmap.md` and the eight-file Sprint 22 prompt suite |
| `b635d06e6f92c239f218cd68d606675dd2c7eb36` | `Investigate Sprint 22 Context Engine` | `docs/architecture/context-engine-investigation.md` |
| `eb7a25f36dcc2bda5bf8d05be80e67dc6b02cc32` | `Define Sprint 22 Context Engine contract` | `docs/adr/0044-context-engine.md` |
| `b19a51bce84c0271b7a616076cdd298a50208079` | `Implement Sprint 22 context request boundary` | `crates/analysis/src/context/mod.rs`, `crates/analysis/src/lib.rs` |
| `089806f680f7776088004322e877fb5345d8042d` | `Implement Sprint 22 deterministic context selection` | `crates/analysis/src/context/mod.rs`, `crates/analysis/src/context/selection.rs` |
| `42a5522e9f91d1ebc9304b75ba540c87c678a6f2` | `Implement Sprint 22 budgeted context assembly` | `crates/analysis/src/context/mod.rs`, `crates/analysis/src/context/assembly.rs` |
| `fdc190da7a3560c194fed33aee8979afd8cbdcde` | `Complete Sprint 22 Context Engine evidence` | `crates/analysis/tests/context_engine.rs`, `README.md`, `docs/Architecture.md`, `docs/architecture/semantic-model-2.md` |

The range changes only the reusable Context Engine framework and planning,
investigation and ADR evidence, additive `oneagent-analysis` Context domain and
implementation, public evaluation, and current-state documentation. It changes
no manifest, lockfile, graph implementation, Runtime, CLI, protocol, adapter,
Coverage state, or source-ingestion behavior.

## Acceptance evidence matrix

| Criterion | Evidence | Result |
| --- | --- | --- |
| Planning readiness | The committed framework audit closes the reusable task-contract gap; the plan orders eight prerequisite-gated prompts and preserves the exact Sprint 21 retirement gate. | pass |
| Investigation | Repository-backed evidence inventories graph/query/provenance, analysis, consumers, dependencies, fixtures, platform behavior, missing source-range/text facilities, and decision questions before architecture selection. | pass |
| Accepted architecture | ADR-0044 fixes ownership, request, seeds, policy, validation precedence, selection, relevance, provenance, rendering, budget, errors, evaluation, compatibility, and deferred scope. | pass |
| Canonical data boundary | Every request borrows one immutable `SemanticGraph`; the engine uses read-only query facts, retains owned projections, and neither mutates nor retains the graph. | pass |
| Public request types | `Explain`, node-ID and exact-name seeds with optional kind constraints, closed direction/filter policy, byte budget, selection, item, bundle, reason, path, and typed error projections are additive and documented. | pass |
| Validation and seed resolution | Budget, policy, raw seed, identifier, exact resolution, kind compatibility, deduplication, ordering, and unique-seed/candidate compatibility follow the accepted all-or-nothing precedence. | pass |
| Failures and precedence | Invalid budget/policy/sets/count/identifier, missing, ambiguous, incompatible, too-many-seed, insufficient-budget, and checked-accounting outcomes are typed, stable, and return no partial bundle. | pass |
| Traversal and filtering | Outgoing, incoming, and both directions; all eleven accepted edge kinds; optional node kinds; depth zero and four; and empty neighborhoods are exercised. | pass |
| Relevance and ties | Path length, per-step edge priority, outgoing-before-incoming, stable edge ID, seed ID, and final candidate ID produce one explicit deterministic order without learned or hidden scores. | pass |
| Bounds, cycles, and deduplication | Depth and candidate bounds are strict; cycles terminate; duplicate seeds, paths, and provenance collapse canonically; exact eligible omission is computed before truncation. | pass |
| Candidate order | Mandatory seeds precede related nodes; best-path order and final stable identity remain equal across reversed graph and seed insertion and fresh repetition. | pass |
| Costs and budget accounting | Complete fragment UTF-8 length is the only item cost; checked sums, exact used/remaining bytes, minimum/maximum validation, ASCII, and non-ASCII boundaries are covered. | pass |
| Admission, omission, and truncation | All seeds must fit or fail atomically; related items use whole-fragment prefix admission; candidate and budget omission flags/counts remain distinct; no partial text is emitted. | pass |
| Bundle identity and order | The owned bundle retains intent, canonical seeds, ordered admitted items, requested budget, exact accounting, omissions, and byte-for-byte concatenated rendering. | pass |
| Provenance | Node and chosen-edge provenance sort/deduplicate by the accepted tuple without modifying original graph vectors; absent provenance remains representable. | pass |
| Explanations | Every item retains one typed seed/related reason, selected seed, depth, and the complete chosen direction/kind/edge-ID/provenance path. | pass |
| Rendering | Closed exhaustive node/edge/direction vocabularies and exact two-line length-prefixed ASCII/non-ASCII string oracles match ADR-0044 without debug formatting or source fabrication. | pass |
| Public evaluation corpus and oracle | Eleven public integration tests use only exported analysis/common/graph surfaces, constructed Rust graphs, fixed strings, and the production analysis pipeline; no filesystem corpus, network, service, sleep, or external data is required. | pass |
| Reordered and repeated equality | Unit and public targets compare reversed graph/provenance/seed inputs and repeated fresh engine calls for complete owned-result equality. | pass |
| Dependency impact | `cargo tree -p oneagent-analysis --edges normal` is unchanged and contains only the existing BSL, common, and graph dependencies; manifests and `Cargo.lock` are unchanged. | pass |
| Graph and analysis compatibility | Existing production `SemanticAnalysisPipeline` facts feed Context directly; analysis and affected graph-query tests plus the complete workspace suite pass unchanged. | pass |
| Platforms | Review validation passed on `aarch64-apple-darwin`; implementation uses portable synchronous Rust collections/strings and repository CI retains macOS 14 and Windows targets. | pass |
| Documentation truth | README, Architecture, Semantic Model, Roadmap plan, investigation, ADR-0044, public API docs, and evaluation agree on the implemented source-independent first slice. | pass |
| Scope containment | No source extraction, tokenizer, provider/model, embedding, graph mutation, persistence, Runtime/HTTP/CLI/protocol, MCP, IDE, performance, quality, or security claim entered the implementation. | pass |

## Findings

No blocking or non-blocking findings.

## Missing evidence

None.

The review counted only non-zero focused targets: 8 request tests, 7 selection
tests, 8 assembly tests, 11 public Context Engine tests, and the complete 27-unit
plus 11-integration `oneagent-analysis` package result.

## Validation

The review independently reran the focused and compatibility matrix:

- `cargo test -p oneagent-analysis context::tests` — 8 passed.
- `cargo test -p oneagent-analysis context::selection` — 7 passed.
- `cargo test -p oneagent-analysis context::assembly` — 8 passed.
- `cargo test -p oneagent-analysis --test context_engine` — 11 passed.
- `cargo test -p oneagent-analysis` — 27 unit and 11 public integration tests passed; doctests also completed successfully.
- `cargo test -p oneagent-graph query` — non-zero graph unit/integration query matches passed.
- `cargo tree -p oneagent-analysis --edges normal` — only the existing BSL, common, graph, and graph-transitive metadata crates.

The canonical complete gate also passed:

- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps`
- `git diff --check`

The managed sandbox denies loopback bind without additional local permission;
the complete suite ran with bounded loopback permission. No external network or
service was used.

## Deferred scope

Qualified/fuzzy/source/editor seeds, source ranges and raw text, tokenization,
learned scoring, embeddings/vector search, providers/models, prompt templates,
conversation state, graph mutation or persisted bundles, incremental Context
repair, Runtime/HTTP/CLI/protocol exposure, tools, MCP, IDE integration,
performance/quality/security claims, and new Context Coverage state remain
deferred.

## Risk assessment

Residual risk is bounded to the accepted semantic-only first slice. Relevance
is an explicit fixed graph policy rather than a measured quality signal;
candidate discovery computes the complete eligible depth-bounded set before
truncation; and byte budgeting is intentionally not a model-token estimate.
Large-workspace performance and model usefulness are unmeasured by contract.
These accepted limits do not block ADR-0044 or the Sprint 23 provider boundary.

## Previous-suite retirement

After the `pass` decision, `git ls-files` and the filesystem both contained
exactly the seven planned Sprint 21 prompt files and no additional or untracked
file. Repository search found no retained Markdown link dependency on an
individual deleted prompt. The exact suite is retired atomically with this
review; the complete Sprint 22 suite,
`docs/codex/prompts/run-next-sprint.md`, non-adjacent suites, and `.codex/`
remain unchanged.
