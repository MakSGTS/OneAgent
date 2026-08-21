# Semantic Model 2.0

## Purpose

Semantic Model 2.0 defines the unified Knowledge Graph used by OneAgent to represent the structure, code, data access, UI, dependencies, and derived semantics of a 1C:Enterprise configuration.

The model replaces the long-term use of isolated declaration, call, and metadata graphs with a single semantic foundation.

The Knowledge Graph is intended to support:

* semantic navigation;
* declaration and reference search;
* call hierarchy;
* metadata dependency analysis;
* data access analysis;
* impact analysis;
* architectural diagnostics;
* AI context construction;
* MCP tools;
* IDE integrations.

Semantic Model 2.0 does not replace source parsers or metadata loaders. It receives semantic facts produced by them.

## Position in the architecture

```text
EDT workspace
Filesystem workspace
Future source adapters
        │
        ▼
Source discovery
        │
        ▼
Metadata and BSL extraction
        │
        ▼
Semantic contributors
        │
        ▼
Knowledge Graph
        │
        ├── Navigation
        ├── Reference search
        ├── Call hierarchy
        ├── Dependency analysis
        ├── Impact analysis
        ├── Diagnostics
        └── AI Context Engine
```

The graph is source-format independent.

The graph must not know how EDT XML is stored, how files are discovered, or how BSL is parsed.

## Architectural boundaries

### `oneagent-common`

Owns shared low-level concepts:

* source paths;
* source identifiers;
* source spans;
* stable primitive identifiers;
* common diagnostics;
* shared naming primitives where appropriate.

### `oneagent-metadata`

Owns the source-independent semantic model of 1C metadata:

* metadata objects;
* attributes;
* tabular sections;
* forms;
* commands;
* standard attributes;
* dimensions;
* resources;
* measures;
* other metadata members.

It must not depend on EDT-specific XML structures.

### `oneagent-bsl`

Owns BSL syntax and semantic extraction:

* modules;
* procedures;
* functions;
* parameters;
* variables;
* call expressions;
* references;
* scopes;
* future type information.

It must not own the global Knowledge Graph.

### `oneagent-graph`

Owns the generic Knowledge Graph:

* node and edge identities;
* node and edge kinds;
* provenance;
* graph storage;
* graph indexes;
* graph builder;
* schema validation;
* graph queries;
* graph traversal.

It must not parse BSL, read EDT files, or discover the filesystem.

### `oneagent-edt`

Owns EDT-specific loading and conversion:

* EDT project structure;
* metadata descriptor reading;
* module discovery;
* conversion from EDT artifacts into source-independent metadata and BSL input.

Ignored repository-local EDT projects used as planning and fixture-derivation
evidence are inventoried in
[Local EDT Source Corpora](edt-source-corpora.md). Corpus presence does not by
itself expand accepted parser or semantic contracts and is never a runtime or
CI prerequisite.

### `oneagent-runtime`

Owns orchestration:

* graph construction pipeline;
* contributor execution;
* lifecycle;
* snapshots;
* invalidation;
* incremental rebuilds;
* API exposure.

ADR-0037 assigns the implemented Sprint 15 long-running execution boundary to
`oneagent-runtime`: the composition root registers uniquely named services, the
application owns one built container, and the running container owns every
top-level task and per-service cancellation source through deterministic reverse
shutdown and complete join. Public in-memory integration tests prove startup,
rollback, requested shutdown, terminal failures, cleanup, and independent fresh
runs. This implementation does not move graph meaning or source ingestion into
Runtime.

ADR-0038 governs the implemented Sprint 16 HTTP boundary without changing
semantic authority. Runtime owns one Axum service and listener, exposes only
stable liveness and lifecycle-derived readiness probes, and retains listener,
connection, cancellation, and task lifetimes under ADR-0037. Public raw-loopback
tests prove the exact wire matrix across `Initializing`, `Running`, and
`Stopping`, named bind failure, graceful shutdown, listener release, and fresh
repetition.

ADR-0039 governs the implemented Sprint 17 initial Workspace orchestration
boundary without changing graph authority. Runtime owns one configured root,
runs the production filesystem detector and EDT/Designer builders in one joined
blocking task, validates every configuration, and atomically publishes an
immutable source-neutral snapshot. The snapshot retains separate graphs ordered
by Configuration identity plus exact names, source roots/formats, diagnostics,
reference ledgers/statistics, and reports. Public integration evidence proves
both production formats, deterministic fresh runs, empty discovery, invalid and
conflicting roots, duplicate identity, later adapter failure atomicity,
lifecycle-derived readiness, reverse cleanup, and closed observation. Runtime
does not merge graphs or become a semantic contributor. Cache/persistence and
supported CLI behavior remain deferred to Sprints 20–21.
The
[Sprint 17 integration review](../reviews/sprint-17-workspace-service.md)
records `pass`; Sprint 17 is completed.

ADR-0040 governs the implemented Sprint 18 Graph Query API without changing
`SemanticGraph` authority. One observer-backed transport-neutral Runtime
component obtains exactly one immutable Workspace snapshot per call and exposes
only configuration listing, exact node lookup, direct incoming/outgoing
relations, and bounded breadth-first traversal. The existing Runtime-owned HTTP
listener maps those operations to four exact `/api/v1` GET routes, lifecycle
and snapshot availability, closed enum vocabularies, bounded owned projections,
and stable JSON success/error schemas. Payloads, provenance, diagnostics,
reports, mutation, arbitrary query languages, and aggregate graphs are not
exposed. Public raw-loopback evidence exercises both production formats,
selection and ordering, the complete first-slice error/method/path matrix,
`Initializing`/`Running`/`Stopping`, snapshot and listener cleanup, and equal
fresh runs. The
[Sprint 18 integration review](../reviews/sprint-18-graph-query-api.md) records
`pass`; Sprint 18 is completed.

ADR-0041 governs the implemented Sprint 19 File Watching boundary without
changing graph or adapter authority. After the initial Workspace build, one
Runtime-owned polling source compares normalized recursive scans containing
complete file bytes and ignores confirmed tool/cache directories. The Workspace
service serializes complete production rebuilds, coalesces changes during a
build, and atomically replaces the published immutable snapshot only after a
valid all-or-nothing result. Failed observation or rebuild attempts retain the
last valid snapshot and become public update status; a later change can recover.
Existing health/readiness and Graph Query wire contracts remain unchanged, and
shutdown joins observation and build work before clearing the snapshot and
closing status observation. Public production evidence covers EDT and Designer
XML modifications, removal and addition/rename-equivalent changes, immutable
replacement, query visibility, failure retention and recovery, fresh runs, and
complete cleanup. Sprint 19 implementation and public evidence are complete;
the
[Sprint 19 integration review](../reviews/sprint-19-file-watching.md) records
`pass`. Sprint 19 is completed and Sprint 20 Persistent Cache is the unique
`next` target.

ADR-0042 governs the implemented Sprint 20 Persistent Cache boundary without
changing graph, adapter, health, or Graph Query authority. Runtime persists only
complete validated `WorkspaceSnapshot` state in one private versioned
Workspace-local entry keyed by exact complete source state and explicit schema
and semantic-build versions. A candidate is reconstructed through checked domain
APIs and complete validation before publication; missing, changed,
incompatible, corrupt, or unavailable state falls back to a clean production
build. Stable initial and File Watching builds replace the cache before immutable
publication, while write failure or unstable-source skip remains recoverable.
Cache-owned paths are excluded from observation, and shutdown closes typed cache
status observation while preserving the complete entry for a fresh process.
Public mixed EDT/Designer evidence proves cold/write, warm-hit complete-state and
query equivalence, version/source invalidation, corruption and write-failure
recovery, watched replacement, lifecycle/health compatibility, cleanup, and
fresh reuse. The
[Sprint 20 integration review](../reviews/sprint-20-persistent-cache.md) records
`pass`; Sprint 20 is completed and Sprint 21 CLI Client is the unique `next`
target.

## Core principles

### Unified semantics

Metadata objects, modules, BSL declarations, source files, queries, and UI elements are represented in one graph.

For example:

```text
Document.SalesInvoice
    ├── HasAttribute ──▶ Product
    ├── HasAttribute ──▶ Quantity
    ├── HasModule ─────▶ ObjectModule
    ├── HasForm ───────▶ DocumentForm
    └── HasCommand ────▶ Post
```

```text
Document.SalesInvoice.ObjectModule
    └── Declares ──▶ Posting
```

```text
Posting
    ├── Calls ─────▶ InventoryManagement.WriteMovements
    └── WritesTo ──▶ AccumulationRegister.Stock
```

### Stable identity

Every node and edge has a deterministic identifier.

Identity must remain stable across repeated indexing of unchanged source content.

Identity must not depend on insertion order or nondeterministic collection iteration.

### Explicit provenance

Every fact must be traceable to its source and producer.

A graph consumer must be able to distinguish:

* a parsed declaration;
* a resolved reference;
* a derived dependency;
* an external or unresolved target.

### Incomplete knowledge is preserved

Unresolved and ambiguous references are retained rather than discarded.

The graph must represent what OneAgent knows and what it does not know.

### Canonical edge direction

Each semantic relation has one stored direction.

Reverse traversal is provided through graph indexes.

For example:

```text
Module ── Declares ──▶ Procedure
```

The inverse relation is queried through the incoming index rather than stored as a second edge.

### Deterministic output

Graph iteration, serialization, snapshots, diagnostics, and context candidate ordering must be reproducible.

## Node model

A node represents a semantic entity.

Conceptual structure:

```rust
pub struct Node {
    pub id: NodeId,
    pub kind: NodeKind,
    pub identity: NodeIdentity,
    pub properties: NodeProperties,
    pub provenance: Vec<Provenance>,
}
```

### Node identity

```rust
pub struct NodeIdentity {
    pub name: String,
    pub qualified_name: Option<String>,
    pub uuid: Option<String>,
}
```

The final implementation may use stronger domain types, but the graph must preserve:

* local name;
* qualified name when available;
* metadata UUID when available;
* stable semantic identity.

### Node properties

Core properties should be typed.

A fully dynamic property map must not be the primary domain model.

Recommended structure:

```rust
pub struct NodeProperties {
    pub common: CommonNodeProperties,
    pub specific: SpecificNodeProperties,
}
```

```rust
pub enum SpecificNodeProperties {
    Workspace(WorkspaceProperties),
    Source(SourceProperties),
    Metadata(MetadataProperties),
    Module(ModuleProperties),
    Symbol(SymbolProperties),
    Type(TypeProperties),
    Data(DataProperties),
    Ui(UiProperties),
    Derived(DerivedProperties),
    External(ExternalProperties),
    None,
}
```

An extension property map may be added for experimental or adapter-specific data, but it must not replace typed core properties.

## Node taxonomy

```rust
pub enum NodeKind {
    Workspace(WorkspaceNodeKind),
    Source(SourceNodeKind),
    Metadata(MetadataNodeKind),
    Module(ModuleNodeKind),
    Symbol(SymbolNodeKind),
    Type(TypeNodeKind),
    Data(DataNodeKind),
    Ui(UiNodeKind),
    Derived(DerivedNodeKind),
    External(ExternalNodeKind),
}
```

## Workspace nodes

```rust
pub enum WorkspaceNodeKind {
    Workspace,
    Project,
    Configuration,
    Extension,
    Library,
}
```

Workspace nodes define the indexed project hierarchy.

## Source nodes

```rust
pub enum SourceNodeKind {
    Directory,
    File,
    MetadataFile,
    ModuleFile,
    FormFile,
    TemplateFile,
    QueryFile,
}
```

Source nodes are required for:

* source navigation;
* provenance;
* incremental indexing;
* invalidation;
* diagnostics;
* source fragment extraction.

## Metadata nodes

The initial taxonomy should support the primary 1C metadata categories.

```rust
pub enum MetadataNodeKind {
    Configuration,
    Subsystem,

    Catalog,
    Document,
    Enumeration,
    Constant,
    Report,
    DataProcessor,
    ChartOfAccounts,
    ChartOfCharacteristicTypes,
    ChartOfCalculationTypes,
    ExchangePlan,
    BusinessProcess,
    Task,

    InformationRegister,
    AccumulationRegister,
    AccountingRegister,
    CalculationRegister,

    CommonModule,
    CommonForm,
    CommonCommand,
    CommonAttribute,
    CommonTemplate,
    Role,
    Interface,
    SessionParameter,
    FunctionalOption,
    FunctionalOptionsParameter,
    DefinedType,
    SettingsStorage,
    WebService,
    HttpService,
    WsReference,
    XdtoPackage,
    Language,
    Style,
    StyleItem,
}
```

