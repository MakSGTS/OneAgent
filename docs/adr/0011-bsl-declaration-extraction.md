# ADR-0011: BSL Declaration Extraction

## Status

Accepted

## Context

The semantic graph contains module nodes, but procedures and functions are not
yet represented.

## Decision

Introduce `oneagent-bsl` as an independent domain-oriented library.

- The first stage extracts top-level procedure and function declarations.
- Russian and English BSL keywords are supported.
- Extracted symbols include stable IDs, names, line numbers and export flags.
- The initial extractor is deliberately line-oriented and does not claim to be
  a complete BSL parser.
- Expressions, scopes, calls, queries and type inference are deferred to later
  parser stages.
- The crate has no dependency on EDT, filesystem, graph, HTTP, MCP or LLM APIs.

## Consequences

- BSL symbol extraction can be tested independently from source adapters.
- EDT and Designer adapters can reuse the same BSL layer.
- The graph integration can remain a separate concern.
- A future parser implementation may replace the extractor behind the same trait.
