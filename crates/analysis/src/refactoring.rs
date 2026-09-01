//! Immutable source evidence and deterministic read-only refactoring planning.
//!
//! This module owns source-independent retained documents, occurrences, plans,
//! and previews. It performs no filesystem access or source mutation.

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{Display, Formatter};
use std::sync::Arc;

use oneagent_bsl::{BslSymbolKind, bsl_callable_id, bsl_name_key, bsl_names_equal};
use oneagent_common::{EntityId, SourcePath, SourcePosition, SourceSpan, sha256, sha256_hex};
use oneagent_graph::{GraphNode, NodeId, NodeKind, SemanticGraph, SemanticGraphQuery};

pub use crate::change_impact::ChangeImpactPublicationId as WorkspacePublicationId;

/// Maximum raw bytes retained by one source document.
pub const MAX_SOURCE_DOCUMENT_BYTES: usize = 1_048_576;
/// Maximum documents retained by one Configuration source-evidence set.
pub const MAX_SOURCE_DOCUMENTS_PER_CONFIGURATION: usize = 4_096;
/// Maximum aggregate raw bytes retained by one Configuration source-evidence set.
pub const MAX_SOURCE_BYTES_PER_CONFIGURATION: usize = 67_108_864;
/// Maximum exact occurrences retained by one source document.
pub const MAX_SOURCE_OCCURRENCES_PER_DOCUMENT: usize = 4_096;
/// Maximum UTF-8 bytes in a source-evidence identity component.
pub const MAX_SOURCE_IDENTITY_BYTES: usize = 4_096;
/// Maximum UTF-8 bytes in one captured BSL identifier token.
pub const MAX_SOURCE_IDENTIFIER_BYTES: usize = 256;

const UTF8_BOM: &[u8; 3] = b"\xef\xbb\xbf";

/// Closed immutable source-evidence construction failure kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SourceEvidenceErrorKind {
    /// A document or mapped target identity violates its byte bound.
    InvalidIdentity,
    /// A path is not a confined Workspace-relative Configuration descendant.
    InvalidConfinedPath,
    /// An inclusive source-evidence bound was exceeded.
    BoundExceeded,
    /// Captured source bytes are not UTF-8.
    UnsupportedEncoding,
    /// Captured source starts with more than one UTF-8 BOM.
    MalformedBom,
    /// The format and BSL module role combination is unsupported.
    UnsupportedSourceFormat,
    /// Occurrence document or content-version evidence is incompatible.
    IncompatibleEvidence,
    /// An occurrence range, token, or mapping invariant is invalid.
    InvalidOccurrence,
    /// Unequal occurrence evidence occupies the same exact range.
    DuplicateConflict,
    /// Unequal occurrence ranges intersect.
    OverlappingOccurrences,
    /// Checked source-evidence arithmetic overflowed.
    ArithmeticOverflow,
}

/// Redacted source-evidence error containing only a category and optional counts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceEvidenceError {
    kind: SourceEvidenceErrorKind,
    actual: Option<usize>,
    maximum: Option<usize>,
}

impl SourceEvidenceError {
    const fn new(kind: SourceEvidenceErrorKind) -> Self {
        Self {
            kind,
            actual: None,
            maximum: None,
        }
    }

    const fn bounded(actual: usize, maximum: usize) -> Self {
        Self {
            kind: SourceEvidenceErrorKind::BoundExceeded,
            actual: Some(actual),
            maximum: Some(maximum),
        }
    }

    /// Returns the closed failure kind.
    #[must_use]
    pub const fn kind(self) -> SourceEvidenceErrorKind {
        self.kind
    }

    /// Returns the rejected count for a bounded failure.
    #[must_use]
    pub const fn actual(self) -> Option<usize> {
        self.actual
    }

    /// Returns the accepted maximum for a bounded failure.
    #[must_use]
    pub const fn maximum(self) -> Option<usize> {
        self.maximum
    }
}

impl Display for SourceEvidenceError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match (self.actual, self.maximum) {
            (Some(actual), Some(maximum)) => write!(
                formatter,
                "source evidence rejected a bounded count: kind={:?}, actual={actual}, maximum={maximum}",
                self.kind
            ),
            _ => write!(
                formatter,
                "source evidence rejected input: kind={:?}",
                self.kind
            ),
        }
    }
}

impl std::error::Error for SourceEvidenceError {}

fn validate_identity_component(value: &EntityId) -> Result<(), SourceEvidenceError> {
    if value.as_str().len() > MAX_SOURCE_IDENTITY_BYTES {
        return Err(SourceEvidenceError::bounded(
            value.as_str().len(),
            MAX_SOURCE_IDENTITY_BYTES,
        ));
    }
    Ok(())
}

/// Structured identity of exactly one BSL document in a Configuration.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourceDocumentId {
    configuration_id: EntityId,
    module_id: EntityId,
}

impl SourceDocumentId {
    /// Creates a bounded structured source-document identity.
    ///
    /// # Errors
    ///
    /// Returns a redacted error when either identity component is over-bound.
    pub fn new(
        configuration_id: EntityId,
        module_id: EntityId,
    ) -> Result<Self, SourceEvidenceError> {
        validate_identity_component(&configuration_id)?;
        validate_identity_component(&module_id)?;
        Ok(Self {
            configuration_id,
            module_id,
        })
    }

    /// Returns the canonical Configuration identity.
    #[must_use]
    pub const fn configuration_id(&self) -> &EntityId {
        &self.configuration_id
    }

    /// Returns the canonical Module identity.
    #[must_use]
    pub const fn module_id(&self) -> &EntityId {
        &self.module_id
    }
}

/// Accepted production source format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SourceFormat {
    /// 1C:EDT project source.
    Edt,
    /// Hierarchical Designer XML source.
    DesignerXml,
}

/// Accepted BSL module role.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BslModuleRole {
    /// Metadata object module.
    Object,
    /// Metadata manager module.
    Manager,
    /// Common module.
    Common,
    /// Managed or ordinary form module.
    Form,
    /// Command module.
    Command,
}

/// Workspace-relative path proven to descend from one Configuration root.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ConfinedSourcePath(SourcePath);

impl ConfinedSourcePath {
    /// Validates Workspace-relative and Configuration-root confinement.
    ///
    /// # Errors
    ///
    /// Returns a redacted error when either path is absolute or the source path
    /// is not a strict descendant of `configuration_root`.
    pub fn new(
        workspace_relative_path: SourcePath,
        configuration_root: &SourcePath,
    ) -> Result<Self, SourceEvidenceError> {
        let path = workspace_relative_path.as_str();
        let root = configuration_root.as_str();
        if !is_relative(path)
            || !is_relative(root)
            || path == root
            || !path
                .strip_prefix(root)
                .is_some_and(|suffix| suffix.starts_with('/'))
        {
            return Err(SourceEvidenceError::new(
                SourceEvidenceErrorKind::InvalidConfinedPath,
            ));
        }
        Ok(Self(workspace_relative_path))
    }

    /// Returns the normalized Workspace-relative path.
    #[must_use]
    pub const fn path(&self) -> &SourcePath {
        &self.0
    }
}

fn is_relative(path: &str) -> bool {
    !path.starts_with('/') && path.as_bytes().get(1) != Some(&b':')
}

/// Deterministic exact-content identity `(raw byte length, SHA-256 bytes)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourceContentVersion {
    raw_byte_len: usize,
    digest: [u8; 32],
}

impl SourceContentVersion {
    /// Computes a deterministic version from exact captured bytes.
    #[must_use]
    pub fn from_bytes(raw: &[u8]) -> Self {
        Self {
            raw_byte_len: raw.len(),
            digest: sha256(raw),
        }
    }

    /// Returns the exact raw byte length.
    #[must_use]
    pub const fn raw_byte_len(self) -> usize {
        self.raw_byte_len
    }

    /// Returns all 32 SHA-256 digest bytes.
    #[must_use]
    pub const fn digest(self) -> [u8; 32] {
        self.digest
    }
}

/// Non-empty zero-based half-open range in exact raw UTF-8 bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourceByteRange {
    start_byte: usize,
    end_byte: usize,
}

impl SourceByteRange {
    /// Creates a non-empty half-open range.
    ///
    /// # Errors
    ///
    /// Returns a redacted error when the range is empty or reversed.
    pub const fn new(start_byte: usize, end_byte: usize) -> Result<Self, SourceEvidenceError> {
        if start_byte >= end_byte {
            return Err(SourceEvidenceError::new(
                SourceEvidenceErrorKind::InvalidOccurrence,
            ));
        }
        Ok(Self {
            start_byte,
            end_byte,
        })
    }

    /// Returns the inclusive start byte.
    #[must_use]
    pub const fn start_byte(self) -> usize {
        self.start_byte
    }

    /// Returns the exclusive end byte.
    #[must_use]
    pub const fn end_byte(self) -> usize {
        self.end_byte
    }
}

/// Exact BSL occurrence category admitted by the first refactoring family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SourceOccurrenceKind {
    /// Top-level Procedure or Function declaration identifier.
    Declaration,
    /// Unqualified direct call final identifier.
    LocalCall,
    /// Qualified direct call final identifier.
    QualifiedCall,
}

/// Resolution outcome retained for one syntactically relevant occurrence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SourceOccurrenceResolution {
    /// The occurrence maps to exactly one Graph target.
    Unique,
    /// No accepted Graph target could be resolved.
    Unresolved,
    /// More than one accepted Graph target could match.
    Ambiguous,
    /// The syntactic candidate is outside the first-family mapping contract.
    Unsupported,
}

/// Immutable exact BSL declaration or direct-call occurrence.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourceOccurrence {
    document_id: SourceDocumentId,
    content_version: SourceContentVersion,
    range: SourceByteRange,
    kind: SourceOccurrenceKind,
    token: Box<str>,
    mapped_target_id: Option<EntityId>,
    resolution: SourceOccurrenceResolution,
}