Metadata members are separate nodes:

```rust
pub enum MetadataMemberNodeKind {
    Attribute,
    StandardAttribute,
    TabularSection,
    Dimension,
    Resource,
    Measure,
    Recalculation,
    Form,
    Command,
    Template,
}
```

The exact Rust layout may either embed member kinds into `MetadataNodeKind` or use a dedicated nested category.

The semantic requirement is that metadata members remain independently addressable.

## Module nodes

```rust
pub enum ModuleNodeKind {
    CommonModule,
    ObjectModule,
    ManagerModule,
    RecordSetModule,
    FormModule,
    CommandModule,
    SessionModule,
    ExternalConnectionModule,
    ManagedApplicationModule,
    OrdinaryApplicationModule,
}
```

A module node is distinct from:

* its source file;
* its owner metadata object;
* symbols declared inside the module.

## Symbol nodes

```rust
pub enum SymbolNodeKind {
    Procedure,
    Function,
    Parameter,
    LocalVariable,
    ModuleVariable,
    Property,
    Region,
    Label,
}
```

Variable kinds may later be represented through a nested scope classification.

## Type nodes

```rust
pub enum TypeNodeKind {
    Primitive,
    Platform,
    MetadataReference,
    MetadataObject,
    MetadataManager,
    Collection,
    Union,
    Unknown,
}
```

Type nodes are introduced gradually during type resolution.

The graph must be able to represent unknown and partially inferred types.

## Data nodes

```rust
pub enum DataNodeKind {
    Query,
    QuerySource,
    QueryField,
    QueryParameter,
    TemporaryTable,
    DataCompositionSchema,
    DataSet,
    DataCompositionField,
}
```

Data nodes connect BSL execution with metadata storage.

## UI nodes

```rust
pub enum UiNodeKind {
    Form,
    FormAttribute,
    FormCommand,
    FormElement,
    FormEvent,
}
```

A metadata form may initially be represented as a metadata member and later linked to a richer UI structure.

## Derived nodes

```rust
pub enum DerivedNodeKind {
    Component,
    DependencyCluster,
    EntryPoint,
    UseCase,
    ContextBundle,
    Diagnostic,
}
```

Derived nodes are created by analyzers.

They do not need a direct source declaration, but they must record their producer and supporting facts.

## External nodes

```rust
pub enum ExternalNodeKind {
    UnresolvedReference,
    ExternalSymbol,
    PlatformSymbol,
    ExternalService,
    Unknown,
}
```

External nodes allow references to survive even when their real target is not part of the current graph.

## Edge model

An edge represents a semantic relation.

Conceptual structure:

```rust
pub struct Edge {
    pub id: EdgeId,
    pub kind: EdgeKind,
    pub source: NodeId,
    pub target: NodeId,
    pub properties: EdgeProperties,
    pub provenance: Vec<Provenance>,
}
```

```rust
pub struct EdgeProperties {
    pub origin: FactOrigin,
    pub confidence: Confidence,
    pub resolution: ResolutionState,
}
```

## Edge taxonomy

```rust
pub enum EdgeKind {
    Structural(StructuralEdgeKind),
    Declaration(DeclarationEdgeKind),
    Reference(ReferenceEdgeKind),
    Execution(ExecutionEdgeKind),
    Data(DataEdgeKind),
    Type(TypeEdgeKind),
    Ui(UiEdgeKind),
    Derived(DerivedEdgeKind),
}
```

## Structural edges

```rust
pub enum StructuralEdgeKind {
    Contains,
    Defines,
    DeclaredIn,
    HasModule,
    HasForm,
    HasCommand,
    HasTemplate,
    HasAttribute,
    HasStandardAttribute,
    HasTabularSection,
    HasDimension,
    HasResource,
    HasMeasure,
}
```

Structural edges define graph ownership and composition.

## Declaration edges

```rust
pub enum DeclarationEdgeKind {
    Declares,
    HasParameter,
    Overrides,
    Implements,
    Exports,
}
```

## Reference edges

```rust
pub enum ReferenceEdgeKind {
    References,
    Uses,
    Reads,
    Writes,
    Instantiates,
    ResolvesTo,
    MayResolveTo,
}
```

## Execution edges

```rust
pub enum ExecutionEdgeKind {
    Calls,
    MayCall,
    HandlesEvent,
    Triggers,
    ExecutesQuery,
    InvokesCommand,
}
```

`Calls` is reserved for resolved call targets.

## Data edges

```rust
pub enum DataEdgeKind {
    ReadsFrom,
    WritesTo,
    SelectsField,
    FiltersBy,
    GroupsBy,
    OrdersBy,
    JoinsWith,
    Produces,
    Consumes,
}
```

## Type edges

```rust
pub enum TypeEdgeKind {
    HasType,
    ReturnsType,
    AcceptsType,
    ElementType,
    RefersToMetadataType,
}
```

## UI edges

```rust
pub enum UiEdgeKind {
    Displays,
    BindsTo,
    Executes,
    Handles,
    Opens,
}
```

## Derived edges

```rust
pub enum DerivedEdgeKind {
    DependsOn,
    Affects,
    RelatedTo,
    Includes,
    Reaches,
    CandidateForContext,
}
```

## Provenance model

Conceptual provenance:

```rust
pub struct Provenance {
    pub source: SourceId,
    pub span: Option<SourceSpan>,
    pub producer: ProducerId,
    pub confidence: Confidence,
}
```

Fact origin:

```rust
pub enum FactOrigin {
    Declared,
    Parsed,
    Resolved,
    Derived,
    External,
}
```

Resolution state:

```rust
pub enum ResolutionState {
    NotApplicable,
    Unresolved,
    Partial,
    Ambiguous,
    Resolved,
}
```

Confidence may initially be represented as a closed enum:

```rust
pub enum Confidence {
    Exact,
    High,
    Medium,
    Low,
    Unknown,
}
```

The graph should avoid floating-point confidence until a concrete scoring model requires it.

## Graph storage

The initial graph is embedded and in-memory.

Recommended conceptual structure:

```rust
pub struct KnowledgeGraph {
    nodes: BTreeMap<NodeId, Node>,
    edges: BTreeMap<EdgeId, Edge>,
    outgoing: BTreeMap<NodeId, Vec<EdgeId>>,
    incoming: BTreeMap<NodeId, Vec<EdgeId>>,
}
```

Indexes may include:

```rust
by_kind
by_qualified_name
by_source
by_metadata_uuid
by_symbol_name
by_edge_kind
```

Ordered collections are preferred where deterministic output matters.

Performance-sensitive indexes may use other internal structures if they do not affect observable ordering.

## Graph invariants

The graph must enforce the following invariants.

### Identity invariants

* every node identifier is unique;
* every edge identifier is unique;
* identifiers are deterministic;
* identity generation is centralized;
* identical semantic entities are not duplicated because of loading order.

### Referential invariants

* every edge source exists;
* every edge target exists;
* unresolved references point to explicit unresolved or external nodes;
* dangling edges are not allowed in a finalized graph.

### Structural invariants

* containment relations must not create invalid ownership cycles;
* a symbol has one canonical declaration owner;
* a module has one canonical semantic owner where applicable;
* a source node may define multiple semantic nodes.

### Schema invariants

* edge kinds must be valid for source and target node kinds;
* `Calls` targets procedures or functions;
* `Declares` originates from a declaration container;
* `HasType` targets a type node;
* metadata member relations target compatible metadata member nodes.

### Determinism invariants

* node iteration is stable;
* edge iteration is stable;
* serialized snapshots are stable;
* diagnostic ordering is stable;
* graph projections are stable.

## Graph builder

All graph mutation occurs through `GraphBuilder`.

Conceptual API:

```rust
pub struct GraphBuilder {
    graph: KnowledgeGraph,
    diagnostics: Vec<GraphDiagnostic>,
}
```

```rust
impl GraphBuilder {
    pub fn upsert_node(
        &mut self,
        node: Node,
    ) -> Result<NodeId, GraphError>;

    pub fn add_edge(
        &mut self,
        edge: Edge,
    ) -> Result<EdgeId, GraphError>;

    pub fn find_by_qualified_name(
        &self,
        qualified_name: &str,
    ) -> impl Iterator<Item = &Node>;

    pub fn finish(
        self,
    ) -> Result<KnowledgeGraph, GraphBuildError>;
}
```

The builder is responsible for:

* merging duplicate declarations where identity matches;
* preserving multiple provenance records;
* rejecting incompatible node redefinitions;
* preventing duplicate equivalent edges;
* validating schema compatibility;
* collecting non-fatal diagnostics;
* validating final graph invariants.

## Contributor model

Semantic facts are added by contributors.

Potential contributors:

```text
WorkspaceContributor
SourceContributor
MetadataContributor
ModuleContributor
BslDeclarationContributor
BslReferenceContributor
CallResolutionContributor
TypeResolutionContributor
QueryContributor
UiContributor
DerivedDependencyContributor
```

Conceptual interface:

```rust
pub trait GraphContributor {
    type Error;

    fn contribute(
        &self,
        context: &ContributionContext,
        graph: &mut GraphBuilder,
    ) -> Result<(), Self::Error>;
}
```

Contributors should be small and focused.

A contributor may add nodes, edges, diagnostics, and provenance, but it must not bypass the builder.

## Construction passes

## Pass 1: Source discovery

Creates:

* workspace nodes;
* project nodes;
* configuration nodes;
* directory nodes;
* source file nodes.

## Pass 2: Entity declaration

Creates:

* metadata object nodes;
* metadata member nodes;
* module nodes;
* procedure nodes;
* function nodes;
* parameter nodes;
* variable nodes.

This pass establishes the global declaration index.

## Pass 3: Structural linking

Creates:

* configuration containment;
* metadata ownership;
* object-to-module relations;
* module-to-symbol declarations;
* source-to-semantic definitions.

## Pass 4: Reference collection

Collects:

* BSL symbol references;
* call expressions;
* metadata references;
* type references;
* query references;
* UI handler references.

References may remain unresolved after this pass.

## Pass 5: Reference resolution

Resolves references through:

* lexical scope;
* module scope;
* common module exports;
* metadata namespace;
* metadata manager types;
* object and reference types;
* known platform symbols.

## Pass 6: Derived semantic analysis

Creates:

* dependency edges;
* entry points;
* reachability relations;
* impact relations;
* subsystem coupling;
* context candidates;
* diagnostics.

## Query API

The public query API must not expose internal collections directly.

Initial operations:

```rust
pub trait GraphQuery {
    fn node(
        &self,
        id: &NodeId,
    ) -> Option<&Node>;

    fn outgoing(
        &self,
        id: &NodeId,
        filter: EdgeFilter,
    ) -> Vec<GraphRelation<'_>>;

    fn incoming(
        &self,
        id: &NodeId,
        filter: EdgeFilter,
    ) -> Vec<GraphRelation<'_>>;

    fn neighbors(
        &self,
        id: &NodeId,
        filter: TraversalFilter,
    ) -> Vec<&Node>;
}
```

Additional operations:

* find by qualified name;
* find declarations;
* find references;
* find callers;
* find callees;
* bounded traversal;
* shortest path;
* dependency closure;
* reverse dependency closure.

## Graph projections

Existing specialized graphs may be represented as projections.

Examples:

```text
Declaration Graph
    =
Knowledge Graph filtered to declaration nodes and declaration edges
```

```text
Call Graph
    =
Knowledge Graph filtered to callable symbols and Calls or MayCall edges
```

This allows current consumers to migrate without duplicating semantic extraction.

## Incremental indexing

The graph architecture must support future incremental updates.

A source change should eventually allow OneAgent to:

1. identify the changed source node;
2. invalidate facts produced from that source;
3. remove or replace affected provenance;
4. rebuild affected declarations;
5. rerun dependent resolution passes;
6. update derived facts.

Provenance is therefore required not only for explanation but also for invalidation.

## AI Context Engine

The AI Context Engine consumes the Knowledge Graph through query interfaces.

It must not depend directly on EDT structures or parser internals.

## Context request

Conceptually:

```rust
pub struct ContextRequest {
    pub intent: ContextIntent,
    pub seeds: Vec<ContextSeed>,
    pub token_budget: TokenBudget,
    pub policy: ContextPolicy,
}
```

Possible seeds:

* node identifier;
* qualified name;
* source position;
* source file;
* metadata UUID;
* user-selected text;
* current editor symbol.

## Context traversal

Traversal policy defines:

