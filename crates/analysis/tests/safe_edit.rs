use oneagent_analysis::diagnostics::{DiagnosticEngine, DiagnosticPolicy, DiagnosticReport};
use oneagent_analysis::refactoring::*;
use oneagent_analysis::rules::*;
use oneagent_analysis::safe_edit::*;
use oneagent_analysis::safe_edit::{
    MAX_SAFE_EDIT_BUFFER_BYTES, SafeEditCountingSink, SafeEditError, SafeEditProjectionAdmission,
};
use oneagent_bsl::{BslQuery, BslQueryExtractor, LineBslQueryExtractor, bsl_query_id};
use oneagent_bsl::{BslSymbolKind, bsl_callable_id};
use oneagent_common::{EntityId, EntityName, SourcePath};
use oneagent_graph::*;
use std::fmt::Write;
use std::path::PathBuf;
use std::sync::Arc;

fn id(value: &str) -> EntityId {
    EntityId::new(value).unwrap()
}
fn callable(name: &str) -> EntityId {
    bsl_callable_id(&id("module.main"), BslSymbolKind::Procedure, name).unwrap()
}

#[derive(Clone)]
struct Evidence {
    queries: Vec<BslQuery>,
    root: PathBuf,
    graph: Arc<SemanticGraph>,
    sources: SourceEvidenceSet,
    diagnostics: Arc<[SemanticDiagnostic]>,
    references: Arc<SemanticReferenceRequestLedger>,
    report: SemanticGraphReport,
    validation: SemanticGraphValidationResult,
    rules: RuleExecutionReport,
    findings: DiagnosticReport,
}
impl Evidence {
    fn view(&self) -> SafeEditEvidence<'_> {
        SafeEditEvidence {
            root: &self.root,
            graph: &self.graph,
            sources: &self.sources,
            diagnostics: &self.diagnostics,
            references: &self.references,
            reference_statistics: SemanticReferenceStatistics::new(),
            report: &self.report,
            validation: &self.validation,
            rules: &self.rules,
            findings: &self.findings,
        }
    }
}

