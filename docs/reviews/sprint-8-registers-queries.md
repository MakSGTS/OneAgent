# Sprint 8 Registers and Queries Integration Review

## Review status

Pass recorded on 2026-08-20 against committed Task 5 head
`5fce866448a5559a78b812955cda28ebd0492406`.

Sprint 8 satisfies [ADR-0030](../adr/0030-register-query-semantics.md) and
preserves the completed Query, Reads, Writes, reference, dependency, ownership,
consumer, and Sprint 7 contracts. No blocking or non-blocking findings, missing
acceptance evidence, open questions, compatibility breaks, or scope violations
remain.

## Reviewed baseline and range

The accepted Sprint 8 planning baseline is
`59c0683c6f2e1873d28534fede235f3fe11cdfac`. The reviewed implementation and
evidence range is:

```text
59c0683c6f2e1873d28534fede235f3fe11cdfac..5fce866448a5559a78b812955cda28ebd0492406
```

The commits are dependency ordered:

| Task | Commit | Message |
|---|---|---|
| 1 | `0deab899dbb73bcb34ff43dba921ecc3da04fa62` | `Define Sprint 8 register query graph rules` |
| 2 | `0791a82e81419db4030553632ee1a0c73fd182b9` | `Parse Sprint 8 direct register query sources` |
| 3 | `38b2685ed2f563834230dc7a23fa2ee0e834b558` | `Resolve Sprint 8 query source requests` |
| 4 | `156ef0d49a6714769f6774ac936c7e94ec84a41a` | `Emit Sprint 8 query data dependencies` |
| 5 | `5fce866448a5559a78b812955cda28ebd0492406` | `Complete Sprint 8 production evidence` |

The audit compared this range with ADR-0030, the
[source investigation](../architecture/register-query-source-investigation.md),
ADRs 0017, 0021, 0022, and 0024, Semantic Model 2.0, the Roadmap, live
production code, fixtures, Coverage registries, and executed tests.

## Acceptance evidence matrix

