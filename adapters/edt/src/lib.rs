//! Adapter for reading `1C:EDT` project sources.

mod bsl_graph;
mod coverage;
mod metadata_object;
mod metadata_structure;
mod module_reader;
// The resolver is a production prerequisite for the later Reads emission task.
#[allow(dead_code)]
mod query_source_resolution;
mod role_rights;
mod subsystem_content;
// The parser is a production prerequisite for the later Writes resolution task.
#[allow(dead_code)]
mod writes;
// The resolver is a production prerequisite for the later Writes emission task.
#[allow(dead_code)]
mod writes_resolution;

pub use metadata_object::{
    EdtMetadataObjectDescriptor, EdtMetadataObjectError, EdtMetadataObjectReader,
    FileSystemEdtMetadataObjectReader,
};

pub use metadata_structure::{
    EdtMetadataChildDescriptor, EdtMetadataChildKind, EdtMetadataReferenceDescriptor,
    EdtMetadataReferenceRole, EdtMetadataStructureError, EdtMetadataStructureReader,
    FileSystemEdtMetadataStructureReader,
};

pub use module_reader::{
    EdtModuleDescriptor, EdtModuleError, EdtModuleKind, EdtModuleReader, FileSystemEdtModuleReader,
};

pub use role_rights::{
    EdtRoleObjectRights, EdtRoleRightDeclaration, EdtRoleRightsDescriptor, EdtRoleRightsError,
    EdtRoleRightsReader, EdtRoleRowRestriction, FileSystemEdtRoleRightsReader,
};

pub use subsystem_content::{
    EdtSubsystemContentDescriptor, EdtSubsystemContentError, EdtSubsystemContentReader,
    FileSystemEdtSubsystemContentReader,
};

pub use bsl_graph::{
    AnalyzedBslModule, EdtBslGraphError, add_configuration_module_symbols, add_module_symbols,
    analyze_module,
};
pub use coverage::{EdtSemanticCoverageRegistry, EdtSemanticCoverageReport};

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

use oneagent_graph::{
    AccessRight, Confidence, EdgeKind, FactOrigin, GraphEdge, NodeKind, ProducerId, Provenance,
    ResolutionError, ResolutionState, SemanticDiagnostic, SemanticDiagnosticCode,
    SemanticDiagnosticKind, SemanticDiagnosticSeverity, SemanticGraph, SemanticGraphBuildDiff,
    SemanticGraphReport, SemanticGraphValidationResult, SemanticGraphValidator, SemanticReference,
    SemanticReferenceOutcome, SemanticReferenceStatistics, StandardAttribute,
    StandardAttributeKind,
};
use oneagent_metadata::MetadataKind;
use std::collections::{BTreeMap, BTreeSet};

const EDT_GRAPH_PRODUCER: &str = "oneagent.edt.semantic-graph-builder";
const EDT_SUBSYSTEM_CONTENT_RESOLUTION_PRODUCER: &str = "oneagent.edt.subsystem-content-resolution";

/// Result of building an EDT semantic graph.
///
/// Recoverable semantic reference problems are returned as ordered diagnostics
/// while the graph contains every node and every edge that could be built
/// safely.
#[derive(Debug, Clone)]
pub struct EdtSemanticGraphBuildResult {
    graph: SemanticGraph,
    diagnostics: Vec<SemanticDiagnostic>,
    reference_statistics: SemanticReferenceStatistics,
}

impl EdtSemanticGraphBuildResult {
    /// Creates an EDT semantic graph build result.
    #[must_use]
    pub const fn new(graph: SemanticGraph, diagnostics: Vec<SemanticDiagnostic>) -> Self {
        Self {
            graph,
            diagnostics,
            reference_statistics: SemanticReferenceStatistics::new(),
        }
    }

    /// Creates an EDT semantic graph build result with reference statistics.
    #[must_use]
    pub const fn new_with_reference_statistics(
        graph: SemanticGraph,
        diagnostics: Vec<SemanticDiagnostic>,
        reference_statistics: SemanticReferenceStatistics,
    ) -> Self {
        Self {
            graph,
            diagnostics,
            reference_statistics,
        }
    }

    /// Returns the generated semantic graph.
    #[must_use]
    pub const fn graph(&self) -> &SemanticGraph {
        &self.graph
    }

    /// Consumes the result and returns the generated semantic graph.
    #[must_use]
    pub fn into_graph(self) -> SemanticGraph {
        self.graph
    }

    /// Returns ordered semantic diagnostics.
    #[must_use]
    pub fn diagnostics(&self) -> &[SemanticDiagnostic] {
        &self.diagnostics
    }

    /// Returns immutable semantic reference resolution statistics.
    #[must_use]
    pub const fn reference_statistics(&self) -> &SemanticReferenceStatistics {
        &self.reference_statistics
    }

    /// Builds a deterministic report for this EDT graph build result.
    ///
    /// The report combines graph metrics, recoverable semantic diagnostics and
    /// reference outcome counters captured during graph construction.
    #[must_use]
    pub fn report(&self) -> SemanticGraphReport {
        SemanticGraphReport::from_graph_diagnostics_and_references(
            &self.graph,
            &self.diagnostics,
            self.reference_statistics,
        )
    }

    /// Compares this EDT graph build result with a newer build result.
    #[must_use]
    pub fn diff(&self, current: &Self) -> SemanticGraphBuildDiff {
        SemanticGraphBuildDiff::between(
            &self.graph,
            &self.diagnostics,
            self.reference_statistics,
            &current.graph,
            &current.diagnostics,
            current.reference_statistics,
        )
    }

    /// Validates graph-level and build-level invariants for this EDT build result.
    ///
    /// Validation does not read EDT XML, does not rebuild the graph and does not
    /// rerun semantic resolution. Recoverable semantic diagnostics remain
    /// diagnostics and are not validation issues unless they expose an invalid
    /// build-result state.
    #[must_use]
    pub fn validate(&self) -> SemanticGraphValidationResult {
        SemanticGraphValidator::new().validate_build_result(
            &self.graph,
            &self.diagnostics,
            self.reference_statistics,
        )
    }

    /// Builds a deterministic Semantic Coverage Audit for this EDT build.
    ///
    /// Static graph and EDT support matrices remain separate from observed
    /// occurrence, build metrics and validation outcomes.
    #[must_use]
    pub fn coverage_report(&self) -> EdtSemanticCoverageReport {
        EdtSemanticCoverageReport::for_build_result(self)
    }
}

/// Builds an initial semantic graph from an EDT project.
pub trait EdtSemanticGraphBuilder {
    /// Builds a semantic graph and ordered diagnostics rooted at the EDT configuration.
    ///
    /// # Errors
    ///
    /// Returns an error for fatal project loading, parsing or graph invariant
    /// violations. Recoverable semantic reference problems are returned as
    /// diagnostics inside [`EdtSemanticGraphBuildResult`].
    fn build_graph_with_diagnostics(
        &self,
        project_root: &Path,
    ) -> Result<EdtSemanticGraphBuildResult, EdtGraphError>;

    /// Builds a semantic graph rooted at the EDT configuration.
    ///
    /// This compatibility convenience method discards recoverable diagnostics.
    ///
    /// # Errors
    ///
    /// Returns an error for fatal project loading, parsing or graph invariant
    /// violations.
    fn build_graph(&self, project_root: &Path) -> Result<SemanticGraph, EdtGraphError> {
        self.build_graph_with_diagnostics(project_root)
            .map(EdtSemanticGraphBuildResult::into_graph)
    }
}

/// Filesystem implementation of the EDT semantic graph builder.
#[derive(Debug, Default, Clone, Copy)]
pub struct FileSystemEdtSemanticGraphBuilder;

impl EdtSemanticGraphBuilder for FileSystemEdtSemanticGraphBuilder {
    fn build_graph_with_diagnostics(
        &self,
        project_root: &Path,
    ) -> Result<EdtSemanticGraphBuildResult, EdtGraphError> {
        let configuration = FileSystemEdtConfigurationLoader.load(project_root)?;
        let mut graph = SemanticGraph::new();
        let mut configuration_modules = Vec::new();
        let mut metadata_references = BTreeSet::new();
        let mut metadata_extensions = BTreeSet::new();
        let mut subsystem_content = BTreeSet::new();
        let mut role_rights = Vec::new();
        let mut diagnostics = BTreeSet::new();
        let mut reference_statistics = SemanticReferenceStatistics::new();

        let configuration_id = configuration.id().clone();
        let configuration_path = project_root.join(CONFIGURATION_RELATIVE_PATH);
        let configuration_source = source_id_from_path_fragment(
            &configuration_path,
            format!(
                "metadata_object={};fact=configuration",
                configuration_id.as_str()
            ),
            EdtGraphError::InvalidIdentifier,
        )?;
        insert_node(
            &mut graph,
            configuration_id.clone(),
            configuration.name().clone(),
            NodeKind::Metadata(MetadataKind::Configuration),
            parsed_provenance(configuration_source),
        );

        let source_root = project_root.join("src");
        if !source_root.is_dir() {
            return Ok(EdtSemanticGraphBuildResult::new(graph, Vec::new()));
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

            let collected = collect_top_level_metadata(
                project_root,
                &entry.path(),
                kind,
                &configuration_id,
                &mut graph,
            )?;
            configuration_modules.extend(collected.modules);
            metadata_references.extend(collected.references);
            metadata_extensions.extend(collected.extensions);
            subsystem_content.extend(collected.subsystem_content);
            role_rights.extend(collected.role_rights);
        }

        resolve_metadata_extensions(&mut graph, &metadata_extensions)?;

        emit_subsystem_includes(
            &mut graph,
            &subsystem_content,
            &mut diagnostics,
            &mut reference_statistics,
        )?;

        resolve_metadata_references(
            &mut graph,
            &metadata_references,
            &mut diagnostics,
            &mut reference_statistics,
        )?;

        emit_role_grants(
            &mut graph,
            &role_rights,
            &mut diagnostics,
            &mut reference_statistics,
        )?;

        bsl_graph::add_configuration_module_symbols_with_diagnostics_in_scope(
            &mut graph,
            &configuration_modules,
            query_source_resolution::WorkspaceResolutionScope::Complete,
            &mut diagnostics,
            &mut reference_statistics,
        )
        .map_err(EdtGraphError::Bsl)?;

        Ok(EdtSemanticGraphBuildResult::new_with_reference_statistics(
            graph,
            diagnostics.into_iter().collect(),
            reference_statistics,
        ))
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
        ("CommonCommands", MetadataKind::Command),
        ("CommonForms", MetadataKind::CommonForm),
        ("CommonTemplates", MetadataKind::Template),
        ("HTTPServices", MetadataKind::HttpService),
        ("WebServices", MetadataKind::WebService),
        ("XDTOPackages", MetadataKind::XdtoPackage),
        ("Subsystems", MetadataKind::Subsystem),
    ])
}