* allowed node kinds;
* allowed edge kinds;
* direction;
* maximum depth;
* maximum candidates;
* confidence threshold;
* whether derived facts are allowed.

## Candidate scoring

Candidate score may include:

```text
semantic distance
edge importance
node kind priority
source proximity
resolution confidence
execution reachability
data dependency relevance
token cost
```

## Context bundle

The output of context selection is a `ContextBundle`.

Conceptually:

```rust
pub struct ContextBundle {
    pub target: Vec<NodeId>,
    pub included_nodes: Vec<NodeId>,
    pub included_edges: Vec<EdgeId>,
    pub source_fragments: Vec<SourceFragment>,
    pub explanations: Vec<ContextExplanation>,
}
```

The bundle must explain why each fragment was included.

## Context rendering

Rendered context should contain semantic summaries in addition to source code.

Example:

```text
Target:
  Document.SalesInvoice.Posting

Semantic role:
  Posting handler of SalesInvoice

Direct dependencies:
  - AccumulationRegister.Stock
  - CommonModule.InventoryManagement
  - Catalog.Products

Called procedures:
  - InventoryManagement.ValidateStock
  - InventoryManagement.WriteMovements

Relevant source:
  - ObjectModule.bsl:120-198
  - InventoryManagement.bsl:45-91
```

## Migration from current graphs

Migration must be incremental.

### Stage 1

Introduce graph identity, node kinds, edge kinds, provenance, and graph storage.

### Stage 2

Represent existing metadata objects and members in the Knowledge Graph.

### Stage 3

Represent modules and BSL declarations.

### Stage 4

Represent existing resolved calls.

### Stage 5

Expose current declaration and call graph APIs as projections or compatibility wrappers.

### Stage 6

Migrate runtime and adapters to consume the Knowledge Graph directly.

### Stage 7

Remove obsolete graph structures after all consumers and tests have migrated.

## Roadmap

```text
SM-0 Architecture specification
    ADR
    taxonomy
    identity rules
    provenance
    invariants

SM-1 Knowledge Graph Core
    NodeId
    EdgeId
    Node
    Edge
    NodeKind
    EdgeKind
    Provenance
    KnowledgeGraph
    GraphBuilder
    GraphSchema

SM-2 Metadata graph migration
    metadata objects
    attributes
    tabular sections
    forms
    commands
    standard attributes
    dimensions
    resources
    measures
    source links
    ownership edges

SM-3 BSL semantic graph migration
    modules
    procedures
    functions
    parameters
    variables
    declarations
    local references
    inter-module calls
    unresolved calls

SM-4 Type and reference resolution
    scopes
    symbol tables
    platform types
    metadata types
    common module exports
    ambiguous references
    resolution diagnostics

SM-5 Data access graph
    query extraction
    query AST
    reads
    writes
    fields
    parameters
    temporary tables
    register dependencies

SM-6 UI and entry points
    form internals
    form commands
    event handlers
    subscriptions
    scheduled jobs
    HTTP endpoints
    entry point classification

SM-7 Derived semantic analysis
    dependencies
    impact analysis
    reachability
    dead declarations
    cycles
    architectural boundaries
    subsystem coupling

SM-8 AI Context Engine
    seed resolution
    traversal policies
    candidate scoring
    token budgeting
    source extraction
    deduplication
    context bundles
    context explanations

SM-9 MCP and IDE integration
    graph queries
    context tools
    VS Code position seeds
    incremental refresh
```

## Historical initial implementation boundary

> Historical note: this section records the initial Knowledge Graph proposal.
> It is not the current implementation plan. The repository has since added
> BSL and query parsing, graph query, validation, diff, impact, coverage, and
> resolution facilities. Current execution scope is governed by accepted ADRs
> and `docs/Roadmap.md`; Sprint 4 is bounded by ADR-0026.

The first implementation stage covers only the Knowledge Graph core.

It does not yet include:

* full metadata migration;
* BSL migration;
* type inference;
* query parsing;
* UI parsing;
* AI context scoring.

The initial code target is:

```text
crates/graph/src/
├── identity.rs
├── kind.rs
├── node.rs
├── edge.rs
├── provenance.rs
├── knowledge_graph.rs
├── builder.rs
├── schema.rs
└── error.rs
```

Existing graph modules remain available until migration is complete.

The proposed file tree above is retained as historical design evidence and must
not be used to infer that those exact files exist in the current repository.

## Semantic Coverage Audit

The Semantic Coverage Audit records the implemented EDT-to-graph boundary before
Semantic Coverage Completion. It is an observation and governance API; it does
not change graph construction, resolution, validation, query, or impact
semantics.

The public API is split by dependency direction:

* `SemanticCoverageRegistry::audit()` in `oneagent-graph` describes the
  source-independent graph model, validation rules, Query API, Impact Analysis,
  and provenance paths;
* `SemanticObservedCoverage::for_graph()` counts node kinds, edge kinds, and
  provenance in one graph snapshot without inferring support from absence;
* `EdtSemanticCoverageRegistry::audit()` in `oneagent-edt` describes EDT
  discovery, parsing, node and edge contribution, ownership, and reference
  handling;
* `EdtSemanticGraphBuildResult::coverage_report()` composes both static matrices
  with observed graph coverage, the existing build report, and validation.

No ADR is required for this first audit version. The registry exposes existing
architecture and does not introduce a new semantic rule or dependency direction.

### Coverage semantics

`Supported` means that all evidence required by the capability category is
present. `PartiallySupported` means that implementation exists but one or more
required stages or representative checks are absent. `Unsupported` means that a
known relevant capability has no pipeline implementation. `NotApplicable` is
used only when a stage is intentionally outside the semantic meaning of a
capability. `DeclaredOnly` means that a graph enum variant exists but the EDT
Coverage Registry has not yet been transitioned to production support. It
ordinarily indicates that the EDT pipeline does not emit the capability. Reads
no longer uses this transitional classification: its production evidence and
registry transition are complete.

Capabilities use typed identities based on `MetadataKind`, `NodeKind`,
`EdgeKind`, validation codes, query capabilities, provenance paths, and typed
reference capabilities. Titles and notes never define identity. Evidence and
missing evidence are ordered typed sets. Capabilities sort by stable identity;
gaps sort by priority, category, and identity; observed metrics use ordered maps.

Static support and observed occurrence are intentionally separate. A metadata
kind absent from one configuration remains supported or partially supported
according to the static registry. Conversely, occurrence does not prove complete
support.

### EDT discovery and entity inventory

The configuration root is loaded from `Configuration.mdo`. The top-level EDT
directory registry discovers Catalog, Document, Enumeration, Common Module,
Report, Data Processor, Information Register, Accumulation Register, Accounting
Register, Calculation Register, Business Process, Task, Role, Common Form, HTTP
Service, Web Service, XDTO Package, Subsystem, Common Command, and Common Template
descriptors.
Common commands use the EDT `CommonCommands/<Name>/<Name>.mdo` layout and are
represented as `MetadataKind::Command` rather than the flat child
`NodeKind::Command`. Common templates similarly use
`CommonTemplates/<Name>/<Name>.mdo` and the existing
`MetadataKind::Template`; subordinate template extraction remains outside this
capability.

All discovered top-level descriptors emit `NodeKind::Metadata(kind)` with stable
identity, provenance, and the typed payload accepted by ADR-0023. The
parameterized production fixture in `adapters/edt/tests/payload.rs` accounts for
Configuration and every generic directory mapping, both present and absent
synonyms, Query visibility, validation, provenance, and repeated-build
determinism. `MetadataKind::Form` and `MetadataKind::Unknown` remain explicit
non-applicable fallbacks for this EDT path. The Coverage registry remains
aligned with this evidence through its completed payload transition.

### Typed top-level metadata payload contract

ADR-0023's source-independent payload contract is implemented in the metadata,
graph, and EDT production layers, and its Coverage transition is complete.
`oneagent-metadata` owns `MetadataPayload`; metadata GraphNode values store the
same typed domain value rather than an EDT structure or an untyped graph
property map. Payload is semantic content and does not participate in metadata
or node identity.

The accepted common payload contains only optional synonym. Configuration and
every supported generic top-level metadata kind use this contract. An absent
synonym remains absent and is not replaced with canonical name. The only
accepted kind-specific payload is the Document's deterministic, deduplicated
set of typed register-record targets. Repository code does not currently parse
another justified source-independent top-level payload field.

Identity, canonical name, kind, ownership, provenance, and source path remain
separate from payload. Metadata members, extension, role rights, Subsystem
content, references, and other relations continue to use their existing typed
nodes, edges, diagnostics, and provenance. They are not copied into payload.
Document register-record target components are intrinsic Document content, but
their resolution state, occurrence evidence, diagnostics, and Writes edges
remain separate.

Payload participates in `MetadataObject` and GraphNode equality and in graph
diff semantic content, while stable IDs remain unchanged. The existing Query API
exposes payload through returned GraphNode values without changing exact
canonical-name lookup. Future serialization must use a versioned tagged typed
structure with deterministic collection ordering.

Compatibility constructors may continue to create empty payload while existing
non-metadata producers and tests migrate. Such defaults are not EDT Coverage
evidence. `SemanticPayloadPreserved` may be added independently for a metadata
kind only after production conversion, present/absent and malformed evidence,
payload-kind validation, Query and payload-only diff evidence, representative
full-builder coverage, and repeated-build determinism all pass. Architecture
documentation alone changes neither capability status nor aggregate counts.

### Typed Attribute and TabularSection payload contract

ADR-0028's first subordinate-member payload slice is implemented in the
metadata, graph, and EDT production layers. `MetadataMemberPayload` contains
only an optional synonym and is compatible only with `NodeKind::Attribute` and
`NodeKind::TabularSection`. The payload is semantic display content: it does not
participate in source UUID identity, owner-scoped UUID-less fallback identity,
canonical name lookup, containment, reference-request identity, diagnostics,
statistics, or provenance aggregation.

The EDT metadata-structure reader accepts one direct non-empty
`synonym/value` for a recognized Attribute or TabularSection. Absence remains
an explicit empty member payload. Empty values, duplicate containers or values,
and the unsupported `synonym/content` encoding are typed parser errors. A
nested Attribute retains its own payload and immediate TabularSection owner;
its synonym does not leak to the owner or create a second metadata-object
containment edge.

The production graph builder uses explicit member-payload construction for
both present and absent values while preserving the existing two-phase child
node and ownership-edge contribution order. Query observes payload through the
canonical GraphNode. A synonym-only source change preserves node and edge
identity, is reported as `SemanticContent` by Diff, and is a direct Impact seed
without a new propagation rule. Semantic Index lookup dimensions remain
unchanged.

Graph-domain and EDT `SemanticNode(Attribute)` and
`SemanticNode(TabularSection)` Coverage now require and provide
`SemanticPayloadPreserved`. Their status and the aggregate gap counts remain
unchanged because both capabilities were already `Supported`. Real grants and
ownership fixtures plus generated malformed, UUID-less, equal-name,
source-order, and repeated-build cases provide the accepted evidence boundary.
Number qualifiers, history/search settings, produced types, line-number
settings and standard attributes, multiple locale values, alternative synonym
encodings, deeper nesting, and non-Document owner families remain deferred.

### Accepted Sprint 7 Form and Command boundary

The Sprint 7 source investigation in
`form-command-source-investigation.md` distinguishes the completed declaration
slice from unmodeled executable, reference, and navigation facts. Existing
Common Forms, Common Commands, subordinate Forms, and subordinate Commands keep
their current node kinds, UUID or owner-scoped fallback identity, immediate
metadata ownership, provenance, Query behavior, validation, and Coverage.
Those facts are compatibility constraints rather than new Sprint 7 outcomes.

ADR-0029 defines a bounded extension that the EDT adapter now implements.
Subordinate Form `Module.bsl` and Common/subordinate Command
`CommandModule.bsl` artifacts contribute
ordinary `Module`, `Procedure`, `Function`, Query, Calls, and related existing
BSL facts through the canonical graph pipeline. New subordinate module
identities derive from the canonical Form or Command owner plus a stable module
role. Form and Command nodes own those modules through `Contains`; no parallel
UI-specific symbol graph is introduced.

Mapped `commandParameterType` observations use the public semantic
reference-request lifecycle with a distinct Command parameter role. The source
is the canonical Common or subordinate Command, and the target allowlist is
exactly the nine metadata kinds already accepted for the completed metadata
type-reference slice. Unique resolution may emit both the direct `References`
fact and its justified normalized `DependsOn`; unsupported, missing,
ambiguous, incompatible, malformed, and partial observations emit no edge and
create no placeholder node.

