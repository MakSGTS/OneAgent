use oneagent_edt::{EdtSemanticGraphBuilder, FileSystemEdtSemanticGraphBuilder};
use oneagent_graph::{EdgeKind, FactOrigin, NodeId, NodeKind};
use oneagent_metadata::MetadataKind;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

const CONFIGURATION: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<mdclass:Configuration xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass"
    uuid="10000000-0000-0000-0000-000000000000">
    <name>Demo</name>
</mdclass:Configuration>
"#;

const DOCUMENT: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<mdclass:Document xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass"
    uuid="20000000-0000-0000-0000-000000000000">
    <name>Sales</name>
    <forms uuid="21000000-0000-0000-0000-000000000000">
        <name>DocumentForm</name>
    </forms>
    <forms uuid="22000000-0000-0000-0000-000000000000">
        <name>OptionalForm</name>
    </forms>
    <commands uuid="23000000-0000-0000-0000-000000000000">
        <name>PostAndClose</name>
    </commands>
</mdclass:Document>
"#;

const COMMON_COMMAND: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<mdclass:CommonCommand xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass"
    uuid="30000000-0000-0000-0000-000000000000">
    <name>GlobalOpen</name>
</mdclass:CommonCommand>
"#;

const COMMON_FORM: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<mdclass:CommonForm xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass"
    uuid="40000000-0000-0000-0000-000000000000">
    <name>Workspace</name>
</mdclass:CommonForm>
"#;

fn write(path: &Path, content: &str) {
    fs::create_dir_all(path.parent().expect("parent directory must exist"))
        .expect("directory must be created");
    fs::write(path, content).expect("fixture file must be created");
}

fn create_project() -> tempfile::TempDir {
    let root = tempdir().expect("temporary directory must be created");
    write(
        &root.path().join("src/Configuration/Configuration.mdo"),
        CONFIGURATION,
    );
    write(&root.path().join("src/Documents/Sales/Sales.mdo"), DOCUMENT);
    write(
        &root.path().join("src/Documents/Sales/ObjectModule.bsl"),
        "Procedure BeforeWrite()\nEndProcedure",
    );
    write(
        &root
            .path()
            .join("src/Documents/Sales/Forms/DocumentForm/Module.bsl"),
        "Procedure FormAction()\nEndProcedure\nFunction FormValue()\nEndFunction",
    );
    write(
        &root
            .path()
            .join("src/Documents/Sales/Commands/PostAndClose/CommandModule.bsl"),
        "Procedure CommandProcessing()\n    Helper();\nEndProcedure\nProcedure Helper()\nEndProcedure",
    );
    write(
        &root
            .path()
            .join("src/CommonCommands/GlobalOpen/GlobalOpen.mdo"),
        COMMON_COMMAND,
    );
    write(
        &root
            .path()
            .join("src/CommonCommands/GlobalOpen/CommandModule.bsl"),
        "Procedure Execute()\nEndProcedure",
    );
    write(
        &root.path().join("src/CommonForms/Workspace/Workspace.mdo"),
        COMMON_FORM,
    );
    write(
        &root.path().join("src/CommonForms/Workspace/Module.bsl"),
        "Procedure Show()\nEndProcedure",
    );
    root
}

