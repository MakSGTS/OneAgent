use std::sync::atomic::{AtomicBool, Ordering};

use oneagent_analysis::refactoring::{
    BslModuleRole, ConfinedSourcePath, DEFAULT_REFACTORING_PREVIEW_ENTRIES,
    MAX_REFACTORING_CANDIDATES, MAX_REFACTORING_DEPENDENCIES, MAX_REFACTORING_OPERATIONS,
    MAX_REFACTORING_PREVIEW_ENTRIES, MAX_SOURCE_IDENTIFIER_BYTES, NeverCancelledRefactoring,
    OperationId, RefactoringBound, RefactoringCancellationSignal, RefactoringCompleteness,
    RefactoringErrorKind, RefactoringFamily, RefactoringOperation, RefactoringOperationKind,
    RefactoringPlan, RefactoringPlanner, RefactoringPlannerInput, RefactoringPreconditionSet,
    RefactoringPreview, RefactoringPreviewEntry, RefactoringRequest, RefactoringSourcePrecondition,
    RefactoringTarget, SourceByteRange, SourceContentVersion, SourceDocument, SourceDocumentId,
    SourceEvidenceCompleteness, SourceEvidenceSet, SourceFormat, SourceOccurrence,
    SourceOccurrenceKind, SourceOccurrenceResolution, WorkspacePublicationId,
};
use oneagent_bsl::{BslSymbolKind, bsl_callable_id};
use oneagent_common::{EntityId, EntityName, SourcePath, SourcePosition, SourceSpan};
use oneagent_graph::{GraphEdge, GraphNode, NodeKind, SemanticGraph};
use oneagent_metadata::MetadataKind;

const CONFIGURATION: &str = "configuration.main";
const MODULE: &str = "module.main";
const OLD_NAME: &str = "OldName";
const NEW_NAME: &str = "NewName";

fn id(value: impl Into<String>) -> EntityId {
    EntityId::new(value).expect("identity must be valid")
}

fn document_id(module: &str) -> SourceDocumentId {
    SourceDocumentId::new(id(CONFIGURATION), id(module)).expect("document ID must be valid")
}

fn target_id() -> EntityId {
    bsl_callable_id(&id(MODULE), BslSymbolKind::Procedure, OLD_NAME)
        .expect("target ID must be valid")
}

fn occurrence(
    document_id: &SourceDocumentId,
    version: SourceContentVersion,
    range: SourceByteRange,
) -> SourceOccurrence {
    SourceOccurrence::new(
        document_id.clone(),
        version,
        range,
        SourceOccurrenceKind::Declaration,
        OLD_NAME,
        Some(target_id()),
        SourceOccurrenceResolution::Unique,
    )
    .expect("declaration occurrence must be valid")
}

#[derive(Clone)]
struct Fixture {
    request: RefactoringRequest,
    target: RefactoringTarget,
    preconditions: RefactoringPreconditionSet,
    declaration: RefactoringOperation,
    local_call: RefactoringOperation,
    qualified_call: RefactoringOperation,
}

fn fixture() -> Fixture {
    let document_id = document_id(MODULE);
    let source = b"Procedure OldName()\nOldName(); Module.OldName();\nEndProcedure\n";
    let version = SourceContentVersion::from_bytes(source);
    let declaration_range = SourceByteRange::new(10, 17).expect("range must be valid");
    let declaration_occurrence = occurrence(&document_id, version, declaration_range);
    let request = RefactoringRequest::new(
        RefactoringFamily::BslCallableRenameV1,
        WorkspacePublicationId::initial(),
        id(CONFIGURATION),
        target_id(),
        NEW_NAME,
    )
    .expect("request must be valid");
    let target = RefactoringTarget::new(
        id(CONFIGURATION),
        target_id(),
        NodeKind::Procedure,
        id(MODULE),
        declaration_occurrence,
        NEW_NAME,
    )
    .expect("target must be valid");
    let preconditions = RefactoringPreconditionSet::new(
        WorkspacePublicationId::initial(),
        id(CONFIGURATION),
        target_id(),
        NodeKind::Procedure,
        id(MODULE),
        vec![RefactoringSourcePrecondition::new(
            document_id.clone(),
            version,
        )],
    )
    .expect("preconditions must be valid");
    let declaration = operation(
        RefactoringOperationKind::ReplaceDeclarationIdentifier,
        SourceOccurrenceKind::Declaration,
        &document_id,
        version,
        10,
        17,
        OLD_NAME,
        NEW_NAME,
        &[],
    );
    let local_call = operation(
        RefactoringOperationKind::ReplaceDirectCallIdentifier,
        SourceOccurrenceKind::LocalCall,
        &document_id,
        version,
        20,
        27,
        OLD_NAME,
        NEW_NAME,
        &[],
    );
    let qualified_call = operation(
        RefactoringOperationKind::ReplaceDirectCallIdentifier,
        SourceOccurrenceKind::QualifiedCall,
        &document_id,
        version,
        38,
        45,
        OLD_NAME,
        NEW_NAME,
        &[],
    );
    Fixture {
        request,
        target,
        preconditions,
        declaration,
        local_call,
        qualified_call,
    }
}

#[allow(clippy::too_many_arguments)]
fn operation(
    kind: RefactoringOperationKind,
    occurrence_kind: SourceOccurrenceKind,
    document_id: &SourceDocumentId,
    version: SourceContentVersion,
    start: usize,
    end: usize,
    expected: &str,
    replacement: &str,
    dependencies: &[OperationId],
) -> RefactoringOperation {
    RefactoringOperation::new(
        kind,
        occurrence_kind,
        document_id.clone(),
        version,
        SourceByteRange::new(start, end).expect("range must be valid"),
        expected,
        replacement,
        dependencies,
    )
    .expect("operation must be valid")
}

