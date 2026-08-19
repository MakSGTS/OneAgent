use oneagent_common::{EntityId, EntityName};
use oneagent_edt::{
    EdtSemanticGraphBuildResult, EdtSemanticGraphBuilder, FileSystemEdtSemanticGraphBuilder,
};
use oneagent_graph::{
    EdgeKind, FactOrigin, GraphNode, ImpactNodeStatus, ImpactReasonKind, NodeId, NodeKind,
    ResolutionState, SemanticDiagnosticCode, SemanticGraphEdgeFilter,
    SemanticGraphTraversalDirection, SemanticGraphTraversalOptions, SemanticImpactAnalyzer,
    SemanticImpactOptions,
};
use oneagent_metadata::MetadataKind;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

const CONFIGURATION: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<mdclass:Configuration xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass"
    uuid="10000000-0000-0000-0000-000000000000">
    <name>FormNavigation</name>
</mdclass:Configuration>
"#;

const PRODUCTS_ID: &str = "20000000-0000-0000-0000-000000000000";
const SHARED_FORM_ID: &str = "21000000-0000-0000-0000-000000000000";
const GLOBAL_COMMAND_ID: &str = "30000000-0000-0000-0000-000000000000";
const WORKSPACE_ID: &str = "40000000-0000-0000-0000-000000000000";
const EXECUTE_ID: &str = "30000000-0000-0000-0000-000000000000:command_module:procedure:Execute";

const PRODUCTS: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<mdclass:Catalog xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass"
    uuid="20000000-0000-0000-0000-000000000000">
    <name>Products</name>
    <forms uuid="21000000-0000-0000-0000-000000000000"><name>Shared</name></forms>
    <forms uuid="22000000-0000-0000-0000-000000000000"><name>Duplicate</name></forms>
    <forms uuid="23000000-0000-0000-0000-000000000000"><name>Duplicate</name></forms>
</mdclass:Catalog>
"#;

const PRODUCTS_REORDERED: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<mdclass:Catalog xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass"
    uuid="20000000-0000-0000-0000-000000000000">
    <name>Products</name>
    <forms uuid="23000000-0000-0000-0000-000000000000"><name>Duplicate</name></forms>
    <forms uuid="22000000-0000-0000-0000-000000000000"><name>Duplicate</name></forms>
    <forms uuid="21000000-0000-0000-0000-000000000000"><name>Shared</name></forms>
</mdclass:Catalog>
"#;

const COMMAND: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<mdclass:CommonCommand xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass"
    uuid="30000000-0000-0000-0000-000000000000">
    <name>GlobalOpen</name>
</mdclass:CommonCommand>
"#;

const COMMAND_MODULE: &str = r#"Procedure Execute()
    OpenForm("Catalog.Products.Form.Shared");
    OpenForm("Catalog.Products.Form.Shared");
    OpenForm("CommonForm.Workspace");
    OpenForm("Catalog.Absent.Form.Shared");
    OpenForm("Catalog.Products.Form.Absent");
    OpenForm("Catalog.Products.Form.Duplicate");
    OpenForm("Catalog.Duplicate.Form.Shared");
    OpenForm("Catalog.TypeCollision.Form.Shared");
    OpenForm("Catalog.Products");
    OpenForm(TargetName);
    OpenForm("Catalog.Products.Form");
EndProcedure

Function WrongCallable()
    OpenForm("CommonForm.Workspace");
EndFunction
"#;

fn write(path: &Path, content: &str) {
    fs::create_dir_all(path.parent().expect("parent directory must exist"))
        .expect("fixture directory must be created");
    fs::write(path, content).expect("fixture file must be created");
}

fn descriptor(root_tag: &str, id: &str, name: &str, synonym: Option<&str>) -> String {
    let synonym = synonym
        .map(|value| format!("    <synonym><key>en</key><value>{value}</value></synonym>\n"))
        .unwrap_or_default();
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<mdclass:{root_tag} xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass" uuid="{id}">
    <name>{name}</name>
{synonym}</mdclass:{root_tag}>
"#
    )
}

