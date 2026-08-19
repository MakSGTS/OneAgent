# Task 07: Emit canonical Form navigation

Continue OneAgent development.

## Reporting

- Communicate with the user in Russian.
- Keep code, identifiers, documentation, comments, Rustdoc, tests, errors,
  public APIs, prompt text, and commit messages in English.
- Report only live repository evidence or accepted architecture.

## Profile

`docs/codex/profiles/graph-implementation.md`

## Template

`docs/codex/templates/graph-emission-task.md`

Read the Profile, Template, their required Core and Workflow modules, and
`docs/codex/README.md` completely before acting.

## Authoritative documents

- `docs/Roadmap.md`, Sprint 7 Task 07;
- `docs/architecture/semantic-model-2.md`;
- `docs/architecture/form-command-source-investigation.md`;
- `docs/adr/0006-semantic-graph.md`;
- `docs/adr/0029-form-command-navigation-semantics.md`.

## Required gate

Proceed only when Task 06 is committed with message
`Parse Sprint 7 static form navigation` or is proven `already_complete`,
and its candidates enforce the complete accepted source and grammar boundary.

## Task

Resolve accepted Task 06 candidates after all metadata, Form, Command, Module,
Procedure, and Function nodes are collected, then emit only canonical
provenance-backed `Procedure --Opens--> Form` facts.

## Source contract

Use typed Task 06 accepted candidates and typed rejected observations from
accepted Common or subordinate Command module Procedures. Reuse current exact
name-and-kind resolution, owner/child navigation, diagnostics, reports, build
validation, Diff, Query, and Impact infrastructure. Do not parse source strings
again in the emitter.

## Scope

Resolution and canonical graph emission for Task 06 accepted candidates.

## Included

- Resolve `CommonForm.<Name>` by exact name and
  `Metadata(CommonForm)` kind.
- Resolve explicit subordinate targets in two stages: exact metadata owner by
  mapped kind and name, then exact `NodeKind::Form` child by name under that
  owner.
- Preserve explicit workspace completeness and typed unique, missing,
  ambiguous, incompatible, partial, invalid-owner, missing-child,
  ambiguous-child, and duplicate outcomes.
- Project Task 06 malformed, unsupported, dynamic, default, shorthand,
  wrong-module, and wrong-callable outcomes through the accepted deterministic
  diagnostic or unsupported-observation path without attempting resolution.
- Emit one canonical `Opens` edge on unique success with deterministic
  resolved provenance identifying Command, module, Procedure, literal, source
  occurrence, and target.
- Integrate diagnostics, reports, graph/build Diff, generic Query dependency and
  usage, reverse Impact, validation, source-order independence, and repeated
  builds.

## Excluded

- Global Form-name fallback, inferred owner kind, resolution or edge emission
  for default/shorthand/generated Form targets, dynamic values, unsupported
  prefixes, or placeholders.
- Calls from Functions or outside accepted Command modules.
- Companion `References`, `DependsOn`, `Calls`, ownership, or execution
  edge for the navigation fact.
- Form internals, Command Groups, Coverage transitions, or Sprint status.

## Acceptance criteria

- Common and subordinate targets resolve only through their accepted exact
  scopes; equal Form names under different owners remain independent.
- Wrong, missing, ambiguous, incompatible, and partial owner/child outcomes
  emit deterministic diagnostics and no edge.
- Dynamic, default, shorthand, malformed, unsupported, wrong-module, and
  wrong-callable observations retain their accepted typed outcome and emit no
  edge.
- Duplicate equivalent evidence creates one edge with sorted deduplicated
  provenance.
- Every emitted edge has standard source-target-kind identity and exact
  resolved provenance.
- Outgoing dependency, incoming usage, bounded traversal, reverse Impact,
  Query, Diff, validation, reports, and repeated builds expose `Opens`
  deterministically.
- No companion relation or placeholder is emitted.
- Existing modules, calls, references, dependencies, reads, writes, and Form
  declaration behavior remains green.

## Task-specific validation

Run focused checks first:

```bash
cargo test -p oneagent-graph --test validation
cargo test -p oneagent-graph --test query
cargo test -p oneagent-graph --test impact
cargo test -p oneagent-graph --test diff
cargo test -p oneagent-edt
```

Then run the complete workspace validation from
`docs/codex/core/validation.md`, including `git diff --check`.

## Commit

Commit only when the current launching instruction explicitly authorizes this
task or the master Sprint 7 commit sequence. Stage only task-owned graph/EDT
implementation, fixture, test, and necessary documentation paths, then create
one commit:

```text
Emit Sprint 7 form navigation
```

Never stage the prompt suite, use broad staging, or create an empty commit.

## Final report additions

Report resolution stages, provenance, typed failures, Query/Diff/Impact/report
behavior, excluded companion facts, preserved behavior, files, tests,
validation, commit hash, exact Git status, and the Task 08 gate.
