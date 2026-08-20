//! Private production projection for EDT XDTO Package and service semantics.

use oneagent_common::{EntityId, EntityName};
use oneagent_graph::{
    Confidence, EdgeKind, FactOrigin, GraphEdge, GraphNode, GraphNodePayload,
    HttpServiceMethodPayload, HttpServiceUrlTemplatePayload, NodeId, NodeKind, ProducerId,
    Provenance, ResolutionState, SemanticDiagnostic, SemanticDiagnosticCode,
    SemanticDiagnosticKind, SemanticDiagnosticSeverity, SemanticGraph, SemanticReference,
    SemanticReferenceCategory, SemanticReferenceRequest, SemanticReferenceRequestLedger,
    SemanticReferenceRequestOutcome, WebServiceOperationPayload, WebServiceParameterPayload,
    XdtoTypePayload, xdto_type_id,
};
use oneagent_metadata::{
    CommonMetadataPayload, HttpServiceMetadataPayload, MetadataKind, MetadataPayload,
    MetadataSpecificPayload, WebServiceMetadataPayload, WebServiceXdtoPackage,
    XdtoPackageMetadataPayload,
};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::{
    EdtGraphError, EdtHttpServiceDescriptor, EdtMetadataObjectDescriptor,
    EdtServiceDescriptorReader, EdtWebServiceDescriptor, EdtWebServiceXdtoPackage,
    EdtXdtoPackageDescriptor, EdtXdtoPackageReader, FileSystemEdtServiceDescriptorReader,
    FileSystemEdtXdtoPackageReader,
};

const PRODUCER: &str = "oneagent.edt.xdto-service-emission";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum EdtXdtoServiceSource {
    XdtoPackage(EdtXdtoPackageDescriptor),
    HttpService(EdtHttpServiceDescriptor),
    WebService(EdtWebServiceDescriptor),
}

impl EdtXdtoServiceSource {
    fn metadata(&self) -> &EdtMetadataObjectDescriptor {
        match self {
            Self::XdtoPackage(source) => source.metadata(),
            Self::HttpService(source) => source.metadata(),
            Self::WebService(source) => source.metadata(),
        }
    }
}

pub(crate) fn collect_source(
    metadata: &EdtMetadataObjectDescriptor,
) -> Result<Option<EdtXdtoServiceSource>, EdtGraphError> {
    match metadata.kind() {
        MetadataKind::XdtoPackage => FileSystemEdtXdtoPackageReader
            .read(metadata)
            .map(EdtXdtoServiceSource::XdtoPackage)
            .map(Some)
            .map_err(EdtGraphError::XdtoPackage),
        MetadataKind::HttpService => FileSystemEdtServiceDescriptorReader
            .read_http(metadata)
            .map(EdtXdtoServiceSource::HttpService)
            .map(Some)
            .map_err(EdtGraphError::ServiceDescriptor),
        MetadataKind::WebService => FileSystemEdtServiceDescriptorReader
            .read_web(metadata)
            .map(EdtXdtoServiceSource::WebService)
            .map(Some)
            .map_err(EdtGraphError::ServiceDescriptor),
        _ => Ok(None),
    }
}

pub(crate) fn metadata_payload(source: &EdtXdtoServiceSource) -> MetadataPayload {
    let common = CommonMetadataPayload::new(source.metadata().synonym().map(str::to_owned));
    let specific = match source {
        EdtXdtoServiceSource::XdtoPackage(source) => MetadataSpecificPayload::XdtoPackage(
            XdtoPackageMetadataPayload::new(source.namespace()),
        ),
        EdtXdtoServiceSource::HttpService(source) => {
            MetadataSpecificPayload::HttpService(HttpServiceMetadataPayload::new(source.root_url()))
        }
        EdtXdtoServiceSource::WebService(source) => {
            let packages = source
                .xdto_package()
                .into_iter()
                .map(|package| match package {
                    EdtWebServiceXdtoPackage::Repository(name) => {
                        WebServiceXdtoPackage::Repository(name.clone())
                    }
                    EdtWebServiceXdtoPackage::ExternalNamespace(namespace) => {
                        WebServiceXdtoPackage::ExternalNamespace(namespace.clone())
                    }
                });
            MetadataSpecificPayload::WebService(WebServiceMetadataPayload::new(
                source.namespace(),
                packages,
            ))
        }
    };
    MetadataPayload::new(common, Some(specific))
}

