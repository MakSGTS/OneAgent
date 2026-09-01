//! Integration between EDT module files, BSL declarations and the semantic graph.

use oneagent_bsl::{
    BslCall, BslCallError, BslCallExtractor, BslCallResolver, BslDeclarationExtractor,
    BslModuleSymbols, BslParseError, BslQuery, BslQueryError, BslQueryExtractor, BslSymbol,
    BslSymbolKind, CrossModuleCallResolver, LineBslCallExtractor, LineBslDeclarationExtractor,
    LineBslQueryExtractor, LocalBslCallResolver, QualifiedBslCallResolver, QueryLanguageDiagnostic,
    QueryLanguageDiagnosticKind, QueryLanguageParser, UnresolvedBslCall, UnresolvedCrossModuleCall,
};
use oneagent_common::{
    EntityId, EntityName, EntityNameError, SourceLocation, SourcePath, SourcePosition, SourceSpan,
};
use oneagent_graph::{
    Confidence, EdgeKind, FactOrigin, GraphEdge, GraphError, NodeKind, ProducerId, Provenance,
    ResolutionError, ResolutionState, SemanticDiagnostic, SemanticDiagnosticCode,
    SemanticDiagnosticKind, SemanticDiagnosticSeverity, SemanticGraph, SemanticReference,
    SemanticReferenceOutcome, SemanticReferenceRequest, SemanticReferenceRequestLedger,
    SemanticReferenceRequestOutcome, SemanticReferenceStatistics,
};
use oneagent_metadata::MetadataKind;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::Path;

use crate::EdtModuleDescriptor;
use crate::query_source_resolution::{
    QuerySourceRequestError, QuerySourceResolutionIndex, WorkspaceResolutionScope,
    collect_query_source_requests,
};

const EDT_BSL_GRAPH_PRODUCER: &str = "oneagent.edt.bsl-graph";
const QUERY_LANGUAGE_PARSER_STAGE: &str = "oneagent.bsl.query-language-parser";
const QUERY_SOURCE_RESOLVER_STAGE: &str = "oneagent.edt.query-source-resolution";
const QUERY_READS_CONTRIBUTOR: &str = "oneagent.edt.query-reads";
const QUERY_DEPENDENCY_CONTRIBUTOR: &str = "oneagent.edt.query-dependency";
const QUERY_SOURCE_DIAGNOSTIC_CONTRIBUTOR: &str = "oneagent.edt.query-source-diagnostic";

/// Parsed declarations and calls collected from one EDT BSL module.
///
/// The analysis result does not mutate the semantic graph. This allows all
/// configuration modules to be analyzed before call relations are resolved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnalyzedBslModule {
    module_id: EntityId,
    module_name: EntityName,
    symbols: Vec<BslSymbol>,
    calls: Vec<BslCall>,
    queries: Vec<BslQuery>,
    source: Option<EntityId>,
    source_path: Option<SourcePath>,
}

impl AnalyzedBslModule {
    /// Creates an analyzed BSL module.
    #[must_use]
    pub const fn new(
        module_id: EntityId,
        module_name: EntityName,
        symbols: Vec<BslSymbol>,
        calls: Vec<BslCall>,
    ) -> Self {
        Self::new_with_source(module_id, module_name, symbols, calls, None)
    }

    /// Creates an analyzed BSL module with a source identifier.
    #[must_use]
    pub const fn new_with_source(
        module_id: EntityId,
        module_name: EntityName,
        symbols: Vec<BslSymbol>,
        calls: Vec<BslCall>,
        source: Option<EntityId>,
    ) -> Self {
        Self::new_with_source_and_queries(
            module_id,
            module_name,
            symbols,
            calls,
            Vec::new(),
            source,
        )
    }

    /// Creates an analyzed BSL module with queries and a source identifier.
    #[must_use]
    pub const fn new_with_source_and_queries(
        module_id: EntityId,
        module_name: EntityName,
        symbols: Vec<BslSymbol>,
        calls: Vec<BslCall>,
        queries: Vec<BslQuery>,
        source: Option<EntityId>,
    ) -> Self {
        Self::new_with_source_queries_and_path(
            module_id,
            module_name,
            symbols,
            calls,
            queries,
            source,
            None,
        )
    }

    const fn new_with_source_queries_and_path(
        module_id: EntityId,
        module_name: EntityName,
        symbols: Vec<BslSymbol>,
        calls: Vec<BslCall>,
        queries: Vec<BslQuery>,
        source: Option<EntityId>,
        source_path: Option<SourcePath>,
    ) -> Self {
        Self {
            module_id,
            module_name,
            symbols,
            calls,
            queries,
            source,
            source_path,
        }
    }

    /// Returns the module identifier.
    #[must_use]
    pub const fn module_id(&self) -> &EntityId {
        &self.module_id
    }

    /// Returns the module name.
    #[must_use]
    pub const fn module_name(&self) -> &EntityName {
        &self.module_name
    }

    /// Returns declarations collected from the module.
    #[must_use]
    pub fn symbols(&self) -> &[BslSymbol] {
        &self.symbols
    }

    /// Returns calls collected from the module.
    #[must_use]
    pub fn calls(&self) -> &[BslCall] {
        &self.calls
    }

    /// Returns static query declarations collected from the module.
    #[must_use]
    pub fn queries(&self) -> &[BslQuery] {
        &self.queries
    }

    /// Returns the source identifier used for graph provenance.
    #[must_use]
    pub const fn source(&self) -> Option<&EntityId> {
        self.source.as_ref()
    }

    /// Returns the validated source evidence path when analysis observed one.
    #[must_use]
    pub const fn source_path(&self) -> Option<&SourcePath> {
        self.source_path.as_ref()
    }

    fn as_module_symbols(&self) -> BslModuleSymbols {
        BslModuleSymbols::new(
            self.module_id.clone(),
            self.module_name.clone(),
            self.symbols.clone(),
        )
    }
}

/// Reads and analyzes one EDT BSL module without changing the semantic graph.
///
/// # Errors
///
/// Returns an error when the module cannot be read or parsed.
pub fn analyze_module(module: &EdtModuleDescriptor) -> Result<AnalyzedBslModule, EdtBslGraphError> {
    let fallback;
    let raw_source = if let Some(raw_source) = module.raw_source() {
        raw_source
    } else {
        fallback = fs::read(module.path()).map_err(|source| EdtBslGraphError::ReadModule {
            path: module.path().to_path_buf(),
            source,
        })?;
        &fallback
    };
    let source =
        std::str::from_utf8(raw_source).map_err(|source| EdtBslGraphError::ReadModule {
            path: module.path().to_path_buf(),
            source: std::io::Error::new(std::io::ErrorKind::InvalidData, source),
        })?;

    let symbols = LineBslDeclarationExtractor
        .extract(module.id(), source)
        .map_err(EdtBslGraphError::ParseDeclarations)?;

    let calls = LineBslCallExtractor
        .extract_calls(module.id(), source)
        .map_err(EdtBslGraphError::ParseCalls)?;

    let queries = LineBslQueryExtractor
        .extract_queries(module.id(), source)
        .map_err(EdtBslGraphError::ParseQueries)?;

    let source_path = SourcePath::new(
        module
            .path()
            .to_str()
            .ok_or(EdtBslGraphError::InvalidSourceLocation)?,
    )
    .map_err(|_| EdtBslGraphError::InvalidSourceLocation)?;

    Ok(AnalyzedBslModule::new_with_source_queries_and_path(
        module.id().clone(),
        module.name().clone(),
        symbols,
        calls,
        queries,
        Some(source_id_from_path(module.path())?),
        Some(source_path),
    ))
}

