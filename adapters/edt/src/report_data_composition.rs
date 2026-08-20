//! Typed reader for Report Data Composition Schema declarations and artifacts.

use oneagent_common::{EntityId, EntityName};
use oneagent_graph::{
    DataCompositionIdentityError, DataSetKind, data_composition_field_id, data_set_id,
};
use oneagent_metadata::MetadataKind;
use quick_xml::Reader;
use quick_xml::escape::unescape;
use quick_xml::events::{BytesCData, BytesRef, BytesStart, BytesText, Event};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::{Component, Path, PathBuf};

use crate::EdtMetadataObjectDescriptor;

const METADATA_NAMESPACE: &str = "http://g5.1c.ru/v8/dt/metadata/mdclass";
const DATA_COMPOSITION_NAMESPACE: &str = "http://v8.1c.ru/8.1/data-composition-system/schema";
const XSI_NAMESPACE: &str = "http://www.w3.org/2001/XMLSchema-instance";

/// Parsed Report Data Composition source model without graph projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdtReportDataCompositionDescriptor {
    report_id: EntityId,
    report_name: EntityName,
    descriptor_path: PathBuf,
    schemas: Vec<EdtDataCompositionSchemaDescriptor>,
}

impl EdtReportDataCompositionDescriptor {
    /// Returns the canonical Report UUID accepted by the metadata reader.
    #[must_use]
    pub const fn report_id(&self) -> &EntityId {
        &self.report_id
    }

    /// Returns the canonical Report name accepted by the metadata reader.
    #[must_use]
    pub const fn report_name(&self) -> &EntityName {
        &self.report_name
    }

    /// Returns the joined Report descriptor path.
    #[must_use]
    pub fn descriptor_path(&self) -> &Path {
        &self.descriptor_path
    }

    /// Returns schemas ordered by their source UUID identity.
    #[must_use]
    pub fn schemas(&self) -> &[EdtDataCompositionSchemaDescriptor] {
        &self.schemas
    }

    /// Returns every recoverable deferred or unsupported source observation.
    pub fn observations(&self) -> impl Iterator<Item = &EdtDataCompositionObservation> {
        self.schemas
            .iter()
            .flat_map(|schema| schema.observations.iter())
    }
}

/// One accepted UUID-backed Data Composition Schema artifact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdtDataCompositionSchemaDescriptor {
    id: EntityId,
    name: EntityName,
    main: bool,
    artifact_path: PathBuf,
    data_source: Option<EdtDataCompositionDataSource>,
    data_sets: Vec<EdtDataCompositionDataSet>,
    observations: Vec<EdtDataCompositionObservation>,
}

impl EdtDataCompositionSchemaDescriptor {
    /// Returns the exact template UUID used as canonical Schema identity.
    #[must_use]
    pub const fn id(&self) -> &EntityId {
        &self.id
    }

    /// Returns the exact declared template name.
    #[must_use]
    pub const fn name(&self) -> &EntityName {
        &self.name
    }

    /// Returns whether the Report selects this Schema as its main schema.
    #[must_use]
    pub const fn is_main(&self) -> bool {
        self.main
    }

    /// Returns the exact joined `.dcs` artifact path.
    #[must_use]
    pub fn artifact_path(&self) -> &Path {
        &self.artifact_path
    }

    /// Returns the accepted root local data source, when declared.
    #[must_use]
    pub const fn data_source(&self) -> Option<&EdtDataCompositionDataSource> {
        self.data_source.as_ref()
    }

    /// Returns direct accepted Data Sets ordered by stable owner-scoped identity.
    #[must_use]
    pub fn data_sets(&self) -> &[EdtDataCompositionDataSet] {
        &self.data_sets
    }

    /// Returns canonical recoverable observations for this Schema.
    #[must_use]
    pub fn observations(&self) -> &[EdtDataCompositionObservation] {
        &self.observations
    }
}

/// Accepted root Data Composition data source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdtDataCompositionDataSource {
    name: EntityName,
}

impl EdtDataCompositionDataSource {
    /// Returns the exact local source name. The accepted slice requires `DataSource1`.
    #[must_use]
    pub const fn name(&self) -> &EntityName {
        &self.name
    }
}

/// One accepted direct Data Set declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdtDataCompositionDataSet {
    id: EntityId,
    name: EntityName,
    kind: DataSetKind,
    data_source: Option<EntityName>,
    fields: Vec<EdtDataCompositionField>,
    query: Option<String>,
}

impl EdtDataCompositionDataSet {
    /// Returns the stable owner-scoped Data Set identity.
    #[must_use]
    pub const fn id(&self) -> &EntityId {
        &self.id
    }

    /// Returns the exact direct Data Set name.
    #[must_use]
    pub const fn name(&self) -> &EntityName {
        &self.name
    }

    /// Returns the accepted Data Set kind.
    #[must_use]
    pub const fn kind(&self) -> DataSetKind {
        self.kind
    }

    /// Returns the exact referenced local data source for Query/Object Data Sets.
    #[must_use]
    pub const fn data_source(&self) -> Option<&EntityName> {
        self.data_source.as_ref()
    }

    /// Returns direct named fields ordered by stable owner-scoped identity.
    #[must_use]
    pub fn fields(&self) -> &[EdtDataCompositionField] {
        &self.fields
    }

    /// Returns the complete opaque Query text for a Query Data Set.
    #[must_use]
    pub fn query(&self) -> Option<&str> {
        self.query.as_deref()
    }
}

/// One accepted direct named Data Composition Field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdtDataCompositionField {
    id: EntityId,
    name: EntityName,
    data_path: EntityName,
}

impl EdtDataCompositionField {
    /// Returns the stable owner-scoped Field identity.
    #[must_use]
    pub const fn id(&self) -> &EntityId {
        &self.id
    }

    /// Returns the exact direct field name.
    #[must_use]
    pub const fn name(&self) -> &EntityName {
        &self.name
    }

    /// Returns the exact direct field data path.
    #[must_use]
    pub const fn data_path(&self) -> &EntityName {
        &self.data_path
    }
}

/// Recoverable source classification that intentionally emits no graph entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EdtDataCompositionObservationKind {
    /// A nested Data Set has no accepted stable first-slice identity.
    NestedDataSet,
    /// A direct field folder is outside the named Field entity contract.
    FieldFolder,
    /// A direct Data Set has an unknown `xsi:type`.
    UnsupportedDataSetType,
    /// A direct field has an unknown `xsi:type`.
    UnsupportedFieldType,
}

/// Deterministic occurrence context for a recoverable source construct.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EdtDataCompositionObservation {
    kind: EdtDataCompositionObservationKind,
    artifact_path: PathBuf,
    owner_id: EntityId,
    raw_type: Option<String>,
    occurrence_ordinal: usize,
}

impl EdtDataCompositionObservation {
    /// Returns the recoverable source classification.
    #[must_use]
    pub const fn kind(&self) -> EdtDataCompositionObservationKind {
        self.kind
    }

    /// Returns the artifact containing the observation.
    #[must_use]
    pub fn artifact_path(&self) -> &Path {
        &self.artifact_path
    }

    /// Returns the nearest accepted Schema or Data Set owner identity.
    #[must_use]
    pub const fn owner_id(&self) -> &EntityId {
        &self.owner_id
    }

    /// Returns the exact decoded unknown `xsi:type`, when applicable.
    #[must_use]
    pub fn raw_type(&self) -> Option<&str> {
        self.raw_type.as_deref()
    }

    /// Returns the zero-based XML occurrence ordinal within this artifact.
    #[must_use]
    pub const fn occurrence_ordinal(&self) -> usize {
        self.occurrence_ordinal
    }
}

/// Reads Report Data Composition declarations joined to their `.dcs` artifacts.
pub trait EdtReportDataCompositionReader {
    /// Reads source content for one already accepted top-level Report descriptor.
    ///
    /// # Errors
    ///
    /// Returns a typed fatal error when the Report declaration or one selected
    /// first-slice Schema subtree cannot be accepted completely.
    fn read(
        &self,
        report_directory: &Path,
        report: &EdtMetadataObjectDescriptor,
    ) -> Result<EdtReportDataCompositionDescriptor, EdtReportDataCompositionError>;
}