impl SourceOccurrence {
    /// Creates a bounded occurrence with a checked resolution/target invariant.
    ///
    /// # Errors
    ///
    /// Returns a redacted error for an empty or over-bound token, over-bound
    /// target identity, or a target inconsistent with the resolution.
    pub fn new(
        document_id: SourceDocumentId,
        content_version: SourceContentVersion,
        range: SourceByteRange,
        kind: SourceOccurrenceKind,
        token: impl Into<String>,
        mapped_target_id: Option<EntityId>,
        resolution: SourceOccurrenceResolution,
    ) -> Result<Self, SourceEvidenceError> {
        let token = token.into();
        if token.is_empty() {
            return Err(SourceEvidenceError::new(
                SourceEvidenceErrorKind::InvalidOccurrence,
            ));
        }
        if token.len() > MAX_SOURCE_IDENTIFIER_BYTES {
            return Err(SourceEvidenceError::bounded(
                token.len(),
                MAX_SOURCE_IDENTIFIER_BYTES,
            ));
        }
        if let Some(target) = mapped_target_id.as_ref() {
            validate_identity_component(target)?;
        }
        if (resolution == SourceOccurrenceResolution::Unique) != mapped_target_id.is_some() {
            return Err(SourceEvidenceError::new(
                SourceEvidenceErrorKind::InvalidOccurrence,
            ));
        }
        Ok(Self {
            document_id,
            content_version,
            range,
            kind,
            token: token.into_boxed_str(),
            mapped_target_id,
            resolution,
        })
    }

    /// Returns the containing document identity.
    #[must_use]
    pub const fn document_id(&self) -> &SourceDocumentId {
        &self.document_id
    }

    /// Returns the exact content version used during extraction.
    #[must_use]
    pub const fn content_version(&self) -> SourceContentVersion {
        self.content_version
    }

    /// Returns the exact raw identifier range.
    #[must_use]
    pub const fn range(&self) -> SourceByteRange {
        self.range
    }

    /// Returns the occurrence category.
    #[must_use]
    pub const fn kind(&self) -> SourceOccurrenceKind {
        self.kind
    }

    /// Returns the captured identifier token.
    #[must_use]
    pub fn token(&self) -> &str {
        &self.token
    }

    /// Returns the uniquely mapped target, when present.
    #[must_use]
    pub const fn mapped_target_id(&self) -> Option<&EntityId> {
        self.mapped_target_id.as_ref()
    }

    /// Returns the retained mapping outcome.
    #[must_use]
    pub const fn resolution(&self) -> SourceOccurrenceResolution {
        self.resolution
    }
}

/// Family-specific proof that every relevant candidate was retained.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SourceEvidenceCompleteness {
    /// Complete evidence for `bsl_callable_rename_v1`.
    BslCallableRenameV1,
}

/// One immutable bounded source document and canonical exact occurrence ledger.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceDocument {
    id: SourceDocumentId,
    format: SourceFormat,
    module_role: BslModuleRole,
    path: ConfinedSourcePath,
    raw_content: Arc<[u8]>,
    content_version: SourceContentVersion,
    occurrences: Arc<[SourceOccurrence]>,
    completeness: SourceEvidenceCompleteness,
}

impl SourceDocument {
    /// Validates and retains one complete immutable document atomically.
    ///
    /// Exact duplicate occurrences collapse after the input-count bound is
    /// checked. Unequal same-range or overlapping evidence fails closed.
    ///
    /// # Errors
    ///
    /// Returns a redacted closed error for invalid encoding, BOM, bounds,
    /// incompatible versions/identities, invalid ranges/tokens, or conflicts.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: SourceDocumentId,
        format: SourceFormat,
        module_role: BslModuleRole,
        path: ConfinedSourcePath,
        raw_content: Vec<u8>,
        mut occurrences: Vec<SourceOccurrence>,
        completeness: SourceEvidenceCompleteness,
    ) -> Result<Self, SourceEvidenceError> {
        if format == SourceFormat::DesignerXml
            && matches!(module_role, BslModuleRole::Form | BslModuleRole::Command)
        {
            return Err(SourceEvidenceError::new(
                SourceEvidenceErrorKind::UnsupportedSourceFormat,
            ));
        }
        if raw_content.len() > MAX_SOURCE_DOCUMENT_BYTES {
            return Err(SourceEvidenceError::bounded(
                raw_content.len(),
                MAX_SOURCE_DOCUMENT_BYTES,
            ));
        }
        if occurrences.len() > MAX_SOURCE_OCCURRENCES_PER_DOCUMENT {
            return Err(SourceEvidenceError::bounded(
                occurrences.len(),
                MAX_SOURCE_OCCURRENCES_PER_DOCUMENT,
            ));
        }
        let source = std::str::from_utf8(&raw_content)
            .map_err(|_| SourceEvidenceError::new(SourceEvidenceErrorKind::UnsupportedEncoding))?;
        if raw_content
            .strip_prefix(UTF8_BOM)
            .is_some_and(|remainder| remainder.starts_with(UTF8_BOM))
        {
            return Err(SourceEvidenceError::new(
                SourceEvidenceErrorKind::MalformedBom,
            ));
        }

        let content_version = SourceContentVersion::from_bytes(&raw_content);
        for occurrence in &occurrences {
            validate_occurrence(&id, content_version, source, occurrence)?;
        }

        occurrences.sort_unstable();
        occurrences.dedup();
        for pair in occurrences.windows(2) {
            let previous = &pair[0];
            let current = &pair[1];
            if previous.range == current.range {
                return Err(SourceEvidenceError::new(
                    SourceEvidenceErrorKind::DuplicateConflict,
                ));
            }
            if previous.range.end_byte() > current.range.start_byte() {
                return Err(SourceEvidenceError::new(
                    SourceEvidenceErrorKind::OverlappingOccurrences,
                ));
            }
        }

        Ok(Self {
            id,
            format,
            module_role,
            path,
            raw_content: Arc::from(raw_content),
            content_version,
            occurrences: Arc::from(occurrences),
            completeness,
        })
    }

    /// Returns the structured document identity.
    #[must_use]
    pub const fn id(&self) -> &SourceDocumentId {
        &self.id
    }

    /// Returns the production source format.
    #[must_use]
    pub const fn format(&self) -> SourceFormat {
        self.format
    }

    /// Returns the accepted BSL module role.
    #[must_use]
    pub const fn module_role(&self) -> BslModuleRole {
        self.module_role
    }

    /// Returns the confined Workspace-relative source path.
    #[must_use]
    pub const fn path(&self) -> &ConfinedSourcePath {
        &self.path
    }

    /// Returns the exact retained raw bytes, including BOM and line endings.
    #[must_use]
    pub fn raw_content(&self) -> &[u8] {
        &self.raw_content
    }

    /// Returns the exact deterministic content version.
    #[must_use]
    pub const fn content_version(&self) -> SourceContentVersion {
        self.content_version
    }

    /// Returns occurrences in canonical structured-identity order.
    #[must_use]
    pub fn occurrences(&self) -> &[SourceOccurrence] {
        &self.occurrences
    }

    /// Returns the family-specific completeness proof.
    #[must_use]
    pub const fn completeness(&self) -> SourceEvidenceCompleteness {
        self.completeness
    }
}

/// Canonically ordered complete source evidence for one Configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceEvidenceSet {
    configuration_id: EntityId,
    documents: Arc<[SourceDocument]>,
    total_raw_bytes: usize,
}

impl SourceEvidenceSet {
    /// Validates bounded documents, aggregate bytes, identities, and path aliases.
    ///
    /// Duplicate document identities and paths fail even when the complete
    /// document values are equal. The resulting documents are ordered by their
    /// structured identity independently of producer discovery order.
    ///
    /// # Errors
    ///
    /// Returns a redacted error for an over-bound collection, arithmetic
    /// overflow, incompatible Configuration identity, duplicate identity, or
    /// path alias.
    pub fn new(
        configuration_id: EntityId,
        mut documents: Vec<SourceDocument>,
    ) -> Result<Self, SourceEvidenceError> {
        validate_identity_component(&configuration_id)?;
        if documents.len() > MAX_SOURCE_DOCUMENTS_PER_CONFIGURATION {
            return Err(SourceEvidenceError::bounded(
                documents.len(),
                MAX_SOURCE_DOCUMENTS_PER_CONFIGURATION,
            ));
        }

        let mut total_raw_bytes = 0_usize;
        for document in &documents {
            if document.id().configuration_id() != &configuration_id {
                return Err(SourceEvidenceError::new(
                    SourceEvidenceErrorKind::IncompatibleEvidence,
                ));
            }
            total_raw_bytes = total_raw_bytes
                .checked_add(document.raw_content().len())
                .ok_or_else(|| {
                    SourceEvidenceError::new(SourceEvidenceErrorKind::ArithmeticOverflow)
                })?;
            if total_raw_bytes > MAX_SOURCE_BYTES_PER_CONFIGURATION {
                return Err(SourceEvidenceError::bounded(
                    total_raw_bytes,
                    MAX_SOURCE_BYTES_PER_CONFIGURATION,
                ));
            }
        }

        documents.sort_unstable_by(|left, right| left.id().cmp(right.id()));
        if documents
            .windows(2)
            .any(|pair| pair[0].id() == pair[1].id())
        {
            return Err(SourceEvidenceError::new(
                SourceEvidenceErrorKind::DuplicateConflict,
            ));
        }
        let mut paths = BTreeSet::new();
        if documents
            .iter()
            .any(|document| !paths.insert(document.path().path().as_str()))
        {
            return Err(SourceEvidenceError::new(
                SourceEvidenceErrorKind::DuplicateConflict,
            ));
        }

        Ok(Self {
            configuration_id,
            documents: Arc::from(documents),
            total_raw_bytes,
        })
    }

    /// Returns the canonical Configuration identity.
    #[must_use]
    pub const fn configuration_id(&self) -> &EntityId {
        &self.configuration_id
    }

    /// Returns documents in canonical structured-identity order.
    #[must_use]
    pub fn documents(&self) -> &[SourceDocument] {
        &self.documents
    }

    /// Returns the checked aggregate retained raw byte count.
    #[must_use]
    pub const fn total_raw_bytes(&self) -> usize {
        self.total_raw_bytes
    }
}

fn validate_occurrence(
    document_id: &SourceDocumentId,
    content_version: SourceContentVersion,
    source: &str,
    occurrence: &SourceOccurrence,
) -> Result<(), SourceEvidenceError> {
    if occurrence.document_id() != document_id || occurrence.content_version() != content_version {
        return Err(SourceEvidenceError::new(
            SourceEvidenceErrorKind::IncompatibleEvidence,
        ));
    }
    let range = occurrence.range();
    if range.end_byte() > source.len()
        || !source.is_char_boundary(range.start_byte())
        || !source.is_char_boundary(range.end_byte())
        || source.get(range.start_byte()..range.end_byte()) != Some(occurrence.token())
    {
        return Err(SourceEvidenceError::new(
            SourceEvidenceErrorKind::InvalidOccurrence,
        ));
    }
    Ok(())
}