ADR-0029 also defines one direct navigation relation now emitted by the
production builder:

```text
Procedure --Opens--> Form
```

The first producer is limited to a complete static literal `OpenForm(...)`
call inside an accepted Command-module Procedure. `CommonForm.<Name>` resolves
to a Common Form metadata node. An explicit
`<SupportedKind>.<Owner>.Form.<Name>` target resolves first to the exact typed
metadata owner and then to its exact subordinate Form child. The edge carries
resolved provenance, participates directly in generic dependency, usage, and
reverse Impact navigation, and does not create companion `References` or
`DependsOn` edges.

Default-form aliases, shorthand ListForm or ObjectForm spellings, dynamic or
computed targets, calls outside accepted Command-module Procedures, Form
internals, Form commands and events, Command Groups, multilingual subordinate
payload, explicit command execution, and other conceptual UI edges remain
deferred. The conceptual nested UI taxonomy remains a target vocabulary, not
the live public Rust layout or a wildcard endpoint policy.

The repository-owned Sprint 7 EDT fixture and focused negative/partial tests
now prove the independent source-independent graph, EDT parser, production
emission, Query, Diff, complete/incremental index equivalence, Impact,
diagnostic, report, determinism, and Coverage contracts. The EDT
`semantic_edge.opens` capability is therefore `Supported`; this evidence does
not promote the deferred UI taxonomy or mark Sprint 7 completed before its
integration review.

`MetadataKind::Form` and `NodeKind::Metadata(MetadataKind::Form)` are not
applicable to the EDT adapter: common forms are top-level
`MetadataKind::CommonForm` entities, while forms owned by documents, catalogs,
and other metadata objects emit the flat `NodeKind::Form` variant.
`MetadataKind::Unknown` is a source-independent fallback marker for metadata
kinds that are unknown to or not yet represented by the domain model. It is not
an EDT domain entity: EDT discovery has no directory mapping for it, metadata
parsers do not return it as a normal descriptor kind, and unsupported source
directories are ignored rather than emitted as unknown nodes. Consequently,
`metadata_entity.unknown` remains explicitly registered but is `NotApplicable`
to the EDT pipeline.
`semantic_node.metadata.unknown` is a distinct node-layer capability, but it is
also `NotApplicable` to EDT for the same production-source reason: without a
discovered and parsed `MetadataKind::Unknown` entity there is no stable source
object, descriptor path, semantic identity, or provenance contract from which to
emit `NodeKind::Metadata(MetadataKind::Unknown)`. The EDT adapter therefore does
not create synthetic unknown metadata entities or synthetic unknown metadata
nodes for forward compatibility. Unknown source directories remain ignored by
discovery, and reference-resolution failures are represented by typed
diagnostics rather than fallback metadata nodes.
The separate graph-domain `semantic_node.unknown` capability remains supported
as a source-independent model fallback, but the EDT-specific flat
`semantic_node.unknown` capability is also `NotApplicable`: the adapter has no
legitimate production source, identity contract, or provenance contract for
emitting a flat unknown semantic node.
Commands embedded in another metadata descriptor similarly emit the flat
`NodeKind::Command` variant.

### Semantic node inventory

The EDT pipeline currently emits:

* top-level `NodeKind::Metadata(kind)` nodes for the directory registry above;
* flat `Role` and `Subsystem` nodes derived from their EDT metadata objects
  while preserving the original `NodeKind::Metadata(...)` object nodes;
* `Module`, `Procedure`, and `Function` nodes from known BSL module files and
  declaration extraction;
* `Query` nodes from static BSL Query declarations inside known procedures or
  functions when the local binding and full query text are statically known;
* `Attribute`, `StandardAttribute`, `TabularSection`, `Form`, `Command`,
  `Dimension`, `Resource`, and `Measure` child nodes from metadata descriptors
  or deterministic metadata-object semantics.

EDT represents an accounting-register measure as an
`AccountingRegisterResource` in the `<resources>` collection. The generic
structure reader preserves that source vocabulary, while EDT graph conversion
maps a resource owned by `MetadataKind::AccountingRegister` to
`NodeKind::Measure`. Resources of accumulation and other register kinds remain
`NodeKind::Resource`. Measure identity uses the source UUID when available and
its provenance identifies the original accounting-register descriptor and
resource member.

EDT role objects produce two coexisting semantic representations: the original
`NodeKind::Metadata(MetadataKind::Role)` metadata object node and a flat
`NodeKind::Role` semantic node. The flat role node uses the role metadata object
UUID plus the `:role` suffix as its deterministic identity and attaches
provenance to the role `.mdo` descriptor with `fact=role_node` context. No
ownership edge is emitted for the flat role node because the current graph
validator does not require ownership for `NodeKind::Role`.

EDT subsystem objects follow the same specialized-node convention. The pipeline
preserves the original `NodeKind::Metadata(MetadataKind::Subsystem)` metadata
object node and emits a flat `NodeKind::Subsystem` semantic node with identity
`<subsystem_metadata_object_id>:subsystem`. Its provenance points to the
subsystem `.mdo` descriptor with `fact=subsystem_node` context. Subsystem
hierarchy, membership, command interface integration, and navigation behavior
are not inferred by this node capability.

Document metadata objects derive platform-provided `StandardAttribute` nodes
for `Ref`, `DeletionMark`, `Date`, `Number`, and `Posted`. The node identity
uses the existing graph-domain convention
`<metadata_object_id>:standard_attribute:<kind>`, and provenance points to the
document `.mdo` descriptor with `member=standard_attribute:<kind>` context.
Standard attributes are connected to their metadata object with the existing
`Contains` relation because `NodeKind::StandardAttribute` requires an owner at
graph validation time. The ownership edge uses the graph-domain
`StandardAttribute` insertion path, keeps deterministic edge identity through
source node, target node, and `EdgeKind::Contains`, and carries the same
member-level provenance as the standard attribute fact.

The flat `Unknown` variant is not emitted by the EDT pipeline. It is classified
as `NotApplicable` for EDT rather than `Unsupported` because unsupported source
directories are ignored by discovery and parser or resolution failures remain
typed diagnostics instead of graph fallback nodes. This distinguishes a missing
implementation for a valid EDT capability from an impossible adapter-specific
production path.

### Query entity contract

`NodeKind::Query` represents a stable source declaration of one complete
1C query-language program. A Query node is a semantic entity in the graph; it
is not the public Semantic Query API, not an arbitrary string that happens to
contain query text, not a BSL runtime `Query` object by itself, not a query
execution event, and not a `Reads`, `Writes`, or `DependsOn` edge.

The EDT pipeline implements the first Query source slice for static BSL Query
declarations inside known procedures or functions. The supported syntax is a
local binding initialized with `New Query("...")`, `Query = New Query;` followed
by `Query.Text = "..."`, or the Russian equivalents `Новый Запрос` and
`.Текст`. The extractor does not parse the 1C query language; it only proves
that one complete query program is statically available for a stable local
binding. The `semantic_node.query` coverage capability is `Supported` for this
slice.

#### Source categories

Query sources MUST be source declarations that can be rediscovered
deterministically and attached to one structural owner. The accepted source
categories are:

* static BSL query declarations: complete query text supplied directly in a BSL
  module through the supported constructor or `.Text` literal assignment forms;
* metadata-owned query declarations: complete query text stored inside a stable
  metadata member such as a report data-composition dataset or a dynamic-list
  query setting once the corresponding EDT metadata parser exists.

The following categories are possible future sources, but are not accepted by
the first implementation slice until their EDT representation and identity
rules are proven: standalone EDT artifacts that contain only query text,
report internals beyond named data-composition datasets, form dynamic-list
settings beyond their stable member path, and query text stored in external
resources tracked by the workspace.

Generated fragments, runtime-concatenated text, reassigned local bindings, text
loaded from an untracked external source, and partial query snippets MUST NOT
produce Query nodes in the implemented first slice. They MAY later produce
diagnostics or lower-confidence facts after a separate architecture decision.

#### Canonical entity boundary

One Query entity is one complete source declaration of a query program owned by
one graph node. A metadata member MAY contain one or more Query entities only
when each entity has a stable local declaration identity. A BSL procedure or
function MAY contain multiple Query entities when each static declaration can
be distinguished independently.

Query text itself is payload evidence for future extraction, but it is not the
primary identity. Formatting-only changes SHOULD NOT change identity when a
stable metadata member path or named BSL binding remains unchanged. Changing
the query body without changing the owner and local declaration identity MUST
preserve the same Query node identity and later appear as semantic content
change. Moving the declaration to another owner MUST create a different Query
identity. Moving an unnamed BSL declaration that has only a source-range
anchor MAY change identity; such declarations are therefore outside the first
implementation slice.

Nested subqueries, temporary-table statements, and multiple statements inside
one static query text are part of the same Query entity until query-language
analysis introduces child data nodes. Query nodes model source declarations,
not runtime executions or individual executions of a reused query object.

#### Ownership

Every Query node MUST have exactly one structural owner represented by the
existing `Contains` relation. For the implemented BSL source slice, the owner
MUST be the nearest known procedure or function node. Module-scope Query
declarations are not emitted by the first slice. For future metadata sources,
the owner SHOULD be the closest semantic metadata member node when such a node
exists; otherwise the owner is the metadata object node and the Query provenance
MUST include the nested member path that produced it.

No new ownership edge kind is required for Query node support. Data-access
relations derived from query-language analysis are separate semantic edges and
MUST NOT be treated as ownership.

#### Stable identity

Query identity MUST reuse the existing deterministic `NodeId` strategy:

* source UUID when the source declaration has its own stable UUID;
* owner identity plus canonical metadata member path for EDT metadata members;
* owner identity plus canonical BSL declaration identifier for named static BSL
  query declarations;
* source path and source range only as a secondary disambiguator when no more
  stable local identity exists.

Identity MUST NOT use filesystem traversal order, map iteration order,
display text alone, a collection index without a stable source contract, or a
hash of the full query text as the primary key. Query text MAY participate in a
semantic-content fingerprint used by future diffing, but not in the stable
node identity.

Multiple Query entities under one owner MUST be distinguished by stable local
declaration identity: metadata member path and dataset name for metadata
sources, or named BSL binding plus source declaration anchor for BSL sources.
Duplicate declarations with the same owner and local identity MUST be rejected,
deduplicated, or diagnosed deterministically by a future implementation; they
MUST NOT be resolved by insertion order.

#### Provenance

Query provenance MUST identify the source artifact, producer, owner context,
and local declaration context. For EDT metadata sources, provenance MUST include
the descriptor path and canonical member path, such as a dataset or dynamic-list
query element. For BSL sources, provenance MUST include the BSL module source
and enough local context to identify the declaration, such as procedure or
function, binding name, and source range when available.

The current `Provenance` model can represent Query facts by using a stable
source identifier with a path fragment that encodes member or BSL declaration
context. It does not yet provide a structured public source-range type; if a
future implementation needs first-class ranges for BSL Query provenance, that
range model is a separate prerequisite and MUST NOT be added implicitly by
Query node emission.

The implemented BSL slice uses the module source path plus a deterministic
fragment containing the Query node id, owner id, and local binding name. The
node and its `Contains` ownership edge receive `Declared` provenance from the
EDT BSL graph contributor.

#### Extraction boundary

Query extraction is distinct from 1C query-language parsing. The minimum
extraction result needed to create a Query node is: owner identity, local
declaration identity, source artifact, provenance source identifier, and the
complete raw query text or a pointer to it. Full query-language AST,
referenced tables, parameters, read/write sets, temporary-table analysis, and
dependency classification belong to later Data Access Graph work.

The first Query node implementation does not require full query-language
analysis. It preserves the static raw query text in the extracted BSL model so
later analysis can derive `Reads`, `Writes`, `DependsOn`, query fields, query
parameters, and diagnostics without changing the Query node identity.

#### Relation to Query API and semantic edges

The Semantic Query API is a graph access interface over an already-built graph.
It does not produce Query nodes and does not define Query entity identity.
Query nodes, once emitted, will be retrievable through the existing Query API
like any other node kind unless a concrete API gap is found later.

Query node support is independent from `Reads`, `Writes`, and `DependsOn`.
Current extraction creates the `NodeKind::Query` node and its ownership edge for
the supported BSL slice. Production query-language analysis now emits Reads and
normalized DependsOn for the parser's completely accepted fixture-backed forms
and uniquely resolved Catalog, Information Register, Accumulation Register, or
Accounting Register targets. `Writes` has a separate accepted
architecture contract in `docs/adr/0022-writes-semantics.md`; its first slice is
a BSL Procedure mutation fact rather than a Query fact, and its implementation
and registry transition are complete. Query-derived `DependsOn` remains a
separate normalized relation derived from the retained Reads fact. None of
these edges is a prerequisite for creating the Query node,
and Query identity and ownership do not depend on query text.

