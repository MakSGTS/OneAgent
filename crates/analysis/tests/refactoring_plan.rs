use oneagent_analysis::refactoring::{
    ConfinedSourcePath, DEFAULT_REFACTORING_PREVIEW_ENTRIES, MAX_REFACTORING_CANDIDATES,
    MAX_REFACTORING_DEPENDENCIES, MAX_REFACTORING_OPERATIONS, MAX_REFACTORING_PREVIEW_ENTRIES,
    MAX_SOURCE_IDENTIFIER_BYTES, OperationId, RefactoringBound, RefactoringCompleteness,
    RefactoringErrorKind, RefactoringFamily, RefactoringOperation, RefactoringOperationKind,
    RefactoringPlan, RefactoringPreconditionSet, RefactoringPreview, RefactoringPreviewEntry,
    RefactoringRequest, RefactoringSourcePrecondition, RefactoringTarget, SourceByteRange,
    SourceContentVersion, SourceDocumentId, SourceOccurrence, SourceOccurrenceKind,
    SourceOccurrenceResolution, WorkspacePublicationId,
};
use oneagent_bsl::{BslSymbolKind, bsl_callable_id};
use oneagent_common::{EntityId, SourcePath, SourcePosition, SourceSpan};
use oneagent_graph::NodeKind;

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
