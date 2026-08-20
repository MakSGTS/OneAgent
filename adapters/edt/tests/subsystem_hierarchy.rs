use oneagent_common::EntityId;
use oneagent_edt::{
    EdtGraphError, EdtSemanticGraphBuilder, EdtSubsystemHierarchyError,
    FileSystemEdtSemanticGraphBuilder,
};
use oneagent_graph::{
    Confidence, EdgeKind, FactOrigin, NodeId, NodeKind, Provenance, ResolutionState,
    SemanticDiagnosticCode,
};
use oneagent_metadata::MetadataKind;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

fn id(value: &str) -> EntityId {
    EntityId::new(value).expect("fixture identifier must be valid")
}

fn write_configuration(root: &Path) {
    let directory = root.join("src/Configuration");
    fs::create_dir_all(&directory).expect("configuration directory must be created");
    fs::write(
        directory.join("Configuration.mdo"),
        r#"<?xml version="1.0" encoding="UTF-8"?>
<mdclass:Configuration xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass" uuid="id-configuration">
  <name>SubsystemHierarchyFixture</name>
</mdclass:Configuration>
"#,
    )
    .expect("configuration descriptor must be written");
}

fn write_document(root: &Path, directory_name: &str, uuid: &str, semantic_name: &str) {
    let directory = root.join("src/Documents").join(directory_name);
    fs::create_dir_all(&directory).expect("Document directory must be created");
    fs::write(
        directory.join(format!("{directory_name}.mdo")),
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<mdclass:Document xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass" uuid="{uuid}">
  <name>{semantic_name}</name>
</mdclass:Document>
"#
        ),
    )
    .expect("Document descriptor must be written");
}

fn subsystem_directory(root: &Path, hierarchy_path: &[&str]) -> PathBuf {
    let mut directory = root.join("src/Subsystems");
    for (index, name) in hierarchy_path.iter().enumerate() {
        if index > 0 {
            directory.push("Subsystems");
        }
        directory.push(name);
    }
    directory
}

fn write_subsystem(
    root: &Path,
    hierarchy_path: &[&str],
    uuid: &str,
    children: &[&str],
    parent: Option<&str>,
    content: &[&str],
) {
    let directory = subsystem_directory(root, hierarchy_path);
    fs::create_dir_all(&directory).expect("Subsystem directory must be created");
    let name = hierarchy_path
        .last()
        .expect("Subsystem hierarchy path must have a name");
    let mut fields = String::new();
    for token in content {
        writeln!(fields, "  <content>{token}</content>").expect("writing to a String must succeed");
    }
    for child in children {
        writeln!(fields, "  <subsystems>{child}</subsystems>")
            .expect("writing to a String must succeed");
    }
    if let Some(parent) = parent {
        writeln!(fields, "  <parentSubsystem>{parent}</parentSubsystem>")
            .expect("writing to a String must succeed");
    }
    fs::write(
        directory.join(format!("{name}.mdo")),
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<mdclass:Subsystem xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass" uuid="{uuid}">
  <name>{name}</name>
{fields}</mdclass:Subsystem>
"#
        ),
    )
    .expect("Subsystem descriptor must be written");
}