// Derived from the tracked refactoring_plan.rs grammar and canonical constructors.
#[allow(clippy::too_many_lines)]
fn evidence(name: &str) -> Evidence {
    let raw =
        format!("Procedure {name}()\nQuery = New Query(\"SELECT Ref FROM Catalog.Missing\");\nEndProcedure\nProcedure Caller()\n{name}();\nQuery = New Query(\"SELECT Ref FROM Catalog.Missing\");\nEndProcedure\n")
            .into_bytes();
    let version = SourceContentVersion::from_bytes(&raw);
    let document = SourceDocumentId::new(id("configuration.main"), id("module.main")).unwrap();
    let mut occurrences: Vec<_> = String::from_utf8(raw.clone())
        .unwrap()
        .match_indices(name)
        .enumerate()
        .map(|(index, (start, _))| {
            SourceOccurrence::new(
                document.clone(),
                version,
                SourceByteRange::new(start, start + name.len()).unwrap(),
                if index == 0 {
                    SourceOccurrenceKind::Declaration
                } else {
                    SourceOccurrenceKind::LocalCall
                },
                name,
                Some(callable(name)),
                SourceOccurrenceResolution::Unique,
            )
            .unwrap()
        })
        .collect();
    let start = std::str::from_utf8(&raw).unwrap().find("Caller").unwrap();
    occurrences.push(
        SourceOccurrence::new(
            document.clone(),
            version,
            SourceByteRange::new(start, start + "Caller".len()).unwrap(),
            SourceOccurrenceKind::Declaration,
            "Caller",
            Some(callable("Caller")),
            SourceOccurrenceResolution::Unique,
        )
        .unwrap(),
    );
    let source = SourceDocument::new(
        document,
        SourceFormat::Edt,
        BslModuleRole::Common,
        ConfinedSourcePath::new(
            SourcePath::new("configuration/Main.bsl").unwrap(),
            &SourcePath::new("configuration").unwrap(),
        )
        .unwrap(),
        raw,
        occurrences,
        SourceEvidenceCompleteness::BslCallableRenameV1,
    )
    .unwrap();
    let sources = SourceEvidenceSet::new(id("configuration.main"), vec![source]).unwrap();
    let queries = LineBslQueryExtractor
        .extract_queries(
            &id("module.main"),
            std::str::from_utf8(sources.documents()[0].raw_content()).unwrap(),
        )
        .unwrap();
    assert_eq!(queries.len(), 2);
    let mut graph = SemanticGraph::new();
    for (identity, name, kind) in [
        (
            id("configuration.main"),
            "Configuration",
            NodeKind::Metadata(oneagent_metadata::MetadataKind::Configuration),
        ),
        (id("module.main"), "Main", NodeKind::Module),
        (callable(name), name, NodeKind::Procedure),
        (callable("Caller"), "Caller", NodeKind::Procedure),
    ] {
        graph.insert_node(GraphNode::new_with_provenance(
            identity,
            EntityName::new(name).unwrap(),
            kind,
            vec![Provenance::new(
                None,
                ProducerId::new("fixture"),
                FactOrigin::Declared,
                Confidence::Exact,
                ResolutionState::NotApplicable,
            )],
        ));
    }
    for (source, target, kind) in [
        (
            id("configuration.main"),
            id("module.main"),
            EdgeKind::Contains,
        ),
        (id("module.main"), callable(name), EdgeKind::Contains),
        (id("module.main"), callable("Caller"), EdgeKind::Contains),
        (callable("Caller"), callable(name), EdgeKind::Calls),
    ] {
        graph
            .insert_edge(GraphEdge::new_with_provenance(
                source,
                target,
                kind,
                vec![Provenance::new(
                    None,
                    ProducerId::new("fixture"),
                    FactOrigin::Declared,
                    Confidence::Exact,
                    ResolutionState::NotApplicable,
                )],
            ))
            .unwrap();
    }
    for query in &queries {
        let provenance = vec![Provenance::new(
            None,
            ProducerId::new("fixture.query"),
            FactOrigin::Parsed,
            Confidence::Exact,
            ResolutionState::NotApplicable,
        )];
        graph.insert_node(GraphNode::new_with_provenance(
            query.id().clone(),
            query.binding_name().clone(),
            NodeKind::Query,
            provenance.clone(),
        ));
        graph
            .insert_edge(GraphEdge::new_with_provenance(
                query.owner_id().clone(),
                query.id().clone(),
                EdgeKind::Contains,
                provenance,
            ))
            .unwrap();
    }
    let reference = SemanticReference::Name(EntityName::new("Missing").unwrap());
    let collected = Provenance::new(
        Some(id("source")),
        ProducerId::new("collector"),
        FactOrigin::Parsed,
        Confidence::Exact,
        ResolutionState::Unresolved,
    );
    let resolved = Provenance::new(
        Some(id("source")),
        ProducerId::new("resolver"),
        FactOrigin::Resolved,
        Confidence::Exact,
        ResolutionState::Unresolved,
    );
    let request = SemanticReferenceRequest::collected(
        id("module.main"),
        SemanticReferenceCategory::MetadataType,
        reference.clone(),
        [NodeKind::Module],
        [collected],
    )
    .unwrap()
    .into_missing_target([resolved.clone()])
    .unwrap();
    let references = Arc::new(SemanticReferenceRequestLedger::from_requests([request]).unwrap());
    let diagnostics: Arc<[SemanticDiagnostic]> = Arc::from([SemanticDiagnostic::new(
        SemanticDiagnosticCode::ReferenceUnresolved,
        SemanticDiagnosticSeverity::Error,
        SemanticDiagnosticKind::UnresolvedTarget,
        "pre-existing finding",
        reference,
    )
    .with_source_node(id("module.main"))
    .with_expected_kinds(vec![NodeKind::Module])
    .with_provenance(vec![resolved])]);
    let validation = SemanticGraphValidator::new().validate(&graph);
    assert!(validation.is_valid(), "{validation:?}");
    let findings = DiagnosticEngine
        .build(&diagnostics, &validation, &DiagnosticPolicy::default())
        .unwrap();
    let registry = RuleRegistry::<Arc<dyn Rule>>::new([]).unwrap();
    let configuration = RuleConfiguration::default();
    let rules = RuleEngine
        .execute(
            &registry,
            &RulePlan::new(&registry, &configuration).unwrap(),
            &configuration,
            &RuleContext::new(&graph, &validation, &findings),
            &NeverCancelled,
        )
        .unwrap();
    Evidence {
        queries,
        root: PathBuf::from(env!("CARGO_MANIFEST_DIR")),
        report: SemanticGraphReport::from_graph_diagnostics_and_reference_requests(
            &graph,
            &diagnostics,
            &references,
        ),
        graph: Arc::new(graph),
        sources,
        diagnostics,
        references,
        validation,
        rules,
        findings,
    }
}

