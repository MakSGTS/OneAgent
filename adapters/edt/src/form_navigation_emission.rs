//! Resolution and graph emission for typed static Form-navigation observations.

use oneagent_common::EntityId;
use oneagent_graph::{
    Confidence, EdgeKind, FactOrigin, GraphEdge, GraphError, NodeKind, ProducerId, Provenance,
    ResolutionError, ResolutionState, SemanticDiagnostic, SemanticDiagnosticCode,
    SemanticDiagnosticKind, SemanticDiagnosticSeverity, SemanticGraph, SemanticReference,
    SemanticReferenceOutcome, SemanticReferenceStatistics, SemanticResolutionIndex,
};
use oneagent_metadata::MetadataKind;
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::PathBuf;

use crate::form_navigation::{
    EdtFormNavigationCandidate, EdtFormNavigationOutcomeKind, EdtFormNavigationParseOutcome,
    EdtFormNavigationRejection, EdtFormNavigationRejectionReason, EdtFormNavigationTarget,
    EdtFormNavigationUnsupportedTargetReason, extract_form_navigation_candidates,
};
use crate::query_source_resolution::WorkspaceResolutionScope;
use crate::{EdtModuleDescriptor, EdtModuleKind};

const FORM_NAVIGATION_PRODUCER: &str = "oneagent.edt.form-navigation";

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct CollectedFormNavigation {
    outcomes: Vec<EdtFormNavigationParseOutcome>,
}

pub(crate) fn collect_form_navigation(
    modules: &[EdtModuleDescriptor],
) -> Result<CollectedFormNavigation, EdtFormNavigationEmissionError> {
    let mut modules = modules.iter().collect::<Vec<_>>();
    modules.sort_by(|left, right| left.id().cmp(right.id()));
    let mut outcomes = Vec::new();
    for module in modules {
        let source = fs::read_to_string(module.path()).map_err(|source| {
            EdtFormNavigationEmissionError::ReadModule {
                path: module.path().to_path_buf(),
                source,
            }
        })?;
        outcomes.extend(extract_form_navigation_candidates(module, &source));
    }
    outcomes.sort_by(compare_outcomes);
    Ok(CollectedFormNavigation { outcomes })
}

fn compare_outcomes(
    left: &EdtFormNavigationParseOutcome,
    right: &EdtFormNavigationParseOutcome,
) -> Ordering {
    let (left_module, left_line, left_column, left_statement) = outcome_location(left);
    let (right_module, right_line, right_column, right_statement) = outcome_location(right);
    left_module
        .cmp(right_module)
        .then_with(|| left_line.cmp(&right_line))
        .then_with(|| left_column.cmp(&right_column))
        .then_with(|| left.kind().cmp(&right.kind()))
        .then_with(|| left_statement.cmp(right_statement))
}

fn outcome_location(outcome: &EdtFormNavigationParseOutcome) -> (&EntityId, usize, usize, &str) {
    match outcome {
        EdtFormNavigationParseOutcome::Candidate(candidate) => (
            &candidate.module_id,
            candidate.location.line,
            candidate.location.column,
            &candidate.raw_statement,
        ),
        EdtFormNavigationParseOutcome::Rejected(rejection) => (
            &rejection.module_id,
            rejection.location.line,
            rejection.location.column,
            &rejection.raw_statement,
        ),
    }
}