fn populate_nested_project(root: &Path, reverse: bool) {
    write_configuration(root);
    let mut documents = [
        ("TopTarget", "id-document-top"),
        ("SharedTarget", "id-document-shared"),
        ("LeafTarget", "id-document-leaf"),
        ("EndTarget", "id-document-end"),
        ("OtherTarget", "id-document-other"),
    ];
    if reverse {
        documents.reverse();
    }
    for (name, uuid) in documents {
        write_document(root, name, uuid, name);
    }

    let root_children = if reverse {
        ["Shared", "Branch"]
    } else {
        ["Branch", "Shared"]
    };
    let mut descriptors = vec![
        (
            vec!["Root"],
            "id-subsystem-root",
            root_children.to_vec(),
            None,
            vec!["Document.TopTarget"],
        ),
        (
            vec!["Root", "Branch"],
            "id-subsystem-branch",
            vec!["Leaf"],
            Some("Subsystem.Root"),
            vec!["Document.SharedTarget", "Subsystem.Branch"],
        ),
        (
            vec!["Root", "Branch", "Leaf"],
            "id-subsystem-leaf",
            vec!["Deep"],
            Some("Subsystem.Root.Subsystem.Branch"),
            vec!["Document.LeafTarget"],
        ),
        (
            vec!["Root", "Branch", "Leaf", "Deep"],
            "id-subsystem-deep",
            vec!["End"],
            Some("Subsystem.Root.Subsystem.Branch.Subsystem.Leaf"),
            vec![],
        ),
        (
            vec!["Root", "Branch", "Leaf", "Deep", "End"],
            "id-subsystem-end",
            vec![],
            Some("Subsystem.Root.Subsystem.Branch.Subsystem.Leaf.Subsystem.Deep"),
            vec!["Document.EndTarget"],
        ),
        (
            vec!["Root", "Shared"],
            "id-subsystem-root-shared",
            vec![],
            Some("Subsystem.Root"),
            vec![],
        ),
        (
            vec!["Other"],
            "id-subsystem-other",
            vec!["Shared"],
            None,
            vec!["Document.OtherTarget"],
        ),
        (
            vec!["Other", "Shared"],
            "id-subsystem-other-shared",
            vec![],
            Some("Subsystem.Other"),
            vec![],
        ),
    ];
    if reverse {
        descriptors.reverse();
    }
    for (path, uuid, children, parent, content) in descriptors {
        write_subsystem(root, &path, uuid, &children, parent, &content);
    }
}

fn includes_snapshot(
    result: &oneagent_edt::EdtSemanticGraphBuildResult,
) -> Vec<(EntityId, EntityId, Vec<Provenance>)> {
    result
        .graph()
        .query()
        .edges_by_kind(EdgeKind::Includes)
        .into_iter()
        .map(|edge| {
            (
                edge.source().clone(),
                edge.target().clone(),
                edge.provenance().to_vec(),
            )
        })
        .collect()
}

fn assert_root_branch_provenance(edge: &oneagent_graph::GraphEdge) {
    assert_eq!(edge.provenance().len(), 1);
    let provenance = &edge.provenance()[0];
    let source = provenance
        .source()
        .expect("hierarchy edge provenance must have source")
        .as_str();
    assert!(source.starts_with("src/Subsystems/Root/Root.mdo#"));
    for fragment in [
        "parent_metadata_uuid#",
        ":id-subsystem-root",
        "child_metadata_uuid#",
        ":id-subsystem-branch",
        "child_descriptor#",
        ":src/Subsystems/Root/Subsystems/Branch/Branch.mdo",
        "parent_field#",
        ":mdclass:Subsystem/subsystems",
        "raw_child#6:Branch",
        "child_field#",
        ":mdclass:Subsystem/parentSubsystem",
        "raw_parent#14:Subsystem.Root",
        "resolved_parent#",
        ":id-subsystem-root:subsystem",
        "resolved_child#",
        ":id-subsystem-branch:subsystem",
    ] {
        assert!(
            source.contains(fragment),
            "missing provenance fragment {fragment}"
        );
    }
    assert_eq!(
        provenance.producer().as_str(),
        "oneagent.edt.subsystem-hierarchy-resolution"
    );
    assert_eq!(provenance.origin(), FactOrigin::Resolved);
    assert_eq!(provenance.confidence(), Confidence::Exact);
    assert_eq!(provenance.resolution(), ResolutionState::Resolved);
}

