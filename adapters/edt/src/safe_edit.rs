//! Pure captured-byte EDT producer projection for checked callable renames.

use crate::bsl_graph::{
    AnalyzedBslModule, CapturedQueryEvidence, EDT_BSL_GRAPH_PRODUCER, analyze_captured_module,
    captured_query_evidence, captured_unresolved_call_diagnostic, query_provenance_from_source,
    write_query_source_id,
};
use crate::query_source_resolution::QuerySourceResolutionIndex;
use oneagent_analysis::refactoring::{MAX_SOURCE_OCCURRENCES_PER_DOCUMENT, SourceFormat};
use oneagent_analysis::safe_edit::{
    SafeEditCountingSink, SafeEditDiagnostic, SafeEditError, SafeEditExpected, SafeEditFactKey,
    SafeEditFactValue, SafeEditProducerInput, SafeEditProducerProjection,
    SafeEditProjectionAdmission, SafeEditProjectionFact, SafeEditProvenance, SafeEditQueryFact,
    SafeEditRequest,
};
use oneagent_bsl::BslQuery;
use oneagent_common::EntityId;
use oneagent_graph::{EdgeKind, NodeKind, Provenance};
use std::path::Path;

/// Projects canonical expected facts from the complete captured before publication.
///
/// The input contains exact result bytes and no candidate or filesystem callback.
///
/// # Errors
/// Rejects incompatible producer evidence, changed Query semantics or exhausted admission.
pub fn project_safe_edit_provenance(
    input: SafeEditProducerInput<'_>,
    admission: &mut SafeEditProjectionAdmission,
) -> Result<SafeEditProducerProjection, SafeEditError> {
    let checkpoint = admission.retained_bytes();
    let result = project(input, admission);
    if result.is_err() {
        admission.release(admission.retained_bytes() - checkpoint)?;
    }
    result
}

fn project(
    input: SafeEditProducerInput<'_>,
    admission: &mut SafeEditProjectionAdmission,
) -> Result<SafeEditProducerProjection, SafeEditError> {
    let mut facts = input.unchanged_facts(admission)?;
    // Existing canonical resolver working storage remains scoped to this call.
    // No index record is retained in the transaction projection.
    let index = QuerySourceResolutionIndex::new(input.graph());
    let mut query_count = 0usize;
    for module in input.modules() {
        if module.document().format() != SourceFormat::Edt {
            return Err(SafeEditError::SemanticMismatch);
        }
        let before = analyze_captured_module(
            module.document().id().module_id(),
            module.name(),
            Path::new(module.path().as_str()),
            module.document().raw_content(),
            Some(MAX_SOURCE_OCCURRENCES_PER_DOCUMENT),
        )
        .map_err(|_| SafeEditError::SemanticMismatch)?;
        let after = analyze_captured_module(
            module.document().id().module_id(),
            module.name(),
            Path::new(module.path().as_str()),
            module.result(),
            Some(MAX_SOURCE_OCCURRENCES_PER_DOCUMENT),
        )
        .map_err(|_| SafeEditError::SemanticMismatch)?;
        if before.symbols().len() != after.symbols().len()
            || before.queries().len() != after.queries().len()
        {
            return Err(SafeEditError::SemanticMismatch);
        }
        for (old, new) in before.symbols().iter().zip(after.symbols()) {
            let expected = mapped_owner(&input, old.id());
            let expected_name = if old.id() == input.plan().target().target_node_id() {
                input.plan().request().desired_name()
            } else {
                old.name().as_str()
            };
            if new.id() != expected
                || new.name().as_str() != expected_name
                || new.kind() != old.kind()
                || new.line() != old.line()
                || new.is_exported() != old.is_exported()
            {
                return Err(SafeEditError::SemanticMismatch);
            }
        }
        query_count = query_count
            .checked_add(project_queries(
                &input, &mut facts, &before, &after, &index, admission,
            )?)
            .ok_or(SafeEditError::ProjectionBounds)?;
        for call in before.calls() {
            let old = captured_unresolved_call_diagnostic(&before, call);
            if input.diagnostics().contains(&old) {
                let mut matches = after
                    .calls()
                    .iter()
                    .filter(|candidate| candidate.id() == call.id());
                let new_call = matches.next().ok_or(SafeEditError::SemanticMismatch)?;
                if matches.next().is_some() {
                    return Err(SafeEditError::SemanticMismatch);
                }
                let expected = captured_unresolved_call_diagnostic(&after, new_call);
                map_diagnostic(&input, &mut facts, &old, &expected, admission)?;
            }
        }
    }
    if query_count
        != input
            .graph()
            .nodes()
            .filter(|node| node.kind() == NodeKind::Query)
            .count()
    {
        return Err(SafeEditError::SemanticMismatch);
    }
    SafeEditProducerProjection::new(input, facts, admission)
}

