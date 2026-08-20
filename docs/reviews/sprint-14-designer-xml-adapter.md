# Sprint 14 Designer XML Adapter Integration Review

## Review status

Pass recorded on 2026-08-21 against committed Task 7 head
`19d56818a1345b4cced43db7275165ff24ce0748`.

Sprint 14 satisfies [ADR-0036](../adr/0036-designer-xml-adapter.md). The
dedicated source adapter discovers hierarchical Designer XML 2.20 projects,
loads explicit complete or partial scopes, parses the accepted configuration,
20 top-level metadata families and generic module roles, and contributes the
existing canonical configuration, metadata, Module, Procedure, Function, and
immediate `Contains` facts. No blocking or non-blocking findings, missing
acceptance evidence, open questions, or scope violations remain.

## Reviewed baseline and range

The parent readiness baseline is
`5b8c57b44247ffed5b26a52877b3b333bbf64703`. The reviewed planning through
Task 7 range is:

```text
5b8c57b44247ffed5b26a52877b3b333bbf64703..19d56818a1345b4cced43db7275165ff24ce0748
```

The commits are dependency ordered:

| Stage | Commit | Message |
|---|---|---|
| Planning | `73751ebc5390b1705fa53ad97af5331bde0ceb06` | `Plan Sprint 14 Designer XML adapter` |
| Task 1 | `0ea2736ace0088118045b4be61e4a6bd7b7cd858` | `Investigate Sprint 14 Designer XML source contracts` |
| Task 2 | `e6efaf625c081cf45928bd632a1101887d369f55` | `Define Sprint 14 Designer XML adapter contract` |
| Task 3 | `04a2e7ef5fffb62f14bab94d1b3f9bc1ebd9e098` | `Implement Sprint 14 Designer XML discovery` |
| Task 4 | `c6866d1eb7d93806333719b2941bda3ee487f83b` | `Parse Sprint 14 Designer XML metadata` |
| Task 5 | `de1039354259c8a9e1413c38b4ab9d1760e2673a` | `Parse Sprint 14 Designer XML modules` |
| Task 6 | `1bc78f39c3d3a728301570732a3fcf84883e9563` | `Emit Sprint 14 Designer XML semantics` |
| Task 7 | `19d56818a1345b4cced43db7275165ff24ce0748` | `Complete Sprint 14 conformance evidence` |

The audit compared the exact 44-file range, containing 6,917 insertions and 7
deletions, with ADR-0036, the source investigation and corpus registration,
Semantic Model 2.0, Roadmap, the committed Sprint 14 prompts, production code,
fixtures, Coverage, and executed tests. Per-commit path review found no
unrelated change, and `.codex/` was untouched.

## Acceptance evidence matrix

| Area | Committed and executed evidence | Result |
|---|---|---|
| Planning and commit chain | Planning and Tasks 1–7 are linear, committed, dependency ordered, and leave no task-created uncommitted state. | pass |
| Source and tooling provenance | The registered ignored pair records exact raw paths and hashes. The CI fixture records official 1C:Enterprise 8.3.27.2214 import/selective-export treatment, the bounded EDT compatibility rewrites, and every raw/reduced SHA-256. | pass |
| Detection and project boundary | Both markers, exact XML version, candidate boundaries, depth behavior, EDT/Designer marker conflict, symlink rejection, and incomplete candidates are deterministic. | pass |
| Complete and partial scope | Caller-declared complete/partial scope is explicit; partial absence emits no placeholder, while malformed accepted configuration, metadata, or BSL returns no graph. | pass |
| Configuration loading | Exact UUID, name, first direct synonym content, marker agreement, namespace/version, duplicate field, malformed XML, and repeated/reordered loads match ADR-0036. | pass |
| Metadata assembly | The 20 accepted top-level families enumerate canonically by kind, name, UUID, and path. Calculation Registers remain explicitly unsupported for lack of paired root/path evidence. | pass |
| Metadata failures and deferred artifacts | Missing, duplicate ID, path/name/root mismatch, foreign namespace, unsupported version, symlink, unknown, nested, and deferred artifacts have bounded typed outcomes without Unknown placeholders. | pass |
| Module assembly and BSL | Generic Object, Manager, and Common roles use owner/role identity, exact normalized source, deterministic ordering, and public source-independent BSL declaration extraction. | pass |
| Module failures | Missing optional roles emit nothing; duplicate, orphan, wrong-kind, mismatched, unreadable, symlinked, and malformed BSL inputs are typed and deterministic. | pass |
| Canonical graph contribution | Existing UUID and owner/role identities, kinds, exact names, common payloads, modules, Procedures, Functions, and immediate `Contains` facts are emitted without new graph kinds or edge kinds. | pass |
| Provenance | Every accepted node and edge has exact producer evidence with raw-file SHA-256; identities exclude paths and hashes. Official-fixture configuration and Common Module hashes are asserted. | pass |
| Paired conformance | Public EDT and Designer builders produce equal non-empty canonical partial projections containing configuration, Common Module, module, Procedure, and three ownership edges. | pass |
| Deliberate differences | Only paths/layout, producer IDs, XML vocabulary/order, BOM/line endings, raw hashes/provenance, and explicitly deferred artifacts remain outside equality and are documented. | pass |
| Controlled change | One Designer Common Module synonym change produces exactly one modified node with only `SemanticContent`; an empty or filtered-away oracle cannot pass. | pass |
| Determinism and negative outcomes | Reordered and repeated discovery, parsing, module loading, graph building, conformance building, missing/partial input, malformed input, unsupported version, and marker conflict are executed. | pass |
| Public consumers | Query, Diff, report, Validation, provenance report, exact name/kind lookup, and ownership navigation expose the accepted projection and remain deterministic. | pass |
| Complete and incremental indexes | Every canonical node resolves through complete Query and Resolution indexes. Designer-shaped add, semantic refresh, reparent, and remove transitions equal clean complete index rebuilds. | pass |
| Coverage | Designer-specific Coverage has 58 deterministic capabilities: 55 `Supported`, one `Unsupported`, two `NotApplicable`, and the single accepted Calculation Register evidence gap. Graph Domain and EDT Coverage are unchanged. | pass |
| EDT and unrelated compatibility | Full EDT, BSL, Metadata, Graph, Workspace, and Filesystem package suites pass; no EDT production code or public graph API changed. | pass |
| Documentation and scope | ADR, investigation, corpus registration, fixture README, Semantic Model, Roadmap, prompts, implementation, Coverage, and deferred scope agree. | pass |
| Workspace gate and repository safety | Every focused target matched tests; all six canonical workspace commands succeeded. Sprint 13 retirement inventory is exact, has no untracked file, and no retained Markdown link depends on a deleted prompt. | pass |

