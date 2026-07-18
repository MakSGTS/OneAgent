# ADR-0014: Local BSL Call Resolution

## Status

Accepted

## Context

BSL calls contain source and target names, but semantic graph edges require
stable identifiers.

## Decision

Introduce local call resolution inside `oneagent-bsl`.

- Declaration names are indexed case-insensitively.
- Calls between declarations in the same module are resolved to stable IDs.
- Qualified targets are deferred to cross-module resolution.
- Calls outside procedures and functions remain unresolved.
- Missing source and target symbols have explicit reason codes.
- The resolver remains independent from EDT and the semantic graph.

## Consequences

- Local `calls` edges can be created deterministically.
- Cross-module resolution can be implemented as a separate stage.
- Unresolved calls remain available for diagnostics rather than being discarded.