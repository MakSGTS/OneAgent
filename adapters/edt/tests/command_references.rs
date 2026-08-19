use oneagent_common::EntityId;
use oneagent_edt::{EdtSemanticGraphBuilder, FileSystemEdtSemanticGraphBuilder};
use oneagent_graph::{
    EdgeKind, FactOrigin, ImpactNodeStatus, NodeId, NodeKind, ResolutionState,
    SemanticDiagnosticCode, SemanticDiagnosticKind, SemanticImpactAnalyzer, SemanticImpactOptions,
    SemanticReference, SemanticReferenceCategory, SemanticReferenceRequestOutcome,
};
use oneagent_metadata::MetadataKind;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

const CONFIGURATION: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<mdclass:Configuration xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass"
    uuid="10000000-0000-0000-0000-000000000000">
    <name>CommandReferences</name>
</mdclass:Configuration>
"#;

const PRODUCTS_ID: &str = "20000000-0000-0000-0000-000000000000";
const TASK_ID: &str = "30000000-0000-0000-0000-000000000000";
const WRONG_KIND_ID: &str = "40000000-0000-0000-0000-000000000000";
const SALES_ID: &str = "50000000-0000-0000-0000-000000000000";
const RESOLVED_CHILD_ID: &str = "51000000-0000-0000-0000-000000000000";
const MISSING_CHILD_ID: &str = "52000000-0000-0000-0000-000000000000";
const INCOMPATIBLE_CHILD_ID: &str = "53000000-0000-0000-0000-000000000000";
const AMBIGUOUS_CHILD_ID: &str = "54000000-0000-0000-0000-000000000000";
const RESOLVED_COMMON_ID: &str = "60000000-0000-0000-0000-000000000000";
const MULTI_COMMON_ID: &str = "61000000-0000-0000-0000-000000000000";

const SALES_COMMANDS: &str = r#"
    <commands uuid="51000000-0000-0000-0000-000000000000">
        <name>ResolvedChild</name>
        <commandParameterType>
            <types>CatalogRef.Products</types>
            <types>CatalogRef.Products</types>
        </commandParameterType>
    </commands>
    <commands uuid="52000000-0000-0000-0000-000000000000">
        <name>MissingChild</name>
        <commandParameterType><types>DocumentRef.Absent</types></commandParameterType>
    </commands>
    <commands uuid="53000000-0000-0000-0000-000000000000">
        <name>IncompatibleChild</name>
        <commandParameterType><types>TaskRef.WrongKind</types></commandParameterType>
    </commands>
    <commands uuid="54000000-0000-0000-0000-000000000000">
        <name>AmbiguousChild</name>
        <commandParameterType><types>DocumentRef.Duplicate</types></commandParameterType>
    </commands>
    <commands uuid="55000000-0000-0000-0000-000000000000">
        <name>UnsupportedChild</name>
        <commandParameterType><types>DefinedType.DocumentComment</types></commandParameterType>
    </commands>
    <commands uuid="56000000-0000-0000-0000-000000000000">
        <name>MalformedChild</name>
        <commandParameterType><types>CatalogRef.</types></commandParameterType>
    </commands>
    <commands uuid="57000000-0000-0000-0000-000000000000">
        <name>PrimitiveChild</name>
        <commandParameterType><types>String</types></commandParameterType>
    </commands>
    <commands uuid="58000000-0000-0000-0000-000000000000">
        <name>EmptyChild</name>
        <commandParameterType/>
    </commands>
    <commands uuid="59000000-0000-0000-0000-000000000000">
        <name>MissingParameterChild</name>
    </commands>
"#;

fn write(path: &Path, content: &str) {
    fs::create_dir_all(path.parent().expect("parent directory must exist"))
        .expect("fixture directory must be created");
    fs::write(path, content).expect("fixture file must be created");
}

fn metadata_descriptor(root_tag: &str, id: &str, name: &str, synonym: Option<&str>) -> String {
    let synonym = synonym
        .map(|value| format!("    <synonym><key>en</key><content>{value}</content></synonym>\n"))
        .unwrap_or_default();
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<mdclass:{root_tag} xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass" uuid="{id}">
    <name>{name}</name>
{synonym}</mdclass:{root_tag}>
"#
    )
}

fn sales_descriptor(reversed: bool) -> String {
    let commands = if reversed {
        SALES_COMMANDS.trim()
    } else {
        SALES_COMMANDS
    };
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<mdclass:Document xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass" uuid="{SALES_ID}">
    <name>Sales</name>
{commands}
</mdclass:Document>
"#
    )
}

