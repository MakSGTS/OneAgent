//! Pure, exhaustive postconditions for the bounded callable rename transaction.
//!
//! This module grants no authority and performs no filesystem operations.

use crate::diagnostics::{DiagnosticEngine, DiagnosticPolicy, DiagnosticReport};
use crate::publication::WorkspacePublicationId;
use crate::refactoring::{
    RefactoringOperation, RefactoringPlan, SourceDocument, SourceEvidenceSet,
};
use crate::refactoring::{SourceContentVersion, SourceFormat};
use crate::rules::RuleExecutionReport;
use oneagent_common::SourcePath;
use oneagent_common::{EntityId, EntityName};
use oneagent_graph::{EdgeKind, NodeKind, SemanticReferenceRequestId};
use oneagent_graph::{
    Provenance, SemanticDiagnostic, SemanticGraph, SemanticGraphReport,
    SemanticGraphValidationResult, SemanticReference, SemanticReferenceRequest,
    SemanticReferenceRequestLedger, SemanticReferenceStatistics,
};
use std::fmt::{Display, Formatter};
use std::sync::Arc;

/// Closed comparison failure; never retains rejected evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SafeEditError {
    /// Complete structures differ, regardless of their digest identities.
    PlanMismatch,
    /// Invalid version, token, ordering, range or output bound.
    InvalidReplacement,
    /// A candidate differs outside the exact accepted transformation.
    SemanticMismatch,
    /// Checked transaction-owned projection storage would exceed admission.
    ProjectionBounds,
}

impl Display for SafeEditError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "safe edit comparison failed: {self:?}")
    }
}
impl std::error::Error for SafeEditError {}

/// Shared inclusive allowance for transaction-owned raw and projection storage.
pub const MAX_SAFE_EDIT_BUFFER_BYTES: usize = 268_435_456;

/// Checked counting sink used by the same canonical encoder as emission.
///
/// Counting owns no output storage and never parses, clones or formats a string.
#[derive(Debug, Default)]
pub struct SafeEditCountingSink {
    bytes: usize,
}

impl SafeEditCountingSink {
    /// Returns the counted UTF-8 output length.
    #[must_use]
    pub const fn bytes(&self) -> usize {
        self.bytes
    }
}

impl std::fmt::Write for SafeEditCountingSink {
    fn write_str(&mut self, value: &str) -> std::fmt::Result {
        self.bytes = self.bytes.checked_add(value.len()).ok_or(std::fmt::Error)?;
        Ok(())
    }
}

/// Per-attempt accounting shared by raw buffers and expected producer evidence.
///
/// The supplied retained count includes the Runtime's live baseline, verification,
/// replacement and recovery reservations. This is not an additional allowance.
#[derive(Debug)]
pub struct SafeEditProjectionAdmission {
    retained: usize,
    peak: usize,
}

#[cfg(test)]
mod allocation_audit {
    #[derive(Default)]
    pub(super) struct Audit {
        pub(super) fail_at: Option<usize>,
        pub(super) boundaries: Vec<(usize, usize, Option<usize>)>,
    }
    thread_local! {
        pub(super) static CURRENT: std::cell::RefCell<Audit> = std::cell::RefCell::default();
    }
    pub(super) fn before(reserved: usize, bytes: usize) -> bool {
        CURRENT.with_borrow_mut(|audit| {
            audit.boundaries.push((reserved, bytes, None));
            audit.fail_at == Some(audit.boundaries.len())
        })
    }
    pub(super) fn allocated(bytes: usize) {
        CURRENT.with_borrow_mut(|audit| audit.boundaries.last_mut().unwrap().2 = Some(bytes));
    }
}

impl SafeEditProjectionAdmission {
    fn copy_transaction<T>(
        &mut self,
        copy: impl FnOnce(&mut Self) -> Result<T, SafeEditError>,
    ) -> Result<T, SafeEditError> {
        let checkpoint = self.retained;
        let result = copy(self);
        if result.is_err() {
            self.release(self.retained - checkpoint)?;
        }
        result
    }
    /// Copies exact UTF-8 bytes into fallibly allocated, prepaid storage.
    ///
    /// # Errors
    /// Rejects exhausted admission or allocation failure.
    pub fn copy_string(&mut self, value: &str) -> Result<String, SafeEditError> {
        self.string(value.len(), |sink| sink.write_str(value))
    }

    /// Copies a validated identity without changing its encoding.
    ///
    /// # Errors
    /// Rejects exhausted admission or allocation failure.
    pub fn copy_id(&mut self, value: &EntityId) -> Result<EntityId, SafeEditError> {
        EntityId::new(self.copy_string(value.as_str())?)
            .map_err(|_| SafeEditError::SemanticMismatch)
    }
    /// Starts accounting from the complete already-reserved attempt storage.
    ///
    /// # Errors
    /// Rejects an already excessive reservation.
    pub const fn new(retained: usize) -> Result<Self, SafeEditError> {
        if retained > MAX_SAFE_EDIT_BUFFER_BYTES {
            Err(SafeEditError::ProjectionBounds)
        } else {
            Ok(Self {
                retained,
                peak: retained,
            })
        }
    }

    /// Returns the live checked reservation, including the initial raw storage.
    #[must_use]
    pub const fn retained_bytes(&self) -> usize {
        self.retained
    }

    /// Returns the maximum simultaneous reservation observed in this attempt.
    #[must_use]
    pub const fn peak_bytes(&self) -> usize {
        self.peak
    }

    /// Reserves storage before constructing or retaining its output.
    ///
    /// # Errors
    /// Rejects overflow and one byte beyond the shared allowance.
    pub fn reserve(&mut self, bytes: usize) -> Result<(), SafeEditError> {
        let next = self
            .retained
            .checked_add(bytes)
            .ok_or(SafeEditError::ProjectionBounds)?;
        if next > MAX_SAFE_EDIT_BUFFER_BYTES {
            return Err(SafeEditError::ProjectionBounds);
        }
        self.retained = next;
        self.peak = self.peak.max(next);
        Ok(())
    }

    /// Releases storage only after its corresponding allocation is destroyed.
    ///
    /// # Errors
    /// Rejects a release larger than the live reservation.
    pub fn release(&mut self, bytes: usize) -> Result<(), SafeEditError> {
        self.retained = self
            .retained
            .checked_sub(bytes)
            .ok_or(SafeEditError::ProjectionBounds)?;
        Ok(())
    }

    /// Counts one vector's inline storage without allocation.
    ///
    /// Nested strings and vectors must be counted separately.
    ///
    /// # Errors
    /// Rejects capacity multiplication overflow.
    pub const fn vector_bytes<T>(capacity: usize) -> Result<usize, SafeEditError> {
        match capacity.checked_mul(std::mem::size_of::<T>()) {
            Some(bytes) => Ok(bytes),
            None => Err(SafeEditError::ProjectionBounds),
        }
    }

    /// Allocates a prepaid exact-capacity vector and checks allocator capacity.
    ///
    /// The caller must never grow the vector implicitly and must release the
    /// returned byte count after destroying the vector.
    ///
    /// # Errors
    /// Rejects overflow, exhausted admission, allocation failure or excess capacity.
    pub fn vector<T>(&mut self, capacity: usize) -> Result<Vec<T>, SafeEditError> {
        let bytes = Self::vector_bytes::<T>(capacity)?;
        self.reserve(bytes)?;
        #[cfg(test)]
        if allocation_audit::before(self.retained, bytes) {
            self.release(bytes)?;
            return Err(SafeEditError::ProjectionBounds);
        }
        let mut output = Vec::new();
        if output.try_reserve_exact(capacity).is_err()
            || (std::mem::size_of::<T>() != 0 && output.capacity() != capacity)
        {
            drop(output);
            self.release(bytes)?;
            return Err(SafeEditError::ProjectionBounds);
        }
        #[cfg(test)]
        allocation_audit::allocated(output.capacity().saturating_mul(std::mem::size_of::<T>()));
        Ok(output)
    }