fn project_queries(
    input: &SafeEditProducerInput<'_>,
    facts: &mut Vec<SafeEditProjectionFact>,
    before: &AnalyzedBslModule,
    after: &AnalyzedBslModule,
    index: &QuerySourceResolutionIndex,
    admission: &mut SafeEditProjectionAdmission,
) -> Result<usize, SafeEditError> {
    let mut query_count = 0usize;
    for query in before.queries() {
        let mut matches = after.queries().iter().filter(|candidate| {
            candidate.owner_id() == mapped_owner(input, query.owner_id())
                && candidate.binding_name() == query.binding_name()
        });
        let expected = matches.next().ok_or(SafeEditError::SemanticMismatch)?;
        if matches.next().is_some() {
            return Err(SafeEditError::SemanticMismatch);
        }
        let query_fact = SafeEditQueryFact::new(query, expected, admission)?;
        push(
            facts,
            SafeEditProjectionFact::new(
                SafeEditFactKey::QueryFact {
                    before: admission.copy_id(query.id())?,
                },
                SafeEditFactValue::Query(query_fact),
            )?,
        )?;
        if query.owner_id() == input.plan().target().target_node_id() {
            push(
                facts,
                SafeEditProjectionFact::new(
                    SafeEditFactKey::NodeIdentity {
                        before: admission.copy_id(query.id())?,
                    },
                    SafeEditFactValue::Identity(admission.copy_id(expected.id())?),
                )?,
            )?;
        } else if query.id() != expected.id() {
            return Err(SafeEditError::SemanticMismatch);
        }
        let old_provenance = query_provenance(before, query, admission)?;
        let new_provenance = query_provenance(after, expected, admission)?;
        map_query_provenance(
            input,
            facts,
            query,
            &old_provenance,
            &new_provenance,
            admission,
        )?;
        let old_evidence = captured_query_evidence(before, query, input.graph(), index)
            .map_err(|_| SafeEditError::SemanticMismatch)?;
        let expected_evidence = captured_query_evidence(after, expected, input.graph(), index)
            .map_err(|_| SafeEditError::SemanticMismatch)?;
        map_nested_evidence(
            input,
            facts,
            &old_evidence,
            &expected_evidence,
            expected,
            admission,
        )?;
        release_provenance(old_provenance, admission)?;
        release_provenance(new_provenance, admission)?;
        query_count = query_count
            .checked_add(1)
            .ok_or(SafeEditError::ProjectionBounds)?;
    }
    Ok(query_count)
}

