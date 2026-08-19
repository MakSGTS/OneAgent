# Task 01: Implement Form and Command graph prerequisites

Continue OneAgent development.

## Reporting

- Communicate with the user in Russian.
- Keep code, identifiers, documentation, comments, Rustdoc, tests, errors,
  public APIs, prompt text, and commit messages in English.
- Report only live repository evidence or accepted architecture.

## Profile

`docs/codex/profiles/graph-implementation.md`

## Template

`docs/codex/templates/graph-model-task.md`

Read the Profile, Template, their required Core and Workflow modules, and
`docs/codex/README.md` completely before acting.

## Authoritative documents

- `docs/Roadmap.md`, Sprint 7 execution plan;
- `docs/architecture/semantic-model-2.md`;
- `docs/architecture/form-command-source-investigation.md`;
- `docs/adr/0006-semantic-graph.md`;
- `docs/adr/0017-depends-on-semantics.md`;
- `docs/adr/0024-reference-request-provenance.md`;
- `docs/adr/0025-references-endpoint-validation.md`;
- `docs/adr/0029-form-command-navigation-semantics.md`.

## Required gate

Proceed only when the source investigation, accepted ADR-0029, Sprint 7 Roadmap
plan, and synchronized Semantic Model boundary are committed as one immutable
planning baseline and Sprint 7 is the live target. Existing Form/Command
declarations and Common Form module behavior are prerequisites, not work to
recreate.

## Task

Implement only the source-independent graph prerequisites required by ADR-0029.
Keep EDT parsing and production emission out of this task.

## Scope

One additive graph-domain contract with consumer compatibility evidence.

## Included

- Add `EdgeKind::Opens` with stable machine-readable identity and deterministic
  equality and ordering.
- Accept exactly `Procedure --Opens--> Form` and
  `Procedure --Opens--> Metadata(CommonForm)` endpoint kinds; reject every
  unrelated source, target, Unknown, and wildcard Metadata pair.
- Integrate `Opens` into exhaustive enum consumers, generic edge filtering,
  direct dependency/usage classification, bounded traversal, reverse Impact,
  graph/build Diff, Semantic Index, incremental-index equivalence, reports, and
  graph-domain Coverage declarations.
- Add precise `Form --Contains--> Module`,
  `Command --Contains--> Module`, and
  `Metadata(Command) --Contains--> Module` ownership rules while preserving
  existing Common Form module ownership.
- Add exact Command-source `References` and `DependsOn` matrices for only
  the nine ADR-0029 metadata targets.
- Update public Rustdoc and tests for additive public API behavior.

## Excluded

- EDT readers, fixtures, source discovery, BSL extraction, or graph emission.
- New node kinds, Form internals, Command Groups, placeholders, execution
  relations, or payload changes.
- Companion `References` or `DependsOn` for `Opens`.
- Broad Procedure, Command, Metadata, Unknown, or UI endpoint fallbacks.
- Coverage transitions that claim EDT production support.

## Acceptance criteria

- Existing EdgeKind machine identities remain unchanged; `Opens` is additive.
- Endpoint tests cover the complete positive matrix and exhaustive negative
  families in both directions with deterministic validation issue ordering.
- Every accepted module has exactly one compatible owner; missing, multiple,
  self, and incompatible ownership remains invalid.
- Command references and dependencies accept both Common and subordinate
  Command sources and exactly the accepted nine target kinds.
- `Opens` is a direct dependency and incoming usage relation and propagates
  reverse Impact from changed Form to opening Procedure.
- Diff, Query, report, complete Semantic Index, and incremental updates observe
  `Opens` deterministically without changing existing ordering contracts.
- Existing Contains, References, DependsOn, Calls, Reads, Writes, Includes,
  Grants, Extends, Query, Diff, Impact, and Coverage regressions remain green.
- No parser or producer behavior changes.

## Task-specific validation

Run focused checks first:

```bash
cargo test -p oneagent-graph --lib
cargo test -p oneagent-graph --test validation
cargo test -p oneagent-graph --test query
cargo test -p oneagent-graph --test diff
cargo test -p oneagent-graph --test impact
cargo test -p oneagent-graph --test coverage
```

Then run the complete workspace validation from
`docs/codex/core/validation.md`, including `git diff --check`. Report
zero-match filters separately.

## Commit

Commit only when the current launching instruction explicitly authorizes this
task or the master Sprint 7 commit sequence. After successful validation, stage
only task-owned graph implementation, tests, and required current-state
documentation, then create one commit:

```text
Define Sprint 7 graph navigation model
```

Never stage the prompt suite, use broad staging, or create an empty commit.

## Final report additions

Report graph-model and public API impact, endpoint matrices,
validation/query/index/Diff/Impact behavior, Coverage declaration impact,
preserved behavior, files, tests, validation, commit hash, exact Git status,
and the Task 02 gate.