    /// Emits an exact canonical context after a separate nonallocating count pass.
    ///
    /// The writer rejects growth; no formatted temporary string is needed.
    ///
    /// # Errors
    /// Rejects count/emission disagreement, allocation failure or exhausted admission.
    pub fn string(
        &mut self,
        bytes: usize,
        emit: impl FnOnce(&mut dyn std::fmt::Write) -> std::fmt::Result,
    ) -> Result<String, SafeEditError> {
        struct FixedWriter {
            raw: Vec<u8>,
        }
        impl std::fmt::Write for FixedWriter {
            fn write_str(&mut self, value: &str) -> std::fmt::Result {
                if value.len() > self.raw.capacity() - self.raw.len() {
                    return Err(std::fmt::Error);
                }
                self.raw.extend_from_slice(value.as_bytes());
                Ok(())
            }
        }
        let mut output = FixedWriter {
            raw: self.vector::<u8>(bytes)?,
        };
        if emit(&mut output).is_err() || output.raw.len() != bytes {
            drop(output);
            self.release(bytes)?;
            return Err(SafeEditError::ProjectionBounds);
        }
        // Only valid UTF-8 reaches FixedWriter through fmt::Write.
        String::from_utf8(output.raw).map_err(|_| SafeEditError::ProjectionBounds)
    }
}

/// Compares every canonical plan field, not just its identity.
///
/// # Errors
/// Returns `PlanMismatch` for any unequal structure.
pub fn compare_plan(
    expected: &RefactoringPlan,
    actual: &RefactoringPlan,
) -> Result<(), SafeEditError> {
    if expected == actual {
        Ok(())
    } else {
        Err(SafeEditError::PlanMismatch)
    }
}

/// Computes exact replacement bytes after checking all operations and output size.
///
/// Operations must be in canonical descending raw-offset order for this document.
/// The caller reserves aggregate storage before calling this function.
///
/// # Errors
/// Returns a closed error before allocating output for invalid or oversized input.
pub fn replacement_bytes(
    document: &SourceDocument,
    operations: &[&RefactoringOperation],
    maximum: usize,
) -> Result<Vec<u8>, SafeEditError> {
    let invalid = SafeEditError::InvalidReplacement;
    let raw = document.raw_content();
    let text = std::str::from_utf8(raw).map_err(|_| invalid)?;
    let mut length = raw.len();
    let mut previous_start = raw.len();
    for operation in operations {
        let range = operation.range();
        if operation.document_id() != document.id()
            || operation.content_version() != document.content_version()
            || range.end_byte() > previous_start
            || !text.is_char_boundary(range.start_byte())
            || !text.is_char_boundary(range.end_byte())
            || raw.get(range.start_byte()..range.end_byte())
                != Some(operation.expected().as_bytes())
        {
            return Err(invalid);
        }
        length = length
            .checked_sub(range.end_byte() - range.start_byte())
            .and_then(|n| n.checked_add(operation.replacement().len()))
            .ok_or(invalid)?;
        previous_start = range.start_byte();
    }
    if raw.len() > maximum || length > maximum {
        return Err(invalid);
    }
    let mut result = Vec::new();
    result
        .try_reserve_exact(length)
        .map_err(|_| SafeEditError::ProjectionBounds)?;
    if result.capacity() != length {
        return Err(SafeEditError::ProjectionBounds);
    }
    let mut cursor = 0;
    for operation in operations.iter().rev() {
        result.extend_from_slice(&raw[cursor..operation.range().start_byte()]);
        result.extend_from_slice(operation.replacement().as_bytes());
        cursor = operation.range().end_byte();
    }
    result.extend_from_slice(&raw[cursor..]);
    Ok(result)
}

/// Borrowed complete evidence from one production Configuration build.
pub struct SafeEditEvidence<'a> {
    /// Exact discovered Configuration root; opaque to this pure comparator.
    pub root: &'a std::path::Path,
    /// Canonical graph.
    pub graph: &'a SemanticGraph,
    /// Complete captured document and occurrence inventory.
    pub sources: &'a SourceEvidenceSet,
    /// Complete producer diagnostics.
    pub diagnostics: &'a [SemanticDiagnostic],
    /// Complete terminal reference ledger.
    pub references: &'a SemanticReferenceRequestLedger,
    /// Producer reference statistics.
    pub reference_statistics: SemanticReferenceStatistics,
    /// Complete graph report.
    pub report: &'a SemanticGraphReport,
    /// Successful graph/build validation.
    pub validation: &'a SemanticGraphValidationResult,
    /// Complete rule execution evidence.
    pub rules: &'a RuleExecutionReport,
    /// Canonical normalized diagnostic composition.
    pub findings: &'a DiagnosticReport,
}

/// One complete captured module and its exact, already-admitted replacement.
pub struct SafeEditModuleInput<'a> {
    document: &'a SourceDocument,
    owner: &'a EntityId,
    name: &'a EntityName,
    path: &'a SourcePath,
    result: &'a [u8],
}

impl<'a> SafeEditModuleInput<'a> {
    /// Binds a captured document to its producer facts and exact result bytes.
    ///
    /// The complete input constructor checks graph ownership and path coverage.
    ///
    /// # Errors
    /// Rejects invalid UTF-8 before any producer allocation.
    pub fn new(
        document: &'a SourceDocument,
        owner: &'a EntityId,
        name: &'a EntityName,
        path: &'a SourcePath,
        result: &'a [u8],
    ) -> Result<Self, SafeEditError> {
        std::str::from_utf8(result).map_err(|_| SafeEditError::InvalidReplacement)?;
        Ok(Self {
            document,
            owner,
            name,
            path,
            result,
        })
    }
    /// Returns the complete original document.
    #[must_use]
    pub const fn document(&self) -> &'a SourceDocument {
        self.document
    }
    /// Returns the module's canonical owner.
    #[must_use]
    pub const fn owner(&self) -> &'a EntityId {
        self.owner
    }
    /// Returns the module's canonical name.
    #[must_use]
    pub const fn name(&self) -> &'a EntityName {
        self.name
    }
    /// Returns the captured producer path, supplied by confined Runtime evidence.
    #[must_use]
    pub const fn path(&self) -> &'a SourcePath {
        self.path
    }
    /// Returns exact result bytes, borrowing unchanged documents too.
    #[must_use]
    pub const fn result(&self) -> &'a [u8] {
        self.result
    }
}

/// Complete immutable input to a pure adapter projector; contains no candidate.
pub struct SafeEditProducerInput<'a> {
    root: &'a std::path::Path,
    publication: WorkspacePublicationId,
    graph: &'a Arc<SemanticGraph>,
    sources: &'a SourceEvidenceSet,
    diagnostics: &'a Arc<[SemanticDiagnostic]>,
    references: &'a Arc<SemanticReferenceRequestLedger>,
    plan: &'a RefactoringPlan,
    modules: &'a [SafeEditModuleInput<'a>],
}

