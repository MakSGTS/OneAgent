//! Adapter for reading `1C:EDT` project sources.

mod bsl_graph;
mod metadata_object;
mod metadata_structure;
mod module_reader;

pub use metadata_object::{
    EdtMetadataObjectDescriptor, EdtMetadataObjectError, EdtMetadataObjectReader,
    FileSystemEdtMetadataObjectReader,
};

pub use metadata_structure::{
    EdtMetadataChildDescriptor, EdtMetadataChildKind, EdtMetadataStructureError,
    EdtMetadataStructureReader, FileSystemEdtMetadataStructureReader,
};

pub use module_reader::{
    EdtModuleDescriptor, EdtModuleError, EdtModuleKind, EdtModuleReader, FileSystemEdtModuleReader,
};

pub use bsl_graph::{
    AnalyzedBslModule, EdtBslGraphError, add_configuration_module_symbols, add_module_symbols,
    analyze_module,
};

use oneagent_common::{EntityId, EntityName};
use oneagent_workspace::{Configuration, WorkspaceFormat};
use quick_xml::Reader;
use quick_xml::events::Event;
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};

const CONFIGURATION_RELATIVE_PATH: &str = "src/Configuration/Configuration.mdo";

/// Port for loading a semantic configuration from an EDT project.
pub trait EdtConfigurationLoader {
    /// Loads an EDT configuration rooted at `project_root`.
    ///
    /// # Errors
    ///
    /// Returns an error when the project structure or XML is invalid.
    fn load(&self, project_root: &Path) -> Result<Configuration, EdtLoadError>;
}

/// Filesystem implementation of the EDT configuration loader.
#[derive(Debug, Default, Clone, Copy)]
pub struct FileSystemEdtConfigurationLoader;

impl EdtConfigurationLoader for FileSystemEdtConfigurationLoader {
    fn load(&self, project_root: &Path) -> Result<Configuration, EdtLoadError> {
        let configuration_path = project_root.join(CONFIGURATION_RELATIVE_PATH);

        if !configuration_path.is_file() {
            return Err(EdtLoadError::ConfigurationFileNotFound(configuration_path));
        }

        let xml =
            fs::read_to_string(&configuration_path).map_err(|source| EdtLoadError::ReadFile {
                path: configuration_path.clone(),
                source,
            })?;

        let descriptor = parse_configuration_descriptor(&xml)?;

        let id = EntityId::new(
            descriptor
                .uuid
                .unwrap_or_else(|| format!("edt:{}", project_root.display())),
        )
        .map_err(|_| EdtLoadError::InvalidIdentifier)?;

        let name = EntityName::new(descriptor.name).map_err(|_| EdtLoadError::MissingName)?;

        Ok(Configuration::new(
            id,
            name,
            project_root,
            WorkspaceFormat::Edt,
        ))
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
struct ConfigurationDescriptor {
    name: String,
    uuid: Option<String>,
    synonym: Option<String>,
}

fn parse_configuration_descriptor(xml: &str) -> Result<ConfigurationDescriptor, EdtLoadError> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut descriptor = ConfigurationDescriptor::default();
    let mut path = Vec::<String>::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(event)) => {
                path.push(local_name(event.name().as_ref()));

                for attribute in event.attributes().with_checks(false) {
                    let attribute = attribute
                        .map_err(|source| EdtLoadError::MalformedXml(source.to_string()))?;
                    if local_name(attribute.key.as_ref()) == "uuid" && descriptor.uuid.is_none() {
                        descriptor.uuid = Some(
                            attribute
                                .decode_and_unescape_value(reader.decoder())
                                .map_err(|source| EdtLoadError::MalformedXml(source.to_string()))?
                                .into_owned(),
                        );
                    }
                }
            }
            Ok(Event::Empty(event)) => {
                for attribute in event.attributes().with_checks(false) {
                    let attribute = attribute
                        .map_err(|source| EdtLoadError::MalformedXml(source.to_string()))?;
                    if local_name(attribute.key.as_ref()) == "uuid" && descriptor.uuid.is_none() {
                        descriptor.uuid = Some(
                            attribute
                                .decode_and_unescape_value(reader.decoder())
                                .map_err(|source| EdtLoadError::MalformedXml(source.to_string()))?
                                .into_owned(),
                        );
                    }
                }
            }
            Ok(Event::Text(event)) => {
                let text = event
                    .decode()
                    .map_err(|source| EdtLoadError::MalformedXml(source.to_string()))?
                    .into_owned();

                match path.last().map(String::as_str) {
                    Some("name") if descriptor.name.is_empty() => descriptor.name = text,
                    Some("content") if is_synonym_content_path(&path) => {
                        descriptor.synonym.get_or_insert(text);
                    }
                    _ => {}
                }
            }
            Ok(Event::End(_)) => {
                path.pop();
            }
            Ok(Event::Eof) => break,
            Ok(_) => {}
            Err(source) => return Err(EdtLoadError::MalformedXml(source.to_string())),
        }
    }

    if descriptor.name.trim().is_empty() {
        return Err(EdtLoadError::MissingName);
    }

    Ok(descriptor)
}