pub(crate) fn emit_form_navigation(
    graph: &mut SemanticGraph,
    collected: &CollectedFormNavigation,
    workspace_scope: WorkspaceResolutionScope,
    diagnostics: &mut BTreeSet<SemanticDiagnostic>,
    statistics: &mut SemanticReferenceStatistics,
) -> Result<(), EdtFormNavigationEmissionError> {
    let mut evidence_by_edge = BTreeMap::<(EntityId, EntityId), Vec<Provenance>>::new();
    {
        let index = SemanticResolutionIndex::new(graph);
        for outcome in &collected.outcomes {
            match outcome {
                EdtFormNavigationParseOutcome::Candidate(candidate) => process_candidate(
                    &index,
                    candidate,
                    workspace_scope,
                    &mut evidence_by_edge,
                    diagnostics,
                    statistics,
                )?,
                EdtFormNavigationParseOutcome::Rejected(rejection) => {
                    record_parser_rejection(rejection, diagnostics, statistics)?;
                }
            }
        }
    }

    for ((source, target), mut provenance) in evidence_by_edge {
        provenance.sort_by(|left, right| left.source().cmp(&right.source()));
        provenance.dedup();
        graph
            .insert_edge(GraphEdge::new_with_provenance(
                source,
                target,
                EdgeKind::Opens,
                provenance,
            ))
            .map_err(EdtFormNavigationEmissionError::Graph)?;
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn process_candidate(
    index: &SemanticResolutionIndex<'_>,
    candidate: &EdtFormNavigationCandidate,
    workspace_scope: WorkspaceResolutionScope,
    evidence_by_edge: &mut BTreeMap<(EntityId, EntityId), Vec<Provenance>>,
    diagnostics: &mut BTreeSet<SemanticDiagnostic>,
    statistics: &mut SemanticReferenceStatistics,
) -> Result<(), EdtFormNavigationEmissionError> {
    let command_id = match resolve_navigation_source(index, candidate) {
        Ok(command_id) => command_id,
        Err(error) => {
            record_source_diagnostic(candidate, error, diagnostics, statistics)?;
            return Ok(());
        }
    };
    let target = match resolve_navigation_target(index, candidate) {
        Ok(target) => target,
        Err(error) => {
            record_target_diagnostic(candidate, error, workspace_scope, diagnostics, statistics)?;
            return Ok(());
        }
    };

    evidence_by_edge
        .entry((candidate.procedure_id.clone(), target.clone()))
        .or_default()
        .push(navigation_provenance(
            candidate,
            Some(&command_id),
            Some(&target),
            "resolved",
            FactOrigin::Resolved,
            ResolutionState::Resolved,
        )?);
    statistics.record(SemanticReferenceOutcome::Resolved, true);
    Ok(())
}

fn resolve_navigation_source(
    index: &SemanticResolutionIndex<'_>,
    candidate: &EdtFormNavigationCandidate,
) -> Result<EntityId, NavigationSourceError> {
    let procedure = index
        .resolve_entity_id(&candidate.procedure_id)
        .map_err(NavigationSourceError::Procedure)?;
    if procedure.kind() != NodeKind::Procedure {
        return Err(NavigationSourceError::ProcedureKind {
            id: procedure.id().clone(),
            actual: procedure.kind(),
        });
    }
    let module = index
        .resolve_entity_id(&candidate.module_id)
        .map_err(NavigationSourceError::Module)?;
    if module.kind() != NodeKind::Module {
        return Err(NavigationSourceError::ModuleKind {
            id: module.id().clone(),
            actual: module.kind(),
        });
    }
    index
        .resolve_owned_child(module.id(), procedure.id())
        .map_err(NavigationSourceError::ProcedureOwnership)?;
    let command = index
        .resolve_owner(module.id())
        .map_err(NavigationSourceError::CommandOwnership)?;
    if !matches!(
        command.kind(),
        NodeKind::Command | NodeKind::Metadata(MetadataKind::Command)
    ) {
        return Err(NavigationSourceError::CommandKind {
            id: command.id().clone(),
            actual: command.kind(),
        });
    }
    Ok(command.id().clone())
}

fn resolve_navigation_target(
    index: &SemanticResolutionIndex<'_>,
    candidate: &EdtFormNavigationCandidate,
) -> Result<EntityId, NavigationTargetError> {
    match &candidate.target {
        EdtFormNavigationTarget::CommonForm { form_name } => index
            .resolve_name_of_kind(form_name, NodeKind::Metadata(MetadataKind::CommonForm))
            .map(|target| target.id().clone())
            .map_err(|source| NavigationTargetError::CommonForm { source }),
        EdtFormNavigationTarget::SubordinateForm {
            owner_kind,
            owner_name,
            form_name,
        } => {
            let owner = index
                .resolve_name_of_kind(owner_name, NodeKind::Metadata(*owner_kind))
                .map_err(|source| NavigationTargetError::Owner {
                    expected_kind: *owner_kind,
                    source,
                })?;
            index
                .resolve_child_of_kind(owner.id(), form_name, NodeKind::Form)
                .map(|target| target.id().clone())
                .map_err(|source| NavigationTargetError::Child {
                    owner: owner.id().clone(),
                    source,
                })
        }
    }
}

fn record_parser_rejection(
    rejection: &EdtFormNavigationRejection,
    diagnostics: &mut BTreeSet<SemanticDiagnostic>,
    statistics: &mut SemanticReferenceStatistics,
) -> Result<(), EdtFormNavigationEmissionError> {
    let (code, kind, outcome, message) = match rejection.reason {
        EdtFormNavigationRejectionReason::MalformedStatement
        | EdtFormNavigationRejectionReason::IncompleteStatement => (
            SemanticDiagnosticCode::ReferenceMalformedFormat,
            SemanticDiagnosticKind::MalformedReferenceFormat,
            SemanticReferenceOutcome::MalformedFormat,
            "static OpenForm statement is malformed or incomplete",
        ),
        EdtFormNavigationRejectionReason::UnsupportedTarget(_)
        | EdtFormNavigationRejectionReason::DynamicFirstArgument
        | EdtFormNavigationRejectionReason::UnsupportedModuleKind(_)
        | EdtFormNavigationRejectionReason::MissingContainingSymbol
        | EdtFormNavigationRejectionReason::UnsupportedContainingSymbol(_) => (
            SemanticDiagnosticCode::ReferenceUnsupportedPrefix,
            SemanticDiagnosticKind::UnsupportedReferencePrefix,
            SemanticReferenceOutcome::UnsupportedPrefix,
            "OpenForm observation is outside the accepted static navigation boundary",
        ),
    };
    let raw = rejection
        .literal
        .clone()
        .unwrap_or_else(|| rejection.raw_statement.clone());
    let mut diagnostic = SemanticDiagnostic::new(
        code,
        SemanticDiagnosticSeverity::Error,
        kind,
        message,
        SemanticReference::Raw(raw),
    )
    .with_provenance(vec![rejection_provenance(rejection)?]);
    diagnostic = diagnostic.with_source_node(
        rejection
            .containing_symbol_id
            .clone()
            .unwrap_or_else(|| rejection.module_id.clone()),
    );
    diagnostics.insert(diagnostic);
    statistics.record(outcome, true);
    Ok(())
}

fn record_source_diagnostic(
    candidate: &EdtFormNavigationCandidate,
    error: NavigationSourceError,
    diagnostics: &mut BTreeSet<SemanticDiagnostic>,
    statistics: &mut SemanticReferenceStatistics,
) -> Result<(), EdtFormNavigationEmissionError> {
    let (expected, actual, candidates) = match error {
        NavigationSourceError::Procedure(source)
        | NavigationSourceError::Module(source)
        | NavigationSourceError::ProcedureOwnership(source)
        | NavigationSourceError::CommandOwnership(source) => {
            return record_resolution_diagnostic(
                candidate,
                source,
                WorkspaceResolutionScope::Complete,
                "navigation_source",
                diagnostics,
                statistics,
            );
        }
        NavigationSourceError::ProcedureKind { id, actual } => {
            (vec![NodeKind::Procedure], Some(actual), vec![id])
        }
        NavigationSourceError::ModuleKind { id, actual } => {
            (vec![NodeKind::Module], Some(actual), vec![id])
        }
        NavigationSourceError::CommandKind { id, actual } => (
            vec![NodeKind::Command, NodeKind::Metadata(MetadataKind::Command)],
            Some(actual),
            vec![id],
        ),
    };
    diagnostics.insert(
        SemanticDiagnostic::new(
            SemanticDiagnosticCode::ReferenceIncompatibleKind,
            SemanticDiagnosticSeverity::Error,
            SemanticDiagnosticKind::IncompatibleTargetKind,
            "OpenForm source has an incompatible semantic kind",
            SemanticReference::Raw(candidate.literal.clone()),
        )
        .with_source_node(candidate.procedure_id.clone())
        .with_expected_kinds(expected)
        .with_candidates(candidates)
        .with_actual_kind(actual.expect("source kind diagnostic must retain actual kind"))
        .with_provenance(vec![navigation_provenance(
            candidate,
            None,
            None,
            "incompatible_source",
            FactOrigin::Resolved,
            ResolutionState::Unresolved,
        )?]),
    );
    statistics.record(SemanticReferenceOutcome::IncompatibleTargetKind, true);
    Ok(())
}

fn record_target_diagnostic(
    candidate: &EdtFormNavigationCandidate,
    error: NavigationTargetError,
    workspace_scope: WorkspaceResolutionScope,
    diagnostics: &mut BTreeSet<SemanticDiagnostic>,
    statistics: &mut SemanticReferenceStatistics,
) -> Result<(), EdtFormNavigationEmissionError> {
    let (stage, source) = match error {
        NavigationTargetError::CommonForm { source } => ("common_form", source),
        NavigationTargetError::Owner {
            expected_kind,
            source,
        } => (metadata_owner_stage(expected_kind), source),
        NavigationTargetError::Child { owner, source } => {
            return record_child_resolution_diagnostic(
                candidate,
                &owner,
                source,
                workspace_scope,
                diagnostics,
                statistics,
            );
        }
    };
    record_resolution_diagnostic(
        candidate,
        source,
        workspace_scope,
        stage,
        diagnostics,
        statistics,
    )
}

#[allow(clippy::too_many_arguments)]
fn record_child_resolution_diagnostic(
    candidate: &EdtFormNavigationCandidate,
    owner: &EntityId,
    error: ResolutionError,
    workspace_scope: WorkspaceResolutionScope,
    diagnostics: &mut BTreeSet<SemanticDiagnostic>,
    statistics: &mut SemanticReferenceStatistics,
) -> Result<(), EdtFormNavigationEmissionError> {
    record_resolution_diagnostic_with_context(
        candidate,
        error,
        workspace_scope,
        "subordinate_form",
        Some(owner),
        diagnostics,
        statistics,
    )
}

#[allow(clippy::too_many_arguments)]
fn record_resolution_diagnostic(
    candidate: &EdtFormNavigationCandidate,
    error: ResolutionError,
    workspace_scope: WorkspaceResolutionScope,
    stage: &str,
    diagnostics: &mut BTreeSet<SemanticDiagnostic>,
    statistics: &mut SemanticReferenceStatistics,
) -> Result<(), EdtFormNavigationEmissionError> {
    record_resolution_diagnostic_with_context(
        candidate,
        error,
        workspace_scope,
        stage,
        None,
        diagnostics,
        statistics,
    )
}

#[allow(clippy::too_many_arguments)]
fn record_resolution_diagnostic_with_context(
    candidate: &EdtFormNavigationCandidate,
    error: ResolutionError,
    workspace_scope: WorkspaceResolutionScope,
    stage: &str,
    owner: Option<&EntityId>,
    diagnostics: &mut BTreeSet<SemanticDiagnostic>,
    statistics: &mut SemanticReferenceStatistics,
) -> Result<(), EdtFormNavigationEmissionError> {
    let is_missing = matches!(&error, ResolutionError::MissingTarget { .. });
    let (mut diagnostic, outcome, resolution, outcome_name) =
        if is_missing && workspace_scope == WorkspaceResolutionScope::Partial {
            (
                SemanticDiagnostic::new(
                    SemanticDiagnosticCode::ReferenceUnresolved,
                    SemanticDiagnosticSeverity::Warning,
                    SemanticDiagnosticKind::UnresolvedTarget,
                    "static Form target is absent from the partial workspace",
                    SemanticReference::Raw(candidate.literal.clone()),
                ),
                SemanticReferenceOutcome::Unresolved,
                ResolutionState::Partial,
                "partial_workspace_absent",
            )
        } else {
            let outcome = match &error {
                ResolutionError::MissingTarget { .. } => SemanticReferenceOutcome::Unresolved,
                ResolutionError::AmbiguousTarget { .. } => SemanticReferenceOutcome::Ambiguous,
                ResolutionError::IncompatibleNodeKind { .. } => {
                    SemanticReferenceOutcome::IncompatibleTargetKind
                }
                ResolutionError::InvalidOwnerReference { .. } => {
                    SemanticReferenceOutcome::InvalidOwnerReference
                }
            };
            let resolution = match outcome {
                SemanticReferenceOutcome::Ambiguous => ResolutionState::Ambiguous,
                _ => ResolutionState::Unresolved,
            };
            let outcome_name = match outcome {
                SemanticReferenceOutcome::Unresolved => "missing",
                SemanticReferenceOutcome::Ambiguous => "ambiguous",
                SemanticReferenceOutcome::IncompatibleTargetKind => "incompatible",
                SemanticReferenceOutcome::InvalidOwnerReference => "invalid_owner",
                _ => unreachable!("only resolution failures enter navigation diagnostics"),
            };
            (
                SemanticDiagnostic::from_resolution_error_with_reference(
                    error,
                    Some(SemanticReference::Raw(candidate.literal.clone())),
                ),
                outcome,
                resolution,
                outcome_name,
            )
        };
    diagnostic = diagnostic
        .with_source_node(candidate.procedure_id.clone())
        .with_provenance(vec![navigation_provenance_with_context(
            candidate,
            None,
            None,
            stage,
            owner,
            outcome_name,
            FactOrigin::Resolved,
            resolution,
        )?]);
    diagnostics.insert(diagnostic);
    statistics.record(outcome, true);
    Ok(())
}

fn rejection_provenance(
    rejection: &EdtFormNavigationRejection,
) -> Result<Provenance, EdtFormNavigationEmissionError> {
    let mut context = base_context(
        rejection.module_path.to_string_lossy().as_ref(),
        rejection.module_id.as_str(),
        rejection
            .containing_symbol_id
            .as_ref()
            .map_or("", EntityId::as_str),
        rejection.raw_statement.as_str(),
        rejection.location.line,
        rejection.location.column,
    );
    append_context(
        &mut context,
        "module_kind",
        module_kind_name(rejection.module_kind),
    );
    append_context(
        &mut context,
        "callable_kind",
        rejection
            .containing_symbol_kind
            .map_or("missing", oneagent_bsl::BslSymbolKind::as_str),
    );
    append_context(
        &mut context,
        "callable_name",
        rejection
            .containing_symbol_name
            .as_ref()
            .map_or("", oneagent_common::EntityName::as_str),
    );
    append_context(
        &mut context,
        "rejection",
        rejection_reason_name(rejection.reason),
    );
    append_context(
        &mut context,
        "parser_outcome",
        outcome_kind_name(rejection.reason.outcome_kind()),
    );
    provenance_from_context(context, FactOrigin::Parsed, ResolutionState::Unresolved)
}

fn navigation_provenance(
    candidate: &EdtFormNavigationCandidate,
    command: Option<&EntityId>,
    target: Option<&EntityId>,
    outcome: &str,
    origin: FactOrigin,
    resolution: ResolutionState,
) -> Result<Provenance, EdtFormNavigationEmissionError> {
    navigation_provenance_with_context(
        candidate, command, target, "target", None, outcome, origin, resolution,
    )
}

#[allow(clippy::too_many_arguments)]
fn navigation_provenance_with_context(
    candidate: &EdtFormNavigationCandidate,
    command: Option<&EntityId>,
    target: Option<&EntityId>,
    stage: &str,
    owner: Option<&EntityId>,
    outcome: &str,
    origin: FactOrigin,
    resolution: ResolutionState,
) -> Result<Provenance, EdtFormNavigationEmissionError> {
    let mut context = base_context(
        candidate.module_path.to_string_lossy().as_ref(),
        candidate.module_id.as_str(),
        candidate.procedure_id.as_str(),
        candidate.literal.as_str(),
        candidate.location.line,
        candidate.location.column,
    );
    append_context(
        &mut context,
        "procedure_name",
        candidate.procedure_name.as_str(),
    );
    append_context(&mut context, "stage", stage);
    append_context(
        &mut context,
        "command",
        command.map_or("", EntityId::as_str),
    );
    append_context(&mut context, "owner", owner.map_or("", EntityId::as_str));
    append_context(&mut context, "target", target.map_or("", EntityId::as_str));
    append_context(&mut context, "outcome", outcome);
    provenance_from_context(context, origin, resolution)
}

fn base_context(
    path: &str,
    module: &str,
    procedure: &str,
    observation: &str,
    line: usize,
    column: usize,
) -> String {
    let mut context = String::from("form_navigation");
    append_context(&mut context, "path", path);
    append_context(&mut context, "module", module);
    append_context(&mut context, "procedure", procedure);
    append_context(&mut context, "observation", observation);
    append_context(&mut context, "line", &line.to_string());
    append_context(&mut context, "column", &column.to_string());
    context
}

fn append_context(context: &mut String, key: &str, value: &str) {
    context.push(';');
    context.push_str(key);
    context.push('#');
    context.push_str(&value.len().to_string());
    context.push(':');
    context.push_str(value);
}

fn provenance_from_context(
    context: String,
    origin: FactOrigin,
    resolution: ResolutionState,
) -> Result<Provenance, EdtFormNavigationEmissionError> {
    let source = EntityId::new(context)
        .map_err(|_| EdtFormNavigationEmissionError::InvalidSourceIdentifier)?;
    Ok(Provenance::new(
        Some(source),
        ProducerId::new(FORM_NAVIGATION_PRODUCER),
        origin,
        Confidence::Exact,
        resolution,
    ))
}

const fn metadata_owner_stage(kind: MetadataKind) -> &'static str {
    match kind {
        MetadataKind::Catalog => "owner_catalog",
        MetadataKind::Document => "owner_document",
        MetadataKind::Report => "owner_report",
        MetadataKind::DataProcessor => "owner_data_processor",
        MetadataKind::InformationRegister => "owner_information_register",
        MetadataKind::AccumulationRegister => "owner_accumulation_register",
        MetadataKind::AccountingRegister => "owner_accounting_register",
        MetadataKind::CalculationRegister => "owner_calculation_register",
        MetadataKind::BusinessProcess => "owner_business_process",
        MetadataKind::Task => "owner_task",
        _ => "owner_unsupported",
    }
}

const fn module_kind_name(kind: EdtModuleKind) -> &'static str {
    kind.as_str()
}