fn plan(before: &Evidence) -> RefactoringPlan {
    let doc = &before.sources.documents()[0];
    let request = RefactoringRequest::new(
        RefactoringFamily::BslCallableRenameV1,
        WorkspacePublicationId::initial(),
        id("configuration.main"),
        callable("OldName"),
        "NewName",
    )
    .unwrap();
    let target = RefactoringTarget::new(
        id("configuration.main"),
        callable("OldName"),
        NodeKind::Procedure,
        id("module.main"),
        doc.occurrences()[0].clone(),
        "NewName",
    )
    .unwrap();
    let preconditions = RefactoringPreconditionSet::new(
        WorkspacePublicationId::initial(),
        id("configuration.main"),
        callable("OldName"),
        NodeKind::Procedure,
        id("module.main"),
        vec![RefactoringSourcePrecondition::new(
            doc.id().clone(),
            doc.content_version(),
        )],
    )
    .unwrap();
    let operations = doc
        .occurrences()
        .iter()
        .filter(|occurrence| occurrence.mapped_target_id() == Some(&callable("OldName")))
        .map(|occurrence| {
            RefactoringOperation::new(
                if occurrence.kind() == SourceOccurrenceKind::Declaration {
                    RefactoringOperationKind::ReplaceDeclarationIdentifier
                } else {
                    RefactoringOperationKind::ReplaceDirectCallIdentifier
                },
                occurrence.kind(),
                doc.id().clone(),
                doc.content_version(),
                occurrence.range(),
                occurrence.token(),
                "NewName",
                &[],
            )
            .unwrap()
        })
        .collect();
    RefactoringPlan::new(request, target, preconditions, operations).unwrap()
}

fn projection(
    before: &Evidence,
    plan: &RefactoringPlan,
    after: &Evidence,
) -> SafeEditProducerProjection {
    projection_with(before, plan, after, |_, _| {}).unwrap()
}

fn projection_with(
    before: &Evidence,
    plan: &RefactoringPlan,
    after: &Evidence,
    mutate_facts: impl FnOnce(&mut Vec<SafeEditProjectionFact>, &mut SafeEditProjectionAdmission),
) -> Result<SafeEditProducerProjection, SafeEditError> {
    let doc = &before.sources.documents()[0];
    let path = SourcePath::new(
        before
            .root
            .join(doc.path().path().as_str())
            .to_str()
            .unwrap(),
    )
    .unwrap();
    let owner = id("configuration.main");
    let name = EntityName::new("Main").unwrap();
    let modules = [SafeEditModuleInput::new(
        doc,
        &owner,
        &name,
        &path,
        after.sources.documents()[0].raw_content(),
    )
    .unwrap()];
    let input = SafeEditProducerInput::new(
        WorkspacePublicationId::initial(),
        &before.root,
        &before.root,
        &before.graph,
        &before.sources,
        &before.diagnostics,
        &before.references,
        plan,
        &modules,
    )
    .unwrap();
    let mut admission = SafeEditProjectionAdmission::new(0).unwrap();
    let mut facts = input.unchanged_facts(&mut admission).unwrap();
    for query in &before.queries {
        let owner = if query.owner_id() == plan.target().target_node_id() {
            plan.target().expected_post_rename_node_id()
        } else {
            query.owner_id()
        };
        let expected = BslQuery::new(
            bsl_query_id(owner, query.binding_name().as_str()).unwrap(),
            owner.clone(),
            if owner == query.owner_id() {
                query.owner_name().clone()
            } else {
                EntityName::new(plan.operations()[0].replacement()).unwrap()
            },
            query.binding_name().clone(),
            query.text().to_owned(),
            query.line(),
        );
        if owner != query.owner_id() {
            facts.push(
                SafeEditProjectionFact::new(
                    SafeEditFactKey::NodeIdentity {
                        before: admission.copy_id(query.id()).unwrap(),
                    },
                    SafeEditFactValue::Identity(admission.copy_id(expected.id()).unwrap()),
                )
                .unwrap(),
            );
        }
        facts.push(
            SafeEditProjectionFact::new(
                SafeEditFactKey::QueryFact {
                    before: admission.copy_id(query.id()).unwrap(),
                },
                SafeEditFactValue::Query(
                    SafeEditQueryFact::new(query, &expected, &mut admission).unwrap(),
                ),
            )
            .unwrap(),
        );
    }
    mutate_facts(&mut facts, &mut admission);
    SafeEditProducerProjection::new(input, facts, &mut admission)
}