#### Determinism, errors, and first slice

Query extraction MUST be deterministic: source discovery order, filesystem
enumeration order, and map iteration order MUST NOT affect identity, ordering,
or provenance. Repeated extraction of the same source MUST produce the same
Query entity and repeated graph builds MUST produce an empty graph diff and
empty build-result diff when inputs are unchanged. Duplicate extraction of the
same declaration MUST be deduplicated or diagnosed deterministically.

Empty query text, malformed query language, unsupported source format,
dynamically constructed text, partial snippets, ambiguous reassignment, missing
owner identity, and missing provenance do not produce a supported Query node in
the implemented first slice. Malformed query language may still produce a Query
node when extraction has a complete static source declaration; the Reads
producer now reports the current parser's typed syntax diagnostic and emits no
Reads edge. Parser failures in BSL Query extraction remain distinct from
query-language diagnostics.

The first Query-node implementation slice targets static BSL query declarations
inside a known procedure or function. This source has a real EDT input family, stable
module ownership, existing BSL module discovery, an existing symbol owner model,
and provenance source identifiers. The slice is restricted to declarations with
a stable local binding and complete statically available text. The later Reads
producer consumes this Query entity without changing its identity or ownership;
the accepted first Writes slice uses existing Procedure nodes independently,
and the accepted direct-source Query Reads and DependsOn producer consumes the
Query without changing it.

The ordered follow-up tasks are:

1. Completed: follow `docs/adr/0021-reads-semantics.md` and investigate the minimum
   query-language grammar, bilingual lexical behavior, source locations, and
   typed diagnostics using repository-owned real-source evidence;
2. Completed: implement query-language parsing, a typed source model, diagnostics, and
   all-or-nothing first-slice completeness without graph emission;
3. Completed: resolve accepted persistent query sources to exact metadata kinds and
   normalized local names without placeholder targets;
4. Completed: add precise Reads validation for the accepted endpoint matrix
   without graph emission or a Coverage transition;
5. Completed: emit EDT Reads edges with deterministic provenance and focused,
   production, Query, Impact, negative, and repeated-build integration evidence;
6. Completed: follow `docs/adr/0022-writes-semantics.md` for the
   narrow `Procedure --Writes--> Metadata(AccumulationRegister)` contract based
   on a complete `RegisterRecords.<Name>.Write()` statement and the owning
   Document's matching `<registerRecords>` declaration;
7. Completed: preserve typed Document register-record declarations, implement
   complete Writes candidate extraction, exact resolution, precise validation,
   production emission, integration evidence, and the final registry-only
   transition to `Supported` as ordered independent tasks;
8. Completed: derive Query `DependsOn` from the terminal public QuerySource
   request and retained Reads fact for the four accepted direct persistent
   source families;
9. add metadata-owned Query sources such as data-composition datasets or
   dynamic-list query settings after their EDT parser contracts are defined.

### Ownership inventory

`Contains` is stored from owner to child. EDT emits configuration-to-object,
metadata-object-to-module, module-to-procedure/function, and
metadata-object-to-child relations with provenance. Validation constrains
containment endpoints and single-owner rules; Query exposes owners and children;
Impact Analysis can opt into child-to-owner and owner-to-child propagation.

Attribute ownership preserves the immediate semantic owner. Top-level
attributes remain owned by their metadata object, while an attribute nested in
a tabular section is owned only by the nearest enclosing `TabularSection` node.
The XML reader emits each tabular section before its buffered nested attributes,
uses the immediate owner in UUID-less fallback identity, and keeps source UUIDs
unchanged. The EDT contributor inserts all child nodes and collects their type
references before it emits ownership edges, so graph construction does not
depend on nested XML completion order. Measure node conversion reuses generic
child containment: an accounting-register metadata object owns each emitted
`NodeKind::Measure` through the existing `EdgeKind::Contains` relation. The
edge uses the same deterministic source, target, kind identity strategy as other
graph edges and carries provenance pointing to the accounting-register `.mdo`
descriptor with `edge=contains` context.

### Reference and resolution inventory

Metadata type references are extracted only from Attribute, Dimension, and
Resource `<types>` values. Typed prefix mappings currently cover Catalog,
Document, Enumeration, Information Register, Accumulation Register, Accounting
Register, Calculation Register, Business Process, and Task targets. Requests
preserve source object, source member, role, expected target kind, target name,
and descriptor path. Resolution emits `References` edges or typed missing,
ambiguous, and incompatible-kind diagnostics; both outcomes update reference
statistics and receive provenance. Catalog and Document targets have successful
representative integration fixtures. Enumeration, Information Register,
Accumulation Register, Accounting Register, Calculation Register, Business
Process, and Task now share the same production-builder representative test.
For every mapped target kind it proves exact source and target identity,
`References`, companion first-slice `DependsOn`, deterministic provenance,
Query navigation, graph validation, resolution statistics, and repeated-build
stability. All nine metadata-reference capabilities are `Supported`.

ADR-0024 is implemented by the public, source-independent
`SemanticReferenceRequest` build-observation type in `oneagent-graph`. Stable
identity is derived from source node, semantic category, typed target reference,
and the sorted expected-kind set; candidates, current resolution state, typed
outcome, and provenance are mutable content of that identity. Checked lifecycle
transitions preserve collection provenance and append resolver evidence. The
deterministically ordered ledger supplies immutable filters, request-aware
reports, added/removed/modified build diffs, and typed build validation.
Requests remain build observations rather than graph nodes or edges, so graph
equality and `SemanticGraphQuery` remain graph-only.

The EDT first slice converts accepted Attribute, Dimension, and Resource type
observations to `SemanticReferenceCategory::MetadataType` at collection time.
Descriptor paths and EDT roles remain private projection evidence. Terminal
requests canonically drive `References`, ADR-0017 `DependsOn`, failed
diagnostics, and accepted-request statistics once; non-migrated request families
remain in explicitly named legacy statistics during migration. Complete and
explicitly partial workspace scopes preserve missing versus partial outcomes
without placeholder nodes. `EdtSemanticGraphBuildResult` exposes ordered
requests and their immutable query, and its report, diff, and validation paths
use the ledger.

Current ADR-0024 evidence is mapped as follows:

| Criterion | Evidence |
|---|---|
| Checked identity, lifecycle, ordering, aggregation, and query filters | `oneagent_graph::reference_request` unit tests |
| Report derivation, partial compatibility mapping, request diff, and validation | `crates/graph/tests/reference_request_build.rs` |
| All nine mapped metadata target kinds and repeated-build stability | `oneagent_edt::graph_tests::resolves_all_mapped_metadata_reference_target_kinds_through_production_builder` |
| Collection and resolver provenance with exact resolved projections | `oneagent_edt::graph_tests::resolves_metadata_reference_and_depends_on_edges` |
| Missing, ambiguous, and incompatible terminal projections | focused `oneagent_edt::graph_tests::*_metadata_reference_*` tests |
| Explicit partial production orchestration with no placeholder or failure projection | `oneagent_edt::graph_tests::production_builder_preserves_explicit_partial_workspace_request` |
| Duplicate aggregation without ledger, edge, diagnostic, or statistics duplication | `oneagent_edt::graph_tests::duplicate_metadata_type_reference_creates_one_depends_on_edge` and `duplicate_identical_reference_diagnostic_is_deduplicated` |
| Stable identity and modified-not-remove/add lifecycle diff | `oneagent_edt::graph_tests::request_identity_survives_missing_to_resolved_production_diff` |
| Public QuerySource lifecycle, four direct source kinds, terminal outcomes, and production projections | focused `oneagent_edt::query_source_resolution` tests and `sprint8_full_builder_matrix_is_complete_deterministic_and_consumer_visible` |

BSL calls, Writes targets, protected resources, Subsystem content, and extension
targets remain private until each family defines source identity, category,
completeness, projection, duplicate, and statistics contracts. Query sources
use the public request lifecycle for the accepted four direct persistent source
families. The
graph-domain and EDT ReferenceRequest Coverage entries transitioned
independently after their respective audits passed. Both entries are now
`Supported`, and Roadmap item 24 is complete for the accepted metadata-reference
first slice.

BSL calls are extracted and local or qualified calls can resolve to `Calls`
edges. Every extracted call now contributes exactly one final reference outcome:
an unqualified call is handled by local resolution and a qualified call by
cross-module resolution. Successful outcomes emit a `Calls` edge; unresolved
outcomes emit the existing typed unresolved-reference diagnostic and update EDT
build reference statistics. Diagnostic provenance identifies the source BSL
file and stable call identity, and the source procedure or function is attached
when available. There is no resolved-without-edge path in the current metadata
reference flow: successful metadata resolution immediately emits a `References`
edge.

### Edge, validation, query, and impact inventory

The EDT pipeline emits all nine current edge kinds with provenance: `Contains`,
`Calls`, `References`, `Reads`, `Writes`, `Grants`, `Includes`, `Extends`, and
the first `DependsOn` slice. Reads production is limited to the parser's
accepted forms and the Catalog and Information Register allowlist. Writes
production is limited to accepted Document Object Module Procedures and
resolved Accumulation Register targets. Unsupported source forms are retained
as typed outcomes and emit no placeholder edges.

ADR-0025 audits every endpoint rule. `Contains` uses the accepted ownership
matrix; `Calls` uses callable endpoints; `Reads`, `Writes`, `Grants`,
`Includes`, `Extends`, and `DependsOn` match their accepted first-slice ADRs.
`References` now also uses its accepted first-slice matrices. All nine rules
are precise rather than permissive fallbacks; Unknown and unsupported future
endpoint families are rejected.

The References policy preserves both current production slices:
Attribute, Dimension, or Resource to one of the nine mapped metadata-reference
target kinds; and AccessRight to one of the five protected-resource metadata
kinds implemented by the Grants pipeline. Its exact 32 accepted pairs and all
1,489 other pairs over the current 39-kind node inventory have deterministic
graph-domain schema evidence. Typed graph-validation tests retain endpoint
kinds, node and edge identities, provenance context, and issue ordering;
metadata-reference and Grants production regressions remain valid. Every other
pair, including Unknown and placeholder endpoints, is deferred. Roadmap item 25
is complete without a Coverage transition or count change.
Validation also checks missing endpoints, ownership, forbidden self-loops,
ownership cycles, node and edge provenance, and build/report counter
consistency.

The Query API exposes all stored edge kinds. Dependency and usage classification
includes `Calls`, `References`, `Reads`, `Writes`, and `DependsOn`; `Contains` is
handled by ownership navigation. Impact Analysis uses the same dependency
classification and supports optional `Contains` ownership propagation. `Grants`,
`Includes`, and `Extends` are intentionally excluded from the first impact
policy.

`EdgeKind::Reads` is governed by
`docs/adr/0021-reads-semantics.md`. It represents a direct resolved persistent
data-access fact stored as `reader --Reads--> data source`. The accepted first
production slice uses the existing static BSL Query entity as the source and an
exact resolved top-level metadata object as the target:

```text
NodeKind::Query
    --Reads-->
NodeKind::Metadata(MetadataKind::Catalog | MetadataKind::InformationRegister |
                   MetadataKind::AccumulationRegister |
                   MetadataKind::AccountingRegister)
```

The first slice accepts only one completely parsed top-level `SELECT` statement
with exactly one direct persistent source and no additional source-producing
construct. Joins, unions, nested queries, multiple statements, batches,
temporary tables, virtual tables, external or parameter data sources, dynamic
or incomplete text, malformed input, and metadata namespaces outside the
explicit allowlist produce typed diagnostics and no Reads edge for the Query.
The producer must not emit a supported fragment from an otherwise unsupported
query because that would expose an incomplete read set as complete evidence.

Repository evidence does not define a complete 1C query-language grammar. The
committed parser therefore proves complete first-slice source classification
only for its accepted fixture-backed forms and preserves deterministic source
locations. The public QuerySource request lifecycle uses the parsed metadata
kind and normalized local name, requires exactly one compatible in-graph target,
and does not invent placeholder, Unknown, or external nodes. Its terminal ledger
preserves deterministic collection and resolver provenance and distinguishes
missing, ambiguous, incompatible, and partial-workspace outcomes.

