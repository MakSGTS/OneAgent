# Subsystem Hierarchy Source Investigation

## Purpose

This record supplies the repository-owned source and implementation evidence
needed to plan Sprint 10 Subsystems and Composition. It does not change graph
semantics, production discovery, parser behavior, Coverage status, or public
APIs.

## Baseline

The investigation was prepared from committed Sprint 9 completion head
`4ff5d5cf0788c732fcb194ecac816e2e9e2ed34d`. The working tree also contained
the unrelated untracked files `docs/codex/prompts/run-next-sprint.md` and
`docs/roadmap-calendar-forecast.md`; neither file is evidence owned by this
record.

Sprint 9 records `pass` in
[its integration review](../reviews/sprint-9-roles-access-rights.md), and the
[Roadmap](../Roadmap.md) identifies Sprint 10 Subsystems and Composition as the
unique next planning target.

## Evidence levels

- **Confirmed** means directly observed in committed repository source, tests,
  fixtures, implementation, or Git history.
- **Accepted** means required by an accepted ADR or Semantic Model contract.
- **Unknown** means the repository does not yet establish a safe production
  rule.

## Confirmed EDT source inventory

The repository-local representative source tree under
`OneAgent_EDTproject/src/Subsystems/` contains 127 parseable
`mdclass:Subsystem` descriptors:

| Dimension | Confirmed value |
|---|---:|
| Top-level descriptors | 13 |
| Nested descriptors | 114 |
| Maximum hierarchy depth | 5 |
| Depth 1 / 2 / 3 / 4 / 5 descriptors | 13 / 64 / 39 / 9 / 2 |
| Nested descriptors with exactly one direct `parentSubsystem` | 114 |
| Top-level descriptors with `parentSubsystem` | 0 |
| Direct `subsystems` declarations | 114 |
| Physical immediate nested descriptor directories | 114 |
| Parent descriptors whose direct declarations differ from physical children | 0 |
| Repeated child names inside one `subsystems` list | 0 |
| Nested `parentSubsystem` values differing from the physical ancestor chain | 0 |

The root descriptor
`OneAgent_EDTproject/src/Subsystems/DNSCore/DNSCore.mdo` demonstrates direct
child declarations such as `<subsystems>Common</subsystems>`. The child
descriptor
`OneAgent_EDTproject/src/Subsystems/DNSCore/Subsystems/Common/Common.mdo`
declares `<parentSubsystem>Subsystem.DNSCore</parentSubsystem>`. The deeper
descriptor
`OneAgent_EDTproject/src/Subsystems/EquipmentSupport/Subsystems/Peripherals/Subsystems/ReceiptPrinters/ReceiptPrinters.mdo`
declares
`Subsystem.EquipmentSupport.Subsystem.Peripherals` and directly names its
`CashRegisterShift` child.

The qualified parent vocabulary is therefore confirmed by the live source as alternating
`Subsystem` and exact local-name components. Physical nesting uses a repeated
`Subsystems/<LocalName>` directory pair. All inspected descriptors preserve a
UUID and direct `<name>` value. Six local names are non-unique across different
parents (`Bank`, `Core`, `Delivery`, `MonthEndClosing`, `Print`, and
`UserMonitoring`), so global local-name resolution cannot identify nested
Subsystems safely.

The repository root `.gitignore` excludes `OneAgent_EDTproject/`. These files
are explicitly selected as real-source evidence by the Sprint bootstrap and are
available in the live workspace, but they are not a committed fixture or an
immutable future-chat baseline. Production completion therefore requires a
tracked reduced fixture whose README records exact source paths, selected
fragments, and source hashes. The counts above are planning-time investigation
evidence and must be rechecked before deriving that fixture.

Nested descriptors also contain direct `<content>` declarations. Their XML
shape is the same direct `mdclass:Subsystem/content` field already accepted for
top-level membership by [ADR-0020](../adr/0020-includes-semantics.md), but the
current production reader never receives nested descriptors.

## Confirmed implementation boundary

`FileSystemEdtSemanticGraphBuilder` reads only immediate object directories
under each supported top-level metadata directory. `Subsystems` maps to
`MetadataKind::Subsystem`, but `collect_top_level_metadata` does not descend
through nested `Subsystems` directories.

For every discovered top-level Subsystem, the current builder:

- inserts the existing `NodeKind::Metadata(MetadataKind::Subsystem)` node;
- inserts a flat `NodeKind::Subsystem` node with identity
  `<metadata UUID>:subsystem`;
- retains configuration ownership only for the metadata object node;
- parses only direct `<content>` through `EdtSubsystemContentReader`;
- resolves allowlisted content after all top-level graph nodes exist;
- emits direct `NodeKind::Subsystem --Includes--> NodeKind::Metadata(kind)`;
- deliberately rejects `Subsystem.<...>` content as recognized but deferred.

The production and test entry points are concentrated in:

- `adapters/edt/src/lib.rs`;
- `adapters/edt/src/metadata_object.rs`;
- `adapters/edt/src/subsystem_content.rs`;
- `adapters/edt/tests/includes.rs`;
- `adapters/edt/tests/coverage.rs`.

The metadata object reader accepts any object directory containing exactly one
`.mdo` descriptor and already preserves UUID, name, kind, synonym, descriptor
path, and existing auxiliary observations. Its public documentation calls the
input top-level, but the parsing mechanics do not encode hierarchy.

## Accepted compatibility constraints