/// Maximum Configurations selected by one refactoring request.
pub const MAX_REFACTORING_CONFIGURATIONS: usize = 1;
/// Maximum targets selected by one refactoring request.
pub const MAX_REFACTORING_TARGETS: usize = 1;
/// Maximum candidate occurrences admitted by one plan construction.
pub const MAX_REFACTORING_CANDIDATES: usize = 65_536;
/// Maximum operations retained by one complete refactoring plan.
pub const MAX_REFACTORING_OPERATIONS: usize = 65_536;
/// Maximum dependency edges admitted by the first refactoring family.
pub const MAX_REFACTORING_DEPENDENCIES: usize = 0;
/// Maximum public preview entries requested by one product projection.
pub const MAX_REFACTORING_PREVIEW_ENTRIES: usize = 100;
/// Default public preview entry limit.
pub const DEFAULT_REFACTORING_PREVIEW_ENTRIES: usize = 50;
/// Fixed lowercase hexadecimal length of a SHA-256 plan or operation identity.
pub const REFACTORING_IDENTITY_RENDERING_BYTES: usize = 64;

/// Closed first-slice refactoring family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RefactoringFamily {
    /// Rename one top-level BSL Procedure or Function and all accepted direct calls.
    BslCallableRenameV1,
}

impl RefactoringFamily {
    /// Returns the stable family tag used by canonical identities and projections.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::BslCallableRenameV1 => "bsl_callable_rename_v1",
        }
    }
}

/// Closed inclusive admission-bound vocabulary for the plan domain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RefactoringBound {
    /// UTF-8 bytes in one semantic or document identity component.
    IdentityBytes,
    /// UTF-8 bytes in one desired, expected, or replacement identifier.
    IdentifierBytes,
    /// Source documents named by one precondition set.
    DocumentsPerConfiguration,
    /// Candidate operations presented to one plan construction.
    CandidateOccurrences,
    /// Unique operations retained by one complete plan.
    PlannedOperations,
    /// Dependency edges presented to one operation.
    DependencyEdges,
    /// Entries requested by a public preview projection.
    PreviewEntries,
}

impl RefactoringBound {
    /// Returns the inclusive maximum for this bound.
    #[must_use]
    pub const fn maximum(self) -> usize {
        match self {
            Self::IdentityBytes => MAX_SOURCE_IDENTITY_BYTES,
            Self::IdentifierBytes => MAX_SOURCE_IDENTIFIER_BYTES,
            Self::DocumentsPerConfiguration => MAX_SOURCE_DOCUMENTS_PER_CONFIGURATION,
            Self::CandidateOccurrences | Self::PlannedOperations => MAX_REFACTORING_OPERATIONS,
            Self::DependencyEdges => MAX_REFACTORING_DEPENDENCIES,
            Self::PreviewEntries => MAX_REFACTORING_PREVIEW_ENTRIES,
        }
    }
}

/// Closed refactoring plan-domain failure kind in deterministic precedence order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RefactoringErrorKind {
    /// Cooperative cancellation was observed by an evaluator.
    Cancelled,
    /// A request has an invalid shape or unsupported family value.
    InvalidRequest,
    /// An inclusive scalar or collection bound was exceeded.
    BoundExceeded,
    /// The requested publication is not the selected immutable publication.
    PublicationMismatch,
    /// The selected Configuration does not exist.
    ConfigurationNotFound,
    /// The selected target does not exist.
    TargetNotFound,
    /// The target kind is outside the first family.
    UnsupportedTarget,
    /// The target does not have exactly one accepted owner Module.
    AmbiguousOwner,
    /// The selected source format or module role is unsupported.
    UnsupportedSourceFormat,
    /// Required immutable source evidence is absent.
    SourceEvidenceMissing,
    /// Source evidence is not complete for the selected family.
    SourceEvidenceIncomplete,
    /// Structured evidence values disagree or forbidden dependencies were supplied.
    IncompatibleEvidence,
    /// An operation names a stale content version.
    StaleSourceVersion,
    /// A declaration or operation occurrence is invalid.
    InvalidOccurrence,
    /// A target-related occurrence cannot be resolved uniquely.
    AmbiguousOccurrence,
    /// The desired BSL identifier is invalid or reserved.
    InvalidDesiredName,
    /// The desired name is BSL-equivalent to the current name.
    NoChange,
    /// Another callable in the owner Module has the desired name.
    NameCollision,
    /// Equal identities carry unequal complete structured values.
    IdentityCollision,
    /// Unequal operations occupy the same exact source range.
    DuplicateConflict,
    /// Unequal operation ranges intersect in one document.
    OverlappingOperations,
    /// Checked counter or canonical-encoding arithmetic overflowed.
    ArithmeticOverflow,
}

impl RefactoringErrorKind {
    const fn message(self) -> &'static str {
        match self {
            Self::Cancelled => "refactoring planning was cancelled",
            Self::InvalidRequest => "refactoring request is invalid",
            Self::BoundExceeded => "refactoring input exceeds an accepted bound",
            Self::PublicationMismatch => "refactoring publication does not match",
            Self::ConfigurationNotFound => "refactoring Configuration was not found",
            Self::TargetNotFound => "refactoring target was not found",
            Self::UnsupportedTarget => "refactoring target is unsupported",
            Self::AmbiguousOwner => "refactoring target owner is ambiguous",
            Self::UnsupportedSourceFormat => "refactoring source format is unsupported",
            Self::SourceEvidenceMissing => "refactoring source evidence is missing",
            Self::SourceEvidenceIncomplete => "refactoring source evidence is incomplete",
            Self::IncompatibleEvidence => "refactoring evidence is incompatible",
            Self::StaleSourceVersion => "refactoring source version is stale",
            Self::InvalidOccurrence => "refactoring occurrence is invalid",
            Self::AmbiguousOccurrence => "refactoring occurrence is ambiguous",
            Self::InvalidDesiredName => "refactoring desired name is invalid",
            Self::NoChange => "refactoring request would make no change",
            Self::NameCollision => "refactoring desired name collides with another callable",
            Self::IdentityCollision => "refactoring identity collides with unequal evidence",
            Self::DuplicateConflict => "refactoring operations conflict at one range",
            Self::OverlappingOperations => "refactoring operations overlap",
            Self::ArithmeticOverflow => "refactoring checked arithmetic overflowed",
        }
    }
}

/// Redacted refactoring domain error with optional non-sensitive bound counts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RefactoringError {
    kind: RefactoringErrorKind,
    bound: Option<RefactoringBound>,
    actual: Option<usize>,
    maximum: Option<usize>,
}

impl RefactoringError {
    const fn closed(kind: RefactoringErrorKind) -> Self {
        Self {
            kind,
            bound: None,
            actual: None,
            maximum: None,
        }
    }

    const fn bounded(bound: RefactoringBound, actual: usize) -> Self {
        Self {
            kind: RefactoringErrorKind::BoundExceeded,
            bound: Some(bound),
            actual: Some(actual),
            maximum: Some(bound.maximum()),
        }
    }

    /// Returns the closed failure kind.
    #[must_use]
    pub const fn kind(self) -> RefactoringErrorKind {
        self.kind
    }

    /// Returns the violated bound category, when applicable.
    #[must_use]
    pub const fn bound(self) -> Option<RefactoringBound> {
        self.bound
    }

    /// Returns the rejected non-sensitive count, when applicable.
    #[must_use]
    pub const fn actual(self) -> Option<usize> {
        self.actual
    }

    /// Returns the accepted inclusive maximum, when applicable.
    #[must_use]
    pub const fn maximum(self) -> Option<usize> {
        self.maximum
    }
}

impl Display for RefactoringError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.kind.message())
    }
}

impl std::error::Error for RefactoringError {}

fn validate_plan_identity(value: &EntityId) -> Result<(), RefactoringError> {
    if value.as_str().len() > MAX_SOURCE_IDENTITY_BYTES {
        return Err(RefactoringError::bounded(
            RefactoringBound::IdentityBytes,
            value.as_str().len(),
        ));
    }
    Ok(())
}

fn validate_desired_name(value: &str) -> Result<(), RefactoringError> {
    if value.len() > MAX_SOURCE_IDENTIFIER_BYTES {
        return Err(RefactoringError::bounded(
            RefactoringBound::IdentifierBytes,
            value.len(),
        ));
    }
    let mut scalars = value.chars();
    let Some(first) = scalars.next() else {
        return Err(RefactoringError::closed(
            RefactoringErrorKind::InvalidDesiredName,
        ));
    };
    if !(first == '_' || first.is_alphabetic())
        || !scalars.all(|scalar| scalar == '_' || scalar.is_alphanumeric())
        || RESERVED_BSL_NAMES.contains(&bsl_name_key(value).as_str())
    {
        return Err(RefactoringError::closed(
            RefactoringErrorKind::InvalidDesiredName,
        ));
    }
    Ok(())
}

const RESERVED_BSL_NAMES: &[&str] = &[
    "if",
    "если",
    "elsif",
    "иначеесли",
    "while",
    "пока",
    "for",
    "для",
    "foreach",
    "длякаждого",
    "return",
    "возврат",
    "procedure",
    "процедура",
    "function",
    "функция",
    "endprocedure",
    "конецпроцедуры",
    "endfunction",
    "конецфункции",
    "export",
    "экспорт",
];

/// Immutable validated request for one supported refactoring family and target.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RefactoringRequest {
    family: RefactoringFamily,
    expected_publication_id: WorkspacePublicationId,
    configuration_id: EntityId,
    target_node_id: EntityId,
    desired_name: Box<str>,
}

impl RefactoringRequest {
    /// Creates one bounded request without paths, source text, or caller operations.
    ///
    /// # Errors
    ///
    /// Returns a redacted error for an over-bound identity or an invalid,
    /// reserved, empty, or over-bound desired BSL name.
    pub fn new(
        family: RefactoringFamily,
        expected_publication_id: WorkspacePublicationId,
        configuration_id: EntityId,
        target_node_id: EntityId,
        desired_name: impl Into<String>,
    ) -> Result<Self, RefactoringError> {
        validate_plan_identity(&configuration_id)?;
        validate_plan_identity(&target_node_id)?;
        let desired_name = desired_name.into();
        validate_desired_name(&desired_name)?;
        Ok(Self {
            family,
            expected_publication_id,
            configuration_id,
            target_node_id,
            desired_name: desired_name.into_boxed_str(),
        })
    }

    /// Returns the closed first-slice family.
    #[must_use]
    pub const fn family(&self) -> RefactoringFamily {
        self.family
    }

    /// Returns the expected immutable Workspace publication.
    #[must_use]
    pub const fn expected_publication_id(&self) -> WorkspacePublicationId {
        self.expected_publication_id
    }

