//! Deterministic resolution of parsed query sources against EDT metadata nodes.

use oneagent_bsl::{
    BslQuery, QueryLanguageParseResult, QuerySourceCategory, QuerySourceOccurrence,
};
use oneagent_common::{EntityId, EntityName, EntityNameError};
use oneagent_graph::{
    Confidence, FactOrigin, NodeKind, ProducerId, Provenance, ResolutionState, SemanticGraph,
    SemanticReference, SemanticReferenceCategory, SemanticReferenceRequest,
    SemanticReferenceRequestError, SemanticReferenceRequestId, SemanticReferenceRequestLedger,
};
use oneagent_metadata::MetadataKind;
use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};

const QUERY_SOURCE_COLLECTOR: &str = "oneagent.edt.query-source-collection";
const QUERY_SOURCE_RESOLVER: &str = "oneagent.edt.query-source-resolution";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum QuerySourceResolutionOutcome {
    Resolved { target_id: EntityId },
    MissingTarget,
    AmbiguousTarget { candidates: Vec<EntityId> },
    IncompatibleTargetKind { candidates: Vec<EntityId> },
    PartialWorkspaceTargetAbsent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WorkspaceResolutionScope {
    Complete,
    Partial,
}

#[derive(Debug)]
pub(crate) enum QuerySourceRequestError {
    InvalidSourceIdentifier,
    InvalidTargetName(EntityNameError),
    InvalidCollectedRequest {
        request_id: SemanticReferenceRequestId,
    },
    Request(SemanticReferenceRequestError),
}

impl Display for QuerySourceRequestError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidSourceIdentifier => {
                formatter.write_str("query source request provenance identifier is invalid")
            }
            Self::InvalidTargetName(error) => {
                write!(
                    formatter,
                    "query source request target name is invalid: {error}"
                )
            }
            Self::InvalidCollectedRequest { request_id } => write!(
                formatter,
                "query source request `{request_id}` has invalid collected content"
            ),
            Self::Request(error) => write!(formatter, "query source request is invalid: {error}"),
        }
    }
}

impl std::error::Error for QuerySourceRequestError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidTargetName(error) => Some(error),
            Self::Request(error) => Some(error),
            Self::InvalidSourceIdentifier | Self::InvalidCollectedRequest { .. } => None,
        }
    }
}

