# Sprint 7 Forms and Commands Integration Review

## Review status

Pass recorded on 2026-08-19 against committed Task 8 head
`c16e136eeff2df3296669f8ad682adbd9cdd3180`.

Sprint 7 satisfies ADR-0029 and preserves the completed Form, Command,
metadata-reference, BSL, ownership, and Sprint 6 member contracts. No blocking
or non-blocking findings, missing acceptance evidence, open questions,
compatibility breaks, or scope violations remain.

## Reviewed baseline and range

The accepted Sprint 7 planning baseline is
`77a52c6821e64f8fe7b9c71d2304a4ab77585cd7`. The reviewed implementation and
evidence range is:

```text
77a52c6821e64f8fe7b9c71d2304a4ab77585cd7..c16e136eeff2df3296669f8ad682adbd9cdd3180
```

The commits are dependency ordered:

| Task | Commit | Message |
|---|---|---|
| 1 | `cd3a94c73bb5e5196fca2dcab57108bd71ab11eb` | `Define Sprint 7 graph navigation model` |
| 2 | `cce27f614f8502d86df04f545175ce2d22f25209` | `Parse Sprint 7 form and command modules` |
| 3 | `74bef59dec0272e3a335400720ae1ece3e5274f8` | `Emit Sprint 7 form and command modules` |
| 4 | `aff51ad30426d3e29e6af5c03a7b9be791085c2e` | `Parse Sprint 7 command parameter references` |
| 5 | `5cb4c2e34f6e5170f54b8a913ef6b6c2a6171804` | `Integrate Sprint 7 command references` |
| 6 | `d0377cf7772fbe896406842a5a609bdfecaa0235` | `Parse Sprint 7 static form navigation` |
| 7 | `ef1ba9398fffca46f80aa865e952cee5d8e88c34` | `Emit Sprint 7 form navigation` |
| 8 | `c16e136eeff2df3296669f8ad682adbd9cdd3180` | `Complete Sprint 7 production evidence` |

The range contains the accepted graph model, EDT parsers and production
emission, focused and repository-fixture evidence, Coverage transitions, and
current-state documentation. The worktree was clean before the review record
was created.

## Acceptance evidence matrix