Reads uses standard `(source, target, EdgeKind::Reads)` identity and resolved,
exact provenance. Multiple source occurrences for one Query-target pair support
one edge; distinct evidence is sorted and deduplicated before insertion. The
Reads validator accepts only Query-to-allowlisted-Metadata endpoints. The Writes
validator independently enforces the accepted
Procedure-to-Accumulation-Register matrix.

The existing Query API and Impact Analysis classification remain unchanged.
Reads and its normalized Query-origin DependsOn participate in outgoing
dependency, incoming usage, and reverse dependency-to-usage Impact traversal.
Impact reports one affected Query with deterministic reasons for both edges.
No reverse edge, transitive closure, dedicated query method, References edge,
or ownership projection is added by this slice.

`semantic_edge.reads` is `Supported`. Parser investigation, typed
parsing and diagnostics, exact metadata resolution, precise validation,
production EDT emission, provenance aggregation, positive and negative tests,
Query and Impact evidence, full-builder integration, and repeated-build
determinism are complete. The confirmed pipe-style multiline decoder retains
the private copied/collapsed-quote/inserted-LF source map. Repository-owned raw
fixtures and parser/full-builder tests cover unsupported structure, virtual
tables, and temporary tables without partial source programs or Reads edges.
The registry records the complete required evidence and the representative
production integration test. The Sprint 8 real-format fixture extends that
evidence across all four accepted source families without changing the
capability status or registry aggregate counts.

`EdgeKind::Writes` is governed by
`docs/adr/0022-writes-semantics.md`. It represents a direct resolved persistent
mutation fact stored as `writer --Writes--> persistent mutation target`. The
accepted smallest production slice is:

```text
NodeKind::Procedure
    --Writes-->
NodeKind::Metadata(MetadataKind::AccumulationRegister)
```

The source must be an existing Procedure in the Object Module of an EDT
Document. A Writes-specific extractor must prove one complete standalone,
zero-argument `RegisterRecords.<Name>.Write()` statement. The owning Document
descriptor must independently declare exactly
`AccumulationRegister.<Name>` in `<registerRecords>`, and resolution must find
exactly one compatible existing top-level metadata node. The syntax, owner
declaration, and resolved target are jointly required; no one source of evidence
is sufficient by itself.

This contract distinguishes platform Document register-record persistence from
file, binary, text, archive, UI, external, dynamic, and local object writes that
share the `.Write(...)` spelling. The current `BslCall` model can locate a
qualified candidate and its containing symbol, but it does not preserve
arguments, receiver type, or general local value flow. The first slice therefore
does not accept `ProductSale.Write(...)`, arbitrary object variables, chained or
aliased receivers, argument-bearing calls, other register families, localized
spellings, or Query mutations. Unsupported, missing, ambiguous, incompatible,
and dynamic candidates produce typed diagnostics and no placeholder, Unknown,
external, or lower-confidence graph target.

Writes uses standard `(source, target, EdgeKind::Writes)` identity and exact
resolved provenance covering the Procedure, Object Module, owning Document,
source occurrence, matching descriptor declaration, and resolved target.
Repeated occurrences for one Procedure-target pair support one edge; distinct
provenance is sorted and deduplicated before insertion. The validator enforces
only the accepted Procedure-to-Accumulation-Register matrix.

Existing Query and Impact behavior is unchanged: Writes remains an outgoing
dependency, incoming mutation usage, and reverse Impact traversal edge. The
first producer emits no companion Calls, References, Reads, Grants,
DependsOn, or Contains fact. Role rights, Reads, generic Calls, and metadata
declarations do not imply Writes.

The accepted first slice is implemented end to end: typed Document
register-record declarations, complete candidate extraction, exact owning
declaration and target resolution, precise validation, canonical production
emission, deterministic provenance aggregation, typed diagnostics, and
production, negative, Query, Impact, duplicate, and repeated-build evidence are
present. The final registry-only transition records the complete required
evidence and classifies `semantic_edge.writes` as `Supported`. Deferred register
families, object persistence, value/type flow, query mutation, localized
spellings, and write-derived `DependsOn` remain outside this slice.

`EdgeKind::DependsOn` is governed by
`docs/adr/0017-depends-on-semantics.md`. It is a materialized normalized direct
semantic dependency stored as `dependent --DependsOn--> dependency`. EDT emits
the first implementation slice for resolved metadata member type references:
`Attribute`, `Dimension`, or `Resource` nodes depend on resolved
`Metadata(...)` target nodes. Primitive, built-in, unresolved, ambiguous, and
incompatible type references do not emit `DependsOn`. The edge uses the
standard `(source, target, EdgeKind::DependsOn)` identity and derived
provenance identifying the metadata member type reference. The implemented
Sprint 8 slice also derives Query-origin dependencies from terminal resolved
QuerySource requests and their retained Reads facts for Catalog, Information
Register, Accumulation Register, and Accounting Register targets.

### Accepted Sprint 8 direct register Query boundary

The Sprint 8 source investigation in
`docs/architecture/register-query-source-investigation.md` distinguishes Query
declaration sources, query-language persistent sources, register virtual
tables, register metadata objects, and semantic relations. ADR-0030 accepts the
additive implementation boundary now present in production.

The implemented parser expansion retains the existing static named BSL Query
entity and the all-or-nothing one-`SELECT`, one-direct-source contract. It adds
only these exact persistent source categories:

```text
AccumulationRegister.<Name>
AccountingRegister.<Name>
```

They map to existing top-level
`NodeKind::Metadata(MetadataKind::AccumulationRegister)` and
`NodeKind::Metadata(MetadataKind::AccountingRegister)` targets. Catalog and
Information Register remain the implemented compatibility baseline.
Calculation Register, other metadata namespaces, Russian register namespace
spellings without repository evidence, JOIN, UNION, nesting, batches,
temporary/external/parameter sources, general expression grammar, and register
virtual tables remain deferred. A virtual-table occurrence continues to emit
only its typed no-edge diagnostic; no base-register access is inferred.

Accepted parsed source observations use the existing public
`SemanticReferenceCategory::QuerySource` lifecycle. The canonical request
source is the existing Query node, the request has exactly one expected
metadata kind, collection and resolver provenance remain deterministic, and
missing, ambiguous, incompatible, partial, malformed, unsupported, dynamic, or
incomplete outcomes create no placeholder or resolved edge. Statistics are
derived once from terminal requests rather than independently from projections.

Unique resolution retains the direct fact and adds one normalized fact:

```text
Query --Reads-----> Metadata(Catalog | InformationRegister
                              | AccumulationRegister | AccountingRegister)
Query --DependsOn-> Metadata(Catalog | InformationRegister
                              | AccumulationRegister | AccountingRegister)
```

`Reads` remains resolved, direct data-access evidence. `DependsOn` is a derived
normalized direct dependency proven by the same terminal resolved request and
retains the Reads fact. Both use standard `(source, target, edge kind)` identity
and sorted, deduplicated provenance. Generic dependency queries intentionally
observe both relations; reverse Impact keeps one affected Query node with
deterministic per-edge reasons. No reverse relation or transitive closure is
stored.

The graph validator enumerates the four target kinds for both Query-origin
relations rather than accept a wildcard Metadata target. Existing Attribute,
Dimension, Resource, and Command `DependsOn` matrices remain additive and
unchanged. Writes remains an independent Procedure mutation fact and does not
gain a companion dependency in this slice.

The existing Query, Reads, DependsOn, and ReferenceRequest Coverage capabilities
remain `Supported` for their current implemented slices. The Sprint 8
real-format EDT fixture and production-builder evidence cover exact parser,
request, resolution, validation, emission, Query, Diff, Impact, report,
diagnostic, repeated-build, source-order, and clean/incremental-index-equivalence
behavior. The EDT registry remains 101 capabilities (96 `Supported`, 5
`NotApplicable`) and the graph registry remains 85 capabilities (82
`Supported`, 3 `NotApplicable`), with no Critical, High, or Medium gaps.

`EdgeKind::Extends` is governed by
`docs/adr/0018-extends-semantics.md`. It represents an explicit, resolved,
direct extension relation stored as `extending entity --Extends--> directly
extended base entity`. The first production slice is metadata-object extension:
`NodeKind::Metadata(kind)` extends another resolved `NodeKind::Metadata(kind)`
of the same kind when an EDT source artifact explicitly declares the base,
borrowed, original, or extended metadata object. EDT parses adopted metadata
object descriptors through `ObjectBelonging=Adopted` and
`ExtendedConfigurationObject=<uuid>`, resolves the declared target by stable
metadata object id, emits `Extends` only when source and target are
`NodeKind::Metadata` of the same `MetadataKind`, and skips missing,
incompatible, or self-extension facts. The edge uses the standard
`(source, target, EdgeKind::Extends)` identity and resolved provenance
identifying the adopted EDT descriptor and declared target id.

`EdgeKind::Grants` is governed by
`docs/adr/0019-grants-semantics.md`. It represents an explicit, direct,
declared allow grant from an access subject to a scoped access-right entity:
`access subject --Grants--> scoped access right`. The accepted first production
slice is EDT role object-right declarations represented as
`NodeKind::Role --Grants--> NodeKind::AccessRight`, where the target preserves
both protected-resource identity and right identity. `NodeKind::AccessRight`
is a first-class graph-domain node representing one stable scoped access
capability: one right or operation applied to one protected resource. Its
identity is `AccessRight(protected_resource_identity, right_identity)` and is
stored as a deterministic component-preserving `EntityId` independent from
display name, provenance, parser state, insertion order, or random UUIDs. A direct
`Role --Grants--> Metadata(...)` edge is not accepted because it would collapse
multiple rights on the same protected resource into one graph edge identity.
The graph validator accepts only the precise endpoint shape
`NodeKind::Role --Grants--> NodeKind::AccessRight`; broad metadata targets and
unrelated source kinds are rejected. EDT production support reads the adjacent
`Rights.rights` for every discovered Role, accepts only explicit `true`
declarations, and supports `Configuration`, `Catalog`, `Document`,
`InformationRegister`, and `AccumulationRegister` protected-resource prefixes.
Resolved declarations emit one shared scoped access-right node per
resource/right pair, one Grants edge per role/access-right pair, and a companion
`AccessRight --References--> Metadata(...)` edge. Provenance is aggregated,
sorted, and deduplicated before graph insertion. False values, default flags,
and unsupported authorization declarations do not add other authorization facts;
missing, ambiguous, incompatible, and unsupported targets create no grant or
placeholder node. `semantic_node.access_right` and `semantic_edge.grants` are
Supported by the EDT Coverage Registry.

The Sprint 9 extension governed by
`docs/adr/0031-conditional-grants-semantics.md` is implemented. It preserves an optional EDT
`restrictionByCondition/condition` as opaque typed AccessRight content without
parsing or evaluating the expression. Unconditional AccessRight identity remains
byte-for-byte compatible. A conditional right appends one length-delimited
canonical condition component to the existing resource/right identity so that
conditional and unconditional declarations, and distinct condition texts,
cannot merge. The endpoint matrix remains Role-to-AccessRight Grants plus the
AccessRight-to-resource References companion. The production EDT builder carries
the typed payload through resolution and deterministic aggregation, records the
present or absent restriction in provenance, and exposes it unchanged through
Query, Resolution, Diff, Impact, reports, validation, and complete or incremental
Semantic Index views. Real-fixture, duplicate, reordered, repeated-build,
Coverage-regression, and clean-rebuild-equivalence tests provide the production
completion evidence. The Sprint 9 integration review records `pass`.
`Grants` is distinct from `Includes` membership, `Contains` ownership,
`Reads`/`Writes` data access, `DependsOn` dependencies, effective runtime
authorization, denied access, inherited access, user assignment, access groups,
and BSP access-profile semantics.

`EdgeKind::Includes` is governed by
`docs/adr/0020-includes-semantics.md`. It represents direct, explicit,
source-declared composition membership. The accepted EDT source contract is a
direct repeated `mdclass:Subsystem/content` value in a top-level
`src/Subsystems/<Name>/<Name>.mdo` descriptor, stored as
`NodeKind::Subsystem --Includes--> NodeKind::Metadata(kind)`. The source is the
existing flat Subsystem node. A `Role.<Name>` token resolves to
`NodeKind::Metadata(MetadataKind::Role)`, never to the flat `NodeKind::Role`
access-control subject. Direction is from the declaring Subsystem to the direct
member, standard edge identity is `(source, target, EdgeKind::Includes)`, and
transitive closure is not stored.