impl From<SemanticReferenceRequestError> for QuerySourceRequestError {
    fn from(error: SemanticReferenceRequestError) -> Self {
        Self::Request(error)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct QuerySourceCandidate {
    id: EntityId,
    name: EntityName,
    kind: NodeKind,
}

pub(crate) fn collect_query_source_requests(
    parse_result: &QueryLanguageParseResult,
    module_source: Option<&EntityId>,
    query: &BslQuery,
) -> Result<Option<SemanticReferenceRequestLedger>, QuerySourceRequestError> {
    if !parse_result.is_source_set_complete() || !parse_result.diagnostics().is_empty() {
        return Ok(None);
    }

    let Some(program) = parse_result.program() else {
        return Ok(None);
    };
    let mut requests = SemanticReferenceRequestLedger::new();

    for source in program.sources() {
        let target_name = EntityName::new(source.local_name().to_owned())
            .map_err(QuerySourceRequestError::InvalidTargetName)?;
        let request = SemanticReferenceRequest::collected(
            query.id().clone(),
            SemanticReferenceCategory::QuerySource,
            SemanticReference::Name(target_name),
            [expected_metadata_kind(source.category())],
            [query_source_collection_provenance(
                module_source,
                query,
                source,
            )?],
        )?;
        requests.insert(request)?;
    }

    Ok(Some(requests))
}

/// Immutable query-source resolution index for one semantic graph snapshot.
#[derive(Debug)]
pub(crate) struct QuerySourceResolutionIndex {
    candidates_by_lookup_key: BTreeMap<String, BTreeMap<EntityId, QuerySourceCandidate>>,
}

impl QuerySourceResolutionIndex {
    #[must_use]
    pub(crate) fn new(graph: &SemanticGraph) -> Self {
        let mut candidates_by_lookup_key =
            BTreeMap::<String, BTreeMap<EntityId, QuerySourceCandidate>>::new();

        for node in graph.nodes() {
            let candidate = QuerySourceCandidate {
                id: node.id().clone(),
                name: node.name().clone(),
                kind: node.kind(),
            };
            candidates_by_lookup_key
                .entry(query_source_lookup_key(candidate.name.as_str()))
                .or_default()
                .insert(candidate.id.clone(), candidate);
        }

        Self {
            candidates_by_lookup_key,
        }
    }

    /// Resolves accepted occurrences directly for focused resolver policy tests.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn resolve(
        &self,
        parse_result: &QueryLanguageParseResult,
        workspace_scope: WorkspaceResolutionScope,
    ) -> Option<Vec<QuerySourceResolutionOutcome>> {
        if !parse_result.is_source_set_complete() || !parse_result.diagnostics().is_empty() {
            return None;
        }

        let program = parse_result.program()?;
        Some(
            program
                .sources()
                .iter()
                .map(|source| self.resolve_occurrence(source, workspace_scope))
                .collect(),
        )
    }

    pub(crate) fn resolve_requests(
        &self,
        collected: &SemanticReferenceRequestLedger,
        workspace_scope: WorkspaceResolutionScope,
    ) -> Result<SemanticReferenceRequestLedger, QuerySourceRequestError> {
        let mut terminal = SemanticReferenceRequestLedger::new();

        for request in collected.requests() {
            let SemanticReference::Name(target_name) = request.reference() else {
                return Err(invalid_collected_request(request));
            };
            let [expected_kind] = request.expected_kinds() else {
                return Err(invalid_collected_request(request));
            };
            if request.category() != SemanticReferenceCategory::QuerySource
                || !is_supported_expected_kind(*expected_kind)
            {
                return Err(invalid_collected_request(request));
            }

            let outcome = self.resolve_name(target_name.as_str(), *expected_kind, workspace_scope);
            let provenance = [query_source_resolver_provenance(
                request,
                *expected_kind,
                workspace_scope,
                &outcome,
            )?];
            let request = match outcome {
                QuerySourceResolutionOutcome::Resolved { target_id } => request
                    .clone()
                    .into_resolved(target_id, *expected_kind, provenance)?,
                QuerySourceResolutionOutcome::MissingTarget => {
                    request.clone().into_missing_target(provenance)?
                }
                QuerySourceResolutionOutcome::PartialWorkspaceTargetAbsent => {
                    request.clone().into_partial_workspace([], provenance)?
                }
                QuerySourceResolutionOutcome::AmbiguousTarget { candidates } => request
                    .clone()
                    .into_ambiguous_target(candidates, provenance)?,
                QuerySourceResolutionOutcome::IncompatibleTargetKind { candidates } => request
                    .clone()
                    .into_incompatible_target_kind(candidates, provenance)?,
            };
            terminal.insert(request)?;
        }

        Ok(terminal)
    }

    #[cfg(test)]
    fn resolve_occurrence(
        &self,
        source: &QuerySourceOccurrence,
        workspace_scope: WorkspaceResolutionScope,
    ) -> QuerySourceResolutionOutcome {
        let expected_kind = expected_metadata_kind(source.category());
        self.resolve_name(source.local_name(), expected_kind, workspace_scope)
    }

    fn resolve_name(
        &self,
        target_name: &str,
        expected_kind: NodeKind,
        workspace_scope: WorkspaceResolutionScope,
    ) -> QuerySourceResolutionOutcome {
        let lookup_key = query_source_lookup_key(target_name);
        let Some(candidates) = self.candidates_by_lookup_key.get(&lookup_key) else {
            return absent_target_outcome(workspace_scope);
        };

        debug_assert!(
            candidates.values().all(|candidate| {
                query_source_lookup_key(candidate.name.as_str()) == lookup_key
            })
        );

        let compatible = candidates
            .values()
            .filter(|candidate| candidate.kind == expected_kind)
            .map(|candidate| candidate.id.clone())
            .collect::<Vec<_>>();

        match compatible.as_slice() {
            [] => QuerySourceResolutionOutcome::IncompatibleTargetKind {
                candidates: candidates
                    .values()
                    .map(|candidate| candidate.id.clone())
                    .collect(),
            },
            [target_id] => QuerySourceResolutionOutcome::Resolved {
                target_id: target_id.clone(),
            },
            _ => QuerySourceResolutionOutcome::AmbiguousTarget {
                candidates: compatible,
            },
        }
    }
}

fn invalid_collected_request(request: &SemanticReferenceRequest) -> QuerySourceRequestError {
    QuerySourceRequestError::InvalidCollectedRequest {
        request_id: request.id().clone(),
    }
}

fn query_source_collection_provenance(
    module_source: Option<&EntityId>,
    query: &BslQuery,
    source: &QuerySourceOccurrence,
) -> Result<Provenance, QuerySourceRequestError> {
    let mut context = module_source.map_or_else(
        || query.id().as_str().to_owned(),
        |source| source.as_str().to_owned(),
    );
    context.push_str("#query_source_request");
    append_context(&mut context, "query", query.id().as_str());
    append_context(&mut context, "owner", query.owner_id().as_str());
    append_context(&mut context, "binding", query.binding_name().as_str());
    append_context(&mut context, "declaration_line", &query.line().to_string());
    append_context(&mut context, "raw_source", source.raw_spelling());
    append_context(
        &mut context,
        "range",
        &format!(
            "{}..{}",
            source.location().start_byte(),
            source.location().end_byte()
        ),
    );
    append_context(
        &mut context,
        "category",
        query_source_category_name(source.category()),
    );
    append_context(&mut context, "namespace", source.namespace());
    append_context(&mut context, "local_name", source.local_name());
    if let Some(alias) = source.alias() {
        append_context(&mut context, "alias", alias);
    }

    request_provenance(
        context,
        QUERY_SOURCE_COLLECTOR,
        FactOrigin::Parsed,
        ResolutionState::Unresolved,
    )
}

fn query_source_resolver_provenance(
    request: &SemanticReferenceRequest,
    expected_kind: NodeKind,
    workspace_scope: WorkspaceResolutionScope,
    outcome: &QuerySourceResolutionOutcome,
) -> Result<Provenance, QuerySourceRequestError> {
    let mut context = request.source_node().as_str().to_owned();
    context.push_str("#query_source_resolution");
    append_context(&mut context, "request", request.id().as_str());
    let SemanticReference::Name(target_name) = request.reference() else {
        return Err(invalid_collected_request(request));
    };
    append_context(&mut context, "target_name", target_name.as_str());
    append_context(
        &mut context,
        "lookup_key",
        &query_source_lookup_key(target_name.as_str()),
    );
    append_context(
        &mut context,
        "expected_kind",
        query_target_kind_name(expected_kind),
    );
    append_context(
        &mut context,
        "workspace_scope",
        workspace_scope_name(workspace_scope),
    );
    append_context(&mut context, "outcome", query_source_outcome_name(outcome));
    for candidate in query_source_outcome_candidates(outcome) {
        append_context(&mut context, "candidate", candidate.as_str());
    }

    request_provenance(
        context,
        QUERY_SOURCE_RESOLVER,
        FactOrigin::Resolved,
        query_source_outcome_state(outcome),
    )
}

fn request_provenance(
    context: String,
    producer: &'static str,
    origin: FactOrigin,
    resolution: ResolutionState,
) -> Result<Provenance, QuerySourceRequestError> {
    let source =
        EntityId::new(context).map_err(|_| QuerySourceRequestError::InvalidSourceIdentifier)?;
    Ok(Provenance::new(
        Some(source),
        ProducerId::new(producer),
        origin,
        Confidence::Exact,
        resolution,
    ))
}

fn append_context(context: &mut String, key: &str, value: &str) {
    use std::fmt::Write as _;

    write!(context, ";{key}#{}:{value}", value.len())
        .expect("writing query source provenance context to a String must succeed");
}

const fn query_source_outcome_state(outcome: &QuerySourceResolutionOutcome) -> ResolutionState {
    match outcome {
        QuerySourceResolutionOutcome::Resolved { .. } => ResolutionState::Resolved,
        QuerySourceResolutionOutcome::MissingTarget
        | QuerySourceResolutionOutcome::IncompatibleTargetKind { .. } => {
            ResolutionState::Unresolved
        }
        QuerySourceResolutionOutcome::AmbiguousTarget { .. } => ResolutionState::Ambiguous,
        QuerySourceResolutionOutcome::PartialWorkspaceTargetAbsent => ResolutionState::Partial,
    }
}

const fn query_source_outcome_name(outcome: &QuerySourceResolutionOutcome) -> &'static str {
    match outcome {
        QuerySourceResolutionOutcome::Resolved { .. } => "resolved",
        QuerySourceResolutionOutcome::MissingTarget => "missing_target",
        QuerySourceResolutionOutcome::AmbiguousTarget { .. } => "ambiguous_target",
        QuerySourceResolutionOutcome::IncompatibleTargetKind { .. } => "incompatible_target_kind",
        QuerySourceResolutionOutcome::PartialWorkspaceTargetAbsent => "partial_workspace",
    }
}