#[derive(Debug, Default)]
struct CollectedTopLevelMetadata {
    modules: Vec<EdtModuleDescriptor>,
    references: BTreeSet<PendingMetadataReference>,
    extensions: BTreeSet<PendingMetadataExtension>,
    subsystem_content: BTreeSet<PendingSubsystemContentObservation>,
    role_rights: Vec<EdtRoleRightsDescriptor>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct PendingMetadataReference {
    descriptor_path: PathBuf,
    metadata_object_id: EntityId,
    source_id: EntityId,
    role: EdtMetadataReferenceRole,
    target_kind: MetadataKind,
    target_name: EntityName,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct PendingMetadataExtension {
    descriptor_path: PathBuf,
    source_id: EntityId,
    source_kind: MetadataKind,
    target_id: EntityId,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct PendingSubsystemContentObservation {
    descriptor_path: PathBuf,
    subsystem_id: EntityId,
    subsystem_node_id: EntityId,
    raw_token: String,
    target: SubsystemContentTarget,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum SubsystemContentTarget {
    Malformed,
    Unsupported {
        prefix: String,
        deferred: bool,
    },
    Resolvable {
        metadata_kind: MetadataKind,
        local_name: EntityName,
    },
}

fn collect_top_level_metadata(
    project_root: &Path,
    directory: &Path,
    kind: MetadataKind,
    configuration_id: &EntityId,
    graph: &mut SemanticGraph,
) -> Result<CollectedTopLevelMetadata, EdtGraphError> {
    let mut collected = CollectedTopLevelMetadata::default();

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
        let object = collect_metadata_object(
            project_root,
            &object_directory,
            kind,
            configuration_id,
            graph,
        )?;

        collected.modules.extend(object.modules);
        collected.references.extend(object.references);
        collected.extensions.extend(object.extensions);
        collected.subsystem_content.extend(object.subsystem_content);
        collected.role_rights.extend(object.role_rights);
    }

    Ok(collected)
}

fn collect_metadata_object(
    project_root: &Path,
    object_directory: &Path,
    kind: MetadataKind,
    configuration_id: &EntityId,
    graph: &mut SemanticGraph,
) -> Result<CollectedTopLevelMetadata, EdtGraphError> {
    let metadata_reader = FileSystemEdtMetadataObjectReader;
    let module_reader = FileSystemEdtModuleReader;
    let structure_reader = FileSystemEdtMetadataStructureReader;
    let mut collected = CollectedTopLevelMetadata::default();

    let descriptor = metadata_reader
        .read(object_directory, kind)
        .map_err(EdtGraphError::MetadataObject)?;
    let descriptor_source = metadata_object_source_id(&descriptor)?;

    insert_node(
        graph,
        descriptor.id().clone(),
        descriptor.name().clone(),
        NodeKind::Metadata(descriptor.kind()),
        declared_provenance(descriptor_source),
    );

    if descriptor.kind() == MetadataKind::Role {
        insert_role_node(graph, &descriptor)?;
        collected.role_rights.push(
            FileSystemEdtRoleRightsReader
                .read(object_directory, descriptor.id())
                .map_err(EdtGraphError::RoleRights)?,
        );
    }

    if descriptor.kind() == MetadataKind::Subsystem {
        insert_subsystem_node(graph, &descriptor)?;
        collect_subsystem_content(project_root, &descriptor, &mut collected.subsystem_content)?;
    }

    if let Some(extension) = descriptor.extension() {
        collected.extensions.insert(PendingMetadataExtension {
            descriptor_path: descriptor.descriptor_path().to_path_buf(),
            source_id: descriptor.id().clone(),
            source_kind: descriptor.kind(),
            target_id: extension.extended_configuration_object_id().clone(),
        });
    }

    insert_standard_attribute_nodes(graph, &descriptor)?;

    insert_edge(
        graph,
        configuration_id.clone(),
        descriptor.id().clone(),
        EdgeKind::Contains,
        declared_provenance(contains_edge_source_id(
            descriptor.descriptor_path(),
            descriptor.id(),
            configuration_id,
            descriptor.id(),
        )?),
    )?;

    let children = structure_reader
        .read_children(&descriptor)
        .map_err(EdtGraphError::MetadataStructure)?;

    for child in &children {
        collect_metadata_child(graph, &descriptor, child, &mut collected.references)?;
    }

    for child in &children {
        collect_metadata_child_ownership(graph, &descriptor, child)?;
    }

    let modules = module_reader
        .read_modules(descriptor.id(), descriptor.name(), object_directory)
        .map_err(EdtGraphError::Module)?;

    for module in &modules {
        let module_source = module_source_id(&descriptor, module)?;

        insert_node(
            graph,
            module.id().clone(),
            module.name().clone(),
            NodeKind::Module,
            parsed_provenance(module_source.clone()),
        );

        insert_edge(
            graph,
            descriptor.id().clone(),
            module.id().clone(),
            EdgeKind::Contains,
            parsed_provenance(contains_edge_source_id(
                module.path(),
                descriptor.id(),
                descriptor.id(),
                module.id(),
            )?),
        )?;
    }

    collected.modules.extend(modules);

    Ok(collected)
}

fn collect_subsystem_content(
    project_root: &Path,
    descriptor: &EdtMetadataObjectDescriptor,
    observations: &mut BTreeSet<PendingSubsystemContentObservation>,
) -> Result<(), EdtGraphError> {
    let content = FileSystemEdtSubsystemContentReader
        .read(descriptor)
        .map_err(EdtGraphError::SubsystemContent)?;
    let descriptor_path = content
        .descriptor_path()
        .strip_prefix(project_root)
        .map_err(|_| EdtGraphError::SubsystemDescriptorOutsideProject {
            project_root: project_root.to_path_buf(),
            path: content.descriptor_path().to_path_buf(),
        })?
        .to_path_buf();
    let subsystem_node_id = subsystem_node_id(descriptor)?;

    for raw_token in content.raw_content() {
        observations.insert(PendingSubsystemContentObservation {
            descriptor_path: descriptor_path.clone(),
            subsystem_id: content.subsystem_id().clone(),
            subsystem_node_id: subsystem_node_id.clone(),
            raw_token: raw_token.clone(),
            target: normalize_subsystem_content_target(raw_token),
        });
    }

    Ok(())
}

fn normalize_subsystem_content_target(raw_token: &str) -> SubsystemContentTarget {
    let mut components = raw_token.split('.');
    let prefix = components.next().unwrap_or_default();
    let Some(local_name) = components.next() else {
        return SubsystemContentTarget::Malformed;
    };

    if prefix.is_empty() || local_name.is_empty() || components.next().is_some() {
        return SubsystemContentTarget::Malformed;
    }

    let metadata_kind = match prefix {
        "Catalog" => MetadataKind::Catalog,
        "Document" => MetadataKind::Document,
        "Enum" => MetadataKind::Enumeration,
        "CommonModule" => MetadataKind::CommonModule,
        "Report" => MetadataKind::Report,
        "DataProcessor" => MetadataKind::DataProcessor,
        "InformationRegister" => MetadataKind::InformationRegister,
        "AccumulationRegister" => MetadataKind::AccumulationRegister,
        "AccountingRegister" => MetadataKind::AccountingRegister,
        "CalculationRegister" => MetadataKind::CalculationRegister,
        "BusinessProcess" => MetadataKind::BusinessProcess,
        "Task" => MetadataKind::Task,
        "Role" => MetadataKind::Role,
        "CommonCommand" => MetadataKind::Command,
        "CommonForm" => MetadataKind::CommonForm,
        "CommonTemplate" => MetadataKind::Template,
        "HTTPService" => MetadataKind::HttpService,
        "WebService" => MetadataKind::WebService,
        "XDTOPackage" => MetadataKind::XdtoPackage,
        "Subsystem" => {
            return SubsystemContentTarget::Unsupported {
                prefix: prefix.to_owned(),
                deferred: true,
            };
        }
        _ => {
            return SubsystemContentTarget::Unsupported {
                prefix: prefix.to_owned(),
                deferred: false,
            };
        }
    };
    let Ok(local_name) = EntityName::new(local_name) else {
        return SubsystemContentTarget::Malformed;
    };

    SubsystemContentTarget::Resolvable {
        metadata_kind,
        local_name,
    }
}

fn insert_standard_attribute_nodes(
    graph: &mut SemanticGraph,
    descriptor: &EdtMetadataObjectDescriptor,
) -> Result<(), EdtGraphError> {
    for kind in standard_attribute_kinds(descriptor.kind()) {
        let attribute = StandardAttribute::new(
            descriptor.id().clone(),
            *kind,
            vec![declared_provenance(standard_attribute_source_id(
                descriptor, *kind,
            )?)],
        )
        .map_err(|_| EdtGraphError::InvalidIdentifier)?;

        graph
            .insert_standard_attribute(&attribute)
            .map_err(EdtGraphError::Graph)?;
    }

    Ok(())
}

fn insert_role_node(
    graph: &mut SemanticGraph,
    descriptor: &EdtMetadataObjectDescriptor,
) -> Result<(), EdtGraphError> {
    insert_node(
        graph,
        role_node_id(descriptor)?,
        descriptor.name().clone(),
        NodeKind::Role,
        declared_provenance(role_node_source_id(descriptor)?),
    );

    Ok(())
}

fn insert_subsystem_node(
    graph: &mut SemanticGraph,
    descriptor: &EdtMetadataObjectDescriptor,
) -> Result<(), EdtGraphError> {
    insert_node(
        graph,
        subsystem_node_id(descriptor)?,
        descriptor.name().clone(),
        NodeKind::Subsystem,
        declared_provenance(subsystem_node_source_id(descriptor)?),
    );

    Ok(())
}

const fn standard_attribute_kinds(kind: MetadataKind) -> &'static [StandardAttributeKind] {
    match kind {
        MetadataKind::Document => &[
            StandardAttributeKind::Ref,
            StandardAttributeKind::DeletionMark,
            StandardAttributeKind::Date,
            StandardAttributeKind::Number,
            StandardAttributeKind::Posted,
        ],
        _ => &[],
    }
}

fn collect_metadata_child(
    graph: &mut SemanticGraph,
    descriptor: &EdtMetadataObjectDescriptor,
    child: &EdtMetadataChildDescriptor,
    references: &mut BTreeSet<PendingMetadataReference>,
) -> Result<(), EdtGraphError> {
    let child_source = metadata_child_source_id(descriptor, child)?;

    let child_node_kind = semantic_child_node_kind(descriptor.kind(), child.kind());

    insert_node(
        graph,
        child.id().clone(),
        child.name().clone(),
        child_node_kind,
        declared_provenance(child_source),
    );

    if is_depends_on_metadata_member_source(child_node_kind) {
        for reference in child.references() {
            references.insert(PendingMetadataReference {
                descriptor_path: descriptor.descriptor_path().to_path_buf(),
                metadata_object_id: descriptor.id().clone(),
                source_id: child.id().clone(),
                role: reference.role(),
                target_kind: reference.target_kind(),
                target_name: reference.target_name().clone(),
            });
        }
    }

    Ok(())
}

fn collect_metadata_child_ownership(
    graph: &mut SemanticGraph,
    descriptor: &EdtMetadataObjectDescriptor,
    child: &EdtMetadataChildDescriptor,
) -> Result<(), EdtGraphError> {
    insert_edge(
        graph,
        child.parent_id().clone(),
        child.id().clone(),
        EdgeKind::Contains,
        declared_provenance(contains_edge_source_id(
            descriptor.descriptor_path(),
            descriptor.id(),
            child.parent_id(),
            child.id(),
        )?),
    )?;

    Ok(())
}

const fn is_depends_on_metadata_member_source(kind: NodeKind) -> bool {
    matches!(
        kind,
        NodeKind::Attribute | NodeKind::Dimension | NodeKind::Resource
    )
}

const fn semantic_child_node_kind(
    owner_kind: MetadataKind,
    child_kind: EdtMetadataChildKind,
) -> NodeKind {
    if matches!(
        (owner_kind, child_kind),
        (
            MetadataKind::AccountingRegister,
            EdtMetadataChildKind::Resource
        )
    ) {
        NodeKind::Measure
    } else {
        child_kind.node_kind()
    }
}

fn resolve_metadata_references(
    graph: &mut SemanticGraph,
    references: &BTreeSet<PendingMetadataReference>,
    diagnostics: &mut BTreeSet<SemanticDiagnostic>,
    reference_statistics: &mut SemanticReferenceStatistics,
) -> Result<(), EdtGraphError> {
    let resolved_references = {
        let index = graph.resolution_index();
        let mut resolved_references = Vec::new();

        for reference in references {
            let reference_context = SemanticReference::Name(reference.target_name.clone());
            match index.resolve_name_of_kind(
                &reference.target_name,
                NodeKind::Metadata(reference.target_kind),
            ) {
                Ok(target) => {
                    reference_statistics.record(SemanticReferenceOutcome::Resolved, true);
                    resolved_references.push((reference.clone(), target.id().clone()));
                }
                Err(error) => {
                    reference_statistics.record(
                        SemanticReferenceOutcome::from_resolution_error(&error),
                        true,
                    );
                    diagnostics.insert(metadata_reference_diagnostic(
                        reference,
                        error,
                        reference_context,
                    )?);
                }
            }
        }

        resolved_references
    };

    for (reference, target_id) in resolved_references {
        let reference_source = metadata_reference_source_id(&reference)?;
        insert_edge(
            graph,
            reference.source_id.clone(),
            target_id.clone(),
            EdgeKind::References,
            resolved_provenance(reference_source),
        )?;
        insert_edge(
            graph,
            reference.source_id.clone(),
            target_id.clone(),
            EdgeKind::DependsOn,
            derived_provenance(metadata_dependency_source_id(&reference, &target_id)?),
        )?;
    }

    Ok(())
}

fn emit_subsystem_includes(
    graph: &mut SemanticGraph,
    observations: &BTreeSet<PendingSubsystemContentObservation>,
    diagnostics: &mut BTreeSet<SemanticDiagnostic>,
    reference_statistics: &mut SemanticReferenceStatistics,
) -> Result<(), EdtGraphError> {
    let resolved_sources = {
        let index = graph.resolution_index();
        let mut resolved_sources = BTreeMap::<(EntityId, EntityId), BTreeSet<EntityId>>::new();

        for observation in observations {
            let source = graph.node(&observation.subsystem_node_id).ok_or_else(|| {
                EdtGraphError::InvalidSubsystemSource {
                    subsystem_id: observation.subsystem_id.clone(),
                    subsystem_node_id: observation.subsystem_node_id.clone(),
                    actual_kind: None,
                }
            })?;
            if source.kind() != NodeKind::Subsystem {
                return Err(EdtGraphError::InvalidSubsystemSource {
                    subsystem_id: observation.subsystem_id.clone(),
                    subsystem_node_id: observation.subsystem_node_id.clone(),
                    actual_kind: Some(source.kind()),
                });
            }

            match &observation.target {
                SubsystemContentTarget::Malformed => {
                    reference_statistics.record(SemanticReferenceOutcome::MalformedFormat, true);
                    diagnostics.insert(subsystem_content_input_diagnostic(
                        observation,
                        SemanticDiagnosticCode::ReferenceMalformedFormat,
                        SemanticDiagnosticKind::MalformedReferenceFormat,
                        "EDT Subsystem content reference must contain exactly one `.` separator with non-empty components",
                    )?);
                }
                SubsystemContentTarget::Unsupported { prefix, deferred } => {
                    reference_statistics.record(SemanticReferenceOutcome::UnsupportedPrefix, true);
                    let message = if *deferred {
                        format!(
                            "EDT Subsystem content prefix `{prefix}` is recognized but deferred"
                        )
                    } else {
                        format!("EDT Subsystem content prefix `{prefix}` is unsupported")
                    };
                    diagnostics.insert(subsystem_content_input_diagnostic(
                        observation,
                        SemanticDiagnosticCode::ReferenceUnsupportedPrefix,
                        SemanticDiagnosticKind::UnsupportedReferencePrefix,
                        message,
                    )?);
                }
                SubsystemContentTarget::Resolvable {
                    metadata_kind,
                    local_name,
                } => match index
                    .resolve_name_of_kind(local_name, NodeKind::Metadata(*metadata_kind))
                {
                    Ok(target) => {
                        reference_statistics.record(SemanticReferenceOutcome::Resolved, true);
                        let source_id = subsystem_content_source_id(
                            observation,
                            "resolved",
                            Some(target.id()),
                        )?;
                        resolved_sources
                            .entry((observation.subsystem_node_id.clone(), target.id().clone()))
                            .or_default()
                            .insert(source_id);
                    }
                    Err(error) => {
                        reference_statistics.record(
                            SemanticReferenceOutcome::from_resolution_error(&error),
                            true,
                        );
                        diagnostics.insert(subsystem_content_resolution_diagnostic(
                            observation,
                            *metadata_kind,
                            error,
                        )?);
                    }
                },
            }
        }

        resolved_sources
    };

    for ((source_id, target_id), sources) in resolved_sources {
        let provenance = sources
            .into_iter()
            .map(|source| subsystem_content_provenance(source, ResolutionState::Resolved))
            .collect();
        graph
            .insert_edge(GraphEdge::new_with_provenance(
                source_id,
                target_id,
                EdgeKind::Includes,
                provenance,
            ))
            .map_err(EdtGraphError::Graph)?;
    }

    Ok(())
}

fn subsystem_content_input_diagnostic(
    observation: &PendingSubsystemContentObservation,
    code: SemanticDiagnosticCode,
    kind: SemanticDiagnosticKind,
    message: impl Into<String>,
) -> Result<SemanticDiagnostic, EdtGraphError> {
    Ok(SemanticDiagnostic::new(
        code,
        SemanticDiagnosticSeverity::Error,
        kind,
        message,
        SemanticReference::Raw(observation.raw_token.clone()),
    )
    .with_source_node(observation.subsystem_node_id.clone())
    .with_provenance(vec![subsystem_content_provenance(
        subsystem_content_source_id(observation, "rejected", None)?,
        ResolutionState::Unresolved,
    )]))
}

fn subsystem_content_resolution_diagnostic(
    observation: &PendingSubsystemContentObservation,
    metadata_kind: MetadataKind,
    error: ResolutionError,
) -> Result<SemanticDiagnostic, EdtGraphError> {
    let resolution = if matches!(&error, ResolutionError::AmbiguousTarget { .. }) {
        ResolutionState::Ambiguous
    } else {
        ResolutionState::Unresolved
    };

    Ok(SemanticDiagnostic::from_resolution_error_with_reference(
        error,
        Some(SemanticReference::Raw(observation.raw_token.clone())),
    )
    .with_source_node(observation.subsystem_node_id.clone())
    .with_expected_kinds(vec![NodeKind::Metadata(metadata_kind)])
    .with_provenance(vec![subsystem_content_provenance(
        subsystem_content_source_id(observation, "unresolved", None)?,
        resolution,
    )]))
}

fn resolve_metadata_extensions(
    graph: &mut SemanticGraph,
    extensions: &BTreeSet<PendingMetadataExtension>,
) -> Result<(), EdtGraphError> {
    let resolved_extensions = extensions
        .iter()
        .filter_map(|extension| {
            if extension.source_id == extension.target_id {
                return None;
            }

            let target = graph.node(&extension.target_id)?;
            if target.kind() != NodeKind::Metadata(extension.source_kind) {
                return None;
            }

            Some((extension.clone(), target.id().clone()))
        })
        .collect::<Vec<_>>();

    for (extension, target_id) in resolved_extensions {
        insert_edge(
            graph,
            extension.source_id.clone(),
            target_id.clone(),
            EdgeKind::Extends,
            resolved_provenance(metadata_extension_source_id(&extension, &target_id)?),
        )?;
    }

    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ResolvedRoleGrantObservation {
    rights_path: PathBuf,
    role_id: EntityId,
    role_node_id: EntityId,
    declared_resource_name: EntityName,
    resource_id: EntityId,
    right_id: EntityId,
}

fn emit_role_grants(
    graph: &mut SemanticGraph,
    role_rights: &[EdtRoleRightsDescriptor],
    diagnostics: &mut BTreeSet<SemanticDiagnostic>,
    reference_statistics: &mut SemanticReferenceStatistics,
) -> Result<(), EdtGraphError> {
    let observations =
        resolve_role_grant_observations(graph, role_rights, diagnostics, reference_statistics)?;

    insert_resolved_role_grants(graph, &observations)
}

fn resolve_role_grant_observations(
    graph: &SemanticGraph,
    role_rights: &[EdtRoleRightsDescriptor],
    diagnostics: &mut BTreeSet<SemanticDiagnostic>,
    reference_statistics: &mut SemanticReferenceStatistics,
) -> Result<BTreeSet<ResolvedRoleGrantObservation>, EdtGraphError> {
    let index = graph.resolution_index();
    let mut observations = BTreeSet::new();

    for descriptor in role_rights {
        let role_node_id = role_node_id_from_metadata_id(descriptor.role_id())?;

        for object in descriptor.objects() {
            let Some((target_kind, target_name)) =
                protected_resource_reference(object.resource_name())?
            else {
                continue;
            };

            for right in object.rights().iter().filter(|right| right.value()) {
                let right_id = EntityId::new(right.name().as_str())
                    .map_err(|_| EdtGraphError::InvalidIdentifier)?;
                let reference_context = SemanticReference::Name(object.resource_name().clone());

                match index.resolve_name_of_kind(&target_name, NodeKind::Metadata(target_kind)) {
                    Ok(target) => {
                        reference_statistics.record(SemanticReferenceOutcome::Resolved, true);
                        observations.insert(ResolvedRoleGrantObservation {
                            rights_path: descriptor.source_path().to_path_buf(),
                            role_id: descriptor.role_id().clone(),
                            role_node_id: role_node_id.clone(),
                            declared_resource_name: object.resource_name().clone(),
                            resource_id: target.id().clone(),
                            right_id,
                        });
                    }
                    Err(error) => {
                        reference_statistics.record(
                            SemanticReferenceOutcome::from_resolution_error(&error),
                            true,
                        );
                        diagnostics.insert(role_grant_diagnostic(
                            descriptor,
                            &role_node_id,
                            object.resource_name(),
                            &right_id,
                            target_kind,
                            error,
                            reference_context,
                        )?);
                    }
                }
            }
        }
    }

    Ok(observations)
}

fn insert_resolved_role_grants(
    graph: &mut SemanticGraph,
    observations: &BTreeSet<ResolvedRoleGrantObservation>,
) -> Result<(), EdtGraphError> {
    let mut access_right_sources = BTreeMap::<(EntityId, EntityId), BTreeSet<EntityId>>::new();
    let mut reference_sources = BTreeMap::<(EntityId, EntityId), BTreeSet<EntityId>>::new();
    let mut grant_sources = BTreeMap::<(EntityId, EntityId, EntityId), BTreeSet<EntityId>>::new();

    for observation in observations {
        let access_key = (
            observation.resource_id.clone(),
            observation.right_id.clone(),
        );
        access_right_sources
            .entry(access_key.clone())
            .or_default()
            .insert(role_grant_source_id(observation, "fact=access_right")?);
        reference_sources
            .entry(access_key.clone())
            .or_default()
            .insert(role_grant_source_id(observation, "edge=references")?);
        grant_sources
            .entry((observation.role_node_id.clone(), access_key.0, access_key.1))
            .or_default()
            .insert(role_grant_source_id(observation, "edge=grants")?);
    }

    let mut access_right_ids = BTreeMap::<(EntityId, EntityId), EntityId>::new();

    for ((resource_id, right_id), sources) in access_right_sources {
        let provenance = sources.into_iter().map(resolved_provenance).collect();
        let access_right = AccessRight::new(resource_id.clone(), right_id.clone(), provenance)
            .map_err(|_| EdtGraphError::InvalidIdentifier)?;
        let access_right_id = access_right.id().clone();
        graph.insert_access_right(&access_right);
        access_right_ids.insert((resource_id, right_id), access_right_id);
    }

    for ((resource_id, right_id), sources) in reference_sources {
        let access_right_id = access_right_ids
            .get(&(resource_id.clone(), right_id))
            .expect("aggregated access right must exist")
            .clone();
        let provenance = sources.into_iter().map(resolved_provenance).collect();
        graph
            .insert_edge(GraphEdge::new_with_provenance(
                access_right_id,
                resource_id,
                EdgeKind::References,
                provenance,
            ))
            .map_err(EdtGraphError::Graph)?;
    }

    for ((role_node_id, resource_id, right_id), sources) in grant_sources {
        let access_right_id = access_right_ids
            .get(&(resource_id, right_id))
            .expect("aggregated access right must exist")
            .clone();
        let provenance = sources.into_iter().map(resolved_provenance).collect();
        graph
            .insert_edge(GraphEdge::new_with_provenance(
                role_node_id,
                access_right_id,
                EdgeKind::Grants,
                provenance,
            ))
            .map_err(EdtGraphError::Graph)?;
    }

    Ok(())
}

fn protected_resource_reference(
    qualified_name: &EntityName,
) -> Result<Option<(MetadataKind, EntityName)>, EdtGraphError> {
    let Some((prefix, local_name)) = qualified_name.as_str().split_once('.') else {
        return Ok(None);
    };
    if local_name.is_empty() {
        return Ok(None);
    }

    let kind = match prefix {
        "Configuration" => MetadataKind::Configuration,
        "Catalog" => MetadataKind::Catalog,
        "Document" => MetadataKind::Document,
        "InformationRegister" => MetadataKind::InformationRegister,
        "AccumulationRegister" => MetadataKind::AccumulationRegister,
        _ => return Ok(None),
    };
    let name = EntityName::new(local_name).map_err(|_| EdtGraphError::InvalidName)?;

    Ok(Some((kind, name)))
}

fn role_grant_diagnostic(
    descriptor: &EdtRoleRightsDescriptor,
    role_node_id: &EntityId,
    declared_resource_name: &EntityName,
    right_id: &EntityId,
    target_kind: MetadataKind,
    error: ResolutionError,
    reference_context: SemanticReference,
) -> Result<SemanticDiagnostic, EdtGraphError> {
    let source = role_grant_source_id(
        &ResolvedRoleGrantObservation {
            rights_path: descriptor.source_path().to_path_buf(),
            role_id: descriptor.role_id().clone(),
            role_node_id: role_node_id.clone(),
            declared_resource_name: declared_resource_name.clone(),
            resource_id: EntityId::new("unresolved")
                .map_err(|_| EdtGraphError::InvalidIdentifier)?,
            right_id: right_id.clone(),
        },
        "fact=grant_resolution",
    )?;

    Ok(
        SemanticDiagnostic::from_resolution_error_with_reference(error, Some(reference_context))
            .with_source_node(role_node_id.clone())
            .with_expected_kinds(vec![NodeKind::Metadata(target_kind)])
            .with_provenance(vec![resolved_provenance(source)]),
    )
}

fn role_grant_source_id(
    observation: &ResolvedRoleGrantObservation,
    fact_kind: &str,
) -> Result<EntityId, EdtGraphError> {
    source_id_from_path_fragment(
        &observation.rights_path,
        format!(
            "role_metadata={};role={};protected_resource={};resolved_resource={};right={};value=true;accepted_explicit_allow=true;{}",
            observation.role_id.as_str(),
            observation.role_node_id.as_str(),
            observation.declared_resource_name.as_str(),
            observation.resource_id.as_str(),
            observation.right_id.as_str(),
            fact_kind,
        ),
        EdtGraphError::InvalidIdentifier,
    )
}

fn metadata_reference_diagnostic(
    reference: &PendingMetadataReference,
    error: ResolutionError,
    reference_context: SemanticReference,
) -> Result<SemanticDiagnostic, EdtGraphError> {
    Ok(
        SemanticDiagnostic::from_resolution_error_with_reference(error, Some(reference_context))
            .with_source_node(reference.source_id.clone())
            .with_expected_kinds(vec![NodeKind::Metadata(reference.target_kind)])
            .with_provenance(vec![resolved_provenance(metadata_reference_source_id(
                reference,
            )?)]),
    )
}

fn insert_node(
    graph: &mut SemanticGraph,
    id: EntityId,
    name: EntityName,
    kind: NodeKind,
    provenance: Provenance,
) {
    graph.insert_node_with_provenance(id, name, kind, provenance);
}

fn insert_edge(
    graph: &mut SemanticGraph,
    source: EntityId,
    target: EntityId,
    kind: EdgeKind,
    provenance: Provenance,
) -> Result<bool, EdtGraphError> {
    graph
        .insert_edge_with_provenance(source, target, kind, provenance)
        .map_err(EdtGraphError::Graph)
}

fn parsed_provenance(source: EntityId) -> Provenance {
    graph_provenance(
        source,
        FactOrigin::Parsed,
        Confidence::Exact,
        ResolutionState::NotApplicable,
    )
}

fn declared_provenance(source: EntityId) -> Provenance {
    graph_provenance(
        source,
        FactOrigin::Declared,
        Confidence::Exact,
        ResolutionState::NotApplicable,
    )
}

fn resolved_provenance(source: EntityId) -> Provenance {
    graph_provenance(
        source,
        FactOrigin::Resolved,
        Confidence::High,
        ResolutionState::Resolved,
    )
}

fn derived_provenance(source: EntityId) -> Provenance {
    graph_provenance(
        source,
        FactOrigin::Derived,
        Confidence::High,
        ResolutionState::Resolved,
    )
}

fn graph_provenance(
    source: EntityId,
    origin: FactOrigin,
    confidence: Confidence,
    resolution: ResolutionState,
) -> Provenance {
    Provenance::new(
        Some(source),
        ProducerId::new(EDT_GRAPH_PRODUCER),
        origin,
        confidence,
        resolution,
    )
}

fn subsystem_content_provenance(source: EntityId, resolution: ResolutionState) -> Provenance {
    Provenance::new(
        Some(source),
        ProducerId::new(EDT_SUBSYSTEM_CONTENT_RESOLUTION_PRODUCER),
        FactOrigin::Resolved,
        Confidence::Exact,
        resolution,
    )
}

fn subsystem_content_source_id(
    observation: &PendingSubsystemContentObservation,
    outcome: &str,
    resolved_target: Option<&EntityId>,
) -> Result<EntityId, EdtGraphError> {
    let mut fields = vec![
        encoded_source_field("stage", "subsystem_content_resolution"),
        encoded_source_field("subsystem_metadata_uuid", observation.subsystem_id.as_str()),
        encoded_source_field("subsystem_node", observation.subsystem_node_id.as_str()),
        encoded_source_field("field", "mdclass:Subsystem/content"),
        encoded_source_field("raw", &observation.raw_token),
    ];

    match &observation.target {
        SubsystemContentTarget::Malformed => {
            fields.push(encoded_source_field("format", "malformed"));
        }
        SubsystemContentTarget::Unsupported { prefix, deferred } => {
            fields.push(encoded_source_field("prefix", prefix));
            fields.push(encoded_source_field(
                "support",
                if *deferred { "deferred" } else { "unsupported" },
            ));
        }
        SubsystemContentTarget::Resolvable {
            metadata_kind,
            local_name,
        } => {
            fields.push(encoded_source_field("target_kind", metadata_kind.as_str()));
            fields.push(encoded_source_field("target_name", local_name.as_str()));
        }
    }
    if let Some(target) = resolved_target {
        fields.push(encoded_source_field("resolved_target", target.as_str()));
    }
    fields.push(encoded_source_field("outcome", outcome));

    source_id_from_path_fragment(
        &observation.descriptor_path,
        fields.join(";"),
        EdtGraphError::InvalidIdentifier,
    )
}

fn encoded_source_field(name: &str, value: &str) -> String {
    format!("{name}#{}:{value}", value.len())
}

fn metadata_object_source_id(
    descriptor: &EdtMetadataObjectDescriptor,
) -> Result<EntityId, EdtGraphError> {
    source_id_from_path_fragment(
        descriptor.descriptor_path(),
        format!(
            "metadata_object={};fact=metadata_object",
            descriptor.id().as_str()
        ),
        EdtGraphError::InvalidIdentifier,
    )
}

fn role_node_id(descriptor: &EdtMetadataObjectDescriptor) -> Result<EntityId, EdtGraphError> {
    role_node_id_from_metadata_id(descriptor.id())
}

fn role_node_id_from_metadata_id(metadata_id: &EntityId) -> Result<EntityId, EdtGraphError> {
    EntityId::new(format!("{}:role", metadata_id.as_str()))
        .map_err(|_| EdtGraphError::InvalidIdentifier)
}

fn role_node_source_id(
    descriptor: &EdtMetadataObjectDescriptor,
) -> Result<EntityId, EdtGraphError> {
    source_id_from_path_fragment(
        descriptor.descriptor_path(),
        format!(
            "metadata_object={};fact=role_node",
            descriptor.id().as_str()
        ),
        EdtGraphError::InvalidIdentifier,
    )
}

fn subsystem_node_id(descriptor: &EdtMetadataObjectDescriptor) -> Result<EntityId, EdtGraphError> {
    EntityId::new(format!("{}:subsystem", descriptor.id().as_str()))
        .map_err(|_| EdtGraphError::InvalidIdentifier)
}

fn subsystem_node_source_id(
    descriptor: &EdtMetadataObjectDescriptor,
) -> Result<EntityId, EdtGraphError> {
    source_id_from_path_fragment(
        descriptor.descriptor_path(),
        format!(
            "metadata_object={};fact=subsystem_node",
            descriptor.id().as_str()
        ),
        EdtGraphError::InvalidIdentifier,
    )
}

fn standard_attribute_source_id(
    descriptor: &EdtMetadataObjectDescriptor,
    kind: StandardAttributeKind,
) -> Result<EntityId, EdtGraphError> {
    source_id_from_path_fragment(
        descriptor.descriptor_path(),
        format!(
            "metadata_object={};member=standard_attribute:{}",
            descriptor.id().as_str(),
            kind.as_str()
        ),
        EdtGraphError::InvalidIdentifier,
    )
}

fn metadata_child_source_id(
    descriptor: &EdtMetadataObjectDescriptor,
    child: &EdtMetadataChildDescriptor,
) -> Result<EntityId, EdtGraphError> {
    source_id_from_path_fragment(
        descriptor.descriptor_path(),
        format!(
            "metadata_object={};member={}:{}",
            descriptor.id().as_str(),
            child.kind().as_str(),
            child.id().as_str()
        ),
        EdtGraphError::InvalidIdentifier,
    )
}

fn module_source_id(
    descriptor: &EdtMetadataObjectDescriptor,
    module: &EdtModuleDescriptor,
) -> Result<EntityId, EdtGraphError> {
    source_id_from_path_fragment(
        module.path(),
        format!(
            "metadata_object={};module={}",
            descriptor.id().as_str(),
            module.id().as_str()
        ),
        EdtGraphError::InvalidIdentifier,
    )
}

fn contains_edge_source_id(
    path: &Path,
    metadata_object_id: &EntityId,
    source: &EntityId,
    target: &EntityId,
) -> Result<EntityId, EdtGraphError> {
    source_id_from_path_fragment(
        path,
        format!(
            "metadata_object={};edge=contains;source={};target={}",
            metadata_object_id.as_str(),
            source.as_str(),
            target.as_str()
        ),
        EdtGraphError::InvalidIdentifier,
    )
}

fn metadata_reference_source_id(
    reference: &PendingMetadataReference,
) -> Result<EntityId, EdtGraphError> {
    source_id_from_path_fragment(
        &reference.descriptor_path,
        format!(
            "metadata_object={};edge=references;source={};role={};target_kind={};target_name={}",
            reference.metadata_object_id.as_str(),
            reference.source_id.as_str(),
            reference.role.as_str(),
            reference.target_kind.as_str(),
            reference.target_name.as_str()
        ),
        EdtGraphError::InvalidIdentifier,
    )
}

fn metadata_dependency_source_id(
    reference: &PendingMetadataReference,
    target_id: &EntityId,
) -> Result<EntityId, EdtGraphError> {
    source_id_from_path_fragment(
        &reference.descriptor_path,
        format!(
            "metadata_object={};edge=depends_on;origin=metadata_member_type_reference;source={};role={};target_kind={};target_name={};target={}",
            reference.metadata_object_id.as_str(),
            reference.source_id.as_str(),
            reference.role.as_str(),
            reference.target_kind.as_str(),
            reference.target_name.as_str(),
            target_id.as_str()
        ),
        EdtGraphError::InvalidIdentifier,
    )
}

fn metadata_extension_source_id(
    extension: &PendingMetadataExtension,
    target_id: &EntityId,
) -> Result<EntityId, EdtGraphError> {
    source_id_from_path_fragment(
        &extension.descriptor_path,
        format!(
            "metadata_object={};edge=extends;origin=metadata_object_extension;source={};target_kind={};declared_target={};target={}",
            extension.source_id.as_str(),
            extension.source_id.as_str(),
            extension.source_kind.as_str(),
            extension.target_id.as_str(),
            target_id.as_str()
        ),
        EdtGraphError::InvalidIdentifier,
    )
}

fn source_id_from_path_fragment(
    path: &Path,
    fragment: impl AsRef<str>,
    error: EdtGraphError,
) -> Result<EntityId, EdtGraphError> {
    EntityId::new(format!(
        "{}#{}",
        path.to_string_lossy().replace('\\', "/"),
        fragment.as_ref()
    ))
    .map_err(|_| error)
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

    /// Direct Subsystem content declarations could not be read.
    SubsystemContent(EdtSubsystemContentError),

    /// A discovered Subsystem descriptor is outside the project root.
    SubsystemDescriptorOutsideProject {
        /// EDT project root.
        project_root: PathBuf,
        /// Discovered Subsystem descriptor path.
        path: PathBuf,
    },

    /// The flat Subsystem source required for Includes emission is invalid.
    InvalidSubsystemSource {
        /// Declaring Subsystem metadata object identifier.
        subsystem_id: EntityId,
        /// Expected flat Subsystem node identifier.
        subsystem_node_id: EntityId,
        /// Actual node kind, or `None` when the node is missing.
        actual_kind: Option<NodeKind>,
    },

    /// A role-right artifact could not be read.
    RoleRights(EdtRoleRightsError),

    /// Semantic graph validation failed.
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
            Self::SubsystemContent(error) => {
                write!(formatter, "failed to read EDT Subsystem content: {error}")
            }
            Self::SubsystemDescriptorOutsideProject { project_root, path } => write!(
                formatter,
                "EDT Subsystem descriptor {} is outside project root {}",
                path.display(),
                project_root.display()
            ),
            Self::InvalidSubsystemSource {
                subsystem_id,
                subsystem_node_id,
                actual_kind,
            } => write!(
                formatter,
                "flat Subsystem source `{subsystem_node_id}` for metadata object `{subsystem_id}` is invalid; actual kind: {actual_kind:?}"
            ),
            Self::RoleRights(error) => {
                write!(formatter, "failed to read EDT role rights: {error}")
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
            Self::SubsystemContent(error) => Some(error),
            Self::RoleRights(error) => Some(error),
            Self::Graph(error) => Some(error),
            Self::InvalidIdentifier
            | Self::InvalidName
            | Self::SubsystemDescriptorOutsideProject { .. }
            | Self::InvalidSubsystemSource { .. } => None,
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
    use oneagent_common::{EntityId, EntityName};
    use oneagent_graph::{
        EdgeKind, FactOrigin, GraphEdge, GraphNode, NodeId, NodeKind, ResolutionError,
        ResolutionState, SemanticCoverageCapabilityId, SemanticCoverageStatus,
        SemanticDiagnosticCode, SemanticDiagnosticKind, SemanticDiagnosticSeverity, SemanticGraph,
        SemanticReference, SemanticReferenceCapability, SemanticReferenceStatistics,
    };
    use oneagent_metadata::MetadataKind;
    use std::collections::BTreeSet;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::tempdir;

    use super::{
        EdtGraphError, EdtSemanticGraphBuildResult, EdtSemanticGraphBuilder,
        FileSystemEdtSemanticGraphBuilder, PendingSubsystemContentObservation,
        SubsystemContentTarget, emit_subsystem_includes, normalize_subsystem_content_target,
    };

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
        <type>
            <types>CatalogRef.Products</types>
            <types>CatalogRef.Products</types>
        </type>
    </attributes>

    <attributes uuid="aaaaaaaa-2222-2222-2222-222222222222">
        <name>Warehouse</name>
        <type>
            <types>CatalogRef.Products</types>
        </type>
    </attributes>

    <tabularSections uuid="aaaaaaaa-3333-3333-3333-333333333333">
        <name>Goods</name>
    </tabularSections>

    <forms uuid="aaaaaaaa-4444-4444-4444-444444444444">
        <name>DocumentForm</name>
    </forms>

        <commands uuid="aaaaaaaa-5555-5555-5555-555555555555">
        <name>PostAndClose</name>
    </commands>
    
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

    const COMMON_COMMAND_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<mdclass:CommonCommand
    xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass"
    uuid="cccccccc-1111-2222-3333-444444444444">
    <name>RefreshData</name>
    <synonym>
        <key>en</key>
        <content>Refresh data</content>
    </synonym>
</mdclass:CommonCommand>
"#;

    const COMMON_TEMPLATE_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<mdclass:CommonTemplate
    xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass"
    uuid="dddddddd-aaaa-bbbb-cccc-111111111111">
    <name>Invoice</name>
    <synonym>
        <key>en</key>
        <content>Invoice</content>
    </synonym>
    <templateType>SpreadsheetDocument</templateType>
</mdclass:CommonTemplate>
"#;

    const ROLE_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<mdclass:Role
    xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass"
    uuid="eeeeeeee-1111-2222-3333-444444444444">
    <name>SalesManager</name>
</mdclass:Role>
"#;

    const DEFAULT_ROLE_RIGHTS_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<Rights xmlns="http://v8.1c.ru/8.2/roles">
    <setForNewObjects>false</setForNewObjects>
    <setForAttributesByDefault>false</setForAttributesByDefault>
    <independentRightsOfChildObjects>false</independentRightsOfChildObjects>
</Rights>
"#;

    const SUBSYSTEM_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<mdclass:Subsystem
    xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass"
    uuid="ffffffff-1111-2222-3333-444444444444">
    <name>SalesSubsystem</name>
</mdclass:Subsystem>
"#;

    const ACCUMULATION_REGISTER_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<mdclass:AccumulationRegister
    xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass"
    uuid="44444444-4444-4444-4444-444444444444">
    <name>StockBalance</name>

    <dimensions uuid="55555555-5555-5555-5555-555555555555">
        <name>Product</name>
        <type>
            <types>CatalogRef.Products</types>
        </type>
    </dimensions>

    <dimensions uuid="66666666-6666-6666-6666-666666666666">
        <name>Warehouse</name>
    </dimensions>

    <resources uuid="77777777-7777-7777-7777-777777777777">
        <name>Quantity</name>
        <type>
            <types>DocumentRef.Sales</types>
        </type>
    </resources>
</mdclass:AccumulationRegister>
"#;

    const ACCOUNTING_REGISTER_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<mdclass:AccountingRegister
    xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass"
    uuid="abababab-1111-2222-3333-444444444444">
    <name>GeneralLedger</name>

    <resources uuid="cdcdcdcd-1111-2222-3333-444444444444">
        <name>Amount</name>
    </resources>
</mdclass:AccountingRegister>
"#;

    #[test]
    fn subsystem_content_normalization_uses_the_explicit_first_slice_allowlist() {
        let mappings = [
            ("Catalog", MetadataKind::Catalog),
            ("Document", MetadataKind::Document),
            ("Enum", MetadataKind::Enumeration),
            ("CommonModule", MetadataKind::CommonModule),
            ("Report", MetadataKind::Report),
            ("DataProcessor", MetadataKind::DataProcessor),
            ("InformationRegister", MetadataKind::InformationRegister),
            ("AccumulationRegister", MetadataKind::AccumulationRegister),
            ("AccountingRegister", MetadataKind::AccountingRegister),
            ("CalculationRegister", MetadataKind::CalculationRegister),
            ("BusinessProcess", MetadataKind::BusinessProcess),
            ("Task", MetadataKind::Task),
            ("Role", MetadataKind::Role),
            ("CommonCommand", MetadataKind::Command),
            ("CommonForm", MetadataKind::CommonForm),
            ("CommonTemplate", MetadataKind::Template),
            ("HTTPService", MetadataKind::HttpService),
            ("WebService", MetadataKind::WebService),
            ("XDTOPackage", MetadataKind::XdtoPackage),
        ];

        for (prefix, expected_kind) in mappings {
            assert_eq!(
                normalize_subsystem_content_target(&format!("{prefix}.ExactName")),
                SubsystemContentTarget::Resolvable {
                    metadata_kind: expected_kind,
                    local_name: EntityName::new("ExactName").expect("name must be valid"),
                }
            );
        }

        assert_eq!(
            normalize_subsystem_content_target("Subsystem.Child"),
            SubsystemContentTarget::Unsupported {
                prefix: "Subsystem".to_owned(),
                deferred: true,
            }
        );
        for prefix in ["Configuration", "Form", "Unknown", "document"] {
            assert_eq!(
                normalize_subsystem_content_target(&format!("{prefix}.ExactName")),
                SubsystemContentTarget::Unsupported {
                    prefix: prefix.to_owned(),
                    deferred: false,
                }
            );
        }
        for raw in [
            "",
            "Document",
            ".ExactName",
            "Document.",
            "Document.Too.Many",
            "Document. ",
        ] {
            assert_eq!(
                normalize_subsystem_content_target(raw),
                SubsystemContentTarget::Malformed
            );
        }
    }

    #[test]
    fn subsystem_content_missing_flat_source_is_a_build_invariant_error() {
        let subsystem_id = EntityId::new("b72ed007-5756-4a1d-b27d-e74aef13083f")
            .expect("identifier must be valid");
        let subsystem_node_id =
            EntityId::new(format!("{subsystem_id}:subsystem")).expect("identifier must be valid");
        let observations = BTreeSet::from([PendingSubsystemContentObservation {
            descriptor_path: PathBuf::from("src/Subsystems/TestObject/TestObject.mdo"),
            subsystem_id: subsystem_id.clone(),
            subsystem_node_id: subsystem_node_id.clone(),
            raw_token: "Document.Target".to_owned(),
            target: SubsystemContentTarget::Resolvable {
                metadata_kind: MetadataKind::Document,
                local_name: EntityName::new("Target").expect("name must be valid"),
            },
        }]);
        let mut graph = SemanticGraph::new();
        let mut diagnostics = BTreeSet::new();
        let mut statistics = SemanticReferenceStatistics::new();

        let error =
            emit_subsystem_includes(&mut graph, &observations, &mut diagnostics, &mut statistics)
                .expect_err("missing flat Subsystem source must fail the build");

        assert!(matches!(
            error,
            EdtGraphError::InvalidSubsystemSource {
                subsystem_id: actual_metadata_id,
                subsystem_node_id: actual_node_id,
                actual_kind: None,
            } if actual_metadata_id == subsystem_id && actual_node_id == subsystem_node_id
        ));
        assert!(diagnostics.is_empty());
        assert!(statistics.is_empty());
    }

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

    fn replace_document_descriptor(root: &tempfile::TempDir, xml: &str) {
        fs::write(root.path().join("src/Documents/Sales/Sales.mdo"), xml)
            .expect("document descriptor must be replaced");
    }

    fn replace_object_module(root: &tempfile::TempDir, source: &str) {
        fs::write(
            root.path().join("src/Documents/Sales/ObjectModule.bsl"),
            source,
        )
        .expect("object module must be replaced");
    }

    fn add_catalog_descriptor(
        root: &tempfile::TempDir,
        directory_name: &str,
        uuid: &str,
        name: &str,
    ) {
        let directory = root.path().join("src/Catalogs").join(directory_name);
        fs::create_dir_all(&directory).expect("catalog directory must be created");
        fs::write(
            directory.join(format!("{directory_name}.mdo")),
            format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<mdclass:Catalog
    xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass"
    uuid="{uuid}">
    <name>{name}</name>
</mdclass:Catalog>
"#
            ),
        )
        .expect("catalog descriptor must be created");
    }

    fn add_adopted_document_descriptor(
        root: &tempfile::TempDir,
        directory_name: &str,
        uuid: &str,
        name: &str,
        extended_configuration_object: &str,
    ) {
        add_adopted_metadata_descriptor(
            root,
            "Documents",
            "Document",
            directory_name,
            uuid,
            name,
            extended_configuration_object,
        );
    }

    fn add_adopted_catalog_descriptor(
        root: &tempfile::TempDir,
        directory_name: &str,
        uuid: &str,
        name: &str,
        extended_configuration_object: &str,
    ) {
        add_adopted_metadata_descriptor(
            root,
            "Catalogs",
            "Catalog",
            directory_name,
            uuid,
            name,
            extended_configuration_object,
        );
    }

    fn add_adopted_metadata_descriptor(
        root: &tempfile::TempDir,
        directory: &str,
        xml_kind: &str,
        directory_name: &str,
        uuid: &str,
        name: &str,
        extended_configuration_object: &str,
    ) {
        let object_directory = root.path().join("src").join(directory).join(directory_name);
        fs::create_dir_all(&object_directory).expect("adopted object directory must be created");
        fs::write(
            object_directory.join(format!("{directory_name}.mdo")),
            format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<mdclass:{xml_kind}
    xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass"
    uuid="{uuid}">
    <Properties>
        <Name>{name}</Name>
        <ObjectBelonging>Adopted</ObjectBelonging>
        <ExtendedConfigurationObject>{extended_configuration_object}</ExtendedConfigurationObject>
    </Properties>
</mdclass:{xml_kind}>
"#
            ),
        )
        .expect("adopted metadata descriptor must be created");
    }

    fn add_common_command_descriptor(root: &tempfile::TempDir) {
        let directory = root.path().join("src/CommonCommands/RefreshData");
        fs::create_dir_all(&directory).expect("common command directory must be created");
        fs::write(directory.join("RefreshData.mdo"), COMMON_COMMAND_XML)
            .expect("common command descriptor must be created");
    }

    fn add_common_template_descriptor(root: &tempfile::TempDir) {
        let directory = root.path().join("src/CommonTemplates/Invoice");
        fs::create_dir_all(&directory).expect("common template directory must be created");
        fs::write(directory.join("Invoice.mdo"), COMMON_TEMPLATE_XML)
            .expect("common template descriptor must be created");
    }

    fn add_archive_common_template_descriptor(root: &tempfile::TempDir) {
        let directory = root.path().join("src/CommonTemplates/Archive");
        fs::create_dir_all(&directory).expect("second common template directory must be created");
        fs::write(
            directory.join("Archive.mdo"),
            r#"<?xml version="1.0" encoding="UTF-8"?>
<mdclass:CommonTemplate
    xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass"
    uuid="eeeeeeee-aaaa-bbbb-cccc-111111111111">
    <name>Archive</name>
    <templateType>TextDocument</templateType>
</mdclass:CommonTemplate>
"#,
        )
        .expect("second common template descriptor must be created");
    }

    fn add_accounting_register_descriptor(root: &tempfile::TempDir) {
        let directory = root.path().join("src/AccountingRegisters/GeneralLedger");
        fs::create_dir_all(&directory).expect("accounting register directory must be created");
        fs::write(directory.join("GeneralLedger.mdo"), ACCOUNTING_REGISTER_XML)
            .expect("accounting register descriptor must be created");
    }

    fn add_role_descriptor(root: &tempfile::TempDir) {
        let directory = root.path().join("src/Roles/SalesManager");
        fs::create_dir_all(&directory).expect("role directory must be created");
        fs::write(directory.join("SalesManager.mdo"), ROLE_XML)
            .expect("role descriptor must be created");
        fs::write(directory.join("Rights.rights"), DEFAULT_ROLE_RIGHTS_XML)
            .expect("role rights must be created");
    }

    fn add_read_only_role_descriptor(root: &tempfile::TempDir) {
        let directory = root.path().join("src/Roles/ReadOnly");
        fs::create_dir_all(&directory).expect("second role directory must be created");
        fs::write(
            directory.join("ReadOnly.mdo"),
            r#"<?xml version="1.0" encoding="UTF-8"?>
<mdclass:Role
    xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass"
    uuid="eeeeeeee-aaaa-bbbb-cccc-555555555555">
    <name>ReadOnly</name>
</mdclass:Role>
"#,
        )
        .expect("second role descriptor must be created");
        fs::write(directory.join("Rights.rights"), DEFAULT_ROLE_RIGHTS_XML)
            .expect("second role rights must be created");
    }

    fn add_subsystem_descriptor(root: &tempfile::TempDir) {
        let directory = root.path().join("src/Subsystems/SalesSubsystem");
        fs::create_dir_all(&directory).expect("subsystem directory must be created");
        fs::write(directory.join("SalesSubsystem.mdo"), SUBSYSTEM_XML)
            .expect("subsystem descriptor must be created");
    }

    fn add_reports_subsystem_descriptor(root: &tempfile::TempDir) {
        let directory = root.path().join("src/Subsystems/Reports");
        fs::create_dir_all(&directory).expect("second subsystem directory must be created");
        fs::write(
            directory.join("Reports.mdo"),
            r#"<?xml version="1.0" encoding="UTF-8"?>
<mdclass:Subsystem
    xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass"
    uuid="ffffffff-aaaa-bbbb-cccc-555555555555">
    <name>Reports</name>
</mdclass:Subsystem>
"#,
        )
        .expect("second subsystem descriptor must be created");
    }

    fn document_with_reference(reference_type: &str) -> String {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<mdclass:Document
    xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass"
    uuid="aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee">
    <name>Sales</name>

    <attributes uuid="aaaaaaaa-1111-1111-1111-111111111111">
        <name>Company</name>
        <type>
            <types>{reference_type}</types>
        </type>
    </attributes>
</mdclass:Document>
"#
        )
    }

    fn document_with_duplicate_reference(reference_type: &str) -> String {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<mdclass:Document
    xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass"
    uuid="aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee">
    <name>Sales</name>

    <attributes uuid="aaaaaaaa-1111-1111-1111-111111111111">
        <name>Company</name>
        <type>
            <types>{reference_type}</types>
            <types>{reference_type}</types>
        </type>
    </attributes>
</mdclass:Document>
"#
        )
    }

