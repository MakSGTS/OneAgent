# ADR-0063: Refactoring Planner

## Status

Accepted.

## Context

Sprint 40 must produce one bounded deterministic semantic refactoring plan and
preview without changing source or implying permission to change it. The
[investigation](../architecture/refactoring-planner-investigation.md) confirms
that Graph already owns callable identity, containment, calls, queries, diff,
and impact, while the current adapters and Workspace publication retain neither
exact immutable BSL content nor complete declaration and call ranges.

The smallest repository-backed family is a rename of one top-level BSL
`Procedure` or `Function` plus every supported direct call occurrence that
resolves uniquely to it in one complete Configuration. EDT currently proves
local and qualified call resolution; Designer XML currently proves only
declarations. Neither format yet proves exact editable occurrences or
cross-format plan equality. Tasks 3-4 must close that internal prerequisite
before planner evaluation begins.

Planning and preview are evidence only. They are not source edits, code
actions, transactions, authorization, or a promise that a later apply will
succeed. This decision therefore preserves the mutation boundary in the
[Refactoring and Safe Edits workflow](../codex/workflows/refactoring-safe-edits.md)
for Sprint 41.

## Decision

### Authority, owner, and dependency direction

`oneagent-analysis::refactoring` owns the source-independent immutable source
document, occurrence, request, target, precondition, operation, plan, preview,
summary, completeness, bound, and closed failure contracts. It also owns the
pure planner and preview evaluation. The accepted dependency direction is:

```text
oneagent-common source primitives + oneagent-bsl lexical/resolution rules
    + oneagent-graph semantic identity/query authority
        -> oneagent-analysis::refactoring contracts and evaluation
            -> EDT and Designer XML evidence producers
                -> Runtime Workspace publication
                    -> MCP projection through Tool Policy
```

Adapters parse and capture source evidence but do not decide plan semantics.
Runtime owns source lifecycle and atomic publication but does not evaluate
rename conflicts. MCP owns only the wire projection and authorization check.
No reverse dependency into Graph, transport-owned domain type, or planner
implementation in an adapter, Runtime handler, or client is accepted.

Graph remains the sole authority for Configuration, Module, Procedure, and
Function identity; ownership; `Calls`; and deterministic queries. The planner
uses `SemanticGraphQuery` and the BSL-owned callable identity constructor and
name-equivalence function. It must not rebuild a semantic graph, infer identity
from a path, scan Graph storage directly, or use diagnostics, impact, Git
status, repository paths, or model output as semantic authority. Existing
Graph diff, impact, diagnostics, rules, validation, and Coverage semantics do
not change.

Tasks 3-4 may add internal crate dependencies in this direction, but they must
not add a third-party production package. The dependency-free SHA-256 already
proven by Designer source provenance may move to one canonical reusable owner;
there must not be two content-version implementations.

### First family and supported matrix

The only accepted family is `bsl_callable_rename_v1`. One request selects
exactly one top-level `NodeKind::Procedure` or `NodeKind::Function` in exactly
one Configuration and replaces:

1. its one exact declaration identifier; and
2. every exact accepted direct call identifier mapped uniquely to that target
   inside the same complete Configuration publication.

The supported matrix is:

| Source format | Target declaration | Local direct call | Qualified direct call |
|---|---|---|---|
| EDT | Top-level Procedure or Function in an accepted BSL module | Accepted | Accepted only when it resolves uniquely inside the Configuration |
| Designer XML | Top-level Procedure or Function in accepted Object, Manager, or Common BSL module roles | Accepted after paired Task 4 evidence | Accepted after paired Task 4 evidence and only for an exported target |

An exported target admits both unique local calls and unique calls of the
lexical form `ModuleName.CallableName(...)`. A non-exported target admits only
unique local calls. The qualifier and surrounding expression are unchanged;
only the final callable identifier is an operation range.

Dynamic execution, strings, comments, query text, reflection, computed member
access, callbacks, event binding, extension/override dispatch, more than one
qualifier segment, calls outside the Configuration, nested declarations,
parameters or variables with the same spelling, metadata renames, Module
renames, and every other refactoring family are unsupported. Unsupported
syntax never produces a guessed operation. A target-related unsupported,
unresolved, or ambiguous candidate makes the whole request incomplete and
fails it; unrelated diagnostics do not affect the plan.