fn query_source_outcome_candidates(outcome: &QuerySourceResolutionOutcome) -> &[EntityId] {
    match outcome {
        QuerySourceResolutionOutcome::Resolved { target_id } => std::slice::from_ref(target_id),
        QuerySourceResolutionOutcome::AmbiguousTarget { candidates }
        | QuerySourceResolutionOutcome::IncompatibleTargetKind { candidates } => candidates,
        QuerySourceResolutionOutcome::MissingTarget
        | QuerySourceResolutionOutcome::PartialWorkspaceTargetAbsent => &[],
    }
}

fn query_source_lookup_key(value: &str) -> String {
    value.to_lowercase()
}

const fn expected_metadata_kind(category: QuerySourceCategory) -> NodeKind {
    match category {
        QuerySourceCategory::Catalog => NodeKind::Metadata(MetadataKind::Catalog),
        QuerySourceCategory::InformationRegister => {
            NodeKind::Metadata(MetadataKind::InformationRegister)
        }
        QuerySourceCategory::AccumulationRegister => {
            NodeKind::Metadata(MetadataKind::AccumulationRegister)
        }
        QuerySourceCategory::AccountingRegister => {
            NodeKind::Metadata(MetadataKind::AccountingRegister)
        }
    }
}

const fn is_supported_expected_kind(kind: NodeKind) -> bool {
    matches!(
        kind,
        NodeKind::Metadata(
            MetadataKind::Catalog
                | MetadataKind::InformationRegister
                | MetadataKind::AccumulationRegister
                | MetadataKind::AccountingRegister
        )
    )
}

