//! Private production emission for resolved EDT Writes candidates.

use oneagent_common::EntityId;
use oneagent_graph::{
    Confidence, EdgeKind, FactOrigin, GraphEdge, GraphError, NodeKind, ProducerId, Provenance,
    ResolutionState, SemanticGraph,
};
use oneagent_metadata::MetadataKind;
use std::collections::BTreeMap;
use std::fs;

use crate::metadata_object::{
    EdtDocumentRegisterDeclaration, EdtDocumentRegisterDeclarationProvenance,
};
use crate::query_source_resolution::WorkspaceResolutionScope;
use crate::writes::{EdtWritesCandidate, EdtWritesParseOutcome, extract_writes_candidates};
use crate::writes_resolution::{EdtWritesResolutionIndex, EdtWritesResolutionOutcome};
use crate::{EdtBslGraphError, EdtGraphError, EdtMetadataObjectDescriptor, EdtModuleDescriptor};

const WRITES_PARSER_STAGE: &str = "oneagent.edt.writes-parser";
const DOCUMENT_REGISTER_READER_STAGE: &str = "oneagent.edt.document-register-declaration-reader";
const WRITES_RESOLVER_STAGE: &str = "oneagent.edt.writes-resolution";
const WRITES_CONTRIBUTOR_STAGE: &str = "oneagent.edt.writes-emission";

/// Metadata owner and modules associated explicitly during EDT discovery.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EdtWritesSource {
    owner: EdtMetadataObjectDescriptor,
    modules: Vec<EdtModuleDescriptor>,
}

impl EdtWritesSource {
    pub(crate) const fn new(
        owner: EdtMetadataObjectDescriptor,
        modules: Vec<EdtModuleDescriptor>,
    ) -> Self {
        Self { owner, modules }
    }
}

/// Extracts, resolves, aggregates, and emits accepted Writes observations.
pub(crate) fn emit_resolved_writes(
    graph: &mut SemanticGraph,
    sources: &[EdtWritesSource],
    workspace_scope: WorkspaceResolutionScope,
) -> Result<usize, EdtGraphError> {
    let owners = unique_owners(sources);
    let mut candidates = Vec::new();

    for source in sources {
        for module in &source.modules {
            let module_source = fs::read_to_string(module.path()).map_err(|source| {
                EdtGraphError::Bsl(EdtBslGraphError::ReadModule {
                    path: module.path().to_path_buf(),
                    source,
                })
            })?;

            candidates.extend(
                extract_writes_candidates(&source.owner, module, &module_source)
                    .into_iter()
                    .filter_map(|outcome| match outcome {
                        EdtWritesParseOutcome::Candidate(candidate) => Some(*candidate),
                        EdtWritesParseOutcome::Rejected(_) => None,
                    }),
            );
        }
    }

    emit_resolved_candidates(graph, &owners, &candidates, workspace_scope)
}

fn unique_owners(sources: &[EdtWritesSource]) -> Vec<EdtMetadataObjectDescriptor> {
    sources
        .iter()
        .map(|source| {
            (
                (
                    source.owner.id().clone(),
                    source.owner.name().clone(),
                    source.owner.descriptor_path().to_path_buf(),
                ),
                source.owner.clone(),
            )
        })
        .collect::<BTreeMap<_, _>>()
        .into_values()
        .collect()
}

fn emit_resolved_candidates(
    graph: &mut SemanticGraph,
    owners: &[EdtMetadataObjectDescriptor],
    candidates: &[EdtWritesCandidate],
    workspace_scope: WorkspaceResolutionScope,
) -> Result<usize, EdtGraphError> {
    for candidate in candidates {
        require_node_kind(
            graph,
            &candidate.owner_id,
            NodeKind::Metadata(MetadataKind::Document),
        )?;
        require_node_kind(graph, &candidate.module_id, NodeKind::Module)?;
        require_node_kind(graph, &candidate.procedure_id, NodeKind::Procedure)?;
    }

    let resolution_index = EdtWritesResolutionIndex::new(owners, graph);
    let outcomes = resolution_index.resolve(candidates, workspace_scope);
    let mut evidence_by_edge = BTreeMap::<(EntityId, EntityId, EdgeKind), Vec<Provenance>>::new();

    for (candidate, outcome) in candidates.iter().zip(outcomes) {
        contribute_resolution_outcome(graph, candidate, outcome, &mut evidence_by_edge)?;
    }

    let mut inserted = 0;
    for ((source_id, target_id, kind), mut provenance) in evidence_by_edge {
        provenance.sort_by(compare_provenance);
        provenance.dedup();
        inserted += usize::from(
            graph
                .insert_edge(GraphEdge::new_with_provenance(
                    source_id, target_id, kind, provenance,
                ))
                .map_err(EdtGraphError::Graph)?,
        );
    }

    Ok(inserted)
}

