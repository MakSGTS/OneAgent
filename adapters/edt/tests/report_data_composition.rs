use oneagent_common::{EntityId, EntityName};
use oneagent_edt::{
    EdtGraphError, EdtReportDataCompositionError, EdtSemanticGraphBuilder,
    FileSystemEdtSemanticGraphBuilder,
};
use oneagent_graph::{
    DataSetKind, EdgeKind, FactOrigin, NodeId, NodeKind, NodeModifiedAspect, ResolutionState,
    SemanticDiagnosticCode, SemanticGraph, SemanticImpactAnalyzer, SemanticImpactOptions,
    data_composition_field_id, data_set_id, data_set_query_id,
};
use oneagent_metadata::MetadataKind;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::{TempDir, tempdir};

const DCS_ROOT_OPEN: &str = r#"<DataCompositionSchema xmlns="http://v8.1c.ru/8.1/data-composition-system/schema" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">"#;

struct GeneratedProject {
    _root: TempDir,
    path: PathBuf,
}

impl GeneratedProject {
    fn new() -> Self {
        let root = tempdir().expect("temporary project root must be created");
        let path = root.path().to_path_buf();
        let configuration_directory = path.join("src/Configuration");
        fs::create_dir_all(&configuration_directory)
            .expect("configuration directory must be created");
        fs::write(
            configuration_directory.join("Configuration.mdo"),
            r#"<mdclass:Configuration xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass" uuid="configuration-id"><name>GeneratedConfiguration</name></mdclass:Configuration>"#,
        )
        .expect("configuration descriptor must be written");
        Self { _root: root, path }
    }

    fn write_report(
        &self,
        report_name: &str,
        report_id: &str,
        templates: &str,
        main: &str,
        artifacts: &[(&str, &str)],
    ) {
        let directory = self.path.join("src/Reports").join(report_name);
        fs::create_dir_all(&directory).expect("Report directory must be created");
        fs::write(
            directory.join(format!("{report_name}.mdo")),
            format!(
                r#"<mdclass:Report xmlns:mdclass="http://g5.1c.ru/v8/dt/metadata/mdclass" uuid="{report_id}"><name>{report_name}</name>{main}{templates}</mdclass:Report>"#,
            ),
        )
        .expect("Report descriptor must be written");
        for (name, xml) in artifacts {
            let template_directory = directory.join("Templates").join(name);
            fs::create_dir_all(&template_directory).expect("template directory must be created");
            fs::write(template_directory.join("Template.dcs"), xml)
                .expect("DCS artifact must be written");
        }
    }

