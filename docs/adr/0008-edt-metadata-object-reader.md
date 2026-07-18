# ADR-0008: EDT Metadata Object Reader

## Status

Accepted

## Context

The initial EDT graph builder identifies top-level objects from directory names.
Directory names are useful for discovery but are not stable semantic identities.

## Decision

Introduce a universal top-level EDT metadata descriptor reader.

- The reader locates exactly one `.mdo` file in an object directory.
- UUID, metadata name and synonym are extracted from XML.
- The UUID becomes the stable `EntityId`.
- EDT-specific parsing remains inside `oneagent-edt`.
- Missing and ambiguous descriptors are explicit errors.
- Graph integration is performed in a separate step.

## Consequences

- Renaming a source directory does not change semantic object identity.
- Graph nodes can use actual EDT UUID values.
- The same reader supports documents, catalogs, registers, reports, roles and
  other top-level metadata kinds.
- Nested forms and commands require dedicated readers.