fn contribute_resolution_outcome(
    graph: &SemanticGraph,
    candidate: &EdtWritesCandidate,
    outcome: EdtWritesResolutionOutcome,
    evidence_by_edge: &mut BTreeMap<(EntityId, EntityId, EdgeKind), Vec<Provenance>>,
) -> Result<(), EdtGraphError> {
    let EdtWritesResolutionOutcome::Resolved {
        declaration,
        target_id,
    } = outcome
    else {
        return Ok(());
    };

    require_node_kind(
        graph,
        &target_id,
        NodeKind::Metadata(MetadataKind::AccumulationRegister),
    )?;

    evidence_by_edge
        .entry((
            candidate.procedure_id.clone(),
            target_id.clone(),
            EdgeKind::Writes,
        ))
        .or_default()
        .push(writes_provenance(candidate, &declaration, &target_id)?);

    Ok(())
}

fn require_node_kind(
    graph: &SemanticGraph,
    id: &EntityId,
    expected_kind: NodeKind,
) -> Result<(), EdtGraphError> {
    if graph
        .node(id)
        .is_some_and(|node| node.kind() == expected_kind)
    {
        return Ok(());
    }

    Err(EdtGraphError::Graph(GraphError::MissingNode(id.clone())))
}

fn writes_provenance(
    candidate: &EdtWritesCandidate,
    declaration: &EdtDocumentRegisterDeclaration,
    target_id: &EntityId,
) -> Result<Provenance, EdtGraphError> {
    let declaration_ordinals = declaration_ordinals(declaration);
    let mut context = candidate.module_path.to_string_lossy().replace('\\', "/");
    context.push_str("#writes");
    append_candidate_context(&mut context, candidate);
    append_declaration_context(&mut context, declaration, &declaration_ordinals, target_id);
    append_producer_context(&mut context);

    let source = EntityId::new(context).map_err(|_| EdtGraphError::InvalidIdentifier)?;
    Ok(Provenance::new(
        Some(source),
        ProducerId::new(WRITES_CONTRIBUTOR_STAGE),
        FactOrigin::Resolved,
        Confidence::Exact,
        ResolutionState::Resolved,
    ))
}