impl<'a> SafeEditProducerInput<'a> {
    /// Checks complete document/producer ownership and exact replacement coverage.
    ///
    /// `workspace_root` and producer paths come from the Runtime's confined baseline.
    /// This function performs no filesystem operations or parsing.
    ///
    /// # Errors
    /// Rejects a different publication, missing/extra modules, owners, paths or bytes.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        publication: WorkspacePublicationId,
        workspace_root: &'a std::path::Path,
        configuration_root: &'a std::path::Path,
        graph: &'a Arc<SemanticGraph>,
        sources: &'a SourceEvidenceSet,
        diagnostics: &'a Arc<[SemanticDiagnostic]>,
        references: &'a Arc<SemanticReferenceRequestLedger>,
        plan: &'a RefactoringPlan,
        modules: &'a [SafeEditModuleInput<'a>],
    ) -> Result<Self, SafeEditError> {
        require(
            publication == plan.request().expected_publication_id()
                && configuration_root.starts_with(workspace_root)
                && sources.configuration_id() == plan.request().configuration_id()
                && sources.documents().len() == modules.len(),
        )?;
        require(
            graph
                .nodes()
                .filter(|node| node.kind() == NodeKind::Module)
                .count()
                == sources.documents().len(),
        )?;
        for (document, module) in sources.documents().iter().zip(modules) {
            require(std::ptr::eq(document, module.document))?;
            let node = graph
                .node(document.id().module_id())
                .ok_or(SafeEditError::SemanticMismatch)?;
            require(node.kind() == NodeKind::Module && node.name() == module.name)?;
            let mut owners = graph
                .edges()
                .filter(|edge| edge.kind() == EdgeKind::Contains && edge.target() == node.id());
            require(
                owners
                    .next()
                    .is_some_and(|edge| edge.source() == module.owner)
                    && owners.next().is_none(),
            )?;
            let relative = std::path::Path::new(module.path.as_str())
                .strip_prefix(workspace_root)
                .map_err(|_| SafeEditError::SemanticMismatch)?;
            require(relative == std::path::Path::new(document.path().path().as_str()))?;
            require(exact_result_matches(plan, document, module.result))?;
        }
        Ok(Self {
            root: configuration_root,
            publication,
            graph,
            sources,
            diagnostics,
            references,
            plan,
            modules,
        })
    }
    /// Returns the complete before graph.
    #[must_use]
    pub fn graph(&self) -> &SemanticGraph {
        self.graph
    }
    /// Returns the complete before source evidence.
    #[must_use]
    pub const fn sources(&self) -> &'a SourceEvidenceSet {
        self.sources
    }
    /// Returns all before producer diagnostics in canonical order.
    #[must_use]
    pub fn diagnostics(&self) -> &[SemanticDiagnostic] {
        self.diagnostics
    }
    /// Returns the complete before terminal ledger.
    #[must_use]
    pub fn references(&self) -> &SemanticReferenceRequestLedger {
        self.references
    }
    /// Returns the exact complete plan.
    #[must_use]
    pub const fn plan(&self) -> &'a RefactoringPlan {
        self.plan
    }
    /// Returns the complete ordered module inventory.
    #[must_use]
    pub const fn modules(&self) -> &'a [SafeEditModuleInput<'a>] {
        self.modules
    }

    /// Creates the prepaid complete unchanged fact inventory and target identity.
    ///
    /// Query records and dependent identities have reserved slots but must be
    /// supplied from canonical before/result extraction before freeze.
    ///
    /// # Errors
    /// Rejects checked capacity overflow or exhausted shared admission.
    pub fn unchanged_facts(
        &self,
        admission: &mut SafeEditProjectionAdmission,
    ) -> Result<Vec<SafeEditProjectionFact>, SafeEditError> {
        let mut count = 1usize;
        let mut nested = self
            .plan
            .target()
            .target_node_id()
            .as_str()
            .len()
            .checked_add(
                self.plan
                    .target()
                    .expected_post_rename_node_id()
                    .as_str()
                    .len(),
            )
            .ok_or(SafeEditError::ProjectionBounds)?;
        for node in self.graph.nodes() {
            count = count
                .checked_add(1)
                .ok_or(SafeEditError::ProjectionBounds)?;
            nested = nested
                .checked_add(node.id().as_str().len())
                .ok_or(SafeEditError::ProjectionBounds)?;
            if node.kind() == NodeKind::Query {
                count = count
                    .checked_add(1)
                    .ok_or(SafeEditError::ProjectionBounds)?;
                if self.graph.edges().any(|edge| {
                    edge.kind() == EdgeKind::Contains
                        && edge.source() == self.plan.target().target_node_id()
                        && edge.target() == node.id()
                }) {
                    count = count
                        .checked_add(1)
                        .ok_or(SafeEditError::ProjectionBounds)?;
                }
            }
        }
        for edge in self.graph.edges() {
            count = count
                .checked_add(1)
                .ok_or(SafeEditError::ProjectionBounds)?;
            nested = nested
                .checked_add(edge.source().as_str().len())
                .and_then(|n| n.checked_add(edge.target().as_str().len()))
                .ok_or(SafeEditError::ProjectionBounds)?;
        }
        for request in self.references.requests() {
            count = count
                .checked_add(1)
                .ok_or(SafeEditError::ProjectionBounds)?;
            nested = nested
                .checked_add(request.id().as_str().len())
                .ok_or(SafeEditError::ProjectionBounds)?;
        }
        count = count
            .checked_add(self.diagnostics.len())
            .ok_or(SafeEditError::ProjectionBounds)?;
        let mut records = admission.vector::<SafeEditProjectionFact>(count)?;
        let _ = nested; // All nested copies reserve their own exact storage below.
        records.push(SafeEditProjectionFact::new(
            SafeEditFactKey::NodeIdentity {
                before: admission.copy_id(self.plan.target().target_node_id())?,
            },
            SafeEditFactValue::Identity(
                admission.copy_id(self.plan.target().expected_post_rename_node_id())?,
            ),
        )?);
        for node in self.graph.nodes() {
            records.push(SafeEditProjectionFact::new(
                SafeEditFactKey::NodeFact {
                    before: admission.copy_id(node.id())?,
                },
                SafeEditFactValue::Provenance(SafeEditExpected::Unchanged),
            )?);
        }
        for edge in self.graph.edges() {
            records.push(SafeEditProjectionFact::new(
                SafeEditFactKey::EdgeFact {
                    before_source: admission.copy_id(edge.source())?,
                    kind: edge.kind(),
                    before_target: admission.copy_id(edge.target())?,
                },
                SafeEditFactValue::Provenance(SafeEditExpected::Unchanged),
            )?);
        }
        for request in self.references.requests() {
            records.push(SafeEditProjectionFact::new(
                SafeEditFactKey::RequestFact {
                    before: SafeEditRequestId::copy(request.id(), admission)?,
                },
                SafeEditFactValue::Request(SafeEditExpected::Unchanged),
            )?);
        }
        for before_ordinal in 0..self.diagnostics.len() {
            records.push(SafeEditProjectionFact::new(
                SafeEditFactKey::DiagnosticFact { before_ordinal },
                SafeEditFactValue::Diagnostic(SafeEditExpected::Unchanged),
            )?);
        }
        Ok(records)
    }
}

fn exact_result_matches(plan: &RefactoringPlan, document: &SourceDocument, result: &[u8]) -> bool {
    let mut before_offset = 0;
    let mut result_offset = 0usize;
    for operation in plan
        .operations()
        .iter()
        .rev()
        .filter(|op| op.document_id() == document.id())
    {
        let range = operation.range();
        if range.start_byte() < before_offset
            || range.end_byte() > document.raw_content().len()
            || operation.content_version() != document.content_version()
            || document
                .raw_content()
                .get(range.start_byte()..range.end_byte())
                != Some(operation.expected().as_bytes())
        {
            return false;
        }
        for bytes in [
            &document.raw_content()[before_offset..range.start_byte()],
            operation.replacement().as_bytes(),
        ] {
            let Some(end) = result_offset.checked_add(bytes.len()) else {
                return false;
            };
            if result.get(result_offset..end) != Some(bytes) {
                return false;
            }
            result_offset = end;
        }
        before_offset = range.end_byte();
    }
    result.get(result_offset..) == Some(&document.raw_content()[before_offset..])
}

/// Typed before-side fact key, scoped by its containing Configuration projection.
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum SafeEditFactKey {
    /// The target or a directly owned, format-supported Query identity.
    NodeIdentity { before: EntityId },
    /// One complete node provenance list.
    NodeFact { before: EntityId },
    /// One complete edge provenance list; no opaque edge ID decoding.
    EdgeFact {
        before_source: EntityId,
        kind: EdgeKind,
        before_target: EntityId,
    },
    /// One complete canonical terminal request.
    RequestFact { before: SafeEditRequestId },
    /// One diagnostic in the exact bound before collection.
    DiagnosticFact { before_ordinal: usize },
    /// One format-supported Query, reconciled independently with the Graph.
    QueryFact { before: EntityId },
}

/// Owned canonical request identity. Only an existing Graph identity can supply it.
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct SafeEditRequestId(String);

impl SafeEditRequestId {
    /// Copies an existing canonical identity without formatting or parsing it.
    ///
    /// # Errors
    /// Rejects exhausted admission or allocation failure.
    pub fn copy(
        value: &SemanticReferenceRequestId,
        admission: &mut SafeEditProjectionAdmission,
    ) -> Result<Self, SafeEditError> {
        Ok(Self(admission.copy_string(value.as_str())?))
    }
    /// Compares the complete canonical identity.
    #[must_use]
    pub fn matches(&self, value: &SemanticReferenceRequestId) -> bool {
        self.0 == value.as_str()
    }
}

