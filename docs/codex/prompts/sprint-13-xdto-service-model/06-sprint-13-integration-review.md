# Task 06: Review the integrated Sprint 13 baseline

Continue OneAgent development.

## Reporting

- Communicate with the user in Russian.
- Keep code, identifiers, repository documentation, comments, tests, errors,
  public APIs, prompt text, review artifacts, and commit messages in English.
- Report only live repository evidence or accepted architecture.

## Profile

`docs/codex/profiles/review.md`

## Template

`docs/codex/templates/review-task.md`

Read the Profile, Template, required Core and Workflow modules, and
`docs/codex/README.md` completely before acting.

## Authoritative documents

- `docs/Roadmap.md`, Sprint 13 execution plan and Task 06;
- `docs/architecture/xdto-service-source-investigation.md`;
- `docs/architecture/semantic-model-2.md`;
- `docs/adr/0006-semantic-graph.md`;
- `docs/adr/0007-edt-to-semantic-graph.md`;
- `docs/adr/0008-edt-metadata-object-reader.md`;
- `docs/adr/0023-typed-metadata-payload.md`;
- `docs/adr/0024-reference-request-provenance.md`;
- `docs/adr/0025-references-endpoint-validation.md`;
- `docs/adr/0033-event-subscription-semantics.md`;
- `docs/adr/0035-xdto-service-semantics.md`;
- the committed Sprint 13 prompt suite.

## Required gate

Proceed only when the accepted Sprint 13 planning baseline and Tasks 01–05 are
committed or proven `already_complete`, all required implementation validation
passed, and no task-created uncommitted change remains.

## Review target

Review the exact committed range from the parent of the Sprint 13 planning
commit through the committed Task 05 head. Do not rely on prompt claims or
commit subjects as implementation proof.

## Reviewed baseline / commit or diff range

Resolve exact full hashes from live Git history. Verify commit order, subjects,
owned paths, prerequisite chain, and final Task 05 repository state before
reviewing behavior.

## Scope

- Every ADR-0035 public node/payload/metadata payload/request/identity/
  ownership/endpoint/Impact contract.
- Exact XDTO descriptor/artifact join, direct Value/Object types, fatal and
  deferred behavior, and no nested property/import speculation.
- Exact HTTP URL Template/Method and Web Operation/Parameter fields, optional
  values, hierarchy, malformed/unsupported outcomes, and ordering.
- Existing metadata/module/symbol compatibility and exact internal XDTO
  package/type and owned HTTP/Web Function resolution.
- Public requests, internal/external policy, References, Triggers, provenance,
  diagnostics, statistics, determinism, and no placeholders.
- Generic Query, Diff, reports, Validation, Impact policy, complete and
  incremental indexes, tracked fixture provenance, Coverage, and documentation.
- Repository safety, exact Sprint 12 suite inventory, Sprint 14 eligibility,
  and the v0.3 release hand-off boundary.

## Excluded

- Silent fixes to implementation, tests, ADRs, prompts, fixtures, Coverage, or
  docs.
- XDTO properties/imports/restrictions/external nodes, HTTP route/runtime,
  WSDL/SOAP/transport, Designer XML, persistence, Runtime/API/CLI, MCP/LSP/IDE,
  serialization, benchmark, or Sprint 14 implementation.
- Refactoring or cleanup unrelated to a review finding.
- Deletion of the current Sprint 13 suite, bootstrap prompt, older non-adjacent
  suites, untracked files, or paths outside the verified Sprint 12 suite.

## Review criteria

- One canonical semantic meaning per node, payload, request, and edge.
- Stable UUID and collision-safe owner/name identities independent from
  content, position, and traversal.
- Exact source parsing, fatal/deferred/external boundaries, and no invented
  defaults or placeholder facts.
- Precise immediate ownership, payload compatibility, request lifecycle,
  endpoint validation, provenance, diagnostics/statistics, and determinism.
- Existing top-level metadata/module/symbol and unrelated relation compatibility.
- Generic consumer and complete/incremental index equivalence.
- Contains/Triggers non-propagation and unchanged References policy outside the
  additive ADR-0035 pairs.
