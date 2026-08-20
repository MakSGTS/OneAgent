# Sprint 13 XDTO and Service Model Integration Review

## Review status

Pass recorded on 2026-08-20 against committed implementation and recovery head
`5af338cd679a950c3ed262d1b777892186c92e22`.

Sprint 13 satisfies
[ADR-0035](../adr/0035-xdto-service-semantics.md), including the corrected
Function handler contract and deterministic multi-package XDTO namespace
resolution. It preserves existing metadata, module, symbol, request, relation,
consumer, and index behavior. No blocking or non-blocking findings, missing
acceptance evidence, open questions, or scope violations remain.

## Reviewed baseline and range

The parent baseline is
`cf59854baebc6fe88add0de5a0e5b6858b755a19`. The reviewed planning,
implementation, evidence, and recovery range is:

```text
cf59854baebc6fe88add0de5a0e5b6858b755a19..5af338cd679a950c3ed262d1b777892186c92e22
```

The commits are dependency ordered:

| Stage | Commit | Message |
|---|---|---|
| Planning | `3054fdb3f6767f6e630541a29fe47c1e1ad918a6` | `Plan Sprint 13 XDTO and service model` |
| Task 1 | `37d41bca3394ed711adf8d413258ef16fb5981e0` | `Implement Sprint 13 XDTO and service graph model` |
| Task 2 | `feec25e44214dfd7b170df479b77da320e61bc07` | `Parse Sprint 13 XDTO package schemas` |
| Task 3 | `3b480d5bf33dad7f3f3da754672da2e05887f04b` | `Parse Sprint 13 HTTP and Web service descriptors` |
| Task 4 | `84ba0ca4a1d27e9956fe06ba4f173b83b6b91864` | `Emit Sprint 13 XDTO and service semantics` |
| Handler recovery | `2a6dbbe94ba6e547aff4b4d804cf090b7d674382` | `Correct Sprint 13 Web handler target kind` |
| Handler recovery | `9b82c18cf1594754e68438fccf512c11bfb01ddd` | `Correct Sprint 13 HTTP handler target kind` |
| Task 5 | `7203627fdf7e9fb5247bb6fb569f45293ff6080f` | `Complete Sprint 13 production evidence` |
| Resolution recovery | `5af338cd679a950c3ed262d1b777892186c92e22` | `Fix ambiguous XDTO namespace resolution` |

The initial Task 6 review was blocked because XDTO type resolution selected the
first local package for a duplicated namespace. The recovery commit retains all
deterministically ordered package owners, resolves child candidates across the
complete namespace set, and proves every failed `XdtoType` terminal outcome.
Task 6 was then repeated against the corrected committed head.

The audit compared the exact range with ADRs 0006, 0007, 0008, 0023, 0024,
0025, 0033, and 0035, Semantic Model 2.0, the source investigation, Roadmap,
committed Sprint 13 prompts, production code, tracked fixture, Coverage
registries, and executed tests. The range contains 51 changed files, 9,227
insertions, and 87 deletions; per-commit path review found no unrelated change.

## Acceptance evidence matrix

| Area | Committed and executed evidence | Result |
|---|---|---|
| Planning and commit chain | Planning, Tasks 1-5, and three explicit recovery commits are linear, committed, dependency ordered, and leave no task-created uncommitted change. | pass |
| Live corpus and fixture provenance | Investigation covers 20 XDTO pairs, 12,666 unique direct types, 35 HTTP Methods, 119 Web Operations, and 360 Parameters. All 17 live and reduced fixture SHA-256 pairs were recomputed without mismatch. | pass |
| Node, payload, and public enum model | Five additive node kinds and closed typed payloads preserve exact accepted content; compatible metadata payloads and exhaustive repository consumers compile. | pass |
| Identity and collision behavior | UUID-backed service children and length-prefixed XDTO owner/name identities remain stable, content independent, and collision safe. | pass |
| XDTO join, types, errors, and deferred constructs | Exact descriptor/artifact joining, namespace agreement, direct Value/Object types, fatal cardinality/XML failures, and typed deferred nested/import observations are covered without speculation. | pass |
| HTTP and Web fields and errors | Required hierarchy and UUID/name values, optional HTTP method, package forms, XDTO types, nillability, directions, handlers, malformed values, and deterministic ordering are covered. | pass |
| Metadata enrichment | Existing XDTO Package, HTTP Service, and Web Service nodes retain identity and receive only compatible typed metadata payload content. | pass |
| Modules and symbols | Existing service Module discovery and BSL symbol extraction remain authoritative and unchanged. All 35 HTTP and 119 Web handlers resolve as owned Functions. | pass |
| Ownership | Only the five ADR-0035 immediate Contains pairs are added. Every emitted child has exactly one owner and no shortcut, reverse, placeholder, or transitive ownership is emitted. | pass |
| Internal, external, and absent declarations | Repository declarations create typed requests; external namespaces and absent optional package declarations create no false local request, placeholder, diagnostic, or edge. | pass |
| Public request lifecycle | Package, type, and callable requests preserve collection and resolution provenance, stable identities, deterministic candidates, terminal outcomes, and ledger/report consistency. | pass |
| Package, type, and handler resolution | Unique targets resolve exactly. Missing, ambiguous, incompatible, and invalid-owner outcomes are typed. Multi-package namespace collisions inspect all owners and never select the first package silently. | pass |
| References | Only the five additive ADR-0035 endpoint pairs are emitted for resolved internal targets. Failed and external declarations emit no References edge. | pass |
| Triggers | HTTP Methods and Web Operations dispatch only to resolved owned Functions; failed handlers emit neither References nor Triggers. Triggers remains non-propagating for Impact. | pass |
| Provenance | Accepted nodes, ownership, requests, resolution, diagnostics, References, and Triggers carry deterministic exact producer evidence. | pass |
| Diagnostics and statistics | Failed requests project matching stable codes, kinds, candidates, provenance, and exact aggregate outcomes without duplicate observations. | pass |
| Generic Query | Kind lookup, node lookup, owner/children navigation, endpoint adjacency, and repeated construction expose every accepted fact deterministically. | pass |
| Diff | Add/remove and payload, target, request, relation, direction, route, method, and external/internal boundary changes have stable identities and exact change scopes. | pass |
| Reports | Node, edge, diagnostic, request, provenance, and statistics distributions derive from stored canonical state and remain deterministic. | pass |
| Validation | Exact payload, owner, References, Triggers, request/edge, diagnostic, and report consistency contracts accept canonical graphs and reject invalid states. | pass |
| Impact | Contains and Triggers remain outside dependency propagation; direct payload and relation transitions retain the accepted bounded impact behavior. | pass |
| Complete Semantic Index | All five new kinds, payloads, ownership, References, and Triggers are visible through complete generic indexes. | pass |
| Incremental equivalence | XDTO/service node, payload, ownership, endpoint, and relation transitions match clean complete index rebuilds. | pass |
| Coverage | Graph Domain is exactly 96 capabilities: 92 `Supported` and 4 `NotApplicable`. EDT is exactly 120 capabilities: 115 `Supported` and 5 `NotApplicable`. Both registries have zero gaps. | pass |
| Documentation | ADR-0035, investigation, fixture README, Semantic Model, Roadmap, prompts, implementation, handler kinds, and executable aggregate counts agree. | pass |
| Unrelated compatibility | Full workspace tests preserve Metadata, modules, symbols, every existing edge family, Query, Diff, reports, Validation, Impact, request lifecycle, and complete/incremental indexes. | pass |
| Workspace gate and repository safety | Every focused filter matched tests and every canonical command exited successfully. The exact six-file Sprint 12 inventory matches, no Markdown link depends on it, and both user-owned untracked files remain untouched. | pass |
| Deferred scope | XDTO properties/imports/restrictions/external nodes, route/runtime transport, WSDL/SOAP, Designer XML, persistence, Runtime/API/CLI, MCP/LSP/IDE, serialization, and benchmarks remain deferred. | pass |