/// Lossless owned provenance, including a canonical path copied without normalization.
#[derive(PartialEq, Eq)]
pub struct SafeEditProvenance {
    source: Option<EntityId>,
    location: Option<(String, Option<oneagent_common::SourceSpan>)>,
    producer: String,
    origin: oneagent_graph::FactOrigin,
    confidence: oneagent_graph::Confidence,
    resolution: oneagent_graph::ResolutionState,
}

impl SafeEditProvenance {
    /// Returns the canonical source key for prepaid producer ordering.
    #[must_use]
    pub const fn source(&self) -> Option<&EntityId> {
        self.source.as_ref()
    }
    /// Copies every canonical field after its exact allocation is admitted.
    ///
    /// # Errors
    /// Rejects exhausted admission or allocation failure.
    pub fn copy(
        value: &Provenance,
        admission: &mut SafeEditProjectionAdmission,
    ) -> Result<Self, SafeEditError> {
        admission.copy_transaction(|admission| Self::copy_inner(value, admission))
    }

    fn copy_inner(
        value: &Provenance,
        admission: &mut SafeEditProjectionAdmission,
    ) -> Result<Self, SafeEditError> {
        Ok(Self {
            source: value.source().map(|id| admission.copy_id(id)).transpose()?,
            location: value
                .location()
                .map(|location| {
                    Ok((
                        admission.copy_string(location.path().as_str())?,
                        location.span(),
                    ))
                })
                .transpose()?,
            producer: admission.copy_string(value.producer().as_str())?,
            origin: value.origin(),
            confidence: value.confidence(),
            resolution: value.resolution(),
        })
    }

    /// Copies a complete ordered collection without sorting or deduplication.
    ///
    /// # Errors
    /// Rejects exhausted admission or allocation failure.
    pub fn copy_all(
        values: &[Provenance],
        admission: &mut SafeEditProjectionAdmission,
    ) -> Result<Vec<Self>, SafeEditError> {
        admission.copy_transaction(|admission| {
            let mut result = admission.vector(values.len())?;
            for value in values {
                result.push(Self::copy(value, admission)?);
            }
            Ok(result)
        })
    }

    /// Compares every field, including the presence of optional anchors.
    #[must_use]
    pub fn matches(&self, value: &Provenance) -> bool {
        self.source.as_ref() == value.source()
            && self
                .location
                .as_ref()
                .map(|(path, span)| (path.as_str(), *span))
                == value
                    .location()
                    .map(|location| (location.path().as_str(), location.span()))
            && self.producer == value.producer().as_str()
            && self.origin == value.origin()
            && self.confidence == value.confidence()
            && self.resolution == value.resolution()
    }
}

fn provenance_matches(expected: &[SafeEditProvenance], actual: &[Provenance]) -> bool {
    expected.len() == actual.len() && expected.iter().zip(actual).all(|(a, b)| a.matches(b))
}

#[cfg(test)]
mod allocation_tests {
    use super::*;
    use oneagent_graph::{
        Confidence, FactOrigin, ProducerId, ResolutionState, SemanticDiagnosticCode,
        SemanticDiagnosticKind, SemanticDiagnosticSeverity, SemanticReferenceCategory,
    };

    #[test]
    fn every_nested_copy_allocation_is_prepaid_and_releases_on_failure() {
        let id = |value| EntityId::new(value).unwrap();
        let provenance = |state| {
            Provenance::new_with_location(
                Some(id("nested.source")),
                Some(oneagent_common::SourceLocation::new(
                    SourcePath::new("nested/source.bsl").unwrap(),
                    None,
                )),
                ProducerId::new("nested.producer"),
                FactOrigin::Resolved,
                Confidence::Exact,
                state,
            )
        };
        let reference = SemanticReference::Child {
            owner: id("nested.owner"),
            name: EntityName::new("NestedName").unwrap(),
        };
        let request = SemanticReferenceRequest::collected(
            id("module"),
            SemanticReferenceCategory::MetadataType,
            reference.clone(),
            [NodeKind::Module],
            [provenance(ResolutionState::Unresolved)],
        )
        .unwrap()
        .into_ambiguous_target(
            [id("candidate.one"), id("candidate.two")],
            [provenance(ResolutionState::Ambiguous)],
        )
        .unwrap();
        let diagnostic = SemanticDiagnostic::new(
            SemanticDiagnosticCode::ReferenceUnresolved,
            SemanticDiagnosticSeverity::Error,
            SemanticDiagnosticKind::UnresolvedTarget,
            "nested finding",
            reference,
        )
        .with_source_node(id("module"))
        .with_expected_kinds(vec![NodeKind::Module])
        .with_candidates(vec![id("candidate.one"), id("candidate.two")])
        .with_provenance(vec![provenance(ResolutionState::Unresolved)]);
        for diagnostic_copy in [false, true] {
            let copy = |admission: &mut SafeEditProjectionAdmission| {
                if diagnostic_copy {
                    SafeEditDiagnostic::copy(&diagnostic, admission).map(drop)
                } else {
                    SafeEditRequest::copy(&request, admission).map(drop)
                }
            };
            allocation_audit::CURRENT
                .with_borrow_mut(|audit| *audit = allocation_audit::Audit::default());
            let mut admission = SafeEditProjectionAdmission::new(71).unwrap();
            copy(&mut admission).unwrap();
            let boundaries =
                allocation_audit::CURRENT.with_borrow(|audit| audit.boundaries.clone());
            assert!(boundaries.len() > 8);
            let mut retained = 71;
            for (reserved, requested, actual) in &boundaries {
                retained += requested;
                assert_eq!(*reserved, retained, "reservation must precede allocation");
                assert_eq!(
                    *actual,
                    Some(*requested),
                    "actual capacity exceeded prepaid storage"
                );
            }
            assert_eq!(admission.retained_bytes(), retained);
            for ordinal in 1..=boundaries.len() {
                allocation_audit::CURRENT.with_borrow_mut(|audit| {
                    *audit = allocation_audit::Audit {
                        fail_at: Some(ordinal),
                        ..Default::default()
                    }
                });
                let mut admission = SafeEditProjectionAdmission::new(71).unwrap();
                assert_eq!(copy(&mut admission), Err(SafeEditError::ProjectionBounds));
                assert_eq!(admission.retained_bytes(), 71);
                allocation_audit::CURRENT.with_borrow(|audit| {
                    assert_eq!(audit.boundaries.len(), ordinal);
                    assert!(audit.boundaries.last().unwrap().2.is_none());
                    assert!(
                        audit.boundaries[..ordinal - 1]
                            .iter()
                            .all(|(_, requested, actual)| *actual == Some(*requested))
                    );
                });
            }
            allocation_audit::CURRENT
                .with_borrow_mut(|audit| *audit = allocation_audit::Audit::default());
            eprintln!(
                "nested copy allocation boundaries ({diagnostic_copy}): {}",
                boundaries.len()
            );
        }
    }
}

fn copy_reference(
    value: &SemanticReference,
    admission: &mut SafeEditProjectionAdmission,
) -> Result<SemanticReference, SafeEditError> {
    fn name(
        value: &EntityName,
        admission: &mut SafeEditProjectionAdmission,
    ) -> Result<EntityName, SafeEditError> {
        EntityName::new(admission.copy_string(value.as_str())?)
            .map_err(|_| SafeEditError::SemanticMismatch)
    }
    Ok(match value {
        SemanticReference::Raw(value) => SemanticReference::Raw(admission.copy_string(value)?),
        SemanticReference::NodeId(value) => {
            SemanticReference::NodeId(admission.copy_string(value)?)
        }
        SemanticReference::Name(value) => SemanticReference::Name(name(value, admission)?),
        SemanticReference::Child { owner, name: value } => SemanticReference::Child {
            owner: admission.copy_id(owner)?,
            name: name(value, admission)?,
        },
        SemanticReference::Owner { child } => SemanticReference::Owner {
            child: admission.copy_id(child)?,
        },
        SemanticReference::OwnedChild { owner, child } => SemanticReference::OwnedChild {
            owner: admission.copy_id(owner)?,
            child: admission.copy_id(child)?,
        },
    })
}

fn copy_kinds(
    values: &[NodeKind],
    admission: &mut SafeEditProjectionAdmission,
) -> Result<Vec<NodeKind>, SafeEditError> {
    let mut result = admission.vector(values.len())?;
    result.extend_from_slice(values);
    Ok(result)
}