const fn rejection_reason_name(reason: EdtFormNavigationRejectionReason) -> &'static str {
    match reason {
        EdtFormNavigationRejectionReason::MalformedStatement => "malformed_statement",
        EdtFormNavigationRejectionReason::UnsupportedTarget(target) => {
            unsupported_target_reason_name(target)
        }
        EdtFormNavigationRejectionReason::DynamicFirstArgument => "dynamic_first_argument",
        EdtFormNavigationRejectionReason::IncompleteStatement => "incomplete_statement",
        EdtFormNavigationRejectionReason::UnsupportedModuleKind(_) => "wrong_module",
        EdtFormNavigationRejectionReason::MissingContainingSymbol => "missing_callable",
        EdtFormNavigationRejectionReason::UnsupportedContainingSymbol(_) => "wrong_callable",
    }
}

const fn unsupported_target_reason_name(
    reason: EdtFormNavigationUnsupportedTargetReason,
) -> &'static str {
    match reason {
        EdtFormNavigationUnsupportedTargetReason::DefaultFormAlias => "default_form_alias",
        EdtFormNavigationUnsupportedTargetReason::ShorthandForm => "shorthand_form",
        EdtFormNavigationUnsupportedTargetReason::UnsupportedPrefix => "unsupported_prefix",
        EdtFormNavigationUnsupportedTargetReason::InvalidTargetShape => "invalid_target_shape",
        EdtFormNavigationUnsupportedTargetReason::InvalidName => "invalid_name",
    }
}