fn create_project() -> tempfile::TempDir {
    let root = tempdir().expect("temporary project must be created");
    write(
        &root.path().join("src/Configuration/Configuration.mdo"),
        CONFIGURATION,
    );
    write(
        &root.path().join("src/Catalogs/Products/Products.mdo"),
        PRODUCTS,
    );
    for (directory, id) in [
        ("DuplicateOne", "24000000-0000-0000-0000-000000000000"),
        ("DuplicateTwo", "25000000-0000-0000-0000-000000000000"),
    ] {
        write(
            &root
                .path()
                .join(format!("src/Catalogs/{directory}/{directory}.mdo")),
            &descriptor("Catalog", id, "Duplicate", None),
        );
    }
    write(
        &root
            .path()
            .join("src/Documents/TypeCollision/TypeCollision.mdo"),
        &descriptor(
            "Document",
            "26000000-0000-0000-0000-000000000000",
            "TypeCollision",
            None,
        ),
    );
    write(
        &root
            .path()
            .join("src/CommonCommands/GlobalOpen/GlobalOpen.mdo"),
        COMMAND,
    );
    write(
        &root
            .path()
            .join("src/CommonCommands/GlobalOpen/CommandModule.bsl"),
        COMMAND_MODULE,
    );
    write(
        &root.path().join("src/CommonForms/Workspace/Workspace.mdo"),
        &descriptor("CommonForm", WORKSPACE_ID, "Workspace", Some("Workspace")),
    );
    root
}

fn node_id(value: &str) -> NodeId {
    NodeId::new(value)
}

fn entity_id(value: &str) -> EntityId {
    EntityId::new(value).expect("test identifier must be valid")
}

fn assert_navigation_statistics_and_diagnostics(result: &EdtSemanticGraphBuildResult) {
    let statistics = *result.reference_statistics();
    assert_eq!(statistics.total(), 24);
    assert_eq!(statistics.resolved(), 3);
    assert_eq!(statistics.unresolved(), 14);
    assert_eq!(statistics.ambiguous(), 2);
    assert_eq!(statistics.incompatible_target_kind(), 1);
    assert_eq!(statistics.unsupported_prefix(), 4);
    assert_eq!(statistics.outcome_total(), statistics.total());
    assert_eq!(result.report().resolution(), result.reference_statistics());

    let navigation_diagnostics = result
        .diagnostics()
        .iter()
        .filter(|diagnostic| {
            diagnostic
                .provenance()
                .iter()
                .any(|evidence| evidence.producer().as_str() == "oneagent.edt.form-navigation")
        })
        .collect::<Vec<_>>();
    assert_eq!(navigation_diagnostics.len(), 9);
    for (code, expected) in [
        (SemanticDiagnosticCode::ReferenceUnresolved, 2),
        (SemanticDiagnosticCode::ReferenceAmbiguous, 2),
        (SemanticDiagnosticCode::ReferenceIncompatibleKind, 1),
    ] {
        assert_eq!(
            navigation_diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.code() == code)
                .count(),
            expected
        );
    }
    assert!(navigation_diagnostics.iter().all(|diagnostic| {
        diagnostic.source_node().is_some() && !diagnostic.provenance().is_empty()
    }));
}

#[test]
fn form_navigation_emits_only_unique_resolved_opens_with_stable_evidence() {
    let root = create_project();
    let first = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(root.path())
        .expect("Form navigation project must build");
    let repeated = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(root.path())
        .expect("repeated Form navigation build must succeed");
    let query = first.graph().query();
    let execute = node_id(EXECUTE_ID);
    let opens = query.outgoing_edges_by_kind(&execute, EdgeKind::Opens);

    assert_eq!(opens.len(), 2);
    let shared = opens
        .iter()
        .find(|edge| edge.target().as_str() == SHARED_FORM_ID)
        .expect("subordinate Form must be opened");
    let workspace = opens
        .iter()
        .find(|edge| edge.target().as_str() == WORKSPACE_ID)
        .expect("CommonForm must be opened");
    assert_eq!(shared.provenance().len(), 2);
    assert_eq!(workspace.provenance().len(), 1);
    for evidence in shared.provenance().iter().chain(workspace.provenance()) {
        assert_eq!(evidence.origin(), FactOrigin::Resolved);
        assert_eq!(evidence.resolution(), ResolutionState::Resolved);
        assert_eq!(evidence.producer().as_str(), "oneagent.edt.form-navigation");
        let source = evidence
            .source()
            .expect("OpenForm evidence must retain exact source context")
            .as_str();
        assert!(source.contains("command#36:30000000-0000-0000-0000-000000000000"));
        assert!(source.contains("procedure_name#7:Execute"));
        assert!(source.contains("line#"));
        assert!(source.contains("target#"));
    }

    assert!(
        query
            .outgoing_edges_by_kind(&execute, EdgeKind::References)
            .is_empty()
    );
    assert!(
        query
            .outgoing_edges_by_kind(&execute, EdgeKind::DependsOn)
            .is_empty()
    );
    assert!(
        query
            .outgoing_edges_by_kind(&execute, EdgeKind::Calls)
            .is_empty()
    );

    let dependencies = query.direct_dependencies_by_kind(&execute, EdgeKind::Opens);
    assert_eq!(dependencies.len(), 2);
    assert_eq!(
        query
            .direct_usages_by_kind(&node_id(WORKSPACE_ID), EdgeKind::Opens)
            .len(),
        1
    );
    let traversal = query.traverse(
        &execute,
        &SemanticGraphTraversalOptions::new(SemanticGraphTraversalDirection::Downstream, 1)
            .with_edge_filter(SemanticGraphEdgeFilter::only(EdgeKind::Opens)),
    );
    assert_eq!(traversal.len(), 2);
    assert!(
        traversal
            .iter()
            .all(|node| node.depth() == 1 && node.via_edge().is_some())
    );

    assert_navigation_statistics_and_diagnostics(&first);

    assert!(first.validate().is_valid());
    assert!(first.graph().diff(repeated.graph()).is_empty());
    assert!(first.diff(&repeated).is_empty());

    write(
        &root.path().join("src/Catalogs/Products/Products.mdo"),
        PRODUCTS_REORDERED,
    );
    let reordered = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(root.path())
        .expect("reordered metadata observations must build");
    assert!(first.graph().diff(reordered.graph()).is_empty());
    assert!(first.diff(&reordered).is_empty());
}

