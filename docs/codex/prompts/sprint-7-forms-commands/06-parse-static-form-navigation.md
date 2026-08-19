# Task 06: Parse static Form navigation candidates

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

- `docs/Roadmap.md`, Sprint 7 Task 06;
- `docs/architecture/semantic-model-2.md`;
- `docs/architecture/form-command-source-investigation.md`;
- `docs/adr/0029-form-command-navigation-semantics.md`;
- accepted BSL extraction and symbol ADRs located from current repository
  evidence.

## Required gate

Proceed only when Task 05 is committed with message
`Integrate Sprint 7 command references` or is proven `already_complete`.
Before implementation, inspect exact positive and negative repository-owned
`OpenForm` excerpts and the live BSL parser/call/declaration APIs.

## Task

Implement a typed complete-statement extractor for the first ADR-0029 static
Form-opening grammar. This task produces candidates only; it does not resolve a
graph target or emit an edge.

## Source evidence

Use exact repository-owned Common and subordinate `CommandModule.bsl` excerpts
recorded by the source investigation. Prove positive explicit Common and
subordinate targets and negative dynamic, shorthand, default, comment, string,
Function, multiline, and malformed forms before generalizing tests.

## Scope

Typed BSL extraction for the first exact static navigation grammar only.

## Included

- Accept calls only from an accepted Common or subordinate Command module and
  directly inside a parsed Procedure.
- Require exact callee `OpenForm`, one complete static first-argument string
  literal, and a complete call statement.
- Parse only `CommonForm.<FormName>` and
  `<SupportedKind>.<OwnerName>.Form.<FormName>`.
- Support only Catalog, Document, Report, DataProcessor,
  InformationRegister, AccumulationRegister, AccountingRegister,
  CalculationRegister, BusinessProcess, and Task owner prefixes.
- Preserve Command module, containing Procedure, complete literal, parsed
  target kind, owner name when applicable, Form name, source location, and
  deterministic candidate order.
- Define typed accepted, malformed, unsupported, dynamic, incomplete,
  wrong-module, and wrong-callable outcomes.
- Add exact-source, multiline, ordering, duplicate, malformed, and repeated
  extraction tests.

## Excluded

- Graph target lookup, diagnostics projection, `Opens`, `References`,
  `DependsOn`, `Calls`, Coverage, or Roadmap changes.
- Variables, concatenation, computed/localized values, default Form aliases,
  ListForm/ObjectForm shorthand, generated Forms, or unsupported prefixes.
- Calls in Functions, Form modules, ordinary metadata modules, comments, or
  strings that merely contain `OpenForm(`.
- General platform-call semantics or `Form.form` parsing.

## Acceptance criteria

- Candidate creation requires every accepted source, callable, syntax, literal,
  and grammar condition.
- Multiline complete calls preserve one deterministic source occurrence;
  incomplete or malformed statements do not become edge-producing candidates.
- Comments and unrelated string contents produce no false candidate.
- Dynamic, default, shorthand, unsupported, Function, and wrong-module cases
  remain typed non-edge-producing outcomes.
- Equivalent source-order permutations and repeated extraction produce stable
  ordered output.
- Existing BSL declaration, call, query, and Writes extraction behavior remains
  unchanged.
- No graph resolution or emission occurs.

## Task-specific validation

Run focused checks first:

```bash
cargo test -p oneagent-bsl
cargo test -p oneagent-edt bsl_graph
```

Then run the complete workspace validation from
`docs/codex/core/validation.md`, including `git diff --check`. Report
zero-match filters separately.

## Commit

Commit only when the current launching instruction explicitly authorizes this
task or the master Sprint 7 commit sequence. Stage only task-owned BSL/EDT
parser, fixture, test, and necessary documentation paths, then create one
commit:

```text
Parse Sprint 7 static form navigation
```

Never stage the prompt suite, use broad staging, or create an empty commit.

## Final report additions

Report exact source evidence, candidate schema, accepted grammar, typed rejected
forms, source-location and ordering behavior, files, tests, validation, commit
hash, exact Git status, and the Task 07 gate.
