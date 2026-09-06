//! Immutable source-document evidence produced from captured Designer XML modules.

use std::fmt::{Display, Formatter};
use std::path::{Component, Path, PathBuf};

use oneagent_analysis::refactoring::{
    BslModuleRole, ConfinedSourcePath, MAX_SOURCE_OCCURRENCES_PER_DOCUMENT, SourceByteRange,
    SourceContentVersion, SourceDocument, SourceDocumentId, SourceEvidenceAdmission,
    SourceEvidenceCompleteness, SourceEvidenceError, SourceEvidenceSet, SourceFormat,
    SourceOccurrence, SourceOccurrenceKind, SourceOccurrenceResolution,
};
use oneagent_bsl::{
    BslCall, BslCallKind, BslParseError, BslSymbol, LineBslCallExtractor,
    LineBslDeclarationExtractor, bsl_names_equal,
};
use oneagent_common::{EntityId, SourcePath};

use crate::{DesignerXmlModuleDescriptor, DesignerXmlModuleKind};

pub(crate) fn build_source_evidence(
    workspace_root: &Path,
    project_root: &Path,
    configuration_id: &EntityId,
    modules: &[DesignerXmlModuleDescriptor],
) -> Result<SourceEvidenceSet, DesignerXmlSourceEvidenceError> {
    admit_modules(modules)?;
    let analyzed = modules
        .iter()
        .map(|module| {
            let analysis = analyze(module)?;
            admitted_occurrence_capacity(analysis.symbols.len(), analysis.calls.len())?;
            Ok::<_, DesignerXmlSourceEvidenceError>(analysis)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let roots = ConfinedRoots::new(workspace_root, project_root)?;
    let mut documents = Vec::with_capacity(modules.len());
    for (module, analysis) in modules.iter().zip(&analyzed) {
        documents.push(build_document(
            &roots,
            configuration_id,
            module,
            analysis,
            &analyzed,
        )?);
    }
    SourceEvidenceSet::new(configuration_id.clone(), documents).map_err(Into::into)
}

fn admit_modules(
    modules: &[DesignerXmlModuleDescriptor],
) -> Result<usize, DesignerXmlSourceEvidenceError> {
    admit_document_lengths(
        modules.len(),
        modules
            .iter()
            .map(|module| Ok(module.source().raw_source().len())),
    )
}

fn admit_document_lengths(
    document_count: usize,
    raw_byte_lengths: impl IntoIterator<Item = Result<usize, DesignerXmlSourceEvidenceError>>,
) -> Result<usize, DesignerXmlSourceEvidenceError> {
    let mut admission = SourceEvidenceAdmission::new(document_count)?;
    for raw_byte_len in raw_byte_lengths {
        admission.admit_document(raw_byte_len?)?;
    }
    admission.finish().map_err(Into::into)
}

fn admitted_occurrence_capacity(
    declarations: usize,
    calls: usize,
) -> Result<usize, DesignerXmlSourceEvidenceError> {
    SourceEvidenceAdmission::occurrence_capacity(declarations, calls).map_err(Into::into)
}

struct AnalyzedModule<'module> {
    descriptor: &'module DesignerXmlModuleDescriptor,
    symbols: Vec<BslSymbol>,
    calls: Vec<BslCall>,
}

fn analyze(
    module: &DesignerXmlModuleDescriptor,
) -> Result<AnalyzedModule<'_>, DesignerXmlSourceEvidenceError> {
    let source = std::str::from_utf8(module.source().raw_source()).map_err(|_| {
        DesignerXmlSourceEvidenceError::InvalidCapturedUtf8(
            module.source().artifact_path().to_path_buf(),
        )
    })?;
    let symbols = LineBslDeclarationExtractor.extract_bounded(
        module.id(),
        source,
        MAX_SOURCE_OCCURRENCES_PER_DOCUMENT,
    )?;
    let calls = LineBslCallExtractor.extract_calls_bounded(
        module.id(),
        source,
        symbols.len(),
        MAX_SOURCE_OCCURRENCES_PER_DOCUMENT,
    )?;
    Ok(AnalyzedModule {
        descriptor: module,
        symbols,
        calls,
    })
}