/// Analyzes all modules before adding declarations and call relations.
///
/// Processing is performed in two passes:
///
/// 1. Insert declarations from every module.
/// 2. Resolve local and qualified cross-module calls.
///
/// Module nodes must already exist in `graph`.
///
/// # Errors
///
/// Returns an error when a module cannot be read or parsed, or when a graph
/// edge references a node absent from the semantic graph.
pub fn add_configuration_module_symbols(
    graph: &mut SemanticGraph,
    modules: &[EdtModuleDescriptor],
) -> Result<usize, EdtBslGraphError> {
    let mut diagnostics = BTreeSet::new();
    let mut reference_statistics = SemanticReferenceStatistics::new();

    add_configuration_module_symbols_with_diagnostics(
        graph,
        modules,
        &mut diagnostics,
        &mut reference_statistics,
    )
}

pub(crate) fn add_configuration_module_symbols_with_diagnostics(
    graph: &mut SemanticGraph,
    modules: &[EdtModuleDescriptor],
    diagnostics: &mut BTreeSet<SemanticDiagnostic>,
    reference_statistics: &mut SemanticReferenceStatistics,
) -> Result<usize, EdtBslGraphError> {
    let mut reference_requests = SemanticReferenceRequestLedger::new();
    add_configuration_module_symbols_with_diagnostics_in_scope(
        graph,
        modules,
        WorkspaceResolutionScope::Partial,
        diagnostics,
        reference_statistics,
        &mut reference_requests,
    )
}

pub(crate) fn add_configuration_module_symbols_with_diagnostics_in_scope(
    graph: &mut SemanticGraph,
    modules: &[EdtModuleDescriptor],
    workspace_scope: WorkspaceResolutionScope,
    diagnostics: &mut BTreeSet<SemanticDiagnostic>,
    reference_statistics: &mut SemanticReferenceStatistics,
    reference_requests: &mut SemanticReferenceRequestLedger,
) -> Result<usize, EdtBslGraphError> {
    let analyzed_modules = modules
        .iter()
        .map(analyze_module)
        .collect::<Result<Vec<_>, _>>()?;

    add_analyzed_modules(
        graph,
        &analyzed_modules,
        workspace_scope,
        diagnostics,
        reference_statistics,
        reference_requests,
    )
}

/// Adds previously analyzed modules to the semantic graph in two passes.
///
/// Module nodes must already exist in `graph`.
///
/// # Errors
///
/// Returns an error when an inserted graph edge references a missing node.
fn add_analyzed_modules(
    graph: &mut SemanticGraph,
    modules: &[AnalyzedBslModule],
    workspace_scope: WorkspaceResolutionScope,
    diagnostics: &mut BTreeSet<SemanticDiagnostic>,
    reference_statistics: &mut SemanticReferenceStatistics,
    reference_requests: &mut SemanticReferenceRequestLedger,
) -> Result<usize, EdtBslGraphError> {
    // Pass 1: insert declarations from every module.
    for module in modules {
        insert_declarations(graph, module)?;
        insert_queries(graph, module)?;
    }

    insert_query_reads(
        graph,
        modules,
        workspace_scope,
        diagnostics,
        reference_statistics,
        reference_requests,
    )?;

    let available_modules = modules
        .iter()
        .map(AnalyzedBslModule::as_module_symbols)
        .collect::<Vec<_>>();

    // Pass 2: resolve calls only after every declaration is present.
    for (module, module_symbols) in modules.iter().zip(&available_modules) {
        insert_local_calls(graph, module, diagnostics, reference_statistics)?;
        insert_cross_module_calls(
            graph,
            module,
            module_symbols,
            &available_modules,
            diagnostics,
            reference_statistics,
        )?;
    }

    Ok(modules.iter().map(|module| module.symbols().len()).sum())
}

/// Adds declarations and local calls from one module.
///
/// This compatibility wrapper preserves the previous public API. Qualified
/// calls cannot be resolved unless modules are supplied together through
/// [`add_configuration_module_symbols`].
///
/// # Errors
///
/// Returns an error when the module cannot be read, parsed, resolved or
/// inserted into the graph.
pub fn add_module_symbols(
    graph: &mut SemanticGraph,
    module: &EdtModuleDescriptor,
) -> Result<usize, EdtBslGraphError> {
    let analyzed_module = analyze_module(module)?;
    let mut diagnostics = BTreeSet::new();
    let mut reference_statistics = SemanticReferenceStatistics::new();
    let mut reference_requests = SemanticReferenceRequestLedger::new();
    add_analyzed_modules(
        graph,
        std::slice::from_ref(&analyzed_module),
        WorkspaceResolutionScope::Partial,
        &mut diagnostics,
        &mut reference_statistics,
        &mut reference_requests,
    )
}

fn insert_declarations(
    graph: &mut SemanticGraph,
    module: &AnalyzedBslModule,
) -> Result<(), EdtBslGraphError> {
    for symbol in module.symbols() {
        let node_kind = match symbol.kind() {
            BslSymbolKind::Procedure => NodeKind::Procedure,
            BslSymbolKind::Function => NodeKind::Function,
        };

        graph.insert_node_with_provenance(
            symbol.id().clone(),
            symbol.name().clone(),
            node_kind,
            declared_provenance_with_location(
                module.source(),
                declaration_location(module, symbol.line())?,
            ),
        );

        graph
            .insert_edge_with_provenance(
                module.module_id().clone(),
                symbol.id().clone(),
                EdgeKind::Contains,
                declared_provenance_with_location(
                    module.source(),
                    declaration_location(module, symbol.line())?,
                ),
            )
            .map_err(EdtBslGraphError::Graph)?;
    }

    Ok(())
}

fn insert_queries(
    graph: &mut SemanticGraph,
    module: &AnalyzedBslModule,
) -> Result<(), EdtBslGraphError> {
    for query in module.queries() {
        graph.insert_node_with_provenance(
            query.id().clone(),
            query.binding_name().clone(),
            NodeKind::Query,
            query_provenance(module, query)?,
        );

        graph
            .insert_edge_with_provenance(
                query.owner_id().clone(),
                query.id().clone(),
                EdgeKind::Contains,
                query_provenance(module, query)?,
            )
            .map_err(EdtBslGraphError::Graph)?;
    }

    Ok(())
}

fn insert_query_reads(
    graph: &mut SemanticGraph,
    modules: &[AnalyzedBslModule],
    workspace_scope: WorkspaceResolutionScope,
    diagnostics: &mut BTreeSet<SemanticDiagnostic>,
    reference_statistics: &mut SemanticReferenceStatistics,
    reference_requests: &mut SemanticReferenceRequestLedger,
) -> Result<(), EdtBslGraphError> {
    let mut collected_requests = SemanticReferenceRequestLedger::new();

    for module in modules {
        for query in module.queries() {
            let Some(query_node) = graph.node(query.id()) else {
                return Err(EdtBslGraphError::Graph(GraphError::MissingNode(
                    query.id().clone(),
                )));
            };
            if query_node.kind() != NodeKind::Query {
                return Err(EdtBslGraphError::Graph(GraphError::MissingNode(
                    query.id().clone(),
                )));
            }

            let parse_result = QueryLanguageParser.parse(query.text());
            if !parse_result.is_source_set_complete()
                || parse_result.program().is_none()
                || !parse_result.diagnostics().is_empty()
            {
                for diagnostic in parse_result.diagnostics() {
                    record_query_language_diagnostic(
                        module,
                        query,
                        *diagnostic,
                        diagnostics,
                        reference_statistics,
                    )?;
                }
                continue;
            }

            let Some(collected) =
                collect_query_source_requests(&parse_result, module.source(), query)
                    .map_err(EdtBslGraphError::from)?
            else {
                continue;
            };
            for request in collected.requests() {
                collected_requests
                    .insert(request.clone())
                    .map_err(QuerySourceRequestError::Request)
                    .map_err(EdtBslGraphError::from)?;
            }
        }
    }

    let terminal_requests = QuerySourceResolutionIndex::new(graph)
        .resolve_requests(&collected_requests, workspace_scope)
        .map_err(EdtBslGraphError::from)?;
    let mut evidence_by_edge = BTreeMap::<(EntityId, EntityId, EdgeKind), Vec<Provenance>>::new();

    for request in terminal_requests.requests() {
        project_query_source_request(graph, request, diagnostics, &mut evidence_by_edge)?;
        reference_requests
            .insert(request.clone())
            .map_err(QuerySourceRequestError::Request)
            .map_err(EdtBslGraphError::from)?;
    }

    for ((query_id, target_id, kind), mut provenance) in evidence_by_edge {
        provenance.sort_by(|left, right| left.source().cmp(&right.source()));
        provenance.dedup();
        graph
            .insert_edge(GraphEdge::new_with_provenance(
                query_id, target_id, kind, provenance,
            ))
            .map_err(EdtBslGraphError::Graph)?;
    }

    Ok(())
}