fn copy_ids(
    values: &[EntityId],
    admission: &mut SafeEditProjectionAdmission,
) -> Result<Vec<EntityId>, SafeEditError> {
    let mut result = admission.vector(values.len())?;
    for value in values {
        result.push(admission.copy_id(value)?);
    }
    Ok(result)
}

/// Full canonical terminal request represented by inspectable prepaid fields.
#[derive(PartialEq, Eq)]
pub struct SafeEditRequest {
    id: SafeEditRequestId,
    source: EntityId,
    category: oneagent_graph::SemanticReferenceCategory,
    reference: SemanticReference,
    expected_kinds: Vec<NodeKind>,
    candidates: Vec<EntityId>,
    state: oneagent_graph::ResolutionState,
    outcome: oneagent_graph::SemanticReferenceRequestOutcome,
    provenance: Vec<SafeEditProvenance>,
}

impl SafeEditRequest {
    /// Returns the canonical producer source identity.
    #[must_use]
    pub const fn source_node(&self) -> &EntityId {
        &self.source
    }

    /// Compares the complete resolution payload independently of producer identity/context.
    #[must_use]
    pub fn same_resolution_as(&self, other: &Self) -> bool {
        self.category == other.category
            && self.reference == other.reference
            && self.expected_kinds == other.expected_kinds
            && self.candidates == other.candidates
            && self.state == other.state
            && self.outcome == other.outcome
    }
    /// Copies a canonical terminal value; the original constructor remains its owner.
    ///
    /// # Errors
    /// Rejects nonterminal evidence, exhausted admission or allocation failure.
    pub fn copy(
        value: &SemanticReferenceRequest,
        admission: &mut SafeEditProjectionAdmission,
    ) -> Result<Self, SafeEditError> {
        admission.copy_transaction(|admission| Self::copy_inner(value, admission))
    }

    fn copy_inner(
        value: &SemanticReferenceRequest,
        admission: &mut SafeEditProjectionAdmission,
    ) -> Result<Self, SafeEditError> {
        require(value.outcome() != oneagent_graph::SemanticReferenceRequestOutcome::Collected)?;
        Ok(Self {
            id: SafeEditRequestId::copy(value.id(), admission)?,
            source: admission.copy_id(value.source_node())?,
            category: value.category(),
            reference: copy_reference(value.reference(), admission)?,
            expected_kinds: copy_kinds(value.expected_kinds(), admission)?,
            candidates: copy_ids(value.candidates(), admission)?,
            state: value.state(),
            outcome: value.outcome(),
            provenance: SafeEditProvenance::copy_all(value.provenance(), admission)?,
        })
    }
    /// Compares every field and the exact canonical ID, without allocating.
    #[must_use]
    pub fn matches(&self, value: &SemanticReferenceRequest) -> bool {
        self.id.matches(value.id())
            && &self.source == value.source_node()
            && self.category == value.category()
            && &self.reference == value.reference()
            && self.expected_kinds == value.expected_kinds()
            && self.candidates == value.candidates()
            && self.state == value.state()
            && self.outcome == value.outcome()
            && provenance_matches(&self.provenance, value.provenance())
    }
}

/// Full canonical diagnostic represented by inspectable prepaid fields.
#[derive(PartialEq, Eq)]
pub struct SafeEditDiagnostic {
    code: oneagent_graph::SemanticDiagnosticCode,
    severity: oneagent_graph::SemanticDiagnosticSeverity,
    kind: oneagent_graph::SemanticDiagnosticKind,
    message: String,
    reference: SemanticReference,
    source: Option<EntityId>,
    expected_kinds: Vec<NodeKind>,
    actual_kind: Option<NodeKind>,
    candidates: Vec<EntityId>,
    provenance: Vec<SafeEditProvenance>,
}

impl SafeEditDiagnostic {
    /// Compares every payload field except the separately mapped source/provenance.
    #[must_use]
    pub fn same_payload_as(&self, other: &Self) -> bool {
        self.code == other.code
            && self.severity == other.severity
            && self.kind == other.kind
            && self.message == other.message
            && self.reference == other.reference
            && self.expected_kinds == other.expected_kinds
            && self.actual_kind == other.actual_kind
            && self.candidates == other.candidates
    }
    /// Copies all fields from a canonical diagnostic without normalizing its contents.
    ///
    /// # Errors
    /// Rejects exhausted admission or allocation failure.
    pub fn copy(
        value: &SemanticDiagnostic,
        admission: &mut SafeEditProjectionAdmission,
    ) -> Result<Self, SafeEditError> {
        admission.copy_transaction(|admission| Self::copy_inner(value, admission))
    }

    fn copy_inner(
        value: &SemanticDiagnostic,
        admission: &mut SafeEditProjectionAdmission,
    ) -> Result<Self, SafeEditError> {
        Ok(Self {
            code: value.code(),
            severity: value.severity(),
            kind: value.kind(),
            message: admission.copy_string(value.message())?,
            reference: copy_reference(value.reference(), admission)?,
            source: value
                .source_node()
                .map(|id| admission.copy_id(id))
                .transpose()?,
            expected_kinds: copy_kinds(value.expected_kinds(), admission)?,
            actual_kind: value.actual_kind(),
            candidates: copy_ids(value.candidates(), admission)?,
            provenance: SafeEditProvenance::copy_all(value.provenance(), admission)?,
        })
    }
    /// Compares every typed field, option and ordered collection without allocation.
    #[must_use]
    pub fn matches(&self, value: &SemanticDiagnostic) -> bool {
        self.code == value.code()
            && self.severity == value.severity()
            && self.kind == value.kind()
            && self.message == value.message()
            && &self.reference == value.reference()
            && self.source.as_ref() == value.source_node()
            && self.expected_kinds == value.expected_kinds()
            && self.actual_kind == value.actual_kind()
            && self.candidates == value.candidates()
            && provenance_matches(&self.provenance, value.provenance())
    }
}

/// Explicit unchanged evidence or an independently produced expected value.
#[derive(Debug, PartialEq, Eq)]
pub enum SafeEditExpected<T> {
    /// Resolve the checked key against the exact retained before publication.
    Unchanged,
    /// Own a prepaid canonical expected value.
    Expected(T),
}

/// Captured Query pairing; excludes the parser's private working source map.
#[derive(PartialEq, Eq)]
pub struct SafeEditQueryFact {
    before_owner: EntityId,
    expected_owner: EntityId,
    binding: String,
    text: String,
    before_line: usize,
    expected_line: usize,
}

impl SafeEditQueryFact {
    /// Checks the exact binding/text pair before retaining prepaid owned copies.
    ///
    /// # Errors
    /// Rejects changed query semantics or declaration line.
    pub fn new(
        before: &oneagent_bsl::BslQuery,
        expected: &oneagent_bsl::BslQuery,
        admission: &mut SafeEditProjectionAdmission,
    ) -> Result<Self, SafeEditError> {
        require(
            before.binding_name() == expected.binding_name()
                && before.text() == expected.text()
                && before.line() == expected.line(),
        )?;
        admission.copy_transaction(|admission| {
            Ok(Self {
                before_owner: admission.copy_id(before.owner_id())?,
                expected_owner: admission.copy_id(expected.owner_id())?,
                binding: admission.copy_string(before.binding_name().as_str())?,
                text: admission.copy_string(before.text())?,
                before_line: before.line(),
                expected_line: expected.line(),
            })
        })
    }
}

/// Typed expected fact payload; key/value compatibility is checked at freeze.
#[derive(PartialEq, Eq)]
pub enum SafeEditFactValue {
    /// Canonical changed node identity.
    Identity(EntityId),
    /// Complete node or edge provenance.
    Provenance(SafeEditExpected<Vec<SafeEditProvenance>>),
    /// Full terminal request.
    Request(SafeEditExpected<SafeEditRequest>),
    /// Full diagnostic.
    Diagnostic(SafeEditExpected<SafeEditDiagnostic>),
    /// Exact captured Query pairing.
    Query(SafeEditQueryFact),
}

/// One prepaid record; checked against independently enumerated keys at freeze.
pub struct SafeEditProjectionFact {
    key: SafeEditFactKey,
    value: SafeEditFactValue,
}

impl SafeEditProjectionFact {
    /// Returns the typed before key for producer dispatch.
    #[must_use]
    pub const fn key(&self) -> &SafeEditFactKey {
        &self.key
    }