fn declaration_ordinals(declaration: &EdtDocumentRegisterDeclaration) -> String {
    let mut declaration_ordinals = match &declaration.provenance {
        EdtDocumentRegisterDeclarationProvenance::Single(context) => vec![context.ordinal],
        EdtDocumentRegisterDeclarationProvenance::Duplicate(contexts) => {
            contexts.iter().map(|context| context.ordinal).collect()
        }
    };
    declaration_ordinals.sort_unstable();
    declaration_ordinals.dedup();
    declaration_ordinals
        .into_iter()
        .map(|ordinal| ordinal.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

fn append_candidate_context(context: &mut String, candidate: &EdtWritesCandidate) {
    append_context(context, "procedure_id", candidate.procedure_id.as_str());
    append_context(context, "procedure_name", candidate.procedure_name.as_str());
    append_context(context, "module_id", candidate.module_id.as_str());
    append_context(
        context,
        "module_artifact",
        &candidate.module_path.to_string_lossy().replace('\\', "/"),
    );
    append_context(context, "owner_id", candidate.owner_id.as_str());
    append_context(context, "owner_name", candidate.owner_name.as_str());
    append_context(
        context,
        "candidate_line",
        &candidate.location.line.to_string(),
    );
    append_context(
        context,
        "candidate_column",
        &candidate.location.column.to_string(),
    );
    append_context(context, "raw_statement", &candidate.raw_statement);
    append_context(context, "receiver_spelling", &candidate.receiver_spelling);
    append_context(context, "method_spelling", &candidate.method_spelling);
    append_context(context, "register_name", &candidate.local_name);
    append_context(context, "normalized_register", &candidate.lookup_key);
    append_context(
        context,
        "zero_arguments",
        if candidate.zero_arguments {
            "true"
        } else {
            "false"
        },
    );
    append_context(
        context,
        "complete_statement",
        if candidate.complete_statement {
            "true"
        } else {
            "false"
        },
    );
}

fn append_declaration_context(
    context: &mut String,
    declaration: &EdtDocumentRegisterDeclaration,
    declaration_ordinals: &str,
    target_id: &EntityId,
) {
    append_context(
        context,
        "declaration_descriptor",
        &declaration
            .descriptor_path
            .to_string_lossy()
            .replace('\\', "/"),
    );
    append_context(context, "raw_declaration", &declaration.raw_value);
    append_context(context, "declaration_namespace", &declaration.namespace);
    append_context(context, "declaration_local_name", &declaration.local_name);
    append_context(
        context,
        "declaration_kind",
        declaration.kind.map_or("unsupported", MetadataKind::as_str),
    );
    append_context(context, "declaration_ordinals", declaration_ordinals);
    append_context(context, "resolved_target", target_id.as_str());
    append_context(
        context,
        "target_kind",
        MetadataKind::AccumulationRegister.as_str(),
    );
}

fn append_producer_context(context: &mut String) {
    append_context(context, "parser_stage", WRITES_PARSER_STAGE);
    append_context(
        context,
        "declaration_reader_stage",
        DOCUMENT_REGISTER_READER_STAGE,
    );
    append_context(context, "resolver_stage", WRITES_RESOLVER_STAGE);
    append_context(context, "contributor_stage", WRITES_CONTRIBUTOR_STAGE);
}

fn append_context(context: &mut String, key: &str, value: &str) {
    context.push(';');
    context.push_str(key);
    context.push('#');
    context.push_str(&value.len().to_string());
    context.push(':');
    context.push_str(value);
}

fn compare_provenance(left: &Provenance, right: &Provenance) -> std::cmp::Ordering {
    left.source()
        .cmp(&right.source())
        .then_with(|| left.producer().cmp(right.producer()))
        .then_with(|| left.origin().cmp(&right.origin()))
        .then_with(|| left.confidence().cmp(&right.confidence()))
        .then_with(|| left.resolution().cmp(&right.resolution()))
}

#[cfg(test)]
mod tests {
    use oneagent_common::{EntityId, EntityName};
    use oneagent_graph::{
        Confidence, EdgeKind, FactOrigin, GraphError, GraphNode, NodeKind, ResolutionState,
        SemanticGraph,
    };
    use oneagent_metadata::MetadataKind;
    use std::collections::BTreeMap;
    use std::fs;
    use std::path::{Path, PathBuf};
    use tempfile::tempdir;

    use super::{
        EdtWritesSource, contribute_resolution_outcome, emit_resolved_candidates,
        emit_resolved_writes, unique_owners,
    };
    use crate::query_source_resolution::WorkspaceResolutionScope;
    use crate::writes::{EdtWritesCandidate, EdtWritesParseOutcome, extract_writes_candidates};
    use crate::writes_resolution::EdtWritesResolutionOutcome;
    use crate::{
        EdtGraphError, EdtMetadataObjectReader, EdtModuleKind, EdtModuleReader,
        EdtSemanticGraphBuilder, FileSystemEdtMetadataObjectReader, FileSystemEdtModuleReader,
        FileSystemEdtSemanticGraphBuilder,
    };

    fn id(value: &str) -> EntityId {
        EntityId::new(value).expect("identifier must be valid")
    }

    fn name(value: &str) -> EntityName {
        EntityName::new(value).expect("name must be valid")
    }

    fn writes_project_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/writes_project")
    }

    fn write_source(
        root: &Path,
        identifier: &str,
        object_name: &str,
        register_records: &str,
        module_source: &str,
    ) -> EdtWritesSource {
        let object_directory = root.join(object_name);
        fs::create_dir_all(&object_directory).expect("object directory must be created");
        fs::write(
            object_directory.join(format!("{object_name}.mdo")),
            format!(
                r#"<mdclass:Document xmlns:mdclass="urn:test" uuid="{identifier}">
    <name>{object_name}</name>
{register_records}
</mdclass:Document>"#
            ),
        )
        .expect("Document descriptor must be written");
        fs::write(object_directory.join("ObjectModule.bsl"), module_source)
            .expect("Object Module must be written");

        let owner = FileSystemEdtMetadataObjectReader
            .read(&object_directory, MetadataKind::Document)
            .expect("Document descriptor must load");
        let modules = FileSystemEdtModuleReader
            .read_modules(owner.id(), owner.name(), &object_directory)
            .expect("Object Module must load");

        EdtWritesSource::new(owner, modules)
    }

    fn insert_source_context(
        graph: &mut SemanticGraph,
        source: &EdtWritesSource,
        procedure_kind: Option<NodeKind>,
    ) -> EntityId {
        graph.insert_node(GraphNode::new(
            source.owner.id().clone(),
            source.owner.name().clone(),
            NodeKind::Metadata(MetadataKind::Document),
        ));
        let module = source
            .modules
            .iter()
            .find(|module| module.kind() == EdtModuleKind::Object)
            .expect("Object Module must exist");
        graph.insert_node(GraphNode::new(
            module.id().clone(),
            module.name().clone(),
            NodeKind::Module,
        ));
        let procedure_id = id(&format!("{}:procedure:Posting", module.id().as_str()));
        if let Some(kind) = procedure_kind {
            graph.insert_node(GraphNode::new(procedure_id.clone(), name("Posting"), kind));
        }
        procedure_id
    }

    fn insert_accumulation_register(
        graph: &mut SemanticGraph,
        identifier: &str,
        register_name: &str,
    ) {
        graph.insert_node(GraphNode::new(
            id(identifier),
            name(register_name),
            NodeKind::Metadata(MetadataKind::AccumulationRegister),
        ));
    }

    fn accepted_candidates(source: &EdtWritesSource) -> Vec<EdtWritesCandidate> {
        let module = source
            .modules
            .iter()
            .find(|module| module.kind() == EdtModuleKind::Object)
            .expect("Object Module must exist");
        let module_source = fs::read_to_string(module.path()).expect("module source must load");
        extract_writes_candidates(&source.owner, module, &module_source)
            .into_iter()
            .filter_map(|outcome| match outcome {
                EdtWritesParseOutcome::Candidate(candidate) => Some(*candidate),
                EdtWritesParseOutcome::Rejected(_) => None,
            })
            .collect()
    }

    fn writes_snapshot(
        graph: &SemanticGraph,
    ) -> Vec<(EntityId, EntityId, Vec<oneagent_graph::Provenance>)> {
        graph
            .edges()
            .filter(|edge| edge.kind() == EdgeKind::Writes)
            .map(|edge| {
                (
                    edge.source().clone(),
                    edge.target().clone(),
                    edge.provenance().to_vec(),
                )
            })
            .collect()
    }

    #[test]
    fn repository_project_emits_two_exact_valid_writes_edges() {
        let result = FileSystemEdtSemanticGraphBuilder
            .build_graph_with_diagnostics(&writes_project_root())
            .expect("fixture graph must build");
        let graph = result.graph();
        let writes = graph
            .edges()
            .filter(|edge| edge.kind() == EdgeKind::Writes)
            .collect::<Vec<_>>();
        let procedure_id =
            id("ed647f67-f8fe-476b-8823-8d52b365ab20:object_module:procedure:Posting");

        assert_eq!(writes.len(), 2);
        assert_eq!(writes[0].source(), &procedure_id);
        assert_eq!(
            writes[0].target(),
            &id("ac997c18-b62c-4bc3-9079-9a729ad5253c")
        );
        assert_eq!(writes[1].source(), &procedure_id);
        assert_eq!(
            writes[1].target(),
            &id("f014a53e-bf0e-4dc4-9a8c-93ef663d9108")
        );
        assert_eq!(
            graph.node(&procedure_id).map(GraphNode::kind),
            Some(NodeKind::Procedure)
        );

        for edge in writes {
            assert_eq!(
                graph.node(edge.target()).map(GraphNode::kind),
                Some(NodeKind::Metadata(MetadataKind::AccumulationRegister))
            );
            assert_eq!(edge.provenance().len(), 1);
            let provenance = &edge.provenance()[0];
            assert_eq!(provenance.origin(), FactOrigin::Resolved);
            assert_eq!(provenance.confidence(), Confidence::Exact);
            assert_eq!(provenance.resolution(), ResolutionState::Resolved);
            assert_eq!(
                provenance.producer().as_str(),
                "oneagent.edt.writes-emission"
            );
            let source = provenance
                .source()
                .expect("Writes provenance source must exist")
                .as_str();
            for field in [
                "procedure_id#",
                "module_id#",
                "owner_id#",
                "module_artifact#",
                "candidate_line#",
                "candidate_column#",
                "raw_statement#",
                "normalized_register#",
                "declaration_descriptor#",
                "raw_declaration#",
                "declaration_kind#",
                "declaration_ordinals#",
                "resolved_target#",
                "target_kind#",
                "parser_stage#",
                "declaration_reader_stage#",
                "resolver_stage#",
                "contributor_stage#",
            ] {
                assert!(source.contains(field), "missing provenance field {field}");
            }
        }

        assert!(result.validate().is_valid());
        assert!(graph.edges().all(|edge| {
            edge.source() != &procedure_id
                || !matches!(
                    edge.target().as_str(),
                    "ac997c18-b62c-4bc3-9079-9a729ad5253c" | "f014a53e-bf0e-4dc4-9a8c-93ef663d9108"
                )
                || edge.kind() == EdgeKind::Writes
        }));
    }

    #[test]
    fn rejected_and_every_non_resolved_outcome_emit_no_edge_or_placeholder() {
        let root = tempdir().expect("temporary directory must be created");
        let source = write_source(
            root.path(),
            "document-no-edge",
            "NoEdgeDocument",
            "",
            concat!(
                "Procedure Posting()\n",
                "    LocalObject.Write();\n",
                "    RegisterRecords.Missing.Write();\n",
                "EndProcedure\n",
            ),
        );
        let mut graph = SemanticGraph::new();
        insert_source_context(&mut graph, &source, Some(NodeKind::Procedure));
        let node_count = graph.node_count();

        assert_eq!(
            emit_resolved_writes(
                &mut graph,
                std::slice::from_ref(&source),
                WorkspaceResolutionScope::Complete,
            )
            .expect("rejected and unresolved outcomes must be ignored"),
            0
        );
        assert_eq!(graph.node_count(), node_count);
        assert!(writes_snapshot(&graph).is_empty());

        let candidate = accepted_candidates(&source)
            .into_iter()
            .next()
            .expect("accepted unresolved candidate must exist");
        let outcomes = vec![
            EdtWritesResolutionOutcome::MissingOwner,
            EdtWritesResolutionOutcome::AmbiguousOwner {
                descriptor_paths: vec![PathBuf::from("a.mdo"), PathBuf::from("z.mdo")],
            },
            EdtWritesResolutionOutcome::MissingDeclaration,
            EdtWritesResolutionOutcome::UnsupportedDeclaration {
                declarations: Vec::new(),
            },
            EdtWritesResolutionOutcome::AmbiguousDeclaration {
                declarations: Vec::new(),
            },
            EdtWritesResolutionOutcome::MissingTarget,
            EdtWritesResolutionOutcome::PartialWorkspaceTargetAbsent,
            EdtWritesResolutionOutcome::IncompatibleTargetKind {
                candidates: vec![id("catalog.missing")],
            },
            EdtWritesResolutionOutcome::AmbiguousTarget {
                candidates: vec![id("target.a"), id("target.z")],
            },
        ];
        let mut evidence = BTreeMap::new();
        for outcome in outcomes {
            contribute_resolution_outcome(&graph, &candidate, outcome, &mut evidence)
                .expect("non-resolved outcome must not fail");
        }
        assert!(evidence.is_empty());
        assert_eq!(graph.node_count(), node_count);
    }

    #[test]
    fn missing_or_wrong_kind_procedure_is_an_invariant_failure() {
        let root = tempdir().expect("temporary directory must be created");
        let source = write_source(
            root.path(),
            "document-invariant",
            "InvariantDocument",
            "    <registerRecords>AccumulationRegister.Stock</registerRecords>",
            concat!(
                "Procedure Posting()\n",
                "    RegisterRecords.Stock.Write();\n",
                "EndProcedure\n",
            ),
        );

        let mut missing = SemanticGraph::new();
        let procedure_id = insert_source_context(&mut missing, &source, None);
        insert_accumulation_register(&mut missing, "target.stock", "Stock");
        let error = emit_resolved_writes(
            &mut missing,
            std::slice::from_ref(&source),
            WorkspaceResolutionScope::Complete,
        )
        .expect_err("missing Procedure must fail");
        assert!(matches!(
            error,
            EdtGraphError::Graph(GraphError::MissingNode(ref actual)) if actual == &procedure_id
        ));

        let mut wrong_kind = SemanticGraph::new();
        insert_source_context(&mut wrong_kind, &source, Some(NodeKind::Function));
        insert_accumulation_register(&mut wrong_kind, "target.stock", "Stock");
        let error = emit_resolved_writes(
            &mut wrong_kind,
            std::slice::from_ref(&source),
            WorkspaceResolutionScope::Complete,
        )
        .expect_err("wrong-kind Procedure must fail");
        assert!(matches!(
            error,
            EdtGraphError::Graph(GraphError::MissingNode(ref actual)) if actual == &procedure_id
        ));
        assert!(writes_snapshot(&wrong_kind).is_empty());
    }

    #[test]
    fn duplicate_occurrences_and_declarations_aggregate_before_insertion() {
        let root = tempdir().expect("temporary directory must be created");
        let source = write_source(
            root.path(),
            "document-duplicates",
            "DuplicateDocument",
            concat!(
                "    <registerRecords>AccumulationRegister.Stock</registerRecords>\n",
                "    <registerRecords>AccumulationRegister.Stock</registerRecords>",
            ),
            concat!(
                "Procedure Posting()\n",
                "    RegisterRecords.Stock.Write();\n",
                "    RegisterRecords.Stock.Write();\n",
                "EndProcedure\n",
            ),
        );
        let mut graph = SemanticGraph::new();
        insert_source_context(&mut graph, &source, Some(NodeKind::Procedure));
        insert_accumulation_register(&mut graph, "target.stock", "Stock");
        let node_count = graph.node_count();

        assert_eq!(
            emit_resolved_writes(
                &mut graph,
                &[source.clone(), source],
                WorkspaceResolutionScope::Complete,
            )
            .expect("duplicate evidence must emit"),
            1
        );

        let writes = writes_snapshot(&graph);
        assert_eq!(writes.len(), 1);
        assert_eq!(writes[0].2.len(), 2);
        assert_eq!(graph.node_count(), node_count);
        let provenance_sources = writes[0]
            .2
            .iter()
            .map(|provenance| {
                provenance
                    .source()
                    .expect("provenance source must exist")
                    .as_str()
            })
            .collect::<Vec<_>>();
        assert!(
            provenance_sources
                .iter()
                .all(|source| source.contains("declaration_ordinals#3:1,2"))
        );
        assert!(
            provenance_sources
                .iter()
                .any(|source| source.contains("candidate_line#1:2"))
        );
        assert!(
            provenance_sources
                .iter()
                .any(|source| source.contains("candidate_line#1:3"))
        );
    }

    #[test]
    fn candidate_owner_and_target_order_do_not_change_normalized_output() {
        let root = tempdir().expect("temporary directory must be created");
        let source = write_source(
            root.path(),
            "document-order",
            "OrderDocument",
            concat!(
                "    <registerRecords>AccumulationRegister.Alpha</registerRecords>\n",
                "    <registerRecords>AccumulationRegister.Beta</registerRecords>",
            ),
            concat!(
                "Procedure Posting()\n",
                "    RegisterRecords.Beta.Write();\n",
                "    RegisterRecords.Alpha.Write();\n",
                "EndProcedure\n",
            ),
        );
        let unrelated = write_source(
            root.path(),
            "document-unrelated",
            "UnrelatedDocument",
            "",
            "Procedure Ignore()\nEndProcedure\n",
        );
        let mut candidates = accepted_candidates(&source);
        let owners = unique_owners(&[unrelated.clone(), source.clone()]);
        let mut first = SemanticGraph::new();
        insert_source_context(&mut first, &source, Some(NodeKind::Procedure));
        insert_accumulation_register(&mut first, "target.beta", "Beta");
        insert_accumulation_register(&mut first, "target.alpha", "Alpha");
        let mut second = SemanticGraph::new();
        insert_accumulation_register(&mut second, "target.alpha", "Alpha");
        insert_accumulation_register(&mut second, "target.beta", "Beta");
        insert_source_context(&mut second, &source, Some(NodeKind::Procedure));

        emit_resolved_candidates(
            &mut first,
            &owners,
            &candidates,
            WorkspaceResolutionScope::Complete,
        )
        .expect("normal order must emit");
        candidates.reverse();
        let mut reversed_owners = owners;
        reversed_owners.reverse();
        emit_resolved_candidates(
            &mut second,
            &reversed_owners,
            &candidates,
            WorkspaceResolutionScope::Complete,
        )
        .expect("reversed order must emit");

        assert_eq!(writes_snapshot(&first), writes_snapshot(&second));
    }
}
