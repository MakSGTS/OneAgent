//! Deterministic resolution of parsed query sources against EDT metadata nodes.

use oneagent_bsl::{QueryLanguageParseResult, QuerySourceCategory, QuerySourceOccurrence};
use oneagent_common::{EntityId, EntityName};
use oneagent_graph::{NodeKind, SemanticGraph};
use oneagent_metadata::MetadataKind;
use std::collections::BTreeMap;

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

#[derive(Debug, Clone, PartialEq, Eq)]
struct QuerySourceCandidate {
    id: EntityId,
    name: EntityName,
    kind: NodeKind,
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

    /// Resolves every source only when parsing proved the complete accepted source set.
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
        if program
            .sources()
            .iter()
            .any(|source| !is_private_resolution_category(source.category()))
        {
            return None;
        }

        Some(
            program
                .sources()
                .iter()
                .map(|source| self.resolve_occurrence(source, workspace_scope))
                .collect(),
        )
    }

    fn resolve_occurrence(
        &self,
        source: &QuerySourceOccurrence,
        workspace_scope: WorkspaceResolutionScope,
    ) -> QuerySourceResolutionOutcome {
        let lookup_key = query_source_lookup_key(source.local_name());
        let expected_kind = expected_metadata_kind(source.category());
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

fn query_source_lookup_key(value: &str) -> String {
    value.to_lowercase()
}

const fn is_private_resolution_category(category: QuerySourceCategory) -> bool {
    matches!(
        category,
        QuerySourceCategory::Catalog | QuerySourceCategory::InformationRegister
    )
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
    use oneagent_bsl::QueryLanguageParser;
    use oneagent_common::{EntityId, EntityName};
    use oneagent_graph::{GraphNode, NodeKind, SemanticGraph};
    use oneagent_metadata::MetadataKind;

    use super::{
        QuerySourceResolutionIndex, QuerySourceResolutionOutcome, WorkspaceResolutionScope,
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

    fn resolve(
        graph: &SemanticGraph,
        source: &str,
        workspace_scope: WorkspaceResolutionScope,
    ) -> Option<Vec<QuerySourceResolutionOutcome>> {
        let parse_result = QueryLanguageParser.parse(source);

        QuerySourceResolutionIndex::new(graph).resolve(&parse_result, workspace_scope)
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
    fn parsed_register_categories_wait_for_the_public_request_task() {
        let graph = SemanticGraph::new();

        for source in [
            "SELECT Ref FROM AccumulationRegister.InventoryCost",
            "SELECT Ref FROM AccountingRegister.FinancialAccounting",
        ] {
            assert_eq!(
                resolve(&graph, source, WorkspaceResolutionScope::Complete),
                None
            );
        }
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
