use oneagent_analysis::refactoring::{
    BslModuleRole, NeverCancelledRefactoring, RefactoringEvaluation, RefactoringFamily,
    RefactoringPlanner, RefactoringPlannerInput, RefactoringRequest, SourceEvidenceSet,
    SourceFormat, SourceOccurrence, SourceOccurrenceKind, SourceOccurrenceResolution,
    WorkspacePublicationId,
};
use oneagent_common::{EntityId, EntityName, sha256_hex};
use oneagent_designer_xml::{
    DesignerXmlBuildScope, DesignerXmlSemanticCoverageReport, DesignerXmlSemanticGraphBuilder,
    FileSystemDesignerXmlSemanticGraphBuilder,
};
use oneagent_edt::{EdtSemanticGraphBuilder, FileSystemEdtSemanticGraphBuilder};
use oneagent_graph::{
    Confidence, GraphEdge, GraphNode, GraphNodePayload, NodeId, NodeKind, SemanticGraph,
};
use oneagent_metadata::{CommonMetadataPayload, MetadataKind, MetadataPayload};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

const COMMON_MODULE_ID: &str = "dc24575c-a787-411d-93bd-494271291d73";
const CONFIGURATION_ID: &str = "408a41e7-907a-4fb3-8999-83d1e8b6e093";
const CONFIGURATION_HASH: &str = "b7eed83a154d0f68c858f10d991ee985fb6d7df878f7abb328c1e441d57a2bdd";
const COMMON_MODULE_HASH: &str = "cafbab22d5a4494797aaf15b097d5118b22f60bf16e7017e147ce6048d482e3e";

fn fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sprint14_conformance")
}

fn designer_fixture() -> PathBuf {
    fixture_root().join("designer")
}

fn edt_fixture() -> PathBuf {
    fixture_root().join("edt")
}

fn accepted_metadata_kind(kind: MetadataKind) -> bool {
    matches!(
        kind,
        MetadataKind::Configuration
            | MetadataKind::Subsystem
            | MetadataKind::Catalog
            | MetadataKind::Document
            | MetadataKind::Enumeration
            | MetadataKind::CommonModule
            | MetadataKind::Report
            | MetadataKind::DataProcessor
            | MetadataKind::InformationRegister
            | MetadataKind::AccumulationRegister
            | MetadataKind::AccountingRegister
            | MetadataKind::BusinessProcess
            | MetadataKind::Task
            | MetadataKind::Role
            | MetadataKind::CommonForm
            | MetadataKind::Command
            | MetadataKind::Template
            | MetadataKind::HttpService
            | MetadataKind::WebService
            | MetadataKind::XdtoPackage
            | MetadataKind::EventSubscription
    )
}

fn canonical_projection(source: &SemanticGraph) -> SemanticGraph {
    let mut included = source
        .nodes()
        .filter_map(|node| match node.kind() {
            NodeKind::Metadata(kind) if accepted_metadata_kind(kind) => Some(node.id().clone()),
            _ => None,
        })
        .collect::<BTreeSet<_>>();

    for child_kind in [NodeKind::Module, NodeKind::Procedure, NodeKind::Function] {
        for node in source.nodes().filter(|node| node.kind() == child_kind) {
            let owners = source.query().owners(&NodeId::new(node.id().as_str()));
            if owners.len() == 1 && included.contains(owners[0].id()) {
                included.insert(node.id().clone());
            }
        }
    }

    let mut projected = SemanticGraph::new();
    for node in source.nodes().filter(|node| included.contains(node.id())) {
        projected.insert_node(canonical_node(node));
    }
    for edge in source.edges().filter(|edge| {
        edge.kind() == oneagent_graph::EdgeKind::Contains
            && included.contains(edge.source())
            && included.contains(edge.target())
    }) {
        projected
            .insert_edge(GraphEdge::new(
                edge.source().clone(),
                edge.target().clone(),
                edge.kind(),
            ))
            .expect("canonical ownership endpoints must exist");
    }
    projected
}

fn canonical_node(node: &GraphNode) -> GraphNode {
    if let NodeKind::Metadata(kind) = node.kind() {
        let synonym = node
            .metadata_payload()
            .and_then(|payload| payload.common().synonym())
            .map(str::to_owned);
        GraphNode::new_with_payload(
            node.id().clone(),
            node.name().clone(),
            node.kind(),
            GraphNodePayload::Metadata(MetadataPayload::new(
                CommonMetadataPayload::new(synonym),
                None,
            )),
        )
        .unwrap_or_else(|_| panic!("canonical payload must be valid for {kind}"))
    } else {
        GraphNode::new(node.id().clone(), node.name().clone(), node.kind())
    }
}