fn complete_plan(fixture: &Fixture) -> RefactoringPlan {
    RefactoringPlan::new(
        fixture.request.clone(),
        fixture.target.clone(),
        fixture.preconditions.clone(),
        vec![
            fixture.declaration.clone(),
            fixture.local_call.clone(),
            fixture.qualified_call.clone(),
        ],
    )
    .expect("complete plan must be valid")
}

fn confined_path() -> ConfinedSourcePath {
    ConfinedSourcePath::new(
        SourcePath::new("configuration/CommonModules/Main/Ext/Module.bsl")
            .expect("path must be valid"),
        &SourcePath::new("configuration").expect("root must be valid"),
    )
    .expect("path must be confined")
}

fn span(line: u32, start: u32, end: u32) -> SourceSpan {
    SourceSpan::new(
        SourcePosition::new(line, start).expect("start must be valid"),
        SourcePosition::new(line, end).expect("end must be valid"),
    )
    .expect("span must be valid")
}

#[test]
fn public_domain_constructs_complete_plan_summary_and_read_only_preview() {
    let fixture = fixture();
    let plan = complete_plan(&fixture);

    assert_eq!(plan.completeness(), RefactoringCompleteness::Complete);
    assert_eq!(plan.completeness().as_str(), "complete");
    assert_eq!(plan.id().as_str().len(), 64);
    assert!(
        plan.id()
            .as_str()
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    );
    assert_eq!(plan.request().desired_name(), NEW_NAME);
    assert_eq!(plan.target().target_node_id(), &target_id());
    assert_eq!(plan.target().target_kind(), NodeKind::Procedure);
    assert_eq!(plan.operations().len(), 3);
    assert_eq!(plan.operations()[0].range().start_byte(), 38);
    assert_eq!(plan.operations()[1].range().start_byte(), 20);
    assert_eq!(plan.operations()[2].range().start_byte(), 10);

    let summary = plan.summary();
    assert_eq!(summary.requested_targets(), 1);
    assert_eq!(summary.planned_targets(), 1);
    assert_eq!(summary.conflicted_targets(), 0);
    assert_eq!(summary.rejected_targets(), 0);
    assert_eq!(summary.documents(), 1);
    assert_eq!(summary.candidate_occurrences(), 3);
    assert_eq!(summary.exact_duplicates_collapsed(), 0);
    assert_eq!(summary.declaration_operations(), 1);
    assert_eq!(summary.local_call_operations(), 1);
    assert_eq!(summary.qualified_call_operations(), 1);
    assert_eq!(summary.planned_operations(), 3);
    assert_eq!(summary.omitted_operations(), 0);
    assert_eq!(summary.returned_operations(), 3);

    let entries = plan
        .operations()
        .iter()
        .enumerate()
        .map(|(index, operation)| {
            let line = u32::try_from(index)
                .expect("preview fixture index must fit u32")
                .checked_add(1)
                .expect("preview fixture line must fit u32");
            RefactoringPreviewEntry::new(operation, confined_path(), span(line, 1, 8))
                .expect("preview entry must be valid")
        })
        .collect();
    let preview = RefactoringPreview::new(&plan, entries).expect("preview must be complete");
    assert_eq!(preview.plan_id(), plan.id());
    assert_eq!(preview.completeness(), RefactoringCompleteness::Complete);
    assert_eq!(preview.entries().len(), plan.operations().len());
    assert_eq!(preview.entries()[0].replacement(), NEW_NAME);
    assert_eq!(
        preview.entries()[0].path().path().as_str(),
        "configuration/CommonModules/Main/Ext/Module.bsl"
    );
}

#[test]
fn request_name_bounds_grammar_reserved_words_and_errors_are_redacted() {
    let exact = "a".repeat(MAX_SOURCE_IDENTIFIER_BYTES);
    RefactoringRequest::new(
        RefactoringFamily::BslCallableRenameV1,
        WorkspacePublicationId::initial(),
        id(CONFIGURATION),
        target_id(),
        exact,
    )
    .expect("exact desired-name bound must pass");

    let secret = format!("{}secret-source", "a".repeat(MAX_SOURCE_IDENTIFIER_BYTES));
    let over = RefactoringRequest::new(
        RefactoringFamily::BslCallableRenameV1,
        WorkspacePublicationId::initial(),
        id(CONFIGURATION),
        target_id(),
        &secret,
    )
    .expect_err("one-over desired-name bound must fail");
    assert_eq!(over.kind(), RefactoringErrorKind::BoundExceeded);
    assert_eq!(over.bound(), Some(RefactoringBound::IdentifierBytes));
    assert_eq!(over.actual(), Some(secret.len()));
    assert_eq!(over.maximum(), Some(MAX_SOURCE_IDENTIFIER_BYTES));
    assert!(!over.to_string().contains("secret-source"));

    for invalid in [
        "",
        "1Name",
        "Name.Value",
        "Name Value",
        "procedure",
        "ЭКСПОРТ",
    ] {
        let error = RefactoringRequest::new(
            RefactoringFamily::BslCallableRenameV1,
            WorkspacePublicationId::initial(),
            id(CONFIGURATION),
            target_id(),
            invalid,
        )
        .expect_err("invalid desired name must fail");
        assert_eq!(error.kind(), RefactoringErrorKind::InvalidDesiredName);
        if !invalid.is_empty() {
            assert!(!error.to_string().contains(invalid));
        }
    }
}

