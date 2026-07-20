# ADR 0008: Semantic Model 2.0 Knowledge Graph

## Status

Accepted

## Context

OneAgent is evolving from a set of specialized loaders and semantic graphs into a platform for intelligent analysis of 1C:Enterprise configurations.

The current implementation already provides several foundational capabilities:

* EDT workspace loading;
* metadata loading;
* module loading;
* BSL parsing;
* declaration graph construction;
* call graph construction;
* deterministic metadata identifiers;
* cross-module call resolution.

These capabilities currently represent semantic information through several domain-specific structures. Metadata objects, BSL declarations, modules, and calls are modeled independently and are connected only where a specific feature requires it.

This approach is sufficient for isolated analysis tasks, but it does not provide a unified semantic representation of the configuration.

Future OneAgent capabilities require a common model that can answer questions such as:

* Which modules belong to a metadata object?
* Which procedures read from or write to a register?
* Which forms, commands, and event handlers lead to a specific procedure?
* Which metadata objects are affected by a code change?
* Which source fragments are relevant to a user request?
* Why was a specific source fragment selected for an AI context?
* Which references are unresolved or ambiguous?
* Which facts came directly from source files and which were derived by analysis?

To support these scenarios, OneAgent requires a unified Knowledge Graph that represents source files, metadata objects, modules, symbols, types, queries, UI elements, and derived semantic concepts.

The Knowledge Graph must become the semantic foundation for:

* navigation;
* reference search;
* dependency analysis;
* impact analysis;
* architectural diagnostics;
* MCP tools;
* IDE integrations;
* AI Context Engine.

## Decision

OneAgent will introduce Semantic Model 2.0 as a unified, deterministic, embedded Knowledge Graph.

The Knowledge Graph will be implemented in the `oneagent-graph` crate and will not depend on EDT, the filesystem, or a specific source format.

Loaders, parsers, resolvers, and analyzers will contribute facts to the graph through controlled graph-building APIs.

The existing declaration and call graph implementations will remain available during migration. They will be incrementally replaced or exposed as projections of the unified Knowledge Graph.

## Graph identity

Every semantic node and edge must have a stable identifier.

Identifiers must not depend on:

* insertion order;
* collection indexes;
* traversal order;
* process-specific memory addresses;
* nondeterministic hash iteration.

Node identity will be derived from the strongest available semantic identity source.

The preferred identity sources are:

1. metadata UUID;
2. semantic path;
3. qualified name;
4. declaration signature;
5. source location as a fallback.

Examples of semantic identities include:

```text
oneagent://configuration/<configuration-id>
oneagent://metadata/catalog/Products
oneagent://metadata/catalog/Products/attribute/Code
oneagent://module/Catalog.Products.ObjectModule
oneagent://symbol/Catalog.Products.ObjectModule/BeforeWrite
```

The concrete serialization format may evolve, but identity construction must remain centralized and deterministic.

A source file is not considered identical to the semantic entity declared in that file.

For example:

```text
Catalogs/Products/Products.mdo
    defines
Catalog.Products
```

and:

```text
Catalogs/Products/ObjectModule.bsl
    defines
Catalog.Products.ObjectModule
```

The source file, metadata object, module, and procedures declared inside the module are separate graph nodes.

## Node taxonomy

Graph nodes will be classified through a structured `NodeKind` taxonomy.

The top-level taxonomy will include the following categories:

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

The taxonomy is intentionally hierarchical.

A single flat enumeration containing every possible 1C entity would become difficult to maintain and would make unrelated changes affect the central graph model.

### Workspace nodes

Workspace nodes represent project-level structure.

Examples:

* workspace;
* project;
* configuration;
* extension;
* library.

### Source nodes

Source nodes represent physical source artifacts.

Examples:

* directory;
* file;
* metadata file;
* module file;
* form file;
* template file;
* query file.

Source nodes are required for:

* provenance;
* incremental indexing;
* invalidation;
* diagnostics;
* navigation to source;
* source fragment extraction.

### Metadata nodes

Metadata nodes represent 1C metadata entities.

Examples:

* catalog;
* document;
* enumeration;
* constant;
* report;
* data processor;
* information register;
* accumulation register;
* accounting register;
* calculation register;
* common module;
* common form;
* common command;
* common attribute;
* role;
* subsystem;
* HTTP service;
* web service;
* XDTO package.

Metadata members are also represented as nodes.

Examples:

* attribute;
* standard attribute;
* tabular section;
* dimension;
* resource;
* measure;
* form;
* command;
* template.

Attributes, dimensions, resources, and other metadata members must not be stored only as unstructured properties of their owner.

Representing them as nodes enables:

* precise references;
* field-level dependency analysis;
* source navigation;
* impact analysis;
* type resolution.

### Module nodes

Modules are separate semantic nodes even when they belong to metadata objects.

Examples:

* common module;
* object module;
* manager module;
* record set module;
* form module;
* command module;
* session module;
* external connection module;
* managed application module;
* ordinary application module.

Example relation:

```text
Catalog.Products
    HasModule
Catalog.Products.ObjectModule
```

### Symbol nodes

Symbol nodes represent BSL declarations.

Examples:

* procedure;
* function;
* parameter;
* local variable;
* module variable;
* property;
* region;
* label.

### Type nodes

Type nodes represent reusable semantic types.

Examples:

* primitive type;
* platform type;
* metadata reference type;
* metadata object type;
* metadata manager type;
* collection type;
* union type;
* unknown type.

Examples of type identities include:

```text
CatalogRef.Products
CatalogObject.Products
CatalogManager.Products
DocumentRef.SalesInvoice
ValueTable
String
Number
```

### Data nodes

Data nodes represent data-access and query semantics.

Examples:

* query;
* query source;
* query field;
* query parameter;
* temporary table;
* data composition schema;
* data set;
* data composition field.

### UI nodes

UI nodes represent internal form and command structures.

Examples:

* form;
* form attribute;
* form command;
* form element;
* form event.

### Derived nodes

Derived nodes represent concepts produced by semantic analysis rather than directly declared in source files.

Examples:

* component;
* dependency cluster;
* entry point;
* use case;
* context bundle;
* diagnostic.

### External nodes

External nodes represent referenced entities that are outside the indexed workspace or are not yet resolved.

Examples:

* external library symbol;
* external service;
* unknown platform member;
* unresolved semantic target.

## Edge taxonomy

Graph edges will be classified through a structured `EdgeKind` taxonomy.

The top-level taxonomy will include:

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

### Structural edges

Structural edges represent ownership and composition.

Examples:

* `Contains`;
* `Defines`;
* `DeclaredIn`;
* `HasModule`;
* `HasForm`;
* `HasCommand`;
* `HasTemplate`;
* `HasAttribute`;
* `HasStandardAttribute`;
* `HasTabularSection`;
* `HasDimension`;
* `HasResource`;
* `HasMeasure`.

Only one canonical direction should be stored for each relation.

Reverse navigation must be provided by graph indexes rather than by duplicating inverse edges.

### Declaration edges

Declaration edges represent symbol declarations and declaration semantics.

Examples:

* `Declares`;
* `HasParameter`;
* `Overrides`;
* `Implements`;
* `Exports`.

### Reference edges

Reference edges represent semantic use of another entity.

Examples:

* `References`;
* `Uses`;
* `Reads`;
* `Writes`;
* `Instantiates`;
* `ResolvesTo`;
* `MayResolveTo`.

### Execution edges

Execution edges represent control-flow relationships.

Examples:

* `Calls`;
* `MayCall`;
* `HandlesEvent`;
* `Triggers`;
* `ExecutesQuery`;
* `InvokesCommand`.

`Calls` must only be used when the target is resolved with sufficient confidence.