pub(crate) fn emit_declarations(
    graph: &mut SemanticGraph,
    source: &EdtXdtoServiceSource,
) -> Result<(), EdtGraphError> {
    match source {
        EdtXdtoServiceSource::XdtoPackage(source) => emit_xdto_types(graph, source),
        EdtXdtoServiceSource::HttpService(source) => emit_http_children(graph, source),
        EdtXdtoServiceSource::WebService(source) => emit_web_children(graph, source),
    }
}

fn emit_xdto_types(
    graph: &mut SemanticGraph,
    source: &EdtXdtoPackageDescriptor,
) -> Result<(), EdtGraphError> {
    for declaration in source.types() {
        let id = xdto_type_id(source.metadata().id(), declaration.name())
            .map_err(|_| EdtGraphError::InvalidIdentifier)?;
        insert_payload_node(
            graph,
            id.clone(),
            declaration.name().clone(),
            NodeKind::XdtoType,
            GraphNodePayload::XdtoType(XdtoTypePayload::new(declaration.kind())),
            declaration_provenance(
                source.artifact_path(),
                "xdto_type",
                &id,
                declaration.name().as_str(),
            )?,
        )?;
        insert_contains(
            graph,
            source.metadata().id().clone(),
            id.clone(),
            declaration_provenance(
                source.artifact_path(),
                "xdto_type_ownership",
                &id,
                declaration.name().as_str(),
            )?,
        )?;
    }
    Ok(())
}

fn emit_http_children(
    graph: &mut SemanticGraph,
    source: &EdtHttpServiceDescriptor,
) -> Result<(), EdtGraphError> {
    for template in source.url_templates() {
        insert_payload_node(
            graph,
            template.id().clone(),
            template.name().clone(),
            NodeKind::HttpServiceUrlTemplate,
            GraphNodePayload::HttpServiceUrlTemplate(HttpServiceUrlTemplatePayload::new(
                template.template(),
            )),
            declaration_provenance(
                source.metadata().descriptor_path(),
                "http_url_template",
                template.id(),
                template.name().as_str(),
            )?,
        )?;
        insert_contains(
            graph,
            source.metadata().id().clone(),
            template.id().clone(),
            declaration_provenance(
                source.metadata().descriptor_path(),
                "http_url_template_ownership",
                template.id(),
                template.name().as_str(),
            )?,
        )?;
        for method in template.methods() {
            insert_payload_node(
                graph,
                method.id().clone(),
                method.name().clone(),
                NodeKind::HttpServiceMethod,
                GraphNodePayload::HttpServiceMethod(HttpServiceMethodPayload::new(
                    method.http_method().cloned(),
                )),
                declaration_provenance(
                    source.metadata().descriptor_path(),
                    "http_method",
                    method.id(),
                    method.name().as_str(),
                )?,
            )?;
            insert_contains(
                graph,
                template.id().clone(),
                method.id().clone(),
                declaration_provenance(
                    source.metadata().descriptor_path(),
                    "http_method_ownership",
                    method.id(),
                    method.name().as_str(),
                )?,
            )?;
        }
    }
    Ok(())
}

