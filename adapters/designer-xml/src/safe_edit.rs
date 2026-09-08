//! Pure canonical Designer provenance projection for checked callable renames.

use crate::DesignerXmlModuleKind;
use crate::semantic_graph::{
    BSL_PRODUCER, GRAPH_PRODUCER, declaration_provenance_from_source,
    module_provenance_from_source, write_declaration_source_id, write_module_source_id,
};
use oneagent_analysis::refactoring::{
    BslModuleRole, MAX_SOURCE_OCCURRENCES_PER_DOCUMENT, SourceFormat,
};
use oneagent_analysis::safe_edit::{
    SafeEditCountingSink, SafeEditError, SafeEditExpected, SafeEditFactKey, SafeEditFactValue,
    SafeEditModuleInput, SafeEditProducerInput, SafeEditProducerProjection,
    SafeEditProjectionAdmission, SafeEditProjectionFact, SafeEditProvenance,
};
use oneagent_bsl::LineBslDeclarationExtractor;
use oneagent_common::{EntityId, sha256_hex};
use oneagent_graph::{EdgeKind, Provenance};
use std::path::Path;

/// Projects complete canonical provenance from captured before and exact result bytes.
///
/// No candidate or filesystem entry point is available to this function.
///
/// # Errors
/// Rejects incompatible producer evidence, an incomplete mapping or resource admission.
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
    for module in input.modules() {
        if module.document().format() != SourceFormat::DesignerXml {
            return Err(SafeEditError::SemanticMismatch);
        }
        let role = match module.document().module_role() {
            BslModuleRole::Common => DesignerXmlModuleKind::Common,
            BslModuleRole::Object => DesignerXmlModuleKind::Object,
            BslModuleRole::Manager => DesignerXmlModuleKind::Manager,
            _ => return Err(SafeEditError::SemanticMismatch),
        };
        // These are scoped results of the existing canonical parser. No parser
        // result is moved into the frozen projection.
        let before = LineBslDeclarationExtractor
            .extract_bounded(
                module.document().id().module_id(),
                std::str::from_utf8(module.document().raw_content())
                    .map_err(|_| SafeEditError::SemanticMismatch)?,
                MAX_SOURCE_OCCURRENCES_PER_DOCUMENT,
            )
            .map_err(|_| SafeEditError::SemanticMismatch)?;
        let after = LineBslDeclarationExtractor
            .extract_bounded(
                module.document().id().module_id(),
                std::str::from_utf8(module.result())
                    .map_err(|_| SafeEditError::SemanticMismatch)?,
                MAX_SOURCE_OCCURRENCES_PER_DOCUMENT,
            )
            .map_err(|_| SafeEditError::SemanticMismatch)?;
        if before.len() != after.len() {
            return Err(SafeEditError::SemanticMismatch);
        }
        let old_source = source(module, module.document().raw_content(), role, admission)?;
        let new_source = source(module, module.result(), role, admission)?;
        let old = module_provenance(module, &old_source, admission)?;
        let new = module_provenance(module, &new_source, admission)?;
        map_fact(
            &input,
            &mut facts,
            module.owner(),
            module.document().id().module_id(),
            &old,
            &new,
            admission,
        )?;
        release_provenance(old, admission)?;
        release_provenance(new, admission)?;
        for (old_symbol, new_symbol) in before.iter().zip(&after) {
            let is_target = old_symbol.id() == input.plan().target().target_node_id();
            let expected_id = if is_target {
                input.plan().target().expected_post_rename_node_id()
            } else {
                old_symbol.id()
            };
            let expected_name = if is_target {
                input.plan().request().desired_name()
            } else {
                old_symbol.name().as_str()
            };
            if new_symbol.id() != expected_id
                || new_symbol.name().as_str() != expected_name
                || old_symbol.kind() != new_symbol.kind()
                || old_symbol.line() != new_symbol.line()
                || old_symbol.is_exported() != new_symbol.is_exported()
            {
                return Err(SafeEditError::SemanticMismatch);
            }
            let old = declaration(
                module,
                &old_source,
                old_symbol.id(),
                old_symbol.line(),
                admission,
            )?;
            let new = declaration(
                module,
                &new_source,
                new_symbol.id(),
                new_symbol.line(),
                admission,
            )?;
            map_fact(
                &input,
                &mut facts,
                module.document().id().module_id(),
                old_symbol.id(),
                &old,
                &new,
                admission,
            )?;
            release_provenance(old, admission)?;
            release_provenance(new, admission)?;
        }
        let bytes = old_source.as_str().len() + new_source.as_str().len();
        drop(old_source);
        drop(new_source);
        admission.release(bytes)?;
    }
    SafeEditProducerProjection::new(input, facts, admission)
}