[ADR-0020](../adr/0020-includes-semantics.md) accepts `Includes` as direct,
declared composition membership and excludes transitive closure from persisted
graph facts. It also explicitly defers nested discovery, hierarchy sources,
Subsystem targets, and transitive membership. The current first-slice endpoint
matrix accepts only flat Subsystem sources and allowlisted metadata-object
targets.

[ADR-0025](../adr/0025-references-endpoint-validation.md) requires every
supported edge family to use an explicit endpoint matrix. The current validator
rejects a flat Subsystem target for `Includes`, so hierarchy production cannot
be enabled without a preceding graph contract and focused negative tests.

The [Semantic Model](semantic-model-2.md) requires stable UUID-derived node
identity, deterministic edge identity and provenance, canonical storage,
source-independent validation, generic query visibility, deterministic Diff and
Impact behavior, and clean complete/incremental index equivalence. It currently
describes nested discovery, hierarchy, and transitive membership as deferred.

Existing top-level node IDs, metadata node ownership, direct content Includes,
provenance, diagnostics, reference statistics, query ordering, dependency and
Impact exclusions, and Coverage aggregates are compatibility baselines.

## Source agreement and failure matrix

The real source provides two mutually confirming direct hierarchy projections:

| Source projection | Confirmed meaning | Required failure evidence before production |
|---|---|---|
| Parent `<subsystems>ChildName</subsystems>` | The declaring descriptor names an immediate child | Missing directory, extra directory, duplicate declaration, invalid name, and ambiguous child candidates |
| Child `<parentSubsystem>Subsystem.A[.Subsystem.B...]</parentSubsystem>` | The child names its complete parent path | Missing field, multiple fields, malformed components, wrong prefix, unresolved component, incompatible physical parent, and parent cycle |
| Physical `Subsystems/<Name>` nesting | Repository layout identifies the immediate containing descriptor directory | Non-directory entries, missing descriptor, multiple descriptors, unreadable path, reordered enumeration, and symlink/escape handling under existing filesystem safety rules |

The inspected production corpus has no disagreement, missing parent, duplicate
child declaration, or cycle. That absence is positive consistency evidence, not
proof that malformed workspaces are impossible. Generated fixtures must cover
every rejected or recoverable case accepted by the architecture decision.

## Consumer and testability inventory

The current graph already exposes stable generic operations needed to observe
direct hierarchy edges: all-edge, edge-kind, incoming, and outgoing queries;
edge identity; validation; Diff; reports; and complete or incremental indexes.
No existing consumer computes transitive Subsystem membership.

The production test oracle can observe:

- exact nested metadata and flat Subsystem IDs from repository UUIDs;
- direct parent-child edges and provenance from both XML projections and path;
- nested direct content Includes;
- repeated and reordered build equality;
- duplicate local-name disambiguation by full ancestor path and UUID;
- query traversal without persisting derived closure;
- removal, reparenting, content changes, and clean-rebuild index equivalence;
- unchanged top-level and unrelated semantic facts;
- typed fatal or recoverable outcomes for malformed source, as accepted later.

The live 127-descriptor project is the real-source provenance input selected by
the bootstrap, not a committed test oracle. A tracked reduced fixture with a
documented derivation record is required for repeatable positive, negative, and
transition tests, following existing representative fixture conventions.

## Unknowns requiring an architecture decision

- Whether direct Subsystem hierarchy uses `Includes` between flat Subsystem
  nodes or introduces another representation. Existing evidence favors reuse,
  but ADR-0020 does not authorize it.
- Which of the two confirming XML projections is canonical when a malformed
  workspace disagrees, and whether disagreement is fatal or recoverable.
- The exact public query surface, traversal order, cycle defense, and inclusion
  of each Subsystem's own direct metadata members in transitive membership.
- Whether nested metadata Subsystem objects retain configuration ownership
  exactly like top-level metadata objects. The existing configuration inventory
  model favors compatibility, but the hierarchy relation must remain distinct.
- Whether hierarchy facts participate in Impact propagation. Existing Includes
  behavior excludes them, and no source evidence justifies changing that
  policy during Sprint 10.
- Whether Coverage needs a new hierarchy capability or only expanded evidence
  for existing Subsystem and Includes capabilities. Registry changes require an
  accepted capability boundary and executable evidence.

## Decision-ready smallest slice

The repository contains enough evidence to accept and test one bounded Sprint
10 slice:

1. recursively discover only `mdclass:Subsystem` descriptors through the
   repository-proven `Subsystems/<Name>` layout;
2. validate the matching direct `subsystems` and qualified `parentSubsystem`
   declarations without inferring absent hierarchy;
3. preserve existing UUID-derived metadata and flat Subsystem identities;
4. represent only direct hierarchy facts in canonical storage;
5. reuse the current direct content contract for every successfully discovered
   nested Subsystem;
6. compute deterministic transitive membership as a read-only query projection,
   never as persisted closure;
7. retain existing dependency and Impact exclusions unless a later decision
   supplies separate evidence.

Command-interface navigation, inferred directory-only hierarchy, unsupported
content prefixes, subsystem aliases, cross-project hierarchy, cyclic recovery,
and unrelated metadata-family expansion remain outside this slice.

## Framework readiness

The existing investigation, architecture, parser implementation, graph
implementation, graph model, graph emission, review, sprint-planning, and
sequential-execution contracts express the required evidence, safety,
validation, and reporting boundaries. No reusable Codex Framework gap is
confirmed for Sprint 10.
