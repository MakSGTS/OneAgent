# ADR-0003: Semantic Domain Model

## Status

Accepted

## Context

General-purpose coding agents operate on files and text. Large `1C:Enterprise`
configurations produce excessive context when raw XML and module files are sent
directly to a language model.

## Decision

`OneAgent` introduces a semantic domain model independent from source formats.

- `oneagent-common` owns validated identifiers and names.
- `oneagent-metadata` owns metadata kinds, objects and the semantic tree.
- `oneagent-workspace` owns local workspaces and configurations.
- Domain entities do not depend on HTTP, MCP, LSP, IDE APIs or LLM providers.
- Source adapters will translate EDT and Designer XML into this model.

## Consequences

- Language models can receive compact structured information.
- EDT and Designer exports share one internal representation.
- Search, indexing and dependency analysis can be implemented independently
  from IDE integrations.
- Unsupported metadata kinds can be added incrementally.
