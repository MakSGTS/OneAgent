# Architecture

OneAgent uses a modular Rust workspace centered on a source-independent semantic
graph. The architecture distinguishes the current implementation from planned
product adapters so that roadmap intent is not mistaken for available behavior.

## Current implementation

1. **Shared and domain crates**
   - `oneagent-common` owns shared typed primitives.
   - `oneagent-metadata` owns the typed 1C metadata model.
   - `oneagent-workspace` owns workspace and project abstractions.
   - `oneagent-bsl` owns BSL lexical and syntax analysis.
2. **Semantic core**
   - `oneagent-graph` owns canonical semantic nodes, edges, provenance,
     validation, query, diff, impact, coverage, and resolution APIs.
   - `oneagent-analysis` contributes source-independent declaration and call
     analysis over the BSL and graph contracts.
3. **Source adapters**
   - `oneagent-edt` reads supported EDT artifacts and contributes facts to the
     canonical semantic graph.
   - `oneagent-workspace-fs` discovers supported workspaces through the
     filesystem boundary.
4. **Applications and protocol foundation**
   - `oneagent-runtime` contains composition, configuration, state, and lifecycle
     foundations. It is not yet the long-running Runtime API described for v0.4.
   - `oneagent-cli` is a package placeholder and is not yet a supported client.
   - `oneagent-protocol` is a package foundation and does not yet expose HTTP,
     MCP, or LSP contracts.

`SemanticGraph` is the canonical semantic authority. Adapters may observe source
formats and contribute provenance-backed facts, but source-specific identities
and parser state must not become competing graph truth. Derived facilities such
as query, resolution, reports, diffs, impact analysis, and the Sprint 4 Semantic
Index remain read-only views over graph snapshots.

## Planned boundaries

The roadmap assigns future boundaries explicitly:

- Designer XML ingestion extends the source-adapter layer in Sprint 14.
- Runtime services, HTTP, persistence, and the supported CLI arrive in Sprints
  15–21.
- MCP, VS Code, LSP, and EDT product integration arrive in Sprints 28–35.
- Git change ingestion arrives in Sprint 38 as an input adapter, not a semantic
  authority.

Detailed accepted decisions live in `docs/adr`. The dependency-ordered delivery
sequence and status live only in `docs/Roadmap.md`.
