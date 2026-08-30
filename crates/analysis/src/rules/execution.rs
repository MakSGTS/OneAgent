//! Bounded deterministic rule execution over immutable canonical evidence.

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use oneagent_common::EntityId;
use oneagent_graph::{SemanticGraph, SemanticGraphValidationResult};

use crate::diagnostics::{
    DiagnosticCategory, DiagnosticFamily, DiagnosticReport, DiagnosticSeverity,
    MAX_DIAGNOSTIC_MESSAGE_BYTES, MAX_DIAGNOSTIC_NODE_ANCHORS, MAX_DIAGNOSTIC_PROVENANCE_RECORDS,
};

use super::{
    MAX_RULE_DIAGNOSTIC_CODE_BYTES, RuleConfiguration, RuleDiagnosticCode, RuleEngineError,
    RuleEngineErrorKind, RuleId, RulePlan, RuleRegistration, RuleRegistry, RuleSettingValue,
    validate_identifier,
};

/// Maximum number of input diagnostics accepted from one rule.
pub const MAX_RULE_DIAGNOSTICS_PER_RULE: usize = 4_096;
/// Maximum number of normalized rule diagnostics in one execution report.
pub const MAX_RULE_DIAGNOSTICS: usize = 65_536;

/// Stable bounded rule-owned failure classification.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RuleFailureCode(Box<str>);

impl RuleFailureCode {
    /// Creates a rule failure code without normalization.
    ///
    /// # Errors
    ///
    /// Returns a redacted [`RuleEngineError`] when the value violates the
    /// accepted rule-code grammar or byte bound.
    pub fn new(value: impl Into<String>) -> Result<Self, RuleEngineError> {
        let value = value.into();
        validate_identifier(
            &value,
            MAX_RULE_DIAGNOSTIC_CODE_BYTES,
            RuleEngineErrorKind::InvalidRuleFailureCode,
        )?;
        Ok(Self(value.into_boxed_str()))
    }

    fn invalid_rule_output() -> Self {
        Self(Box::from("invalid_rule_output"))
    }

    /// Returns the exact accepted failure code.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Minimal cooperative cancellation observation boundary for rules.
pub trait RuleCancellationSignal: Send + Sync {
    /// Returns whether execution cancellation has been requested.
    fn is_cancelled(&self) -> bool;
}

/// Cancellation signal that never requests cancellation.
#[derive(Debug, Default, Clone, Copy)]
pub struct NeverCancelled;

impl RuleCancellationSignal for NeverCancelled {
    fn is_cancelled(&self) -> bool {
        false
    }
}

/// Exact immutable canonical evidence available to every rule.
#[derive(Debug, Clone, Copy)]
pub struct RuleContext<'evidence> {
    graph: &'evidence SemanticGraph,
    validation: &'evidence SemanticGraphValidationResult,
    base_diagnostics: &'evidence DiagnosticReport,
}

impl<'evidence> RuleContext<'evidence> {
    /// Creates a borrowed source-independent rule context.
    #[must_use]
    pub const fn new(
        graph: &'evidence SemanticGraph,
        validation: &'evidence SemanticGraphValidationResult,
        base_diagnostics: &'evidence DiagnosticReport,
    ) -> Self {
        Self {
            graph,
            validation,
            base_diagnostics,
        }
    }

    /// Returns the immutable canonical semantic graph.
    #[must_use]
    pub const fn graph(self) -> &'evidence SemanticGraph {
        self.graph
    }

    /// Returns the caller-supplied complete graph validation result.
    #[must_use]
    pub const fn validation(self) -> &'evidence SemanticGraphValidationResult {
        self.validation
    }

    /// Returns the Semantic/Validation-only base diagnostic report.
    #[must_use]
    pub const fn base_diagnostics(self) -> &'evidence DiagnosticReport {
        self.base_diagnostics
    }
}

/// One object-safe trusted source-independent rule.
pub trait Rule: RuleRegistration {
    /// Evaluates the rule synchronously over immutable canonical evidence.
    fn evaluate(
        &self,
        context: &RuleContext<'_>,
        cancellation: &dyn RuleCancellationSignal,
    ) -> RuleEvaluation;
}