const fn query_source_category_name(category: QuerySourceCategory) -> &'static str {
    match category {
        QuerySourceCategory::Catalog => "catalog",
        QuerySourceCategory::InformationRegister => "information_register",
        QuerySourceCategory::AccumulationRegister => "accumulation_register",
        QuerySourceCategory::AccountingRegister => "accounting_register",
    }
}

const fn query_target_kind_name(kind: NodeKind) -> &'static str {
    match kind {
        NodeKind::Metadata(MetadataKind::Catalog) => "metadata.catalog",
        NodeKind::Metadata(MetadataKind::InformationRegister) => "metadata.information_register",
        NodeKind::Metadata(MetadataKind::AccumulationRegister) => "metadata.accumulation_register",
        NodeKind::Metadata(MetadataKind::AccountingRegister) => "metadata.accounting_register",
        _ => "unsupported",
    }
}

const fn workspace_scope_name(scope: WorkspaceResolutionScope) -> &'static str {
    match scope {
        WorkspaceResolutionScope::Complete => "complete",
        WorkspaceResolutionScope::Partial => "partial",
    }
}

const fn absent_target_outcome(
    workspace_scope: WorkspaceResolutionScope,
) -> QuerySourceResolutionOutcome {
    match workspace_scope {
        WorkspaceResolutionScope::Complete => QuerySourceResolutionOutcome::MissingTarget,
        WorkspaceResolutionScope::Partial => {
            QuerySourceResolutionOutcome::PartialWorkspaceTargetAbsent
        }
    }
}

#[cfg(test)]
mod tests {
    use oneagent_bsl::{BslQuery, QueryLanguageParser};
    use oneagent_common::{EntityId, EntityName};
    use oneagent_graph::{
        FactOrigin, GraphNode, NodeKind, ResolutionState, SemanticGraph, SemanticReference,
        SemanticReferenceCategory, SemanticReferenceRequestError, SemanticReferenceRequestLedger,
        SemanticReferenceRequestOutcome,
    };
    use oneagent_metadata::MetadataKind;

    use super::{
        QUERY_SOURCE_COLLECTOR, QUERY_SOURCE_RESOLVER, QuerySourceResolutionIndex,
        QuerySourceResolutionOutcome, WorkspaceResolutionScope, collect_query_source_requests,
    };

    fn id(value: &str) -> EntityId {
        EntityId::new(value).expect("identifier must be valid")
    }

    fn name(value: &str) -> EntityName {
        EntityName::new(value).expect("name must be valid")
    }

    fn insert_node(graph: &mut SemanticGraph, identifier: &str, value: &str, kind: NodeKind) {
        graph.insert_node(GraphNode::new(id(identifier), name(value), kind));
    }

    fn query(identifier: &str, text: &str) -> BslQuery {
        BslQuery::new(
            id(identifier),
            id("module.sales.procedure.run"),
            name("Run"),
            name("Query"),
            text.to_owned(),
            42,
        )
    }

    fn collected(query: &BslQuery) -> oneagent_graph::SemanticReferenceRequestLedger {
        let parse_result = QueryLanguageParser.parse(query.text());
        collect_query_source_requests(
            &parse_result,
            Some(&id("oneagent://source/CommonModules/Sales/Module.bsl")),
            query,
        )
        .expect("accepted query source must produce a request ledger")
        .expect("accepted query source must enter collection")
    }

    fn resolve(
        graph: &SemanticGraph,
        source: &str,
        workspace_scope: WorkspaceResolutionScope,
    ) -> Option<Vec<QuerySourceResolutionOutcome>> {
        let parse_result = QueryLanguageParser.parse(source);

        QuerySourceResolutionIndex::new(graph).resolve(&parse_result, workspace_scope)
    }

