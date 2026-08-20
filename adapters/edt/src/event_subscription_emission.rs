//! Private production projection for EDT Event Subscription semantics.

use oneagent_common::EntityId;
use oneagent_graph::{
    Confidence, EdgeKind, FactOrigin, GraphEdge, GraphNode, GraphNodePayload, NodeKind, ProducerId,
    Provenance, ResolutionState, SemanticDiagnostic, SemanticDiagnosticCode,
    SemanticDiagnosticKind, SemanticDiagnosticSeverity, SemanticGraph, SemanticReference,
    SemanticReferenceOutcome, SemanticReferenceStatistics,
};
use oneagent_metadata::{
    CommonMetadataPayload, EventSubscriptionMetadataPayload, MetadataKind, MetadataPayload,
    MetadataSpecificPayload,
};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::event_subscription_resolution::{
    EdtEventSubscriptionHandlerResolutionOutcome, EdtEventSubscriptionResolutionIndex,
    EdtEventSubscriptionSourceResolution, EdtEventSubscriptionSourceResolutionOutcome,
};
use crate::{
    EdtEventSubscriptionDescriptor, EdtEventSubscriptionReader,
    EdtEventSubscriptionSourceOutcomeKind, EdtEventSubscriptionSourceReason, EdtGraphError,
    FileSystemEdtEventSubscriptionReader,
};

const EVENT_SUBSCRIPTION_PRODUCER: &str = "oneagent.edt.event-subscription-emission";

pub(crate) fn collect_event_subscription_directory(
    directory: &Path,
    configuration_id: &EntityId,
    graph: &mut SemanticGraph,
) -> Result<Vec<EdtEventSubscriptionDescriptor>, EdtGraphError> {
    let mut object_directories = BTreeSet::<PathBuf>::new();
    for entry in fs::read_dir(directory).map_err(|source| EdtGraphError::ReadDirectory {
        path: directory.to_path_buf(),
        source,
    })? {
        let entry = entry.map_err(|source| EdtGraphError::ReadDirectoryEntry {
            path: directory.to_path_buf(),
            source,
        })?;
        if entry
            .file_type()
            .map_err(|source| EdtGraphError::ReadFileType {
                path: entry.path(),
                source,
            })?
            .is_dir()
        {
            object_directories.insert(entry.path());
        }
    }

    let mut descriptors = Vec::new();
    for object_directory in object_directories {
        let descriptor = FileSystemEdtEventSubscriptionReader
            .read(&object_directory)
            .map_err(EdtGraphError::EventSubscription)?;
        emit_event_subscription_declaration(graph, configuration_id, &descriptor)?;
        descriptors.push(descriptor);
    }
    descriptors.sort_by(|left, right| {
        left.id()
            .cmp(right.id())
            .then_with(|| left.descriptor_path().cmp(right.descriptor_path()))
    });
    Ok(descriptors)
}

fn emit_event_subscription_declaration(
    graph: &mut SemanticGraph,
    configuration_id: &EntityId,
    descriptor: &EdtEventSubscriptionDescriptor,
) -> Result<(), EdtGraphError> {
    let payload = MetadataPayload::new(
        CommonMetadataPayload::new(descriptor.synonym().map(str::to_owned)),
        Some(MetadataSpecificPayload::EventSubscription(
            EventSubscriptionMetadataPayload::new(descriptor.event().clone()),
        )),
    );
    let node = GraphNode::new_with_payload_and_provenance(
        descriptor.id().clone(),
        descriptor.name().clone(),
        NodeKind::Metadata(MetadataKind::EventSubscription),
        GraphNodePayload::Metadata(payload),
        vec![declaration_provenance(
            descriptor,
            "metadata_node",
            descriptor.id(),
        )?],
    )
    .map_err(EdtGraphError::NodePayload)?;
    graph.insert_node(node);
    graph
        .insert_edge(GraphEdge::new_with_provenance(
            configuration_id.clone(),
            descriptor.id().clone(),
            EdgeKind::Contains,
            vec![declaration_provenance(
                descriptor,
                "configuration_ownership",
                configuration_id,
            )?],
        ))
        .map_err(EdtGraphError::Graph)?;
    Ok(())
}

