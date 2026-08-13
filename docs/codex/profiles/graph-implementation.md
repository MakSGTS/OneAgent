# Graph Implementation Profile

## Purpose

Use this profile for graph model and graph emission implementation tasks that
change semantic node or edge representation, resolution, provenance, validation,
query behavior, or Coverage evidence.

## Required Core modules

- `docs/codex/core/repository-safety.md`
- `docs/codex/core/repository-investigation.md`
- `docs/codex/core/change-contract.md`
- `docs/codex/core/validation.md`
- `docs/codex/core/final-report.md`

## Required Workflow modules

- `docs/codex/workflows/implementation.md`
- `docs/codex/workflows/graph-model.md` when public graph model contracts change
- `docs/codex/workflows/graph-emission.md` when producers emit semantic nodes or edges

## Task-family expectations

- Reuse accepted identity, endpoint, provenance, and validation contracts.
- Preserve deterministic graph generation and repeated-build stability.
- Transition Coverage Registry only after complete implementation evidence exists.
- Do not combine unrelated graph capabilities in one task.