/// Filesystem implementation of [`EdtReportDataCompositionReader`].
#[derive(Debug, Default, Clone, Copy)]
pub struct FileSystemEdtReportDataCompositionReader;

impl EdtReportDataCompositionReader for FileSystemEdtReportDataCompositionReader {
    fn read(
        &self,
        report_directory: &Path,
        report: &EdtMetadataObjectDescriptor,
    ) -> Result<EdtReportDataCompositionDescriptor, EdtReportDataCompositionError> {
        if report.kind() != MetadataKind::Report {
            return Err(EdtReportDataCompositionError::UnexpectedMetadataKind(
                report.kind(),
            ));
        }
        if !report_directory.is_dir() {
            return Err(EdtReportDataCompositionError::ReportDirectoryNotFound(
                report_directory.to_path_buf(),
            ));
        }

        let descriptor_xml = fs::read_to_string(report.descriptor_path()).map_err(|source| {
            EdtReportDataCompositionError::ReadReportDescriptor {
                path: report.descriptor_path().to_path_buf(),
                source,
            }
        })?;
        let root = parse_xml(&descriptor_xml).map_err(|message| {
            EdtReportDataCompositionError::MalformedReportXml {
                path: report.descriptor_path().to_path_buf(),
                message,
            }
        })?;
        let raw = parse_report_root(&root, report)?;
        let artifacts = discover_artifacts(report_directory, &raw.templates)?;
        let main_name =
            parse_main_selection(raw.main_selection.as_deref(), report.name(), &raw.templates)?;
        let mut schemas = Vec::with_capacity(raw.templates.len());

        for template in raw.templates {
            let artifact_path = artifacts
                .get(template.name.as_str())
                .expect("every accepted declaration must have one joined artifact")
                .clone();
            let xml = fs::read_to_string(&artifact_path).map_err(|source| {
                EdtReportDataCompositionError::ReadArtifact {
                    path: artifact_path.clone(),
                    source,
                }
            })?;
            let root = parse_xml(&xml).map_err(|message| {
                EdtReportDataCompositionError::MalformedArtifactXml {
                    path: artifact_path.clone(),
                    message,
                }
            })?;
            schemas.push(parse_schema(
                root,
                template,
                main_name.as_ref(),
                artifact_path,
            )?);
        }
        schemas.sort_by(|left, right| left.id.cmp(&right.id));

        Ok(EdtReportDataCompositionDescriptor {
            report_id: report.id().clone(),
            report_name: report.name().clone(),
            descriptor_path: report.descriptor_path().to_path_buf(),
            schemas,
        })
    }
}

#[derive(Debug)]
struct RawReport {
    templates: Vec<RawTemplate>,
    main_selection: Option<String>,
}

#[derive(Debug)]
struct RawTemplate {
    id: EntityId,
    name: EntityName,
}

fn parse_report_root(
    root: &XmlElement,
    report: &EdtMetadataObjectDescriptor,
) -> Result<RawReport, EdtReportDataCompositionError> {
    if root.name != "mdclass:Report" {
        return Err(EdtReportDataCompositionError::UnexpectedReportRoot(
            root.name.clone(),
        ));
    }
    if root.attribute("xmlns:mdclass") != Some(METADATA_NAMESPACE) {
        return Err(EdtReportDataCompositionError::UnsupportedReportNamespace(
            root.attribute("xmlns:mdclass").map(str::to_owned),
        ));
    }
    if root.attribute("uuid") != Some(report.id().as_str()) {
        return Err(EdtReportDataCompositionError::ReportIdentityMismatch);
    }
    let report_names = root.direct_texts("name");
    if report_names.as_slice() != [report.name().as_str()] {
        return Err(EdtReportDataCompositionError::ReportIdentityMismatch);
    }

    let main_values = root.direct_texts("mainDataCompositionSchema");
    if main_values.len() > 1 {
        return Err(EdtReportDataCompositionError::MultipleMainSelectors);
    }
    let mut templates = Vec::new();
    for element in root.children_named("templates") {
        let template_types = element.direct_texts("templateType");
        if !template_types
            .iter()
            .any(|template_type| template_type == "DataCompositionSchema")
        {
            continue;
        }
        if template_types.as_slice() != ["DataCompositionSchema"] {
            return Err(EdtReportDataCompositionError::InvalidTemplateType);
        }
        let raw_id = element
            .attribute("uuid")
            .ok_or(EdtReportDataCompositionError::InvalidTemplateUuid)?;
        let id = EntityId::new(raw_id)
            .map_err(|_| EdtReportDataCompositionError::InvalidTemplateUuid)?;
        let names = element.direct_texts("name");
        let [raw_name] = names.as_slice() else {
            return Err(EdtReportDataCompositionError::InvalidTemplateName);
        };
        let name = EntityName::new(raw_name)
            .map_err(|_| EdtReportDataCompositionError::InvalidTemplateName)?;
        if !is_safe_path_component(name.as_str()) {
            return Err(EdtReportDataCompositionError::InvalidTemplateName);
        }
        templates.push(RawTemplate { id, name });
    }
    templates.sort_by(|left, right| left.id.cmp(&right.id));

    let mut ids = BTreeSet::new();
    let mut names = BTreeSet::new();
    for template in &templates {
        if !ids.insert(template.id.clone()) {
            return Err(EdtReportDataCompositionError::DuplicateTemplateUuid(
                template.id.clone(),
            ));
        }
        if !names.insert(template.name.clone()) {
            return Err(EdtReportDataCompositionError::DuplicateTemplateName(
                template.name.clone(),
            ));
        }
    }

    Ok(RawReport {
        templates,
        main_selection: main_values.into_iter().next(),
    })
}

fn parse_main_selection(
    raw: Option<&str>,
    report_name: &EntityName,
    templates: &[RawTemplate],
) -> Result<Option<EntityName>, EdtReportDataCompositionError> {
    let Some(raw) = raw else {
        return Ok(None);
    };
    let value = raw.trim();
    let components = value.split('.').collect::<Vec<_>>();
    if components.len() != 4
        || components[0] != "Report"
        || components[1] != report_name.as_str()
        || components[2] != "Template"
        || components[3].is_empty()
    {
        return Err(EdtReportDataCompositionError::MalformedMainSelector(
            value.to_owned(),
        ));
    }
    let name = EntityName::new(components[3])
        .map_err(|_| EdtReportDataCompositionError::MalformedMainSelector(value.to_owned()))?;
    if !templates.iter().any(|template| template.name == name) {
        return Err(EdtReportDataCompositionError::UndeclaredMainSchema(name));
    }
    Ok(Some(name))
}

fn discover_artifacts(
    report_directory: &Path,
    templates: &[RawTemplate],
) -> Result<BTreeMap<String, PathBuf>, EdtReportDataCompositionError> {
    let templates_directory = report_directory.join("Templates");
    let mut artifacts = BTreeMap::new();

    for template in templates {
        let directory = templates_directory.join(template.name.as_str());
        let exact = directory.join("Template.dcs");
        let candidates = case_insensitive_artifact_candidates(&directory)?;
        if candidates.len() > 1 {
            return Err(EdtReportDataCompositionError::AmbiguousArtifact {
                directory,
                candidates,
            });
        }
        if !exact.exists() {
            return Err(EdtReportDataCompositionError::MissingArtifact(exact));
        }
        artifacts.insert(template.name.as_str().to_owned(), exact);
    }

    if templates_directory.is_dir() {
        let declared = templates
            .iter()
            .map(|template| template.name.as_str())
            .collect::<BTreeSet<_>>();
        for entry in sorted_directory_entries(&templates_directory)? {
            if !entry.is_dir() {
                continue;
            }
            let Some(name) = entry.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            let artifact = entry.join("Template.dcs");
            if artifact.exists() && !declared.contains(name) {
                return Err(EdtReportDataCompositionError::ExtraArtifact(artifact));
            }
        }
    }

    Ok(artifacts)
}

