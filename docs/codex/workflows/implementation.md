# Implementation Workflow

Use this workflow for implementing accepted architecture.

## Required behavior

- List authoritative ADRs and architecture documents in the task prompt.
- Treat accepted decisions as fixed.
- Investigate repository APIs, tests, fixtures, and technical constraints.
- Implement only within task acceptance criteria.
- Avoid architecture reselection during implementation.
- Add focused tests and regression tests appropriate to the change.
- Synchronize documentation when behavior changes.
- Transition Coverage Registry only when complete evidence exists.

## Blocker procedure

If authoritative architecture cannot be implemented:

1. stop the affected implementation;
2. describe the concrete blocker;
3. identify the authoritative decision that cannot be implemented;
4. avoid inventing an alternative architecture;
5. leave unrelated work unchanged;
6. propose a separate architecture task if needed.
