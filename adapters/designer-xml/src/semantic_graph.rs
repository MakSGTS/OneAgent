//! Production semantic graph emission for hierarchical Designer XML sources.

use oneagent_bsl::{
    BslDeclarationExtractor, BslParseError, BslSymbolKind, LineBslDeclarationExtractor,
};
use oneagent_common::{EntityId, SourceLocation, SourcePath, SourcePosition, SourceSpan};
use oneagent_graph::{
    Confidence, EdgeKind, FactOrigin, GraphError, GraphNode, GraphNodePayload,
    GraphNodePayloadError, NodeKind, ProducerId, Provenance, ResolutionState, SemanticGraph,
};
use oneagent_metadata::{MetadataKind, MetadataPayload};
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};

use crate::source_hash::sha256_hex;
use crate::{
    DesignerXmlBuildScope, DesignerXmlLoadError, DesignerXmlMetadataObjectDescriptor,
    DesignerXmlMetadataObjectError, DesignerXmlMetadataObjectReader, DesignerXmlModuleDescriptor,
    DesignerXmlModuleError, DesignerXmlModuleReader, FileSystemDesignerXmlConfigurationLoader,
    FileSystemDesignerXmlMetadataObjectReader, FileSystemDesignerXmlModuleReader,
};

const CONFIGURATION_FILE: &str = "Configuration.xml";
const GRAPH_PRODUCER: &str = "oneagent.designer-xml.semantic-graph-builder";
const BSL_PRODUCER: &str = "oneagent.designer-xml.bsl-declarations";

/// Builds the accepted source-independent semantic graph slice from Designer XML.
pub trait DesignerXmlSemanticGraphBuilder {
    /// Builds a graph with explicit complete or partial input semantics.
    ///
    /// # Errors
    ///
    /// Returns a fatal error without a graph when any supplied accepted stage fails.
    fn build_graph(
        &self,
        project_root: &Path,
        scope: DesignerXmlBuildScope,
    ) -> Result<SemanticGraph, DesignerXmlGraphError>;
}

/// Filesystem implementation of [`DesignerXmlSemanticGraphBuilder`].
#[derive(Debug, Default, Clone, Copy)]
pub struct FileSystemDesignerXmlSemanticGraphBuilder;

impl DesignerXmlSemanticGraphBuilder for FileSystemDesignerXmlSemanticGraphBuilder {
    fn build_graph(
        &self,
        project_root: &Path,
        scope: DesignerXmlBuildScope,
    ) -> Result<SemanticGraph, DesignerXmlGraphError> {
        let (configuration, configuration_payload) =
            FileSystemDesignerXmlConfigurationLoader::load_with_payload(project_root, scope)?;
        let metadata = FileSystemDesignerXmlMetadataObjectReader.read_all(project_root, scope)?;
        let modules =
            FileSystemDesignerXmlModuleReader.read_modules(project_root, scope, &metadata)?;

        let mut graph = SemanticGraph::new();
        let configuration_path = project_root.join(CONFIGURATION_FILE);
        let configuration_provenance = provenance_from_file(
            &configuration_path,
            &format!("configuration={}", configuration.id().as_str()),
            GRAPH_PRODUCER,
        )?;
        insert_metadata_node(
            &mut graph,
            configuration.id().clone(),
            configuration.name().clone(),
            MetadataKind::Configuration,
            configuration_payload,
            configuration_provenance,
        )?;

        for descriptor in &metadata {
            emit_metadata(&mut graph, configuration.id(), descriptor)?;
        }
        for module in &modules {
            emit_module_and_declarations(&mut graph, module)?;
        }
        Ok(graph)
    }
}