const fn outcome_kind_name(kind: EdtFormNavigationOutcomeKind) -> &'static str {
    match kind {
        EdtFormNavigationOutcomeKind::Accepted => "accepted",
        EdtFormNavigationOutcomeKind::Malformed => "malformed",
        EdtFormNavigationOutcomeKind::Unsupported => "unsupported",
        EdtFormNavigationOutcomeKind::Dynamic => "dynamic",
        EdtFormNavigationOutcomeKind::Incomplete => "incomplete",
        EdtFormNavigationOutcomeKind::WrongModule => "wrong_module",
        EdtFormNavigationOutcomeKind::WrongCallable => "wrong_callable",
    }
}

#[derive(Debug)]
enum NavigationSourceError {
    Procedure(ResolutionError),
    ProcedureKind { id: EntityId, actual: NodeKind },
    Module(ResolutionError),
    ModuleKind { id: EntityId, actual: NodeKind },
    ProcedureOwnership(ResolutionError),
    CommandOwnership(ResolutionError),
    CommandKind { id: EntityId, actual: NodeKind },
}

#[derive(Debug)]
enum NavigationTargetError {
    CommonForm {
        source: ResolutionError,
    },
    Owner {
        expected_kind: MetadataKind,
        source: ResolutionError,
    },
    Child {
        owner: EntityId,
        source: ResolutionError,
    },
}

