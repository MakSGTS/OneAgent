//! Integration between EDT module files, BSL declarations and the semantic graph.

use oneagent_bsl::{
    BslCallError, BslCallExtractor, BslCallResolver, BslDeclarationExtractor, BslParseError,
    BslSymbolKind, LineBslCallExtractor, LineBslDeclarationExtractor, LocalBslCallResolver,
};
use oneagent_graph::{EdgeKind, GraphEdge, GraphError, GraphNode, NodeKind, SemanticGraph};
use std::fmt::{Display, Formatter};
use std::fs;

use crate::EdtModuleDescriptor;

/// Adds top-level BSL declarations and local call relations to the semantic graph.
///
/// # Errors
///
/// Returns an error when the module cannot be read, parsed, resolved or inserted
/// into the graph.
pub fn add_module_symbols(
    graph: &mut SemanticGraph,
    module: &EdtModuleDescriptor,
) -> Result<usize, EdtBslGraphError> {
    let source =
        fs::read_to_string(module.path()).map_err(|source| EdtBslGraphError::ReadModule {
            path: module.path().to_path_buf(),
            source,
        })?;

    let symbols = LineBslDeclarationExtractor
        .extract(module.id(), &source)
        .map_err(EdtBslGraphError::ParseDeclarations)?;

    for symbol in &symbols {
        let node_kind = match symbol.kind() {
            BslSymbolKind::Procedure => NodeKind::Procedure,
            BslSymbolKind::Function => NodeKind::Function,
        };

        graph.insert_node(GraphNode::new(
            symbol.id().clone(),
            symbol.name().clone(),
            node_kind,
        ));

        graph
            .insert_edge(GraphEdge::new(
                module.id().clone(),
                symbol.id().clone(),
                EdgeKind::Contains,
            ))
            .map_err(EdtBslGraphError::Graph)?;
    }

    let calls = LineBslCallExtractor
        .extract_calls(module.id(), &source)
        .map_err(EdtBslGraphError::ParseCalls)?;

    let resolution = LocalBslCallResolver.resolve(&symbols, &calls);

    for resolved_call in resolution.resolved() {
        graph
            .insert_edge(GraphEdge::new(
                resolved_call.origin_id().clone(),
                resolved_call.destination_id().clone(),
                EdgeKind::Calls,
            ))
            .map_err(EdtBslGraphError::Graph)?;
    }

    Ok(symbols.len())
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

    use super::add_module_symbols;

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
}
