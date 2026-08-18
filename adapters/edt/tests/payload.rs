use oneagent_common::EntityName;
use oneagent_edt::{
    EdtGraphError, EdtMetadataObjectError, EdtSemanticGraphBuildResult, EdtSemanticGraphBuilder,
    FileSystemEdtSemanticGraphBuilder,
};
use oneagent_graph::{EdgeKind, FactOrigin, NodeId, NodeKind, NodeModifiedAspect};
use oneagent_metadata::{MetadataKind, MetadataSpecificPayload};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

#[derive(Debug, Clone, Copy)]
struct PayloadCase {
    directory: &'static str,
    xml_kind: &'static str,
    id: &'static str,
    name: &'static str,
    kind: MetadataKind,
}

const CONFIGURATION: PayloadCase = PayloadCase {
    directory: "Configuration",
    xml_kind: "Configuration",
    id: "payload-configuration",
    name: "PayloadConfiguration",
    kind: MetadataKind::Configuration,
};

const PAYLOAD_CASES: [PayloadCase; 20] = [
    PayloadCase {
        directory: "Catalogs",
        xml_kind: "Catalog",
        id: "payload-catalog",
        name: "PayloadCatalog",
        kind: MetadataKind::Catalog,
    },
    PayloadCase {
        directory: "Documents",
        xml_kind: "Document",
        id: "payload-document",
        name: "PayloadDocument",
        kind: MetadataKind::Document,
    },
    PayloadCase {
        directory: "Enums",
        xml_kind: "Enum",
        id: "payload-enumeration",
        name: "PayloadEnumeration",
        kind: MetadataKind::Enumeration,
    },
    PayloadCase {
        directory: "CommonModules",
        xml_kind: "CommonModule",
        id: "payload-common-module",
        name: "PayloadCommonModule",
        kind: MetadataKind::CommonModule,
    },
    PayloadCase {
        directory: "Reports",
        xml_kind: "Report",
        id: "payload-report",
        name: "PayloadReport",
        kind: MetadataKind::Report,
    },
    PayloadCase {
        directory: "DataProcessors",
        xml_kind: "DataProcessor",
        id: "payload-data-processor",
        name: "PayloadDataProcessor",
        kind: MetadataKind::DataProcessor,
    },
    PayloadCase {
        directory: "InformationRegisters",
        xml_kind: "InformationRegister",
        id: "payload-information-register",
        name: "PayloadInformationRegister",
        kind: MetadataKind::InformationRegister,
    },
    PayloadCase {
        directory: "AccumulationRegisters",
        xml_kind: "AccumulationRegister",
        id: "payload-accumulation-register",
        name: "PayloadAccumulationRegister",
        kind: MetadataKind::AccumulationRegister,
    },
    PayloadCase {
        directory: "AccountingRegisters",
        xml_kind: "AccountingRegister",
        id: "payload-accounting-register",
        name: "PayloadAccountingRegister",
        kind: MetadataKind::AccountingRegister,
    },
    PayloadCase {
        directory: "CalculationRegisters",
        xml_kind: "CalculationRegister",
        id: "payload-calculation-register",
        name: "PayloadCalculationRegister",
        kind: MetadataKind::CalculationRegister,
    },
    PayloadCase {
        directory: "BusinessProcesses",
        xml_kind: "BusinessProcess",
        id: "payload-business-process",
        name: "PayloadBusinessProcess",
        kind: MetadataKind::BusinessProcess,
    },
    PayloadCase {
        directory: "Tasks",
        xml_kind: "Task",
        id: "payload-task",
        name: "PayloadTask",
        kind: MetadataKind::Task,
    },
    PayloadCase {
        directory: "Roles",
        xml_kind: "Role",
        id: "payload-role",
        name: "PayloadRole",
        kind: MetadataKind::Role,
    },
    PayloadCase {
        directory: "CommonCommands",
        xml_kind: "CommonCommand",
        id: "payload-command",
        name: "PayloadCommand",
        kind: MetadataKind::Command,
    },
    PayloadCase {
        directory: "CommonForms",
        xml_kind: "CommonForm",
        id: "payload-common-form",
        name: "PayloadCommonForm",
        kind: MetadataKind::CommonForm,
    },
    PayloadCase {
        directory: "CommonTemplates",
        xml_kind: "CommonTemplate",
        id: "payload-template",
        name: "PayloadTemplate",
        kind: MetadataKind::Template,
    },
    PayloadCase {
        directory: "HTTPServices",
        xml_kind: "HTTPService",
        id: "payload-http-service",
        name: "PayloadHttpService",
        kind: MetadataKind::HttpService,
    },
    PayloadCase {
        directory: "WebServices",
        xml_kind: "WebService",
        id: "payload-web-service",
        name: "PayloadWebService",
        kind: MetadataKind::WebService,
    },
    PayloadCase {
        directory: "XDTOPackages",
        xml_kind: "XDTOPackage",
        id: "payload-xdto-package",
        name: "PayloadXdtoPackage",
        kind: MetadataKind::XdtoPackage,
    },
    PayloadCase {
        directory: "Subsystems",
        xml_kind: "Subsystem",
        id: "payload-subsystem",
        name: "PayloadSubsystem",
        kind: MetadataKind::Subsystem,
    },
];