fn is_synonym_content_path(path: &[String]) -> bool {
    path.len() >= 2 && path[path.len() - 2] == "synonym" && path[path.len() - 1] == "content"
}

fn local_name(name: &[u8]) -> String {
    let name = String::from_utf8_lossy(name);
    name.rsplit(':').next().unwrap_or(&name).to_owned()
}

/// Errors produced while loading an EDT configuration.
#[derive(Debug)]
pub enum EdtLoadError {
    /// `Configuration.mdo` was not found.
    ConfigurationFileNotFound(PathBuf),
    /// The configuration file could not be read.
    ReadFile {
        /// File path.
        path: PathBuf,
        /// Underlying I/O error.
        source: std::io::Error,
    },
    /// The XML document is malformed.
    MalformedXml(String),
    /// The configuration has no metadata name.
    MissingName,
    /// A stable configuration identifier could not be created.
    InvalidIdentifier,
}

impl Display for EdtLoadError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ConfigurationFileNotFound(path) => write!(
                formatter,
                "EDT configuration file was not found: {}",
                path.display()
            ),
            Self::ReadFile { path, source } => write!(
                formatter,
                "failed to read EDT configuration {}: {source}",
                path.display()
            ),
            Self::MalformedXml(message) => {
                write!(formatter, "malformed EDT configuration XML: {message}")
            }
            Self::MissingName => formatter.write_str("EDT configuration name is missing"),
            Self::InvalidIdentifier => {
                formatter.write_str("EDT configuration identifier is invalid")
            }
        }
    }
}

impl std::error::Error for EdtLoadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ReadFile { source, .. } => Some(source),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use tempfile::tempdir;

    use super::{
        EdtConfigurationLoader, FileSystemEdtConfigurationLoader, parse_configuration_descriptor,
    };

    const CONFIGURATION_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<mdclass:Configuration
    xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass"
    uuid="40d7b43a-b34f-4e6f-a756-6d7f0dc850f0">
    <name>DemoConfiguration</name>
    <synonym>
        <key>ru</key>
        <content>Демонстрационная конфигурация</content>
    </synonym>
</mdclass:Configuration>
"#;

    #[test]
    fn parses_configuration_descriptor() {
        let descriptor =
            parse_configuration_descriptor(CONFIGURATION_XML).expect("XML must be valid");

        assert_eq!(descriptor.name, "DemoConfiguration");
        assert_eq!(
            descriptor.uuid.as_deref(),
            Some("40d7b43a-b34f-4e6f-a756-6d7f0dc850f0")
        );
        assert_eq!(
            descriptor.synonym.as_deref(),
            Some("Демонстрационная конфигурация")
        );
    }

    #[test]
    fn loads_configuration_from_edt_project() {
        let root = tempdir().expect("temporary directory must be created");
        let configuration_directory = root.path().join("src/Configuration");

        fs::create_dir_all(&configuration_directory)
            .expect("configuration directory must be created");
        fs::write(
            configuration_directory.join("Configuration.mdo"),
            CONFIGURATION_XML,
        )
        .expect("configuration file must be created");

        let configuration = FileSystemEdtConfigurationLoader
            .load(root.path())
            .expect("configuration must load");

        assert_eq!(configuration.name().as_str(), "DemoConfiguration");
        assert_eq!(
            configuration.id().as_str(),
            "40d7b43a-b34f-4e6f-a756-6d7f0dc850f0"
        );
    }

    #[test]
    fn rejects_configuration_without_name() {
        let xml = r#"<mdclass:Configuration uuid="id" />"#;
        let error = parse_configuration_descriptor(xml).expect_err("missing name must be rejected");
        assert_eq!(error.to_string(), "EDT configuration name is missing");
    }
}