## Validation

The focused review matrix passed against the committed recovery head:

| Command | Result |
|---|---|
| `cargo test -p oneagent-metadata` | 11 passed; metadata doc-test target contained 0 tests |
| `cargo test -p oneagent-graph` | 293 passed: 157 unit and 136 integration; graph doc-test target contained 0 tests |
| `cargo test -p oneagent-edt --lib xdto_package::tests` | 8 passed; 249 filtered out |
| `cargo test -p oneagent-edt --lib service_descriptor::tests` | 7 passed; 250 filtered out |
| `cargo test -p oneagent-edt --test xdto_services` | 11 passed |
| `cargo test -p oneagent-edt --test coverage` | 5 passed |
| `cargo test -p oneagent-edt --test semantic_index` | 5 passed |

Every focused command matched the intended tests. Zero-test doc-test targets are
not counted as acceptance evidence. The resolution recovery additionally ran
`cargo test -p oneagent-edt --lib xdto_service_emission::tests`: 1 passed and
256 were filtered out.

The complete workspace gate passed at the reviewed implementation state:

```text
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
git diff --check
```

All commands exited successfully. Workspace unit and integration targets
executed 703 tests with zero failures; doc-test targets contained zero tests.

## Findings, missing evidence, and risks

- Blocking findings: none.
- Non-blocking findings: none.
- Missing acceptance evidence: none.
- Open questions: none.
- Scope violations: none.

The five additive `NodeKind` and closed payload variants plus two public request
categories expand exhaustive public enums. Repository consumers are migrated
and the full workspace gate proves internal compatibility. External exhaustive
consumers must accept the additive variants; this is the accepted ADR-0035
migration effect, not a review finding.

Nested XDTO schema content, external platform types, transport/runtime behavior,
and Designer XML remain explicit future scope. They are not missing Sprint 13
work.

## Previous-suite retirement

The verified Sprint 12 suite contained exactly these six tracked files, all of
which are retired in the Sprint 13 review commit:

- `docs/codex/prompts/sprint-12-skd-report-model/00-sprint-12-execution-loop.md`;
- `docs/codex/prompts/sprint-12-skd-report-model/01-implement-data-composition-graph-model.md`;
- `docs/codex/prompts/sprint-12-skd-report-model/02-parse-report-data-composition-schemas.md`;
- `docs/codex/prompts/sprint-12-skd-report-model/03-emit-report-data-composition-semantics.md`;
- `docs/codex/prompts/sprint-12-skd-report-model/04-complete-sprint-12-production-evidence.md`;
- `docs/codex/prompts/sprint-12-skd-report-model/05-sprint-12-integration-review.md`.

The complete seven-file Sprint 13 prompt suite and untracked bootstrap prompt
remain untouched.

## Decision

`pass`

Sprint 13 is complete. Repository-proven XDTO Package/type and HTTP/Web service
declarations now have stable typed nodes, payloads, immediate ownership, public
requests, exact internal References, owned Function Triggers, provenance,
diagnostics, statistics, and complete generic consumer/index evidence. Sprint
14 Designer XML Adapter is the next planning target. The v0.3 release
integration review remains ineligible until Sprint 14 completes and passes its
own integration review.
