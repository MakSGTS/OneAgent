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
pipeline does not emit it.

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
identity and provenance. Their status is `PartiallySupported`: the descriptor
reader parses identity, name, synonym, kind, and path, while the graph node keeps
only identity, name, kind, and provenance. Descriptor payload beyond the current
graph contract is not represented as typed semantic payload. Dedicated
representative fixtures currently exist for Configuration, Catalog, Document,
Common Module, Accumulation Register, Common Command, and Common Template; the
other generic directory mappings lack dedicated integration fixtures.

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
the supported BSL slice. Future query-language analysis may then produce
references and data-access facts. `Reads` and `Writes` describe data access
derived from a Query or BSL symbol. `DependsOn` describes semantic dependency.
None of these edges is a prerequisite for creating the Query node, and they
remain separate coverage gaps.

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
node when extraction has a complete static source declaration; syntax
diagnostics belong to future query-language analysis. Parser failures in
extraction itself SHOULD produce typed diagnostics only after a diagnostics
contract is added.

The first implementation slice targets static BSL query declarations inside a
known procedure or function. This source has a real EDT input family, stable
module ownership, existing BSL module discovery, an existing symbol owner model,
and provenance source identifiers. The slice is restricted to declarations with
a stable local binding and complete statically available text, and it does not
emit `Reads`, `Writes`, or `DependsOn`.

The ordered follow-up tasks are:

1. add query-language parsing and diagnostics in a separate task;
2. derive `Reads` and `Writes` from parsed query sources in separate edge
   capability tasks;
3. derive `DependsOn` after data-access and dependency semantics are defined;
4. add metadata-owned Query sources such as data-composition datasets or
   dynamic-list query settings after their EDT parser contracts are defined.

### Ownership inventory

`Contains` is stored from owner to child. EDT emits configuration-to-object,
metadata-object-to-module, module-to-procedure/function, and
metadata-object-to-child relations with provenance. Validation constrains
containment endpoints and single-owner rules; Query exposes owners and children;
Impact Analysis can opt into child-to-owner and owner-to-child propagation.

Attribute ownership is only partially supported. The XML reader recognizes
nested attribute elements, but currently assigns the top-level metadata object
as parent. Attributes nested in a tabular section therefore do not preserve the
tabular-section owner. Measure node conversion reuses generic child containment:
an accounting-register metadata object owns each emitted
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
representative integration fixtures; the other mapped targets lack successful
fixtures.

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

The EDT pipeline emits `Contains`, `References`, and `Calls`, each with
provenance. `Reads`, `Writes`, `Grants`, `Includes`, `Extends`, and `DependsOn`
are declared but not emitted by EDT.

Validation has explicit endpoint rules for `Contains`, `Calls`, and `References`.
The remaining edge kinds are currently accepted by broad schema rules and are
therefore structurally visible but not semantically constrained. Validation also
checks missing endpoints, ownership, forbidden self-loops, ownership cycles,
node and edge provenance, and build/report counter consistency.

The Query API exposes all stored edge kinds. Dependency and usage classification
includes `Calls`, `References`, `Reads`, `Writes`, and `DependsOn`; `Contains` is
handled by ownership navigation. Impact Analysis uses the same dependency
classification and supports optional `Contains` ownership propagation. `Grants`,
`Includes`, and `Extends` are intentionally excluded from the first impact
policy.

`EdgeKind::DependsOn` is governed by
`docs/adr/0017-depends-on-semantics.md`. It is a materialized normalized direct
semantic dependency stored as `dependent --DependsOn--> dependency`. The first
implementation slice is limited to resolved EDT metadata member type
references, represented as dependencies from `Attribute`, `Dimension`, or
`Resource` nodes to `Metadata(...)` nodes. The edge remains declared but not
emitted by EDT until that separate production task is implemented.

### Provenance inventory