#[test]
fn nested_production_emits_direct_hierarchy_ownership_content_and_transitive_query() {
    let root = tempfile::tempdir().expect("temporary EDT project must be created");
    populate_nested_project(root.path(), false);

    let first = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(root.path())
        .expect("nested Subsystem project must build");
    let repeated = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(root.path())
        .expect("repeated nested Subsystem build must succeed");
    let graph = first.graph();
    let hierarchy_edges = graph
        .query()
        .edges_by_kind(EdgeKind::Includes)
        .into_iter()
        .filter(|edge| {
            graph
                .node(edge.target())
                .is_some_and(|node| node.kind() == NodeKind::Subsystem)
        })
        .collect::<Vec<_>>();

    assert_eq!(
        graph
            .nodes_by_kind(NodeKind::Metadata(MetadataKind::Subsystem))
            .len(),
        8
    );
    assert_eq!(graph.nodes_by_kind(NodeKind::Subsystem).len(), 8);
    assert_eq!(hierarchy_edges.len(), 6);
    assert_eq!(graph.query().edges_by_kind(EdgeKind::Includes).len(), 11);
    assert_eq!(first.reference_statistics().resolved(), 5);
    assert_eq!(first.reference_statistics().unsupported_prefix(), 1);
    assert_eq!(first.diagnostics().len(), 1);
    assert_eq!(
        first.diagnostics()[0].code(),
        SemanticDiagnosticCode::ReferenceUnsupportedPrefix
    );

    let configuration = NodeId::new("id-configuration");
    let owned_subsystems = graph
        .query()
        .children_by_kind(&configuration, NodeKind::Metadata(MetadataKind::Subsystem));
    assert_eq!(owned_subsystems.len(), 8);
    for subsystem in graph.nodes_by_kind(NodeKind::Subsystem) {
        assert!(
            graph
                .query()
                .owner(&NodeId::new(subsystem.id().as_str()))
                .is_none()
        );
    }

    let root_id = NodeId::new("id-subsystem-root:subsystem");
    let transitive = graph
        .query()
        .transitive_subsystem_members(&root_id)
        .into_iter()
        .map(|node| node.id().clone())
        .collect::<Vec<_>>();
    assert_eq!(
        transitive,
        [
            id("id-document-end"),
            id("id-document-leaf"),
            id("id-document-shared"),
            id("id-document-top"),
        ]
    );
    assert!(!graph.edges().any(|edge| {
        edge.source().as_str() == "id-subsystem-root:subsystem"
            && edge.target().as_str() == "id-subsystem-end:subsystem"
            && edge.kind() == EdgeKind::Includes
    }));

    let root_branch = hierarchy_edges
        .iter()
        .find(|edge| {
            edge.source().as_str() == "id-subsystem-root:subsystem"
                && edge.target().as_str() == "id-subsystem-branch:subsystem"
        })
        .expect("direct Root-to-Branch hierarchy edge must exist");
    assert_root_branch_provenance(root_branch);

    let shared_ids = graph
        .nodes_by_kind(NodeKind::Subsystem)
        .into_iter()
        .filter(|node| node.name().as_str() == "Shared")
        .map(|node| node.id().as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        shared_ids,
        [
            "id-subsystem-other-shared:subsystem",
            "id-subsystem-root-shared:subsystem",
        ]
    );
    assert!(first.validate().is_valid());
    assert!(first.diff(&repeated).is_empty());
}

#[test]
fn reordered_projects_and_repeated_builds_have_equal_composition_output() {
    let first_root = tempfile::tempdir().expect("first EDT project must be created");
    let second_root = tempfile::tempdir().expect("second EDT project must be created");
    populate_nested_project(first_root.path(), false);
    populate_nested_project(second_root.path(), true);

    let first = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(first_root.path())
        .expect("first nested project must build");
    let second = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(second_root.path())
        .expect("reordered nested project must build");

    assert_eq!(includes_snapshot(&first), includes_snapshot(&second));
    assert_eq!(first.diagnostics(), second.diagnostics());
    assert_eq!(first.reference_statistics(), second.reference_statistics());
}

