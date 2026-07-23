# Architecture Workflow

Use this workflow for new ADRs, unresolved semantics, endpoint contracts,
identity contracts, production-source decisions, and architecture-only roadmap
work.

## Required analysis

- Identify the authoritative existing ADRs and architecture documents.
- Analyze alternatives only when architecture is unresolved.
- State the chosen outcome explicitly.
- Record rejected alternatives and why they were rejected.
- Define the canonical semantic statement.
- Define direction, identity, endpoint matrix, and scope.
- Define the minimal first production slice.
- Define deferred scope and non-goals.
- Define implementation prerequisites.
- Define validator and provenance contracts when relevant.
- Define Coverage Registry completion criteria when relevant.

## Boundaries

Do not implement production behavior in an architecture task unless the prompt
explicitly includes implementation scope. Do not mark a capability `Supported`
based on architecture documentation alone.