    /// Returns the selected Graph Configuration identity.
    #[must_use]
    pub const fn configuration_id(&self) -> &EntityId {
        &self.configuration_id
    }

    /// Returns the exact pre-rename Graph target identity.
    #[must_use]
    pub const fn target_node_id(&self) -> &EntityId {
        &self.target_node_id
    }

    /// Returns the validated desired identifier separately from target identity.
    #[must_use]
    pub fn desired_name(&self) -> &str {
        &self.desired_name
    }
}

/// Immutable validated target identity and its exact declaration evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefactoringTarget {
    configuration_id: EntityId,
    target_node_id: EntityId,
    target_kind: NodeKind,
    owner_module_id: EntityId,
    declaration: SourceOccurrence,
    expected_post_rename_node_id: EntityId,
}

impl RefactoringTarget {
    /// Creates a target bound to one Procedure or Function declaration.
    ///
    /// # Errors
    ///
    /// Returns a redacted error unless the declaration is unique, belongs to
    /// the owner Module and Configuration, and maps to the pre-rename target.
    pub fn new(
        configuration_id: EntityId,
        target_node_id: EntityId,
        target_kind: NodeKind,
        owner_module_id: EntityId,
        declaration: SourceOccurrence,
        desired_name: &str,
    ) -> Result<Self, RefactoringError> {
        for identity in [&configuration_id, &target_node_id, &owner_module_id] {
            validate_plan_identity(identity)?;
        }
        validate_desired_name(desired_name)?;
        if !matches!(target_kind, NodeKind::Procedure | NodeKind::Function) {
            return Err(RefactoringError::closed(
                RefactoringErrorKind::UnsupportedTarget,
            ));
        }
        if declaration.kind() != SourceOccurrenceKind::Declaration
            || declaration.resolution() != SourceOccurrenceResolution::Unique
            || declaration.mapped_target_id() != Some(&target_node_id)
            || declaration.document_id().configuration_id() != &configuration_id
            || declaration.document_id().module_id() != &owner_module_id
        {
            return Err(RefactoringError::closed(
                RefactoringErrorKind::InvalidOccurrence,
            ));
        }
        let symbol_kind = match target_kind {
            NodeKind::Procedure => BslSymbolKind::Procedure,
            NodeKind::Function => BslSymbolKind::Function,
            _ => unreachable!("unsupported kinds were rejected above"),
        };
        let expected_post_rename_node_id =
            bsl_callable_id(&owner_module_id, symbol_kind, desired_name)
                .map_err(|_| RefactoringError::closed(RefactoringErrorKind::IdentityCollision))?;
        validate_plan_identity(&expected_post_rename_node_id)?;
        Ok(Self {
            configuration_id,
            target_node_id,
            target_kind,
            owner_module_id,
            declaration,
            expected_post_rename_node_id,
        })
    }

    /// Returns the selected Configuration identity.
    #[must_use]
    pub const fn configuration_id(&self) -> &EntityId {
        &self.configuration_id
    }

    /// Returns the stable pre-rename target identity.
    #[must_use]
    pub const fn target_node_id(&self) -> &EntityId {
        &self.target_node_id
    }

    /// Returns the Graph-owned target kind.
    #[must_use]
    pub const fn target_kind(&self) -> NodeKind {
        self.target_kind
    }

    /// Returns the single owning Module identity.
    #[must_use]
    pub const fn owner_module_id(&self) -> &EntityId {
        &self.owner_module_id
    }

    /// Returns the exact uniquely mapped declaration occurrence.
    #[must_use]
    pub const fn declaration(&self) -> &SourceOccurrence {
        &self.declaration
    }

    /// Returns the separately derived expected post-rename identity.
    #[must_use]
    pub const fn expected_post_rename_node_id(&self) -> &EntityId {
        &self.expected_post_rename_node_id
    }
}

/// One immutable document/version pair required by every operation in a plan.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RefactoringSourcePrecondition {
    document_id: SourceDocumentId,
    content_version: SourceContentVersion,
}

impl RefactoringSourcePrecondition {
    /// Creates one exact immutable source precondition.
    #[must_use]
    pub const fn new(document_id: SourceDocumentId, content_version: SourceContentVersion) -> Self {
        Self {
            document_id,
            content_version,
        }
    }

    /// Returns the structured source-document identity.
    #[must_use]
    pub const fn document_id(&self) -> &SourceDocumentId {
        &self.document_id
    }

    /// Returns the exact retained source content version.
    #[must_use]
    pub const fn content_version(&self) -> SourceContentVersion {
        self.content_version
    }
}

/// Canonical immutable publication, target, owner, and source preconditions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefactoringPreconditionSet {
    publication_id: WorkspacePublicationId,
    configuration_id: EntityId,
    target_node_id: EntityId,
    target_kind: NodeKind,
    owner_module_id: EntityId,
    documents: Arc<[RefactoringSourcePrecondition]>,
}

impl RefactoringPreconditionSet {
    /// Validates and canonically orders one complete source precondition set.
    ///
    /// Exact duplicate pairs collapse. Two versions for one document fail
    /// closed independently of input order.
    ///
    /// # Errors
    ///
    /// Returns a redacted error for empty, over-bound, incompatible, or
    /// conflicting source preconditions.
    pub fn new(
        publication_id: WorkspacePublicationId,
        configuration_id: EntityId,
        target_node_id: EntityId,
        target_kind: NodeKind,
        owner_module_id: EntityId,
        documents: Vec<RefactoringSourcePrecondition>,
    ) -> Result<Self, RefactoringError> {
        for identity in [&configuration_id, &target_node_id, &owner_module_id] {
            validate_plan_identity(identity)?;
        }
        if !matches!(target_kind, NodeKind::Procedure | NodeKind::Function) {
            return Err(RefactoringError::closed(
                RefactoringErrorKind::UnsupportedTarget,
            ));
        }
        if documents.is_empty() {
            return Err(RefactoringError::closed(
                RefactoringErrorKind::SourceEvidenceMissing,
            ));
        }
        if documents.len() > MAX_SOURCE_DOCUMENTS_PER_CONFIGURATION {
            return Err(RefactoringError::bounded(
                RefactoringBound::DocumentsPerConfiguration,
                documents.len(),
            ));
        }
        let mut canonical = BTreeMap::new();
        for document in documents {
            if document.document_id().configuration_id() != &configuration_id {
                return Err(RefactoringError::closed(
                    RefactoringErrorKind::IncompatibleEvidence,
                ));
            }
            match canonical.entry(document.document_id().clone()) {
                std::collections::btree_map::Entry::Vacant(entry) => {
                    entry.insert(document);
                }
                std::collections::btree_map::Entry::Occupied(entry) if entry.get() == &document => {
                }
                std::collections::btree_map::Entry::Occupied(_) => {
                    return Err(RefactoringError::closed(
                        RefactoringErrorKind::IncompatibleEvidence,
                    ));
                }
            }
        }
        Ok(Self {
            publication_id,
            configuration_id,
            target_node_id,
            target_kind,
            owner_module_id,
            documents: Arc::from(canonical.into_values().collect::<Vec<_>>()),
        })
    }

    /// Returns the process-local Workspace publication identity.
    #[must_use]
    pub const fn publication_id(&self) -> WorkspacePublicationId {
        self.publication_id
    }

    /// Returns the selected Configuration identity.
    #[must_use]
    pub const fn configuration_id(&self) -> &EntityId {
        &self.configuration_id
    }

    /// Returns the exact pre-rename target identity.
    #[must_use]
    pub const fn target_node_id(&self) -> &EntityId {
        &self.target_node_id
    }

    /// Returns the Graph-owned target kind.
    #[must_use]
    pub const fn target_kind(&self) -> NodeKind {
        self.target_kind
    }

    /// Returns the single owner Module identity.
    #[must_use]
    pub const fn owner_module_id(&self) -> &EntityId {
        &self.owner_module_id
    }

    /// Returns document/version pairs in canonical document-identity order.
    #[must_use]
    pub fn documents(&self) -> &[RefactoringSourcePrecondition] {
        &self.documents
    }
}

/// Stable SHA-256 identity of one complete structured operation.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OperationId(Box<str>);

impl OperationId {
    /// Returns the 64-byte lowercase hexadecimal rendering.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for OperationId {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Closed operation vocabulary for `bsl_callable_rename_v1`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RefactoringOperationKind {
    /// Replace the selected declaration identifier.
    ReplaceDeclarationIdentifier,
    /// Replace one accepted local or qualified direct-call identifier.
    ReplaceDirectCallIdentifier,
}

impl RefactoringOperationKind {
    /// Returns the stable operation tag.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ReplaceDeclarationIdentifier => "replace_declaration_identifier",
            Self::ReplaceDirectCallIdentifier => "replace_direct_call_identifier",
        }
    }
}

/// One immutable bounded identifier replacement operation.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RefactoringOperation {
    id: OperationId,
    kind: RefactoringOperationKind,
    occurrence_kind: SourceOccurrenceKind,
    document_id: SourceDocumentId,
    content_version: SourceContentVersion,
    range: SourceByteRange,
    expected: Box<str>,
    replacement: Box<str>,
}