    fn document_with_composite_reference() -> String {
        r#"<?xml version="1.0" encoding="UTF-8"?>
<mdclass:Document
    xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass"
    uuid="aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee">
    <name>Sales</name>

    <attributes uuid="aaaaaaaa-1111-1111-1111-111111111111">
        <name>Company</name>
        <type>
            <types>CatalogRef.Products</types>
            <types>DocumentRef.Sales</types>
        </type>
    </attributes>
</mdclass:Document>
"#
        .to_owned()
    }

    fn document_with_two_missing_references(reverse_order: bool) -> String {
        let first = r#"
    <attributes uuid="aaaaaaaa-1111-1111-1111-111111111111">
        <name>Company</name>
        <type>
            <types>CatalogRef.MissingProducts</types>
        </type>
    </attributes>
"#;
        let second = r#"
    <attributes uuid="aaaaaaaa-2222-2222-2222-222222222222">
        <name>Warehouse</name>
        <type>
            <types>CatalogRef.MissingProducts</types>
        </type>
    </attributes>
"#;
        let (left, right) = if reverse_order {
            (second, first)
        } else {
            (first, second)
        };

        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<mdclass:Document
    xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass"
    uuid="aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee">
    <name>Sales</name>
{left}{right}</mdclass:Document>
"#
        )
    }

    fn company_attribute_id() -> oneagent_common::EntityId {
        oneagent_common::EntityId::new("aaaaaaaa-1111-1111-1111-111111111111")
            .expect("identifier must be valid")
    }

    fn expected_edge_id(
        source: &oneagent_common::EntityId,
        target: &oneagent_common::EntityId,
        kind: EdgeKind,
    ) -> String {
        let kind_code = match kind {
            EdgeKind::Contains => "contains",
            EdgeKind::Calls => "calls",
            EdgeKind::References => "references",
            EdgeKind::Reads => "reads",
            EdgeKind::Writes => "writes",
            EdgeKind::Grants => "grants",
            EdgeKind::Includes => "includes",
            EdgeKind::Extends => "extends",
            EdgeKind::DependsOn => "depends_on",
        };

        format!(
            "edge:source#{}:{};target#{}:{};kind:{}",
            source.as_str().len(),
            source.as_str(),
            target.as_str().len(),
            target.as_str(),
            kind_code
        )
    }

    fn named_node<'graph>(
        graph: &'graph SemanticGraph,
        kind: NodeKind,
        node_name: &str,
        message: &str,
    ) -> &'graph oneagent_graph::GraphNode {
        graph
            .nodes_by_kind(kind)
            .into_iter()
            .find(|node| node.name().as_str() == node_name)
            .expect(message)
    }

    fn assert_company_reference_provenance(edge: &GraphEdge) {
        assert_eq!(edge.provenance().len(), 1);
        assert_eq!(edge.provenance()[0].origin(), FactOrigin::Resolved);
        assert_eq!(edge.provenance()[0].resolution(), ResolutionState::Resolved);
        assert!(
            edge.provenance()[0]
                .source()
                .expect("reference edge source must exist")
                .as_str()
                .ends_with(
                    "/src/Documents/Sales/Sales.mdo#metadata_object=aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee;edge=references;source=aaaaaaaa-1111-1111-1111-111111111111;role=type;target_kind=catalog;target_name=Products"
                )
        );
    }

    fn assert_company_dependency_provenance(edge: &GraphEdge) {
        assert_eq!(edge.provenance().len(), 1);
        assert_eq!(edge.provenance()[0].origin(), FactOrigin::Derived);
        assert_eq!(edge.provenance()[0].resolution(), ResolutionState::Resolved);
        assert!(
            edge.provenance()[0]
                .source()
                .expect("dependency edge source must exist")
                .as_str()
                .ends_with(
                    "/src/Documents/Sales/Sales.mdo#metadata_object=aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee;edge=depends_on;origin=metadata_member_type_reference;source=aaaaaaaa-1111-1111-1111-111111111111;role=type;target_kind=catalog;target_name=Products;target=11111111-aaaa-bbbb-cccc-222222222222"
                )
        );
    }

    fn assert_metadata_member_provenance(graph: &SemanticGraph) {
        let metadata_member_expectations = [
            (
                NodeKind::Attribute,
                "Company",
                "/src/Documents/Sales/Sales.mdo#metadata_object=aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee;member=attribute:aaaaaaaa-1111-1111-1111-111111111111",
            ),
            (
                NodeKind::Attribute,
                "Warehouse",
                "/src/Documents/Sales/Sales.mdo#metadata_object=aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee;member=attribute:aaaaaaaa-2222-2222-2222-222222222222",
            ),
            (
                NodeKind::TabularSection,
                "Goods",
                "/src/Documents/Sales/Sales.mdo#metadata_object=aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee;member=tabular_section:aaaaaaaa-3333-3333-3333-333333333333",
            ),
            (
                NodeKind::Form,
                "DocumentForm",
                "/src/Documents/Sales/Sales.mdo#metadata_object=aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee;member=form:aaaaaaaa-4444-4444-4444-444444444444",
            ),
            (
                NodeKind::Command,
                "PostAndClose",
                "/src/Documents/Sales/Sales.mdo#metadata_object=aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee;member=command:aaaaaaaa-5555-5555-5555-555555555555",
            ),
            (
                NodeKind::Dimension,
                "Product",
                "/src/AccumulationRegisters/StockBalance/StockBalance.mdo#metadata_object=44444444-4444-4444-4444-444444444444;member=dimension:55555555-5555-5555-5555-555555555555",
            ),
            (
                NodeKind::Dimension,
                "Warehouse",
                "/src/AccumulationRegisters/StockBalance/StockBalance.mdo#metadata_object=44444444-4444-4444-4444-444444444444;member=dimension:66666666-6666-6666-6666-666666666666",
            ),
            (
                NodeKind::Resource,
                "Quantity",
                "/src/AccumulationRegisters/StockBalance/StockBalance.mdo#metadata_object=44444444-4444-4444-4444-444444444444;member=resource:77777777-7777-7777-7777-777777777777",
            ),
        ];

        for (kind, name, expected_source) in metadata_member_expectations {
            let member = graph
                .nodes_by_kind(kind)
                .into_iter()
                .find(|node| node.name().as_str() == name)
                .expect("metadata member node must exist");
            let source = member.provenance()[0]
                .source()
                .expect("metadata member source must exist")
                .as_str();

            assert_eq!(member.provenance()[0].origin(), FactOrigin::Declared);
            assert!(source.contains(expected_source));
        }
    }

    #[test]
    fn builds_graph_with_configuration_and_metadata_objects() {
        let root = create_edt_project();

        let graph = FileSystemEdtSemanticGraphBuilder
            .build_graph(root.path())
            .expect("graph must build");

        assert_eq!(graph.node_count(), 24);
        assert_eq!(graph.edge_count(), 32);
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
        assert_eq!(graph.nodes_by_kind(NodeKind::StandardAttribute).len(), 5);
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
                .nodes_by_kind(NodeKind::StandardAttribute)
                .iter()
                .any(|node| node.name().as_str() == "Number")
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
        assert_eq!(graph.nodes_by_kind(NodeKind::Command).len(), 1);

        assert!(
            graph
                .nodes_by_kind(NodeKind::Command)
                .iter()
                .any(|node| node.name().as_str() == "PostAndClose")
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
    fn discovers_top_level_common_command_as_metadata_entity() {
        let root = create_edt_project();
        add_common_command_descriptor(&root);
        let archive_directory = root.path().join("src/CommonCommands/ArchiveData");
        fs::create_dir_all(&archive_directory)
            .expect("second common command directory must be created");
        fs::write(
            archive_directory.join("ArchiveData.mdo"),
            r#"<?xml version="1.0" encoding="UTF-8"?>
<mdclass:CommonCommand
    xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass"
    uuid="bbbbbbbb-1111-2222-3333-444444444444">
    <name>ArchiveData</name>
</mdclass:CommonCommand>
"#,
        )
        .expect("second common command descriptor must be created");

        let first = FileSystemEdtSemanticGraphBuilder
            .build_graph_with_diagnostics(root.path())
            .expect("graph with common command must build");
        let second = FileSystemEdtSemanticGraphBuilder
            .build_graph_with_diagnostics(root.path())
            .expect("repeated graph build must succeed");
        let graph = first.graph();
        let command_id = NodeId::new("cccccccc-1111-2222-3333-444444444444");
        let configuration = graph
            .nodes_by_kind(NodeKind::Metadata(MetadataKind::Configuration))
            .into_iter()
            .next()
            .expect("configuration node must exist");
        let command = graph
            .query()
            .node(&command_id)
            .expect("common command must be queryable by stable UUID");
        let owner = graph
            .query()
            .owner(&command_id)
            .expect("configuration must own common command");
        let command_children = graph.query().children_by_kind(
            &NodeId::new(configuration.id().as_str()),
            NodeKind::Metadata(MetadataKind::Command),
        );
        let contains = graph
            .query()
            .incoming_edges_by_kind(&command_id, EdgeKind::Contains)
            .into_iter()
            .next()
            .expect("configuration containment edge must exist");
        let coverage = first.coverage_report();

        assert_eq!(command.name().as_str(), "RefreshData");
        assert_eq!(command.kind(), NodeKind::Metadata(MetadataKind::Command));
        assert_eq!(owner.id(), configuration.id());
        assert_eq!(
            command_children
                .iter()
                .map(|child| child.id().as_str())
                .collect::<Vec<_>>(),
            vec![
                "bbbbbbbb-1111-2222-3333-444444444444",
                "cccccccc-1111-2222-3333-444444444444",
            ]
        );
        assert_eq!(contains.source(), configuration.id());
        assert_eq!(command.provenance().len(), 1);
        assert!(
            command.provenance()[0]
                .source()
                .expect("command provenance source must exist")
                .as_str()
                .ends_with(
                    "/src/CommonCommands/RefreshData/RefreshData.mdo#metadata_object=cccccccc-1111-2222-3333-444444444444;fact=metadata_object"
                )
        );
        assert_eq!(contains.provenance().len(), 1);
        assert!(
            contains.provenance()[0]
                .source()
                .expect("containment provenance source must exist")
                .as_str()
                .ends_with(
                    "/src/CommonCommands/RefreshData/RefreshData.mdo#metadata_object=cccccccc-1111-2222-3333-444444444444;edge=contains;source=11111111-2222-3333-4444-555555555555;target=cccccccc-1111-2222-3333-444444444444"
                )
        );
        assert_eq!(
            coverage.observed().nodes()[&NodeKind::Metadata(MetadataKind::Command)].total(),
            2
        );
        assert_eq!(
            coverage.observed().nodes()[&NodeKind::Metadata(MetadataKind::Command)]
                .without_provenance(),
            0
        );
        assert_eq!(graph.nodes_by_kind(NodeKind::Command).len(), 1);
        assert!(first.validate().is_valid());
        assert!(graph.diff(second.graph()).is_empty());
        assert!(first.diff(&second).is_empty());
    }

    #[test]
    fn rejects_top_level_common_command_without_name() {
        let root = create_edt_project();
        let directory = root.path().join("src/CommonCommands/BrokenCommand");
        fs::create_dir_all(&directory).expect("common command directory must be created");
        fs::write(
            directory.join("BrokenCommand.mdo"),
            r#"<?xml version="1.0" encoding="UTF-8"?>
<mdclass:CommonCommand
    xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass"
    uuid="dddddddd-1111-2222-3333-444444444444">
</mdclass:CommonCommand>
"#,
        )
        .expect("invalid common command descriptor must be created");

        let error = FileSystemEdtSemanticGraphBuilder
            .build_graph(root.path())
            .expect_err("common command without name must be rejected");

        assert!(
            error
                .to_string()
                .contains("metadata object name is missing")
        );
    }

    #[test]
    fn discovers_top_level_common_template_as_metadata_entity() {
        let root = create_edt_project();
        add_common_template_descriptor(&root);
        add_archive_common_template_descriptor(&root);

        let first = FileSystemEdtSemanticGraphBuilder
            .build_graph_with_diagnostics(root.path())
            .expect("graph with common template must build");
        let second = FileSystemEdtSemanticGraphBuilder
            .build_graph_with_diagnostics(root.path())
            .expect("repeated graph build must succeed");
        let graph = first.graph();
        let template_id = NodeId::new("dddddddd-aaaa-bbbb-cccc-111111111111");
        let configuration = graph
            .nodes_by_kind(NodeKind::Metadata(MetadataKind::Configuration))
            .into_iter()
            .next()
            .expect("configuration node must exist");
        let template = graph
            .query()
            .node(&template_id)
            .expect("common template must be queryable by stable UUID");
        let owner = graph
            .query()
            .owner(&template_id)
            .expect("configuration must own common template");
        let template_children = graph.query().children_by_kind(
            &NodeId::new(configuration.id().as_str()),
            NodeKind::Metadata(MetadataKind::Template),
        );
        let contains = graph
            .query()
            .incoming_edges_by_kind(&template_id, EdgeKind::Contains)
            .into_iter()
            .next()
            .expect("configuration containment edge must exist");
        let coverage = first.coverage_report();
        let capability = coverage
            .edt_pipeline()
            .capability(SemanticCoverageCapabilityId::MetadataEntity(
                MetadataKind::Template,
            ))
            .expect("template coverage must exist");

        assert_eq!(template.name().as_str(), "Invoice");
        assert_eq!(template.kind(), NodeKind::Metadata(MetadataKind::Template));
        assert_eq!(owner.id(), configuration.id());
        assert_eq!(
            template_children
                .iter()
                .map(|child| child.id().as_str())
                .collect::<Vec<_>>(),
            vec![
                "dddddddd-aaaa-bbbb-cccc-111111111111",
                "eeeeeeee-aaaa-bbbb-cccc-111111111111",
            ]
        );
        assert_eq!(contains.source(), configuration.id());
        assert_eq!(template.provenance().len(), 1);
        assert!(
            template.provenance()[0]
                .source()
                .expect("template provenance source must exist")
                .as_str()
                .ends_with(
                    "/src/CommonTemplates/Invoice/Invoice.mdo#metadata_object=dddddddd-aaaa-bbbb-cccc-111111111111;fact=metadata_object"
                )
        );
        assert_eq!(contains.provenance().len(), 1);
        assert!(
            contains.provenance()[0]
                .source()
                .expect("containment provenance source must exist")
                .as_str()
                .ends_with(
                    "/src/CommonTemplates/Invoice/Invoice.mdo#metadata_object=dddddddd-aaaa-bbbb-cccc-111111111111;edge=contains;source=11111111-2222-3333-4444-555555555555;target=dddddddd-aaaa-bbbb-cccc-111111111111"
                )
        );
        assert_eq!(
            coverage.observed().nodes()[&NodeKind::Metadata(MetadataKind::Template)].total(),
            2
        );
        assert_eq!(
            coverage.observed().nodes()[&NodeKind::Metadata(MetadataKind::Template)]
                .without_provenance(),
            0
        );
        assert_eq!(
            capability.status(),
            SemanticCoverageStatus::PartiallySupported
        );
        assert!(first.validate().is_valid());
        assert!(graph.diff(second.graph()).is_empty());
        assert!(first.diff(&second).is_empty());
    }

    #[test]
    fn emits_role_semantic_nodes_alongside_metadata_role_objects() {
        let root = create_edt_project();
        add_role_descriptor(&root);
        add_read_only_role_descriptor(&root);

        let first = FileSystemEdtSemanticGraphBuilder
            .build_graph_with_diagnostics(root.path())
            .expect("graph with roles must build");
        let second = FileSystemEdtSemanticGraphBuilder
            .build_graph_with_diagnostics(root.path())
            .expect("repeated graph build must succeed");
        let graph = first.graph();
        let role_metadata_id = NodeId::new("eeeeeeee-1111-2222-3333-444444444444");
        let role_node_id = NodeId::new("eeeeeeee-1111-2222-3333-444444444444:role");
        let role_metadata = graph
            .query()
            .node(&role_metadata_id)
            .expect("metadata role object must remain queryable by stable UUID");
        let role = graph
            .query()
            .node(&role_node_id)
            .expect("role semantic node must be queryable by stable derived id");
        let repeated_role = second
            .graph()
            .query()
            .node(&role_node_id)
            .expect("repeated build must preserve role identity");
        let role_sources = role
            .provenance()
            .iter()
            .map(|provenance| {
                provenance
                    .source()
                    .expect("role provenance source must exist")
                    .as_str()
                    .to_owned()
            })
            .collect::<Vec<_>>();
        let repeated_role_sources = repeated_role
            .provenance()
            .iter()
            .map(|provenance| {
                provenance
                    .source()
                    .expect("repeated role provenance source must exist")
                    .as_str()
                    .to_owned()
            })
            .collect::<Vec<_>>();

        assert!(first.diagnostics().is_empty());
        assert_eq!(role_metadata.name().as_str(), "SalesManager");
        assert_eq!(role_metadata.kind(), NodeKind::Metadata(MetadataKind::Role));
        assert_eq!(role.name().as_str(), "SalesManager");
        assert_eq!(role.kind(), NodeKind::Role);
        assert_ne!(role_metadata.id(), role.id());
        assert_eq!(role.id(), repeated_role.id());
        assert_eq!(role_sources, repeated_role_sources);
        assert_eq!(
            graph
                .nodes_by_kind(NodeKind::Metadata(MetadataKind::Role))
                .len(),
            2
        );
        assert_eq!(graph.nodes_by_kind(NodeKind::Role).len(), 2);
        assert_eq!(role.provenance().len(), 1);
        assert_eq!(role.provenance()[0].origin(), FactOrigin::Declared);
        assert!(
            role.provenance()[0]
                .source()
                .expect("role provenance source must exist")
                .as_str()
                .ends_with(
                    "/src/Roles/SalesManager/SalesManager.mdo#metadata_object=eeeeeeee-1111-2222-3333-444444444444;fact=role_node"
                )
        );
        assert!(graph.query().owner(&role_node_id).is_none());
        assert!(first.validate().is_valid());
        assert!(graph.diff(second.graph()).is_empty());
        assert!(first.diff(&second).is_empty());
    }

    #[test]
    fn emits_subsystem_semantic_nodes_alongside_metadata_subsystem_objects() {
        let root = create_edt_project();
        add_subsystem_descriptor(&root);
        add_reports_subsystem_descriptor(&root);

        let first = FileSystemEdtSemanticGraphBuilder
            .build_graph_with_diagnostics(root.path())
            .expect("graph with subsystems must build");
        let second = FileSystemEdtSemanticGraphBuilder
            .build_graph_with_diagnostics(root.path())
            .expect("repeated graph build must succeed");
        let graph = first.graph();
        let subsystem_metadata_id = NodeId::new("ffffffff-1111-2222-3333-444444444444");
        let subsystem_node_id = NodeId::new("ffffffff-1111-2222-3333-444444444444:subsystem");
        let subsystem_metadata = graph
            .query()
            .node(&subsystem_metadata_id)
            .expect("metadata subsystem object must remain queryable by stable UUID");
        let subsystem = graph
            .query()
            .node(&subsystem_node_id)
            .expect("subsystem semantic node must be queryable by stable derived id");
        let repeated_subsystem = second
            .graph()
            .query()
            .node(&subsystem_node_id)
            .expect("repeated build must preserve subsystem identity");
        let subsystem_sources = subsystem
            .provenance()
            .iter()
            .map(|provenance| {
                provenance
                    .source()
                    .expect("subsystem provenance source must exist")
                    .as_str()
                    .to_owned()
            })
            .collect::<Vec<_>>();
        let repeated_subsystem_sources = repeated_subsystem
            .provenance()
            .iter()
            .map(|provenance| {
                provenance
                    .source()
                    .expect("repeated subsystem provenance source must exist")
                    .as_str()
                    .to_owned()
            })
            .collect::<Vec<_>>();

        assert!(first.diagnostics().is_empty());
        assert_eq!(subsystem_metadata.name().as_str(), "SalesSubsystem");
        assert_eq!(
            subsystem_metadata.kind(),
            NodeKind::Metadata(MetadataKind::Subsystem)
        );
        assert_eq!(subsystem.name().as_str(), "SalesSubsystem");
        assert_eq!(subsystem.kind(), NodeKind::Subsystem);
        assert_ne!(subsystem_metadata.id(), subsystem.id());
        assert_eq!(subsystem.id(), repeated_subsystem.id());
        assert_eq!(subsystem_sources, repeated_subsystem_sources);
        assert_eq!(
            graph
                .nodes_by_kind(NodeKind::Metadata(MetadataKind::Subsystem))
                .len(),
            2
        );
        assert_eq!(graph.nodes_by_kind(NodeKind::Subsystem).len(), 2);
        assert_eq!(
            graph
                .nodes_by_kind(NodeKind::Metadata(MetadataKind::Document))
                .len(),
            1
        );
        assert_eq!(subsystem.provenance().len(), 1);
        assert_eq!(subsystem.provenance()[0].origin(), FactOrigin::Declared);
        assert!(
            subsystem.provenance()[0]
                .source()
                .expect("subsystem provenance source must exist")
                .as_str()
                .ends_with(
                    "/src/Subsystems/SalesSubsystem/SalesSubsystem.mdo#metadata_object=ffffffff-1111-2222-3333-444444444444;fact=subsystem_node"
                )
        );
        assert!(graph.query().owner(&subsystem_node_id).is_none());
        assert!(first.validate().is_valid());
        assert!(graph.diff(second.graph()).is_empty());
        assert!(first.diff(&second).is_empty());
    }

    #[test]
    fn emits_document_standard_attribute_nodes_through_production_graph_builder() {
        let root = create_edt_project();

        let first = FileSystemEdtSemanticGraphBuilder
            .build_graph_with_diagnostics(root.path())
            .expect("graph with document standard attributes must build");
        let second = FileSystemEdtSemanticGraphBuilder
            .build_graph_with_diagnostics(root.path())
            .expect("repeated graph build must succeed");
        let graph = first.graph();
        let document_id = NodeId::new("aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee");
        let number_id =
            NodeId::new("aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee:standard_attribute:number");
        let catalog_code_id =
            NodeId::new("11111111-aaaa-bbbb-cccc-222222222222:standard_attribute:code");
        let number = graph
            .query()
            .node(&number_id)
            .expect("document Number standard attribute must exist");
        let repeated_number = second
            .graph()
            .query()
            .node(&number_id)
            .expect("repeated build must preserve standard attribute identity");
        let owner = graph
            .query()
            .owner(&number_id)
            .expect("document must own its standard attribute");
        let standard_attributes = graph
            .query()
            .children_by_kind(&document_id, NodeKind::StandardAttribute);
        let company = graph
            .nodes_by_kind(NodeKind::Attribute)
            .into_iter()
            .find(|node| node.name().as_str() == "Company")
            .expect("ordinary attribute must remain an Attribute node");

        assert!(first.diagnostics().is_empty());
        assert_eq!(graph.nodes_by_kind(NodeKind::StandardAttribute).len(), 5);
        assert_eq!(
            standard_attributes
                .iter()
                .map(|node| node.name().as_str())
                .collect::<Vec<_>>(),
            vec!["Date", "DeletionMark", "Number", "Posted", "Ref"]
        );
        assert_eq!(number.kind(), NodeKind::StandardAttribute);
        assert_eq!(number.name().as_str(), "Number");
        assert_eq!(number.id(), repeated_number.id());
        assert_eq!(owner.id().as_str(), document_id.as_str());
        assert_eq!(graph.query().owner_edges(&number_id).len(), 1);
        assert_eq!(company.kind(), NodeKind::Attribute);
        assert!(graph.query().node(&catalog_code_id).is_none());
        assert_eq!(number.provenance().len(), 1);
        assert_eq!(number.provenance()[0].origin(), FactOrigin::Declared);
        assert!(
            number.provenance()[0]
                .source()
                .expect("standard attribute provenance source must exist")
                .as_str()
                .ends_with(
                    "/src/Documents/Sales/Sales.mdo#metadata_object=aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee;member=standard_attribute:number"
                )
        );
        assert_eq!(number.provenance(), repeated_number.provenance());
        assert!(first.validate().is_valid());
        assert!(graph.diff(second.graph()).is_empty());
        assert!(first.diff(&second).is_empty());
    }

    #[test]
    fn emits_document_standard_attribute_ownership_edges_through_production_graph_builder() {
        let root = create_edt_project();

        let first = FileSystemEdtSemanticGraphBuilder
            .build_graph_with_diagnostics(root.path())
            .expect("graph with document standard attributes must build");
        let second = FileSystemEdtSemanticGraphBuilder
            .build_graph_with_diagnostics(root.path())
            .expect("repeated graph build must succeed");
        let graph = first.graph();
        let repeated_graph = second.graph();
        let document_id = NodeId::new("aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee");
        let standard_attribute_ids = [
            "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee:standard_attribute:date",
            "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee:standard_attribute:deletion_mark",
            "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee:standard_attribute:number",
            "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee:standard_attribute:posted",
            "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee:standard_attribute:ref",
        ];
        let standard_attributes = graph
            .query()
            .children_by_kind(&document_id, NodeKind::StandardAttribute);

        assert!(first.diagnostics().is_empty());
        assert_eq!(standard_attributes.len(), standard_attribute_ids.len());

        for attribute_id in standard_attribute_ids {
            let attribute_id = NodeId::new(attribute_id);
            let attribute = graph
                .query()
                .node(&attribute_id)
                .expect("standard attribute node must exist");
            let owner = graph
                .query()
                .owner(&attribute_id)
                .expect("document must own each standard attribute");
            let owner_edges = graph.query().owner_edges(&attribute_id);
            let repeated_owner_edges = repeated_graph.query().owner_edges(&attribute_id);

            assert_eq!(attribute.kind(), NodeKind::StandardAttribute);
            assert_eq!(owner.id().as_str(), document_id.as_str());
            assert_eq!(owner_edges.len(), 1);
            assert_eq!(repeated_owner_edges.len(), 1);
            assert_eq!(owner_edges[0].source().as_str(), document_id.as_str());
            assert_eq!(owner_edges[0].target().as_str(), attribute_id.as_str());
            assert_eq!(owner_edges[0].kind(), EdgeKind::Contains);
            assert_eq!(owner_edges[0].source(), repeated_owner_edges[0].source());
            assert_eq!(owner_edges[0].target(), repeated_owner_edges[0].target());
            assert_eq!(owner_edges[0].kind(), repeated_owner_edges[0].kind());
            assert_eq!(
                owner_edges[0].provenance(),
                repeated_owner_edges[0].provenance()
            );
            assert_eq!(owner_edges[0].provenance().len(), 1);
            assert_eq!(owner_edges[0].provenance(), attribute.provenance());
            assert_eq!(
                owner_edges[0].provenance()[0].origin(),
                FactOrigin::Declared
            );
            assert!(
                owner_edges[0].provenance()[0]
                    .source()
                    .expect("standard attribute ownership provenance source must exist")
                    .as_str()
                    .contains(
                        "/src/Documents/Sales/Sales.mdo#metadata_object=aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee;member=standard_attribute:"
                    )
            );
        }

        assert!(first.validate().is_valid());
        assert!(graph.diff(second.graph()).is_empty());
        assert!(first.diff(&second).is_empty());
    }

    #[test]
    fn rejects_top_level_common_template_without_name() {
        let root = create_edt_project();
        let directory = root.path().join("src/CommonTemplates/BrokenTemplate");
        fs::create_dir_all(&directory).expect("common template directory must be created");
        fs::write(
            directory.join("BrokenTemplate.mdo"),
            r#"<?xml version="1.0" encoding="UTF-8"?>
<mdclass:CommonTemplate
    xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass"
    uuid="ffffffff-aaaa-bbbb-cccc-111111111111">
    <templateType>SpreadsheetDocument</templateType>
</mdclass:CommonTemplate>
"#,
        )
        .expect("invalid common template descriptor must be created");

        let error = FileSystemEdtSemanticGraphBuilder
            .build_graph(root.path())
            .expect_err("common template without name must be rejected");

        assert!(
            error
                .to_string()
                .contains("metadata object name is missing")
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

    fn assert_before_write_query_provenance(query: &GraphNode) {
        assert_eq!(query.provenance().len(), 1);
        assert_eq!(query.provenance()[0].origin(), FactOrigin::Declared);
        assert!(
            query.provenance()[0]
                .source()
                .expect("query provenance source must exist")
                .as_str()
                .contains(
                    "/src/Documents/Sales/ObjectModule.bsl#bsl_query=aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee:object_module:procedure:BeforeWrite:query:Query;owner=aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee:object_module:procedure:BeforeWrite;binding=Query"
                )
        );
    }

    #[test]
    fn emits_static_bsl_query_nodes_through_production_graph_builder() {
        let root = create_edt_project();
        replace_object_module(
            &root,
            concat!(
                "Procedure BeforeWrite()\n",
                "    Query = New Query;\n",
                "    Query.Text = \"SELECT Ref FROM Catalog.Products\";\n",
                "EndProcedure\n",
                "\n",
                "Function GetQuery()\n",
                "    Query = New Query;\n",
                "    Query.Text = \"SELECT Ref FROM Catalog.Products\";\n",
                "    Return Query;\n",
                "EndFunction",
            ),
        );

        let first = FileSystemEdtSemanticGraphBuilder
            .build_graph_with_diagnostics(root.path())
            .expect("graph with static BSL queries must build");
        let second = FileSystemEdtSemanticGraphBuilder
            .build_graph_with_diagnostics(root.path())
            .expect("repeated graph build must succeed");
        let before_write_query_id = NodeId::new(
            "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee:object_module:procedure:BeforeWrite:query:Query",
        );
        let get_query_id = NodeId::new(
            "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee:object_module:function:GetQuery:query:Query",
        );
        let before_write_owner_id =
            NodeId::new("aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee:object_module:procedure:BeforeWrite");
        let get_query_owner_id =
            NodeId::new("aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee:object_module:function:GetQuery");
        let graph = first.graph();
        let query_api = graph.query();
        let before_write_query = query_api
            .node(&before_write_query_id)
            .expect("BeforeWrite query node must exist");
        let get_query = query_api
            .node(&get_query_id)
            .expect("GetQuery query node must exist");
        let before_write_owner = query_api
            .owner(&before_write_query_id)
            .expect("BeforeWrite query owner must exist");
        let get_query_owner = query_api
            .owner(&get_query_id)
            .expect("GetQuery query owner must exist");
        let repeated_query = second
            .graph()
            .query()
            .node(&before_write_query_id)
            .expect("repeated build must preserve query identity");

        assert!(first.diagnostics().is_empty());
        assert_eq!(graph.nodes_by_kind(NodeKind::Query).len(), 2);
        assert_eq!(before_write_query.kind(), NodeKind::Query);
        assert_eq!(get_query.kind(), NodeKind::Query);
        assert_eq!(before_write_query.name().as_str(), "Query");
        assert_eq!(before_write_query.id(), repeated_query.id());
        assert_eq!(
            before_write_owner.id().as_str(),
            before_write_owner_id.as_str()
        );
        assert_eq!(get_query_owner.id().as_str(), get_query_owner_id.as_str());
        assert_ne!(before_write_query.id(), get_query.id());
        assert_eq!(query_api.owner_edges(&before_write_query_id).len(), 1);
        assert_eq!(query_api.owner_edges(&get_query_id).len(), 1);
        assert_before_write_query_provenance(before_write_query);
        let reads = query_api.edges_by_kind(EdgeKind::Reads);
        assert_eq!(reads.len(), 2);
        assert!(reads.iter().all(|edge| {
            edge.target().as_str() == "11111111-aaaa-bbbb-cccc-222222222222"
                && !edge.provenance().is_empty()
        }));
        assert_eq!(
            reads
                .iter()
                .map(|edge| edge.source().as_str())
                .collect::<BTreeSet<_>>(),
            BTreeSet::from([before_write_query_id.as_str(), get_query_id.as_str()])
        );
        assert!(query_api.edges_by_kind(EdgeKind::Writes).is_empty());
        assert!(
            query_api
                .outgoing_edges_by_kind(&before_write_query_id, EdgeKind::DependsOn)
                .is_empty()
        );
        assert!(
            query_api
                .outgoing_edges_by_kind(&get_query_id, EdgeKind::DependsOn)
                .is_empty()
        );
        assert!(first.validate().is_valid());
        assert!(graph.diff(second.graph()).is_empty());
        assert!(first.diff(&second).is_empty());
    }

    #[test]
    fn unsupported_bsl_query_patterns_do_not_emit_query_nodes() {
        let root = create_edt_project();
        replace_object_module(
            &root,
            concat!(
                "Procedure BeforeWrite()\n",
                "    Query = New Query;\n",
                "    Query.Text = QueryText;\n",
                "    OtherQuery = New Query;\n",
                "    OtherQuery.Text = \"SELECT Ref FROM Catalog.Products\";\n",
                "    OtherQuery.Text = \"SELECT Ref FROM Catalog.Services\";\n",
                "    PlainText = \"SELECT Ref FROM Catalog.Products\";\n",
                "EndProcedure",
            ),
        );

        let result = FileSystemEdtSemanticGraphBuilder
            .build_graph_with_diagnostics(root.path())
            .expect("unsupported query patterns must not fail graph build");

        assert!(result.graph().nodes_by_kind(NodeKind::Query).is_empty());
        assert!(result.validate().is_valid());
    }

    #[test]
    fn preserves_unresolved_bsl_calls_as_deterministic_build_diagnostics() {
        let root = create_edt_project();
        let module_path = root.path().join("src/Documents/Sales/ObjectModule.bsl");
        fs::write(
            &module_path,
            concat!(
                "Procedure BeforeWrite()\n",
                "    AccessManagement.CheckAccess();\n",
                "    MissingLocal();\n",
                "    MissingModule.Execute();\n",
                "EndProcedure",
            ),
        )
        .expect("object module must be replaced");

        let first = FileSystemEdtSemanticGraphBuilder
            .build_graph_with_diagnostics(root.path())
            .expect("unresolved calls must not fail graph build");
        let second = FileSystemEdtSemanticGraphBuilder
            .build_graph_with_diagnostics(root.path())
            .expect("repeated graph build must succeed");
        let before_write_id =
            NodeId::new("aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee:object_module:procedure:BeforeWrite");
        let before_write = first
            .graph()
            .query()
            .node(&before_write_id)
            .expect("BeforeWrite procedure must retain its stable ID");
        let calls = first
            .graph()
            .query()
            .outgoing_edges_by_kind(&before_write_id, EdgeKind::Calls);
        let diagnostics = first
            .diagnostics()
            .iter()
            .filter(|diagnostic| {
                diagnostic.code() == SemanticDiagnosticCode::ReferenceUnresolved
                    && diagnostic.source_node() == Some(before_write.id())
            })
            .collect::<Vec<_>>();

        assert_eq!(calls.len(), 1);
        assert_eq!(diagnostics.len(), 2);
        assert_eq!(first.diagnostics(), second.diagnostics());
        assert_eq!(first.reference_statistics(), second.reference_statistics());
        assert_eq!(first.reference_statistics().total(), 7);
        assert_eq!(first.reference_statistics().resolved(), 5);
        assert_eq!(first.reference_statistics().unresolved(), 2);
        assert_eq!(first.reference_statistics().with_provenance(), 7);
        assert!(first.validate().is_valid());

        for diagnostic in diagnostics {
            assert_eq!(diagnostic.kind(), SemanticDiagnosticKind::UnresolvedTarget);
            assert_eq!(
                diagnostic.expected_kinds(),
                &[NodeKind::Procedure, NodeKind::Function]
            );
            assert_eq!(diagnostic.provenance().len(), 1);
            assert_eq!(
                diagnostic.provenance()[0].resolution(),
                ResolutionState::Unresolved
            );
            assert!(
                diagnostic.provenance()[0]
                    .source()
                    .expect("diagnostic source must exist")
                    .as_str()
                    .contains("/src/Documents/Sales/ObjectModule.bsl#bsl_call=")
            );
        }
    }

    #[test]
    fn attaches_provenance_to_edt_graph_facts() {
        let root = create_edt_project();

        let graph = FileSystemEdtSemanticGraphBuilder
            .build_graph(root.path())
            .expect("graph must build");

        let document = graph
            .nodes_by_kind(NodeKind::Metadata(MetadataKind::Document))
            .into_iter()
            .next()
            .expect("document node must exist");
        let object_module = graph
            .nodes_by_kind(NodeKind::Module)
            .into_iter()
            .find(|node| node.name().as_str() == "ObjectModule")
            .expect("object module node must exist");
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

        let document_source = document.provenance()[0]
            .source()
            .expect("document source must exist")
            .as_str();

        assert_eq!(document.provenance()[0].origin(), FactOrigin::Declared);
        assert!(document_source.contains(
            "/src/Documents/Sales/Sales.mdo#metadata_object=aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee;fact=metadata_object"
        ));

        assert_eq!(object_module.provenance()[0].origin(), FactOrigin::Parsed);
        assert!(
            object_module.provenance()[0]
                .source()
                .expect("module source must exist")
                .as_str()
                .contains(
                    "/src/Documents/Sales/ObjectModule.bsl#metadata_object=aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee;module=aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee:object_module"
                )
        );

        assert_eq!(before_write.provenance()[0].origin(), FactOrigin::Declared);
        assert!(
            before_write.provenance()[0]
                .source()
                .expect("procedure source must exist")
                .as_str()
                .ends_with("/src/Documents/Sales/ObjectModule.bsl")
        );

        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].target(), check_access.id());
        assert_eq!(calls[0].provenance()[0].origin(), FactOrigin::Resolved);
        assert_eq!(
            calls[0].provenance()[0].resolution(),
            ResolutionState::Resolved
        );

        assert_metadata_member_provenance(&graph);

        let document_contains_company = graph
            .outgoing_by_kind(document.id(), EdgeKind::Contains)
            .into_iter()
            .find(|edge| edge.target().as_str() == "aaaaaaaa-1111-1111-1111-111111111111")
            .expect("document must contain Company attribute");
        let company_edge_source = document_contains_company.provenance()[0]
            .source()
            .expect("Company edge source must exist")
            .as_str();

        assert_eq!(
            document_contains_company.provenance()[0].origin(),
            FactOrigin::Declared
        );
        assert!(company_edge_source.contains(
            "/src/Documents/Sales/Sales.mdo#metadata_object=aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee;edge=contains;source=aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee;target=aaaaaaaa-1111-1111-1111-111111111111"
        ));

        let module_edge = graph
            .outgoing_by_kind(document.id(), EdgeKind::Contains)
            .into_iter()
            .find(|edge| edge.target() == object_module.id())
            .expect("document must contain object module");
        let module_edge_source = module_edge.provenance()[0]
            .source()
            .expect("module edge source must exist")
            .as_str();

        assert_eq!(module_edge.provenance()[0].origin(), FactOrigin::Parsed);
        assert!(module_edge_source.contains(
            "/src/Documents/Sales/ObjectModule.bsl#metadata_object=aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee;edge=contains;source=aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee;target=aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee:object_module"
        ));
    }

    #[test]
    fn resolves_metadata_reference_and_depends_on_edges() {
        let root = create_edt_project();

        let result = FileSystemEdtSemanticGraphBuilder
            .build_graph_with_diagnostics(root.path())
            .expect("graph must build");
        let repeated = FileSystemEdtSemanticGraphBuilder
            .build_graph_with_diagnostics(root.path())
            .expect("repeated graph must build");
        let graph = result.graph();
        let products = named_node(
            graph,
            NodeKind::Metadata(MetadataKind::Catalog),
            "Products",
            "Products catalog must exist",
        );
        let sales_document = named_node(
            graph,
            NodeKind::Metadata(MetadataKind::Document),
            "Sales",
            "Sales document must exist",
        );
        let company = named_node(graph, NodeKind::Attribute, "Company", "Company must exist");
        let warehouse = named_node(
            graph,
            NodeKind::Attribute,
            "Warehouse",
            "Warehouse must exist",
        );
        let product_dimension = named_node(
            graph,
            NodeKind::Dimension,
            "Product",
            "Product dimension must exist",
        );
        let quantity_resource = named_node(
            graph,
            NodeKind::Resource,
            "Quantity",
            "Quantity resource must exist",
        );

        let company_references = graph.outgoing_by_kind(company.id(), EdgeKind::References);
        let warehouse_references = graph.outgoing_by_kind(warehouse.id(), EdgeKind::References);
        let product_references =
            graph.outgoing_by_kind(product_dimension.id(), EdgeKind::References);
        let quantity_references =
            graph.outgoing_by_kind(quantity_resource.id(), EdgeKind::References);
        let company_dependencies = graph.outgoing_by_kind(company.id(), EdgeKind::DependsOn);
        let warehouse_dependencies = graph.outgoing_by_kind(warehouse.id(), EdgeKind::DependsOn);
        let product_dependencies =
            graph.outgoing_by_kind(product_dimension.id(), EdgeKind::DependsOn);
        let quantity_dependencies =
            graph.outgoing_by_kind(quantity_resource.id(), EdgeKind::DependsOn);
        assert_eq!(company_references.len(), 1);
        assert_eq!(warehouse_references.len(), 1);
        assert_eq!(product_references.len(), 1);
        assert_eq!(quantity_references.len(), 1);
        assert_eq!(company_dependencies.len(), 1);
        assert_eq!(warehouse_dependencies.len(), 1);
        assert_eq!(product_dependencies.len(), 1);
        assert_eq!(quantity_dependencies.len(), 1);
        assert_eq!(company_references[0].target(), products.id());
        assert_eq!(warehouse_references[0].target(), products.id());
        assert_eq!(product_references[0].target(), products.id());
        assert_eq!(quantity_references[0].target(), sales_document.id());
        assert_eq!(company_dependencies[0].target(), products.id());
        assert_eq!(warehouse_dependencies[0].target(), products.id());
        assert_eq!(product_dependencies[0].target(), products.id());
        assert_eq!(quantity_dependencies[0].target(), sales_document.id());
        assert_eq!(
            expected_edge_id(company.id(), products.id(), EdgeKind::DependsOn),
            expected_edge_id(
                named_node(
                    repeated.graph(),
                    NodeKind::Attribute,
                    "Company",
                    "repeated Company attribute must exist",
                )
                .id(),
                named_node(
                    repeated.graph(),
                    NodeKind::Metadata(MetadataKind::Catalog),
                    "Products",
                    "repeated Products catalog must exist",
                )
                .id(),
                EdgeKind::DependsOn,
            )
        );
        assert_eq!(company.provenance().len(), 1);
        assert_company_reference_provenance(company_references[0]);
        assert_company_dependency_provenance(company_dependencies[0]);
        assert!(graph.diff(repeated.graph()).is_empty());
        assert!(result.diff(&repeated).is_empty());
        assert!(result.diagnostics().is_empty());
    }

    #[test]
    fn emits_metadata_extends_edges_for_adopted_objects() {
        let root = create_edt_project();
        add_adopted_document_descriptor(
            &root,
            "SalesExtension",
            "bbbbbbbb-bbbb-cccc-dddd-eeeeeeeeeeee",
            "SalesExtension",
            "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee",
        );
        add_adopted_catalog_descriptor(
            &root,
            "ProductsExtension",
            "cccccccc-bbbb-cccc-dddd-eeeeeeeeeeee",
            "ProductsExtension",
            "11111111-aaaa-bbbb-cccc-222222222222",
        );

        let result = FileSystemEdtSemanticGraphBuilder
            .build_graph_with_diagnostics(root.path())
            .expect("graph must build");
        let repeated = FileSystemEdtSemanticGraphBuilder
            .build_graph_with_diagnostics(root.path())
            .expect("repeated graph must build");
        let graph = result.graph();
        let sales = named_node(
            graph,
            NodeKind::Metadata(MetadataKind::Document),
            "Sales",
            "base document must exist",
        );
        let sales_extension = named_node(
            graph,
            NodeKind::Metadata(MetadataKind::Document),
            "SalesExtension",
            "adopted document must exist",
        );
        let products = named_node(
            graph,
            NodeKind::Metadata(MetadataKind::Catalog),
            "Products",
            "base catalog must exist",
        );
        let products_extension = named_node(
            graph,
            NodeKind::Metadata(MetadataKind::Catalog),
            "ProductsExtension",
            "adopted catalog must exist",
        );

        let document_edges = graph.outgoing_by_kind(sales_extension.id(), EdgeKind::Extends);
        let catalog_edges = graph.outgoing_by_kind(products_extension.id(), EdgeKind::Extends);

        assert_eq!(document_edges.len(), 1);
        assert_eq!(catalog_edges.len(), 1);
        assert_eq!(document_edges[0].target(), sales.id());
        assert_eq!(catalog_edges[0].target(), products.id());
        assert_eq!(
            document_edges[0].provenance()[0].origin(),
            FactOrigin::Resolved
        );
        assert_eq!(
            document_edges[0].provenance()[0].resolution(),
            ResolutionState::Resolved
        );
        assert!(
            document_edges[0].provenance()[0]
                .source()
                .expect("extension edge source must exist")
                .as_str()
                .ends_with(
                    "/src/Documents/SalesExtension/SalesExtension.mdo#metadata_object=bbbbbbbb-bbbb-cccc-dddd-eeeeeeeeeeee;edge=extends;origin=metadata_object_extension;source=bbbbbbbb-bbbb-cccc-dddd-eeeeeeeeeeee;target_kind=document;declared_target=aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee;target=aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee"
                )
        );
        assert_eq!(
            expected_edge_id(sales_extension.id(), sales.id(), EdgeKind::Extends),
            expected_edge_id(
                named_node(
                    repeated.graph(),
                    NodeKind::Metadata(MetadataKind::Document),
                    "SalesExtension",
                    "repeated adopted document must exist",
                )
                .id(),
                named_node(
                    repeated.graph(),
                    NodeKind::Metadata(MetadataKind::Document),
                    "Sales",
                    "repeated base document must exist",
                )
                .id(),
                EdgeKind::Extends,
            )
        );
        assert!(graph.diff(repeated.graph()).is_empty());
        assert!(result.diff(&repeated).is_empty());
        assert!(result.diagnostics().is_empty());
    }

    #[test]
    fn metadata_extends_edges_do_not_replace_metadata_nodes() {
        let root = create_edt_project();
        add_adopted_document_descriptor(
            &root,
            "SalesExtension",
            "bbbbbbbb-bbbb-cccc-dddd-eeeeeeeeeeee",
            "SalesExtension",
            "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee",
        );

        let graph = FileSystemEdtSemanticGraphBuilder
            .build_graph(root.path())
            .expect("graph must build");

        assert_eq!(
            graph
                .nodes_by_kind(NodeKind::Metadata(MetadataKind::Document))
                .into_iter()
                .filter(|node| matches!(node.name().as_str(), "Sales" | "SalesExtension"))
                .count(),
            2
        );
        assert!(graph.edges().any(|edge| edge.kind() == EdgeKind::Extends));
    }

    #[test]
    fn normal_metadata_does_not_emit_extends_edge() {
        let root = create_edt_project();

        let graph = FileSystemEdtSemanticGraphBuilder
            .build_graph(root.path())
            .expect("graph must build");

        assert!(graph.edges().all(|edge| edge.kind() != EdgeKind::Extends));
    }

    #[test]
    fn missing_metadata_extension_target_is_skipped() {
        let root = create_edt_project();
        add_adopted_document_descriptor(
            &root,
            "SalesExtension",
            "bbbbbbbb-bbbb-cccc-dddd-eeeeeeeeeeee",
            "SalesExtension",
            "99999999-bbbb-cccc-dddd-eeeeeeeeeeee",
        );

        let result = FileSystemEdtSemanticGraphBuilder
            .build_graph_with_diagnostics(root.path())
            .expect("graph must build");

        assert!(
            result
                .graph()
                .edges()
                .all(|edge| edge.kind() != EdgeKind::Extends)
        );
        assert!(result.diagnostics().is_empty());
    }

    #[test]
    fn incompatible_metadata_extension_target_kind_is_skipped() {
        let root = create_edt_project();
        add_adopted_document_descriptor(
            &root,
            "SalesExtension",
            "bbbbbbbb-bbbb-cccc-dddd-eeeeeeeeeeee",
            "SalesExtension",
            "11111111-aaaa-bbbb-cccc-222222222222",
        );

        let result = FileSystemEdtSemanticGraphBuilder
            .build_graph_with_diagnostics(root.path())
            .expect("graph must build");

        assert!(
            result
                .graph()
                .edges()
                .all(|edge| edge.kind() != EdgeKind::Extends)
        );
        assert!(result.diagnostics().is_empty());
    }

    #[test]
    fn composite_metadata_type_creates_one_edge_per_distinct_target() {
        let root = create_edt_project();
        replace_document_descriptor(&root, &document_with_composite_reference());

        let result = FileSystemEdtSemanticGraphBuilder
            .build_graph_with_diagnostics(root.path())
            .expect("graph must build");
        let graph = result.graph();
        let company_id = company_attribute_id();
        let products = graph
            .nodes_by_kind(NodeKind::Metadata(MetadataKind::Catalog))
            .into_iter()
            .find(|node| node.name().as_str() == "Products")
            .expect("Products catalog must exist");
        let sales = graph
            .nodes_by_kind(NodeKind::Metadata(MetadataKind::Document))
            .into_iter()
            .find(|node| node.name().as_str() == "Sales")
            .expect("Sales document must exist");
        let targets = graph
            .outgoing_by_kind(&company_id, EdgeKind::References)
            .into_iter()
            .map(|edge| edge.target().clone())
            .collect::<Vec<_>>();
        let dependency_targets = graph
            .outgoing_by_kind(&company_id, EdgeKind::DependsOn)
            .into_iter()
            .map(|edge| edge.target().clone())
            .collect::<Vec<_>>();

        assert_eq!(targets.len(), 2);
        assert!(targets.contains(products.id()));
        assert!(targets.contains(sales.id()));
        assert_eq!(dependency_targets.len(), 2);
        assert!(dependency_targets.contains(products.id()));
        assert!(dependency_targets.contains(sales.id()));
        assert!(result.diagnostics().is_empty());
    }

    #[test]
    fn primitive_metadata_type_does_not_emit_depends_on_edge() {
        let root = create_edt_project();
        replace_document_descriptor(&root, &document_with_reference("String"));

        let result = FileSystemEdtSemanticGraphBuilder
            .build_graph_with_diagnostics(root.path())
            .expect("primitive type must not fail graph build");
        let company_id = company_attribute_id();

        assert!(result.diagnostics().is_empty());
        assert!(
            result
                .graph()
                .outgoing_by_kind(&company_id, EdgeKind::References)
                .is_empty()
        );
        assert!(
            result
                .graph()
                .outgoing_by_kind(&company_id, EdgeKind::DependsOn)
                .is_empty()
        );
    }

    #[test]
    fn duplicate_metadata_type_reference_creates_one_depends_on_edge() {
        let root = create_edt_project();
        replace_document_descriptor(
            &root,
            &document_with_duplicate_reference("CatalogRef.Products"),
        );

        let result = FileSystemEdtSemanticGraphBuilder
            .build_graph_with_diagnostics(root.path())
            .expect("duplicate resolved reference must not fail graph build");
        let company_id = company_attribute_id();

        assert_eq!(
            result
                .graph()
                .outgoing_by_kind(&company_id, EdgeKind::References)
                .len(),
            1
        );
        assert_eq!(
            result
                .graph()
                .outgoing_by_kind(&company_id, EdgeKind::DependsOn)
                .len(),
            1
        );
        assert!(result.diagnostics().is_empty());
    }

    #[test]
    fn build_result_report_counts_resolved_references() {
        let root = create_edt_project();

        let result = FileSystemEdtSemanticGraphBuilder
            .build_graph_with_diagnostics(root.path())
            .expect("graph must build");
        let report = result.report();

        assert_eq!(result.reference_statistics().total(), 5);
        assert_eq!(result.reference_statistics().resolved(), 5);
        assert_eq!(result.reference_statistics().outcome_total(), 5);
        assert_eq!(result.reference_statistics().with_provenance(), 5);
        assert_eq!(report.graph().total_nodes(), result.graph().node_count());
        assert_eq!(report.graph().total_edges(), result.graph().edge_count());
        assert_eq!(report.graph().total_diagnostics(), 0);
        assert_eq!(report.resolution().total(), 5);
        assert_eq!(report.resolution().resolved(), 5);
        assert_eq!(report.resolution().resolution_rate().numerator(), 5);
        assert_eq!(report.resolution().resolution_rate().denominator(), 5);
        assert_eq!(report.provenance().references_with_provenance(), 5);
    }

    #[test]
    fn build_result_report_counts_failed_references_and_diagnostics() {
        let root = create_edt_project();
        replace_document_descriptor(
            &root,
            &document_with_reference("CatalogRef.MissingProducts"),
        );

        let result = FileSystemEdtSemanticGraphBuilder
            .build_graph_with_diagnostics(root.path())
            .expect("recoverable missing target must not fail graph build");
        let report = result.report();

        assert_eq!(result.reference_statistics().total(), 4);
        assert_eq!(result.reference_statistics().resolved(), 3);
        assert_eq!(result.reference_statistics().unresolved(), 1);
        assert_eq!(result.reference_statistics().outcome_total(), 4);
        assert_eq!(report.graph().total_diagnostics(), 1);
        assert_eq!(report.graph().recoverable_diagnostics(), 1);
        assert_eq!(
            report.diagnostics().by_code()[&SemanticDiagnosticCode::ReferenceUnresolved],
            1
        );
        assert_eq!(
            report.diagnostics().by_kind()[&SemanticDiagnosticKind::UnresolvedTarget],
            1
        );
        assert_eq!(report.diagnostics().with_provenance(), 1);
        assert_eq!(report.resolution().total(), 4);
        assert_eq!(report.resolution().resolved(), 3);
        assert_eq!(report.resolution().unresolved(), 1);
        assert_eq!(report.resolution().resolution_rate().numerator(), 3);
        assert_eq!(report.resolution().resolution_rate().denominator(), 4);
    }

    #[test]
    fn build_result_diff_compares_graph_diagnostics_and_statistics() {
        let previous_root = create_edt_project();
        replace_document_descriptor(
            &previous_root,
            &document_with_reference("CatalogRef.MissingProducts"),
        );
        let current_root = create_edt_project();

        let previous = FileSystemEdtSemanticGraphBuilder
            .build_graph_with_diagnostics(previous_root.path())
            .expect("previous graph must build");
        let current = FileSystemEdtSemanticGraphBuilder
            .build_graph_with_diagnostics(current_root.path())
            .expect("current graph must build");
        let diff = previous.diff(&current);

        assert!(!diff.graph().added_edges().is_empty());
        assert_eq!(diff.diagnostics().removed().len(), 1);
        assert!(diff.diagnostics().added().is_empty());
        assert!(diff.resolution().changed_metrics().iter().any(|change| {
            *change.key() == oneagent_graph::ResolutionStatisticsMetric::ResolvedReferences
        }));
        assert!(diff.summary().edge_changes() > 0);
        assert_eq!(diff.summary().diagnostic_changes(), 1);
    }

    #[test]
    fn build_graph_convenience_returns_graph_without_diagnostics() {
        let root = create_edt_project();
        replace_document_descriptor(
            &root,
            &document_with_reference("CatalogRef.MissingProducts"),
        );

        let graph = FileSystemEdtSemanticGraphBuilder
            .build_graph(root.path())
            .expect("recoverable diagnostics must not fail compatibility graph build");

        assert!(graph.node(&company_attribute_id()).is_some());
    }

    #[test]
    fn missing_metadata_reference_target_is_reported() {
        let root = create_edt_project();
        replace_document_descriptor(
            &root,
            &document_with_reference("CatalogRef.MissingProducts"),
        );

        let result = FileSystemEdtSemanticGraphBuilder
            .build_graph_with_diagnostics(root.path())
            .expect("recoverable missing target must not fail graph build");
        let diagnostics = result.diagnostics();
        let graph = result.graph();
        let company_id = company_attribute_id();

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].code(),
            SemanticDiagnosticCode::ReferenceUnresolved
        );
        assert_eq!(
            diagnostics[0].kind(),
            SemanticDiagnosticKind::UnresolvedTarget
        );
        assert_eq!(diagnostics[0].severity(), SemanticDiagnosticSeverity::Error);
        assert_eq!(diagnostics[0].source_node(), Some(&company_id));
        assert_eq!(
            diagnostics[0].expected_kinds(),
            &[NodeKind::Metadata(MetadataKind::Catalog)]
        );
        assert_eq!(diagnostics[0].provenance().len(), 1);
        assert!(matches!(
            diagnostics[0].reference(),
            SemanticReference::Name(name) if name.as_str() == "MissingProducts"
        ));
        assert!(graph.node(&company_id).is_some());
        assert!(
            graph
                .outgoing_by_kind(&company_id, EdgeKind::References)
                .is_empty()
        );
        assert!(
            graph
                .outgoing_by_kind(&company_id, EdgeKind::DependsOn)
                .is_empty()
        );

        let product_dimension = graph
            .nodes_by_kind(NodeKind::Dimension)
            .into_iter()
            .find(|node| node.name().as_str() == "Product")
            .expect("Product dimension must exist");
        assert_eq!(
            graph
                .outgoing_by_kind(product_dimension.id(), EdgeKind::References)
                .len(),
            1
        );
        assert_eq!(
            graph
                .outgoing_by_kind(product_dimension.id(), EdgeKind::DependsOn)
                .len(),
            1
        );
    }

    #[test]
    fn ambiguous_metadata_reference_target_is_reported() {
        let root = create_edt_project();
        add_catalog_descriptor(
            &root,
            "ProductsCopy",
            "22222222-aaaa-bbbb-cccc-333333333333",
            "Products",
        );

        let result = FileSystemEdtSemanticGraphBuilder
            .build_graph_with_diagnostics(root.path())
            .expect("recoverable ambiguity must not fail graph build");
        let diagnostics = result.diagnostics();
        let company_id = company_attribute_id();

        assert_eq!(diagnostics.len(), 3);
        assert!(diagnostics.iter().all(|diagnostic| {
            diagnostic.code() == SemanticDiagnosticCode::ReferenceAmbiguous
                && diagnostic.kind() == SemanticDiagnosticKind::AmbiguousTarget
                && diagnostic.severity() == SemanticDiagnosticSeverity::Error
        }));
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.source_node() == Some(&company_id))
        );
        assert!(
            result
                .graph()
                .outgoing_by_kind(&company_id, EdgeKind::References)
                .is_empty()
        );
        assert!(
            result
                .graph()
                .outgoing_by_kind(&company_id, EdgeKind::DependsOn)
                .is_empty()
        );
    }

    #[test]
    fn incompatible_metadata_reference_target_kind_is_reported() {
        let root = create_edt_project();
        replace_document_descriptor(&root, &document_with_reference("DocumentRef.Products"));

        let result = FileSystemEdtSemanticGraphBuilder
            .build_graph_with_diagnostics(root.path())
            .expect("recoverable incompatible kind must not fail graph build");
        let diagnostics = result.diagnostics();
        let company_id = company_attribute_id();

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].code(),
            SemanticDiagnosticCode::ReferenceIncompatibleKind
        );
        assert_eq!(
            diagnostics[0].kind(),
            SemanticDiagnosticKind::IncompatibleTargetKind
        );
        assert_eq!(
            diagnostics[0].actual_kind(),
            Some(NodeKind::Metadata(MetadataKind::Catalog))
        );
        assert_eq!(
            diagnostics[0].expected_kinds(),
            &[NodeKind::Metadata(MetadataKind::Document)]
        );
        assert!(
            result
                .graph()
                .outgoing_by_kind(&company_id, EdgeKind::References)
                .is_empty()
        );
        assert!(
            result
                .graph()
                .outgoing_by_kind(&company_id, EdgeKind::DependsOn)
                .is_empty()
        );
    }

    #[test]
    fn duplicate_identical_reference_diagnostic_is_deduplicated() {
        let root = create_edt_project();
        replace_document_descriptor(
            &root,
            &document_with_duplicate_reference("CatalogRef.MissingProducts"),
        );

        let result = FileSystemEdtSemanticGraphBuilder
            .build_graph_with_diagnostics(root.path())
            .expect("recoverable duplicate missing reference must not fail graph build");

        assert_eq!(result.diagnostics().len(), 1);
        assert_eq!(
            result.diagnostics()[0].code(),
            SemanticDiagnosticCode::ReferenceUnresolved
        );
    }

    #[test]
    fn different_sources_to_same_missing_target_create_distinct_diagnostics() {
        let root = create_edt_project();
        replace_document_descriptor(&root, &document_with_two_missing_references(false));

        let result = FileSystemEdtSemanticGraphBuilder
            .build_graph_with_diagnostics(root.path())
            .expect("recoverable missing references must not fail graph build");

        assert_eq!(result.diagnostics().len(), 2);
        assert_ne!(
            result.diagnostics()[0].source_node(),
            result.diagnostics()[1].source_node()
        );
    }

    #[test]
    fn diagnostic_order_does_not_depend_on_reference_collection_order() {
        let root = create_edt_project();
        replace_document_descriptor(&root, &document_with_two_missing_references(false));

        let normal_diagnostics = FileSystemEdtSemanticGraphBuilder
            .build_graph_with_diagnostics(root.path())
            .expect("normal graph must build")
            .diagnostics()
            .to_vec();
        replace_document_descriptor(&root, &document_with_two_missing_references(true));
        let reversed_diagnostics = FileSystemEdtSemanticGraphBuilder
            .build_graph_with_diagnostics(root.path())
            .expect("reversed graph must build")
            .diagnostics()
            .to_vec();

        assert_eq!(normal_diagnostics, reversed_diagnostics);
    }

    #[test]
    fn metadata_object_contains_structure_and_modules() {
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

        // Two attributes, five standard attributes, one tabular section, one form,
        // one command, object module and manager module.
        assert_eq!(children.len(), 12);

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
                .filter(|kind| **kind == NodeKind::StandardAttribute)
                .count(),
            5
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
                .filter(|kind| **kind == NodeKind::Command)
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
    fn emits_accounting_register_resource_as_measure_node() {
        let root = create_edt_project();
        add_accounting_register_descriptor(&root);

        let first = FileSystemEdtSemanticGraphBuilder
            .build_graph_with_diagnostics(root.path())
            .expect("graph with accounting register measure must build");
        let second = FileSystemEdtSemanticGraphBuilder
            .build_graph_with_diagnostics(root.path())
            .expect("repeated graph build must succeed");
        let measure_id = NodeId::new("cdcdcdcd-1111-2222-3333-444444444444");
        let measure = first
            .graph()
            .query()
            .node(&measure_id)
            .expect("accounting register resource must emit a measure node");
        let repeated_measure = second
            .graph()
            .query()
            .node(&measure_id)
            .expect("repeated build must preserve measure identity");
        let resource = first
            .graph()
            .nodes_by_kind(NodeKind::Resource)
            .into_iter()
            .find(|node| node.name().as_str() == "Quantity")
            .expect("accumulation register resource must remain a resource");
        let coverage = first.coverage_report();
        let capability = coverage
            .edt_pipeline()
            .capability(SemanticCoverageCapabilityId::SemanticNode(
                NodeKind::Measure,
            ))
            .expect("measure node coverage must exist");
        let ownership = coverage
            .edt_pipeline()
            .capability(SemanticCoverageCapabilityId::OwnershipRelation(
                NodeKind::Measure,
            ))
            .expect("measure ownership coverage must remain registered");

        assert_eq!(measure.id().as_str(), measure_id.as_str());
        assert_eq!(measure.id(), repeated_measure.id());
        assert_eq!(measure.name().as_str(), "Amount");
        assert_eq!(measure.kind(), NodeKind::Measure);
        assert_eq!(resource.kind(), NodeKind::Resource);
        assert_eq!(first.graph().nodes_by_kind(NodeKind::Measure).len(), 1);
        assert_eq!(measure.provenance().len(), 1);
        assert_eq!(measure.provenance()[0].origin(), FactOrigin::Declared);
        assert!(
            measure.provenance()[0]
                .source()
                .expect("measure provenance source must exist")
                .as_str()
                .ends_with(
                    "/src/AccountingRegisters/GeneralLedger/GeneralLedger.mdo#metadata_object=abababab-1111-2222-3333-444444444444;member=resource:cdcdcdcd-1111-2222-3333-444444444444"
                )
        );
        assert_eq!(capability.status(), SemanticCoverageStatus::Supported);
        assert!(capability.missing_evidence().is_empty());
        assert_eq!(ownership.status(), SemanticCoverageStatus::Supported);
        assert!(ownership.missing_evidence().is_empty());
        assert!(
            coverage
                .edt_pipeline()
                .gaps()
                .iter()
                .all(|gap| gap.capability_id() != ownership.id())
        );
        assert!(first.graph().diff(second.graph()).is_empty());
        assert!(first.diff(&second).is_empty());
    }

    #[test]
    fn emits_accounting_register_measure_with_owner() {
        let root = create_edt_project();
        add_accounting_register_descriptor(&root);

        let first = FileSystemEdtSemanticGraphBuilder
            .build_graph_with_diagnostics(root.path())
            .expect("graph with accounting register measure must build");
        let second = FileSystemEdtSemanticGraphBuilder
            .build_graph_with_diagnostics(root.path())
            .expect("repeated graph build must succeed");
        let graph = first.graph();
        let repeated_graph = second.graph();
        let register_id = NodeId::new("abababab-1111-2222-3333-444444444444");
        let measure_id = NodeId::new("cdcdcdcd-1111-2222-3333-444444444444");
        let measure = graph
            .query()
            .node(&measure_id)
            .expect("accounting register resource must emit a measure node");
        let owner = graph
            .query()
            .owner(&measure_id)
            .expect("accounting register must own the measure");
        let owner_edges = graph.query().owner_edges(&measure_id);
        let repeated_owner_edges = repeated_graph.query().owner_edges(&measure_id);
        let register_children = graph
            .query()
            .children_by_kind(&register_id, NodeKind::Measure);

        assert!(first.diagnostics().is_empty());
        assert_eq!(measure.kind(), NodeKind::Measure);
        assert_eq!(owner.id().as_str(), register_id.as_str());
        assert_eq!(
            owner.kind(),
            NodeKind::Metadata(MetadataKind::AccountingRegister)
        );
        assert_eq!(register_children.len(), 1);
        assert_eq!(register_children[0].id().as_str(), measure_id.as_str());
        assert_eq!(owner_edges.len(), 1);
        assert_eq!(repeated_owner_edges.len(), 1);
        assert_eq!(owner_edges[0].source().as_str(), register_id.as_str());
        assert_eq!(owner_edges[0].target().as_str(), measure_id.as_str());
        assert_eq!(owner_edges[0].kind(), EdgeKind::Contains);
        assert_eq!(owner_edges[0].source(), repeated_owner_edges[0].source());
        assert_eq!(owner_edges[0].target(), repeated_owner_edges[0].target());
        assert_eq!(owner_edges[0].kind(), repeated_owner_edges[0].kind());
        assert_eq!(owner_edges[0].provenance().len(), 1);
        assert_eq!(
            owner_edges[0].provenance()[0].origin(),
            FactOrigin::Declared
        );
        assert!(
            owner_edges[0].provenance()[0]
                .source()
                .expect("measure ownership provenance source must exist")
                .as_str()
                .ends_with(
                    "/src/AccountingRegisters/GeneralLedger/GeneralLedger.mdo#metadata_object=abababab-1111-2222-3333-444444444444;edge=contains;source=abababab-1111-2222-3333-444444444444;target=cdcdcdcd-1111-2222-3333-444444444444"
                )
        );
        assert!(first.graph().validate().is_valid());
        assert!(first.graph().diff(second.graph()).is_empty());
        assert!(first.diff(&second).is_empty());
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

    #[test]
    fn ignores_generic_top_level_form_and_preserves_subordinate_form_semantics() {
        let root = create_edt_project();
        let common_form_directory = root.path().join("src/CommonForms/Workspace");
        fs::create_dir_all(&common_form_directory).expect("common form directory must be created");
        fs::write(
            common_form_directory.join("Workspace.mdo"),
            r#"<?xml version="1.0" encoding="UTF-8"?>
<mdclass:CommonForm
    xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass"
    uuid="ffffffff-1111-2222-3333-444444444444">
    <name>Workspace</name>
</mdclass:CommonForm>
"#,
        )
        .expect("common form descriptor must be created");
        let top_level_form_directory = root.path().join("src/Forms/UnexpectedForm");
        fs::create_dir_all(&top_level_form_directory)
            .expect("unsupported top-level form directory must be created");
        fs::write(
            top_level_form_directory.join("UnexpectedForm.mdo"),
            r#"<?xml version="1.0" encoding="UTF-8"?>
<mdclass:DocumentForm
    xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass"
    uuid="eeeeeeee-1111-2222-3333-444444444444">
    <name>UnexpectedForm</name>
</mdclass:DocumentForm>
"#,
        )
        .expect("unsupported top-level form descriptor must be created");

        let first = FileSystemEdtSemanticGraphBuilder
            .build_graph_with_diagnostics(root.path())
            .expect("graph must build while ignoring unsupported top-level form directory");
        let second = FileSystemEdtSemanticGraphBuilder
            .build_graph_with_diagnostics(root.path())
            .expect("repeated graph build must succeed");
        let graph = first.graph();
        let common_form = graph
            .query()
            .node(&NodeId::new("ffffffff-1111-2222-3333-444444444444"))
            .expect("common form must remain a top-level metadata entity");
        let document = graph
            .nodes_by_kind(NodeKind::Metadata(MetadataKind::Document))
            .into_iter()
            .next()
            .expect("document node must exist");
        let form_id = NodeId::new("aaaaaaaa-4444-4444-4444-444444444444");
        let form = graph
            .query()
            .node(&form_id)
            .expect("subordinate form must retain its stable UUID");
        let owner = graph
            .query()
            .owner(&form_id)
            .expect("document must own subordinate form");
        let forms = graph
            .query()
            .children_by_kind(&NodeId::new(document.id().as_str()), NodeKind::Form);

        assert!(
            graph
                .nodes_by_kind(NodeKind::Metadata(MetadataKind::Form))
                .is_empty()
        );
        assert_eq!(
            common_form.kind(),
            NodeKind::Metadata(MetadataKind::CommonForm)
        );
        assert_eq!(form.kind(), NodeKind::Form);
        assert_eq!(form.name().as_str(), "DocumentForm");
        assert_eq!(owner.id(), document.id());
        assert_eq!(forms, vec![form]);
        assert!(
            form.provenance()[0]
                .source()
                .expect("form provenance source must exist")
                .as_str()
                .ends_with(
                    "/src/Documents/Sales/Sales.mdo#metadata_object=aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee;member=form:aaaaaaaa-4444-4444-4444-444444444444"
                )
        );
        assert!(first.validate().is_valid());
        assert!(graph.diff(second.graph()).is_empty());
        assert!(first.diff(&second).is_empty());
    }
    #[test]
    fn metadata_object_contains_command() {
        let root = create_edt_project();

        let graph = FileSystemEdtSemanticGraphBuilder
            .build_graph(root.path())
            .expect("graph must build");

        let document = graph
            .nodes_by_kind(NodeKind::Metadata(MetadataKind::Document))
            .into_iter()
            .next()
            .expect("document node must exist");

        let command = graph
            .nodes_by_kind(NodeKind::Command)
            .into_iter()
            .find(|node| node.name().as_str() == "PostAndClose")
            .expect("document command node must exist");

        let children = graph.outgoing_by_kind(document.id(), EdgeKind::Contains);

        assert!(children.iter().any(|edge| edge.target() == command.id()));
    }

    #[test]
    fn semantic_resolution_finds_metadata_members_after_graph_build() {
        let root = create_edt_project();

        let graph = FileSystemEdtSemanticGraphBuilder
            .build_graph(root.path())
            .expect("graph must build");
        let document = graph
            .nodes_by_kind(NodeKind::Metadata(MetadataKind::Document))
            .into_iter()
            .next()
            .expect("document node must exist");
        let index = graph.resolution_index();

        let company = index
            .resolve_child(
                document.id(),
                &EntityName::new("Company").expect("name must be valid"),
            )
            .expect("Company attribute must resolve under document");
        let owner = index
            .resolve_owner(company.id())
            .expect("Company owner must resolve");
        let ambiguous = index
            .resolve_name(&EntityName::new("Warehouse").expect("name must be valid"))
            .expect_err("Warehouse name must be ambiguous across document and register");

        assert_eq!(company.kind(), NodeKind::Attribute);
        assert_eq!(owner.id(), document.id());
        assert_eq!(
            ambiguous,
            ResolutionError::AmbiguousTarget {
                reference: SemanticReference::Name(
                    EntityName::new("Warehouse").expect("name must be valid")
                ),
                candidates: vec![
                    oneagent_common::EntityId::new("66666666-6666-6666-6666-666666666666")
                        .expect("identifier must be valid"),
                    oneagent_common::EntityId::new("aaaaaaaa-2222-2222-2222-222222222222")
                        .expect("identifier must be valid"),
                ],
            }
        );
    }

    #[test]
    fn build_result_validation_uses_graph_validation_api() {
        let result = EdtSemanticGraphBuildResult::new(SemanticGraph::new(), Vec::new());

        let validation = result.validate();

        assert!(validation.is_valid());
        assert!(validation.issues().is_empty());
    }

    #[test]
    fn build_result_coverage_report_combines_static_and_observed_coverage() {
        let root = create_edt_project();
        let result = FileSystemEdtSemanticGraphBuilder
            .build_graph_with_diagnostics(root.path())
            .expect("graph must build");

        let coverage = result.coverage_report();

        assert!(coverage.graph_domain().is_consistent());
        assert!(coverage.edt_pipeline().is_consistent());
        assert_eq!(
            coverage.observed().total_nodes(),
            result.graph().node_count()
        );
        assert_eq!(
            coverage.observed().total_edges(),
            result.graph().edge_count()
        );
        assert_eq!(
            coverage.observed().nodes()[&NodeKind::Attribute].without_provenance(),
            0
        );
        assert_eq!(
            coverage.observed().edges()[&EdgeKind::References].without_provenance(),
            0
        );
        assert_eq!(
            coverage.observed().edges()[&EdgeKind::DependsOn].without_provenance(),
            0
        );
        assert_eq!(
            coverage
                .edt_pipeline()
                .capability(SemanticCoverageCapabilityId::SemanticNode(
                    NodeKind::StandardAttribute,
                ))
                .expect("standard attribute coverage must exist")
                .status(),
            SemanticCoverageStatus::Supported
        );
        assert_eq!(
            coverage
                .edt_pipeline()
                .capability(SemanticCoverageCapabilityId::MetadataReference(
                    SemanticReferenceCapability::MetadataType(MetadataKind::Catalog),
                ))
                .expect("catalog reference coverage must exist")
                .status(),
            SemanticCoverageStatus::Supported
        );
        assert!(coverage.validation().is_valid());
        assert_eq!(
            coverage.build_report().graph().total_nodes(),
            result.graph().node_count()
        );
    }
}