Ambiguous or partial call resolution must be represented explicitly through `MayCall`, `MayResolveTo`, or an unresolved reference.

### Data edges

Data edges represent data access and query semantics.

Examples:

* `ReadsFrom`;
* `WritesTo`;
* `SelectsField`;
* `FiltersBy`;
* `GroupsBy`;
* `OrdersBy`;
* `JoinsWith`;
* `Produces`;
* `Consumes`.

### Type edges

Type edges represent semantic type relationships.

Examples:

* `HasType`;
* `ReturnsType`;
* `AcceptsType`;
* `ElementType`;
* `RefersToMetadataType`.

### UI edges

UI edges represent UI bindings and UI-triggered execution.

Examples:

* `Displays`;
* `BindsTo`;
* `Executes`;
* `Handles`;
* `Opens`.

### Derived edges

Derived edges represent facts inferred by analysis.

Examples:

* `DependsOn`;
* `Affects`;
* `RelatedTo`;
* `Includes`;
* `Reaches`;
* `CandidateForContext`.

## Provenance

Every graph fact must retain provenance.

Provenance must allow OneAgent to explain:

* which source artifact produced the fact;
* which source span contains the declaration or reference;
* which component produced the fact;
* whether the fact was parsed, resolved, derived, or imported;
* how confident the system is in the fact.

The conceptual provenance model is:

```rust
pub struct Provenance {
    pub source: SourceId,
    pub span: Option<SourceSpan>,
    pub producer: ProducerId,
    pub confidence: Confidence,
}
```

The concrete Rust representation may differ, but the same information must remain expressible.

Graph facts will distinguish their origin:

```rust
pub enum FactOrigin {
    Declared,
    Parsed,
    Resolved,
    Derived,
    External,
}
```

Direct and derived facts must not be indistinguishable.

For example:

```text
Procedure A Calls Procedure B
```

may be a resolved fact derived directly from a BSL call expression.

By contrast:

```text
Subsystem Sales DependsOn Subsystem Warehouse
```

is a derived architectural fact produced by analysis.

## Unresolved references

Unresolved references must not be discarded.

If a parser detects a reference but resolution cannot identify a target, the graph must retain:

* source node;
* source span;
* textual reference;
* reference category;
* current resolution state;
* candidate targets when available.

Resolution state will support at least:

```rust
pub enum ResolutionState {
    NotApplicable,
    Unresolved,
    Partial,
    Ambiguous,
    Resolved,
}
```

This allows OneAgent to support:

* incomplete workspaces;
* partially loaded configurations;
* dynamic BSL code;
* ambiguous exported methods;
* future incremental resolution.

## Graph construction pipeline

Knowledge Graph construction will use explicit passes.

The initial pipeline is:

```text
Pass 1: Source discovery

Pass 2: Entity declaration

Pass 3: Structural linking

Pass 4: Reference collection

Pass 5: Reference resolution

Pass 6: Derived semantic analysis
```

### Pass 1: Source discovery

Creates source-level nodes for:

* workspaces;
* projects;
* directories;
* files;
* supported source artifacts.

### Pass 2: Entity declaration

Creates nodes for:

* metadata objects;
* metadata members;
* modules;
* BSL procedures;
* BSL functions;
* parameters;
* variables;
* other declarations.

All declarations must be registered before inter-module reference resolution begins.

### Pass 3: Structural linking

Creates structural relations such as:

* configuration contains metadata object;
* metadata object has module;
* metadata object has attribute;
* source file defines module;
* module declares procedure.

### Pass 4: Reference collection

Collects unresolved semantic references such as:

* procedure calls;
* symbol references;
* metadata references;
* query table references;
* type references.

### Pass 5: Reference resolution

Resolves collected references using:

* lexical scope;
* module scope;
* exported common module methods;
* metadata namespaces;
* known platform types;
* known metadata types.

### Pass 6: Derived semantic analysis

Produces derived semantic facts such as:

* dependency edges;
* entry points;
* reachability;
* impact relationships;
* subsystem coupling;
* context candidates.

## Graph storage

The initial Knowledge Graph implementation will be embedded in the OneAgent runtime.

An external graph database will not be required.

The graph must provide deterministic iteration and deterministic serialization.

The initial storage design should use ordered collections where stable output is required.

Conceptually:

```rust
pub struct KnowledgeGraph {
    nodes: BTreeMap<NodeId, Node>,
    edges: BTreeMap<EdgeId, Edge>,
    outgoing: BTreeMap<NodeId, Vec<EdgeId>>,
    incoming: BTreeMap<NodeId, Vec<EdgeId>>,
}
```

Additional indexes may include:

* node kind;
* qualified name;
* source file;
* metadata UUID;
* symbol name;
* edge kind.

The internal storage implementation may be optimized later without changing the public semantic model.

## Graph builder

Loaders and analyzers must not directly mutate graph storage.

Graph construction will be performed through a `GraphBuilder`.

The builder will be responsible for:

* deterministic node insertion;
* node upsert behavior;
* edge creation;
* duplicate detection;
* provenance merging;
* diagnostic collection;
* schema validation;
* final invariant validation.

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

    pub fn finish(
        self,
    ) -> Result<KnowledgeGraph, GraphBuildError>;
}
```

## Graph contributors

Semantic producers will contribute facts through controlled interfaces.

Conceptually:

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

Potential contributors include:

* source contributor;
* metadata contributor;
* BSL declaration contributor;
* BSL reference contributor;
* call resolver;
* type resolver;
* query contributor;
* UI contributor;
* derived dependency analyzer.

## Graph schema validation

The graph will validate whether an edge kind is permitted between two node kinds.

Conceptually:

```rust
pub trait GraphSchema {
    fn allows(
        &self,
        source: &NodeKind,
        edge: &EdgeKind,
        target: &NodeKind,
    ) -> bool;
}
```

Examples of valid relations:

```text
MetadataObject
    HasAttribute
Attribute
```

```text
Module
    Declares
Procedure
```

```text
Procedure
    Calls
Procedure
```

```text
SourceFile
    Defines
Module
```

```text
Symbol
    HasType
Type
```

Schema validation must prevent semantically invalid graph states while allowing unresolved and external targets to be represented explicitly.

## Determinism

Knowledge Graph construction must be deterministic.

Given the same workspace contents and configuration, OneAgent must produce:

* identical node identifiers;
* identical edge identifiers;
* identical ordering in serialized snapshots;
* identical diagnostics;
* identical AI context candidate ordering before token-budget truncation.

Determinism is required for:

* reproducible tests;
* stable snapshots;
* meaningful graph diffs;
* cache keys;
* incremental indexing;
* explainable AI context generation.

## Query model

The Knowledge Graph will expose a query API independent of its internal storage.

The initial query capabilities will include:

* lookup by node identifier;
* lookup by qualified name;
* incoming relations;
* outgoing relations;
* filtered neighbors;
* callers;
* callees;
* declarations;
* references;
* source definitions;
* bounded traversal;
* shortest path where applicable.

Future semantic queries will support:

* seed node selection;
* traversal direction;
* allowed edge kinds;
* maximum depth;
* node budget;
* confidence filters;
* source filters;
* context scoring.

## AI Context Engine

The AI Context Engine will be built on top of the Knowledge Graph.

It will not operate as a simple file search mechanism.

The context-building pipeline will include:

```text
User request
    ↓
Intent extraction
    ↓
Semantic seed resolution
    ↓
Knowledge Graph traversal
    ↓
Candidate scoring
    ↓
Token-budget-aware selection
    ↓
Source fragment extraction
    ↓