EDT attaches provenance while creating metadata object nodes, child nodes,
module nodes, symbol nodes, ownership edges, resolved reference edges, and
resolution diagnostics. Pending metadata references preserve enough source
context to construct provenance, but the pending request itself does not carry a
public graph-domain `Provenance` value. This request-level gap is partial rather
than a missing provenance gap on emitted nodes or edges.

### Known limitations and ordered completion backlog

The audit assigns priority by explicit policy. Missing provenance on an emitted
fact and silently ignored references are critical. Missing core entities,
ownership relations, references, or emitted edges are high priority. Partial
variant support and missing representative tests are medium priority.

The Critical BSL call observability gap remains closed. The High
`metadata_entity.command` discovery and emission gap is also closed: EDT
`CommonCommands` are parsed through the universal top-level descriptor path,
emitted with stable UUID identity and provenance, and owned by the configuration.
The capability remains `PartiallySupported` only because complete typed metadata
payload preservation is a separate Medium gap. The former
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

The remaining thematic Semantic Coverage Completion backlog is:

1. **High — nested Tabular Section ownership.** Preserve nested parent context
   so tabular-section attributes are owned by the tabular section. Acceptance:
   correct `Contains` direction, owner validation, provenance, and positive and
   invalid-owner tests.
2. **High — declared semantic edges.** Add producer-specific tasks for Reads,
   Writes, Grants, Includes, Extends, and DependsOn rather than a generic edge
   task. `DependsOn` now has an accepted architecture contract in
   `docs/adr/0017-depends-on-semantics.md`; its first production slice remains
   pending. Acceptance for each: extraction source, endpoint rule, provenance,
   Query semantics, Impact policy decision, and tests.
3. **Medium — metadata payload completion.** Define and preserve the typed
   payload expected for each supported top-level metadata kind. Acceptance:
   fields parsed by EDT are either represented, explicitly excluded by contract,
   or recorded as a known limitation.
4. **Medium — metadata reference fixtures.** Add successful fixtures for
   Enumeration, Information Register, Accumulation Register, Accounting
   Register, Calculation Register, Business Process, and Task targets.
5. **Medium — reference-request provenance.** Decide whether pending reference
   requests become a public graph-domain type; if accepted, attach provenance at
   extraction time without changing resolution semantics.
6. **Medium — broad endpoint validation.** Replace permissive rules for future
    emitted dependency, access, composition, and extension edges with typed
    endpoint policies and negative tests.

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
verifies repeated-build determinism. Query-language parsing and data-access
edges remain separate tasks.

The former `semantic_node.role` High gap is closed. The EDT pipeline now emits
flat `NodeKind::Role` nodes for every discovered role metadata object while
preserving the existing `NodeKind::Metadata(MetadataKind::Role)` object node.
Repeated builds preserve role node identity, provenance, and graph/build-result
diff stability. Role access-right modeling and role-derived semantic edges
remain separate future tasks.

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
object node. Repeated builds preserve subsystem node identity, provenance, and
graph/build-result diff stability. Subsystem hierarchy and membership remain
separate future capabilities.

The EDT registry now reports 6 High gaps and retains 44 Medium gaps. Combined
with the graph-domain registry, the current Semantic Coverage audit reports
0 Critical gaps, 6 High gaps, and 45 Medium gaps. Other ownership and
declared-edge capabilities remain independent typed gaps and are not reclassified
by these focused coverage changes. Sprint 3 Integration Review remains blocked
while High gaps remain.

The former High `metadata_entity.template` gap is closed. EDT now discovers
Common Template descriptors through the generic top-level path, emits stable
`NodeKind::Metadata(MetadataKind::Template)` nodes and configuration ownership
edges with provenance, and verifies deterministic Query API results. The
capability remains `PartiallySupported` only because complete typed template
payload preservation belongs to the shared Medium payload-completion item.

The audit intentionally does not implement these backlog items, add serialization
or a CLI, scan source code at runtime, introduce quality percentages, or change
Semantic Resolution, Validation, Query, Impact Analysis, graph identity, or EDT
graph construction.

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
