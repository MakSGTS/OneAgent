//! Immutable source-document evidence produced from captured EDT modules.

use std::fmt::{Display, Formatter};
use std::path::{Component, Path, PathBuf};

use oneagent_analysis::refactoring::{
    BslModuleRole, ConfinedSourcePath, SourceByteRange, SourceContentVersion, SourceDocument,
    SourceDocumentId, SourceEvidenceCompleteness, SourceEvidenceError, SourceEvidenceSet,
    SourceFormat, SourceOccurrence, SourceOccurrenceKind, SourceOccurrenceResolution,
};
use oneagent_bsl::{BslCallKind, bsl_names_equal};
use oneagent_common::{EntityId, SourcePath};

use crate::{
    AnalyzedBslModule, EdtBslGraphError, EdtModuleDescriptor, EdtModuleKind, analyze_module,
};

pub(crate) fn build_source_evidence(
    workspace_root: &Path,
    project_root: &Path,
    configuration_id: &EntityId,
    modules: &[EdtModuleDescriptor],
) -> Result<SourceEvidenceSet, EdtSourceEvidenceError> {
    let analyzed = modules
        .iter()
        .map(analyze_module)
        .collect::<Result<Vec<_>, _>>()?;
    let roots = ConfinedRoots::new(workspace_root, project_root)?;
    let mut documents = Vec::with_capacity(modules.len());
    for (module, analysis) in modules.iter().zip(&analyzed) {
        let raw = module.raw_source().ok_or_else(|| {
            EdtSourceEvidenceError::MissingCapturedSource(module.path().to_path_buf())
        })?;
        documents.push(build_document(
            &roots,
            configuration_id,
            module,
            analysis,
            &analyzed,
            raw,
        )?);
    }
    SourceEvidenceSet::new(configuration_id.clone(), documents).map_err(Into::into)
}