    #[test]
    fn collects_one_canonical_query_source_request_with_occurrence_provenance() {
        let query = query(
            "query.sales.products",
            "SELECT Ref FROM Catalog.Products AS Products",
        );
        let ledger = collected(&query);
        let request = &ledger.requests()[0];

        assert_eq!(ledger.len(), 1);
        assert_eq!(request.source_node(), query.id());
        assert_eq!(request.category(), SemanticReferenceCategory::QuerySource);
        assert!(matches!(
            request.reference(),
            SemanticReference::Name(target) if target.as_str() == "Products"
        ));
        assert_eq!(
            request.expected_kinds(),
            [NodeKind::Metadata(MetadataKind::Catalog)]
        );
        assert_eq!(request.state(), ResolutionState::Unresolved);
        assert_eq!(
            request.outcome(),
            SemanticReferenceRequestOutcome::Collected
        );
        assert!(request.candidates().is_empty());
        assert_eq!(request.provenance().len(), 1);

        let provenance = &request.provenance()[0];
        assert_eq!(provenance.producer().as_str(), QUERY_SOURCE_COLLECTOR);
        assert_eq!(provenance.origin(), FactOrigin::Parsed);
        assert_eq!(provenance.resolution(), ResolutionState::Unresolved);
        let source = provenance
            .source()
            .expect("collection provenance must retain source context")
            .as_str();
        for evidence in [
            "oneagent://source/CommonModules/Sales/Module.bsl",
            "query.sales.products",
            "module.sales.procedure.run",
            "Catalog.Products",
            "16..32",
            "catalog",
        ] {
            assert!(source.contains(evidence), "missing evidence `{evidence}`");
        }
    }

    #[test]
    fn resolves_all_supported_categories_to_exact_terminal_request_kinds() {
        let cases = [
            (
                "query.catalog",
                "SELECT Ref FROM Catalog.Products",
                "catalog.products",
                "Products",
                MetadataKind::Catalog,
            ),
            (
                "query.information-register",
                "SELECT Ref FROM InformationRegister.ObjectsToDelete",
                "information-register.objects-to-delete",
                "ObjectsToDelete",
                MetadataKind::InformationRegister,
            ),
            (
                "query.accumulation-register",
                "SELECT Ref FROM AccumulationRegister.InventoryCost",
                "accumulation-register.inventory-cost",
                "InventoryCost",
                MetadataKind::AccumulationRegister,
            ),
            (
                "query.accounting-register",
                "SELECT Ref FROM AccountingRegister.FinancialAccounting",
                "accounting-register.financial-accounting",
                "FinancialAccounting",
                MetadataKind::AccountingRegister,
            ),
        ];
        let mut graph = SemanticGraph::new();
        let mut collected_requests = SemanticReferenceRequestLedger::new();

        for (query_id, source, target_id, target_name, metadata_kind) in cases {
            insert_node(
                &mut graph,
                target_id,
                target_name,
                NodeKind::Metadata(metadata_kind),
            );
            for request in collected(&query(query_id, source)).requests() {
                collected_requests
                    .insert(request.clone())
                    .expect("distinct collected requests must aggregate");
            }
        }

        let terminal = QuerySourceResolutionIndex::new(&graph)
            .resolve_requests(&collected_requests, WorkspaceResolutionScope::Complete)
            .expect("all exact targets must resolve");

        assert_eq!(terminal.len(), 4);
        for request in terminal.requests() {
            assert_eq!(request.outcome(), SemanticReferenceRequestOutcome::Resolved);
            assert_eq!(request.state(), ResolutionState::Resolved);
            assert_eq!(request.expected_kinds().len(), 1);
            assert_eq!(request.candidates().len(), 1);
            assert_eq!(request.provenance().len(), 2);
            assert!(request.provenance().iter().any(|provenance| {
                provenance.producer().as_str() == QUERY_SOURCE_RESOLVER
                    && provenance.origin() == FactOrigin::Resolved
                    && provenance.resolution() == ResolutionState::Resolved
            }));
        }
    }

