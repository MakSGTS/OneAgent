# Graph Emission Workflow

Use this workflow for resolution, semantic node emission, semantic edge emission,
provenance, and deduplication.

## Required behavior

- Reuse existing canonical node resolution where possible.
- Use deterministic node and edge identities.
- Do not create placeholder nodes unless accepted architecture explicitly allows
  them.
- Define missing, ambiguous, external, unsupported, and malformed-source policy.
- Treat provenance as evidence, not identity.
- Preserve repeated-build determinism.
- Deduplicate duplicate observations deterministically.
- Add or update precise validator matrix when production emission requires it.
- Add positive, negative, regression, and repeated-build tests.
- Transition Coverage Registry only after complete implementation evidence.