impl<T> Rule for std::sync::Arc<T>
where
    T: Rule + ?Sized,
{
    fn evaluate(
        &self,
        context: &RuleContext<'_>,
        cancellation: &dyn RuleCancellationSignal,
    ) -> RuleEvaluation {
        self.as_ref().evaluate(context, cancellation)
    }
}

/// Closed synchronous rule-owned evaluation outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleEvaluation {
    /// Evaluation completed and returned bounded diagnostic candidates.
    Completed(Vec<RuleDiagnostic>),
    /// Canonical evidence does not make the rule applicable.
    NotApplicable,
    /// Evaluation failed with one stable bounded rule-owned code.
    Failed(RuleFailureCode),
}

/// One rule-produced source-independent diagnostic candidate or result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleDiagnostic {
    rule_id: RuleId,
    code: RuleDiagnosticCode,
    severity: DiagnosticSeverity,
    category: DiagnosticCategory,
    message: String,
    node_anchors: Vec<EntityId>,
    observed_provenance_count: usize,
}

impl RuleDiagnostic {
    /// Creates one rule diagnostic candidate.
    ///
    /// Node anchors are sorted and exact repeats collapse. Message, anchor,
    /// graph-membership, and provenance bounds are validated by [`RuleEngine`]
    /// before the diagnostic becomes reportable.
    #[must_use]
    pub fn new(
        rule_id: RuleId,
        code: RuleDiagnosticCode,
        severity: DiagnosticSeverity,
        category: DiagnosticCategory,
        message: impl Into<String>,
        node_anchors: impl IntoIterator<Item = EntityId>,
    ) -> Self {
        let mut canonical_anchors = BTreeSet::new();
        for anchor in node_anchors {
            canonical_anchors.insert(anchor);
            if canonical_anchors.len() > MAX_DIAGNOSTIC_NODE_ANCHORS {
                break;
            }
        }
        Self {
            rule_id,
            code,
            severity,
            category,
            message: message.into(),
            node_anchors: canonical_anchors.into_iter().collect(),
            observed_provenance_count: 0,
        }
    }

    /// Returns the producing rule identifier.
    #[must_use]
    pub const fn rule_id(&self) -> &RuleId {
        &self.rule_id
    }

    /// Returns the rule-local diagnostic code.
    #[must_use]
    pub const fn code(&self) -> &RuleDiagnosticCode {
        &self.code
    }

    /// Returns normalized severity.
    #[must_use]
    pub const fn severity(&self) -> DiagnosticSeverity {
        self.severity
    }

    /// Returns the source-independent reporting category.
    #[must_use]
    pub const fn category(&self) -> DiagnosticCategory {
        self.category
    }

    /// Returns the bounded diagnostic message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns canonical Graph node anchors.
    #[must_use]
    pub fn node_anchors(&self) -> &[EntityId] {
        &self.node_anchors
    }

    /// Returns the Graph-derived observed provenance record count.
    #[must_use]
    pub const fn observed_provenance_count(&self) -> usize {
        self.observed_provenance_count
    }

    fn identity(&self) -> (&RuleId, &RuleDiagnosticCode, &[EntityId]) {
        (&self.rule_id, &self.code, &self.node_anchors)
    }

    fn with_observed_provenance_count(&self, count: usize) -> Self {
        let mut normalized = self.clone();
        normalized.observed_provenance_count = count;
        normalized
    }
}

impl PartialOrd for RuleDiagnostic {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RuleDiagnostic {
    fn cmp(&self, other: &Self) -> Ordering {
        self.identity().cmp(&other.identity()).then_with(|| {
            (
                self.severity,
                self.category,
                &self.message,
                self.observed_provenance_count,
            )
                .cmp(&(
                    other.severity,
                    other.category,
                    &other.message,
                    other.observed_provenance_count,
                ))
        })
    }
}

/// Closed terminal status of one admitted planned rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RuleStatus {
    /// Configuration disabled the rule before evaluation.
    Disabled,
    /// Evaluation determined that canonical evidence was not applicable.
    NotApplicable,
    /// Evaluation completed and every returned diagnostic was accepted.
    Completed,
    /// A dependency did not complete successfully.
    Blocked,
    /// Rule evaluation or returned output failed.
    Failed,
    /// Cancellation prevented publication or evaluation.
    Cancelled,
}