    #[test]
    fn terminal_requests_distinguish_missing_partial_ambiguous_and_incompatible() {
        let missing_query = query("query.missing", "SELECT Ref FROM Catalog.Missing");
        let missing = collected(&missing_query);
        let empty = SemanticGraph::new();
        let complete = QuerySourceResolutionIndex::new(&empty)
            .resolve_requests(&missing, WorkspaceResolutionScope::Complete)
            .expect("complete resolution must finish");
        let partial = QuerySourceResolutionIndex::new(&empty)
            .resolve_requests(&missing, WorkspaceResolutionScope::Partial)
            .expect("partial resolution must finish");

        assert_eq!(
            complete.requests()[0].outcome(),
            SemanticReferenceRequestOutcome::MissingTarget
        );
        assert_eq!(complete.requests()[0].state(), ResolutionState::Unresolved);
        assert!(complete.requests()[0].candidates().is_empty());
        assert_eq!(
            partial.requests()[0].outcome(),
            SemanticReferenceRequestOutcome::PartialWorkspace
        );
        assert_eq!(partial.requests()[0].state(), ResolutionState::Partial);
        assert!(partial.requests()[0].candidates().is_empty());

        let products_query = query("query.products", "SELECT Ref FROM Catalog.Products");
        let products = collected(&products_query);
        let mut ambiguous_graph = SemanticGraph::new();
        insert_node(
            &mut ambiguous_graph,
            "catalog.z",
            "Products",
            NodeKind::Metadata(MetadataKind::Catalog),
        );
        insert_node(
            &mut ambiguous_graph,
            "catalog.a",
            "PRODUCTS",
            NodeKind::Metadata(MetadataKind::Catalog),
        );
        insert_node(
            &mut ambiguous_graph,
            "document.products",
            "products",
            NodeKind::Metadata(MetadataKind::Document),
        );
        let ambiguous = QuerySourceResolutionIndex::new(&ambiguous_graph)
            .resolve_requests(&products, WorkspaceResolutionScope::Complete)
            .expect("ambiguous resolution must finish");
        assert_eq!(
            ambiguous.requests()[0].outcome(),
            SemanticReferenceRequestOutcome::AmbiguousTarget
        );
        assert_eq!(
            ambiguous.requests()[0].candidates(),
            [id("catalog.a"), id("catalog.z")]
        );

        let mut incompatible_graph = SemanticGraph::new();
        insert_node(
            &mut incompatible_graph,
            "unknown.products",
            "Products",
            NodeKind::Unknown,
        );
        insert_node(
            &mut incompatible_graph,
            "document.products",
            "PRODUCTS",
            NodeKind::Metadata(MetadataKind::Document),
        );
        let incompatible = QuerySourceResolutionIndex::new(&incompatible_graph)
            .resolve_requests(&products, WorkspaceResolutionScope::Complete)
            .expect("incompatible resolution must finish");
        assert_eq!(
            incompatible.requests()[0].outcome(),
            SemanticReferenceRequestOutcome::IncompatibleTargetKind
        );
        assert_eq!(
            incompatible.requests()[0].candidates(),
            [id("document.products"), id("unknown.products")]
        );

        insert_node(
            &mut incompatible_graph,
            "catalog.products",
            "products",
            NodeKind::Metadata(MetadataKind::Catalog),
        );
        let resolved = QuerySourceResolutionIndex::new(&incompatible_graph)
            .resolve_requests(&products, WorkspaceResolutionScope::Complete)
            .expect("one compatible candidate must win");
        assert_eq!(
            resolved.requests()[0].outcome(),
            SemanticReferenceRequestOutcome::Resolved
        );
        assert_eq!(
            resolved.requests()[0].candidates(),
            [id("catalog.products")]
        );
    }

    #[test]
    fn duplicate_reordered_and_repeated_request_resolution_is_deterministic() {
        let first_query = query("query.products", "SELECT Ref FROM Catalog.Products");
        let shifted_query = query("query.products", "  SELECT Ref FROM Catalog.Products");
        let mut duplicates = collected(&first_query);
        let shifted = collected(&shifted_query);
        let first_id = duplicates.requests()[0].id().clone();
        assert_eq!(first_id, shifted.requests()[0].id().clone());
        assert!(
            !duplicates
                .insert(shifted.requests()[0].clone())
                .expect("equivalent collection observations must merge")
        );
        assert_eq!(duplicates.len(), 1);
        assert_eq!(duplicates.requests()[0].provenance().len(), 2);

        let nodes = [
            ("catalog.z", "Products", MetadataKind::Catalog),
            ("catalog.a", "PRODUCTS", MetadataKind::Catalog),
            ("document.products", "products", MetadataKind::Document),
        ];
        let mut normal = SemanticGraph::new();
        let mut reversed = SemanticGraph::new();
        for (identifier, target_name, kind) in nodes {
            insert_node(
                &mut normal,
                identifier,
                target_name,
                NodeKind::Metadata(kind),
            );
        }
        for (identifier, target_name, kind) in nodes.into_iter().rev() {
            insert_node(
                &mut reversed,
                identifier,
                target_name,
                NodeKind::Metadata(kind),
            );
        }

        let first = QuerySourceResolutionIndex::new(&normal)
            .resolve_requests(&duplicates, WorkspaceResolutionScope::Complete)
            .expect("normal resolution must finish");
        let repeated = QuerySourceResolutionIndex::new(&normal)
            .resolve_requests(&duplicates, WorkspaceResolutionScope::Complete)
            .expect("repeated resolution must finish");
        let reordered = QuerySourceResolutionIndex::new(&reversed)
            .resolve_requests(&duplicates, WorkspaceResolutionScope::Complete)
            .expect("reordered resolution must finish");

        assert_eq!(first, repeated);
        assert_eq!(first, reordered);
        assert_eq!(first.requests()[0].provenance().len(), 3);
        assert_eq!(
            first.requests()[0].candidates(),
            [id("catalog.a"), id("catalog.z")]
        );
    }