fn emit_web_children(
    graph: &mut SemanticGraph,
    source: &EdtWebServiceDescriptor,
) -> Result<(), EdtGraphError> {
    for operation in source.operations() {
        insert_payload_node(
            graph,
            operation.id().clone(),
            operation.name().clone(),
            NodeKind::WebServiceOperation,
            GraphNodePayload::WebServiceOperation(WebServiceOperationPayload::new(
                operation.returning_type().clone(),
                operation.nillable(),
            )),
            declaration_provenance(
                source.metadata().descriptor_path(),
                "web_operation",
                operation.id(),
                operation.name().as_str(),
            )?,
        )?;
        insert_contains(
            graph,
            source.metadata().id().clone(),
            operation.id().clone(),
            declaration_provenance(
                source.metadata().descriptor_path(),
                "web_operation_ownership",
                operation.id(),
                operation.name().as_str(),
            )?,
        )?;
        for parameter in operation.parameters() {
            insert_payload_node(
                graph,
                parameter.id().clone(),
                parameter.name().clone(),
                NodeKind::WebServiceParameter,
                GraphNodePayload::WebServiceParameter(WebServiceParameterPayload::new(
                    parameter.value_type().clone(),
                    parameter.nillable(),
                    parameter.direction(),
                )),
                declaration_provenance(
                    source.metadata().descriptor_path(),
                    "web_parameter",
                    parameter.id(),
                    parameter.name().as_str(),
                )?,
            )?;
            insert_contains(
                graph,
                operation.id().clone(),
                parameter.id().clone(),
                declaration_provenance(
                    source.metadata().descriptor_path(),
                    "web_parameter_ownership",
                    parameter.id(),
                    parameter.name().as_str(),
                )?,
            )?;
        }
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
    let node =
        GraphNode::new_with_payload_and_provenance(id, name, kind, payload, vec![provenance])
            .map_err(EdtGraphError::NodePayload)?;
    graph.insert_node(node);
    Ok(())
}

fn insert_contains(
    graph: &mut SemanticGraph,
    owner: EntityId,
    child: EntityId,
    provenance: Provenance,
) -> Result<(), EdtGraphError> {
    graph
        .insert_edge(GraphEdge::new_with_provenance(
            owner,
            child,
            EdgeKind::Contains,
            vec![provenance],
        ))
        .map_err(EdtGraphError::Graph)?;
    Ok(())
}

pub(crate) fn resolve_and_emit(
    graph: &mut SemanticGraph,
    sources: &[EdtXdtoServiceSource],
    diagnostics: &mut BTreeSet<SemanticDiagnostic>,
    ledger: &mut SemanticReferenceRequestLedger,
) -> Result<(), EdtGraphError> {
    let namespaces = repository_namespaces(sources);
    let mut intents = collect_intents(sources, &namespaces)?;
    intents.sort_by(|left, right| left.request.id().cmp(right.request.id()));

    for intent in intents {
        let terminal = resolve_intent(graph, intent)?;
        if terminal.request.outcome() == SemanticReferenceRequestOutcome::Resolved {
            let target = terminal.request.candidates()[0].clone();
            insert_resolution_edge(
                graph,
                terminal.request.source_node().clone(),
                target.clone(),
                EdgeKind::References,
                terminal.path,
                terminal.request.category().as_str(),
                terminal.request.reference(),
            )?;
            if terminal.request.category() == SemanticReferenceCategory::Callable {
                insert_resolution_edge(
                    graph,
                    terminal.request.source_node().clone(),
                    target,
                    EdgeKind::Triggers,
                    terminal.path,
                    "handler_dispatch",
                    terminal.request.reference(),
                )?;
            }
        } else {
            diagnostics.insert(request_diagnostic(graph, &terminal.request));
        }
        ledger
            .insert(terminal.request)
            .map_err(EdtGraphError::ReferenceRequest)?;
    }
    Ok(())
}

fn repository_namespaces(sources: &[EdtXdtoServiceSource]) -> BTreeMap<String, Vec<EntityId>> {
    let mut namespaces = BTreeMap::<String, Vec<EntityId>>::new();
    for source in sources {
        if let EdtXdtoServiceSource::XdtoPackage(package) = source {
            namespaces
                .entry(package.namespace().to_owned())
                .or_default()
                .push(package.metadata().id().clone());
        }
    }
    for packages in namespaces.values_mut() {
        packages.sort();
        packages.dedup();
    }
    namespaces
}

struct PendingIntent<'a> {
    request: SemanticReferenceRequest,
    path: &'a Path,
    owners: Vec<EntityId>,
}

struct TerminalIntent<'a> {
    request: SemanticReferenceRequest,
    path: &'a Path,
}