fn common_command(id: &str, name: &str, values: &[&str]) -> String {
    let values = values
        .iter()
        .map(|value| format!("        <types>{value}</types>"))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<mdclass:CommonCommand xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass" uuid="{id}">
    <name>{name}</name>
    <commandParameterType>
{values}
    </commandParameterType>
</mdclass:CommonCommand>
"#
    )
}

fn create_project() -> tempfile::TempDir {
    let root = tempdir().expect("temporary directory must be created");
    write(
        &root.path().join("src/Configuration/Configuration.mdo"),
        CONFIGURATION,
    );
    write(
        &root.path().join("src/Catalogs/Products/Products.mdo"),
        &metadata_descriptor("Catalog", PRODUCTS_ID, "Products", Some("Products")),
    );
    write(
        &root
            .path()
            .join("src/Tasks/PerformerTask/PerformerTask.mdo"),
        &metadata_descriptor("Task", TASK_ID, "PerformerTask", None),
    );
    write(
        &root.path().join("src/Documents/WrongKind/WrongKind.mdo"),
        &metadata_descriptor("Document", WRONG_KIND_ID, "WrongKind", None),
    );
    write(
        &root
            .path()
            .join("src/Documents/DuplicateOne/DuplicateOne.mdo"),
        &metadata_descriptor(
            "Document",
            "41000000-0000-0000-0000-000000000000",
            "Duplicate",
            None,
        ),
    );
    write(
        &root
            .path()
            .join("src/Documents/DuplicateTwo/DuplicateTwo.mdo"),
        &metadata_descriptor(
            "Document",
            "42000000-0000-0000-0000-000000000000",
            "Duplicate",
            None,
        ),
    );
    write(
        &root.path().join("src/Documents/Sales/Sales.mdo"),
        &sales_descriptor(false),
    );
    write(
        &root
            .path()
            .join("src/CommonCommands/ResolvedCommon/ResolvedCommon.mdo"),
        &common_command(
            RESOLVED_COMMON_ID,
            "ResolvedCommon",
            &["TaskRef.PerformerTask"],
        ),
    );
    write(
        &root
            .path()
            .join("src/CommonCommands/MultiCommon/MultiCommon.mdo"),
        &common_command(
            MULTI_COMMON_ID,
            "MultiCommon",
            &["TaskRef.PerformerTask", "CatalogRef.Products"],
        ),
    );
    root
}

fn id(value: &str) -> EntityId {
    EntityId::new(value).expect("test identifier must be valid")
}

fn assert_resolved_edges(
    result: &oneagent_edt::EdtSemanticGraphBuildResult,
    source: &str,
    targets: &[&str],
) {
    let source = id(source);
    let references = result
        .graph()
        .outgoing_by_kind(&source, EdgeKind::References);
    let dependencies = result
        .graph()
        .outgoing_by_kind(&source, EdgeKind::DependsOn);
    let expected = targets.iter().map(|target| id(target)).collect::<Vec<_>>();

    assert_eq!(references.len(), expected.len());
    assert_eq!(dependencies.len(), expected.len());
    for target in expected {
        assert!(references.iter().any(|edge| edge.target() == &target));
        assert!(dependencies.iter().any(|edge| edge.target() == &target));
    }
    for edge in references {
        let provenance = &edge.provenance()[0];
        assert_eq!(provenance.origin(), FactOrigin::Resolved);
        assert_eq!(provenance.resolution(), ResolutionState::Resolved);
        let source = provenance
            .source()
            .expect("resolved reference provenance must retain source context")
            .as_str();
        assert!(source.contains("role=command_parameter_type"));
        assert!(source.contains("raw_token="));
        assert!(source.contains("target="));
    }
    for edge in dependencies {
        assert_eq!(edge.provenance()[0].origin(), FactOrigin::Derived);
        assert_eq!(edge.provenance()[0].resolution(), ResolutionState::Resolved);
        assert!(
            edge.provenance()[0]
                .source()
                .expect("derived dependency provenance must retain origin")
                .as_str()
                .contains("origin=command_parameter_type_reference")
        );
    }
}

