# Define Sprint 37 Rules Engine

Continue OneAgent development.

## Reporting

- Prompt and repository artifacts: English.
- User-visible reports: Russian.

## Profile and template

- `docs/codex/profiles/architecture.md`
- `docs/codex/templates/architecture-task.md`

## Required workflow

`docs/codex/workflows/architecture.md`

## Authoritative documents

- `docs/Roadmap.md`
- `docs/Architecture.md`
- `docs/architecture/semantic-model-2.md`
- `docs/architecture/rules-engine-investigation.md`
- `docs/architecture/diagnostics-engine-evidence.md`
- `docs/adr/0008-semantic-model-2-knowledge-graph.md`
- `docs/adr/0039-workspace-service.md`
- `docs/adr/0042-persistent-cache.md`
- `docs/adr/0058-diagnostics-engine.md`
- `docs/reviews/sprint-36-diagnostics-engine.md`

## Prerequisite

Task 1 is committed, the Rules Engine framework prerequisite remains committed,
and the investigation contains no blocking evidence gap or unapproved
production dependency.

## Task

Create `docs/adr/0059-rules-engine.md` and synchronize only planning-level
architecture text required by the accepted decision. Implement no production
behavior.

## Required decisions

- Fix the first-slice canonical immutable inputs and retain Graph, validation,
  provenance, source-location, and Diagnostics Engine authority.
- Assign one source-independent owner and dependency direction for rule domain,
  registry, dependency planning, configuration, execution, aggregate results,
  Workspace composition, cache/rebuild, Runtime, and projections.
- Fix typed rule identity, registration lifecycle, deterministic enumeration,
  duplicate/conflicting registration behavior, registry bounds, and public
  string compatibility.
- Fix dependency semantics, validation, canonical execution order, missing,
  self, duplicate, incompatible, and cyclic outcomes, and exact bounds.
- Define the first-slice configuration authority, defaults, identity,
  precedence, validation, compatibility, scope, and lifecycle without inventing
  an unsupported external grammar.
- Fix applicability and terminal status vocabulary, execution ownership,
  cancellation, rule failure and dependency-block behavior, continuation versus
  fail-closed policy, completeness, bounds, redacted errors, and deterministic
  aggregate results.
- Define how rule-produced diagnostic evidence maps into ADR-0058 identity,
  family/code/kind, collision, order, suppression, summary, provenance,
  location, bounds, and complete-report behavior. Preserve Graph facts and
  validation results unchanged.
- Define immutable Workspace publication, cache serialization or recomputation,
  invalidation, rebuild/watcher equivalence, supported projections, public API
  migration, and unchanged client/protocol behavior.
- Fix exact repository-owned acceptance evidence for registry, dependency,
  configuration, execution, failure, cancellation, diagnostic integration,
  snapshot/cache/rebuild, public-process compatibility, dependency, scope, and
  complete workspace matrices.
- Record rejected alternatives and defer dynamic plugins, scripts, remote rule
  acquisition, user configuration grammar/UI, source mutation, fixes, safe
  edits, mutable documents, new protocol capabilities, telemetry, and broad
  performance or security claims.

## Acceptance evidence

ADR-0059 is `Accepted`, maps every investigation question to one explicit
decision or deferral, assigns production behavior to Tasks 3–7, identifies all
public consumers and migrations, preserves accepted authority, introduces no
dependency without approval, and agrees with the Roadmap and Sprint 38
boundary.

## Excluded scope

Rust implementation, behavior-encoding fixtures or tests, Cargo changes,
dependency approval, prompt-suite retirement, Sprint completion, Git Change
Adapter work, dynamic plugins, source edits, and product UI.

## Validation

Run ADR/investigation question coverage; input/identity/registration/
dependency/configuration/execution/result/diagnostic consistency;
ownership/dependency/API/cache/protocol compatibility; sensitive-data and
deferred-scope audits; Markdown link checks; `git diff --check`; and
unrelated-change inspection.

## Suggested commit message

`Define Sprint 37 rules engine`

## Final report additions

Report accepted inputs, ownership, rule identity and registry, dependencies and
order, configuration, applicability, lifecycle, failure and cancellation,
results and diagnostic mapping, bounds and errors, Workspace/cache/
compatibility, evidence, rejected alternatives, deferred scope, and unchanged
production behavior.