fn reject(mutator: impl FnOnce(&mut Evidence)) {
    let before = evidence("OldName");
    let plan = plan(&before);
    let original = evidence("NewName");
    validate_postconditions(
        &plan,
        &before.view(),
        &original.view(),
        &projection(&before, &plan, &original),
    )
    .unwrap();
    let expected = projection(&before, &plan, &original);
    let mut candidate = original;
    mutator(&mut candidate);
    assert!(validate_postconditions(&plan, &before.view(), &candidate.view(), &expected).is_err());
}

#[test]
fn inventory_and_untouched_evidence_mismatch_rejects() {
    reject(|value| value.root.push("different"));
    reject(|value| {
        value.sources = SourceEvidenceSet::new(id("configuration.main"), vec![]).unwrap();
    });
    for kind in ["format", "role", "path", "identity", "extra", "bytes"] {
        reject(|value| {
            let old = &value.sources.documents()[0];
            let identity = if kind == "identity" || kind == "extra" {
                SourceDocumentId::new(id("configuration.main"), id("module.extra")).unwrap()
            } else {
                old.id().clone()
            };
            let mut bytes = old.raw_content().to_vec();
            if kind == "bytes" {
                bytes.extend_from_slice(b"// unrelated\n");
            }
            let version = SourceContentVersion::from_bytes(&bytes);
            let occurrences = old
                .occurrences()
                .iter()
                .map(|o| {
                    SourceOccurrence::new(
                        identity.clone(),
                        version,
                        o.range(),
                        o.kind(),
                        o.token(),
                        o.mapped_target_id().cloned(),
                        o.resolution(),
                    )
                    .unwrap()
                })
                .collect();
            let changed = SourceDocument::new(
                identity,
                if kind == "format" {
                    SourceFormat::DesignerXml
                } else {
                    old.format()
                },
                if kind == "role" {
                    BslModuleRole::Object
                } else {
                    old.module_role()
                },
                if kind == "path" || kind == "extra" {
                    ConfinedSourcePath::new(
                        SourcePath::new("configuration/Extra.bsl").unwrap(),
                        &SourcePath::new("configuration").unwrap(),
                    )
                    .unwrap()
                } else {
                    old.path().clone()
                },
                bytes,
                occurrences,
                old.completeness(),
            )
            .unwrap();
            let documents = if kind == "extra" {
                vec![old.clone(), changed]
            } else {
                vec![changed]
            };
            value.sources = SourceEvidenceSet::new(id("configuration.main"), documents).unwrap();
        });
    }
}

