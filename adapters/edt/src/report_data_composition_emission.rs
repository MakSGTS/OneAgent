//! Production projection for parsed Report Data Composition source facts.

use oneagent_common::{EntityId, EntityName};
use oneagent_graph::{
    Confidence, DataCompositionFieldPayload, DataCompositionSchemaPayload, DataSetPayload,
    EdgeKind, FactOrigin, GraphNode, GraphNodePayload, NodeKind, ProducerId, Provenance,
    ResolutionState, SemanticDiagnostic, SemanticDiagnosticCode, SemanticDiagnosticKind,
    SemanticDiagnosticSeverity, SemanticGraph, SemanticReference, SemanticReferenceOutcome,
    SemanticReferenceStatistics, data_set_query_id,
};
use oneagent_metadata::MetadataKind;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use crate::{
    EdtDataCompositionObservation, EdtDataCompositionObservationKind, EdtGraphError,
    EdtReportDataCompositionDescriptor,
};

const PRODUCER: &str = "oneagent.edt.report-data-composition-emission";

pub(crate) fn emit_report_data_composition(
    project_root: &Path,
    graph: &mut SemanticGraph,
    descriptors: &[EdtReportDataCompositionDescriptor],
    diagnostics: &mut BTreeSet<SemanticDiagnostic>,
    legacy_statistics: &mut SemanticReferenceStatistics,
) -> Result<(), EdtGraphError> {
    let mut ordered = descriptors.iter().collect::<Vec<_>>();
    ordered.sort_by(|left, right| left.report_id().cmp(right.report_id()));

    for descriptor in ordered {
        ensure_report_owner(graph, descriptor)?;
        for schema in descriptor.schemas() {
            emit_schema(
                project_root,
                graph,
                descriptor,
                schema,
                diagnostics,
                legacy_statistics,
            )?;
        }
    }
    Ok(())
}

fn emit_schema(
    project_root: &Path,
    graph: &mut SemanticGraph,
    descriptor: &EdtReportDataCompositionDescriptor,
    schema: &crate::EdtDataCompositionSchemaDescriptor,
    diagnostics: &mut BTreeSet<SemanticDiagnostic>,
    legacy_statistics: &mut SemanticReferenceStatistics,
) -> Result<(), EdtGraphError> {
    let artifact = project_relative_artifact(project_root, schema.artifact_path())?;
    insert_payload_node(
        graph,
        schema.id().clone(),
        schema.name().clone(),
        NodeKind::DataCompositionSchema,
        GraphNodePayload::DataCompositionSchema(DataCompositionSchemaPayload::new(
            schema.is_main(),
        )),
        provenance(source_id(
            &artifact,
            &[
                component("report", descriptor.report_id().as_str()),
                component("schema", schema.id().as_str()),
                "role=schema".to_owned(),
                component("name", schema.name().as_str()),
                format!("main={}", schema.is_main()),
            ],
        )?),
    )?;
    insert_contains(
        graph,
        descriptor.report_id(),
        schema.id(),
        &artifact,
        "report_to_schema",
    )?;
    for data_set in schema.data_sets() {
        emit_data_set(graph, descriptor, schema, data_set, &artifact)?;
    }
    for observation in schema.observations() {
        emit_observation(&artifact, observation, diagnostics, legacy_statistics)?;
    }
    Ok(())
}

fn emit_data_set(
    graph: &mut SemanticGraph,
    descriptor: &EdtReportDataCompositionDescriptor,
    schema: &crate::EdtDataCompositionSchemaDescriptor,
    data_set: &crate::EdtDataCompositionDataSet,
    artifact: &Path,
) -> Result<(), EdtGraphError> {
    insert_payload_node(
        graph,
        data_set.id().clone(),
        data_set.name().clone(),
        NodeKind::DataSet,
        GraphNodePayload::DataSet(
            DataSetPayload::new(data_set.kind(), data_set.data_source().cloned())
                .map_err(EdtGraphError::DataSetPayload)?,
        ),
        provenance(source_id(
            artifact,
            &[
                component("report", descriptor.report_id().as_str()),
                component("schema", schema.id().as_str()),
                component("data_set", data_set.id().as_str()),
                "role=data_set".to_owned(),
                component("name", data_set.name().as_str()),
                format!("kind={}", data_set.kind().as_str()),
                optional_component(
                    "data_source",
                    data_set.data_source().map(EntityName::as_str),
                ),
            ],
        )?),
    )?;
    insert_contains(
        graph,
        schema.id(),
        data_set.id(),
        artifact,
        "schema_to_data_set",
    )?;
    for field in data_set.fields() {
        emit_field(graph, descriptor, schema, data_set, field, artifact)?;
    }
    if let Some(query) = data_set.query() {
        emit_query(graph, descriptor, schema, data_set, query, artifact)?;
    }
    Ok(())
}