#[derive(Debug)]
pub enum EdtFormNavigationEmissionError {
    ReadModule {
        path: PathBuf,
        source: std::io::Error,
    },
    Graph(GraphError),
    InvalidSourceIdentifier,
}

impl Display for EdtFormNavigationEmissionError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReadModule { path, source } => {
                write!(
                    formatter,
                    "failed to read navigation module `{}`: {source}",
                    path.display()
                )
            }
            Self::Graph(error) => write!(formatter, "failed to emit Form navigation: {error}"),
            Self::InvalidSourceIdentifier => {
                formatter.write_str("failed to create Form navigation provenance source")
            }
        }
    }
}

impl std::error::Error for EdtFormNavigationEmissionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ReadModule { source, .. } => Some(source),
            Self::Graph(error) => Some(error),
            Self::InvalidSourceIdentifier => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use oneagent_common::{EntityId, EntityName};
    use oneagent_graph::{
        EdgeKind, GraphEdge, GraphNode, NodeKind, ResolutionState, SemanticDiagnosticCode,
        SemanticDiagnosticSeverity, SemanticGraph, SemanticReferenceStatistics,
    };
    use oneagent_metadata::MetadataKind;
    use std::collections::BTreeSet;
    use std::path::PathBuf;

    use super::{CollectedFormNavigation, emit_form_navigation};
    use crate::form_navigation::{
        EdtFormNavigationCandidate, EdtFormNavigationParseOutcome, EdtFormNavigationSourceLocation,
        EdtFormNavigationTarget,
    };
    use crate::query_source_resolution::WorkspaceResolutionScope;

    fn id(value: &str) -> EntityId {
        EntityId::new(value).expect("test identifier must be valid")
    }

    fn name(value: &str) -> EntityName {
        EntityName::new(value).expect("test name must be valid")
    }

    fn candidate(module_id: &str, procedure_id: &str) -> EdtFormNavigationCandidate {
        EdtFormNavigationCandidate {
            module_id: id(module_id),
            module_path: PathBuf::from("CommandModule.bsl"),
            procedure_id: id(procedure_id),
            procedure_name: name("Execute"),
            raw_statement: "OpenForm(\"CommonForm.Missing\");".to_owned(),
            literal: "CommonForm.Missing".to_owned(),
            target: EdtFormNavigationTarget::CommonForm {
                form_name: name("Missing"),
            },
            location: EdtFormNavigationSourceLocation { line: 2, column: 5 },
        }
    }

    fn insert_node(graph: &mut SemanticGraph, value: &str, node_name: &str, kind: NodeKind) {
        graph.insert_node(GraphNode::new(id(value), name(node_name), kind));
    }

    fn insert_contains(graph: &mut SemanticGraph, owner: &str, child: &str) {
        graph
            .insert_edge(GraphEdge::new(id(owner), id(child), EdgeKind::Contains))
            .expect("test ownership must be valid");
    }

    fn source_graph() -> SemanticGraph {
        let mut graph = SemanticGraph::new();
        insert_node(
            &mut graph,
            "command",
            "Open",
            NodeKind::Metadata(MetadataKind::Command),
        );
        insert_node(&mut graph, "module", "CommandModule", NodeKind::Module);
        insert_node(&mut graph, "procedure", "Execute", NodeKind::Procedure);
        insert_contains(&mut graph, "command", "module");
        insert_contains(&mut graph, "module", "procedure");
        graph
    }

    #[test]
    fn partial_workspace_missing_target_is_a_warning_without_an_edge() {
        let mut graph = source_graph();
        let collected = CollectedFormNavigation {
            outcomes: vec![EdtFormNavigationParseOutcome::Candidate(Box::new(
                candidate("module", "procedure"),
            ))],
        };
        let mut diagnostics = BTreeSet::new();
        let mut statistics = SemanticReferenceStatistics::new();

        emit_form_navigation(
            &mut graph,
            &collected,
            WorkspaceResolutionScope::Partial,
            &mut diagnostics,
            &mut statistics,
        )
        .expect("partial navigation resolution must remain recoverable");

        assert!(graph.query().edges_by_kind(EdgeKind::Opens).is_empty());
        assert_eq!(statistics.total(), 1);
        assert_eq!(statistics.unresolved(), 1);
        let diagnostic = diagnostics
            .iter()
            .next()
            .expect("partial target must produce a diagnostic");
        assert_eq!(
            diagnostic.code(),
            SemanticDiagnosticCode::ReferenceUnresolved
        );
        assert_eq!(diagnostic.severity(), SemanticDiagnosticSeverity::Warning);
        assert_eq!(diagnostic.provenance().len(), 1);
        assert_eq!(
            diagnostic.provenance()[0].resolution(),
            ResolutionState::Partial
        );
    }

    #[test]
    fn invalid_procedure_owner_is_typed_and_emits_no_edge() {
        let mut graph = SemanticGraph::new();
        insert_node(
            &mut graph,
            "command",
            "Open",
            NodeKind::Metadata(MetadataKind::Command),
        );
        insert_node(&mut graph, "module", "CommandModule", NodeKind::Module);
        insert_node(&mut graph, "other_module", "OtherModule", NodeKind::Module);
        insert_node(&mut graph, "procedure", "Execute", NodeKind::Procedure);
        insert_contains(&mut graph, "command", "module");
        insert_contains(&mut graph, "other_module", "procedure");
        let collected = CollectedFormNavigation {
            outcomes: vec![EdtFormNavigationParseOutcome::Candidate(Box::new(
                candidate("module", "procedure"),
            ))],
        };
        let mut diagnostics = BTreeSet::new();
        let mut statistics = SemanticReferenceStatistics::new();

        emit_form_navigation(
            &mut graph,
            &collected,
            WorkspaceResolutionScope::Complete,
            &mut diagnostics,
            &mut statistics,
        )
        .expect("invalid source ownership must remain recoverable");

        assert!(graph.query().edges_by_kind(EdgeKind::Opens).is_empty());
        assert_eq!(statistics.invalid_owner_reference(), 1);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics
                .iter()
                .next()
                .expect("diagnostic must exist")
                .code(),
            SemanticDiagnosticCode::ReferenceInvalidOwner
        );
    }
}