use oneagent_graph::{EdgeKind, GraphEdge, GraphNode, NodeKind, SemanticGraph};
use oneagent_metadata::MetadataKind;
use std::collections::BTreeMap;

/// Builds an initial semantic graph from an EDT project.
pub trait EdtSemanticGraphBuilder {
    /// Builds a semantic graph rooted at the EDT configuration.
    ///
    /// # Errors
    ///
    /// Returns an error when project metadata cannot be read or represented.
    fn build_graph(&self, project_root: &Path) -> Result<SemanticGraph, EdtGraphError>;
}

/// Filesystem implementation of the EDT semantic graph builder.
#[derive(Debug, Default, Clone, Copy)]
pub struct FileSystemEdtSemanticGraphBuilder;

impl EdtSemanticGraphBuilder for FileSystemEdtSemanticGraphBuilder {
    fn build_graph(&self, project_root: &Path) -> Result<SemanticGraph, EdtGraphError> {
        let configuration = FileSystemEdtConfigurationLoader.load(project_root)?;
        let mut graph = SemanticGraph::new();
        let mut configuration_modules = Vec::new();

        let configuration_id = configuration.id().clone();
        graph.insert_node(GraphNode::new(
            configuration_id.clone(),
            configuration.name().clone(),
            NodeKind::Metadata(MetadataKind::Configuration),
        ));

        let source_root = project_root.join("src");
        if !source_root.is_dir() {
            return Ok(graph);
        }

        let kind_by_directory = supported_metadata_directories();

        for entry in fs::read_dir(&source_root).map_err(|source| EdtGraphError::ReadDirectory {
            path: source_root.clone(),
            source,
        })? {
            let entry = entry.map_err(|source| EdtGraphError::ReadDirectoryEntry {
                path: source_root.clone(),
                source,
            })?;

            let file_type = entry
                .file_type()
                .map_err(|source| EdtGraphError::ReadFileType {
                    path: entry.path(),
                    source,
                })?;

            if !file_type.is_dir() {
                continue;
            }

            let directory_name = entry.file_name().to_string_lossy().into_owned();
            let Some(kind) = kind_by_directory.get(directory_name.as_str()).copied() else {
                continue;
            };

            configuration_modules.extend(collect_top_level_metadata(
                &entry.path(),
                kind,
                &configuration_id,
                &mut graph,
            )?);
        }

        add_configuration_module_symbols(&mut graph, &configuration_modules)
            .map_err(EdtGraphError::Bsl)?;

        Ok(graph)
    }
}

fn supported_metadata_directories() -> BTreeMap<&'static str, MetadataKind> {
    BTreeMap::from([
        ("Catalogs", MetadataKind::Catalog),
        ("Documents", MetadataKind::Document),
        ("Enums", MetadataKind::Enumeration),
        ("CommonModules", MetadataKind::CommonModule),
        ("Reports", MetadataKind::Report),
        ("DataProcessors", MetadataKind::DataProcessor),
        ("InformationRegisters", MetadataKind::InformationRegister),
        ("AccumulationRegisters", MetadataKind::AccumulationRegister),
        ("AccountingRegisters", MetadataKind::AccountingRegister),
        ("CalculationRegisters", MetadataKind::CalculationRegister),
        ("BusinessProcesses", MetadataKind::BusinessProcess),
        ("Tasks", MetadataKind::Task),
        ("Roles", MetadataKind::Role),
        ("CommonForms", MetadataKind::CommonForm),
        ("HTTPServices", MetadataKind::HttpService),
        ("WebServices", MetadataKind::WebService),
        ("XDTOPackages", MetadataKind::XdtoPackage),
        ("Subsystems", MetadataKind::Subsystem),
    ])
}