fn case_insensitive_artifact_candidates(
    directory: &Path,
) -> Result<Vec<PathBuf>, EdtReportDataCompositionError> {
    if !directory.is_dir() {
        return Ok(Vec::new());
    }
    let mut candidates = sorted_directory_entries(directory)?
        .into_iter()
        .filter(|path| {
            path.extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| extension.eq_ignore_ascii_case("dcs"))
        })
        .collect::<Vec<_>>();
    candidates.sort();
    Ok(candidates)
}

fn sorted_directory_entries(
    directory: &Path,
) -> Result<Vec<PathBuf>, EdtReportDataCompositionError> {
    let directory_entries =
        fs::read_dir(directory).map_err(|source| EdtReportDataCompositionError::ReadDirectory {
            path: directory.to_path_buf(),
            source,
        })?;
    let mut entries = Vec::new();
    for entry in directory_entries {
        let entry = entry.map_err(|source| EdtReportDataCompositionError::ReadDirectory {
            path: directory.to_path_buf(),
            source,
        })?;
        entries.push(entry.path());
    }
    entries.sort();
    Ok(entries)
}

fn parse_schema(
    root: XmlElement,
    template: RawTemplate,
    main_name: Option<&EntityName>,
    artifact_path: PathBuf,
) -> Result<EdtDataCompositionSchemaDescriptor, EdtReportDataCompositionError> {
    if root.name != "DataCompositionSchema" {
        return Err(EdtReportDataCompositionError::UnexpectedArtifactRoot {
            path: artifact_path,
            root: root.name,
        });
    }
    if root.attribute("xmlns") != Some(DATA_COMPOSITION_NAMESPACE) {
        return Err(
            EdtReportDataCompositionError::UnsupportedArtifactNamespace {
                path: artifact_path,
                namespace: root.attribute("xmlns").map(str::to_owned),
            },
        );
    }
    if root.attribute("xmlns:xsi") != Some(XSI_NAMESPACE) {
        return Err(EdtReportDataCompositionError::UnsupportedXsiNamespace {
            path: artifact_path,
            namespace: root.attribute("xmlns:xsi").map(str::to_owned),
        });
    }
    for element in &root.children {
        let local = xml_local_name(&element.name);
        if matches!(local, "dataSource" | "dataSet") {
            validate_schema_element(element, local, &artifact_path)?;
        }
    }

    let data_source = parse_root_data_source(&root)?;
    let mut data_sets = Vec::new();
    let mut observations = Vec::new();
    let mut occurrence_ordinal = 0;

    for element in root.children_named("dataSet") {
        let raw_type = element.attribute("xsi:type").map(str::to_owned);
        let Some(kind) = raw_type.as_deref().and_then(data_set_kind) else {
            observations.push(EdtDataCompositionObservation {
                kind: EdtDataCompositionObservationKind::UnsupportedDataSetType,
                artifact_path: artifact_path.clone(),
                owner_id: template.id.clone(),
                raw_type,
                occurrence_ordinal,
            });
            occurrence_ordinal += 1;
            continue;
        };
        data_sets.push(parse_data_set(
            element,
            kind,
            &template.id,
            &artifact_path,
            &mut observations,
            &mut occurrence_ordinal,
        )?);
    }
    for element in root
        .children
        .iter()
        .filter(|element| element.name != "dataSet")
    {
        collect_nested_data_sets(
            element,
            &template.id,
            &artifact_path,
            &mut observations,
            &mut occurrence_ordinal,
            false,
        );
    }
    data_sets.sort_by(|left, right| left.id.cmp(&right.id));
    let mut names = BTreeSet::new();
    for data_set in &data_sets {
        if !names.insert(data_set.name.clone()) {
            return Err(EdtReportDataCompositionError::DuplicateDataSetName {
                schema_id: template.id.clone(),
                name: data_set.name.clone(),
            });
        }
    }
    if data_sets
        .iter()
        .any(|data_set| matches!(data_set.kind, DataSetKind::Query | DataSetKind::Object))
        && data_source.is_none()
    {
        return Err(EdtReportDataCompositionError::InvalidRootDataSource(
            EdtDataCompositionSourceReason::Missing,
        ));
    }
    observations.sort();

    Ok(EdtDataCompositionSchemaDescriptor {
        id: template.id,
        main: main_name.is_some_and(|name| *name == template.name),
        name: template.name,
        artifact_path,
        data_source,
        data_sets,
        observations,
    })
}

fn parse_root_data_source(
    root: &XmlElement,
) -> Result<Option<EdtDataCompositionDataSource>, EdtReportDataCompositionError> {
    let sources = root.children_named("dataSource");
    if sources.len() > 1 {
        return Err(EdtReportDataCompositionError::InvalidRootDataSource(
            EdtDataCompositionSourceReason::Multiple,
        ));
    }
    let Some(source) = sources.first() else {
        return Ok(None);
    };
    let names = source.direct_texts("name");
    let source_types = source.direct_texts("dataSourceType");
    if names.as_slice() != ["DataSource1"] {
        return Err(EdtReportDataCompositionError::InvalidRootDataSource(
            EdtDataCompositionSourceReason::InvalidName,
        ));
    }
    if source_types.as_slice() != ["Local"] {
        return Err(EdtReportDataCompositionError::InvalidRootDataSource(
            EdtDataCompositionSourceReason::InvalidType,
        ));
    }
    let name = EntityName::new(names[0].as_str()).map_err(|_| {
        EdtReportDataCompositionError::InvalidRootDataSource(
            EdtDataCompositionSourceReason::InvalidName,
        )
    })?;
    Ok(Some(EdtDataCompositionDataSource { name }))
}

fn parse_data_set(
    element: &XmlElement,
    kind: DataSetKind,
    schema_id: &EntityId,
    artifact_path: &Path,
    observations: &mut Vec<EdtDataCompositionObservation>,
    occurrence_ordinal: &mut usize,
) -> Result<EdtDataCompositionDataSet, EdtReportDataCompositionError> {
    let names = element.direct_texts("name");
    let [raw_name] = names.as_slice() else {
        return Err(EdtReportDataCompositionError::InvalidDataSetName(
            schema_id.clone(),
        ));
    };
    let name = EntityName::new(raw_name)
        .map_err(|_| EdtReportDataCompositionError::InvalidDataSetName(schema_id.clone()))?;
    let id = data_set_id(schema_id, &name).map_err(identity_error)?;
    let sources = element.direct_texts("dataSource");
    let data_source = match kind {
        DataSetKind::Query | DataSetKind::Object if sources.as_slice() == ["DataSource1"] => {
            Some(EntityName::new("DataSource1").expect("accepted source name must be valid"))
        }
        DataSetKind::Query | DataSetKind::Object => {
            return Err(EdtReportDataCompositionError::InvalidDataSetSource {
                data_set_id: id,
                reason: if sources.is_empty() {
                    EdtDataCompositionSourceReason::Missing
                } else if sources.len() > 1 {
                    EdtDataCompositionSourceReason::Multiple
                } else {
                    EdtDataCompositionSourceReason::InvalidName
                },
            });
        }
        DataSetKind::Union if sources.is_empty() => None,
        DataSetKind::Union => {
            return Err(EdtReportDataCompositionError::InvalidDataSetSource {
                data_set_id: id,
                reason: EdtDataCompositionSourceReason::Unexpected,
            });
        }
    };
    let queries = element.direct_texts_preserving_whitespace("query");
    let query = match kind {
        DataSetKind::Query if queries.len() == 1 && !queries[0].trim().is_empty() => {
            Some(queries[0].clone())
        }
        DataSetKind::Query => {
            return Err(EdtReportDataCompositionError::InvalidQueryCardinality(id));
        }
        DataSetKind::Object | DataSetKind::Union if queries.is_empty() => None,
        DataSetKind::Object | DataSetKind::Union => {
            return Err(EdtReportDataCompositionError::UnexpectedQuery(id));
        }
    };

    let mut fields = Vec::new();
    let direct_fields = element
        .children
        .iter()
        .filter(|field| xml_local_name(&field.name) == "field")
        .collect::<Vec<_>>();
    for field in direct_fields {
        validate_schema_element(field, "field", artifact_path)?;
        match field.attribute("xsi:type") {
            Some("DataSetFieldField") => fields.push(parse_field(field, &id)?),
            Some("DataSetFieldFolder") => observations.push(EdtDataCompositionObservation {
                kind: EdtDataCompositionObservationKind::FieldFolder,
                artifact_path: artifact_path.to_path_buf(),
                owner_id: id.clone(),
                raw_type: Some("DataSetFieldFolder".to_owned()),
                occurrence_ordinal: *occurrence_ordinal,
            }),
            raw_type => observations.push(EdtDataCompositionObservation {
                kind: EdtDataCompositionObservationKind::UnsupportedFieldType,
                artifact_path: artifact_path.to_path_buf(),
                owner_id: id.clone(),
                raw_type: raw_type.map(str::to_owned),
                occurrence_ordinal: *occurrence_ordinal,
            }),
        }
        *occurrence_ordinal += 1;
    }
    fields.sort_by(|left, right| left.id.cmp(&right.id));
    let mut field_names = BTreeSet::new();
    for field in &fields {
        if !field_names.insert(field.name.clone()) {
            return Err(EdtReportDataCompositionError::DuplicateFieldName {
                data_set_id: id,
                name: field.name.clone(),
            });
        }
    }

    collect_nested_data_sets(
        element,
        &id,
        artifact_path,
        observations,
        occurrence_ordinal,
        true,
    );

    Ok(EdtDataCompositionDataSet {
        id,
        name,
        kind,
        data_source,
        fields,
        query,
    })
}