fn collect_intents<'a>(
    sources: &'a [EdtXdtoServiceSource],
    namespaces: &BTreeMap<String, Vec<EntityId>>,
) -> Result<Vec<PendingIntent<'a>>, EdtGraphError> {
    let mut intents = Vec::new();
    for source in sources {
        match source {
            EdtXdtoServiceSource::XdtoPackage(_) => {}
            EdtXdtoServiceSource::HttpService(service) => {
                let module = common_module_id(service.metadata().id())?;
                for method in service
                    .url_templates()
                    .iter()
                    .flat_map(crate::EdtHttpUrlTemplate::methods)
                {
                    intents.push(callable_intent(
                        method.id().clone(),
                        method.handler().clone(),
                        module.clone(),
                        NodeKind::Function,
                        service.metadata().descriptor_path(),
                    )?);
                }
            }
            EdtXdtoServiceSource::WebService(service) => {
                if let Some(EdtWebServiceXdtoPackage::Repository(name)) = service.xdto_package() {
                    intents.push(request_intent(
                        service.metadata().id().clone(),
                        SemanticReferenceCategory::XdtoPackage,
                        SemanticReference::Name(name.clone()),
                        NodeKind::Metadata(MetadataKind::XdtoPackage),
                        Vec::new(),
                        service.metadata().descriptor_path(),
                    )?);
                }
                let module = common_module_id(service.metadata().id())?;
                for operation in service.operations() {
                    if let Some(packages) = namespaces.get(operation.returning_type().namespace()) {
                        intents.push(type_intent(
                            operation.id().clone(),
                            operation.returning_type().name().clone(),
                            packages,
                            service.metadata().descriptor_path(),
                        )?);
                    }
                    intents.push(callable_intent(
                        operation.id().clone(),
                        operation.procedure_name().clone(),
                        module.clone(),
                        NodeKind::Function,
                        service.metadata().descriptor_path(),
                    )?);
                    for parameter in operation.parameters() {
                        if let Some(packages) = namespaces.get(parameter.value_type().namespace()) {
                            intents.push(type_intent(
                                parameter.id().clone(),
                                parameter.value_type().name().clone(),
                                packages,
                                service.metadata().descriptor_path(),
                            )?);
                        }
                    }
                }
            }
        }
    }
    Ok(intents)
}

fn request_intent(
    source: EntityId,
    category: SemanticReferenceCategory,
    reference: SemanticReference,
    expected: NodeKind,
    owners: Vec<EntityId>,
    path: &Path,
) -> Result<PendingIntent<'_>, EdtGraphError> {
    let provenance = request_provenance(
        path,
        "collection",
        &source,
        category.as_str(),
        &reference,
        ResolutionState::Unresolved,
    )?;
    Ok(PendingIntent {
        request: SemanticReferenceRequest::collected(
            source,
            category,
            reference,
            [expected],
            [provenance],
        )
        .map_err(EdtGraphError::ReferenceRequest)?,
        path,
        owners,
    })
}

fn callable_intent(
    source: EntityId,
    name: EntityName,
    module: EntityId,
    expected: NodeKind,
    path: &Path,
) -> Result<PendingIntent<'_>, EdtGraphError> {
    request_intent(
        source,
        SemanticReferenceCategory::Callable,
        SemanticReference::Child {
            owner: module.clone(),
            name,
        },
        expected,
        vec![module],
        path,
    )
}

fn type_intent<'a>(
    source: EntityId,
    name: EntityName,
    packages: &[EntityId],
    path: &'a Path,
) -> Result<PendingIntent<'a>, EdtGraphError> {
    let owner = packages
        .first()
        .cloned()
        .ok_or(EdtGraphError::InvalidIdentifier)?;
    request_intent(
        source,
        SemanticReferenceCategory::XdtoType,
        SemanticReference::Child {
            owner: owner.clone(),
            name,
        },
        NodeKind::XdtoType,
        packages.to_vec(),
        path,
    )
}