fn emit_field(
    graph: &mut SemanticGraph,
    descriptor: &EdtReportDataCompositionDescriptor,
    schema: &crate::EdtDataCompositionSchemaDescriptor,
    data_set: &crate::EdtDataCompositionDataSet,
    field: &crate::EdtDataCompositionField,
    artifact: &Path,
) -> Result<(), EdtGraphError> {
    insert_payload_node(
        graph,
        field.id().clone(),
        field.name().clone(),
        NodeKind::DataCompositionField,
        GraphNodePayload::DataCompositionField(DataCompositionFieldPayload::new(
            field.data_path().clone(),
        )),
        provenance(source_id(
            artifact,
            &[
                component("report", descriptor.report_id().as_str()),
                component("schema", schema.id().as_str()),
                component("data_set", data_set.id().as_str()),
                component("field", field.id().as_str()),
                "role=field".to_owned(),
                component("name", field.name().as_str()),
                component("data_path", field.data_path().as_str()),
            ],
        )?),
    )?;
    insert_contains(
        graph,
        data_set.id(),
        field.id(),
        artifact,
        "data_set_to_field",
    )?;
    Ok(())
}

fn emit_query(
    graph: &mut SemanticGraph,
    descriptor: &EdtReportDataCompositionDescriptor,
    schema: &crate::EdtDataCompositionSchemaDescriptor,
    data_set: &crate::EdtDataCompositionDataSet,
    query: &str,
    artifact: &Path,
) -> Result<(), EdtGraphError> {
    let query_id =
        data_set_query_id(data_set.id()).map_err(|_| EdtGraphError::InvalidIdentifier)?;
    insert_plain_node(
        graph,
        query_id.clone(),
        EntityName::new("Query").expect("fixed Query role name must be valid"),
        NodeKind::Query,
        provenance(source_id(
            artifact,
            &[
                component("report", descriptor.report_id().as_str()),
                component("schema", schema.id().as_str()),
                component("data_set", data_set.id().as_str()),
                component("query", query_id.as_str()),
                "role=query".to_owned(),
                component("text", query),
            ],
        )?),
    )?;
    insert_contains(
        graph,
        data_set.id(),
        &query_id,
        artifact,
        "data_set_to_query",
    )?;
    Ok(())
}

fn ensure_report_owner(
    graph: &SemanticGraph,
    descriptor: &EdtReportDataCompositionDescriptor,
) -> Result<(), EdtGraphError> {
    let actual_kind = graph
        .node(descriptor.report_id())
        .map(oneagent_graph::GraphNode::kind);
    if actual_kind != Some(NodeKind::Metadata(MetadataKind::Report)) {
        return Err(EdtGraphError::InvalidReportDataCompositionOwner {
            report_id: descriptor.report_id().clone(),
            actual_kind,
        });
    }
    Ok(())
}

fn insert_payload_node(
    graph: &mut SemanticGraph,
    id: EntityId,
    name: EntityName,
    kind: NodeKind,
    payload: GraphNodePayload,
    provenance: Provenance,
) -> Result<(), EdtGraphError> {
    ensure_new_node(graph, &id)?;
    let node =
        GraphNode::new_with_payload_and_provenance(id, name, kind, payload, vec![provenance])
            .map_err(EdtGraphError::NodePayload)?;
    graph.insert_node(node);
    Ok(())
}

fn insert_plain_node(
    graph: &mut SemanticGraph,
    id: EntityId,
    name: EntityName,
    kind: NodeKind,
    provenance: Provenance,
) -> Result<(), EdtGraphError> {
    ensure_new_node(graph, &id)?;
    graph.insert_node(GraphNode::new_with_provenance(
        id,
        name,
        kind,
        vec![provenance],
    ));
    Ok(())
}

fn ensure_new_node(graph: &SemanticGraph, id: &EntityId) -> Result<(), EdtGraphError> {
    if graph.node(id).is_some() {
        return Err(EdtGraphError::DuplicateDataCompositionNode(id.clone()));
    }
    Ok(())
}

