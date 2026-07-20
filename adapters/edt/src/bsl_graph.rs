//! Integration between EDT module files, BSL declarations and the semantic graph.

use oneagent_bsl::{
    BslCall, BslCallError, BslCallExtractor, BslCallResolver, BslDeclarationExtractor,
    BslModuleSymbols, BslParseError, BslSymbol, BslSymbolKind, CrossModuleCallResolver,
    LineBslCallExtractor, LineBslDeclarationExtractor, LocalBslCallResolver,
    QualifiedBslCallResolver,
};
use oneagent_common::{EntityId, EntityName};
use oneagent_graph::{
    Confidence, EdgeKind, FactOrigin, GraphError, NodeKind, ProducerId, Provenance,
    ResolutionState, SemanticGraph,
};
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::Path;

use crate::EdtModuleDescriptor;

const EDT_BSL_GRAPH_PRODUCER: &str = "oneagent.edt.bsl-graph";

/// Parsed declarations and calls collected from one EDT BSL module.
///
/// The analysis result does not mutate the semantic graph. This allows all
/// configuration modules to be analyzed before call relations are resolved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnalyzedBslModule {
    module_id: EntityId,
    module_name: EntityName,
    symbols: Vec<BslSymbol>,
    calls: Vec<BslCall>,
    source: Option<EntityId>,
}

impl AnalyzedBslModule {
    /// Creates an analyzed BSL module.
    #[must_use]
    pub const fn new(
        module_id: EntityId,
        module_name: EntityName,
        symbols: Vec<BslSymbol>,
        calls: Vec<BslCall>,
    ) -> Self {
        Self::new_with_source(module_id, module_name, symbols, calls, None)
    }

    /// Creates an analyzed BSL module with a source identifier.
    #[must_use]
    pub const fn new_with_source(
        module_id: EntityId,
        module_name: EntityName,
        symbols: Vec<BslSymbol>,
        calls: Vec<BslCall>,
        source: Option<EntityId>,
    ) -> Self {
        Self {
            module_id,
            module_name,
            symbols,
            calls,
            source,
        }
    }

    /// Returns the module identifier.
    #[must_use]
    pub const fn module_id(&self) -> &EntityId {
        &self.module_id
    }

    /// Returns the module name.
    #[must_use]
    pub const fn module_name(&self) -> &EntityName {
        &self.module_name
    }

    /// Returns declarations collected from the module.
    #[must_use]
    pub fn symbols(&self) -> &[BslSymbol] {
        &self.symbols
    }

    /// Returns calls collected from the module.
    #[must_use]
    pub fn calls(&self) -> &[BslCall] {
        &self.calls
    }

    /// Returns the source identifier used for graph provenance.
    #[must_use]
    pub const fn source(&self) -> Option<&EntityId> {
        self.source.as_ref()
    }

    fn as_module_symbols(&self) -> BslModuleSymbols {
        BslModuleSymbols::new(
            self.module_id.clone(),
            self.module_name.clone(),
            self.symbols.clone(),
        )
    }
}

/// Reads and analyzes one EDT BSL module without changing the semantic graph.
///
/// # Errors
///
/// Returns an error when the module cannot be read or parsed.
pub fn analyze_module(module: &EdtModuleDescriptor) -> Result<AnalyzedBslModule, EdtBslGraphError> {
    let source =
        fs::read_to_string(module.path()).map_err(|source| EdtBslGraphError::ReadModule {
            path: module.path().to_path_buf(),
            source,
        })?;

    let symbols = LineBslDeclarationExtractor
        .extract(module.id(), &source)
        .map_err(EdtBslGraphError::ParseDeclarations)?;

    let calls = LineBslCallExtractor
        .extract_calls(module.id(), &source)
        .map_err(EdtBslGraphError::ParseCalls)?;

    Ok(AnalyzedBslModule::new_with_source(
        module.id().clone(),
        module.name().clone(),
        symbols,
        calls,
        Some(source_id_from_path(module.path())?),
    ))
}

/// Analyzes all modules before adding declarations and call relations.
///
/// Processing is performed in two passes:
///
/// 1. Insert declarations from every module.
/// 2. Resolve local and qualified cross-module calls.
///
/// Module nodes must already exist in `graph`.
///
/// # Errors
///
/// Returns an error when a module cannot be read or parsed, or when a graph
/// edge references a node absent from the semantic graph.
pub fn add_configuration_module_symbols(
    graph: &mut SemanticGraph,
    modules: &[EdtModuleDescriptor],
) -> Result<usize, EdtBslGraphError> {
    let analyzed_modules = modules
        .iter()
        .map(analyze_module)
        .collect::<Result<Vec<_>, _>>()?;

    add_analyzed_modules(graph, &analyzed_modules)
}