impl RefactoringOperation {
    /// Creates one operation and its canonical identity.
    ///
    /// The first family accepts no dependency edge. Expected and replacement
    /// identifiers are non-empty and bounded before retention.
    ///
    /// # Errors
    ///
    /// Returns a redacted error for an identifier bound, empty token, or any
    /// supplied dependency.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        kind: RefactoringOperationKind,
        occurrence_kind: SourceOccurrenceKind,
        document_id: SourceDocumentId,
        content_version: SourceContentVersion,
        range: SourceByteRange,
        expected: impl Into<String>,
        replacement: impl Into<String>,
        dependencies: &[OperationId],
    ) -> Result<Self, RefactoringError> {
        if !matches!(
            (kind, occurrence_kind),
            (
                RefactoringOperationKind::ReplaceDeclarationIdentifier,
                SourceOccurrenceKind::Declaration
            ) | (
                RefactoringOperationKind::ReplaceDirectCallIdentifier,
                SourceOccurrenceKind::LocalCall | SourceOccurrenceKind::QualifiedCall
            )
        ) {
            return Err(RefactoringError::closed(
                RefactoringErrorKind::IncompatibleEvidence,
            ));
        }
        if !dependencies.is_empty() {
            return Err(RefactoringError::bounded(
                RefactoringBound::DependencyEdges,
                dependencies.len(),
            ));
        }
        let expected = expected.into();
        let replacement = replacement.into();
        for value in [&expected, &replacement] {
            if value.is_empty() {
                return Err(RefactoringError::closed(
                    RefactoringErrorKind::InvalidOccurrence,
                ));
            }
            if value.len() > MAX_SOURCE_IDENTIFIER_BYTES {
                return Err(RefactoringError::bounded(
                    RefactoringBound::IdentifierBytes,
                    value.len(),
                ));
            }
        }
        validate_desired_name(&replacement)?;
        let mut encoding = Vec::new();
        encode_string(&mut encoding, kind.as_str())?;
        encode_document_id(&mut encoding, &document_id)?;
        encode_content_version(&mut encoding, content_version)?;
        encode_usize(&mut encoding, range.start_byte())?;
        encode_usize(&mut encoding, range.end_byte())?;
        encode_string(&mut encoding, &expected)?;
        encode_string(&mut encoding, &replacement)?;
        let id = OperationId(sha256_hex(&encoding).into_boxed_str());
        Ok(Self {
            id,
            kind,
            occurrence_kind,
            document_id,
            content_version,
            range,
            expected: expected.into_boxed_str(),
            replacement: replacement.into_boxed_str(),
        })
    }

    /// Returns the stable complete-structure operation identity.
    #[must_use]
    pub const fn id(&self) -> &OperationId {
        &self.id
    }

    /// Returns the closed operation kind.
    #[must_use]
    pub const fn kind(&self) -> RefactoringOperationKind {
        self.kind
    }

    /// Returns the declaration, local-call, or qualified-call source category.
    #[must_use]
    pub const fn occurrence_kind(&self) -> SourceOccurrenceKind {
        self.occurrence_kind
    }

    /// Returns the containing document identity.
    #[must_use]
    pub const fn document_id(&self) -> &SourceDocumentId {
        &self.document_id
    }

    /// Returns the exact retained content version required by this operation.
    #[must_use]
    pub const fn content_version(&self) -> SourceContentVersion {
        self.content_version
    }

    /// Returns the exact raw-byte identifier range.
    #[must_use]
    pub const fn range(&self) -> SourceByteRange {
        self.range
    }

    /// Returns the exact expected captured identifier token.
    #[must_use]
    pub fn expected(&self) -> &str {
        &self.expected
    }

    /// Returns the desired replacement identifier.
    #[must_use]
    pub fn replacement(&self) -> &str {
        &self.replacement
    }

    /// Returns the only accepted dependency set, which is empty in v1.
    #[must_use]
    pub const fn dependencies(&self) -> &[OperationId] {
        &[]
    }
}

impl PartialOrd for RefactoringOperation {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RefactoringOperation {
    fn cmp(&self, other: &Self) -> Ordering {
        self.document_id
            .cmp(&other.document_id)
            .then_with(|| other.range.start_byte().cmp(&self.range.start_byte()))
            .then_with(|| other.range.end_byte().cmp(&self.range.end_byte()))
            .then_with(|| self.kind.cmp(&other.kind))
            .then_with(|| self.id.cmp(&other.id))
            .then_with(|| self.occurrence_kind.cmp(&other.occurrence_kind))
    }
}

/// The only successful completeness value for a first-family plan or preview.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RefactoringCompleteness {
    /// Every admitted target-related operation is present and none was omitted.
    Complete,
}

impl RefactoringCompleteness {
    /// Returns the stable public completeness rendering.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Complete => "complete",
        }
    }
}

/// Reconciled checked counters for one complete refactoring plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RefactoringPlanSummary {
    requested_targets: usize,
    planned_targets: usize,
    conflicted_targets: usize,
    rejected_targets: usize,
    documents: usize,
    candidate_occurrences: usize,
    exact_duplicates_collapsed: usize,
    declaration_operations: usize,
    local_call_operations: usize,
    qualified_call_operations: usize,
    planned_operations: usize,
    omitted_operations: usize,
    returned_operations: usize,
}

macro_rules! summary_accessors {
    ($($(#[$meta:meta])* $name:ident),+ $(,)?) => {
        $(
            $(#[$meta])*
            #[must_use]
            pub const fn $name(self) -> usize {
                self.$name
            }
        )+
    };
}

impl RefactoringPlanSummary {
    summary_accessors!(
        /// Returns requested target count, fixed at one.
        requested_targets,
        /// Returns planned target count, fixed at one.
        planned_targets,
        /// Returns conflicted target count, fixed at zero on success.
        conflicted_targets,
        /// Returns rejected target count, fixed at zero on success.
        rejected_targets,
        /// Returns distinct documents used by operations.
        documents,
        /// Returns candidate operations before exact duplicate collapse.
        candidate_occurrences,
        /// Returns exact duplicate operations collapsed during normalization.
        exact_duplicates_collapsed,
        /// Returns retained declaration operations, fixed at one.
        declaration_operations,
        /// Returns retained local direct-call operations.
        local_call_operations,
        /// Returns retained qualified direct-call operations.
        qualified_call_operations,
        /// Returns total retained operations.
        planned_operations,
        /// Returns omitted internal operations, fixed at zero.
        omitted_operations,
        /// Returns complete internal operations, equal to planned operations.
        returned_operations,
    );
}

/// Stable SHA-256 identity of one complete structured plan.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PlanId(Box<str>);

impl PlanId {
    /// Returns the 64-byte lowercase hexadecimal rendering.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for PlanId {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Complete immutable source-independent first-family refactoring plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefactoringPlan {
    id: PlanId,
    request: RefactoringRequest,
    target: RefactoringTarget,
    preconditions: RefactoringPreconditionSet,
    operations: Arc<[RefactoringOperation]>,
    summary: RefactoringPlanSummary,
    completeness: RefactoringCompleteness,
}

impl RefactoringPlan {
    /// Validates, normalizes, reconciles, and identifies one complete plan.
    ///
    /// Exact duplicates collapse. Version conflicts, same-range conflicts,
    /// overlaps, or missing declaration operations reject the whole construction.
    ///
    /// # Errors
    ///
    /// Returns one closed redacted failure and no partial plan.
    pub fn new(
        request: RefactoringRequest,
        target: RefactoringTarget,
        preconditions: RefactoringPreconditionSet,
        operations: Vec<RefactoringOperation>,
    ) -> Result<Self, RefactoringError> {
        validate_plan_relationships(&request, &target, &preconditions)?;
        if operations.is_empty() {
            return Err(RefactoringError::closed(
                RefactoringErrorKind::SourceEvidenceMissing,
            ));
        }
        if operations.len() > MAX_REFACTORING_CANDIDATES {
            return Err(RefactoringError::bounded(
                RefactoringBound::CandidateOccurrences,
                operations.len(),
            ));
        }
        let candidate_occurrences = operations.len();
        let mut by_id = BTreeMap::<OperationId, RefactoringOperation>::new();
        for operation in operations {
            validate_operation_relationship(&request, &target, &preconditions, &operation)?;
            match by_id.entry(operation.id().clone()) {
                std::collections::btree_map::Entry::Vacant(entry) => {
                    entry.insert(operation);
                }
                std::collections::btree_map::Entry::Occupied(entry)
                    if entry.get() == &operation => {}
                std::collections::btree_map::Entry::Occupied(_) => {
                    return Err(RefactoringError::closed(
                        RefactoringErrorKind::IdentityCollision,
                    ));
                }
            }
        }
        let mut operations = by_id.into_values().collect::<Vec<_>>();
        if operations.len() > MAX_REFACTORING_OPERATIONS {
            return Err(RefactoringError::bounded(
                RefactoringBound::PlannedOperations,
                operations.len(),
            ));
        }
        operations.sort_unstable();
        validate_operation_conflicts(&operations)?;
        let summary = summarize_operations(candidate_occurrences, &operations, &preconditions)?;
        let id = plan_id(&request, &target, &preconditions, &operations)?;
        Ok(Self {
            id,
            request,
            target,
            preconditions,
            operations: Arc::from(operations),
            summary,
            completeness: RefactoringCompleteness::Complete,
        })
    }

    /// Returns the stable complete-structure plan identity.
    #[must_use]
    pub const fn id(&self) -> &PlanId {
        &self.id
    }

    /// Returns the validated request.
    #[must_use]
    pub const fn request(&self) -> &RefactoringRequest {
        &self.request
    }

    /// Returns the validated target and exact declaration evidence.
    #[must_use]
    pub const fn target(&self) -> &RefactoringTarget {
        &self.target
    }

    /// Returns all immutable source preconditions.
    #[must_use]
    pub const fn preconditions(&self) -> &RefactoringPreconditionSet {
        &self.preconditions
    }

    /// Returns operations in canonical future safe-application order.
    #[must_use]
    pub fn operations(&self) -> &[RefactoringOperation] {
        &self.operations
    }

    /// Returns reconciled checked counters.
    #[must_use]
    pub const fn summary(&self) -> RefactoringPlanSummary {
        self.summary
    }

    /// Returns complete plan status.
    #[must_use]
    pub const fn completeness(&self) -> RefactoringCompleteness {
        self.completeness
    }
}

/// One structured no-snippet preview entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefactoringPreviewEntry {
    operation_id: OperationId,
    kind: RefactoringOperationKind,
    path: ConfinedSourcePath,
    range: SourceByteRange,
    position: SourceSpan,
    replacement: Box<str>,
}

impl RefactoringPreviewEntry {
    /// Projects one operation with a confined path and derived one-based span.
    ///
    /// # Errors
    ///
    /// Returns a redacted error when the derived span is empty.
    pub fn new(
        operation: &RefactoringOperation,
        path: ConfinedSourcePath,
        position: SourceSpan,
    ) -> Result<Self, RefactoringError> {
        if position.start() == position.end() {
            return Err(RefactoringError::closed(
                RefactoringErrorKind::InvalidOccurrence,
            ));
        }
        Ok(Self {
            operation_id: operation.id().clone(),
            kind: operation.kind(),
            path,
            range: operation.range(),
            position,
            replacement: operation.replacement().into(),
        })
    }

    /// Returns the projected operation identity.
    #[must_use]
    pub const fn operation_id(&self) -> &OperationId {
        &self.operation_id
    }

    /// Returns the projected operation kind.
    #[must_use]
    pub const fn kind(&self) -> RefactoringOperationKind {
        self.kind
    }

    /// Returns the confined Workspace-relative path.
    #[must_use]
    pub const fn path(&self) -> &ConfinedSourcePath {
        &self.path
    }

    /// Returns the exact raw-byte range without source content.
    #[must_use]
    pub const fn range(&self) -> SourceByteRange {
        self.range
    }

    /// Returns the derived one-based exclusive-end line/column span.
    #[must_use]
    pub const fn position(&self) -> SourceSpan {
        self.position
    }

    /// Returns the bounded replacement identifier.
    #[must_use]
    pub fn replacement(&self) -> &str {
        &self.replacement
    }
}

/// Complete deterministic read-only structured preview of one immutable plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefactoringPreview {
    plan_id: PlanId,
    entries: Arc<[RefactoringPreviewEntry]>,
    completeness: RefactoringCompleteness,
}

impl RefactoringPreview {
    /// Validates a complete entry-for-operation projection in plan order.
    ///
    /// # Errors
    ///
    /// Returns a redacted error when any operation is missing, reordered, or
    /// projected with incompatible kind, range, or replacement evidence.
    pub fn new(
        plan: &RefactoringPlan,
        entries: Vec<RefactoringPreviewEntry>,
    ) -> Result<Self, RefactoringError> {
        if entries.len() != plan.operations().len()
            || entries
                .iter()
                .zip(plan.operations())
                .any(|(entry, operation)| {
                    entry.operation_id() != operation.id()
                        || entry.kind() != operation.kind()
                        || entry.range() != operation.range()
                        || entry.replacement() != operation.replacement()
                })
        {
            return Err(RefactoringError::closed(
                RefactoringErrorKind::IncompatibleEvidence,
            ));
        }
        Ok(Self {
            plan_id: plan.id().clone(),
            entries: Arc::from(entries),
            completeness: RefactoringCompleteness::Complete,
        })
    }

    /// Returns the identity of the complete plan being projected.
    #[must_use]
    pub const fn plan_id(&self) -> &PlanId {
        &self.plan_id
    }

    /// Returns all structured entries in canonical plan order.
    #[must_use]
    pub fn entries(&self) -> &[RefactoringPreviewEntry] {
        &self.entries
    }

    /// Returns complete preview status.
    #[must_use]
    pub const fn completeness(&self) -> RefactoringCompleteness {
        self.completeness
    }
}

/// Borrowed immutable Graph and source evidence for one Configuration publication.
#[derive(Debug, Clone, Copy)]
pub struct RefactoringPlannerInput<'evidence> {
    publication_id: WorkspacePublicationId,
    configuration_id: &'evidence EntityId,
    graph: &'evidence SemanticGraph,
    source_evidence: &'evidence SourceEvidenceSet,
}

