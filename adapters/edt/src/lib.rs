//! Adapter for reading `1C:EDT` project sources.

mod bsl_graph;
mod command_parameter;
mod coverage;
mod event_subscription;
mod event_subscription_emission;
// The resolver is a production prerequisite for later Event Subscription emission.
#[allow(dead_code)]
mod event_subscription_resolution;
mod form_navigation;
mod form_navigation_emission;
mod metadata_object;
mod metadata_structure;
mod module_reader;
// The resolver is a production prerequisite for the later Reads emission task.
#[allow(dead_code)]
mod query_source_resolution;
mod report_data_composition;
mod report_data_composition_emission;
mod role_rights;
mod service_descriptor;
mod subsystem_content;
mod subsystem_hierarchy;
mod writes;
mod writes_emission;
mod writes_resolution;
mod xdto_package;
mod xdto_service_emission;

pub use metadata_object::{
    EdtMetadataObjectDescriptor, EdtMetadataObjectError, EdtMetadataObjectReader,
    FileSystemEdtMetadataObjectReader,
};

pub use report_data_composition::{
    EdtDataCompositionDataSet, EdtDataCompositionDataSource, EdtDataCompositionField,
    EdtDataCompositionObservation, EdtDataCompositionObservationKind,
    EdtDataCompositionSchemaDescriptor, EdtDataCompositionSourceReason,
    EdtReportDataCompositionDescriptor, EdtReportDataCompositionError,
    EdtReportDataCompositionReader, FileSystemEdtReportDataCompositionReader,
};

pub use event_subscription::{
    EdtEventSubscriptionDescriptor, EdtEventSubscriptionError, EdtEventSubscriptionHandler,
    EdtEventSubscriptionHandlerReason, EdtEventSubscriptionReader,
    EdtEventSubscriptionSourceContext, EdtEventSubscriptionSourceObservation,
    EdtEventSubscriptionSourceOutcomeKind, EdtEventSubscriptionSourceReason,
    FileSystemEdtEventSubscriptionReader,
};

pub use command_parameter::{
    EdtCommandParameterSourceKind, EdtCommandParameterTypeObservation,
    EdtCommandParameterTypeOutcomeKind, EdtCommandParameterTypeReason,
};

pub use metadata_structure::{
    EdtMetadataChildDescriptor, EdtMetadataChildKind, EdtMetadataReferenceDescriptor,
    EdtMetadataReferenceRole, EdtMetadataStructureError, EdtMetadataStructureReader,
    FileSystemEdtMetadataStructureReader,
};

pub use module_reader::{
    EdtModuleDescriptor, EdtModuleError, EdtModuleKind, EdtModuleLayoutObservation,
    EdtModuleLayoutOutcomeKind, EdtModuleLayoutRejectionReason, EdtModuleOwnerKind,
    EdtModuleReader, FileSystemEdtModuleReader,
};

pub use role_rights::{
    EdtRoleObjectRights, EdtRoleRightDeclaration, EdtRoleRightsDescriptor, EdtRoleRightsError,
    EdtRoleRightsReader, EdtRoleRowRestriction, FileSystemEdtRoleRightsReader,
};

pub use service_descriptor::{
    EdtHttpMethod, EdtHttpServiceDescriptor, EdtHttpUrlTemplate, EdtServiceDescriptorError,
    EdtServiceDescriptorReader, EdtWebServiceDescriptor, EdtWebServiceOperation,
    EdtWebServiceParameter, EdtWebServiceXdtoPackage, FileSystemEdtServiceDescriptorReader,
};

pub use subsystem_content::{
    EdtSubsystemContentDescriptor, EdtSubsystemContentError, EdtSubsystemContentReader,
    FileSystemEdtSubsystemContentReader,
};

pub use subsystem_hierarchy::{
    EdtSubsystemHierarchy, EdtSubsystemHierarchyDescriptor, EdtSubsystemHierarchyError,
    EdtSubsystemHierarchyReader, EdtSubsystemHierarchyRelation,
    FileSystemEdtSubsystemHierarchyReader,
};

pub use xdto_package::{
    EdtXdtoDeferredKind, EdtXdtoDeferredObservation, EdtXdtoPackageDescriptor, EdtXdtoPackageError,
    EdtXdtoPackageReader, EdtXdtoTypeDeclaration, FileSystemEdtXdtoPackageReader,
};

pub use bsl_graph::{
    AnalyzedBslModule, EdtBslGraphError, add_configuration_module_symbols, add_module_symbols,
    analyze_module,
};
pub use coverage::{EdtSemanticCoverageRegistry, EdtSemanticCoverageReport};
pub use form_navigation_emission::EdtFormNavigationEmissionError;

use oneagent_common::{EntityId, EntityName, SourceLocation, SourcePath};
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

impl FileSystemEdtConfigurationLoader {
    fn load_with_payload(
        project_root: &Path,
    ) -> Result<(Configuration, MetadataPayload), EdtLoadError> {
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
        let payload = MetadataPayload::new(CommonMetadataPayload::new(descriptor.synonym), None);

        Ok((
            Configuration::new(id, name, project_root, WorkspaceFormat::Edt),
            payload,
        ))
    }
}

impl EdtConfigurationLoader for FileSystemEdtConfigurationLoader {
    fn load(&self, project_root: &Path) -> Result<Configuration, EdtLoadError> {
        Self::load_with_payload(project_root).map(|(configuration, _)| configuration)
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

    use super::{FileSystemEdtConfigurationLoader, parse_configuration_descriptor};

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

        let compatibility_configuration =
            <FileSystemEdtConfigurationLoader as super::EdtConfigurationLoader>::load(
                &FileSystemEdtConfigurationLoader,
                root.path(),
            )
            .expect("compatibility configuration loader must succeed");
        let (configuration, payload) =
            FileSystemEdtConfigurationLoader::load_with_payload(root.path())
                .expect("configuration must load");

        assert_eq!(compatibility_configuration.id(), configuration.id());
        assert_eq!(compatibility_configuration.name(), configuration.name());
        assert_eq!(configuration.name().as_str(), "DemoConfiguration");
        assert_eq!(
            configuration.id().as_str(),
            "40d7b43a-b34f-4e6f-a756-6d7f0dc850f0"
        );
        assert_eq!(
            payload.common().synonym(),
            Some("Демонстрационная конфигурация")
        );
        assert_eq!(payload.specific(), None);
    }

    #[test]
    fn configuration_payload_distinguishes_absent_synonym() {
        let descriptor = parse_configuration_descriptor(
            r#"<mdclass:Configuration uuid="id"><name>Main</name></mdclass:Configuration>"#,
        )
        .expect("configuration without synonym must be valid");

        assert_eq!(descriptor.synonym, None);
    }

    #[test]
    fn rejects_malformed_configuration_synonym_xml() {
        let error = parse_configuration_descriptor(
            r#"<mdclass:Configuration uuid="id"><name>Main</name><synonym><content>Broken</synonym></mdclass:Configuration>"#,
        )
        .expect_err("malformed synonym XML must be rejected");

        assert!(matches!(error, super::EdtLoadError::MalformedXml(_)));
    }

    #[test]
    fn rejects_configuration_without_name() {
        let xml = r#"<mdclass:Configuration uuid="id" />"#;
        let error = parse_configuration_descriptor(xml).expect_err("missing name must be rejected");
        assert_eq!(error.to_string(), "EDT configuration name is missing");
    }
}

use oneagent_graph::{
    AccessRight, AccessRightRowRestriction, Confidence, DataSetPayloadError, EdgeKind, FactOrigin,
    GraphEdge, GraphNode, GraphNodePayload, GraphNodePayloadError, NodeKind, ProducerId,
    Provenance, ResolutionError, ResolutionState, SemanticDiagnostic, SemanticDiagnosticCode,
    SemanticDiagnosticKind, SemanticDiagnosticSeverity, SemanticGraph, SemanticGraphBuildDiff,
    SemanticGraphBuildSnapshot, SemanticGraphReport, SemanticGraphValidationResult,
    SemanticGraphValidator, SemanticReference, SemanticReferenceCategory, SemanticReferenceOutcome,
    SemanticReferenceRequest, SemanticReferenceRequestError, SemanticReferenceRequestId,
    SemanticReferenceRequestLedger, SemanticReferenceRequestOutcome, SemanticReferenceRequestQuery,
    SemanticReferenceStatistics, StandardAttribute, StandardAttributeKind,
};
use oneagent_metadata::{
    CommonMetadataPayload, DocumentMetadataPayload, MetadataKind, MetadataMemberPayload,
    MetadataPayload, MetadataRegisterRecord, MetadataSpecificPayload,
};
use std::collections::{BTreeMap, BTreeSet};

const EDT_GRAPH_PRODUCER: &str = "oneagent.edt.semantic-graph-builder";
const EDT_SUBSYSTEM_CONTENT_RESOLUTION_PRODUCER: &str = "oneagent.edt.subsystem-content-resolution";
const EDT_SUBSYSTEM_HIERARCHY_RESOLUTION_PRODUCER: &str =
    "oneagent.edt.subsystem-hierarchy-resolution";
const EDT_METADATA_REFERENCE_COLLECTION_PRODUCER: &str =
    "oneagent.edt.metadata-reference-collection";
const EDT_COMMAND_PARAMETER_REJECTION_PRODUCER: &str = "oneagent.edt.command-parameter-rejection";

/// Result of building an EDT semantic graph.
///
/// Recoverable semantic reference problems are returned as ordered diagnostics
/// while the graph contains every node and every edge that could be built
/// safely.
#[derive(Debug, Clone)]
pub struct EdtSemanticGraphBuildResult {
    graph: SemanticGraph,
    diagnostics: Vec<SemanticDiagnostic>,
    reference_requests: SemanticReferenceRequestLedger,
    legacy_reference_statistics: SemanticReferenceStatistics,
    reference_statistics: SemanticReferenceStatistics,
}

impl EdtSemanticGraphBuildResult {
    /// Creates an EDT semantic graph build result.
    #[must_use]
    pub const fn new(graph: SemanticGraph, diagnostics: Vec<SemanticDiagnostic>) -> Self {
        Self {
            graph,
            diagnostics,
            reference_requests: SemanticReferenceRequestLedger::new(),
            legacy_reference_statistics: SemanticReferenceStatistics::new(),
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
            reference_requests: SemanticReferenceRequestLedger::new(),
            legacy_reference_statistics: reference_statistics,
            reference_statistics,
        }
    }