/// Adds previously analyzed modules to the semantic graph in two passes.
///
/// Module nodes must already exist in `graph`.
///
/// # Errors
///
/// Returns an error when an inserted graph edge references a missing node.
fn add_analyzed_modules(
    graph: &mut SemanticGraph,
    modules: &[AnalyzedBslModule],
) -> Result<usize, EdtBslGraphError> {
    // Pass 1: insert declarations from every module.
    for module in modules {
        insert_declarations(graph, module)?;
    }

    let available_modules = modules
        .iter()
        .map(AnalyzedBslModule::as_module_symbols)
        .collect::<Vec<_>>();

    // Pass 2: resolve calls only after every declaration is present.
    for (module, module_symbols) in modules.iter().zip(&available_modules) {
        insert_local_calls(graph, module)?;
        insert_cross_module_calls(graph, module, module_symbols, &available_modules)?;
    }

    Ok(modules.iter().map(|module| module.symbols().len()).sum())
}

/// Adds declarations and local calls from one module.
///
/// This compatibility wrapper preserves the previous public API. Qualified
/// calls cannot be resolved unless modules are supplied together through
/// [`add_configuration_module_symbols`].
///
/// # Errors
///
/// Returns an error when the module cannot be read, parsed, resolved or
/// inserted into the graph.
pub fn add_module_symbols(
    graph: &mut SemanticGraph,
    module: &EdtModuleDescriptor,
) -> Result<usize, EdtBslGraphError> {
    let analyzed_module = analyze_module(module)?;
    add_analyzed_modules(graph, std::slice::from_ref(&analyzed_module))
}

fn insert_declarations(
    graph: &mut SemanticGraph,
    module: &AnalyzedBslModule,
) -> Result<(), EdtBslGraphError> {
    for symbol in module.symbols() {
        let node_kind = match symbol.kind() {
            BslSymbolKind::Procedure => NodeKind::Procedure,
            BslSymbolKind::Function => NodeKind::Function,
        };

        graph.insert_node_with_provenance(
            symbol.id().clone(),
            symbol.name().clone(),
            node_kind,
            declared_provenance(module.source()),
        );

        graph
            .insert_edge_with_provenance(
                module.module_id().clone(),
                symbol.id().clone(),
                EdgeKind::Contains,
                declared_provenance(module.source()),
            )
            .map_err(EdtBslGraphError::Graph)?;
    }

    Ok(())
}

fn insert_local_calls(
    graph: &mut SemanticGraph,
    module: &AnalyzedBslModule,
) -> Result<(), EdtBslGraphError> {
    let resolution = LocalBslCallResolver.resolve(module.symbols(), module.calls());

    for resolved_call in resolution.resolved() {
        insert_call_edge(
            graph,
            resolved_call.origin_id(),
            resolved_call.destination_id(),
            module.source(),
        )?;
    }

    Ok(())
}

fn insert_cross_module_calls(
    graph: &mut SemanticGraph,
    module: &AnalyzedBslModule,
    current_module: &BslModuleSymbols,
    available_modules: &[BslModuleSymbols],
) -> Result<(), EdtBslGraphError> {
    let resolution = QualifiedBslCallResolver.resolve_cross_module_calls(
        current_module,
        available_modules,
        module.calls(),
    );

    for resolved_call in resolution.resolved() {
        insert_call_edge(
            graph,
            resolved_call.origin_id(),
            resolved_call.destination_id(),
            module.source(),
        )?;
    }

    Ok(())
}

fn insert_call_edge(
    graph: &mut SemanticGraph,
    origin_id: &EntityId,
    destination_id: &EntityId,
    source: Option<&EntityId>,
) -> Result<(), EdtBslGraphError> {
    graph
        .insert_edge_with_provenance(
            origin_id.clone(),
            destination_id.clone(),
            EdgeKind::Calls,
            resolved_provenance(source),
        )
        .map(|_| ())
        .map_err(EdtBslGraphError::Graph)
}

fn declared_provenance(source: Option<&EntityId>) -> Provenance {
    bsl_provenance(
        source,
        FactOrigin::Declared,
        Confidence::Exact,
        ResolutionState::NotApplicable,
    )
}

fn resolved_provenance(source: Option<&EntityId>) -> Provenance {
    bsl_provenance(
        source,
        FactOrigin::Resolved,
        Confidence::High,
        ResolutionState::Resolved,
    )
}