fn project_query_source_request(
    graph: &SemanticGraph,
    request: &SemanticReferenceRequest,
    diagnostics: &mut BTreeSet<SemanticDiagnostic>,
    evidence_by_edge: &mut BTreeMap<(EntityId, EntityId, EdgeKind), Vec<Provenance>>,
) -> Result<(), EdtBslGraphError> {
    if request.outcome() != SemanticReferenceRequestOutcome::Resolved {
        return project_query_source_diagnostic(request, diagnostics);
    }
    let [target_id] = request.candidates() else {
        return Err(invalid_query_source_request(request));
    };
    let [expected_kind] = request.expected_kinds() else {
        return Err(invalid_query_source_request(request));
    };
    let target = graph
        .node(target_id)
        .ok_or_else(|| EdtBslGraphError::Graph(GraphError::MissingNode(target_id.clone())))?;
    if target.kind() != *expected_kind {
        return Err(invalid_query_source_request(request));
    }

    for (kind, producer, origin) in [
        (
            EdgeKind::Reads,
            QUERY_READS_CONTRIBUTOR,
            FactOrigin::Resolved,
        ),
        (
            EdgeKind::DependsOn,
            QUERY_DEPENDENCY_CONTRIBUTOR,
            FactOrigin::Derived,
        ),
    ] {
        evidence_by_edge
            .entry((request.source_node().clone(), target_id.clone(), kind))
            .or_default()
            .extend(query_request_projection_provenance(
                request,
                Some((target_id, *expected_kind)),
                kind,
                producer,
                origin,
                ResolutionState::Resolved,
            )?);
    }
    Ok(())
}

fn record_query_language_diagnostic(
    module: &AnalyzedBslModule,
    query: &BslQuery,
    parser_diagnostic: QueryLanguageDiagnostic,
    diagnostics: &mut BTreeSet<SemanticDiagnostic>,
    reference_statistics: &mut SemanticReferenceStatistics,
) -> Result<(), EdtBslGraphError> {
    let (code, kind, outcome) = match parser_diagnostic.kind() {
        QueryLanguageDiagnosticKind::MalformedSyntax => (
            SemanticDiagnosticCode::QueryLanguageMalformedSyntax,
            SemanticDiagnosticKind::QueryLanguageMalformedSyntax,
            SemanticReferenceOutcome::MalformedFormat,
        ),
        QueryLanguageDiagnosticKind::UnsupportedStructure => (
            SemanticDiagnosticCode::QueryLanguageUnsupportedStructure,
            SemanticDiagnosticKind::QueryLanguageUnsupportedStructure,
            SemanticReferenceOutcome::UnsupportedPrefix,
        ),
        QueryLanguageDiagnosticKind::UnsupportedPersistentNamespace => (
            SemanticDiagnosticCode::QueryLanguageUnsupportedPersistentNamespace,
            SemanticDiagnosticKind::QueryLanguageUnsupportedPersistentNamespace,
            SemanticReferenceOutcome::UnsupportedPrefix,
        ),
        QueryLanguageDiagnosticKind::VirtualTableSource => (
            SemanticDiagnosticCode::QueryLanguageVirtualTableSource,
            SemanticDiagnosticKind::QueryLanguageVirtualTableSource,
            SemanticReferenceOutcome::UnsupportedPrefix,
        ),
        QueryLanguageDiagnosticKind::TemporaryTableSource => (
            SemanticDiagnosticCode::QueryLanguageTemporaryTableSource,
            SemanticDiagnosticKind::QueryLanguageTemporaryTableSource,
            SemanticReferenceOutcome::UnsupportedPrefix,
        ),
        QueryLanguageDiagnosticKind::ExternalOrParameterDataSource => (
            SemanticDiagnosticCode::QueryLanguageExternalOrParameterDataSource,
            SemanticDiagnosticKind::QueryLanguageExternalOrParameterDataSource,
            SemanticReferenceOutcome::UnsupportedPrefix,
        ),
    };
    let location = parser_diagnostic.location();
    let provenance = query_diagnostic_provenance(
        module.source(),
        query,
        parser_diagnostic.kind().as_str(),
        location.start_byte(),
        location.end_byte(),
        FactOrigin::Parsed,
        ResolutionState::Unresolved,
    )?;

    diagnostics.insert(
        SemanticDiagnostic::new(
            code,
            SemanticDiagnosticSeverity::Error,
            kind,
            parser_diagnostic.message(),
            SemanticReference::Raw(query.text().to_owned()),
        )
        .with_source_node(query.id().clone())
        .with_provenance(vec![provenance]),
    );
    reference_statistics.record(outcome, true);
    Ok(())
}

fn project_query_source_diagnostic(
    request: &SemanticReferenceRequest,
    diagnostics: &mut BTreeSet<SemanticDiagnostic>,
) -> Result<(), EdtBslGraphError> {
    let (code, severity, kind, message) = match request.outcome() {
        SemanticReferenceRequestOutcome::MissingTarget => (
            SemanticDiagnosticCode::ReferenceUnresolved,
            SemanticDiagnosticSeverity::Error,
            SemanticDiagnosticKind::UnresolvedTarget,
            "query source metadata target could not be resolved",
        ),
        SemanticReferenceRequestOutcome::AmbiguousTarget => (
            SemanticDiagnosticCode::ReferenceAmbiguous,
            SemanticDiagnosticSeverity::Error,
            SemanticDiagnosticKind::AmbiguousTarget,
            "query source metadata target is ambiguous",
        ),
        SemanticReferenceRequestOutcome::IncompatibleTargetKind => (
            SemanticDiagnosticCode::ReferenceIncompatibleKind,
            SemanticDiagnosticSeverity::Error,
            SemanticDiagnosticKind::IncompatibleTargetKind,
            "query source metadata target has an incompatible kind",
        ),
        SemanticReferenceRequestOutcome::PartialWorkspace => (
            SemanticDiagnosticCode::ReferenceUnresolved,
            SemanticDiagnosticSeverity::Warning,
            SemanticDiagnosticKind::UnresolvedTarget,
            "query source metadata target is absent from the partial workspace",
        ),
        SemanticReferenceRequestOutcome::Collected
        | SemanticReferenceRequestOutcome::Resolved
        | SemanticReferenceRequestOutcome::InvalidOwnerReference => {
            return Err(invalid_query_source_request(request));
        }
    };

    diagnostics.insert(
        SemanticDiagnostic::new(code, severity, kind, message, request.reference().clone())
            .with_source_node(request.source_node().clone())
            .with_expected_kinds(request.expected_kinds().to_vec())
            .with_candidates(request.candidates().to_vec())
            .with_provenance(query_request_projection_provenance(
                request,
                None,
                EdgeKind::Reads,
                QUERY_SOURCE_DIAGNOSTIC_CONTRIBUTOR,
                FactOrigin::Resolved,
                request.state(),
            )?),
    );
    Ok(())
}