fn build_document(
    roots: &ConfinedRoots,
    configuration_id: &EntityId,
    module: &EdtModuleDescriptor,
    analysis: &AnalyzedBslModule,
    available: &[AnalyzedBslModule],
    raw: &[u8],
) -> Result<SourceDocument, EdtSourceEvidenceError> {
    let document_id = SourceDocumentId::new(configuration_id.clone(), module.id().clone())?;
    let version = SourceContentVersion::from_bytes(raw);
    let source = std::str::from_utf8(raw)
        .map_err(|_| EdtSourceEvidenceError::InvalidCapturedUtf8(module.path().to_path_buf()))?;
    let mut occurrences = Vec::with_capacity(analysis.symbols().len() + analysis.calls().len());

    for symbol in analysis.symbols() {
        let range = symbol
            .identifier_range()
            .ok_or(EdtSourceEvidenceError::MissingExactRange)?;
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

    for call in analysis.calls() {
        let range = call
            .identifier_range()
            .ok_or(EdtSourceEvidenceError::MissingExactRange)?;
        let (kind, mapped_target_id, resolution) = map_call(
            call.kind(),
            call.target_symbol().as_str(),
            analysis,
            available,
        );
        occurrences.push(SourceOccurrence::new(
            document_id.clone(),
            version,
            SourceByteRange::new(range.start_byte(), range.end_byte())?,
            kind,
            source_token(source, range.start_byte(), range.end_byte())?,
            mapped_target_id,
            resolution,
        )?);
    }

    SourceDocument::new(
        document_id,
        SourceFormat::Edt,
        module_role(module.kind()),
        roots.confine(module.path())?,
        raw.to_vec(),
        occurrences,
        SourceEvidenceCompleteness::BslCallableRenameV1,
    )
    .map_err(Into::into)
}

fn map_call(
    kind: Option<BslCallKind>,
    target: &str,
    current: &AnalyzedBslModule,
    available: &[AnalyzedBslModule],
) -> (
    SourceOccurrenceKind,
    Option<EntityId>,
    SourceOccurrenceResolution,
) {
    match kind {
        Some(BslCallKind::Local) => {
            let candidates = current
                .symbols()
                .iter()
                .filter(|symbol| bsl_names_equal(symbol.name().as_str(), target))
                .map(|symbol| symbol.id().clone())
                .collect::<Vec<_>>();
            mapped(SourceOccurrenceKind::LocalCall, candidates)
        }
        Some(BslCallKind::Qualified) => {
            let Some((module_name, symbol_name)) = split_qualified(target) else {
                return (
                    SourceOccurrenceKind::QualifiedCall,
                    None,
                    SourceOccurrenceResolution::Unsupported,
                );
            };
            let candidates = available
                .iter()
                .filter(|module| bsl_names_equal(module.module_name().as_str(), module_name))
                .flat_map(AnalyzedBslModule::symbols)
                .filter(|symbol| {
                    symbol.is_exported() && bsl_names_equal(symbol.name().as_str(), symbol_name)
                })
                .map(|symbol| symbol.id().clone())
                .collect::<Vec<_>>();
            mapped(SourceOccurrenceKind::QualifiedCall, candidates)
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

fn source_token(source: &str, start: usize, end: usize) -> Result<String, EdtSourceEvidenceError> {
    source
        .get(start..end)
        .map(str::to_owned)
        .ok_or(EdtSourceEvidenceError::MissingExactRange)
}

const fn module_role(kind: EdtModuleKind) -> BslModuleRole {
    match kind {
        EdtModuleKind::Object => BslModuleRole::Object,
        EdtModuleKind::Manager => BslModuleRole::Manager,
        EdtModuleKind::Common => BslModuleRole::Common,
        EdtModuleKind::Form => BslModuleRole::Form,
        EdtModuleKind::Command => BslModuleRole::Command,
    }
}

struct ConfinedRoots {
    workspace: PathBuf,
    configuration: PathBuf,
    configuration_relative: Option<SourcePath>,
}

impl ConfinedRoots {
    fn new(workspace_root: &Path, project_root: &Path) -> Result<Self, EdtSourceEvidenceError> {
        let workspace = workspace_root.canonicalize().map_err(|source| {
            EdtSourceEvidenceError::InspectPath(workspace_root.to_path_buf(), source)
        })?;
        let configuration = project_root.canonicalize().map_err(|source| {
            EdtSourceEvidenceError::InspectPath(project_root.to_path_buf(), source)
        })?;
        let relative = configuration
            .strip_prefix(&workspace)
            .map_err(|_| EdtSourceEvidenceError::EscapingPath(configuration.clone()))?;
        if !safe_relative(relative) {
            return Err(EdtSourceEvidenceError::EscapingPath(configuration));
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

    fn confine(&self, path: &Path) -> Result<ConfinedSourcePath, EdtSourceEvidenceError> {
        let canonical = path
            .canonicalize()
            .map_err(|source| EdtSourceEvidenceError::InspectPath(path.to_path_buf(), source))?;
        if !canonical.starts_with(&self.configuration) {
            return Err(EdtSourceEvidenceError::EscapingPath(canonical));
        }
        let relative = canonical
            .strip_prefix(&self.workspace)
            .map_err(|_| EdtSourceEvidenceError::EscapingPath(canonical.clone()))?;
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

fn source_path(path: &Path) -> Result<SourcePath, EdtSourceEvidenceError> {
    if !safe_relative(path) {
        return Err(EdtSourceEvidenceError::EscapingPath(path.to_path_buf()));
    }
    SourcePath::new(path.to_string_lossy().replace('\\', "/"))
        .map_err(|_| EdtSourceEvidenceError::InvalidPath(path.to_path_buf()))
}

#[derive(Debug)]
pub enum EdtSourceEvidenceError {
    Analyze(EdtBslGraphError),
    Domain(SourceEvidenceError),
    InspectPath(PathBuf, std::io::Error),
    EscapingPath(PathBuf),
    InvalidPath(PathBuf),
    MissingCapturedSource(PathBuf),
    InvalidCapturedUtf8(PathBuf),
    MissingExactRange,
}

impl Display for EdtSourceEvidenceError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Analyze(error) => {
                write!(formatter, "failed to analyze captured EDT source: {error}")
            }
            Self::Domain(error) => write!(formatter, "invalid EDT source evidence: {error}"),
            Self::InspectPath(path, error) => write!(
                formatter,
                "failed to inspect EDT source path {}: {error}",
                path.display()
            ),
            Self::EscapingPath(path) => write!(
                formatter,
                "EDT source path escapes its Workspace or Configuration root: {}",
                path.display()
            ),
            Self::InvalidPath(path) => {
                write!(formatter, "EDT source path is invalid: {}", path.display())
            }
            Self::MissingCapturedSource(path) => write!(
                formatter,
                "EDT source was not retained during discovery: {}",
                path.display()
            ),
            Self::InvalidCapturedUtf8(path) => write!(
                formatter,
                "captured EDT source is not UTF-8: {}",
                path.display()
            ),
            Self::MissingExactRange => {
                formatter.write_str("captured EDT BSL occurrence has no exact range")
            }
        }
    }
}

impl std::error::Error for EdtSourceEvidenceError {}

impl From<EdtBslGraphError> for EdtSourceEvidenceError {
    fn from(value: EdtBslGraphError) -> Self {
        Self::Analyze(value)
    }
}

impl From<SourceEvidenceError> for EdtSourceEvidenceError {
    fn from(value: SourceEvidenceError) -> Self {
        Self::Domain(value)
    }
}
