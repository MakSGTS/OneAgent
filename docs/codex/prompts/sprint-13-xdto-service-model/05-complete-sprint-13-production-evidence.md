# Task 05: Complete Sprint 13 production evidence

Continue OneAgent development.

## Reporting

- Communicate with the user in Russian.
- Keep code, identifiers, repository documentation, comments, Rustdoc, tests,
  errors, public APIs, prompt text, and commit messages in English.
- Report only live repository evidence or accepted architecture.

## Profile

`docs/codex/profiles/graph-implementation.md`

## Template

`docs/codex/templates/graph-emission-task.md`

Read the Profile, Template, required Core and Workflow modules, and
`docs/codex/README.md` completely before acting.

## Authoritative documents

- `docs/Roadmap.md`, Sprint 13 Task 05;
- `docs/architecture/xdto-service-source-investigation.md`;
- `docs/architecture/semantic-model-2.md`;
- `docs/adr/0023-typed-metadata-payload.md`;
- `docs/adr/0024-reference-request-provenance.md`;
- `docs/adr/0025-references-endpoint-validation.md`;
- `docs/adr/0033-event-subscription-semantics.md`;
- `docs/adr/0035-xdto-service-semantics.md`.

## Required gate

Proceed only when Tasks 01–04 are committed, all implementation validation
passed, live production emits the accepted ADR-0035 slice, and no task-created
uncommitted change remains.

## Task

Complete provenance-backed source, generic consumer, complete/incremental index,
Coverage, aggregate-count, and current-state documentation evidence for
ADR-0035. Do not add another semantic or source capability.

## Source contract / production source

Create one tracked reduced fixture under existing EDT conventions. Its README
must trace every selected `.mdo`, `Package.xdto`, and `Module.bsl` to exact
ignored `OneAgent_EDTproject/src/` paths, source SHA-256, selected/reduced
treatment, and reduced SHA-256. Generated cases may cover negatives and
transitions but cannot replace positive production evidence.

## Scope

One evidence-completion boundary proving the integrated feature through every
affected generic consumer and synchronizing implemented current-state docs.

## Included

- Include small/mixed/large XDTO direct-type shapes, HTTP absent/explicit method
  values, Web internal/external/absent package declarations, internal/external
  types, operations/parameters/directions, and exact handler dispatch.
- Prove metadata payload, child payload/identity/ownership, public request,
  References/Triggers, provenance, diagnostic/statistics, and external policy.
- Prove generic Query navigation for every new kind/owner/relation/request and
  no XDTO/service-specific query authority.
- Prove payload/type/template/method/direction/package/type/handler add/remove/
  modify target behavior in Diff and deterministic reports.
- Prove Validation and Impact policy, reordered/repeated builder equality, and
  existing relation/request compatibility.
- Prove complete and incremental Semantic Index clean-rebuild equivalence for
  new nodes, ownership, payload changes, internal/external request transitions,
  References, Triggers, and deferred-observation changes.
- Add/update Graph Domain and EDT Coverage only where executable evidence
  justifies the accepted first slice; recompute aggregate counts from registries.
- Synchronize `docs/architecture/semantic-model-2.md` and Sprint 13 Roadmap
  current-state text while keeping Sprint 13 `next` or `active`, not completed.

## Excluded

- New graph, parser, identity, payload, resolution, diagnostic, statistics, or
  relation semantics.
- XDTO property/import/restriction nodes, external schema nodes, route/runtime/
  transport/WSDL behavior, Designer XML, or other deferred scope.
- Manual/unproven Coverage rows, Sprint 14 planning/framework updates,
  previous-suite retirement, or unrelated fixes.

## Acceptance criteria

- Every applicable ADR-0035 Graph Domain and EDT Coverage completion criterion
  has executable passing evidence.
- Fixture README proves exact live derivation and hashes for all selected XDTO,
  HTTP, Web, and handler artifacts without hidden ignored-file dependency.
- Fixture proves direct-type categories, service hierarchy, internal/external/
  absent declarations, directions, handler requests, References, and Triggers.
- Query, Diff, reports, Validation, Impact policy, requests, diagnostics/
  statistics, reordered/repeated builds, and unrelated behavior match ADR-0035.
- Complete and incremental indexes match clean rebuilds for every required
  add/remove/modify/ownership/request/reference/dispatch/external transition.
- Coverage statuses, limitations, evidence, and aggregate counts derive from
  live registries and agree across graph, EDT, Semantic Model, Roadmap, and
  review inputs.
- All ADR-0035 deferred source/runtime/Designer XML scope remains explicit and
  unimplemented.
- Full workspace validation succeeds.

## Repository Safety

- Recheck Git state, Task 01–04 range, exact live artifact provenance,
  consumers, indexes, Coverage, docs, tests, and applicable `AGENTS.md`.
- Preserve unrelated user files and prompt suites; do not stage ignored live
  project artifacts or a bulk copy of large schemas.
- Add only the reduced tracked fixture with explicit derivation evidence.
- Do not stage or commit without launch-time authorization; never use broad
  staging or destructive Git commands.

## Task-specific validation

```bash
cargo test -p oneagent-metadata
cargo test -p oneagent-graph
cargo test -p oneagent-edt --test xdto_services
cargo test -p oneagent-edt --test coverage
cargo test -p oneagent-edt --test semantic_index
```

Then run the complete workspace validation from
`docs/codex/core/validation.md`, including `git diff --check`. Report zero-match
filters separately.

## Suggested commit message

```text
Complete Sprint 13 production evidence
```

When authorized, stage only task-owned evidence tests/fixture, Coverage updates,
and synchronized current-state documentation. Do not stage prompts, ignored
live files, Sprint 14 work, or unrelated paths; do not create an empty commit.

## Final report additions

Report acceptance matrix, fixture provenance/hashes, consumer/index
transitions, request/reference/dispatch evidence, external policy, Coverage
statuses/counts, documentation synchronization, deferred scope, files/tests,
validation, commit hash, final Git status, and the Task 06 gate.