fn collect_top_level_metadata(
    directory: &Path,
    kind: MetadataKind,
    configuration_id: &EntityId,
    graph: &mut SemanticGraph,
) -> Result<Vec<EdtModuleDescriptor>, EdtGraphError> {
    let metadata_reader = FileSystemEdtMetadataObjectReader;
    let module_reader = FileSystemEdtModuleReader;
    let mut configuration_modules = Vec::new();

    for entry in fs::read_dir(directory).map_err(|source| EdtGraphError::ReadDirectory {
        path: directory.to_path_buf(),
        source,
    })? {
        let entry = entry.map_err(|source| EdtGraphError::ReadDirectoryEntry {
            path: directory.to_path_buf(),
            source,
        })?;

        if !entry
            .file_type()
            .map_err(|source| EdtGraphError::ReadFileType {
                path: entry.path(),
                source,
            })?
            .is_dir()
        {
            continue;
        }

        let object_directory = entry.path();

        let descriptor = metadata_reader
            .read(&object_directory, kind)
            .map_err(EdtGraphError::MetadataObject)?;

        graph.insert_node(GraphNode::new(
            descriptor.id().clone(),
            descriptor.name().clone(),
            NodeKind::Metadata(descriptor.kind()),
        ));

        graph
            .insert_edge(GraphEdge::new(
                configuration_id.clone(),
                descriptor.id().clone(),
                EdgeKind::Contains,
            ))
            .map_err(EdtGraphError::Graph)?;

        let structure_reader = FileSystemEdtMetadataStructureReader;

        let children = structure_reader
            .read_children(&descriptor)
            .map_err(EdtGraphError::MetadataStructure)?;

        for child in children {
            graph.insert_node(GraphNode::new(
                child.id().clone(),
                child.name().clone(),
                child.kind().node_kind(),
            ));

            graph
                .insert_edge(GraphEdge::new(
                    child.parent_id().clone(),
                    child.id().clone(),
                    EdgeKind::Contains,
                ))
                .map_err(EdtGraphError::Graph)?;
        }

        let modules = module_reader
            .read_modules(descriptor.id(), descriptor.name(), &object_directory)
            .map_err(EdtGraphError::Module)?;

        for module in &modules {
            graph.insert_node(GraphNode::new(
                module.id().clone(),
                module.name().clone(),
                NodeKind::Module,
            ));

            graph
                .insert_edge(GraphEdge::new(
                    descriptor.id().clone(),
                    module.id().clone(),
                    EdgeKind::Contains,
                ))
                .map_err(EdtGraphError::Graph)?;
        }

        configuration_modules.extend(modules);
    }

    Ok(configuration_modules)
}

/// Errors produced while building an EDT semantic graph.
#[derive(Debug)]
pub enum EdtGraphError {
    /// EDT configuration loading failed.
    Load(EdtLoadError),
    /// A directory could not be read.
    ReadDirectory {
        /// Directory path.
        path: PathBuf,
        /// Underlying I/O error.
        source: std::io::Error,
    },
    /// A directory entry could not be read.
    ReadDirectoryEntry {
        /// Parent directory path.
        path: PathBuf,
        /// Underlying I/O error.
        source: std::io::Error,
    },
    /// File type metadata could not be read.
    ReadFileType {
        /// Entry path.
        path: PathBuf,
        /// Underlying I/O error.
        source: std::io::Error,
    },
    /// A stable identifier could not be created.
    InvalidIdentifier,
    /// A stable name could not be created.
    InvalidName,
    /// Semantic graph validation failed.
    /// A top-level metadata descriptor could not be read.
    MetadataObject(EdtMetadataObjectError),

    /// The internal structure of a metadata object could not be read.
    MetadataStructure(EdtMetadataStructureError),

    /// A metadata object module could not be read.
    Module(EdtModuleError),
    Graph(oneagent_graph::GraphError),

    /// BSL symbols could not be added to the graph.
    Bsl(EdtBslGraphError),
}

