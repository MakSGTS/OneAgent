use oneagent_edt::{
    EdtSemanticGraphBuildResult, EdtSemanticGraphBuilder, FileSystemEdtSemanticGraphBuilder,
};
use oneagent_graph::{
    EdgeKind, FactOrigin, NodeId, NodeKind, ResolutionState, SemanticCoverageCapabilityId,
    SemanticCoverageGapPriority, SemanticCoverageStatus,
};
use oneagent_metadata::MetadataKind;
use std::fs;
use std::path::{Path, PathBuf};

fn ownership_fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/ownership_project")
}

fn node_id(value: &str) -> NodeId {
    NodeId::new(value)
}

fn write_order_fixture(root: &Path, tabular_section_first: bool) {
    let configuration_directory = root.join("src/Configuration");
    let document_directory = root.join("src/Documents/Sales");
    fs::create_dir_all(&configuration_directory).expect("configuration directory must be created");
    fs::create_dir_all(&document_directory).expect("Document directory must be created");
    fs::write(
        configuration_directory.join("Configuration.mdo"),
        include_str!("fixtures/ownership_project/src/Configuration/Configuration.mdo"),
    )
    .expect("configuration descriptor must be written");

    let top_level_attribute = r#"  <attributes uuid="21000000-0000-0000-0000-000000000000">
    <name>Company</name>
  </attributes>"#;
    let tabular_section = r#"  <tabularSections uuid="22000000-0000-0000-0000-000000000000">
    <name>Products</name>
    <attributes uuid="23000000-0000-0000-0000-000000000000">
      <name>Product</name>
      <type><types>DocumentRef.Sales</types></type>
    </attributes>
  </tabularSections>"#;
    let (first, second) = if tabular_section_first {
        (tabular_section, top_level_attribute)
    } else {
        (top_level_attribute, tabular_section)
    };
    fs::write(
        document_directory.join("Sales.mdo"),
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<mdclass:Document xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass" uuid="20000000-0000-0000-0000-000000000000">
  <name>Sales</name>
{first}
{second}
</mdclass:Document>
"#
        ),
    )
    .expect("Document descriptor must be written");
}

fn assert_immediate_ownership(result: &EdtSemanticGraphBuildResult) {
    let graph = result.graph();
    let query = graph.query();
    let document_id = node_id("20000000-0000-0000-0000-000000000000");
    let top_level_attribute_id = node_id("21000000-0000-0000-0000-000000000000");
    let tabular_section_id = node_id("22000000-0000-0000-0000-000000000000");
    let nested_attribute_id = node_id("23000000-0000-0000-0000-000000000000");

    let document = query
        .node(&document_id)
        .expect("Document metadata node must exist");
    let tabular_section = query
        .node(&tabular_section_id)
        .expect("TabularSection node must exist");
    let nested_attribute = query
        .node(&nested_attribute_id)
        .expect("nested Attribute node must exist");
    let nested_owners = query.owners(&nested_attribute_id);
    let tabular_children = query.children_by_kind(&tabular_section_id, NodeKind::Attribute);
    let nested_ownership_edge = query
        .owner_edges(&nested_attribute_id)
        .into_iter()
        .next()
        .expect("nested Attribute ownership edge must exist");

    assert!(result.diagnostics().is_empty());
    assert_eq!(result.reference_statistics().total(), 2);
    assert_eq!(result.reference_statistics().resolved(), 2);
    assert_eq!(document.kind(), NodeKind::Metadata(MetadataKind::Document));
    assert_eq!(tabular_section.kind(), NodeKind::TabularSection);
    assert_eq!(nested_attribute.kind(), NodeKind::Attribute);
    assert_eq!(query.owner(&tabular_section_id), Some(document));
    assert_eq!(query.owner(&top_level_attribute_id), Some(document));
    assert_eq!(nested_owners, vec![tabular_section]);
    assert!(tabular_children.contains(&nested_attribute));
    assert!(
        query
            .children_by_kind(&document_id, NodeKind::Attribute)
            .iter()
            .all(|child| child.id() != nested_attribute.id())
    );
    assert_eq!(nested_ownership_edge.source(), tabular_section.id());
    assert_eq!(nested_ownership_edge.target(), nested_attribute.id());
    assert_eq!(nested_ownership_edge.kind(), EdgeKind::Contains);

    let nested_references =
        query.outgoing_edges_by_kind(&nested_attribute_id, EdgeKind::References);
    let nested_dependencies =
        query.outgoing_edges_by_kind(&nested_attribute_id, EdgeKind::DependsOn);
    assert_eq!(nested_references.len(), 1);
    assert_eq!(nested_references[0].target(), document.id());
    assert_eq!(nested_references[0].provenance().len(), 1);
    assert_eq!(
        nested_references[0].provenance()[0].origin(),
        FactOrigin::Resolved
    );
    assert_eq!(
        nested_references[0].provenance()[0].resolution(),
        ResolutionState::Resolved
    );
    assert_eq!(nested_dependencies.len(), 1);
    assert_eq!(nested_dependencies[0].target(), document.id());
    assert_eq!(nested_dependencies[0].provenance().len(), 1);
}