#[test]
#[allow(clippy::too_many_lines)]
fn node_and_edge_projection_rejects_unrelated_changes() {
    for kind in [
        "missing_query",
        "extra_query",
        "query_binding",
        "other_callable_query",
        "query_owner",
    ] {
        reject(|value| {
            let selected = bsl_query_id(
                &callable(if kind == "other_callable_query" {
                    "Caller"
                } else {
                    "NewName"
                }),
                "Query",
            )
            .unwrap();
            let mut graph = SemanticGraph::new();
            for node in value.graph.nodes() {
                if node.id() == &selected && kind == "missing_query" {
                    continue;
                }
                let changed = if node.id() == &selected
                    && matches!(kind, "query_binding" | "other_callable_query")
                {
                    GraphNode::new_with_provenance(
                        node.id().clone(),
                        EntityName::new("OtherBinding").unwrap(),
                        node.kind(),
                        node.provenance().to_vec(),
                    )
                } else {
                    node.clone()
                };
                graph.insert_node(changed);
            }
            if kind == "extra_query" {
                graph.insert_node(GraphNode::new(
                    id("extra.query"),
                    EntityName::new("Extra").unwrap(),
                    NodeKind::Query,
                ));
            }
            for edge in value.graph.edges() {
                if graph.node(edge.target()).is_none() {
                    continue;
                }
                graph
                    .insert_edge(if kind == "query_owner" && edge.target() == &selected {
                        GraphEdge::new_with_provenance(
                            callable("Caller"),
                            selected.clone(),
                            edge.kind(),
                            edge.provenance().to_vec(),
                        )
                    } else {
                        edge.clone()
                    })
                    .unwrap();
            }
            value.graph = Arc::new(graph);
        });
    }
    for kind in ["name", "kind", "provenance", "edge"] {
        reject(|value| {
            let mut graph = SemanticGraph::new();
            for node in value.graph.nodes() {
                let selected = node.id() == &callable("NewName");
                let changed = match kind {
                    "name" if selected => GraphNode::new(
                        node.id().clone(),
                        EntityName::new("Other").unwrap(),
                        node.kind(),
                    ),
                    "kind" if selected => {
                        GraphNode::new(node.id().clone(), node.name().clone(), NodeKind::Function)
                    }
                    "provenance" if selected => GraphNode::new_with_provenance(
                        node.id().clone(),
                        node.name().clone(),
                        node.kind(),
                        vec![Provenance::new(
                            None,
                            ProducerId::new("mutant"),
                            FactOrigin::Declared,
                            Confidence::Exact,
                            ResolutionState::NotApplicable,
                        )],
                    ),
                    _ => node.clone(),
                };
                graph.insert_node(changed);
            }
            for edge in value.graph.edges() {
                if kind != "edge" || edge.kind() != EdgeKind::Calls {
                    graph.insert_edge(edge.clone()).unwrap();
                }
            }
            value.graph = Arc::new(graph);
        });
    }
    for field in [
        "source",
        "location",
        "producer",
        "origin",
        "confidence",
        "resolution",
    ] {
        reject(|value| {
            let mut graph = SemanticGraph::new();
            for node in value.graph.nodes() {
                let mut provenance = node.provenance().to_vec();
                if node.id() == &callable("Caller") {
                    let p = &provenance[0];
                    provenance[0] = Provenance::new_with_location(
                        if field == "source" {
                            Some(id("different.source"))
                        } else {
                            p.source().cloned()
                        },
                        if field == "location" {
                            Some(oneagent_common::SourceLocation::new(
                                SourcePath::new("configuration/Other.bsl").unwrap(),
                                None,
                            ))
                        } else {
                            p.location().cloned()
                        },
                        if field == "producer" {
                            ProducerId::new("different")
                        } else {
                            p.producer().clone()
                        },
                        if field == "origin" {
                            FactOrigin::External
                        } else {
                            p.origin()
                        },
                        if field == "confidence" {
                            Confidence::Unknown
                        } else {
                            p.confidence()
                        },
                        if field == "resolution" {
                            ResolutionState::Ambiguous
                        } else {
                            p.resolution()
                        },
                    );
                }
                graph.insert_node(
                    GraphNode::new_with_payload_and_provenance(
                        node.id().clone(),
                        node.name().clone(),
                        node.kind(),
                        node.payload().clone(),
                        provenance,
                    )
                    .unwrap(),
                );
            }
            for edge in value.graph.edges() {
                graph.insert_edge(edge.clone()).unwrap();
            }
            value.graph = Arc::new(graph);
        });
    }
}

#[test]
fn occurrence_projection_rejects_omission_and_ambiguity() {
    reject(|value| {
        let old = &value.sources.documents()[0];
        let source = SourceDocument::new(
            old.id().clone(),
            old.format(),
            old.module_role(),
            old.path().clone(),
            old.raw_content().to_vec(),
            vec![old.occurrences()[0].clone()],
            old.completeness(),
        )
        .unwrap();
        value.sources = SourceEvidenceSet::new(id("configuration.main"), vec![source]).unwrap();
    });
    for resolution in [
        SourceOccurrenceResolution::Ambiguous,
        SourceOccurrenceResolution::Unresolved,
        SourceOccurrenceResolution::Unsupported,
    ] {
        reject(|value| {
            let old = &value.sources.documents()[0];
            let mut occurrences = old.occurrences().to_vec();
            let o = &occurrences[0];
            occurrences[0] = SourceOccurrence::new(
                o.document_id().clone(),
                o.content_version(),
                o.range(),
                o.kind(),
                o.token(),
                None,
                resolution,
            )
            .unwrap();
            value.sources = SourceEvidenceSet::new(
                id("configuration.main"),
                vec![
                    SourceDocument::new(
                        old.id().clone(),
                        old.format(),
                        old.module_role(),
                        old.path().clone(),
                        old.raw_content().to_vec(),
                        occurrences,
                        old.completeness(),
                    )
                    .unwrap(),
                ],
            )
            .unwrap();
        });
    }
}