#[test]
fn target_and_preconditions_reject_unsupported_or_conflicting_evidence() {
    let fixture = fixture();
    let declaration = fixture.target.declaration().clone();
    let unsupported = RefactoringTarget::new(
        id(CONFIGURATION),
        target_id(),
        NodeKind::Module,
        id(MODULE),
        declaration,
        NEW_NAME,
    )
    .expect_err("unsupported target kind must fail");
    assert_eq!(unsupported.kind(), RefactoringErrorKind::UnsupportedTarget);

    let document = fixture.preconditions.documents()[0].clone();
    let conflicting = RefactoringSourcePrecondition::new(
        document.document_id().clone(),
        SourceContentVersion::from_bytes(b"different"),
    );
    let error = RefactoringPreconditionSet::new(
        WorkspacePublicationId::initial(),
        id(CONFIGURATION),
        target_id(),
        NodeKind::Procedure,
        id(MODULE),
        vec![document.clone(), conflicting],
    )
    .expect_err("two versions for one document must fail");
    assert_eq!(error.kind(), RefactoringErrorKind::IncompatibleEvidence);

    let canonical = RefactoringPreconditionSet::new(
        WorkspacePublicationId::initial(),
        id(CONFIGURATION),
        target_id(),
        NodeKind::Procedure,
        id(MODULE),
        vec![document.clone(), document],
    )
    .expect("exact precondition duplicates must collapse");
    assert_eq!(canonical.documents().len(), 1);
}

#[test]
fn operation_bounds_and_forbidden_dependencies_are_exact() {
    let fixture = fixture();
    assert_eq!(fixture.declaration.dependencies(), &[]);
    assert_eq!(MAX_REFACTORING_DEPENDENCIES, 0);

    let dependency_error = RefactoringOperation::new(
        RefactoringOperationKind::ReplaceDirectCallIdentifier,
        SourceOccurrenceKind::LocalCall,
        fixture.declaration.document_id().clone(),
        fixture.declaration.content_version(),
        SourceByteRange::new(50, 57).expect("range must be valid"),
        OLD_NAME,
        NEW_NAME,
        &[fixture.declaration.id().clone()],
    )
    .expect_err("one-over dependency bound must fail");
    assert_eq!(dependency_error.kind(), RefactoringErrorKind::BoundExceeded);
    assert_eq!(
        dependency_error.bound(),
        Some(RefactoringBound::DependencyEdges)
    );
    assert_eq!(dependency_error.actual(), Some(1));
    assert_eq!(dependency_error.maximum(), Some(0));

    let exact = "a".repeat(MAX_SOURCE_IDENTIFIER_BYTES);
    RefactoringOperation::new(
        RefactoringOperationKind::ReplaceDirectCallIdentifier,
        SourceOccurrenceKind::LocalCall,
        fixture.declaration.document_id().clone(),
        fixture.declaration.content_version(),
        SourceByteRange::new(50, 57).expect("range must be valid"),
        exact.clone(),
        exact,
        &[],
    )
    .expect("exact operation identifier bound must pass");
    let over = RefactoringOperation::new(
        RefactoringOperationKind::ReplaceDirectCallIdentifier,
        SourceOccurrenceKind::LocalCall,
        fixture.declaration.document_id().clone(),
        fixture.declaration.content_version(),
        SourceByteRange::new(50, 57).expect("range must be valid"),
        "a".repeat(MAX_SOURCE_IDENTIFIER_BYTES + 1),
        NEW_NAME,
        &[],
    )
    .expect_err("one-over operation identifier bound must fail");
    assert_eq!(over.kind(), RefactoringErrorKind::BoundExceeded);
}

#[test]
fn reordered_and_repeated_operations_have_stable_identity_order_and_summary() {
    let fixture = fixture();
    let first = RefactoringPlan::new(
        fixture.request.clone(),
        fixture.target.clone(),
        fixture.preconditions.clone(),
        vec![
            fixture.declaration.clone(),
            fixture.local_call.clone(),
            fixture.qualified_call.clone(),
            fixture.local_call.clone(),
        ],
    )
    .expect("plan with one exact duplicate must pass");
    let reordered = RefactoringPlan::new(
        fixture.request.clone(),
        fixture.target.clone(),
        fixture.preconditions.clone(),
        vec![
            fixture.local_call.clone(),
            fixture.qualified_call.clone(),
            fixture.local_call.clone(),
            fixture.declaration.clone(),
        ],
    )
    .expect("reordered plan must pass");

    assert_eq!(first, reordered);
    assert_eq!(first.id(), reordered.id());
    assert_eq!(first.summary().candidate_occurrences(), 4);
    assert_eq!(first.summary().exact_duplicates_collapsed(), 1);
    assert_eq!(first.operations().len(), 3);
    assert_eq!(first.operations()[0].id().as_str().len(), 64);

    let changed = RefactoringRequest::new(
        RefactoringFamily::BslCallableRenameV1,
        WorkspacePublicationId::initial(),
        id(CONFIGURATION),
        target_id(),
        "DifferentName",
    )
    .expect("changed request must be valid");
    assert_ne!(first.request(), &changed);
}

