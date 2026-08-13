use oneagent_common::EntityId;
use oneagent_edt::{EdtGraphError, EdtSemanticGraphBuilder, FileSystemEdtSemanticGraphBuilder};
use oneagent_graph::{
    Confidence, EdgeKind, FactOrigin, NodeId, NodeKind, ResolutionState, SemanticDiagnosticCode,
    SemanticDiagnosticKind, SemanticReference,
};
use oneagent_metadata::MetadataKind;
use std::fs;
use std::path::{Path, PathBuf};

fn includes_fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/includes_project")
}

fn id(value: &str) -> EntityId {
    EntityId::new(value).expect("identifier must be valid")
}

fn node_id(value: &str) -> NodeId {
    NodeId::new(value)
}

fn write_configuration(root: &Path) {
    let directory = root.join("src/Configuration");
    fs::create_dir_all(&directory).expect("configuration directory must be created");
    fs::write(
        directory.join("Configuration.mdo"),
        r#"<?xml version="1.0" encoding="UTF-8"?>
<mdclass:Configuration xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass" uuid="10000000-0000-0000-0000-000000000000">
  <name>IncludesNegativeFixture</name>
</mdclass:Configuration>
"#,
    )
    .expect("configuration descriptor must be written");
}

fn write_metadata(
    root: &Path,
    directory: &str,
    xml_kind: &str,
    source_name: &str,
    uuid: &str,
    semantic_name: &str,
) {
    let object_directory = root.join("src").join(directory).join(source_name);
    fs::create_dir_all(&object_directory).expect("metadata directory must be created");
    fs::write(
        object_directory.join(format!("{source_name}.mdo")),
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<mdclass:{xml_kind} xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass" uuid="{uuid}">
  <name>{semantic_name}</name>
</mdclass:{xml_kind}>
"#
        ),
    )
    .expect("metadata descriptor must be written");
}

fn write_subsystem(root: &Path, content: &[&str]) {
    let directory = root.join("src/Subsystems/TestObject");
    fs::create_dir_all(&directory).expect("Subsystem directory must be created");
    let content = content
        .iter()
        .map(|token| format!("  <content>{token}</content>"))
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(
        directory.join("TestObject.mdo"),
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<mdclass:Subsystem xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass" uuid="b72ed007-5756-4a1d-b27d-e74aef13083f">
  <name>TestObject</name>
{content}
</mdclass:Subsystem>
"#
        ),
    )
    .expect("Subsystem descriptor must be written");
}

#[test]
fn includes_real_edt_fixture_emits_direct_metadata_members_with_stable_provenance() {
    let first = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(&includes_fixture())
        .expect("real EDT Includes fixture must build");
    let repeated = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(&includes_fixture())
        .expect("repeated Includes build must succeed");
    let graph = first.graph();
    let query = graph.query();
    let subsystem_id = id("b72ed007-5756-4a1d-b27d-e74aef13083f:subsystem");
    let subsystem_node_id = node_id(subsystem_id.as_str());
    let role_metadata_id = id("30000000-0000-0000-0000-000000000000");
    let flat_role_id = id("30000000-0000-0000-0000-000000000000:role");
    let includes = query.edges_by_kind(EdgeKind::Includes);

    assert!(first.diagnostics().is_empty());
    assert_eq!(first.reference_statistics().resolved(), 3);
    assert_eq!(includes.len(), 3);
    assert!(includes.iter().all(|edge| edge.source() == &subsystem_id));
    assert!(
        includes
            .iter()
            .any(|edge| edge.target() == &role_metadata_id)
    );
    assert!(includes.iter().all(|edge| edge.target() != &flat_role_id));
    assert_eq!(
        graph
            .node(&role_metadata_id)
            .expect("metadata Role target must exist")
            .kind(),
        NodeKind::Metadata(MetadataKind::Role)
    );

    let outgoing = query.outgoing_edges_by_kind(&subsystem_node_id, EdgeKind::Includes);
    assert_eq!(outgoing, includes);
    for edge in &includes {
        let incoming =
            query.incoming_edges_by_kind(&node_id(edge.target().as_str()), EdgeKind::Includes);
        assert!(incoming.contains(edge));
        assert_eq!(edge.provenance().len(), 1);
        let provenance = &edge.provenance()[0];
        let source = provenance.source().expect("Includes source must exist");
        assert!(
            source
                .as_str()
                .starts_with("src/Subsystems/TestObject/TestObject.mdo#")
        );
        assert!(
            source
                .as_str()
                .contains("field#25:mdclass:Subsystem/content")
        );
        assert!(
            source
                .as_str()
                .contains("subsystem_metadata_uuid#36:b72ed007-5756-4a1d-b27d-e74aef13083f")
        );
        assert!(source.as_str().contains("target_kind#"));
        assert!(source.as_str().contains("target_name#"));
        assert!(source.as_str().contains("resolved_target#"));
        assert_eq!(
            provenance.producer().as_str(),
            "oneagent.edt.subsystem-content-resolution"
        );
        assert_eq!(provenance.origin(), FactOrigin::Resolved);
        assert_eq!(provenance.confidence(), Confidence::Exact);
        assert_eq!(provenance.resolution(), ResolutionState::Resolved);
    }

    assert!(graph.edges().all(|edge| {
        edge.source() != &subsystem_id || matches!(edge.kind(), EdgeKind::Includes)
    }));
    assert!(first.diff(&repeated).is_empty());
    assert!(first.validate().is_valid());
}