fn bsl_provenance(
    source: Option<&EntityId>,
    origin: FactOrigin,
    confidence: Confidence,
    resolution: ResolutionState,
) -> Provenance {
    Provenance::new(
        source.cloned(),
        ProducerId::new(EDT_BSL_GRAPH_PRODUCER),
        origin,
        confidence,
        resolution,
    )
}

fn source_id_from_path(path: &Path) -> Result<EntityId, EdtBslGraphError> {
    EntityId::new(path.to_string_lossy().replace('\\', "/"))
        .map_err(|_| EdtBslGraphError::InvalidSourceIdentifier)
}

/// Error produced while adding BSL declarations to the EDT semantic graph.
#[derive(Debug)]
pub enum EdtBslGraphError {
    /// A module source file could not be read.
    ReadModule {
        /// Module source path.
        path: std::path::PathBuf,
        /// Underlying I/O error.
        source: std::io::Error,
    },

    /// BSL declaration extraction failed.
    ParseDeclarations(BslParseError),

    /// BSL call extraction failed.
    ParseCalls(BslCallError),

    /// Semantic graph validation failed.
    Graph(GraphError),

    /// A module source identifier could not be created.
    InvalidSourceIdentifier,
}

impl Display for EdtBslGraphError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReadModule { path, source } => {
                write!(
                    formatter,
                    "failed to read BSL module {}: {source}",
                    path.display()
                )
            }

            Self::ParseDeclarations(error) => {
                write!(formatter, "failed to parse BSL declarations: {error}")
            }

            Self::ParseCalls(error) => {
                write!(formatter, "failed to parse BSL calls: {error}")
            }

            Self::Graph(error) => {
                write!(formatter, "semantic graph error: {error}")
            }

            Self::InvalidSourceIdentifier => {
                formatter.write_str("EDT BSL source identifier is invalid")
            }
        }
    }
}

