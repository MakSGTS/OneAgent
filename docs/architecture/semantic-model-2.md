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
This adapter-specific classification does not change the separate graph-domain
or EDT capability for the flat `semantic_node.unknown` node kind.
Commands embedded in another metadata descriptor similarly emit the flat
`NodeKind::Command` variant.

### Semantic node inventory

The EDT pipeline currently emits:

* top-level `NodeKind::Metadata(kind)` nodes for the directory registry above;
* `Module`, `Procedure`, and `Function` nodes from known BSL module files and
  declaration extraction;
* `Attribute`, `TabularSection`, `Form`, `Command`, `Dimension`, `Resource`, and
  `Measure` child nodes from metadata descriptors.

EDT represents an accounting-register measure as an
`AccountingRegisterResource` in the `<resources>` collection. The generic
structure reader preserves that source vocabulary, while EDT graph conversion
maps a resource owned by `MetadataKind::AccountingRegister` to
`NodeKind::Measure`. Resources of accumulation and other register kinds remain
`NodeKind::Resource`. Measure identity uses the source UUID when available and
its provenance identifies the original accounting-register descriptor and
resource member.

`StandardAttribute` has a graph-domain model and insertion tests, but the EDT
structure reader does not extract or emit it. `Query`, the flat `Role` and
`Subsystem` variants, and `Unknown` are also not emitted by the EDT pipeline.
EDT roles and subsystems use `NodeKind::Metadata(MetadataKind::Role)` and
`NodeKind::Metadata(MetadataKind::Subsystem)` instead.

### Ownership inventory

`Contains` is stored from owner to child. EDT emits configuration-to-object,
metadata-object-to-module, module-to-procedure/function, and
metadata-object-to-child relations with provenance. Validation constrains
containment endpoints and single-owner rules; Query exposes owners and children;
Impact Analysis can opt into child-to-owner and owner-to-child propagation.

Attribute ownership is only partially supported. The XML reader recognizes
nested attribute elements, but currently assigns the top-level metadata object
as parent. Attributes nested in a tabular section therefore do not preserve the
tabular-section owner. Standard Attribute ownership is not emitted. Measure node
conversion reuses generic child containment to preserve graph validity, but
`ownership_relation.measure` remains a separate High capability pending its own
ownership-specific evidence and review.

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
so it is also `NotApplicable`. The flat `semantic_node.unknown` capability
remains a separate High gap.

The remaining thematic Semantic Coverage Completion backlog is:

1. **High — Query EDT contribution applicability.** Determine whether
   `NodeKind::Query` has a production EDT source in the current semantic graph
   pipeline or should remain unsupported pending query extraction.
2. **High — Standard Attribute EDT contribution.** Extend metadata structure
   extraction and EDT graph contribution for `StandardAttribute`. Acceptance:
   stable node identity, typed kind payload, owner edge, provenance, validation,
   and representative tests.
3. **High — Measure ownership evidence.** Review and test the existing generic
   containment path independently from Measure node emission before changing
   `ownership_relation.measure` status.
4. **High — nested Tabular Section ownership.** Preserve nested parent context
   so tabular-section attributes are owned by the tabular section. Acceptance:
   correct `Contains` direction, owner validation, provenance, and positive and
   invalid-owner tests.
5. **High — declared semantic edges.** Add producer-specific tasks for Reads,
   Writes, Grants, Includes, Extends, and DependsOn rather than a generic edge
   task. Acceptance for each: extraction source, endpoint rule, provenance,
   Query semantics, Impact policy decision, and tests.
6. **Medium — metadata payload completion.** Define and preserve the typed
   payload expected for each supported top-level metadata kind. Acceptance:
   fields parsed by EDT are either represented, explicitly excluded by contract,
   or recorded as a known limitation.
7. **Medium — metadata reference fixtures.** Add successful fixtures for
   Enumeration, Information Register, Accumulation Register, Accounting
   Register, Calculation Register, Business Process, and Task targets.
8. **Medium — reference-request provenance.** Decide whether pending reference
   requests become a public graph-domain type; if accepted, attach provenance at
   extraction time without changing resolution semantics.
9. **Medium — broad endpoint validation.** Replace permissive rules for future
    emitted dependency, access, composition, and extension edges with typed
    endpoint policies and negative tests.

The former `semantic_node.measure` High gap is closed. A representative EDT
Accounting Register fixture now proves production parsing, semantic kind
selection, stable UUID identity, source provenance, and repeated-build
determinism. The node capability is `Supported`; ownership coverage remains a
separate task.

The EDT registry now reports 13 High gaps and retains 44 Medium gaps. Combined
with the graph-domain registry, the current Semantic Coverage audit reports
0 Critical gaps, 13 High gaps, and 45 Medium gaps. Other fallback, flat-node,
ownership, and declared-edge capabilities remain independent typed gaps and are
not reclassified by these focused coverage changes. Sprint 3 Integration Review
remains blocked while High gaps remain.

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