    /// Creates a build result with canonical accepted requests and legacy observations.
    ///
    /// `legacy_reference_statistics` must exclude every accepted request in
    /// `reference_requests`.
    #[must_use]
    pub fn new_with_reference_requests(
        graph: SemanticGraph,
        diagnostics: Vec<SemanticDiagnostic>,
        reference_requests: SemanticReferenceRequestLedger,
        legacy_reference_statistics: SemanticReferenceStatistics,
    ) -> Self {
        let reference_statistics =
            SemanticReferenceStatistics::from_reference_requests(&reference_requests)
                .including_legacy_observations(legacy_reference_statistics);
        Self {
            graph,
            diagnostics,
            reference_requests,
            legacy_reference_statistics,
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

    /// Returns canonical accepted semantic reference requests in stable order.
    #[must_use]
    pub fn reference_requests(&self) -> &[SemanticReferenceRequest] {
        self.reference_requests.requests()
    }

    /// Returns an immutable query view over canonical reference requests.
    #[must_use]
    pub fn reference_request_query(&self) -> SemanticReferenceRequestQuery<'_> {
        self.reference_requests.query()
    }

    /// Builds a deterministic report for this EDT graph build result.
    ///
    /// The report combines graph metrics, recoverable semantic diagnostics and
    /// reference outcome counters captured during graph construction.
    #[must_use]
    pub fn report(&self) -> SemanticGraphReport {
        SemanticGraphReport::from_graph_diagnostics_reference_requests_and_legacy_observations(
            &self.graph,
            &self.diagnostics,
            &self.reference_requests,
            self.legacy_reference_statistics,
        )
    }

    /// Compares this EDT graph build result with a newer build result.
    #[must_use]
    pub fn diff(&self, current: &Self) -> SemanticGraphBuildDiff {
        SemanticGraphBuildDiff::between_with_reference_requests(
            SemanticGraphBuildSnapshot::with_legacy_observations(
                &self.graph,
                &self.diagnostics,
                &self.reference_requests,
                self.legacy_reference_statistics,
            ),
            SemanticGraphBuildSnapshot::with_legacy_observations(
                &current.graph,
                &current.diagnostics,
                &current.reference_requests,
                current.legacy_reference_statistics,
            ),
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
        SemanticGraphValidator::new()
            .validate_build_result_with_reference_requests_and_legacy_observations(
                &self.graph,
                &self.diagnostics,
                &self.reference_requests,
                self.legacy_reference_statistics,
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
        Self::build_graph_with_metadata_reference_scope(
            project_root,
            query_source_resolution::WorkspaceResolutionScope::Complete,
        )
    }
}

impl FileSystemEdtSemanticGraphBuilder {
    fn build_graph_with_metadata_reference_scope(
        project_root: &Path,
        metadata_reference_scope: query_source_resolution::WorkspaceResolutionScope,
    ) -> Result<EdtSemanticGraphBuildResult, EdtGraphError> {
        let (configuration, configuration_payload) =
            FileSystemEdtConfigurationLoader::load_with_payload(project_root)?;
        let mut graph = SemanticGraph::new();
        let mut collected_metadata = CollectedTopLevelMetadata::default();
        let mut diagnostics = BTreeSet::new();
        let mut reference_statistics = SemanticReferenceStatistics::new();

        let configuration_id = configuration.id().clone();
        insert_configuration_node(
            &mut graph,
            project_root,
            &configuration,
            configuration_payload,
        )?;

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

            let collected = collect_supported_metadata_directory(
                project_root,
                &entry.path(),
                kind,
                &configuration_id,
                &mut graph,
            )?;
            collected_metadata.extend(collected)?;
        }

        report_data_composition_emission::emit_report_data_composition(
            project_root,
            &mut graph,
            &collected_metadata.report_data_composition,
            &mut diagnostics,
            &mut reference_statistics,
        )?;

        collected_metadata
            .modules
            .sort_by(|left, right| left.id().cmp(right.id()));

        resolve_metadata_extensions(&mut graph, &collected_metadata.extensions)?;
        emit_subsystem_composition(
            &mut graph,
            &collected_metadata.subsystem_hierarchy,
            &collected_metadata.subsystem_content,
            &mut diagnostics,
            &mut reference_statistics,
        )?;

        emit_rejected_command_parameters(
            &collected_metadata.references,
            &mut diagnostics,
            &mut reference_statistics,
        )?;

        let mut reference_requests = resolve_metadata_references(
            &mut graph,
            &collected_metadata.references,
            &mut diagnostics,
            metadata_reference_scope,
        )?;

        emit_role_grants(
            &mut graph,
            &collected_metadata.role_rights,
            &mut diagnostics,
            &mut reference_statistics,
        )?;

        add_module_and_xdto_semantics(
            &mut graph,
            &collected_metadata,
            metadata_reference_scope,
            &mut diagnostics,
            &mut reference_statistics,
            &mut reference_requests,
        )?;

        finish_configuration_graph_build(
            graph,
            &collected_metadata.writes_sources,
            &collected_metadata.event_subscriptions,
            diagnostics,
            reference_statistics,
            reference_requests,
        )
    }
}

fn add_module_and_xdto_semantics(
    graph: &mut SemanticGraph,
    collected: &CollectedTopLevelMetadata,
    metadata_reference_scope: query_source_resolution::WorkspaceResolutionScope,
    diagnostics: &mut BTreeSet<SemanticDiagnostic>,
    reference_statistics: &mut SemanticReferenceStatistics,
    reference_requests: &mut SemanticReferenceRequestLedger,
) -> Result<(), EdtGraphError> {
    add_configuration_module_semantics(
        graph,
        &collected.modules,
        metadata_reference_scope,
        diagnostics,
        reference_statistics,
        reference_requests,
    )?;
    xdto_service_emission::resolve_and_emit(
        graph,
        &collected.xdto_service_sources,
        diagnostics,
        reference_requests,
    )
}

fn collect_supported_metadata_directory(
    project_root: &Path,
    directory: &Path,
    kind: MetadataKind,
    configuration_id: &EntityId,
    graph: &mut SemanticGraph,
) -> Result<CollectedTopLevelMetadata, EdtGraphError> {
    if kind == MetadataKind::Subsystem {
        collect_subsystem_hierarchy(project_root, configuration_id, graph)
    } else if kind == MetadataKind::EventSubscription {
        Ok(CollectedTopLevelMetadata {
            event_subscriptions: event_subscription_emission::collect_event_subscription_directory(
                directory,
                configuration_id,
                graph,
            )?,
            ..CollectedTopLevelMetadata::default()
        })
    } else {
        collect_top_level_metadata(project_root, directory, kind, configuration_id, graph)
    }
}

fn add_configuration_module_semantics(
    graph: &mut SemanticGraph,
    modules: &[EdtModuleDescriptor],
    workspace_scope: query_source_resolution::WorkspaceResolutionScope,
    diagnostics: &mut BTreeSet<SemanticDiagnostic>,
    reference_statistics: &mut SemanticReferenceStatistics,
    reference_requests: &mut SemanticReferenceRequestLedger,
) -> Result<(), EdtGraphError> {
    let form_navigation = form_navigation_emission::collect_form_navigation(modules)
        .map_err(EdtGraphError::FormNavigation)?;
    bsl_graph::add_configuration_module_symbols_with_diagnostics_in_scope(
        graph,
        modules,
        workspace_scope,
        diagnostics,
        reference_statistics,
        reference_requests,
    )
    .map_err(EdtGraphError::Bsl)?;
    form_navigation_emission::emit_form_navigation(
        graph,
        &form_navigation,
        workspace_scope,
        diagnostics,
        reference_statistics,
    )
    .map_err(EdtGraphError::FormNavigation)
}

fn insert_configuration_node(
    graph: &mut SemanticGraph,
    project_root: &Path,
    configuration: &Configuration,
    payload: MetadataPayload,
) -> Result<(), EdtGraphError> {
    let configuration_path = project_root.join(CONFIGURATION_RELATIVE_PATH);
    let configuration_source = source_id_from_path_fragment(
        &configuration_path,
        format!(
            "metadata_object={};fact=configuration",
            configuration.id().as_str()
        ),
        EdtGraphError::InvalidIdentifier,
    )?;
    insert_metadata_node(
        graph,
        configuration.id().clone(),
        configuration.name().clone(),
        MetadataKind::Configuration,
        payload,
        parsed_provenance(configuration_source),
    )
}

fn finish_configuration_graph_build(
    mut graph: SemanticGraph,
    writes_sources: &[writes_emission::EdtWritesSource],
    event_subscriptions: &[EdtEventSubscriptionDescriptor],
    mut diagnostics: BTreeSet<SemanticDiagnostic>,
    mut reference_statistics: SemanticReferenceStatistics,
    reference_requests: SemanticReferenceRequestLedger,
) -> Result<EdtSemanticGraphBuildResult, EdtGraphError> {
    event_subscription_emission::emit_resolved_event_subscriptions(
        &mut graph,
        event_subscriptions,
        &mut diagnostics,
        &mut reference_statistics,
    )?;
    writes_emission::emit_resolved_writes(
        &mut graph,
        writes_sources,
        query_source_resolution::WorkspaceResolutionScope::Complete,
        &mut diagnostics,
        &mut reference_statistics,
    )?;
    Ok(EdtSemanticGraphBuildResult::new_with_reference_requests(
        graph,
        diagnostics.into_iter().collect(),
        reference_requests,
        reference_statistics,
    ))
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
        ("EventSubscriptions", MetadataKind::EventSubscription),
    ])
}

#[derive(Debug, Default)]
struct CollectedTopLevelMetadata {
    modules: Vec<EdtModuleDescriptor>,
    writes_sources: Vec<writes_emission::EdtWritesSource>,
    event_subscriptions: Vec<EdtEventSubscriptionDescriptor>,
    references: MetadataReferenceCollection,
    extensions: BTreeSet<PendingMetadataExtension>,
    subsystem_content: BTreeSet<PendingSubsystemContentObservation>,
    subsystem_hierarchy: BTreeSet<PendingSubsystemHierarchyObservation>,
    role_rights: Vec<EdtRoleRightsDescriptor>,
    report_data_composition: Vec<EdtReportDataCompositionDescriptor>,
    xdto_service_sources: Vec<xdto_service_emission::EdtXdtoServiceSource>,
}