#[test]
fn nested_content_failures_remain_recoverable_and_do_not_create_self_loops() {
    let root = tempfile::tempdir().expect("temporary EDT project must be created");
    write_configuration(root.path());
    write_document(root.path(), "Valid", "id-valid", "Valid");
    write_document(root.path(), "AmbiguousOne", "id-ambiguous-one", "Ambiguous");
    write_document(root.path(), "AmbiguousTwo", "id-ambiguous-two", "Ambiguous");
    write_document(root.path(), "WrongKind", "id-wrong-kind", "WrongKind");
    let wrong_kind_path = root.path().join("src/Documents/WrongKind");
    let catalog_path = root.path().join("src/Catalogs/WrongKind");
    fs::create_dir_all(catalog_path.parent().expect("Catalogs parent must exist"))
        .expect("Catalogs directory must be created");
    fs::rename(&wrong_kind_path, &catalog_path).expect("wrong-kind fixture must be moved");
    let document_descriptor = catalog_path.join("WrongKind.mdo");
    let catalog_xml = fs::read_to_string(&document_descriptor)
        .expect("wrong-kind descriptor must be readable")
        .replace("mdclass:Document", "mdclass:Catalog");
    fs::write(&document_descriptor, catalog_xml).expect("Catalog descriptor must be written");

    write_subsystem(root.path(), &["Root"], "id-root", &["Child"], None, &[]);
    write_subsystem(
        root.path(),
        &["Root", "Child"],
        "id-child",
        &[],
        Some("Subsystem.Root"),
        &[
            "",
            "Document",
            ".MissingPrefix",
            "Document.",
            "Document.Too.Many",
            "Configuration.SubsystemHierarchyFixture",
            "Form.Child",
            "Unknown.Value",
            "document.Valid",
            "Subsystem.Child",
            "Document.Missing",
            "Document.WrongKind",
            "Document.Ambiguous",
            "Document.Valid",
            "Document.Valid",
        ],
    );

    let result = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(root.path())
        .expect("nested content failures must remain recoverable");
    let statistics = result.reference_statistics();
    assert_eq!(statistics.total(), 14);
    assert_eq!(statistics.malformed_format(), 5);
    assert_eq!(statistics.unsupported_prefix(), 5);
    assert_eq!(statistics.unresolved(), 1);
    assert_eq!(statistics.incompatible_target_kind(), 1);
    assert_eq!(statistics.ambiguous(), 1);
    assert_eq!(statistics.resolved(), 1);
    assert_eq!(result.diagnostics().len(), 13);
    assert_eq!(
        result
            .graph()
            .query()
            .edges_by_kind(EdgeKind::Includes)
            .len(),
        2
    );
    assert!(!result.graph().edges().any(|edge| {
        edge.source().as_str() == edge.target().as_str() && edge.kind() == EdgeKind::Includes
    }));
    assert!(result.validate().is_valid());
}

#[test]
fn hierarchy_projection_failures_are_fatal_before_a_build_result_exists() {
    let undeclared = tempfile::tempdir().expect("temporary EDT project must be created");
    write_configuration(undeclared.path());
    write_subsystem(undeclared.path(), &["Root"], "root", &[], None, &[]);
    write_subsystem(
        undeclared.path(),
        &["Root", "Extra"],
        "extra",
        &[],
        Some("Subsystem.Root"),
        &[],
    );
    let error = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(undeclared.path())
        .expect_err("undeclared physical child must fail the complete build");
    assert!(matches!(
        error,
        EdtGraphError::SubsystemHierarchy(
            EdtSubsystemHierarchyError::UndeclaredChildDirectory { .. }
        )
    ));

    let mismatch = tempfile::tempdir().expect("temporary EDT project must be created");
    write_configuration(mismatch.path());
    write_subsystem(mismatch.path(), &["Root"], "root", &["Child"], None, &[]);
    write_subsystem(
        mismatch.path(),
        &["Root", "Child"],
        "child",
        &[],
        Some("Subsystem.Other"),
        &[],
    );
    let error = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(mismatch.path())
        .expect_err("mismatched qualified parent must fail the complete build");
    assert!(matches!(
        error,
        EdtGraphError::SubsystemHierarchy(EdtSubsystemHierarchyError::ParentPathMismatch { .. })
    ));
}