fn emit_metadata(
    graph: &mut SemanticGraph,
    configuration_id: &EntityId,
    descriptor: &DesignerXmlMetadataObjectDescriptor,
) -> Result<(), DesignerXmlGraphError> {
    let provenance = provenance_from_file(
        descriptor.source().artifact_path(),
        &format!(
            "metadata={};kind={}",
            descriptor.id().as_str(),
            descriptor.kind().as_str()
        ),
        GRAPH_PRODUCER,
    )?;
    insert_metadata_node(
        graph,
        descriptor.id().clone(),
        descriptor.name().clone(),
        descriptor.kind(),
        descriptor.payload().clone(),
        provenance.clone(),
    )?;
    insert_contains(
        graph,
        configuration_id.clone(),
        descriptor.id().clone(),
        provenance,
    )
}

fn emit_module_and_declarations(
    graph: &mut SemanticGraph,
    module: &DesignerXmlModuleDescriptor,
) -> Result<(), DesignerXmlGraphError> {
    let module_source = source_id(
        module.source().artifact_path(),
        module.source().raw_source(),
        &format!(
            "module={};role={}",
            module.id().as_str(),
            module.kind().as_str()
        ),
    )?;
    let module_location = file_location(module.source().artifact_path())?;
    let module_provenance = parsed_provenance(module_source.clone(), GRAPH_PRODUCER)
        .with_location(module_location.clone());
    insert_unique_node(
        graph,
        GraphNode::new_with_provenance(
            module.id().clone(),
            module.name().clone(),
            NodeKind::Module,
            vec![module_provenance.clone()],
        ),
    )?;
    insert_contains(
        graph,
        module.owner_id().clone(),
        module.id().clone(),
        module_provenance,
    )?;

    let symbols = LineBslDeclarationExtractor.extract(module.id(), module.source_text())?;
    for symbol in symbols {
        let symbol_source = EntityId::new(format!(
            "{};declaration={};line={}",
            module_source.as_str(),
            symbol.id().as_str(),
            symbol.line()
        ))
        .map_err(|_| DesignerXmlGraphError::InvalidSourceIdentifier)?;
        let provenance = parsed_provenance(symbol_source, BSL_PRODUCER).with_location(
            declaration_location(module.source().artifact_path(), symbol.line())?,
        );
        let kind = match symbol.kind() {
            BslSymbolKind::Procedure => NodeKind::Procedure,
            BslSymbolKind::Function => NodeKind::Function,
        };
        insert_unique_node(
            graph,
            GraphNode::new_with_provenance(
                symbol.id().clone(),
                symbol.name().clone(),
                kind,
                vec![provenance.clone()],
            ),
        )?;
        insert_contains(graph, module.id().clone(), symbol.id().clone(), provenance)?;
    }
    Ok(())
}

fn insert_metadata_node(
    graph: &mut SemanticGraph,
    id: EntityId,
    name: oneagent_common::EntityName,
    kind: MetadataKind,
    payload: MetadataPayload,
    provenance: Provenance,
) -> Result<(), DesignerXmlGraphError> {
    let node = GraphNode::new_with_payload_and_provenance(
        id,
        name,
        NodeKind::Metadata(kind),
        GraphNodePayload::Metadata(payload),
        vec![provenance],
    )?;
    insert_unique_node(graph, node)
}

fn insert_unique_node(
    graph: &mut SemanticGraph,
    node: GraphNode,
) -> Result<(), DesignerXmlGraphError> {
    let id = node.id().clone();
    if graph.insert_node(node).is_some() {
        return Err(DesignerXmlGraphError::DuplicateNode(id));
    }
    Ok(())
}

fn insert_contains(
    graph: &mut SemanticGraph,
    owner: EntityId,
    child: EntityId,
    provenance: Provenance,
) -> Result<(), DesignerXmlGraphError> {
    if !graph.insert_edge_with_provenance(
        owner.clone(),
        child.clone(),
        EdgeKind::Contains,
        provenance,
    )? {
        return Err(DesignerXmlGraphError::DuplicateContains { owner, child });
    }
    Ok(())
}

fn provenance_from_file(
    path: &Path,
    fact: &str,
    producer: &'static str,
) -> Result<Provenance, DesignerXmlGraphError> {
    let raw = fs::read(path).map_err(|source| DesignerXmlGraphError::ReadSource {
        path: path.to_path_buf(),
        source,
    })?;
    Ok(parsed_provenance(source_id(path, &raw, fact)?, producer))
}

