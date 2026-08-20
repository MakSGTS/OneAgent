# Task 02: Parse Report Data Composition Schemas

Continue OneAgent development.

## Reporting

- Communicate with the user in Russian.
- Keep code, identifiers, repository documentation, comments, Rustdoc, tests,
  errors, public APIs, prompt text, and commit messages in English.
- Report only live repository evidence or accepted architecture.

## Profile

`docs/codex/profiles/parser-implementation.md`

## Template

`docs/codex/templates/parser-task.md`

Read the Profile, Template, required Core and Workflow modules, and
`docs/codex/README.md` completely before acting.

## Authoritative documents

- `docs/Roadmap.md`, Sprint 12 Task 02;
- `docs/architecture/report-data-composition-source-investigation.md`;
- `docs/architecture/semantic-model-2.md`;
- `docs/adr/0008-edt-metadata-object-reader.md`;
- `docs/adr/0023-typed-metadata-payload.md`;
- `docs/adr/0034-report-data-composition-semantics.md`.

## Required gate

Proceed only when Task 01 is committed, all required implementation validation
passed, and the live baseline contains the accepted graph nodes, payloads,
identity helpers, ownership rules, consumers, indexes, and graph Coverage
without task-created uncommitted changes.

## Task

Implement a deterministic typed EDT reader that joins repository-proven Report
Data Composition template declarations with their `.dcs` artifacts. Return
accepted and deferred source outcomes only; do not emit graph facts.

## Source evidence / fixtures

- The ignored live corpus under `OneAgent_EDTproject/src/Reports/` has 56
  Reports, 56 exact UUID-backed DCS declarations/artifacts, 70 direct data sets,
  970 direct named fields, and 38 direct Query declarations.
- `AccessGroupsMembers` proves a main Query schema.
- `VolumeIntegrityCheck` proves a main Object schema.
- `AccountCardFinancialAccounting` proves a direct Union schema.
- `ControlOfProductsAccounting` proves eight deferred nested duplicate-name
  Union children.
- `UniversalReport` proves a valid empty main schema.
- `FinancialReport` proves a valid non-main schema.
- Generated test artifacts may cover malformed cases. The tracked production
  fixture is owned by Task 04; do not commit ignored live artifacts.

## Scope

One parser boundary preserving exact accepted Report/DCS content, structural
errors, and typed deferred constructs for later production projection.

## Included

- Add a focused `report_data_composition` module or the smallest equally scoped
  reader justified by current code.
- Join the existing Report descriptor UUID/name to direct unique DCS template
  UUID/name/type declarations and zero or one exact main-schema selector.
- Require exact `Templates/<name>/Template.dcs` correspondence and exact Data
  Composition Schema namespace/root.
- Parse zero or one direct `DataSource1` of type `Local` according to valid
  empty/non-empty corpus shapes.
- Parse uniquely named direct `DataSetQuery`, `DataSetObject`, and
  `DataSetUnion` elements with their accepted data-source cardinality.
- Parse direct `DataSetFieldField` elements with unique non-empty `field` name
  and `dataPath`.
- Preserve exactly one complete non-empty Query text for each direct Query data
  set without query-language parsing.
- Return typed deferred observations for nested data sets and
  `DataSetFieldFolder`; classify unknown `xsi:type` separately.
- Canonicalize accepted output by semantic identity independently from XML and
  filesystem order while retaining source context for provenance.
- Return typed deterministic errors for malformed XML, wrong root/namespace,
  unreadable/ambiguous/missing artifact, duplicate UUID/name, malformed or
  undeclared main selector, invalid data source, invalid/duplicate direct names,
  missing field path, and invalid query cardinality.
- Add live-shape positive and generated missing, duplicate, mismatched,
  malformed, reordered, and repeated-read tests.

## Excluded

- Graph node/payload/edge insertion, production diagnostics/statistics, or
  builder integration.
- Query-language parsing, target resolution, QuerySource requests, Reads,
  DependsOn, References, result fields, or lineage.
- Semantic modeling of nested Union children, folders, parameters, calculated
  fields, settings, templates, layouts, or runtime composition.
- Tracked production fixture, Coverage transition, or current-state docs.

## Acceptance criteria

- All 56 live declarations and `.dcs` artifacts agree under the accepted parser
  contract; no ignored file is required by committed tests.
- Valid empty, main, non-main, Query, Object, and Union schemas preserve exact
  UUID/name/kind/main/source/field/path/query content.
- Accepted direct data sets and fields are unique, stable-ID sortable, and
  semantically equal after XML/filesystem reordering.
- Missing/extra/ambiguous artifacts, duplicate UUID/name, malformed main
  selector, wrong root/namespace, malformed XML, unreadable input, invalid
  source, query-cardinality mismatch, and invalid required values have distinct
  deterministic typed errors.
- The eight nested duplicate-name Union children and six folders become
  distinct deferred outcomes, never accepted empty-name or ordinal entities.
- Unknown data-set/field types remain typed unsupported source evidence.
- Repeated reads are equal and no graph, diagnostics/statistics, production,
  request-ledger, relation, or Coverage behavior changes.
- Full workspace validation succeeds.

## Repository Safety

- Recheck Git state, Task 01 commit, Report reader APIs/usages, XML fixtures,
  ignored-source status, tests, and applicable `AGENTS.md` before editing.
- Preserve unrelated user files and do not modify the committed prompt suite.
- Do not add dependencies; use existing quick-xml and filesystem facilities.
- Do not stage ignored live project artifacts.
- Do not stage or commit without launch-time authorization; never use broad
  staging or destructive Git commands.

## Task-specific validation

```bash
cargo test -p oneagent-edt --lib report_data_composition::tests
cargo test -p oneagent-edt --lib metadata_object::tests
```

Then run the complete workspace validation from
`docs/codex/core/validation.md`, including `git diff --check`. Report zero-match
filters separately.

## Suggested commit message

```text
Parse Sprint 12 report data composition schemas
```

When authorized, stage only the task-owned EDT parser/source model, module
exports, focused tests, and small generated parser fixtures if necessary. Do
not stage graph emission, production fixtures, Coverage/docs completion,
planning prompts, ignored live artifacts, or unrelated paths; do not create an
empty commit.

## Final report additions

Report source evidence, parsed descriptor/artifact contract, fatal versus
deferred policy, ordering/identity inputs, query opacity, fixture usage,
files/tests, validation, commit hash, final Git status, and the Task 03 gate.