fn assert_complete_consumers(graph: &SemanticGraph) {
    assert!(
        graph.node_count() > 3,
        "projection must be non-empty and material"
    );
    assert_eq!(graph.edge_count() + 1, graph.node_count());
    for node in graph.nodes() {
        assert_eq!(
            graph
                .resolution_index()
                .resolve_entity_id_of_kind(node.id(), node.kind())
                .expect("complete Resolution index must find every projected node")
                .id(),
            node.id()
        );
        assert_eq!(
            graph
                .query()
                .node(&NodeId::new(node.id().as_str()))
                .expect("complete Query index must find every projected node")
                .kind(),
            node.kind()
        );
        if node.kind() != NodeKind::Metadata(MetadataKind::Configuration) {
            assert!(
                graph
                    .query()
                    .owner(&NodeId::new(node.id().as_str()))
                    .is_some(),
                "every projected child must have exactly one owner"
            );
        }
    }
    assert!(graph.validate().is_valid());
    assert_eq!(graph.validate().error_count(), 0);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TerminalOutcome {
    Success,
    Failure,
}

#[test]
fn paired_first_slice_is_non_empty_and_equal() {
    let designer_result = FileSystemDesignerXmlSemanticGraphBuilder
        .build_graph(&designer_fixture(), DesignerXmlBuildScope::Partial);
    let edt_result = FileSystemEdtSemanticGraphBuilder.build_graph(&edt_fixture());
    let designer_outcome = terminal_outcome(&designer_result);
    let edt_outcome = terminal_outcome(&edt_result);
    assert_eq!(designer_outcome, edt_outcome);
    assert_eq!(designer_outcome, TerminalOutcome::Success);
    let designer = designer_result.expect("official selective Designer fixture must build");
    let edt = edt_result.expect("paired reduced EDT fixture must build");
    assert_designer_provenance(&designer);

    let repeated = FileSystemDesignerXmlSemanticGraphBuilder
        .build_graph(&designer_fixture(), DesignerXmlBuildScope::Partial)
        .expect("repeated official Designer fixture build must succeed");
    assert!(designer.diff(&repeated).is_empty());
    let designer_projection = canonical_projection(&designer);
    let edt_projection = canonical_projection(&edt);

    assert_complete_consumers(&designer_projection);
    assert_complete_consumers(&edt_projection);
    let projection_diff = designer_projection.diff(&edt_projection);
    assert!(
        projection_diff.is_empty(),
        "paired canonical projection differs: {projection_diff:#?}"
    );
    assert_eq!(designer_projection.report(), edt_projection.report());
    assert_eq!(
        designer_projection
            .query()
            .nodes_by_name_and_kind(
                &EntityName::new("DynamicSecurityOverridable").expect("name must be valid"),
                NodeKind::Metadata(MetadataKind::CommonModule),
            )
            .len(),
        1
    );

    let coverage = DesignerXmlSemanticCoverageReport::for_graph(&designer);
    assert!(coverage.designer_pipeline().is_consistent());
    assert!(coverage.observed().total_nodes() > 0);
    assert!(coverage.observed().total_edges() > 0);
    assert_eq!(coverage.graph_report(), &designer.report());
    assert!(coverage.validation().is_valid());
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CanonicalOccurrence {
    configuration_id: String,
    module_id: String,
    module_role: BslModuleRole,
    kind: SourceOccurrenceKind,
    token: String,
    mapped_target: Option<String>,
    resolution: SourceOccurrenceResolution,
}

fn canonical_source_projection(
    evidence: &oneagent_analysis::refactoring::SourceEvidenceSet,
) -> Vec<CanonicalOccurrence> {
    evidence
        .documents()
        .iter()
        .flat_map(|document| {
            document
                .occurrences()
                .iter()
                .map(|occurrence| CanonicalOccurrence {
                    configuration_id: document.id().configuration_id().as_str().to_owned(),
                    module_id: document.id().module_id().as_str().to_owned(),
                    module_role: document.module_role(),
                    kind: occurrence.kind(),
                    token: occurrence.token().to_owned(),
                    mapped_target: occurrence
                        .mapped_target_id()
                        .map(|id| id.as_str().to_owned()),
                    resolution: occurrence.resolution(),
                })
        })
        .collect()
}

fn production_planner_evaluation(
    graph: &SemanticGraph,
    evidence: &SourceEvidenceSet,
) -> RefactoringEvaluation {
    let target = evidence
        .documents()
        .iter()
        .flat_map(oneagent_analysis::refactoring::SourceDocument::occurrences)
        .find(|occurrence| {
            occurrence.kind() == SourceOccurrenceKind::Declaration
                && occurrence.token() == "FillSecurityCollection"
        })
        .and_then(SourceOccurrence::mapped_target_id)
        .expect("paired production declaration must map uniquely")
        .clone();
    let request = RefactoringRequest::new(
        RefactoringFamily::BslCallableRenameV1,
        WorkspacePublicationId::initial(),
        evidence.configuration_id().clone(),
        target,
        "FillSecurityCollectionRenamed",
    )
    .expect("production planner request must be valid");
    RefactoringPlanner
        .evaluate(
            RefactoringPlannerInput::new(
                WorkspacePublicationId::initial(),
                evidence.configuration_id(),
                graph,
                evidence,
            ),
            &request,
            &NeverCancelledRefactoring,
        )
        .expect("paired production evidence must produce a complete plan and preview")
}

fn assert_paired_production_planner(
    designer_graph: &SemanticGraph,
    designer_evidence: &SourceEvidenceSet,
    edt_graph: &SemanticGraph,
    edt_evidence: &SourceEvidenceSet,
) {
    let retained_designer_evidence = designer_evidence.clone();
    let retained_edt_evidence = edt_evidence.clone();
    let designer_plan = production_planner_evaluation(designer_graph, designer_evidence);
    let repeated_designer_plan = production_planner_evaluation(designer_graph, designer_evidence);
    let edt_plan = production_planner_evaluation(edt_graph, edt_evidence);

    assert_eq!(designer_plan, repeated_designer_plan);
    for evaluation in [&designer_plan, &edt_plan] {
        assert_eq!(evaluation.plan().operations().len(), 3);
        assert_eq!(evaluation.plan().summary().declaration_operations(), 1);
        assert_eq!(evaluation.plan().summary().local_call_operations(), 1);
        assert_eq!(evaluation.plan().summary().qualified_call_operations(), 1);
        assert_eq!(evaluation.plan().summary().omitted_operations(), 0);
        assert_eq!(evaluation.preview().entries().len(), 3);
    }
    assert_ne!(
        designer_plan.plan().id(),
        edt_plan.plan().id(),
        "different exact bytes, ranges, and content versions remain semantic preconditions"
    );
    assert_eq!(designer_evidence, &retained_designer_evidence);
    assert_eq!(edt_evidence, &retained_edt_evidence);
}

#[test]
fn paired_production_builders_publish_equal_canonical_occurrence_evidence() {
    let designer = FileSystemDesignerXmlSemanticGraphBuilder
        .build_graph_with_source_evidence(
            &fixture_root(),
            &designer_fixture(),
            DesignerXmlBuildScope::Partial,
        )
        .expect("paired Designer evidence must build");
    let edt = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_source_evidence(&fixture_root(), &edt_fixture())
        .expect("paired EDT evidence must build");
    let repeated = FileSystemDesignerXmlSemanticGraphBuilder
        .build_graph_with_source_evidence(
            &fixture_root(),
            &designer_fixture(),
            DesignerXmlBuildScope::Partial,
        )
        .expect("repeated Designer evidence must build");

    assert_eq!(designer.source_evidence(), repeated.source_evidence());
    let designer_document = &designer.source_evidence().documents()[0];
    let edt_document = &edt.source_evidence().documents()[0];
    assert_eq!(designer_document.format(), SourceFormat::DesignerXml);
    assert_eq!(edt_document.format(), SourceFormat::Edt);
    assert_ne!(designer_document.path(), edt_document.path());
    assert_ne!(designer_document.raw_content(), edt_document.raw_content());
    assert_ne!(
        designer_document.content_version(),
        edt_document.content_version()
    );
    assert_ne!(
        designer_document.occurrences()[0].range(),
        edt_document.occurrences()[0].range()
    );
    let designer_module_source = designer
        .graph()
        .node(designer_document.id().module_id())
        .expect("Designer module node must exist")
        .provenance()[0]
        .source()
        .expect("Designer module provenance must retain a source identity");
    assert!(
        designer_module_source
            .as_str()
            .contains(&sha256_hex(designer_document.raw_content())),
        "Designer Graph provenance digest must agree with retained exact bytes"
    );
    assert_eq!(
        canonical_source_projection(designer.source_evidence()),
        canonical_source_projection(edt.source_evidence())
    );
    assert_eq!(designer_document.occurrences().len(), 4);
    assert_eq!(
        designer_document
            .occurrences()
            .iter()
            .map(SourceOccurrence::kind)
            .collect::<Vec<_>>(),
        [
            SourceOccurrenceKind::Declaration,
            SourceOccurrenceKind::Declaration,
            SourceOccurrenceKind::LocalCall,
            SourceOccurrenceKind::QualifiedCall,
        ]
    );
    assert_eq!(
        designer
            .graph()
            .edges()
            .filter(|edge| edge.kind() == oneagent_graph::EdgeKind::Calls)
            .count(),
        0,
        "Task 4 must not add new Designer Graph facts"
    );
    assert_eq!(
        edt.graph()
            .edges()
            .filter(|edge| edge.kind() == oneagent_graph::EdgeKind::Calls)
            .count(),
        1
    );

    assert_paired_production_planner(
        designer.graph(),
        designer.source_evidence(),
        edt.graph(),
        edt.source_evidence(),
    );
}

fn terminal_outcome<T, E>(result: &Result<T, E>) -> TerminalOutcome {
    if result.is_ok() {
        TerminalOutcome::Success
    } else {
        TerminalOutcome::Failure
    }
}

fn assert_designer_provenance(graph: &SemanticGraph) {
    assert_eq!(graph.report().provenance().nodes_without_provenance(), 0);
    assert_eq!(graph.report().provenance().edges_without_provenance(), 0);
    for provenance in graph
        .nodes()
        .flat_map(GraphNode::provenance)
        .chain(graph.edges().flat_map(GraphEdge::provenance))
    {
        assert_eq!(provenance.confidence(), Confidence::Exact);
        assert!(
            provenance
                .source()
                .is_some_and(|source| source.as_str().contains("#sha256="))
        );
        assert!(
            provenance
                .producer()
                .as_str()
                .starts_with("oneagent.designer-xml.")
        );
    }

    let configuration_source = graph
        .node(&EntityId::new(CONFIGURATION_ID).expect("configuration id must be valid"))
        .expect("configuration must exist")
        .provenance()[0]
        .source()
        .expect("configuration source must exist")
        .as_str();
    assert!(configuration_source.contains(CONFIGURATION_HASH));
    let common_module_source = graph
        .node(&EntityId::new(COMMON_MODULE_ID).expect("Common module id must be valid"))
        .expect("Common module must exist")
        .provenance()[0]
        .source()
        .expect("Common module source must exist")
        .as_str();
    assert!(common_module_source.contains(COMMON_MODULE_HASH));
}

#[test]
fn controlled_synonym_change_produces_one_exact_difference() {
    let baseline = FileSystemDesignerXmlSemanticGraphBuilder
        .build_graph(&designer_fixture(), DesignerXmlBuildScope::Partial)
        .expect("baseline Designer fixture must build");
    let temporary = tempdir().expect("temporary directory must be created");
    copy_tree(&designer_fixture(), temporary.path());
    let common_module_path = temporary
        .path()
        .join("CommonModules/DynamicSecurityOverridable.xml");
    let raw = fs::read(&common_module_path).expect("Common module descriptor must be read");
    let source = String::from_utf8(raw).expect("Common module descriptor must be UTF-8");
    let changed = source.replacen(
        "Dynamic security overridable",
        "Dynamic security overridable changed",
        1,
    );
    assert_ne!(source, changed, "controlled change must alter the fixture");
    fs::write(&common_module_path, changed).expect("controlled fixture must be written");

    let changed = FileSystemDesignerXmlSemanticGraphBuilder
        .build_graph(temporary.path(), DesignerXmlBuildScope::Partial)
        .expect("controlled Designer fixture must build");
    let diff = canonical_projection(&baseline).diff(&canonical_projection(&changed));

    assert_eq!(diff.summary().nodes_modified(), 1);
    assert_eq!(diff.summary().total_changes(), 1);
    assert_eq!(diff.modified_nodes()[0].id().as_str(), COMMON_MODULE_ID);
    assert_eq!(
        diff.modified_nodes()[0].modified_aspects(),
        [oneagent_graph::NodeModifiedAspect::SemanticContent]
    );
}

#[test]
fn partial_and_invalid_terminal_outcomes_are_not_silently_successful() {
    let temporary = tempdir().expect("temporary directory must be created");
    copy_tree(&designer_fixture(), temporary.path());
    fs::remove_dir_all(temporary.path().join("CommonModules"))
        .expect("Common module subset must be removed");
    let partial = FileSystemDesignerXmlSemanticGraphBuilder
        .build_graph(temporary.path(), DesignerXmlBuildScope::Partial)
        .expect("explicit partial fixture must build without placeholders");
    assert!(
        partial
            .node(&EntityId::new(COMMON_MODULE_ID).expect("Common module id must be valid"))
            .is_none()
    );

    fs::write(temporary.path().join("Configuration.xml"), "<malformed>")
        .expect("accepted descriptor must be corrupted");
    assert!(
        FileSystemDesignerXmlSemanticGraphBuilder
            .build_graph(temporary.path(), DesignerXmlBuildScope::Partial)
            .is_err(),
        "malformed accepted input must return no graph"
    );
}

fn copy_tree(source: &Path, target: &Path) {
    for entry in fs::read_dir(source).expect("fixture directory must be readable") {
        let entry = entry.expect("fixture entry must be readable");
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());
        if entry
            .file_type()
            .expect("fixture type must be readable")
            .is_dir()
        {
            fs::create_dir_all(&target_path).expect("fixture directory must be copied");
            copy_tree(&source_path, &target_path);
        } else {
            fs::copy(&source_path, &target_path).expect("fixture file must be copied");
        }
    }
}