impl<'evidence> RefactoringPlannerInput<'evidence> {
    /// Creates a borrowed planner input without reading or cloning source content.
    #[must_use]
    pub const fn new(
        publication_id: WorkspacePublicationId,
        configuration_id: &'evidence EntityId,
        graph: &'evidence SemanticGraph,
        source_evidence: &'evidence SourceEvidenceSet,
    ) -> Self {
        Self {
            publication_id,
            configuration_id,
            graph,
            source_evidence,
        }
    }

    /// Returns the immutable Workspace publication identity.
    #[must_use]
    pub const fn publication_id(self) -> WorkspacePublicationId {
        self.publication_id
    }

    /// Returns the selected canonical Configuration identity.
    #[must_use]
    pub const fn configuration_id(self) -> &'evidence EntityId {
        self.configuration_id
    }

    /// Returns the complete semantic Graph snapshot.
    #[must_use]
    pub const fn graph(self) -> &'evidence SemanticGraph {
        self.graph
    }

    /// Returns the complete retained source evidence.
    #[must_use]
    pub const fn source_evidence(self) -> &'evidence SourceEvidenceSet {
        self.source_evidence
    }
}

/// Minimal cooperative cancellation observation boundary for planning.
pub trait RefactoringCancellationSignal: Send + Sync {
    /// Returns whether planning cancellation was requested.
    fn is_cancelled(&self) -> bool;
}

/// Cancellation signal that never requests cancellation.
#[derive(Debug, Default, Clone, Copy)]
pub struct NeverCancelledRefactoring;

impl RefactoringCancellationSignal for NeverCancelledRefactoring {
    fn is_cancelled(&self) -> bool {
        false
    }
}

/// Atomic complete plan and deterministic read-only preview.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefactoringEvaluation {
    plan: RefactoringPlan,
    preview: RefactoringPreview,
}

impl RefactoringEvaluation {
    /// Returns the complete immutable plan.
    #[must_use]
    pub const fn plan(&self) -> &RefactoringPlan {
        &self.plan
    }

    /// Returns the complete deterministic read-only preview.
    #[must_use]
    pub const fn preview(&self) -> &RefactoringPreview {
        &self.preview
    }

    /// Splits the atomic evaluation into its immutable plan and preview values.
    #[must_use]
    pub fn into_parts(self) -> (RefactoringPlan, RefactoringPreview) {
        (self.plan, self.preview)
    }
}

/// Stateless Graph-backed evaluator for the accepted first refactoring family.
#[derive(Debug, Default, Clone, Copy)]
pub struct RefactoringPlanner;

impl RefactoringPlanner {
    /// Validates one immutable publication and builds a complete plan and preview.
    ///
    /// The evaluator uses only the supplied Graph query API and retained source
    /// evidence. Any failure or cancellation returns no partial plan or preview.
    ///
    /// # Errors
    ///
    /// Returns one closed redacted failure for invalid, missing, ambiguous,
    /// stale, conflicting, incomplete, cancelled, or over-bound evidence.
    pub fn evaluate(
        &self,
        input: RefactoringPlannerInput<'_>,
        request: &RefactoringRequest,
        cancellation: &dyn RefactoringCancellationSignal,
    ) -> Result<RefactoringEvaluation, RefactoringError> {
        validate_planner_request(input, request, cancellation)?;
        let query = SemanticGraphQuery::new(input.graph());
        let (target_node, owner_module) = resolve_planner_target(
            &query,
            input.configuration_id(),
            request.target_node_id(),
            cancellation,
        )?;
        let admitted = admit_planner_source_evidence(input, target_node, cancellation)?;
        let target = validate_planner_collisions(
            input.configuration_id(),
            request,
            &query,
            target_node,
            owner_module,
            admitted.declaration,
        )?;

        observe_refactoring_cancellation(cancellation)?;
        if admitted.candidate_count > MAX_REFACTORING_CANDIDATES {
            return Err(RefactoringError::bounded(
                RefactoringBound::CandidateOccurrences,
                admitted.candidate_count,
            ));
        }
        let plan =
            build_refactoring_plan(input, request, target, admitted.occurrences, cancellation)?;

        observe_refactoring_cancellation(cancellation)?;
        let preview = build_refactoring_preview(&plan, input.source_evidence(), cancellation)?;
        observe_refactoring_cancellation(cancellation)?;
        Ok(RefactoringEvaluation { plan, preview })
    }
}

fn validate_planner_request(
    input: RefactoringPlannerInput<'_>,
    request: &RefactoringRequest,
    cancellation: &dyn RefactoringCancellationSignal,
) -> Result<(), RefactoringError> {
    observe_refactoring_cancellation(cancellation)?;
    validate_plan_identity(input.configuration_id())?;
    validate_plan_identity(request.configuration_id())?;
    validate_plan_identity(request.target_node_id())?;
    validate_desired_name(request.desired_name())?;
    observe_refactoring_cancellation(cancellation)?;
    if request.expected_publication_id() != input.publication_id() {
        return Err(RefactoringError::closed(
            RefactoringErrorKind::PublicationMismatch,
        ));
    }
    observe_refactoring_cancellation(cancellation)?;
    if request.configuration_id() != input.configuration_id() {
        return Err(RefactoringError::closed(
            RefactoringErrorKind::ConfigurationNotFound,
        ));
    }
    Ok(())
}

fn resolve_planner_target<'graph>(
    query: &SemanticGraphQuery<'graph>,
    configuration_id: &EntityId,
    target_id: &EntityId,
    cancellation: &dyn RefactoringCancellationSignal,
) -> Result<(&'graph GraphNode, &'graph GraphNode), RefactoringError> {
    let configuration = query
        .node_by_entity_id(configuration_id)
        .filter(|node| {
            matches!(node.kind(), NodeKind::Metadata(kind) if kind.as_str() == "configuration")
        })
        .ok_or_else(|| RefactoringError::closed(RefactoringErrorKind::ConfigurationNotFound))?;
    let target = query
        .node_by_entity_id(target_id)
        .ok_or_else(|| RefactoringError::closed(RefactoringErrorKind::TargetNotFound))?;
    if !matches!(target.kind(), NodeKind::Procedure | NodeKind::Function) {
        return Err(RefactoringError::closed(
            RefactoringErrorKind::UnsupportedTarget,
        ));
    }
    validate_plan_identity(target.id())?;
    let owners = query.owners(&NodeId::new(target.id().as_str()));
    if owners.len() != 1 || owners[0].kind() != NodeKind::Module {
        return Err(RefactoringError::closed(
            RefactoringErrorKind::AmbiguousOwner,
        ));
    }
    let module = owners[0];
    validate_plan_identity(module.id())?;
    if !has_unique_configuration_owner_chain(query, module.id(), configuration.id(), cancellation)?
    {
        return Err(RefactoringError::closed(
            RefactoringErrorKind::AmbiguousOwner,
        ));
    }
    Ok((target, module))
}

struct AdmittedPlannerSourceEvidence {
    candidate_count: usize,
    declaration: SourceOccurrence,
    occurrences: Vec<SourceOccurrence>,
}

