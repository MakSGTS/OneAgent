//! Private deterministic codec for complete Workspace cache snapshots.

use std::fmt::{Display, Formatter};
use std::path::{Component, Path, PathBuf};

use oneagent_common::{EntityId, EntityName};
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

use super::{WorkspaceConfigurationSnapshot, WorkspaceSnapshot, snapshot_from_parts};

const CACHE_FORMAT: &str = "oneagent.workspace-cache";
const SCHEMA_VERSION: u32 = 1;
const SEMANTIC_VERSION: u32 = 1;
const FNV_OFFSET_BASIS: u64 = 14_695_981_039_346_656_037;
const FNV_PRIME: u64 = 1_099_511_628_211;

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

pub(super) struct WorkspaceCacheCodec;

impl WorkspaceCacheCodec {
    pub(super) fn encode(
        source: &WorkspaceCacheSource,
        workspace_root: &Path,
        snapshot: &WorkspaceSnapshot,
    ) -> Result<Vec<u8>, WorkspaceCacheCodecError> {
        validate_source(source)?;
        let workspace = WorkspaceDto::from_snapshot(workspace_root, snapshot)?;
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
        envelope.workspace.into_snapshot(workspace_root)
    }
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
    nodes: Vec<NodeDto>,
    edges: Vec<EdgeDto>,
    diagnostics: Vec<DiagnosticDto>,
    reference_requests: Vec<ReferenceRequestDto>,
    reference_statistics: ReferenceStatisticsDto,
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
    producer: String,
    origin: FactOriginDto,
    confidence: ConfidenceDto,
    resolution: ResolutionStateDto,
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
    let mut previous: Option<&[String]> = None;
    for entry in &source.entries {
        validate_components(&entry.path)?;
        if previous.is_some_and(|path| path >= entry.path.as_slice()) {
            return Err(inconsistent(
                "workspace cache source entries are not in unique canonical order",
            ));
        }
        previous = Some(&entry.path);
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
    let mut path = root.to_path_buf();
    for component in components {
        path.push(component);
    }
    Ok(path)
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
        Ok(Self::new(
            value.source.map(entity_id).transpose()?,
            ProducerId::new(value.producer),
            value.origin.into(),
            value.confidence.into(),
            value.resolution.into(),
        ))
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
        workspace_root: &Path,
        snapshot: &WorkspaceSnapshot,
    ) -> Result<Self, WorkspaceCacheCodecError> {
        let configurations = snapshot
            .configurations()
            .iter()
            .map(|configuration| ConfigurationDto::from_snapshot(workspace_root, configuration))
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
        workspace_root: &Path,
    ) -> Result<WorkspaceSnapshot, WorkspaceCacheCodecError> {
        let expected = self.clone();
        let mut configurations = Vec::with_capacity(self.configurations.len());
        let mut previous_id: Option<EntityId> = None;
        for configuration in self.configurations {
            let configuration = configuration.into_snapshot(workspace_root)?;
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
        let snapshot = WorkspaceSnapshot { configurations };
        let reconstructed = Self::from_snapshot(workspace_root, &snapshot)?;
        if reconstructed != expected {
            return Err(inconsistent(
                "workspace cache semantic content is not canonically normalized",
            ));
        }
        Ok(snapshot)
    }
}

impl ConfigurationDto {
    fn from_snapshot(
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
        workspace_root: &Path,
    ) -> Result<WorkspaceConfigurationSnapshot, WorkspaceCacheCodecError> {
        let root_path = joined_path(workspace_root, &self.root)?;
        let format = match self.format {
            FormatDto::Edt => WorkspaceFormat::Edt,
            FormatDto::DesignerXml => WorkspaceFormat::DesignerXml,
        };

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
            diagnostics,
            ledger,
            total_statistics,
            report,
        )
        .map_err(|error| invalid(format!("workspace cache snapshot is invalid: {error}")))
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use oneagent_common::EntityName;
    use oneagent_graph::{
        AccessRightPayload, AccessRightRowRestriction, Confidence, DataCompositionFieldPayload,
        DataCompositionSchemaPayload, DataSetKind, DataSetPayload, EdgeKind, FactOrigin, GraphNode,
        GraphNodePayload, HttpServiceMethodPayload, HttpServiceUrlTemplatePayload, NodeKind,
        ResolutionState, SemanticDiagnostic, SemanticDiagnosticCode, SemanticDiagnosticKind,
        SemanticDiagnosticSeverity, SemanticGraph, SemanticGraphReport, SemanticReference,
        SemanticReferenceCategory, SemanticReferenceOutcome, SemanticReferenceRequestLedger,
        SemanticReferenceRequestOutcome, SemanticReferenceStatistics, WebServiceOperationPayload,
        WebServiceParameterDirection, WebServiceParameterPayload, XdtoTypeKind, XdtoTypePayload,
        XdtoTypeReference,
    };
    use oneagent_metadata::{
        CommonMetadataPayload, DocumentMetadataPayload, EventSubscriptionMetadataPayload,
        HttpServiceMetadataPayload, MetadataKind, MetadataMemberPayload, MetadataPayload,
        MetadataRegisterRecord, MetadataSpecificPayload, WebServiceMetadataPayload,
        WebServiceXdtoPackage, XdtoPackageMetadataPayload,
    };

