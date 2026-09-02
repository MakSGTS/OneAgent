//! Private deterministic codec for complete Workspace cache snapshots.

use std::fmt::{Display, Formatter};
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Component, Path, PathBuf};

use oneagent_analysis::refactoring::{
    BslModuleRole, ConfinedSourcePath, SourceByteRange, SourceContentVersion, SourceDocument,
    SourceDocumentId, SourceEvidenceCompleteness, SourceEvidenceSet, SourceFormat,
    SourceOccurrence, SourceOccurrenceKind, SourceOccurrenceResolution,
};
use oneagent_common::{
    EntityId, EntityName, SourceLocation, SourcePath, SourcePosition, SourceSpan,
};
use oneagent_graph::{
    AccessRightPayload, AccessRightRowRestriction, Confidence, DataCompositionFieldPayload,
    DataCompositionSchemaPayload, DataSetKind, DataSetPayload, EdgeKind, FactOrigin, GraphEdge,
    GraphNode, GraphNodePayload, HttpServiceMethodPayload, HttpServiceUrlTemplatePayload, NodeKind,
    ProducerId, Provenance, ResolutionState, SemanticDiagnostic, SemanticDiagnosticCode,
    SemanticDiagnosticKind, SemanticDiagnosticSeverity, SemanticGraph, SemanticGraphReport,
    SemanticGraphValidator, SemanticReference, SemanticReferenceCategory, SemanticReferenceOutcome,
    SemanticReferenceRequest, SemanticReferenceRequestLedger, SemanticReferenceRequestOutcome,
    SemanticReferenceStatistics, WebServiceOperationPayload, WebServiceParameterDirection,
    WebServiceParameterPayload, XdtoTypeKind, XdtoTypePayload, XdtoTypeReference,
};
use oneagent_metadata::{
    CommonMetadataPayload, DocumentMetadataPayload, EventSubscriptionMetadataPayload,
    HttpServiceMetadataPayload, MetadataKind, MetadataMemberPayload, MetadataPayload,
    MetadataRegisterRecord, MetadataSpecificPayload, WebServiceMetadataPayload,
    WebServiceXdtoPackage, XdtoPackageMetadataPayload,
};
use oneagent_workspace::WorkspaceFormat;
use serde::{Deserialize, Serialize};

use super::change::{WorkspaceFileEntry, WorkspaceFileState};
use super::{WorkspaceConfigurationSnapshot, WorkspaceSnapshot, snapshot_from_parts};

const CACHE_FORMAT: &str = "oneagent.workspace-cache";
// Review and bump this whenever persisted fields or closed vocabularies change.
const SCHEMA_VERSION: u32 = 1;
// Bump this in the same logical change as any behavior that can change a
// complete snapshot for equal source state; package and Git versions do not
// replace this manual compatibility boundary.
const SEMANTIC_VERSION: u32 = 6;
const FNV_OFFSET_BASIS: u64 = 14_695_981_039_346_656_037;
const FNV_PRIME: u64 = 1_099_511_628_211;
const CACHE_OWNER_DIRECTORY: &str = ".oneagent";
const CACHE_DIRECTORY: &str = "cache";
const CACHE_FILE: &str = "workspace-v1.json";
const CACHE_TEMPORARY_FILE: &str = "workspace-v1.tmp";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum WorkspaceCacheCodecErrorKind {
    Malformed,
    Partial,
    Duplicate,
    Unsupported,
    Incompatible,
    NonCanonical,
    ChecksumMismatch,
    SourceMismatch,
    Invalid,
    Inconsistent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct WorkspaceCacheCodecError {
    kind: WorkspaceCacheCodecErrorKind,
    message: String,
}

impl WorkspaceCacheCodecError {
    fn new(kind: WorkspaceCacheCodecErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    pub(super) const fn kind(&self) -> WorkspaceCacheCodecErrorKind {
        self.kind
    }
}

impl Display for WorkspaceCacheCodecError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for WorkspaceCacheCodecError {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct WorkspaceCacheSource {
    pub(super) entries: Vec<WorkspaceCacheSourceEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct WorkspaceCacheSourceEntry {
    pub(super) path: Vec<String>,
    pub(super) kind: WorkspaceCacheSourceEntryKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) bytes: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum WorkspaceCacheSourceEntryKind {
    Directory,
    RegularFile,
    Other,
}

impl TryFrom<&WorkspaceFileState> for WorkspaceCacheSource {
    type Error = WorkspaceCacheCodecError;

    fn try_from(value: &WorkspaceFileState) -> Result<Self, Self::Error> {
        let entries = value
            .entries()
            .map(|(path, entry)| {
                let path = path
                    .components()
                    .map(|component| match component {
                        Component::Normal(value) => value
                            .to_str()
                            .map(ToOwned::to_owned)
                            .ok_or_else(|| invalid("workspace cache source path is not UTF-8")),
                        _ => Err(invalid(
                            "workspace cache source path contains a non-relative component",
                        )),
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                let (kind, bytes) = match entry {
                    WorkspaceFileEntry::Directory => {
                        (WorkspaceCacheSourceEntryKind::Directory, None)
                    }
                    WorkspaceFileEntry::RegularFile(bytes) => (
                        WorkspaceCacheSourceEntryKind::RegularFile,
                        Some(bytes.clone()),
                    ),
                    WorkspaceFileEntry::Other => (WorkspaceCacheSourceEntryKind::Other, None),
                };
                Ok(WorkspaceCacheSourceEntry { path, kind, bytes })
            })
            .collect::<Result<Vec<_>, WorkspaceCacheCodecError>>()?;
        let source = Self { entries };
        validate_source(&source)?;
        Ok(source)
    }
}

pub(super) struct WorkspaceCacheCodec;

impl WorkspaceCacheCodec {
    pub(super) fn encode(
        source: &WorkspaceCacheSource,
        workspace_root: &Path,
        snapshot: &WorkspaceSnapshot,
    ) -> Result<Vec<u8>, WorkspaceCacheCodecError> {
        validate_source(source)?;
        let workspace = WorkspaceDto::from_snapshot(source, workspace_root, snapshot)?;
        let checksum = content_checksum(source, &workspace)?;
        let envelope = EnvelopeDto {
            format: CACHE_FORMAT.to_owned(),
            schema_version: SCHEMA_VERSION,
            semantic_version: SEMANTIC_VERSION,
            content_checksum: checksum,
            source: source.clone(),
            workspace,
        };
        serde_json::to_vec(&envelope).map_err(serde_encode_error)
    }

    pub(super) fn decode(
        bytes: &[u8],
        expected_source: &WorkspaceCacheSource,
        workspace_root: &Path,
    ) -> Result<WorkspaceSnapshot, WorkspaceCacheCodecError> {
        validate_source(expected_source)?;
        let envelope: EnvelopeDto = serde_json::from_slice(bytes).map_err(serde_decode_error)?;
        if envelope.format != CACHE_FORMAT
            || envelope.schema_version != SCHEMA_VERSION
            || envelope.semantic_version != SEMANTIC_VERSION
        {
            return Err(WorkspaceCacheCodecError::new(
                WorkspaceCacheCodecErrorKind::Incompatible,
                "workspace cache format or version is incompatible",
            ));
        }
        let canonical = serde_json::to_vec(&envelope).map_err(serde_encode_error)?;
        if canonical != bytes {
            return Err(WorkspaceCacheCodecError::new(
                WorkspaceCacheCodecErrorKind::NonCanonical,
                "workspace cache bytes are not canonical",
            ));
        }
        let checksum = content_checksum(&envelope.source, &envelope.workspace)?;
        if checksum != envelope.content_checksum {
            return Err(WorkspaceCacheCodecError::new(
                WorkspaceCacheCodecErrorKind::ChecksumMismatch,
                "workspace cache content checksum does not match",
            ));
        }
        validate_source(&envelope.source)?;
        if envelope.source != *expected_source {
            return Err(WorkspaceCacheCodecError::new(
                WorkspaceCacheCodecErrorKind::SourceMismatch,
                "workspace cache source state does not match",
            ));
        }
        envelope
            .workspace
            .into_snapshot(&envelope.source, workspace_root)
    }
}

/// Closed outcome of the latest persistent Workspace cache load attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceCacheLoadOutcome {
    /// No load has been attempted by this service.
    NotAttempted,
    /// A complete compatible snapshot matched the observed source exactly.
    Hit,
    /// No cache candidate exists at the fixed Workspace-local location.
    Missing,
    /// The candidate describes a different complete source state.
    SourceChanged,
    /// The candidate uses an unsupported cache format or semantic version.
    Incompatible,
    /// The candidate is malformed, incomplete, noncanonical, or semantically invalid.
    Corrupt,
    /// The candidate could not be accessed safely or completely.
    Unavailable,
}

#[derive(Debug)]
pub(super) struct WorkspaceCacheLoad {
    outcome: WorkspaceCacheLoadOutcome,
    snapshot: Option<WorkspaceSnapshot>,
}

impl WorkspaceCacheLoad {
    pub(super) const fn outcome(&self) -> WorkspaceCacheLoadOutcome {
        self.outcome
    }

    pub(super) fn into_snapshot(self) -> Option<WorkspaceSnapshot> {
        self.snapshot
    }

    #[cfg(test)]
    pub(super) const fn hit(snapshot: WorkspaceSnapshot) -> Self {
        Self {
            outcome: WorkspaceCacheLoadOutcome::Hit,
            snapshot: Some(snapshot),
        }
    }

    pub(super) const fn without_snapshot(outcome: WorkspaceCacheLoadOutcome) -> Self {
        Self {
            outcome,
            snapshot: None,
        }
    }
}

/// Closed outcome of the latest persistent Workspace cache write decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceCacheWriteOutcome {
    /// No write has been attempted or skipped by this service.
    NotAttempted,
    /// A complete validated snapshot replaced the current cache entry.
    Succeeded,
    /// The source changed during the build, so no cache entry was written.
    SkippedUnstableSource,
    /// The cache entry could not be replaced safely and completely.
    Failed,
}

pub(super) trait WorkspaceCacheStorage: Send + Sync {
    fn load(&self, state: &WorkspaceFileState) -> WorkspaceCacheLoad;

    fn write(
        &self,
        state: &WorkspaceFileState,
        snapshot: &WorkspaceSnapshot,
    ) -> WorkspaceCacheWriteOutcome;
}

#[derive(Debug)]
pub(super) struct WorkspaceCacheStore {
    workspace_root: PathBuf,
    #[cfg(test)]
    failure: Option<WorkspaceCacheFailurePoint>,
}

impl WorkspaceCacheStore {
    pub(super) fn new(workspace_root: PathBuf) -> Self {
        Self {
            workspace_root,
            #[cfg(test)]
            failure: None,
        }
    }

    pub(super) fn load(&self, state: &WorkspaceFileState) -> WorkspaceCacheLoad {
        let Ok(source) = WorkspaceCacheSource::try_from(state) else {
            return WorkspaceCacheLoad::without_snapshot(WorkspaceCacheLoadOutcome::Unavailable);
        };
        let owner = self.workspace_root.join(CACHE_OWNER_DIRECTORY);
        let cache = owner.join(CACHE_DIRECTORY);
        for directory in [&owner, &cache] {
            match existing_real_directory(directory) {
                Ok(true) => {}
                Ok(false) => {
                    return WorkspaceCacheLoad::without_snapshot(
                        WorkspaceCacheLoadOutcome::Missing,
                    );
                }
                Err(_) => {
                    return WorkspaceCacheLoad::without_snapshot(
                        WorkspaceCacheLoadOutcome::Unavailable,
                    );
                }
            }
        }

        let candidate = cache.join(CACHE_FILE);
        match existing_regular_file(&candidate) {
            Ok(true) => {}
            Ok(false) => {
                return WorkspaceCacheLoad::without_snapshot(WorkspaceCacheLoadOutcome::Missing);
            }
            Err(_) => {
                return WorkspaceCacheLoad::without_snapshot(
                    WorkspaceCacheLoadOutcome::Unavailable,
                );
            }
        }
        let Ok(bytes) = fs::read(candidate) else {
            return WorkspaceCacheLoad::without_snapshot(WorkspaceCacheLoadOutcome::Unavailable);
        };
        match WorkspaceCacheCodec::decode(&bytes, &source, &self.workspace_root) {
            Ok(snapshot) => WorkspaceCacheLoad {
                outcome: WorkspaceCacheLoadOutcome::Hit,
                snapshot: Some(snapshot),
            },
            Err(error) => WorkspaceCacheLoad::without_snapshot(match error.kind() {
                WorkspaceCacheCodecErrorKind::Incompatible => {
                    WorkspaceCacheLoadOutcome::Incompatible
                }
                WorkspaceCacheCodecErrorKind::SourceMismatch => {
                    WorkspaceCacheLoadOutcome::SourceChanged
                }
                WorkspaceCacheCodecErrorKind::Malformed
                | WorkspaceCacheCodecErrorKind::Partial
                | WorkspaceCacheCodecErrorKind::Duplicate
                | WorkspaceCacheCodecErrorKind::Unsupported
                | WorkspaceCacheCodecErrorKind::NonCanonical
                | WorkspaceCacheCodecErrorKind::ChecksumMismatch
                | WorkspaceCacheCodecErrorKind::Invalid
                | WorkspaceCacheCodecErrorKind::Inconsistent => WorkspaceCacheLoadOutcome::Corrupt,
            }),
        }
    }

    pub(super) fn write(
        &self,
        state: &WorkspaceFileState,
        snapshot: &WorkspaceSnapshot,
    ) -> WorkspaceCacheWriteOutcome {
        let result = self.write_inner(state, snapshot);
        if result.is_err() {
            self.cleanup_temporary();
            WorkspaceCacheWriteOutcome::Failed
        } else {
            WorkspaceCacheWriteOutcome::Succeeded
        }
    }

    fn write_inner(
        &self,
        state: &WorkspaceFileState,
        snapshot: &WorkspaceSnapshot,
    ) -> Result<(), WorkspaceCacheStoreError> {
        let source =
            WorkspaceCacheSource::try_from(state).map_err(WorkspaceCacheStoreError::Codec)?;
        let bytes = WorkspaceCacheCodec::encode(&source, &self.workspace_root, snapshot)
            .map_err(WorkspaceCacheStoreError::Codec)?;
        WorkspaceCacheCodec::decode(&bytes, &source, &self.workspace_root)
            .map_err(WorkspaceCacheStoreError::Codec)?;

        let owner = self.workspace_root.join(CACHE_OWNER_DIRECTORY);
        let cache = owner.join(CACHE_DIRECTORY);
        ensure_real_directory(&owner).map_err(WorkspaceCacheStoreError::Io)?;
        ensure_real_directory(&cache).map_err(WorkspaceCacheStoreError::Io)?;

        let temporary = cache.join(CACHE_TEMPORARY_FILE);
        remove_existing_regular_file(&temporary).map_err(WorkspaceCacheStoreError::Io)?;
        #[cfg(test)]
        self.inject(WorkspaceCacheFailurePoint::CreateTemporary)?;
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .map_err(WorkspaceCacheStoreError::Io)?;
        #[cfg(test)]
        self.inject(WorkspaceCacheFailurePoint::WriteTemporary)?;
        file.write_all(&bytes)
            .map_err(WorkspaceCacheStoreError::Io)?;
        #[cfg(test)]
        self.inject(WorkspaceCacheFailurePoint::SyncTemporary)?;
        file.sync_all().map_err(WorkspaceCacheStoreError::Io)?;
        drop(file);

        #[cfg(test)]
        self.inject(WorkspaceCacheFailurePoint::ReadBackTemporary)?;
        let read_back = fs::read(&temporary).map_err(WorkspaceCacheStoreError::Io)?;
        WorkspaceCacheCodec::decode(&read_back, &source, &self.workspace_root)
            .map_err(WorkspaceCacheStoreError::Codec)?;
        if read_back != bytes {
            return Err(WorkspaceCacheStoreError::Verification);
        }

        let candidate = cache.join(CACHE_FILE);
        #[cfg(test)]
        self.inject(WorkspaceCacheFailurePoint::RemoveCurrent)?;
        remove_existing_regular_file(&candidate).map_err(WorkspaceCacheStoreError::Io)?;
        #[cfg(test)]
        self.inject(WorkspaceCacheFailurePoint::RenameTemporary)?;
        fs::rename(&temporary, &candidate).map_err(WorkspaceCacheStoreError::Io)?;
        Ok(())
    }

    fn cleanup_temporary(&self) {
        let owner = self.workspace_root.join(CACHE_OWNER_DIRECTORY);
        if !existing_real_directory(&owner).is_ok_and(|exists| exists) {
            return;
        }
        let cache = owner.join(CACHE_DIRECTORY);
        if !existing_real_directory(&cache).is_ok_and(|exists| exists) {
            return;
        }
        let temporary = cache.join(CACHE_TEMPORARY_FILE);
        if existing_regular_file(&temporary).is_ok_and(|exists| exists) {
            let _ = fs::remove_file(temporary);
        }
    }

    #[cfg(test)]
    fn inject(&self, point: WorkspaceCacheFailurePoint) -> Result<(), WorkspaceCacheStoreError> {
        if self.failure == Some(point) {
            Err(WorkspaceCacheStoreError::Injected(point))
        } else {
            Ok(())
        }
    }

    #[cfg(test)]
    fn with_failure(mut self, failure: WorkspaceCacheFailurePoint) -> Self {
        self.failure = Some(failure);
        self
    }
}

impl WorkspaceCacheStorage for WorkspaceCacheStore {
    fn load(&self, state: &WorkspaceFileState) -> WorkspaceCacheLoad {
        Self::load(self, state)
    }

    fn write(
        &self,
        state: &WorkspaceFileState,
        snapshot: &WorkspaceSnapshot,
    ) -> WorkspaceCacheWriteOutcome {
        Self::write(self, state, snapshot)
    }
}

#[derive(Debug)]
enum WorkspaceCacheStoreError {
    Io(io::Error),
    Codec(WorkspaceCacheCodecError),
    Verification,
    #[cfg(test)]
    Injected(WorkspaceCacheFailurePoint),
}

impl Display for WorkspaceCacheStoreError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "workspace cache I/O failed: {error}"),
            Self::Codec(error) => write!(formatter, "workspace cache codec failed: {error}"),
            Self::Verification => {
                formatter.write_str("workspace cache read-back verification failed")
            }
            #[cfg(test)]
            Self::Injected(point) => {
                write!(formatter, "workspace cache failure injected at {point:?}")
            }
        }
    }
}

