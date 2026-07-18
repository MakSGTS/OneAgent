# ADR-0012: BSL Symbols in Semantic Graph

## Status

Accepted

## Context

The semantic graph contains module nodes, while `oneagent-bsl` can extract
top-level procedure and function declarations. These two capabilities must be
connected without coupling the BSL crate to EDT or graph infrastructure.

## Decision

The EDT adapter owns the integration layer.

- `oneagent-bsl` remains independent from EDT and the semantic graph.
- Each module is read and passed to `LineBslDeclarationExtractor`.
- Procedures become `NodeKind::Procedure`.
- Functions become `NodeKind::Function`.
- Modules connect to declarations using `contains`.
- Parsing and graph insertion errors are wrapped by `EdtBslGraphError`.

## Consequences

- The graph gains navigable BSL symbols.
- Future call analysis can attach `calls` edges to existing symbol nodes.
- A future full parser may replace the extractor without changing graph consumers.