## Validation

The focused review matrix passed against committed Task 7 head:

| Command | Result |
|---|---|
| `cargo test -p oneagent-workspace --no-fail-fast` | 1 passed; doc-test target contained 0 tests |
| `cargo test -p oneagent-workspace-fs --no-fail-fast` | 5 passed; doc-test target contained 0 tests |
| `cargo test -p oneagent-designer-xml --no-fail-fast` | 34 passed: 31 unit and 3 conformance; doc-test target contained 0 tests |
| `cargo test -p oneagent-edt --no-fail-fast` | 350 passed; doc-test target contained 0 tests |
| `cargo test -p oneagent-bsl --no-fail-fast` | 37 passed; doc-test target contained 0 tests |
| `cargo test -p oneagent-metadata --no-fail-fast` | 11 passed; doc-test target contained 0 tests |
| `cargo test -p oneagent-graph --no-fail-fast` | 294 passed: 158 unit and 136 integration; doc-test target contained 0 tests |

Every focused command matched the intended tests. Zero-test doc-test targets
are not counted as acceptance evidence.

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
executed 743 tests with zero failures; doc-test targets contained zero tests.

## Findings, missing evidence, and risks

- Blocking findings: none.
- Non-blocking findings: none.
- Missing acceptance evidence: none.
- Open questions: none.
- Scope violations: none.

Calculation Register discovery is the one explicit Designer Coverage gap
because the paired corpus does not prove its root/path contract. Metadata
members, specialized source-family semantics, non-ownership relations, flat
dumps, extensions, parent configurations, binary artifacts, runtime/API/CLI,
persistence, MCP/LSP/IDE, packaging, and performance remain accepted deferred
scope rather than missing Sprint 14 work.

## Previous-suite retirement

The verified Sprint 13 suite contained exactly these seven tracked files, all
of which are retired in the Sprint 14 review commit:

- `docs/codex/prompts/sprint-13-xdto-service-model/00-sprint-13-execution-loop.md`;
- `docs/codex/prompts/sprint-13-xdto-service-model/01-implement-xdto-service-graph-model.md`;
- `docs/codex/prompts/sprint-13-xdto-service-model/02-parse-xdto-packages.md`;
- `docs/codex/prompts/sprint-13-xdto-service-model/03-parse-http-web-services.md`;
- `docs/codex/prompts/sprint-13-xdto-service-model/04-emit-xdto-service-semantics.md`;
- `docs/codex/prompts/sprint-13-xdto-service-model/05-complete-sprint-13-production-evidence.md`;
- `docs/codex/prompts/sprint-13-xdto-service-model/06-sprint-13-integration-review.md`.

The complete nine-file Sprint 14 prompt suite and
`docs/codex/prompts/run-next-sprint.md` remain untouched.

## Decision

`pass`

Sprint 14 is complete. Hierarchical Designer XML 2.20 sources now contribute
the accepted source-independent configuration, top-level metadata, generic
module, BSL declaration, and immediate ownership slice with exact provenance
and cross-adapter conformance. The v0.3 release integration review is eligible.
Sprint 15 Runtime Service Container is the next planning target only after that
release gate completes; this review does not execute the release review or plan
Sprint 15.