    /// Replaces a draft value while retaining its immutable checked key.
    ///
    /// # Errors
    /// Rejects a value incompatible with the key category.
    pub fn set_expected(&mut self, value: SafeEditFactValue) -> Result<(), SafeEditError> {
        require(matches!(
            (&self.key, &value),
            (
                SafeEditFactKey::NodeFact { .. } | SafeEditFactKey::EdgeFact { .. },
                SafeEditFactValue::Provenance(_)
            ) | (
                SafeEditFactKey::RequestFact { .. },
                SafeEditFactValue::Request(_)
            ) | (
                SafeEditFactKey::DiagnosticFact { .. },
                SafeEditFactValue::Diagnostic(_)
            )
        ))?;
        self.value = value;
        Ok(())
    }

    /// Checks typed key/value compatibility without allocating or deduplicating.
    ///
    /// # Errors
    /// Rejects incompatible key/value categories.
    pub fn new(key: SafeEditFactKey, value: SafeEditFactValue) -> Result<Self, SafeEditError> {
        require(matches!(
            (&key, &value),
            (
                SafeEditFactKey::NodeIdentity { .. },
                SafeEditFactValue::Identity(_)
            ) | (
                SafeEditFactKey::NodeFact { .. } | SafeEditFactKey::EdgeFact { .. },
                SafeEditFactValue::Provenance(_)
            ) | (
                SafeEditFactKey::RequestFact { .. },
                SafeEditFactValue::Request(_)
            ) | (
                SafeEditFactKey::DiagnosticFact { .. },
                SafeEditFactValue::Diagnostic(_)
            ) | (
                SafeEditFactKey::QueryFact { .. },
                SafeEditFactValue::Query(_)
            )
        ))?;
        Ok(Self { key, value })
    }
}

/// Frozen producer expectation, retaining the exact immutable before owners.
pub struct SafeEditProducerProjection {
    root: String,
    publication: WorkspacePublicationId,
    graph: Arc<SemanticGraph>,
    sources: SourceEvidenceSet,
    diagnostics: Arc<[SemanticDiagnostic]>,
    references: Arc<SemanticReferenceRequestLedger>,
    plan: RefactoringPlan,
    result_versions: Vec<SourceContentVersion>,
    facts: Vec<SafeEditProjectionFact>,
    consumed: Vec<std::cell::Cell<u8>>,
}

impl SafeEditProducerProjection {
    /// Freezes a complete independently produced inventory before any source write.
    ///
    /// Record and nested value storage must already be prepaid by the producer.
    /// This checks every expected before key independently, without allocating a map.
    ///
    /// # Errors
    /// Rejects missing, extra, duplicate, conflicting or ineligible mappings.
    #[allow(clippy::too_many_lines, clippy::needless_pass_by_value)]
    pub fn new(
        input: SafeEditProducerInput<'_>,
        mut facts: Vec<SafeEditProjectionFact>,
        admission: &mut SafeEditProjectionAdmission,
    ) -> Result<Self, SafeEditError> {
        fn identity<'b>(facts: &'b [SafeEditProjectionFact], id: &'b EntityId) -> &'b EntityId {
            facts
                .iter()
                .find_map(|fact| match (&fact.key, &fact.value) {
                    (
                        SafeEditFactKey::NodeIdentity { before },
                        SafeEditFactValue::Identity(expected),
                    ) if before == id => Some(expected),
                    _ => None,
                })
                .unwrap_or(id)
        }
        // Consume the producer draft at the freeze boundary; only the checked
        // immutable binding escapes into the retained projection.
        facts.sort_unstable_by(|left, right| left.key.cmp(&right.key));
        require(facts.windows(2).all(|pair| pair[0].key != pair[1].key))?;
        let has = |key: &SafeEditFactKey| facts.binary_search_by(|fact| fact.key.cmp(key)).is_ok();
        let target = input.plan.target().target_node_id();
        let mut required = 0usize;
        for node in input.graph.nodes() {
            // Temporary keys borrow their IDs through the following comparisons;
            // no cloning or uncharged key inventory is needed.
            require(facts.iter().any(|fact| matches!(&fact.key, SafeEditFactKey::NodeFact { before } if before == node.id())))?;
            required = required
                .checked_add(1)
                .ok_or(SafeEditError::ProjectionBounds)?;
            if node.kind() == NodeKind::Query {
                require(
                    input
                        .modules
                        .iter()
                        .all(|module| module.document.format() == SourceFormat::Edt),
                )?;
                let record = facts
                    .iter()
                    .find_map(|fact| match (&fact.key, &fact.value) {
                        (
                            SafeEditFactKey::QueryFact { before },
                            SafeEditFactValue::Query(query),
                        ) if before == node.id() => Some(query),
                        _ => None,
                    })
                    .ok_or(SafeEditError::SemanticMismatch)?;
                let mut owners = input
                    .graph
                    .edges()
                    .filter(|edge| edge.kind() == EdgeKind::Contains && edge.target() == node.id());
                require(
                    owners
                        .next()
                        .is_some_and(|edge| edge.source() == &record.before_owner)
                        && owners.next().is_none(),
                )?;
                require(
                    record.binding == node.name().as_str()
                        && identity(&facts, &record.before_owner) == &record.expected_owner
                        && record.before_line == record.expected_line,
                )?;
                // Canonical identities are checked after reserving their exact known
                // before/expected lengths from captured producer records.
                let scratch = node
                    .id()
                    .as_str()
                    .len()
                    .checked_add(identity(&facts, node.id()).as_str().len())
                    .and_then(|n| n.checked_mul(2))
                    .ok_or(SafeEditError::ProjectionBounds)?;
                admission.reserve(scratch)?;
                let canonical = (|| {
                    let old = oneagent_bsl::bsl_query_id(&record.before_owner, &record.binding)
                        .map_err(|_| SafeEditError::SemanticMismatch)?;
                    let new = oneagent_bsl::bsl_query_id(&record.expected_owner, &record.binding)
                        .map_err(|_| SafeEditError::SemanticMismatch)?;
                    require(&old == node.id() && &new == identity(&facts, node.id()))
                })();
                admission.release(scratch)?;
                canonical?;
                required = required
                    .checked_add(1)
                    .ok_or(SafeEditError::ProjectionBounds)?;
            }
            let is_dependent = node.kind() == NodeKind::Query
                && input.graph.edges().any(|edge| {
                    edge.kind() == EdgeKind::Contains
                        && edge.source() == target
                        && edge.target() == node.id()
                });
            if node.id() == target || is_dependent {
                require(identity(&facts, node.id()) != node.id())?;
                required = required
                    .checked_add(1)
                    .ok_or(SafeEditError::ProjectionBounds)?;
            } else {
                require(identity(&facts, node.id()) == node.id())?;
            }
        }
        require(identity(&facts, target) == input.plan.target().expected_post_rename_node_id())?;
        for fact in &facts {
            if let (
                SafeEditFactKey::NodeIdentity { before },
                SafeEditFactValue::Identity(expected),
            ) = (&fact.key, &fact.value)
            {
                require(
                    input.graph.node(before).is_some()
                        && before != expected
                        && input.graph.node(expected).is_none(),
                )?;
                require(facts.iter().filter(|other| matches!(&other.value, SafeEditFactValue::Identity(id) if id == expected)).count() == 1)?;
            }
        }
        for edge in input.graph.edges() {
            require(facts.iter().any(|fact| matches!(&fact.key, SafeEditFactKey::EdgeFact { before_source, kind, before_target }
                if before_source == edge.source() && *kind == edge.kind() && before_target == edge.target())))?;
            required = required
                .checked_add(1)
                .ok_or(SafeEditError::ProjectionBounds)?;
        }
        for request in input.references.requests() {
            require(facts.iter().any(|fact| matches!(&fact.key, SafeEditFactKey::RequestFact { before } if before.matches(request.id()))))?;
            required = required
                .checked_add(1)
                .ok_or(SafeEditError::ProjectionBounds)?;
        }
        for before_ordinal in 0..input.diagnostics.len() {
            require(has(&SafeEditFactKey::DiagnosticFact { before_ordinal }))?;
            required = required
                .checked_add(1)
                .ok_or(SafeEditError::ProjectionBounds)?;
        }
        require(required == facts.len())?;
        let mut result_versions = admission.vector::<SourceContentVersion>(input.modules.len())?;
        for module in input.modules {
            result_versions.push(SourceContentVersion::from_bytes(module.result));
        }
        let bit_bytes = facts
            .len()
            .checked_add(7)
            .ok_or(SafeEditError::ProjectionBounds)?
            / 8;
        let mut consumed = admission.vector::<std::cell::Cell<u8>>(bit_bytes)?;
        consumed.resize_with(bit_bytes, || std::cell::Cell::new(0));
        Ok(Self {
            root: admission
                .copy_string(input.root.to_str().ok_or(SafeEditError::SemanticMismatch)?)?,
            publication: input.publication,
            graph: Arc::clone(input.graph),
            sources: input.sources.copy_for_safe_edit(admission)?,
            diagnostics: Arc::clone(input.diagnostics),
            references: Arc::clone(input.references),
            plan: input.plan.copy_for_safe_edit(admission)?,
            result_versions,
            facts,
            consumed,
        })
    }