fn build_document(
    roots: &ConfinedRoots,
    configuration_id: &EntityId,
    module: &DesignerXmlModuleDescriptor,
    analysis: &AnalyzedModule<'_>,
    available: &[AnalyzedModule<'_>],
) -> Result<SourceDocument, DesignerXmlSourceEvidenceError> {
    let raw = module.source().raw_source();
    let source = std::str::from_utf8(raw).map_err(|_| {
        DesignerXmlSourceEvidenceError::InvalidCapturedUtf8(
            module.source().artifact_path().to_path_buf(),
        )
    })?;
    let document_id = SourceDocumentId::new(configuration_id.clone(), module.id().clone())?;
    let version = SourceContentVersion::from_bytes(raw);
    let capacity = admitted_occurrence_capacity(analysis.symbols.len(), analysis.calls.len())?;
    let mut occurrences = Vec::with_capacity(capacity);
    for symbol in &analysis.symbols {
        let range = symbol
            .identifier_range()
            .ok_or(DesignerXmlSourceEvidenceError::MissingExactRange)?;
        occurrences.push(SourceOccurrence::new(
            document_id.clone(),
            version,
            SourceByteRange::new(range.start_byte(), range.end_byte())?,
            SourceOccurrenceKind::Declaration,
            source_token(source, range.start_byte(), range.end_byte())?,
            Some(symbol.id().clone()),
            SourceOccurrenceResolution::Unique,
        )?);
    }
    for call in &analysis.calls {
        let range = call
            .identifier_range()
            .ok_or(DesignerXmlSourceEvidenceError::MissingExactRange)?;
        let (kind, mapped_target, resolution) = map_call(
            call.kind(),
            call.target_symbol().as_str(),
            analysis,
            available,
        );
        occurrences.push(SourceOccurrence::new_with_lexical_owner(
            document_id.clone(),
            version,
            SourceByteRange::new(range.start_byte(), range.end_byte())?,
            kind,
            source_token(source, range.start_byte(), range.end_byte())?,
            lexical_owner_token(call.target_symbol().as_str()).map(str::to_owned),
            mapped_target,
            resolution,
        )?);
    }
    SourceDocument::new(
        document_id,
        SourceFormat::DesignerXml,
        module_role(module.kind()),
        roots.confine(module.source().artifact_path())?,
        raw.to_vec(),
        occurrences,
        SourceEvidenceCompleteness::BslCallableRenameV1,
    )
    .map_err(Into::into)
}

fn map_call(
    kind: Option<BslCallKind>,
    target: &str,
    current: &AnalyzedModule<'_>,
    available: &[AnalyzedModule<'_>],
) -> (
    SourceOccurrenceKind,
    Option<EntityId>,
    SourceOccurrenceResolution,
) {
    match kind {
        Some(BslCallKind::Local) => mapped(
            SourceOccurrenceKind::LocalCall,
            current
                .symbols
                .iter()
                .filter(|symbol| bsl_names_equal(symbol.name().as_str(), target))
                .map(|symbol| symbol.id().clone()),
        ),
        Some(BslCallKind::Qualified) => {
            let Some((module_name, symbol_name)) = split_qualified(target) else {
                return (
                    SourceOccurrenceKind::QualifiedCall,
                    None,
                    SourceOccurrenceResolution::Unsupported,
                );
            };
            mapped(
                SourceOccurrenceKind::QualifiedCall,
                available
                    .iter()
                    .filter(|module| {
                        bsl_names_equal(module.descriptor.name().as_str(), module_name)
                    })
                    .flat_map(|module| &module.symbols)
                    .filter(|symbol| {
                        symbol.is_exported() && bsl_names_equal(symbol.name().as_str(), symbol_name)
                    })
                    .map(|symbol| symbol.id().clone()),
            )
        }
        Some(BslCallKind::Unsupported) | None => (
            if target.contains('.') {
                SourceOccurrenceKind::QualifiedCall
            } else {
                SourceOccurrenceKind::LocalCall
            },
            None,
            SourceOccurrenceResolution::Unsupported,
        ),
    }
}

fn mapped(
    kind: SourceOccurrenceKind,
    candidates: impl IntoIterator<Item = EntityId>,
) -> (
    SourceOccurrenceKind,
    Option<EntityId>,
    SourceOccurrenceResolution,
) {
    let mut candidates = candidates.into_iter();
    match (candidates.next(), candidates.next()) {
        (None, _) => (kind, None, SourceOccurrenceResolution::Unresolved),
        (Some(target), None) => (kind, Some(target), SourceOccurrenceResolution::Unique),
        (Some(_), Some(_)) => (kind, None, SourceOccurrenceResolution::Ambiguous),
    }
}

fn split_qualified(value: &str) -> Option<(&str, &str)> {
    let (module, symbol) = value.split_once('.')?;
    (!module.is_empty() && !symbol.is_empty() && !symbol.contains('.')).then_some((module, symbol))
}

fn lexical_owner_token(value: &str) -> Option<&str> {
    let (qualifier, _) = value.rsplit_once('.')?;
    qualifier
        .rsplit('.')
        .next()
        .filter(|owner| !owner.is_empty())
}

fn source_token(
    source: &str,
    start: usize,
    end: usize,
) -> Result<String, DesignerXmlSourceEvidenceError> {
    source
        .get(start..end)
        .map(str::to_owned)
        .ok_or(DesignerXmlSourceEvidenceError::MissingExactRange)
}

const fn module_role(kind: DesignerXmlModuleKind) -> BslModuleRole {
    match kind {
        DesignerXmlModuleKind::Object => BslModuleRole::Object,
        DesignerXmlModuleKind::Manager => BslModuleRole::Manager,
        DesignerXmlModuleKind::Common => BslModuleRole::Common,
    }
}

struct ConfinedRoots {
    workspace: PathBuf,
    configuration: PathBuf,
    configuration_relative: Option<SourcePath>,
}

impl ConfinedRoots {
    fn new(
        workspace_root: &Path,
        project_root: &Path,
    ) -> Result<Self, DesignerXmlSourceEvidenceError> {
        let workspace = workspace_root.canonicalize().map_err(|source| {
            DesignerXmlSourceEvidenceError::InspectPath(workspace_root.to_path_buf(), source)
        })?;
        let configuration = project_root.canonicalize().map_err(|source| {
            DesignerXmlSourceEvidenceError::InspectPath(project_root.to_path_buf(), source)
        })?;
        let relative = configuration
            .strip_prefix(&workspace)
            .map_err(|_| DesignerXmlSourceEvidenceError::EscapingPath(configuration.clone()))?;
        if !safe_relative(relative) {
            return Err(DesignerXmlSourceEvidenceError::EscapingPath(configuration));
        }
        let configuration_relative = (!relative.as_os_str().is_empty())
            .then(|| source_path(relative))
            .transpose()?;
        Ok(Self {
            workspace,
            configuration,
            configuration_relative,
        })
    }

    fn confine(&self, path: &Path) -> Result<ConfinedSourcePath, DesignerXmlSourceEvidenceError> {
        let canonical = path.canonicalize().map_err(|source| {
            DesignerXmlSourceEvidenceError::InspectPath(path.to_path_buf(), source)
        })?;
        if !canonical.starts_with(&self.configuration) {
            return Err(DesignerXmlSourceEvidenceError::EscapingPath(canonical));
        }
        let relative = canonical
            .strip_prefix(&self.workspace)
            .map_err(|_| DesignerXmlSourceEvidenceError::EscapingPath(canonical.clone()))?;
        let relative = source_path(relative)?;
        match &self.configuration_relative {
            Some(configuration_root) => ConfinedSourcePath::new(relative, configuration_root),
            None => ConfinedSourcePath::new_at_workspace_root(relative),
        }
        .map_err(Into::into)
    }
}

fn safe_relative(path: &Path) -> bool {
    path.components()
        .all(|component| matches!(component, Component::Normal(_)))
}

fn source_path(path: &Path) -> Result<SourcePath, DesignerXmlSourceEvidenceError> {
    if !safe_relative(path) {
        return Err(DesignerXmlSourceEvidenceError::EscapingPath(
            path.to_path_buf(),
        ));
    }
    SourcePath::new(path.to_string_lossy().replace('\\', "/"))
        .map_err(|_| DesignerXmlSourceEvidenceError::InvalidPath(path.to_path_buf()))
}

#[derive(Debug)]
pub enum DesignerXmlSourceEvidenceError {
    ParseDeclarations(BslParseError),
    ParseCalls(oneagent_bsl::BslCallError),
    Domain(SourceEvidenceError),
    InspectPath(PathBuf, std::io::Error),
    EscapingPath(PathBuf),
    InvalidPath(PathBuf),
    InvalidCapturedUtf8(PathBuf),
    MissingExactRange,
}

impl Display for DesignerXmlSourceEvidenceError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ParseDeclarations(error) => write!(
                formatter,
                "failed to parse captured Designer XML declarations: {error}"
            ),
            Self::ParseCalls(error) => write!(
                formatter,
                "failed to parse captured Designer XML calls: {error}"
            ),
            Self::Domain(error) => {
                write!(formatter, "invalid Designer XML source evidence: {error}")
            }
            Self::InspectPath(path, error) => write!(
                formatter,
                "failed to inspect Designer XML source path {}: {error}",
                path.display()
            ),
            Self::EscapingPath(path) => write!(
                formatter,
                "Designer XML source path escapes its Workspace or Configuration root: {}",
                path.display()
            ),
            Self::InvalidPath(path) => write!(
                formatter,
                "Designer XML source path is invalid: {}",
                path.display()
            ),
            Self::InvalidCapturedUtf8(path) => write!(
                formatter,
                "captured Designer XML source is not UTF-8: {}",
                path.display()
            ),
            Self::MissingExactRange => {
                formatter.write_str("captured Designer XML BSL occurrence has no exact range")
            }
        }
    }
}