The family is not `Supported` until Tasks 3-4 prove complete paired evidence.
Architecture acceptance alone does not change either adapter's Coverage
Registry.

### Immutable source documents and exact occurrences

One `SourceDocument` is identified by the structured pair
`(configuration_id, module_id)`. Exactly one accepted BSL document may exist
for that pair. Format, display name, filesystem discovery order, and path do
not participate in document identity. The document separately owns:

- its source format and accepted module role;
- one slash-normalized path relative to the Workspace root and confined under
  both Workspace and Configuration roots;
- exact captured raw bytes;
- a `SourceContentVersion` equal to `(raw_byte_len, sha256(raw_bytes))`; and
- canonically ordered exact declaration and direct-call occurrences plus a
  family-specific completeness marker.

Accepted content is UTF-8 with at most one leading UTF-8 BOM. CRLF, CR, and LF
are preserved in raw bytes. An analysis view may remove the BOM and normalize
line endings to LF, but every occurrence maps back to the raw document as a
non-empty zero-based half-open UTF-8 byte range `[start, end)`. Both endpoints
must be within the raw length and on UTF-8 scalar boundaries. The exact bytes
in the range must decode to the recorded identifier token. Existing one-based
`SourceSpan` point locations remain navigation evidence and are not operation
ranges.

`SourceContentVersion` equality requires both the raw length and all 32 digest
bytes. The digest is lowercase hexadecimal only when an internal diagnostic or
test fixture needs a rendering. Raw content, the digest, absolute paths, and
adapter provenance IDs are never part of the public projection.

Adapters must read each accepted regular non-symlink BSL source once while
building the candidate Workspace snapshot, before publication. They must
capture or reject the complete document atomically; a planner and preview may
use only the retained immutable document and may never reopen, restat, or
probe its source path. Duplicate document IDs, two paths for one document ID,
path aliasing, an escaping path, unsupported encoding, malformed BOM, an
over-bound document, or conflicting bytes/version/evidence rejects the
candidate publication.

An occurrence owns `document_id`, `content_version`, exact byte range,
occurrence kind `declaration|local_call|qualified_call`, exact captured token,
optional mapped target Node ID, and resolution
`unique|unresolved|ambiguous|unsupported`. The mapped target is present if and
only if resolution is `unique`. Its structured identity is the complete tuple.
Exactly one declaration must map uniquely to the selected target. The complete
evidence marker means that every accepted module was scanned and every
syntactically relevant direct call candidate has one retained outcome; it does
not mean unsupported syntax became supported.

### Publication and target identity

The process-local publication sequence accepted by
[ADR-0061](0061-change-impact-analysis.md) is generalized, not duplicated.
`oneagent-analysis::publication::WorkspacePublicationId` becomes the single
checked non-zero `u64` identity for a successful complete Workspace
publication. `ChangeImpactPublicationId` remains a source-compatible alias or
projection of that exact type and counter; no conversion can produce a second
sequence.

Publication `1`, checked increments, failed-attempt behavior, reset on a fresh
service, cache exclusion, and non-global/non-time semantics remain those of
ADR-0061. Every `WorkspaceSnapshot`, its adjacent impact report, and every
refactoring request observed through that snapshot use the same ID. Equality
is numeric only within the running `WorkspaceService`; a plan is stale outside
that process lifetime or against any other publication ID.

`RefactoringTarget` binds:

- Configuration ID;
- exact pre-rename Node ID and kind;
- exactly one owning Module Node ID;
- the one declaration occurrence and its source document/version; and
- the expected post-rename Node ID.

The expected post-rename ID is produced by the same BSL-owned constructor as a
fresh declaration: `module_id`, callable kind, and desired name are composed by
that owner. Analysis does not reproduce the string format. The pre-rename Node
ID remains target identity for the plan; current name, desired name, display
labels, and expected post-rename ID do not replace it.

The target is rejected when it is missing, has the wrong kind, has zero or
multiple owners, is not owned by a Module in the selected Configuration, has
zero or multiple exact declarations, has incompatible occurrence evidence, or
would produce a post-rename ID already occupied by a different Graph node.

### Desired-name and collision contract

The accepted desired name is one to 256 UTF-8 bytes. Its first Unicode scalar
is `_` or `char::is_alphabetic`; every later scalar is `_` or
`char::is_alphanumeric`. Dots, whitespace, controls, combining-only starts,
and every other character are rejected.

