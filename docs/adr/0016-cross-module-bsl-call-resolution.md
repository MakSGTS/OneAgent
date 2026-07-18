# ADR-0016: Cross-Module BSL Call Resolution

## Status

Accepted

## Context

Local call resolution supports calls between declarations in one module, but
qualified expressions such as `AccessManagement.CheckRights()` require a
configuration-wide symbol index.

## Decision

Introduce cross-module call resolution in `oneagent-bsl`.

- Modules expose stable IDs, names and declarations through `BslModuleSymbols`.
- Qualified calls use the `Module.Symbol` form.
- Module and symbol lookup is case-insensitive.
- Only exported target procedures and functions may be resolved.
- Local and cross-module resolution remain separate stages.
- The resolver remains independent from EDT and the semantic graph.

## Consequences

- Common-module calls can be resolved to stable declaration IDs.
- Visibility rules are enforced before graph edges are created.
- More complex qualified expressions remain explicitly unresolved.