fn insert_contains(
    graph: &mut SemanticGraph,
    owner: &EntityId,
    child: &EntityId,
    artifact: &Path,
    role: &str,
) -> Result<(), EdtGraphError> {
    let source = source_id(
        artifact,
        &[
            component("owner", owner.as_str()),
            component("child", child.as_str()),
            format!("role={role}"),
            "edge=contains".to_owned(),
        ],
    )?;
    graph
        .insert_edge_with_provenance(
            owner.clone(),
            child.clone(),
            EdgeKind::Contains,
            provenance(source),
        )
        .map_err(EdtGraphError::Graph)?;
    Ok(())
}

fn emit_observation(
    artifact: &Path,
    observation: &EdtDataCompositionObservation,
    diagnostics: &mut BTreeSet<SemanticDiagnostic>,
    legacy_statistics: &mut SemanticReferenceStatistics,
) -> Result<(), EdtGraphError> {
    let (code, kind, role, message) = match observation.kind() {
        EdtDataCompositionObservationKind::NestedDataSet => (
            SemanticDiagnosticCode::DataCompositionNestedDataSetDeferred,
            SemanticDiagnosticKind::DataCompositionNestedDataSetDeferred,
            "nested_data_set",
            "nested Data Composition Data Set is deferred without a stable identity",
        ),
        EdtDataCompositionObservationKind::FieldFolder => (
            SemanticDiagnosticCode::DataCompositionFieldFolderDeferred,
            SemanticDiagnosticKind::DataCompositionFieldFolderDeferred,
            "field_folder",
            "Data Composition field folder is deferred outside the named Field model",
        ),
        EdtDataCompositionObservationKind::UnsupportedDataSetType => (
            SemanticDiagnosticCode::DataCompositionUnsupportedDataSetType,
            SemanticDiagnosticKind::DataCompositionUnsupportedDataSetType,
            "unsupported_data_set_type",
            "Data Composition Data Set type is unsupported",
        ),
        EdtDataCompositionObservationKind::UnsupportedFieldType => (
            SemanticDiagnosticCode::DataCompositionUnsupportedFieldType,
            SemanticDiagnosticKind::DataCompositionUnsupportedFieldType,
            "unsupported_field_type",
            "Data Composition field type is unsupported",
        ),
    };
    let raw_type = observation.raw_type().unwrap_or("absent");
    let reference = format!(
        "data_composition:{role};owner#{}:{};type#{}:{};occurrence={}",
        observation.owner_id().as_str().len(),
        observation.owner_id().as_str(),
        raw_type.len(),
        raw_type,
        observation.occurrence_ordinal(),
    );
    let source = source_id(
        artifact,
        &[
            component("owner", observation.owner_id().as_str()),
            format!("role={role}"),
            component("raw_type", raw_type),
            format!("occurrence={}", observation.occurrence_ordinal()),
            "outcome=deferred".to_owned(),
        ],
    )?;
    diagnostics.insert(
        SemanticDiagnostic::new(
            code,
            SemanticDiagnosticSeverity::Warning,
            kind,
            message,
            SemanticReference::Raw(reference),
        )
        .with_source_node(observation.owner_id().clone())
        .with_provenance(vec![Provenance::new(
            Some(source),
            ProducerId::new(PRODUCER),
            FactOrigin::Parsed,
            Confidence::Exact,
            ResolutionState::Partial,
        )]),
    );
    legacy_statistics.record(SemanticReferenceOutcome::UnsupportedPrefix, true);
    Ok(())
}

fn project_relative_artifact(project_root: &Path, path: &Path) -> Result<PathBuf, EdtGraphError> {
    path.strip_prefix(project_root)
        .map(Path::to_path_buf)
        .map_err(
            |_| EdtGraphError::ReportDataCompositionArtifactOutsideProject {
                project_root: project_root.to_path_buf(),
                path: path.to_path_buf(),
            },
        )
}

fn source_id(path: &Path, components: &[String]) -> Result<EntityId, EdtGraphError> {
    EntityId::new(format!(
        "{}#{}",
        path.to_string_lossy().replace('\\', "/"),
        components.join(";")
    ))
    .map_err(|_| EdtGraphError::InvalidIdentifier)
}

fn component(label: &str, value: &str) -> String {
    format!("{label}#{}:{value}", value.len())
}

fn optional_component(label: &str, value: Option<&str>) -> String {
    value.map_or_else(
        || format!("{label}=absent"),
        |value| component(label, value),
    )
}

fn provenance(source: EntityId) -> Provenance {
    Provenance::new(
        Some(source),
        ProducerId::new(PRODUCER),
        FactOrigin::Parsed,
        Confidence::Exact,
        ResolutionState::NotApplicable,
    )
}