BSL name equality is exactly Rust Unicode `to_lowercase()` over the whole
string, matching current local and qualified resolution. One BSL-owned helper
must serve resolution, occurrence mapping, no-op detection, and collision
checks. Locale, filesystem case rules, Unicode normalization, and
`EntityName`'s exact ordering are not substitutes.

The following case-folded tokens are reserved for v1 and cannot be desired
names:

```text
if если elsif иначеесли while пока for для foreach длякаждого
return возврат procedure процедура function функция
endprocedure конецпроцедуры endfunction конецфункции export экспорт
```

The request fails as `NoChange` when the desired name is BSL-equivalent to the
current name, including a case-only change. It fails as `NameCollision` when
any other Procedure or Function owned by the same Module has an equivalent
name, regardless of callable kind. It fails as `IdentityCollision` when the
expected post-rename ID collides with any other Graph node. Qualified calls do
not rename the Module qualifier; an exported target's uniquely mapped final
identifier ranges are included like local calls.

### Request, preconditions, and plan identity

`RefactoringRequest` contains exactly family `bsl_callable_rename_v1`, expected
Workspace publication ID, Configuration ID, target pre-rename Node ID, and
desired name. It contains no path, source text, caller-supplied occurrence,
diff, diagnostic, impact result, Git evidence, operation, or authorization.

The immutable planner input is one borrowed `Arc<WorkspaceSnapshot>`
publication projected into Analysis as the selected complete Configuration
Graph plus its complete source-document set. The planner records one
`RefactoringPreconditionSet` containing the publication ID, Configuration ID,
target identity and kind, owner Module ID, and the ordered `(document_id,
content_version)` pairs used by every operation. It accepts no source read or
caller override after capture.

`PlanId` is lowercase SHA-256 over an unambiguous canonical binary encoding:
the versioned family tag; each UTF-8 string preceded by its big-endian `u64`
byte length; fixed integers in big-endian form; the complete request and target
tuple; ordered source preconditions; and ordered operation IDs. It excludes
paths, raw content, preview rendering, summary, timestamps, process/task IDs,
errors, discovery/request order, and protocol limits. A differing semantic
precondition or operation must produce a different canonical byte stream.

Hash collision safety is fail-closed: two equal IDs with unequal complete
structured values are `IdentityCollision`, never equality or deduplication.

### Operations, duplicates, conflicts, and total order

The closed operation vocabulary is:

- `ReplaceDeclarationIdentifier`; and
- `ReplaceDirectCallIdentifier`.

Each `RefactoringOperation` owns operation kind, document ID, content version,
exact raw byte range, exact expected captured token, desired replacement, and
`OperationId`. The operation ID uses the same canonical encoding and SHA-256
rule over all preceding fields. The expected and replacement values are each
bounded to 256 UTF-8 bytes. The range must contain exactly the expected token,
and its BSL-equivalence key must equal the old target name.

Operations have no dependency edges in v1. The only accepted dependency set is
empty, its maximum count is zero, and any supplied dependency or cycle is
`IncompatibleEvidence`. Preview projection order is the future safe-application
order, although Sprint 40 never applies it:

1. document ID ascending;
2. range start descending;
3. range end descending;
4. declaration before direct call; and
5. operation ID ascending.

This total order is independent of Graph insertion, adapter discovery,
filesystem, hash-map, request, and occurrence order.

Exact duplicates have every structured operation field equal and collapse
before summary construction. An equal operation ID with unequal fields, the
same range with unequal fields, different replacements at one anchor,
incompatible versions for one document, or any intersecting non-equal ranges
in one document rejects the whole request. Adjacent ranges do not overlap.
No last-writer-wins, encounter-order selection, partial conflict set, or
best-effort plan exists.

### Completeness, summary, and preview

A successful `RefactoringPlan` has the only v1 completeness value `complete`.
Success proves one planned target, one declaration operation, every supported
uniquely resolved direct call operation, no unresolved target-related
candidate, no conflict, and no omitted internal operation. Any missing,
incomplete, stale, ambiguous, conflicting, unsupported target-related, or
over-bound evidence returns an error and no plan or preview.

The plan summary uses checked arithmetic and records:

- targets: `requested=1`, `planned=1`, `conflicted=0`, `rejected=0`;
- documents: total distinct documents used;
- candidates, exact duplicates collapsed, declaration operations, local-call
  operations, qualified-call operations, and total planned operations; and