    #[test]
    fn conflicting_terminal_content_for_one_request_identity_is_rejected() {
        let query = query("query.products", "SELECT Ref FROM Catalog.Products");
        let collected = collected(&query);
        let empty = SemanticGraph::new();
        let missing = QuerySourceResolutionIndex::new(&empty)
            .resolve_requests(&collected, WorkspaceResolutionScope::Complete)
            .expect("missing resolution must finish");
        let mut graph = SemanticGraph::new();
        insert_node(
            &mut graph,
            "catalog.products",
            "Products",
            NodeKind::Metadata(MetadataKind::Catalog),
        );
        let resolved = QuerySourceResolutionIndex::new(&graph)
            .resolve_requests(&collected, WorkspaceResolutionScope::Complete)
            .expect("resolved resolution must finish");
        let mut combined = missing;

        assert!(matches!(
            combined.insert(resolved.requests()[0].clone()),
            Err(SemanticReferenceRequestError::ConflictingTerminalContent { .. })
        ));
    }

    #[test]
    fn resolves_english_and_russian_catalog_names_case_insensitively() {
        let mut english_graph = SemanticGraph::new();
        insert_node(
            &mut english_graph,
            "catalog.products",
            "products",
            NodeKind::Metadata(MetadataKind::Catalog),
        );
        let english = QueryLanguageParser.parse("SELECT Ref FROM Catalog.PRODUCTS");
        let english_source = &english.program().expect("query must parse").sources()[0];
        let english_outcomes = QuerySourceResolutionIndex::new(&english_graph)
            .resolve(&english, WorkspaceResolutionScope::Complete)
            .expect("complete query must be resolved");

        assert_eq!(english_source.raw_spelling(), "Catalog.PRODUCTS");
        assert_eq!(english_source.local_name(), "PRODUCTS");
        assert_eq!(
            english_outcomes,
            vec![QuerySourceResolutionOutcome::Resolved {
                target_id: id("catalog.products"),
            }]
        );

        let mut russian_graph = SemanticGraph::new();
        insert_node(
            &mut russian_graph,
            "catalog.nomenclature",
            "номенклатура",
            NodeKind::Metadata(MetadataKind::Catalog),
        );

        assert_eq!(
            resolve(
                &russian_graph,
                "ВЫБРАТЬ Ссылка ИЗ Справочник.НОМЕНКЛАТУРА",
                WorkspaceResolutionScope::Complete,
            ),
            Some(vec![QuerySourceResolutionOutcome::Resolved {
                target_id: id("catalog.nomenclature"),
            }])
        );
    }

    #[test]
    fn resolves_information_register_by_category_and_exact_kind() {
        let mut graph = SemanticGraph::new();
        insert_node(
            &mut graph,
            "information-register.objects-to-delete",
            "objectstodelete",
            NodeKind::Metadata(MetadataKind::InformationRegister),
        );

        assert_eq!(
            resolve(
                &graph,
                "SELECT Ref FROM InformationRegister.ObjectsToDelete AS Tab",
                WorkspaceResolutionScope::Complete,
            ),
            Some(vec![QuerySourceResolutionOutcome::Resolved {
                target_id: id("information-register.objects-to-delete"),
            }])
        );
    }

    #[test]
    fn preserves_multi_scalar_unicode_lowercase_expansion() {
        let mut graph = SemanticGraph::new();
        insert_node(
            &mut graph,
            "catalog.expanded-name",
            "i\u{307}tem",
            NodeKind::Metadata(MetadataKind::Catalog),
        );

        assert_eq!(
            resolve(
                &graph,
                "SELECT Ref FROM Catalog.İTEM",
                WorkspaceResolutionScope::Complete,
            ),
            Some(vec![QuerySourceResolutionOutcome::Resolved {
                target_id: id("catalog.expanded-name"),
            }])
        );
    }