fn map_nested_evidence(
    input: &SafeEditProducerInput<'_>,
    facts: &mut [SafeEditProjectionFact],
    before: &CapturedQueryEvidence,
    expected: &CapturedQueryEvidence,
    query: &BslQuery,
    admission: &mut SafeEditProjectionAdmission,
) -> Result<(), SafeEditError> {
    if before.requests.len() != expected.requests.len()
        || before.edges.len() != expected.edges.len()
        || before.diagnostics.len() != expected.diagnostics.len()
    {
        return Err(SafeEditError::SemanticMismatch);
    }
    for old in before.requests.requests() {
        if !input.references().requests().contains(old) {
            return Err(SafeEditError::SemanticMismatch);
        }
        let mut matches = expected.requests.requests().iter().filter(|new| {
            new.source_node() == query.id()
                && new.category() == old.category()
                && new.reference() == old.reference()
                && new.expected_kinds() == old.expected_kinds()
        });
        let new = matches.next().ok_or(SafeEditError::SemanticMismatch)?;
        if matches.next().is_some()
            || new.candidates() != old.candidates()
            || new.state() != old.state()
            || new.outcome() != old.outcome()
        {
            return Err(SafeEditError::SemanticMismatch);
        }
        let fact = facts.iter_mut().find(|fact| matches!(fact.key(), SafeEditFactKey::RequestFact { before } if before.matches(old.id())))
            .ok_or(SafeEditError::SemanticMismatch)?;
        if new != old {
            fact.set_expected(SafeEditFactValue::Request(SafeEditExpected::Expected(
                SafeEditRequest::copy(new, admission)?,
            )))?;
        }
    }
    for ((source, target, kind), old) in &before.edges {
        let edge = input
            .graph()
            .edges()
            .find(|edge| edge.source() == source && edge.target() == target && edge.kind() == *kind)
            .ok_or(SafeEditError::SemanticMismatch)?;
        if edge.provenance() != old {
            return Err(SafeEditError::SemanticMismatch);
        }
        let new = expected
            .edges
            .iter()
            .find(|((new_source, new_target, new_kind), _)| {
                new_source == query.id() && new_target == target && new_kind == kind
            })
            .map(|(_, values)| values)
            .ok_or(SafeEditError::SemanticMismatch)?;
        let fact = facts.iter_mut().find(|fact| matches!(fact.key(), SafeEditFactKey::EdgeFact { before_source, kind: k, before_target }
            if before_source == source && before_target == target && k == kind)).ok_or(SafeEditError::SemanticMismatch)?;
        if new != old {
            fact.set_expected(SafeEditFactValue::Provenance(SafeEditExpected::Expected(
                SafeEditProvenance::copy_all(new, admission)?,
            )))?;
        }
    }
    for (old, new) in before.diagnostics.iter().zip(&expected.diagnostics) {
        if old.code() != new.code()
            || old.kind() != new.kind()
            || old.severity() != new.severity()
            || old.message() != new.message()
            || old.reference() != new.reference()
            || old.candidates() != new.candidates()
            || old.expected_kinds() != new.expected_kinds()
            || old.actual_kind() != new.actual_kind()
        {
            return Err(SafeEditError::SemanticMismatch);
        }
        map_diagnostic(input, facts, old, new, admission)?;
    }
    Ok(())
}

fn map_diagnostic(
    input: &SafeEditProducerInput<'_>,
    facts: &mut [SafeEditProjectionFact],
    old: &oneagent_graph::SemanticDiagnostic,
    new: &oneagent_graph::SemanticDiagnostic,
    admission: &mut SafeEditProjectionAdmission,
) -> Result<(), SafeEditError> {
    let ordinal = input
        .diagnostics()
        .iter()
        .position(|value| value == old)
        .ok_or(SafeEditError::SemanticMismatch)?;
    let fact = facts.iter_mut().find(|fact| matches!(fact.key(), SafeEditFactKey::DiagnosticFact { before_ordinal } if *before_ordinal == ordinal))
        .ok_or(SafeEditError::SemanticMismatch)?;
    if new != old {
        fact.set_expected(SafeEditFactValue::Diagnostic(SafeEditExpected::Expected(
            SafeEditDiagnostic::copy(new, admission)?,
        )))?;
    }
    Ok(())
}

fn mapped_owner<'a>(input: &'a SafeEditProducerInput<'_>, owner: &'a EntityId) -> &'a EntityId {
    if owner == input.plan().target().target_node_id() {
        input.plan().target().expected_post_rename_node_id()
    } else {
        owner
    }
}

fn push(
    facts: &mut Vec<SafeEditProjectionFact>,
    fact: SafeEditProjectionFact,
) -> Result<(), SafeEditError> {
    if facts.len() == facts.capacity() {
        return Err(SafeEditError::ProjectionBounds);
    }
    facts.push(fact);
    Ok(())
}

fn query_provenance(
    module: &AnalyzedBslModule,
    query: &BslQuery,
    admission: &mut SafeEditProjectionAdmission,
) -> Result<Provenance, SafeEditError> {
    let mut count = SafeEditCountingSink::default();
    write_query_source_id(&mut count, module, query)
        .map_err(|_| SafeEditError::ProjectionBounds)?;
    let source = admission.string(count.bytes(), |sink| {
        write_query_source_id(sink, module, query)
    })?;
    admission.reserve(
        EDT_BSL_GRAPH_PRODUCER
            .len()
            .checked_add(module.source_path().map_or(0, |path| path.as_str().len()))
            .ok_or(SafeEditError::ProjectionBounds)?,
    )?;
    query_provenance_from_source(
        module,
        query,
        EntityId::new(source).map_err(|_| SafeEditError::SemanticMismatch)?,
    )
    .map_err(|_| SafeEditError::SemanticMismatch)
}

fn provenance_bytes(value: &Provenance) -> Result<usize, SafeEditError> {
    value
        .source()
        .map_or(0, |id| id.as_str().len())
        .checked_add(value.producer().as_str().len())
        .and_then(|n| {
            n.checked_add(
                value
                    .location()
                    .map_or(0, |location| location.path().as_str().len()),
            )
        })
        .ok_or(SafeEditError::ProjectionBounds)
}

