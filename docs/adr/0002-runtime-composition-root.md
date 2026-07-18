# ADR-0002: Runtime as Composition Root

## Status

Accepted

## Context

`OneAgent` will expose multiple adapters, including HTTP, MCP, LSP, CLI and IDE integrations.
The platform requires one place that owns dependency construction and application lifecycle.

## Decision

`OneAgent Runtime` is the composition root.

- `main.rs` contains no domain or infrastructure logic.
- `AppBuilder` constructs the application.
- `AppState` contains immutable shared state.
- `Lifecycle` controls explicit state transitions.
- Configuration is supplied through a provider abstraction.
- Dependency-injection frameworks are not used.

## Consequences

- Adapters remain independent from dependency construction.
- Services can be reused by HTTP, MCP, LSP and CLI.
- Runtime startup and shutdown remain testable.
- Additional configuration providers can be introduced without changing consumers.