    fn bound_to(
        &self,
        plan: &RefactoringPlan,
        before: &SafeEditEvidence<'_>,
        after: &SafeEditEvidence<'_>,
    ) -> Result<(), SafeEditError> {
        compare_plan(&self.plan, plan)?;
        require(
            self.publication == plan.request().expected_publication_id()
                && std::path::Path::new(&self.root) == before.root
                && std::ptr::eq(self.graph.as_ref(), before.graph)
                && &self.sources == before.sources
                && self.diagnostics.as_ref() == before.diagnostics
                && self.references.as_ref() == before.references
                && self.result_versions.len() == after.sources.documents().len(),
        )?;
        for (version, document) in self.result_versions.iter().zip(after.sources.documents()) {
            require(*version == document.content_version())?;
        }
        Ok(())
    }

    fn identity<'a>(&'a self, id: &'a EntityId) -> &'a EntityId {
        self.facts
            .iter()
            .find_map(|fact| match (&fact.key, &fact.value) {
                (
                    SafeEditFactKey::NodeIdentity { before },
                    SafeEditFactValue::Identity(expected),
                ) if before == id => Some(expected),
                _ => None,
            })
            .unwrap_or(id)
    }

    fn consume(
        &self,
        matches: impl Fn(&SafeEditFactKey) -> bool,
    ) -> Result<&SafeEditFactValue, SafeEditError> {
        let index = self
            .facts
            .iter()
            .position(|fact| matches(&fact.key))
            .ok_or(SafeEditError::SemanticMismatch)?;
        let mask = 1u8 << (index % 8);
        let byte = &self.consumed[index / 8];
        require(byte.get() & mask == 0)?;
        byte.set(byte.get() | mask);
        Ok(&self.facts[index].value)
    }

    fn consume_identities_and_queries(&self) -> Result<(), SafeEditError> {
        require(self.consumed.iter().all(|byte| byte.get() == 0))?;
        for fact in &self.facts {
            if matches!(
                fact.key,
                SafeEditFactKey::NodeIdentity { .. } | SafeEditFactKey::QueryFact { .. }
            ) {
                self.consume(|key| key == &fact.key)?;
            }
        }
        Ok(())
    }

    fn complete(&self) -> Result<(), SafeEditError> {
        require(
            (0..self.facts.len())
                .all(|index| self.consumed[index / 8].get() & (1 << (index % 8)) != 0),
        )
    }
}

/// Borrowed exact target and coordinate transformation; never produces graph facts.
pub struct SafeEditProjection<'a> {
    plan: &'a RefactoringPlan,
    producer: &'a SafeEditProducerProjection,
}

impl<'a> SafeEditProjection<'a> {
    /// Creates a borrowed transformation for already bounded evidence.
    #[must_use]
    pub const fn new(plan: &'a RefactoringPlan, producer: &'a SafeEditProducerProjection) -> Self {
        Self { plan, producer }
    }

    fn offset(&self, document: &SourceDocument, offset: usize) -> Result<usize, SafeEditError> {
        let mut transformed = offset;
        for operation in self
            .plan
            .operations()
            .iter()
            .rev()
            .filter(|o| o.document_id() == document.id())
        {
            let range = operation.range();
            if offset >= range.end_byte() {
                transformed = transformed
                    .checked_sub(range.end_byte() - range.start_byte())
                    .and_then(|n| n.checked_add(operation.replacement().len()))
                    .ok_or(SafeEditError::SemanticMismatch)?;
            } else if offset > range.start_byte() {
                return Err(SafeEditError::SemanticMismatch);
            }
        }
        Ok(transformed)
    }

    fn reference_matches(
        &self,
        before: &SemanticReference,
        targets: &[EntityId],
        expected: &SemanticReference,
    ) -> bool {
        let renamed = targets.len() == 1 && &targets[0] == self.plan.target().target_node_id();
        let name = |old: &EntityName, new: &EntityName| {
            new.as_str()
                == if renamed {
                    self.plan.request().desired_name()
                } else {
                    old.as_str()
                }
        };
        match (before, expected) {
            (SemanticReference::Raw(old), SemanticReference::Raw(new)) => !renamed && old == new,
            (SemanticReference::NodeId(old), SemanticReference::NodeId(new)) => {
                let mapped = self
                    .producer
                    .facts
                    .iter()
                    .find_map(|fact| match (&fact.key, &fact.value) {
                        (
                            SafeEditFactKey::NodeIdentity { before },
                            SafeEditFactValue::Identity(expected),
                        ) if before.as_str() == old => Some(expected.as_str()),
                        _ => None,
                    })
                    .unwrap_or(old);
                new == mapped
            }
            (SemanticReference::Name(old), SemanticReference::Name(new)) => name(old, new),
            (
                SemanticReference::Child { owner, name: old },
                SemanticReference::Child {
                    owner: new_owner,
                    name: new,
                },
            ) => self.producer.identity(owner) == new_owner && name(old, new),
            (SemanticReference::Owner { child }, SemanticReference::Owner { child: new }) => {
                self.producer.identity(child) == new
            }
            (
                SemanticReference::OwnedChild { owner, child },
                SemanticReference::OwnedChild {
                    owner: new_owner,
                    child: new_child,
                },
            ) => {
                self.producer.identity(owner) == new_owner
                    && self.producer.identity(child) == new_child
            }
            _ => false,
        }
    }
}

fn require(equal: bool) -> Result<(), SafeEditError> {
    if equal {
        Ok(())
    } else {
        Err(SafeEditError::SemanticMismatch)
    }
}

/// Compares complete unchanged evidence, including provenance omitted by edge equality.
///
/// # Errors
/// Returns a closed error for any non-equivalent record.
pub fn validate_equivalence(
    before: &SafeEditEvidence<'_>,
    after: &SafeEditEvidence<'_>,
) -> Result<(), SafeEditError> {
    require(
        before.root == after.root
            && before.sources == after.sources
            && before.diagnostics == after.diagnostics
            && before.references == after.references
            && before.reference_statistics == after.reference_statistics
            && before.report == after.report
            && before.validation == after.validation
            && before.rules == after.rules
            && before.findings == after.findings,
    )?;
    require(before.graph.nodes().eq(after.graph.nodes()))?;
    require(before.graph.edges().count() == after.graph.edges().count())?;
    for (old, new) in before.graph.edges().zip(after.graph.edges()) {
        require(old == new && old.provenance() == new.provenance())?;
    }
    Ok(())
}