- internal operations: `omitted=0`, `returned=planned`.

The identities above are invariants, not configurable counters. Any overflow
or failed reconciliation is `ArithmeticOverflow` and returns no result.

`RefactoringPreview` is a deterministic structured projection of the complete
plan and retained documents. It is not a patched document, unified diff, or
source snippet. Each entry contains operation ID and kind, a confined
Workspace-relative path, exact byte range, derived one-based line/column range,
and the bounded replacement identifier. It contains no surrounding source,
expected old token, raw bytes, content version, digest, absolute path,
provenance ID, repository value, or internal error.

Preview lines are separated by the retained CRLF, CR, or LF sequence; an
optional leading BOM is not a column. Columns count Unicode scalar values and
the preview end coordinate is exclusive. Preview generation validates these
positions against the retained raw bytes and does not normalize or emit line
endings. Repeated preview over the same plan and publication is byte-for-byte
equal. It mutates no source, repository, Workspace, cache, editor, protocol
state, or plan state and grants no edit authorization.

### Admission bounds

All count and byte bounds are inclusive and are checked before retaining or
cloning the affected collection or value. Exact-limit inputs are accepted;
one-over inputs fail atomically.

| Value | Maximum |
|---|---:|
| Selected Configurations / targets | 1 / 1 |
| Documents per Configuration | 4,096 |
| Raw bytes per document | 1,048,576 |
| Raw bytes across documents | 67,108,864 |
| Occurrences per document | 4,096 |
| Candidate occurrences and planned operations | 65,536 each |
| Dependency edges | 0 |
| Configuration, Module, Node, and document identity component | 4,096 UTF-8 bytes |
| Confined path | existing `SourcePath` bound of 4,096 UTF-8 bytes |
| Desired, expected, or replacement identifier | 256 UTF-8 bytes |
| SHA-256 identity rendering | 64 ASCII bytes |
| Public preview entries | request `1..=100`, default `50` |
| Tool arguments / serialized tool output | existing 65,536 UTF-8 bytes each |
| Public diagnostic detail | existing 512 UTF-8 bytes |
| MCP message | existing 1,048,576 bytes |

The 67,108,864-byte aggregate source bound is checked incrementally before
publication. A public preview limit never changes the owned plan. Public
projection reports complete plan totals plus `returnedOperations`,
`omittedOperations`, and `truncated`; the last is true exactly when the
presentation omits entries. It may not describe that projection as a partial
plan. Output that still exceeds Tool Policy's bound is `result_too_large`;
strings and entries are never byte-truncated.

### Closed failures and deterministic precedence

The domain failure kinds are:

```text
Cancelled InvalidRequest BoundExceeded PublicationMismatch
ConfigurationNotFound TargetNotFound UnsupportedTarget AmbiguousOwner
UnsupportedSourceFormat SourceEvidenceMissing SourceEvidenceIncomplete
IncompatibleEvidence StaleSourceVersion InvalidOccurrence
AmbiguousOccurrence InvalidDesiredName NoChange NameCollision
IdentityCollision DuplicateConflict OverlappingOperations ArithmeticOverflow
```

No error retains or renders rejected source content, desired text beyond the
already accepted request, absolute path, content digest, repository
configuration, credential, environment value, policy internal, or raw error
chain. Public messages are constant per failure kind.

Evaluation follows this precedence:

1. cancellation at entry;
2. request shape, scalar byte bounds, family, and desired-name grammar;
3. expected/current publication equality;
4. Configuration and target lookup, kind, and single owner;
5. source format, document-set admission, and completeness;
6. declaration, version, range, token, and target mapping validity;
7. no-op, sibling-name, and expected-ID collision checks;
8. canonical occurrence normalization, duplicate and overlap checks;
9. aggregate count/byte bounds and checked summaries; and
10. operation, plan, and preview identity/projection construction.

The planner checks cancellation before every stage, before every document, and
before returning. Cancellation at a checkpoint returns `Cancelled` and no
partial result. Within one stage, failures are selected by the closed enum
order above and then by canonical document ID, range, and operation ID; source
encounter order never selects the error. A synchronous Graph query is not
interruptible, but Runtime must join any active bounded planner work during
shutdown.

### Workspace lifecycle and persistent cache