#[test]
fn replacement_rejects_invalid_ranges_versions_and_tokens() {
    let before = evidence("OldName");
    let plan = plan(&before);
    let document = &before.sources.documents()[0];
    let operations: Vec<_> = plan.operations().iter().collect();
    assert_eq!(
        replacement_bytes(document, &operations, 1_048_576).unwrap(),
        evidence("NewName").sources.documents()[0].raw_content()
    );
    let mut reversed = operations.clone();
    reversed.reverse();
    assert!(replacement_bytes(document, &reversed, 1_048_576).is_err());
    assert!(
        replacement_bytes(
            &evidence("NewName").sources.documents()[0],
            &operations,
            1_048_576
        )
        .is_err()
    );
    assert!(replacement_bytes(document, &operations, 1).is_err());
    for kind in ["token", "range", "overlap", "document"] {
        let old = operations[0];
        let changed = RefactoringOperation::new(
            old.kind(),
            old.occurrence_kind(),
            if kind == "document" {
                SourceDocumentId::new(id("configuration.main"), id("other")).unwrap()
            } else {
                old.document_id().clone()
            },
            old.content_version(),
            if kind == "range" {
                SourceByteRange::new(0, 7).unwrap()
            } else {
                old.range()
            },
            if kind == "token" {
                "BadName"
            } else {
                old.expected()
            },
            old.replacement(),
            old.dependencies(),
        )
        .unwrap();
        let mut values = vec![&changed];
        if kind == "overlap" {
            values.push(&changed);
        }
        assert!(
            replacement_bytes(document, &values, 1_048_576).is_err(),
            "{kind}"
        );
    }
    // SourceDocument rejects malformed UTF-8/token/range combinations before any
    // comparator value exists; this is constructor evidence, not a fake mutant.
    assert!(
        SourceDocument::new(
            document.id().clone(),
            document.format(),
            document.module_role(),
            document.path().clone(),
            vec![0xff],
            vec![],
            document.completeness()
        )
        .is_err()
    );
}

#[test]
fn anchor_and_reference_projection_rejects_loss() {
    reject(|value| value.references = Arc::new(SemanticReferenceRequestLedger::new()));
    for candidate in ["canonical.candidate.a", "canonical.candidate.b"] {
        reject(|value| {
            let old = &value.references.requests()[0];
            let request = SemanticReferenceRequest::collected(
                old.source_node().clone(),
                old.category(),
                old.reference().clone(),
                old.expected_kinds().iter().copied(),
                [Provenance::new(
                    None,
                    ProducerId::new("collection"),
                    FactOrigin::Parsed,
                    Confidence::Exact,
                    ResolutionState::Unresolved,
                )],
            )
            .unwrap()
            .into_resolved(
                id(candidate),
                NodeKind::Module,
                [Provenance::new(
                    None,
                    ProducerId::new("resolver"),
                    FactOrigin::Resolved,
                    Confidence::Exact,
                    ResolutionState::Resolved,
                )],
            )
            .unwrap();
            value.references =
                Arc::new(SemanticReferenceRequestLedger::from_requests([request]).unwrap());
        });
    }
    for mutate_reference in [true, false] {
        reject(|value| {
            let old = &value.references.requests()[0];
            let provenance = if mutate_reference {
                old.provenance().to_vec()
            } else {
                old.provenance()
                    .iter()
                    .map(|p| {
                        Provenance::new(
                            p.source().cloned(),
                            ProducerId::new("changed"),
                            p.origin(),
                            p.confidence(),
                            p.resolution(),
                        )
                    })
                    .collect()
            };
            let changed = SemanticReferenceRequest::reconstruct_terminal(
                old.source_node().clone(),
                old.category(),
                if mutate_reference {
                    SemanticReference::Name(EntityName::new("Different").unwrap())
                } else {
                    old.reference().clone()
                },
                old.expected_kinds().iter().copied(),
                old.candidates().iter().cloned(),
                old.state(),
                old.outcome(),
                provenance,
            )
            .unwrap();
            assert_ne!(old, &changed);
            value.references =
                Arc::new(SemanticReferenceRequestLedger::from_requests([changed]).unwrap());
        });
    }
}