    use super::{
        AccessRightPayloadDto, ConfidenceDto, DataSetKindDto, DiagnosticCodeDto, DiagnosticKindDto,
        EdgeKindDto, EnvelopeDto, FactOriginDto, MetadataKindDto, NodeKindDto, NodePayloadDto,
        ReferenceCategoryDto, ReferenceDto, ReferenceRequestOutcomeDto, ResolutionStateDto,
        WebServiceParameterDirectionDto, WorkspaceCacheCodec, WorkspaceCacheCodecErrorKind,
        WorkspaceCacheSource, WorkspaceCacheSourceEntry, WorkspaceCacheSourceEntryKind,
        WorkspaceDto, XdtoTypeKindDto, content_checksum,
    };
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

    fn canonical_bytes(envelope: &mut EnvelopeDto) -> Vec<u8> {
        envelope.content_checksum = content_checksum(&envelope.source, &envelope.workspace)
            .expect("test checksum must encode");
        serde_json::to_vec(envelope).expect("test envelope must encode")
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
        assert_eq!(envelope.semantic_version, 1);
        assert!(envelope.content_checksum.starts_with("fnv1a64:"));
        assert_eq!(envelope.content_checksum.len(), 24);
    }

    #[test]
    fn mixed_clean_build_round_trip_is_complete_and_byte_deterministic() {
        let root = fixture_root();
        let source = source();
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

        assert_eq!(first, second);
        assert_eq!(first, reencoded);
        assert_eq!(
            WorkspaceDto::from_snapshot(&root, &clean).expect("clean DTO must build"),
            WorkspaceDto::from_snapshot(&root, &decoded).expect("decoded DTO must build")
        );
        for (expected, actual) in clean.configurations().iter().zip(decoded.configurations()) {
            assert_eq!(expected.configuration_id(), actual.configuration_id());
            assert_eq!(expected.configuration_name(), actual.configuration_name());
            assert_eq!(expected.report(), actual.report());
        }
    }

    #[test]
    fn edt_designer_and_mixed_snapshots_each_round_trip() {
        let root = fixture_root();
        let clean = fixture_snapshot(&root);
        let source = source();

        for configuration in clean.configurations() {
            let single = WorkspaceSnapshot {
                configurations: vec![configuration.clone()],
            };
            let bytes = WorkspaceCacheCodec::encode(&source, &root, &single)
                .expect("single-format snapshot must encode");
            let decoded = WorkspaceCacheCodec::decode(&bytes, &source, &root)
                .expect("single-format snapshot must decode");
            assert_eq!(decoded.len(), 1);
            assert_eq!(decoded.configurations()[0].format(), configuration.format());

            let direct_root_bytes =
                WorkspaceCacheCodec::encode(&source, configuration.root_path(), &single)
                    .expect("configuration at the Workspace root must encode");
            let direct_root_decoded =
                WorkspaceCacheCodec::decode(&direct_root_bytes, &source, configuration.root_path())
                    .expect("configuration at the Workspace root must decode");
            assert_eq!(
                direct_root_decoded.configurations()[0].root_path(),
                configuration.root_path()
            );
        }

        let mut reordered = clean.configurations().to_vec();
        reordered.reverse();
        let error = WorkspaceCacheCodec::encode(
            &source,
            &root,
            &WorkspaceSnapshot {
                configurations: reordered,
            },
        )
        .expect_err("configuration reorder must violate canonical order");
        assert_eq!(error.kind(), WorkspaceCacheCodecErrorKind::Inconsistent);
    }

    #[test]
    fn diagnostic_and_legacy_reference_evidence_round_trips() {
        let root = Path::new("/workspace");
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
        let configuration = snapshot_from_parts(
            &configuration_root,
            oneagent_workspace::WorkspaceFormat::Edt,
            graph,
            vec![diagnostic],
            SemanticReferenceRequestLedger::new(),
            statistics,
            report,
        )
        .expect("diagnostic-rich snapshot must be valid");
        let snapshot = WorkspaceSnapshot {
            configurations: vec![configuration],
        };
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

        envelope.schema_version = 2;
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
        let source = source();
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
}