fn invalid_query_source_request(request: &SemanticReferenceRequest) -> EdtBslGraphError {
    EdtBslGraphError::InvalidQuerySourceRequest {
        request_id: request.id().clone(),
    }
}

fn query_request_projection_provenance(
    request: &SemanticReferenceRequest,
    target: Option<(&EntityId, NodeKind)>,
    edge_kind: EdgeKind,
    producer: &'static str,
    origin: FactOrigin,
    resolution: ResolutionState,
) -> Result<Vec<Provenance>, EdtBslGraphError> {
    let mut provenance = Vec::new();
    for evidence in request
        .provenance()
        .iter()
        .filter(|evidence| evidence.origin() == FactOrigin::Parsed)
    {
        let mut context = evidence.source().map_or_else(
            || request.source_node().as_str().to_owned(),
            |source| source.as_str().to_owned(),
        );
        context.push_str("#query_request_projection");
        append_context(&mut context, "request", request.id().as_str());
        append_context(&mut context, "query", request.source_node().as_str());
        append_context(&mut context, "outcome", request.outcome().as_str());
        append_context(&mut context, "projection", edge_kind_name(edge_kind));
        append_context(
            &mut context,
            "collection_evidence",
            evidence.source().map_or("none", EntityId::as_str),
        );
        if let Some((target_id, target_kind)) = target {
            append_context(&mut context, "resolved_target", target_id.as_str());
            append_context(
                &mut context,
                "target_kind",
                query_target_kind_name(target_kind),
            );
        }
        if edge_kind == EdgeKind::DependsOn {
            append_context(&mut context, "proving_fact", "reads");
            append_context(&mut context, "normalization", "query_data_dependency");
        }
        for candidate in request.candidates() {
            append_context(&mut context, "candidate", candidate.as_str());
        }

        let source =
            EntityId::new(context).map_err(|_| EdtBslGraphError::InvalidSourceIdentifier)?;
        provenance.push(Provenance::new(
            Some(source),
            ProducerId::new(producer),
            origin,
            Confidence::Exact,
            resolution,
        ));
    }

    if provenance.is_empty() {
        return Err(invalid_query_source_request(request));
    }
    provenance.sort_by(|left, right| left.source().cmp(&right.source()));
    provenance.dedup();
    Ok(provenance)
}

fn query_diagnostic_provenance(
    module_source: Option<&EntityId>,
    query: &BslQuery,
    diagnostic_kind: &str,
    start_byte: usize,
    end_byte: usize,
    origin: FactOrigin,
    resolution: ResolutionState,
) -> Result<Provenance, EdtBslGraphError> {
    let mut context = query_context(module_source, query);
    append_context(&mut context, "query_text", query.text());
    append_context(&mut context, "diagnostic_kind", diagnostic_kind);
    append_context(&mut context, "range", &format!("{start_byte}..{end_byte}"));
    provenance_from_context(context, origin, resolution)
}

fn query_context(module_source: Option<&EntityId>, query: &BslQuery) -> String {
    let mut context = module_source.map_or_else(
        || query.id().as_str().to_owned(),
        |source| source.as_str().to_owned(),
    );
    context.push_str("#query_reads");
    append_context(&mut context, "query", query.id().as_str());
    append_context(&mut context, "owner", query.owner_id().as_str());
    append_context(&mut context, "binding", query.binding_name().as_str());
    append_context(&mut context, "declaration_line", &query.line().to_string());
    append_context(&mut context, "parser_stage", QUERY_LANGUAGE_PARSER_STAGE);
    append_context(&mut context, "resolver_stage", QUERY_SOURCE_RESOLVER_STAGE);
    append_context(&mut context, "contributor_stage", QUERY_READS_CONTRIBUTOR);
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
) -> Result<Provenance, EdtBslGraphError> {
    let source = EntityId::new(context).map_err(|_| EdtBslGraphError::InvalidSourceIdentifier)?;
    Ok(Provenance::new(
        Some(source),
        ProducerId::new(QUERY_READS_CONTRIBUTOR),
        origin,
        Confidence::Exact,
        resolution,
    ))
}

const fn edge_kind_name(kind: EdgeKind) -> &'static str {
    match kind {
        EdgeKind::Reads => "reads",
        EdgeKind::DependsOn => "depends_on",
        _ => "diagnostic",
    }
}

fn query_target_kind_name(kind: NodeKind) -> &'static str {
    match kind {
        NodeKind::Metadata(MetadataKind::Catalog) => "metadata.catalog",
        NodeKind::Metadata(MetadataKind::InformationRegister) => "metadata.information_register",
        NodeKind::Metadata(MetadataKind::AccumulationRegister) => "metadata.accumulation_register",
        NodeKind::Metadata(MetadataKind::AccountingRegister) => "metadata.accounting_register",
        _ => "unsupported",
    }
}

fn insert_local_calls(
    graph: &mut SemanticGraph,
    module: &AnalyzedBslModule,
    diagnostics: &mut BTreeSet<SemanticDiagnostic>,
    reference_statistics: &mut SemanticReferenceStatistics,
) -> Result<(), EdtBslGraphError> {
    let resolution = LocalBslCallResolver.resolve(module.symbols(), module.calls());

    for resolved_call in resolution.resolved() {
        insert_call_edge(
            graph,
            resolved_call.origin_id(),
            resolved_call.destination_id(),
            module.source(),
        )?;
    }

    for call in module
        .calls()
        .iter()
        .filter(|call| !is_qualified_call(call))
    {
        if local_call_is_unresolved(call, resolution.unresolved()) {
            record_unresolved_call(module, call, diagnostics, reference_statistics);
        } else {
            reference_statistics.record(SemanticReferenceOutcome::Resolved, true);
        }
    }

    Ok(())
}

fn insert_cross_module_calls(
    graph: &mut SemanticGraph,
    module: &AnalyzedBslModule,
    current_module: &BslModuleSymbols,
    available_modules: &[BslModuleSymbols],
    diagnostics: &mut BTreeSet<SemanticDiagnostic>,
    reference_statistics: &mut SemanticReferenceStatistics,
) -> Result<(), EdtBslGraphError> {
    let resolution = QualifiedBslCallResolver.resolve_cross_module_calls(
        current_module,
        available_modules,
        module.calls(),
    );

    for resolved_call in resolution.resolved() {
        insert_call_edge(
            graph,
            resolved_call.origin_id(),
            resolved_call.destination_id(),
            module.source(),
        )?;
    }

    for call in module.calls().iter().filter(|call| is_qualified_call(call)) {
        if cross_module_call_is_unresolved(call, resolution.unresolved()) {
            record_unresolved_call(module, call, diagnostics, reference_statistics);
        } else {
            reference_statistics.record(SemanticReferenceOutcome::Resolved, true);
        }
    }

    Ok(())
}

fn is_qualified_call(call: &BslCall) -> bool {
    call.target_symbol().as_str().contains('.')
}

fn local_call_is_unresolved(call: &BslCall, unresolved: &[UnresolvedBslCall]) -> bool {
    unresolved.iter().any(|candidate| {
        candidate.source_name() == call.source_symbol()
            && candidate.target_name() == call.target_symbol()
            && candidate.line() == call.line()
    })
}

fn cross_module_call_is_unresolved(
    call: &BslCall,
    unresolved: &[UnresolvedCrossModuleCall],
) -> bool {
    unresolved.iter().any(|candidate| {
        candidate.source_name() == call.source_symbol()
            && candidate.target_name() == call.target_symbol()
            && candidate.line() == call.line()
    })
}