#[test]
fn duplicate_overlap_version_and_summary_conflicts_fail_atomically() {
    let fixture = fixture();
    let same_range = operation(
        RefactoringOperationKind::ReplaceDirectCallIdentifier,
        SourceOccurrenceKind::LocalCall,
        fixture.declaration.document_id(),
        fixture.declaration.content_version(),
        10,
        17,
        "OLDNAME",
        NEW_NAME,
        &[],
    );
    let duplicate_conflict = RefactoringPlan::new(
        fixture.request.clone(),
        fixture.target.clone(),
        fixture.preconditions.clone(),
        vec![fixture.declaration.clone(), same_range],
    )
    .expect_err("unequal same-range operations must fail");
    assert_eq!(
        duplicate_conflict.kind(),
        RefactoringErrorKind::DuplicateConflict
    );

    let overlap = operation(
        RefactoringOperationKind::ReplaceDirectCallIdentifier,
        SourceOccurrenceKind::LocalCall,
        fixture.declaration.document_id(),
        fixture.declaration.content_version(),
        15,
        22,
        OLD_NAME,
        NEW_NAME,
        &[],
    );
    let overlap_error = RefactoringPlan::new(
        fixture.request.clone(),
        fixture.target.clone(),
        fixture.preconditions.clone(),
        vec![fixture.declaration.clone(), overlap],
    )
    .expect_err("intersecting ranges must fail");
    assert_eq!(
        overlap_error.kind(),
        RefactoringErrorKind::OverlappingOperations
    );

    let conflicting_classification = operation(
        RefactoringOperationKind::ReplaceDirectCallIdentifier,
        SourceOccurrenceKind::QualifiedCall,
        fixture.local_call.document_id(),
        fixture.local_call.content_version(),
        fixture.local_call.range().start_byte(),
        fixture.local_call.range().end_byte(),
        fixture.local_call.expected(),
        fixture.local_call.replacement(),
        &[],
    );
    let identity_collision = RefactoringPlan::new(
        fixture.request.clone(),
        fixture.target.clone(),
        fixture.preconditions.clone(),
        vec![
            fixture.declaration.clone(),
            fixture.local_call.clone(),
            conflicting_classification,
        ],
    )
    .expect_err("equal IDs with unequal source classification must fail");
    assert_eq!(
        identity_collision.kind(),
        RefactoringErrorKind::IdentityCollision
    );

    let stale = operation(
        RefactoringOperationKind::ReplaceDirectCallIdentifier,
        SourceOccurrenceKind::LocalCall,
        fixture.declaration.document_id(),
        SourceContentVersion::from_bytes(b"stale"),
        20,
        27,
        OLD_NAME,
        NEW_NAME,
        &[],
    );
    let stale_error = RefactoringPlan::new(
        fixture.request,
        fixture.target,
        fixture.preconditions,
        vec![fixture.declaration, stale],
    )
    .expect_err("stale operation version must fail");
    assert_eq!(stale_error.kind(), RefactoringErrorKind::StaleSourceVersion);
}

#[test]
fn exact_and_one_over_candidate_bounds_are_checked_before_normalization() {
    let fixture = fixture();
    assert_eq!(MAX_REFACTORING_CANDIDATES, MAX_REFACTORING_OPERATIONS);
    let exact = RefactoringPlan::new(
        fixture.request.clone(),
        fixture.target.clone(),
        fixture.preconditions.clone(),
        vec![fixture.declaration.clone(); MAX_REFACTORING_CANDIDATES],
    )
    .expect("exact candidate bound must pass before duplicate collapse");
    assert_eq!(
        exact.summary().candidate_occurrences(),
        MAX_REFACTORING_CANDIDATES
    );
    assert_eq!(exact.operations().len(), 1);

    let over = RefactoringPlan::new(
        fixture.request,
        fixture.target,
        fixture.preconditions,
        vec![fixture.declaration; MAX_REFACTORING_CANDIDATES + 1],
    )
    .expect_err("one-over candidate bound must fail atomically");
    assert_eq!(over.kind(), RefactoringErrorKind::BoundExceeded);
    assert_eq!(over.bound(), Some(RefactoringBound::CandidateOccurrences));
    assert_eq!(over.actual(), Some(MAX_REFACTORING_CANDIDATES + 1));
    assert_eq!(over.maximum(), Some(MAX_REFACTORING_CANDIDATES));
}

#[test]
fn no_change_and_incomplete_or_reordered_preview_fail_closed() {
    let fixture = fixture();
    let no_change_request = RefactoringRequest::new(
        RefactoringFamily::BslCallableRenameV1,
        WorkspacePublicationId::initial(),
        id(CONFIGURATION),
        target_id(),
        "oldname",
    )
    .expect("case-only request shape is valid before target comparison");
    let no_change_target = RefactoringTarget::new(
        id(CONFIGURATION),
        target_id(),
        NodeKind::Procedure,
        id(MODULE),
        fixture.target.declaration().clone(),
        "oldname",
    )
    .expect("case-only target shape must be valid");
    let no_change = RefactoringPlan::new(
        no_change_request,
        no_change_target,
        fixture.preconditions.clone(),
        vec![fixture.declaration.clone()],
    )
    .expect_err("case-only rename must fail as no change");
    assert_eq!(no_change.kind(), RefactoringErrorKind::NoChange);

    let plan = complete_plan(&fixture);
    let entries = plan
        .operations()
        .iter()
        .enumerate()
        .map(|(index, operation)| {
            let line = u32::try_from(index)
                .expect("preview fixture index must fit u32")
                .checked_add(1)
                .expect("preview fixture line must fit u32");
            RefactoringPreviewEntry::new(operation, confined_path(), span(line, 1, 8))
                .expect("preview entry must be valid")
        })
        .collect::<Vec<_>>();
    let incomplete = RefactoringPreview::new(&plan, entries[..2].to_vec())
        .expect_err("incomplete preview must fail");
    assert_eq!(
        incomplete.kind(),
        RefactoringErrorKind::IncompatibleEvidence
    );

    let reordered = RefactoringPreview::new(
        &plan,
        vec![entries[1].clone(), entries[0].clone(), entries[2].clone()],
    )
    .expect_err("reordered preview must fail");
    assert_eq!(reordered.kind(), RefactoringErrorKind::IncompatibleEvidence);

    assert_eq!(DEFAULT_REFACTORING_PREVIEW_ENTRIES, 50);
    assert_eq!(MAX_REFACTORING_PREVIEW_ENTRIES, 100);
    assert_eq!(
        RefactoringBound::PreviewEntries.maximum(),
        MAX_REFACTORING_PREVIEW_ENTRIES
    );
}