impl Display for EdtGraphError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Load(error) => write!(formatter, "failed to load EDT configuration: {error}"),
            Self::ReadDirectory { path, source } => {
                write!(
                    formatter,
                    "failed to read directory {}: {source}",
                    path.display()
                )
            }
            Self::ReadDirectoryEntry { path, source } => {
                write!(
                    formatter,
                    "failed to read an entry in {}: {source}",
                    path.display()
                )
            }
            Self::ReadFileType { path, source } => {
                write!(
                    formatter,
                    "failed to read file type for {}: {source}",
                    path.display()
                )
            }
            Self::MetadataObject(error) => {
                write!(formatter, "failed to read EDT metadata object: {error}")
            }
            Self::MetadataStructure(error) => {
                write!(
                    formatter,
                    "failed to read EDT metadata object structure: {error}"
                )
            }
            Self::Module(error) => {
                write!(formatter, "failed to read EDT module: {error}")
            }
            Self::InvalidIdentifier => formatter.write_str("failed to create EDT graph identifier"),
            Self::InvalidName => formatter.write_str("failed to create EDT graph name"),
            Self::Graph(error) => write!(formatter, "semantic graph error: {error}"),
            Self::Bsl(error) => {
                write!(formatter, "failed to add BSL symbols to graph: {error}")
            }
        }
    }
}

impl std::error::Error for EdtGraphError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Load(error) => Some(error),
            Self::ReadDirectory { source, .. }
            | Self::ReadDirectoryEntry { source, .. }
            | Self::ReadFileType { source, .. } => Some(source),
            Self::MetadataObject(error) => Some(error),
            Self::MetadataStructure(error) => Some(error),
            Self::Module(error) => Some(error),
            Self::Graph(error) => Some(error),
            Self::InvalidIdentifier | Self::InvalidName => None,
            Self::Bsl(error) => Some(error),
        }
    }
}

impl From<EdtLoadError> for EdtGraphError {
    fn from(error: EdtLoadError) -> Self {
        Self::Load(error)
    }
}

#[cfg(test)]
mod graph_tests {
    use oneagent_graph::{EdgeKind, NodeKind};
    use oneagent_metadata::MetadataKind;
    use std::fs;
    use tempfile::tempdir;

    use super::{EdtSemanticGraphBuilder, FileSystemEdtSemanticGraphBuilder};

    const CONFIGURATION_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<mdclass:Configuration
    xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass"
    uuid="11111111-2222-3333-4444-555555555555">
    <name>DemoConfiguration</name>
</mdclass:Configuration>
"#;
    const DOCUMENT_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<mdclass:Document
    xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass"
    uuid="aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee">
    <name>Sales</name>

    <attributes uuid="aaaaaaaa-1111-1111-1111-111111111111">
        <name>Company</name>
    </attributes>

    <attributes uuid="aaaaaaaa-2222-2222-2222-222222222222">
        <name>Warehouse</name>
    </attributes>

    <tabularSections uuid="aaaaaaaa-3333-3333-3333-333333333333">
        <name>Goods</name>
    </tabularSections>

    <forms uuid="aaaaaaaa-4444-4444-4444-444444444444">
        <name>DocumentForm</name>
    </forms>
</mdclass:Document>
"#;

    const CATALOG_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<mdclass:Catalog
    xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass"
    uuid="11111111-aaaa-bbbb-cccc-222222222222">
    <name>Products</name>
</mdclass:Catalog>
"#;

    const COMMON_MODULE_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<mdclass:CommonModule
    xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass"
    uuid="99999999-8888-7777-6666-555555555555">
    <name>AccessManagement</name>
</mdclass:CommonModule>
"#;

    const ACCUMULATION_REGISTER_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<mdclass:AccumulationRegister
    xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass"
    uuid="44444444-4444-4444-4444-444444444444">
    <name>StockBalance</name>

    <dimensions uuid="55555555-5555-5555-5555-555555555555">
        <name>Product</name>
    </dimensions>

    <dimensions uuid="66666666-6666-6666-6666-666666666666">
        <name>Warehouse</name>
    </dimensions>

    <resources uuid="77777777-7777-7777-7777-777777777777">
        <name>Quantity</name>
    </resources>
</mdclass:AccumulationRegister>
"#;