    fn rewrite_artifact(&self, report: &str, template: &str, xml: &str) {
        fs::write(
            self.path
                .join("src/Reports")
                .join(report)
                .join("Templates")
                .join(template)
                .join("Template.dcs"),
            xml,
        )
        .expect("DCS artifact must be rewritten");
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

fn id(value: &str) -> EntityId {
    EntityId::new(value).expect("identifier must be valid")
}

fn name(value: &str) -> EntityName {
    EntityName::new(value).expect("name must be valid")
}

fn template(id: &str, name: &str) -> String {
    format!(
        r#"<templates uuid="{id}"><name>{name}</name><templateType>DataCompositionSchema</templateType></templates>"#
    )
}

fn schema(body: &str) -> String {
    format!("{DCS_ROOT_OPEN}{body}</DataCompositionSchema>")
}

fn data_source() -> &'static str {
    "<dataSource><name>DataSource1</name><dataSourceType>Local</dataSourceType></dataSource>"
}

fn field(name: &str, path: &str) -> String {
    format!(
        r#"<field xsi:type="DataSetFieldField"><dataPath>{path}</dataPath><field>{name}</field></field>"#
    )
}

fn query_data_set(name: &str, fields: &str, query: &str) -> String {
    format!(
        r#"<dataSet xsi:type="DataSetQuery"><name>{name}</name>{fields}<dataSource>DataSource1</dataSource><query>{query}</query></dataSet>"#
    )
}

fn query_transition_schema(field_path: &str, query: &str) -> String {
    schema(&format!(
        "{}{}",
        data_source(),
        query_data_set("DataSet", &field("Field", field_path), query,)
    ))
}

fn object_transition_schema(field_path: &str) -> String {
    schema(&format!(
        r#"{}<dataSet xsi:type="DataSetObject"><name>DataSet</name>{}<dataSource>DataSource1</dataSource><objectName>RuntimeTable</objectName></dataSet>"#,
        data_source(),
        field("Field", field_path)
    ))
}

fn build(project: &GeneratedProject) -> oneagent_edt::EdtSemanticGraphBuildResult {
    FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(project.path())
        .expect("generated production project must build")
}

fn rich_project() -> GeneratedProject {
    let project = GeneratedProject::new();
    let templates = [
        template("schema-query", "QuerySchema"),
        template("schema-object", "ObjectSchema"),
        template("schema-union", "UnionSchema"),
        template("schema-empty", "EmptySchema"),
    ]
    .join("");
    let query_schema = schema(&format!(
        "{}{}",
        data_source(),
        query_data_set("QuerySet", &field("QueryField", "Query.Path"), "SELECT 1")
    ));
    let object_schema = schema(&format!(
        r#"{}<dataSet xsi:type="DataSetObject"><name>ObjectSet</name>{}<dataSource>DataSource1</dataSource><objectName>RuntimeTable</objectName></dataSet>"#,
        data_source(),
        field("ObjectField", "Object.Path")
    ));
    let union_schema = schema(&format!(
        r#"{}<dataSet xsi:type="DataSetUnion"><name>UnionSet</name>{}</dataSet>"#,
        data_source(),
        field("UnionField", "Union.Path")
    ));
    project.write_report(
        "RichReport",
        "report-rich",
        &templates,
        "<mainDataCompositionSchema>Report.RichReport.Template.QuerySchema</mainDataCompositionSchema>",
        &[
            ("QuerySchema", &query_schema),
            ("ObjectSchema", &object_schema),
            ("UnionSchema", &union_schema),
            ("EmptySchema", &schema("")),
        ],
    );
    project
}

fn production_fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/sprint12_report_data_composition_project")
}

fn single_report_project(main: bool, artifact: &str) -> GeneratedProject {
    let project = GeneratedProject::new();
    let main = if main {
        "<mainDataCompositionSchema>Report.TransitionReport.Template.Main</mainDataCompositionSchema>"
    } else {
        ""
    };
    project.write_report(
        "TransitionReport",
        "report-transition",
        &template("schema-transition", "Main"),
        main,
        &[("Main", artifact)],
    );
    project
}

#[test]
fn live_derived_fixture_is_typed_consumer_visible_and_deterministic() {
    let fixture = production_fixture();
    let first = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(&fixture)
        .expect("live-derived Report Data Composition fixture must build");
    let repeated = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(&fixture)
        .expect("repeated Report Data Composition fixture build must succeed");

    assert_fixture_shape(&first);
    assert_builds_equal(&first, &repeated);
    assert!(first.diff(&repeated).is_empty());
}

fn assert_fixture_shape(result: &oneagent_edt::EdtSemanticGraphBuildResult) {
    let graph = result.graph();
    let query = graph.query();
    assert_eq!(
        query.nodes_by_kind(NodeKind::DataCompositionSchema).len(),
        7
    );
    assert_eq!(query.nodes_by_kind(NodeKind::DataSet).len(), 6);
    assert_eq!(query.nodes_by_kind(NodeKind::DataCompositionField).len(), 6);
    assert_eq!(query.nodes_by_kind(NodeKind::Query).len(), 3);
    assert_eq!(query.edges_by_kind(EdgeKind::Contains).len(), 29);
    assert_fixture_payloads(graph);
    assert_fixture_diagnostics_and_relation_boundary(result);
    assert_eq!(
        result.report().nodes().by_kind()[&NodeKind::DataCompositionSchema],
        7
    );
    assert_eq!(result.report().nodes().by_kind()[&NodeKind::DataSet], 6);
    assert_eq!(
        result.report().nodes().by_kind()[&NodeKind::DataCompositionField],
        6
    );
    assert!(result.validate().is_valid());
}

