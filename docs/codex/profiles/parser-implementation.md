# Parser Implementation Profile

## Purpose

Use this profile for implementing parsing of one real source artifact family.

## Required Core modules

- `docs/codex/core/repository-safety.md`
- `docs/codex/core/repository-investigation.md`
- `docs/codex/core/change-contract.md`
- `docs/codex/core/validation.md`
- `docs/codex/core/final-report.md`

## Required Workflow modules

- `docs/codex/workflows/implementation.md`
- `docs/codex/workflows/parser.md`

## Task-family expectations

- Use repository-owned source evidence or fixtures.
- Do not invent serialized formats, XML elements, attributes, or fields.
- Keep parser output separate from graph emission unless the task explicitly
  includes both.
- Define malformed, missing, and optional input behavior.