    #[test]
    fn does_not_apply_unicode_normalization() {
        let mut graph = SemanticGraph::new();
        insert_node(
            &mut graph,
            "catalog.decomposed-name",
            "Cafe\u{301}",
            NodeKind::Metadata(MetadataKind::Catalog),
        );

        assert_eq!(
            resolve(
                &graph,
                "SELECT Ref FROM Catalog.Café",
                WorkspaceResolutionScope::Complete,
            ),
            Some(vec![QuerySourceResolutionOutcome::MissingTarget])
        );
    }

    #[test]
    fn unique_compatible_candidate_wins_over_incompatible_candidates() {
        let mut graph = SemanticGraph::new();
        insert_node(
            &mut graph,
            "document.products",
            "PRODUCTS",
            NodeKind::Metadata(MetadataKind::Document),
        );
        insert_node(
            &mut graph,
            "catalog.products",
            "Products",
            NodeKind::Metadata(MetadataKind::Catalog),
        );

        assert_eq!(
            resolve(
                &graph,
                "SELECT Ref FROM Catalog.products",
                WorkspaceResolutionScope::Complete,
            ),
            Some(vec![QuerySourceResolutionOutcome::Resolved {
                target_id: id("catalog.products"),
            }])
        );
    }

    #[test]
    fn compatible_collisions_are_ambiguous_and_deterministically_ordered() {
        let nodes = [
            ("catalog.z", "PRODUCTS"),
            ("catalog.a", "Products"),
            ("catalog.m", "products"),
        ];
        let mut normal = SemanticGraph::new();
        let mut reversed = SemanticGraph::new();

        for (identifier, value) in nodes {
            insert_node(
                &mut normal,
                identifier,
                value,
                NodeKind::Metadata(MetadataKind::Catalog),
            );
        }
        for (identifier, value) in nodes.into_iter().rev() {
            insert_node(
                &mut reversed,
                identifier,
                value,
                NodeKind::Metadata(MetadataKind::Catalog),
            );
        }

        let expected = Some(vec![QuerySourceResolutionOutcome::AmbiguousTarget {
            candidates: vec![id("catalog.a"), id("catalog.m"), id("catalog.z")],
        }]);
        let normal_outcome = resolve(
            &normal,
            "SELECT Ref FROM Catalog.Products",
            WorkspaceResolutionScope::Complete,
        );
        let reversed_outcome = resolve(
            &reversed,
            "SELECT Ref FROM Catalog.Products",
            WorkspaceResolutionScope::Complete,
        );

        assert_eq!(normal_outcome, expected);
        assert_eq!(reversed_outcome, expected);
    }

    #[test]
    fn incompatible_candidates_are_reported_in_deterministic_order() {
        let mut graph = SemanticGraph::new();
        insert_node(
            &mut graph,
            "unknown.products",
            "products",
            NodeKind::Unknown,
        );
        insert_node(
            &mut graph,
            "document.products",
            "PRODUCTS",
            NodeKind::Metadata(MetadataKind::Document),
        );

        assert_eq!(
            resolve(
                &graph,
                "SELECT Ref FROM Catalog.Products",
                WorkspaceResolutionScope::Partial,
            ),
            Some(vec![QuerySourceResolutionOutcome::IncompatibleTargetKind {
                candidates: vec![id("document.products"), id("unknown.products")],
            },])
        );
    }

    #[test]
    fn absent_target_uses_explicit_workspace_scope() {
        let graph = SemanticGraph::new();
        let complete = resolve(
            &graph,
            "SELECT Ref FROM Catalog.Missing",
            WorkspaceResolutionScope::Complete,
        );
        let partial = resolve(
            &graph,
            "SELECT Ref FROM Catalog.Missing",
            WorkspaceResolutionScope::Partial,
        );

        assert_eq!(
            complete,
            Some(vec![QuerySourceResolutionOutcome::MissingTarget])
        );
        assert_eq!(
            partial,
            Some(vec![
                QuerySourceResolutionOutcome::PartialWorkspaceTargetAbsent,
            ])
        );
    }

    #[test]
    fn rejected_parse_result_does_not_enter_resolution_or_emit_edges() {
        let mut graph = SemanticGraph::new();
        insert_node(
            &mut graph,
            "catalog.products",
            "Products",
            NodeKind::Metadata(MetadataKind::Catalog),
        );
        let parse_result = QueryLanguageParser.parse("SELECT Ref FROM Catalog.Products EXTRA");
        let outcomes = QuerySourceResolutionIndex::new(&graph)
            .resolve(&parse_result, WorkspaceResolutionScope::Complete);

        assert_eq!(outcomes, None);
        assert_eq!(graph.edges().count(), 0);
    }
}