fn resolve_intent<'a>(
    graph: &SemanticGraph,
    intent: PendingIntent<'a>,
) -> Result<TerminalIntent<'a>, EdtGraphError> {
    let query = graph.query();
    let expected = intent.request.expected_kinds()[0];
    let (SemanticReference::Name(name) | SemanticReference::Child { name, .. }) =
        intent.request.reference()
    else {
        return Err(EdtGraphError::InvalidXdtoServiceRequest);
    };
    let (compatible, incompatible, invalid_owner) = if intent.owners.is_empty() {
        let named = query.nodes_by_name(name);
        let compatible = named
            .iter()
            .filter(|node| node.kind() == expected)
            .map(|node| node.id().clone())
            .collect();
        let incompatible = named
            .iter()
            .filter(|node| node.kind() != expected)
            .map(|node| node.id().clone())
            .collect();
        (compatible, incompatible, Vec::new())
    } else {
        let mut direct = intent
            .owners
            .iter()
            .flat_map(|owner| query.children(&NodeId::new(owner.as_str())))
            .filter(|node| node.name() == name)
            .collect::<Vec<_>>();
        direct.sort_by(|left, right| left.id().cmp(right.id()));
        direct.dedup_by(|left, right| left.id() == right.id());
        let compatible = direct
            .iter()
            .filter(|node| node.kind() == expected)
            .map(|node| node.id().clone())
            .collect::<Vec<_>>();
        let incompatible = direct
            .iter()
            .filter(|node| node.kind() != expected)
            .map(|node| node.id().clone())
            .collect::<Vec<_>>();
        let invalid_owner = if direct.is_empty() {
            query
                .nodes_by_name_and_kind(name, expected)
                .into_iter()
                .filter(|node| {
                    let node_owners = query.owners(&NodeId::new(node.id().as_str()));
                    !intent
                        .owners
                        .iter()
                        .any(|owner| node_owners.iter().any(|candidate| candidate.id() == owner))
                })
                .map(|node| node.id().clone())
                .collect()
        } else {
            Vec::new()
        };
        (compatible, incompatible, invalid_owner)
    };

    let state = match compatible.len() {
        0 => ResolutionState::Unresolved,
        1 => ResolutionState::Resolved,
        _ => ResolutionState::Ambiguous,
    };
    let provenance = request_provenance(
        intent.path,
        "resolution",
        intent.request.source_node(),
        intent.request.category().as_str(),
        intent.request.reference(),
        state,
    )?;
    let request = match compatible.as_slice() {
        [target] => intent
            .request
            .into_resolved(target.clone(), expected, [provenance]),
        [_, _, ..] => intent
            .request
            .into_ambiguous_target(compatible, [provenance]),
        [] if !incompatible.is_empty() => intent
            .request
            .into_incompatible_target_kind(incompatible, [provenance]),
        [] if !invalid_owner.is_empty() => intent
            .request
            .into_invalid_owner_reference(invalid_owner, [provenance]),
        [] => intent.request.into_missing_target([provenance]),
    }
    .map_err(EdtGraphError::ReferenceRequest)?;
    Ok(TerminalIntent {
        request,
        path: intent.path,
    })
}