pub(crate) fn emit_resolved_event_subscriptions(
    graph: &mut SemanticGraph,
    descriptors: &[EdtEventSubscriptionDescriptor],
    diagnostics: &mut BTreeSet<SemanticDiagnostic>,
    reference_statistics: &mut SemanticReferenceStatistics,
) -> Result<usize, EdtGraphError> {
    let mut ordered = descriptors.iter().collect::<Vec<_>>();
    ordered.sort_by(|left, right| {
        left.id()
            .cmp(right.id())
            .then_with(|| left.descriptor_path().cmp(right.descriptor_path()))
    });

    let mut evidence_by_edge = BTreeMap::<(EntityId, EntityId, EdgeKind), Vec<Provenance>>::new();
    for descriptor in ordered {
        let resolution = EdtEventSubscriptionResolutionIndex::new(graph).resolve(descriptor);
        debug_assert_eq!(resolution.descriptor_id(), descriptor.id());
        debug_assert_eq!(resolution.descriptor_path(), descriptor.descriptor_path());

        for source in resolution.sources() {
            contribute_source(
                descriptor,
                source,
                &mut evidence_by_edge,
                diagnostics,
                reference_statistics,
            )?;
        }
        contribute_handler(
            descriptor,
            resolution.handler().outcome(),
            &mut evidence_by_edge,
            diagnostics,
            reference_statistics,
        )?;
    }

    let mut inserted = 0;
    for ((source, target, kind), mut provenance) in evidence_by_edge {
        provenance.sort_by(compare_provenance);
        provenance.dedup();
        inserted += usize::from(
            graph
                .insert_edge(GraphEdge::new_with_provenance(
                    source, target, kind, provenance,
                ))
                .map_err(EdtGraphError::Graph)?,
        );
    }
    Ok(inserted)
}

fn contribute_source(
    descriptor: &EdtEventSubscriptionDescriptor,
    source: &EdtEventSubscriptionSourceResolution,
    evidence_by_edge: &mut BTreeMap<(EntityId, EntityId, EdgeKind), Vec<Provenance>>,
    diagnostics: &mut BTreeSet<SemanticDiagnostic>,
    statistics: &mut SemanticReferenceStatistics,
) -> Result<(), EdtGraphError> {
    match source.outcome() {
        EdtEventSubscriptionSourceResolutionOutcome::Resolved { target_ids } => {
            statistics.record(SemanticReferenceOutcome::Resolved, true);
            for target_id in target_ids {
                let evidence = evidence_by_edge
                    .entry((
                        descriptor.id().clone(),
                        target_id.clone(),
                        EdgeKind::References,
                    ))
                    .or_default();
                for context in source.observation().contexts() {
                    evidence.push(source_provenance(
                        descriptor,
                        source,
                        context.occurrence_ordinal(),
                        Some(target_id),
                        "resolved",
                        ResolutionState::Resolved,
                    )?);
                }
            }
        }
        outcome => {
            let (diagnostic, statistics_outcome) = source_diagnostic(descriptor, source, outcome)?;
            diagnostics.insert(diagnostic);
            statistics.record(statistics_outcome, true);
        }
    }
    Ok(())
}

fn contribute_handler(
    descriptor: &EdtEventSubscriptionDescriptor,
    outcome: &EdtEventSubscriptionHandlerResolutionOutcome,
    evidence_by_edge: &mut BTreeMap<(EntityId, EntityId, EdgeKind), Vec<Provenance>>,
    diagnostics: &mut BTreeSet<SemanticDiagnostic>,
    statistics: &mut SemanticReferenceStatistics,
) -> Result<(), EdtGraphError> {
    if let EdtEventSubscriptionHandlerResolutionOutcome::Resolved { target_id } = outcome {
        statistics.record(SemanticReferenceOutcome::Resolved, true);
        for kind in [EdgeKind::References, EdgeKind::Triggers] {
            evidence_by_edge
                .entry((descriptor.id().clone(), target_id.clone(), kind))
                .or_default()
                .push(handler_provenance(
                    descriptor,
                    Some(target_id),
                    kind,
                    "resolved",
                    ResolutionState::Resolved,
                )?);
        }
    } else {
        let (diagnostic, statistics_outcome) = handler_diagnostic(descriptor, outcome)?;
        diagnostics.insert(diagnostic);
        statistics.record(statistics_outcome, true);
    }
    Ok(())
}