fn source_id(path: &Path, raw: &[u8], fact: &str) -> Result<EntityId, DesignerXmlGraphError> {
    EntityId::new(format!(
        "{}#sha256={};{fact}",
        path.to_string_lossy().replace('\\', "/"),
        sha256_hex(raw)
    ))
    .map_err(|_| DesignerXmlGraphError::InvalidSourceIdentifier)
}

fn parsed_provenance(source: EntityId, producer: &'static str) -> Provenance {
    Provenance::new(
        Some(source),
        ProducerId::new(producer),
        FactOrigin::Parsed,
        Confidence::Exact,
        ResolutionState::NotApplicable,
    )
}

fn file_location(path: &Path) -> Result<SourceLocation, DesignerXmlGraphError> {
    let path = path
        .to_str()
        .ok_or(DesignerXmlGraphError::InvalidSourceLocation)
        .and_then(|path| {
            SourcePath::new(path).map_err(|_| DesignerXmlGraphError::InvalidSourceLocation)
        })?;
    Ok(SourceLocation::new(path, None))
}

fn declaration_location(path: &Path, line: usize) -> Result<SourceLocation, DesignerXmlGraphError> {
    let path = file_location(path)?.path().clone();
    let line = u32::try_from(line).map_err(|_| DesignerXmlGraphError::InvalidSourceLocation)?;
    let position =
        SourcePosition::new(line, 1).map_err(|_| DesignerXmlGraphError::InvalidSourceLocation)?;
    let span = SourceSpan::new(position, position)
        .map_err(|_| DesignerXmlGraphError::InvalidSourceLocation)?;
    Ok(SourceLocation::new(path, Some(span)))
}

/// Fatal errors produced by the Designer XML semantic builder.
#[derive(Debug)]
pub enum DesignerXmlGraphError {
    /// Configuration loading failed.
    Configuration(DesignerXmlLoadError),
    /// Metadata descriptor assembly failed.
    Metadata(DesignerXmlMetadataObjectError),
    /// Module assembly failed.
    Module(DesignerXmlModuleError),
    /// A source artifact could not be read for exact provenance.
    ReadSource {
        /// Source path.
        path: PathBuf,
        /// Underlying I/O error.
        source: std::io::Error,
    },
    /// A provenance source identifier could not be represented.
    InvalidSourceIdentifier,
    /// A module path or declaration position could not become typed evidence.
    InvalidSourceLocation,
    /// Typed graph payload construction failed.
    NodePayload(GraphNodePayloadError),
    /// Graph endpoint validation failed.
    Graph(GraphError),
    /// BSL declaration extraction failed.
    Bsl(BslParseError),
    /// Two accepted facts produced one node identity.
    DuplicateNode(EntityId),
    /// One ownership edge was emitted more than once.
    DuplicateContains {
        /// Owner identifier.
        owner: EntityId,
        /// Child identifier.
        child: EntityId,
    },
}

impl From<DesignerXmlLoadError> for DesignerXmlGraphError {
    fn from(value: DesignerXmlLoadError) -> Self {
        Self::Configuration(value)
    }
}

impl From<DesignerXmlMetadataObjectError> for DesignerXmlGraphError {
    fn from(value: DesignerXmlMetadataObjectError) -> Self {
        Self::Metadata(value)
    }
}

impl From<DesignerXmlModuleError> for DesignerXmlGraphError {
    fn from(value: DesignerXmlModuleError) -> Self {
        Self::Module(value)
    }
}

impl From<GraphNodePayloadError> for DesignerXmlGraphError {
    fn from(value: GraphNodePayloadError) -> Self {
        Self::NodePayload(value)
    }
}

impl From<GraphError> for DesignerXmlGraphError {
    fn from(value: GraphError) -> Self {
        Self::Graph(value)
    }
}