fn name(value: &str) -> EntityName {
    EntityName::new(value).expect("name must be valid")
}

fn insert_node(graph: &mut SemanticGraph, identity: &EntityId, value: &str, kind: NodeKind) {
    graph.insert_node(GraphNode::new(identity.clone(), name(value), kind));
}

fn insert_contains(graph: &mut SemanticGraph, owner: &EntityId, child: &EntityId) {
    graph
        .insert_edge(GraphEdge::new(
            owner.clone(),
            child.clone(),
            oneagent_graph::EdgeKind::Contains,
        ))
        .expect("ownership endpoints must exist");
}

struct PlannerFixture {
    configuration_id: EntityId,
    graph: SemanticGraph,
    evidence: SourceEvidenceSet,
    request: RefactoringRequest,
}

impl PlannerFixture {
    fn input(&self) -> RefactoringPlannerInput<'_> {
        RefactoringPlannerInput::new(
            WorkspacePublicationId::initial(),
            &self.configuration_id,
            &self.graph,
            &self.evidence,
        )
    }
}

fn planner_fixture(reverse: bool) -> PlannerFixture {
    planner_fixture_for(reverse, OLD_NAME, NEW_NAME, NodeKind::Procedure)
}

fn planner_fixture_for(
    reverse: bool,
    old_name: &str,
    new_name: &str,
    target_kind: NodeKind,
) -> PlannerFixture {
    let configuration_id = id(CONFIGURATION);
    let module_id = id(MODULE);
    let symbol_kind = match target_kind {
        NodeKind::Procedure => BslSymbolKind::Procedure,
        NodeKind::Function => BslSymbolKind::Function,
        _ => panic!("planner fixture requires a supported callable kind"),
    };
    let callable_id = bsl_callable_id(&module_id, symbol_kind, old_name)
        .expect("fixture target identity must be valid");
    let mut graph = SemanticGraph::new();
    let nodes = [
        (
            configuration_id.clone(),
            "Main",
            NodeKind::Metadata(MetadataKind::Configuration),
        ),
        (module_id.clone(), "Main", NodeKind::Module),
        (callable_id.clone(), old_name, target_kind),
    ];
    let node_order: Box<dyn Iterator<Item = _>> = if reverse {
        Box::new(nodes.into_iter().rev())
    } else {
        Box::new(nodes.into_iter())
    };
    for (identity, node_name, kind) in node_order {
        insert_node(&mut graph, &identity, node_name, kind);
    }
    insert_contains(&mut graph, &configuration_id, &module_id);
    insert_contains(&mut graph, &module_id, &callable_id);

    let keyword = match target_kind {
        NodeKind::Procedure => "Procedure",
        NodeKind::Function => "Function",
        _ => unreachable!("fixture kind was validated above"),
    };
    let raw = format!(
        "\u{feff}{keyword} {old_name}()\r\n{old_name}(); Module.{old_name}();\rEnd{keyword}\n"
    )
    .into_bytes();
    let version = SourceContentVersion::from_bytes(&raw);
    let document_id = document_id(MODULE);
    let source = std::str::from_utf8(&raw).expect("planner fixture must be UTF-8");
    let mut matches = source.match_indices(old_name);
    let mut occurrences = [
        SourceOccurrenceKind::Declaration,
        SourceOccurrenceKind::LocalCall,
        SourceOccurrenceKind::QualifiedCall,
    ]
    .into_iter()
    .map(|kind| {
        let (start, token) = matches.next().expect("fixture occurrence must exist");
        SourceOccurrence::new(
            document_id.clone(),
            version,
            SourceByteRange::new(start, start + token.len()).expect("range must be valid"),
            kind,
            token,
            Some(callable_id.clone()),
            SourceOccurrenceResolution::Unique,
        )
        .expect("fixture occurrence must be valid")
    })
    .collect::<Vec<_>>();
    if reverse {
        occurrences.reverse();
    }
    let document = SourceDocument::new(
        document_id,
        SourceFormat::Edt,
        BslModuleRole::Common,
        ConfinedSourcePath::new(
            SourcePath::new("configuration/CommonModules/Main/Module.bsl")
                .expect("path must be valid"),
            &SourcePath::new("configuration").expect("root must be valid"),
        )
        .expect("path must be confined"),
        raw,
        occurrences,
        SourceEvidenceCompleteness::BslCallableRenameV1,
    )
    .expect("source document must be valid");
    let evidence = SourceEvidenceSet::new(configuration_id.clone(), vec![document])
        .expect("source evidence must be valid");
    let request = RefactoringRequest::new(
        RefactoringFamily::BslCallableRenameV1,
        WorkspacePublicationId::initial(),
        configuration_id.clone(),
        callable_id,
        new_name,
    )
    .expect("request must be valid");
    PlannerFixture {
        configuration_id,
        graph,
        evidence,
        request,
    }
}

