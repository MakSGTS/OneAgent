use oneagent_common::{EntityId, EntityName};
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