    fn create_edt_project() -> tempfile::TempDir {
        let root = tempdir().expect("temporary directory must be created");

        fs::create_dir_all(root.path().join("src/Configuration"))
            .expect("configuration directory must be created");
        fs::write(
            root.path().join("src/Configuration/Configuration.mdo"),
            CONFIGURATION_XML,
        )
        .expect("configuration file must be created");

        fs::create_dir_all(root.path().join("src/Documents/Sales"))
            .expect("document directory must be created");
        fs::create_dir_all(root.path().join("src/Catalogs/Products"))
            .expect("catalog directory must be created");
        fs::create_dir_all(root.path().join("src/CommonModules/AccessManagement"))
            .expect("common module directory must be created");
        fs::create_dir_all(root.path().join("src/AccumulationRegisters/StockBalance"))
            .expect("accumulation register directory must be created");
        fs::write(
            root.path().join("src/Documents/Sales/Sales.mdo"),
            DOCUMENT_XML,
        )
        .expect("document descriptor must be created");

        fs::write(
            root.path().join("src/Catalogs/Products/Products.mdo"),
            CATALOG_XML,
        )
        .expect("catalog descriptor must be created");

        fs::write(
            root.path()
                .join("src/CommonModules/AccessManagement/AccessManagement.mdo"),
            COMMON_MODULE_XML,
        )
        .expect("common module descriptor must be created");
        fs::write(
            root.path().join("src/Documents/Sales/ObjectModule.bsl"),
            concat!(
                "Procedure BeforeWrite()\n",
                "    AccessManagement.CheckAccess();\n",
                "EndProcedure",
            ),
        )
        .expect("object module must be created");

        fs::write(
            root.path().join("src/Documents/Sales/ManagerModule.bsl"),
            "Function GetData()\nEndFunction",
        )
        .expect("manager module must be created");

        fs::write(
            root.path()
                .join("src/CommonModules/AccessManagement/Module.bsl"),
            "Procedure CheckAccess() Export\nEndProcedure",
        )
        .expect("common module must be created");

        fs::write(
            root.path()
                .join("src/AccumulationRegisters/StockBalance/StockBalance.mdo"),
            ACCUMULATION_REGISTER_XML,
        )
        .expect("accumulation register descriptor must be created");

        root
    }

