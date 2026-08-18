# OneAgent Architecture Audit

Point-in-time audit performed manually on 2026-08-18. The repository does not
currently contain an `oneagent-audit.sh` generator; this document must not be
treated as continuously generated state.

## Baseline

- Project: `oneagent`
- Git branch: `main`
- Source baseline before documentation reconciliation: `63d8c4f930b8`
- Rust host: `aarch64-apple-darwin`
- Rust version: `rustc 1.97.1 (8bab26f4f 2026-07-14)`
- Cargo version: `cargo 1.97.1 (c980f4866 2026-06-30)`
- Audit date: 2026-08-18

The audit includes the documentation reconciliation in the working tree. The
untracked `docs/codex/prompts/` directory predates and is outside this audit
change.

## Summary

| Check | Result |
|---|---:|
| Cargo manifests, including workspace root | 12 |
| Workspace packages | 11 |
| Workspace tests listed | 448 |
| Untracked Git entries outside this change | 1 directory |
| Ignored backup artifacts | 4 |
| Duplicate root `runtime/` path | absent |
| TODO/FIXME/HACK markers in Rust source | 0 |
| `cargo fmt --all -- --check` | PASS |
| `cargo check --workspace` | PASS |
| `cargo test --workspace` | PASS |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | PASS |
| `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps` | PASS |

## Workspace packages

| Path | Package | Current responsibility |
|---|---|---|
| `adapters/edt` | `oneagent-edt` | EDT artifact ingestion and semantic graph contribution |
| `adapters/filesystem` | `oneagent-workspace-fs` | Filesystem workspace discovery |
| `apps/cli` | `oneagent-cli` | Package placeholder; supported CLI is planned for Sprint 21 |
| `apps/runtime` | `oneagent-runtime` | Runtime composition, configuration, state, and lifecycle foundation |
| `crates/analysis` | `oneagent-analysis` | Source-independent declaration and call analysis |
| `crates/bsl` | `oneagent-bsl` | BSL and supported query-language parsing |
| `crates/common` | `oneagent-common` | Shared typed primitives |
| `crates/graph` | `oneagent-graph` | Canonical semantic graph and derived analysis APIs |
| `crates/metadata` | `oneagent-metadata` | Typed 1C metadata model |
| `crates/protocol` | `oneagent-protocol` | Protocol package foundation; transport contracts are not implemented |
| `crates/workspace` | `oneagent-workspace` | Workspace and project abstractions |

## Project-goal alignment

| Project goal | Evidence | Assessment |
|---|---|---|
| Modular cross-platform Rust foundation | Cargo workspace, macOS and Windows CI, all quality gates passing | aligned |
| Source-independent semantic model | Typed graph, provenance, validation, query, diff, impact, coverage, and resolution APIs | aligned |
| 1C source ingestion | Implemented EDT and filesystem adapters | aligned for current scope; Designer XML is assigned to Sprint 14 |
| Semantic indexing | Existing resolution index and scan-based Query API | incomplete by design; Sprint 4 boundary is accepted in ADR-0026 |
| Incremental workspace updates | Graph/build diff facilities exist | incomplete by design; incremental index lifecycle belongs to Sprint 5 |
| Long-running product Runtime and APIs | Runtime lifecycle foundation exists; CLI and protocol packages are placeholders | planned in Sprints 15–21 |
| AI, MCP, and IDE integration | Vision and product boundaries are documented | planned in Sprints 22–35; no current implementation claim |
| Git-aware intelligence | No Git adapter exists | explicitly assigned to Sprint 38 |

The implementation is consistent with the early semantic-core phase of the
project vision. The main discrepancies were documentation drift: a duplicated
date-based task sequence, planned integrations described as current, a stale
historical implementation boundary, and this obsolete audit snapshot. Those
items are corrected by the 2026-08-18 roadmap reconciliation.

## Repository hygiene observations

The obsolete root `runtime/` directory and the tracked backup files reported by
the previous audit are gone. Four ignored local `.bak` files remain under
`adapters/edt/src/`:

```text
adapters/edt/src/lib.rs.bak
adapters/edt/src/lib.rs.before-common-module-name.bak
adapters/edt/src/lib.rs.before-production-call-test.bak
adapters/edt/src/module_reader.rs.before-common-module-name.bak
```

They are not tracked and do not affect builds. They should be removed only by
their owner after confirming they are no longer needed.

## Current next actions

1. Execute Sprint 4 against ADR-0026 and record its integration review.
2. Keep release forecasts separate from the dependency-ordered sprint roadmap.
3. Replace CLI, protocol, Runtime, and extension placeholders only in their
   assigned sprints.
4. Re-run this manual snapshot when repository structure or release status
   changes, or add a reviewed audit tool before claiming automatic generation.