impl CollectedTopLevelMetadata {
    fn extend(&mut self, other: Self) -> Result<(), EdtGraphError> {
        self.modules.extend(other.modules);
        self.writes_sources.extend(other.writes_sources);
        self.event_subscriptions.extend(other.event_subscriptions);
        self.references.extend(other.references)?;
        self.extensions.extend(other.extensions);
        self.subsystem_content.extend(other.subsystem_content);
        self.subsystem_hierarchy.extend(other.subsystem_hierarchy);
        self.role_rights.extend(other.role_rights);
        self.report_data_composition
            .extend(other.report_data_composition);
        self.xdto_service_sources.extend(other.xdto_service_sources);
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct MetadataReferenceProjectionEvidence {
    descriptor_path: PathBuf,
    metadata_object_id: EntityId,
    source_id: EntityId,
    role: EdtMetadataReferenceRole,
    target_kind: MetadataKind,
    target_name: EntityName,
    raw_token: Option<String>,
    occurrence_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct RejectedCommandParameterEvidence {
    descriptor_path: PathBuf,
    metadata_object_id: EntityId,
    source_id: EntityId,
    raw_token: Option<String>,
    outcome: EdtCommandParameterTypeOutcomeKind,
    reason: EdtCommandParameterTypeReason,
    occurrence_count: usize,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct MetadataReferenceCollection {
    requests: SemanticReferenceRequestLedger,
    evidence: BTreeMap<SemanticReferenceRequestId, BTreeSet<MetadataReferenceProjectionEvidence>>,
    rejected_command_parameters: BTreeSet<RejectedCommandParameterEvidence>,
}

impl MetadataReferenceCollection {
    fn insert(
        &mut self,
        evidence: MetadataReferenceProjectionEvidence,
    ) -> Result<(), EdtGraphError> {
        let request = SemanticReferenceRequest::collected(
            evidence.source_id.clone(),
            SemanticReferenceCategory::MetadataType,
            SemanticReference::Name(evidence.target_name.clone()),
            [NodeKind::Metadata(evidence.target_kind)],
            [metadata_reference_collection_provenance(
                metadata_reference_collection_source_id(&evidence)?,
            )],
        )
        .map_err(EdtGraphError::ReferenceRequest)?;
        let request_id = request.id().clone();
        self.requests
            .insert(request)
            .map_err(EdtGraphError::ReferenceRequest)?;
        self.evidence
            .entry(request_id)
            .or_default()
            .insert(evidence);
        Ok(())
    }

    fn extend(&mut self, other: Self) -> Result<(), EdtGraphError> {
        for request in other.requests.requests().iter().cloned() {
            self.requests
                .insert(request)
                .map_err(EdtGraphError::ReferenceRequest)?;
        }
        for (request_id, evidence) in other.evidence {
            self.evidence
                .entry(request_id)
                .or_default()
                .extend(evidence);
        }
        self.rejected_command_parameters
            .extend(other.rejected_command_parameters);
        Ok(())
    }

    fn insert_rejected_command_parameter(&mut self, evidence: RejectedCommandParameterEvidence) {
        self.rejected_command_parameters.insert(evidence);
    }

    const fn requests(&self) -> &SemanticReferenceRequestLedger {
        &self.requests
    }

    fn evidence(
        &self,
        request_id: &SemanticReferenceRequestId,
    ) -> Option<&BTreeSet<MetadataReferenceProjectionEvidence>> {
        self.evidence.get(request_id)
    }

    fn rejected_command_parameters(&self) -> &BTreeSet<RejectedCommandParameterEvidence> {
        &self.rejected_command_parameters
    }
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
struct PendingSubsystemHierarchyObservation {
    parent_descriptor_path: PathBuf,
    child_descriptor_path: PathBuf,
    parent_metadata_id: EntityId,
    child_metadata_id: EntityId,
    parent_subsystem_node_id: EntityId,
    child_subsystem_node_id: EntityId,
    raw_child_declaration: String,
    raw_parent_declaration: String,
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

fn collect_subsystem_hierarchy(
    project_root: &Path,
    configuration_id: &EntityId,
    graph: &mut SemanticGraph,
) -> Result<CollectedTopLevelMetadata, EdtGraphError> {
    let hierarchy = FileSystemEdtSubsystemHierarchyReader
        .read(project_root)
        .map_err(EdtGraphError::SubsystemHierarchy)?;
    let descriptors_by_id = hierarchy
        .descriptors()
        .iter()
        .map(|descriptor| (descriptor.metadata().id().clone(), descriptor))
        .collect::<BTreeMap<_, _>>();
    let mut collected = CollectedTopLevelMetadata::default();

    for source in hierarchy.descriptors() {
        let descriptor = source.metadata().clone();
        let object_directory = descriptor
            .descriptor_path()
            .parent()
            .ok_or_else(|| EdtGraphError::InvalidSubsystemDescriptorDirectory {
                path: descriptor.descriptor_path().to_path_buf(),
            })?
            .to_path_buf();
        let object = collect_metadata_descriptor(
            project_root,
            &object_directory,
            descriptor,
            configuration_id,
            graph,
        )?;
        collected.extend(object)?;
    }

    for relation in hierarchy.relations() {
        let parent = descriptors_by_id.get(relation.parent_id()).ok_or_else(|| {
            EdtGraphError::InvalidSubsystemHierarchyRelation {
                parent_id: relation.parent_id().clone(),
                child_id: relation.child_id().clone(),
            }
        })?;
        let child = descriptors_by_id.get(relation.child_id()).ok_or_else(|| {
            EdtGraphError::InvalidSubsystemHierarchyRelation {
                parent_id: relation.parent_id().clone(),
                child_id: relation.child_id().clone(),
            }
        })?;
        collected
            .subsystem_hierarchy
            .insert(PendingSubsystemHierarchyObservation {
                parent_descriptor_path: project_relative_subsystem_descriptor_path(
                    project_root,
                    parent.metadata().descriptor_path(),
                )?,
                child_descriptor_path: project_relative_subsystem_descriptor_path(
                    project_root,
                    child.metadata().descriptor_path(),
                )?,
                parent_metadata_id: relation.parent_id().clone(),
                child_metadata_id: relation.child_id().clone(),
                parent_subsystem_node_id: subsystem_node_id(parent.metadata())?,
                child_subsystem_node_id: subsystem_node_id(child.metadata())?,
                raw_child_declaration: relation.raw_child_declaration().to_owned(),
                raw_parent_declaration: relation.raw_parent_declaration().to_owned(),
            });
    }

    Ok(collected)
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

        collected.extend(object)?;
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
    let descriptor = FileSystemEdtMetadataObjectReader
        .read(object_directory, kind)
        .map_err(EdtGraphError::MetadataObject)?;
    collect_metadata_descriptor(
        project_root,
        object_directory,
        descriptor,
        configuration_id,
        graph,
    )
}

fn insert_metadata_with_xdto_service_semantics(
    graph: &mut SemanticGraph,
    descriptor: &EdtMetadataObjectDescriptor,
    collected: &mut CollectedTopLevelMetadata,
) -> Result<(), EdtGraphError> {
    let descriptor_source = metadata_object_source_id(descriptor)?;
    let xdto_service_source = xdto_service_emission::collect_source(descriptor)?;
    let payload = xdto_service_source.as_ref().map_or_else(
        || metadata_payload(descriptor),
        xdto_service_emission::metadata_payload,
    );
    insert_metadata_node(
        graph,
        descriptor.id().clone(),
        descriptor.name().clone(),
        descriptor.kind(),
        payload,
        declared_provenance(descriptor_source),
    )?;
    if let Some(source) = xdto_service_source {
        xdto_service_emission::emit_declarations(graph, &source)?;
        collected.xdto_service_sources.push(source);
    }
    Ok(())
}

fn collect_metadata_descriptor(
    project_root: &Path,
    object_directory: &Path,
    descriptor: EdtMetadataObjectDescriptor,
    configuration_id: &EntityId,
    graph: &mut SemanticGraph,
) -> Result<CollectedTopLevelMetadata, EdtGraphError> {
    let module_reader = FileSystemEdtModuleReader;
    let structure_reader = FileSystemEdtMetadataStructureReader;
    let mut collected = CollectedTopLevelMetadata::default();
    insert_metadata_with_xdto_service_semantics(graph, &descriptor, &mut collected)?;

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

    collect_report_data_composition(object_directory, &descriptor, &mut collected)?;

    collect_top_level_command_parameter_types(graph, &descriptor, &mut collected.references)?;

    let children = structure_reader
        .read_children(&descriptor)
        .map_err(EdtGraphError::MetadataStructure)?;

    for child in &children {
        collect_metadata_child(graph, &descriptor, child, &mut collected.references)?;
    }

    for child in &children {
        collect_metadata_child_ownership(graph, &descriptor, child)?;
    }

    let form_command_observations = module_reader
        .read_form_command_modules(&descriptor, &children, object_directory)
        .map_err(EdtGraphError::Module)?;
    let form_command_modules =
        emit_form_command_modules(graph, &descriptor, &form_command_observations)?;

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
            parsed_provenance_with_file_location(module_source.clone(), module.path())?,
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

    collected
        .writes_sources
        .push(writes_emission::EdtWritesSource::new(
            descriptor,
            modules.clone(),
        ));
    collected.modules.extend(modules);
    collected.modules.extend(form_command_modules);

    Ok(collected)
}

fn collect_report_data_composition(
    object_directory: &Path,
    descriptor: &EdtMetadataObjectDescriptor,
    collected: &mut CollectedTopLevelMetadata,
) -> Result<(), EdtGraphError> {
    if descriptor.kind() == MetadataKind::Report {
        collected.report_data_composition.push(
            FileSystemEdtReportDataCompositionReader
                .read(object_directory, descriptor)
                .map_err(EdtGraphError::ReportDataComposition)?,
        );
    }
    Ok(())
}

fn emit_form_command_modules(
    graph: &mut SemanticGraph,
    descriptor: &EdtMetadataObjectDescriptor,
    observations: &[EdtModuleLayoutObservation],
) -> Result<Vec<EdtModuleDescriptor>, EdtGraphError> {
    let mut modules = Vec::new();

    for observation in observations {
        if observation.outcome() != EdtModuleLayoutOutcomeKind::Accepted {
            continue;
        }
        let owner_id = observation
            .owner_id()
            .expect("accepted module observation must have an owner");
        let owner_kind = observation
            .owner_kind()
            .expect("accepted module observation must have an owner kind");
        let module = observation
            .module()
            .expect("accepted module observation must have a descriptor");
        let actual_owner_kind = graph.node(owner_id).map(oneagent_graph::GraphNode::kind);
        let compatible = matches!(
            (owner_kind, actual_owner_kind),
            (EdtModuleOwnerKind::Form, Some(NodeKind::Form))
                | (EdtModuleOwnerKind::Command, Some(NodeKind::Command))
                | (
                    EdtModuleOwnerKind::CommonCommand,
                    Some(NodeKind::Metadata(MetadataKind::Command))
                )
        );
        if !compatible {
            return Err(EdtGraphError::InvalidFormCommandModuleOwner {
                owner_id: owner_id.clone(),
                expected_kind: owner_kind,
                actual_kind: actual_owner_kind,
            });
        }

        let module_source = form_command_module_source_id(descriptor, owner_id, module)?;
        insert_node(
            graph,
            module.id().clone(),
            module.name().clone(),
            NodeKind::Module,
            parsed_provenance_with_file_location(module_source, module.path())?,
        );
        insert_edge(
            graph,
            owner_id.clone(),
            module.id().clone(),
            EdgeKind::Contains,
            parsed_provenance(contains_edge_source_id(
                module.path(),
                descriptor.id(),
                owner_id,
                module.id(),
            )?),
        )?;
        modules.push(module.clone());
    }

    modules.sort_by(|left, right| left.id().cmp(right.id()));
    modules.dedup_by(|left, right| left.id() == right.id());
    Ok(modules)
}

fn metadata_payload(descriptor: &EdtMetadataObjectDescriptor) -> MetadataPayload {
    let common = CommonMetadataPayload::new(descriptor.synonym().map(str::to_owned));
    let specific = (descriptor.kind() == MetadataKind::Document).then(|| {
        let register_records = descriptor
            .document_register_declarations()
            .iter()
            .filter_map(|outcome| {
                let declaration = match outcome {
                    metadata_object::EdtDocumentRegisterDeclarationOutcome::Supported(
                        declaration,
                    )
                    | metadata_object::EdtDocumentRegisterDeclarationOutcome::UnsupportedKind(
                        declaration,
                    ) => declaration,
                    metadata_object::EdtDocumentRegisterDeclarationOutcome::UnsupportedNamespace(
                        _,
                    )
                    | metadata_object::EdtDocumentRegisterDeclarationOutcome::Malformed(_)
                    | metadata_object::EdtDocumentRegisterDeclarationOutcome::Ambiguous(_) => {
                        return None;
                    }
                };
                let target_kind = declaration.kind?;
                let target_name = EntityName::new(declaration.local_name.clone()).ok()?;

                Some(MetadataRegisterRecord::new(target_kind, target_name))
            });

        MetadataSpecificPayload::Document(DocumentMetadataPayload::new(register_records))
    });

    MetadataPayload::new(common, specific)
}

fn collect_subsystem_content(
    project_root: &Path,
    descriptor: &EdtMetadataObjectDescriptor,
    observations: &mut BTreeSet<PendingSubsystemContentObservation>,
) -> Result<(), EdtGraphError> {
    let content = FileSystemEdtSubsystemContentReader
        .read(descriptor)
        .map_err(EdtGraphError::SubsystemContent)?;
    let descriptor_path =
        project_relative_subsystem_descriptor_path(project_root, content.descriptor_path())?;
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

fn project_relative_subsystem_descriptor_path(
    project_root: &Path,
    descriptor_path: &Path,
) -> Result<PathBuf, EdtGraphError> {
    if let Ok(relative) = descriptor_path.strip_prefix(project_root) {
        return Ok(relative.to_path_buf());
    }
    if let Ok(canonical_project_root) = fs::canonicalize(project_root)
        && let Ok(relative) = descriptor_path.strip_prefix(canonical_project_root)
    {
        return Ok(relative.to_path_buf());
    }
    Err(EdtGraphError::SubsystemDescriptorOutsideProject {
        project_root: project_root.to_path_buf(),
        path: descriptor_path.to_path_buf(),
    })
}

fn normalize_subsystem_content_target(raw_token: &str) -> SubsystemContentTarget {
    let mut components = raw_token.split('.');
    let prefix = components.next().unwrap_or_default();
    let Some(local_name) = components.next() else {
        return SubsystemContentTarget::Malformed;
    };

    if prefix.is_empty() || local_name.is_empty() {
        return SubsystemContentTarget::Malformed;
    }

    if prefix == "Subsystem" {
        if components.any(str::is_empty) {
            return SubsystemContentTarget::Malformed;
        }
        return SubsystemContentTarget::Unsupported {
            prefix: prefix.to_owned(),
            deferred: true,
        };
    }

    if components.next().is_some() {
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
    references: &mut MetadataReferenceCollection,
) -> Result<(), EdtGraphError> {
    let child_source = metadata_child_source_id(descriptor, child)?;

    let child_node_kind = semantic_child_node_kind(descriptor.kind(), child.kind());

    let provenance = declared_provenance(child_source);
    if matches!(
        child_node_kind,
        NodeKind::Attribute | NodeKind::TabularSection
    ) {
        insert_metadata_member_node(
            graph,
            child.id().clone(),
            child.name().clone(),
            child_node_kind,
            child.member_payload().clone(),
            provenance,
        )?;
    } else {
        insert_node(
            graph,
            child.id().clone(),
            child.name().clone(),
            child_node_kind,
            provenance,
        );
    }

    if is_depends_on_metadata_member_source(child_node_kind) {
        for reference in child.references() {
            references.insert(MetadataReferenceProjectionEvidence {
                descriptor_path: descriptor.descriptor_path().to_path_buf(),
                metadata_object_id: descriptor.id().clone(),
                source_id: child.id().clone(),
                role: reference.role(),
                target_kind: reference.target_kind(),
                target_name: reference.target_name().clone(),
                raw_token: None,
                occurrence_count: 1,
            })?;
        }
    }

    collect_command_parameter_types(
        graph,
        descriptor,
        child.command_parameter_types(),
        references,
    )?;

    Ok(())
}

fn collect_command_parameter_types(
    graph: &SemanticGraph,
    descriptor: &EdtMetadataObjectDescriptor,
    observations: &[EdtCommandParameterTypeObservation],
    references: &mut MetadataReferenceCollection,
) -> Result<(), EdtGraphError> {
    for observation in observations {
        let actual_kind = graph
            .node(observation.source_id())
            .map(oneagent_graph::GraphNode::kind);
        let valid_source = matches!(
            (observation.source_kind(), actual_kind),
            (
                EdtCommandParameterSourceKind::CommonCommand,
                Some(NodeKind::Metadata(MetadataKind::Command))
            ) | (
                EdtCommandParameterSourceKind::SubordinateCommand,
                Some(NodeKind::Command)
            )
        );
        if !valid_source {
            return Err(EdtGraphError::InvalidCommandParameterSource {
                source_id: observation.source_id().clone(),
                source_kind: observation.source_kind(),
                actual_kind,
            });
        }

        match observation.outcome() {
            EdtCommandParameterTypeOutcomeKind::Accepted => {
                references.insert(MetadataReferenceProjectionEvidence {
                    descriptor_path: observation.descriptor_path().to_path_buf(),
                    metadata_object_id: descriptor.id().clone(),
                    source_id: observation.source_id().clone(),
                    role: observation.role(),
                    target_kind: observation
                        .target_kind()
                        .expect("accepted Command parameter must have a target kind"),
                    target_name: observation
                        .target_name()
                        .expect("accepted Command parameter must have a target name")
                        .clone(),
                    raw_token: observation.raw_token().map(str::to_owned),
                    occurrence_count: observation.occurrence_count(),
                })?;
            }
            EdtCommandParameterTypeOutcomeKind::Unsupported
            | EdtCommandParameterTypeOutcomeKind::Malformed => {
                references.insert_rejected_command_parameter(RejectedCommandParameterEvidence {
                    descriptor_path: observation.descriptor_path().to_path_buf(),
                    metadata_object_id: descriptor.id().clone(),
                    source_id: observation.source_id().clone(),
                    raw_token: observation.raw_token().map(str::to_owned),
                    outcome: observation.outcome(),
                    reason: observation
                        .reason()
                        .expect("rejected Command parameter must have a typed reason"),
                    occurrence_count: observation.occurrence_count(),
                });
            }
            EdtCommandParameterTypeOutcomeKind::Ignored
            | EdtCommandParameterTypeOutcomeKind::Missing => {}
        }
    }

    Ok(())
}

fn collect_top_level_command_parameter_types(
    graph: &SemanticGraph,
    descriptor: &EdtMetadataObjectDescriptor,
    references: &mut MetadataReferenceCollection,
) -> Result<(), EdtGraphError> {
    collect_command_parameter_types(
        graph,
        descriptor,
        descriptor.command_parameter_types(),
        references,
    )
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

fn emit_rejected_command_parameters(
    references: &MetadataReferenceCollection,
    diagnostics: &mut BTreeSet<SemanticDiagnostic>,
    reference_statistics: &mut SemanticReferenceStatistics,
) -> Result<(), EdtGraphError> {
    for evidence in references.rejected_command_parameters() {
        let (reference_outcome, code, kind, message) = match evidence.outcome {
            EdtCommandParameterTypeOutcomeKind::Malformed => (
                SemanticReferenceOutcome::MalformedFormat,
                SemanticDiagnosticCode::ReferenceMalformedFormat,
                SemanticDiagnosticKind::MalformedReferenceFormat,
                "EDT Command parameter type is malformed",
            ),
            EdtCommandParameterTypeOutcomeKind::Unsupported => (
                SemanticReferenceOutcome::UnsupportedPrefix,
                SemanticDiagnosticCode::ReferenceUnsupportedPrefix,
                SemanticDiagnosticKind::UnsupportedReferencePrefix,
                "EDT Command parameter type is unsupported",
            ),
            EdtCommandParameterTypeOutcomeKind::Accepted
            | EdtCommandParameterTypeOutcomeKind::Ignored
            | EdtCommandParameterTypeOutcomeKind::Missing => {
                unreachable!("only rejected Command parameters enter rejection projection")
            }
        };
        reference_statistics.record(reference_outcome, true);
        let raw_reference = evidence.raw_token.clone().unwrap_or_else(|| {
            format!(
                "commandParameterType:{}",
                command_parameter_reason_str(evidence.reason)
            )
        });
        diagnostics.insert(
            SemanticDiagnostic::new(
                code,
                SemanticDiagnosticSeverity::Error,
                kind,
                format!(
                    "{message}: {}",
                    command_parameter_reason_str(evidence.reason)
                ),
                SemanticReference::Raw(raw_reference),
            )
            .with_source_node(evidence.source_id.clone())
            .with_provenance(vec![command_parameter_rejection_provenance(
                rejected_command_parameter_source_id(evidence)?,
            )]),
        );
    }

    Ok(())
}

fn resolve_metadata_references(
    graph: &mut SemanticGraph,
    references: &MetadataReferenceCollection,
    diagnostics: &mut BTreeSet<SemanticDiagnostic>,
    workspace_scope: query_source_resolution::WorkspaceResolutionScope,
) -> Result<SemanticReferenceRequestLedger, EdtGraphError> {
    let resolved_references = {
        let index = graph.resolution_index();
        let mut resolved_references = Vec::new();
        let mut terminal_requests = SemanticReferenceRequestLedger::new();

        for request in references.requests().requests() {
            let evidence = references.evidence(request.id()).ok_or_else(|| {
                EdtGraphError::InvalidMetadataReferenceRequest {
                    request_id: request.id().clone(),
                }
            })?;
            let SemanticReference::Name(target_name) = request.reference() else {
                return Err(EdtGraphError::InvalidMetadataReferenceRequest {
                    request_id: request.id().clone(),
                });
            };
            let expected_kind = request.expected_kinds()[0];
            match index.resolve_name_of_kind(target_name, expected_kind) {
                Ok(target) => {
                    let resolver_provenance = metadata_reference_resolver_provenance(
                        evidence,
                        ResolutionState::Resolved,
                        Some(target.id()),
                    )?;
                    let terminal = request
                        .clone()
                        .into_resolved(target.id().clone(), target.kind(), resolver_provenance)
                        .map_err(EdtGraphError::ReferenceRequest)?;
                    resolved_references.push((terminal.clone(), evidence, target.id().clone()));
                    terminal_requests
                        .insert(terminal)
                        .map_err(EdtGraphError::ReferenceRequest)?;
                }
                Err(error) => {
                    let terminal = terminal_metadata_reference_request(
                        request,
                        evidence,
                        &error,
                        workspace_scope,
                    )?;
                    if terminal.outcome() != SemanticReferenceRequestOutcome::PartialWorkspace {
                        diagnostics.insert(metadata_reference_diagnostic(&terminal, error));
                    }
                    terminal_requests
                        .insert(terminal)
                        .map_err(EdtGraphError::ReferenceRequest)?;
                }
            }
        }

        (terminal_requests, resolved_references)
    };

    for (request, evidence, target_id) in resolved_references.1 {
        let reference_provenance = metadata_reference_resolver_provenance(
            evidence,
            ResolutionState::Resolved,
            Some(&target_id),
        )?;
        graph
            .insert_edge(GraphEdge::new_with_provenance(
                request.source_node().clone(),
                target_id.clone(),
                EdgeKind::References,
                reference_provenance,
            ))
            .map_err(EdtGraphError::Graph)?;
        let dependency_provenance = evidence
            .iter()
            .map(|evidence| {
                metadata_dependency_source_id(evidence, &target_id).map(derived_provenance)
            })
            .collect::<Result<Vec<_>, _>>()?;
        graph
            .insert_edge(GraphEdge::new_with_provenance(
                request.source_node().clone(),
                target_id,
                EdgeKind::DependsOn,
                dependency_provenance,
            ))
            .map_err(EdtGraphError::Graph)?;
    }

    Ok(resolved_references.0)
}

fn terminal_metadata_reference_request(
    request: &SemanticReferenceRequest,
    evidence: &BTreeSet<MetadataReferenceProjectionEvidence>,
    error: &ResolutionError,
    workspace_scope: query_source_resolution::WorkspaceResolutionScope,
) -> Result<SemanticReferenceRequest, EdtGraphError> {
    let (state, candidates) = match error {
        ResolutionError::MissingTarget { .. }
            if workspace_scope == query_source_resolution::WorkspaceResolutionScope::Partial =>
        {
            (ResolutionState::Partial, Vec::new())
        }
        ResolutionError::MissingTarget { .. } => (ResolutionState::Unresolved, Vec::new()),
        ResolutionError::AmbiguousTarget { candidates, .. } => {
            (ResolutionState::Ambiguous, candidates.clone())
        }
        ResolutionError::IncompatibleNodeKind { id, .. } => {
            (ResolutionState::Unresolved, vec![id.clone()])
        }
        ResolutionError::InvalidOwnerReference { owner, child } => (
            ResolutionState::Unresolved,
            vec![owner.clone(), child.clone()],
        ),
    };
    let provenance = metadata_reference_resolver_provenance(evidence, state, None)?;

    let terminal = match error {
        ResolutionError::MissingTarget { .. }
            if workspace_scope == query_source_resolution::WorkspaceResolutionScope::Partial =>
        {
            request
                .clone()
                .into_partial_workspace(candidates, provenance)
        }
        ResolutionError::MissingTarget { .. } => request.clone().into_missing_target(provenance),
        ResolutionError::AmbiguousTarget { .. } => request
            .clone()
            .into_ambiguous_target(candidates, provenance),
        ResolutionError::IncompatibleNodeKind { .. } => request
            .clone()
            .into_incompatible_target_kind(candidates, provenance),
        ResolutionError::InvalidOwnerReference { .. } => request
            .clone()
            .into_invalid_owner_reference(candidates, provenance),
    };
    terminal.map_err(EdtGraphError::ReferenceRequest)
}

fn metadata_reference_resolver_provenance(
    evidence: &BTreeSet<MetadataReferenceProjectionEvidence>,
    resolution: ResolutionState,
    resolved_target: Option<&EntityId>,
) -> Result<Vec<Provenance>, EdtGraphError> {
    evidence
        .iter()
        .map(|evidence| {
            metadata_reference_source_id(evidence, resolved_target).map(|source| {
                graph_provenance(source, FactOrigin::Resolved, Confidence::High, resolution)
            })
        })
        .collect()
}

fn emit_subsystem_composition(
    graph: &mut SemanticGraph,
    hierarchy: &BTreeSet<PendingSubsystemHierarchyObservation>,
    content: &BTreeSet<PendingSubsystemContentObservation>,
    diagnostics: &mut BTreeSet<SemanticDiagnostic>,
    statistics: &mut SemanticReferenceStatistics,
) -> Result<(), EdtGraphError> {
    emit_subsystem_hierarchy(graph, hierarchy)?;
    emit_subsystem_includes(graph, content, diagnostics, statistics)
}

fn emit_subsystem_hierarchy(
    graph: &mut SemanticGraph,
    observations: &BTreeSet<PendingSubsystemHierarchyObservation>,
) -> Result<(), EdtGraphError> {
    for observation in observations {
        for node_id in [
            &observation.parent_subsystem_node_id,
            &observation.child_subsystem_node_id,
        ] {
            let actual_kind = graph.node(node_id).map(GraphNode::kind);
            if actual_kind != Some(NodeKind::Subsystem) {
                return Err(EdtGraphError::InvalidSubsystemHierarchyEndpoint {
                    node_id: node_id.clone(),
                    actual_kind,
                });
            }
        }

        insert_edge(
            graph,
            observation.parent_subsystem_node_id.clone(),
            observation.child_subsystem_node_id.clone(),
            EdgeKind::Includes,
            subsystem_hierarchy_provenance(subsystem_hierarchy_source_id(observation)?),
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
    row_restriction: Option<AccessRightRowRestriction>,
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
                let row_restriction = right
                    .row_restriction()
                    .map(|restriction| AccessRightRowRestriction::new(restriction.condition()))
                    .transpose()
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
                            row_restriction,
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
    let mut access_right_sources = BTreeMap::<
        (EntityId, EntityId, Option<AccessRightRowRestriction>),
        BTreeSet<EntityId>,
    >::new();
    let mut reference_sources = BTreeMap::<
        (EntityId, EntityId, Option<AccessRightRowRestriction>),
        BTreeSet<EntityId>,
    >::new();
    let mut grant_sources = BTreeMap::<
        (
            EntityId,
            EntityId,
            EntityId,
            Option<AccessRightRowRestriction>,
        ),
        BTreeSet<EntityId>,
    >::new();

    for observation in observations {
        let access_key = (
            observation.resource_id.clone(),
            observation.right_id.clone(),
            observation.row_restriction.clone(),
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
            .entry((
                observation.role_node_id.clone(),
                access_key.0,
                access_key.1,
                access_key.2,
            ))
            .or_default()
            .insert(role_grant_source_id(observation, "edge=grants")?);
    }

    let mut access_right_ids =
        BTreeMap::<(EntityId, EntityId, Option<AccessRightRowRestriction>), EntityId>::new();

    for ((resource_id, right_id, row_restriction), sources) in access_right_sources {
        let provenance = sources.into_iter().map(resolved_provenance).collect();
        let access_right = AccessRight::new_with_row_restriction(
            resource_id.clone(),
            right_id.clone(),
            row_restriction.clone(),
            provenance,
        )
        .map_err(|_| EdtGraphError::InvalidIdentifier)?;
        let access_right_id = access_right.id().clone();
        graph.insert_access_right(&access_right);
        access_right_ids.insert((resource_id, right_id, row_restriction), access_right_id);
    }

    for ((resource_id, right_id, row_restriction), sources) in reference_sources {
        let access_right_id = access_right_ids
            .get(&(resource_id.clone(), right_id, row_restriction))
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

    for ((role_node_id, resource_id, right_id, row_restriction), sources) in grant_sources {
        let access_right_id = access_right_ids
            .get(&(resource_id, right_id, row_restriction))
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
            row_restriction: None,
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
    let row_restriction = observation.row_restriction.as_ref().map_or_else(
        || "row_restriction=absent".to_owned(),
        |restriction| {
            format!(
                "row_restriction#{}:{}",
                restriction.condition().len(),
                restriction.condition()
            )
        },
    );
    source_id_from_path_fragment(
        &observation.rights_path,
        format!(
            "role_metadata={};role={};protected_resource={};resolved_resource={};right={};value=true;accepted_explicit_allow=true;{};{}",
            observation.role_id.as_str(),
            observation.role_node_id.as_str(),
            observation.declared_resource_name.as_str(),
            observation.resource_id.as_str(),
            observation.right_id.as_str(),
            row_restriction,
            fact_kind,
        ),
        EdtGraphError::InvalidIdentifier,
    )
}

fn metadata_reference_diagnostic(
    request: &SemanticReferenceRequest,
    error: ResolutionError,
) -> SemanticDiagnostic {
    let resolver_provenance = request
        .provenance()
        .iter()
        .filter(|provenance| provenance.origin() == FactOrigin::Resolved)
        .cloned()
        .collect();
    SemanticDiagnostic::from_resolution_error_with_reference(
        error,
        Some(request.reference().clone()),
    )
    .with_source_node(request.source_node().clone())
    .with_expected_kinds(request.expected_kinds().to_vec())
    .with_candidates(request.candidates().to_vec())
    .with_provenance(resolver_provenance)
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

fn insert_metadata_node(
    graph: &mut SemanticGraph,
    id: EntityId,
    name: EntityName,
    kind: MetadataKind,
    payload: MetadataPayload,
    provenance: Provenance,
) -> Result<(), EdtGraphError> {
    let node = GraphNode::new_with_payload_and_provenance(
        id,
        name,
        NodeKind::Metadata(kind),
        GraphNodePayload::Metadata(payload),
        vec![provenance],
    )
    .map_err(EdtGraphError::NodePayload)?;
    graph.insert_node(node);
    Ok(())
}

fn insert_metadata_member_node(
    graph: &mut SemanticGraph,
    id: EntityId,
    name: EntityName,
    kind: NodeKind,
    payload: MetadataMemberPayload,
    provenance: Provenance,
) -> Result<(), EdtGraphError> {
    let node = GraphNode::new_with_payload_and_provenance(
        id,
        name,
        kind,
        GraphNodePayload::MetadataMember(payload),
        vec![provenance],
    )
    .map_err(EdtGraphError::NodePayload)?;
    graph.insert_node(node);
    Ok(())
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

fn parsed_provenance_with_file_location(
    source: EntityId,
    path: &Path,
) -> Result<Provenance, EdtGraphError> {
    let path = path
        .to_str()
        .ok_or(EdtGraphError::InvalidSourceLocation)
        .and_then(|path| SourcePath::new(path).map_err(|_| EdtGraphError::InvalidSourceLocation))?;
    Ok(parsed_provenance(source).with_location(SourceLocation::new(path, None)))
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

fn subsystem_hierarchy_provenance(source: EntityId) -> Provenance {
    Provenance::new(
        Some(source),
        ProducerId::new(EDT_SUBSYSTEM_HIERARCHY_RESOLUTION_PRODUCER),
        FactOrigin::Resolved,
        Confidence::Exact,
        ResolutionState::Resolved,
    )
}

fn subsystem_hierarchy_source_id(
    observation: &PendingSubsystemHierarchyObservation,
) -> Result<EntityId, EdtGraphError> {
    let child_descriptor = observation
        .child_descriptor_path
        .to_string_lossy()
        .replace('\\', "/");
    let fields = [
        encoded_source_field("stage", "subsystem_hierarchy_resolution"),
        encoded_source_field(
            "parent_metadata_uuid",
            observation.parent_metadata_id.as_str(),
        ),
        encoded_source_field(
            "child_metadata_uuid",
            observation.child_metadata_id.as_str(),
        ),
        encoded_source_field("child_descriptor", &child_descriptor),
        encoded_source_field("parent_field", "mdclass:Subsystem/subsystems"),
        encoded_source_field("raw_child", &observation.raw_child_declaration),
        encoded_source_field("child_field", "mdclass:Subsystem/parentSubsystem"),
        encoded_source_field("raw_parent", &observation.raw_parent_declaration),
        encoded_source_field(
            "resolved_parent",
            observation.parent_subsystem_node_id.as_str(),
        ),
        encoded_source_field(
            "resolved_child",
            observation.child_subsystem_node_id.as_str(),
        ),
        encoded_source_field("outcome", "resolved"),
    ];
    source_id_from_path_fragment(
        &observation.parent_descriptor_path,
        fields.join(";"),
        EdtGraphError::InvalidIdentifier,
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

fn form_command_module_source_id(
    descriptor: &EdtMetadataObjectDescriptor,
    owner_id: &EntityId,
    module: &EdtModuleDescriptor,
) -> Result<EntityId, EdtGraphError> {
    source_id_from_path_fragment(
        module.path(),
        format!(
            "metadata_object={};owner={};module={}",
            descriptor.id().as_str(),
            owner_id.as_str(),
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
    reference: &MetadataReferenceProjectionEvidence,
    resolved_target: Option<&EntityId>,
) -> Result<EntityId, EdtGraphError> {
    let fragment = match reference.role {
        EdtMetadataReferenceRole::Type => format!(
            "metadata_object={};edge=references;source={};role={};target_kind={};target_name={}",
            reference.metadata_object_id.as_str(),
            reference.source_id.as_str(),
            reference.role.as_str(),
            reference.target_kind.as_str(),
            reference.target_name.as_str()
        ),
        EdtMetadataReferenceRole::CommandParameterType => format!(
            "metadata_object={};edge=references;source={};role={};raw_token={};occurrences={};target_kind={};target_name={};target={}",
            reference.metadata_object_id.as_str(),
            reference.source_id.as_str(),
            reference.role.as_str(),
            reference.raw_token.as_deref().unwrap_or_default(),
            reference.occurrence_count,
            reference.target_kind.as_str(),
            reference.target_name.as_str(),
            resolved_target.map(EntityId::as_str).unwrap_or_default()
        ),
    };
    source_id_from_path_fragment(
        &reference.descriptor_path,
        fragment,
        EdtGraphError::InvalidIdentifier,
    )
}

fn metadata_dependency_source_id(
    reference: &MetadataReferenceProjectionEvidence,
    target_id: &EntityId,
) -> Result<EntityId, EdtGraphError> {
    let origin = match reference.role {
        EdtMetadataReferenceRole::Type => "metadata_member_type_reference",
        EdtMetadataReferenceRole::CommandParameterType => "command_parameter_type_reference",
    };
    let command_parameter = match reference.role {
        EdtMetadataReferenceRole::Type => String::new(),
        EdtMetadataReferenceRole::CommandParameterType => format!(
            ";raw_token={};occurrences={}",
            reference.raw_token.as_deref().unwrap_or_default(),
            reference.occurrence_count
        ),
    };
    source_id_from_path_fragment(
        &reference.descriptor_path,
        format!(
            "metadata_object={};edge=depends_on;origin={origin};source={};role={}{command_parameter};target_kind={};target_name={};target={}",
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

fn metadata_reference_collection_source_id(
    reference: &MetadataReferenceProjectionEvidence,
) -> Result<EntityId, EdtGraphError> {
    let command_parameter = match reference.role {
        EdtMetadataReferenceRole::Type => String::new(),
        EdtMetadataReferenceRole::CommandParameterType => format!(
            ";raw_token={};occurrences={}",
            reference.raw_token.as_deref().unwrap_or_default(),
            reference.occurrence_count
        ),
    };
    source_id_from_path_fragment(
        &reference.descriptor_path,
        format!(
            "metadata_object={};fact=reference_request_collection;source={};role={}{command_parameter};target_kind={};target_name={}",
            reference.metadata_object_id.as_str(),
            reference.source_id.as_str(),
            reference.role.as_str(),
            reference.target_kind.as_str(),
            reference.target_name.as_str()
        ),
        EdtGraphError::InvalidIdentifier,
    )
}

fn metadata_reference_collection_provenance(source: EntityId) -> Provenance {
    Provenance::new(
        Some(source),
        ProducerId::new(EDT_METADATA_REFERENCE_COLLECTION_PRODUCER),
        FactOrigin::Declared,
        Confidence::Exact,
        ResolutionState::Unresolved,
    )
}

fn rejected_command_parameter_source_id(
    evidence: &RejectedCommandParameterEvidence,
) -> Result<EntityId, EdtGraphError> {
    source_id_from_path_fragment(
        &evidence.descriptor_path,
        format!(
            "metadata_object={};fact=command_parameter_rejection;source={};outcome={};reason={};raw_token={};occurrences={}",
            evidence.metadata_object_id.as_str(),
            evidence.source_id.as_str(),
            command_parameter_outcome_str(evidence.outcome),
            command_parameter_reason_str(evidence.reason),
            evidence.raw_token.as_deref().unwrap_or_default(),
            evidence.occurrence_count
        ),
        EdtGraphError::InvalidIdentifier,
    )
}

fn command_parameter_rejection_provenance(source: EntityId) -> Provenance {
    Provenance::new(
        Some(source),
        ProducerId::new(EDT_COMMAND_PARAMETER_REJECTION_PRODUCER),
        FactOrigin::Parsed,
        Confidence::Exact,
        ResolutionState::Unresolved,
    )
}

const fn command_parameter_outcome_str(
    outcome: EdtCommandParameterTypeOutcomeKind,
) -> &'static str {
    match outcome {
        EdtCommandParameterTypeOutcomeKind::Accepted => "accepted",
        EdtCommandParameterTypeOutcomeKind::Ignored => "ignored",
        EdtCommandParameterTypeOutcomeKind::Unsupported => "unsupported",
        EdtCommandParameterTypeOutcomeKind::Malformed => "malformed",
        EdtCommandParameterTypeOutcomeKind::Missing => "missing",
    }
}

const fn command_parameter_reason_str(reason: EdtCommandParameterTypeReason) -> &'static str {
    match reason {
        EdtCommandParameterTypeReason::MissingContainer => "missing_container",
        EdtCommandParameterTypeReason::EmptyContainer => "empty_container",
        EdtCommandParameterTypeReason::DuplicateContainer => "duplicate_container",
        EdtCommandParameterTypeReason::PrimitiveType => "primitive_type",
        EdtCommandParameterTypeReason::DeferredDefinedType => "deferred_defined_type",
        EdtCommandParameterTypeReason::UnsupportedPlatformType => "unsupported_platform_type",
        EdtCommandParameterTypeReason::UnsupportedPrefix => "unsupported_prefix",
        EdtCommandParameterTypeReason::EmptyValue => "empty_value",
        EdtCommandParameterTypeReason::MissingComponent => "missing_component",
        EdtCommandParameterTypeReason::AdditionalComponents => "additional_components",
        EdtCommandParameterTypeReason::EmptyComponent => "empty_component",
        EdtCommandParameterTypeReason::InvalidTargetName => "invalid_target_name",
    }
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
    /// A module path could not become typed source evidence.
    InvalidSourceLocation,
    /// Semantic graph validation failed.
    /// A top-level metadata descriptor could not be read.
    MetadataObject(EdtMetadataObjectError),

    /// Report Data Composition declarations or artifacts could not be read.
    ReportDataComposition(EdtReportDataCompositionError),

    /// An Event Subscription descriptor could not be read.
    EventSubscription(EdtEventSubscriptionError),

    /// An XDTO Package descriptor/artifact join could not be read.
    XdtoPackage(EdtXdtoPackageError),

    /// An HTTP or Web Service descriptor could not be read.
    ServiceDescriptor(EdtServiceDescriptorError),

    /// An internal XDTO/service request shape is incompatible with its projection phase.
    InvalidXdtoServiceRequest,

    /// The internal structure of a metadata object could not be read.
    MetadataStructure(EdtMetadataStructureError),

    /// A metadata object module could not be read.
    Module(EdtModuleError),

    /// Direct Subsystem content declarations could not be read.
    SubsystemContent(EdtSubsystemContentError),

    /// Recursive Subsystem hierarchy source discovery failed.
    SubsystemHierarchy(EdtSubsystemHierarchyError),

    /// A hierarchy descriptor path has no containing object directory.
    InvalidSubsystemDescriptorDirectory {
        /// Discovered descriptor path.
        path: PathBuf,
    },

    /// A hierarchy relation refers to a descriptor absent from its source model.
    InvalidSubsystemHierarchyRelation {
        /// Parent metadata UUID.
        parent_id: EntityId,
        /// Child metadata UUID.
        child_id: EntityId,
    },

    /// A hierarchy relation has a missing or incompatible flat Subsystem endpoint.
    InvalidSubsystemHierarchyEndpoint {
        /// Expected flat Subsystem node identifier.
        node_id: EntityId,
        /// Actual node kind, or `None` when the node is missing.
        actual_kind: Option<NodeKind>,
    },

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

    /// An accepted Form or Command module observation has no compatible graph owner.
    InvalidFormCommandModuleOwner {
        /// Canonical owner identifier from the parser observation.
        owner_id: EntityId,
        /// Accepted parser owner family.
        expected_kind: EdtModuleOwnerKind,
        /// Actual graph node kind, or `None` when the owner is missing.
        actual_kind: Option<NodeKind>,
    },

    /// A Command parameter observation has no compatible canonical graph source.
    InvalidCommandParameterSource {
        /// Canonical Command identifier from the parser observation.
        source_id: EntityId,
        /// Accepted parser source family.
        source_kind: EdtCommandParameterSourceKind,
        /// Actual graph node kind, or `None` when the source is missing.
        actual_kind: Option<NodeKind>,
    },

    /// A role-right artifact could not be read.
    RoleRights(EdtRoleRightsError),

    /// Semantic graph validation failed.
    Graph(oneagent_graph::GraphError),

    /// A public semantic reference request invariant failed.
    ReferenceRequest(SemanticReferenceRequestError),

    /// Adapter projection evidence is missing or incompatible with a request.
    InvalidMetadataReferenceRequest {
        /// Stable request identity.
        request_id: SemanticReferenceRequestId,
    },

    /// Typed metadata payload conflicts with its graph node kind.
    NodePayload(GraphNodePayloadError),

    /// A parsed Data Set payload violates its source cardinality contract.
    DataSetPayload(DataSetPayloadError),

    /// A Report Data Composition source identity collides with an existing node.
    DuplicateDataCompositionNode(EntityId),

    /// The accepted Report owner is missing or has an incompatible node kind.
    InvalidReportDataCompositionOwner {
        /// Report identity.
        report_id: EntityId,
        /// Actual graph kind, or `None` when missing.
        actual_kind: Option<NodeKind>,
    },

    /// A parsed DCS artifact path is outside the current project root.
    ReportDataCompositionArtifactOutsideProject {
        /// Current project root.
        project_root: PathBuf,
        /// Artifact path.
        path: PathBuf,
    },

    /// BSL symbols could not be added to the graph.
    Bsl(EdtBslGraphError),

    /// Static Form navigation could not be collected or emitted.
    FormNavigation(form_navigation_emission::EdtFormNavigationEmissionError),
}

impl Display for EdtGraphError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Load(error) => write!(formatter, "failed to load EDT configuration: {error}"),
            Self::ReadDirectory { .. }
            | Self::ReadDirectoryEntry { .. }
            | Self::ReadFileType { .. } => format_file_system_graph_error(self, formatter),
            Self::MetadataObject(error) => {
                write!(formatter, "failed to read EDT metadata object: {error}")
            }
            Self::ReportDataComposition(_)
            | Self::DataSetPayload(_)
            | Self::DuplicateDataCompositionNode(_)
            | Self::InvalidReportDataCompositionOwner { .. }
            | Self::ReportDataCompositionArtifactOutsideProject { .. } => {
                format_report_data_composition_graph_error(self, formatter)
            }
            Self::EventSubscription(error) => {
                write!(formatter, "failed to read EDT Event Subscription: {error}")
            }
            Self::XdtoPackage(error) => {
                write!(formatter, "failed to read EDT XDTO Package: {error}")
            }
            Self::ServiceDescriptor(error) => {
                write!(formatter, "failed to read EDT service descriptor: {error}")
            }
            Self::InvalidXdtoServiceRequest => {
                formatter.write_str("invalid internal EDT XDTO/service reference request")
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
            Self::SubsystemHierarchy(_)
            | Self::InvalidSubsystemDescriptorDirectory { .. }
            | Self::InvalidSubsystemHierarchyRelation { .. }
            | Self::InvalidSubsystemHierarchyEndpoint { .. } => {
                format_subsystem_hierarchy_graph_error(self, formatter)
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
            Self::InvalidFormCommandModuleOwner {
                owner_id,
                expected_kind,
                actual_kind,
            } => write!(
                formatter,
                "Form or Command module owner `{owner_id}` is incompatible with {expected_kind:?}; actual kind: {actual_kind:?}"
            ),
            Self::InvalidCommandParameterSource {
                source_id,
                source_kind,
                actual_kind,
            } => write!(
                formatter,
                "Command parameter source `{source_id}` is incompatible with {source_kind:?}; actual kind: {actual_kind:?}"
            ),
            Self::RoleRights(error) => {
                write!(formatter, "failed to read EDT role rights: {error}")
            }
            Self::InvalidIdentifier => formatter.write_str("failed to create EDT graph identifier"),
            Self::InvalidName => formatter.write_str("failed to create EDT graph name"),
            Self::InvalidSourceLocation => {
                formatter.write_str("failed to create EDT source location")
            }
            Self::Graph(error) => write!(formatter, "semantic graph error: {error}"),
            Self::ReferenceRequest(error) => {
                write!(formatter, "semantic reference request error: {error}")
            }
            Self::InvalidMetadataReferenceRequest { request_id } => write!(
                formatter,
                "metadata reference projection evidence is invalid for request `{request_id}`"
            ),
            Self::NodePayload(error) => write!(formatter, "semantic graph node error: {error}"),
            Self::Bsl(error) => {
                write!(formatter, "failed to add BSL symbols to graph: {error}")
            }
            Self::FormNavigation(error) => {
                write!(formatter, "failed to emit EDT Form navigation: {error}")
            }
        }
    }
}

fn format_file_system_graph_error(
    error: &EdtGraphError,
    formatter: &mut Formatter<'_>,
) -> std::fmt::Result {
    match error {
        EdtGraphError::ReadDirectory { path, source } => write!(
            formatter,
            "failed to read directory {}: {source}",
            path.display()
        ),
        EdtGraphError::ReadDirectoryEntry { path, source } => write!(
            formatter,
            "failed to read an entry in {}: {source}",
            path.display()
        ),
        EdtGraphError::ReadFileType { path, source } => write!(
            formatter,
            "failed to read file type for {}: {source}",
            path.display()
        ),
        _ => unreachable!("file-system formatter received another error category"),
    }
}

fn format_report_data_composition_graph_error(
    error: &EdtGraphError,
    formatter: &mut Formatter<'_>,
) -> std::fmt::Result {
    match error {
        EdtGraphError::ReportDataComposition(error) => write!(
            formatter,
            "failed to read Report Data Composition source: {error}"
        ),
        EdtGraphError::DataSetPayload(error) => write!(
            formatter,
            "invalid Report Data Composition Data Set: {error}"
        ),
        EdtGraphError::DuplicateDataCompositionNode(id) => write!(
            formatter,
            "Report Data Composition node identity is duplicated: {id}"
        ),
        EdtGraphError::InvalidReportDataCompositionOwner {
            report_id,
            actual_kind,
        } => write!(
            formatter,
            "Report Data Composition owner `{report_id}` is invalid; actual kind: {actual_kind:?}"
        ),
        EdtGraphError::ReportDataCompositionArtifactOutsideProject { project_root, path } => {
            write!(
                formatter,
                "Report Data Composition artifact {} is outside project root {}",
                path.display(),
                project_root.display()
            )
        }
        _ => unreachable!("Data Composition formatter received another error category"),
    }
}

fn format_subsystem_hierarchy_graph_error(
    error: &EdtGraphError,
    formatter: &mut Formatter<'_>,
) -> std::fmt::Result {
    match error {
        EdtGraphError::SubsystemHierarchy(error) => {
            write!(formatter, "failed to read EDT Subsystem hierarchy: {error}")
        }
        EdtGraphError::InvalidSubsystemDescriptorDirectory { path } => write!(
            formatter,
            "EDT Subsystem descriptor has no object directory: {}",
            path.display()
        ),
        EdtGraphError::InvalidSubsystemHierarchyRelation {
            parent_id,
            child_id,
        } => write!(
            formatter,
            "EDT Subsystem hierarchy relation `{parent_id}` -> `{child_id}` has missing source descriptors"
        ),
        EdtGraphError::InvalidSubsystemHierarchyEndpoint {
            node_id,
            actual_kind,
        } => write!(
            formatter,
            "flat Subsystem hierarchy endpoint `{node_id}` is invalid; actual kind: {actual_kind:?}"
        ),
        _ => unreachable!("Subsystem hierarchy formatter received another error category"),
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
            Self::ReportDataComposition(error) => Some(error),
            Self::EventSubscription(error) => Some(error),
            Self::XdtoPackage(error) => Some(error),
            Self::ServiceDescriptor(error) => Some(error),
            Self::MetadataStructure(error) => Some(error),
            Self::Module(error) => Some(error),
            Self::SubsystemContent(error) => Some(error),
            Self::SubsystemHierarchy(error) => Some(error),
            Self::RoleRights(error) => Some(error),
            Self::Graph(error) => Some(error),
            Self::ReferenceRequest(error) => Some(error),
            Self::NodePayload(error) => Some(error),
            Self::DataSetPayload(error) => Some(error),
            Self::InvalidIdentifier
            | Self::InvalidName
            | Self::InvalidSourceLocation
            | Self::InvalidXdtoServiceRequest
            | Self::InvalidMetadataReferenceRequest { .. }
            | Self::InvalidSubsystemDescriptorDirectory { .. }
            | Self::InvalidSubsystemHierarchyRelation { .. }
            | Self::InvalidSubsystemHierarchyEndpoint { .. }
            | Self::SubsystemDescriptorOutsideProject { .. }
            | Self::InvalidSubsystemSource { .. }
            | Self::InvalidFormCommandModuleOwner { .. }
            | Self::InvalidCommandParameterSource { .. }
            | Self::DuplicateDataCompositionNode(_)
            | Self::InvalidReportDataCompositionOwner { .. }
            | Self::ReportDataCompositionArtifactOutsideProject { .. } => None,
            Self::Bsl(error) => Some(error),
            Self::FormNavigation(error) => Some(error),
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
        EdgeKind, FactOrigin, GraphEdge, GraphNode, NodeId, NodeKind, NodeModifiedAspect,
        ResolutionError, ResolutionState, SemanticCoverageCapabilityId, SemanticCoverageStatus,
        SemanticDiagnosticCode, SemanticDiagnosticKind, SemanticDiagnosticSeverity, SemanticGraph,
        SemanticReference, SemanticReferenceCapability, SemanticReferenceCategory,
        SemanticReferenceRequestOutcome, SemanticReferenceStatistics,
    };
    use oneagent_metadata::{MetadataKind, MetadataSpecificPayload};
    use std::collections::BTreeSet;
    use std::fmt::Write as _;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::tempdir;

    use super::{
        EdtGraphError, EdtMetadataReferenceRole, EdtSemanticGraphBuildResult,
        EdtSemanticGraphBuilder, FileSystemEdtSemanticGraphBuilder, MetadataReferenceCollection,
        MetadataReferenceProjectionEvidence, PendingSubsystemContentObservation,
        SubsystemContentTarget, emit_subsystem_includes, normalize_subsystem_content_target,
        resolve_metadata_references,
    };

    const CONFIGURATION_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<mdclass:Configuration
    xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass"
    uuid="11111111-2222-3333-4444-555555555555">
    <name>DemoConfiguration</name>
    <synonym>
        <key>ru</key>
        <content>Демонстрационная конфигурация</content>
    </synonym>
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

    #[derive(Debug, Clone, Copy)]
    struct MetadataReferenceCase {
        directory: &'static str,
        element: &'static str,
        target_name: &'static str,
        reference_type: &'static str,
        member_name: &'static str,
        member_id: &'static str,
        target_id: &'static str,
        target_kind: MetadataKind,
    }

    const METADATA_REFERENCE_CASES: [MetadataReferenceCase; 9] = [
        MetadataReferenceCase {
            directory: "Catalogs",
            element: "Catalog",
            target_name: "Products",
            reference_type: "CatalogRef.Products",
            member_name: "CatalogTarget",
            member_id: "30000000-0000-0000-0000-000000000001",
            target_id: "20000000-0000-0000-0000-000000000001",
            target_kind: MetadataKind::Catalog,
        },
        MetadataReferenceCase {
            directory: "Documents",
            element: "Document",
            target_name: "ReferenceOwner",
            reference_type: "DocumentRef.ReferenceOwner",
            member_name: "DocumentTarget",
            member_id: "30000000-0000-0000-0000-000000000002",
            target_id: "20000000-0000-0000-0000-000000000002",
            target_kind: MetadataKind::Document,
        },
        MetadataReferenceCase {
            directory: "Enums",
            element: "Enum",
            target_name: "Priority",
            reference_type: "EnumRef.Priority",
            member_name: "EnumerationTarget",
            member_id: "30000000-0000-0000-0000-000000000003",
            target_id: "20000000-0000-0000-0000-000000000003",
            target_kind: MetadataKind::Enumeration,
        },
        MetadataReferenceCase {
            directory: "InformationRegisters",
            element: "InformationRegister",
            target_name: "Prices",
            reference_type: "InformationRegisterRecordSet.Prices",
            member_name: "InformationRegisterTarget",
            member_id: "30000000-0000-0000-0000-000000000004",
            target_id: "20000000-0000-0000-0000-000000000004",
            target_kind: MetadataKind::InformationRegister,
        },
        MetadataReferenceCase {
            directory: "AccumulationRegisters",
            element: "AccumulationRegister",
            target_name: "Stock",
            reference_type: "AccumulationRegisterRecordKey.Stock",
            member_name: "AccumulationRegisterTarget",
            member_id: "30000000-0000-0000-0000-000000000005",
            target_id: "20000000-0000-0000-0000-000000000005",
            target_kind: MetadataKind::AccumulationRegister,
        },
        MetadataReferenceCase {
            directory: "AccountingRegisters",
            element: "AccountingRegister",
            target_name: "Ledger",
            reference_type: "AccountingRegisterRecordSet.Ledger",
            member_name: "AccountingRegisterTarget",
            member_id: "30000000-0000-0000-0000-000000000006",
            target_id: "20000000-0000-0000-0000-000000000006",
            target_kind: MetadataKind::AccountingRegister,
        },
        MetadataReferenceCase {
            directory: "CalculationRegisters",
            element: "CalculationRegister",
            target_name: "Payroll",
            reference_type: "CalculationRegisterRecordKey.Payroll",
            member_name: "CalculationRegisterTarget",
            member_id: "30000000-0000-0000-0000-000000000007",
            target_id: "20000000-0000-0000-0000-000000000007",
            target_kind: MetadataKind::CalculationRegister,
        },
        MetadataReferenceCase {
            directory: "BusinessProcesses",
            element: "BusinessProcess",
            target_name: "Approval",
            reference_type: "BusinessProcessRef.Approval",
            member_name: "BusinessProcessTarget",
            member_id: "30000000-0000-0000-0000-000000000008",
            target_id: "20000000-0000-0000-0000-000000000008",
            target_kind: MetadataKind::BusinessProcess,
        },
        MetadataReferenceCase {
            directory: "Tasks",
            element: "Task",
            target_name: "WorkItem",
            reference_type: "TaskRef.WorkItem",
            member_name: "TaskTarget",
            member_id: "30000000-0000-0000-0000-000000000009",
            target_id: "20000000-0000-0000-0000-000000000009",
            target_kind: MetadataKind::Task,
        },
    ];

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
        assert_eq!(
            normalize_subsystem_content_target(
                "Subsystem.StandardSubsystems.Subsystem.ObjectAttributesLock"
            ),
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

    fn create_all_metadata_reference_targets_project() -> tempfile::TempDir {
        let root = tempdir().expect("temporary directory must be created");
        let configuration_directory = root.path().join("src/Configuration");
        fs::create_dir_all(&configuration_directory)
            .expect("configuration directory must be created");
        fs::write(
            configuration_directory.join("Configuration.mdo"),
            CONFIGURATION_XML,
        )
        .expect("configuration descriptor must be created");

        for case in METADATA_REFERENCE_CASES
            .iter()
            .filter(|case| case.target_kind != MetadataKind::Document)
        {
            let directory = root
                .path()
                .join("src")
                .join(case.directory)
                .join(case.target_name);
            fs::create_dir_all(&directory).expect("target directory must be created");
            fs::write(
                directory.join(format!("{}.mdo", case.target_name)),
                format!(
                    r#"<?xml version="1.0" encoding="UTF-8"?>
<mdclass:{element}
    xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass"
    uuid="{target_id}">
    <name>{target_name}</name>
</mdclass:{element}>
"#,
                    element = case.element,
                    target_id = case.target_id,
                    target_name = case.target_name,
                ),
            )
            .expect("target descriptor must be created");
        }

        let attributes =
            METADATA_REFERENCE_CASES
                .iter()
                .fold(String::new(), |mut attributes, case| {
                    write!(
                        attributes,
                        r#"
    <attributes uuid="{member_id}">
        <name>{member_name}</name>
        <type>
            <types>{reference_type}</types>
        </type>
    </attributes>
"#,
                        member_id = case.member_id,
                        member_name = case.member_name,
                        reference_type = case.reference_type,
                    )
                    .expect("metadata reference attribute must be rendered");
                    attributes
                });
        let document_case = METADATA_REFERENCE_CASES
            .iter()
            .find(|case| case.target_kind == MetadataKind::Document)
            .expect("document case must exist");
        let document_directory = root
            .path()
            .join("src/Documents")
            .join(document_case.target_name);
        fs::create_dir_all(&document_directory).expect("document directory must be created");
        fs::write(
            document_directory.join(format!("{}.mdo", document_case.target_name)),
            format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<mdclass:Document
    xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass"
    uuid="{target_id}">
    <name>{target_name}</name>
{attributes}</mdclass:Document>
"#,
                target_id = document_case.target_id,
                target_name = document_case.target_name,
            ),
        )
        .expect("document descriptor must be created");

        root
    }

    fn replace_document_descriptor(root: &tempfile::TempDir, xml: &str) {
        fs::write(root.path().join("src/Documents/Sales/Sales.mdo"), xml)
            .expect("document descriptor must be replaced");
    }

    fn document_with_register_records(declarations: &[&str]) -> String {
        let mut content =
            String::from("    <synonym><key>en</key><content>Sales document</content></synonym>\n");
        for declaration in declarations {
            writeln!(
                content,
                "    <registerRecords>{declaration}</registerRecords>"
            )
            .expect("Document register record must be rendered");
        }

        DOCUMENT_XML.replacen(
            "</mdclass:Document>",
            &format!("{content}</mdclass:Document>"),
            1,
        )
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
            EdgeKind::Opens => "opens",
            EdgeKind::Triggers => "triggers",
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
    fn preserves_common_metadata_payload_without_changing_query_identity() {
        let root = create_edt_project();
        add_common_command_descriptor(&root);

        let first = FileSystemEdtSemanticGraphBuilder
            .build_graph_with_diagnostics(root.path())
            .expect("graph with common payload must build");
        let second = FileSystemEdtSemanticGraphBuilder
            .build_graph_with_diagnostics(root.path())
            .expect("repeated graph build must succeed");
        let query = first.graph().query();
        let configuration = query
            .node(&NodeId::new("11111111-2222-3333-4444-555555555555"))
            .expect("configuration node must exist");
        let catalog = query
            .node(&NodeId::new("11111111-aaaa-bbbb-cccc-222222222222"))
            .expect("catalog node must exist");
        let command = query
            .node(&NodeId::new("cccccccc-1111-2222-3333-444444444444"))
            .expect("command node must exist");

        assert_eq!(
            configuration
                .metadata_payload()
                .expect("configuration payload must exist")
                .common()
                .synonym(),
            Some("Демонстрационная конфигурация")
        );
        assert_eq!(
            catalog
                .metadata_payload()
                .expect("catalog payload must exist")
                .common()
                .synonym(),
            None
        );
        assert_eq!(
            command
                .metadata_payload()
                .expect("command payload must exist")
                .common()
                .synonym(),
            Some("Refresh data")
        );
        assert_eq!(
            query.nodes_by_name(&EntityName::new("RefreshData").expect("name must be valid")),
            vec![command]
        );
        assert!(
            query
                .nodes_by_name(&EntityName::new("Refresh data").expect("name must be valid"))
                .is_empty()
        );
        assert!(first.validate().is_valid());
        assert!(first.graph().diff(second.graph()).is_empty());
        assert!(first.diff(&second).is_empty());
    }

    fn assert_document_payload(
        result: &EdtSemanticGraphBuildResult,
        expected_records: &[(MetadataKind, &str)],
    ) {
        let query = result.graph().query();
        let document = query
            .node(&NodeId::new("aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee"))
            .expect("Document must be queryable by stable UUID");
        let payload = document
            .metadata_payload()
            .expect("Document metadata payload must exist");
        let Some(MetadataSpecificPayload::Document(document_payload)) = payload.specific() else {
            panic!("Document-specific payload must exist");
        };
        let records = document_payload
            .register_records()
            .iter()
            .map(|record| (record.target_kind(), record.target_name().as_str()))
            .collect::<Vec<_>>();

        assert_eq!(document.kind(), NodeKind::Metadata(MetadataKind::Document));
        assert_eq!(payload.common().synonym(), Some("Sales document"));
        assert_eq!(records, expected_records);
        assert_eq!(
            query.nodes_by_name(&EntityName::new("Sales").expect("name must be valid")),
            vec![document]
        );
        assert!(
            query
                .nodes_by_name(
                    &EntityName::new("Sales document").expect("synonym must be a valid name")
                )
                .is_empty()
        );
        assert!(query.edges_by_kind(EdgeKind::Writes).is_empty());
        assert!(result.validate().is_valid());
    }

    fn assert_payload_only_document_change(
        previous: &EdtSemanticGraphBuildResult,
        current: &EdtSemanticGraphBuildResult,
    ) {
        let document_id = NodeId::new("aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee");
        let graph_diff = previous.graph().diff(current.graph());
        let build_diff = previous.diff(current);

        assert!(graph_diff.added_nodes().is_empty());
        assert!(graph_diff.removed_nodes().is_empty());
        assert_eq!(graph_diff.modified_nodes().len(), 1);
        assert_eq!(graph_diff.modified_nodes()[0].id(), &document_id);
        assert_eq!(
            graph_diff.modified_nodes()[0].modified_aspects(),
            &[NodeModifiedAspect::SemanticContent]
        );
        assert!(graph_diff.added_edges().is_empty());
        assert!(graph_diff.removed_edges().is_empty());
        assert!(graph_diff.modified_edges().is_empty());
        assert_eq!(build_diff.summary().node_changes(), 1);
        assert_eq!(build_diff.summary().edge_changes(), 0);
        assert_eq!(build_diff.summary().diagnostic_changes(), 0);
        assert_eq!(build_diff.summary().resolution_metric_changes(), 0);
        assert!(
            current
                .graph()
                .query()
                .edges_by_kind(EdgeKind::Writes)
                .is_empty()
        );
        assert!(current.validate().is_valid());
    }

    #[test]
    fn preserves_canonical_document_register_record_payload() {
        let root = create_edt_project();
        let first_order = [
            "CalculationRegister.Payroll",
            "AccumulationRegister.StockBalance",
            "InformationRegister.Prices",
            "AccountingRegister.GeneralLedger",
            "InformationRegister.Prices",
            "LocalizedRegister.Hidden",
            "NameOnly",
            "AccumulationRegister.CaseTarget",
            "AccumulationRegister.CASETARGET",
        ];
        replace_document_descriptor(&root, &document_with_register_records(&first_order));

        let first = FileSystemEdtSemanticGraphBuilder
            .build_graph_with_diagnostics(root.path())
            .expect("graph with Document payload must build");
        let repeated = FileSystemEdtSemanticGraphBuilder
            .build_graph_with_diagnostics(root.path())
            .expect("repeated graph build must succeed");

        assert_document_payload(
            &first,
            &[
                (MetadataKind::InformationRegister, "Prices"),
                (MetadataKind::AccumulationRegister, "StockBalance"),
                (MetadataKind::AccountingRegister, "GeneralLedger"),
                (MetadataKind::CalculationRegister, "Payroll"),
            ],
        );
        assert!(first.graph().diff(repeated.graph()).is_empty());
        assert!(first.diff(&repeated).is_empty());

        let reordered = [
            "NameOnly",
            "InformationRegister.Prices",
            "AccumulationRegister.CASETARGET",
            "LocalizedRegister.Hidden",
            "AccountingRegister.GeneralLedger",
            "AccumulationRegister.StockBalance",
            "InformationRegister.Prices",
            "AccumulationRegister.CaseTarget",
            "CalculationRegister.Payroll",
        ];
        replace_document_descriptor(&root, &document_with_register_records(&reordered));
        let reordered = FileSystemEdtSemanticGraphBuilder
            .build_graph_with_diagnostics(root.path())
            .expect("reordered Document payload must build");

        assert!(first.graph().diff(reordered.graph()).is_empty());
        assert!(first.diff(&reordered).is_empty());

        let changed = [
            "InformationRegister.Prices",
            "AccumulationRegister.StockBalance",
            "AccountingRegister.GeneralLedger",
        ];
        replace_document_descriptor(&root, &document_with_register_records(&changed));
        let changed = FileSystemEdtSemanticGraphBuilder
            .build_graph_with_diagnostics(root.path())
            .expect("changed Document payload must build");

        assert_payload_only_document_change(&first, &changed);
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
        assert_eq!(capability.status(), SemanticCoverageStatus::Supported);
        assert_eq!(capability.evidence(), capability.required_evidence());
        assert!(capability.missing_evidence().is_empty());
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
        let location = query.provenance()[0]
            .location()
            .expect("query location must exist");
        assert!(
            location
                .path()
                .as_str()
                .ends_with("/src/Documents/Sales/ObjectModule.bsl")
        );
        let point = location.span().expect("query declaration point must exist");
        assert_eq!(point.start().line(), 2);
        assert_eq!(point.start().column(), 1);
        assert_eq!(point.start(), point.end());
    }

    fn assert_query_data_dependencies(graph: &SemanticGraph, query_ids: [&NodeId; 2]) {
        let query = graph.query();
        assert!(query.edges_by_kind(EdgeKind::Writes).is_empty());
        assert_eq!(
            query_ids
                .iter()
                .map(|query_id| {
                    query
                        .outgoing_edges_by_kind(query_id, EdgeKind::DependsOn)
                        .len()
                })
                .sum::<usize>(),
            2
        );
        for query_id in query_ids {
            assert_eq!(
                query
                    .outgoing_edges_by_kind(query_id, EdgeKind::DependsOn)
                    .len(),
                1
            );
        }
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
        assert_query_data_dependencies(graph, [&before_write_query_id, &get_query_id]);
        assert_eq!(
            first
                .reference_request_query()
                .by_category(SemanticReferenceCategory::QuerySource)
                .len(),
            2
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
        assert_edt_source_locations(object_module, before_write);

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

    fn assert_edt_source_locations(object_module: &GraphNode, before_write: &GraphNode) {
        let module_location = object_module.provenance()[0]
            .location()
            .expect("module location must exist");
        assert!(
            module_location
                .path()
                .as_str()
                .ends_with("/src/Documents/Sales/ObjectModule.bsl")
        );
        assert_eq!(module_location.span(), None);

        let procedure_location = before_write.provenance()[0]
            .location()
            .expect("procedure location must exist");
        let point = procedure_location
            .span()
            .expect("procedure declaration point must exist");
        assert_eq!(point.start().line(), 1);
        assert_eq!(point.start().column(), 1);
        assert_eq!(point.start(), point.end());
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
        assert_reference_request_observability(&result, &repeated);
        assert!(graph.diff(repeated.graph()).is_empty());
        assert!(result.diff(&repeated).is_empty());
        assert!(result.diagnostics().is_empty());
    }

    fn assert_reference_request_observability(
        result: &EdtSemanticGraphBuildResult,
        repeated: &EdtSemanticGraphBuildResult,
    ) {
        assert_eq!(result.reference_requests().len(), 4);
        assert_eq!(result.reference_requests(), repeated.reference_requests());
        assert_eq!(
            result
                .reference_request_query()
                .by_category(SemanticReferenceCategory::MetadataType)
                .len(),
            4
        );
        for request in result.reference_requests() {
            assert_eq!(request.category(), SemanticReferenceCategory::MetadataType);
            assert_eq!(request.outcome(), SemanticReferenceRequestOutcome::Resolved);
            assert_eq!(request.state(), ResolutionState::Resolved);
            assert_eq!(request.candidates().len(), 1);
            assert_eq!(request.provenance().len(), 2);
            let collection = request
                .provenance()
                .iter()
                .find(|provenance| provenance.origin() == FactOrigin::Declared)
                .expect("collection provenance must exist");
            let resolution = request
                .provenance()
                .iter()
                .find(|provenance| provenance.origin() == FactOrigin::Resolved)
                .expect("resolver provenance must exist");
            assert_eq!(
                collection.producer().as_str(),
                super::EDT_METADATA_REFERENCE_COLLECTION_PRODUCER
            );
            assert_eq!(collection.resolution(), ResolutionState::Unresolved);
            assert_eq!(resolution.resolution(), ResolutionState::Resolved);
        }
        assert_eq!(result.reference_statistics().total(), 5);
        assert!(result.validate().is_valid());
    }

    fn assert_mapped_metadata_reference_case(
        result: &EdtSemanticGraphBuildResult,
        repeated: &EdtSemanticGraphBuildResult,
        case: MetadataReferenceCase,
    ) {
        let query = result.graph().query();
        let repeated_query = repeated.graph().query();
        let member_id = EntityId::new(case.member_id).expect("member id must be valid");
        let target_id = EntityId::new(case.target_id).expect("target id must be valid");
        let member = result
            .graph()
            .node(&member_id)
            .expect("source metadata member must exist");
        let target = result
            .graph()
            .node(&target_id)
            .expect("target metadata object must exist");
        assert_eq!(member.kind(), NodeKind::Attribute);
        assert_eq!(member.name().as_str(), case.member_name);
        assert_eq!(target.kind(), NodeKind::Metadata(case.target_kind));
        assert_eq!(target.name().as_str(), case.target_name);

        let member_node_id = NodeId::new(case.member_id);
        assert_eq!(
            query
                .node(&member_node_id)
                .expect("Query must expose source member")
                .id(),
            &member_id
        );
        let references = query.outgoing_edges_by_kind(&member_node_id, EdgeKind::References);
        let dependencies = query.outgoing_edges_by_kind(&member_node_id, EdgeKind::DependsOn);
        let repeated_references =
            repeated_query.outgoing_edges_by_kind(&member_node_id, EdgeKind::References);
        let repeated_dependencies =
            repeated_query.outgoing_edges_by_kind(&member_node_id, EdgeKind::DependsOn);
        assert_eq!(references.len(), 1);
        assert_eq!(dependencies.len(), 1);
        assert_eq!(references[0].target(), &target_id);
        assert_eq!(dependencies[0].target(), &target_id);
        assert_eq!(references, repeated_references);
        assert_eq!(dependencies, repeated_dependencies);
        let request = result
            .reference_request_query()
            .by_source(&member_id)
            .into_iter()
            .next()
            .expect("mapped request must be observable");
        let repeated_request = repeated
            .reference_request_query()
            .request(request.id())
            .expect("repeated request identity must be stable");
        assert_eq!(request, repeated_request);
        assert_eq!(request.outcome(), SemanticReferenceRequestOutcome::Resolved);
        assert_eq!(request.state(), ResolutionState::Resolved);
        assert_eq!(
            request.expected_kinds(),
            &[NodeKind::Metadata(case.target_kind)]
        );
        assert_eq!(request.candidates(), std::slice::from_ref(&target_id));
        assert!(matches!(
            request.reference(),
            SemanticReference::Name(name) if name.as_str() == case.target_name
        ));
        assert!(request.provenance().iter().any(|provenance| {
            provenance.origin() == FactOrigin::Declared
                && provenance.resolution() == ResolutionState::Unresolved
        }));
        assert!(request.provenance().iter().any(|provenance| {
            provenance.origin() == FactOrigin::Resolved
                && provenance.resolution() == ResolutionState::Resolved
        }));
        assert_eq!(references[0].provenance().len(), 1);
        assert_eq!(dependencies[0].provenance().len(), 1);
        assert_eq!(references[0].provenance()[0].origin(), FactOrigin::Resolved);
        assert_eq!(
            references[0].provenance()[0].resolution(),
            ResolutionState::Resolved
        );
        assert_eq!(
            dependencies[0].provenance()[0].origin(),
            FactOrigin::Derived
        );
        assert_eq!(
            dependencies[0].provenance()[0].resolution(),
            ResolutionState::Resolved
        );
        let expected_target_context = format!(
            "target_kind={};target_name={}",
            case.target_kind.as_str(),
            case.target_name
        );
        for edge in [references[0], dependencies[0]] {
            assert!(
                edge.provenance()[0]
                    .source()
                    .expect("reference provenance source must exist")
                    .as_str()
                    .contains(&expected_target_context)
            );
        }
    }

    #[test]
    fn resolves_all_mapped_metadata_reference_target_kinds_through_production_builder() {
        let root = create_all_metadata_reference_targets_project();
        let result = FileSystemEdtSemanticGraphBuilder
            .build_graph_with_diagnostics(root.path())
            .expect("graph must build");
        let repeated = FileSystemEdtSemanticGraphBuilder
            .build_graph_with_diagnostics(root.path())
            .expect("repeated graph must build");

        assert!(result.diagnostics().is_empty());
        assert!(result.validate().is_valid());
        for actual in [
            result.reference_statistics().total(),
            result.reference_statistics().resolved(),
            result.reference_statistics().outcome_total(),
            result.reference_statistics().with_provenance(),
            result.report().resolution().resolved(),
        ] {
            assert_eq!(actual, METADATA_REFERENCE_CASES.len());
        }
        assert!(
            result
                .graph()
                .nodes_by_kind(NodeKind::Metadata(MetadataKind::Unknown))
                .is_empty()
        );
        assert!(result.graph().nodes_by_kind(NodeKind::Unknown).is_empty());
        for case in METADATA_REFERENCE_CASES {
            assert_mapped_metadata_reference_case(&result, &repeated, case);
        }

        assert_eq!(
            result.graph().nodes().collect::<Vec<_>>(),
            repeated.graph().nodes().collect::<Vec<_>>()
        );
        assert_eq!(
            result.graph().edges().collect::<Vec<_>>(),
            repeated.graph().edges().collect::<Vec<_>>()
        );
        assert!(result.graph().diff(repeated.graph()).is_empty());
        assert!(result.diff(&repeated).is_empty());
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
        let repeated = FileSystemEdtSemanticGraphBuilder
            .build_graph_with_diagnostics(root.path())
            .expect("repeated duplicate build must succeed");
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
        let requests = result.reference_request_query().by_source(&company_id);
        assert_eq!(requests.len(), 1);
        assert_eq!(
            requests[0].outcome(),
            SemanticReferenceRequestOutcome::Resolved
        );
        assert_eq!(requests[0].provenance().len(), 2);
        assert_eq!(result.reference_statistics().total(), 4);
        assert_eq!(result.reference_statistics().resolved(), 4);
        assert_eq!(result.reference_requests(), repeated.reference_requests());
        assert!(result.diff(&repeated).is_empty());
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
        assert_eq!(diff.reference_requests().added().len(), 2);
        assert_eq!(diff.reference_requests().removed().len(), 1);
        assert_eq!(diff.reference_requests().modified().len(), 2);
        assert!(diff.reference_requests().modified().iter().all(|change| {
            change.modified_aspects()
                == [oneagent_graph::ReferenceRequestModifiedAspect::Provenance]
        }));
        assert_eq!(diff.summary().reference_request_changes(), 5);
    }

    #[test]
    fn request_identity_survives_missing_to_resolved_production_diff() {
        let root = create_edt_project();
        replace_document_descriptor(
            &root,
            &document_with_reference("CatalogRef.MissingProducts"),
        );
        let previous = FileSystemEdtSemanticGraphBuilder
            .build_graph_with_diagnostics(root.path())
            .expect("missing-target snapshot must build");
        add_catalog_descriptor(
            &root,
            "MissingProducts",
            "33333333-aaaa-bbbb-cccc-444444444444",
            "MissingProducts",
        );
        let current = FileSystemEdtSemanticGraphBuilder
            .build_graph_with_diagnostics(root.path())
            .expect("resolved snapshot must build");
        let company_id = company_attribute_id();
        let previous_request = previous
            .reference_request_query()
            .by_source(&company_id)
            .into_iter()
            .next()
            .expect("previous request must exist");
        let current_request = current
            .reference_request_query()
            .by_source(&company_id)
            .into_iter()
            .next()
            .expect("current request must exist");
        let diff = previous.diff(&current);

        assert_eq!(previous_request.id(), current_request.id());
        assert_eq!(
            previous_request.outcome(),
            SemanticReferenceRequestOutcome::MissingTarget
        );
        assert_eq!(
            current_request.outcome(),
            SemanticReferenceRequestOutcome::Resolved
        );
        assert!(diff.reference_requests().added().is_empty());
        assert!(diff.reference_requests().removed().is_empty());
        assert_eq!(diff.reference_requests().modified().len(), 1);
        assert_eq!(
            diff.reference_requests().modified()[0].modified_aspects(),
            &[
                oneagent_graph::ReferenceRequestModifiedAspect::Candidates,
                oneagent_graph::ReferenceRequestModifiedAspect::State,
                oneagent_graph::ReferenceRequestModifiedAspect::Outcome,
                oneagent_graph::ReferenceRequestModifiedAspect::Provenance,
            ]
        );
        assert_eq!(diff.diagnostics().removed().len(), 1);
        assert_eq!(
            current
                .graph()
                .outgoing_by_kind(&company_id, EdgeKind::References)
                .len(),
            1
        );
        assert_eq!(
            current
                .graph()
                .outgoing_by_kind(&company_id, EdgeKind::DependsOn)
                .len(),
            1
        );
        assert!(previous.validate().is_valid());
        assert!(current.validate().is_valid());
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
        let request = result
            .reference_request_query()
            .by_source(&company_id)
            .into_iter()
            .next()
            .expect("missing request must be observable");
        assert_eq!(
            request.outcome(),
            SemanticReferenceRequestOutcome::MissingTarget
        );
        assert_eq!(request.state(), ResolutionState::Unresolved);
        assert!(request.candidates().is_empty());
        assert!(request.provenance().iter().any(|provenance| {
            provenance.origin() == FactOrigin::Declared
                && provenance.resolution() == ResolutionState::Unresolved
        }));
        assert!(request.provenance().iter().any(|provenance| {
            provenance.origin() == FactOrigin::Resolved
                && provenance.resolution() == ResolutionState::Unresolved
        }));
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
        assert!(result.validate().is_valid());
        assert_eq!(
            graph
                .outgoing_by_kind(product_dimension.id(), EdgeKind::DependsOn)
                .len(),
            1
        );
    }

    #[test]
    fn partial_workspace_preserves_request_without_failure_projection() {
        let source_id = EntityId::new("attribute.partial").expect("source id must be valid");
        let mut graph = SemanticGraph::new();
        graph.insert_node(GraphNode::new(
            source_id.clone(),
            EntityName::new("PartialAttribute").expect("source name must be valid"),
            NodeKind::Attribute,
        ));
        let mut collected = MetadataReferenceCollection::default();
        collected
            .insert(MetadataReferenceProjectionEvidence {
                descriptor_path: PathBuf::from("src/Documents/Partial/Partial.mdo"),
                metadata_object_id: EntityId::new("document.partial")
                    .expect("metadata object id must be valid"),
                source_id: source_id.clone(),
                role: EdtMetadataReferenceRole::Type,
                target_kind: MetadataKind::Catalog,
                target_name: EntityName::new("AbsentInPartialWorkspace")
                    .expect("target name must be valid"),
                raw_token: None,
                occurrence_count: 1,
            })
            .expect("accepted request must be collected");
        let collected_id = collected.requests().requests()[0].id().clone();
        let mut diagnostics = BTreeSet::new();

        let terminal = resolve_metadata_references(
            &mut graph,
            &collected,
            &mut diagnostics,
            super::query_source_resolution::WorkspaceResolutionScope::Partial,
        )
        .expect("partial resolution must succeed");
        let request = terminal
            .query()
            .request(&collected_id)
            .expect("terminal request must preserve identity");

        assert_eq!(
            request.outcome(),
            SemanticReferenceRequestOutcome::PartialWorkspace
        );
        assert_eq!(request.state(), ResolutionState::Partial);
        assert!(request.candidates().is_empty());
        assert!(diagnostics.is_empty());
        assert!(graph.edges().next().is_none());
        let statistics = SemanticReferenceStatistics::from_reference_requests(&terminal);
        assert_eq!(statistics.total(), 1);
        assert_eq!(statistics.unresolved(), 1);
    }

    #[test]
    fn command_request_identity_survives_partial_to_resolved_transition() {
        let source_id = EntityId::new("command.partial").expect("command identifier must be valid");
        let target_id =
            EntityId::new("catalog.partial-target").expect("target identifier must be valid");
        let mut graph = SemanticGraph::new();
        graph.insert_node(GraphNode::new(
            source_id.clone(),
            EntityName::new("PartialCommand").expect("command name must be valid"),
            NodeKind::Command,
        ));
        let mut collected = MetadataReferenceCollection::default();
        collected
            .insert(MetadataReferenceProjectionEvidence {
                descriptor_path: PathBuf::from("src/Documents/Partial/Partial.mdo"),
                metadata_object_id: EntityId::new("document.partial")
                    .expect("metadata object id must be valid"),
                source_id: source_id.clone(),
                role: EdtMetadataReferenceRole::CommandParameterType,
                target_kind: MetadataKind::Catalog,
                target_name: EntityName::new("PartialTarget").expect("target name must be valid"),
                raw_token: Some("CatalogRef.PartialTarget".to_owned()),
                occurrence_count: 2,
            })
            .expect("accepted command request must be collected");
        let collected_id = collected.requests().requests()[0].id().clone();
        let mut partial_diagnostics = BTreeSet::new();
        let partial = resolve_metadata_references(
            &mut graph,
            &collected,
            &mut partial_diagnostics,
            super::query_source_resolution::WorkspaceResolutionScope::Partial,
        )
        .expect("partial command resolution must succeed");

        graph.insert_node(GraphNode::new(
            target_id.clone(),
            EntityName::new("PartialTarget").expect("target name must be valid"),
            NodeKind::Metadata(MetadataKind::Catalog),
        ));
        let mut complete_diagnostics = BTreeSet::new();
        let complete = resolve_metadata_references(
            &mut graph,
            &collected,
            &mut complete_diagnostics,
            super::query_source_resolution::WorkspaceResolutionScope::Complete,
        )
        .expect("complete command resolution must succeed");
        let partial_request = partial
            .query()
            .request(&collected_id)
            .expect("partial request must remain observable");
        let complete_request = complete
            .query()
            .request(&collected_id)
            .expect("resolved request must retain identity");

        assert_eq!(partial_request.id(), complete_request.id());
        assert_eq!(
            partial_request.outcome(),
            SemanticReferenceRequestOutcome::PartialWorkspace
        );
        assert_eq!(partial_request.state(), ResolutionState::Partial);
        assert_eq!(
            complete_request.outcome(),
            SemanticReferenceRequestOutcome::Resolved
        );
        assert_eq!(complete_request.state(), ResolutionState::Resolved);
        assert_eq!(complete_request.candidates(), &[target_id]);
        assert!(partial_diagnostics.is_empty());
        assert!(complete_diagnostics.is_empty());
        assert_eq!(
            graph
                .outgoing_by_kind(&source_id, EdgeKind::References)
                .len(),
            1
        );
        assert_eq!(
            graph
                .outgoing_by_kind(&source_id, EdgeKind::DependsOn)
                .len(),
            1
        );
    }

    #[test]
    fn production_builder_preserves_explicit_partial_workspace_request() {
        let root = create_edt_project();
        replace_document_descriptor(
            &root,
            &document_with_reference("CatalogRef.MissingProducts"),
        );

        let result = FileSystemEdtSemanticGraphBuilder::build_graph_with_metadata_reference_scope(
            root.path(),
            super::query_source_resolution::WorkspaceResolutionScope::Partial,
        )
        .expect("partial production build must succeed");
        let company_id = company_attribute_id();
        let request = result
            .reference_request_query()
            .by_source(&company_id)
            .into_iter()
            .next()
            .expect("partial request must be observable");

        assert_eq!(
            request.outcome(),
            SemanticReferenceRequestOutcome::PartialWorkspace
        );
        assert_eq!(request.state(), ResolutionState::Partial);
        assert!(request.candidates().is_empty());
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
        assert_eq!(result.reference_statistics().total(), 4);
        assert_eq!(result.reference_statistics().resolved(), 3);
        assert_eq!(result.reference_statistics().unresolved(), 1);
        assert!(result.validate().is_valid());
    }

    #[test]
    fn production_builder_propagates_explicit_partial_scope_to_query_requests() {
        let root = create_edt_project();
        replace_object_module(
            &root,
            concat!(
                "Procedure BeforeWrite()\n",
                "    Query = New Query;\n",
                "    Query.Text = \"SELECT Ref FROM Catalog.MissingProducts\";\n",
                "EndProcedure",
            ),
        );

        let result = FileSystemEdtSemanticGraphBuilder::build_graph_with_metadata_reference_scope(
            root.path(),
            super::query_source_resolution::WorkspaceResolutionScope::Partial,
        )
        .expect("partial Query production build must succeed");
        let requests = result
            .reference_request_query()
            .by_category(SemanticReferenceCategory::QuerySource);

        assert_eq!(requests.len(), 1);
        assert_eq!(
            requests[0].outcome(),
            SemanticReferenceRequestOutcome::PartialWorkspace
        );
        assert_eq!(requests[0].state(), ResolutionState::Partial);
        assert!(requests[0].candidates().is_empty());
        assert!(
            result
                .graph()
                .outgoing_by_kind(requests[0].source_node(), EdgeKind::Reads)
                .is_empty()
        );
        assert!(
            result
                .graph()
                .outgoing_by_kind(requests[0].source_node(), EdgeKind::DependsOn)
                .is_empty()
        );
        let diagnostic = result
            .diagnostics()
            .iter()
            .find(|diagnostic| diagnostic.source_node() == Some(requests[0].source_node()))
            .expect("partial Query request must retain its warning projection");
        assert_eq!(diagnostic.severity(), SemanticDiagnosticSeverity::Warning);
        assert_eq!(diagnostic.reference(), requests[0].reference());
        assert!(result.validate().is_valid());
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
        let request = result
            .reference_request_query()
            .by_source(&company_id)
            .into_iter()
            .next()
            .expect("ambiguous request must be observable");
        assert_eq!(
            request.outcome(),
            SemanticReferenceRequestOutcome::AmbiguousTarget
        );
        assert_eq!(request.state(), ResolutionState::Ambiguous);
        assert_eq!(request.candidates().len(), 2);
        assert_eq!(
            diagnostics
                .iter()
                .find(|diagnostic| diagnostic.source_node() == Some(&company_id))
                .expect("Company diagnostic must exist")
                .candidates(),
            request.candidates()
        );
        assert!(result.validate().is_valid());
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
        let request = result
            .reference_request_query()
            .by_source(&company_id)
            .into_iter()
            .next()
            .expect("incompatible request must be observable");
        assert_eq!(
            request.outcome(),
            SemanticReferenceRequestOutcome::IncompatibleTargetKind
        );
        assert_eq!(request.state(), ResolutionState::Unresolved);
        assert_eq!(request.candidates().len(), 1);
        assert_eq!(diagnostics[0].candidates(), request.candidates());
        assert!(
            result
                .graph()
                .outgoing_by_kind(&company_id, EdgeKind::References)
                .is_empty()
        );
        assert!(result.validate().is_valid());
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
        assert_eq!(
            result
                .reference_request_query()
                .by_source(&company_attribute_id())
                .len(),
            1
        );
        assert_eq!(result.reference_statistics().unresolved(), 1);
        assert!(result.validate().is_valid());
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