fn source_diagnostic(
    descriptor: &EdtEventSubscriptionDescriptor,
    source: &EdtEventSubscriptionSourceResolution,
    outcome: &EdtEventSubscriptionSourceResolutionOutcome,
) -> Result<(SemanticDiagnostic, SemanticReferenceOutcome), EdtGraphError> {
    let observation = source.observation();
    let (code, kind, statistics_outcome, message, candidates, resolution, outcome_name) =
        match outcome {
            EdtEventSubscriptionSourceResolutionOutcome::Missing => (
                SemanticDiagnosticCode::ReferenceUnresolved,
                SemanticDiagnosticKind::UnresolvedTarget,
                SemanticReferenceOutcome::Unresolved,
                "Event Subscription source target could not be resolved",
                Vec::new(),
                ResolutionState::Unresolved,
                "missing",
            ),
            EdtEventSubscriptionSourceResolutionOutcome::Ambiguous { candidates } => (
                SemanticDiagnosticCode::ReferenceAmbiguous,
                SemanticDiagnosticKind::AmbiguousTarget,
                SemanticReferenceOutcome::Ambiguous,
                "Event Subscription source target is ambiguous",
                candidates.clone(),
                ResolutionState::Ambiguous,
                "ambiguous",
            ),
            EdtEventSubscriptionSourceResolutionOutcome::IncompatibleKind { candidates } => (
                SemanticDiagnosticCode::ReferenceIncompatibleKind,
                SemanticDiagnosticKind::IncompatibleTargetKind,
                SemanticReferenceOutcome::IncompatibleTargetKind,
                "Event Subscription source target has an incompatible kind",
                candidates.clone(),
                ResolutionState::Unresolved,
                "incompatible_kind",
            ),
            EdtEventSubscriptionSourceResolutionOutcome::RejectedObservation { kind, reason } => {
                rejected_source_classification(*kind, *reason)
            }
            EdtEventSubscriptionSourceResolutionOutcome::Resolved { .. } => {
                unreachable!("resolved sources do not produce diagnostics")
            }
        };
    let mut provenance = observation
        .contexts()
        .iter()
        .map(|context| {
            source_provenance(
                descriptor,
                source,
                context.occurrence_ordinal(),
                None,
                outcome_name,
                resolution,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    provenance.sort_by(compare_provenance);
    provenance.dedup();

    let mut diagnostic = SemanticDiagnostic::new(
        code,
        SemanticDiagnosticSeverity::Error,
        kind,
        message,
        SemanticReference::Raw(observation.raw_selector().to_owned()),
    )
    .with_source_node(descriptor.id().clone())
    .with_candidates(candidates)
    .with_provenance(provenance);
    if let Some(target_kind) = observation.target_kind() {
        diagnostic = diagnostic.with_expected_kinds(vec![NodeKind::Metadata(target_kind)]);
    }
    Ok((diagnostic, statistics_outcome))
}

#[allow(clippy::type_complexity)]
fn rejected_source_classification(
    kind: EdtEventSubscriptionSourceOutcomeKind,
    reason: EdtEventSubscriptionSourceReason,
) -> (
    SemanticDiagnosticCode,
    SemanticDiagnosticKind,
    SemanticReferenceOutcome,
    &'static str,
    Vec<EntityId>,
    ResolutionState,
    &'static str,
) {
    match kind {
        EdtEventSubscriptionSourceOutcomeKind::Malformed => (
            SemanticDiagnosticCode::ReferenceMalformedFormat,
            SemanticDiagnosticKind::MalformedReferenceFormat,
            SemanticReferenceOutcome::MalformedFormat,
            "Event Subscription source selector is malformed",
            Vec::new(),
            ResolutionState::Unresolved,
            source_reason_name(reason),
        ),
        EdtEventSubscriptionSourceOutcomeKind::Unsupported => (
            SemanticDiagnosticCode::ReferenceUnsupportedPrefix,
            SemanticDiagnosticKind::UnsupportedReferencePrefix,
            SemanticReferenceOutcome::UnsupportedPrefix,
            "Event Subscription source selector uses an unsupported prefix",
            Vec::new(),
            ResolutionState::Unresolved,
            source_reason_name(reason),
        ),
        EdtEventSubscriptionSourceOutcomeKind::Supported => {
            unreachable!("supported source observations cannot use rejected classification")
        }
    }
}

fn handler_diagnostic(
    descriptor: &EdtEventSubscriptionDescriptor,
    outcome: &EdtEventSubscriptionHandlerResolutionOutcome,
) -> Result<(SemanticDiagnostic, SemanticReferenceOutcome), EdtGraphError> {
    let classification = handler_diagnostic_classification(outcome);
    let diagnostic = SemanticDiagnostic::new(
        classification.code,
        SemanticDiagnosticSeverity::Error,
        classification.kind,
        classification.message,
        SemanticReference::Raw(descriptor.handler().raw_path().to_owned()),
    )
    .with_source_node(descriptor.id().clone())
    .with_expected_kinds(classification.expected)
    .with_candidates(classification.candidates)
    .with_provenance(vec![handler_provenance(
        descriptor,
        None,
        EdgeKind::References,
        classification.outcome_name,
        classification.resolution,
    )?]);
    Ok((diagnostic, classification.statistics_outcome))
}

struct HandlerDiagnosticClassification {
    code: SemanticDiagnosticCode,
    kind: SemanticDiagnosticKind,
    statistics_outcome: SemanticReferenceOutcome,
    message: &'static str,
    expected: Vec<NodeKind>,
    candidates: Vec<EntityId>,
    resolution: ResolutionState,
    outcome_name: &'static str,
}

fn handler_diagnostic_classification(
    outcome: &EdtEventSubscriptionHandlerResolutionOutcome,
) -> HandlerDiagnosticClassification {
    let common_module = NodeKind::Metadata(MetadataKind::CommonModule);
    match outcome {
        EdtEventSubscriptionHandlerResolutionOutcome::MissingCommonModule => {
            HandlerDiagnosticClassification::missing(
                "Event Subscription handler Common Module could not be resolved",
                common_module,
                "missing_common_module",
            )
        }
        EdtEventSubscriptionHandlerResolutionOutcome::AmbiguousCommonModule { candidates } => {
            HandlerDiagnosticClassification::ambiguous(
                "Event Subscription handler Common Module is ambiguous",
                common_module,
                candidates,
                "ambiguous_common_module",
            )
        }
        EdtEventSubscriptionHandlerResolutionOutcome::IncompatibleCommonModuleKind {
            candidates,
        } => HandlerDiagnosticClassification::incompatible(
            "Event Subscription handler Common Module has an incompatible kind",
            common_module,
            candidates,
            "incompatible_common_module",
        ),
        EdtEventSubscriptionHandlerResolutionOutcome::MissingModule { .. } => {
            HandlerDiagnosticClassification::missing(
                "Event Subscription handler Module could not be resolved",
                NodeKind::Module,
                "missing_module",
            )
        }
        EdtEventSubscriptionHandlerResolutionOutcome::AmbiguousModule { candidates, .. } => {
            HandlerDiagnosticClassification::ambiguous(
                "Event Subscription handler Module is ambiguous",
                NodeKind::Module,
                candidates,
                "ambiguous_module",
            )
        }
        EdtEventSubscriptionHandlerResolutionOutcome::IncompatibleModuleKind {
            candidates, ..
        } => HandlerDiagnosticClassification::incompatible(
            "Event Subscription handler Module has an incompatible kind",
            NodeKind::Module,
            candidates,
            "incompatible_module",
        ),
        EdtEventSubscriptionHandlerResolutionOutcome::MissingSymbol { .. } => {
            HandlerDiagnosticClassification::missing(
                "Event Subscription handler Procedure could not be resolved",
                NodeKind::Procedure,
                "missing_symbol",
            )
        }
        EdtEventSubscriptionHandlerResolutionOutcome::AmbiguousSymbol { candidates, .. } => {
            HandlerDiagnosticClassification::ambiguous(
                "Event Subscription handler Procedure is ambiguous",
                NodeKind::Procedure,
                candidates,
                "ambiguous_symbol",
            )
        }
        EdtEventSubscriptionHandlerResolutionOutcome::IncompatibleSymbolKind {
            candidates, ..
        } => HandlerDiagnosticClassification::incompatible(
            "Event Subscription handler symbol is not a Procedure",
            NodeKind::Procedure,
            candidates,
            "incompatible_symbol",
        ),
        EdtEventSubscriptionHandlerResolutionOutcome::InvalidOwner { candidates, .. } => {
            HandlerDiagnosticClassification::invalid_owner(candidates)
        }
        EdtEventSubscriptionHandlerResolutionOutcome::Resolved { .. } => {
            unreachable!("resolved handler does not produce a diagnostic")
        }
    }
}

impl HandlerDiagnosticClassification {
    fn missing(message: &'static str, expected: NodeKind, outcome_name: &'static str) -> Self {
        Self::new(
            SemanticDiagnosticCode::ReferenceUnresolved,
            SemanticDiagnosticKind::UnresolvedTarget,
            SemanticReferenceOutcome::Unresolved,
            message,
            expected,
            Vec::new(),
            ResolutionState::Unresolved,
            outcome_name,
        )
    }

    fn ambiguous(
        message: &'static str,
        expected: NodeKind,
        candidates: &[EntityId],
        outcome_name: &'static str,
    ) -> Self {
        Self::new(
            SemanticDiagnosticCode::ReferenceAmbiguous,
            SemanticDiagnosticKind::AmbiguousTarget,
            SemanticReferenceOutcome::Ambiguous,
            message,
            expected,
            candidates.to_vec(),
            ResolutionState::Ambiguous,
            outcome_name,
        )
    }

    fn incompatible(
        message: &'static str,
        expected: NodeKind,
        candidates: &[EntityId],
        outcome_name: &'static str,
    ) -> Self {
        Self::new(
            SemanticDiagnosticCode::ReferenceIncompatibleKind,
            SemanticDiagnosticKind::IncompatibleTargetKind,
            SemanticReferenceOutcome::IncompatibleTargetKind,
            message,
            expected,
            candidates.to_vec(),
            ResolutionState::Unresolved,
            outcome_name,
        )
    }

    fn invalid_owner(candidates: &[EntityId]) -> Self {
        Self::new(
            SemanticDiagnosticCode::ReferenceInvalidOwner,
            SemanticDiagnosticKind::InvalidOwnerReference,
            SemanticReferenceOutcome::InvalidOwnerReference,
            "Event Subscription handler Procedure is not owned by the declared Common Module",
            NodeKind::Procedure,
            candidates.to_vec(),
            ResolutionState::Unresolved,
            "invalid_owner",
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn new(
        code: SemanticDiagnosticCode,
        kind: SemanticDiagnosticKind,
        statistics_outcome: SemanticReferenceOutcome,
        message: &'static str,
        expected: NodeKind,
        candidates: Vec<EntityId>,
        resolution: ResolutionState,
        outcome_name: &'static str,
    ) -> Self {
        Self {
            code,
            kind,
            statistics_outcome,
            message,
            expected: vec![expected],
            candidates,
            resolution,
            outcome_name,
        }
    }
}

fn declaration_provenance(
    descriptor: &EdtEventSubscriptionDescriptor,
    role: &str,
    target: &EntityId,
) -> Result<Provenance, EdtGraphError> {
    Ok(provenance(
        declaration_source_id(descriptor, role, target)?,
        FactOrigin::Declared,
        ResolutionState::NotApplicable,
    ))
}

fn source_provenance(
    descriptor: &EdtEventSubscriptionDescriptor,
    source: &EdtEventSubscriptionSourceResolution,
    occurrence: usize,
    target: Option<&EntityId>,
    outcome: &str,
    resolution: ResolutionState,
) -> Result<Provenance, EdtGraphError> {
    Ok(provenance(
        source_observation_source_id(descriptor, source, occurrence, target, outcome)?,
        FactOrigin::Resolved,
        resolution,
    ))
}

fn handler_provenance(
    descriptor: &EdtEventSubscriptionDescriptor,
    target: Option<&EntityId>,
    edge_kind: EdgeKind,
    outcome: &str,
    resolution: ResolutionState,
) -> Result<Provenance, EdtGraphError> {
    Ok(provenance(
        handler_observation_source_id(descriptor, target, edge_kind, outcome)?,
        FactOrigin::Resolved,
        resolution,
    ))
}

fn provenance(source: EntityId, origin: FactOrigin, resolution: ResolutionState) -> Provenance {
    Provenance::new(
        Some(source),
        ProducerId::new(EVENT_SUBSCRIPTION_PRODUCER),
        origin,
        Confidence::Exact,
        resolution,
    )
}

fn declaration_source_id(
    descriptor: &EdtEventSubscriptionDescriptor,
    role: &str,
    target: &EntityId,
) -> Result<EntityId, EdtGraphError> {
    encoded_source_id(
        descriptor.descriptor_path(),
        &[
            ("stage", "event_subscription_declaration"),
            ("role", role),
            ("subscription", descriptor.id().as_str()),
            ("event", descriptor.event().as_str()),
            ("target", target.as_str()),
            ("producer", EVENT_SUBSCRIPTION_PRODUCER),
        ],
    )
}

fn source_observation_source_id(
    descriptor: &EdtEventSubscriptionDescriptor,
    source: &EdtEventSubscriptionSourceResolution,
    occurrence: usize,
    target: Option<&EntityId>,
    outcome: &str,
) -> Result<EntityId, EdtGraphError> {
    let occurrence = occurrence.to_string();
    let target_kind = source
        .observation()
        .target_kind()
        .map_or("none", MetadataKind::as_str);
    encoded_source_id(
        descriptor.descriptor_path(),
        &[
            ("stage", "event_subscription_source_resolution"),
            ("role", "source_selector"),
            ("subscription", descriptor.id().as_str()),
            ("occurrence", &occurrence),
            ("raw_selector", source.observation().raw_selector()),
            ("normalized_selector", source.observation().raw_selector()),
            ("target_kind", target_kind),
            ("target", target.map_or("none", EntityId::as_str)),
            ("outcome", outcome),
            ("producer", EVENT_SUBSCRIPTION_PRODUCER),
        ],
    )
}

fn handler_observation_source_id(
    descriptor: &EdtEventSubscriptionDescriptor,
    target: Option<&EntityId>,
    edge_kind: EdgeKind,
    outcome: &str,
) -> Result<EntityId, EdtGraphError> {
    let edge = match edge_kind {
        EdgeKind::References => "references",
        EdgeKind::Triggers => "triggers",
        _ => unreachable!("handler provenance is only created for accepted handler relations"),
    };
    encoded_source_id(
        descriptor.descriptor_path(),
        &[
            ("stage", "event_subscription_handler_resolution"),
            ("role", "handler"),
            ("subscription", descriptor.id().as_str()),
            ("raw_path", descriptor.handler().raw_path()),
            ("normalized_path", descriptor.handler().raw_path()),
            ("module", descriptor.handler().module_name().as_str()),
            ("procedure", descriptor.handler().procedure_name().as_str()),
            ("edge", edge),
            ("target", target.map_or("none", EntityId::as_str)),
            ("outcome", outcome),
            ("producer", EVENT_SUBSCRIPTION_PRODUCER),
        ],
    )
}

fn encoded_source_id(path: &Path, fields: &[(&str, &str)]) -> Result<EntityId, EdtGraphError> {
    let mut context = String::new();
    for (index, (name, value)) in fields.iter().enumerate() {
        if index > 0 {
            context.push(';');
        }
        context.push_str(name);
        context.push('#');
        context.push_str(&value.len().to_string());
        context.push(':');
        context.push_str(value);
    }
    EntityId::new(format!(
        "{}#{context}",
        path.to_string_lossy().replace('\\', "/")
    ))
    .map_err(|_| EdtGraphError::InvalidIdentifier)
}

const fn source_reason_name(reason: EdtEventSubscriptionSourceReason) -> &'static str {
    match reason {
        EdtEventSubscriptionSourceReason::UnsupportedPrefix => "unsupported_prefix",
        EdtEventSubscriptionSourceReason::EmptyValue => "empty_value",
        EdtEventSubscriptionSourceReason::EmptyComponent => "empty_component",
        EdtEventSubscriptionSourceReason::AdditionalComponents => "additional_components",
    }
}

fn compare_provenance(left: &Provenance, right: &Provenance) -> std::cmp::Ordering {
    left.source()
        .cmp(&right.source())
        .then_with(|| left.producer().cmp(right.producer()))
        .then_with(|| left.origin().cmp(&right.origin()))
        .then_with(|| left.confidence().cmp(&right.confidence()))
        .then_with(|| left.resolution().cmp(&right.resolution()))
}