fn case(kind: MetadataKind) -> PayloadCase {
    PAYLOAD_CASES
        .iter()
        .copied()
        .find(|case| case.kind == kind)
        .expect("payload case must exist")
}

fn descriptor_path(root: &Path, case: PayloadCase) -> PathBuf {
    if case.kind == MetadataKind::Configuration {
        root.join("src/Configuration/Configuration.mdo")
    } else {
        root.join("src")
            .join(case.directory)
            .join(case.name)
            .join(format!("{}.mdo", case.name))
    }
}

fn synonym_xml(name: &str, include_synonym: bool) -> String {
    if include_synonym {
        format!("    <synonym><key>en</key><content>{name} synonym</content></synonym>\n")
    } else {
        String::new()
    }
}

fn kind_specific_xml(kind: MetadataKind) -> &'static str {
    match kind {
        MetadataKind::Document => {
            r#"    <registerRecords>CalculationRegister.PayloadCalculationRegister</registerRecords>
    <registerRecords>InformationRegister.PayloadInformationRegister</registerRecords>
    <registerRecords>AccumulationRegister.PayloadAccumulationRegister</registerRecords>
    <registerRecords>AccountingRegister.PayloadAccountingRegister</registerRecords>
    <registerRecords>InformationRegister.PayloadInformationRegister</registerRecords>
    <registerRecords>LocalizedRegister.Hidden</registerRecords>
    <registerRecords>NameOnly</registerRecords>
    <registerRecords>AccumulationRegister.CaseTarget</registerRecords>
    <registerRecords>AccumulationRegister.CASETARGET</registerRecords>
    <attributes uuid="payload-document-catalog-reference">
        <name>CatalogTarget</name>
        <type><types>CatalogRef.PayloadCatalog</types></type>
    </attributes>
    <attributes uuid="payload-document-missing-reference">
        <name>MissingTarget</name>
        <type><types>CatalogRef.MissingCatalog</types></type>
    </attributes>
"#
        }
        MetadataKind::Subsystem => "    <content>Catalog.PayloadCatalog</content>\n",
        _ => "",
    }
}

fn write_descriptor(root: &Path, case: PayloadCase, include_synonym: bool) {
    let path = descriptor_path(root, case);
    fs::create_dir_all(path.parent().expect("descriptor parent must exist"))
        .expect("descriptor directory must be created");
    fs::write(
        path,
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<mdclass:{xml_kind} xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass" uuid="{id}">
    <name>{name}</name>
{synonym}{specific}</mdclass:{xml_kind}>
"#,
            xml_kind = case.xml_kind,
            id = case.id,
            name = case.name,
            synonym = synonym_xml(case.name, include_synonym),
            specific = kind_specific_xml(case.kind),
        ),
    )
    .expect("metadata descriptor must be written");
}