fn source(
    module: &SafeEditModuleInput<'_>,
    raw: &[u8],
    role: DesignerXmlModuleKind,
    admission: &mut SafeEditProjectionAdmission,
) -> Result<EntityId, SafeEditError> {
    let mut count = SafeEditCountingSink::default();
    write_module_source_id(
        &mut count,
        module.path().as_str(),
        "0000000000000000000000000000000000000000000000000000000000000000",
        module.document().id().module_id(),
        role,
    )
    .map_err(|_| SafeEditError::ProjectionBounds)?;
    admission.reserve(64)?;
    let digest = sha256_hex(raw);
    let value = admission.string(count.bytes(), |sink| {
        write_module_source_id(
            sink,
            module.path().as_str(),
            &digest,
            module.document().id().module_id(),
            role,
        )
    })?;
    drop(digest);
    admission.release(64)?;
    EntityId::new(value).map_err(|_| SafeEditError::SemanticMismatch)
}

fn module_provenance(
    module: &SafeEditModuleInput<'_>,
    source: &EntityId,
    admission: &mut SafeEditProjectionAdmission,
) -> Result<Provenance, SafeEditError> {
    admission.reserve(
        module
            .path()
            .as_str()
            .len()
            .checked_mul(2)
            .and_then(|n| n.checked_add(source.as_str().len()))
            .and_then(|n| n.checked_add(GRAPH_PRODUCER.len()))
            .ok_or(SafeEditError::ProjectionBounds)?,
    )?;
    let result = module_provenance_from_source(Path::new(module.path().as_str()), source.clone())
        .map_err(|_| SafeEditError::SemanticMismatch)?;
    admission.release(module.path().as_str().len())?;
    Ok(result)
}