impl std::error::Error for DesignerXmlSourceEvidenceError {}

impl From<BslParseError> for DesignerXmlSourceEvidenceError {
    fn from(value: BslParseError) -> Self {
        Self::ParseDeclarations(value)
    }
}
impl From<oneagent_bsl::BslCallError> for DesignerXmlSourceEvidenceError {
    fn from(value: oneagent_bsl::BslCallError) -> Self {
        Self::ParseCalls(value)
    }
}
impl From<SourceEvidenceError> for DesignerXmlSourceEvidenceError {
    fn from(value: SourceEvidenceError) -> Self {
        Self::Domain(value)
    }
}

#[cfg(test)]
mod admission_tests {
    use oneagent_analysis::refactoring::{
        MAX_SOURCE_BYTES_PER_CONFIGURATION, MAX_SOURCE_DOCUMENT_BYTES,
        MAX_SOURCE_DOCUMENTS_PER_CONFIGURATION, MAX_SOURCE_OCCURRENCES_PER_DOCUMENT,
        SourceEvidenceErrorKind,
    };

    use super::{
        DesignerXmlSourceEvidenceError, admit_document_lengths, admitted_occurrence_capacity,
    };

    fn assert_bound(error: &DesignerXmlSourceEvidenceError, actual: usize, maximum: usize) {
        let DesignerXmlSourceEvidenceError::Domain(error) = error else {
            panic!("admission must preserve the source-evidence domain error");
        };
        assert_eq!(error.kind(), SourceEvidenceErrorKind::BoundExceeded);
        assert_eq!(error.actual(), Some(actual));
        assert_eq!(error.maximum(), Some(maximum));
    }