| Area | Commits, files, and executed evidence | Result |
|---|---|---|
| Graph endpoint rules and consumers | Task 1 extends only the exact `Query --Reads/DependsOn--> Metadata(Catalog \| InformationRegister \| AccumulationRegister \| AccountingRegister)` matrices in `crates/graph/src/validation.rs`. `validation.rs`, `query.rs`, and `impact.rs` prove exhaustive endpoints, distinct direct and normalized facts, generic dependency/usage navigation, one affected Query with both reasons, and unchanged unrelated matrices. `incremental_index::tests::query_register_data_source_edges_match_clean_rebuild` proves clean/incremental equivalence. | pass |
| Parser categories and completeness | Task 2 adds only `AccumulationRegister` and `AccountingRegister` categories in `crates/bsl/src/query_language.rs`. The two reduced query fixtures and their manifest preserve real qualified sources, aliases, locations, and target mappings. Parser tests prove exact locations, namespace boundaries, typed rejection, unconsumed input, virtual/temporary/parameter forms, and all-or-nothing output. | pass |
| Public QuerySource lifecycle | Task 3 collects canonical Query-source requests with one expected metadata kind and deterministic collection provenance, then resolves them through terminal public ledger states. `query_source_resolution` tests cover all four kinds, missing, ambiguous, incompatible, partial, collisions, Unicode lowercase behavior without normalization, duplicates, reordering, repeated resolution, identity, terminal conflicts, and rejected parse results. | pass |
| Production projections | Task 4 makes the terminal request ledger the canonical source of Query diagnostics, request statistics, retained resolved `Reads`, and derived `DependsOn`. `bsl_graph` and `adapters/edt/tests/reads.rs` prove distinct canonical identities and provenance, proving-Reads context, deduplication, no placeholders, no edge on failures or parser rejection, Query/report/build Diff visibility, validation, Impact, deterministic diagnostics, and unchanged legacy parser-rejection counting. | pass |
| Full-builder evidence | Task 5 adds `adapters/edt/tests/fixtures/sprint8_registers_queries_project` and `sprint8_registers_queries.rs`. The manifest maps every reduced artifact to exact real source ranges, blobs, descriptor identities, and SHA-256 values. The filesystem builder proves both new register families, existing Catalog and Information Register compatibility, Procedure ownership, requests, Reads, DependsOn, statistics, reports, validation, Query, Diff, Impact, source-order independence, repeated builds, and absence of Writes or References projections. | pass |
| Negative and workspace behavior | Parser, resolver, `reads.rs`, `bsl_graph`, and `production_builder_propagates_explicit_partial_scope_to_query_requests` tests jointly cover malformed, unsupported, virtual, temporary, parameter, missing, ambiguous, incompatible, partial, duplicate, and invariant-failure paths. Unsupported or incomplete programs emit no accepted request projection, Reads, DependsOn, or placeholder target. | pass |
| Identity, provenance, and reconciliation | Query identity and `Contains` ownership remain independent from query text. Request identity excludes mutable lifecycle content. Collection and resolver provenance, candidates, edge evidence, diagnostics, statistics, reports, request/build Diff, graph Diff, validation, repeated builds, and reordered inputs reconcile deterministically. Each unique resolved request produces exactly one Reads and one DependsOn per Query-target pair. | pass |
| Compatibility | Existing Catalog and Information Register Reads gain only the accepted additive normalized dependency. Writes remains Procedure-to-Accumulation-Register without a companion dependency. Metadata and Command References/DependsOn, Calls, Opens, Includes, Grants, Extends, typed payload, Query APIs, and completed Sprint 7 behavior remain green in the complete graph and EDT suites. No production dependency or transport surface changed. | pass |
| Coverage and aggregates | `coverage::tests::sprint8_query_evidence_preserves_supported_coverage_aggregates` and the representative-evidence audit keep Reads, DependsOn, and ReferenceRequest `Supported`. EDT remains exactly 101 capabilities: 96 `Supported`, 5 `NotApplicable`; graph remains exactly 85: 82 `Supported`, 3 `NotApplicable`. Both retain zero Critical, High, and Medium gaps. | pass |
| Scope containment | The range adds no Calculation Register source, register virtual-table inference, JOIN, UNION, nesting, batch, temporary/external/parameter acceptance, new Query declaration family, Query mutation, write-derived dependency, register payload/member model, Designer XML path, persistence, Runtime, API transport, dependency, or framework change. | pass |

## Validation

The required focused review matrix executed against committed Task 5 head and
passed:

| Command | Result |
|---|---|
| `cargo test -p oneagent-bsl` | 37 passed |
| `cargo test -p oneagent-graph` | 252 passed: 138 unit and 114 integration |
| `cargo test -p oneagent-edt` | 252 passed: 204 unit and 48 integration |
| `cargo test -p oneagent-edt sprint8_full_builder_matrix_is_complete_deterministic_and_consumer_visible` | 1 matched and passed |

The exact integration filter reported zero matches in the other 12 EDT test
binaries. Those expected zero-match results are not counted as acceptance
evidence. Package doc-test targets also reported zero tests separately.

The complete workspace gate passed at the reviewed baseline:

```text
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace                         # 565 tests passed
cargo clippy --workspace --all-targets --all-features -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
git diff --check
```

Workspace unit and integration tests reported 565 passed, 0 failed, and 0
ignored. Workspace doc-test targets with zero tests are not included in the
565-test count.

## Findings, missing evidence, and risks

- Blocking findings: none.
- Non-blocking findings: none.
- Missing acceptance evidence: none.
- Open questions: none.
- Compatibility breaks: none.
- Scope violations: none.

The accepted grammar remains intentionally narrow. Calculation Registers,
register virtual tables, JOIN, UNION, nesting, batches, temporary, external,
and parameter tables, broader expressions, new Query declaration sources,
Query mutation, write-derived dependencies, and register payload or member
expansion remain explicit deferred boundaries rather than review findings.

## Decision

`pass`

Sprint 8 is complete. Direct static Query data sources now include the accepted
Catalog, Information Register, Accumulation Register, and Accounting Register
families through public QuerySource requests, retained Reads, and normalized
Query-origin DependsOn facts without changing Query identity or ownership.
Sprint 9 Roles and Access Rights is eligible as the next planning target; v0.3
remains planned through the Sprint 14 release integration review.