fn assert_request_lifecycle(
    first: &oneagent_edt::EdtSemanticGraphBuildResult,
    repeated: &oneagent_edt::EdtSemanticGraphBuildResult,
) {
    assert_eq!(first.reference_requests().len(), 7);
    assert_eq!(first.reference_requests(), repeated.reference_requests());
    assert_eq!(
        first
            .reference_request_query()
            .by_category(SemanticReferenceCategory::MetadataType)
            .len(),
        7
    );
    assert_eq!(
        first
            .reference_request_query()
            .by_outcome(SemanticReferenceRequestOutcome::Resolved)
            .len(),
        4
    );
    assert_eq!(
        first
            .reference_request_query()
            .by_outcome(SemanticReferenceRequestOutcome::MissingTarget)
            .len(),
        1
    );
    assert_eq!(
        first
            .reference_request_query()
            .by_outcome(SemanticReferenceRequestOutcome::IncompatibleTargetKind)
            .len(),
        1
    );
    assert_eq!(
        first
            .reference_request_query()
            .by_outcome(SemanticReferenceRequestOutcome::AmbiguousTarget)
            .len(),
        1
    );

    let resolved_child = first
        .reference_request_query()
        .by_source(&id(RESOLVED_CHILD_ID));
    assert_eq!(resolved_child.len(), 1);
    assert_eq!(
        resolved_child[0].expected_kinds(),
        &[NodeKind::Metadata(MetadataKind::Catalog)]
    );
    assert_eq!(resolved_child[0].candidates(), &[id(PRODUCTS_ID)]);
    assert!(matches!(
        resolved_child[0].reference(),
        SemanticReference::Name(name) if name.as_str() == "Products"
    ));
    assert!(resolved_child[0].provenance().iter().any(|provenance| {
        provenance.origin() == FactOrigin::Declared
            && provenance
                .source()
                .is_some_and(|source| source.as_str().contains("occurrences=2"))
    }));
    assert!(resolved_child[0].provenance().iter().any(|provenance| {
        provenance.origin() == FactOrigin::Resolved
            && provenance.resolution() == ResolutionState::Resolved
    }));
    assert_eq!(
        first
            .reference_request_query()
            .by_source(&id(INCOMPATIBLE_CHILD_ID))[0]
            .outcome(),
        SemanticReferenceRequestOutcome::IncompatibleTargetKind
    );
    assert_eq!(
        first
            .reference_request_query()
            .by_source(&id(AMBIGUOUS_CHILD_ID))[0]
            .candidates()
            .len(),
        2
    );
}

fn assert_command_edges(first: &oneagent_edt::EdtSemanticGraphBuildResult) {
    assert_resolved_edges(first, RESOLVED_CHILD_ID, &[PRODUCTS_ID]);
    assert_resolved_edges(first, RESOLVED_COMMON_ID, &[TASK_ID]);
    assert_resolved_edges(first, MULTI_COMMON_ID, &[PRODUCTS_ID, TASK_ID]);
    assert_eq!(
        first
            .graph()
            .edges()
            .filter(|edge| edge.kind() == EdgeKind::References)
            .count(),
        4
    );
    assert_eq!(
        first
            .graph()
            .edges()
            .filter(|edge| edge.kind() == EdgeKind::DependsOn)
            .count(),
        4
    );
}

fn assert_diagnostics_and_reports(
    first: &oneagent_edt::EdtSemanticGraphBuildResult,
    repeated: &oneagent_edt::EdtSemanticGraphBuildResult,
) {
    let diagnostic_codes = first
        .diagnostics()
        .iter()
        .map(oneagent_graph::SemanticDiagnostic::code)
        .collect::<Vec<_>>();
    assert_eq!(first.diagnostics().len(), 5);
    for expected in [
        SemanticDiagnosticCode::ReferenceUnresolved,
        SemanticDiagnosticCode::ReferenceIncompatibleKind,
        SemanticDiagnosticCode::ReferenceAmbiguous,
        SemanticDiagnosticCode::ReferenceUnsupportedPrefix,
        SemanticDiagnosticCode::ReferenceMalformedFormat,
    ] {
        assert!(diagnostic_codes.contains(&expected));
    }
    assert!(first.diagnostics().iter().any(|diagnostic| {
        diagnostic.kind() == SemanticDiagnosticKind::UnsupportedReferencePrefix
            && diagnostic.source_node() == Some(&id("55000000-0000-0000-0000-000000000000"))
    }));
    assert!(first.diagnostics().iter().any(|diagnostic| {
        diagnostic.kind() == SemanticDiagnosticKind::MalformedReferenceFormat
            && diagnostic.source_node() == Some(&id("56000000-0000-0000-0000-000000000000"))
    }));
    for ignored in [
        "57000000-0000-0000-0000-000000000000",
        "58000000-0000-0000-0000-000000000000",
        "59000000-0000-0000-0000-000000000000",
    ] {
        assert!(
            first
                .reference_request_query()
                .by_source(&id(ignored))
                .is_empty()
        );
        assert!(
            first
                .diagnostics()
                .iter()
                .all(|diagnostic| diagnostic.source_node() != Some(&id(ignored)))
        );
    }

    let statistics = *first.reference_statistics();
    assert_eq!(statistics.total(), 9);
    assert_eq!(statistics.resolved(), 4);
    assert_eq!(statistics.unresolved(), 1);
    assert_eq!(statistics.ambiguous(), 1);
    assert_eq!(statistics.incompatible_target_kind(), 1);
    assert_eq!(statistics.unsupported_prefix(), 1);
    assert_eq!(statistics.malformed_format(), 1);
    assert_eq!(statistics.with_provenance(), 9);
    assert!(first.validate().is_valid());
    assert_eq!(first.report().resolution().total(), 9);
    assert_eq!(first.report().graph().total_diagnostics(), 5);
    assert!(
        first
            .graph()
            .query()
            .node(&NodeId::new(RESOLVED_COMMON_ID))
            .is_some()
    );
    assert!(first.graph().diff(repeated.graph()).is_empty());
    assert!(first.diff(repeated).is_empty());
}