fn admit_planner_source_evidence(
    input: RefactoringPlannerInput<'_>,
    target: &GraphNode,
    cancellation: &dyn RefactoringCancellationSignal,
) -> Result<AdmittedPlannerSourceEvidence, RefactoringError> {
    observe_refactoring_cancellation(cancellation)?;
    let evidence = input.source_evidence();
    if evidence.configuration_id() != input.configuration_id() {
        return Err(RefactoringError::closed(
            RefactoringErrorKind::IncompatibleEvidence,
        ));
    }
    if evidence.documents().is_empty() {
        return Err(RefactoringError::closed(
            RefactoringErrorKind::SourceEvidenceMissing,
        ));
    }

    let mut candidate_count = 0_usize;
    let mut occurrences = Vec::new();
    let mut declaration_count = 0_usize;
    let mut declaration = None;
    let mut failure = None;
    for document in evidence.documents() {
        observe_refactoring_cancellation(cancellation)?;
        validate_planner_document(document)?;
        candidate_count = candidate_count
            .checked_add(document.occurrences().len())
            .ok_or_else(|| RefactoringError::closed(RefactoringErrorKind::ArithmeticOverflow))?;
        for occurrence in document.occurrences() {
            if let Some(kind) = planner_occurrence_failure(target, occurrence) {
                let candidate = (kind, document.id().clone(), occurrence.range());
                if failure.as_ref().is_none_or(|current| &candidate < current) {
                    failure = Some(candidate);
                }
            }
            if occurrence.mapped_target_id() == Some(target.id()) {
                if occurrence.kind() == SourceOccurrenceKind::Declaration {
                    declaration_count = declaration_count.checked_add(1).ok_or_else(|| {
                        RefactoringError::closed(RefactoringErrorKind::ArithmeticOverflow)
                    })?;
                    declaration.get_or_insert_with(|| occurrence.clone());
                }
                if occurrences.len() < MAX_REFACTORING_CANDIDATES {
                    occurrences.push(occurrence.clone());
                }
            }
        }
    }
    observe_refactoring_cancellation(cancellation)?;
    if let Some((kind, _, _)) = failure {
        return Err(RefactoringError::closed(kind));
    }
    if declaration_count == 0 {
        return Err(RefactoringError::closed(
            RefactoringErrorKind::SourceEvidenceMissing,
        ));
    }
    if declaration_count != 1 {
        return Err(RefactoringError::closed(
            RefactoringErrorKind::InvalidOccurrence,
        ));
    }
    Ok(AdmittedPlannerSourceEvidence {
        candidate_count,
        declaration: declaration
            .ok_or_else(|| RefactoringError::closed(RefactoringErrorKind::SourceEvidenceMissing))?,
        occurrences,
    })
}

fn validate_planner_document(document: &SourceDocument) -> Result<(), RefactoringError> {
    if document.format() == SourceFormat::DesignerXml
        && !matches!(
            document.module_role(),
            BslModuleRole::Object | BslModuleRole::Manager | BslModuleRole::Common
        )
    {
        return Err(RefactoringError::closed(
            RefactoringErrorKind::UnsupportedSourceFormat,
        ));
    }
    if document.completeness() != SourceEvidenceCompleteness::BslCallableRenameV1 {
        return Err(RefactoringError::closed(
            RefactoringErrorKind::SourceEvidenceIncomplete,
        ));
    }
    Ok(())
}

fn planner_occurrence_failure(
    target: &GraphNode,
    occurrence: &SourceOccurrence,
) -> Option<RefactoringErrorKind> {
    if occurrence.resolution() != SourceOccurrenceResolution::Unique
        && bsl_names_equal(occurrence.token(), target.name().as_str())
    {
        Some(RefactoringErrorKind::AmbiguousOccurrence)
    } else if occurrence.mapped_target_id() == Some(target.id())
        && !bsl_names_equal(occurrence.token(), target.name().as_str())
    {
        Some(RefactoringErrorKind::InvalidOccurrence)
    } else {
        None
    }
}

fn validate_planner_collisions(
    configuration_id: &EntityId,
    request: &RefactoringRequest,
    query: &SemanticGraphQuery<'_>,
    target_node: &GraphNode,
    owner_module: &GraphNode,
    declaration: SourceOccurrence,
) -> Result<RefactoringTarget, RefactoringError> {
    if bsl_names_equal(request.desired_name(), target_node.name().as_str()) {
        return Err(RefactoringError::closed(RefactoringErrorKind::NoChange));
    }
    for kind in [NodeKind::Procedure, NodeKind::Function] {
        for sibling in query.children_by_kind(&NodeId::new(owner_module.id().as_str()), kind) {
            if sibling.id() != target_node.id()
                && bsl_names_equal(sibling.name().as_str(), request.desired_name())
            {
                return Err(RefactoringError::closed(
                    RefactoringErrorKind::NameCollision,
                ));
            }
        }
    }
    let target = RefactoringTarget::new(
        configuration_id.clone(),
        target_node.id().clone(),
        target_node.kind(),
        owner_module.id().clone(),
        declaration,
        request.desired_name(),
    )?;
    if query
        .node_by_entity_id(target.expected_post_rename_node_id())
        .is_some_and(|node| node.id() != target_node.id())
    {
        return Err(RefactoringError::closed(
            RefactoringErrorKind::IdentityCollision,
        ));
    }
    Ok(target)
}

fn build_refactoring_plan(
    input: RefactoringPlannerInput<'_>,
    request: &RefactoringRequest,
    target: RefactoringTarget,
    occurrences: Vec<SourceOccurrence>,
    cancellation: &dyn RefactoringCancellationSignal,
) -> Result<RefactoringPlan, RefactoringError> {
    let mut operations = Vec::with_capacity(occurrences.len());
    let mut source_preconditions = BTreeMap::new();
    for occurrence in occurrences {
        observe_refactoring_cancellation(cancellation)?;
        let kind = match occurrence.kind() {
            SourceOccurrenceKind::Declaration => {
                RefactoringOperationKind::ReplaceDeclarationIdentifier
            }
            SourceOccurrenceKind::LocalCall | SourceOccurrenceKind::QualifiedCall => {
                RefactoringOperationKind::ReplaceDirectCallIdentifier
            }
        };
        source_preconditions.insert(
            occurrence.document_id().clone(),
            RefactoringSourcePrecondition::new(
                occurrence.document_id().clone(),
                occurrence.content_version(),
            ),
        );
        operations.push(RefactoringOperation::new(
            kind,
            occurrence.kind(),
            occurrence.document_id().clone(),
            occurrence.content_version(),
            occurrence.range(),
            occurrence.token(),
            request.desired_name(),
            &[],
        )?);
    }
    let preconditions = RefactoringPreconditionSet::new(
        input.publication_id(),
        input.configuration_id().clone(),
        target.target_node_id().clone(),
        target.target_kind(),
        target.owner_module_id().clone(),
        source_preconditions.into_values().collect(),
    )?;
    RefactoringPlan::new(request.clone(), target, preconditions, operations)
}

fn build_refactoring_preview(
    plan: &RefactoringPlan,
    evidence: &SourceEvidenceSet,
    cancellation: &dyn RefactoringCancellationSignal,
) -> Result<RefactoringPreview, RefactoringError> {
    let mut entries = Vec::with_capacity(plan.operations().len());
    for operation in plan.operations() {
        observe_refactoring_cancellation(cancellation)?;
        let document = evidence
            .documents()
            .binary_search_by(|candidate| candidate.id().cmp(operation.document_id()))
            .ok()
            .map(|index| &evidence.documents()[index])
            .ok_or_else(|| RefactoringError::closed(RefactoringErrorKind::SourceEvidenceMissing))?;
        if document.content_version() != operation.content_version() {
            return Err(RefactoringError::closed(
                RefactoringErrorKind::StaleSourceVersion,
            ));
        }
        let range = operation.range();
        let raw = document.raw_content();
        if raw.get(range.start_byte()..range.end_byte()) != Some(operation.expected().as_bytes()) {
            return Err(RefactoringError::closed(
                RefactoringErrorKind::InvalidOccurrence,
            ));
        }
        let position = raw_range_to_source_span(raw, range)?;
        entries.push(RefactoringPreviewEntry::new(
            operation,
            document.path().clone(),
            position,
        )?);
    }
    RefactoringPreview::new(plan, entries)
}

fn raw_range_to_source_span(
    raw: &[u8],
    range: SourceByteRange,
) -> Result<SourceSpan, RefactoringError> {
    let source = std::str::from_utf8(raw)
        .map_err(|_| RefactoringError::closed(RefactoringErrorKind::InvalidOccurrence))?;
    if range.end_byte() > raw.len()
        || !source.is_char_boundary(range.start_byte())
        || !source.is_char_boundary(range.end_byte())
    {
        return Err(RefactoringError::closed(
            RefactoringErrorKind::InvalidOccurrence,
        ));
    }
    let start = raw_offset_to_source_position(raw, range.start_byte())?;
    let end = raw_offset_to_source_position(raw, range.end_byte())?;
    SourceSpan::new(start, end)
        .map_err(|_| RefactoringError::closed(RefactoringErrorKind::InvalidOccurrence))
}

fn raw_offset_to_source_position(
    raw: &[u8],
    offset: usize,
) -> Result<SourcePosition, RefactoringError> {
    let mut cursor = usize::from(raw.starts_with(UTF8_BOM)) * UTF8_BOM.len();
    if offset < cursor || offset > raw.len() {
        return Err(RefactoringError::closed(
            RefactoringErrorKind::InvalidOccurrence,
        ));
    }
    let mut line = 1_u32;
    let mut column = 1_u32;
    while cursor < offset {
        match raw[cursor] {
            b'\r' => {
                cursor += 1;
                if cursor < raw.len() && raw[cursor] == b'\n' {
                    cursor += 1;
                }
                line = line.checked_add(1).ok_or_else(|| {
                    RefactoringError::closed(RefactoringErrorKind::ArithmeticOverflow)
                })?;
                column = 1;
            }
            b'\n' => {
                cursor += 1;
                line = line.checked_add(1).ok_or_else(|| {
                    RefactoringError::closed(RefactoringErrorKind::ArithmeticOverflow)
                })?;
                column = 1;
            }
            _ => {
                let scalar = std::str::from_utf8(&raw[cursor..])
                    .map_err(|_| RefactoringError::closed(RefactoringErrorKind::InvalidOccurrence))?
                    .chars()
                    .next()
                    .ok_or_else(|| {
                        RefactoringError::closed(RefactoringErrorKind::InvalidOccurrence)
                    })?;
                cursor = cursor.checked_add(scalar.len_utf8()).ok_or_else(|| {
                    RefactoringError::closed(RefactoringErrorKind::ArithmeticOverflow)
                })?;
                column = column.checked_add(1).ok_or_else(|| {
                    RefactoringError::closed(RefactoringErrorKind::ArithmeticOverflow)
                })?;
            }
        }
        if cursor > offset {
            return Err(RefactoringError::closed(
                RefactoringErrorKind::InvalidOccurrence,
            ));
        }
    }
    SourcePosition::new(line, column)
        .map_err(|_| RefactoringError::closed(RefactoringErrorKind::InvalidOccurrence))
}

fn observe_refactoring_cancellation(
    cancellation: &dyn RefactoringCancellationSignal,
) -> Result<(), RefactoringError> {
    if cancellation.is_cancelled() {
        Err(RefactoringError::closed(RefactoringErrorKind::Cancelled))
    } else {
        Ok(())
    }
}