Each `WorkspaceConfigurationSnapshot` gains one immutable complete source
evidence set or the candidate publication fails. Runtime lends one cloned
`Arc<WorkspaceSnapshot>` to a complete request. A newer publication may become
current after that clone without changing the in-flight result. A request that
names a different publication fails `PublicationMismatch`; it is never silently
retargeted.

Initial, equal, and recovered successful rebuilds receive normal publication
IDs and complete source evidence. Failed, cancelled, stale, over-bound, or
source-incomplete rebuilds publish nothing, consume no ID, and retain the last
valid snapshot. Stopping the observer prevents new requests; already cloned
snapshots remain immutable. A fresh service restarts at publication `1` and no
plan or precondition is valid across that boundary. No plan cache, plan
history, mutable planner state, detached task, or cross-client session is
introduced.

Workspace cache schema remains `1` and semantic compatibility advances from
`5` to `6`. The private source-state envelope remains the owner of exact cached
regular-file bytes. The semantic cache DTO adds the canonical document and
occurrence manifest without duplicating raw content. Decode reconstructs each
document from the accepted source-state bytes, recomputes its content version,
and validates every identity, range, token, mapping, ordering, and completeness
claim before publication. Version `5`, missing, conflicting, stale, corrupt,
or non-canonical evidence follows the existing reject-and-clean-rebuild path.
Cold and accepted version-`6` warm publications must expose equal source
evidence and equal plans. Publication IDs and plans are never persisted.

### Runtime, Tool Policy, MCP, and compatibility

Sprint 40 adds exactly one tool, `oneagent.refactor.plan`, making the catalog
eight lexicographically ordered tools. The tool is annotated read-only and its
only Tool Policy effect is `ReadOnly`; it uses the existing actor, request,
policy revision, deny behavior, execution path, and argument/output bounds.
Planning or preview never requests `LocalMutation`, confirmation, or another
effect, and a successful response has no authorization value usable by Sprint
41.

The request schema has exactly required `publicationId`, `configurationId`,
`targetNodeId`, and `desiredName`, plus optional `limit` from `1` through `100`
with default `50`; unknown fields are invalid. `publicationId` is a non-zero
JSON integer representable as `u64`. Runtime validates UTF-8 byte bounds before
lookup and clones exactly one current snapshot at call start.

The result exposes family, numeric publication ID, Configuration ID, plan ID,
target pre-rename Node ID/kind/owner and expected post-rename Node ID, desired
name, completeness `complete`, reconciled summary, bounded structured preview,
total/returned/omitted operation counts, `truncated`, `readOnly=true`, and
`editAuthorization="none"`. It exposes no internal source version, expected
token, raw source, absolute path, or mutable handle.

Domain invalid request and bound failures map to `invalid_arguments`; missing
Configuration or target maps to `not_found`; every stale, incompatible,
incomplete, conflict, cancellation, or arithmetic failure maps to
`execution_failed`; Tool Policy denial remains `policy_denied`; and encoded
output overflow remains `result_too_large`. Existing validation precedence
before handler execution is preserved.

The same additive schema and response are served under negotiated MCP
`2025-06-18`, `2025-11-25`, and `2026-07-28`. Frames remain sequential and
bounded at 1 MiB. Every existing tool definition, input, result, error,
annotation, and the immutable test constructor remains unchanged. The VS Code
connection catalog assertion migrates from seven to eight names, but no VS
Code command, code action, edit request, preview UI, or automatic invocation is
added. HTTP, CLI, LSP, EDT UI, and other clients gain no endpoint.

### Deterministic evidence and implementation gates

Tasks 3-4 are a hard prerequisite to Tasks 5-8. They must prove the document,
raw content, version, range, mapping, completeness, confinement, and bounds
contract for both adapters with the tracked paired EDT/Designer fixture. The
paired projection must include the same declaration plus local and exported
qualified call occurrences and remain equal across accepted BOM and line-ending
forms. Missing, malformed, unresolved, ambiguous, collision, non-UTF-8,
symlink, reordered, exact-limit, and one-over cases must fail or compare exactly
as specified.

Planner evaluation then proves empty/cancelled inputs; Procedure and Function;
English and Russian declarations; local, repeated, same-line, cross-line, and
qualified calls; non-exported behavior; every target/source/version failure;
no-op and collisions; duplicate, same-anchor, overlap, and forbidden
dependency cases; checked summaries; exact and one-over bounds; input-order
independence; stable/different identities; and unchanged retained source.
Synthetic graphs may supplement conflicts but cannot replace production
adapter evidence.