    #[test]
    fn builds_graph_with_configuration_and_metadata_objects() {
        let root = create_edt_project();

        let graph = FileSystemEdtSemanticGraphBuilder
            .build_graph(root.path())
            .expect("graph must build");

        assert_eq!(graph.node_count(), 18);
        assert_eq!(graph.edge_count(), 18);
        assert_eq!(graph.nodes_by_kind(NodeKind::Module).len(), 3);
        assert_eq!(graph.nodes_by_kind(NodeKind::Procedure).len(), 2);
        assert_eq!(graph.nodes_by_kind(NodeKind::Function).len(), 1);
        assert_eq!(
            graph
                .nodes_by_kind(NodeKind::Metadata(MetadataKind::AccumulationRegister))
                .len(),
            1
        );
        assert_eq!(graph.nodes_by_kind(NodeKind::Dimension).len(), 2);
        assert_eq!(graph.nodes_by_kind(NodeKind::Resource).len(), 1);
        assert_eq!(
            graph
                .nodes_by_kind(NodeKind::Metadata(MetadataKind::Document))
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
        assert_eq!(graph.nodes_by_kind(NodeKind::Attribute).len(), 2);
        assert_eq!(graph.nodes_by_kind(NodeKind::TabularSection).len(), 1);
        assert_eq!(graph.nodes_by_kind(NodeKind::Form).len(), 1);

        assert!(
            graph
                .nodes_by_kind(NodeKind::Attribute)
                .iter()
                .any(|node| node.name().as_str() == "Company")
        );
        assert!(
            graph
                .nodes_by_kind(NodeKind::TabularSection)
                .iter()
                .any(|node| node.name().as_str() == "Goods")
        );
        assert!(
            graph
                .nodes_by_kind(NodeKind::Form)
                .iter()
                .any(|node| node.name().as_str() == "DocumentForm")
        );
    }

    #[test]
    fn configuration_contains_discovered_objects() {
        let root = create_edt_project();

        let graph = FileSystemEdtSemanticGraphBuilder
            .build_graph(root.path())
            .expect("graph must build");

        let configuration = graph
            .nodes_by_kind(NodeKind::Metadata(MetadataKind::Configuration))
            .into_iter()
            .next()
            .expect("configuration node must exist");

        assert_eq!(
            graph
                .outgoing_by_kind(configuration.id(), EdgeKind::Contains)
                .len(),
            4
        );
    }

    #[test]
    fn resolves_cross_module_call_through_production_graph_builder() {
        let root = create_edt_project();

        let graph = FileSystemEdtSemanticGraphBuilder
            .build_graph(root.path())
            .expect("graph must build");

        let before_write = graph
            .nodes_by_kind(NodeKind::Procedure)
            .into_iter()
            .find(|node| node.name().as_str() == "BeforeWrite")
            .expect("BeforeWrite procedure must exist");

        let check_access = graph
            .nodes_by_kind(NodeKind::Procedure)
            .into_iter()
            .find(|node| node.name().as_str() == "CheckAccess")
            .expect("CheckAccess procedure must exist");

        let calls = graph.outgoing_by_kind(before_write.id(), EdgeKind::Calls);

        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].target(), check_access.id());
    }

    #[test]
    fn metadata_object_contains_attributes_tabular_sections_and_modules() {
        let root = create_edt_project();

        let graph = FileSystemEdtSemanticGraphBuilder
            .build_graph(root.path())
            .expect("graph must build");

        let document = graph
            .nodes_by_kind(NodeKind::Metadata(MetadataKind::Document))
            .into_iter()
            .next()
            .expect("document node must exist");

        let children = graph.outgoing_by_kind(document.id(), EdgeKind::Contains);

        assert_eq!(children.len(), 6);

        let child_kinds = children
            .iter()
            .map(|edge| {
                graph
                    .node(edge.target())
                    .expect("contained node must exist")
                    .kind()
            })
            .collect::<Vec<_>>();

        assert_eq!(
            child_kinds
                .iter()
                .filter(|kind| **kind == NodeKind::Attribute)
                .count(),
            2
        );

        assert_eq!(
            child_kinds
                .iter()
                .filter(|kind| **kind == NodeKind::TabularSection)
                .count(),
            1
        );

        assert_eq!(
            child_kinds
                .iter()
                .filter(|kind| **kind == NodeKind::Form)
                .count(),
            1
        );

        assert_eq!(
            child_kinds
                .iter()
                .filter(|kind| **kind == NodeKind::Module)
                .count(),
            2
        );
    }

    #[test]
    fn accumulation_register_contains_dimensions_and_resources() {
        let root = create_edt_project();

        let graph = FileSystemEdtSemanticGraphBuilder
            .build_graph(root.path())
            .expect("graph must build");

        let register = graph
            .nodes_by_kind(NodeKind::Metadata(MetadataKind::AccumulationRegister))
            .into_iter()
            .next()
            .expect("accumulation register node must exist");

        let children = graph.outgoing_by_kind(register.id(), EdgeKind::Contains);

        assert_eq!(children.len(), 3);

        let child_kinds = children
            .iter()
            .map(|edge| {
                graph
                    .node(edge.target())
                    .expect("contained register node must exist")
                    .kind()
            })
            .collect::<Vec<_>>();

        assert_eq!(
            child_kinds
                .iter()
                .filter(|kind| **kind == NodeKind::Dimension)
                .count(),
            2
        );

        assert_eq!(
            child_kinds
                .iter()
                .filter(|kind| **kind == NodeKind::Resource)
                .count(),
            1
        );
    }
    #[test]
    fn metadata_object_contains_form() {
        let root = create_edt_project();

        let graph = FileSystemEdtSemanticGraphBuilder
            .build_graph(root.path())
            .expect("graph must build");

        let document = graph
            .nodes_by_kind(NodeKind::Metadata(MetadataKind::Document))
            .into_iter()
            .next()
            .expect("document node must exist");

        let form = graph
            .nodes_by_kind(NodeKind::Form)
            .into_iter()
            .find(|node| node.name().as_str() == "DocumentForm")
            .expect("document form node must exist");

        let children = graph.outgoing_by_kind(document.id(), EdgeKind::Contains);

        assert!(children.iter().any(|edge| edge.target() == form.id()));
    }
}