Production discovery now covers top-level and nested Subsystems through the
exact `Subsystems/<Name>` hierarchy accepted by ADR-0032. Every nested
descriptor must agree with its parent's direct `subsystems` declaration, its
own complete qualified `parentSubsystem`, and its immediate physical nesting.
The producer resolves direct content by exact metadata kind and local name,
does not invent placeholder targets, and attaches deterministic resolved
provenance containing the project-relative descriptor path, raw token, parsed
target, resolved node, and a stable subsystem-content resolution producer
stage. Unsupported metadata families and semantic meaning for
`Subsystem.<...>` content remain deferred.
Includes is distinct from `Contains` ownership, `References` linkage, `Grants`
authorization, and `DependsOn` dependency. It remains excluded from dependency
and Impact traversal; generic outgoing, incoming, and all-edge queries are
sufficient for direct membership navigation.

Direct source extraction is implemented by `EdtSubsystemContentReader`. The
reader accepts every hierarchy-reader-discovered Subsystem metadata descriptor,
requires the `mdclass:Subsystem` root, and returns the Subsystem metadata ID,
descriptor path, and only direct child `<content>` observations. Raw tokens are
XML-decoded without trimming, case conversion, aliasing, localization, or
qualified-name splitting; empty observations are preserved, while equivalent
observations are sorted and deduplicated deterministically. Missing direct
content is valid, descendant content is ignored, and malformed or wrong-kind
descriptors return typed reader errors.

The production builder collects deterministic pending observations while
processing the complete validated hierarchy, normalizes only the explicit
ADR-0020 prefix allowlist, and resolves after all metadata and flat Subsystem
nodes exist. It emits configuration ownership for nested metadata Subsystems and
direct hierarchy Includes between flat Subsystems with exact hierarchy
provenance. Malformed qualified content tokens and unsupported or deferred
prefixes use distinct typed diagnostics and reference-statistics outcomes;
missing, ambiguous, and incompatible targets reuse the exact graph resolution
categories. Resolved source-target pairs are aggregated through canonical edge
identity, and equivalent provenance is sorted and deduplicated before insertion.
The graph validator accepts both the allowlisted metadata-member endpoint and
the flat Subsystem hierarchy endpoint, while rejecting self-loops and directed
Subsystem hierarchy cycles deterministically.

This accepted contract partially supersedes only ADR-0017's older classification
of Subsystem membership as `Contains`; ADR-0017's `DependsOn` decisions remain
unchanged, as do ADR-0007 configuration ownership and ADR-0019 access-grant
semantics. The production path, exact resolution, canonical emission, validator,
generic query evidence, dependency and Impact exclusions, negative outcomes,
determinism evidence, and real-format integration fixture are implemented, so
`semantic_edge.includes` is `Supported` by the EDT Coverage Registry.

Sprint 10 is governed by
`docs/adr/0032-subsystem-hierarchy-semantics.md`. The accepted extension keeps
the existing UUID-derived metadata and flat Subsystem representations, adds
direct `NodeKind::Subsystem --Includes--> NodeKind::Subsystem` hierarchy only
when the parent's `<subsystems>`, the child's qualified `<parentSubsystem>`, and
the immediate physical nesting agree, and applies the ADR-0020 direct content
contract to every successfully discovered nested Subsystem. Configuration
ownership of metadata Subsystem objects remains `Contains`; flat Subsystem
hierarchy remains composition rather than ownership.

Only direct hierarchy and direct metadata membership are canonical graph facts.
Transitive metadata membership is an ordered, cycle-safe, source-independent
Query projection across outgoing Subsystem hierarchy edges and is never stored
as derived Includes closure. Includes remains excluded from dependency and
Impact traversal. `Subsystem.<...>` content, command-interface behavior,
directory-only inference, placeholder Subsystems, contradictory-source
recovery, and unrelated metadata-family expansion remain deferred. The tracked
Sprint 10 production fixture and generated transition matrices cover five
source-proven depths, duplicate local names, shared/nested direct content,
deferred self-content, provenance, deterministic consumers, and complete and
incremental index equivalence. The coarse Coverage capabilities remain
`Supported` without registry or aggregate-count changes. The Sprint 10
integration review records `pass`.

Sprint 11 Event Subscriptions is governed by
`docs/adr/0033-event-subscription-semantics.md`. The accepted first slice adds a
top-level `MetadataKind::EventSubscription` with UUID identity, configuration
ownership, optional common synonym, and closed typed event-name payload.
Source selectors and handler paths remain relation evidence rather than copied
payload.

The accepted canonical relations are direct resolved
`Metadata(EventSubscription) --References--> Metadata(supported source kind)`,
`Metadata(EventSubscription) --References--> Procedure`, and
`Metadata(EventSubscription) --Triggers--> Procedure`. Qualified source
selectors resolve one exact name and kind; bare source-family selectors resolve
the complete stable-ID-ordered set of current graph metadata nodes for the
mapped family. The first source matrix is limited to Catalog, Document,
Information Register, Accumulation Register, Accounting Register, Calculation
Register, Business Process, and Task. Unsupported Constants, Defined Types,
Exchange Plans, and Chart families remain typed diagnostics without Unknown or
placeholder targets.

Handlers resolve through exact declared Common Module ownership to one
Procedure. Export status is not part of this ownership contract. A multiline
declaration audit corrected the planning observation: all 93 unique live
handler paths are exported, while the tracked reduced fixture recomposes an
exact live non-exported owned Procedure as a handler target to exercise the
accepted export-agnostic rule. `Triggers` has only the
EventSubscription-to-Procedure endpoint and is not independently added to the
dependency or Impact policy; the companion References fact supplies the
existing dependency navigation. Multi-target source selectors remain outside
the ADR-0024 public single-target request ledger until a compatible request
lifecycle is accepted.

Sprint 11 Tasks 1-5 now provide production and executable evidence. EDT
discovers `EventSubscriptions`, preserves typed payload and configuration
ownership, resolves exact/family sources and owned handlers after BSL symbol
insertion, emits aggregated source and handler `References` plus handler
`Triggers`, and projects deterministic diagnostics and statistics without
adding public reference requests. The tracked
`adapters/edt/tests/fixtures/sprint11_event_subscriptions_project/` fixture
records live paths, source hashes, reduction treatment, and fixture SHA-256
values. Generic Query, Diff, report, Validation, dependency/Impact policy,
complete index, and incremental clean-rebuild transitions cover subscription,
event, source, handler, and relation changes.

Executable registry state is Graph Domain 88 capabilities: 84 `Supported` and
4 `NotApplicable`; EDT 104 capabilities: 99 `Supported` and 5
`NotApplicable`. Both registries have zero Critical, High, or Medium gaps.
`MetadataKind::EventSubscription`, its EDT metadata node, and
`EdgeKind::Triggers` have complete production evidence. The Sprint 11
integration review records `pass`; Sprint 11 is completed and Sprint 12 SKD
and Report Model is the next planning target.

Sprint 12 SKD and Report Model is governed by the accepted planning contract in
`docs/adr/0034-report-data-composition-semantics.md` and the repository-owned
source evidence in
`docs/architecture/report-data-composition-source-investigation.md`. The first
slice adds source-independent Data Composition Schema, direct Data Set, and
direct named Data Composition Field nodes plus metadata-owned Query declarations
under existing Report metadata. Immediate ownership is
`Report --Contains--> DataCompositionSchema --Contains--> DataSet`, with
`DataSet --Contains--> DataCompositionField` and `DataSet --Contains--> Query`.

Schema identity uses the declared Report-template UUID. Direct Data Set and
Field identities are collision-safe owner/local-name tuples, and each accepted
direct Query uses a fixed role under its Data Set. Main-schema selection, Data
Set kind/local data source, and Field data path are typed semantic content rather
than identity. Eight nested duplicate-name Union children and six field folders
remain typed deferred source observations because the repository supplies no
stable first-slice identity for them.

The real corpus contains 46 direct-or-nested DCS queries, and none satisfies
the current complete-source Query parser. Sprint 12 therefore preserves 38
direct metadata-owned Query declarations without emitting QuerySource requests,
Reads, DependsOn, References, partial source candidates, or query-language
diagnostics. DCS query grammar, virtual tables, batches, temporary tables,
nested Union entities, field folders, settings, and runtime composition remain
deferred.

Sprint 12 Tasks 1-4 now provide graph-domain, parser, production, and executable
evidence for this bounded slice. The EDT Report path joins declared template
UUID/name/main-selection values to exact `.dcs` artifacts, emits typed accepted
entities with deterministic content-bearing provenance, and projects nested,
folder, and unsupported observations only through typed diagnostics and legacy
rejected-observation statistics. Fatal structural source failures produce no
successful partial build result. The tracked
`adapters/edt/tests/fixtures/sprint12_report_data_composition_project/` fixture
records exact ignored live paths, source hashes, reduction treatment, and
reduced-artifact SHA-256 values for Query, Object, Union, empty main, non-main,
nested-deferred, and folder-deferred shapes.

Generic Query, Diff, report, Validation, Impact exclusion, complete index, and
incremental clean-rebuild evidence cover Schema/DataSet/Field/Query
add/remove, main-role, Data Set kind/source, Field path, Query text, ownership,
and deferred-observation transitions. The public request ledger remains empty
for DCS queries, and no DCS `Reads`, `DependsOn`, or `References` relation is
emitted. Executable registry state is Graph Domain 91 capabilities: 87
`Supported` and 4 `NotApplicable`; EDT 110 capabilities: 105 `Supported` and 5
`NotApplicable`. Both registries have zero gaps. Sprint 12 remained active until
the independent Task 5 integration review recorded a non-blocking decision.

The
[Sprint 12 integration review](../reviews/sprint-12-skd-report-model.md)
subsequently records `pass` against committed Task 4 head
`ba9f8350bc78784052a56ab95680a019719a1792`. Sprint 12 is completed without a
review-time production, public API, Coverage, or deferred-scope change, and
made Sprint 13 XDTO and Service Model the next planning target.

Sprint 13 XDTO and Service Model is governed by the accepted planning contract
in `docs/adr/0035-xdto-service-semantics.md` and the repository-owned evidence
in `docs/architecture/xdto-service-source-investigation.md`. The bounded first
slice now enriches existing XDTO Package, HTTP Service, and Web Service metadata
payloads; adds direct XDTO Type, HTTP URL Template/Method, and Web Service
Operation/Parameter nodes with exact immediate ownership; migrates accepted
package, type, and callable declarations to public reference requests; and emits
precise internal References plus declarative handler Triggers.

The planning corpus contains 20 XDTO descriptor/schema pairs with 12,666
uniquely named direct Value/Object types, two HTTP Services with 35 URL
Templates and 35 Methods, and eight Web Services with 119 Operations and 360
Parameters. All 154 handler declarations resolve uniquely to existing owned
service-module callables: all 35 HTTP and 119 Web handlers are Functions, and
zero are Procedures. EDT XML handler-field names do not define the BSL symbol
kind. The tracked
`adapters/edt/tests/fixtures/sprint13_xdto_services_project/` reduction records
every selected live artifact path plus source and reduced SHA-256 values and
proves small/mixed/large direct-type shapes, HTTP optional methods, Web package
forms, types, directions, and exact Function dispatch without an ignored-file
dependency.

Generic Query, Diff, reports, Validation, Impact policy, complete indexes, and
incremental clean-rebuild transitions cover all five new kinds, immediate
ownership, payload and target changes, internal/external request projections,
References, Triggers, and deferred-property stability. Executable registry
state is Graph Domain 96 capabilities: 92 `Supported` and 4 `NotApplicable`;
EDT 120 capabilities: 115 `Supported` and 5 `NotApplicable`. Both registries
have zero gaps. Sprint 13 remained `active` until the independent Task 6
integration review recorded its decision.

The
[Sprint 13 integration review](../reviews/sprint-13-xdto-service-model.md)
records `pass` against committed recovery head
`5af338cd679a950c3ed262d1b777892186c92e22`. Sprint 13 is completed without a
review-time production, public API, Coverage, or deferred-scope change and makes
Sprint 14 Designer XML Adapter the next planning target.

Later corpus-separated
[Web Service XDTO package evidence](web-service-xdto-packages-source-investigation.md)
proves valid direct declaration cardinalities two and four in Retail, including
four repository packages in `EquipmentService` and a mixed repository/external
pair in `MobileService`. The accepted ADR-0035 amendment keeps the existing
zero-or-more source-independent payload, exact request categories, endpoint
matrices, global namespace/type resolution, Function handler targets, and
Coverage state. The corrective gate is now complete: the EDT parser and
production builder consume the canonical declaration collection and emit one
package request per unique repository declaration. The tracked Retail
reduction covers multiple and mixed declarations plus global namespace/type
resolution, and Coverage remains unchanged. At the committed correction
baseline, Sprint 13 stayed historically completed and Sprint 14 became the
unique eligible next sprint; the v0.3 release integration review remained
gated on Sprint 14 completion.