fn request_diagnostic(
    graph: &SemanticGraph,
    request: &SemanticReferenceRequest,
) -> SemanticDiagnostic {
    let (code, kind, message) = match request.outcome() {
        SemanticReferenceRequestOutcome::MissingTarget => (
            SemanticDiagnosticCode::ReferenceUnresolved,
            SemanticDiagnosticKind::UnresolvedTarget,
            "XDTO or service reference target could not be resolved",
        ),
        SemanticReferenceRequestOutcome::AmbiguousTarget => (
            SemanticDiagnosticCode::ReferenceAmbiguous,
            SemanticDiagnosticKind::AmbiguousTarget,
            "XDTO or service reference target is ambiguous",
        ),
        SemanticReferenceRequestOutcome::IncompatibleTargetKind => (
            SemanticDiagnosticCode::ReferenceIncompatibleKind,
            SemanticDiagnosticKind::IncompatibleTargetKind,
            "XDTO or service reference target has incompatible kind",
        ),
        SemanticReferenceRequestOutcome::InvalidOwnerReference => (
            SemanticDiagnosticCode::ReferenceInvalidOwner,
            SemanticDiagnosticKind::InvalidOwnerReference,
            "XDTO or service reference target has invalid owner",
        ),
        _ => unreachable!("only failed terminal requests create diagnostics"),
    };
    let mut diagnostic = SemanticDiagnostic::new(
        code,
        SemanticDiagnosticSeverity::Error,
        kind,
        message,
        request.reference().clone(),
    )
    .with_source_node(request.source_node().clone())
    .with_expected_kinds(request.expected_kinds().to_vec())
    .with_candidates(request.candidates().to_vec())
    .with_provenance(request.provenance().to_vec());
    if request.outcome() == SemanticReferenceRequestOutcome::IncompatibleTargetKind
        && let Some(actual) = request
            .candidates()
            .first()
            .and_then(|candidate| graph.node(candidate))
            .map(GraphNode::kind)
    {
        diagnostic = diagnostic.with_actual_kind(actual);
    }
    diagnostic
}

fn insert_resolution_edge(
    graph: &mut SemanticGraph,
    source: EntityId,
    target: EntityId,
    kind: EdgeKind,
    path: &Path,
    role: &str,
    reference: &SemanticReference,
) -> Result<(), EdtGraphError> {
    let edge_kind = format!("{kind:?}");
    let provenance = request_provenance(
        path,
        role,
        &source,
        &edge_kind,
        reference,
        ResolutionState::Resolved,
    )?;
    graph
        .insert_edge(GraphEdge::new_with_provenance(
            source,
            target,
            kind,
            vec![provenance],
        ))
        .map_err(EdtGraphError::Graph)?;
    Ok(())
}

fn common_module_id(service: &EntityId) -> Result<EntityId, EdtGraphError> {
    EntityId::new(format!("{}:common_module", service.as_str()))
        .map_err(|_| EdtGraphError::InvalidIdentifier)
}

fn declaration_provenance(
    path: &Path,
    role: &str,
    node: &EntityId,
    value: &str,
) -> Result<Provenance, EdtGraphError> {
    provenance(
        path,
        role,
        node,
        value,
        FactOrigin::Declared,
        ResolutionState::NotApplicable,
    )
}

fn request_provenance(
    path: &Path,
    stage: &str,
    source: &EntityId,
    role: &str,
    reference: &SemanticReference,
    resolution: ResolutionState,
) -> Result<Provenance, EdtGraphError> {
    provenance(
        path,
        stage,
        source,
        &format!("{role}:{reference:?}"),
        if stage == "collection" {
            FactOrigin::Declared
        } else {
            FactOrigin::Resolved
        },
        resolution,
    )
}