fn assert_fixture_payloads(graph: &SemanticGraph) {
    let query_schema = id("b4233d51-daa9-47ff-8b51-f65c31fc8037");
    let query_data_set = data_set_id(&query_schema, &name("DataSet"))
        .expect("fixture Data Set identity must be valid");
    let query_id =
        data_set_query_id(&query_data_set).expect("fixture Query identity must be valid");
    assert!(
        graph
            .node(&query_schema)
            .and_then(oneagent_graph::GraphNode::data_composition_schema_payload)
            .expect("fixture Schema payload must exist")
            .is_main()
    );
    assert_eq!(
        graph
            .node(&query_data_set)
            .and_then(oneagent_graph::GraphNode::data_set_payload)
            .expect("fixture Data Set payload must exist")
            .kind(),
        DataSetKind::Query
    );
    assert_eq!(
        graph
            .query()
            .owner(&NodeId::new(query_id.as_str()))
            .expect("fixture Data Set must own Query")
            .id(),
        &query_data_set
    );
    let financial_schema = graph
        .node(&id("5f25a4ab-1a3e-4676-ab32-d3c92e7e39e6"))
        .and_then(oneagent_graph::GraphNode::data_composition_schema_payload)
        .expect("non-main fixture Schema payload must exist");
    assert!(!financial_schema.is_main());
    assert!(
        graph
            .node(&query_id)
            .expect("fixture Query must exist")
            .provenance()
            .iter()
            .any(|provenance| provenance.source().is_some_and(|source| {
                source.as_str().contains("AccessGroups.Profile AS Profile")
            }))
    );
}

fn assert_fixture_diagnostics_and_relation_boundary(
    result: &oneagent_edt::EdtSemanticGraphBuildResult,
) {
    let codes = result
        .diagnostics()
        .iter()
        .map(oneagent_graph::SemanticDiagnostic::code)
        .collect::<BTreeSet<_>>();
    assert_eq!(
        codes,
        BTreeSet::from([
            SemanticDiagnosticCode::DataCompositionNestedDataSetDeferred,
            SemanticDiagnosticCode::DataCompositionFieldFolderDeferred,
        ])
    );
    assert_eq!(result.reference_statistics().total(), 2);
    assert_eq!(result.reference_statistics().unsupported_prefix(), 2);
    assert!(result.reference_requests().is_empty());
    for kind in [EdgeKind::Reads, EdgeKind::DependsOn, EdgeKind::References] {
        assert!(result.graph().query().edges_by_kind(kind).is_empty());
    }
}

#[test]
fn production_emits_typed_main_non_main_empty_query_object_and_union_semantics() {
    let project = rich_project();
    let result = build(&project);
    let graph = result.graph();
    let query = graph.query();
    let report_id = id("report-rich");
    let query_schema_id = id("schema-query");
    let query_data_set_id =
        data_set_id(&query_schema_id, &name("QuerySet")).expect("Data Set identity must be valid");
    let query_field_id = data_composition_field_id(&query_data_set_id, &name("QueryField"))
        .expect("Field identity must be valid");
    let metadata_query_id =
        data_set_query_id(&query_data_set_id).expect("Query identity must be valid");

    assert_eq!(
        graph
            .node(&report_id)
            .expect("existing Report node must remain")
            .kind(),
        NodeKind::Metadata(MetadataKind::Report)
    );
    assert_eq!(
        query.nodes_by_kind(NodeKind::DataCompositionSchema).len(),
        4
    );
    assert_eq!(query.nodes_by_kind(NodeKind::DataSet).len(), 3);
    assert_eq!(query.nodes_by_kind(NodeKind::DataCompositionField).len(), 3);
    assert_eq!(query.nodes_by_kind(NodeKind::Query).len(), 1);
    assert!(
        graph
            .node(&query_schema_id)
            .expect("Query Schema must exist")
            .data_composition_schema_payload()
            .expect("Schema payload must exist")
            .is_main()
    );
    assert_eq!(
        graph
            .node(&query_data_set_id)
            .expect("Query Data Set must exist")
            .data_set_payload()
            .expect("Data Set payload must exist")
            .kind(),
        DataSetKind::Query
    );
    assert_eq!(
        graph
            .node(&query_field_id)
            .expect("Field must exist")
            .data_composition_field_payload()
            .expect("Field payload must exist")
            .data_path()
            .as_str(),
        "Query.Path"
    );
    assert_eq!(
        query
            .owner(&NodeId::new(metadata_query_id.as_str()))
            .expect("Data Set must own Query")
            .id(),
        &query_data_set_id
    );
    assert_eq!(
        query
            .owner(&NodeId::new(query_schema_id.as_str()))
            .expect("Report must own Schema")
            .id(),
        &report_id
    );
    assert_emitted_provenance(graph);
    assert_eq!(query.edges_by_kind(EdgeKind::Contains).len(), 12);
    for kind in [EdgeKind::Reads, EdgeKind::DependsOn, EdgeKind::References] {
        assert!(query.edges_by_kind(kind).is_empty());
    }
    assert!(result.diagnostics().is_empty());
    assert!(result.reference_requests().is_empty());
    assert_eq!(result.reference_statistics().total(), 0);
    assert!(result.validate().is_valid());
}

