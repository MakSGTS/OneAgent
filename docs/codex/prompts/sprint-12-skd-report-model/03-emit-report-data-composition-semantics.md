# Task 03: Emit Report Data Composition semantics

Continue OneAgent development.

## Reporting

- Communicate with the user in Russian.
- Keep code, identifiers, repository documentation, comments, Rustdoc, tests,
  errors, public APIs, prompt text, and commit messages in English.
- Report only live repository evidence or accepted architecture.

## Profile

`docs/codex/profiles/graph-implementation.md`

## Template

`docs/codex/templates/graph-emission-task.md`

Read the Profile, Template, required Core and Workflow modules, and
`docs/codex/README.md` completely before acting.

## Authoritative documents

- `docs/Roadmap.md`, Sprint 12 Task 03;
- `docs/architecture/report-data-composition-source-investigation.md`;
- `docs/architecture/semantic-model-2.md`;
- `docs/adr/0007-edt-to-semantic-graph.md`;
- `docs/adr/0008-edt-metadata-object-reader.md`;
- `docs/adr/0021-reads-semantics.md`;
- `docs/adr/0024-reference-request-provenance.md`;
- `docs/adr/0030-register-query-semantics.md`;
- `docs/adr/0034-report-data-composition-semantics.md`.

## Required gate

Proceed only when Tasks 01 and 02 are committed, all required implementation
validation passed, and the live baseline contains the accepted graph model and
typed Report/DCS parser without task-created uncommitted changes.

## Task

Integrate the committed Report Data Composition parser with the filesystem
semantic graph builder. Emit accepted nodes and immediate Contains ownership
with deterministic provenance and typed deferred diagnostics/statistics. Do not
analyze DCS query language or emit data-source relations.

## Source contract / production source

Use only the Task 02 typed source model and Task 01 public graph APIs. Preserve
the existing universal Report discovery, metadata identity/payload, members,
forms, commands, modules, BSL declarations, and references. Do not reparse DCS
or create a second identity authority inside the producer.

## Scope

One production projection joining committed Report/DCS source facts to the
committed graph entity and ownership model.

## Included

- Invoke the dedicated parser only for discovered `MetadataKind::Report`
  objects without changing other top-level metadata behavior.
- Preserve the existing Report node and configuration ownership exactly.
- Insert UUID-backed Data Composition Schema nodes with typed main payload and
  Report-to-Schema Contains.
- Insert direct Data Set nodes with typed kind/source payload and
  Schema-to-DataSet Contains.
- Insert direct named Field nodes with typed data-path payload and
  DataSet-to-Field Contains.
- Insert one stable metadata-owned Query plus DataSet-to-Query Contains for
  every accepted direct Query data set.
- Attach non-empty deterministic provenance carrying identities, artifact path,
  semantic role, content required for Diff, stable producer, origin,
  confidence, and resolution without putting provenance into identity.
- Project nested data sets, field folders, and unknown child types through
  deterministic typed diagnostics and one legacy rejected-observation outcome
  per source observation.
- Preserve fatal parser errors as complete build failures with no successful
  partial result; preserve independently accepted siblings after recoverable
  deferred outcomes.
- Add generated production projects covering main/non-main/empty Query/Object/
  Union, payload/content modifications, deferred outcomes, fatal errors,
  source/filesystem reordering, repeated builds, and unrelated compatibility.

## Excluded

- New graph or parser semantics beyond Tasks 01–02.
- Query-language parsing, QuerySource requests, candidate resolution,
  query-language diagnostics, Reads, DependsOn, References, field lineage, or
  partial query facts.
- Nested Union child, folder, parameter, settings, template, layout, or runtime
  composition entities.
- Final provenance-backed fixture, broad consumer/index/Coverage evidence, or
  current-state documentation completion.

## Acceptance criteria

- Valid main/non-main/empty Query/Object/Union projects emit exact typed nodes,
  payloads, provenance, and immediate unique owners while existing Report facts
  remain unchanged.
- Query text changes preserve metadata-owned Query identity and yield modified
  source evidence; source/XML/filesystem order never changes canonical output.
- Nested children, field folders, and unknown types emit no Unknown,
  placeholder, empty-name, ordinal, or guessed node and are typed/counted once.
- Fatal source errors yield no successful partial build; recoverable deferred
  outcomes preserve accepted owners and siblings.
- Public QuerySource ledger contents/counts remain unchanged; no new Reads,
  DependsOn, References, candidate, or query-language diagnostic is emitted.
- Generic Query and Validation observe accepted nodes/ownership, and Contains
  remains excluded from dependency/Impact propagation.
- Reordered inputs and repeated builds produce equal graph, payload,
  provenance, diagnostics, statistics, ledger, report, and validation results.
- Existing metadata, members, modules, Calls, Reads, Writes, Grants, Includes,
  Extends, Opens, Triggers, DependsOn, References, diagnostics, statistics, and
  request-ledger regressions pass.
- EDT Coverage status does not transition in this task and full workspace
  validation succeeds.

## Repository Safety

- Recheck Git state, exact Task 01–02 range, builder phases, parser APIs,
  diagnostics/statistics, tests, Coverage, and applicable `AGENTS.md`.
- Preserve unrelated user files and do not modify the committed prompt suite.
- Do not add dependencies or commit ignored live source artifacts.
- Do not stage or commit without launch-time authorization; never use broad
  staging or destructive Git commands.

## Task-specific validation

```bash
cargo test -p oneagent-edt --test report_data_composition
cargo test -p oneagent-graph --test validation
```

Then run the complete workspace validation from
`docs/codex/core/validation.md`, including `git diff --check`. Report zero-match
filters separately.

## Suggested commit message

```text
Emit Sprint 12 report data composition semantics
```

When authorized, stage only task-owned EDT builder/emission code, focused
production tests, generated project support, and necessary diagnostic/statistics
updates. Do not stage final fixtures, Coverage/docs completion, planning prompts,
ignored live artifacts, or unrelated paths; do not create an empty commit.

## Final report additions

Report production path, node/payload/identity/ownership inventory, provenance,
fatal/deferred behavior, diagnostics/statistics, query-relation non-emission,
determinism, Coverage non-transition, files/tests, validation, commit hash,
final Git status, and the Task 04 gate.
