# Graph Emission Task Template

## Purpose

Use this template for production emission of semantic nodes or edges, including
resolution, provenance, deduplication, diagnostics, validation, and Coverage
evidence.

## Recommended profile

- `docs/codex/profiles/graph-implementation.md`

## Required base template

- `docs/codex/templates/task-prompt.md`

## Required task-specific sections

- Source contract / production source

## Additional acceptance requirements

- Reuse existing canonical resolution and identity conventions.
- Attach provenance using the accepted provenance model.
- Preserve repeated-build determinism.
- Add positive, negative, regression, and repeated-build tests as applicable.
- Transition Coverage Registry only after complete evidence exists.

## Additional report sections

- Production path
- Identity strategy
- Provenance strategy
- Coverage transition
- Remaining gaps

## Additional validation

- Run focused producer and graph tests first.
- Run affected crate checks and full workspace validation when production Rust
  behavior changes.
