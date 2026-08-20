use oneagent_common::{EntityId, EntityName};
use oneagent_metadata::{CommonMetadataPayload, MetadataKind, MetadataPayload};

use crate::incremental_index::NormalizedSemanticIndexChanges;
use crate::semantic_index::{AcceptedSemanticIndex, SemanticIndexState};
use crate::{EdgeKind, GraphEdge, GraphNode, GraphNodePayload, NodeId, NodeKind, SemanticGraph};

const CONFIGURATION_ID: &str = "408a41e7-907a-4fb3-8999-83d1e8b6e093";
const COMMON_MODULE_ID: &str = "dc24575c-a787-411d-93bd-494271291d73";
const MODULE_ID: &str = "dc24575c-a787-411d-93bd-494271291d73:module:common";
const PROCEDURE_ID: &str =
    "dc24575c-a787-411d-93bd-494271291d73:module:common:procedure:FillSecurityCollection";
const FUNCTION_ID: &str =
    "dc24575c-a787-411d-93bd-494271291d73:module:common:function:ReadSecurityCollection";

#[test]
fn designer_shaped_transitions_match_clean_complete_index_rebuilds() {
    let initial = designer_graph(None, MODULE_ID, false);
    let accepted = AcceptedSemanticIndex::rebuild(&initial);
    assert_eq!(accepted.state(), &SemanticIndexState::from_graph(&initial));

    let added = designer_graph(None, MODULE_ID, true);
    let accepted = transition_and_assert(&accepted, &added);

    let modified = designer_graph(Some("Dynamic security overridable"), MODULE_ID, true);
    let accepted = transition_and_assert(&accepted, &modified);

    let reparented = designer_graph(Some("Dynamic security overridable"), COMMON_MODULE_ID, true);
    let accepted = transition_and_assert(&accepted, &reparented);
    assert_eq!(
        accepted
            .query()
            .owner(&NodeId::new(PROCEDURE_ID))
            .expect("reparented Procedure must have one owner")
            .id()
            .as_str(),
        COMMON_MODULE_ID
    );

    let removed = designer_graph(
        Some("Dynamic security overridable"),
        COMMON_MODULE_ID,
        false,
    );
    let accepted = transition_and_assert(&accepted, &removed);
    assert!(accepted.query().node(&NodeId::new(FUNCTION_ID)).is_none());
}

fn transition_and_assert<'current>(
    previous: &AcceptedSemanticIndex<'_>,
    current: &'current SemanticGraph,
) -> AcceptedSemanticIndex<'current> {
    let changes = NormalizedSemanticIndexChanges::between(previous.graph(), current)
        .expect("Designer-shaped transition must normalize");
    let accepted = previous
        .transition(current, &changes)
        .expect("Designer-shaped transition must publish");
    assert_eq!(accepted.state(), &SemanticIndexState::from_graph(current));
    assert_eq!(
        accepted.query().nodes().len(),
        current.query().nodes().len()
    );
    assert_eq!(
        accepted.query().edges().len(),
        current.query().edges().len()
    );
    for node in current.nodes() {
        assert_eq!(
            accepted
                .resolution_index()
                .resolve_entity_id_of_kind(node.id(), node.kind())
                .expect("incremental Resolution index must match clean rebuild")
                .id(),
            node.id()
        );
    }
    accepted
}

fn designer_graph(
    synonym: Option<&str>,
    procedure_owner: &str,
    include_function: bool,
) -> SemanticGraph {
    let mut graph = SemanticGraph::new();
    insert_metadata(
        &mut graph,
        CONFIGURATION_ID,
        "DNSWorldEdition",
        MetadataKind::Configuration,
        Some("DNS: WE"),
    );
    insert_metadata(
        &mut graph,
        COMMON_MODULE_ID,
        "DynamicSecurityOverridable",
        MetadataKind::CommonModule,
        synonym,
    );
    insert_node(&mut graph, MODULE_ID, "CommonModule", NodeKind::Module);
    insert_node(
        &mut graph,
        PROCEDURE_ID,
        "FillSecurityCollection",
        NodeKind::Procedure,
    );
    insert_contains(&mut graph, CONFIGURATION_ID, COMMON_MODULE_ID);
    insert_contains(&mut graph, COMMON_MODULE_ID, MODULE_ID);
    insert_contains(&mut graph, procedure_owner, PROCEDURE_ID);

    if include_function {
        insert_node(
            &mut graph,
            FUNCTION_ID,
            "ReadSecurityCollection",
            NodeKind::Function,
        );
        insert_contains(&mut graph, MODULE_ID, FUNCTION_ID);
    }
    graph
}

fn insert_metadata(
    graph: &mut SemanticGraph,
    id: &str,
    name: &str,
    kind: MetadataKind,
    synonym: Option<&str>,
) {
    graph.insert_node(
        GraphNode::new_with_payload(
            entity_id(id),
            entity_name(name),
            NodeKind::Metadata(kind),
            GraphNodePayload::Metadata(MetadataPayload::new(
                CommonMetadataPayload::new(synonym.map(str::to_owned)),
                None,
            )),
        )
        .expect("Designer-shaped metadata payload must be valid"),
    );
}

fn insert_node(graph: &mut SemanticGraph, id: &str, name: &str, kind: NodeKind) {
    graph.insert_node(GraphNode::new(entity_id(id), entity_name(name), kind));
}

fn insert_contains(graph: &mut SemanticGraph, owner: &str, child: &str) {
    graph
        .insert_edge(GraphEdge::new(
            entity_id(owner),
            entity_id(child),
            EdgeKind::Contains,
        ))
        .expect("Designer-shaped ownership endpoints must exist");
}

fn entity_id(value: &str) -> EntityId {
    EntityId::new(value).expect("test identity must be valid")
}

fn entity_name(value: &str) -> EntityName {
    EntityName::new(value).expect("test name must be valid")
}