fn release_provenance(
    value: Provenance,
    admission: &mut SafeEditProjectionAdmission,
) -> Result<(), SafeEditError> {
    let bytes = provenance_bytes(&value)?;
    drop(value);
    admission.release(bytes)
}

fn map_query_provenance(
    input: &SafeEditProducerInput<'_>,
    facts: &mut [SafeEditProjectionFact],
    query: &BslQuery,
    before: &Provenance,
    expected: &Provenance,
    admission: &mut SafeEditProjectionAdmission,
) -> Result<(), SafeEditError> {
    let node = input
        .graph()
        .node(query.id())
        .ok_or(SafeEditError::SemanticMismatch)?;
    let edge = input
        .graph()
        .edges()
        .find(|edge| {
            edge.kind() == EdgeKind::Contains
                && edge.target() == query.id()
                && edge.source() == query.owner_id()
        })
        .ok_or(SafeEditError::SemanticMismatch)?;
    if node.provenance() != std::slice::from_ref(before)
        || edge.provenance() != std::slice::from_ref(before)
    {
        return Err(SafeEditError::SemanticMismatch);
    }
    for fact in facts {
        let matches = match fact.key() {
            SafeEditFactKey::NodeFact { before } => before == query.id(),
            SafeEditFactKey::EdgeFact {
                before_source,
                kind,
                before_target,
            } => {
                *kind == EdgeKind::Contains
                    && before_source == query.owner_id()
                    && before_target == query.id()
            }
            _ => false,
        };
        if matches && before != expected {
            let values = SafeEditProvenance::copy_all(std::slice::from_ref(expected), admission)?;
            fact.set_expected(SafeEditFactValue::Provenance(SafeEditExpected::Expected(
                values,
            )))?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[allow(clippy::too_many_lines)]
    fn edt_projection_preserves_nested_query_evidence() {
        #[allow(clippy::too_many_lines)]
        fn complete_producer_before_reproduction() {
            use crate::EdtSemanticGraphBuilder;
            use oneagent_analysis::refactoring::*;
            use oneagent_analysis::safe_edit::*;
            use oneagent_common::SourcePath;
            use oneagent_graph::*;
            use std::sync::Arc;
            let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../..")
                .canonicalize()
                .unwrap();
            let root = workspace.join("adapters/edt/tests/fixtures/reads_project");
            let built = crate::FileSystemEdtSemanticGraphBuilder
                .build_graph_with_source_evidence(&workspace, &root)
                .unwrap();
            let sources = built.source_evidence();
            let target = built
                .graph()
                .edges()
                .find(|edge| {
                    edge.kind() == EdgeKind::Contains
                        && built.graph().node(edge.target()).unwrap().kind() == NodeKind::Query
                })
                .unwrap()
                .source()
                .clone();
            let diagnostics: Arc<[SemanticDiagnostic]> =
                Arc::from(built.build().diagnostics().to_vec());
            let references = Arc::new(
                SemanticReferenceRequestLedger::from_requests(
                    built.build().reference_requests().iter().cloned(),
                )
                .unwrap(),
            );
            for desired in ["ДлинноеНовоеИмя", "Я"] {
                let request = RefactoringRequest::new(
                    RefactoringFamily::BslCallableRenameV1,
                    WorkspacePublicationId::initial(),
                    sources.configuration_id().clone(),
                    target.clone(),
                    desired,
                )
                .unwrap();
                let evaluated = RefactoringPlanner
                    .evaluate(
                        RefactoringPlannerInput::new(
                            WorkspacePublicationId::initial(),
                            sources.configuration_id(),
                            built.graph(),
                            sources,
                        ),
                        &request,
                        &NeverCancelledRefactoring,
                    )
                    .unwrap();
                let plan = evaluated.plan();
                let results: Vec<_> = sources
                    .documents()
                    .iter()
                    .map(|doc| {
                        let operations: Vec<_> = plan
                            .operations()
                            .iter()
                            .filter(|operation| operation.document_id() == doc.id())
                            .collect();
                        if operations.is_empty() {
                            doc.raw_content().to_vec()
                        } else {
                            replacement_bytes(doc, &operations, 1_048_576).unwrap()
                        }
                    })
                    .collect();
                let paths: Vec<_> = sources
                    .documents()
                    .iter()
                    .map(|doc| {
                        SourcePath::new(
                            workspace.join(doc.path().path().as_str()).to_str().unwrap(),
                        )
                        .unwrap()
                    })
                    .collect();
                for corrupt in [false, true] {
                    let mut graph = SemanticGraph::new();
                    let module = built
                        .graph()
                        .nodes()
                        .find(|node| node.kind() == NodeKind::Query)
                        .unwrap()
                        .id();
                    for node in built.graph().nodes() {
                        graph.insert_node(if corrupt && node.id() == module {
                            GraphNode::new_with_payload_and_provenance(
                                node.id().clone(),
                                node.name().clone(),
                                node.kind(),
                                node.payload().clone(),
                                vec![],
                            )
                            .unwrap()
                        } else {
                            node.clone()
                        });
                    }
                    for edge in built.graph().edges() {
                        graph.insert_edge(edge.clone()).unwrap();
                    }
                    let graph = Arc::new(graph);
                    let modules: Vec<_> = sources
                        .documents()
                        .iter()
                        .enumerate()
                        .map(|(index, doc)| {
                            let owner = graph
                                .edges()
                                .find(|edge| {
                                    edge.kind() == EdgeKind::Contains
                                        && edge.target() == doc.id().module_id()
                                })
                                .unwrap()
                                .source();
                            SafeEditModuleInput::new(
                                doc,
                                owner,
                                graph.node(doc.id().module_id()).unwrap().name(),
                                &paths[index],
                                &results[index],
                            )
                            .unwrap()
                        })
                        .collect();
                    let input = SafeEditProducerInput::new(
                        WorkspacePublicationId::initial(),
                        &workspace,
                        &root,
                        &graph,
                        sources,
                        &diagnostics,
                        &references,
                        plan,
                        &modules,
                    )
                    .unwrap();
                    let mut admission = SafeEditProjectionAdmission::new(0).unwrap();
                    let projected = project_safe_edit_provenance(input, &mut admission);
                    if corrupt {
                        assert!(matches!(projected, Err(SafeEditError::SemanticMismatch)));
                        assert_eq!(
                            admission.retained_bytes(),
                            0,
                            "failed reproduction releases all projection leases"
                        );
                    } else {
                        assert!(projected.is_ok());
                        assert!(admission.retained_bytes() > 0);
                    }
                }
            }
        }

        complete_producer_before_reproduction();
        let raw = include_str!(
            "../tests/fixtures/reads_project/src/Documents/QueryHost/ObjectModule.bsl"
        );
        let module = EntityId::new("module").unwrap();
        let name = oneagent_common::EntityName::new("Module").unwrap();
        let path = Path::new("src/Модуль/ObjectModule.bsl");
        let original = analyze_captured_module(&module, &name, path, raw.as_bytes(), None).unwrap();
        assert!(!original.queries().is_empty());
        let owner = original.queries()[0].owner_name().as_str();
        for replacement in ["ДлинноеНовоеИмя", "Я", owner] {
            let result = raw.replace(owner, replacement);
            let after =
                analyze_captured_module(&module, &name, path, result.as_bytes(), None).unwrap();
            assert_eq!(original.queries().len(), after.queries().len());
            for (before, query) in original.queries().iter().zip(after.queries()) {
                assert_eq!(before.text(), query.text());
                assert_eq!(before.binding_name(), query.binding_name());
                assert_eq!(before.line(), query.line());
                let mut count = SafeEditCountingSink::default();
                write_query_source_id(&mut count, &after, query).unwrap();
                let mut admission = SafeEditProjectionAdmission::new(0).unwrap();
                let source = admission
                    .string(count.bytes(), |out| {
                        write_query_source_id(out, &after, query)
                    })
                    .unwrap();
                assert_eq!(source.capacity(), admission.retained_bytes());
                let canonical =
                    query_provenance_from_source(&after, query, EntityId::new(source).unwrap())
                        .unwrap();
                let copied = SafeEditProvenance::copy(&canonical, &mut admission).unwrap();
                assert!(copied.matches(&canonical));
                let mut short = SafeEditProjectionAdmission::new(
                    oneagent_analysis::safe_edit::MAX_SAFE_EDIT_BUFFER_BYTES - count.bytes() + 1,
                )
                .unwrap();
                assert!(
                    short
                        .string(count.bytes(), |out| write_query_source_id(
                            out, &after, query
                        ))
                        .is_err()
                );
                assert!(!format!("{copied:?}").contains("bsl_query"));
            }
        }
    }
}
