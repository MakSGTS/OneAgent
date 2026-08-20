# Task 04: Emit XDTO and service semantics

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

- `docs/Roadmap.md`, Sprint 13 Task 04;
- `docs/architecture/xdto-service-source-investigation.md`;
- `docs/architecture/semantic-model-2.md`;
- `docs/adr/0007-edt-to-semantic-graph.md`;
- `docs/adr/0008-edt-metadata-object-reader.md`;
- `docs/adr/0023-typed-metadata-payload.md`;
- `docs/adr/0024-reference-request-provenance.md`;
- `docs/adr/0025-references-endpoint-validation.md`;
- `docs/adr/0033-event-subscription-semantics.md`;
- `docs/adr/0035-xdto-service-semantics.md`.

## Required gate

Proceed only when Tasks 01–03 are committed, focused/full validation passed,
the graph contract and both parser families agree with ADR-0035, and no
task-created uncommitted change remains.

## Task

Integrate the committed parsers into the production EDT graph builder, enrich
existing metadata nodes, emit accepted owned child nodes, migrate accepted
package/type/callable declarations to public requests, resolve them after BSL
symbol insertion, and project exact References/Triggers or typed diagnostics.

## Source contract / production source

Use existing top-level XDTO/HTTP/Web discovery and module/symbol production.
Join each discovered object only to its committed parser result. Do not reparse
generic metadata, duplicate module identities, or create an adapter-local graph
authority.

## Scope

One production projection from committed source models to committed graph,
request, provenance, diagnostic, and statistics contracts.

## Included

- Invoke XDTO/service parsers only for their exact metadata kinds and enrich the
  existing metadata node payload without changing UUID/name/kind/ownership.
- Insert direct XDTO Type, HTTP URL Template/Method, and Web Operation/Parameter
  nodes with committed payload/identity and exact immediate Contains edges.
- Preserve existing service Module and Procedure/Function nodes unchanged.
- Collect public XDTO Package, XDTO Type, and Callable requests with collection
  provenance before resolution; distinguish internal from external declarations.
- Resolve package by exact metadata name, type by exact namespace/package and
  owned child name, and HTTP/Web handlers by exact Function under the owning
  service Module after BSL symbols exist.
- Project resolved requests to exact ADR-0035 References; project resolved
  handlers additionally to Triggers with relation-specific provenance.
- Project missing, ambiguous, incompatible, and invalid-owner internal outcomes
  to typed diagnostics and derive legacy statistics exactly once from terminal
  public requests.
- Preserve valid external package/type declarations as payload/source outcomes
  with no request, candidate, placeholder, edge, or false missing diagnostic.
- Treat fatal parser/join failures as complete build failure while preserving
  accepted siblings for deferred/non-fatal constructs.
- Add generated projects covering positive internal/external/absent cases,
  every terminal resolution outcome, fatal/deferred behavior, payload/target
  changes, source/filesystem ordering, and repeated builds.

## Excluded

- New graph or parser semantics beyond Tasks 01–03.
- Final provenance-backed reduced fixture, broad generic consumer/index/Coverage
  matrix, or current-state documentation completion.
- XDTO properties/import dependencies/external nodes, route matching, transport,
  WSDL/SOAP, publication/runtime behavior, Designer XML, or BSL body analysis.
- Coverage transitions or unrelated refactors.

## Acceptance criteria

- Existing top-level metadata, configuration ownership, payload synonyms,
  service modules, symbols, Calls, and unrelated facts remain unchanged.
- Valid internal package declarations resolve to metadata XDTO Packages; valid
  internal type declarations resolve to exact owned XDTO Types; all live HTTP
  and Web handlers resolve to exact owned Functions.
- Each accepted child has one immediate owner and non-empty deterministic
  declaration/ownership provenance; no reverse/transitive/placeholder fact is
  emitted.
- Every accepted internal declaration produces one terminal public request with
  collection and resolution provenance; duplicates aggregate deterministically.
- Resolved handlers emit both References and Triggers; type/package resolutions
  emit only References; no relation is emitted for any non-resolved request.
- External declarations remain inspectable typed payload/source facts but do not
  enter the request ledger, statistics, diagnostics, candidates, or graph as
  local targets.
- Missing, ambiguous, incompatible, and invalid-owner generated cases have exact
  request state/outcome/candidates, diagnostic code/kind/provenance, and derived
  statistics with no successful edge.
- Fatal source errors return no successful partial build; deferred constructs
  retain accepted siblings without Unknown or guessed nodes.
- Query, request report, and Validation observe exact facts; Contains/Triggers
  Impact policy remains non-propagating.
- Reordered input and repeated builds produce equal graph, payload, requests,
  provenance, diagnostics, statistics, report, and validation.
- EDT Coverage does not transition in this task and full workspace validation
  succeeds.

## Repository Safety

- Recheck Git state, Task 01–03 commits, builder phases, module/symbol timing,
  request ledger, diagnostics/statistics, tests, Coverage, and `AGENTS.md`.
- Preserve unrelated user files, ignored live source, and all prompt suites.
- Do not add dependencies, copy full live schemas into tracked tests, or change
  unrelated producers.
- Do not stage or commit without launch-time authorization; never use broad
  staging or destructive Git commands.

## Task-specific validation

```bash
cargo test -p oneagent-edt --test xdto_services
cargo test -p oneagent-graph --test validation
cargo test -p oneagent-graph --test reference_request_build
```

Then run the complete workspace validation from
`docs/codex/core/validation.md`, including `git diff --check`. Report zero-match
filters separately.

## Suggested commit message

```text
Emit Sprint 13 XDTO and service semantics
```

When authorized, stage only task-owned EDT builder/emission/resolution code,
generated production tests/support, and necessary diagnostics/statistics
updates. Do not stage final fixtures, Coverage/docs, prompts, ignored live
artifacts, or unrelated paths; do not create an empty commit.

## Final report additions

Report production phase/order, metadata enrichment, node/ownership inventory,
request/resolution matrices, internal/external policy, provenance, diagnostics/
statistics, determinism, Coverage non-transition, files/tests, validation,
commit hash, final Git status, and the Task 05 gate.