fn assert_emitted_provenance(graph: &SemanticGraph) {
    assert!(
        graph
            .nodes()
            .filter(|node| {
                matches!(
                    node.kind(),
                    NodeKind::DataCompositionSchema
                        | NodeKind::DataSet
                        | NodeKind::DataCompositionField
                        | NodeKind::Query
                )
            })
            .all(|node| {
                !node.provenance().is_empty()
                    && node.provenance().iter().all(|provenance| {
                        provenance.producer().as_str()
                            == "oneagent.edt.report-data-composition-emission"
                            && provenance.origin() == FactOrigin::Parsed
                            && provenance.resolution() == ResolutionState::NotApplicable
                            && provenance.source().is_some_and(|source| {
                                source
                                    .as_str()
                                    .starts_with("src/Reports/RichReport/Templates/")
                            })
                    })
            })
    );
}

#[test]
fn production_payload_transitions_preserve_identity_and_have_exact_diff_scope() {
    let baseline_artifact = query_transition_schema("Path.Before", "SELECT 1");
    let project = single_report_project(false, &baseline_artifact);
    let baseline = build(&project);
    project.write_report(
        "TransitionReport",
        "report-transition",
        &template("schema-transition", "Main"),
        "<mainDataCompositionSchema>Report.TransitionReport.Template.Main</mainDataCompositionSchema>",
        &[("Main", &baseline_artifact)],
    );
    let main = build(&project);
    let object_artifact = object_transition_schema("Path.Before");
    project.write_report(
        "TransitionReport",
        "report-transition",
        &template("schema-transition", "Main"),
        "",
        &[("Main", &object_artifact)],
    );
    let object = build(&project);
    let field_artifact = query_transition_schema("Path.After", "SELECT 1");
    project.write_report(
        "TransitionReport",
        "report-transition",
        &template("schema-transition", "Main"),
        "",
        &[("Main", &field_artifact)],
    );
    let field_changed = build(&project);
    let schema_id = id("schema-transition");
    let data_set_id =
        data_set_id(&schema_id, &name("DataSet")).expect("Data Set identity must be valid");
    let field_id = data_composition_field_id(&data_set_id, &name("Field"))
        .expect("Field identity must be valid");
    let query_id = data_set_query_id(&data_set_id).expect("Query identity must be valid");

    let main_diff = baseline.diff(&main);
    assert_modified_content_and_provenance(main_diff.graph(), &schema_id);
    assert_eq!(main_diff.graph().modified_nodes().len(), 1);

    let kind_diff = baseline.diff(&object);
    assert_modified_content_and_provenance(kind_diff.graph(), &data_set_id);
    assert_eq!(kind_diff.graph().modified_nodes().len(), 1);
    assert_eq!(kind_diff.graph().removed_nodes().len(), 1);
    assert_eq!(
        kind_diff.graph().removed_nodes()[0].id().as_str(),
        query_id.as_str()
    );

    let field_diff = baseline.diff(&field_changed);
    assert_modified_content_and_provenance(field_diff.graph(), &field_id);
    assert_eq!(field_diff.graph().modified_nodes().len(), 1);
    for result in [&baseline, &main, &object, &field_changed] {
        assert!(result.validate().is_valid());
        assert!(result.reference_requests().is_empty());
    }
}