fn parse_field(
    element: &XmlElement,
    data_set_id: &EntityId,
) -> Result<EdtDataCompositionField, EdtReportDataCompositionError> {
    let names = element.direct_texts("field");
    let [raw_name] = names.as_slice() else {
        return Err(EdtReportDataCompositionError::InvalidFieldName(
            data_set_id.clone(),
        ));
    };
    let name = EntityName::new(raw_name)
        .map_err(|_| EdtReportDataCompositionError::InvalidFieldName(data_set_id.clone()))?;
    let paths = element.direct_texts("dataPath");
    let [raw_path] = paths.as_slice() else {
        return Err(EdtReportDataCompositionError::InvalidFieldPath {
            data_set_id: data_set_id.clone(),
            field_name: name,
        });
    };
    let data_path =
        EntityName::new(raw_path).map_err(|_| EdtReportDataCompositionError::InvalidFieldPath {
            data_set_id: data_set_id.clone(),
            field_name: name.clone(),
        })?;
    let id = data_composition_field_id(data_set_id, &name).map_err(identity_error)?;
    Ok(EdtDataCompositionField {
        id,
        name,
        data_path,
    })
}

fn collect_nested_data_sets(
    element: &XmlElement,
    owner_id: &EntityId,
    artifact_path: &Path,
    observations: &mut Vec<EdtDataCompositionObservation>,
    occurrence_ordinal: &mut usize,
    skip_current: bool,
) {
    if !skip_current && element.name == "dataSet" {
        observations.push(EdtDataCompositionObservation {
            kind: EdtDataCompositionObservationKind::NestedDataSet,
            artifact_path: artifact_path.to_path_buf(),
            owner_id: owner_id.clone(),
            raw_type: element.attribute("xsi:type").map(str::to_owned),
            occurrence_ordinal: *occurrence_ordinal,
        });
        *occurrence_ordinal += 1;
    }
    for child in &element.children {
        collect_nested_data_sets(
            child,
            owner_id,
            artifact_path,
            observations,
            occurrence_ordinal,
            false,
        );
    }
}

const fn data_set_kind(raw: &str) -> Option<DataSetKind> {
    match raw.as_bytes() {
        b"DataSetQuery" => Some(DataSetKind::Query),
        b"DataSetObject" => Some(DataSetKind::Object),
        b"DataSetUnion" => Some(DataSetKind::Union),
        _ => None,
    }
}

fn identity_error(_: DataCompositionIdentityError) -> EdtReportDataCompositionError {
    EdtReportDataCompositionError::InvalidDerivedIdentity
}

fn validate_schema_element(
    element: &XmlElement,
    expected_local_name: &str,
    artifact_path: &Path,
) -> Result<(), EdtReportDataCompositionError> {
    let namespace = if element.name == expected_local_name {
        element.attribute("xmlns")
    } else {
        element
            .name
            .split_once(':')
            .and_then(|(prefix, _)| element.attribute(&format!("xmlns:{prefix}")))
    };
    if element.name != expected_local_name
        || namespace.is_some_and(|namespace| namespace != DATA_COMPOSITION_NAMESPACE)
    {
        return Err(
            EdtReportDataCompositionError::UnsupportedArtifactNamespace {
                path: artifact_path.to_path_buf(),
                namespace: namespace.map(str::to_owned),
            },
        );
    }
    Ok(())
}

fn xml_local_name(name: &str) -> &str {
    name.rsplit(':').next().unwrap_or(name)
}

fn is_safe_path_component(value: &str) -> bool {
    let mut components = Path::new(value).components();
    matches!(components.next(), Some(Component::Normal(_))) && components.next().is_none()
}

#[derive(Debug, Clone)]
struct XmlElement {
    name: String,
    attributes: BTreeMap<String, String>,
    text: String,
    children: Vec<Self>,
}

impl XmlElement {
    fn attribute(&self, name: &str) -> Option<&str> {
        self.attributes.get(name).map(String::as_str)
    }

    fn children_named(&self, name: &str) -> Vec<&Self> {
        self.children
            .iter()
            .filter(|child| child.name == name)
            .collect()
    }

    fn direct_texts(&self, name: &str) -> Vec<String> {
        self.children_named(name)
            .into_iter()
            .map(|child| child.text.trim().to_owned())
            .collect()
    }

    fn direct_texts_preserving_whitespace(&self, name: &str) -> Vec<String> {
        self.children_named(name)
            .into_iter()
            .map(|child| child.text.clone())
            .collect()
    }
}

fn parse_xml(xml: &str) -> Result<XmlElement, String> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);
    let mut stack = Vec::<XmlElement>::new();
    let mut root = None;

    loop {
        match reader.read_event() {
            Ok(Event::Start(event)) => {
                stack.push(xml_element(&reader, &event)?);
            }
            Ok(Event::Empty(event)) => {
                let element = xml_element(&reader, &event)?;
                attach_element(element, &mut stack, &mut root)?;
            }
            Ok(Event::Text(event)) => append_xml_text(&event, &mut stack)?,
            Ok(Event::CData(event)) => append_xml_cdata(&event, &mut stack)?,
            Ok(Event::GeneralRef(event)) => append_xml_reference(&event, &mut stack)?,
            Ok(Event::End(_)) => {
                let element = stack
                    .pop()
                    .ok_or_else(|| "unexpected closing element".to_owned())?;
                attach_element(element, &mut stack, &mut root)?;
            }
            Ok(Event::Eof) => break,
            Ok(Event::Decl(_) | Event::PI(_) | Event::Comment(_) | Event::DocType(_)) => {}
            Err(source) => return Err(source.to_string()),
        }
    }
    if !stack.is_empty() {
        return Err("unexpected end of file before the root was closed".to_owned());
    }
    root.ok_or_else(|| "XML root element is missing".to_owned())
}

fn xml_element(reader: &Reader<&[u8]>, event: &BytesStart<'_>) -> Result<XmlElement, String> {
    let mut attributes = BTreeMap::new();
    for attribute in event.attributes().with_checks(false) {
        let attribute = attribute.map_err(|source| source.to_string())?;
        let name = String::from_utf8_lossy(attribute.key.as_ref()).into_owned();
        let value = attribute
            .decode_and_unescape_value(reader.decoder())
            .map_err(|source| source.to_string())?
            .into_owned();
        if attributes.insert(name.clone(), value).is_some() {
            return Err(format!("duplicate XML attribute `{name}`"));
        }
    }
    Ok(XmlElement {
        name: String::from_utf8_lossy(event.name().as_ref()).into_owned(),
        attributes,
        text: String::new(),
        children: Vec::new(),
    })
}

fn attach_element(
    element: XmlElement,
    stack: &mut [XmlElement],
    root: &mut Option<XmlElement>,
) -> Result<(), String> {
    if let Some(parent) = stack.last_mut() {
        parent.children.push(element);
    } else if root.replace(element).is_some() {
        return Err("multiple XML root elements".to_owned());
    }
    Ok(())
}