/// One complete plan-ordered terminal rule result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleResult {
    rule_id: RuleId,
    status: RuleStatus,
    failure_code: Option<RuleFailureCode>,
    diagnostic_count: usize,
}

impl RuleResult {
    fn new(
        rule_id: RuleId,
        status: RuleStatus,
        failure_code: Option<RuleFailureCode>,
        diagnostic_count: usize,
    ) -> Self {
        Self {
            rule_id,
            status,
            failure_code,
            diagnostic_count,
        }
    }

    /// Returns the exact planned rule identifier.
    #[must_use]
    pub const fn rule_id(&self) -> &RuleId {
        &self.rule_id
    }

    /// Returns the terminal rule status.
    #[must_use]
    pub const fn status(&self) -> RuleStatus {
        self.status
    }

    /// Returns the rule-owned or engine-owned stable failure code.
    #[must_use]
    pub const fn failure_code(&self) -> Option<&RuleFailureCode> {
        self.failure_code.as_ref()
    }

    /// Returns accepted diagnostics produced by this completed rule.
    #[must_use]
    pub const fn diagnostic_count(&self) -> usize {
        self.diagnostic_count
    }
}

/// Reconciled counters for one complete rule execution.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct RuleExecutionSummary {
    total: usize,
    by_status: BTreeMap<RuleStatus, usize>,
}

impl RuleExecutionSummary {
    fn from_results(results: &[RuleResult]) -> Result<Self, RuleEngineError> {
        let mut by_status = BTreeMap::new();
        for result in results {
            let count = by_status.entry(result.status()).or_insert(0usize);
            *count = count.checked_add(1).ok_or_else(|| {
                RuleEngineError::new(RuleEngineErrorKind::InconsistentRuleExecution)
            })?;
        }
        let total = by_status.values().try_fold(0usize, |total, &count| {
            total
                .checked_add(count)
                .ok_or_else(|| RuleEngineError::new(RuleEngineErrorKind::InconsistentRuleExecution))
        })?;
        if total != results.len() {
            return Err(RuleEngineError::new(
                RuleEngineErrorKind::InconsistentRuleExecution,
            ));
        }
        Ok(Self { total, by_status })
    }

    /// Returns the number of admitted rules.
    #[must_use]
    pub const fn total(&self) -> usize {
        self.total
    }

    /// Returns terminal counts by closed status.
    #[must_use]
    pub const fn by_status(&self) -> &BTreeMap<RuleStatus, usize> {
        &self.by_status
    }

    /// Returns the count for one terminal status.
    #[must_use]
    pub fn status_count(&self, status: RuleStatus) -> usize {
        self.by_status.get(&status).copied().unwrap_or_default()
    }
}

/// Complete deterministic terminal rule execution report.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct RuleExecutionReport {
    results: Vec<RuleResult>,
    diagnostics: Vec<RuleDiagnostic>,
    summary: RuleExecutionSummary,
}

impl RuleExecutionReport {
    /// Returns plan-ordered terminal results.
    #[must_use]
    pub fn results(&self) -> &[RuleResult] {
        &self.results
    }

    /// Returns accepted diagnostics in canonical identity/content order.
    #[must_use]
    pub fn diagnostics(&self) -> &[RuleDiagnostic] {
        &self.diagnostics
    }

    /// Returns reconciled terminal status counters.
    #[must_use]
    pub const fn summary(&self) -> &RuleExecutionSummary {
        &self.summary
    }
}

/// Stateless sequential deterministic Rules Engine.
#[derive(Debug, Default, Clone, Copy)]
pub struct RuleEngine;

enum RuleStep {
    Terminal {
        status: RuleStatus,
        failure_code: Option<RuleFailureCode>,
        diagnostics: Vec<RuleDiagnostic>,
    },
    CancelRemaining,
}

