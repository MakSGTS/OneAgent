# Implement Sprint 37 Rule Registry

Continue OneAgent development.

## Reporting

- Prompt and repository artifacts: English.
- User-visible reports: Russian.

## Profile and template

- `docs/codex/profiles/rules-engine-implementation.md`
- `docs/codex/templates/rules-engine-task.md`

## Required workflows

- `docs/codex/workflows/implementation.md`
- `docs/codex/workflows/rules-engine.md`

## Authoritative documents

- `docs/Roadmap.md`
- `docs/Architecture.md`
- `docs/architecture/semantic-model-2.md`
- `docs/architecture/rules-engine-investigation.md`
- `docs/adr/0008-semantic-model-2-knowledge-graph.md`
- `docs/adr/0058-diagnostics-engine.md`
- `docs/adr/0059-rules-engine.md`

## Prerequisite

Task 2 is committed and ADR-0059 is accepted with no blocking domain,
dependency, or migration question.

## Task

Implement only the ADR-0059 source-independent rule domain and deterministic
registry. Do not plan dependencies or execute rules yet.

## Required behavior and evidence

- Place every public type and invariant in the exact ADR-selected owner and
  preserve dependency direction and existing Graph/Diagnostics authority.
- Implement the accepted validated rule identity, metadata, registration
  representation, registry owner, construction lifecycle, accessors, equality,
  total order, bounds, and closed redacted registry errors.
- Enforce accepted duplicate and conflicting-registration behavior
  independently from insertion, iterator, scheduler, filesystem, or hash order.
- Expose only the accepted deterministic immutable registry view. Do not add
  mutable global registration, dynamic loading, scripts, plugins, remote
  discovery, configuration, dependency planning, or execution.
- Add focused tests for empty/single/multiple registries, every vocabulary
  member, invalid identities and metadata, exact duplicate, conflicting
  registration, reordered construction, exact/over count and component bounds,
  stable enumeration, error redaction, and repeated construction.
- Update only public exports and Rustdoc required by the domain. Preserve
  existing Graph, Analysis, diagnostics, Workspace, cache, Runtime, protocol,
  adapter, and Coverage behavior.

## Excluded scope

Dependency graph validation or ordering, configuration, applicability,
execution, cancellation, rule results, diagnostic production, Workspace/cache
composition, protocol/UI changes, new production dependencies, current-state
documentation, and Sprint completion.

## Validation

Run non-zero focused rule identity/metadata/registration/duplicate/conflict/
ordering/bound/redaction tests and affected package/API/Rustdoc checks, then:

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets --all-features -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
git diff --check
```

## Suggested commit message

`Implement Sprint 37 rule registry`

## Final report additions

Report domain owner and public API, identity/metadata/registry/duplicate/bound/
error behavior, deterministic enumeration, exact focused tests and counts,
dependency impact, full-gate results, and deferred planning/execution.