impl From<BslParseError> for DesignerXmlGraphError {
    fn from(value: BslParseError) -> Self {
        Self::Bsl(value)
    }
}

impl Display for DesignerXmlGraphError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Configuration(source) => {
                write!(formatter, "Designer XML configuration failed: {source}")
            }
            Self::Metadata(source) => write!(formatter, "Designer XML metadata failed: {source}"),
            Self::Module(source) => write!(formatter, "Designer XML module failed: {source}"),
            Self::ReadSource { path, source } => write!(
                formatter,
                "failed to read Designer XML provenance source {}: {source}",
                path.display()
            ),
            Self::InvalidSourceIdentifier => {
                formatter.write_str("invalid Designer XML provenance source identifier")
            }
            Self::InvalidSourceLocation => {
                formatter.write_str("invalid Designer XML source location")
            }
            Self::NodePayload(source) => write!(formatter, "invalid graph payload: {source}"),
            Self::Graph(source) => write!(formatter, "Designer XML graph failed: {source}"),
            Self::Bsl(source) => write!(formatter, "Designer XML BSL failed: {source}"),
            Self::DuplicateNode(id) => write!(formatter, "duplicate Designer XML node {id}"),
            Self::DuplicateContains { owner, child } => {
                write!(
                    formatter,
                    "duplicate Designer XML ownership {owner} -> {child}"
                )
            }
        }
    }
}