fn write_separate_semantic_facts(root: &Path, include_synonym: bool) {
    let document = case(MetadataKind::Document);
    fs::write(
        descriptor_path(root, document)
            .parent()
            .expect("Document parent must exist")
            .join("ObjectModule.bsl"),
        "Procedure PayloadProcedure()\nEndProcedure",
    )
    .expect("Document module must be written");

    let role = case(MetadataKind::Role);
    fs::write(
        descriptor_path(root, role)
            .parent()
            .expect("Role parent must exist")
            .join("Rights.rights"),
        r#"<?xml version="1.0" encoding="UTF-8"?>
<Rights xmlns="http://v8.1c.ru/8.2/roles">
    <setForNewObjects>false</setForNewObjects>
    <setForAttributesByDefault>false</setForAttributesByDefault>
    <independentRightsOfChildObjects>false</independentRightsOfChildObjects>
    <object>
        <name>Catalog.PayloadCatalog</name>
        <right><name>Read</name><value>true</value></right>
    </object>
</Rights>
"#,
    )
    .expect("Role rights must be written");

    let directory = root.join("src/Catalogs/PayloadCatalogExtension");
    fs::create_dir_all(&directory).expect("extension directory must be created");
    fs::write(
        directory.join("PayloadCatalogExtension.mdo"),
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<mdclass:Catalog xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass" uuid="payload-catalog-extension">
    <name>PayloadCatalogExtension</name>
{synonym}    <objectBelonging>Adopted</objectBelonging>
    <extendedConfigurationObject>payload-catalog</extendedConfigurationObject>
</mdclass:Catalog>
"#,
            synonym = synonym_xml("PayloadCatalogExtension", include_synonym),
        ),
    )
    .expect("extension descriptor must be written");
}

fn write_fixture(root: &Path, include_synonyms: bool) {
    write_descriptor(root, CONFIGURATION, include_synonyms);
    for case in PAYLOAD_CASES {
        write_descriptor(root, case, include_synonyms);
    }
    write_separate_semantic_facts(root, include_synonyms);
}

fn expected_document_records() -> Vec<(MetadataKind, &'static str)> {
    vec![
        (
            MetadataKind::InformationRegister,
            "PayloadInformationRegister",
        ),
        (
            MetadataKind::AccumulationRegister,
            "PayloadAccumulationRegister",
        ),
        (
            MetadataKind::AccountingRegister,
            "PayloadAccountingRegister",
        ),
        (
            MetadataKind::CalculationRegister,
            "PayloadCalculationRegister",
        ),
    ]
}

fn assert_case_payload(
    result: &EdtSemanticGraphBuildResult,
    case: PayloadCase,
    include_synonym: bool,
) {
    let query = result.graph().query();
    let node = query
        .node(&NodeId::new(case.id))
        .expect("metadata node must be queryable by stable identity");
    let payload = node
        .metadata_payload()
        .expect("metadata node must expose typed payload");
    let expected_synonym = include_synonym.then(|| format!("{} synonym", case.name));

    assert_eq!(node.kind(), NodeKind::Metadata(case.kind));
    assert_eq!(payload.common().synonym(), expected_synonym.as_deref());
    let named = query.nodes_by_name(&EntityName::new(case.name).expect("name must be valid"));
    assert!(named.contains(&node));
    assert_eq!(
        named.len(),
        if matches!(case.kind, MetadataKind::Role | MetadataKind::Subsystem) {
            2
        } else {
            1
        }
    );
    assert_eq!(node.provenance().len(), 1);
    assert_eq!(
        node.provenance()[0].origin(),
        if case.kind == MetadataKind::Configuration {
            FactOrigin::Parsed
        } else {
            FactOrigin::Declared
        }
    );
    assert!(node.provenance()[0].source().is_some());

    match (case.kind, payload.specific()) {
        (MetadataKind::Document, Some(MetadataSpecificPayload::Document(document))) => {
            let records = document
                .register_records()
                .iter()
                .map(|record| (record.target_kind(), record.target_name().as_str()))
                .collect::<Vec<_>>();
            assert_eq!(records, expected_document_records());
        }
        (_, None) => {}
        (kind, specific) => panic!("unexpected {kind:?} payload: {specific:?}"),
    }
}

