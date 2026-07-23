# Graph Model Workflow

Use this workflow for changes to `NodeKind`, `EdgeKind`, graph identity, graph
validation infrastructure, graph query/filter behavior, serialization, or public
graph APIs.

## Required considerations

- Deterministic identity.
- Canonical representation.
- Equality, ordering, and collision behavior.
- Endpoint compatibility.
- Public API impact.
- Serialization or persisted-data impact, when applicable.
- Unknown, unsupported, and fallback variants.
- Validation and query behavior.
- Regression tests for existing graph behavior.

## Boundary

Do not change parser or producer behavior unless the task explicitly includes
that scope.