fn append_xml_text(event: &BytesText<'_>, stack: &mut [XmlElement]) -> Result<(), String> {
    let decoded = event.decode().map_err(|source| source.to_string())?;
    let decoded = unescape(&decoded).map_err(|source| source.to_string())?;
    if let Some(element) = stack.last_mut() {
        element.text.push_str(&decoded);
    }
    Ok(())
}

fn append_xml_cdata(event: &BytesCData<'_>, stack: &mut [XmlElement]) -> Result<(), String> {
    let decoded = event.decode().map_err(|source| source.to_string())?;
    if let Some(element) = stack.last_mut() {
        element.text.push_str(&decoded);
    }
    Ok(())
}

fn append_xml_reference(event: &BytesRef<'_>, stack: &mut [XmlElement]) -> Result<(), String> {
    let reference = event.decode().map_err(|source| source.to_string())?;
    let encoded = format!("&{reference};");
    let decoded = unescape(&encoded).map_err(|source| source.to_string())?;
    if let Some(element) = stack.last_mut() {
        element.text.push_str(&decoded);
    }
    Ok(())
}

/// Typed reason for an invalid Data Composition data-source declaration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdtDataCompositionSourceReason {
    /// The required source is absent.
    Missing,
    /// More than one direct source declaration exists.
    Multiple,
    /// The exact `DataSource1` name contract is violated.
    InvalidName,
    /// The exact root `Local` source type contract is violated.
    InvalidType,
    /// A Union unexpectedly declares a direct source reference.
    Unexpected,
}

/// Fatal Report Data Composition source error.
#[derive(Debug)]
pub enum EdtReportDataCompositionError {
    /// The supplied descriptor is not a Report.
    UnexpectedMetadataKind(MetadataKind),
    /// The supplied Report directory is absent.
    ReportDirectoryNotFound(PathBuf),
    /// A source directory could not be enumerated.
    ReadDirectory {
        /// Directory path.
        path: PathBuf,
        /// Underlying I/O error.
        source: std::io::Error,
    },
    /// The accepted Report descriptor could not be read again for the join.
    ReadReportDescriptor {
        /// Descriptor path.
        path: PathBuf,
        /// Underlying I/O error.
        source: std::io::Error,
    },
    /// The Report descriptor XML is malformed.
    MalformedReportXml {
        /// Descriptor path.
        path: PathBuf,
        /// Parser message.
        message: String,
    },
    /// The descriptor root is not `mdclass:Report`.
    UnexpectedReportRoot(String),
    /// The metadata namespace is absent or unsupported.
    UnsupportedReportNamespace(Option<String>),
    /// Re-read UUID or direct name differs from the accepted Report descriptor.
    ReportIdentityMismatch,
    /// A DCS declaration has no valid unique UUID.
    InvalidTemplateUuid,
    /// A DCS declaration has no valid unique path-safe name.
    InvalidTemplateName,
    /// A DCS declaration repeats or mixes its required exact template type.
    InvalidTemplateType,
    /// Two DCS declarations share one UUID.
    DuplicateTemplateUuid(EntityId),
    /// Two DCS declarations share one name.
    DuplicateTemplateName(EntityName),
    /// More than one direct main selector exists.
    MultipleMainSelectors,
    /// The main selector does not have the exact four-component grammar.
    MalformedMainSelector(String),
    /// The main selector names no accepted DCS declaration.
    UndeclaredMainSchema(EntityName),
    /// One exact declared artifact is absent.
    MissingArtifact(PathBuf),
    /// More than one case-equivalent artifact candidate exists.
    AmbiguousArtifact {
        /// Template directory.
        directory: PathBuf,
        /// Ordered candidates.
        candidates: Vec<PathBuf>,
    },
    /// An undeclared direct template owns an extra exact DCS artifact.
    ExtraArtifact(PathBuf),
    /// A joined DCS artifact could not be read.
    ReadArtifact {
        /// Artifact path.
        path: PathBuf,
        /// Underlying I/O error.
        source: std::io::Error,
    },
    /// A joined DCS artifact XML is malformed.
    MalformedArtifactXml {
        /// Artifact path.
        path: PathBuf,
        /// Parser message.
        message: String,
    },
    /// The DCS root local name is unexpected.
    UnexpectedArtifactRoot {
        /// Artifact path.
        path: PathBuf,
        /// Actual root name.
        root: String,
    },
    /// The DCS default namespace is absent or unsupported.
    UnsupportedArtifactNamespace {
        /// Artifact path.
        path: PathBuf,
        /// Actual namespace.
        namespace: Option<String>,
    },
    /// The `xsi` namespace required for exact type declarations is unsupported.
    UnsupportedXsiNamespace {
        /// Artifact path.
        path: PathBuf,
        /// Actual namespace.
        namespace: Option<String>,
    },
    /// The root local data source violates the accepted contract.
    InvalidRootDataSource(EdtDataCompositionSourceReason),
    /// A direct accepted Data Set has no valid unique name.
    InvalidDataSetName(EntityId),
    /// Two direct accepted Data Sets share one name.
    DuplicateDataSetName {
        /// Schema identity.
        schema_id: EntityId,
        /// Duplicate name.
        name: EntityName,
    },
    /// A Query/Object/Union direct data-source contract is invalid.
    InvalidDataSetSource {
        /// Data Set identity.
        data_set_id: EntityId,
        /// Typed reason.
        reason: EdtDataCompositionSourceReason,
    },
    /// A Query Data Set does not have exactly one complete non-empty query.
    InvalidQueryCardinality(EntityId),
    /// An Object or Union Data Set unexpectedly declares a query.
    UnexpectedQuery(EntityId),
    /// A direct accepted Field has no valid unique name.
    InvalidFieldName(EntityId),
    /// A direct accepted Field has no valid non-empty data path.
    InvalidFieldPath {
        /// Data Set identity.
        data_set_id: EntityId,
        /// Field name.
        field_name: EntityName,
    },
    /// Two direct accepted Fields share one name.
    DuplicateFieldName {
        /// Data Set identity.
        data_set_id: EntityId,
        /// Duplicate field name.
        name: EntityName,
    },
    /// An accepted owner-scoped identity could not be represented.
    InvalidDerivedIdentity,
}

impl Display for EdtReportDataCompositionError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "Report Data Composition source error: {self:?}")
    }
}