fn assert_payload_fixture(result: &EdtSemanticGraphBuildResult, include_synonyms: bool) {
    assert_case_payload(result, CONFIGURATION, include_synonyms);
    for case in PAYLOAD_CASES {
        assert_case_payload(result, case, include_synonyms);
    }

    let extension = result
        .graph()
        .query()
        .node(&NodeId::new("payload-catalog-extension"))
        .expect("adopted Catalog must exist");
    assert_eq!(
        extension
            .metadata_payload()
            .expect("adopted Catalog payload must exist")
            .specific(),
        None
    );
    let query = result.graph().query();
    assert!(
        query
            .nodes_by_kind(NodeKind::Metadata(MetadataKind::Form))
            .is_empty()
    );
    assert!(
        query
            .nodes_by_kind(NodeKind::Metadata(MetadataKind::Unknown))
            .is_empty()
    );
    for edge_kind in [
        EdgeKind::References,
        EdgeKind::DependsOn,
        EdgeKind::Grants,
        EdgeKind::Includes,
        EdgeKind::Extends,
    ] {
        assert!(!query.edges_by_kind(edge_kind).is_empty());
    }
    assert!(query.edges_by_kind(EdgeKind::Writes).is_empty());
    assert!(!query.nodes_by_kind(NodeKind::Attribute).is_empty());
    assert!(!query.nodes_by_kind(NodeKind::Module).is_empty());
    assert!(!query.nodes_by_kind(NodeKind::AccessRight).is_empty());
    assert!(!result.diagnostics().is_empty());
    assert!(result.reference_statistics().unresolved() > 0);
    assert!(result.validate().is_valid());
}

#[test]
fn payload_matrix_covers_every_supported_edt_metadata_kind() {
    for include_synonyms in [false, true] {
        let root = tempdir().expect("temporary directory must be created");
        write_fixture(root.path(), include_synonyms);

        let first = FileSystemEdtSemanticGraphBuilder
            .build_graph_with_diagnostics(root.path())
            .expect("payload fixture must build");
        let second = FileSystemEdtSemanticGraphBuilder
            .build_graph_with_diagnostics(root.path())
            .expect("repeated payload fixture build must succeed");

        assert_payload_fixture(&first, include_synonyms);
        assert!(first.graph().diff(second.graph()).is_empty());
        assert!(first.diff(&second).is_empty());
    }
}

#[test]
fn payload_only_change_preserves_identity_and_non_payload_facts() {
    let root = tempdir().expect("temporary directory must be created");
    write_fixture(root.path(), true);
    let previous = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(root.path())
        .expect("payload fixture must build");
    let catalog = case(MetadataKind::Catalog);
    write_descriptor(root.path(), catalog, false);
    let current = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(root.path())
        .expect("changed payload fixture must build");
    let repeated = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(root.path())
        .expect("repeated changed payload fixture must build");
    let graph_diff = previous.graph().diff(current.graph());
    let build_diff = previous.diff(&current);

    assert!(graph_diff.added_nodes().is_empty());
    assert!(graph_diff.removed_nodes().is_empty());
    assert_eq!(graph_diff.modified_nodes().len(), 1);
    assert_eq!(graph_diff.modified_nodes()[0].id().as_str(), catalog.id);
    assert_eq!(
        graph_diff.modified_nodes()[0].modified_aspects(),
        &[NodeModifiedAspect::SemanticContent]
    );
    assert!(graph_diff.added_edges().is_empty());
    assert!(graph_diff.removed_edges().is_empty());
    assert!(graph_diff.modified_edges().is_empty());
    assert_eq!(build_diff.summary().node_changes(), 1);
    assert_eq!(build_diff.summary().edge_changes(), 0);
    assert_eq!(build_diff.summary().diagnostic_changes(), 0);
    assert_eq!(build_diff.summary().resolution_metric_changes(), 0);
    assert_eq!(previous.diagnostics(), current.diagnostics());
    assert_eq!(
        previous.reference_statistics(),
        current.reference_statistics()
    );
    assert!(current.validate().is_valid());
    assert!(current.diff(&repeated).is_empty());
}

#[test]
fn malformed_generic_synonym_remains_a_typed_reader_error() {
    let root = tempdir().expect("temporary directory must be created");
    write_fixture(root.path(), false);
    let report = case(MetadataKind::Report);
    fs::write(
        descriptor_path(root.path(), report),
        r#"<mdclass:Report xmlns:mdclass="urn:test" uuid="payload-report">
    <name>PayloadReport</name>
    <synonym><content>Broken</synonym>
</mdclass:Report>"#,
    )
    .expect("malformed descriptor must be written");

    let error = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(root.path())
        .expect_err("malformed synonym XML must fail the build");

    assert!(matches!(
        error,
        EdtGraphError::MetadataObject(EdtMetadataObjectError::MalformedXml(_))
    ));
}