fn populate_invalid_observation_project(root: &Path) {
    write_configuration(root);
    write_metadata(
        root,
        "Documents",
        "Document",
        "Valid",
        "20000000-0000-0000-0000-000000000000",
        "Valid",
    );
    write_metadata(
        root,
        "Documents",
        "Document",
        "AmbiguousOne",
        "20000000-0000-0000-0000-000000000001",
        "Ambiguous",
    );
    write_metadata(
        root,
        "Documents",
        "Document",
        "AmbiguousTwo",
        "20000000-0000-0000-0000-000000000002",
        "Ambiguous",
    );
    write_metadata(
        root,
        "Catalogs",
        "Catalog",
        "WrongKind",
        "40000000-0000-0000-0000-000000000000",
        "WrongKind",
    );
    write_subsystem(
        root,
        &[
            "",
            "Document",
            ".MissingPrefix",
            "Document.",
            "Document.Too.Many",
            "Configuration.IncludesNegativeFixture",
            "Form.Child",
            "Unknown.Value",
            "document.Valid",
            "Subsystem.Other",
            "Catalog.External",
            "Document.WrongKind",
            "Document.Ambiguous",
            "Document.Valid",
            "Document.Valid",
        ],
    );
}

#[test]
fn includes_invalid_observations_are_typed_counted_and_never_emit_placeholders() {
    let root = tempfile::tempdir().expect("temporary EDT project must be created");
    populate_invalid_observation_project(root.path());

    let result = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(root.path())
        .expect("invalid Includes observations must remain recoverable");
    let statistics = *result.reference_statistics();
    let includes = result.graph().query().edges_by_kind(EdgeKind::Includes);

    assert_eq!(statistics.total(), 14);
    assert_eq!(statistics.malformed_format(), 5);
    assert_eq!(statistics.unsupported_prefix(), 5);
    assert_eq!(statistics.unresolved(), 1);
    assert_eq!(statistics.incompatible_target_kind(), 1);
    assert_eq!(statistics.ambiguous(), 1);
    assert_eq!(statistics.resolved(), 1);
    assert_eq!(statistics.outcome_total(), statistics.total());
    assert_eq!(statistics.with_provenance(), statistics.total());
    assert_eq!(result.diagnostics().len(), 13);
    assert_eq!(includes.len(), 1);
    assert_eq!(includes[0].provenance().len(), 1);
    assert!(result.graph().nodes_by_kind(NodeKind::Unknown).is_empty());
    assert!(
        result
            .graph()
            .nodes_by_kind(NodeKind::Metadata(MetadataKind::Unknown))
            .is_empty()
    );

    assert_eq!(
        result
            .diagnostics()
            .iter()
            .filter(|diagnostic| {
                diagnostic.code() == SemanticDiagnosticCode::ReferenceMalformedFormat
                    && diagnostic.kind() == SemanticDiagnosticKind::MalformedReferenceFormat
                    && matches!(diagnostic.reference(), SemanticReference::Raw(_))
            })
            .count(),
        5
    );
    assert_eq!(
        result
            .diagnostics()
            .iter()
            .filter(|diagnostic| {
                diagnostic.code() == SemanticDiagnosticCode::ReferenceUnsupportedPrefix
                    && diagnostic.kind() == SemanticDiagnosticKind::UnsupportedReferencePrefix
            })
            .count(),
        5
    );
    for code in [
        SemanticDiagnosticCode::ReferenceUnresolved,
        SemanticDiagnosticCode::ReferenceIncompatibleKind,
        SemanticDiagnosticCode::ReferenceAmbiguous,
    ] {
        assert_eq!(
            result
                .diagnostics()
                .iter()
                .filter(|diagnostic| diagnostic.code() == code)
                .count(),
            1
        );
    }
    let deferred = result
        .diagnostics()
        .iter()
        .find(|diagnostic| {
            matches!(
                diagnostic.reference(),
                SemanticReference::Raw(raw) if raw == "Subsystem.Other"
            )
        })
        .expect("deferred Subsystem prefix must be diagnosed");
    assert!(deferred.message().contains("recognized but deferred"));

    let ambiguous = result
        .diagnostics()
        .iter()
        .find(|diagnostic| diagnostic.code() == SemanticDiagnosticCode::ReferenceAmbiguous)
        .expect("ambiguous exact-kind target must be diagnosed");
    assert_eq!(
        ambiguous.candidates(),
        &[
            id("20000000-0000-0000-0000-000000000001"),
            id("20000000-0000-0000-0000-000000000002"),
        ]
    );
    assert!(result.validate().is_valid());
}