Context rendering
```

Context candidate scoring may consider:

* semantic graph distance;
* edge importance;
* node kind;
* source proximity;
* resolution confidence;
* declaration relevance;
* execution reachability;
* data-access relevance;
* token cost.

The engine must be able to explain why each context fragment was selected.

## Dependency rules

The Knowledge Graph core must remain independent from source-specific adapters.

The intended dependency direction is:

```text
oneagent-common
    ↑
    ├── oneagent-metadata
    ├── oneagent-bsl
    └── oneagent-graph
```

Source adapters depend on domain crates:

```text
oneagent-common
oneagent-metadata
oneagent-bsl
oneagent-graph
        ↑
    oneagent-edt
        ↑
    oneagent-runtime
```

The following dependencies are prohibited:

* `oneagent-graph` depending on EDT;
* `oneagent-graph` reading the filesystem;
* `oneagent-graph` parsing BSL;
* `oneagent-graph` depending on runtime orchestration;
* metadata domain types depending on EDT serialization details.

## Migration strategy

Semantic Model 2.0 will be introduced incrementally.

The current declaration graph and call graph will not be removed immediately.

Migration stages:

1. define Semantic Model 2.0 architecture;
2. implement graph identity and taxonomy;
3. implement node, edge, and provenance models;
4. implement deterministic graph storage;
5. implement graph builder and schema validation;
6. migrate metadata entities;
7. migrate BSL declarations;
8. migrate call relations;
9. add reference and type resolution;
10. expose legacy graph views as Knowledge Graph projections;
11. remove obsolete graph implementations after all consumers are migrated.

Existing public APIs should remain stable where practical during the transition.

## Consequences

### Positive

* One unified semantic model for metadata, BSL, queries, UI, and derived analysis.
* Stable foundation for navigation and dependency analysis.
* Explicit unresolved and ambiguous references.
* Explainable provenance for every semantic fact.
* Deterministic graph snapshots and AI contexts.
* Clear separation between source adapters and semantic representation.
* Incremental migration from existing graph implementations.
* Foundation for AI Context Engine and MCP tools.
* Ability to perform precise impact analysis down to metadata members and BSL symbols.

### Negative

* The graph model becomes more complex than the current specialized graphs.
* More memory is required to store source nodes, provenance, and indexes.
* Builders and contributors require additional abstractions.
* Existing graph code must be migrated gradually.
* Schema evolution requires careful compatibility management.
* Full value depends on future reference, type, query, and UI analysis.

### Risks

* An excessively detailed taxonomy may slow development.
* An overly generic property model may weaken type safety.
* Premature optimization may complicate the graph core.
* Unstable identifier rules may break snapshots and caches.
* Mixing parsed and derived facts may reduce explainability.
* Direct mutation by adapters may bypass graph invariants.
* Attempting to implement all 1C metadata kinds at once may delay usable milestones.

These risks will be mitigated by:

* hierarchical node and edge taxonomies;
* typed core properties;
* explicit provenance;
* deterministic identity builders;
* controlled graph construction;
* incremental metadata coverage;
* compatibility layers during migration.

## Roadmap alignment

Semantic Model 2.0 introduces the following phases:

```text
SM-0 Architecture specification
SM-1 Knowledge Graph Core
SM-2 Metadata graph migration
SM-3 BSL semantic graph migration
SM-4 Type and reference resolution
SM-5 Data access graph
SM-6 UI and entry points
SM-7 Derived semantic analysis
SM-8 AI Context Engine
SM-9 MCP and IDE integration
```

The remaining tasks from the current metadata Sprint 2 are included in `SM-2 Metadata graph migration`.

These tasks include:

* form;
* command;
* standard attribute;
* resource;
* dimension;
* measure.

## Decision outcome

Semantic Model 2.0 becomes the target semantic architecture of OneAgent.

All new semantic capabilities must either contribute to the Knowledge Graph or consume it through its public query interfaces.

Specialized semantic structures may continue to exist as temporary migration components or optimized projections, but the Knowledge Graph is the canonical long-term representation of the indexed 1C configuration.
