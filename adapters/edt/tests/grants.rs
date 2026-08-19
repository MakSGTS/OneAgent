use oneagent_common::{EntityId, EntityName};
use oneagent_edt::{
    EdtGraphError, EdtRoleRightsError, EdtSemanticGraphBuilder, FileSystemEdtSemanticGraphBuilder,
};
use oneagent_graph::{
    AccessRight, EdgeKind, GraphEdge, GraphNode, NodeKind, SemanticDiagnosticCode, SemanticGraph,
};
use oneagent_metadata::MetadataKind;
use std::fs;
use std::path::{Path, PathBuf};

fn grants_fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/grants_project")
}

fn id(value: &str) -> EntityId {
    EntityId::new(value).expect("identifier must be valid")
}

fn access_right_id(resource_id: &str, right_id: &str) -> EntityId {
    AccessRight::new(id(resource_id), id(right_id), Vec::new())
        .expect("access-right identity must be valid")
        .id()
        .clone()
}

fn write_metadata_descriptor(
    root: &Path,
    directory: &str,
    xml_kind: &str,
    source_name: &str,
    uuid: &str,
    metadata_name: &str,
) {
    let object_directory = root.join("src").join(directory).join(source_name);
    fs::create_dir_all(&object_directory).expect("metadata directory must be created");
    fs::write(
        object_directory.join(format!("{source_name}.mdo")),
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<mdclass:{xml_kind} xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass" uuid="{uuid}">
    <name>{metadata_name}</name>
</mdclass:{xml_kind}>
"#
        ),
    )
    .expect("metadata descriptor must be created");
}

fn write_configuration(root: &Path) {
    let directory = root.join("src/Configuration");
    fs::create_dir_all(&directory).expect("configuration directory must be created");
    fs::write(
        directory.join("Configuration.mdo"),
        r#"<?xml version="1.0" encoding="UTF-8"?>
<mdclass:Configuration xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass" uuid="10000000-0000-0000-0000-000000000000">
    <name>DemoConfiguration</name>
</mdclass:Configuration>
"#,
    )
    .expect("configuration descriptor must be created");
}

fn write_role(root: &Path, source_name: &str, uuid: &str, rights: Option<&str>) {
    write_metadata_descriptor(root, "Roles", "Role", source_name, uuid, source_name);
    if let Some(rights) = rights {
        fs::write(
            root.join("src/Roles")
                .join(source_name)
                .join("Rights.rights"),
            rights,
        )
        .expect("role rights must be created");
    }
}

fn create_grants_project(rights: &str, duplicate_product: bool) -> tempfile::TempDir {
    let root = tempfile::tempdir().expect("temporary EDT project must be created");
    write_configuration(root.path());
    write_metadata_descriptor(
        root.path(),
        "Catalogs",
        "Catalog",
        "Product",
        "20000000-0000-0000-0000-000000000000",
        "Product",
    );
    if duplicate_product {
        write_metadata_descriptor(
            root.path(),
            "Catalogs",
            "Catalog",
            "ProductDuplicate",
            "20000000-0000-0000-0000-000000000001",
            "Product",
        );
    }
    write_metadata_descriptor(
        root.path(),
        "Documents",
        "Document",
        "Sale",
        "30000000-0000-0000-0000-000000000000",
        "Sale",
    );
    write_role(
        root.path(),
        "TestRole",
        "40000000-0000-0000-0000-000000000000",
        Some(rights),
    );

    root
}

fn rights_xml(objects: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<Rights xmlns="http://v8.1c.ru/8.2/roles">
    <setForNewObjects>false</setForNewObjects>
    <setForAttributesByDefault>false</setForAttributesByDefault>
    <independentRightsOfChildObjects>false</independentRightsOfChildObjects>
    {objects}
</Rights>
"#
    )
}