fn assert_modified_content_and_provenance(
    diff: &oneagent_graph::SemanticGraphDiff,
    expected: &EntityId,
) {
    let change = diff
        .modified_nodes()
        .iter()
        .find(|change| change.id().as_str() == expected.as_str())
        .expect("stable entity must be modified");
    assert!(
        change
            .modified_aspects()
            .contains(&NodeModifiedAspect::SemanticContent)
    );
    assert!(
        change
            .modified_aspects()
            .contains(&NodeModifiedAspect::Provenance)
    );
}

#[test]
fn deferred_observation_transition_changes_only_build_level_evidence() {
    let accepted = schema(&format!(
        r#"{}<dataSet xsi:type="DataSetUnion"><name>Union</name>{}</dataSet>"#,
        data_source(),
        field("Field", "Path")
    ));
    let deferred = schema(&format!(
        r#"{}<dataSet xsi:type="DataSetUnion"><name>Union</name>{}<field xsi:type="DataSetFieldFolder"><dataPath>Folder</dataPath></field><dataSet xsi:type="DataSetQuery"><name>Nested</name><dataSource>DataSource1</dataSource><query>SELECT 1</query></dataSet></dataSet>"#,
        data_source(),
        field("Field", "Path")
    ));
    let project = single_report_project(true, &accepted);
    let before = build(&project);
    project.rewrite_artifact("TransitionReport", "Main", &deferred);
    let current = build(&project);
    let diff = before.diff(&current);

    assert!(diff.graph().is_empty());
    assert_eq!(diff.diagnostics().added().len(), 2);
    assert_eq!(diff.reference_requests().summary().total_changes(), 0);
    assert_eq!(before.reference_statistics().total(), 0);
    assert_eq!(current.reference_statistics().total(), 2);
    assert_eq!(
        before
            .graph()
            .query()
            .nodes_by_kind(NodeKind::DataSet)
            .len(),
        1
    );
    assert_eq!(
        current
            .graph()
            .query()
            .nodes_by_kind(NodeKind::DataSet)
            .len(),
        1
    );
    assert!(before.validate().is_valid());
    assert!(current.validate().is_valid());
}

#[test]
fn query_text_changes_preserve_query_identity_and_modify_only_source_evidence() {
    let project = rich_project();
    let before = build(&project);
    let query_schema_id = id("schema-query");
    let data_set_id =
        data_set_id(&query_schema_id, &name("QuerySet")).expect("Data Set identity must be valid");
    let query_id = data_set_query_id(&data_set_id).expect("Query identity must be valid");
    let changed_query_schema = schema(&format!(
        "{}{}",
        data_source(),
        query_data_set("QuerySet", &field("QueryField", "Query.Path"), "SELECT 2",)
    ));

    project.rewrite_artifact("RichReport", "QuerySchema", &changed_query_schema);
    let current = build(&project);
    let diff = before.diff(&current);
    let changed = diff
        .graph()
        .modified_nodes()
        .iter()
        .find(|change| change.id().as_str() == query_id.as_str())
        .expect("stable Query identity must report modified evidence");

    assert!(diff.graph().added_nodes().is_empty());
    assert!(diff.graph().removed_nodes().is_empty());
    assert_eq!(
        changed.modified_aspects(),
        &[NodeModifiedAspect::Provenance]
    );
    assert_eq!(
        before
            .graph()
            .node(&query_id)
            .expect("before Query must exist")
            .kind(),
        NodeKind::Query
    );
    assert_eq!(
        current
            .graph()
            .node(&query_id)
            .expect("current Query must exist")
            .kind(),
        NodeKind::Query
    );
    let impact = SemanticImpactAnalyzer::analyze(
        before.graph(),
        current.graph(),
        diff.graph(),
        &SemanticImpactOptions::new(4),
    )
    .expect("opaque Query evidence impact must succeed");
    assert_eq!(impact.affected_nodes().len(), 1);
    assert_eq!(
        impact.affected_nodes()[0].node_id().as_str(),
        query_id.as_str()
    );
}