fn module_expectations() -> [(&'static str, &'static str, &'static str); 5] {
    [
        (
            "20000000-0000-0000-0000-000000000000:object_module",
            "20000000-0000-0000-0000-000000000000",
            "ObjectModule",
        ),
        (
            "21000000-0000-0000-0000-000000000000:form_module",
            "21000000-0000-0000-0000-000000000000",
            "FormModule",
        ),
        (
            "23000000-0000-0000-0000-000000000000:command_module",
            "23000000-0000-0000-0000-000000000000",
            "CommandModule",
        ),
        (
            "30000000-0000-0000-0000-000000000000:command_module",
            "30000000-0000-0000-0000-000000000000",
            "CommandModule",
        ),
        (
            "40000000-0000-0000-0000-000000000000:common_module",
            "40000000-0000-0000-0000-000000000000",
            "Workspace",
        ),
    ]
}

#[test]
fn module_emission_builds_canonical_nodes_owners_symbols_calls_and_provenance() {
    let root = create_project();
    let first = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(root.path())
        .expect("Form and Command module project must build");
    let second = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(root.path())
        .expect("repeated project build must succeed");
    let graph = first.graph();

    for (module_id, owner_id, expected_name) in module_expectations() {
        let module = graph
            .query()
            .node(&NodeId::new(module_id))
            .expect("canonical module must exist");
        let owner = graph
            .query()
            .owner(&NodeId::new(module_id))
            .expect("canonical module must have one owner");

        assert_eq!(module.kind(), NodeKind::Module);
        assert_eq!(module.name().as_str(), expected_name);
        assert_eq!(owner.id().as_str(), owner_id);
        assert_eq!(module.provenance().len(), 1);
        assert_eq!(module.provenance()[0].origin(), FactOrigin::Parsed);
        assert!(module.provenance()[0].source().is_some());
    }

    for symbol_name in [
        "FormAction",
        "FormValue",
        "CommandProcessing",
        "Helper",
        "Execute",
        "Show",
        "BeforeWrite",
    ] {
        let symbol = graph
            .nodes()
            .find(|node| node.name().as_str() == symbol_name)
            .expect("module declaration must enter the existing BSL pipeline");
        assert!(matches!(
            symbol.kind(),
            NodeKind::Procedure | NodeKind::Function
        ));
        assert!(
            graph
                .query()
                .owner(&NodeId::new(symbol.id().as_str()))
                .is_some()
        );
    }

    let command_processing = graph
        .nodes()
        .find(|node| node.name().as_str() == "CommandProcessing")
        .expect("CommandProcessing procedure must exist");
    let helper = graph
        .nodes()
        .find(|node| node.name().as_str() == "Helper")
        .expect("Helper procedure must exist");
    assert!(graph.edges().any(|edge| {
        edge.source() == command_processing.id()
            && edge.target() == helper.id()
            && edge.kind() == EdgeKind::Calls
    }));
    assert!(graph.query().edges_by_kind(EdgeKind::Opens).is_empty());
    assert!(first.diagnostics().is_empty());
    assert!(first.validate().is_valid());
    assert!(graph.diff(second.graph()).is_empty());
    assert!(first.diff(&second).is_empty());
}

#[test]
fn module_emission_missing_optional_module_emits_no_fact_and_diff_observes_addition() {
    let root = create_project();
    let before = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(root.path())
        .expect("project with missing optional module must build");
    let optional_module_id = NodeId::new("22000000-0000-0000-0000-000000000000:form_module");

    assert!(before.graph().query().node(&optional_module_id).is_none());
    write(
        &root
            .path()
            .join("src/Documents/Sales/Forms/OptionalForm/Module.bsl"),
        "Procedure OptionalAction()\nEndProcedure",
    );
    let after = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(root.path())
        .expect("project with added optional module must build");
    let diff = before.diff(&after);
    let module = after
        .graph()
        .query()
        .node(&optional_module_id)
        .expect("added optional module must exist");
    let owner = after
        .graph()
        .query()
        .owner(&optional_module_id)
        .expect("added optional module must have its Form owner");

    assert_eq!(module.name().as_str(), "FormModule");
    assert_eq!(owner.id().as_str(), "22000000-0000-0000-0000-000000000000");
    assert!(
        diff.graph()
            .added_nodes()
            .iter()
            .any(|change| change.id() == &optional_module_id)
    );
    assert!(
        diff.graph()
            .added_edges()
            .iter()
            .any(|change| change.edge_kind() == EdgeKind::Contains)
    );
    assert!(after.validate().is_valid());
}

#[test]
fn module_emission_preserves_owner_endpoint_kinds_and_no_command_reference_facts() {
    let root = create_project();
    let result = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(root.path())
        .expect("project must build");
    let graph = result.graph();

    let expected_owner_kinds = [
        (
            "21000000-0000-0000-0000-000000000000:form_module",
            NodeKind::Form,
        ),
        (
            "23000000-0000-0000-0000-000000000000:command_module",
            NodeKind::Command,
        ),
        (
            "30000000-0000-0000-0000-000000000000:command_module",
            NodeKind::Metadata(MetadataKind::Command),
        ),
    ];
    for (module_id, owner_kind) in expected_owner_kinds {
        assert_eq!(
            graph
                .query()
                .owner(&NodeId::new(module_id))
                .expect("module owner must exist")
                .kind(),
            owner_kind
        );
    }

    assert!(graph.query().edges_by_kind(EdgeKind::Opens).is_empty());
    assert!(graph.query().edges_by_kind(EdgeKind::References).is_empty());
    assert!(graph.query().edges_by_kind(EdgeKind::DependsOn).is_empty());
}