fn record_unresolved_call(
    module: &AnalyzedBslModule,
    call: &BslCall,
    diagnostics: &mut BTreeSet<SemanticDiagnostic>,
    reference_statistics: &mut SemanticReferenceStatistics,
) {
    let reference = SemanticReference::Name(call.target_symbol().clone());
    let mut diagnostic = SemanticDiagnostic::from_resolution_error_with_reference(
        ResolutionError::MissingTarget {
            reference: reference.clone(),
        },
        Some(reference),
    )
    .with_expected_kinds(vec![NodeKind::Procedure, NodeKind::Function])
    .with_provenance(vec![unresolved_call_provenance(module.source(), call)]);

    if let Some(source_node) = source_node_id(module, call) {
        diagnostic = diagnostic.with_source_node(source_node);
    }

    diagnostics.insert(diagnostic);
    reference_statistics.record(SemanticReferenceOutcome::Unresolved, true);
}

fn source_node_id(module: &AnalyzedBslModule, call: &BslCall) -> Option<EntityId> {
    let source_name = call.source_symbol()?;
    module
        .symbols()
        .iter()
        .find(|symbol| {
            symbol
                .name()
                .as_str()
                .eq_ignore_ascii_case(source_name.as_str())
        })
        .map(|symbol| symbol.id().clone())
}

fn insert_call_edge(
    graph: &mut SemanticGraph,
    origin_id: &EntityId,
    destination_id: &EntityId,
    source: Option<&EntityId>,
) -> Result<(), EdtBslGraphError> {
    graph
        .insert_edge_with_provenance(
            origin_id.clone(),
            destination_id.clone(),
            EdgeKind::Calls,
            resolved_provenance(source),
        )
        .map(|_| ())
        .map_err(EdtBslGraphError::Graph)
}

fn declared_provenance(source: Option<&EntityId>) -> Provenance {
    bsl_provenance(
        source,
        FactOrigin::Declared,
        Confidence::Exact,
        ResolutionState::NotApplicable,
    )
}

fn declared_provenance_with_location(
    source: Option<&EntityId>,
    location: Option<SourceLocation>,
) -> Provenance {
    with_optional_location(declared_provenance(source), location)
}

fn resolved_provenance(source: Option<&EntityId>) -> Provenance {
    bsl_provenance(
        source,
        FactOrigin::Resolved,
        Confidence::High,
        ResolutionState::Resolved,
    )
}

fn unresolved_call_provenance(source: Option<&EntityId>, call: &BslCall) -> Provenance {
    let source = source.map_or_else(
        || call.id().clone(),
        |source| {
            EntityId::new(format!(
                "{}#bsl_call={}",
                source.as_str(),
                call.id().as_str()
            ))
            .expect("a non-empty source and call identifier must produce a valid identifier")
        },
    );

    bsl_provenance(
        Some(&source),
        FactOrigin::Resolved,
        Confidence::Exact,
        ResolutionState::Unresolved,
    )
}

fn query_provenance(
    module: &AnalyzedBslModule,
    query: &BslQuery,
) -> Result<Provenance, EdtBslGraphError> {
    let source = module.source().map_or_else(
        || query.id().clone(),
        |source| {
            EntityId::new(format!(
                "{}#bsl_query={};owner={};binding={}",
                source.as_str(),
                query.id().as_str(),
                query.owner_id().as_str(),
                query.binding_name().as_str()
            ))
            .expect("a non-empty source and query context must produce a valid identifier")
        },
    );

    let provenance = bsl_provenance(
        Some(&source),
        FactOrigin::Declared,
        Confidence::Exact,
        ResolutionState::NotApplicable,
    );
    Ok(with_optional_location(
        provenance,
        declaration_location(module, query.line())?,
    ))
}

fn declaration_location(
    module: &AnalyzedBslModule,
    line: usize,
) -> Result<Option<SourceLocation>, EdtBslGraphError> {
    let Some(path) = module.source_path().cloned() else {
        return Ok(None);
    };
    let line = u32::try_from(line).map_err(|_| EdtBslGraphError::InvalidSourceLocation)?;
    let position =
        SourcePosition::new(line, 1).map_err(|_| EdtBslGraphError::InvalidSourceLocation)?;
    let span =
        SourceSpan::new(position, position).map_err(|_| EdtBslGraphError::InvalidSourceLocation)?;
    Ok(Some(SourceLocation::new(path, Some(span))))
}

fn with_optional_location(provenance: Provenance, location: Option<SourceLocation>) -> Provenance {
    match location {
        Some(location) => provenance.with_location(location),
        None => provenance,
    }
}

fn bsl_provenance(
    source: Option<&EntityId>,
    origin: FactOrigin,
    confidence: Confidence,
    resolution: ResolutionState,
) -> Provenance {
    Provenance::new(
        source.cloned(),
        ProducerId::new(EDT_BSL_GRAPH_PRODUCER),
        origin,
        confidence,
        resolution,
    )
}

fn source_id_from_path(path: &Path) -> Result<EntityId, EdtBslGraphError> {
    EntityId::new(path.to_string_lossy().replace('\\', "/"))
        .map_err(|_| EdtBslGraphError::InvalidSourceIdentifier)
}

/// Error produced while adding BSL declarations to the EDT semantic graph.
#[derive(Debug)]
pub enum EdtBslGraphError {
    /// A module source file could not be read.
    ReadModule {
        /// Module source path.
        path: std::path::PathBuf,
        /// Underlying I/O error.
        source: std::io::Error,
    },

    /// BSL declaration extraction failed.
    ParseDeclarations(BslParseError),

    /// BSL call extraction failed.
    ParseCalls(BslCallError),

    /// BSL query extraction failed.
    ParseQueries(BslQueryError),

    /// Semantic graph validation failed.
    Graph(GraphError),

    /// A query source request violates its canonical adapter contract.
    InvalidQuerySourceRequest {
        /// Stable request identity.
        request_id: oneagent_graph::SemanticReferenceRequestId,
    },

    /// A parsed query source local name could not become a semantic name.
    InvalidQuerySourceTargetName(EntityNameError),

    /// A public semantic reference request invariant failed.
    ReferenceRequest(oneagent_graph::SemanticReferenceRequestError),

    /// A module source identifier could not be created.
    InvalidSourceIdentifier,

    /// A module source path or declaration position could not become typed evidence.
    InvalidSourceLocation,
}

impl Display for EdtBslGraphError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReadModule { path, source } => {
                write!(
                    formatter,
                    "failed to read BSL module {}: {source}",
                    path.display()
                )
            }

            Self::ParseDeclarations(error) => {
                write!(formatter, "failed to parse BSL declarations: {error}")
            }

            Self::ParseCalls(error) => {
                write!(formatter, "failed to parse BSL calls: {error}")
            }

            Self::ParseQueries(error) => {
                write!(formatter, "failed to parse BSL queries: {error}")
            }

            Self::Graph(error) => {
                write!(formatter, "semantic graph error: {error}")
            }

            Self::InvalidQuerySourceRequest { request_id } => write!(
                formatter,
                "query source request `{request_id}` has invalid adapter content"
            ),

            Self::InvalidQuerySourceTargetName(error) => {
                write!(formatter, "query source target name is invalid: {error}")
            }

            Self::ReferenceRequest(error) => {
                write!(formatter, "semantic reference request error: {error}")
            }

            Self::InvalidSourceIdentifier => {
                formatter.write_str("EDT BSL source identifier is invalid")
            }
            Self::InvalidSourceLocation => {
                formatter.write_str("EDT BSL source location is invalid")
            }
        }
    }
}