#[test]
fn form_navigation_diff_observes_a_missing_target_becoming_resolved() {
    let root = create_project();
    let before = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(root.path())
        .expect("initial Form navigation graph must build");
    write(
        &root.path().join("src/Catalogs/Absent/Absent.mdo"),
        r#"<?xml version="1.0" encoding="UTF-8"?>
<mdclass:Catalog xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass" uuid="27000000-0000-0000-0000-000000000000">
    <name>Absent</name>
    <forms uuid="28000000-0000-0000-0000-000000000000"><name>Shared</name></forms>
</mdclass:Catalog>
"#,
    );
    let after = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(root.path())
        .expect("resolved Form navigation graph must build");
    let diff = before.diff(&after);

    assert!(diff.graph().added_edges().iter().any(|edge| {
        edge.edge_kind() == EdgeKind::Opens
            && edge.source().as_str() == EXECUTE_ID
            && edge.target().as_str() == "28000000-0000-0000-0000-000000000000"
    }));
    assert_eq!(
        before.reference_statistics().unresolved(),
        after.reference_statistics().unresolved() + 1
    );
    assert!(before.validate().is_valid());
    assert!(after.validate().is_valid());
}

#[test]
fn changed_common_form_propagates_impact_through_opens() {
    let root = create_project();
    let result = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(root.path())
        .expect("Form navigation graph must build");
    let previous = result.graph().clone();
    let mut current = previous.clone();
    let target = previous
        .node(&entity_id(WORKSPACE_ID))
        .expect("CommonForm target must exist");
    current.insert_node(GraphNode::new_with_provenance(
        target.id().clone(),
        EntityName::new("WorkspaceRenamed").expect("renamed target must be valid"),
        NodeKind::Metadata(MetadataKind::CommonForm),
        target.provenance().to_vec(),
    ));
    let diff = previous.diff(&current);
    let impact =
        SemanticImpactAnalyzer::analyze(&previous, &current, &diff, &SemanticImpactOptions::new(1))
            .expect("OpenForm impact analysis must succeed");
    let usage = impact
        .affected_nodes()
        .iter()
        .find(|node| node.node_id() == &node_id(EXECUTE_ID))
        .expect("opening Procedure must be affected by a changed CommonForm");

    assert_eq!(usage.status(), ImpactNodeStatus::TransitivelyAffected);
    assert!(usage.reasons().iter().any(|reason| {
        reason.kind() == ImpactReasonKind::DependencyPropagation
            && reason.edge_kind() == Some(EdgeKind::Opens)
    }));
}

#[test]
fn common_command_remains_the_owner_of_the_navigation_module() {
    let root = create_project();
    let result = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(root.path())
        .expect("Form navigation project must build");
    let module = node_id(&format!("{GLOBAL_COMMAND_ID}:command_module"));
    let owner = result
        .graph()
        .query()
        .owner(&module)
        .expect("Command module owner must exist");

    assert_eq!(owner.id().as_str(), GLOBAL_COMMAND_ID);
    assert_eq!(owner.kind(), NodeKind::Metadata(MetadataKind::Command));
    assert_eq!(
        result
            .graph()
            .query()
            .owner(&node_id(EXECUTE_ID))
            .expect("Procedure module owner must exist")
            .id()
            .as_str(),
        module.as_str()
    );
    assert_eq!(
        result
            .graph()
            .query()
            .node(&node_id(PRODUCTS_ID))
            .expect("Catalog must remain present")
            .kind(),
        NodeKind::Metadata(MetadataKind::Catalog)
    );
}