fn has_unique_configuration_owner_chain(
    query: &SemanticGraphQuery<'_>,
    child: &EntityId,
    configuration: &EntityId,
    cancellation: &dyn RefactoringCancellationSignal,
) -> Result<bool, RefactoringError> {
    let mut current = child.clone();
    let mut visited = BTreeSet::from([current.clone()]);
    loop {
        observe_refactoring_cancellation(cancellation)?;
        let owners = query.owners(&NodeId::new(current.as_str()));
        if owners.len() != 1 {
            return Ok(false);
        }
        let owner = owners[0];
        validate_plan_identity(owner.id())?;
        if owner.id() == configuration {
            return Ok(true);
        }
        if !visited.insert(owner.id().clone()) {
            return Ok(false);
        }
        current = owner.id().clone();
    }
}

fn validate_plan_relationships(
    request: &RefactoringRequest,
    target: &RefactoringTarget,
    preconditions: &RefactoringPreconditionSet,
) -> Result<(), RefactoringError> {
    if request.configuration_id() != target.configuration_id()
        || request.target_node_id() != target.target_node_id()
        || request.expected_publication_id() != preconditions.publication_id()
        || request.configuration_id() != preconditions.configuration_id()
        || request.target_node_id() != preconditions.target_node_id()
        || target.target_kind() != preconditions.target_kind()
        || target.owner_module_id() != preconditions.owner_module_id()
    {
        return Err(RefactoringError::closed(
            RefactoringErrorKind::IncompatibleEvidence,
        ));
    }
    if bsl_names_equal(request.desired_name(), target.declaration().token()) {
        return Err(RefactoringError::closed(RefactoringErrorKind::NoChange));
    }
    let symbol_kind = match target.target_kind() {
        NodeKind::Procedure => BslSymbolKind::Procedure,
        NodeKind::Function => BslSymbolKind::Function,
        _ => {
            return Err(RefactoringError::closed(
                RefactoringErrorKind::UnsupportedTarget,
            ));
        }
    };
    let expected = bsl_callable_id(
        target.owner_module_id(),
        symbol_kind,
        request.desired_name(),
    )
    .map_err(|_| RefactoringError::closed(RefactoringErrorKind::IdentityCollision))?;
    if &expected != target.expected_post_rename_node_id() {
        return Err(RefactoringError::closed(
            RefactoringErrorKind::IncompatibleEvidence,
        ));
    }
    Ok(())
}

fn validate_operation_relationship(
    request: &RefactoringRequest,
    target: &RefactoringTarget,
    preconditions: &RefactoringPreconditionSet,
    operation: &RefactoringOperation,
) -> Result<(), RefactoringError> {
    if operation.document_id().configuration_id() != request.configuration_id()
        || operation.replacement() != request.desired_name()
        || !bsl_names_equal(operation.expected(), target.declaration().token())
    {
        return Err(RefactoringError::closed(
            RefactoringErrorKind::IncompatibleEvidence,
        ));
    }
    let source = preconditions
        .documents()
        .binary_search_by(|candidate| candidate.document_id().cmp(operation.document_id()))
        .ok()
        .map(|index| &preconditions.documents()[index])
        .ok_or_else(|| RefactoringError::closed(RefactoringErrorKind::SourceEvidenceMissing))?;
    if source.content_version() != operation.content_version() {
        return Err(RefactoringError::closed(
            RefactoringErrorKind::StaleSourceVersion,
        ));
    }
    if operation.kind() == RefactoringOperationKind::ReplaceDeclarationIdentifier
        && (operation.document_id() != target.declaration().document_id()
            || operation.content_version() != target.declaration().content_version()
            || operation.range() != target.declaration().range()
            || operation.expected() != target.declaration().token())
    {
        return Err(RefactoringError::closed(
            RefactoringErrorKind::InvalidOccurrence,
        ));
    }
    Ok(())
}

fn summarize_operations(
    candidate_occurrences: usize,
    operations: &[RefactoringOperation],
    preconditions: &RefactoringPreconditionSet,
) -> Result<RefactoringPlanSummary, RefactoringError> {
    let count = |kind| {
        operations
            .iter()
            .filter(|operation| operation.occurrence_kind() == kind)
            .count()
    };
    let declaration_operations = count(SourceOccurrenceKind::Declaration);
    if declaration_operations != 1 {
        return Err(RefactoringError::closed(
            RefactoringErrorKind::InvalidOccurrence,
        ));
    }
    let local_call_operations = count(SourceOccurrenceKind::LocalCall);
    let qualified_call_operations = count(SourceOccurrenceKind::QualifiedCall);
    let classified_operations = declaration_operations
        .checked_add(local_call_operations)
        .and_then(|count| count.checked_add(qualified_call_operations))
        .ok_or_else(|| RefactoringError::closed(RefactoringErrorKind::ArithmeticOverflow))?;
    if classified_operations != operations.len() {
        return Err(RefactoringError::closed(
            RefactoringErrorKind::IncompatibleEvidence,
        ));
    }
    let exact_duplicates_collapsed = candidate_occurrences
        .checked_sub(operations.len())
        .ok_or_else(|| RefactoringError::closed(RefactoringErrorKind::ArithmeticOverflow))?;
    let documents = operations
        .iter()
        .map(RefactoringOperation::document_id)
        .collect::<BTreeSet<_>>()
        .len();
    if documents != preconditions.documents().len() {
        return Err(RefactoringError::closed(
            RefactoringErrorKind::IncompatibleEvidence,
        ));
    }
    Ok(RefactoringPlanSummary {
        requested_targets: MAX_REFACTORING_TARGETS,
        planned_targets: MAX_REFACTORING_TARGETS,
        conflicted_targets: 0,
        rejected_targets: 0,
        documents,
        candidate_occurrences,
        exact_duplicates_collapsed,
        declaration_operations,
        local_call_operations,
        qualified_call_operations,
        planned_operations: operations.len(),
        omitted_operations: 0,
        returned_operations: operations.len(),
    })
}

fn validate_operation_conflicts(
    operations: &[RefactoringOperation],
) -> Result<(), RefactoringError> {
    let mut versions = BTreeMap::new();
    for operation in operations {
        match versions.entry(operation.document_id()) {
            std::collections::btree_map::Entry::Vacant(entry) => {
                entry.insert(operation.content_version());
            }
            std::collections::btree_map::Entry::Occupied(entry)
                if *entry.get() == operation.content_version() => {}
            std::collections::btree_map::Entry::Occupied(_) => {
                return Err(RefactoringError::closed(
                    RefactoringErrorKind::StaleSourceVersion,
                ));
            }
        }
    }
    for (index, left) in operations.iter().enumerate() {
        for right in operations.iter().skip(index + 1) {
            if left.document_id() != right.document_id() {
                break;
            }
            if left.range() == right.range() {
                return Err(RefactoringError::closed(
                    RefactoringErrorKind::DuplicateConflict,
                ));
            }
            if ranges_intersect(left.range(), right.range()) {
                return Err(RefactoringError::closed(
                    RefactoringErrorKind::OverlappingOperations,
                ));
            }
        }
    }
    Ok(())
}

const fn ranges_intersect(left: SourceByteRange, right: SourceByteRange) -> bool {
    left.start_byte() < right.end_byte() && right.start_byte() < left.end_byte()
}

fn plan_id(
    request: &RefactoringRequest,
    target: &RefactoringTarget,
    preconditions: &RefactoringPreconditionSet,
    operations: &[RefactoringOperation],
) -> Result<PlanId, RefactoringError> {
    let mut encoding = Vec::new();
    encode_string(&mut encoding, request.family().as_str())?;
    encoding.extend_from_slice(&request.expected_publication_id().get().to_be_bytes());
    encode_entity_id(&mut encoding, request.configuration_id())?;
    encode_entity_id(&mut encoding, request.target_node_id())?;
    encode_string(&mut encoding, request.desired_name())?;
    encode_entity_id(&mut encoding, target.configuration_id())?;
    encode_entity_id(&mut encoding, target.target_node_id())?;
    encode_string(&mut encoding, node_kind_tag(target.target_kind()))?;
    encode_entity_id(&mut encoding, target.owner_module_id())?;
    encode_document_id(&mut encoding, target.declaration().document_id())?;
    encode_content_version(&mut encoding, target.declaration().content_version())?;
    encode_usize(&mut encoding, target.declaration().range().start_byte())?;
    encode_usize(&mut encoding, target.declaration().range().end_byte())?;
    encode_string(&mut encoding, target.declaration().token())?;
    encode_entity_id(&mut encoding, target.expected_post_rename_node_id())?;
    encoding.extend_from_slice(&preconditions.publication_id().get().to_be_bytes());
    encode_entity_id(&mut encoding, preconditions.configuration_id())?;
    encode_entity_id(&mut encoding, preconditions.target_node_id())?;
    encode_string(&mut encoding, node_kind_tag(preconditions.target_kind()))?;
    encode_entity_id(&mut encoding, preconditions.owner_module_id())?;
    encode_usize(&mut encoding, preconditions.documents().len())?;
    for document in preconditions.documents() {
        encode_document_id(&mut encoding, document.document_id())?;
        encode_content_version(&mut encoding, document.content_version())?;
    }
    encode_usize(&mut encoding, operations.len())?;
    for operation in operations {
        encode_string(&mut encoding, operation.id().as_str())?;
    }
    Ok(PlanId(sha256_hex(&encoding).into_boxed_str()))
}

const fn node_kind_tag(kind: NodeKind) -> &'static str {
    match kind {
        NodeKind::Procedure => "procedure",
        NodeKind::Function => "function",
        _ => "unsupported",
    }
}

fn encode_entity_id(output: &mut Vec<u8>, value: &EntityId) -> Result<(), RefactoringError> {
    encode_string(output, value.as_str())
}

fn encode_document_id(
    output: &mut Vec<u8>,
    value: &SourceDocumentId,
) -> Result<(), RefactoringError> {
    encode_entity_id(output, value.configuration_id())?;
    encode_entity_id(output, value.module_id())
}

fn encode_content_version(
    output: &mut Vec<u8>,
    value: SourceContentVersion,
) -> Result<(), RefactoringError> {
    encode_usize(output, value.raw_byte_len())?;
    output.extend_from_slice(&value.digest());
    Ok(())
}

fn encode_string(output: &mut Vec<u8>, value: &str) -> Result<(), RefactoringError> {
    encode_usize(output, value.len())?;
    output.extend_from_slice(value.as_bytes());
    Ok(())
}

fn encode_usize(output: &mut Vec<u8>, value: usize) -> Result<(), RefactoringError> {
    let value = u64::try_from(value)
        .map_err(|_| RefactoringError::closed(RefactoringErrorKind::ArithmeticOverflow))?;
    output.extend_from_slice(&value.to_be_bytes());
    Ok(())
}