impl std::error::Error for DesignerXmlGraphError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Configuration(source) => Some(source),
            Self::Metadata(source) => Some(source),
            Self::Module(source) => Some(source),
            Self::ReadSource { source, .. } => Some(source),
            Self::NodePayload(source) => Some(source),
            Self::Graph(source) => Some(source),
            Self::Bsl(source) => Some(source),
            Self::InvalidSourceIdentifier
            | Self::InvalidSourceLocation
            | Self::DuplicateNode(_)
            | Self::DuplicateContains { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DesignerXmlGraphError, DesignerXmlSemanticGraphBuilder,
        FileSystemDesignerXmlSemanticGraphBuilder,
    };
    use crate::DesignerXmlBuildScope;
    use oneagent_common::{EntityId, EntityName};
    use oneagent_graph::{EdgeKind, NodeId, NodeKind};
    use oneagent_metadata::MetadataKind;
    use std::fs;
    use std::path::Path;
    use tempfile::tempdir;

    const DUMP_INFO: &str = r#"<ConfigDumpInfo xmlns="http://v8.1c.ru/8.3/xcf/dumpinfo" format="Hierarchical" version="2.20"><ConfigVersions /></ConfigDumpInfo>"#;
    const CONFIGURATION: &str = r#"<MetaDataObject xmlns="http://v8.1c.ru/8.3/MDClasses" version="2.20"><Configuration uuid="408a41e7-907a-4fb3-8999-83d1e8b6e093"><Properties><Name>DNSWorldEdition</Name><Synonym><v8:item xmlns:v8="http://v8.1c.ru/8.1/data/core"><v8:content>DNS World</v8:content></v8:item></Synonym></Properties></Configuration></MetaDataObject>"#;
    const PRODUCTS: &[u8] = include_bytes!("../tests/fixtures/metadata/Catalogs/Products.xml");
    const DESIGNER_MODULE: &[u8] =
        include_bytes!("../tests/fixtures/modules/designer/DynamicSecurityOverridable.bsl");

    fn write_project(root: &Path) {
        fs::write(root.join("ConfigDumpInfo.xml"), DUMP_INFO).expect("dump marker must be created");
        fs::write(root.join("Configuration.xml"), CONFIGURATION)
            .expect("configuration marker must be created");
    }

    fn write_accepted_sources(root: &Path) {
        write_project(root);
        write_catalog(root);
        write_common_module(root);
    }

    fn write_catalog(root: &Path) {
        fs::create_dir_all(root.join("Catalogs")).expect("Catalogs must be created");
        fs::write(root.join("Catalogs/Products.xml"), PRODUCTS)
            .expect("exact metadata fixture must be written");
    }

    fn write_common_module(root: &Path) {
        fs::create_dir_all(root.join("CommonModules/DynamicSecurityOverridable/Ext"))
            .expect("Common Module path must be created");
        fs::write(
            root.join("CommonModules/DynamicSecurityOverridable.xml"),
            r#"<MetaDataObject xmlns="http://v8.1c.ru/8.3/MDClasses" version="2.20"><CommonModule uuid="dc24575c-a787-411d-93bd-494271291d73"><Properties><Name>DynamicSecurityOverridable</Name><Synonym><v8:item xmlns:v8="http://v8.1c.ru/8.1/data/core"><v8:content>Dynamic security overridable</v8:content></v8:item></Synonym></Properties></CommonModule></MetaDataObject>"#,
        )
        .expect("Common Module descriptor must be written");
        fs::write(
            root.join("CommonModules/DynamicSecurityOverridable/Ext/Module.bsl"),
            DESIGNER_MODULE,
        )
        .expect("exact module fixture must be written");
    }

    #[test]
    fn public_builder_emits_only_accepted_graph_slice() {
        let root = tempdir().expect("temporary directory must be created");
        write_accepted_sources(root.path());

        let graph = FileSystemDesignerXmlSemanticGraphBuilder
            .build_graph(root.path(), DesignerXmlBuildScope::Complete)
            .expect("accepted project must build");

        assert_eq!(graph.node_count(), 5);
        assert_eq!(graph.edge_count(), 4);
        assert_eq!(
            graph
                .nodes_by_kind(NodeKind::Metadata(MetadataKind::Configuration))
                .len(),
            1
        );
        assert_eq!(
            graph
                .nodes_by_kind(NodeKind::Metadata(MetadataKind::Catalog))
                .len(),
            1
        );
        assert_eq!(
            graph
                .nodes_by_kind(NodeKind::Metadata(MetadataKind::CommonModule))
                .len(),
            1
        );
        assert_eq!(graph.nodes_by_kind(NodeKind::Module).len(), 1);
        assert_eq!(graph.nodes_by_kind(NodeKind::Procedure).len(), 1);
        assert!(graph.nodes_by_kind(NodeKind::Function).is_empty());
        assert!(graph.nodes_by_kind(NodeKind::Query).is_empty());
        assert!(graph.edges().all(|edge| edge.kind() == EdgeKind::Contains));

        let module = graph.nodes_by_kind(NodeKind::Module)[0];
        let procedure = graph.nodes_by_kind(NodeKind::Procedure)[0];
        let module_location = module.provenance()[0]
            .location()
            .expect("module location must exist");
        let procedure_location = procedure.provenance()[0]
            .location()
            .expect("procedure location must exist");
        assert!(
            module_location
                .path()
                .as_str()
                .ends_with("/CommonModules/DynamicSecurityOverridable/Ext/Module.bsl")
        );
        assert_eq!(module_location.span(), None);
        let point = procedure_location
            .span()
            .expect("declaration point must exist");
        assert_eq!(point.start().line(), 5);
        assert_eq!(point.start().column(), 1);
        assert_eq!(point.start(), point.end());
    }

    #[test]
    fn query_report_diff_validation_and_provenance_are_deterministic() {
        let root = tempdir().expect("temporary directory must be created");
        write_accepted_sources(root.path());
        let first = FileSystemDesignerXmlSemanticGraphBuilder
            .build_graph(root.path(), DesignerXmlBuildScope::Partial)
            .expect("first build must succeed");
        fs::remove_dir_all(root.path().join("Catalogs"))
            .expect("Catalogs must be removed for reordered recreation");
        fs::remove_dir_all(root.path().join("CommonModules"))
            .expect("CommonModules must be removed for reordered recreation");
        write_common_module(root.path());
        write_catalog(root.path());
        let second = FileSystemDesignerXmlSemanticGraphBuilder
            .build_graph(root.path(), DesignerXmlBuildScope::Partial)
            .expect("reordered repeated build must succeed");

        let procedure = first
            .nodes_by_kind(NodeKind::Procedure)
            .into_iter()
            .next()
            .expect("procedure must exist");
        let owner = first
            .query()
            .owner(&NodeId::new(procedure.id().as_str()))
            .expect("procedure must have exactly one owner");
        assert_eq!(owner.kind(), NodeKind::Module);
        assert_eq!(
            first
                .query()
                .nodes_by_name(&EntityName::new("Products").expect("name must be valid"))
                .len(),
            1
        );

        let report = first.report();
        assert_eq!(report.graph().total_nodes(), 5);
        assert_eq!(report.graph().total_edges(), 4);
        assert_eq!(report.provenance().nodes_without_provenance(), 0);
        assert_eq!(report.provenance().edges_without_provenance(), 0);
        assert!(first.validate().is_valid());
        assert_eq!(first.validate().error_count(), 0);

        let diff = first.diff(&second);
        assert_eq!(diff.summary().total_changes(), 0);
        assert_eq!(
            first.nodes().collect::<Vec<_>>(),
            second.nodes().collect::<Vec<_>>()
        );
        assert!(first.nodes().all(|node| {
            node.provenance().iter().all(|provenance| {
                provenance.confidence() == oneagent_graph::Confidence::Exact
                    && provenance
                        .source()
                        .is_some_and(|source| source.as_str().contains("#sha256="))
            })
        }));
    }

    #[test]
    fn explicit_partial_build_can_contain_only_configuration() {
        let root = tempdir().expect("temporary directory must be created");
        write_project(root.path());

        let graph = FileSystemDesignerXmlSemanticGraphBuilder
            .build_graph(root.path(), DesignerXmlBuildScope::Partial)
            .expect("explicit partial root must build");

        assert_eq!(graph.node_count(), 1);
        assert_eq!(graph.edge_count(), 0);
        assert!(graph.validate().is_valid());
    }

    #[test]
    fn malformed_accepted_metadata_or_bsl_returns_no_graph() {
        let root = tempdir().expect("temporary directory must be created");
        write_accepted_sources(root.path());
        fs::write(root.path().join("Catalogs/Products.xml"), "<broken>")
            .expect("metadata fixture must be mutated");
        assert!(matches!(
            FileSystemDesignerXmlSemanticGraphBuilder
                .build_graph(root.path(), DesignerXmlBuildScope::Partial),
            Err(DesignerXmlGraphError::Metadata(_))
        ));

        fs::write(root.path().join("Catalogs/Products.xml"), PRODUCTS)
            .expect("metadata fixture must be restored");
        fs::write(
            root.path()
                .join("CommonModules/DynamicSecurityOverridable/Ext/Module.bsl"),
            "Procedure MissingParenthesis",
        )
        .expect("module fixture must be mutated");
        assert!(matches!(
            FileSystemDesignerXmlSemanticGraphBuilder
                .build_graph(root.path(), DesignerXmlBuildScope::Complete),
            Err(DesignerXmlGraphError::Bsl(_))
        ));
    }

    #[test]
    fn canonical_node_identities_do_not_include_source_paths() {
        let root = tempdir().expect("temporary directory must be created");
        write_accepted_sources(root.path());

        let graph = FileSystemDesignerXmlSemanticGraphBuilder
            .build_graph(root.path(), DesignerXmlBuildScope::Complete)
            .expect("accepted project must build");
        let root_text = root.path().to_string_lossy();

        assert!(
            graph
                .nodes()
                .all(|node| !node.id().as_str().contains(root_text.as_ref()))
        );
        assert!(
            graph
                .node(
                    &EntityId::new("92bcb692-56c4-4199-bf7e-e33cdd76a310")
                        .expect("id must be valid")
                )
                .is_some()
        );
        assert!(
            graph
                .node(
                    &EntityId::new("dc24575c-a787-411d-93bd-494271291d73:common_module")
                        .expect("id must be valid")
                )
                .is_some()
        );
    }
}