fn assert_determinism_and_impact(
    root: &tempfile::TempDir,
    first: &oneagent_edt::EdtSemanticGraphBuildResult,
) {
    write(
        &root.path().join("src/Documents/Sales/Sales.mdo"),
        &sales_descriptor(true),
    );
    write(
        &root
            .path()
            .join("src/CommonCommands/MultiCommon/MultiCommon.mdo"),
        &common_command(
            MULTI_COMMON_ID,
            "MultiCommon",
            &["CatalogRef.Products", "TaskRef.PerformerTask"],
        ),
    );
    let reordered = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(root.path())
        .expect("reordered project must build");
    assert!(first.diff(&reordered).is_empty());

    write(
        &root.path().join("src/Catalogs/Products/Products.mdo"),
        &metadata_descriptor("Catalog", PRODUCTS_ID, "Products", Some("Updated products")),
    );
    let changed = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(root.path())
        .expect("changed project must build");
    let graph_diff = reordered.graph().diff(changed.graph());
    let impact = SemanticImpactAnalyzer::analyze(
        reordered.graph(),
        changed.graph(),
        &graph_diff,
        &SemanticImpactOptions::new(1),
    )
    .expect("command dependencies must participate in impact analysis");
    for source in [PRODUCTS_ID, RESOLVED_CHILD_ID, MULTI_COMMON_ID] {
        assert!(impact.affected_nodes().iter().any(|node| {
            node.node_id().as_str() == source
                && if source == PRODUCTS_ID {
                    node.status() == ImpactNodeStatus::DirectlyChanged
                } else {
                    node.status() == ImpactNodeStatus::TransitivelyAffected
                }
        }));
    }
}

#[test]
fn command_parameter_references_cover_the_production_lifecycle() {
    let root = create_project();
    let first = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(root.path())
        .expect("command reference project must build");
    let repeated = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(root.path())
        .expect("repeated command reference project must build");

    assert_request_lifecycle(&first, &repeated);
    assert_command_edges(&first);
    assert_diagnostics_and_reports(&first, &repeated);
    assert_determinism_and_impact(&root, &first);
}

#[test]
fn command_request_identity_survives_missing_to_resolved_diff() {
    let root = create_project();
    let previous = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(root.path())
        .expect("missing command target snapshot must build");
    let previous_request = previous
        .reference_request_query()
        .by_source(&id(MISSING_CHILD_ID))[0];

    write(
        &root.path().join("src/Documents/Absent/Absent.mdo"),
        &metadata_descriptor(
            "Document",
            "70000000-0000-0000-0000-000000000000",
            "Absent",
            None,
        ),
    );
    let current = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(root.path())
        .expect("resolved command target snapshot must build");
    let current_request = current
        .reference_request_query()
        .by_source(&id(MISSING_CHILD_ID))[0];
    let diff = previous.diff(&current);

    assert_eq!(previous_request.id(), current_request.id());
    assert_eq!(
        previous_request.outcome(),
        SemanticReferenceRequestOutcome::MissingTarget
    );
    assert_eq!(
        current_request.outcome(),
        SemanticReferenceRequestOutcome::Resolved
    );
    assert!(diff.reference_requests().added().is_empty());
    assert!(diff.reference_requests().removed().is_empty());
    assert_eq!(diff.reference_requests().modified().len(), 1);
    assert_eq!(diff.diagnostics().removed().len(), 1);
    assert_eq!(
        current
            .graph()
            .outgoing_by_kind(&id(MISSING_CHILD_ID), EdgeKind::References)
            .len(),
        1
    );
    assert_eq!(
        current
            .graph()
            .outgoing_by_kind(&id(MISSING_CHILD_ID), EdgeKind::DependsOn)
            .len(),
        1
    );
    assert!(previous.validate().is_valid());
    assert!(current.validate().is_valid());
}