impl RuleEngine {
    /// Executes one complete validated plan over immutable canonical evidence.
    ///
    /// # Errors
    ///
    /// Returns a closed error without a report when the plan, context,
    /// aggregate diagnostic bound, or result reconciliation is invalid.
    pub fn execute<R>(
        &self,
        registry: &RuleRegistry<R>,
        plan: &RulePlan,
        configuration: &RuleConfiguration,
        context: &RuleContext<'_>,
        cancellation: &dyn RuleCancellationSignal,
    ) -> Result<RuleExecutionReport, RuleEngineError>
    where
        R: Rule,
    {
        let expected_plan = RulePlan::new(registry, configuration)?;
        if &expected_plan != plan {
            return Err(RuleEngineError::new(RuleEngineErrorKind::InvalidRulePlan));
        }
        if context
            .base_diagnostics()
            .findings()
            .iter()
            .any(|finding| finding.family() == DiagnosticFamily::Rule)
        {
            return Err(RuleEngineError::new(
                RuleEngineErrorKind::InvalidRuleContext,
            ));
        }
        execute_plan(registry, plan, context, cancellation)
    }
}

fn execute_plan<R>(
    registry: &RuleRegistry<R>,
    plan: &RulePlan,
    context: &RuleContext<'_>,
    cancellation: &dyn RuleCancellationSignal,
) -> Result<RuleExecutionReport, RuleEngineError>
where
    R: Rule,
{
    let mut results = Vec::with_capacity(plan.len());
    let mut diagnostics = Vec::new();
    let mut statuses = BTreeMap::new();

    for (index, entry) in plan.entries().iter().enumerate() {
        match execute_entry(registry, entry, context, cancellation, &statuses)? {
            RuleStep::CancelRemaining => {
                append_cancelled(plan, index, &mut results, &mut statuses);
                break;
            }
            RuleStep::Terminal {
                status,
                failure_code,
                diagnostics: accepted,
            } => {
                let diagnostic_count = accepted.len();
                append_diagnostics(&mut diagnostics, accepted)?;
                push_result(
                    entry.rule_id(),
                    status,
                    failure_code,
                    diagnostic_count,
                    &mut results,
                    &mut statuses,
                );
            }
        }
    }

    if results.len() != plan.len() {
        return Err(RuleEngineError::new(
            RuleEngineErrorKind::InconsistentRuleExecution,
        ));
    }
    diagnostics.sort();
    let summary = RuleExecutionSummary::from_results(&results)?;
    Ok(RuleExecutionReport {
        results,
        diagnostics,
        summary,
    })
}

fn execute_entry<R>(
    registry: &RuleRegistry<R>,
    entry: &super::RulePlanEntry,
    context: &RuleContext<'_>,
    cancellation: &dyn RuleCancellationSignal,
    statuses: &BTreeMap<RuleId, RuleStatus>,
) -> Result<RuleStep, RuleEngineError>
where
    R: Rule,
{
    if cancellation.is_cancelled() {
        return Ok(RuleStep::CancelRemaining);
    }
    if entry.setting() == RuleSettingValue::Disabled {
        return Ok(RuleStep::Terminal {
            status: RuleStatus::Disabled,
            failure_code: None,
            diagnostics: Vec::new(),
        });
    }
    if entry
        .dependencies()
        .iter()
        .any(|dependency| statuses.get(dependency) != Some(&RuleStatus::Completed))
    {
        return Ok(RuleStep::Terminal {
            status: RuleStatus::Blocked,
            failure_code: None,
            diagnostics: Vec::new(),
        });
    }

    let Some(rule) = registry.get(entry.rule_id()) else {
        return Err(RuleEngineError::new(RuleEngineErrorKind::InvalidRulePlan));
    };
    let evaluation = rule.evaluate(context, cancellation);
    if cancellation.is_cancelled() {
        return Ok(RuleStep::CancelRemaining);
    }

    Ok(match evaluation {
        RuleEvaluation::NotApplicable => RuleStep::Terminal {
            status: RuleStatus::NotApplicable,
            failure_code: None,
            diagnostics: Vec::new(),
        },
        RuleEvaluation::Failed(code) => RuleStep::Terminal {
            status: RuleStatus::Failed,
            failure_code: Some(code),
            diagnostics: Vec::new(),
        },
        RuleEvaluation::Completed(candidates) => {
            match normalize_rule_diagnostics(entry.rule_id(), candidates, context.graph()) {
                Ok(diagnostics) => RuleStep::Terminal {
                    status: RuleStatus::Completed,
                    failure_code: None,
                    diagnostics,
                },
                Err(()) => RuleStep::Terminal {
                    status: RuleStatus::Failed,
                    failure_code: Some(RuleFailureCode::invalid_rule_output()),
                    diagnostics: Vec::new(),
                },
            }
        }
    })
}