#[test]
fn diagnostic_and_rule_projection_rejects_non_equivalence() {
    struct DifferentRule(RuleDefinition, bool);
    impl RuleRegistration for DifferentRule {
        fn definition(&self) -> &RuleDefinition {
            &self.0
        }
    }
    impl Rule for DifferentRule {
        fn evaluate(&self, _: &RuleContext<'_>, _: &dyn RuleCancellationSignal) -> RuleEvaluation {
            if self.1 {
                RuleEvaluation::Failed(RuleFailureCode::new("different.failure").unwrap())
            } else {
                RuleEvaluation::NotApplicable
            }
        }
    }
    reject(|value| {
        value.findings = DiagnosticEngine
            .build(
                &[SemanticDiagnostic::new(
                    SemanticDiagnosticCode::ReferenceUnresolved,
                    SemanticDiagnosticSeverity::Warning,
                    SemanticDiagnosticKind::UnresolvedTarget,
                    "changed finding",
                    SemanticReference::Raw("reference".into()),
                )],
                &value.validation,
                &DiagnosticPolicy::default(),
            )
            .unwrap();
    });
    for failed in [false, true] {
        reject(|value| {
            let rule: Arc<dyn Rule> = Arc::new(DifferentRule(
                RuleDefinition::new(RuleId::new("different.rule").unwrap(), []).unwrap(),
                failed,
            ));
            let registry = RuleRegistry::new([rule]).unwrap();
            let configuration = RuleConfiguration::default();
            value.rules = RuleEngine
                .execute(
                    &registry,
                    &RulePlan::new(&registry, &configuration).unwrap(),
                    &configuration,
                    &RuleContext::new(&value.graph, &value.validation, &value.findings),
                    &NeverCancelled,
                )
                .unwrap();
        });
    }
}