fn assert_complete_grant_provenance(
    access_rights: &[&GraphNode],
    grants: &[&GraphEdge],
    references: &[&GraphEdge],
) {
    for (facts, suffix) in [
        (
            access_rights
                .iter()
                .map(|node| node.provenance())
                .collect::<Vec<_>>(),
            ";fact=access_right",
        ),
        (
            grants
                .iter()
                .map(|edge| edge.provenance())
                .collect::<Vec<_>>(),
            ";edge=grants",
        ),
        (
            references
                .iter()
                .map(|edge| edge.provenance())
                .collect::<Vec<_>>(),
            ";edge=references",
        ),
    ] {
        assert!(facts.iter().all(|provenance| {
            !provenance.is_empty()
                && provenance.iter().all(|item| {
                    item.source()
                        .is_some_and(|source| source.as_str().ends_with(suffix))
                })
        }));
    }
}

fn assert_supported_resource_kinds(graph: &SemanticGraph, references: &[&GraphEdge]) {
    for kind in [
        MetadataKind::Configuration,
        MetadataKind::Catalog,
        MetadataKind::Document,
        MetadataKind::InformationRegister,
        MetadataKind::AccumulationRegister,
    ] {
        let resource_ids = graph
            .nodes_by_kind(NodeKind::Metadata(kind))
            .into_iter()
            .map(GraphNode::id)
            .collect::<Vec<_>>();
        assert!(
            references
                .iter()
                .any(|edge| resource_ids.contains(&edge.target()))
        );
    }
}

#[test]
fn grants_real_edt_fixture_emits_scoped_access_rights_with_stable_provenance() {
    let first = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(&grants_fixture())
        .expect("real EDT grants fixture must build");
    let repeated = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(&grants_fixture())
        .expect("repeated grants build must succeed");
    let graph = first.graph();

    let access_rights = graph.nodes_by_kind(NodeKind::AccessRight);
    let grants = graph
        .edges()
        .filter(|edge| edge.kind() == EdgeKind::Grants)
        .collect::<Vec<_>>();
    let access_right_references = graph
        .edges()
        .filter(|edge| {
            edge.kind() == EdgeKind::References
                && graph
                    .node(edge.source())
                    .is_some_and(|node| node.kind() == NodeKind::AccessRight)
        })
        .collect::<Vec<_>>();

    assert!(first.diagnostics().is_empty());
    assert_eq!(access_rights.len(), 39);
    assert_eq!(grants.len(), 50);
    assert_eq!(access_right_references.len(), 39);
    assert_complete_grant_provenance(&access_rights, &grants, &access_right_references);
    assert_supported_resource_kinds(graph, &access_right_references);
    for (member_id, synonym) in [
        ("eff602bb-e50a-4e16-8e5c-abd3a54d2ae3", Some("Proucts")),
        ("3e68cfb1-d0d1-4135-9b7a-49e46d1cc844", None),
        ("3ba9bcb4-cd68-4336-b62f-dee2d7321b6f", Some("Price")),
        ("5b278afd-1f58-4a5b-9118-219f70d8fd3a", Some("Quantity")),
        ("8c8bc708-211f-406d-9f3c-9f6a73b00a91", Some("Ammount")),
    ] {
        let member = graph
            .node(&id(member_id))
            .expect("real grants member node must exist");
        assert_eq!(
            member
                .metadata_member_payload()
                .expect("real grants member payload must exist")
                .synonym(),
            synonym
        );
    }

    let product_read_id = access_right_id("bb9ecb4f-1ae1-4cfd-b2d1-badd172736e9", "Read");
    let product_update_id = access_right_id("bb9ecb4f-1ae1-4cfd-b2d1-badd172736e9", "Update");
    let base_user_id = id("872b31fd-6bc2-44fa-8fbc-1995d6237ed7:role");
    assert!(
        grants
            .iter()
            .any(|edge| edge.source() == &base_user_id && edge.target() == &product_read_id)
    );
    assert!(
        grants
            .iter()
            .any(|edge| edge.source() == &base_user_id && edge.target() == &product_update_id)
    );

    let shared_right_id = access_right_id("da781fc5-b62e-4741-b148-8860c0bc0895", "ThinClient");
    let shared_right = graph
        .node(&shared_right_id)
        .expect("shared configuration access right must exist");
    let shared_reference = access_right_references
        .iter()
        .find(|edge| edge.source() == &shared_right_id)
        .expect("shared access-right reference must exist");
    assert_eq!(shared_right.provenance().len(), 2);
    assert_eq!(shared_reference.provenance().len(), 2);
    for provenance in shared_right.provenance() {
        let source = provenance
            .source()
            .expect("access-right provenance source must exist")
            .as_str();
        assert!(source.contains("/Rights.rights#role_metadata="));
        assert!(source.contains(";role="));
        assert!(source.contains(";protected_resource=Configuration.OneAgentEDTproject"));
        assert!(source.contains(";resolved_resource=da781fc5-b62e-4741-b148-8860c0bc0895"));
        assert!(source.contains(";right=ThinClient;value=true;accepted_explicit_allow=true"));
        assert!(source.ends_with(";fact=access_right"));
    }

    let full_access_id = id("179634e5-6671-4c7d-a02e-4ad4a4ffdca8:role");
    assert!(grants.iter().all(|edge| edge.source() != &full_access_id));
    assert!(grants.iter().all(|edge| {
        graph
            .node(edge.target())
            .is_some_and(|target| target.kind() == NodeKind::AccessRight)
    }));
    assert!(graph.edges().all(|edge| {
        edge.kind() != EdgeKind::Grants
            || graph
                .node(edge.target())
                .is_some_and(|target| !matches!(target.kind(), NodeKind::Metadata(_)))
    }));
    assert!(first.validate().is_valid());
    assert!(graph.diff(repeated.graph()).is_empty());
    assert!(first.diff(&repeated).is_empty());
}