fn provenance(
    path: &Path,
    role: &str,
    node: &EntityId,
    value: &str,
    origin: FactOrigin,
    resolution: ResolutionState,
) -> Result<Provenance, EdtGraphError> {
    let path = path.to_string_lossy();
    let source = EntityId::new(format!(
        "edt_xdto_service;path#{}:{};role#{}:{};node#{}:{};value#{}:{}",
        path.len(),
        path,
        role.len(),
        role,
        node.as_str().len(),
        node.as_str(),
        value.len(),
        value,
    ))
    .map_err(|_| EdtGraphError::InvalidIdentifier)?;
    Ok(Provenance::new(
        Some(source),
        ProducerId::new(PRODUCER),
        origin,
        Confidence::Exact,
        resolution,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(value: &str) -> EntityId {
        EntityId::new(value).expect("test identifier must be valid")
    }

    fn name(value: &str) -> EntityName {
        EntityName::new(value).expect("test name must be valid")
    }

    fn graph_with_owners(owners: &[EntityId]) -> SemanticGraph {
        let mut graph = SemanticGraph::new();
        graph.insert_node(GraphNode::new(
            id("source"),
            name("Source"),
            NodeKind::WebServiceOperation,
        ));
        for owner in owners {
            graph.insert_node(GraphNode::new(
                owner.clone(),
                name(owner.as_str()),
                NodeKind::Metadata(MetadataKind::XdtoPackage),
            ));
        }
        graph
    }

    fn insert_child(graph: &mut SemanticGraph, owner: &EntityId, child: &str, kind: NodeKind) {
        graph.insert_node(GraphNode::new(id(child), name("Result"), kind));
        graph
            .insert_edge(GraphEdge::new(owner.clone(), id(child), EdgeKind::Contains))
            .expect("test ownership must be insertable");
    }

    fn assert_terminal_outcome(
        graph: &SemanticGraph,
        owners: &[EntityId],
        outcome: SemanticReferenceRequestOutcome,
        candidates: &[&str],
        code: SemanticDiagnosticCode,
        kind: SemanticDiagnosticKind,
    ) {
        let intent = type_intent(
            id("source"),
            name("Result"),
            owners,
            Path::new("fixture/Service.mdo"),
        )
        .expect("XDTO type intent must be collected");
        let terminal = resolve_intent(graph, intent).expect("XDTO type intent must terminate");
        assert_eq!(terminal.request.outcome(), outcome);
        assert_eq!(
            terminal
                .request
                .candidates()
                .iter()
                .map(EntityId::as_str)
                .collect::<Vec<_>>(),
            candidates
        );
        assert_eq!(terminal.request.provenance().len(), 2);
        let diagnostic = request_diagnostic(graph, &terminal.request);
        assert_eq!(diagnostic.code(), code);
        assert_eq!(diagnostic.kind(), kind);
        assert_eq!(diagnostic.candidates(), terminal.request.candidates());
    }

    #[test]
    fn xdto_type_resolution_covers_every_failed_terminal_outcome() {
        let owner = id("owner");
        let missing = graph_with_owners(std::slice::from_ref(&owner));
        assert_terminal_outcome(
            &missing,
            std::slice::from_ref(&owner),
            SemanticReferenceRequestOutcome::MissingTarget,
            &[],
            SemanticDiagnosticCode::ReferenceUnresolved,
            SemanticDiagnosticKind::UnresolvedTarget,
        );

        let mut incompatible = graph_with_owners(std::slice::from_ref(&owner));
        insert_child(
            &mut incompatible,
            &owner,
            "incompatible",
            NodeKind::Function,
        );
        assert_terminal_outcome(
            &incompatible,
            std::slice::from_ref(&owner),
            SemanticReferenceRequestOutcome::IncompatibleTargetKind,
            &["incompatible"],
            SemanticDiagnosticCode::ReferenceIncompatibleKind,
            SemanticDiagnosticKind::IncompatibleTargetKind,
        );

        let foreign_owner = id("foreign-owner");
        let mut invalid_owner = graph_with_owners(&[owner.clone(), foreign_owner.clone()]);
        insert_child(
            &mut invalid_owner,
            &foreign_owner,
            "invalid-owner",
            NodeKind::XdtoType,
        );
        assert_terminal_outcome(
            &invalid_owner,
            std::slice::from_ref(&owner),
            SemanticReferenceRequestOutcome::InvalidOwnerReference,
            &["invalid-owner"],
            SemanticDiagnosticCode::ReferenceInvalidOwner,
            SemanticDiagnosticKind::InvalidOwnerReference,
        );

        let first_owner = id("first-owner");
        let second_owner = id("second-owner");
        let owners = [first_owner.clone(), second_owner.clone()];
        let mut ambiguous = graph_with_owners(&owners);
        insert_child(
            &mut ambiguous,
            &first_owner,
            "first-candidate",
            NodeKind::XdtoType,
        );
        insert_child(
            &mut ambiguous,
            &second_owner,
            "second-candidate",
            NodeKind::XdtoType,
        );
        assert_terminal_outcome(
            &ambiguous,
            &owners,
            SemanticReferenceRequestOutcome::AmbiguousTarget,
            &["first-candidate", "second-candidate"],
            SemanticDiagnosticCode::ReferenceAmbiguous,
            SemanticDiagnosticKind::AmbiguousTarget,
        );
    }
}
