# Task 04: Emit Query data dependencies

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

Read the Profile, Template, their required Core and Workflow modules, and
`docs/codex/README.md` completely before acting.

## Authoritative documents

- `docs/Roadmap.md`, Sprint 8 Task 04;
- `docs/architecture/semantic-model-2.md`;
- `docs/architecture/register-query-source-investigation.md`;
- `docs/adr/0017-depends-on-semantics.md`;
- `docs/adr/0021-reads-semantics.md`;
- `docs/adr/0024-reference-request-provenance.md`;
- `docs/adr/0030-register-query-semantics.md`.

## Required gate

Proceed only when Task 03 is committed with message
`Resolve Sprint 8 query source requests` or current committed evidence proves
every Task 03 criterion `already_complete`.

## Task

Integrate Task 03 terminal QuerySource requests into the production EDT build
result. Derive diagnostics and statistics once, retain resolved Reads, and emit
the accepted normalized Query-derived DependsOn projection.

## Source contract and production source

The source is an existing static Query node plus one completely accepted direct
Catalog, Information Register, Accumulation Register, or Accounting Register
source occurrence represented by one terminal public QuerySource request.
Parser-rejected programs do not form a request or projection.

## Scope

One production projection path from canonical terminal requests to diagnostics,
statistics, direct Reads, and derived DependsOn.

## Included

- Expose canonical QuerySource requests through the existing build-result
  ledger and request Query/report/build Diff surfaces.
- Replace independent Query source resolution counters with statistics derived
  once from terminal requests while retaining separately named parser rejection
  accounting where required for compatibility.
- Project current typed missing, ambiguous, incompatible, and partial
  diagnostics from terminal request outcomes with deterministic evidence.
- On unique success, retain one resolved exact `Reads` and emit one derived
  exact `DependsOn` for the same Query-target pair.
- Aggregate sorted, deduplicated provenance for both edges before insertion;
  dependency evidence must identify the terminal request and retained Reads
  fact.
- Prove Query dependency/usage filtering, unique reverse-Impact nodes with
  deterministic Reads/DependsOn reasons, graph/build Diff, reports, validation,
  request consistency, source-order independence, and repeated builds.
- Preserve existing Catalog and Information Register identities while adding
  the same normalized dependency behavior to them.

## Excluded

- Parser grammar/categories, resolver policy, graph endpoint rules, new public
  request types, Coverage transitions, or final representative fixtures.
- References companion edges, write-derived DependsOn, Query Writes, virtual
  tables, Calculation Registers, placeholders, transitive closure, reverse
  edges, weights, or dedicated Query APIs.
- Changes to metadata type, Command, Calls, Writes, Grants, Includes, Extends,
  Opens, or ownership production paths.

## Acceptance criteria

- Every accepted occurrence has one canonical terminal request visible in the
  build result with stable identity and provenance.
- Resolved requests project exactly one Reads and one DependsOn per unique
  Query-target pair; repeated occurrences aggregate provenance rather than
  edges or processed-request totals.
- Failed requests project no resolved edge and no placeholder.
- Parser rejections remain distinct from accepted requests and are not double
  counted.
- Build validation reconciles request outcomes, projections, diagnostics, and
  statistics deterministically.
- Generic dependencies expose two distinct edge facts; filtered results remain
  exact; Impact deduplicates affected nodes.
- Repeated builds have equal requests, diagnostics, statistics, reports, nodes,
  edges, provenance, graph diff, and build-result diff.
- Existing Writes and all unrelated semantic behavior remain green.
- Coverage status and aggregates remain unchanged in this task.

## Repository Safety

- Recheck Git state, production insertion sites, request/report/diff consumers,
  tests, and applicable `AGENTS.md` before editing.
- Preserve unrelated work and keep changes confined to the accepted Query
  production path.
- Do not stage or commit without explicit launching authorization.

## Task-specific validation

Run focused checks first:

```bash
cargo test -p oneagent-graph --test reference_request_build
cargo test -p oneagent-graph --test validation
cargo test -p oneagent-graph --test impact
cargo test -p oneagent-edt reads
```

Confirm filters execute meaningful tests, then run the complete workspace
validation from `docs/codex/core/validation.md`, including `git diff --check`.

## Commit

When explicitly authorized, stage only task-owned EDT production, graph
integration, tests, and necessary documentation, then create one commit:

```text
Emit Sprint 8 query data dependencies
```

Never stage the prompt suite, use broad staging, or create an empty commit.

## Final report additions

Report production path, request projections, direct/derived identity and
provenance, diagnostic/statistics compatibility, Query/Diff/Impact/report and
validation behavior, preserved edges, files, tests, validation, commit hash,
final Git status, and the Task 05 gate.