fn assert_ownership_provenance(result: &EdtSemanticGraphBuildResult) {
    let graph = result.graph();
    let query = graph.query();
    let nested_attribute_id = node_id("23000000-0000-0000-0000-000000000000");
    let nested_attribute = query
        .node(&nested_attribute_id)
        .expect("nested Attribute node must exist");
    let nested_ownership_edge = query
        .owner_edges(&nested_attribute_id)
        .into_iter()
        .next()
        .expect("nested Attribute ownership edge must exist");

    assert_eq!(nested_attribute.provenance().len(), 1);
    assert_eq!(
        nested_attribute.provenance()[0].origin(),
        FactOrigin::Declared
    );
    assert_eq!(
        nested_attribute.provenance()[0].resolution(),
        ResolutionState::NotApplicable
    );
    assert!(
        nested_attribute.provenance()[0]
            .source()
            .expect("nested Attribute provenance source must exist")
            .as_str()
            .ends_with(
                "/src/Documents/Sales/Sales.mdo#metadata_object=20000000-0000-0000-0000-000000000000;member=attribute:23000000-0000-0000-0000-000000000000"
            )
    );
    assert_eq!(nested_ownership_edge.provenance().len(), 1);
    assert_eq!(
        nested_ownership_edge.provenance()[0].origin(),
        FactOrigin::Declared
    );
    assert!(
        nested_ownership_edge.provenance()[0]
            .source()
            .expect("nested ownership provenance source must exist")
            .as_str()
            .ends_with(
                "/src/Documents/Sales/Sales.mdo#metadata_object=20000000-0000-0000-0000-000000000000;edge=contains;source=22000000-0000-0000-0000-000000000000;target=23000000-0000-0000-0000-000000000000"
            )
    );
}

fn assert_supported_coverage(result: &EdtSemanticGraphBuildResult) {
    let coverage = result.coverage_report();
    let ownership = coverage
        .edt_pipeline()
        .capability(SemanticCoverageCapabilityId::OwnershipRelation(
            NodeKind::Attribute,
        ))
        .expect("Attribute ownership coverage must exist");
    assert_eq!(ownership.status(), SemanticCoverageStatus::Supported);
    assert_eq!(ownership.evidence(), ownership.required_evidence());
    assert!(ownership.missing_evidence().is_empty());
    assert!(ownership.limitations().is_empty());
    assert!(coverage.edt_pipeline().gaps().iter().all(|gap| {
        gap.capability_id() != SemanticCoverageCapabilityId::OwnershipRelation(NodeKind::Attribute)
    }));
    assert_eq!(
        coverage
            .edt_pipeline()
            .gaps_by_priority(SemanticCoverageGapPriority::High)
            .len(),
        0
    );
    assert_eq!(
        coverage
            .edt_pipeline()
            .gaps_by_priority(SemanticCoverageGapPriority::Medium)
            .len(),
        11
    );
    for (priority, expected) in [
        (SemanticCoverageGapPriority::Critical, 0),
        (SemanticCoverageGapPriority::High, 0),
        (SemanticCoverageGapPriority::Medium, 11),
    ] {
        let combined = coverage.graph_domain().gaps_by_priority(priority).len()
            + coverage.edt_pipeline().gaps_by_priority(priority).len();
        assert_eq!(combined, expected);
    }
}

#[test]
fn tabular_section_ownership_fixture_builds_with_immediate_owners() {
    let first = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(&ownership_fixture())
        .expect("real EDT ownership fixture must build");
    let repeated = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(&ownership_fixture())
        .expect("repeated ownership fixture build must succeed");

    assert_immediate_ownership(&first);
    assert_ownership_provenance(&first);
    assert_supported_coverage(&first);

    assert!(first.validate().is_valid());
    assert!(first.graph().diff(repeated.graph()).is_empty());
    assert!(first.diff(&repeated).is_empty());
}

#[test]
fn ownership_output_is_independent_from_source_observation_order() {
    let root = tempfile::tempdir().expect("temporary EDT project must be created");
    write_order_fixture(root.path(), false);
    let first = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(root.path())
        .expect("first ownership observation order must build");

    write_order_fixture(root.path(), true);
    let reordered = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(root.path())
        .expect("reordered ownership observations must build");

    assert!(first.graph().diff(reordered.graph()).is_empty());
    assert!(first.diff(&reordered).is_empty());
}