impl std::error::Error for EdtBslGraphError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ReadModule { source, .. } => Some(source),
            Self::ParseDeclarations(error) => Some(error),
            Self::ParseCalls(error) => Some(error),
            Self::Graph(error) => Some(error),
            Self::InvalidSourceIdentifier => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use oneagent_common::{EntityId, EntityName};
    use oneagent_graph::{EdgeKind, GraphNode, NodeKind, SemanticGraph};
    use std::fs;
    use tempfile::tempdir;

    use crate::{EdtModuleDescriptor, EdtModuleKind};

    use super::{add_configuration_module_symbols, add_module_symbols};

    #[test]
    fn adds_procedure_and_function_nodes() {
        let root = tempdir().expect("temporary directory must be created");
        let module_path = root.path().join("ObjectModule.bsl");

        fs::write(
            &module_path,
            concat!(
                "Procedure BeforeWrite() Export\n",
                "    Total();\n",
                "EndProcedure\n",
                "\n",
                "Function Total()\n",
                "    Return 100;\n",
                "EndFunction\n",
            ),
        )
        .expect("module file must be created");

        let module_id =
            EntityId::new("document-id:object_module").expect("identifier must be valid");

        let module = EdtModuleDescriptor::new(
            module_id.clone(),
            EntityName::new("ObjectModule").expect("name must be valid"),
            EdtModuleKind::Object,
            module_path,
        );

        let mut graph = SemanticGraph::new();
        graph.insert_node(GraphNode::new(
            module_id,
            EntityName::new("ObjectModule").expect("name must be valid"),
            NodeKind::Module,
        ));

        let count = add_module_symbols(&mut graph, &module).expect("symbols must be added");

        let procedure = graph
            .nodes_by_kind(NodeKind::Procedure)
            .into_iter()
            .next()
            .expect("procedure node must exist");

        let calls = graph.outgoing_by_kind(procedure.id(), EdgeKind::Calls);

        assert_eq!(calls.len(), 1);

        let function = graph
            .nodes_by_kind(NodeKind::Function)
            .into_iter()
            .next()
            .expect("function node must exist");

        assert_eq!(calls[0].target(), function.id());

        assert_eq!(count, 2);
        assert_eq!(graph.nodes_by_kind(NodeKind::Procedure).len(), 1);
        assert_eq!(graph.nodes_by_kind(NodeKind::Function).len(), 1);
    }

    #[test]
    fn resolves_qualified_call_to_module_collected_later() {
        let root = tempdir().expect("temporary directory must be created");

        let caller_path = root.path().join("CallerModule.bsl");
        let service_path = root.path().join("ServiceModule.bsl");

        fs::write(
            &caller_path,
            concat!(
                "Procedure Run()\n",
                "    ServiceModule.Execute();\n",
                "EndProcedure\n",
            ),
        )
        .expect("caller module must be created");

        fs::write(
            &service_path,
            concat!("Procedure Execute() Export\n", "EndProcedure\n",),
        )
        .expect("service module must be created");

        let caller_id =
            EntityId::new("configuration:caller_module").expect("identifier must be valid");
        let service_id =
            EntityId::new("configuration:service_module").expect("identifier must be valid");

        let caller = EdtModuleDescriptor::new(
            caller_id.clone(),
            EntityName::new("CallerModule").expect("name must be valid"),
            EdtModuleKind::Object,
            caller_path,
        );

        let service = EdtModuleDescriptor::new(
            service_id.clone(),
            EntityName::new("ServiceModule").expect("name must be valid"),
            EdtModuleKind::Common,
            service_path,
        );

        let mut graph = SemanticGraph::new();

        graph.insert_node(GraphNode::new(
            caller_id,
            EntityName::new("CallerModule").expect("name must be valid"),
            NodeKind::Module,
        ));

        graph.insert_node(GraphNode::new(
            service_id,
            EntityName::new("ServiceModule").expect("name must be valid"),
            NodeKind::Module,
        ));

        let count = add_configuration_module_symbols(&mut graph, &[caller, service])
            .expect("configuration symbols must be added");

        let run = graph
            .nodes_by_kind(NodeKind::Procedure)
            .into_iter()
            .find(|node| node.name().as_str() == "Run")
            .expect("Run procedure must exist");

        let execute = graph
            .nodes_by_kind(NodeKind::Procedure)
            .into_iter()
            .find(|node| node.name().as_str() == "Execute")
            .expect("Execute procedure must exist");

        let calls = graph.outgoing_by_kind(run.id(), EdgeKind::Calls);

        assert_eq!(count, 2);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].target(), execute.id());
    }

    #[test]
    fn cross_module_resolution_does_not_depend_on_module_order() {
        let root = tempdir().expect("temporary directory must be created");

        let caller_path = root.path().join("CallerModule.bsl");
        let service_path = root.path().join("ServiceModule.bsl");

        fs::write(
            &caller_path,
            concat!(
                "Procedure Run()\n",
                "    ServiceModule.Execute();\n",
                "EndProcedure\n",
            ),
        )
        .expect("caller module must be created");

        fs::write(
            &service_path,
            concat!("Procedure Execute() Export\n", "EndProcedure\n",),
        )
        .expect("service module must be created");

        let build_graph = |reverse: bool| {
            let caller_id =
                EntityId::new("configuration:caller_module").expect("identifier must be valid");
            let service_id =
                EntityId::new("configuration:service_module").expect("identifier must be valid");

            let caller = EdtModuleDescriptor::new(
                caller_id.clone(),
                EntityName::new("CallerModule").expect("name must be valid"),
                EdtModuleKind::Object,
                caller_path.clone(),
            );

            let service = EdtModuleDescriptor::new(
                service_id.clone(),
                EntityName::new("ServiceModule").expect("name must be valid"),
                EdtModuleKind::Common,
                service_path.clone(),
            );

            let mut graph = SemanticGraph::new();

            graph.insert_node(GraphNode::new(
                caller_id,
                EntityName::new("CallerModule").expect("name must be valid"),
                NodeKind::Module,
            ));

            graph.insert_node(GraphNode::new(
                service_id,
                EntityName::new("ServiceModule").expect("name must be valid"),
                NodeKind::Module,
            ));

            let modules = if reverse {
                vec![service, caller]
            } else {
                vec![caller, service]
            };

            add_configuration_module_symbols(&mut graph, &modules)
                .expect("configuration symbols must be added");

            graph
        };

        let normal = build_graph(false);
        let reversed = build_graph(true);

        let normal_run = normal
            .nodes_by_kind(NodeKind::Procedure)
            .into_iter()
            .find(|node| node.name().as_str() == "Run")
            .expect("Run procedure must exist");

        let reversed_run = reversed
            .nodes_by_kind(NodeKind::Procedure)
            .into_iter()
            .find(|node| node.name().as_str() == "Run")
            .expect("Run procedure must exist");

        let normal_calls = normal.outgoing_by_kind(normal_run.id(), EdgeKind::Calls);
        let reversed_calls = reversed.outgoing_by_kind(reversed_run.id(), EdgeKind::Calls);

        assert_eq!(normal_calls.len(), 1);
        assert_eq!(reversed_calls.len(), 1);
        assert_eq!(normal_calls[0].target(), reversed_calls[0].target());
    }
}
