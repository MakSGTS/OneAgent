//! Deterministic private resolution for parsed EDT Event Subscriptions.

use oneagent_common::{EntityId, EntityName};
use oneagent_graph::{GraphNode, NodeId, NodeKind, SemanticGraph, SemanticGraphQuery};
use oneagent_metadata::MetadataKind;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use crate::{
    EdtEventSubscriptionDescriptor, EdtEventSubscriptionHandler,
    EdtEventSubscriptionSourceObservation, EdtEventSubscriptionSourceOutcomeKind,
    EdtEventSubscriptionSourceReason,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum EdtEventSubscriptionSourceResolutionOutcome {
    Resolved {
        target_ids: Vec<EntityId>,
    },
    Missing,
    Ambiguous {
        candidates: Vec<EntityId>,
    },
    IncompatibleKind {
        candidates: Vec<EntityId>,
    },
    RejectedObservation {
        kind: EdtEventSubscriptionSourceOutcomeKind,
        reason: EdtEventSubscriptionSourceReason,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EdtEventSubscriptionSourceResolution {
    observation: EdtEventSubscriptionSourceObservation,
    outcome: EdtEventSubscriptionSourceResolutionOutcome,
}

impl EdtEventSubscriptionSourceResolution {
    pub(crate) const fn observation(&self) -> &EdtEventSubscriptionSourceObservation {
        &self.observation
    }

    pub(crate) const fn outcome(&self) -> &EdtEventSubscriptionSourceResolutionOutcome {
        &self.outcome
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum EdtEventSubscriptionHandlerResolutionOutcome {
    Resolved {
        target_id: EntityId,
    },
    MissingCommonModule,
    AmbiguousCommonModule {
        candidates: Vec<EntityId>,
    },
    IncompatibleCommonModuleKind {
        candidates: Vec<EntityId>,
    },
    MissingModule {
        owner_id: EntityId,
    },
    AmbiguousModule {
        owner_id: EntityId,
        candidates: Vec<EntityId>,
    },
    IncompatibleModuleKind {
        owner_id: EntityId,
        candidates: Vec<EntityId>,
    },
    MissingSymbol {
        module_id: EntityId,
    },
    AmbiguousSymbol {
        module_id: EntityId,
        candidates: Vec<EntityId>,
    },
    IncompatibleSymbolKind {
        module_id: EntityId,
        candidates: Vec<EntityId>,
    },
    InvalidOwner {
        module_id: EntityId,
        candidates: Vec<EntityId>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EdtEventSubscriptionHandlerResolution {
    handler: EdtEventSubscriptionHandler,
    outcome: EdtEventSubscriptionHandlerResolutionOutcome,
}

impl EdtEventSubscriptionHandlerResolution {
    pub(crate) const fn handler(&self) -> &EdtEventSubscriptionHandler {
        &self.handler
    }

    pub(crate) const fn outcome(&self) -> &EdtEventSubscriptionHandlerResolutionOutcome {
        &self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EdtEventSubscriptionProcessedOutcomeKind {
    Resolved,
    Missing,
    Ambiguous,
    IncompatibleKind,
    InvalidOwner,
    RejectedObservation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum EdtEventSubscriptionProcessedObservation {
    Source { raw_selector: String },
    Handler { raw_path: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EdtEventSubscriptionProcessedOutcome {
    observation: EdtEventSubscriptionProcessedObservation,
    kind: EdtEventSubscriptionProcessedOutcomeKind,
}

impl EdtEventSubscriptionProcessedOutcome {
    pub(crate) const fn observation(&self) -> &EdtEventSubscriptionProcessedObservation {
        &self.observation
    }

    pub(crate) const fn kind(&self) -> EdtEventSubscriptionProcessedOutcomeKind {
        self.kind
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EdtEventSubscriptionResolution {
    descriptor_id: EntityId,
    descriptor_path: PathBuf,
    sources: Vec<EdtEventSubscriptionSourceResolution>,
    handler: EdtEventSubscriptionHandlerResolution,
    processed_outcomes: Vec<EdtEventSubscriptionProcessedOutcome>,
}

impl EdtEventSubscriptionResolution {
    pub(crate) const fn descriptor_id(&self) -> &EntityId {
        &self.descriptor_id
    }

    pub(crate) fn descriptor_path(&self) -> &Path {
        &self.descriptor_path
    }

    pub(crate) fn sources(&self) -> &[EdtEventSubscriptionSourceResolution] {
        &self.sources
    }

    pub(crate) const fn handler(&self) -> &EdtEventSubscriptionHandlerResolution {
        &self.handler
    }

    pub(crate) fn processed_outcomes(&self) -> &[EdtEventSubscriptionProcessedOutcome] {
        &self.processed_outcomes
    }

    pub(crate) fn resolved_source_target_ids(&self) -> Vec<EntityId> {
        self.sources
            .iter()
            .filter_map(|source| match source.outcome() {
                EdtEventSubscriptionSourceResolutionOutcome::Resolved { target_ids } => {
                    Some(target_ids.iter())
                }
                EdtEventSubscriptionSourceResolutionOutcome::Missing
                | EdtEventSubscriptionSourceResolutionOutcome::Ambiguous { .. }
                | EdtEventSubscriptionSourceResolutionOutcome::IncompatibleKind { .. }
                | EdtEventSubscriptionSourceResolutionOutcome::RejectedObservation { .. } => None,
            })
            .flatten()
            .cloned()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct EdtEventSubscriptionResolutionIndex<'graph> {
    query: SemanticGraphQuery<'graph>,
}

impl<'graph> EdtEventSubscriptionResolutionIndex<'graph> {
    #[must_use]
    pub(crate) fn new(graph: &'graph SemanticGraph) -> Self {
        Self {
            query: graph.query(),
        }
    }

    #[must_use]
    pub(crate) fn resolve(
        &self,
        descriptor: &EdtEventSubscriptionDescriptor,
    ) -> EdtEventSubscriptionResolution {
        let sources = descriptor
            .sources()
            .iter()
            .cloned()
            .map(|observation| self.resolve_source(observation))
            .collect::<Vec<_>>();
        let handler = self.resolve_handler(descriptor.handler());
        let mut processed_outcomes = sources
            .iter()
            .map(source_processed_outcome)
            .collect::<Vec<_>>();
        processed_outcomes.push(handler_processed_outcome(&handler));

        EdtEventSubscriptionResolution {
            descriptor_id: descriptor.id().clone(),
            descriptor_path: descriptor.descriptor_path().to_path_buf(),
            sources,
            handler,
            processed_outcomes,
        }
    }

    fn resolve_source(
        &self,
        observation: EdtEventSubscriptionSourceObservation,
    ) -> EdtEventSubscriptionSourceResolution {
        let outcome = match observation.outcome() {
            EdtEventSubscriptionSourceOutcomeKind::Unsupported
            | EdtEventSubscriptionSourceOutcomeKind::Malformed => {
                EdtEventSubscriptionSourceResolutionOutcome::RejectedObservation {
                    kind: observation.outcome(),
                    reason: observation
                        .reason()
                        .expect("rejected source observation must retain its reason"),
                }
            }
            EdtEventSubscriptionSourceOutcomeKind::Supported => {
                let expected = NodeKind::Metadata(
                    observation
                        .target_kind()
                        .expect("supported source observation must retain its target kind"),
                );
                match observation.target_name() {
                    Some(name) => self.resolve_qualified_source(name, expected),
                    None => self.resolve_bare_source(expected),
                }
            }
        };

        EdtEventSubscriptionSourceResolution {
            observation,
            outcome,
        }
    }

    fn resolve_qualified_source(
        &self,
        name: &EntityName,
        expected: NodeKind,
    ) -> EdtEventSubscriptionSourceResolutionOutcome {
        let candidates = self.query.nodes_by_name(name);
        let compatible = candidate_ids_of_kind(&candidates, expected);

        match compatible.as_slice() {
            [] if candidates.is_empty() => EdtEventSubscriptionSourceResolutionOutcome::Missing,
            [] => EdtEventSubscriptionSourceResolutionOutcome::IncompatibleKind {
                candidates: candidate_ids(&candidates),
            },
            [target_id] => EdtEventSubscriptionSourceResolutionOutcome::Resolved {
                target_ids: vec![target_id.clone()],
            },
            _ => EdtEventSubscriptionSourceResolutionOutcome::Ambiguous {
                candidates: compatible,
            },
        }
    }

    fn resolve_bare_source(
        &self,
        expected: NodeKind,
    ) -> EdtEventSubscriptionSourceResolutionOutcome {
        let target_ids = candidate_ids(&self.query.nodes_by_kind(expected));
        if target_ids.is_empty() {
            EdtEventSubscriptionSourceResolutionOutcome::Missing
        } else {
            EdtEventSubscriptionSourceResolutionOutcome::Resolved { target_ids }
        }
    }

    fn resolve_handler(
        &self,
        handler: &EdtEventSubscriptionHandler,
    ) -> EdtEventSubscriptionHandlerResolution {
        let common_module_candidates = self.query.nodes_by_name(handler.module_name());
        let compatible_common_modules = candidate_ids_of_kind(
            &common_module_candidates,
            NodeKind::Metadata(MetadataKind::CommonModule),
        );
        let outcome = match compatible_common_modules.as_slice() {
            [] if common_module_candidates.is_empty() => {
                EdtEventSubscriptionHandlerResolutionOutcome::MissingCommonModule
            }
            [] => EdtEventSubscriptionHandlerResolutionOutcome::IncompatibleCommonModuleKind {
                candidates: candidate_ids(&common_module_candidates),
            },
            [owner_id] => self.resolve_handler_module(owner_id, handler.procedure_name()),
            _ => EdtEventSubscriptionHandlerResolutionOutcome::AmbiguousCommonModule {
                candidates: compatible_common_modules,
            },
        };

        EdtEventSubscriptionHandlerResolution {
            handler: handler.clone(),
            outcome,
        }
    }

    fn resolve_handler_module(
        &self,
        owner_id: &EntityId,
        procedure_name: &EntityName,
    ) -> EdtEventSubscriptionHandlerResolutionOutcome {
        let owner_node_id = NodeId::new(owner_id.as_str());
        let children = self.query.children(&owner_node_id);
        let modules = candidate_ids_of_kind(&children, NodeKind::Module);

        match modules.as_slice() {
            [] if children.is_empty() => {
                EdtEventSubscriptionHandlerResolutionOutcome::MissingModule {
                    owner_id: owner_id.clone(),
                }
            }
            [] => EdtEventSubscriptionHandlerResolutionOutcome::IncompatibleModuleKind {
                owner_id: owner_id.clone(),
                candidates: candidate_ids(&children),
            },
            [module_id] => self.resolve_handler_symbol(module_id, procedure_name),
            _ => EdtEventSubscriptionHandlerResolutionOutcome::AmbiguousModule {
                owner_id: owner_id.clone(),
                candidates: modules,
            },
        }
    }

    fn resolve_handler_symbol(
        &self,
        module_id: &EntityId,
        procedure_name: &EntityName,
    ) -> EdtEventSubscriptionHandlerResolutionOutcome {
        let module_node_id = NodeId::new(module_id.as_str());
        let named_children = self
            .query
            .children(&module_node_id)
            .into_iter()
            .filter(|node| node.name() == procedure_name)
            .collect::<Vec<_>>();
        let procedures = candidate_ids_of_kind(&named_children, NodeKind::Procedure);

        match procedures.as_slice() {
            [target_id] => EdtEventSubscriptionHandlerResolutionOutcome::Resolved {
                target_id: target_id.clone(),
            },
            [] if !named_children.is_empty() => {
                EdtEventSubscriptionHandlerResolutionOutcome::IncompatibleSymbolKind {
                    module_id: module_id.clone(),
                    candidates: candidate_ids(&named_children),
                }
            }
            [] => {
                let elsewhere = self
                    .query
                    .nodes_by_name_and_kind(procedure_name, NodeKind::Procedure)
                    .into_iter()
                    .filter(|node| !self.is_owned_by(node, module_id))
                    .collect::<Vec<_>>();
                if elsewhere.is_empty() {
                    EdtEventSubscriptionHandlerResolutionOutcome::MissingSymbol {
                        module_id: module_id.clone(),
                    }
                } else {
                    EdtEventSubscriptionHandlerResolutionOutcome::InvalidOwner {
                        module_id: module_id.clone(),
                        candidates: candidate_ids(&elsewhere),
                    }
                }
            }
            _ => EdtEventSubscriptionHandlerResolutionOutcome::AmbiguousSymbol {
                module_id: module_id.clone(),
                candidates: procedures,
            },
        }
    }

    fn is_owned_by(&self, node: &GraphNode, owner_id: &EntityId) -> bool {
        self.query
            .owners(&NodeId::new(node.id().as_str()))
            .iter()
            .any(|owner| owner.id() == owner_id)
    }
}

fn candidate_ids(candidates: &[&GraphNode]) -> Vec<EntityId> {
    candidates.iter().map(|node| node.id().clone()).collect()
}

fn candidate_ids_of_kind(candidates: &[&GraphNode], expected: NodeKind) -> Vec<EntityId> {
    candidates
        .iter()
        .filter(|node| node.kind() == expected)
        .map(|node| node.id().clone())
        .collect()
}

fn source_processed_outcome(
    resolution: &EdtEventSubscriptionSourceResolution,
) -> EdtEventSubscriptionProcessedOutcome {
    let kind = match resolution.outcome() {
        EdtEventSubscriptionSourceResolutionOutcome::Resolved { .. } => {
            EdtEventSubscriptionProcessedOutcomeKind::Resolved
        }
        EdtEventSubscriptionSourceResolutionOutcome::Missing => {
            EdtEventSubscriptionProcessedOutcomeKind::Missing
        }
        EdtEventSubscriptionSourceResolutionOutcome::Ambiguous { .. } => {
            EdtEventSubscriptionProcessedOutcomeKind::Ambiguous
        }
        EdtEventSubscriptionSourceResolutionOutcome::IncompatibleKind { .. } => {
            EdtEventSubscriptionProcessedOutcomeKind::IncompatibleKind
        }
        EdtEventSubscriptionSourceResolutionOutcome::RejectedObservation { .. } => {
            EdtEventSubscriptionProcessedOutcomeKind::RejectedObservation
        }
    };

    EdtEventSubscriptionProcessedOutcome {
        observation: EdtEventSubscriptionProcessedObservation::Source {
            raw_selector: resolution.observation().raw_selector().to_owned(),
        },
        kind,
    }
}

fn handler_processed_outcome(
    resolution: &EdtEventSubscriptionHandlerResolution,
) -> EdtEventSubscriptionProcessedOutcome {
    let kind = match resolution.outcome() {
        EdtEventSubscriptionHandlerResolutionOutcome::Resolved { .. } => {
            EdtEventSubscriptionProcessedOutcomeKind::Resolved
        }
        EdtEventSubscriptionHandlerResolutionOutcome::MissingCommonModule
        | EdtEventSubscriptionHandlerResolutionOutcome::MissingModule { .. }
        | EdtEventSubscriptionHandlerResolutionOutcome::MissingSymbol { .. } => {
            EdtEventSubscriptionProcessedOutcomeKind::Missing
        }
        EdtEventSubscriptionHandlerResolutionOutcome::AmbiguousCommonModule { .. }
        | EdtEventSubscriptionHandlerResolutionOutcome::AmbiguousModule { .. }
        | EdtEventSubscriptionHandlerResolutionOutcome::AmbiguousSymbol { .. } => {
            EdtEventSubscriptionProcessedOutcomeKind::Ambiguous
        }
        EdtEventSubscriptionHandlerResolutionOutcome::IncompatibleCommonModuleKind { .. }
        | EdtEventSubscriptionHandlerResolutionOutcome::IncompatibleModuleKind { .. }
        | EdtEventSubscriptionHandlerResolutionOutcome::IncompatibleSymbolKind { .. } => {
            EdtEventSubscriptionProcessedOutcomeKind::IncompatibleKind
        }
        EdtEventSubscriptionHandlerResolutionOutcome::InvalidOwner { .. } => {
            EdtEventSubscriptionProcessedOutcomeKind::InvalidOwner
        }
    };

    EdtEventSubscriptionProcessedOutcome {
        observation: EdtEventSubscriptionProcessedObservation::Handler {
            raw_path: resolution.handler().raw_path().to_owned(),
        },
        kind,
    }
}

#[cfg(test)]
mod tests {
    use oneagent_common::{EntityId, EntityName};
    use oneagent_graph::{EdgeKind, GraphEdge, GraphNode, NodeKind, SemanticGraph};
    use oneagent_metadata::MetadataKind;
    use std::fs;
    use tempfile::tempdir;

    use super::{
        EdtEventSubscriptionHandlerResolutionOutcome, EdtEventSubscriptionProcessedObservation,
        EdtEventSubscriptionProcessedOutcomeKind, EdtEventSubscriptionResolutionIndex,
        EdtEventSubscriptionSourceResolutionOutcome,
    };
    use crate::{EdtEventSubscriptionReader, FileSystemEdtEventSubscriptionReader};

    fn id(value: &str) -> EntityId {
        EntityId::new(value).expect("identifier must be valid")
    }

    fn name(value: &str) -> EntityName {
        EntityName::new(value).expect("name must be valid")
    }

    fn node(id_value: &str, name_value: &str, kind: NodeKind) -> GraphNode {
        GraphNode::new(id(id_value), name(name_value), kind)
    }

    fn insert_contains(graph: &mut SemanticGraph, owner: &str, child: &str) {
        graph
            .insert_edge(GraphEdge::new(id(owner), id(child), EdgeKind::Contains))
            .expect("contains edge must be valid");
    }

    fn canonical_graph(reverse: bool) -> SemanticGraph {
        let nodes = [
            node(
                "catalog.products",
                "Products",
                NodeKind::Metadata(MetadataKind::Catalog),
            ),
            node(
                "catalog.services",
                "Services",
                NodeKind::Metadata(MetadataKind::Catalog),
            ),
            node(
                "document.sales",
                "Sales",
                NodeKind::Metadata(MetadataKind::Document),
            ),
            node(
                "common_module.events",
                "Events",
                NodeKind::Metadata(MetadataKind::CommonModule),
            ),
            node("common_module.events:module", "Events", NodeKind::Module),
            node(
                "common_module.events:module:procedure:BeforeWrite",
                "BeforeWrite",
                NodeKind::Procedure,
            ),
        ];
        let mut graph = SemanticGraph::new();
        if reverse {
            for node in nodes.into_iter().rev() {
                graph.insert_node(node);
            }
        } else {
            for node in nodes {
                graph.insert_node(node);
            }
        }
        insert_contains(
            &mut graph,
            "common_module.events",
            "common_module.events:module",
        );
        insert_contains(
            &mut graph,
            "common_module.events:module",
            "common_module.events:module:procedure:BeforeWrite",
        );
        graph
    }

    fn descriptor(sources: &[&str], handler: &str) -> crate::EdtEventSubscriptionDescriptor {
        let directory = tempdir().expect("temporary directory must be created");
        let descriptor_path = directory.path().join("Subscription.mdo");
        let mut source_values = String::new();
        for source in sources {
            source_values.push_str("<types>");
            source_values.push_str(source);
            source_values.push_str("</types>");
        }
        let xml = format!(
            concat!(
                "<?xml version=\"1.0\" encoding=\"UTF-8\"?>",
                "<mdclass:EventSubscription ",
                "xmlns:mdclass=\"http://g5.1c.ru/v8/dt/metadata/mdclass\" ",
                "uuid=\"00000000-0000-0000-0000-000000000001\">",
                "<name>Subscription</name>",
                "<source>{}</source>",
                "<event>BeforeWrite</event>",
                "<handler>{}</handler>",
                "</mdclass:EventSubscription>",
            ),
            source_values, handler,
        );
        fs::write(&descriptor_path, xml).expect("descriptor must be written");
        FileSystemEdtEventSubscriptionReader
            .read(directory.path())
            .expect("descriptor must parse")
    }

    fn source_outcome<'resolution>(
        resolution: &'resolution super::EdtEventSubscriptionResolution,
        raw_selector: &str,
    ) -> &'resolution EdtEventSubscriptionSourceResolutionOutcome {
        resolution
            .sources()
            .iter()
            .find(|source| source.observation().raw_selector() == raw_selector)
            .expect("source outcome must exist")
            .outcome()
    }

    #[test]
    fn resolves_exact_family_overlapping_and_rejected_sources_once() {
        let descriptor = descriptor(
            &[
                "CatalogObject.Products",
                "CatalogManager.Products",
                "CatalogObject",
                "ConstantValueManager.Flag",
                "Broken.Value.Extra",
                "CatalogObject.Products",
            ],
            "CommonModule.Events.BeforeWrite",
        );
        let graph = canonical_graph(false);
        let resolution = EdtEventSubscriptionResolutionIndex::new(&graph).resolve(&descriptor);

        assert_eq!(resolution.descriptor_id(), descriptor.id());
        assert_eq!(resolution.descriptor_path(), descriptor.descriptor_path());
        assert_eq!(resolution.sources().len(), 5);
        assert_eq!(
            source_outcome(&resolution, "CatalogObject.Products"),
            &EdtEventSubscriptionSourceResolutionOutcome::Resolved {
                target_ids: vec![id("catalog.products")],
            }
        );
        assert_eq!(
            source_outcome(&resolution, "CatalogManager.Products"),
            &EdtEventSubscriptionSourceResolutionOutcome::Resolved {
                target_ids: vec![id("catalog.products")],
            }
        );
        assert_eq!(
            source_outcome(&resolution, "CatalogObject"),
            &EdtEventSubscriptionSourceResolutionOutcome::Resolved {
                target_ids: vec![id("catalog.products"), id("catalog.services")],
            }
        );
        assert!(matches!(
            source_outcome(&resolution, "ConstantValueManager.Flag"),
            EdtEventSubscriptionSourceResolutionOutcome::RejectedObservation {
                kind: crate::EdtEventSubscriptionSourceOutcomeKind::Unsupported,
                reason: crate::EdtEventSubscriptionSourceReason::UnsupportedPrefix,
            }
        ));
        assert!(matches!(
            source_outcome(&resolution, "Broken.Value.Extra"),
            EdtEventSubscriptionSourceResolutionOutcome::RejectedObservation {
                kind: crate::EdtEventSubscriptionSourceOutcomeKind::Malformed,
                reason: crate::EdtEventSubscriptionSourceReason::AdditionalComponents,
            }
        ));
        assert_eq!(
            resolution.resolved_source_target_ids(),
            vec![id("catalog.products"), id("catalog.services")]
        );
        assert_eq!(resolution.processed_outcomes().len(), 6);
        assert_eq!(
            resolution.processed_outcomes()[0].observation(),
            &EdtEventSubscriptionProcessedObservation::Source {
                raw_selector: "Broken.Value.Extra".to_owned(),
            }
        );
        assert_eq!(
            resolution.processed_outcomes()[0].kind(),
            EdtEventSubscriptionProcessedOutcomeKind::RejectedObservation
        );
    }

    #[test]
    fn maps_all_supported_prefixes_to_only_the_accepted_kinds() {
        let cases = [
            ("CatalogObject", MetadataKind::Catalog),
            ("CatalogManager", MetadataKind::Catalog),
            ("DocumentObject", MetadataKind::Document),
            ("DocumentManager", MetadataKind::Document),
            (
                "InformationRegisterRecordSet",
                MetadataKind::InformationRegister,
            ),
            (
                "AccumulationRegisterRecordSet",
                MetadataKind::AccumulationRegister,
            ),
            (
                "AccountingRegisterRecordSet",
                MetadataKind::AccountingRegister,
            ),
            (
                "CalculationRegisterRecordSet",
                MetadataKind::CalculationRegister,
            ),
            ("BusinessProcessObject", MetadataKind::BusinessProcess),
            ("BusinessProcessManager", MetadataKind::BusinessProcess),
            ("TaskObject", MetadataKind::Task),
        ];

        for (prefix, kind) in cases {
            let descriptor = descriptor(&[prefix], "CommonModule.Events.BeforeWrite");
            let source = &descriptor.sources()[0];
            assert_eq!(source.target_kind(), Some(kind));
            assert_eq!(
                source.outcome(),
                crate::EdtEventSubscriptionSourceOutcomeKind::Supported
            );
        }

        let unsupported = descriptor(
            &["ChartOfAccountsObject.Accounts"],
            "CommonModule.Events.BeforeWrite",
        );
        assert_eq!(unsupported.sources()[0].target_kind(), None);
        assert_eq!(
            unsupported.sources()[0].outcome(),
            crate::EdtEventSubscriptionSourceOutcomeKind::Unsupported
        );
    }

    #[test]
    fn distinguishes_qualified_missing_incompatible_and_ambiguous_sources() {
        let descriptor = descriptor(
            &[
                "DocumentObject.Missing",
                "DocumentObject.Products",
                "DocumentObject.Sales",
            ],
            "CommonModule.Events.BeforeWrite",
        );
        let mut graph = canonical_graph(false);
        graph.insert_node(node(
            "document.sales.duplicate",
            "Sales",
            NodeKind::Metadata(MetadataKind::Document),
        ));
        let resolution = EdtEventSubscriptionResolutionIndex::new(&graph).resolve(&descriptor);

        assert_eq!(
            source_outcome(&resolution, "DocumentObject.Missing"),
            &EdtEventSubscriptionSourceResolutionOutcome::Missing
        );
        assert_eq!(
            source_outcome(&resolution, "DocumentObject.Products"),
            &EdtEventSubscriptionSourceResolutionOutcome::IncompatibleKind {
                candidates: vec![id("catalog.products")],
            }
        );
        assert_eq!(
            source_outcome(&resolution, "DocumentObject.Sales"),
            &EdtEventSubscriptionSourceResolutionOutcome::Ambiguous {
                candidates: vec![id("document.sales"), id("document.sales.duplicate")],
            }
        );
    }

    #[test]
    fn treats_an_empty_bare_family_as_missing() {
        let descriptor = descriptor(&["TaskObject"], "CommonModule.Events.BeforeWrite");
        let graph = canonical_graph(false);
        let resolution = EdtEventSubscriptionResolutionIndex::new(&graph).resolve(&descriptor);

        assert_eq!(
            source_outcome(&resolution, "TaskObject"),
            &EdtEventSubscriptionSourceResolutionOutcome::Missing
        );
    }

    #[test]
    fn resolves_owned_procedure_without_export_policy() {
        let descriptor = descriptor(
            &["CatalogObject.Products"],
            "CommonModule.Events.BeforeWrite",
        );
        let graph = canonical_graph(false);
        let resolution = EdtEventSubscriptionResolutionIndex::new(&graph).resolve(&descriptor);

        assert_eq!(
            resolution.handler().outcome(),
            &EdtEventSubscriptionHandlerResolutionOutcome::Resolved {
                target_id: id("common_module.events:module:procedure:BeforeWrite"),
            }
        );
        assert_eq!(resolution.processed_outcomes().len(), 2);
        assert_eq!(
            resolution.processed_outcomes()[1].observation(),
            &EdtEventSubscriptionProcessedObservation::Handler {
                raw_path: "CommonModule.Events.BeforeWrite".to_owned(),
            }
        );
    }

    #[test]
    fn distinguishes_handler_owner_and_module_failure_outcomes() {
        let valid_descriptor = descriptor(
            &["CatalogObject.Products"],
            "CommonModule.Events.BeforeWrite",
        );

        let missing_common_module = SemanticGraph::new();
        let result = EdtEventSubscriptionResolutionIndex::new(&missing_common_module)
            .resolve(&valid_descriptor);
        assert_eq!(
            result.handler().outcome(),
            &EdtEventSubscriptionHandlerResolutionOutcome::MissingCommonModule
        );

        let mut incompatible_common_module = SemanticGraph::new();
        incompatible_common_module.insert_node(node(
            "document.events",
            "Events",
            NodeKind::Metadata(MetadataKind::Document),
        ));
        let result = EdtEventSubscriptionResolutionIndex::new(&incompatible_common_module)
            .resolve(&valid_descriptor);
        assert_eq!(
            result.handler().outcome(),
            &EdtEventSubscriptionHandlerResolutionOutcome::IncompatibleCommonModuleKind {
                candidates: vec![id("document.events")],
            }
        );

        let mut ambiguous_common_module = canonical_graph(false);
        ambiguous_common_module.insert_node(node(
            "common_module.events.duplicate",
            "Events",
            NodeKind::Metadata(MetadataKind::CommonModule),
        ));
        let result = EdtEventSubscriptionResolutionIndex::new(&ambiguous_common_module)
            .resolve(&valid_descriptor);
        assert!(matches!(
            result.handler().outcome(),
            EdtEventSubscriptionHandlerResolutionOutcome::AmbiguousCommonModule { .. }
        ));

        let mut missing_module = SemanticGraph::new();
        missing_module.insert_node(node(
            "common_module.events",
            "Events",
            NodeKind::Metadata(MetadataKind::CommonModule),
        ));
        let result =
            EdtEventSubscriptionResolutionIndex::new(&missing_module).resolve(&valid_descriptor);
        assert!(matches!(
            result.handler().outcome(),
            EdtEventSubscriptionHandlerResolutionOutcome::MissingModule { .. }
        ));

        let mut incompatible_module = missing_module.clone();
        incompatible_module.insert_node(node(
            "common_module.events:child",
            "Events",
            NodeKind::Function,
        ));
        insert_contains(
            &mut incompatible_module,
            "common_module.events",
            "common_module.events:child",
        );
        let result = EdtEventSubscriptionResolutionIndex::new(&incompatible_module)
            .resolve(&valid_descriptor);
        assert!(matches!(
            result.handler().outcome(),
            EdtEventSubscriptionHandlerResolutionOutcome::IncompatibleModuleKind { .. }
        ));

        let mut ambiguous_module = canonical_graph(false);
        ambiguous_module.insert_node(node(
            "common_module.events:module:duplicate",
            "Events",
            NodeKind::Module,
        ));
        insert_contains(
            &mut ambiguous_module,
            "common_module.events",
            "common_module.events:module:duplicate",
        );
        let result =
            EdtEventSubscriptionResolutionIndex::new(&ambiguous_module).resolve(&valid_descriptor);
        assert!(matches!(
            result.handler().outcome(),
            EdtEventSubscriptionHandlerResolutionOutcome::AmbiguousModule { .. }
        ));
    }

    #[test]
    fn distinguishes_handler_symbol_failure_outcomes() {
        let valid_descriptor = descriptor(
            &["CatalogObject.Products"],
            "CommonModule.Events.BeforeWrite",
        );

        let mut missing_symbol = canonical_graph(false);
        let missing_descriptor =
            descriptor(&["CatalogObject.Products"], "CommonModule.Events.Missing");
        let result =
            EdtEventSubscriptionResolutionIndex::new(&missing_symbol).resolve(&missing_descriptor);
        assert!(matches!(
            result.handler().outcome(),
            EdtEventSubscriptionHandlerResolutionOutcome::MissingSymbol { .. }
        ));

        missing_symbol.insert_node(node(
            "common_module.events:module:function:WrongKind",
            "WrongKind",
            NodeKind::Function,
        ));
        insert_contains(
            &mut missing_symbol,
            "common_module.events:module",
            "common_module.events:module:function:WrongKind",
        );
        let wrong_kind_descriptor =
            descriptor(&["CatalogObject.Products"], "CommonModule.Events.WrongKind");
        let result = EdtEventSubscriptionResolutionIndex::new(&missing_symbol)
            .resolve(&wrong_kind_descriptor);
        assert!(matches!(
            result.handler().outcome(),
            EdtEventSubscriptionHandlerResolutionOutcome::IncompatibleSymbolKind { .. }
        ));

        let mut invalid_owner = canonical_graph(false);
        invalid_owner.insert_node(node("other.module", "Other", NodeKind::Module));
        invalid_owner.insert_node(node(
            "other.module:procedure:Elsewhere",
            "Elsewhere",
            NodeKind::Procedure,
        ));
        insert_contains(
            &mut invalid_owner,
            "other.module",
            "other.module:procedure:Elsewhere",
        );
        let invalid_owner_descriptor =
            descriptor(&["CatalogObject.Products"], "CommonModule.Events.Elsewhere");
        let result = EdtEventSubscriptionResolutionIndex::new(&invalid_owner)
            .resolve(&invalid_owner_descriptor);
        assert_eq!(
            result.handler().outcome(),
            &EdtEventSubscriptionHandlerResolutionOutcome::InvalidOwner {
                module_id: id("common_module.events:module"),
                candidates: vec![id("other.module:procedure:Elsewhere")],
            }
        );

        let mut ambiguous_symbol = canonical_graph(false);
        ambiguous_symbol.insert_node(node(
            "common_module.events:module:procedure:BeforeWriteDuplicate",
            "BeforeWrite",
            NodeKind::Procedure,
        ));
        insert_contains(
            &mut ambiguous_symbol,
            "common_module.events:module",
            "common_module.events:module:procedure:BeforeWriteDuplicate",
        );
        let result =
            EdtEventSubscriptionResolutionIndex::new(&ambiguous_symbol).resolve(&valid_descriptor);
        assert!(matches!(
            result.handler().outcome(),
            EdtEventSubscriptionHandlerResolutionOutcome::AmbiguousSymbol { .. }
        ));
    }

    #[test]
    fn malformed_handler_remains_a_typed_fatal_parser_outcome() {
        let directory = tempdir().expect("temporary directory must be created");
        let descriptor_path = directory.path().join("Subscription.mdo");
        let xml = concat!(
            "<mdclass:EventSubscription ",
            "xmlns:mdclass=\"http://g5.1c.ru/v8/dt/metadata/mdclass\" ",
            "uuid=\"00000000-0000-0000-0000-000000000001\">",
            "<name>Subscription</name>",
            "<source><types>CatalogObject.Products</types></source>",
            "<event>BeforeWrite</event>",
            "<handler>CommonModule.Events</handler>",
            "</mdclass:EventSubscription>",
        );
        fs::write(&descriptor_path, xml).expect("descriptor must be written");

        let error = FileSystemEdtEventSubscriptionReader
            .read(directory.path())
            .expect_err("malformed handler must remain fatal");
        assert!(matches!(
            error,
            crate::EdtEventSubscriptionError::InvalidHandler {
                reason: crate::EdtEventSubscriptionHandlerReason::MissingComponents,
                ..
            }
        ));
    }

    #[test]
    fn reordered_graphs_and_repeated_resolution_are_equal_and_read_only() {
        let descriptor = descriptor(
            &[
                "CatalogObject",
                "CatalogManager.Products",
                "DocumentObject.Sales",
            ],
            "CommonModule.Events.BeforeWrite",
        );
        let first_graph = canonical_graph(false);
        let second_graph = canonical_graph(true);
        let first_counts = (first_graph.node_count(), first_graph.edge_count());
        let second_counts = (second_graph.node_count(), second_graph.edge_count());

        let first_index = EdtEventSubscriptionResolutionIndex::new(&first_graph);
        let first = first_index.resolve(&descriptor);
        let repeated = first_index.resolve(&descriptor);
        let reordered =
            EdtEventSubscriptionResolutionIndex::new(&second_graph).resolve(&descriptor);

        assert_eq!(first, repeated);
        assert_eq!(first, reordered);
        assert_eq!(
            (first_graph.node_count(), first_graph.edge_count()),
            first_counts
        );
        assert_eq!(
            (second_graph.node_count(), second_graph.edge_count()),
            second_counts
        );
    }
}