#[test]
fn deferred_and_unsupported_observations_are_diagnostic_counted_without_placeholders() {
    let project = GeneratedProject::new();
    let artifact = schema(&format!(
        r#"{}<dataSet xsi:type="DataSetUnion"><name>Union</name>
  <field xsi:type="DataSetFieldFolder"><dataPath>Folder</dataPath></field>
  <field xsi:type="FutureField"><field>Future</field></field>
  <dataSet xsi:type="DataSetQuery"><name>Nested</name><dataSource>DataSource1</dataSource><query>SELECT 1</query></dataSet>
</dataSet><dataSet xsi:type="FutureDataSet"><name>Future</name></dataSet>"#,
        data_source()
    ));
    project.write_report(
        "DeferredReport",
        "report-deferred",
        &template("schema-deferred", "Main"),
        "",
        &[("Main", &artifact)],
    );

    let result = build(&project);
    let graph = result.graph();
    let codes = result
        .diagnostics()
        .iter()
        .map(oneagent_graph::SemanticDiagnostic::code)
        .collect::<BTreeSet<_>>();

    assert_eq!(result.diagnostics().len(), 4);
    assert_eq!(
        codes,
        BTreeSet::from([
            SemanticDiagnosticCode::DataCompositionNestedDataSetDeferred,
            SemanticDiagnosticCode::DataCompositionFieldFolderDeferred,
            SemanticDiagnosticCode::DataCompositionUnsupportedDataSetType,
            SemanticDiagnosticCode::DataCompositionUnsupportedFieldType,
        ])
    );
    assert!(
        result
            .diagnostics()
            .iter()
            .all(|diagnostic| !diagnostic.provenance().is_empty())
    );
    assert_eq!(result.reference_statistics().total(), 4);
    assert_eq!(result.reference_statistics().unsupported_prefix(), 4);
    assert!(result.reference_requests().is_empty());
    assert_eq!(graph.query().nodes_by_kind(NodeKind::DataSet).len(), 1);
    assert!(graph.query().nodes_by_kind(NodeKind::Query).is_empty());
    assert!(graph.query().nodes_by_kind(NodeKind::Unknown).is_empty());
    assert!(graph.nodes().all(|node| !node.name().as_str().is_empty()));
    for kind in [EdgeKind::Reads, EdgeKind::DependsOn, EdgeKind::References] {
        assert!(graph.query().edges_by_kind(kind).is_empty());
    }
    assert!(result.validate().is_valid());
}

#[test]
fn fatal_report_data_composition_source_error_returns_no_build_result() {
    let project = GeneratedProject::new();
    project.write_report(
        "BrokenReport",
        "report-broken",
        &template("schema-broken", "Missing"),
        "",
        &[],
    );

    let error = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_diagnostics(project.path())
        .expect_err("missing declared DCS artifact must be fatal");

    assert!(matches!(
        error,
        EdtGraphError::ReportDataComposition(EdtReportDataCompositionError::MissingArtifact(_))
    ));
}

#[test]
fn repeated_and_reordered_production_builds_are_equal() {
    let project = rich_project();
    let first = build(&project);
    let repeated = build(&project);
    assert_builds_equal(&first, &repeated);

    let reordered_query_schema = schema(&format!(
        "{}{}",
        query_data_set("QuerySet", &field("QueryField", "Query.Path"), "SELECT 1"),
        data_source()
    ));
    project.rewrite_artifact("RichReport", "QuerySchema", &reordered_query_schema);
    let reordered = build(&project);

    assert_builds_equal(&first, &reordered);
}

fn assert_builds_equal(
    left: &oneagent_edt::EdtSemanticGraphBuildResult,
    right: &oneagent_edt::EdtSemanticGraphBuildResult,
) {
    assert!(left.graph().diff(right.graph()).is_empty());
    assert_eq!(left.diagnostics(), right.diagnostics());
    assert_eq!(left.reference_requests(), right.reference_requests());
    assert_eq!(left.reference_statistics(), right.reference_statistics());
    assert_eq!(left.report(), right.report());
    assert_eq!(left.validate(), right.validate());
}