impl std::error::Error for EdtBslGraphError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ReadModule { source, .. } => Some(source),
            Self::ParseDeclarations(error) => Some(error),
            Self::ParseCalls(error) => Some(error),
            Self::ParseQueries(error) => Some(error),
            Self::Graph(error) => Some(error),
            Self::InvalidQuerySourceTargetName(error) => Some(error),
            Self::ReferenceRequest(error) => Some(error),
            Self::InvalidSourceIdentifier
            | Self::InvalidSourceLocation
            | Self::InvalidQuerySourceRequest { .. } => None,
        }
    }
}

impl From<QuerySourceRequestError> for EdtBslGraphError {
    fn from(error: QuerySourceRequestError) -> Self {
        match error {
            QuerySourceRequestError::InvalidSourceIdentifier => Self::InvalidSourceIdentifier,
            QuerySourceRequestError::InvalidTargetName(error) => {
                Self::InvalidQuerySourceTargetName(error)
            }
            QuerySourceRequestError::InvalidCollectedRequest { request_id } => {
                Self::InvalidQuerySourceRequest { request_id }
            }
            QuerySourceRequestError::Request(error) => Self::ReferenceRequest(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use oneagent_bsl::BslQuery;
    use oneagent_common::{EntityId, EntityName};
    use oneagent_graph::{
        Confidence, EdgeKind, FactOrigin, GraphError, GraphNode, NodeKind, ResolutionState,
        SemanticDiagnosticCode, SemanticDiagnosticKind, SemanticGraph,
        SemanticReferenceRequestLedger, SemanticReferenceRequestOutcome,
        SemanticReferenceStatistics,
    };
    use oneagent_metadata::MetadataKind;
    use std::collections::BTreeSet;
    use std::fs;
    use tempfile::tempdir;

    use crate::{EdtModuleDescriptor, EdtModuleKind};

    use super::{
        AnalyzedBslModule, EdtBslGraphError, WorkspaceResolutionScope,
        add_configuration_module_symbols, add_configuration_module_symbols_with_diagnostics,
        add_configuration_module_symbols_with_diagnostics_in_scope, add_module_symbols,
        insert_query_reads,
    };

    fn id(value: &str) -> EntityId {
        EntityId::new(value).expect("identifier must be valid")
    }

    fn name(value: &str) -> EntityName {
        EntityName::new(value).expect("name must be valid")
    }

    #[test]
    fn adds_procedure_and_function_nodes() {
        let root = tempdir().expect("temporary directory must be created");
        let module_path = root.path().join("ObjectModule.bsl");

        fs::write(
            &module_path,
            concat!(
                "Procedure BeforeWrite() Export\n",
                "    Total();\n",
                "EndProcedure\n",
                "\n",
                "Function Total()\n",
                "    Return 100;\n",
                "EndFunction\n",
            ),
        )
        .expect("module file must be created");

        let module_id =
            EntityId::new("document-id:object_module").expect("identifier must be valid");

        let module = EdtModuleDescriptor::new(
            module_id.clone(),
            EntityName::new("ObjectModule").expect("name must be valid"),
            EdtModuleKind::Object,
            module_path,
        );

        let mut graph = SemanticGraph::new();
        graph.insert_node(GraphNode::new(
            module_id,
            EntityName::new("ObjectModule").expect("name must be valid"),
            NodeKind::Module,
        ));

        let count = add_module_symbols(&mut graph, &module).expect("symbols must be added");

        let procedure = graph
            .nodes_by_kind(NodeKind::Procedure)
            .into_iter()
            .next()
            .expect("procedure node must exist");

        let calls = graph.outgoing_by_kind(procedure.id(), EdgeKind::Calls);

        assert_eq!(calls.len(), 1);

        let function = graph
            .nodes_by_kind(NodeKind::Function)
            .into_iter()
            .next()
            .expect("function node must exist");

        assert_eq!(calls[0].target(), function.id());

        assert_eq!(count, 2);
        assert_eq!(graph.nodes_by_kind(NodeKind::Procedure).len(), 1);
        assert_eq!(graph.nodes_by_kind(NodeKind::Function).len(), 1);
    }

    #[test]
    fn resolves_qualified_call_to_module_collected_later() {
        let root = tempdir().expect("temporary directory must be created");

        let caller_path = root.path().join("CallerModule.bsl");
        let service_path = root.path().join("ServiceModule.bsl");

        fs::write(
            &caller_path,
            concat!(
                "Procedure Run()\n",
                "    ServiceModule.Execute();\n",
                "EndProcedure\n",
            ),
        )
        .expect("caller module must be created");

        fs::write(
            &service_path,
            concat!("Procedure Execute() Export\n", "EndProcedure\n",),
        )
        .expect("service module must be created");

        let caller_id =
            EntityId::new("configuration:caller_module").expect("identifier must be valid");
        let service_id =
            EntityId::new("configuration:service_module").expect("identifier must be valid");

        let caller = EdtModuleDescriptor::new(
            caller_id.clone(),
            EntityName::new("CallerModule").expect("name must be valid"),
            EdtModuleKind::Object,
            caller_path,
        );

        let service = EdtModuleDescriptor::new(
            service_id.clone(),
            EntityName::new("ServiceModule").expect("name must be valid"),
            EdtModuleKind::Common,
            service_path,
        );

        let mut graph = SemanticGraph::new();

        graph.insert_node(GraphNode::new(
            caller_id,
            EntityName::new("CallerModule").expect("name must be valid"),
            NodeKind::Module,
        ));

        graph.insert_node(GraphNode::new(
            service_id,
            EntityName::new("ServiceModule").expect("name must be valid"),
            NodeKind::Module,
        ));

        let count = add_configuration_module_symbols(&mut graph, &[caller, service])
            .expect("configuration symbols must be added");

        let run = graph
            .nodes_by_kind(NodeKind::Procedure)
            .into_iter()
            .find(|node| node.name().as_str() == "Run")
            .expect("Run procedure must exist");

        let execute = graph
            .nodes_by_kind(NodeKind::Procedure)
            .into_iter()
            .find(|node| node.name().as_str() == "Execute")
            .expect("Execute procedure must exist");

        let calls = graph.outgoing_by_kind(run.id(), EdgeKind::Calls);

        assert_eq!(count, 2);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].target(), execute.id());
    }

    #[test]
    fn cross_module_resolution_does_not_depend_on_module_order() {
        let root = tempdir().expect("temporary directory must be created");

        let caller_path = root.path().join("CallerModule.bsl");
        let service_path = root.path().join("ServiceModule.bsl");

        fs::write(
            &caller_path,
            concat!(
                "Procedure Run()\n",
                "    ServiceModule.Execute();\n",
                "EndProcedure\n",
            ),
        )
        .expect("caller module must be created");

        fs::write(
            &service_path,
            concat!("Procedure Execute() Export\n", "EndProcedure\n",),
        )
        .expect("service module must be created");

        let build_graph = |reverse: bool| {
            let caller_id =
                EntityId::new("configuration:caller_module").expect("identifier must be valid");
            let service_id =
                EntityId::new("configuration:service_module").expect("identifier must be valid");

            let caller = EdtModuleDescriptor::new(
                caller_id.clone(),
                EntityName::new("CallerModule").expect("name must be valid"),
                EdtModuleKind::Object,
                caller_path.clone(),
            );

            let service = EdtModuleDescriptor::new(
                service_id.clone(),
                EntityName::new("ServiceModule").expect("name must be valid"),
                EdtModuleKind::Common,
                service_path.clone(),
            );

            let mut graph = SemanticGraph::new();

            graph.insert_node(GraphNode::new(
                caller_id,
                EntityName::new("CallerModule").expect("name must be valid"),
                NodeKind::Module,
            ));

            graph.insert_node(GraphNode::new(
                service_id,
                EntityName::new("ServiceModule").expect("name must be valid"),
                NodeKind::Module,
            ));

            let modules = if reverse {
                vec![service, caller]
            } else {
                vec![caller, service]
            };

            add_configuration_module_symbols(&mut graph, &modules)
                .expect("configuration symbols must be added");

            graph
        };

        let normal = build_graph(false);
        let reversed = build_graph(true);

        let normal_run = normal
            .nodes_by_kind(NodeKind::Procedure)
            .into_iter()
            .find(|node| node.name().as_str() == "Run")
            .expect("Run procedure must exist");

        let reversed_run = reversed
            .nodes_by_kind(NodeKind::Procedure)
            .into_iter()
            .find(|node| node.name().as_str() == "Run")
            .expect("Run procedure must exist");

        let normal_calls = normal.outgoing_by_kind(normal_run.id(), EdgeKind::Calls);
        let reversed_calls = reversed.outgoing_by_kind(reversed_run.id(), EdgeKind::Calls);

        assert_eq!(normal_calls.len(), 1);
        assert_eq!(reversed_calls.len(), 1);
        assert_eq!(normal_calls[0].target(), reversed_calls[0].target());
    }

    #[test]
    fn records_one_diagnostic_outcome_for_each_unresolved_call() {
        let root = tempdir().expect("temporary directory must be created");
        let module_path = root.path().join("CallerModule.bsl");

        fs::write(
            &module_path,
            concat!(
                "Procedure Run()\n",
                "    MissingLocal();\n",
                "    MissingModule.Execute();\n",
                "EndProcedure\n",
            ),
        )
        .expect("caller module must be created");

        let build = || {
            let module_id =
                EntityId::new("configuration:caller_module").expect("identifier must be valid");
            let module = EdtModuleDescriptor::new(
                module_id.clone(),
                EntityName::new("CallerModule").expect("name must be valid"),
                EdtModuleKind::Object,
                module_path.clone(),
            );
            let mut graph = SemanticGraph::new();
            graph.insert_node(GraphNode::new(
                module_id,
                EntityName::new("CallerModule").expect("name must be valid"),
                NodeKind::Module,
            ));
            let mut diagnostics = BTreeSet::new();
            let mut statistics = SemanticReferenceStatistics::new();

            add_configuration_module_symbols_with_diagnostics(
                &mut graph,
                &[module],
                &mut diagnostics,
                &mut statistics,
            )
            .expect("configuration symbols must be added");

            (
                graph,
                diagnostics.into_iter().collect::<Vec<_>>(),
                statistics,
            )
        };

        let (graph, diagnostics, statistics) = build();
        let (_, repeated_diagnostics, repeated_statistics) = build();
        let run_id =
            EntityId::new("configuration:caller_module:procedure:Run").expect("ID must be valid");

        assert_eq!(statistics.total(), 2);
        assert_eq!(statistics.unresolved(), 2);
        assert_eq!(statistics.outcome_total(), 2);
        assert_eq!(statistics.with_provenance(), 2);
        assert_eq!(diagnostics.len(), 2);
        assert_eq!(diagnostics, repeated_diagnostics);
        assert_eq!(statistics, repeated_statistics);
        assert!(graph.node(&run_id).is_some());
        assert!(graph.outgoing_by_kind(&run_id, EdgeKind::Calls).is_empty());

        for diagnostic in &diagnostics {
            assert_eq!(
                diagnostic.code(),
                SemanticDiagnosticCode::ReferenceUnresolved
            );
            assert_eq!(diagnostic.kind(), SemanticDiagnosticKind::UnresolvedTarget);
            assert_eq!(diagnostic.source_node(), Some(&run_id));
            assert_eq!(
                diagnostic.expected_kinds(),
                &[NodeKind::Procedure, NodeKind::Function]
            );
            assert_eq!(diagnostic.provenance().len(), 1);
            assert_eq!(
                diagnostic.provenance()[0].resolution(),
                ResolutionState::Unresolved
            );
            assert!(
                diagnostic.provenance()[0]
                    .source()
                    .expect("diagnostic source must exist")
                    .as_str()
                    .contains("CallerModule.bsl#bsl_call=")
            );
        }
    }

    #[test]
    fn query_reads_orchestration_parses_resolves_and_emits_exact_provenance() {
        let root = tempdir().expect("temporary directory must be created");
        let module_path = root.path().join("ObjectModule.bsl");
        fs::write(
            &module_path,
            concat!(
                "Procedure Run()\n",
                "    Query = New Query;\n",
                "    Query.Text = \"SELECT Ref FROM Catalog.Products\";\n",
                "EndProcedure\n",
            ),
        )
        .expect("module file must be created");

        let module_id = id("document.query_host:object_module");
        let target_id = id("catalog.products");
        let module = EdtModuleDescriptor::new(
            module_id.clone(),
            name("ObjectModule"),
            EdtModuleKind::Object,
            module_path,
        );
        let mut graph = SemanticGraph::new();
        graph.insert_node(GraphNode::new(
            module_id,
            name("ObjectModule"),
            NodeKind::Module,
        ));
        graph.insert_node(GraphNode::new(
            target_id.clone(),
            name("Products"),
            NodeKind::Metadata(MetadataKind::Catalog),
        ));
        let mut diagnostics = BTreeSet::new();
        let mut statistics = SemanticReferenceStatistics::new();
        let mut requests = SemanticReferenceRequestLedger::new();

        add_configuration_module_symbols_with_diagnostics_in_scope(
            &mut graph,
            &[module],
            WorkspaceResolutionScope::Complete,
            &mut diagnostics,
            &mut statistics,
            &mut requests,
        )
        .expect("query Reads must be emitted");

        let reads = graph.query().edges_by_kind(EdgeKind::Reads);
        let dependencies = graph.query().edges_by_kind(EdgeKind::DependsOn);
        assert!(diagnostics.is_empty());
        assert!(statistics.is_empty());
        assert_eq!(requests.len(), 1);
        assert_eq!(
            requests.requests()[0].outcome(),
            SemanticReferenceRequestOutcome::Resolved
        );
        assert_eq!(reads.len(), 1);
        assert_eq!(dependencies.len(), 1);
        assert_eq!(reads[0].target(), &target_id);
        assert_eq!(dependencies[0].target(), &target_id);
        assert_eq!(reads[0].provenance().len(), 1);
        let provenance = &reads[0].provenance()[0];
        let source = provenance
            .source()
            .expect("Reads provenance source must exist")
            .as_str();
        assert!(source.contains("raw_source#16:Catalog.Products"));
        assert!(source.contains("range#6:16..32"));
        assert!(source.contains("category#7:catalog"));
        assert!(source.contains("resolved_target#16:catalog.products"));
        assert_eq!(provenance.producer().as_str(), "oneagent.edt.query-reads");
        assert_eq!(provenance.origin(), FactOrigin::Resolved);
        assert_eq!(provenance.confidence(), Confidence::Exact);
        assert_eq!(provenance.resolution(), ResolutionState::Resolved);
        assert_eq!(
            dependencies[0].provenance()[0].producer().as_str(),
            "oneagent.edt.query-dependency"
        );
        assert_eq!(
            dependencies[0].provenance()[0].origin(),
            FactOrigin::Derived
        );
    }

    #[test]
    fn direct_register_categories_emit_production_requests_reads_and_dependencies() {
        let root = tempdir().expect("temporary directory must be created");
        let module_path = root.path().join("ObjectModule.bsl");
        fs::write(
            &module_path,
            concat!(
                "Procedure ReadAccumulationRegister()\n",
                "    Query = New Query;\n",
                "    Query.Text = \"SELECT Ref FROM AccumulationRegister.InventoryCost\";\n",
                "EndProcedure\n",
                "Procedure ReadAccountingRegister()\n",
                "    Query = New Query;\n",
                "    Query.Text = \"SELECT Ref FROM AccountingRegister.FinancialAccounting\";\n",
                "EndProcedure\n",
            ),
        )
        .expect("module file must be created");

        let module_id = id("document.query_host:object_module");
        let module = EdtModuleDescriptor::new(
            module_id.clone(),
            name("ObjectModule"),
            EdtModuleKind::Object,
            module_path,
        );
        let mut graph = SemanticGraph::new();
        graph.insert_node(GraphNode::new(
            module_id,
            name("ObjectModule"),
            NodeKind::Module,
        ));
        graph.insert_node(GraphNode::new(
            id("accumulation-register.inventory-cost"),
            name("InventoryCost"),
            NodeKind::Metadata(MetadataKind::AccumulationRegister),
        ));
        graph.insert_node(GraphNode::new(
            id("accounting-register.financial-accounting"),
            name("FinancialAccounting"),
            NodeKind::Metadata(MetadataKind::AccountingRegister),
        ));
        let mut diagnostics = BTreeSet::new();
        let mut statistics = SemanticReferenceStatistics::new();
        let mut requests = SemanticReferenceRequestLedger::new();

        add_configuration_module_symbols_with_diagnostics_in_scope(
            &mut graph,
            &[module],
            WorkspaceResolutionScope::Complete,
            &mut diagnostics,
            &mut statistics,
            &mut requests,
        )
        .expect("direct register categories must emit production facts");

        assert_eq!(graph.query().nodes_by_kind(NodeKind::Query).len(), 2);
        assert_eq!(graph.query().edges_by_kind(EdgeKind::Reads).len(), 2);
        assert_eq!(graph.query().edges_by_kind(EdgeKind::DependsOn).len(), 2);
        assert!(diagnostics.is_empty());
        assert!(statistics.is_empty());
        assert_eq!(requests.len(), 2);
        assert!(
            requests
                .requests()
                .iter()
                .all(|request| { request.outcome() == SemanticReferenceRequestOutcome::Resolved })
        );
    }

    #[test]
    fn query_reads_aggregates_and_deduplicates_equivalent_evidence_before_insertion() {
        let root = tempdir().expect("temporary directory must be created");
        let module_path = root.path().join("ObjectModule.bsl");
        let repeated_module_path = root.path().join("RepeatedObjectModule.bsl");
        let source = concat!(
            "Procedure Run()\n",
            "    Query = New Query;\n",
            "    Query.Text = \"SELECT Ref FROM Catalog.Products\";\n",
            "EndProcedure\n",
        );
        fs::write(&module_path, source).expect("module file must be created");
        fs::write(&repeated_module_path, source).expect("repeated module file must be created");

        let module_id = id("document.query_host:object_module");
        let module = EdtModuleDescriptor::new(
            module_id.clone(),
            name("ObjectModule"),
            EdtModuleKind::Object,
            module_path,
        );
        let repeated_module = EdtModuleDescriptor::new(
            module_id.clone(),
            name("ObjectModule"),
            EdtModuleKind::Object,
            repeated_module_path,
        );
        let mut graph = SemanticGraph::new();
        graph.insert_node(GraphNode::new(
            module_id,
            name("ObjectModule"),
            NodeKind::Module,
        ));
        graph.insert_node(GraphNode::new(
            id("catalog.products"),
            name("Products"),
            NodeKind::Metadata(MetadataKind::Catalog),
        ));
        let mut diagnostics = BTreeSet::new();
        let mut statistics = SemanticReferenceStatistics::new();
        let mut requests = SemanticReferenceRequestLedger::new();

        add_configuration_module_symbols_with_diagnostics_in_scope(
            &mut graph,
            &[module, repeated_module],
            WorkspaceResolutionScope::Complete,
            &mut diagnostics,
            &mut statistics,
            &mut requests,
        )
        .expect("duplicate query evidence must be aggregated");

        let reads = graph.query().edges_by_kind(EdgeKind::Reads);
        let dependencies = graph.query().edges_by_kind(EdgeKind::DependsOn);
        assert!(diagnostics.is_empty());
        assert!(statistics.is_empty());
        assert_eq!(requests.len(), 1);
        assert_eq!(requests.requests()[0].provenance().len(), 3);
        assert_eq!(reads.len(), 1);
        assert_eq!(dependencies.len(), 1);
        assert_eq!(reads[0].provenance().len(), 2);
        assert_eq!(dependencies[0].provenance().len(), 2);
    }

    #[test]
    fn query_source_partial_request_projects_warning_without_edges_or_legacy_count() {
        let root = tempdir().expect("temporary directory must be created");
        let module_path = root.path().join("ObjectModule.bsl");
        fs::write(
            &module_path,
            concat!(
                "Procedure Run()\n",
                "    Query = New Query;\n",
                "    Query.Text = \"SELECT Ref FROM Catalog.Missing\";\n",
                "EndProcedure\n",
            ),
        )
        .expect("module file must be created");

        let module_id = id("document.query_host:object_module");
        let module = EdtModuleDescriptor::new(
            module_id.clone(),
            name("ObjectModule"),
            EdtModuleKind::Object,
            module_path,
        );
        let mut graph = SemanticGraph::new();
        graph.insert_node(GraphNode::new(
            module_id,
            name("ObjectModule"),
            NodeKind::Module,
        ));
        let mut diagnostics = BTreeSet::new();
        let mut statistics = SemanticReferenceStatistics::new();
        let mut requests = SemanticReferenceRequestLedger::new();

        add_configuration_module_symbols_with_diagnostics_in_scope(
            &mut graph,
            &[module],
            WorkspaceResolutionScope::Partial,
            &mut diagnostics,
            &mut statistics,
            &mut requests,
        )
        .expect("partial query source must remain recoverable");

        assert!(statistics.is_empty());
        assert_eq!(requests.len(), 1);
        assert_eq!(
            requests.requests()[0].outcome(),
            SemanticReferenceRequestOutcome::PartialWorkspace
        );
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics
                .iter()
                .next()
                .expect("diagnostic must exist")
                .severity(),
            oneagent_graph::SemanticDiagnosticSeverity::Warning
        );
        assert!(graph.query().edges_by_kind(EdgeKind::Reads).is_empty());
        assert!(graph.query().edges_by_kind(EdgeKind::DependsOn).is_empty());
    }

    #[test]
    fn query_reads_treats_a_missing_query_source_node_as_an_invariant_failure() {
        let query_id = id("module:procedure:Run:query:Query");
        let module = AnalyzedBslModule::new_with_source_and_queries(
            id("module"),
            name("Module"),
            Vec::new(),
            Vec::new(),
            vec![BslQuery::new(
                query_id.clone(),
                id("module:procedure:Run"),
                name("Run"),
                name("Query"),
                "SELECT Ref FROM Catalog.Products".to_owned(),
                2,
            )],
            None,
        );
        let mut graph = SemanticGraph::new();
        let mut diagnostics = BTreeSet::new();
        let mut statistics = SemanticReferenceStatistics::new();
        let mut requests = SemanticReferenceRequestLedger::new();

        let error = insert_query_reads(
            &mut graph,
            &[module],
            WorkspaceResolutionScope::Complete,
            &mut diagnostics,
            &mut statistics,
            &mut requests,
        )
        .expect_err("missing Query node must fail graph construction");

        assert!(matches!(
            error,
            EdtBslGraphError::Graph(GraphError::MissingNode(actual)) if actual == query_id
        ));
        assert!(diagnostics.is_empty());
        assert!(statistics.is_empty());
    }
}