#[test]
#[allow(clippy::too_many_lines)]
fn producer_projection_completeness_and_bounds() {
    let mut admission = SafeEditProjectionAdmission::new(MAX_SAFE_EDIT_BUFFER_BYTES - 8).unwrap();
    let output = admission.vector::<u64>(1).unwrap();
    assert_eq!(admission.retained_bytes(), MAX_SAFE_EDIT_BUFFER_BYTES);
    assert!(matches!(
        admission.vector::<u8>(1),
        Err(SafeEditError::ProjectionBounds)
    ));
    drop(output);
    admission.release(8).unwrap();
    assert_eq!(admission.retained_bytes(), MAX_SAFE_EDIT_BUFFER_BYTES - 8);
    assert_eq!(admission.peak_bytes(), MAX_SAFE_EDIT_BUFFER_BYTES);
    assert!(SafeEditProjectionAdmission::new(MAX_SAFE_EDIT_BUFFER_BYTES + 1).is_err());
    assert!(SafeEditProjectionAdmission::vector_bytes::<u64>(usize::MAX).is_err());
    assert!(admission.reserve(usize::MAX).is_err());

    let mut count = SafeEditCountingSink::default();
    write!(&mut count, "Запрос:{}", 17).unwrap();
    let mut admission = SafeEditProjectionAdmission::new(0).unwrap();
    let output = admission
        .string(count.bytes(), |writer| write!(writer, "Запрос:{}", 17))
        .unwrap();
    assert_eq!(output, "Запрос:17");
    assert_eq!(admission.retained_bytes(), output.capacity());
    let retained = admission.retained_bytes();
    assert!(
        admission
            .string(1, |writer| writer.write_str("too long"))
            .is_err()
    );
    assert!(admission.string(2, |writer| writer.write_str("x")).is_err());
    assert_eq!(
        admission.retained_bytes(),
        retained,
        "partial emission releases its reservation"
    );
    drop(output);
    admission.release(count.bytes()).unwrap();
    assert_eq!(admission.retained_bytes(), 0);
    let before = evidence("OldName");
    let request = &before.references.requests()[0];
    let query = &before.queries[0];
    let provenance = before.graph.nodes().next().unwrap().provenance();
    let mut measured = SafeEditProjectionAdmission::new(0).unwrap();
    let copied = SafeEditProvenance::copy_all(provenance, &mut measured).unwrap();
    let initial = MAX_SAFE_EDIT_BUFFER_BYTES - measured.retained_bytes() + 1;
    drop(copied);
    let mut short = SafeEditProjectionAdmission::new(initial).unwrap();
    assert!(SafeEditProvenance::copy_all(provenance, &mut short).is_err());
    assert_eq!(short.retained_bytes(), initial);
    let query_bytes = query.owner_id().as_str().len() * 2
        + query.binding_name().as_str().len()
        + query.text().len();
    for remaining in [query_bytes, query_bytes - 1] {
        let initial = MAX_SAFE_EDIT_BUFFER_BYTES - remaining;
        let mut budget = SafeEditProjectionAdmission::new(initial).unwrap();
        let result = SafeEditQueryFact::new(query, query, &mut budget);
        assert_eq!(result.is_ok(), remaining == query_bytes);
        assert_eq!(
            budget.retained_bytes(),
            if result.is_ok() {
                MAX_SAFE_EDIT_BUFFER_BYTES
            } else {
                initial
            }
        );
    }
    for field in ["binding", "text", "line"] {
        let changed = BslQuery::new(
            query.id().clone(),
            query.owner_id().clone(),
            query.owner_name().clone(),
            if field == "binding" {
                EntityName::new("Other").unwrap()
            } else {
                query.binding_name().clone()
            },
            if field == "text" {
                "SELECT Changed".into()
            } else {
                query.text().into()
            },
            query.line() + usize::from(field == "line"),
        );
        let mut budget = SafeEditProjectionAdmission::new(0).unwrap();
        assert!(SafeEditQueryFact::new(query, &changed, &mut budget).is_err());
        assert_eq!(budget.retained_bytes(), 0);
    }
    let mut admission = SafeEditProjectionAdmission::new(0).unwrap();
    let copied = SafeEditRequest::copy(request, &mut admission).unwrap();
    assert!(copied.matches(request));
    assert!(!format!("{copied:?}").contains("Missing"));
    let bytes = admission.retained_bytes();
    drop(copied);
    admission.release(bytes).unwrap();
    let mut exact = SafeEditProjectionAdmission::new(MAX_SAFE_EDIT_BUFFER_BYTES - bytes).unwrap();
    assert!(
        SafeEditRequest::copy(request, &mut exact)
            .unwrap()
            .matches(request)
    );
    assert_eq!(exact.retained_bytes(), MAX_SAFE_EDIT_BUFFER_BYTES);
    let initial = MAX_SAFE_EDIT_BUFFER_BYTES - bytes + 1;
    let mut short = SafeEditProjectionAdmission::new(initial).unwrap();
    assert!(SafeEditRequest::copy(request, &mut short).is_err());
    assert_eq!(
        short.retained_bytes(),
        initial,
        "partial request copy releases every nested allocation"
    );
    let mut admission = SafeEditProjectionAdmission::new(0).unwrap();
    let diagnostic = SafeEditDiagnostic::copy(&before.diagnostics[0], &mut admission).unwrap();
    assert!(diagnostic.matches(&before.diagnostics[0]));
    assert!(!format!("{diagnostic:?}").contains("pre-existing"));
    let bytes = admission.retained_bytes();
    drop(diagnostic);
    let initial = MAX_SAFE_EDIT_BUFFER_BYTES - bytes + 1;
    let mut short = SafeEditProjectionAdmission::new(initial).unwrap();
    assert!(SafeEditDiagnostic::copy(&before.diagnostics[0], &mut short).is_err());
    assert_eq!(short.retained_bytes(), initial);
    let plan = plan(&before);
    let after = evidence("NewName");
    for kind in [
        "missing",
        "extra",
        "duplicate",
        "conflicting",
        "candidate_key",
    ] {
        assert!(
            projection_with(&before, &plan, &after, |facts, admission| {
                match kind {
                    "missing" => {
                        facts.pop().unwrap();
                    }
                    "extra" | "candidate_key" => {
                        let key = if kind == "extra" {
                            id("absent.node")
                        } else {
                            callable("NewName")
                        };
                        facts.push(
                            SafeEditProjectionFact::new(
                                SafeEditFactKey::NodeFact { before: key },
                                SafeEditFactValue::Provenance(SafeEditExpected::Unchanged),
                            )
                            .unwrap(),
                        );
                    }
                    "duplicate" => facts.push(
                        SafeEditProjectionFact::new(
                            SafeEditFactKey::NodeFact {
                                before: admission.copy_id(&id("module.main")).unwrap(),
                            },
                            SafeEditFactValue::Provenance(SafeEditExpected::Unchanged),
                        )
                        .unwrap(),
                    ),
                    "conflicting" => facts.push(
                        SafeEditProjectionFact::new(
                            SafeEditFactKey::NodeIdentity {
                                before: callable("OldName"),
                            },
                            SafeEditFactValue::Identity(id("conflicting.target")),
                        )
                        .unwrap(),
                    ),
                    _ => unreachable!(),
                }
            })
            .is_err(),
            "{kind}"
        );
    }
    let frozen = projection(&before, &plan, &after);
    validate_postconditions(&plan, &before.view(), &after.view(), &frozen).unwrap();
    assert!(
        validate_postconditions(&plan, &before.view(), &after.view(), &frozen).is_err(),
        "every mapping is consumed once"
    );
}