#[test]
fn grants_false_unsupported_missing_and_incompatible_declarations_emit_no_facts() {
    let rights = rights_xml(
        r"
<object><name>Catalog.Product</name><right><name>Read</name><value>false</value></right></object>
<object><name>Enumeration.Product</name><right><name>Read</name><value>true</value></right></object>
<object><name>Catalog.Missing</name><right><name>Read</name><value>true</value></right></object>
<object><name>Catalog.Sale</name><right><name>Read</name><value>true</value></right></object>
",
    );
    let root = create_grants_project(&rights, false);

    let result = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(root.path())
        .expect("recoverable grant resolution failures must not fail the build");
    let repeated = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(root.path())
        .expect("repeated recoverable grant build must succeed");

    assert!(
        result
            .graph()
            .nodes_by_kind(NodeKind::AccessRight)
            .is_empty()
    );
    assert!(
        result
            .graph()
            .edges()
            .all(|edge| edge.kind() != EdgeKind::Grants)
    );
    assert_eq!(result.diagnostics().len(), 2);
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == SemanticDiagnosticCode::ReferenceUnresolved)
    );
    assert!(result.diagnostics().iter().any(|diagnostic| {
        diagnostic.code() == SemanticDiagnosticCode::ReferenceIncompatibleKind
    }));
    assert_eq!(result.reference_statistics().total(), 2);
    assert_eq!(result.reference_statistics().unresolved(), 1);
    assert_eq!(result.reference_statistics().incompatible_target_kind(), 1);
    assert!(result.validate().is_valid());
    assert!(result.diff(&repeated).is_empty());
}