impl std::error::Error for WorkspaceCacheStoreError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Codec(error) => Some(error),
            Self::Verification => None,
            #[cfg(test)]
            Self::Injected(_) => None,
        }
    }
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WorkspaceCacheFailurePoint {
    CreateTemporary,
    WriteTemporary,
    SyncTemporary,
    ReadBackTemporary,
    RemoveCurrent,
    RenameTemporary,
}

fn existing_real_directory(path: &Path) -> io::Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_dir() => Ok(true),
        Ok(_) => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "workspace cache path component is not a real directory",
        )),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}

fn existing_regular_file(path: &Path) -> io::Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_file() => Ok(true),
        Ok(_) => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "workspace cache candidate is not a regular file",
        )),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}

fn ensure_real_directory(path: &Path) -> io::Result<()> {
    if existing_real_directory(path)? {
        return Ok(());
    }
    fs::create_dir(path)?;
    if existing_real_directory(path)? {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "created workspace cache path is not a real directory",
        ))
    }
}

fn remove_existing_regular_file(path: &Path) -> io::Result<()> {
    if existing_regular_file(path)? {
        fs::remove_file(path)?;
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct EnvelopeDto {
    format: String,
    schema_version: u32,
    semantic_version: u32,
    content_checksum: String,
    source: WorkspaceCacheSource,
    workspace: WorkspaceDto,
}

#[derive(Serialize)]
struct ContentDto<'a> {
    source: &'a WorkspaceCacheSource,
    workspace: &'a WorkspaceDto,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct WorkspaceDto {
    configurations: Vec<ConfigurationDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ConfigurationDto {
    root: Vec<String>,
    format: FormatDto,
    source_evidence: SourceEvidenceDto,
    nodes: Vec<NodeDto>,
    edges: Vec<EdgeDto>,
    diagnostics: Vec<DiagnosticDto>,
    reference_requests: Vec<ReferenceRequestDto>,
    reference_statistics: ReferenceStatisticsDto,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceEvidenceDto {
    configuration_id: String,
    documents: Vec<SourceDocumentDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceDocumentDto {
    configuration_id: String,
    module_id: String,
    format: FormatDto,
    module_role: BslModuleRoleDto,
    path: String,
    content_version: SourceContentVersionDto,
    occurrences: Vec<SourceOccurrenceDto>,
    completeness: SourceEvidenceCompletenessDto,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceOccurrenceDto {
    configuration_id: String,
    module_id: String,
    content_version: SourceContentVersionDto,
    start_byte: usize,
    end_byte: usize,
    kind: SourceOccurrenceKindDto,
    token: String,
    mapped_target_id: Option<String>,
    resolution: SourceOccurrenceResolutionDto,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceContentVersionDto {
    raw_byte_len: usize,
    digest: [u8; 32],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum BslModuleRoleDto {
    Object,
    Manager,
    Common,
    Form,
    Command,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum SourceOccurrenceKindDto {
    Declaration,
    LocalCall,
    QualifiedCall,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum SourceOccurrenceResolutionDto {
    Unique,
    Unresolved,
    Ambiguous,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum SourceEvidenceCompletenessDto {
    BslCallableRenameV1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum FormatDto {
    Edt,
    DesignerXml,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct NodeDto {
    id: String,
    name: String,
    kind: NodeKindDto,
    payload: NodePayloadDto,
    provenance: Vec<ProvenanceDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct EdgeDto {
    source: String,
    target: String,
    kind: EdgeKindDto,
    provenance: Vec<ProvenanceDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "metadata_kind", rename_all = "snake_case")]
enum NodeKindDto {
    Metadata(MetadataKindDto),
    Module,
    Procedure,
    Function,
    Query,
    DataCompositionSchema,
    DataSet,
    DataCompositionField,
    XdtoType,
    HttpServiceUrlTemplate,
    HttpServiceMethod,
    WebServiceOperation,
    WebServiceParameter,
    Form,
    Command,
    Attribute,
    StandardAttribute,
    TabularSection,
    Dimension,
    Resource,
    Measure,
    Role,
    AccessRight,
    Subsystem,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum MetadataKindDto {
    Configuration,
    Subsystem,
    Catalog,
    Document,
    Enumeration,
    CommonModule,
    Report,
    DataProcessor,
    InformationRegister,
    AccumulationRegister,
    AccountingRegister,
    CalculationRegister,
    BusinessProcess,
    Task,
    Role,
    CommonForm,
    Form,
    Command,
    Template,
    HttpService,
    WebService,
    XdtoPackage,
    EventSubscription,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum EdgeKindDto {
    Contains,
    Calls,
    References,
    Reads,
    Writes,
    Grants,
    Includes,
    Extends,
    DependsOn,
    Opens,
    Triggers,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
enum NodePayloadDto {
    None,
    Metadata(MetadataPayloadDto),
    MetadataMember(MetadataMemberPayloadDto),
    AccessRight(AccessRightPayloadDto),
    DataCompositionSchema(DataCompositionSchemaPayloadDto),
    DataSet(DataSetPayloadDto),
    DataCompositionField(DataCompositionFieldPayloadDto),
    XdtoType(XdtoTypePayloadDto),
    HttpServiceUrlTemplate(HttpServiceUrlTemplatePayloadDto),
    HttpServiceMethod(HttpServiceMethodPayloadDto),
    WebServiceOperation(WebServiceOperationPayloadDto),
    WebServiceParameter(WebServiceParameterPayloadDto),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct MetadataPayloadDto {
    synonym: Option<String>,
    specific: Option<MetadataSpecificPayloadDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
enum MetadataSpecificPayloadDto {
    Document(DocumentMetadataPayloadDto),
    EventSubscription(EventSubscriptionMetadataPayloadDto),
    HttpService(HttpServiceMetadataPayloadDto),
    WebService(WebServiceMetadataPayloadDto),
    XdtoPackage(XdtoPackageMetadataPayloadDto),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct DocumentMetadataPayloadDto {
    register_records: Vec<MetadataRegisterRecordDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct MetadataRegisterRecordDto {
    target_kind: MetadataKindDto,
    target_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct EventSubscriptionMetadataPayloadDto {
    event: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct HttpServiceMetadataPayloadDto {
    root_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct WebServiceMetadataPayloadDto {
    namespace: String,
    xdto_packages: Vec<WebServiceXdtoPackageDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
enum WebServiceXdtoPackageDto {
    Repository(String),
    ExternalNamespace(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct XdtoPackageMetadataPayloadDto {
    namespace: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct MetadataMemberPayloadDto {
    synonym: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct AccessRightPayloadDto {
    row_restriction: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct DataCompositionSchemaPayloadDto {
    main: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct DataSetPayloadDto {
    kind: DataSetKindDto,
    data_source: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum DataSetKindDto {
    Query,
    Object,
    Union,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct DataCompositionFieldPayloadDto {
    data_path: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct XdtoTypePayloadDto {
    kind: XdtoTypeKindDto,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum XdtoTypeKindDto {
    Value,
    Object,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct XdtoTypeReferenceDto {
    namespace: String,
    name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct HttpServiceUrlTemplatePayloadDto {
    template: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct HttpServiceMethodPayloadDto {
    http_method: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct WebServiceOperationPayloadDto {
    returning_type: XdtoTypeReferenceDto,
    nillable: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct WebServiceParameterPayloadDto {
    value_type: XdtoTypeReferenceDto,
    nillable: Option<bool>,
    direction: Option<WebServiceParameterDirectionDto>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum WebServiceParameterDirectionDto {
    Out,
    InOut,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProvenanceDto {
    source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    location: Option<SourceLocationDto>,
    producer: String,
    origin: FactOriginDto,
    confidence: ConfidenceDto,
    resolution: ResolutionStateDto,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceLocationDto {
    path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    span: Option<SourceSpanDto>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceSpanDto {
    start: SourcePositionDto,
    end: SourcePositionDto,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SourcePositionDto {
    line: u32,
    column: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum FactOriginDto {
    Declared,
    Parsed,
    Resolved,
    Derived,
    External,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ConfidenceDto {
    Exact,
    High,
    Medium,
    Low,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ResolutionStateDto {
    NotApplicable,
    Unresolved,
    Partial,
    Ambiguous,
    Resolved,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct DiagnosticDto {
    code: DiagnosticCodeDto,
    severity: DiagnosticSeverityDto,
    kind: DiagnosticKindDto,
    message: String,
    reference: ReferenceDto,
    source_node: Option<String>,
    expected_kinds: Vec<NodeKindDto>,
    actual_kind: Option<NodeKindDto>,
    candidates: Vec<String>,
    provenance: Vec<ProvenanceDto>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum DiagnosticSeverityDto {
    Warning,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum DiagnosticCodeDto {
    QueryLanguageMalformedSyntax,
    QueryLanguageUnsupportedStructure,
    QueryLanguageUnsupportedPersistentNamespace,
    QueryLanguageVirtualTableSource,
    QueryLanguageTemporaryTableSource,
    QueryLanguageExternalOrParameterDataSource,
    DataCompositionNestedDataSetDeferred,
    DataCompositionFieldFolderDeferred,
    DataCompositionUnsupportedDataSetType,
    DataCompositionUnsupportedFieldType,
    ReferenceMalformedFormat,
    ReferenceUnsupportedPrefix,
    ReferenceUnresolved,
    ReferenceAmbiguous,
    ReferenceIncompatibleKind,
    ReferenceInvalidOwner,
    DuplicateSemanticEdgeRequest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum DiagnosticKindDto {
    QueryLanguageMalformedSyntax,
    QueryLanguageUnsupportedStructure,
    QueryLanguageUnsupportedPersistentNamespace,
    QueryLanguageVirtualTableSource,
    QueryLanguageTemporaryTableSource,
    QueryLanguageExternalOrParameterDataSource,
    DataCompositionNestedDataSetDeferred,
    DataCompositionFieldFolderDeferred,
    DataCompositionUnsupportedDataSetType,
    DataCompositionUnsupportedFieldType,
    MalformedReferenceFormat,
    UnsupportedReferencePrefix,
    UnresolvedTarget,
    AmbiguousTarget,
    IncompatibleTargetKind,
    InvalidOwnerReference,
    DuplicateSemanticEdgeRequest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ReferenceDto {
    Raw { value: String },
    NodeId { value: String },
    Name { value: String },
    Child { owner: String, name: String },
    Owner { child: String },
    OwnedChild { owner: String, child: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReferenceRequestDto {
    source_node: String,
    category: ReferenceCategoryDto,
    reference: ReferenceDto,
    expected_kinds: Vec<NodeKindDto>,
    candidates: Vec<String>,
    state: ResolutionStateDto,
    outcome: ReferenceRequestOutcomeDto,
    provenance: Vec<ProvenanceDto>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ReferenceCategoryDto {
    MetadataType,
    Callable,
    QuerySource,
    WriteTarget,
    ProtectedResource,
    SubsystemMember,
    ExtensionTarget,
    XdtoPackage,
    XdtoType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ReferenceRequestOutcomeDto {
    Collected,
    Resolved,
    MissingTarget,
    PartialWorkspace,
    AmbiguousTarget,
    IncompatibleTargetKind,
    InvalidOwnerReference,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReferenceStatisticsDto {
    total: u64,
    malformed_format: u64,
    unsupported_prefix: u64,
    resolved: u64,
    unresolved: u64,
    ambiguous: u64,
    incompatible_target_kind: u64,
    invalid_owner_reference: u64,
    duplicate_edge_request: u64,
    with_provenance: u64,
    without_provenance: u64,
}

fn content_checksum(
    source: &WorkspaceCacheSource,
    workspace: &WorkspaceDto,
) -> Result<String, WorkspaceCacheCodecError> {
    let bytes =
        serde_json::to_vec(&ContentDto { source, workspace }).map_err(serde_encode_error)?;
    let hash = bytes.iter().fold(FNV_OFFSET_BASIS, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(FNV_PRIME)
    });
    Ok(format!("fnv1a64:{hash:016x}"))
}

#[allow(clippy::needless_pass_by_value)] // Used directly as a `Result::map_err` callback.
fn serde_encode_error(error: serde_json::Error) -> WorkspaceCacheCodecError {
    WorkspaceCacheCodecError::new(
        WorkspaceCacheCodecErrorKind::Invalid,
        format!("workspace cache encoding failed: {error}"),
    )
}

#[allow(clippy::needless_pass_by_value)] // Used directly as a `Result::map_err` callback.
fn serde_decode_error(error: serde_json::Error) -> WorkspaceCacheCodecError {
    let message = error.to_string();
    let kind = if message.contains("missing field") {
        WorkspaceCacheCodecErrorKind::Partial
    } else if message.contains("duplicate field") {
        WorkspaceCacheCodecErrorKind::Duplicate
    } else if message.contains("unknown field") || message.contains("unknown variant") {
        WorkspaceCacheCodecErrorKind::Unsupported
    } else {
        WorkspaceCacheCodecErrorKind::Malformed
    };
    WorkspaceCacheCodecError::new(kind, format!("workspace cache decoding failed: {message}"))
}

fn invalid(message: impl Into<String>) -> WorkspaceCacheCodecError {
    WorkspaceCacheCodecError::new(WorkspaceCacheCodecErrorKind::Invalid, message)
}

fn inconsistent(message: impl Into<String>) -> WorkspaceCacheCodecError {
    WorkspaceCacheCodecError::new(WorkspaceCacheCodecErrorKind::Inconsistent, message)
}

fn validate_source(source: &WorkspaceCacheSource) -> Result<(), WorkspaceCacheCodecError> {
    let mut previous: Option<PathBuf> = None;
    for entry in &source.entries {
        validate_components(&entry.path)?;
        let path = components_path(&entry.path);
        if previous.as_ref().is_some_and(|previous| previous >= &path) {
            return Err(inconsistent(
                "workspace cache source entries are not in unique canonical order",
            ));
        }
        previous = Some(path);
        match (entry.kind, entry.bytes.is_some()) {
            (WorkspaceCacheSourceEntryKind::RegularFile, true)
            | (
                WorkspaceCacheSourceEntryKind::Directory | WorkspaceCacheSourceEntryKind::Other,
                false,
            ) => {}
            _ => {
                return Err(invalid(
                    "workspace cache source entry bytes contradict its kind",
                ));
            }
        }
    }
    Ok(())
}

fn components_path(components: &[String]) -> PathBuf {
    let mut path = PathBuf::new();
    for component in components {
        path.push(component);
    }
    path
}

fn validate_components(components: &[String]) -> Result<(), WorkspaceCacheCodecError> {
    if components.is_empty() {
        return Err(invalid("workspace cache relative path is empty"));
    }
    validate_root_components(components)
}

fn validate_root_components(components: &[String]) -> Result<(), WorkspaceCacheCodecError> {
    for value in components {
        let mut native = Path::new(value).components();
        if value.is_empty()
            || !matches!(native.next(), Some(Component::Normal(_)))
            || native.next().is_some()
        {
            return Err(invalid(
                "workspace cache relative path component is invalid",
            ));
        }
    }
    Ok(())
}

fn relative_components(root: &Path, path: &Path) -> Result<Vec<String>, WorkspaceCacheCodecError> {
    let relative = path
        .strip_prefix(root)
        .map_err(|_| invalid("workspace cache configuration root is outside the Workspace"))?;
    let components = relative
        .components()
        .map(|component| match component {
            Component::Normal(value) => value
                .to_str()
                .map(ToOwned::to_owned)
                .ok_or_else(|| invalid("workspace cache path is not UTF-8")),
            _ => Err(invalid(
                "workspace cache relative root component is invalid",
            )),
        })
        .collect::<Result<Vec<_>, _>>()?;
    validate_root_components(&components)?;
    Ok(components)
}

fn joined_path(root: &Path, components: &[String]) -> Result<PathBuf, WorkspaceCacheCodecError> {
    validate_root_components(components)?;
    Ok(root.join(components_path(components)))
}

fn entity_id(value: String) -> Result<EntityId, WorkspaceCacheCodecError> {
    EntityId::new(value).map_err(|_| invalid("workspace cache contains an invalid entity ID"))
}

fn entity_name(value: String) -> Result<EntityName, WorkspaceCacheCodecError> {
    EntityName::new(value).map_err(|_| invalid("workspace cache contains an invalid entity name"))
}

macro_rules! bidirectional_enum {
    ($dto:ty, $domain:ty, [$($variant:ident),+ $(,)?]) => {
        impl From<$domain> for $dto {
            fn from(value: $domain) -> Self {
                match value {
                    $(<$domain>::$variant => <$dto>::$variant,)+
                }
            }
        }

        impl From<$dto> for $domain {
            fn from(value: $dto) -> Self {
                match value {
                    $(<$dto>::$variant => <$domain>::$variant,)+
                }
            }
        }
    };
}

bidirectional_enum!(
    MetadataKindDto,
    MetadataKind,
    [
        Configuration,
        Subsystem,
        Catalog,
        Document,
        Enumeration,
        CommonModule,
        Report,
        DataProcessor,
        InformationRegister,
        AccumulationRegister,
        AccountingRegister,
        CalculationRegister,
        BusinessProcess,
        Task,
        Role,
        CommonForm,
        Form,
        Command,
        Template,
        HttpService,
        WebService,
        XdtoPackage,
        EventSubscription,
        Unknown,
    ]
);
bidirectional_enum!(
    EdgeKindDto,
    EdgeKind,
    [
        Contains, Calls, References, Reads, Writes, Grants, Includes, Extends, DependsOn, Opens,
        Triggers,
    ]
);
bidirectional_enum!(DataSetKindDto, DataSetKind, [Query, Object, Union]);
bidirectional_enum!(XdtoTypeKindDto, XdtoTypeKind, [Value, Object]);
bidirectional_enum!(
    WebServiceParameterDirectionDto,
    WebServiceParameterDirection,
    [Out, InOut]
);
bidirectional_enum!(
    FactOriginDto,
    FactOrigin,
    [Declared, Parsed, Resolved, Derived, External]
);
bidirectional_enum!(
    ConfidenceDto,
    Confidence,
    [Exact, High, Medium, Low, Unknown]
);
bidirectional_enum!(
    ResolutionStateDto,
    ResolutionState,
    [NotApplicable, Unresolved, Partial, Ambiguous, Resolved]
);
bidirectional_enum!(
    DiagnosticSeverityDto,
    SemanticDiagnosticSeverity,
    [Warning, Error]
);
bidirectional_enum!(
    DiagnosticCodeDto,
    SemanticDiagnosticCode,
    [
        QueryLanguageMalformedSyntax,
        QueryLanguageUnsupportedStructure,
        QueryLanguageUnsupportedPersistentNamespace,
        QueryLanguageVirtualTableSource,
        QueryLanguageTemporaryTableSource,
        QueryLanguageExternalOrParameterDataSource,
        DataCompositionNestedDataSetDeferred,
        DataCompositionFieldFolderDeferred,
        DataCompositionUnsupportedDataSetType,
        DataCompositionUnsupportedFieldType,
        ReferenceMalformedFormat,
        ReferenceUnsupportedPrefix,
        ReferenceUnresolved,
        ReferenceAmbiguous,
        ReferenceIncompatibleKind,
        ReferenceInvalidOwner,
        DuplicateSemanticEdgeRequest,
    ]
);
bidirectional_enum!(
    DiagnosticKindDto,
    SemanticDiagnosticKind,
    [
        QueryLanguageMalformedSyntax,
        QueryLanguageUnsupportedStructure,
        QueryLanguageUnsupportedPersistentNamespace,
        QueryLanguageVirtualTableSource,
        QueryLanguageTemporaryTableSource,
        QueryLanguageExternalOrParameterDataSource,
        DataCompositionNestedDataSetDeferred,
        DataCompositionFieldFolderDeferred,
        DataCompositionUnsupportedDataSetType,
        DataCompositionUnsupportedFieldType,
        MalformedReferenceFormat,
        UnsupportedReferencePrefix,
        UnresolvedTarget,
        AmbiguousTarget,
        IncompatibleTargetKind,
        InvalidOwnerReference,
        DuplicateSemanticEdgeRequest,
    ]
);
bidirectional_enum!(
    ReferenceCategoryDto,
    SemanticReferenceCategory,
    [
        MetadataType,
        Callable,
        QuerySource,
        WriteTarget,
        ProtectedResource,
        SubsystemMember,
        ExtensionTarget,
        XdtoPackage,
        XdtoType,
    ]
);
bidirectional_enum!(
    ReferenceRequestOutcomeDto,
    SemanticReferenceRequestOutcome,
    [
        Collected,
        Resolved,
        MissingTarget,
        PartialWorkspace,
        AmbiguousTarget,
        IncompatibleTargetKind,
        InvalidOwnerReference,
    ]
);

impl From<NodeKind> for NodeKindDto {
    fn from(value: NodeKind) -> Self {
        match value {
            NodeKind::Metadata(kind) => Self::Metadata(kind.into()),
            NodeKind::Module => Self::Module,
            NodeKind::Procedure => Self::Procedure,
            NodeKind::Function => Self::Function,
            NodeKind::Query => Self::Query,
            NodeKind::DataCompositionSchema => Self::DataCompositionSchema,
            NodeKind::DataSet => Self::DataSet,
            NodeKind::DataCompositionField => Self::DataCompositionField,
            NodeKind::XdtoType => Self::XdtoType,
            NodeKind::HttpServiceUrlTemplate => Self::HttpServiceUrlTemplate,
            NodeKind::HttpServiceMethod => Self::HttpServiceMethod,
            NodeKind::WebServiceOperation => Self::WebServiceOperation,
            NodeKind::WebServiceParameter => Self::WebServiceParameter,
            NodeKind::Form => Self::Form,
            NodeKind::Command => Self::Command,
            NodeKind::Attribute => Self::Attribute,
            NodeKind::StandardAttribute => Self::StandardAttribute,
            NodeKind::TabularSection => Self::TabularSection,
            NodeKind::Dimension => Self::Dimension,
            NodeKind::Resource => Self::Resource,
            NodeKind::Measure => Self::Measure,
            NodeKind::Role => Self::Role,
            NodeKind::AccessRight => Self::AccessRight,
            NodeKind::Subsystem => Self::Subsystem,
            NodeKind::Unknown => Self::Unknown,
        }
    }
}

impl From<NodeKindDto> for NodeKind {
    fn from(value: NodeKindDto) -> Self {
        match value {
            NodeKindDto::Metadata(kind) => Self::Metadata(kind.into()),
            NodeKindDto::Module => Self::Module,
            NodeKindDto::Procedure => Self::Procedure,
            NodeKindDto::Function => Self::Function,
            NodeKindDto::Query => Self::Query,
            NodeKindDto::DataCompositionSchema => Self::DataCompositionSchema,
            NodeKindDto::DataSet => Self::DataSet,
            NodeKindDto::DataCompositionField => Self::DataCompositionField,
            NodeKindDto::XdtoType => Self::XdtoType,
            NodeKindDto::HttpServiceUrlTemplate => Self::HttpServiceUrlTemplate,
            NodeKindDto::HttpServiceMethod => Self::HttpServiceMethod,
            NodeKindDto::WebServiceOperation => Self::WebServiceOperation,
            NodeKindDto::WebServiceParameter => Self::WebServiceParameter,
            NodeKindDto::Form => Self::Form,
            NodeKindDto::Command => Self::Command,
            NodeKindDto::Attribute => Self::Attribute,
            NodeKindDto::StandardAttribute => Self::StandardAttribute,
            NodeKindDto::TabularSection => Self::TabularSection,
            NodeKindDto::Dimension => Self::Dimension,
            NodeKindDto::Resource => Self::Resource,
            NodeKindDto::Measure => Self::Measure,
            NodeKindDto::Role => Self::Role,
            NodeKindDto::AccessRight => Self::AccessRight,
            NodeKindDto::Subsystem => Self::Subsystem,
            NodeKindDto::Unknown => Self::Unknown,
        }
    }
}

impl From<&MetadataPayload> for MetadataPayloadDto {
    fn from(value: &MetadataPayload) -> Self {
        Self {
            synonym: value.common().synonym().map(ToOwned::to_owned),
            specific: value.specific().map(MetadataSpecificPayloadDto::from),
        }
    }
}

impl TryFrom<MetadataPayloadDto> for MetadataPayload {
    type Error = WorkspaceCacheCodecError;

    fn try_from(value: MetadataPayloadDto) -> Result<Self, Self::Error> {
        Ok(Self::new(
            CommonMetadataPayload::new(value.synonym),
            value
                .specific
                .map(MetadataSpecificPayload::try_from)
                .transpose()?,
        ))
    }
}

impl From<&MetadataSpecificPayload> for MetadataSpecificPayloadDto {
    fn from(value: &MetadataSpecificPayload) -> Self {
        match value {
            MetadataSpecificPayload::Document(payload) => {
                Self::Document(DocumentMetadataPayloadDto {
                    register_records: payload
                        .register_records()
                        .iter()
                        .map(|record| MetadataRegisterRecordDto {
                            target_kind: record.target_kind().into(),
                            target_name: record.target_name().as_str().to_owned(),
                        })
                        .collect(),
                })
            }
            MetadataSpecificPayload::EventSubscription(payload) => {
                Self::EventSubscription(EventSubscriptionMetadataPayloadDto {
                    event: payload.event().as_str().to_owned(),
                })
            }
            MetadataSpecificPayload::HttpService(payload) => {
                Self::HttpService(HttpServiceMetadataPayloadDto {
                    root_url: payload.root_url().to_owned(),
                })
            }
            MetadataSpecificPayload::WebService(payload) => {
                Self::WebService(WebServiceMetadataPayloadDto {
                    namespace: payload.namespace().to_owned(),
                    xdto_packages: payload
                        .xdto_packages()
                        .iter()
                        .map(|package| match package {
                            WebServiceXdtoPackage::Repository(name) => {
                                WebServiceXdtoPackageDto::Repository(name.as_str().to_owned())
                            }
                            WebServiceXdtoPackage::ExternalNamespace(namespace) => {
                                WebServiceXdtoPackageDto::ExternalNamespace(namespace.clone())
                            }
                        })
                        .collect(),
                })
            }
            MetadataSpecificPayload::XdtoPackage(payload) => {
                Self::XdtoPackage(XdtoPackageMetadataPayloadDto {
                    namespace: payload.namespace().to_owned(),
                })
            }
        }
    }
}

impl TryFrom<MetadataSpecificPayloadDto> for MetadataSpecificPayload {
    type Error = WorkspaceCacheCodecError;

    fn try_from(value: MetadataSpecificPayloadDto) -> Result<Self, Self::Error> {
        Ok(match value {
            MetadataSpecificPayloadDto::Document(payload) => {
                Self::Document(DocumentMetadataPayload::new(
                    payload
                        .register_records
                        .into_iter()
                        .map(|record| {
                            Ok(MetadataRegisterRecord::new(
                                record.target_kind.into(),
                                entity_name(record.target_name)?,
                            ))
                        })
                        .collect::<Result<Vec<_>, WorkspaceCacheCodecError>>()?,
                ))
            }
            MetadataSpecificPayloadDto::EventSubscription(payload) => Self::EventSubscription(
                EventSubscriptionMetadataPayload::new(entity_name(payload.event)?),
            ),
            MetadataSpecificPayloadDto::HttpService(payload) => {
                Self::HttpService(HttpServiceMetadataPayload::new(payload.root_url))
            }
            MetadataSpecificPayloadDto::WebService(payload) => {
                Self::WebService(WebServiceMetadataPayload::new(
                    payload.namespace,
                    payload
                        .xdto_packages
                        .into_iter()
                        .map(|package| match package {
                            WebServiceXdtoPackageDto::Repository(name) => {
                                entity_name(name).map(WebServiceXdtoPackage::Repository)
                            }
                            WebServiceXdtoPackageDto::ExternalNamespace(namespace) => {
                                Ok(WebServiceXdtoPackage::ExternalNamespace(namespace))
                            }
                        })
                        .collect::<Result<Vec<_>, WorkspaceCacheCodecError>>()?,
                ))
            }
            MetadataSpecificPayloadDto::XdtoPackage(payload) => {
                Self::XdtoPackage(XdtoPackageMetadataPayload::new(payload.namespace))
            }
        })
    }
}

impl From<&GraphNodePayload> for NodePayloadDto {
    fn from(value: &GraphNodePayload) -> Self {
        match value {
            GraphNodePayload::None => Self::None,
            GraphNodePayload::Metadata(payload) => Self::Metadata(payload.into()),
            GraphNodePayload::MetadataMember(payload) => {
                Self::MetadataMember(MetadataMemberPayloadDto {
                    synonym: payload.synonym().map(ToOwned::to_owned),
                })
            }
            GraphNodePayload::AccessRight(payload) => Self::AccessRight(AccessRightPayloadDto {
                row_restriction: payload
                    .row_restriction()
                    .map(|restriction| restriction.condition().to_owned()),
            }),
            GraphNodePayload::DataCompositionSchema(payload) => {
                Self::DataCompositionSchema(DataCompositionSchemaPayloadDto {
                    main: payload.is_main(),
                })
            }
            GraphNodePayload::DataSet(payload) => Self::DataSet(DataSetPayloadDto {
                kind: payload.kind().into(),
                data_source: payload.data_source().map(|name| name.as_str().to_owned()),
            }),
            GraphNodePayload::DataCompositionField(payload) => {
                Self::DataCompositionField(DataCompositionFieldPayloadDto {
                    data_path: payload.data_path().as_str().to_owned(),
                })
            }
            GraphNodePayload::XdtoType(payload) => Self::XdtoType(XdtoTypePayloadDto {
                kind: payload.kind().into(),
            }),
            GraphNodePayload::HttpServiceUrlTemplate(payload) => {
                Self::HttpServiceUrlTemplate(HttpServiceUrlTemplatePayloadDto {
                    template: payload.template().to_owned(),
                })
            }
            GraphNodePayload::HttpServiceMethod(payload) => {
                Self::HttpServiceMethod(HttpServiceMethodPayloadDto {
                    http_method: payload
                        .http_method()
                        .map(|method| method.as_str().to_owned()),
                })
            }
            GraphNodePayload::WebServiceOperation(payload) => {
                Self::WebServiceOperation(WebServiceOperationPayloadDto {
                    returning_type: payload.returning_type().into(),
                    nillable: payload.nillable(),
                })
            }
            GraphNodePayload::WebServiceParameter(payload) => {
                Self::WebServiceParameter(WebServiceParameterPayloadDto {
                    value_type: payload.value_type().into(),
                    nillable: payload.nillable(),
                    direction: payload.direction().map(Into::into),
                })
            }
        }
    }
}

impl From<&XdtoTypeReference> for XdtoTypeReferenceDto {
    fn from(value: &XdtoTypeReference) -> Self {
        Self {
            namespace: value.namespace().to_owned(),
            name: value.name().as_str().to_owned(),
        }
    }
}

impl TryFrom<XdtoTypeReferenceDto> for XdtoTypeReference {
    type Error = WorkspaceCacheCodecError;

    fn try_from(value: XdtoTypeReferenceDto) -> Result<Self, Self::Error> {
        Ok(Self::new(value.namespace, entity_name(value.name)?))
    }
}

impl TryFrom<NodePayloadDto> for GraphNodePayload {
    type Error = WorkspaceCacheCodecError;

    fn try_from(value: NodePayloadDto) -> Result<Self, Self::Error> {
        Ok(match value {
            NodePayloadDto::None => Self::None,
            NodePayloadDto::Metadata(payload) => Self::Metadata(payload.try_into()?),
            NodePayloadDto::MetadataMember(payload) => {
                Self::MetadataMember(MetadataMemberPayload::new(payload.synonym))
            }
            NodePayloadDto::AccessRight(payload) => {
                let restriction = payload
                    .row_restriction
                    .map(AccessRightRowRestriction::new)
                    .transpose()
                    .map_err(|error| invalid(format!("invalid cached row restriction: {error}")))?;
                Self::AccessRight(AccessRightPayload::new(restriction))
            }
            NodePayloadDto::DataCompositionSchema(payload) => {
                Self::DataCompositionSchema(DataCompositionSchemaPayload::new(payload.main))
            }
            NodePayloadDto::DataSet(payload) => Self::DataSet(
                DataSetPayload::new(
                    payload.kind.into(),
                    payload.data_source.map(entity_name).transpose()?,
                )
                .map_err(|error| invalid(format!("invalid cached Data Set payload: {error}")))?,
            ),
            NodePayloadDto::DataCompositionField(payload) => Self::DataCompositionField(
                DataCompositionFieldPayload::new(entity_name(payload.data_path)?),
            ),
            NodePayloadDto::XdtoType(payload) => {
                Self::XdtoType(XdtoTypePayload::new(payload.kind.into()))
            }
            NodePayloadDto::HttpServiceUrlTemplate(payload) => {
                Self::HttpServiceUrlTemplate(HttpServiceUrlTemplatePayload::new(payload.template))
            }
            NodePayloadDto::HttpServiceMethod(payload) => Self::HttpServiceMethod(
                HttpServiceMethodPayload::new(payload.http_method.map(entity_name).transpose()?),
            ),
            NodePayloadDto::WebServiceOperation(payload) => {
                Self::WebServiceOperation(WebServiceOperationPayload::new(
                    payload.returning_type.try_into()?,
                    payload.nillable,
                ))
            }
            NodePayloadDto::WebServiceParameter(payload) => {
                Self::WebServiceParameter(WebServiceParameterPayload::new(
                    payload.value_type.try_into()?,
                    payload.nillable,
                    payload.direction.map(Into::into),
                ))
            }
        })
    }
}

impl From<&Provenance> for ProvenanceDto {
    fn from(value: &Provenance) -> Self {
        Self {
            source: value.source().map(|source| source.as_str().to_owned()),
            location: value.location().map(Into::into),
            producer: value.producer().as_str().to_owned(),
            origin: value.origin().into(),
            confidence: value.confidence().into(),
            resolution: value.resolution().into(),
        }
    }
}

impl TryFrom<ProvenanceDto> for Provenance {
    type Error = WorkspaceCacheCodecError;

    fn try_from(value: ProvenanceDto) -> Result<Self, Self::Error> {
        Ok(Self::new_with_location(
            value.source.map(entity_id).transpose()?,
            value.location.map(SourceLocation::try_from).transpose()?,
            ProducerId::new(value.producer),
            value.origin.into(),
            value.confidence.into(),
            value.resolution.into(),
        ))
    }
}

impl From<&SourceLocation> for SourceLocationDto {
    fn from(value: &SourceLocation) -> Self {
        Self {
            path: value.path().as_str().to_owned(),
            span: value.span().map(Into::into),
        }
    }
}

impl TryFrom<SourceLocationDto> for SourceLocation {
    type Error = WorkspaceCacheCodecError;

    fn try_from(value: SourceLocationDto) -> Result<Self, Self::Error> {
        let path = SourcePath::new(value.path)
            .map_err(|_| invalid("workspace cache source path is invalid"))?;
        let span = value.span.map(SourceSpan::try_from).transpose()?;
        Ok(Self::new(path, span))
    }
}

impl From<SourceSpan> for SourceSpanDto {
    fn from(value: SourceSpan) -> Self {
        Self {
            start: value.start().into(),
            end: value.end().into(),
        }
    }
}

impl TryFrom<SourceSpanDto> for SourceSpan {
    type Error = WorkspaceCacheCodecError;

    fn try_from(value: SourceSpanDto) -> Result<Self, Self::Error> {
        Self::new(value.start.try_into()?, value.end.try_into()?)
            .map_err(|_| invalid("workspace cache source span is invalid"))
    }
}

impl From<SourcePosition> for SourcePositionDto {
    fn from(value: SourcePosition) -> Self {
        Self {
            line: value.line(),
            column: value.column(),
        }
    }
}

impl TryFrom<SourcePositionDto> for SourcePosition {
    type Error = WorkspaceCacheCodecError;

    fn try_from(value: SourcePositionDto) -> Result<Self, Self::Error> {
        Self::new(value.line, value.column)
            .map_err(|_| invalid("workspace cache source position is invalid"))
    }
}

impl From<&SemanticReference> for ReferenceDto {
    fn from(value: &SemanticReference) -> Self {
        match value {
            SemanticReference::Raw(value) => Self::Raw {
                value: value.clone(),
            },
            SemanticReference::NodeId(value) => Self::NodeId {
                value: value.clone(),
            },
            SemanticReference::Name(value) => Self::Name {
                value: value.as_str().to_owned(),
            },
            SemanticReference::Child { owner, name } => Self::Child {
                owner: owner.as_str().to_owned(),
                name: name.as_str().to_owned(),
            },
            SemanticReference::Owner { child } => Self::Owner {
                child: child.as_str().to_owned(),
            },
            SemanticReference::OwnedChild { owner, child } => Self::OwnedChild {
                owner: owner.as_str().to_owned(),
                child: child.as_str().to_owned(),
            },
        }
    }
}

impl TryFrom<ReferenceDto> for SemanticReference {
    type Error = WorkspaceCacheCodecError;

    fn try_from(value: ReferenceDto) -> Result<Self, Self::Error> {
        Ok(match value {
            ReferenceDto::Raw { value } => Self::Raw(value),
            ReferenceDto::NodeId { value } => Self::NodeId(value),
            ReferenceDto::Name { value } => Self::Name(entity_name(value)?),
            ReferenceDto::Child { owner, name } => Self::Child {
                owner: entity_id(owner)?,
                name: entity_name(name)?,
            },
            ReferenceDto::Owner { child } => Self::Owner {
                child: entity_id(child)?,
            },
            ReferenceDto::OwnedChild { owner, child } => Self::OwnedChild {
                owner: entity_id(owner)?,
                child: entity_id(child)?,
            },
        })
    }
}

impl From<&GraphNode> for NodeDto {
    fn from(value: &GraphNode) -> Self {
        Self {
            id: value.id().as_str().to_owned(),
            name: value.name().as_str().to_owned(),
            kind: value.kind().into(),
            payload: value.payload().into(),
            provenance: value.provenance().iter().map(Into::into).collect(),
        }
    }
}

impl TryFrom<NodeDto> for GraphNode {
    type Error = WorkspaceCacheCodecError;

    fn try_from(value: NodeDto) -> Result<Self, Self::Error> {
        let kind = value.kind.into();
        Self::new_with_payload_and_provenance(
            entity_id(value.id)?,
            entity_name(value.name)?,
            kind,
            value.payload.try_into()?,
            value
                .provenance
                .into_iter()
                .map(Provenance::try_from)
                .collect::<Result<Vec<_>, _>>()?,
        )
        .map_err(|error| invalid(format!("invalid cached graph node payload: {error}")))
    }
}

impl From<&GraphEdge> for EdgeDto {
    fn from(value: &GraphEdge) -> Self {
        Self {
            source: value.source().as_str().to_owned(),
            target: value.target().as_str().to_owned(),
            kind: value.kind().into(),
            provenance: value.provenance().iter().map(Into::into).collect(),
        }
    }
}

impl TryFrom<EdgeDto> for GraphEdge {
    type Error = WorkspaceCacheCodecError;

    fn try_from(value: EdgeDto) -> Result<Self, Self::Error> {
        Ok(Self::new_with_provenance(
            entity_id(value.source)?,
            entity_id(value.target)?,
            value.kind.into(),
            value
                .provenance
                .into_iter()
                .map(Provenance::try_from)
                .collect::<Result<Vec<_>, _>>()?,
        ))
    }
}

impl From<&SemanticDiagnostic> for DiagnosticDto {
    fn from(value: &SemanticDiagnostic) -> Self {
        Self {
            code: value.code().into(),
            severity: value.severity().into(),
            kind: value.kind().into(),
            message: value.message().to_owned(),
            reference: value.reference().into(),
            source_node: value.source_node().map(|id| id.as_str().to_owned()),
            expected_kinds: value
                .expected_kinds()
                .iter()
                .copied()
                .map(Into::into)
                .collect(),
            actual_kind: value.actual_kind().map(Into::into),
            candidates: value
                .candidates()
                .iter()
                .map(|id| id.as_str().to_owned())
                .collect(),
            provenance: value.provenance().iter().map(Into::into).collect(),
        }
    }
}

impl TryFrom<DiagnosticDto> for SemanticDiagnostic {
    type Error = WorkspaceCacheCodecError;

    fn try_from(value: DiagnosticDto) -> Result<Self, Self::Error> {
        let mut diagnostic = Self::new(
            value.code.into(),
            value.severity.into(),
            value.kind.into(),
            value.message,
            value.reference.try_into()?,
        )
        .with_expected_kinds(value.expected_kinds.into_iter().map(Into::into).collect())
        .with_candidates(
            value
                .candidates
                .into_iter()
                .map(entity_id)
                .collect::<Result<Vec<_>, _>>()?,
        )
        .with_provenance(
            value
                .provenance
                .into_iter()
                .map(Provenance::try_from)
                .collect::<Result<Vec<_>, _>>()?,
        );
        if let Some(source_node) = value.source_node {
            diagnostic = diagnostic.with_source_node(entity_id(source_node)?);
        }
        if let Some(actual_kind) = value.actual_kind {
            diagnostic = diagnostic.with_actual_kind(actual_kind.into());
        }
        Ok(diagnostic)
    }
}

impl From<&SemanticReferenceRequest> for ReferenceRequestDto {
    fn from(value: &SemanticReferenceRequest) -> Self {
        Self {
            source_node: value.source_node().as_str().to_owned(),
            category: value.category().into(),
            reference: value.reference().into(),
            expected_kinds: value
                .expected_kinds()
                .iter()
                .copied()
                .map(Into::into)
                .collect(),
            candidates: value
                .candidates()
                .iter()
                .map(|id| id.as_str().to_owned())
                .collect(),
            state: value.state().into(),
            outcome: value.outcome().into(),
            provenance: value.provenance().iter().map(Into::into).collect(),
        }
    }
}

impl TryFrom<ReferenceRequestDto> for SemanticReferenceRequest {
    type Error = WorkspaceCacheCodecError;

    fn try_from(value: ReferenceRequestDto) -> Result<Self, Self::Error> {
        Self::reconstruct_terminal(
            entity_id(value.source_node)?,
            value.category.into(),
            value.reference.try_into()?,
            value.expected_kinds.into_iter().map(Into::into),
            value
                .candidates
                .into_iter()
                .map(entity_id)
                .collect::<Result<Vec<_>, _>>()?,
            value.state.into(),
            value.outcome.into(),
            value
                .provenance
                .into_iter()
                .map(Provenance::try_from)
                .collect::<Result<Vec<_>, _>>()?,
        )
        .map_err(|error| invalid(format!("invalid cached reference request: {error}")))
    }
}

impl From<SemanticReferenceStatistics> for ReferenceStatisticsDto {
    fn from(value: SemanticReferenceStatistics) -> Self {
        Self {
            total: value.total() as u64,
            malformed_format: value.malformed_format() as u64,
            unsupported_prefix: value.unsupported_prefix() as u64,
            resolved: value.resolved() as u64,
            unresolved: value.unresolved() as u64,
            ambiguous: value.ambiguous() as u64,
            incompatible_target_kind: value.incompatible_target_kind() as u64,
            invalid_owner_reference: value.invalid_owner_reference() as u64,
            duplicate_edge_request: value.duplicate_edge_request() as u64,
            with_provenance: value.with_provenance() as u64,
            without_provenance: value.without_provenance() as u64,
        }
    }
}

impl ReferenceStatisticsDto {
    fn checked_counts(self) -> Result<[usize; 11], WorkspaceCacheCodecError> {
        let values = [
            self.total,
            self.malformed_format,
            self.unsupported_prefix,
            self.resolved,
            self.unresolved,
            self.ambiguous,
            self.incompatible_target_kind,
            self.invalid_owner_reference,
            self.duplicate_edge_request,
            self.with_provenance,
            self.without_provenance,
        ];
        values
            .map(|value| {
                usize::try_from(value)
                    .map_err(|_| invalid("workspace cache reference counter exceeds usize"))
            })
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?
            .try_into()
            .map_err(|_| invalid("workspace cache reference counter cardinality is invalid"))
    }

    fn into_statistics(self) -> Result<SemanticReferenceStatistics, WorkspaceCacheCodecError> {
        let [
            total,
            malformed,
            unsupported,
            resolved,
            unresolved,
            ambiguous,
            incompatible,
            invalid_owner,
            duplicate,
            with_provenance,
            without_provenance,
        ] = self.checked_counts()?;
        let outcomes = [
            (SemanticReferenceOutcome::MalformedFormat, malformed),
            (SemanticReferenceOutcome::UnsupportedPrefix, unsupported),
            (SemanticReferenceOutcome::Resolved, resolved),
            (SemanticReferenceOutcome::Unresolved, unresolved),
            (SemanticReferenceOutcome::Ambiguous, ambiguous),
            (
                SemanticReferenceOutcome::IncompatibleTargetKind,
                incompatible,
            ),
            (
                SemanticReferenceOutcome::InvalidOwnerReference,
                invalid_owner,
            ),
            (SemanticReferenceOutcome::DuplicateEdgeRequest, duplicate),
        ];
        if outcomes.iter().map(|(_, count)| count).sum::<usize>() != total
            || with_provenance
                .checked_add(without_provenance)
                .is_none_or(|sum| sum != total)
        {
            return Err(inconsistent(
                "workspace cache reference statistics totals are inconsistent",
            ));
        }

        let mut remaining_with_provenance = with_provenance;
        let mut statistics = SemanticReferenceStatistics::new();
        for (outcome, count) in outcomes {
            for _ in 0..count {
                let has_provenance = remaining_with_provenance > 0;
                remaining_with_provenance = remaining_with_provenance.saturating_sub(1);
                statistics.record(outcome, has_provenance);
            }
        }
        Ok(statistics)
    }

    fn checked_subtract(self, represented: Self) -> Result<Self, WorkspaceCacheCodecError> {
        macro_rules! subtract {
            ($field:ident) => {
                self.$field.checked_sub(represented.$field).ok_or_else(|| {
                    inconsistent(concat!(
                        "workspace cache total reference statistic underflows at ",
                        stringify!($field)
                    ))
                })?
            };
        }
        Ok(Self {
            total: subtract!(total),
            malformed_format: subtract!(malformed_format),
            unsupported_prefix: subtract!(unsupported_prefix),
            resolved: subtract!(resolved),
            unresolved: subtract!(unresolved),
            ambiguous: subtract!(ambiguous),
            incompatible_target_kind: subtract!(incompatible_target_kind),
            invalid_owner_reference: subtract!(invalid_owner_reference),
            duplicate_edge_request: subtract!(duplicate_edge_request),
            with_provenance: subtract!(with_provenance),
            without_provenance: subtract!(without_provenance),
        })
    }
}

impl WorkspaceDto {
    fn from_snapshot(
        source: &WorkspaceCacheSource,
        workspace_root: &Path,
        snapshot: &WorkspaceSnapshot,
    ) -> Result<Self, WorkspaceCacheCodecError> {
        let configurations = snapshot
            .configurations()
            .iter()
            .map(|configuration| {
                ConfigurationDto::from_snapshot(source, workspace_root, configuration)
            })
            .collect::<Result<Vec<_>, _>>()?;
        if snapshot
            .configurations()
            .windows(2)
            .any(|pair| pair[0].configuration_id() >= pair[1].configuration_id())
        {
            return Err(inconsistent(
                "Workspace configurations are not in unique canonical order",
            ));
        }
        Ok(Self { configurations })
    }

    fn into_snapshot(
        self,
        source: &WorkspaceCacheSource,
        workspace_root: &Path,
    ) -> Result<WorkspaceSnapshot, WorkspaceCacheCodecError> {
        let expected = self.clone();
        let mut configurations = Vec::with_capacity(self.configurations.len());
        let mut previous_id: Option<EntityId> = None;
        for configuration in self.configurations {
            let configuration = configuration.into_snapshot(source, workspace_root)?;
            if previous_id
                .as_ref()
                .is_some_and(|id| id >= configuration.configuration_id())
            {
                return Err(WorkspaceCacheCodecError::new(
                    WorkspaceCacheCodecErrorKind::Duplicate,
                    "workspace cache configurations are duplicated or not canonically ordered",
                ));
            }
            previous_id = Some(configuration.configuration_id().clone());
            configurations.push(configuration);
        }
        let snapshot = WorkspaceSnapshot::initial(workspace_root.to_path_buf(), configurations);
        let reconstructed = Self::from_snapshot(source, workspace_root, &snapshot)?;
        if reconstructed != expected {
            return Err(inconsistent(
                "workspace cache semantic content is not canonically normalized",
            ));
        }
        Ok(snapshot)
    }
}

impl SourceEvidenceDto {
    fn from_evidence(
        source: &WorkspaceCacheSource,
        evidence: &SourceEvidenceSet,
    ) -> Result<Self, WorkspaceCacheCodecError> {
        let documents = evidence
            .documents()
            .iter()
            .map(|document| SourceDocumentDto::from_document(source, document))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            configuration_id: evidence.configuration_id().as_str().to_owned(),
            documents,
        })
    }

    fn into_evidence(
        self,
        source: &WorkspaceCacheSource,
        configuration_root: &SourcePath,
        workspace_format: WorkspaceFormat,
    ) -> Result<SourceEvidenceSet, WorkspaceCacheCodecError> {
        let expected = self.clone();
        let configuration_id = entity_id(self.configuration_id)?;
        let documents = self
            .documents
            .into_iter()
            .map(|document| {
                document.into_document(
                    source,
                    configuration_root,
                    workspace_format,
                    &configuration_id,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        let evidence = SourceEvidenceSet::new(configuration_id, documents).map_err(|error| {
            invalid(format!(
                "workspace cache source evidence is invalid: {error}"
            ))
        })?;
        if Self::from_evidence(source, &evidence)? != expected {
            return Err(inconsistent(
                "workspace cache source evidence is not canonically normalized",
            ));
        }
        Ok(evidence)
    }
}

impl SourceDocumentDto {
    fn from_document(
        source: &WorkspaceCacheSource,
        document: &SourceDocument,
    ) -> Result<Self, WorkspaceCacheCodecError> {
        let path = document.path().path().as_str();
        let raw = cached_source_bytes(source, path)?;
        if raw != document.raw_content() {
            return Err(inconsistent(
                "workspace cache source document bytes contradict the source envelope",
            ));
        }
        Ok(Self {
            configuration_id: document.id().configuration_id().as_str().to_owned(),
            module_id: document.id().module_id().as_str().to_owned(),
            format: source_format_dto(document.format()),
            module_role: module_role_dto(document.module_role()),
            path: path.to_owned(),
            content_version: document.content_version().into(),
            occurrences: document
                .occurrences()
                .iter()
                .map(SourceOccurrenceDto::from)
                .collect(),
            completeness: SourceEvidenceCompletenessDto::BslCallableRenameV1,
        })
    }

    fn into_document(
        self,
        source: &WorkspaceCacheSource,
        configuration_root: &SourcePath,
        workspace_format: WorkspaceFormat,
        expected_configuration_id: &EntityId,
    ) -> Result<SourceDocument, WorkspaceCacheCodecError> {
        let document_id = SourceDocumentId::new(
            entity_id(self.configuration_id)?,
            entity_id(self.module_id)?,
        )
        .map_err(|error| invalid(format!("workspace cache document ID is invalid: {error}")))?;
        if document_id.configuration_id() != expected_configuration_id {
            return Err(inconsistent(
                "workspace cache source document Configuration is inconsistent",
            ));
        }
        let format = source_format(self.format);
        if format != workspace_source_format(workspace_format) {
            return Err(inconsistent(
                "workspace cache source document format is inconsistent",
            ));
        }
        let path = SourcePath::new(self.path)
            .map_err(|_| invalid("workspace cache source document path is invalid"))?;
        let confined =
            ConfinedSourcePath::new(path.clone(), configuration_root).map_err(|error| {
                invalid(format!(
                    "workspace cache confined source path is invalid: {error}"
                ))
            })?;
        let raw = cached_source_bytes(source, path.as_str())?.to_vec();
        let actual_version = SourceContentVersion::from_bytes(&raw);
        if self.content_version != SourceContentVersionDto::from(actual_version) {
            return Err(inconsistent(
                "workspace cache source document version is stale",
            ));
        }
        let occurrences = self
            .occurrences
            .into_iter()
            .map(|occurrence| occurrence.into_occurrence(&document_id, actual_version))
            .collect::<Result<Vec<_>, _>>()?;
        SourceDocument::new(
            document_id,
            format,
            module_role(self.module_role),
            confined,
            raw,
            occurrences,
            match self.completeness {
                SourceEvidenceCompletenessDto::BslCallableRenameV1 => {
                    SourceEvidenceCompleteness::BslCallableRenameV1
                }
            },
        )
        .map_err(|error| {
            invalid(format!(
                "workspace cache source document is invalid: {error}"
            ))
        })
    }
}

impl SourceOccurrenceDto {
    fn into_occurrence(
        self,
        document_id: &SourceDocumentId,
        actual_version: SourceContentVersion,
    ) -> Result<SourceOccurrence, WorkspaceCacheCodecError> {
        let occurrence_document_id = SourceDocumentId::new(
            entity_id(self.configuration_id)?,
            entity_id(self.module_id)?,
        )
        .map_err(|error| invalid(format!("workspace cache occurrence ID is invalid: {error}")))?;
        if &occurrence_document_id != document_id
            || self.content_version != SourceContentVersionDto::from(actual_version)
        {
            return Err(inconsistent(
                "workspace cache occurrence source precondition is inconsistent",
            ));
        }
        let range = SourceByteRange::new(self.start_byte, self.end_byte).map_err(|error| {
            invalid(format!(
                "workspace cache occurrence range is invalid: {error}"
            ))
        })?;
        SourceOccurrence::new(
            occurrence_document_id,
            actual_version,
            range,
            occurrence_kind(self.kind),
            self.token,
            self.mapped_target_id.map(entity_id).transpose()?,
            occurrence_resolution(self.resolution),
        )
        .map_err(|error| invalid(format!("workspace cache occurrence is invalid: {error}")))
    }
}

impl From<&SourceOccurrence> for SourceOccurrenceDto {
    fn from(value: &SourceOccurrence) -> Self {
        Self {
            configuration_id: value.document_id().configuration_id().as_str().to_owned(),
            module_id: value.document_id().module_id().as_str().to_owned(),
            content_version: value.content_version().into(),
            start_byte: value.range().start_byte(),
            end_byte: value.range().end_byte(),
            kind: occurrence_kind_dto(value.kind()),
            token: value.token().to_owned(),
            mapped_target_id: value
                .mapped_target_id()
                .map(|identity| identity.as_str().to_owned()),
            resolution: occurrence_resolution_dto(value.resolution()),
        }
    }
}

impl From<SourceContentVersion> for SourceContentVersionDto {
    fn from(value: SourceContentVersion) -> Self {
        Self {
            raw_byte_len: value.raw_byte_len(),
            digest: value.digest(),
        }
    }
}

fn cached_source_bytes<'source>(
    source: &'source WorkspaceCacheSource,
    path: &str,
) -> Result<&'source [u8], WorkspaceCacheCodecError> {
    let components = path.split('/').collect::<Vec<_>>();
    let entry = source
        .entries
        .iter()
        .find(|entry| {
            entry.path.len() == components.len()
                && entry
                    .path
                    .iter()
                    .zip(&components)
                    .all(|(actual, expected)| actual == expected)
        })
        .ok_or_else(|| invalid("workspace cache source document is missing from source state"))?;
    if entry.kind != WorkspaceCacheSourceEntryKind::RegularFile {
        return Err(invalid(
            "workspace cache source document is not a regular file",
        ));
    }
    entry
        .bytes
        .as_deref()
        .ok_or_else(|| invalid("workspace cache regular source document has no bytes"))
}

const fn workspace_source_format(format: WorkspaceFormat) -> SourceFormat {
    match format {
        WorkspaceFormat::DesignerXml => SourceFormat::DesignerXml,
        WorkspaceFormat::Edt | WorkspaceFormat::Extension | WorkspaceFormat::Unknown => {
            SourceFormat::Edt
        }
    }
}

const fn source_format_dto(format: SourceFormat) -> FormatDto {
    match format {
        SourceFormat::Edt => FormatDto::Edt,
        SourceFormat::DesignerXml => FormatDto::DesignerXml,
    }
}

const fn source_format(format: FormatDto) -> SourceFormat {
    match format {
        FormatDto::Edt => SourceFormat::Edt,
        FormatDto::DesignerXml => SourceFormat::DesignerXml,
    }
}

const fn module_role_dto(role: BslModuleRole) -> BslModuleRoleDto {
    match role {
        BslModuleRole::Object => BslModuleRoleDto::Object,
        BslModuleRole::Manager => BslModuleRoleDto::Manager,
        BslModuleRole::Common => BslModuleRoleDto::Common,
        BslModuleRole::Form => BslModuleRoleDto::Form,
        BslModuleRole::Command => BslModuleRoleDto::Command,
    }
}

const fn module_role(role: BslModuleRoleDto) -> BslModuleRole {
    match role {
        BslModuleRoleDto::Object => BslModuleRole::Object,
        BslModuleRoleDto::Manager => BslModuleRole::Manager,
        BslModuleRoleDto::Common => BslModuleRole::Common,
        BslModuleRoleDto::Form => BslModuleRole::Form,
        BslModuleRoleDto::Command => BslModuleRole::Command,
    }
}

const fn occurrence_kind_dto(kind: SourceOccurrenceKind) -> SourceOccurrenceKindDto {
    match kind {
        SourceOccurrenceKind::Declaration => SourceOccurrenceKindDto::Declaration,
        SourceOccurrenceKind::LocalCall => SourceOccurrenceKindDto::LocalCall,
        SourceOccurrenceKind::QualifiedCall => SourceOccurrenceKindDto::QualifiedCall,
    }
}

const fn occurrence_kind(kind: SourceOccurrenceKindDto) -> SourceOccurrenceKind {
    match kind {
        SourceOccurrenceKindDto::Declaration => SourceOccurrenceKind::Declaration,
        SourceOccurrenceKindDto::LocalCall => SourceOccurrenceKind::LocalCall,
        SourceOccurrenceKindDto::QualifiedCall => SourceOccurrenceKind::QualifiedCall,
    }
}

const fn occurrence_resolution_dto(
    resolution: SourceOccurrenceResolution,
) -> SourceOccurrenceResolutionDto {
    match resolution {
        SourceOccurrenceResolution::Unique => SourceOccurrenceResolutionDto::Unique,
        SourceOccurrenceResolution::Unresolved => SourceOccurrenceResolutionDto::Unresolved,
        SourceOccurrenceResolution::Ambiguous => SourceOccurrenceResolutionDto::Ambiguous,
        SourceOccurrenceResolution::Unsupported => SourceOccurrenceResolutionDto::Unsupported,
    }
}

const fn occurrence_resolution(
    resolution: SourceOccurrenceResolutionDto,
) -> SourceOccurrenceResolution {
    match resolution {
        SourceOccurrenceResolutionDto::Unique => SourceOccurrenceResolution::Unique,
        SourceOccurrenceResolutionDto::Unresolved => SourceOccurrenceResolution::Unresolved,
        SourceOccurrenceResolutionDto::Ambiguous => SourceOccurrenceResolution::Ambiguous,
        SourceOccurrenceResolutionDto::Unsupported => SourceOccurrenceResolution::Unsupported,
    }
}

impl ConfigurationDto {
    fn from_snapshot(
        source: &WorkspaceCacheSource,
        workspace_root: &Path,
        snapshot: &WorkspaceConfigurationSnapshot,
    ) -> Result<Self, WorkspaceCacheCodecError> {
        let format = match snapshot.format() {
            WorkspaceFormat::Edt => FormatDto::Edt,
            WorkspaceFormat::DesignerXml => FormatDto::DesignerXml,
            WorkspaceFormat::Extension | WorkspaceFormat::Unknown => {
                return Err(WorkspaceCacheCodecError::new(
                    WorkspaceCacheCodecErrorKind::Unsupported,
                    "workspace cache cannot encode an unsupported Workspace format",
                ));
            }
        };
        Ok(Self {
            root: relative_components(workspace_root, snapshot.root_path())?,
            format,
            source_evidence: SourceEvidenceDto::from_evidence(source, snapshot.source_evidence())?,
            nodes: snapshot.graph().nodes().map(NodeDto::from).collect(),
            edges: snapshot.graph().edges().map(EdgeDto::from).collect(),
            diagnostics: snapshot
                .diagnostics()
                .iter()
                .map(DiagnosticDto::from)
                .collect(),
            reference_requests: snapshot
                .reference_requests()
                .requests()
                .iter()
                .map(ReferenceRequestDto::from)
                .collect(),
            reference_statistics: snapshot.reference_statistics().into(),
        })
    }

    #[allow(clippy::too_many_lines)] // Keeps the ordered ADR-0042 validation gates together.
    fn into_snapshot(
        self,
        source: &WorkspaceCacheSource,
        workspace_root: &Path,
    ) -> Result<WorkspaceConfigurationSnapshot, WorkspaceCacheCodecError> {
        let root_path = joined_path(workspace_root, &self.root)?;
        let configuration_root = SourcePath::new(self.root.join("/"))
            .map_err(|_| invalid("workspace cache Configuration source root is invalid"))?;
        let format = match self.format {
            FormatDto::Edt => WorkspaceFormat::Edt,
            FormatDto::DesignerXml => WorkspaceFormat::DesignerXml,
        };
        let source_evidence =
            self.source_evidence
                .into_evidence(source, &configuration_root, format)?;

        let mut graph = SemanticGraph::new();
        let mut previous_node: Option<EntityId> = None;
        for node in self.nodes {
            let node = GraphNode::try_from(node)?;
            if previous_node.as_ref().is_some_and(|id| id >= node.id()) {
                return Err(WorkspaceCacheCodecError::new(
                    WorkspaceCacheCodecErrorKind::Duplicate,
                    "workspace cache graph nodes are duplicated or not canonically ordered",
                ));
            }
            previous_node = Some(node.id().clone());
            if graph.insert_node(node).is_some() {
                return Err(WorkspaceCacheCodecError::new(
                    WorkspaceCacheCodecErrorKind::Duplicate,
                    "workspace cache contains duplicate graph node IDs",
                ));
            }
        }

        let mut previous_edge: Option<GraphEdge> = None;
        for edge in self.edges {
            let edge = GraphEdge::try_from(edge)?;
            if previous_edge.as_ref().is_some_and(|value| value >= &edge) {
                return Err(WorkspaceCacheCodecError::new(
                    WorkspaceCacheCodecErrorKind::Duplicate,
                    "workspace cache graph edges are duplicated or not canonically ordered",
                ));
            }
            previous_edge = Some(edge.clone());
            let inserted = graph.insert_edge(edge).map_err(|error| {
                invalid(format!("workspace cache graph edge is invalid: {error}"))
            })?;
            if !inserted {
                return Err(WorkspaceCacheCodecError::new(
                    WorkspaceCacheCodecErrorKind::Duplicate,
                    "workspace cache contains duplicate graph edges",
                ));
            }
        }

        let diagnostics = self
            .diagnostics
            .into_iter()
            .map(SemanticDiagnostic::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        if diagnostics.windows(2).any(|pair| pair[0] > pair[1]) {
            return Err(inconsistent(
                "workspace cache diagnostics are not canonically ordered",
            ));
        }

        let requests = self
            .reference_requests
            .into_iter()
            .map(SemanticReferenceRequest::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        if requests.windows(2).any(|pair| pair[0].id() >= pair[1].id()) {
            return Err(WorkspaceCacheCodecError::new(
                WorkspaceCacheCodecErrorKind::Duplicate,
                "workspace cache reference requests are duplicated or not canonically ordered",
            ));
        }
        let ledger = SemanticReferenceRequestLedger::from_requests(requests).map_err(|error| {
            invalid(format!(
                "workspace cache request ledger is invalid: {error}"
            ))
        })?;

        let total_statistics = self.reference_statistics.into_statistics()?;
        let represented_statistics = SemanticReferenceStatistics::from_reference_requests(&ledger);
        let legacy_statistics = ReferenceStatisticsDto::from(total_statistics)
            .checked_subtract(represented_statistics.into())?
            .into_statistics()?;
        let report =
            SemanticGraphReport::from_graph_diagnostics_reference_requests_and_legacy_observations(
                &graph,
                &diagnostics,
                &ledger,
                legacy_statistics,
            );
        let validation = SemanticGraphValidator::new()
            .validate_build_result_with_reference_requests_and_report(
                &graph,
                &diagnostics,
                &ledger,
                legacy_statistics,
                &report,
            );
        if !validation.is_valid() {
            return Err(invalid(format!(
                "workspace cache complete build validation failed with {} errors",
                validation.error_count()
            )));
        }

        snapshot_from_parts(
            &root_path,
            format,
            graph,
            source_evidence,
            diagnostics,
            ledger,
            total_statistics,
            report,
            validation,
        )
        .map_err(|error| invalid(format!("workspace cache snapshot is invalid: {error}")))
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    use oneagent_analysis::refactoring::{
        NeverCancelledRefactoring, RefactoringFamily, RefactoringRequest, SourceDocument,
        SourceEvidenceSet, SourceOccurrence, SourceOccurrenceKind,
    };
    use oneagent_common::EntityName;
    use oneagent_graph::{
        AccessRightPayload, AccessRightRowRestriction, Confidence, DataCompositionFieldPayload,
        DataCompositionSchemaPayload, DataSetKind, DataSetPayload, EdgeKind, FactOrigin, GraphNode,
        GraphNodePayload, HttpServiceMethodPayload, HttpServiceUrlTemplatePayload, NodeKind,
        ResolutionState, SemanticDiagnostic, SemanticDiagnosticCode, SemanticDiagnosticKind,
        SemanticDiagnosticSeverity, SemanticGraph, SemanticGraphReport, SemanticGraphValidator,
        SemanticReference, SemanticReferenceCategory, SemanticReferenceOutcome,
        SemanticReferenceRequestLedger, SemanticReferenceRequestOutcome,
        SemanticReferenceStatistics, WebServiceOperationPayload, WebServiceParameterDirection,
        WebServiceParameterPayload, XdtoTypeKind, XdtoTypePayload, XdtoTypeReference,
    };
    use oneagent_metadata::{
        CommonMetadataPayload, DocumentMetadataPayload, EventSubscriptionMetadataPayload,
        HttpServiceMetadataPayload, MetadataKind, MetadataMemberPayload, MetadataPayload,
        MetadataRegisterRecord, MetadataSpecificPayload, WebServiceMetadataPayload,
        WebServiceXdtoPackage, XdtoPackageMetadataPayload,
    };
    use oneagent_workspace::WorkspaceFormat;
    use tempfile::tempdir;

    use super::{
        AccessRightPayloadDto, ConfidenceDto, DataSetKindDto, DiagnosticCodeDto, DiagnosticKindDto,
        EdgeKindDto, EnvelopeDto, FactOriginDto, MetadataKindDto, NodeKindDto, NodePayloadDto,
        ReferenceCategoryDto, ReferenceDto, ReferenceRequestOutcomeDto, ResolutionStateDto,
        WebServiceParameterDirectionDto, WorkspaceCacheCodec, WorkspaceCacheCodecErrorKind,
        WorkspaceCacheFailurePoint, WorkspaceCacheLoadOutcome, WorkspaceCacheSource,
        WorkspaceCacheSourceEntry, WorkspaceCacheSourceEntryKind, WorkspaceCacheStore,
        WorkspaceCacheWriteOutcome, WorkspaceDto, XdtoTypeKindDto, content_checksum,
    };
    use crate::workspace::change::WorkspaceFileState;
    use crate::workspace::{WorkspaceSnapshot, WorkspaceSnapshotBuilder, snapshot_from_parts};

    fn name(value: &str) -> EntityName {
        EntityName::new(value).expect("test name must be valid")
    }

    fn source() -> WorkspaceCacheSource {
        WorkspaceCacheSource {
            entries: vec![
                WorkspaceCacheSourceEntry {
                    path: vec!["configuration".to_owned()],
                    kind: WorkspaceCacheSourceEntryKind::Directory,
                    bytes: None,
                },
                WorkspaceCacheSourceEntry {
                    path: vec!["configuration".to_owned(), "Configuration.xml".to_owned()],
                    kind: WorkspaceCacheSourceEntryKind::RegularFile,
                    bytes: Some(vec![0, 1, 127, 255]),
                },
                WorkspaceCacheSourceEntry {
                    path: vec!["marker".to_owned()],
                    kind: WorkspaceCacheSourceEntryKind::Other,
                    bytes: None,
                },
            ],
        }
    }

    fn fixture_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("workspace_service")
    }

    fn fixture_snapshot(root: &Path) -> WorkspaceSnapshot {
        WorkspaceSnapshotBuilder::new()
            .build(root)
            .expect("tracked EDT and Designer XML fixtures must build")
    }

    fn fixture_source(root: &Path) -> WorkspaceCacheSource {
        let state = WorkspaceFileState::scan(root).expect("fixture source scan must succeed");
        WorkspaceCacheSource::try_from(&state).expect("fixture cache source must be valid")
    }

    fn diagnostic_snapshot(root: &Path) -> WorkspaceSnapshot {
        let configuration_root = root.join("configuration");
        let mut graph = SemanticGraph::new();
        graph.insert_node(GraphNode::new(
            oneagent_common::EntityId::new("configuration:test")
                .expect("configuration ID must be valid"),
            name("Test"),
            NodeKind::Metadata(MetadataKind::Configuration),
        ));
        let diagnostic = SemanticDiagnostic::new(
            SemanticDiagnosticCode::ReferenceMalformedFormat,
            SemanticDiagnosticSeverity::Error,
            SemanticDiagnosticKind::MalformedReferenceFormat,
            "malformed cached reference",
            SemanticReference::Raw("broken".to_owned()),
        );
        let mut statistics = SemanticReferenceStatistics::new();
        statistics.record(SemanticReferenceOutcome::MalformedFormat, false);
        let report = SemanticGraphReport::from_graph_diagnostics_and_references(
            &graph,
            std::slice::from_ref(&diagnostic),
            statistics,
        );
        let validation = SemanticGraphValidator::new()
            .validate_build_result_with_reference_requests_and_report(
                &graph,
                std::slice::from_ref(&diagnostic),
                &SemanticReferenceRequestLedger::new(),
                statistics,
                &report,
            );
        let configuration = snapshot_from_parts(
            &configuration_root,
            oneagent_workspace::WorkspaceFormat::Edt,
            graph,
            SourceEvidenceSet::new(
                oneagent_common::EntityId::new("configuration:test")
                    .expect("configuration ID must be valid"),
                Vec::new(),
            )
            .expect("empty source evidence must be valid"),
            vec![diagnostic],
            SemanticReferenceRequestLedger::new(),
            statistics,
            report,
            validation,
        )
        .expect("diagnostic-rich snapshot must be valid");
        WorkspaceSnapshot::initial(root.to_path_buf(), vec![configuration])
    }

    fn canonical_bytes(envelope: &mut EnvelopeDto) -> Vec<u8> {
        envelope.content_checksum = content_checksum(&envelope.source, &envelope.workspace)
            .expect("test checksum must encode");
        serde_json::to_vec(envelope).expect("test envelope must encode")
    }

    fn cache_file(root: &Path) -> PathBuf {
        root.join(".oneagent/cache/workspace-v1.json")
    }

    fn temporary_file(root: &Path) -> PathBuf {
        root.join(".oneagent/cache/workspace-v1.tmp")
    }

    #[test]
    fn empty_workspace_round_trip_uses_exact_envelope_and_checksum() {
        let root = Path::new("/workspace");
        let source = WorkspaceCacheSource { entries: vec![] };
        let snapshot = WorkspaceSnapshot::default();

        let bytes = WorkspaceCacheCodec::encode(&source, root, &snapshot)
            .expect("empty snapshot must encode");
        let decoded =
            WorkspaceCacheCodec::decode(&bytes, &source, root).expect("empty snapshot must decode");
        let envelope: EnvelopeDto =
            serde_json::from_slice(&bytes).expect("envelope must be valid JSON");

        assert!(decoded.is_empty());
        assert_eq!(envelope.format, "oneagent.workspace-cache");
        assert_eq!(envelope.schema_version, 1);
        assert_eq!(envelope.semantic_version, 6);
        assert_eq!(decoded.root_path(), root);
        assert!(envelope.content_checksum.starts_with("fnv1a64:"));
        assert_eq!(envelope.content_checksum.len(), 24);
    }

    #[test]
    fn mixed_clean_build_round_trip_is_complete_and_byte_deterministic() {
        let root = fixture_root();
        let source = fixture_source(&root);
        let clean = fixture_snapshot(&root);
        assert_eq!(clean.len(), 2, "fixture must cover EDT and Designer XML");
        assert!(
            clean
                .configurations()
                .iter()
                .any(|configuration| !configuration.reference_requests().is_empty()),
            "tracked EDT fixture must retain reference-request evidence"
        );

        let first = WorkspaceCacheCodec::encode(&source, &root, &clean)
            .expect("clean snapshot must encode");
        let second = WorkspaceCacheCodec::encode(&source, &root, &clean)
            .expect("repeated encoding must succeed");
        let decoded =
            WorkspaceCacheCodec::decode(&first, &source, &root).expect("current cache must decode");
        let reencoded = WorkspaceCacheCodec::encode(&source, &root, &decoded)
            .expect("decoded snapshot must re-encode");

        let edt = clean
            .configurations()
            .iter()
            .find(|configuration| configuration.format() == WorkspaceFormat::Edt)
            .expect("tracked EDT Configuration must be present");
        let target = edt
            .source_evidence()
            .documents()
            .iter()
            .flat_map(SourceDocument::occurrences)
            .find(|occurrence| {
                occurrence.kind() == SourceOccurrenceKind::Declaration
                    && occurrence.token() == "Posting"
            })
            .and_then(SourceOccurrence::mapped_target_id)
            .expect("tracked Posting declaration must map uniquely")
            .clone();
        let request = RefactoringRequest::new(
            RefactoringFamily::BslCallableRenameV1,
            clean.publication_id(),
            edt.configuration_id().clone(),
            target,
            "PostingRenamed",
        )
        .expect("cache planner request must be valid");
        let cold_plan = clean
            .plan_refactoring(&request, &NeverCancelledRefactoring)
            .expect("cold snapshot plan must succeed");
        let warm_plan = decoded
            .plan_refactoring(&request, &NeverCancelledRefactoring)
            .expect("warm snapshot plan must succeed");

        assert_eq!(first, second);
        assert_eq!(first, reencoded);
        assert_eq!(cold_plan, warm_plan);
        assert_eq!(
            WorkspaceDto::from_snapshot(&source, &root, &clean).expect("clean DTO must build"),
            WorkspaceDto::from_snapshot(&source, &root, &decoded).expect("decoded DTO must build")
        );
        for (expected, actual) in clean.configurations().iter().zip(decoded.configurations()) {
            assert_eq!(expected.configuration_id(), actual.configuration_id());
            assert_eq!(expected.configuration_name(), actual.configuration_name());
            assert_eq!(expected.report(), actual.report());
            assert_eq!(expected.source_evidence(), actual.source_evidence());
            assert_eq!(
                expected.graph().nodes().collect::<Vec<_>>(),
                actual.graph().nodes().collect::<Vec<_>>()
            );
        }
        assert_eq!(decoded.root_path(), root);
    }

    #[test]
    fn edt_designer_and_mixed_snapshots_each_round_trip() {
        let root = fixture_root();
        let clean = fixture_snapshot(&root);
        let source = fixture_source(&root);

        for configuration in clean.configurations() {
            let single = WorkspaceSnapshot::initial(root.clone(), vec![configuration.clone()]);
            let bytes = WorkspaceCacheCodec::encode(&source, &root, &single)
                .expect("single-format snapshot must encode");
            let decoded = WorkspaceCacheCodec::decode(&bytes, &source, &root)
                .expect("single-format snapshot must decode");
            assert_eq!(decoded.len(), 1);
            assert_eq!(decoded.configurations()[0].format(), configuration.format());
        }

        let mut reordered = clean.configurations().to_vec();
        reordered.reverse();
        let error = WorkspaceCacheCodec::encode(
            &source,
            &root,
            &WorkspaceSnapshot::initial(root.clone(), reordered),
        )
        .expect_err("configuration reorder must violate canonical order");
        assert_eq!(error.kind(), WorkspaceCacheCodecErrorKind::Inconsistent);
    }

    #[test]
    fn diagnostic_and_legacy_reference_evidence_round_trips() {
        let root = Path::new("/workspace");
        let snapshot = diagnostic_snapshot(root);
        let statistics = snapshot.configurations()[0].reference_statistics();
        let source = source();

        let bytes = WorkspaceCacheCodec::encode(&source, root, &snapshot)
            .expect("diagnostic-rich snapshot must encode");
        let decoded = WorkspaceCacheCodec::decode(&bytes, &source, root)
            .expect("diagnostic-rich snapshot must decode");

        assert_eq!(decoded.configurations()[0].diagnostics().len(), 1);
        assert_eq!(
            decoded.configurations()[0].reference_statistics(),
            statistics
        );
        assert_eq!(
            decoded.configurations()[0].report(),
            snapshot.configurations()[0].report()
        );
        assert_eq!(
            decoded.configurations()[0].validation(),
            snapshot.configurations()[0].validation()
        );
        assert_eq!(
            decoded.configurations()[0].rule_execution_report(),
            snapshot.configurations()[0].rule_execution_report()
        );
        assert!(
            decoded.configurations()[0]
                .rule_execution_report()
                .results()
                .is_empty()
        );
        assert_eq!(
            decoded.configurations()[0]
                .rule_execution_report()
                .summary()
                .total(),
            0
        );
        assert_eq!(
            decoded.configurations()[0].diagnostic_report(),
            snapshot.configurations()[0].diagnostic_report()
        );
        assert_eq!(
            decoded.configurations()[0]
                .diagnostic_report()
                .summary()
                .total(),
            2
        );
        assert_eq!(
            decoded.configurations()[0]
                .diagnostic_report()
                .summary()
                .suppressed(),
            0
        );
    }

    #[test]
    fn every_node_payload_variant_round_trips_without_loss() {
        let metadata_specific = [
            MetadataSpecificPayload::Document(DocumentMetadataPayload::new([
                MetadataRegisterRecord::new(MetadataKind::InformationRegister, name("Records")),
            ])),
            MetadataSpecificPayload::EventSubscription(EventSubscriptionMetadataPayload::new(
                name("OnWrite"),
            )),
            MetadataSpecificPayload::HttpService(HttpServiceMetadataPayload::new("/api")),
            MetadataSpecificPayload::WebService(WebServiceMetadataPayload::new(
                "urn:test",
                [
                    WebServiceXdtoPackage::Repository(name("Package")),
                    WebServiceXdtoPackage::ExternalNamespace("urn:external".to_owned()),
                ],
            )),
            MetadataSpecificPayload::XdtoPackage(XdtoPackageMetadataPayload::new("urn:package")),
        ];
        let mut payloads = vec![GraphNodePayload::None];
        payloads.extend(metadata_specific.into_iter().map(|specific| {
            GraphNodePayload::Metadata(MetadataPayload::new(
                CommonMetadataPayload::new(Some("Synonym".to_owned())),
                Some(specific),
            ))
        }));
        payloads.extend([
            GraphNodePayload::MetadataMember(MetadataMemberPayload::new(Some("Member".to_owned()))),
            GraphNodePayload::AccessRight(AccessRightPayload::new(Some(
                AccessRightRowRestriction::new("Allowed = TRUE")
                    .expect("restriction must be valid"),
            ))),
            GraphNodePayload::DataCompositionSchema(DataCompositionSchemaPayload::new(true)),
            GraphNodePayload::DataSet(
                DataSetPayload::new(DataSetKind::Query, Some(name("Main")))
                    .expect("Data Set must be valid"),
            ),
            GraphNodePayload::DataCompositionField(DataCompositionFieldPayload::new(name(
                "Products.Ref",
            ))),
            GraphNodePayload::XdtoType(XdtoTypePayload::new(XdtoTypeKind::Object)),
            GraphNodePayload::HttpServiceUrlTemplate(HttpServiceUrlTemplatePayload::new(
                "/items/{id}",
            )),
            GraphNodePayload::HttpServiceMethod(HttpServiceMethodPayload::new(Some(name("GET")))),
            GraphNodePayload::WebServiceOperation(WebServiceOperationPayload::new(
                XdtoTypeReference::new("urn:test", name("Result")),
                Some(true),
            )),
            GraphNodePayload::WebServiceParameter(WebServiceParameterPayload::new(
                XdtoTypeReference::new("urn:test", name("Input")),
                None,
                Some(WebServiceParameterDirection::InOut),
            )),
        ]);

        assert_eq!(payloads.len(), 16);
        for payload in payloads {
            let dto = NodePayloadDto::from(&payload);
            let reconstructed = GraphNodePayload::try_from(dto)
                .expect("every current payload variant must reconstruct");
            assert_eq!(reconstructed, payload);
        }
    }

    #[test]
    fn every_semantic_reference_variant_round_trips_without_loss() {
        let references = [
            SemanticReference::Raw("raw".to_owned()),
            SemanticReference::NodeId("node:id".to_owned()),
            SemanticReference::Name(name("Target")),
            SemanticReference::Child {
                owner: oneagent_common::EntityId::new("owner").expect("owner ID must be valid"),
                name: name("Child"),
            },
            SemanticReference::Owner {
                child: oneagent_common::EntityId::new("child").expect("child ID must be valid"),
            },
            SemanticReference::OwnedChild {
                owner: oneagent_common::EntityId::new("owner").expect("owner ID must be valid"),
                child: oneagent_common::EntityId::new("child").expect("child ID must be valid"),
            },
        ];

        for reference in references {
            let dto = ReferenceDto::from(&reference);
            assert_eq!(
                SemanticReference::try_from(dto).expect("reference must reconstruct"),
                reference
            );
        }
    }

    #[test]
    #[allow(clippy::too_many_lines)] // One inventory test makes closed-vocabulary gaps visible.
    fn every_closed_semantic_vocabulary_variant_has_a_round_trip() {
        let metadata_kinds = [
            MetadataKind::Configuration,
            MetadataKind::Subsystem,
            MetadataKind::Catalog,
            MetadataKind::Document,
            MetadataKind::Enumeration,
            MetadataKind::CommonModule,
            MetadataKind::Report,
            MetadataKind::DataProcessor,
            MetadataKind::InformationRegister,
            MetadataKind::AccumulationRegister,
            MetadataKind::AccountingRegister,
            MetadataKind::CalculationRegister,
            MetadataKind::BusinessProcess,
            MetadataKind::Task,
            MetadataKind::Role,
            MetadataKind::CommonForm,
            MetadataKind::Form,
            MetadataKind::Command,
            MetadataKind::Template,
            MetadataKind::HttpService,
            MetadataKind::WebService,
            MetadataKind::XdtoPackage,
            MetadataKind::EventSubscription,
            MetadataKind::Unknown,
        ];
        for value in metadata_kinds {
            assert_eq!(MetadataKind::from(MetadataKindDto::from(value)), value);
        }

        let node_kinds = [
            NodeKind::Metadata(MetadataKind::Configuration),
            NodeKind::Module,
            NodeKind::Procedure,
            NodeKind::Function,
            NodeKind::Query,
            NodeKind::DataCompositionSchema,
            NodeKind::DataSet,
            NodeKind::DataCompositionField,
            NodeKind::XdtoType,
            NodeKind::HttpServiceUrlTemplate,
            NodeKind::HttpServiceMethod,
            NodeKind::WebServiceOperation,
            NodeKind::WebServiceParameter,
            NodeKind::Form,
            NodeKind::Command,
            NodeKind::Attribute,
            NodeKind::StandardAttribute,
            NodeKind::TabularSection,
            NodeKind::Dimension,
            NodeKind::Resource,
            NodeKind::Measure,
            NodeKind::Role,
            NodeKind::AccessRight,
            NodeKind::Subsystem,
            NodeKind::Unknown,
        ];
        for value in node_kinds {
            assert_eq!(NodeKind::from(NodeKindDto::from(value)), value);
        }

        let edge_kinds = [
            EdgeKind::Contains,
            EdgeKind::Calls,
            EdgeKind::References,
            EdgeKind::Reads,
            EdgeKind::Writes,
            EdgeKind::Grants,
            EdgeKind::Includes,
            EdgeKind::Extends,
            EdgeKind::DependsOn,
            EdgeKind::Opens,
            EdgeKind::Triggers,
        ];
        for value in edge_kinds {
            assert_eq!(EdgeKind::from(EdgeKindDto::from(value)), value);
        }

        for value in [
            FactOrigin::Declared,
            FactOrigin::Parsed,
            FactOrigin::Resolved,
            FactOrigin::Derived,
            FactOrigin::External,
        ] {
            assert_eq!(FactOrigin::from(FactOriginDto::from(value)), value);
        }
        for value in [
            Confidence::Exact,
            Confidence::High,
            Confidence::Medium,
            Confidence::Low,
            Confidence::Unknown,
        ] {
            assert_eq!(Confidence::from(ConfidenceDto::from(value)), value);
        }
        for value in [
            ResolutionState::NotApplicable,
            ResolutionState::Unresolved,
            ResolutionState::Partial,
            ResolutionState::Ambiguous,
            ResolutionState::Resolved,
        ] {
            assert_eq!(
                ResolutionState::from(ResolutionStateDto::from(value)),
                value
            );
        }
        for value in [DataSetKind::Query, DataSetKind::Object, DataSetKind::Union] {
            assert_eq!(DataSetKind::from(DataSetKindDto::from(value)), value);
        }
        for value in [XdtoTypeKind::Value, XdtoTypeKind::Object] {
            assert_eq!(XdtoTypeKind::from(XdtoTypeKindDto::from(value)), value);
        }
        for value in [
            WebServiceParameterDirection::Out,
            WebServiceParameterDirection::InOut,
        ] {
            assert_eq!(
                WebServiceParameterDirection::from(WebServiceParameterDirectionDto::from(value)),
                value
            );
        }

        let codes = [
            SemanticDiagnosticCode::QueryLanguageMalformedSyntax,
            SemanticDiagnosticCode::QueryLanguageUnsupportedStructure,
            SemanticDiagnosticCode::QueryLanguageUnsupportedPersistentNamespace,
            SemanticDiagnosticCode::QueryLanguageVirtualTableSource,
            SemanticDiagnosticCode::QueryLanguageTemporaryTableSource,
            SemanticDiagnosticCode::QueryLanguageExternalOrParameterDataSource,
            SemanticDiagnosticCode::DataCompositionNestedDataSetDeferred,
            SemanticDiagnosticCode::DataCompositionFieldFolderDeferred,
            SemanticDiagnosticCode::DataCompositionUnsupportedDataSetType,
            SemanticDiagnosticCode::DataCompositionUnsupportedFieldType,
            SemanticDiagnosticCode::ReferenceMalformedFormat,
            SemanticDiagnosticCode::ReferenceUnsupportedPrefix,
            SemanticDiagnosticCode::ReferenceUnresolved,
            SemanticDiagnosticCode::ReferenceAmbiguous,
            SemanticDiagnosticCode::ReferenceIncompatibleKind,
            SemanticDiagnosticCode::ReferenceInvalidOwner,
            SemanticDiagnosticCode::DuplicateSemanticEdgeRequest,
        ];
        for value in codes {
            assert_eq!(
                SemanticDiagnosticCode::from(DiagnosticCodeDto::from(value)),
                value
            );
        }

        let kinds = [
            SemanticDiagnosticKind::QueryLanguageMalformedSyntax,
            SemanticDiagnosticKind::QueryLanguageUnsupportedStructure,
            SemanticDiagnosticKind::QueryLanguageUnsupportedPersistentNamespace,
            SemanticDiagnosticKind::QueryLanguageVirtualTableSource,
            SemanticDiagnosticKind::QueryLanguageTemporaryTableSource,
            SemanticDiagnosticKind::QueryLanguageExternalOrParameterDataSource,
            SemanticDiagnosticKind::DataCompositionNestedDataSetDeferred,
            SemanticDiagnosticKind::DataCompositionFieldFolderDeferred,
            SemanticDiagnosticKind::DataCompositionUnsupportedDataSetType,
            SemanticDiagnosticKind::DataCompositionUnsupportedFieldType,
            SemanticDiagnosticKind::MalformedReferenceFormat,
            SemanticDiagnosticKind::UnsupportedReferencePrefix,
            SemanticDiagnosticKind::UnresolvedTarget,
            SemanticDiagnosticKind::AmbiguousTarget,
            SemanticDiagnosticKind::IncompatibleTargetKind,
            SemanticDiagnosticKind::InvalidOwnerReference,
            SemanticDiagnosticKind::DuplicateSemanticEdgeRequest,
        ];
        for value in kinds {
            assert_eq!(
                SemanticDiagnosticKind::from(DiagnosticKindDto::from(value)),
                value
            );
        }

        let categories = [
            SemanticReferenceCategory::MetadataType,
            SemanticReferenceCategory::Callable,
            SemanticReferenceCategory::QuerySource,
            SemanticReferenceCategory::WriteTarget,
            SemanticReferenceCategory::ProtectedResource,
            SemanticReferenceCategory::SubsystemMember,
            SemanticReferenceCategory::ExtensionTarget,
            SemanticReferenceCategory::XdtoPackage,
            SemanticReferenceCategory::XdtoType,
        ];
        for value in categories {
            assert_eq!(
                SemanticReferenceCategory::from(ReferenceCategoryDto::from(value)),
                value
            );
        }

        let outcomes = [
            SemanticReferenceRequestOutcome::Collected,
            SemanticReferenceRequestOutcome::Resolved,
            SemanticReferenceRequestOutcome::MissingTarget,
            SemanticReferenceRequestOutcome::PartialWorkspace,
            SemanticReferenceRequestOutcome::AmbiguousTarget,
            SemanticReferenceRequestOutcome::IncompatibleTargetKind,
            SemanticReferenceRequestOutcome::InvalidOwnerReference,
        ];
        for value in outcomes {
            assert_eq!(
                SemanticReferenceRequestOutcome::from(ReferenceRequestOutcomeDto::from(value)),
                value
            );
        }
    }

    #[test]
    fn envelope_rejects_incompatible_malformed_partial_duplicate_and_unsupported_values() {
        let root = Path::new("/workspace");
        let empty_source = WorkspaceCacheSource { entries: vec![] };
        let bytes = WorkspaceCacheCodec::encode(&empty_source, root, &WorkspaceSnapshot::default())
            .expect("empty snapshot must encode");
        let mut envelope: EnvelopeDto =
            serde_json::from_slice(&bytes).expect("envelope must parse");

        envelope.semantic_version = 5;
        let incompatible = serde_json::to_vec(&envelope).expect("test envelope must encode");
        assert_eq!(
            WorkspaceCacheCodec::decode(&incompatible, &empty_source, root)
                .expect_err("previous semantic evidence must be rejected")
                .kind(),
            WorkspaceCacheCodecErrorKind::Incompatible
        );

        envelope.schema_version = 2;
        envelope.semantic_version = 6;
        let incompatible = serde_json::to_vec(&envelope).expect("test envelope must encode");
        assert_eq!(
            WorkspaceCacheCodec::decode(&incompatible, &empty_source, root)
                .expect_err("future schema must be rejected")
                .kind(),
            WorkspaceCacheCodecErrorKind::Incompatible
        );
        assert_eq!(
            WorkspaceCacheCodec::decode(b"not-json", &empty_source, root)
                .expect_err("malformed JSON must be rejected")
                .kind(),
            WorkspaceCacheCodecErrorKind::Malformed
        );
        assert_eq!(
            WorkspaceCacheCodec::decode(b"{}", &empty_source, root)
                .expect_err("partial envelope must be rejected")
                .kind(),
            WorkspaceCacheCodecErrorKind::Partial
        );

        let duplicate = bytes
            .strip_prefix(b"{")
            .map(|suffix| {
                let mut value = br#"{"format":"oneagent.workspace-cache","#.to_vec();
                value.extend_from_slice(suffix);
                value
            })
            .expect("canonical JSON must be an object");
        assert_eq!(
            WorkspaceCacheCodec::decode(&duplicate, &empty_source, root)
                .expect_err("duplicate field must be rejected")
                .kind(),
            WorkspaceCacheCodecErrorKind::Duplicate
        );

        let mut value: serde_json::Value =
            serde_json::from_slice(&bytes).expect("envelope must parse as a JSON value");
        value
            .as_object_mut()
            .expect("envelope must be an object")
            .insert("future".to_owned(), serde_json::Value::Bool(true));
        let unsupported = serde_json::to_vec(&value).expect("test JSON must encode");
        assert_eq!(
            WorkspaceCacheCodec::decode(&unsupported, &empty_source, root)
                .expect_err("unknown field must be rejected")
                .kind(),
            WorkspaceCacheCodecErrorKind::Unsupported
        );
    }

    #[test]
    fn envelope_rejects_noncanonical_checksum_and_source_mismatches() {
        let root = Path::new("/workspace");
        let empty_source = WorkspaceCacheSource { entries: vec![] };
        let bytes = WorkspaceCacheCodec::encode(&empty_source, root, &WorkspaceSnapshot::default())
            .expect("empty snapshot must encode");

        let mut noncanonical = bytes.clone();
        noncanonical.push(b'\n');
        assert_eq!(
            WorkspaceCacheCodec::decode(&noncanonical, &empty_source, root)
                .expect_err("alternate whitespace must be rejected")
                .kind(),
            WorkspaceCacheCodecErrorKind::NonCanonical
        );

        let mut envelope: EnvelopeDto =
            serde_json::from_slice(&bytes).expect("envelope must parse");
        envelope.content_checksum = "fnv1a64:0000000000000000".to_owned();
        let checksum = serde_json::to_vec(&envelope).expect("test envelope must encode");
        assert_eq!(
            WorkspaceCacheCodec::decode(&checksum, &empty_source, root)
                .expect_err("changed checksum must be rejected")
                .kind(),
            WorkspaceCacheCodecErrorKind::ChecksumMismatch
        );

        assert_eq!(
            WorkspaceCacheCodec::decode(&bytes, &source(), root)
                .expect_err("different source state must be rejected")
                .kind(),
            WorkspaceCacheCodecErrorKind::SourceMismatch
        );
    }

    #[test]
    fn codec_rejects_invalid_source_paths_order_and_kind_contracts() {
        let root = Path::new("/workspace");
        let snapshot = WorkspaceSnapshot::default();
        let cases = [
            WorkspaceCacheSource {
                entries: vec![WorkspaceCacheSourceEntry {
                    path: vec!["..".to_owned()],
                    kind: WorkspaceCacheSourceEntryKind::Directory,
                    bytes: None,
                }],
            },
            WorkspaceCacheSource {
                entries: vec![
                    WorkspaceCacheSourceEntry {
                        path: vec!["z".to_owned()],
                        kind: WorkspaceCacheSourceEntryKind::Directory,
                        bytes: None,
                    },
                    WorkspaceCacheSourceEntry {
                        path: vec!["a".to_owned()],
                        kind: WorkspaceCacheSourceEntryKind::Directory,
                        bytes: None,
                    },
                ],
            },
            WorkspaceCacheSource {
                entries: vec![WorkspaceCacheSourceEntry {
                    path: vec!["file".to_owned()],
                    kind: WorkspaceCacheSourceEntryKind::RegularFile,
                    bytes: None,
                }],
            },
        ];

        for source in cases {
            assert!(WorkspaceCacheCodec::encode(&source, root, &snapshot).is_err());
        }
    }

    #[test]
    fn reconstruction_rejects_duplicate_invalid_and_inconsistent_build_evidence() {
        let root = fixture_root();
        let source = fixture_source(&root);
        let clean = fixture_snapshot(&root);
        let bytes = WorkspaceCacheCodec::encode(&source, &root, &clean)
            .expect("clean snapshot must encode");

        let mut duplicate: EnvelopeDto =
            serde_json::from_slice(&bytes).expect("envelope must parse");
        let repeated = duplicate.workspace.configurations[0].nodes[0].clone();
        duplicate.workspace.configurations[0]
            .nodes
            .insert(0, repeated);
        let duplicate = canonical_bytes(&mut duplicate);
        assert_eq!(
            WorkspaceCacheCodec::decode(&duplicate, &source, &root)
                .expect_err("duplicate node must be rejected")
                .kind(),
            WorkspaceCacheCodecErrorKind::Duplicate
        );

        let mut invalid_endpoint: EnvelopeDto =
            serde_json::from_slice(&bytes).expect("envelope must parse");
        invalid_endpoint.workspace.configurations[0].edges[0].target = "missing-node".to_owned();
        let invalid_endpoint = canonical_bytes(&mut invalid_endpoint);
        assert_eq!(
            WorkspaceCacheCodec::decode(&invalid_endpoint, &source, &root)
                .expect_err("missing edge endpoint must be rejected")
                .kind(),
            WorkspaceCacheCodecErrorKind::Invalid
        );

        let mut stale_source: EnvelopeDto =
            serde_json::from_slice(&bytes).expect("envelope must parse");
        stale_source.workspace.configurations[0]
            .source_evidence
            .documents[0]
            .content_version
            .raw_byte_len += 1;
        let stale_source = canonical_bytes(&mut stale_source);
        assert_eq!(
            WorkspaceCacheCodec::decode(&stale_source, &source, &root)
                .expect_err("stale source manifest must be rejected")
                .kind(),
            WorkspaceCacheCodecErrorKind::Inconsistent
        );

        let mut invalid_payload: EnvelopeDto =
            serde_json::from_slice(&bytes).expect("envelope must parse");
        invalid_payload.workspace.configurations[0].nodes[0].payload =
            NodePayloadDto::AccessRight(AccessRightPayloadDto {
                row_restriction: None,
            });
        let invalid_payload = canonical_bytes(&mut invalid_payload);
        assert_eq!(
            WorkspaceCacheCodec::decode(&invalid_payload, &source, &root)
                .expect_err("kind/payload mismatch must be rejected")
                .kind(),
            WorkspaceCacheCodecErrorKind::Invalid
        );

        let mut inconsistent_statistics: EnvelopeDto =
            serde_json::from_slice(&bytes).expect("envelope must parse");
        inconsistent_statistics.workspace.configurations[0]
            .reference_statistics
            .total += 1;
        let inconsistent_statistics = canonical_bytes(&mut inconsistent_statistics);
        assert_eq!(
            WorkspaceCacheCodec::decode(&inconsistent_statistics, &source, &root)
                .expect_err("inconsistent statistics must be rejected")
                .kind(),
            WorkspaceCacheCodecErrorKind::Inconsistent
        );

        let mut noncanonical_semantics: EnvelopeDto =
            serde_json::from_slice(&bytes).expect("envelope must parse");
        let request = noncanonical_semantics
            .workspace
            .configurations
            .iter_mut()
            .find_map(|configuration| configuration.reference_requests.first_mut())
            .expect("tracked EDT fixture must contain a request");
        request
            .expected_kinds
            .push(request.expected_kinds[0].clone());
        let noncanonical_semantics = canonical_bytes(&mut noncanonical_semantics);
        assert_eq!(
            WorkspaceCacheCodec::decode(&noncanonical_semantics, &source, &root)
                .expect_err("semantic normalization must not silently repair cache data")
                .kind(),
            WorkspaceCacheCodecErrorKind::Inconsistent
        );
    }

    #[test]
    fn store_missing_write_hit_cleanup_and_repetition_are_deterministic() {
        let root = tempdir().expect("temporary Workspace must be created");
        fs::write(root.path().join("source.txt"), b"stable").expect("source file must be created");
        let state = WorkspaceFileState::scan(root.path()).expect("source scan must succeed");
        let snapshot = diagnostic_snapshot(root.path());
        let store = WorkspaceCacheStore::new(root.path().to_path_buf());

        assert_eq!(
            store.load(&state).outcome(),
            WorkspaceCacheLoadOutcome::Missing
        );
        assert_eq!(
            store.write(&state, &snapshot),
            WorkspaceCacheWriteOutcome::Succeeded
        );
        assert!(cache_file(root.path()).is_file());
        assert!(!temporary_file(root.path()).exists());
        assert_eq!(
            WorkspaceFileState::scan(root.path()).expect("post-write scan must succeed"),
            state,
            "cache-owned content must not contaminate source identity"
        );

        let loaded = store.load(&state);
        assert_eq!(loaded.outcome(), WorkspaceCacheLoadOutcome::Hit);
        let loaded = loaded.into_snapshot().expect("hit must retain a snapshot");
        let source = WorkspaceCacheSource::try_from(&state)
            .expect("observed cache source must remain valid");
        assert_eq!(
            WorkspaceDto::from_snapshot(&source, root.path(), &snapshot)
                .expect("clean snapshot DTO must build"),
            WorkspaceDto::from_snapshot(&source, root.path(), &loaded)
                .expect("loaded snapshot DTO must build")
        );

        fs::write(temporary_file(root.path()), b"stale")
            .expect("stale temporary file must be created");
        assert_eq!(
            store.write(&state, &snapshot),
            WorkspaceCacheWriteOutcome::Succeeded
        );
        assert!(!temporary_file(root.path()).exists());
        assert_eq!(store.load(&state).outcome(), WorkspaceCacheLoadOutcome::Hit);
    }

    #[test]
    fn store_identity_tracks_every_source_change_and_ignores_enumeration_order() {
        let root = tempdir().expect("temporary Workspace must be created");
        let source_file = root.path().join("source.txt");
        fs::write(&source_file, b"stable").expect("source file must be created");
        let baseline = WorkspaceFileState::scan(root.path()).expect("baseline scan must succeed");
        let snapshot = diagnostic_snapshot(root.path());
        let store = WorkspaceCacheStore::new(root.path().to_path_buf());
        assert_eq!(
            store.write(&baseline, &snapshot),
            WorkspaceCacheWriteOutcome::Succeeded
        );

        fs::write(&source_file, b"changed").expect("source content must change");
        let modified = WorkspaceFileState::scan(root.path()).expect("modified scan must succeed");
        assert_eq!(
            store.load(&modified).outcome(),
            WorkspaceCacheLoadOutcome::SourceChanged
        );
        fs::write(&source_file, b"stable").expect("source content must recover");
        assert_eq!(
            store
                .load(&WorkspaceFileState::scan(root.path()).expect("recovery scan must succeed"))
                .outcome(),
            WorkspaceCacheLoadOutcome::Hit
        );

        let added_file = root.path().join("added.txt");
        fs::write(&added_file, b"added").expect("source file must be added");
        assert_eq!(
            store
                .load(&WorkspaceFileState::scan(root.path()).expect("addition scan must succeed"))
                .outcome(),
            WorkspaceCacheLoadOutcome::SourceChanged
        );
        fs::remove_file(&added_file).expect("added source must be removed");

        let renamed_file = root.path().join("renamed.txt");
        fs::rename(&source_file, &renamed_file).expect("source file must be renamed");
        assert_eq!(
            store
                .load(&WorkspaceFileState::scan(root.path()).expect("rename scan must succeed"))
                .outcome(),
            WorkspaceCacheLoadOutcome::SourceChanged
        );
        fs::rename(&renamed_file, &source_file).expect("source rename must recover");
        fs::remove_file(&source_file).expect("source file must be removed");
        assert_eq!(
            store
                .load(&WorkspaceFileState::scan(root.path()).expect("removal scan must succeed"))
                .outcome(),
            WorkspaceCacheLoadOutcome::SourceChanged
        );

        let first = tempdir().expect("first enumeration root must be created");
        let second = tempdir().expect("second enumeration root must be created");
        for (root, paths) in [
            (first.path(), ["a.txt", "z.txt"]),
            (second.path(), ["z.txt", "a.txt"]),
        ] {
            for path in paths {
                fs::write(root.join(path), path.as_bytes())
                    .expect("enumeration source must be created");
            }
        }
        assert_eq!(
            WorkspaceCacheSource::try_from(
                &WorkspaceFileState::scan(first.path()).expect("first scan must succeed")
            )
            .expect("first source identity must build"),
            WorkspaceCacheSource::try_from(
                &WorkspaceFileState::scan(second.path()).expect("second scan must succeed")
            )
            .expect("second source identity must build")
        );
    }

    #[test]
    fn store_classifies_incompatible_corrupt_and_unavailable_candidates() {
        let root = tempdir().expect("temporary Workspace must be created");
        fs::write(root.path().join("source.txt"), b"stable").expect("source file must be created");
        let state = WorkspaceFileState::scan(root.path()).expect("source scan must succeed");
        let snapshot = diagnostic_snapshot(root.path());
        let store = WorkspaceCacheStore::new(root.path().to_path_buf());
        assert_eq!(
            store.write(&state, &snapshot),
            WorkspaceCacheWriteOutcome::Succeeded
        );

        let bytes = fs::read(cache_file(root.path())).expect("cache file must be readable");
        let mut envelope: EnvelopeDto =
            serde_json::from_slice(&bytes).expect("cache envelope must parse");
        envelope.semantic_version += 1;
        fs::write(
            cache_file(root.path()),
            serde_json::to_vec(&envelope).expect("future envelope must encode"),
        )
        .expect("future cache must be written");
        assert_eq!(
            store.load(&state).outcome(),
            WorkspaceCacheLoadOutcome::Incompatible
        );

        let mut envelope: EnvelopeDto =
            serde_json::from_slice(&bytes).expect("cache envelope must parse again");
        envelope.schema_version += 1;
        fs::write(
            cache_file(root.path()),
            serde_json::to_vec(&envelope).expect("future schema envelope must encode"),
        )
        .expect("future schema cache must be written");
        assert_eq!(
            store.load(&state).outcome(),
            WorkspaceCacheLoadOutcome::Incompatible
        );

        fs::write(cache_file(root.path()), b"{").expect("truncated cache must be written");
        assert_eq!(
            store.load(&state).outcome(),
            WorkspaceCacheLoadOutcome::Corrupt
        );
        fs::write(cache_file(root.path()), b"not-json").expect("malformed cache must be written");
        assert_eq!(
            store.load(&state).outcome(),
            WorkspaceCacheLoadOutcome::Corrupt
        );

        fs::remove_file(cache_file(root.path())).expect("cache file must be removed");
        fs::create_dir(cache_file(root.path())).expect("wrong-kind cache must be created");
        assert_eq!(
            store.load(&state).outcome(),
            WorkspaceCacheLoadOutcome::Unavailable
        );
        assert_eq!(
            store.write(&state, &snapshot),
            WorkspaceCacheWriteOutcome::Failed
        );
    }

    #[test]
    fn store_rejects_wrong_kind_components_and_temporary_entries() {
        let owner_root = tempdir().expect("owner test root must be created");
        fs::write(owner_root.path().join("source.txt"), b"stable")
            .expect("source file must be created");
        let state = WorkspaceFileState::scan(owner_root.path()).expect("source scan must succeed");
        let snapshot = diagnostic_snapshot(owner_root.path());
        fs::write(owner_root.path().join(".oneagent"), b"wrong kind")
            .expect("wrong-kind owner must be created");
        let store = WorkspaceCacheStore::new(owner_root.path().to_path_buf());
        assert_eq!(
            store.load(&state).outcome(),
            WorkspaceCacheLoadOutcome::Unavailable
        );
        assert_eq!(
            store.write(&state, &snapshot),
            WorkspaceCacheWriteOutcome::Failed
        );

        let temporary_root = tempdir().expect("temporary entry root must be created");
        fs::write(temporary_root.path().join("source.txt"), b"stable")
            .expect("source file must be created");
        let state =
            WorkspaceFileState::scan(temporary_root.path()).expect("source scan must succeed");
        let snapshot = diagnostic_snapshot(temporary_root.path());
        let store = WorkspaceCacheStore::new(temporary_root.path().to_path_buf());
        assert_eq!(
            store.write(&state, &snapshot),
            WorkspaceCacheWriteOutcome::Succeeded
        );
        let current =
            fs::read(cache_file(temporary_root.path())).expect("current cache must be readable");
        fs::create_dir(temporary_file(temporary_root.path()))
            .expect("wrong-kind temporary entry must be created");
        assert_eq!(
            store.write(&state, &snapshot),
            WorkspaceCacheWriteOutcome::Failed
        );
        assert_eq!(
            fs::read(cache_file(temporary_root.path())).expect("current cache must remain"),
            current
        );
    }

    #[test]
    fn replacement_failures_cleanup_partial_temporary_state_and_recover() {
        let root = tempdir().expect("temporary Workspace must be created");
        fs::write(root.path().join("source.txt"), b"stable").expect("source file must be created");
        let state = WorkspaceFileState::scan(root.path()).expect("source scan must succeed");
        let snapshot = diagnostic_snapshot(root.path());
        let store = WorkspaceCacheStore::new(root.path().to_path_buf());
        assert_eq!(
            store.write(&state, &snapshot),
            WorkspaceCacheWriteOutcome::Succeeded
        );

        for point in [
            WorkspaceCacheFailurePoint::CreateTemporary,
            WorkspaceCacheFailurePoint::WriteTemporary,
            WorkspaceCacheFailurePoint::SyncTemporary,
            WorkspaceCacheFailurePoint::ReadBackTemporary,
            WorkspaceCacheFailurePoint::RemoveCurrent,
            WorkspaceCacheFailurePoint::RenameTemporary,
        ] {
            assert_eq!(
                store.write(&state, &snapshot),
                WorkspaceCacheWriteOutcome::Succeeded,
                "each failure case must start with a current cache"
            );
            let current =
                fs::read(cache_file(root.path())).expect("current cache must be readable");
            let failing = WorkspaceCacheStore::new(root.path().to_path_buf()).with_failure(point);
            assert_eq!(
                failing.write(&state, &snapshot),
                WorkspaceCacheWriteOutcome::Failed
            );
            assert!(!temporary_file(root.path()).exists());
            if point == WorkspaceCacheFailurePoint::RenameTemporary {
                assert!(!cache_file(root.path()).exists());
            } else {
                assert_eq!(
                    fs::read(cache_file(root.path())).expect("current cache must be preserved"),
                    current
                );
            }
            assert_eq!(
                store.write(&state, &snapshot),
                WorkspaceCacheWriteOutcome::Succeeded
            );
            assert_eq!(store.load(&state).outcome(), WorkspaceCacheLoadOutcome::Hit);
        }
    }

    #[test]
    fn load_and_write_outcome_vocabularies_include_deferred_runtime_states() {
        assert_eq!(
            WorkspaceCacheLoadOutcome::NotAttempted,
            WorkspaceCacheLoadOutcome::NotAttempted
        );
        assert_eq!(
            WorkspaceCacheWriteOutcome::NotAttempted,
            WorkspaceCacheWriteOutcome::NotAttempted
        );
        assert_eq!(
            WorkspaceCacheWriteOutcome::SkippedUnstableSource,
            WorkspaceCacheWriteOutcome::SkippedUnstableSource
        );
    }

    #[cfg(unix)]
    #[test]
    fn store_never_follows_cache_owner_or_candidate_symlinks() {
        use std::os::unix::fs::symlink;

        let owner_root = tempdir().expect("owner symlink root must be created");
        let external = tempdir().expect("external root must be created");
        fs::write(owner_root.path().join("source.txt"), b"stable")
            .expect("source file must be created");
        let state = WorkspaceFileState::scan(owner_root.path()).expect("source scan must succeed");
        let snapshot = diagnostic_snapshot(owner_root.path());
        fs::create_dir(external.path().join("cache"))
            .expect("external cache-shaped directory must be created");
        let external_temporary = external.path().join("cache/workspace-v1.tmp");
        fs::write(&external_temporary, b"external temporary")
            .expect("external temporary file must be created");
        symlink(external.path(), owner_root.path().join(".oneagent"))
            .expect("owner symlink must be created");
        let store = WorkspaceCacheStore::new(owner_root.path().to_path_buf());
        assert_eq!(
            store.load(&state).outcome(),
            WorkspaceCacheLoadOutcome::Unavailable
        );
        assert_eq!(
            store.write(&state, &snapshot),
            WorkspaceCacheWriteOutcome::Failed
        );
        assert!(!external.path().join("cache/workspace-v1.json").exists());
        assert_eq!(
            fs::read(&external_temporary).expect("external temporary file must remain"),
            b"external temporary"
        );

        let candidate_root = tempdir().expect("candidate symlink root must be created");
        let external_file = external.path().join("external.json");
        fs::write(&external_file, b"external").expect("external file must be created");
        fs::write(candidate_root.path().join("source.txt"), b"stable")
            .expect("source file must be created");
        let state =
            WorkspaceFileState::scan(candidate_root.path()).expect("source scan must succeed");
        let snapshot = diagnostic_snapshot(candidate_root.path());
        fs::create_dir_all(candidate_root.path().join(".oneagent/cache"))
            .expect("cache directories must be created");
        symlink(&external_file, cache_file(candidate_root.path()))
            .expect("candidate symlink must be created");
        let store = WorkspaceCacheStore::new(candidate_root.path().to_path_buf());
        assert_eq!(
            store.load(&state).outcome(),
            WorkspaceCacheLoadOutcome::Unavailable
        );
        assert_eq!(
            store.write(&state, &snapshot),
            WorkspaceCacheWriteOutcome::Failed
        );
        assert_eq!(
            fs::read(external_file).expect("external file must remain readable"),
            b"external"
        );
    }
}