impl std::error::Error for EdtReportDataCompositionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ReadDirectory { source, .. }
            | Self::ReadReportDescriptor { source, .. }
            | Self::ReadArtifact { source, .. } => Some(source),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use oneagent_graph::DataSetKind;
    use oneagent_metadata::MetadataKind;
    use std::fs;
    use std::path::{Path, PathBuf};
    use tempfile::{TempDir, tempdir};

    use crate::{EdtMetadataObjectReader, FileSystemEdtMetadataObjectReader};

    use super::{
        EdtDataCompositionObservationKind, EdtDataCompositionSourceReason,
        EdtReportDataCompositionError, EdtReportDataCompositionReader,
        FileSystemEdtReportDataCompositionReader,
    };

    const DCS_ROOT_OPEN: &str = r#"<DataCompositionSchema xmlns="http://v8.1c.ru/8.1/data-composition-system/schema" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">"#;

    struct GeneratedReport {
        _root: TempDir,
        directory: PathBuf,
        descriptor_path: PathBuf,
    }

    impl GeneratedReport {
        fn new(descriptor: &str, artifacts: &[(&str, &str)]) -> Self {
            let root = tempdir().expect("temporary root must be created");
            let directory = root.path().join("GeneratedReport");
            fs::create_dir_all(&directory).expect("Report directory must be created");
            let descriptor_path = directory.join("GeneratedReport.mdo");
            fs::write(&descriptor_path, descriptor).expect("Report descriptor must be written");
            for (name, xml) in artifacts {
                let template_directory = directory.join("Templates").join(name);
                fs::create_dir_all(&template_directory)
                    .expect("template directory must be created");
                fs::write(template_directory.join("Template.dcs"), xml)
                    .expect("DCS artifact must be written");
            }
            Self {
                _root: root,
                directory,
                descriptor_path,
            }
        }

        fn read(
            &self,
        ) -> Result<super::EdtReportDataCompositionDescriptor, EdtReportDataCompositionError>
        {
            let report = self.metadata();
            FileSystemEdtReportDataCompositionReader.read(&self.directory, &report)
        }

        fn metadata(&self) -> crate::EdtMetadataObjectDescriptor {
            FileSystemEdtMetadataObjectReader
                .read(&self.directory, MetadataKind::Report)
                .expect("generated Report metadata descriptor must parse")
        }

        fn rewrite_descriptor(&self, xml: &str) {
            fs::write(&self.descriptor_path, xml).expect("descriptor must be rewritten");
        }

        fn rewrite_artifact(&self, name: &str, xml: &str) {
            fs::write(
                self.directory
                    .join("Templates")
                    .join(name)
                    .join("Template.dcs"),
                xml,
            )
            .expect("artifact must be rewritten");
        }
    }

    fn descriptor(templates: &str, main: &str) -> String {
        format!(
            r#"<mdclass:Report xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass" uuid="report-id">
  <name>GeneratedReport</name>
  {main}
  {templates}
</mdclass:Report>"#
        )
    }

    fn template(id: &str, name: &str) -> String {
        format!(
            r#"<templates uuid="{id}"><name>{name}</name><templateType>DataCompositionSchema</templateType></templates>"#
        )
    }

    fn data_source() -> &'static str {
        "<dataSource><name>DataSource1</name><dataSourceType>Local</dataSourceType></dataSource>"
    }

    fn schema(body: &str) -> String {
        format!("{DCS_ROOT_OPEN}{body}</DataCompositionSchema>")
    }

    fn query_data_set(name: &str, fields: &str, query: &str) -> String {
        format!(
            r#"<dataSet xsi:type="DataSetQuery"><name>{name}</name>{fields}<dataSource>DataSource1</dataSource><query>{query}</query></dataSet>"#
        )
    }

    fn field(name: &str, path: &str) -> String {
        format!(
            r#"<field xsi:type="DataSetFieldField"><dataPath>{path}</dataPath><field>{name}</field></field>"#
        )
    }

    fn assert_error(
        descriptor_xml: &str,
        artifacts: &[(&str, &str)],
        predicate: impl FnOnce(&EdtReportDataCompositionError) -> bool,
    ) {
        let generated = GeneratedReport::new(descriptor_xml, artifacts);
        let error = generated.read().expect_err("generated input must fail");
        assert!(predicate(&error), "unexpected error: {error:?}");
    }

    #[test]
    fn parses_query_object_union_empty_and_non_main_schemas() {
        let templates = [
            template("schema-query", "QuerySchema"),
            template("schema-object", "ObjectSchema"),
            template("schema-union", "UnionSchema"),
            template("schema-empty", "EmptySchema"),
        ]
        .join("\n");
        let descriptor = descriptor(
            &templates,
            "<mainDataCompositionSchema>Report.GeneratedReport.Template.QuerySchema</mainDataCompositionSchema>",
        );
        let query_schema = schema(&format!(
            "{}{}",
            data_source(),
            query_data_set(
                "QuerySet",
                &format!(
                    "{}{}",
                    field("Second", "Path.Second"),
                    field("First", "Path.First")
                ),
                "SELECT &amp;Parameter AS Value",
            )
        ));
        let object_schema = schema(&format!(
            r#"{}<dataSet xsi:type="DataSetObject"><name>ObjectSet</name>{}<dataSource>DataSource1</dataSource><objectName>RuntimeTable</objectName></dataSet>"#,
            data_source(),
            field("ObjectField", "Object.Path")
        ));
        let union_schema = schema(&format!(
            r#"{}<dataSet xsi:type="DataSetUnion"><name>UnionSet</name>{}</dataSet>"#,
            data_source(),
            field("UnionField", "Union.Path")
        ));
        let generated = GeneratedReport::new(
            &descriptor,
            &[
                ("QuerySchema", &query_schema),
                ("ObjectSchema", &object_schema),
                ("UnionSchema", &union_schema),
                ("EmptySchema", &schema("")),
            ],
        );

        let parsed = generated.read().expect("accepted schemas must parse");

        assert_eq!(parsed.report_id().as_str(), "report-id");
        assert_eq!(parsed.report_name().as_str(), "GeneratedReport");
        assert_eq!(parsed.schemas().len(), 4);
        let query = parsed
            .schemas()
            .iter()
            .find(|schema| schema.name().as_str() == "QuerySchema")
            .expect("Query Schema must exist");
        assert!(query.is_main());
        assert_eq!(
            query
                .data_source()
                .expect("source must exist")
                .name()
                .as_str(),
            "DataSource1"
        );
        assert_eq!(query.data_sets()[0].kind(), DataSetKind::Query);
        assert_eq!(
            query.data_sets()[0].query(),
            Some("SELECT &Parameter AS Value")
        );
        assert_eq!(
            query.data_sets()[0]
                .fields()
                .iter()
                .map(|field| (field.name().as_str(), field.data_path().as_str()))
                .collect::<Vec<_>>(),
            vec![("First", "Path.First"), ("Second", "Path.Second")]
        );
        assert_eq!(
            parsed
                .schemas()
                .iter()
                .filter(|schema| schema.is_main())
                .count(),
            1
        );
        assert!(
            parsed
                .schemas()
                .iter()
                .find(|schema| schema.name().as_str() == "EmptySchema")
                .expect("empty Schema must exist")
                .data_sets()
                .is_empty()
        );
        assert!(parsed.observations().next().is_none());
    }

    #[test]
    fn accepted_output_is_equal_after_descriptor_data_set_and_field_reordering() {
        let first_templates = format!(
            "{}{}",
            template("schema-b", "SecondSchema"),
            template("schema-a", "FirstSchema")
        );
        let second_templates = format!(
            "{}{}",
            template("schema-a", "FirstSchema"),
            template("schema-b", "SecondSchema")
        );
        let first_data_sets = format!(
            "{}{}",
            query_data_set("Second", &field("B", "Path.B"), "SELECT 2"),
            query_data_set(
                "First",
                &format!("{}{}", field("Z", "Path.Z"), field("A", "Path.A")),
                "SELECT 1",
            )
        );
        let second_data_sets = format!(
            "{}{}",
            query_data_set(
                "First",
                &format!("{}{}", field("A", "Path.A"), field("Z", "Path.Z")),
                "SELECT 1",
            ),
            query_data_set("Second", &field("B", "Path.B"), "SELECT 2")
        );
        let first_descriptor = descriptor(&first_templates, "");
        let second_descriptor = descriptor(&second_templates, "");
        let generated = GeneratedReport::new(
            &first_descriptor,
            &[
                (
                    "FirstSchema",
                    &schema(&format!("{}{first_data_sets}", data_source())),
                ),
                ("SecondSchema", &schema("")),
            ],
        );
        let first = generated.read().expect("first order must parse");

        generated.rewrite_descriptor(&second_descriptor);
        generated.rewrite_artifact(
            "FirstSchema",
            &schema(&format!("{}{second_data_sets}", data_source())),
        );
        let second = generated.read().expect("reordered source must parse");

        assert_eq!(first, second);
        assert_eq!(first.schemas()[0].id().as_str(), "schema-a");
        assert_eq!(first.schemas()[0].data_sets()[0].name().as_str(), "First");
    }

    #[test]
    fn nested_folders_and_unknown_types_are_distinct_deferred_observations() {
        let descriptor = descriptor(&template("schema", "Main"), "");
        let artifact = schema(&format!(
            r#"{}<dataSet xsi:type="DataSetUnion"><name>Union</name>
  <field xsi:type="DataSetFieldFolder"><dataPath>Folder</dataPath></field>
  <field xsi:type="FutureField"><field>Future</field></field>
  <dataSet xsi:type="DataSetQuery"><name>Repeated</name><dataSource>DataSource1</dataSource><query>SELECT 1</query></dataSet>
  <dataSet xsi:type="DataSetQuery"><name>Repeated</name><dataSource>DataSource1</dataSource><query>SELECT 2</query></dataSet>
</dataSet><dataSet xsi:type="FutureDataSet"><name>Future</name></dataSet>"#,
            data_source()
        ));
        let generated = GeneratedReport::new(&descriptor, &[("Main", &artifact)]);

        let parsed = generated
            .read()
            .expect("deferred shapes must be recoverable");
        let schema = &parsed.schemas()[0];
        assert_eq!(schema.data_sets().len(), 1);
        assert_eq!(schema.observations().len(), 5);
        assert_eq!(
            schema
                .observations()
                .iter()
                .map(super::EdtDataCompositionObservation::kind)
                .collect::<Vec<_>>(),
            vec![
                EdtDataCompositionObservationKind::NestedDataSet,
                EdtDataCompositionObservationKind::NestedDataSet,
                EdtDataCompositionObservationKind::FieldFolder,
                EdtDataCompositionObservationKind::UnsupportedDataSetType,
                EdtDataCompositionObservationKind::UnsupportedFieldType,
            ]
        );
        assert_eq!(
            schema
                .observations()
                .iter()
                .filter(|observation| observation.kind()
                    == EdtDataCompositionObservationKind::NestedDataSet)
                .map(super::EdtDataCompositionObservation::occurrence_ordinal)
                .collect::<Vec<_>>(),
            vec![2, 3]
        );
    }

    #[test]
    fn descriptor_declaration_and_main_selector_failures_are_typed() {
        let artifact = schema("");
        assert_error(
            &descriptor(
                &format!(
                    "{}{}",
                    template("same", "First"),
                    template("same", "Second")
                ),
                "",
            ),
            &[("First", &artifact), ("Second", &artifact)],
            |error| {
                matches!(
                    error,
                    EdtReportDataCompositionError::DuplicateTemplateUuid(_)
                )
            },
        );
        assert_error(
            &descriptor(
                &format!(
                    "{}{}",
                    template("first", "Same"),
                    template("second", "Same")
                ),
                "",
            ),
            &[("Same", &artifact)],
            |error| {
                matches!(
                    error,
                    EdtReportDataCompositionError::DuplicateTemplateName(_)
                )
            },
        );
        assert_error(
            &descriptor(
                &template("schema", "Main"),
                "<mainDataCompositionSchema>Report.Wrong.Template.Main</mainDataCompositionSchema>",
            ),
            &[("Main", &artifact)],
            |error| {
                matches!(
                    error,
                    EdtReportDataCompositionError::MalformedMainSelector(_)
                )
            },
        );
        assert_error(
            &descriptor(
                &template("schema", "Main"),
                "<mainDataCompositionSchema>Report.GeneratedReport.Template.Missing</mainDataCompositionSchema>",
            ),
            &[("Main", &artifact)],
            |error| {
                matches!(
                    error,
                    EdtReportDataCompositionError::UndeclaredMainSchema(_)
                )
            },
        );
        assert_error(
            &descriptor(
                &template("schema", "Main"),
                "<mainDataCompositionSchema>Report.GeneratedReport.Template.Main</mainDataCompositionSchema><mainDataCompositionSchema>Report.GeneratedReport.Template.Main</mainDataCompositionSchema>",
            ),
            &[("Main", &artifact)],
            |error| matches!(error, EdtReportDataCompositionError::MultipleMainSelectors),
        );
        assert_error(
            &descriptor(
                r#"<templates uuid="schema"><name>Main</name><templateType>DataCompositionSchema</templateType><templateType>DataCompositionSchema</templateType></templates>"#,
                "",
            ),
            &[("Main", &artifact)],
            |error| matches!(error, EdtReportDataCompositionError::InvalidTemplateType),
        );
    }

    #[test]
    fn joined_report_descriptor_read_xml_identity_and_required_value_failures_are_typed() {
        let valid_descriptor = descriptor(&template("schema", "Main"), "");
        let generated = GeneratedReport::new(&valid_descriptor, &[("Main", &schema(""))]);
        let metadata = generated.metadata();

        generated.rewrite_descriptor("<mdclass:Report>");
        assert!(matches!(
            FileSystemEdtReportDataCompositionReader
                .read(&generated.directory, &metadata)
                .expect_err("malformed joined Report descriptor must fail"),
            EdtReportDataCompositionError::MalformedReportXml { .. }
        ));

        generated.rewrite_descriptor(&valid_descriptor.replace("report-id", "changed-id"));
        assert!(matches!(
            FileSystemEdtReportDataCompositionReader
                .read(&generated.directory, &metadata)
                .expect_err("changed joined Report identity must fail"),
            EdtReportDataCompositionError::ReportIdentityMismatch
        ));

        generated.rewrite_descriptor(&descriptor(
            r"<templates><name>Main</name><templateType>DataCompositionSchema</templateType></templates>",
            "",
        ));
        assert!(matches!(
            FileSystemEdtReportDataCompositionReader
                .read(&generated.directory, &metadata)
                .expect_err("missing DCS UUID must fail"),
            EdtReportDataCompositionError::InvalidTemplateUuid
        ));

        fs::remove_file(&generated.descriptor_path)
            .expect("temporary joined descriptor must be removed");
        assert!(matches!(
            FileSystemEdtReportDataCompositionReader
                .read(&generated.directory, &metadata)
                .expect_err("unreadable joined descriptor must fail"),
            EdtReportDataCompositionError::ReadReportDescriptor { .. }
        ));
    }

    #[test]
    fn artifact_discovery_root_namespace_and_xml_failures_are_typed() {
        let descriptor = descriptor(&template("schema", "Main"), "");
        assert_error(&descriptor, &[], |error| {
            matches!(error, EdtReportDataCompositionError::MissingArtifact(_))
        });
        assert_error(&descriptor, &[("Main", "<Wrong/>")], |error| {
            matches!(
                error,
                EdtReportDataCompositionError::UnexpectedArtifactRoot { .. }
            )
        });
        assert_error(
            &descriptor,
            &[(
                "Main",
                r#"<DataCompositionSchema xmlns="urn:wrong" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"/>"#,
            )],
            |error| {
                matches!(
                    error,
                    EdtReportDataCompositionError::UnsupportedArtifactNamespace { .. }
                )
            },
        );
        assert_error(
            &descriptor,
            &[(
                "Main",
                &schema(
                    r#"<dataSet xmlns="urn:wrong" xsi:type="DataSetUnion"><name>WrongNamespace</name></dataSet>"#,
                ),
            )],
            |error| {
                matches!(
                    error,
                    EdtReportDataCompositionError::UnsupportedArtifactNamespace { .. }
                )
            },
        );
        assert_error(
            &descriptor,
            &[("Main", "<DataCompositionSchema>")],
            |error| {
                matches!(
                    error,
                    EdtReportDataCompositionError::MalformedArtifactXml { .. }
                )
            },
        );

        let generated = GeneratedReport::new(&descriptor, &[("Main", &schema(""))]);
        let duplicate = generated.directory.join("Templates/Main/Second.dcs");
        fs::write(&duplicate, schema("")).expect("ambiguous artifact must be written");
        assert!(matches!(
            generated.read().expect_err("ambiguous artifacts must fail"),
            EdtReportDataCompositionError::AmbiguousArtifact { .. }
        ));

        let generated = GeneratedReport::new(&descriptor, &[("Main", &schema(""))]);
        let extra_directory = generated.directory.join("Templates/Extra");
        fs::create_dir_all(&extra_directory).expect("extra template directory must exist");
        fs::write(extra_directory.join("Template.dcs"), schema(""))
            .expect("extra artifact must be written");
        assert!(matches!(
            generated.read().expect_err("extra artifact must fail"),
            EdtReportDataCompositionError::ExtraArtifact(_)
        ));

        let generated = GeneratedReport::new(&descriptor, &[]);
        let unreadable_artifact = generated.directory.join("Templates/Main/Template.dcs");
        fs::create_dir_all(&unreadable_artifact)
            .expect("directory at artifact path must be created");
        assert!(matches!(
            generated.read().expect_err("unreadable artifact must fail"),
            EdtReportDataCompositionError::ReadArtifact { .. }
        ));
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn data_source_data_set_field_and_query_failures_are_typed() {
        let descriptor = descriptor(&template("schema", "Main"), "");
        let cases = [
            (
                schema(&query_data_set("Query", "", "SELECT 1")),
                "missing root source",
                0,
            ),
            (
                schema(&format!(
                    "{}{}{}",
                    data_source(),
                    data_source(),
                    query_data_set("Query", "", "SELECT 1")
                )),
                "multiple root sources",
                1,
            ),
            (
                schema(&format!(
                    r#"{}<dataSet xsi:type="DataSetQuery"><name>Query</name><dataSource>Other</dataSource><query>SELECT 1</query></dataSet>"#,
                    data_source()
                )),
                "mismatched Data Set source",
                2,
            ),
            (
                schema(&format!(
                    r#"{}<dataSet xsi:type="DataSetQuery"><name>Query</name><dataSource>DataSource1</dataSource></dataSet>"#,
                    data_source()
                )),
                "missing query",
                3,
            ),
            (
                schema(&format!(
                    "{}{}{}",
                    data_source(),
                    query_data_set("Same", "", "SELECT 1"),
                    query_data_set("Same", "", "SELECT 2")
                )),
                "duplicate Data Set name",
                4,
            ),
            (
                schema(&format!(
                    "{}{}",
                    data_source(),
                    query_data_set(
                        "Query",
                        &format!("{}{}", field("Same", "Path.One"), field("Same", "Path.Two")),
                        "SELECT 1",
                    )
                )),
                "duplicate Field name",
                5,
            ),
            (
                schema(&format!(
                    "{}{}",
                    data_source(),
                    query_data_set(
                        "Query",
                        r#"<field xsi:type="DataSetFieldField"><field>MissingPath</field></field>"#,
                        "SELECT 1",
                    )
                )),
                "missing Field path",
                6,
            ),
            (
                schema(
                    r"<dataSource><name>Other</name><dataSourceType>Local</dataSourceType></dataSource>",
                ),
                "invalid root source name",
                7,
            ),
            (
                schema(
                    r"<dataSource><name>DataSource1</name><dataSourceType>Remote</dataSourceType></dataSource>",
                ),
                "invalid root source type",
                8,
            ),
            (
                schema(&format!(
                    r#"{}<dataSet xsi:type="DataSetQuery"><name></name><dataSource>DataSource1</dataSource><query>SELECT 1</query></dataSet>"#,
                    data_source()
                )),
                "invalid Data Set name",
                9,
            ),
            (
                schema(&format!(
                    "{}{}",
                    data_source(),
                    query_data_set("Query", &field("", "Path"), "SELECT 1")
                )),
                "invalid Field name",
                10,
            ),
            (
                schema(&format!(
                    r#"{}<dataSet xsi:type="DataSetObject"><name>Object</name><dataSource>DataSource1</dataSource><query>SELECT 1</query></dataSet>"#,
                    data_source()
                )),
                "unexpected Object query",
                11,
            ),
            (
                schema(&format!(
                    r#"{}<dataSet xsi:type="DataSetUnion"><name>Union</name><dataSource>DataSource1</dataSource></dataSet>"#,
                    data_source()
                )),
                "unexpected Union source",
                12,
            ),
        ];

        for (artifact, label, expected) in cases {
            let generated = GeneratedReport::new(&descriptor, &[("Main", &artifact)]);
            let error = generated.read().expect_err(label);
            let actual = match error {
                EdtReportDataCompositionError::InvalidRootDataSource(
                    EdtDataCompositionSourceReason::Missing,
                ) => 0,
                EdtReportDataCompositionError::InvalidRootDataSource(
                    EdtDataCompositionSourceReason::Multiple,
                ) => 1,
                EdtReportDataCompositionError::InvalidDataSetSource {
                    reason: EdtDataCompositionSourceReason::Unexpected,
                    ..
                } => 12,
                EdtReportDataCompositionError::InvalidDataSetSource { .. } => 2,
                EdtReportDataCompositionError::InvalidQueryCardinality(_) => 3,
                EdtReportDataCompositionError::DuplicateDataSetName { .. } => 4,
                EdtReportDataCompositionError::DuplicateFieldName { .. } => 5,
                EdtReportDataCompositionError::InvalidFieldPath { .. } => 6,
                EdtReportDataCompositionError::InvalidRootDataSource(
                    EdtDataCompositionSourceReason::InvalidName,
                ) => 7,
                EdtReportDataCompositionError::InvalidRootDataSource(
                    EdtDataCompositionSourceReason::InvalidType,
                ) => 8,
                EdtReportDataCompositionError::InvalidDataSetName(_) => 9,
                EdtReportDataCompositionError::InvalidFieldName(_) => 10,
                EdtReportDataCompositionError::UnexpectedQuery(_) => 11,
                other => panic!("unexpected typed error for {label}: {other:?}"),
            };
            assert_eq!(actual, expected, "wrong error classification for {label}");
        }
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn all_live_report_declarations_match_the_accepted_contract() {
        let reports_directory =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../OneAgent_EDTproject/src/Reports");
        let mut report_directories = fs::read_dir(&reports_directory)
            .expect("live Reports directory must be readable")
            .map(|entry| entry.expect("live Report entry must be readable").path())
            .filter(|path| path.is_dir())
            .collect::<Vec<_>>();
        report_directories.sort();

        let mut schema_count = 0;
        let mut source_count = 0;
        let mut data_set_count = 0;
        let mut main_schema_count = 0;
        let mut query_data_set_count = 0;
        let mut object_data_set_count = 0;
        let mut union_data_set_count = 0;
        let mut field_count = 0;
        let mut query_count = 0;
        let mut nested_count = 0;
        let mut folder_count = 0;
        for directory in &report_directories {
            let report = FileSystemEdtMetadataObjectReader
                .read(directory, MetadataKind::Report)
                .expect("live Report descriptor must parse");
            let first = FileSystemEdtReportDataCompositionReader
                .read(directory, &report)
                .expect("live Report Data Composition source must parse");
            let repeated = FileSystemEdtReportDataCompositionReader
                .read(directory, &report)
                .expect("repeated live Report Data Composition read must parse");
            assert_eq!(first, repeated);

            schema_count += first.schemas().len();
            main_schema_count += first
                .schemas()
                .iter()
                .filter(|schema| schema.is_main())
                .count();
            source_count += first
                .schemas()
                .iter()
                .filter(|schema| schema.data_source().is_some())
                .count();
            data_set_count += first
                .schemas()
                .iter()
                .map(|schema| schema.data_sets().len())
                .sum::<usize>();
            for data_set in first
                .schemas()
                .iter()
                .flat_map(super::EdtDataCompositionSchemaDescriptor::data_sets)
            {
                match data_set.kind() {
                    DataSetKind::Query => query_data_set_count += 1,
                    DataSetKind::Object => object_data_set_count += 1,
                    DataSetKind::Union => union_data_set_count += 1,
                }
            }
            field_count += first
                .schemas()
                .iter()
                .flat_map(super::EdtDataCompositionSchemaDescriptor::data_sets)
                .map(|data_set| data_set.fields().len())
                .sum::<usize>();
            query_count += first
                .schemas()
                .iter()
                .flat_map(super::EdtDataCompositionSchemaDescriptor::data_sets)
                .filter(|data_set| data_set.query().is_some())
                .count();
            nested_count += first
                .observations()
                .filter(|observation| {
                    observation.kind() == EdtDataCompositionObservationKind::NestedDataSet
                })
                .count();
            folder_count += first
                .observations()
                .filter(|observation| {
                    observation.kind() == EdtDataCompositionObservationKind::FieldFolder
                })
                .count();
            assert!(first.observations().all(|observation| {
                matches!(
                    observation.kind(),
                    EdtDataCompositionObservationKind::NestedDataSet
                        | EdtDataCompositionObservationKind::FieldFolder
                )
            }));
        }

        assert_eq!(report_directories.len(), 56);
        assert_eq!(schema_count, 56);
        assert_eq!(main_schema_count, 51);
        assert_eq!(source_count, 54);
        assert_eq!(data_set_count, 70);
        assert_eq!(query_data_set_count, 38);
        assert_eq!(object_data_set_count, 25);
        assert_eq!(union_data_set_count, 7);
        assert_eq!(field_count, 970);
        assert_eq!(query_count, 38);
        assert_eq!(nested_count, 8);
        assert_eq!(folder_count, 6);
    }
}