Workspace evidence must change, delete, and make the original file unreadable
after publication and still produce repeated byte-equal plans from the retained
Arc. A successor source version must make the old publication precondition
stale. Cold/warm, equal rebuild, failure/recovery, cancellation, stop,
Configuration add/remove, and filesystem/Git trigger-equivalent end states
must preserve the accepted lifecycle.

Public evidence covers catalog/schema/annotations, Tool Policy allow/deny,
every supported revision, invalid/malformed/stale/conflict/bound/error
precedence, output size/redaction, repetition, reordered JSON, publication
races, EOF, and shutdown while preserving all seven legacy tools. Tasks 3-9
run their focused non-zero suites and canonical workspace gates; Task 10
performs the independent integration review.

## Consequences

OneAgent gains an implementable contract for one complete deterministic
read-only refactoring plan over one immutable Workspace publication. Tasks 3-4
must first make exact source evidence a publication invariant, increasing
bounded snapshot and cache content. Task 8 deliberately adds one compatible
read-only MCP catalog entry and a synchronized connection assertion.

No current product behavior changes in this architecture task. In particular,
no source is read after publication, no source or repository value is changed,
and no plan or preview authorizes an edit. Unsupported or incomplete evidence
reduces availability by failing closed instead of producing a partial plan.

## Rejected alternatives

- Graph-owned source documents, ranges, conflicts, plan identity, or preview
  would turn Graph into a text-edit and product-report authority.
- Adapter-, Runtime-, MCP-, or client-owned planning would duplicate semantics
  and make results depend on source format or transport.
- Re-reading source during planning or preview could combine Graph and text
  from different versions and violates immutable publication ownership.
- Treating point-only `SourceLocation`, collapsed `Calls` edges, impact,
  diagnostics, paths, Git status, or model output as complete occurrences
  cannot prove the required edit set.
- EDT-only planning would make the advertised family depend on source format
  and leave the tracked paired corpus unused.
- Reusing a path, content digest, update attempt, Git commit, timestamp, UUID,
  or a new planner counter as publication identity would compete with the
  existing successful-publication sequence.
- A case-sensitive or locale-dependent rename would disagree with current BSL
  resolution; accepting a case-only rename would make identity and collision
  behavior ambiguous.
- Encounter-order selection, last-writer-wins, overlapping operations, partial
  conflict results, or silent truncation cannot produce a complete plan.
- Full source snippets, unified diffs, patched documents, public content
  versions, or absolute paths expose unnecessary source evidence.
- Overloading `oneagent.symbols`, `oneagent.query`, or `oneagent.impact` would
  change established request schemas and validation precedence.
- Hiding planner output behind an existing tool or Runtime-only API would avoid
  the required product workflow without preserving truthful capability
  discovery.
- Persisting plans or publication IDs would create cross-process freshness,
  validity, eviction, and sensitive-data contracts outside the bounded slice.
- Adding apply, confirmation, transaction, rollback, or post-edit validation
  to Sprint 40 would collapse evidence and authorization phases.

## Deferred scope

Every other refactoring family; nested or indirect calls; dynamic, reflected,
string, query, callback, event, extension, or override references; metadata,
Module, parameter, variable, path, or file renames; multi-target, multi-
Configuration, cross-Workspace, remote, selective, incremental, probabilistic,
or model-generated planning; new Graph facts or traversal; source mutation;
editor workspace edits and code actions; apply authorization; transaction
staging and commit; concurrency recheck; atomicity, rollback, reversibility,
backup, recovery, cleanup, durability, and post-edit rebuild/semantic
validation; filesystem mutation primitives; Git mutation or remote access;
plan persistence, history, telemetry, benchmarks, and broad performance,
security, or interoperability claims; new HTTP, CLI, LSP, EDT, or VS Code UI;
and concurrent MCP dispatch, progress, client sessions, or cancellation
notifications remain outside Sprint 40.

Sprint 41 owns the accepted edit-transaction architecture and implementation.
It must consume a fresh Sprint 40 plan only after rechecking every publication,
document version, range, expected token, confinement, and authorization
precondition immediately before mutation. This ADR does not select its write,
atomicity, rollback, or validation mechanism.
