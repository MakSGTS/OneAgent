# ADR-0007: EDT to Semantic Graph Mapping

## Status

Accepted

## Context

`OneAgent` can load an EDT configuration descriptor and represent semantic
relations in `OneAgent Semantic Graph`. The platform now needs a deterministic
mapping from EDT source layout to initial graph nodes.

## Decision

The EDT adapter builds the initial graph from project structure.

- The configuration becomes the root graph node.
- Supported top-level EDT directories become metadata object nodes.
- The configuration is connected to each discovered object by a `contains` edge.
- Stable object identifiers combine metadata kind, configuration identifier and
  source directory name.
- Unsupported directories are ignored rather than represented as unknown nodes.
- Nested forms, commands, modules and metadata properties are deferred to later
  indexing stages.

## Consequences

- A real EDT project can produce a useful graph before full XML parsing exists.
- Object discovery is deterministic and inexpensive.
- Future readers may enrich existing nodes with UUID values and nested entities.
- Directory naming assumptions remain isolated in the EDT adapter.