#[test]
fn planner_evaluates_graph_target_source_preconditions_and_mixed_line_preview() {
    let fixture = planner_fixture(false);
    let evaluation = RefactoringPlanner
        .evaluate(
            fixture.input(),
            &fixture.request,
            &NeverCancelledRefactoring,
        )
        .expect("complete Graph-backed evaluation must pass");

    assert_eq!(evaluation.plan().operations().len(), 3);
    assert_eq!(evaluation.plan().summary().declaration_operations(), 1);
    assert_eq!(evaluation.plan().summary().local_call_operations(), 1);
    assert_eq!(evaluation.plan().summary().qualified_call_operations(), 1);
    assert_eq!(evaluation.plan().summary().omitted_operations(), 0);
    assert_eq!(evaluation.preview().entries().len(), 3);
    let positions = evaluation
        .preview()
        .entries()
        .iter()
        .map(|entry| {
            (
                entry.position().start().line(),
                entry.position().start().column(),
                entry.position().end().line(),
                entry.position().end().column(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(positions, [(2, 19, 2, 26), (2, 1, 2, 8), (1, 11, 1, 18)]);
    assert_eq!(
        evaluation.preview().entries()[0].path().path().as_str(),
        "configuration/CommonModules/Main/Module.bsl"
    );
}

#[test]
fn planner_is_equal_across_reordered_graph_evidence_and_fresh_repetition() {
    let first = planner_fixture(false);
    let reordered = planner_fixture(true);
    let planner = RefactoringPlanner;
    let first_result = planner
        .evaluate(first.input(), &first.request, &NeverCancelledRefactoring)
        .expect("first evaluation must pass");
    let repeated = planner
        .evaluate(first.input(), &first.request, &NeverCancelledRefactoring)
        .expect("repeated evaluation must pass");
    let reordered_result = planner
        .evaluate(
            reordered.input(),
            &reordered.request,
            &NeverCancelledRefactoring,
        )
        .expect("reordered evaluation must pass");

    assert_eq!(first_result, repeated);
    assert_eq!(first_result, reordered_result);
    assert_eq!(first_result.plan().id(), reordered_result.plan().id());
    assert_eq!(first_result.preview(), reordered_result.preview());
}

struct Cancellation(AtomicBool);

impl RefactoringCancellationSignal for Cancellation {
    fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }
}

#[test]
fn planner_cancellation_publication_and_configuration_failures_are_atomic() {
    let fixture = planner_fixture(false);
    let cancelled = Cancellation(AtomicBool::new(true));
    let selected_cancellation_error = RefactoringPlanner
        .evaluate_selected_configuration(
            WorkspacePublicationId::initial(),
            None,
            &fixture.request,
            &cancelled,
        )
        .expect_err("selection entry cancellation must precede missing Configuration");
    assert_eq!(
        selected_cancellation_error.kind(),
        RefactoringErrorKind::Cancelled
    );
    let cancellation_error = RefactoringPlanner
        .evaluate(fixture.input(), &fixture.request, &cancelled)
        .expect_err("entry cancellation must fail without output");
    assert_eq!(cancellation_error.kind(), RefactoringErrorKind::Cancelled);

    let stale_request = RefactoringRequest::new(
        RefactoringFamily::BslCallableRenameV1,
        WorkspacePublicationId::new(2).expect("publication must be non-zero"),
        fixture.configuration_id.clone(),
        target_id(),
        NEW_NAME,
    )
    .expect("request must be valid");
    let selected_publication_error = RefactoringPlanner
        .evaluate_selected_configuration(
            WorkspacePublicationId::initial(),
            None,
            &stale_request,
            &NeverCancelledRefactoring,
        )
        .expect_err("publication mismatch must precede missing Configuration");
    assert_eq!(
        selected_publication_error.kind(),
        RefactoringErrorKind::PublicationMismatch
    );
    let publication_error = RefactoringPlanner
        .evaluate(fixture.input(), &stale_request, &NeverCancelledRefactoring)
        .expect_err("stale publication must fail without output");
    assert_eq!(
        publication_error.kind(),
        RefactoringErrorKind::PublicationMismatch
    );

    let missing_configuration = RefactoringRequest::new(
        RefactoringFamily::BslCallableRenameV1,
        WorkspacePublicationId::initial(),
        id("configuration.missing"),
        target_id(),
        NEW_NAME,
    )
    .expect("request must be valid");
    let selected_configuration_error = RefactoringPlanner
        .evaluate_selected_configuration(
            WorkspacePublicationId::initial(),
            None,
            &missing_configuration,
            &NeverCancelledRefactoring,
        )
        .expect_err("missing selected Configuration must fail without output");
    assert_eq!(
        selected_configuration_error.kind(),
        RefactoringErrorKind::ConfigurationNotFound
    );
    let configuration_error = RefactoringPlanner
        .evaluate(
            fixture.input(),
            &missing_configuration,
            &NeverCancelledRefactoring,
        )
        .expect_err("missing Configuration must fail without output");
    assert_eq!(
        configuration_error.kind(),
        RefactoringErrorKind::ConfigurationNotFound
    );
}

#[test]
fn planner_rejects_ambiguous_owner_name_and_identity_collisions() {
    let mut ambiguous = planner_fixture(false);
    let second_module = id("module.second");
    insert_node(
        &mut ambiguous.graph,
        &second_module,
        "Second",
        NodeKind::Module,
    );
    insert_contains(
        &mut ambiguous.graph,
        &ambiguous.configuration_id,
        &second_module,
    );
    insert_contains(&mut ambiguous.graph, &second_module, &target_id());
    let owner_error = RefactoringPlanner
        .evaluate(
            ambiguous.input(),
            &ambiguous.request,
            &NeverCancelledRefactoring,
        )
        .expect_err("multiple owners must fail");
    assert_eq!(owner_error.kind(), RefactoringErrorKind::AmbiguousOwner);

    let mut name_collision = planner_fixture(false);
    let sibling = id("module.main:function:another");
    insert_node(
        &mut name_collision.graph,
        &sibling,
        NEW_NAME,
        NodeKind::Function,
    );
    insert_contains(&mut name_collision.graph, &id(MODULE), &sibling);
    let name_error = RefactoringPlanner
        .evaluate(
            name_collision.input(),
            &name_collision.request,
            &NeverCancelledRefactoring,
        )
        .expect_err("equivalent sibling name must fail");
    assert_eq!(name_error.kind(), RefactoringErrorKind::NameCollision);

    let mut identity_collision = planner_fixture(false);
    let expected = bsl_callable_id(&id(MODULE), BslSymbolKind::Procedure, NEW_NAME)
        .expect("expected identity must be valid");
    insert_node(
        &mut identity_collision.graph,
        &expected,
        "DifferentIdentityOwner",
        NodeKind::Unknown,
    );
    let identity_error = RefactoringPlanner
        .evaluate(
            identity_collision.input(),
            &identity_collision.request,
            &NeverCancelledRefactoring,
        )
        .expect_err("occupied post-rename identity must fail");
    assert_eq!(
        identity_error.kind(),
        RefactoringErrorKind::IdentityCollision
    );
}

#[test]
fn planner_rejects_missing_ambiguous_and_incompatible_source_evidence() {
    let mut missing = planner_fixture(false);
    missing.evidence = SourceEvidenceSet::new(missing.configuration_id.clone(), Vec::new())
        .expect("empty evidence shape must be constructible");
    let missing_error = RefactoringPlanner
        .evaluate(
            missing.input(),
            &missing.request,
            &NeverCancelledRefactoring,
        )
        .expect_err("empty evidence must fail");
    assert_eq!(
        missing_error.kind(),
        RefactoringErrorKind::SourceEvidenceMissing
    );

    let mut ambiguous = planner_fixture(false);
    let original = &ambiguous.evidence.documents()[0];
    let local_call_range = original.occurrences()[1].range();
    let ambiguous_occurrence = SourceOccurrence::new(
        original.id().clone(),
        original.content_version(),
        local_call_range,
        SourceOccurrenceKind::LocalCall,
        OLD_NAME,
        None,
        SourceOccurrenceResolution::Ambiguous,
    )
    .expect("ambiguous occurrence shape must be valid");
    let mut occurrences = original.occurrences().to_vec();
    occurrences[1] = ambiguous_occurrence;
    let document = SourceDocument::new(
        original.id().clone(),
        original.format(),
        original.module_role(),
        original.path().clone(),
        original.raw_content().to_vec(),
        occurrences,
        original.completeness(),
    )
    .expect("ambiguous complete ledger must remain structurally valid");
    ambiguous.evidence = SourceEvidenceSet::new(ambiguous.configuration_id.clone(), vec![document])
        .expect("ambiguous source set must be valid");
    let ambiguous_error = RefactoringPlanner
        .evaluate(
            ambiguous.input(),
            &ambiguous.request,
            &NeverCancelledRefactoring,
        )
        .expect_err("target-related ambiguous call must fail");
    assert_eq!(
        ambiguous_error.kind(),
        RefactoringErrorKind::AmbiguousOccurrence
    );

    let incompatible = planner_fixture(false);
    let other_evidence = SourceEvidenceSet::new(id("configuration.other"), Vec::new())
        .expect("other empty evidence must be valid");
    let input = RefactoringPlannerInput::new(
        WorkspacePublicationId::initial(),
        &incompatible.configuration_id,
        &incompatible.graph,
        &other_evidence,
    );
    let incompatible_error = RefactoringPlanner
        .evaluate(input, &incompatible.request, &NeverCancelledRefactoring)
        .expect_err("mismatched source Configuration must fail");
    assert_eq!(
        incompatible_error.kind(),
        RefactoringErrorKind::IncompatibleEvidence
    );
}

#[test]
fn planner_supports_russian_function_and_rejects_missing_unsupported_and_no_change_targets() {
    let russian = planner_fixture_for(false, "СтараяФункция", "НоваяФункция", NodeKind::Function);
    let evaluation = RefactoringPlanner
        .evaluate(
            russian.input(),
            &russian.request,
            &NeverCancelledRefactoring,
        )
        .expect("Russian Function rename must pass");
    assert_eq!(evaluation.plan().target().target_kind(), NodeKind::Function);
    assert_eq!(evaluation.plan().operations().len(), 3);
    assert!(
        evaluation
            .preview()
            .entries()
            .iter()
            .all(|entry| entry.replacement() == "НоваяФункция")
    );

    let fixture = planner_fixture(false);
    let missing_request = RefactoringRequest::new(
        RefactoringFamily::BslCallableRenameV1,
        WorkspacePublicationId::initial(),
        fixture.configuration_id.clone(),
        id("callable.missing"),
        NEW_NAME,
    )
    .expect("missing-target request must be valid");
    let missing = RefactoringPlanner
        .evaluate(
            fixture.input(),
            &missing_request,
            &NeverCancelledRefactoring,
        )
        .expect_err("missing target must fail");
    assert_eq!(missing.kind(), RefactoringErrorKind::TargetNotFound);

    let unsupported_request = RefactoringRequest::new(
        RefactoringFamily::BslCallableRenameV1,
        WorkspacePublicationId::initial(),
        fixture.configuration_id.clone(),
        id(MODULE),
        NEW_NAME,
    )
    .expect("unsupported-target request must be valid");
    let unsupported = RefactoringPlanner
        .evaluate(
            fixture.input(),
            &unsupported_request,
            &NeverCancelledRefactoring,
        )
        .expect_err("Module target must fail");
    assert_eq!(unsupported.kind(), RefactoringErrorKind::UnsupportedTarget);

    let no_change_request = RefactoringRequest::new(
        RefactoringFamily::BslCallableRenameV1,
        WorkspacePublicationId::initial(),
        fixture.configuration_id.clone(),
        target_id(),
        "oldname",
    )
    .expect("case-only request shape must be valid");
    let no_change = RefactoringPlanner
        .evaluate(
            fixture.input(),
            &no_change_request,
            &NeverCancelledRefactoring,
        )
        .expect_err("case-only target rename must fail");
    assert_eq!(no_change.kind(), RefactoringErrorKind::NoChange);
}

fn candidate_bound_document(
    index: usize,
    occurrence_count: usize,
    include_target_declaration: bool,
) -> SourceDocument {
    let module = if include_target_declaration {
        id(MODULE)
    } else {
        id(format!("module.bound.{index:02}"))
    };
    let document_id = SourceDocumentId::new(id(CONFIGURATION), module)
        .expect("bounded document identity must be valid");
    let mut raw = Vec::with_capacity(
        occurrence_count
            .saturating_mul(2)
            .saturating_add(OLD_NAME.len()),
    );
    let mut ranges = Vec::with_capacity(occurrence_count);
    if include_target_declaration {
        raw.extend_from_slice(OLD_NAME.as_bytes());
        ranges.push((
            SourceOccurrenceKind::Declaration,
            0,
            OLD_NAME.len(),
            OLD_NAME,
            target_id(),
        ));
        raw.push(b' ');
    }
    while ranges.len() < occurrence_count {
        let start = raw.len();
        raw.extend_from_slice(b"X ");
        ranges.push((
            SourceOccurrenceKind::LocalCall,
            start,
            start + 1,
            "X",
            id("callable.unrelated"),
        ));
    }
    let version = SourceContentVersion::from_bytes(&raw);
    let occurrences = ranges
        .into_iter()
        .map(|(kind, start, end, token, mapped_target)| {
            SourceOccurrence::new(
                document_id.clone(),
                version,
                SourceByteRange::new(start, end).expect("range must be valid"),
                kind,
                token,
                Some(mapped_target),
                SourceOccurrenceResolution::Unique,
            )
            .expect("bounded occurrence must be valid")
        })
        .collect();
    SourceDocument::new(
        document_id,
        SourceFormat::Edt,
        BslModuleRole::Common,
        ConfinedSourcePath::new(
            SourcePath::new(format!("configuration/Bound/{index:02}.bsl"))
                .expect("path must be valid"),
            &SourcePath::new("configuration").expect("root must be valid"),
        )
        .expect("path must be confined"),
        raw,
        occurrences,
        SourceEvidenceCompleteness::BslCallableRenameV1,
    )
    .expect("bounded source document must be valid")
}

#[test]
fn planner_accepts_exact_candidate_bound_and_rejects_one_over_atomically() {
    let mut fixture = planner_fixture(false);
    let documents_at_exact_bound = (0..16)
        .map(|index| candidate_bound_document(index, 4_096, index == 0))
        .collect::<Vec<_>>();
    assert_eq!(
        documents_at_exact_bound
            .iter()
            .map(|document| document.occurrences().len())
            .sum::<usize>(),
        MAX_REFACTORING_CANDIDATES
    );
    fixture.evidence = SourceEvidenceSet::new(
        fixture.configuration_id.clone(),
        documents_at_exact_bound.clone(),
    )
    .expect("exact candidate bound evidence must be valid");
    let exact = RefactoringPlanner
        .evaluate(
            fixture.input(),
            &fixture.request,
            &NeverCancelledRefactoring,
        )
        .expect("exact admitted candidate bound must pass");
    assert_eq!(exact.plan().operations().len(), 1);

    let mut one_over_documents = documents_at_exact_bound;
    one_over_documents.push(candidate_bound_document(16, 1, false));
    fixture.evidence = SourceEvidenceSet::new(fixture.configuration_id.clone(), one_over_documents)
        .expect("one-over planner evidence must remain valid source evidence");
    let one_over = RefactoringPlanner
        .evaluate(
            fixture.input(),
            &fixture.request,
            &NeverCancelledRefactoring,
        )
        .expect_err("one-over admitted candidate bound must fail atomically");
    assert_eq!(one_over.kind(), RefactoringErrorKind::BoundExceeded);
    assert_eq!(
        one_over.bound(),
        Some(RefactoringBound::CandidateOccurrences)
    );
    assert_eq!(one_over.actual(), Some(MAX_REFACTORING_CANDIDATES + 1));
}
