//! Adapter for reading `1C:EDT` project sources.

mod bsl_graph;
mod coverage;
mod metadata_object;
mod metadata_structure;
mod module_reader;

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
    Confidence, EdgeKind, FactOrigin, NodeKind, ProducerId, Provenance, ResolutionError,
    ResolutionState, SemanticDiagnostic, SemanticGraph, SemanticGraphBuildDiff,
    SemanticGraphReport, SemanticGraphValidationResult, SemanticGraphValidator, SemanticReference,
    SemanticReferenceOutcome, SemanticReferenceStatistics,
};
use oneagent_metadata::MetadataKind;
use std::collections::{BTreeMap, BTreeSet};

const EDT_GRAPH_PRODUCER: &str = "oneagent.edt.semantic-graph-builder";

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

            let collected =
                collect_top_level_metadata(&entry.path(), kind, &configuration_id, &mut graph)?;
            configuration_modules.extend(collected.modules);
            metadata_references.extend(collected.references);
        }

        resolve_metadata_references(
            &mut graph,
            &metadata_references,
            &mut diagnostics,
            &mut reference_statistics,
        )?;

        bsl_graph::add_configuration_module_symbols_with_diagnostics(
            &mut graph,
            &configuration_modules,
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

fn collect_top_level_metadata(
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
        let object = collect_metadata_object(&object_directory, kind, configuration_id, graph)?;

        collected.modules.extend(object.modules);
        collected.references.extend(object.references);
    }

    Ok(collected)
}

fn collect_metadata_object(
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

    for child in structure_reader
        .read_children(&descriptor)
        .map_err(EdtGraphError::MetadataStructure)?
    {
        collect_metadata_child(graph, &descriptor, &child, &mut collected.references)?;
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

fn collect_metadata_child(
    graph: &mut SemanticGraph,
    descriptor: &EdtMetadataObjectDescriptor,
    child: &EdtMetadataChildDescriptor,
    references: &mut BTreeSet<PendingMetadataReference>,
) -> Result<(), EdtGraphError> {
    let child_source = metadata_child_source_id(descriptor, child)?;

    insert_node(
        graph,
        child.id().clone(),
        child.name().clone(),
        child.kind().node_kind(),
        declared_provenance(child_source),
    );

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

    if matches!(
        child.kind(),
        EdtMetadataChildKind::Attribute
            | EdtMetadataChildKind::Dimension
            | EdtMetadataChildKind::Resource
    ) {
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
        insert_edge(
            graph,
            reference.source_id.clone(),
            target_id,
            EdgeKind::References,
            resolved_provenance(metadata_reference_source_id(&reference)?),
        )?;
    }

    Ok(())
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
    use oneagent_common::EntityName;
    use oneagent_graph::{
        EdgeKind, FactOrigin, NodeId, NodeKind, ResolutionError, ResolutionState,
        SemanticCoverageCapabilityId, SemanticCoverageStatus, SemanticDiagnosticCode,
        SemanticDiagnosticKind, SemanticDiagnosticSeverity, SemanticGraph, SemanticReference,
        SemanticReferenceCapability,
    };
    use oneagent_metadata::MetadataKind;
    use std::fs;
    use tempfile::tempdir;

    use super::{
        EdtSemanticGraphBuildResult, EdtSemanticGraphBuilder, FileSystemEdtSemanticGraphBuilder,
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

    fn add_common_command_descriptor(root: &tempfile::TempDir) {
        let directory = root.path().join("src/CommonCommands/RefreshData");
        fs::create_dir_all(&directory).expect("common command directory must be created");
        fs::write(directory.join("RefreshData.mdo"), COMMON_COMMAND_XML)
            .expect("common command descriptor must be created");
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

        assert_eq!(graph.node_count(), 19);
        assert_eq!(graph.edge_count(), 23);
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
    fn resolves_metadata_reference_edges() {
        let root = create_edt_project();

        let result = FileSystemEdtSemanticGraphBuilder
            .build_graph_with_diagnostics(root.path())
            .expect("graph must build");
        let graph = result.graph();
        let products = graph
            .nodes_by_kind(NodeKind::Metadata(MetadataKind::Catalog))
            .into_iter()
            .find(|node| node.name().as_str() == "Products")
            .expect("Products catalog must exist");
        let sales_document = graph
            .nodes_by_kind(NodeKind::Metadata(MetadataKind::Document))
            .into_iter()
            .find(|node| node.name().as_str() == "Sales")
            .expect("Sales document must exist");
        let company = graph
            .nodes_by_kind(NodeKind::Attribute)
            .into_iter()
            .find(|node| node.name().as_str() == "Company")
            .expect("Company attribute must exist");
        let warehouse = graph
            .nodes_by_kind(NodeKind::Attribute)
            .into_iter()
            .find(|node| node.name().as_str() == "Warehouse")
            .expect("Warehouse attribute must exist");
        let product_dimension = graph
            .nodes_by_kind(NodeKind::Dimension)
            .into_iter()
            .find(|node| node.name().as_str() == "Product")
            .expect("Product dimension must exist");
        let quantity_resource = graph
            .nodes_by_kind(NodeKind::Resource)
            .into_iter()
            .find(|node| node.name().as_str() == "Quantity")
            .expect("Quantity resource must exist");

        let company_references = graph.outgoing_by_kind(company.id(), EdgeKind::References);
        let warehouse_references = graph.outgoing_by_kind(warehouse.id(), EdgeKind::References);
        let product_references =
            graph.outgoing_by_kind(product_dimension.id(), EdgeKind::References);
        let quantity_references =
            graph.outgoing_by_kind(quantity_resource.id(), EdgeKind::References);
        let incoming_references = graph
            .incoming(products.id())
            .into_iter()
            .filter(|edge| edge.kind() == EdgeKind::References)
            .collect::<Vec<_>>();

        assert_eq!(company_references.len(), 1);
        assert_eq!(warehouse_references.len(), 1);
        assert_eq!(product_references.len(), 1);
        assert_eq!(quantity_references.len(), 1);
        assert_eq!(incoming_references.len(), 3);
        assert_eq!(company_references[0].target(), products.id());
        assert_eq!(warehouse_references[0].target(), products.id());
        assert_eq!(product_references[0].target(), products.id());
        assert_eq!(quantity_references[0].target(), sales_document.id());
        assert_eq!(company.provenance().len(), 1);
        assert_eq!(company_references[0].provenance().len(), 1);
        assert_eq!(
            company_references[0].provenance()[0].origin(),
            FactOrigin::Resolved
        );
        assert_eq!(
            company_references[0].provenance()[0].resolution(),
            ResolutionState::Resolved
        );
        assert!(
            company_references[0].provenance()[0]
                .source()
                .expect("reference edge source must exist")
                .as_str()
                .ends_with(
                    "/src/Documents/Sales/Sales.mdo#metadata_object=aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee;edge=references;source=aaaaaaaa-1111-1111-1111-111111111111;role=type;target_kind=catalog;target_name=Products"
                )
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

        assert_eq!(targets.len(), 2);
        assert!(targets.contains(products.id()));
        assert!(targets.contains(sales.id()));
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

        // Two attributes, one tabular section, one form, one command,
        // object module and manager module.
        assert_eq!(children.len(), 7);

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
            coverage
                .edt_pipeline()
                .capability(SemanticCoverageCapabilityId::SemanticNode(
                    NodeKind::StandardAttribute,
                ))
                .expect("standard attribute coverage must exist")
                .status(),
            SemanticCoverageStatus::Unsupported
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
