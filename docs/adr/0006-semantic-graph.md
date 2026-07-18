# ADR-0006: OneAgent Semantic Graph

## Status

Accepted

## Context

`OneAgent` needs to answer semantic questions about large `1C:Enterprise`
configurations without repeatedly scanning raw XML and BSL files.

## Decision

Introduce a typed directed graph named `OneAgent Semantic Graph`.

- Nodes represent metadata objects, modules, procedures, functions, queries,
  forms, commands, attributes and other semantic entities.
- Edges represent relations such as `contains`, `calls`, `references`, `reads`,
  `writes`, `grants`, `includes`, `extends` and `depends_on`.
- Node and edge types are explicit enums.
- Edges may only be inserted when both endpoint nodes exist.
- The graph is deterministic by using ordered collections.
- The graph crate remains independent from HTTP, MCP, LSP, IDE and LLM APIs.

## Consequences

- Dependency queries become graph traversals rather than repeated text search.
- Multiple source adapters may populate the same graph.
- Deterministic ordering simplifies tests, serialization and future caching.
- More advanced traversal and indexing may be added without changing domain
  entities.
