# Task 02: Parse direct register Query sources

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

Read the Profile, Template, their required Core and Workflow modules, and
`docs/codex/README.md` completely before acting.

## Authoritative documents

- `docs/Roadmap.md`, Sprint 8 Task 02;
- `docs/architecture/query-language-parser-investigation.md`;
- `docs/architecture/register-query-source-investigation.md`;
- `docs/adr/0021-reads-semantics.md`;
- `docs/adr/0030-register-query-semantics.md`.

## Required gate

Proceed only when Task 01 is committed with message
`Define Sprint 8 register query graph rules` or current committed evidence
proves every Task 01 criterion `already_complete`.

## Task

Extend only the minimum query-language parsed source model with direct
Accumulation and Accounting Register categories. Do not resolve metadata,
create public requests, or emit graph facts.

## Source evidence and fixtures

Reinspect these real source and target chains before defining fixtures:

- `OneAgent_EDTproject/src/CommonModules/Accounting/Module.bsl`, Procedure
  `InventoryCostBeforeWrite`, direct
  `AccumulationRegister.InventoryCost AS OldRecords`;
- `OneAgent_EDTproject/src/AccumulationRegisters/InventoryCost/InventoryCost.mdo`;
- `OneAgent_EDTproject/src/CommonModules/MonthEndTransactions/Module.bsl`,
  Procedure `ARAPUpdateExecute`, direct
  `AccountingRegister.FinancialAccounting AS FinancialAccounting`;
- `OneAgent_EDTproject/src/AccountingRegisters/FinancialAccounting/FinancialAccounting.mdo`;
- their exact declarations in
  `OneAgent_EDTproject/src/Configuration/Configuration.mdo`.

Add raw query fixtures and update their provenance manifest. The complete real
programs exceed the minimum parser grammar, so any single-projection,
single-source reduction must be labeled generated/reduced scaffolding with
exact source ranges and preserved qualified source, alias, and target mapping.

## Scope

Two additive typed persistent categories under the existing all-or-nothing
parser contract.

## Included

- Add `QuerySourceCategory::AccumulationRegister` for exact English
  `AccumulationRegister.<Name>`.
- Add `QuerySourceCategory::AccountingRegister` for exact English
  `AccountingRegister.<Name>`.
- Preserve raw spelling, namespace, local name, optional supported alias, and
  deterministic UTF-8 byte location.
- Extend exhaustive category consumers and public Rustdoc.
- Add positive, case-sensitive namespace, wrong namespace, virtual-table,
  malformed, reordered, and repeated-parser evidence appropriate to this
  additive source classification.
- Preserve every current Catalog, Russian Catalog, Information Register, and
  typed rejection result.

## Excluded

- Calculation Registers or Russian register namespace spellings.
- Projection lists, `WHERE`, `GROUP BY`, `ORDER BY`, functions, comments,
  strings, JOIN, UNION, nesting, batches, temporary/external/parameter tables,
  virtual-table acceptance, or general grammar expansion.
- BSL Query extraction changes, metadata resolution, request lifecycle, graph
  validation or emission, diagnostics projection, Coverage, or Roadmap status.

## Acceptance criteria

- Each accepted fixture produces one complete program and exactly one source of
  the expected typed category.
- Source range slices the raw fixture to the exact qualified source spelling.
- Alias and local name remain raw source evidence and do not affect category.
- A third component or invocation remains `VirtualTableSource` and produces no
  partial program.
- Calculation Register and unrelated namespaces remain typed unsupported
  persistent namespaces.
- Existing parser behavior, diagnostic codes/messages, ordering, and repeated
  results remain compatible.
- Parser output contains no graph or EDT type and no production fact changes.

## Repository Safety

- Recheck Git state, parser definitions, fixture conventions, consumers, and
  applicable `AGENTS.md` before editing.
- Preserve unrelated work and do not modify `OneAgent_EDTproject/`; real
  artifacts are read-only evidence.
- Do not stage or commit unless explicitly authorized by the launching
  instruction.

## Task-specific validation

Run:

```bash
cargo test -p oneagent-bsl query_language
```

Confirm the filter executes meaningful tests, then run the complete workspace
validation from `docs/codex/core/validation.md`, including `git diff --check`.

## Commit

When explicitly authorized, stage only task-owned BSL parser, fixtures,
manifest, tests, and required documentation, then create one commit:

```text
Parse Sprint 8 direct register query sources
```

Never stage the prompt suite, use broad staging, or create an empty commit.

## Final report additions

Report real source evidence, reduction treatment, parsed categories and
locations, unsupported/unknown cases, fixture coverage, preserved grammar,
files, tests, validation, commit hash, final Git status, and the Task 03 gate.
