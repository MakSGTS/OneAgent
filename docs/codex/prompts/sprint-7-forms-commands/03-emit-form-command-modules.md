# Task 03: Emit Form and Command modules

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

- `docs/Roadmap.md`, Sprint 7 Task 03;
- `docs/architecture/semantic-model-2.md`;
- `docs/architecture/form-command-source-investigation.md`;
- `docs/adr/0006-semantic-graph.md`;
- `docs/adr/0007-edt-to-semantic-graph.md`;
- `docs/adr/0029-form-command-navigation-semantics.md`.

## Required gate

Proceed only when Task 02 is committed with message
`Parse Sprint 7 form and command modules` or is proven `already_complete`,
and its parser output satisfies the accepted source and identity contract.

## Task

Convert only accepted Task 02 module observations into canonical Module nodes,
one provenance-backed owner `Contains` edge, and existing BSL declaration and
semantic contributions through the production EDT builder.

## Production source

Use the live Task 02 descriptor API, the EDT semantic graph builder, existing
module contribution and BSL graph pipeline, and repository-owned Form and
Command module artifacts. Locate all current consumers before changing a public
descriptor or builder API.

## Scope

Production conversion of Task 02 observations through the existing module and
BSL graph pipeline.

## Included

- Emit subordinate Form, subordinate Command, and Common Command Module nodes
  using ADR-0029 identities and names.
- Preserve the existing Common Form module node identity and behavior.
- Collect Module nodes before adding exactly one compatible owner
  `Contains` edge.
- Feed accepted module descriptors through the existing Procedure, Function,
  Query, Calls, Reads/Writes where already applicable, diagnostic, and
  provenance pipeline without a parallel UI symbol model.
- Define deterministic behavior for invalid or missing owners, duplicate and
  conflicting observations, reversed discovery order, partial inputs, and
  repeated builds.
- Add positive, negative, production-builder, Query, Validation, graph/build
  Diff, provenance, source-order, and repeated-build tests.

## Excluded

- Command parameter reference requests or projections.
- `OpenForm` extraction or `Opens` emission.
- `Command --Executes--> Procedure` or any inferred handler relation.
- Form internals, Command Groups, payload changes, placeholders, Coverage
  transitions, or Sprint status changes.
- Changes to ordinary metadata modules or existing BSL semantics outside
  supplying the newly accepted modules.

## Acceptance criteria

- Every accepted new module has canonical identity, non-empty deterministic
  provenance, and exactly one accepted Form or Command owner.
- Module-owned Procedure and Function declarations use existing identities and
  ownership behavior.
- Existing Common Form, ordinary metadata module, local/cross-module Calls,
  Query, Reads, Writes, diagnostic, and provenance behavior remains compatible.
- Missing optional modules emit no false node or edge.
- Invalid, missing, multiple, or incompatible owners are rejected or diagnosed
  according to accepted graph/build invariants.
- Reversed equivalent discovery and repeated production builds are equal.
- Graph/build Diff, Query, and Validation observe the new canonical facts.
- No Command parameter or `Opens` fact is emitted.

## Task-specific validation

Run focused checks first:

```bash
cargo test -p oneagent-edt
cargo test -p oneagent-graph --test validation
cargo test -p oneagent-graph --test query
cargo test -p oneagent-graph --test diff
```

Then run the complete workspace validation from
`docs/codex/core/validation.md`, including `git diff --check`.

## Commit

Commit only when the current launching instruction explicitly authorizes this
task or the master Sprint 7 commit sequence. Stage only task-owned producer,
fixture, test, and necessary documentation paths, then create one commit:

```text
Emit Sprint 7 form and command modules
```

Never stage the prompt suite, use broad staging, or create an empty commit.

## Final report additions

Report the production path, identity and ownership strategy, existing BSL
pipeline reuse, provenance, negative behavior, preserved semantics, files,
tests, validation, commit hash, exact Git status, and the Task 04 gate.