#[test]
fn grants_duplicate_declarations_deduplicate_semantics_and_provenance() {
    let rights = rights_xml(
        r"
<object><name>Catalog.Product</name>
    <right><name>Read</name><value>true</value></right>
    <right><name>Read</name><value>true</value></right>
</object>
<object><name>Catalog.Product</name>
    <right><name>Read</name><value>true</value></right>
</object>
",
    );
    let root = create_grants_project(&rights, false);

    let first = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(root.path())
        .expect("duplicate declarations must build");
    let repeated = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(root.path())
        .expect("repeated duplicate build must succeed");
    let access_right = first
        .graph()
        .nodes_by_kind(NodeKind::AccessRight)
        .into_iter()
        .next()
        .expect("deduplicated access right must exist");
    let grant = first
        .graph()
        .edges()
        .find(|edge| edge.kind() == EdgeKind::Grants)
        .expect("deduplicated grant edge must exist");
    let reference = first
        .graph()
        .edges()
        .find(|edge| edge.kind() == EdgeKind::References && edge.source() == access_right.id())
        .expect("deduplicated companion reference must exist");

    assert_eq!(first.graph().nodes_by_kind(NodeKind::AccessRight).len(), 1);
    assert_eq!(
        first
            .graph()
            .edges()
            .filter(|edge| edge.kind() == EdgeKind::Grants)
            .count(),
        1
    );
    assert_eq!(access_right.provenance().len(), 1);
    assert_eq!(grant.provenance().len(), 1);
    assert_eq!(reference.provenance().len(), 1);
    assert_eq!(first.reference_statistics().resolved(), 3);
    assert!(first.diff(&repeated).is_empty());
}

#[test]
fn grants_ambiguous_resource_emits_diagnostic_without_placeholder() {
    let rights = rights_xml(
        r"<object><name>Catalog.Product</name><right><name>Read</name><value>true</value></right></object>",
    );
    let root = create_grants_project(&rights, true);

    let result = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(root.path())
        .expect("ambiguous grant target must remain recoverable");

    assert!(
        result
            .graph()
            .nodes_by_kind(NodeKind::AccessRight)
            .is_empty()
    );
    assert!(
        result
            .graph()
            .edges()
            .all(|edge| edge.kind() != EdgeKind::Grants)
    );
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(
        result.diagnostics()[0].code(),
        SemanticDiagnosticCode::ReferenceAmbiguous
    );
    assert_eq!(result.diagnostics()[0].candidates().len(), 2);
    assert_eq!(result.reference_statistics().ambiguous(), 1);
}

#[test]
fn grants_missing_or_malformed_role_rights_are_typed_build_errors() {
    let missing_root = tempfile::tempdir().expect("temporary EDT project must be created");
    write_configuration(missing_root.path());
    write_role(
        missing_root.path(),
        "MissingRights",
        "40000000-0000-0000-0000-000000000000",
        None,
    );

    let missing_error = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(missing_root.path())
        .expect_err("missing role rights must fail the build");
    assert!(matches!(
        missing_error,
        EdtGraphError::RoleRights(EdtRoleRightsError::RightsFileNotFound(_))
    ));

    let malformed_root = tempfile::tempdir().expect("temporary EDT project must be created");
    write_configuration(malformed_root.path());
    write_role(
        malformed_root.path(),
        "MalformedRights",
        "40000000-0000-0000-0000-000000000000",
        Some("<Rights"),
    );

    let malformed_error = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(malformed_root.path())
        .expect_err("malformed role rights must fail the build");
    assert!(matches!(malformed_error, EdtGraphError::RoleRights(_)));
}

#[test]
fn grants_right_identifier_preserves_the_exact_edt_token() {
    let rights = rights_xml(
        r"<object><name>Catalog.Product</name><right><name>ReadDataHistory</name><value>true</value></right></object>",
    );
    let root = create_grants_project(&rights, false);

    let result = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(root.path())
        .expect("opaque EDT right token must build");
    let expected = access_right_id("20000000-0000-0000-0000-000000000000", "ReadDataHistory");
    let node = result
        .graph()
        .node(&expected)
        .expect("exact EDT right token must participate in identity");

    assert_eq!(node.kind(), NodeKind::AccessRight);
    assert_eq!(
        node.name(),
        &EntityName::new("ReadDataHistory on 20000000-0000-0000-0000-000000000000")
            .expect("access-right display name must be valid")
    );
}