fn append_diagnostics(
    diagnostics: &mut Vec<RuleDiagnostic>,
    accepted: Vec<RuleDiagnostic>,
) -> Result<(), RuleEngineError> {
    let next_count = diagnostics
        .len()
        .checked_add(accepted.len())
        .ok_or_else(|| {
            RuleEngineError::bounded(
                RuleEngineErrorKind::TooManyRuleDiagnostics,
                usize::MAX,
                MAX_RULE_DIAGNOSTICS,
            )
        })?;
    if next_count > MAX_RULE_DIAGNOSTICS {
        return Err(RuleEngineError::bounded(
            RuleEngineErrorKind::TooManyRuleDiagnostics,
            next_count,
            MAX_RULE_DIAGNOSTICS,
        ));
    }
    diagnostics.extend(accepted);
    Ok(())
}

fn append_cancelled(
    plan: &RulePlan,
    start: usize,
    results: &mut Vec<RuleResult>,
    statuses: &mut BTreeMap<RuleId, RuleStatus>,
) {
    for entry in &plan.entries()[start..] {
        push_result(
            entry.rule_id(),
            RuleStatus::Cancelled,
            None,
            0,
            results,
            statuses,
        );
    }
}

fn push_result(
    rule_id: &RuleId,
    status: RuleStatus,
    failure_code: Option<RuleFailureCode>,
    diagnostic_count: usize,
    results: &mut Vec<RuleResult>,
    statuses: &mut BTreeMap<RuleId, RuleStatus>,
) {
    statuses.insert(rule_id.clone(), status);
    results.push(RuleResult::new(
        rule_id.clone(),
        status,
        failure_code,
        diagnostic_count,
    ));
}

fn normalize_rule_diagnostics(
    rule_id: &RuleId,
    candidates: Vec<RuleDiagnostic>,
    graph: &SemanticGraph,
) -> Result<Vec<RuleDiagnostic>, ()> {
    if candidates.len() > MAX_RULE_DIAGNOSTICS_PER_RULE {
        return Err(());
    }

    let mut normalized = BTreeMap::new();
    for candidate in candidates {
        if candidate.rule_id() != rule_id
            || candidate.message().len() > MAX_DIAGNOSTIC_MESSAGE_BYTES
            || candidate.node_anchors().len() > MAX_DIAGNOSTIC_NODE_ANCHORS
        {
            return Err(());
        }

        let mut provenance_count = 0usize;
        for anchor in candidate.node_anchors() {
            let Some(node) = graph.node(anchor) else {
                return Err(());
            };
            provenance_count = provenance_count
                .checked_add(node.provenance().len())
                .ok_or(())?;
            if provenance_count > MAX_DIAGNOSTIC_PROVENANCE_RECORDS {
                return Err(());
            }
        }
        let candidate = candidate.with_observed_provenance_count(provenance_count);
        let identity = (
            candidate.rule_id().clone(),
            candidate.code().clone(),
            candidate.node_anchors().to_vec(),
        );
        match normalized.entry(identity) {
            std::collections::btree_map::Entry::Vacant(entry) => {
                entry.insert(candidate);
            }
            std::collections::btree_map::Entry::Occupied(entry) if entry.get() == &candidate => {}
            std::collections::btree_map::Entry::Occupied(_) => return Err(()),
        }
    }
    Ok(normalized.into_values().collect())
}