/// Compares complete before/after production evidence under the exact rename.
///
/// Call once for every Configuration; the caller must additionally compare the
/// complete Configuration inventory and source-format identities. Unclassifiable
/// producer differences fail closed rather than being normalized by text search.
///
/// # Errors
/// Returns `SemanticMismatch` for any missing, extra or unequal record.
#[allow(clippy::too_many_lines)]
pub fn validate_postconditions(
    plan: &RefactoringPlan,
    before: &SafeEditEvidence<'_>,
    after: &SafeEditEvidence<'_>,
    producer: &SafeEditProducerProjection,
) -> Result<(), SafeEditError> {
    producer.bound_to(plan, before, after)?;
    require(
        before.diagnostics.windows(2).all(|pair| pair[0] < pair[1])
            && after.diagnostics.windows(2).all(|pair| pair[0] < pair[1]),
    )?;
    producer.consume_identities_and_queries()?;
    require(
        before.root == after.root
            && before.sources.configuration_id() == after.sources.configuration_id(),
    )?;
    require(
        before.validation.is_valid()
            && after.validation.is_valid()
            && before.validation == after.validation,
    )?;
    require(
        before.report == after.report && before.reference_statistics == after.reference_statistics,
    )?;
    let projection = SafeEditProjection::new(plan, producer);
    require(before.sources.documents().len() == after.sources.documents().len())?;
    for (old, new) in before
        .sources
        .documents()
        .iter()
        .zip(after.sources.documents())
    {
        require(
            old.id() == new.id()
                && old.path() == new.path()
                && old.format() == new.format()
                && old.module_role() == new.module_role()
                && old.completeness() == new.completeness(),
        )?;
        require(exact_result_matches(plan, old, new.raw_content()))?;
        require(old.occurrences().len() == new.occurrences().len())?;
        for (old_occurrence, new_occurrence) in old.occurrences().iter().zip(new.occurrences()) {
            let operation = plan
                .operations()
                .iter()
                .find(|o| o.document_id() == old.id() && o.range() == old_occurrence.range());
            let start = projection.offset(old, old_occurrence.range().start_byte())?;
            let end = projection.offset(old, old_occurrence.range().end_byte())?;
            require(
                new_occurrence.document_id() == old.id()
                    && new_occurrence.content_version() == new.content_version()
                    && new_occurrence.range().start_byte() == start
                    && new_occurrence.range().end_byte() == end
                    && new_occurrence.kind() == old_occurrence.kind()
                    && new_occurrence.token()
                        == operation.map_or(old_occurrence.token(), |o| o.replacement())
                    && new_occurrence.lexical_owner_token() == old_occurrence.lexical_owner_token()
                    && new_occurrence.mapped_target_id()
                        == old_occurrence
                            .mapped_target_id()
                            .map(|id| producer.identity(id))
                    && new_occurrence.resolution() == old_occurrence.resolution(),
            )?;
        }
    }
    require(before.graph.nodes().count() == after.graph.nodes().count())?;
    for old in before.graph.nodes() {
        let id = producer.identity(old.id());
        let new = after
            .graph
            .node(id)
            .ok_or(SafeEditError::SemanticMismatch)?;
        let expected_name = if old.id() == plan.target().target_node_id() {
            plan.request().desired_name()
        } else {
            old.name().as_str()
        };
        let SafeEditFactValue::Provenance(expected) = producer.consume(
            |key| matches!(key, SafeEditFactKey::NodeFact { before } if before == old.id()),
        )?
        else {
            return Err(SafeEditError::SemanticMismatch);
        };
        let provenance_equal = match expected {
            SafeEditExpected::Unchanged => new.provenance() == old.provenance(),
            SafeEditExpected::Expected(value) => provenance_matches(value, new.provenance()),
        };
        require(
            new.name().as_str() == expected_name
                && new.kind() == old.kind()
                && new.payload() == old.payload()
                && provenance_equal,
        )?;
    }
    require(before.graph.edges().count() == after.graph.edges().count())?;
    for old in before.graph.edges() {
        let source = producer.identity(old.source());
        let target = producer.identity(old.target());
        let new = after
            .graph
            .edges()
            .find(|e| e.source() == source && e.target() == target && e.kind() == old.kind())
            .ok_or(SafeEditError::SemanticMismatch)?;
        let SafeEditFactValue::Provenance(expected) = producer.consume(|key| matches!(key,
            SafeEditFactKey::EdgeFact { before_source, kind, before_target }
                if before_source == old.source() && *kind == old.kind() && before_target == old.target()))?
            else { return Err(SafeEditError::SemanticMismatch); };
        let provenance_equal = match expected {
            SafeEditExpected::Unchanged => new.provenance() == old.provenance(),
            SafeEditExpected::Expected(value) => provenance_matches(value, new.provenance()),
        };
        require(provenance_equal)?;
        // GraphEdge stores no independent ID. The lookup above compares every
        // input to Graph's canonical edge-ID derivation, without allocating four
        // duplicate NodeIds and two identical derived strings after source writes.
    }
    require(before.references.len() == after.references.len())?;
    for request in before.references.requests() {
        let SafeEditFactValue::Request(expected) = producer.consume(|key| matches!(key, SafeEditFactKey::RequestFact { before } if before.matches(request.id())))?
            else { return Err(SafeEditError::SemanticMismatch); };
        let mut matches = after
            .references
            .requests()
            .iter()
            .filter(|actual| match expected {
                SafeEditExpected::Unchanged => *actual == request,
                SafeEditExpected::Expected(value) => value.matches(actual),
            });
        let expected = matches.next().ok_or(SafeEditError::SemanticMismatch)?;
        require(matches.next().is_none())?;
        require(
            expected.source_node() == producer.identity(request.source_node())
                && expected.category() == request.category()
                && expected.state() == request.state()
                && expected.outcome() == request.outcome()
                && expected.expected_kinds() == request.expected_kinds()
                && projection.reference_matches(
                    request.reference(),
                    request.candidates(),
                    expected.reference(),
                )
                && expected
                    .candidates()
                    .iter()
                    .eq(request.candidates().iter().map(|id| producer.identity(id))),
        )?;
        require(
            after
                .references
                .requests()
                .iter()
                .filter(|actual| *actual == expected)
                .count()
                == 1,
        )?;
    }
    require(before.diagnostics.len() == after.diagnostics.len())?;
    for (ordinal, diagnostic) in before.diagnostics.iter().enumerate() {
        let SafeEditFactValue::Diagnostic(expected) = producer.consume(|key| matches!(key, SafeEditFactKey::DiagnosticFact { before_ordinal } if *before_ordinal == ordinal))?
            else { return Err(SafeEditError::SemanticMismatch); };
        let mut matches = after.diagnostics.iter().filter(|actual| match expected {
            SafeEditExpected::Unchanged => *actual == diagnostic,
            SafeEditExpected::Expected(value) => value.matches(actual),
        });
        let expected = matches.next().ok_or(SafeEditError::SemanticMismatch)?;
        require(matches.next().is_none())?;
        require(
            expected.code() == diagnostic.code()
                && expected.kind() == diagnostic.kind()
                && expected.severity() == diagnostic.severity()
                && expected.expected_kinds() == diagnostic.expected_kinds()
                && expected.actual_kind() == diagnostic.actual_kind()
                && expected.message() == diagnostic.message()
                && expected.source_node()
                    == diagnostic.source_node().map(|id| producer.identity(id))
                && expected.candidates().iter().eq(diagnostic
                    .candidates()
                    .iter()
                    .map(|id| producer.identity(id)))
                && projection.reference_matches(
                    diagnostic.reference(),
                    diagnostic.candidates(),
                    expected.reference(),
                ),
        )?;
        require(
            after
                .diagnostics
                .iter()
                .filter(|actual| *actual == expected)
                .count()
                == 1,
        )?;
    }
    producer.complete()?;
    // Report producers validate all derived identities, messages and counts.
    require(before.rules == after.rules)?;
    let expected_findings = DiagnosticEngine
        .build_with_rules(
            after.diagnostics,
            after.validation,
            after.rules.diagnostics(),
            &DiagnosticPolicy::default(),
        )
        .map_err(|_| SafeEditError::SemanticMismatch)?;
    let original_findings = DiagnosticEngine
        .build_with_rules(
            before.diagnostics,
            before.validation,
            before.rules.diagnostics(),
            &DiagnosticPolicy::default(),
        )
        .map_err(|_| SafeEditError::SemanticMismatch)?;
    require(&original_findings == before.findings && &expected_findings == after.findings)
}

macro_rules! redacted_projection_debug {
    ($($ty:ty),+ $(,)?) => { $(impl std::fmt::Debug for $ty {
        fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
            formatter.debug_struct(stringify!($ty)).finish_non_exhaustive()
        }
    })+ };
}
redacted_projection_debug!(
    SafeEditFactKey,
    SafeEditRequestId,
    SafeEditProvenance,
    SafeEditRequest,
    SafeEditDiagnostic,
    SafeEditQueryFact,
    SafeEditFactValue,
    SafeEditProjectionFact
);
