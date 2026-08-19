# Task 02: Parse Form and Command module layouts

Continue OneAgent development.

## Reporting

- Communicate with the user in Russian.
- Keep code, identifiers, documentation, comments, Rustdoc, tests, errors,
  public APIs, prompt text, and commit messages in English.
- Report only live repository evidence or accepted architecture.

## Profile

`docs/codex/profiles/parser-implementation.md`

## Template

`docs/codex/templates/parser-task.md`

Read the Profile, Template, their required Core and Workflow modules, and
`docs/codex/README.md` completely before acting.

## Authoritative documents

- `docs/Roadmap.md`, Sprint 7 Task 02;
- `docs/architecture/semantic-model-2.md`;
- `docs/architecture/form-command-source-investigation.md`;
- `docs/adr/0007-edt-to-semantic-graph.md`;
- `docs/adr/0029-form-command-navigation-semantics.md`.

## Required gate

Proceed only when Task 01 is committed with message
`Define Sprint 7 graph navigation model` or current committed evidence proves
every Task 01 criterion already complete. Reinspect the real EDT artifacts
identified by the source investigation before defining parser behavior.

## Task

Extend EDT module discovery to return typed, deterministic observations for the
accepted Form and Command module layouts. Do not insert graph facts or analyze
BSL in this task.

## Source evidence

Use repository-owned artifacts under `OneAgent_EDTproject/src/`, including the
representative subordinate Form and Command and Common Form/Common Command
paths recorded by the source investigation. Start from the live
`FileSystemEdtModuleReader`, `EdtModuleDescriptor`, `EdtModuleKind`,
metadata-structure descriptors, their consumers, and existing tests. Recheck
all names and paths rather than copying the historical corpus counts.

## Scope

EDT discovery and typed parser observations for the four accepted module paths.

## Included

- Join `Forms/<Name>/Module.bsl` to the exact subordinate Form declared under
  the same metadata owner.
- Join `Commands/<Name>/CommandModule.bsl` to the exact subordinate Command.
- Join `CommonCommands/<Name>/CommandModule.bsl` to the exact Common Command.
- Preserve existing Common Form `Module.bsl` discovery and identity.
- Derive new module identity only as
  `<form-owner-id>:form_module` or
  `<command-owner-id>:command_module`, with canonical names `FormModule` and
  `CommandModule`.
- Define deterministic typed behavior for missing optional modules, orphan
  directories, mismatched names, duplicates, unreadable files, wrong kinds,
  unsupported layouts, reordered observations, and repeated reads.
- Add focused raw-layout and repository-backed parser tests.

## Excluded

- Synthesizing Form or Command owners from directories.
- Parsing `Form.form`, descriptor payload fields, Command Groups, or BSL.
- Graph nodes, ownership edges, callable declarations, references, navigation,
  diagnostics projection, Coverage, or Roadmap status changes.
- Changing accepted Form, Command, or existing Common Form module identities.

## Acceptance criteria

- Every accepted module observation names one existing canonical owner and one
  accepted role without relying on filesystem iteration order.
- Equal Command or Form names under different owners cannot collide.
- Missing optional module artifacts leave their owner valid and produce no
  guessed descriptor.
- Orphaned, duplicate, mismatched, unreadable, and unsupported inputs have
  deterministic typed outcomes and never synthesize owners.
- Existing Object, Manager, Common, and Common Form module-reader behavior
  remains compatible.
- Reordered equivalent layouts and repeated reads return equal ordered output.
- The parser produces no graph fact and changes no Coverage entry.

## Task-specific validation

Run focused checks first:

```bash
cargo test -p oneagent-edt module_reader
cargo test -p oneagent-edt metadata_structure
```

Then run the complete workspace validation from
`docs/codex/core/validation.md`, including `git diff --check`. Confirm each
filter executes meaningful tests.

## Commit

Commit only when the current launching instruction explicitly authorizes this
task or the master Sprint 7 commit sequence. Stage only task-owned EDT parser,
fixture, test, and necessary documentation paths, then create one commit:

```text
Parse Sprint 7 form and command modules
```

Never stage the prompt suite, use broad staging, or create an empty commit.

## Final report additions

Report source evidence, parsed layouts, identity and owner join rules, typed
negative outcomes, fixture coverage, preserved behavior, files, tests,
validation, commit hash, exact Git status, and the Task 03 gate.
