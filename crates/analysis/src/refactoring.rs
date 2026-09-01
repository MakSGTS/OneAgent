//! Immutable source evidence for deterministic refactoring planning.
//!
//! This module owns source-independent retained documents and occurrences. It
//! performs no filesystem access, semantic resolution, planning, or mutation.

use std::collections::BTreeSet;
use std::fmt::{Display, Formatter};
use std::sync::Arc;

use oneagent_common::{EntityId, SourcePath, sha256};

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
