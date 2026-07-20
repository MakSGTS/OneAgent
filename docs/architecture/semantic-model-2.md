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

### `oneagent-runtime`

Owns orchestration:

* graph construction pipeline;
* contributor execution;
* lifecycle;
* snapshots;
* invalidation;
* incremental rebuilds;
* API exposure.

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

## Immediate implementation boundary

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
