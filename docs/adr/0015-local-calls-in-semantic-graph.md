# ADR-0015: Local BSL Calls in Semantic Graph

## Status

Accepted

## Context

The BSL layer can extract declarations and calls and can resolve calls between
symbols in the same module. The semantic graph does not yet contain call edges.

## Decision

The EDT integration layer adds local call relations to the semantic graph.

- Procedures and functions are inserted before call resolution.
- BSL calls are extracted from the same module source.
- `LocalBslCallResolver` resolves calls between declarations in one module.
- Resolved calls become `EdgeKind::Calls`.
- Unresolved and qualified calls are preserved by the resolver but are not added
  to the graph at this stage.
- Cross-module resolution remains a separate stage.

## Consequences

- Local procedure and function dependencies become graph queries.
- Recursive calls are represented naturally as self-referencing edges.
- Cross-module calls can later be added without changing the local resolution model.