- Truthful fixture derivation, Coverage evidence/counts, and current-state docs.
- Full deferred-scope and repository-safety conformance with no blocking
  correctness, regression, missing-evidence, or documentation finding.

## Acceptance evidence matrix

Record pass/fail evidence for: planning/commit chain; live corpus and tracked
fixture provenance; node/payload/public enum model; identity/collision;
XDTO join/types/errors/deferred constructs; HTTP/Web fields/errors; metadata
enrichment; modules/symbols; ownership; internal/external/absent declarations;
public request lifecycle; package/type/handler resolution; References; Triggers;
provenance; diagnostics/statistics; Query; Diff; reports; Validation; Impact;
complete index; incremental equivalence; Coverage; docs; unrelated compatibility;
workspace gate; and deferred scope.

## Authorized review outputs and state transition

When and only when the decision is `pass` or `pass with non-blocking
follow-ups`, create `docs/reviews/sprint-13-xdto-service-model.md`, update
`docs/Roadmap.md` to mark Sprint 13 `completed` and Sprint 14 `next`, and
synchronize the final hand-off statement. Update
`docs/architecture/semantic-model-2.md` only if its implemented current-state
statement requires the final review decision. Do not change implementation.

Sprint 13 completion does not complete v0.3. The v0.3 release integration review
becomes eligible only after Sprint 14 completes.

For `blocked`, create no completion transition, prompt retirement, or partial
review commit. Report the finding and leave Sprint 13 incomplete.

## Verified previous-suite retirement

Only after a non-blocking decision and every focused/full validation command
succeeds, re-enumerate and compare the live tracked inventory with exactly:

```text
docs/codex/prompts/sprint-12-skd-report-model/00-sprint-12-execution-loop.md
docs/codex/prompts/sprint-12-skd-report-model/01-implement-data-composition-graph-model.md
docs/codex/prompts/sprint-12-skd-report-model/02-parse-report-data-composition-schemas.md
docs/codex/prompts/sprint-12-skd-report-model/03-emit-report-data-composition-semantics.md
docs/codex/prompts/sprint-12-skd-report-model/04-complete-sprint-12-production-evidence.md
docs/codex/prompts/sprint-12-skd-report-model/05-sprint-12-integration-review.md
```

If and only if the inventory matches, no untracked file is endangered, and no
current Sprint 13 link depends on these files, explicitly delete these six
tracked files through the normal patch mechanism. Do not use recursive deletion
or globs. Keep the complete Sprint 13 suite and
`docs/codex/prompts/run-next-sprint.md` untouched. Stage the exact deletions,
review artifact, Roadmap transition, and any explicitly required final Semantic
Model state together in the single review commit. Any mismatch blocks deletion
and the final review commit.

## Task-specific validation

Run focused review checks:

```bash
cargo test -p oneagent-metadata
cargo test -p oneagent-graph
cargo test -p oneagent-edt --lib xdto_package::tests
cargo test -p oneagent-edt --lib service_descriptor::tests
cargo test -p oneagent-edt --test xdto_services
cargo test -p oneagent-edt --test coverage
cargo test -p oneagent-edt --test semantic_index
```

Then run the canonical complete workspace validation:

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
git diff --check
```

Record exact results and treat zero matched filters as missing evidence. After
the authorized state/deletion change, rerun `git diff --check` and manually
validate prompt/link inventories before committing.

## Suggested commit message

```text
Complete Sprint 13 XDTO and service model review
```

When authorized and the decision is non-blocking, stage only the review
artifact, Roadmap/current-state transition, and the six exact verified Sprint
12 prompt deletions. Never create a review commit for a blocked decision.

## Final report additions

Report exact reviewed range, acceptance matrix, findings, missing evidence,
scope/exclusion conformance, validation, decision, review artifact/state,
every retired path, retained Sprint 13 suite, commit hash, final Git status,
Sprint 14 eligibility, and the v0.3 release-review gate.