The first slice deliberately excludes 61,435 nested XDTO properties, imports,
restrictions, inline types, external platform schema nodes, route matching,
transport/publication/runtime behavior, and Designer XML. External XDTO
namespace declarations remain typed source content without placeholder nodes or
false local-resolution failures. Sprint 14 continues to own Designer XML and
cross-adapter identity equivalence.

Sprint 14 Designer XML Adapter is governed by
[ADR-0036](../adr/0036-designer-xml-adapter.md) and the repository-owned
[source investigation](designer-xml-source-investigation.md). The accepted
first slice adds no graph kind, edge kind, public semantic identity, or EDT
behavior. A dedicated adapter detects hierarchical Designer XML version 2.20,
loads an explicit complete or partial scope, maps the 20 top-level families
with direct paired Designer evidence, reads the existing generic Object,
Manager, and Common module roles, and contributes existing configuration,
metadata, Module, Procedure, Function, and immediate ownership/declaration
semantics. Designer Calculation Registers remain deferred because the paired
corpus contains no direct artifact proving their root and path shape.

Canonical equality is deliberately narrower than the current EDT builder. It
compares stable UUID/owner-role identities, kinds, exact names, accepted common
payload, ownership, BSL declarations, terminal scope outcome, and generic
consumer/index results. Source paths, producer identifiers, XML vocabulary,
serialization, BOM/line endings, raw provenance, and deferred artifacts remain
adapter-specific. The Task 7 evidence exercises the public EDT and Designer
production builders over an official-tool Designer reduction and a
provenance-backed EDT reduction. Their non-empty canonical partial projections
contain one configuration, one Common Module, its Common module role, one
Procedure, and three immediate `Contains` facts; exact identities, names,
common payload, ownership, declarations, terminal success, Query, Diff, report,
Validation, complete-index, and incremental clean-rebuild results agree. A
controlled Designer synonym change produces exactly one `SemanticContent` node
change.

The dedicated Designer registry contains 58 deterministic capabilities: 55
`supported`, one `unsupported` Calculation Register capability, and two
`not_applicable` capabilities for nested Form and Unknown placeholder facts.
Its only gap is the evidence-backed Calculation Register deferral. This is
adapter-specific Coverage; Graph Domain and EDT Coverage remain unchanged.
The [Sprint 14 integration review](../reviews/sprint-14-designer-xml-adapter.md)
records `pass` against committed Task 7 head
`19d56818a1345b4cced43db7275165ff24ce0748`. Sprint 14 is completed without a
review-time production, public API, Coverage, or deferred-scope change. The
[v0.3 release integration review](../reviews/v0.3-release-review.md) records
`pass` against committed Sprint 14 review head
`8dbb09a2c085c990308fa57621b510150be6c9a2`. The v0.3 boundary is complete. At
that committed release boundary Sprint 15 Runtime Service Container was the
unique next planning target; current Runtime delivery state is governed by
ADR-0037 through ADR-0039 and the live roadmap.

Metadata members, specialized Role/Subsystem/Event/DCS/XDTO/service semantics,
Form/Command/configuration/register module roles, semantic references and other
non-ownership relations, flat dumps, extensions, parent configurations, binary
artifacts, and whole-graph equivalence remain deferred. The documented
EDT-to-Designer form-event loss boundary is outside the first-slice oracle.

### Provenance inventory

EDT attaches provenance while creating metadata object nodes, child nodes,
module nodes, symbol nodes, ownership edges, resolved reference edges, and
resolution diagnostics. Accepted metadata type references are converted at
collection time into public graph-domain `SemanticReferenceRequest` values with
collection provenance. The deterministic request ledger preserves that evidence
through resolution, terminal projection, reporting, validation, and diffing for
the accepted metadata-reference first slice. Other reference-request families
remain private until their source and lifecycle contracts are defined.

### Known limitations and ordered completion backlog

The audit assigns priority by explicit policy. Missing provenance on an emitted
fact and silently ignored references are critical. Missing core entities,
ownership relations, references, or emitted edges are high priority. Partial
variant support and missing representative tests are medium priority.

The Critical BSL call observability gap remains closed. The High
`metadata_entity.command` discovery and emission gap is also closed: EDT
`CommonCommands` are parsed through the universal top-level descriptor path,
emitted with stable UUID identity and provenance, and owned by the configuration.
The capability is `Supported` with complete typed metadata payload evidence. The former
`metadata_entity.form` and `semantic_node.metadata.form` High gaps were stale:
the generic top-level Form concept is not applicable to EDT, whose actual common
and subordinate form representations already use distinct semantic kinds.
The former `metadata_entity.unknown` High gap was a stale applicability
classification. `MetadataKind::Unknown` is fallback-only for this adapter, so
the capability is now `NotApplicable` without adding a producer or emitting
unknown metadata entities. The former `semantic_node.metadata.unknown` High gap
was the corresponding node-layer applicability gap: EDT has no production
metadata entity, parsing path, graph emission contract, representative fixture,
or error-recovery requirement for `NodeKind::Metadata(MetadataKind::Unknown)`,
so it is also `NotApplicable`. The former flat `semantic_node.unknown` High gap
was the corresponding graph-node sentinel classification gap: the graph domain
can model `NodeKind::Unknown`, but EDT has no legitimate source artifact,
identity semantics, provenance semantics, or error-recovery rule requiring such
a node, so the EDT capability is now `NotApplicable` without changing graph
construction.

The former `ownership_relation.attribute` gap is closed. A repository-owned
real-format EDT fixture proves top-level attribute ownership, immediate
tabular-section ownership for nested attributes, the absence of a companion
metadata-object ownership edge, generic owner and children queries, nested type
reference resolution, deterministic node and edge provenance, graph validation,
and repeated-build stability. Positive, incompatible-owner, and multiple-owner
validator tests cover the canonical ownership invariant.

The former `semantic_edge.writes` High gap is closed. The accepted first slice
emits exact provenance-backed edges from Document Object Module Procedures to
declared and resolved Accumulation Registers, enforces the precise endpoint
matrix, preserves typed failure outcomes, and has deterministic production,
Query, Impact, negative, duplicate, and repeated-build evidence. Its registry
capability is `Supported` with complete evidence.

The completed thematic Semantic Coverage work is:

1. **Medium — metadata payload — completed.** The ADR-0023 domain,
   graph integration, EDT conversion, and complete per-kind production evidence
   are implemented. All applicable EDT metadata-entity capabilities have
   complete registry evidence and `Supported` status.
2. **Medium — metadata reference fixtures — completed.** Successful
   production-builder evidence now covers all nine mapped target kinds, and the
   seven formerly partial capabilities have complete deterministic evidence.
3. **Medium — reference-request provenance — completed.** The public
   graph-domain request ledger, deterministic lifecycle, EDT metadata-reference
   migration, collection-time provenance, production evidence, and independent
   graph-domain and EDT Coverage transitions are complete for the accepted
   first slice.
4. **Medium — endpoint validation — completed.** The `References` rule accepts
   exactly the two ADR-0025 production matrices, exhaustive deterministic
   positive and negative validator evidence is present, production regressions
   pass, and the schema comment describes explicit per-edge policies. Coverage
   status and counts are unchanged.

The former `semantic_node.measure` High gap is closed. A representative EDT
Accounting Register fixture now proves production parsing, semantic kind
selection, stable UUID identity, source provenance, and repeated-build
determinism. The node capability is `Supported`.

The former `ownership_relation.measure` High gap is closed. The existing EDT
metadata-child production path emits exactly one `Contains` edge from the owning
Accounting Register metadata node to each emitted `Measure` node, attaches
edge-level provenance, satisfies graph validation, and remains deterministic
across repeated builds.

The former `semantic_node.query` High gap is closed. The EDT BSL pipeline now
extracts static Query declarations with stable local bindings inside known
procedures or functions, emits `NodeKind::Query` nodes with `Contains`
ownership, preserves source provenance with owner and binding context, and
verifies repeated-build determinism. The accepted first query-language parsing
and `Reads` slice is also complete. Sprint 8 adds public QuerySource requests
and normalized Query-origin `DependsOn` for the four accepted direct persistent
source families. Broader grammar and additional source families remain
deferred.

The former `semantic_node.role` High gap is closed. The EDT pipeline now emits
flat `NodeKind::Role` nodes for every discovered role metadata object while
preserving the existing `NodeKind::Metadata(MetadataKind::Role)` object node.
Repeated builds preserve role node identity, provenance, and graph/build-result
diff stability. The accepted role access-right slice is also complete: EDT role
declarations resolve to scoped `NodeKind::AccessRight` nodes and canonical
`Grants` edges. ADR-0031 opaque conditional direct-grant preservation is
implemented with typed payload, deterministic identity, production emission,
generic-consumer evidence, and unchanged Coverage aggregates. Sprint 9 is
completed with a `pass` integration-review decision. Deny semantics, condition
evaluation, inheritance, defaults, profiles, groups, users, and effective
authorization remain deferred.

The former `semantic_node.standard_attribute` High gap is closed. The EDT
pipeline now derives document standard attributes from real document metadata
descriptors, emits `NodeKind::StandardAttribute` nodes through the existing
graph-domain model, preserves ordinary `Attribute` nodes, attaches member-level
provenance, and verifies repeated-build determinism. Catalog, owner-dependent,
and hierarchy-dependent standard attributes remain future scoped extensions.

The former `ownership_relation.standard_attribute` High gap is closed. The
existing EDT standard-attribute production path emits exactly one `Contains`
edge from the owning Document metadata node to each emitted `StandardAttribute`
node, keeps edge identity stable across repeated builds, attaches deterministic
member-level provenance, and satisfies the graph ownership validator.

The former `semantic_node.subsystem` High gap is closed. The EDT pipeline now
emits flat `NodeKind::Subsystem` nodes for every discovered subsystem metadata
object while preserving the existing `NodeKind::Metadata(MetadataKind::Subsystem)`
object node. Recursive production discovery now preserves validated direct
Subsystem hierarchy through flat-node Includes edges, configuration ownership
for nested metadata objects, and nested direct content. Query derives stable
transitive metadata membership without persisted closure. Repeated builds and
complete or incremental index transitions preserve identity, provenance, and
graph/build-result equivalence.

The EDT registry now reports 101 capabilities: 96 `Supported` and 5
`NotApplicable`, with 0 Critical gaps, 0 High gaps, and 0 Medium gaps. The
graph-domain registry reports 85 capabilities: 82 `Supported` and 3
`NotApplicable`, also with no Critical, High, or Medium gaps. All 21 applicable
metadata-entity capabilities, the nine completed metadata-reference
capabilities, and the bounded EDT `semantic_edge.opens` producer are
`Supported`; Form and Unknown metadata entities remain `NotApplicable`.
Sprint 3 Semantic Coverage Integration Review is complete with no blocking
findings.

The former High `metadata_entity.template` gap is closed. EDT now discovers
Common Template descriptors through the generic top-level path, emits stable
`NodeKind::Metadata(MetadataKind::Template)` nodes and configuration ownership
edges with provenance, and verifies deterministic Query API results. The
capability is `Supported` with complete typed payload preservation evidence.

Completion does not broaden the accepted first-slice contracts. Deferred work
remains: broader query-language grammar and source forms beyond the four direct
persistent namespaces;
deny, inheritance, and effective authorization; semantic meaning for
`Subsystem.<...>` content, command-interface behavior, cross-project hierarchy,
and contradictory-source recovery; and reference-request migration for BSL
calls, Writes targets, protected resources, Subsystem content, and extension
targets. Query sources have completed public request migration for the accepted
direct-source boundary. Graph-query transport now exposes only the bounded
ADR-0040 projections; broader serialization, rebuild and persistence, supported
CLI behavior, and quality percentages remain outside the implemented boundary.

## Definition of done for Semantic Model 2.0 core

The Knowledge Graph core is complete when:

* node and edge identifiers are deterministic;
* node and edge taxonomies are typed;
* nodes and edges retain provenance;
* graph storage provides incoming and outgoing indexes;
* graph construction occurs through a builder;
* invalid edges are rejected by schema validation;
* graph snapshots are deterministic;
* unresolved targets can be represented;
* unit tests cover graph invariants;
* existing graph functionality remains operational during migration.