fn declaration(
    module: &SafeEditModuleInput<'_>,
    source: &EntityId,
    symbol: &EntityId,
    line: usize,
    admission: &mut SafeEditProjectionAdmission,
) -> Result<Provenance, SafeEditError> {
    let mut count = SafeEditCountingSink::default();
    write_declaration_source_id(&mut count, source.as_str(), symbol, line)
        .map_err(|_| SafeEditError::ProjectionBounds)?;
    let value = admission.string(count.bytes(), |sink| {
        write_declaration_source_id(sink, source.as_str(), symbol, line)
    })?;
    // declaration_location clones the canonical path while constructing its span.
    admission.reserve(
        module
            .path()
            .as_str()
            .len()
            .checked_mul(2)
            .and_then(|n| n.checked_add(BSL_PRODUCER.len()))
            .ok_or(SafeEditError::ProjectionBounds)?,
    )?;
    let result = declaration_provenance_from_source(
        Path::new(module.path().as_str()),
        EntityId::new(value).map_err(|_| SafeEditError::SemanticMismatch)?,
        line,
    )
    .map_err(|_| SafeEditError::SemanticMismatch)?;
    admission.release(module.path().as_str().len())?;
    Ok(result)
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

fn map_fact(
    input: &SafeEditProducerInput<'_>,
    facts: &mut [SafeEditProjectionFact],
    owner: &EntityId,
    node: &EntityId,
    before: &Provenance,
    expected: &Provenance,
    admission: &mut SafeEditProjectionAdmission,
) -> Result<(), SafeEditError> {
    let old_node = input
        .graph()
        .node(node)
        .ok_or(SafeEditError::SemanticMismatch)?;
    let old_edge = input
        .graph()
        .edges()
        .find(|edge| {
            edge.source() == owner && edge.target() == node && edge.kind() == EdgeKind::Contains
        })
        .ok_or(SafeEditError::SemanticMismatch)?;
    if old_node.provenance() != std::slice::from_ref(before)
        || old_edge.provenance() != std::slice::from_ref(before)
    {
        return Err(SafeEditError::SemanticMismatch);
    }
    for fact in facts {
        let matches = match fact.key() {
            SafeEditFactKey::NodeFact { before } => before == node,
            SafeEditFactKey::EdgeFact {
                before_source,
                kind,
                before_target,
            } => before_source == owner && *kind == EdgeKind::Contains && before_target == node,
            _ => false,
        };
        if matches && before != expected {
            let provenance =
                SafeEditProvenance::copy_all(std::slice::from_ref(expected), admission)?;
            fact.set_expected(SafeEditFactValue::Provenance(SafeEditExpected::Expected(
                provenance,
            )))?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use oneagent_bsl::BslDeclarationExtractor;
    #[test]
    #[allow(clippy::too_many_lines)]
    fn designer_projection_preserves_canonical_provenance() {
        #[allow(clippy::too_many_lines)]
        fn complete_producer_before_reproduction() {
            use crate::DesignerXmlSemanticGraphBuilder;
            use oneagent_analysis::refactoring::*;
            use oneagent_analysis::safe_edit::*;
            use oneagent_common::SourcePath;
            use oneagent_graph::*;
            use std::sync::Arc;
            let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../..")
                .canonicalize()
                .unwrap();
            let root = workspace.join("apps/runtime/tests/fixtures/workspace_service/designer");
            let built = crate::FileSystemDesignerXmlSemanticGraphBuilder
                .build_graph_with_source_evidence(
                    &workspace,
                    &root,
                    crate::DesignerXmlBuildScope::Complete,
                )
                .unwrap();
            let sources = built.source_evidence();
            let target = built
                .graph()
                .nodes()
                .find(|node| node.name().as_str() == "FillSecurityCollection")
                .unwrap()
                .id()
                .clone();
            let diagnostics: Arc<[SemanticDiagnostic]> = Arc::from([]);
            let references = Arc::new(SemanticReferenceRequestLedger::new());
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
                    let module = sources.documents()[0].id().module_id();
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
        let raw = include_bytes!(
            "../tests/fixtures/sprint14_conformance/designer/CommonModules/DynamicSecurityOverridable/Ext/Module.bsl"
        );
        let module = EntityId::new("module").unwrap();
        let path = Path::new("CommonModules/Модуль/Ext/Module.bsl");
        for name in ["ДлинноеНовоеИмя", "Я", "FillSecurityCollection"] {
            let result = std::str::from_utf8(raw)
                .unwrap()
                .replace("FillSecurityCollection", name);
            let canonical = crate::semantic_graph::module_source_id(
                path,
                result.as_bytes(),
                &module,
                DesignerXmlModuleKind::Common,
            )
            .unwrap();
            let mut count = SafeEditCountingSink::default();
            let digest = sha256_hex(result.as_bytes());
            write_module_source_id(
                &mut count,
                path.to_str().unwrap(),
                &digest,
                &module,
                DesignerXmlModuleKind::Common,
            )
            .unwrap();
            let mut admission = SafeEditProjectionAdmission::new(0).unwrap();
            let encoded = admission
                .string(count.bytes(), |out| {
                    write_module_source_id(
                        out,
                        path.to_str().unwrap(),
                        &digest,
                        &module,
                        DesignerXmlModuleKind::Common,
                    )
                })
                .unwrap();
            assert_eq!(canonical.as_str(), encoded);
            assert_eq!(admission.retained_bytes(), encoded.capacity());
            let declarations = LineBslDeclarationExtractor
                .extract(&module, &result)
                .unwrap();
            assert!(declarations.len() >= 2);
            for declaration in declarations {
                let provenance = crate::semantic_graph::declaration_provenance(
                    path,
                    &canonical,
                    declaration.id(),
                    declaration.line(),
                )
                .unwrap();
                let mut count = SafeEditCountingSink::default();
                write_declaration_source_id(
                    &mut count,
                    &encoded,
                    declaration.id(),
                    declaration.line(),
                )
                .unwrap();
                let source = admission
                    .string(count.bytes(), |out| {
                        write_declaration_source_id(
                            out,
                            &encoded,
                            declaration.id(),
                            declaration.line(),
                        )
                    })
                    .unwrap();
                assert_eq!(provenance.source().unwrap().as_str(), source);
                let copied = SafeEditProvenance::copy(&provenance, &mut admission).unwrap();
                assert!(copied.matches(&provenance));
                assert!(!format!("{copied:?}").contains("sha256"));
                let mut exhausted = SafeEditProjectionAdmission::new(
                    oneagent_analysis::safe_edit::MAX_SAFE_EDIT_BUFFER_BYTES,
                )
                .unwrap();
                assert!(
                    exhausted
                        .string(count.bytes(), |out| write_declaration_source_id(
                            out,
                            &encoded,
                            declaration.id(),
                            declaration.line()
                        ))
                        .is_err()
                );
            }
        }
    }
}