    #[test]
    fn production_admission_accepts_exact_and_rejects_one_over_before_collection() {
        let exact_documents = admit_document_lengths(
            MAX_SOURCE_DOCUMENTS_PER_CONFIGURATION,
            (0..MAX_SOURCE_DOCUMENTS_PER_CONFIGURATION).map(|_| Ok(0)),
        )
        .expect("exact document count must be admitted");
        assert_eq!(exact_documents, 0);

        let count_error = admit_document_lengths(
            MAX_SOURCE_DOCUMENTS_PER_CONFIGURATION + 1,
            std::iter::once_with(|| -> Result<usize, DesignerXmlSourceEvidenceError> {
                panic!("over-bound document count must fail before iteration")
            }),
        )
        .expect_err("one-over document count must fail");
        assert_bound(
            &count_error,
            MAX_SOURCE_DOCUMENTS_PER_CONFIGURATION + 1,
            MAX_SOURCE_DOCUMENTS_PER_CONFIGURATION,
        );

        let exact_aggregate = admit_document_lengths(
            MAX_SOURCE_BYTES_PER_CONFIGURATION / MAX_SOURCE_DOCUMENT_BYTES,
            (0..MAX_SOURCE_BYTES_PER_CONFIGURATION / MAX_SOURCE_DOCUMENT_BYTES)
                .map(|_| Ok(MAX_SOURCE_DOCUMENT_BYTES)),
        )
        .expect("exact aggregate byte count must be admitted");
        assert_eq!(exact_aggregate, MAX_SOURCE_BYTES_PER_CONFIGURATION);

        let aggregate_documents =
            MAX_SOURCE_BYTES_PER_CONFIGURATION / MAX_SOURCE_DOCUMENT_BYTES + 1;
        let aggregate_error = admit_document_lengths(
            aggregate_documents,
            (0..aggregate_documents).map(|index| {
                Ok(if index + 1 == aggregate_documents {
                    1
                } else {
                    MAX_SOURCE_DOCUMENT_BYTES
                })
            }),
        )
        .expect_err("one-over aggregate byte count must fail");
        assert_bound(
            &aggregate_error,
            MAX_SOURCE_BYTES_PER_CONFIGURATION + 1,
            MAX_SOURCE_BYTES_PER_CONFIGURATION,
        );

        assert_eq!(
            admitted_occurrence_capacity(MAX_SOURCE_OCCURRENCES_PER_DOCUMENT, 0)
                .expect("exact occurrence count must be admitted"),
            MAX_SOURCE_OCCURRENCES_PER_DOCUMENT
        );
        let occurrence_error = admitted_occurrence_capacity(MAX_SOURCE_OCCURRENCES_PER_DOCUMENT, 1)
            .expect_err("one-over occurrence count must fail");
        assert_bound(
            &occurrence_error,
            MAX_SOURCE_OCCURRENCES_PER_DOCUMENT + 1,
            MAX_SOURCE_OCCURRENCES_PER_DOCUMENT,
        );
    }
}