#[test]
fn includes_output_is_independent_from_source_and_directory_enumeration_order() {
    let first_root = tempfile::tempdir().expect("temporary EDT project must be created");
    let second_root = tempfile::tempdir().expect("temporary EDT project must be created");

    for (root, reverse) in [(first_root.path(), false), (second_root.path(), true)] {
        write_configuration(root);
        let mut metadata = [
            ("Alpha", "20000000-0000-0000-0000-000000000000", "Alpha"),
            ("Beta", "20000000-0000-0000-0000-000000000001", "Beta"),
        ];
        if reverse {
            metadata.reverse();
        }
        for (source_name, uuid, semantic_name) in metadata {
            write_metadata(
                root,
                "Documents",
                "Document",
                source_name,
                uuid,
                semantic_name,
            );
        }
        if reverse {
            write_subsystem(root, &["Document.Beta", "Document.Alpha"]);
        } else {
            write_subsystem(root, &["Document.Alpha", "Document.Beta"]);
        }
    }

    let first = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(first_root.path())
        .expect("first ordered build must succeed");
    let second = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(second_root.path())
        .expect("reverse ordered build must succeed");
    let snapshot = |result: &oneagent_edt::EdtSemanticGraphBuildResult| {
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
            .collect::<Vec<_>>()
    };

    assert_eq!(snapshot(&first), snapshot(&second));
    assert_eq!(first.diagnostics(), second.diagnostics());
    assert_eq!(first.reference_statistics(), second.reference_statistics());
}

#[test]
fn includes_malformed_descriptor_is_a_fatal_typed_build_error() {
    let root = tempfile::tempdir().expect("temporary EDT project must be created");
    write_configuration(root.path());
    let directory = root.path().join("src/Subsystems/Broken");
    fs::create_dir_all(&directory).expect("Subsystem directory must be created");
    fs::write(
        directory.join("Broken.mdo"),
        r#"<mdclass:Subsystem uuid="b72ed007-5756-4a1d-b27d-e74aef13083f"><name>Broken</name></mdclass:Subsystem>"#,
    )
    .expect("invalid Subsystem descriptor must be written");

    let error = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(root.path())
        .expect_err("invalid Subsystem descriptor must fail the build");

    assert!(matches!(error, EdtGraphError::SubsystemContent(_)));
}
