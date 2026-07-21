# OneAgent Roadmap

## Vision

**OneAgent** — кроссплатформенная платформа интеллектуальной разработки для `1C:Enterprise`, основанная на семантической модели конфигурации, графе знаний и интеграции с локальными и облачными LLM.

## Product roadmap

```mermaid
gantt
    title OneAgent Roadmap
    dateFormat  YYYY-MM-DD
    axisFormat  %b %Y

    section v0.1 Foundation
    Cargo workspace and quality gates       :done, f1, 2026-07-01, 7d
    Runtime foundation                      :done, f2, after f1, 5d
    Workspace discovery                     :done, f3, after f2, 5d
    EDT configuration reader                :done, f4, after f3, 5d
    Metadata domain model                   :done, f5, after f4, 5d

    section v0.2 Semantic Core
    Typed semantic graph                    :done, s1, 2026-07-20, 7d
    EDT metadata object descriptors         :done, s2, after s1, 5d
    EDT module nodes                        :done, s3, after s2, 5d
    BSL declaration extraction              :done, s4, after s3, 5d
    Local call extraction and resolution    :active, s5, after s4, 8d
    Cross-module call resolution            :s6, after s5, 8d
    Semantic index                          :s7, after s6, 12d
    Incremental indexing                    :s8, after s7, 12d

    section v0.3 1C Knowledge Model
    Attributes and tabular sections         :k1, 2026-09-01, 10d
    Forms and commands                      :k2, after k1, 12d
    Registers and queries                   :k3, after k2, 15d
    Roles and access rights                 :k4, after k3, 12d
    Subsystems and composition              :k5, after k4, 8d
    Event subscriptions                     :k6, after k5, 8d
    SKD and report model                    :k7, after k6, 15d
    XDTO and service model                  :k8, after k7, 12d

    section v0.4 Runtime API
    Runtime service container               :r1, 2026-11-15, 8d
    HTTP API and health endpoint            :r2, after r1, 8d
    Workspace service                       :r3, after r2, 10d
    Graph query API                         :r4, after r3, 12d
    File watching                           :r5, after r4, 10d
    Persistent cache                        :r6, after r5, 12d

    section v0.5 AI Integration
    Context engine                          :a1, 2027-01-15, 15d
    LLM provider abstraction                :a2, after a1, 10d
    OpenAI-compatible provider              :a3, after a2, 8d
    LM Studio integration                   :a4, after a3, 8d
    Ollama integration                      :a5, after a4, 8d
    Tool execution policy                   :a6, after a5, 12d

    section v0.6 MCP and IDE
    MCP server                              :m1, 2027-03-15, 15d
    MCP semantic tools                      :m2, after m1, 15d
    VS Code extension foundation            :m3, after m2, 15d
    Navigation and symbol search            :m4, after m3, 12d
    AI chat and context panel               :m5, after m4, 12d
    EDT integration prototype               :m6, after m5, 20d

    section v0.7 Intelligence
    Diagnostics engine                      :i1, 2027-06-01, 15d
    Rules engine                            :i2, after i1, 15d
    Change impact analysis                  :i3, after i2, 15d
    Refactoring planner                     :i4, after i3, 20d
    Safe edit transactions                  :i5, after i4, 15d

    section v1.0
    Public API stabilization                :v1, 2027-09-01, 20d
    Plugin SDK                              :v2, after v1, 20d
    Performance and security hardening      :v3, after v2, 20d
    Documentation and examples              :v4, after v3, 15d
    OneAgent 1.0 release                    :milestone, v5, after v4, 1d
```

## Release goals

### v0.1 — Foundation
Stable Rust workspace, Runtime composition root, cross-platform CI, EDT workspace discovery, base metadata model.

### v0.2 — Semantic Core
Typed semantic graph, real EDT UUIDs, module/procedure/function nodes, local and cross-module call graph, semantic and incremental indexes.

### v0.3 — 1C Knowledge Model
Attributes, tabular sections, forms, commands, registers, queries, roles, access rights, subsystems, SKD, XDTO and services.

### v0.4 — Runtime API
Long-running runtime, workspace lifecycle, file watching, graph-query API and persistent cache.

### v0.5 — AI Integration
Context engine, LLM abstraction, LM Studio, Ollama and OpenAI-compatible endpoints.

### v0.6 — MCP and IDE
MCP server, VS Code extension, EDT integration, navigation and semantic tools.

### v0.7 — Intelligence
Diagnostics, impact analysis, planning, refactoring and safe edit transactions.

### v1.0 — Stable Platform
Stable APIs, plugin SDK, performance/security hardening, documentation and examples.

## Sprint 3 Semantic Coverage

- [x] Add a deterministic Semantic Coverage Audit for graph-domain and EDT-specific capabilities.
- [ ] Complete Semantic Coverage; the audit does not mark completion itself.

Ordered follow-up work:

1. **Critical — completed:** retain unresolved BSL calls as typed diagnostics and resolution statistics.
2. **High — completed:** discover and emit top-level Common Command metadata (`metadata_entity.command`); typed payload completion remains Medium.
3. **High — completed:** classify generic top-level Form entity and node capabilities as not applicable to EDT; real common and subordinate forms use distinct semantic kinds.
4. **High — completed:** discover and emit top-level Common Template metadata (`metadata_entity.template`); typed payload completion remains Medium.
5. **High — completed:** classify fallback-only `metadata_entity.unknown` as not applicable to EDT without emitting synthetic entities.
6. **High — completed:** map EDT accounting-register resources to stable, provenance-backed `Measure` nodes (`semantic_node.measure`); ownership coverage remains separate.
7. **High — next:** determine applicability of `semantic_node.metadata.unknown`, the first remaining typed gap.
8. **High:** emit EDT `StandardAttribute` nodes, ownership, and provenance.
9. **High:** review Measure ownership evidence independently from node emission.
10. **High:** preserve tabular-section ownership for nested attributes.
11. **High:** implement producer-specific support for each declared-only semantic edge.
12. **Medium:** define and preserve complete typed metadata payloads.
13. **Medium:** add successful fixtures for every mapped metadata reference target kind.
14. **Medium:** decide and implement reference-request provenance ownership.
15. **Medium:** replace permissive endpoint validation as new edge producers are added.

The EDT Coverage Registry currently contains 14 High gaps and 44 Medium gaps.
Sprint 3 Integration Review remains blocked until all High gaps are resolved or
correctly classified as not applicable.

The detailed capability inventory, missing evidence, acceptance criteria, and
out-of-scope boundaries are recorded in `docs/architecture/semantic-model-2.md`.

## Definition of Done

- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo doc --workspace --no-deps`
- Architecture changes documented by ADR.
- Public APIs documented.
- Tests cover success and failure paths.