| Area | Commits, files, and executed evidence | Result |
|---|---|---|
| Graph model and consumers | Task 1 adds `EdgeKind::Opens`, stable edge identity, the exact Procedure-to-Form/Common Form validator endpoints, dependency and usage classification, reverse Impact propagation, generic Query filtering and traversal, Diff, report, Coverage, Semantic Index, and incremental-index enumeration. `crates/graph/tests/{validation,query,diff,impact,report,coverage}.rs` and graph unit tests passed. | pass |
| Module source parsing | Task 2 extends `adapters/edt/src/module_reader.rs` with owner-aware subordinate Form, subordinate Command, and Common Command module observations. Repository and generated tests prove canonical owner joins, stable role identity, missing optional files, orphan, name mismatch, duplicate owner, wrong owner kind, unsupported layout, unreadable source, equal-name owner scoping, source-order independence, and repeated reads. | pass |
| Module emission and BSL contribution | Task 3 emits canonical `FormModule` and `CommandModule` nodes and one provenance-backed owner edge, then reuses the existing BSL declaration, Query/Reads, Calls, diagnostic, and provenance path. `adapters/edt/tests/module_emission.rs` and the Sprint 7 fixture prove Module, Procedure, Function, Query, ownership, existing Common Form module compatibility, validation, Diff, and repeated builds. | pass |
| Command parameter parsing | Task 4 adds a distinct `CommandParameterType` role and exactly the nine ADR-0029 metadata target kinds. `command_parameter.rs`, `metadata_object.rs`, and `metadata_structure.rs` preserve canonical Command source identity, raw values, mapped names and kinds, duplicate counts, missing/empty containers, primitive, Defined Type, platform, unknown, malformed, multiple, reordered, and repeated outcomes without graph insertion. | pass |
| Command reference lifecycle | Task 5 converts accepted Common and subordinate Command observations into stable public metadata-type requests with collection and resolver provenance. Exact resolution emits one `References` and one derived `DependsOn`; missing, ambiguous, incompatible, partial, malformed, and unsupported paths emit no resolved or placeholder edge. `adapters/edt/tests/command_references.rs` and graph request tests prove ledger, diagnostics, statistics, report, Diff, Query, Impact, duplicate aggregation, and partial-to-resolved identity. | pass |
| Static navigation parser | Task 6 adds a private complete-statement extractor for exact `OpenForm` literals in recognized Command-module Procedures. Parser tests prove Common and explicit subordinate targets, the ten owner prefixes, multiline source, location, ordering, repeated extraction, and typed rejection of dynamic, concatenated, default, shorthand, unsupported, malformed, incomplete, comment/string-only, wrong-module, and Function cases. | pass |
| Navigation resolution and emission | Task 7 resolves Common Forms by exact kind and name and subordinate Forms by exact typed owner followed by owner-scoped child lookup. Unique success emits only `Procedure --Opens--> Form` with resolved exact provenance. `adapters/edt/tests/form_navigation.rs` proves equal-name isolation, missing/ambiguous/incompatible/partial outcomes, duplicate provenance, Query dependency and usage, reverse Impact, Diff, validation, deterministic diagnostics, source-order independence, and repeated builds. No companion `References`, `DependsOn`, `Calls`, or placeholder fact is emitted. | pass |
| Production evidence | Task 8 adds `adapters/edt/tests/fixtures/sprint7_forms_commands_project` and `adapters/edt/tests/sprint7_evidence.rs`. The full filesystem builder proves subordinate Form, subordinate/Common Command, and existing Common Form modules; BSL symbols and Query/Reads; both Command reference source kinds; Common and owner-scoped subordinate navigation; negative and partial diagnostics; provenance; requests; reports; statistics; validation; Diff; and repeated builds. Generic graph tests independently prove complete/incremental index equivalence for every edge kind, including `Opens`. | pass |
| Identity, provenance, and determinism | Existing UUID and owner-scoped fallback identities remain canonical. New module identities use only canonical owner ID and role; edge identity remains `(source, target, kind)`. Requests, candidates, provenance, diagnostics, nodes, edges, reports, and repeated builds use deterministic ordered aggregation. Focused and full package suites passed. | pass |
| Compatibility | Existing Form and Command declaration identity and ownership, Common Form module identity, metadata-member and AccessRight references, Calls, Reads, Writes, Includes, Grants, Extends, typed payload, and Sprint 6 member behavior remain green in the complete graph and EDT suites. | pass |
| Coverage and aggregates | Task 8 transitions EDT `semantic_edge.opens` only after complete production evidence. Live registry tests report 101 EDT capabilities (96 `Supported`, 5 `NotApplicable`) and 85 graph-domain capabilities (82 `Supported`, 3 `NotApplicable`), with zero Critical, High, or Medium gaps. Existing Form/Command declaration capabilities were not reopened. | pass |
| Scope containment | No `Form.form` internals, Command Group entity, multilingual Form/Command payload, dynamic/default/shorthand target resolution, generated or placeholder Form, command execution relation, Designer XML path, persistence, Runtime, AI, MCP, or IDE concern entered the range. | pass |

## Validation

The required focused review matrix executed against committed Task 8 head and
passed:

| Command | Result |
|---|---|
| `cargo test -p oneagent-metadata` | 9 passed |
| `cargo test -p oneagent-bsl` | 35 passed |
| `cargo test -p oneagent-graph` | 249 passed: 137 unit and 112 integration |
| `cargo test -p oneagent-edt` | 239 passed: 195 unit and 44 integration |

No focused acceptance filter was used or matched zero tests. Package doc-test
targets reported zero tests separately and are not counted as acceptance
evidence.

The full workspace gate also passed against the review change:

```text
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace                         # 547 tests passed
cargo clippy --workspace --all-targets --all-features -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
git diff --check
```

Workspace unit and integration tests reported 547 passed, 0 failed, and 0
ignored. Workspace doc-test targets with zero tests are not included in the
547-test count.

## Findings, missing evidence, and risks

- Blocking findings: none.
- Non-blocking findings: none.
- Missing acceptance evidence: none.
- Open questions: none.
- Compatibility breaks: none.
- Scope violations: none.

The accepted slice intentionally leaves Form internals, Command Groups,
localized Form and Command payload, dynamic/default/shorthand/generated Form
targets, form opening outside recognized Command-module Procedures, command
execution and UI binding relations, new Command parameter target families,
external or placeholder Forms, Designer XML equivalence, and runtime UI state
for later evidence-backed decisions. These are explicit deferred boundaries,
not Sprint 7 findings.

## Decision

`pass`

Sprint 7 is complete. Forms and Commands now contribute their accepted modules
and existing BSL semantics, mapped Command parameter references and normalized
dependencies, and precise static Form navigation through the canonical graph
without changing existing entity identities or pulling forward the deferred UI
model. Sprint 8 Registers and Queries is eligible as the next planning target;
v0.3 remains planned until its Sprint 14 release integration review.
