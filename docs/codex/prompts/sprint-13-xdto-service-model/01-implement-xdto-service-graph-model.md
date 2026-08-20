# Task 01: Implement the XDTO and service graph model

Continue OneAgent development.

## Reporting

- Communicate with the user in Russian.
- Keep code, identifiers, repository documentation, comments, Rustdoc, tests,
  errors, public APIs, prompt text, and commit messages in English.
- Report only live repository evidence or accepted architecture.

## Profile

`docs/codex/profiles/graph-implementation.md`

## Template

`docs/codex/templates/graph-model-task.md`

Read the Profile, Template, required Core and Workflow modules, and
`docs/codex/README.md` completely before acting.

## Authoritative documents

- `docs/Roadmap.md`, Sprint 13 Task 01;
- `docs/architecture/xdto-service-source-investigation.md`;
- `docs/architecture/semantic-model-2.md`;
- `docs/adr/0006-semantic-graph.md`;
- `docs/adr/0023-typed-metadata-payload.md`;
- `docs/adr/0024-reference-request-provenance.md`;
- `docs/adr/0025-references-endpoint-validation.md`;
- `docs/adr/0033-event-subscription-semantics.md`;
- `docs/adr/0035-xdto-service-semantics.md`.

## Required gate

Proceed only when the investigation, ADR-0035, Sprint 13 Roadmap plan, Semantic
Model planning text, and complete prompt suite are one committed immutable
planning baseline. Sprint 12 must be completed with `pass`, Sprint 13 must be
the unique `next` target, and no task-created uncommitted change may exist.

## Task

Implement the complete source-independent ADR-0035 graph/model prerequisite:
node and payload types, metadata payload additions, identities, public request
categories, endpoint/ownership rules, exhaustive consumers, generic indexes,
and graph-domain Coverage evidence. Do not parse or emit EDT service/XDTO facts.

## Scope

One public graph-model boundary required by every later Sprint 13 task.

## Included

- Add `XdtoType`, `HttpServiceUrlTemplate`, `HttpServiceMethod`,
  `WebServiceOperation`, and `WebServiceParameter` node kinds with stable codes.
- Add closed compatible graph payloads for direct XDTO type family, HTTP route
  template/optional explicit method, Web return/value type declaration,
  optional nillability, and accepted transfer direction.
- Add closed compatible HTTP Service, Web Service, and XDTO Package variants to
  `MetadataSpecificPayload` without changing existing variants or common
  payload behavior.
- Add collision-safe owner/name direct XDTO type identity; preserve UUID-based
  child identities and every existing identity/code.
- Add public `XdtoPackage` and `XdtoType` reference categories; update canonical
  encoding, display, ordering, reports, diffs, validation, and exhaustive
  consumers without changing existing request IDs.
- Extend Contains only with the five ADR-0035 immediate ownership pairs.
- Extend References and Triggers only with the exact ADR-0035 additive pairs.
- Preserve required unique ownership, provenance validation, Query navigation,
  Diff/report distributions, complete and incremental indexes, and Impact
  policy for the new public surface.
- Add exhaustive positive/negative/collision/order/repeated graph-domain tests
  and justified Graph Domain Coverage capabilities/evidence.

## Excluded

- EDT XML parsing, descriptor/artifact joins, builder insertion, handler/XDTO
  resolution, production requests/edges, diagnostics, or statistics.
- EDT fixtures or Coverage transitions.
- XDTO property/import/restriction models, external namespace nodes, transport
  or runtime semantics, Designer XML, persistence, or serialization.
- New EdgeKind or a service-specific Query/Index authority.

## Acceptance criteria

- Every old enum code, identity, payload, request identity, endpoint rule, and
  public result is unchanged.
- Every new node accepts only its matching payload; every wrong kind/payload
  pair returns the existing typed compatibility error pattern.
- Metadata-specific payload variants are accepted only by their exact metadata
  kinds; content changes preserve node identity and produce semantic-content
  modification evidence.
- XDTO identity uses a length-prefixed owner/name tuple and collision tests
  include delimiter-containing components, type-family changes, reordered
  insertion, and repeated construction.
- Contains accepts only the five new immediate pairs; reversed, transitive,
  unrelated, missing-owner, multiple-owner, self-loop, and wrong-owner cases
  remain invalid and deterministically reported.
- References/Triggers accept exactly ADR-0035 additions and retain every
  existing accepted/forbidden pair.
- Public XDTO request categories have stable deterministic encoding, lifecycle,
  candidate, provenance, ledger, Query/report/diff/build-diff/validation, and
  repeated-build behavior.
- Generic Query, Diff, reports, Validation, complete index, and incremental
  clean-rebuild tests cover add/remove/modify/ownership/reference/dispatch.
- Contains and Triggers remain excluded from default dependency Impact
  propagation; References behavior is unchanged except for explicit new facts.
- Graph Domain Coverage has no unsupported claim and full workspace validation
  succeeds.

## Repository Safety

- Recheck Git state, all enum/payload/request consumers, public APIs, indexes,
  tests, Coverage, and applicable `AGENTS.md` before editing.
- Preserve the two unrelated untracked user files and every prompt suite.
- Do not change Cargo dependencies, EDT code/tests, Roadmap, Semantic Model, or
  Coverage outside graph-domain evidence.
- Do not stage or commit without launch-time authorization; never use broad
  staging or destructive Git commands.

## Task-specific validation

```bash
cargo test -p oneagent-metadata
cargo test -p oneagent-graph
```

Then run the complete workspace validation from
`docs/codex/core/validation.md`, including `git diff --check`. Report zero-match
filters separately.

## Suggested commit message

```text
Implement Sprint 13 XDTO and service graph model
```

When authorized, stage only task-owned metadata/graph model, consumer, Coverage,
and graph-domain test paths. Do not stage EDT files, prompts, planning docs, or
unrelated files; do not create an empty commit.

## Final report additions

Report public API/model impact, stable codes and identities, payload
compatibility, request categories, endpoint matrices, generic consumer/index/
Impact behavior, Coverage evidence, files/tests, validation, commit hash, final
Git status, and the Task 02 gate.
