//! Deterministic private resolution for accepted EDT Writes candidates.

use oneagent_common::{EntityId, EntityName};
use oneagent_graph::{NodeKind, SemanticGraph};
use oneagent_metadata::MetadataKind;
use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::EdtMetadataObjectDescriptor;
use crate::metadata_object::{
    EdtDocumentRegisterDeclaration, EdtDocumentRegisterDeclarationOutcome,
};
use crate::query_source_resolution::WorkspaceResolutionScope;
use crate::writes::EdtWritesCandidate;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum EdtWritesResolutionOutcome {
    Resolved {
        declaration: Box<EdtDocumentRegisterDeclaration>,
        target_id: EntityId,
    },
    MissingOwner,
    AmbiguousOwner {
        descriptor_paths: Vec<PathBuf>,
    },
    MissingDeclaration,
    UnsupportedDeclaration {
        declarations: Vec<EdtDocumentRegisterDeclaration>,
    },
    AmbiguousDeclaration {
        declarations: Vec<EdtDocumentRegisterDeclaration>,
    },
    MissingTarget,
    PartialWorkspaceTargetAbsent,
    IncompatibleTargetKind {
        candidates: Vec<EntityId>,
    },
    AmbiguousTarget {
        candidates: Vec<EntityId>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct EdtWritesTargetCandidate {
    id: EntityId,
    kind: NodeKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum EdtWritesDeclarationResolution {
    Resolved(EdtDocumentRegisterDeclaration),
    Failed(EdtWritesResolutionOutcome),
}

/// Immutable owner, declaration, and graph-target index for one EDT graph snapshot.
#[derive(Debug)]
pub(crate) struct EdtWritesResolutionIndex {
    owners_by_identity: BTreeMap<(EntityId, EntityName), Vec<EdtMetadataObjectDescriptor>>,
    targets_by_lookup_key: BTreeMap<String, BTreeMap<EntityId, EdtWritesTargetCandidate>>,
}

impl EdtWritesResolutionIndex {
    #[must_use]
    pub(crate) fn new(owners: &[EdtMetadataObjectDescriptor], graph: &SemanticGraph) -> Self {
        let mut owners_by_identity =
            BTreeMap::<(EntityId, EntityName), Vec<EdtMetadataObjectDescriptor>>::new();

        for owner in owners
            .iter()
            .filter(|owner| owner.kind() == MetadataKind::Document)
        {
            owners_by_identity
                .entry((owner.id().clone(), owner.name().clone()))
                .or_default()
                .push(owner.clone());
        }
        for matching_owners in owners_by_identity.values_mut() {
            matching_owners
                .sort_by(|left, right| left.descriptor_path().cmp(right.descriptor_path()));
        }

        let mut targets_by_lookup_key =
            BTreeMap::<String, BTreeMap<EntityId, EdtWritesTargetCandidate>>::new();

        for node in graph.nodes() {
            let target = EdtWritesTargetCandidate {
                id: node.id().clone(),
                kind: node.kind(),
            };
            targets_by_lookup_key
                .entry(node.name().as_str().to_lowercase())
                .or_default()
                .insert(target.id.clone(), target);
        }

        Self {
            owners_by_identity,
            targets_by_lookup_key,
        }
    }

    /// Resolves accepted candidates independently and preserves their input order.
    #[must_use]
    pub(crate) fn resolve(
        &self,
        candidates: &[EdtWritesCandidate],
        workspace_scope: WorkspaceResolutionScope,
    ) -> Vec<EdtWritesResolutionOutcome> {
        candidates
            .iter()
            .map(|candidate| self.resolve_candidate(candidate, workspace_scope))
            .collect()
    }

    fn resolve_candidate(
        &self,
        candidate: &EdtWritesCandidate,
        workspace_scope: WorkspaceResolutionScope,
    ) -> EdtWritesResolutionOutcome {
        let owner_identity = (candidate.owner_id.clone(), candidate.owner_name.clone());
        let Some(owners) = self.owners_by_identity.get(&owner_identity) else {
            return EdtWritesResolutionOutcome::MissingOwner;
        };
        let [owner] = owners.as_slice() else {
            return EdtWritesResolutionOutcome::AmbiguousOwner {
                descriptor_paths: owners
                    .iter()
                    .map(|owner| owner.descriptor_path().to_path_buf())
                    .collect(),
            };
        };

        let declaration = match resolve_declaration(owner, &candidate.lookup_key) {
            EdtWritesDeclarationResolution::Resolved(declaration) => declaration,
            EdtWritesDeclarationResolution::Failed(outcome) => return outcome,
        };

        debug_assert_eq!(declaration.lookup_key, candidate.lookup_key);
        self.resolve_target(declaration, workspace_scope)
    }

    fn resolve_target(
        &self,
        declaration: EdtDocumentRegisterDeclaration,
        workspace_scope: WorkspaceResolutionScope,
    ) -> EdtWritesResolutionOutcome {
        let Some(candidates) = self.targets_by_lookup_key.get(&declaration.lookup_key) else {
            return absent_target_outcome(workspace_scope);
        };
        let expected_kind = NodeKind::Metadata(MetadataKind::AccumulationRegister);
        let compatible = candidates
            .values()
            .filter(|candidate| candidate.kind == expected_kind)
            .map(|candidate| candidate.id.clone())
            .collect::<Vec<_>>();

        match compatible.as_slice() {
            [] => EdtWritesResolutionOutcome::IncompatibleTargetKind {
                candidates: candidates.keys().cloned().collect(),
            },
            [target_id] => EdtWritesResolutionOutcome::Resolved {
                declaration: Box::new(declaration),
                target_id: target_id.clone(),
            },
            _ => EdtWritesResolutionOutcome::AmbiguousTarget {
                candidates: compatible,
            },
        }
    }
}

fn resolve_declaration(
    owner: &EdtMetadataObjectDescriptor,
    lookup_key: &str,
) -> EdtWritesDeclarationResolution {
    let mut compatible = Vec::<EdtDocumentRegisterDeclaration>::new();
    let mut ambiguous_compatible = Vec::<EdtDocumentRegisterDeclaration>::new();
    let mut unsupported = Vec::<EdtDocumentRegisterDeclaration>::new();

    for outcome in owner.document_register_declarations() {
        match outcome {
            EdtDocumentRegisterDeclarationOutcome::Supported(declaration)
                if declaration.lookup_key == lookup_key =>
            {
                compatible.push(declaration.clone());
            }
            EdtDocumentRegisterDeclarationOutcome::UnsupportedKind(declaration)
            | EdtDocumentRegisterDeclarationOutcome::UnsupportedNamespace(declaration)
                if declaration.lookup_key == lookup_key =>
            {
                unsupported.push(declaration.clone());
            }
            EdtDocumentRegisterDeclarationOutcome::Ambiguous(ambiguous)
                if ambiguous.lookup_key == lookup_key
                    && ambiguous.kind == MetadataKind::AccumulationRegister =>
            {
                ambiguous_compatible.extend(ambiguous.declarations.iter().cloned());
            }
            EdtDocumentRegisterDeclarationOutcome::Ambiguous(ambiguous)
                if ambiguous.lookup_key == lookup_key =>
            {
                unsupported.extend(ambiguous.declarations.iter().cloned());
            }
            EdtDocumentRegisterDeclarationOutcome::Supported(_)
            | EdtDocumentRegisterDeclarationOutcome::UnsupportedKind(_)
            | EdtDocumentRegisterDeclarationOutcome::UnsupportedNamespace(_)
            | EdtDocumentRegisterDeclarationOutcome::Malformed(_)
            | EdtDocumentRegisterDeclarationOutcome::Ambiguous(_) => {}
        }
    }

    if !ambiguous_compatible.is_empty() || compatible.len() > 1 {
        ambiguous_compatible.extend(compatible);
        sort_declarations(&mut ambiguous_compatible);
        return EdtWritesDeclarationResolution::Failed(
            EdtWritesResolutionOutcome::AmbiguousDeclaration {
                declarations: ambiguous_compatible,
            },
        );
    }

    if let Some(declaration) = compatible.pop() {
        return EdtWritesDeclarationResolution::Resolved(declaration);
    }

    if unsupported.is_empty() {
        EdtWritesDeclarationResolution::Failed(EdtWritesResolutionOutcome::MissingDeclaration)
    } else {
        sort_declarations(&mut unsupported);
        EdtWritesDeclarationResolution::Failed(EdtWritesResolutionOutcome::UnsupportedDeclaration {
            declarations: unsupported,
        })
    }
}

fn sort_declarations(declarations: &mut [EdtDocumentRegisterDeclaration]) {
    declarations.sort_by(|left, right| {
        left.kind
            .cmp(&right.kind)
            .then_with(|| left.namespace.cmp(&right.namespace))
            .then_with(|| left.local_name.cmp(&right.local_name))
            .then_with(|| left.raw_value.cmp(&right.raw_value))
            .then_with(|| left.owner_id.cmp(&right.owner_id))
            .then_with(|| left.owner_name.cmp(&right.owner_name))
            .then_with(|| left.descriptor_path.cmp(&right.descriptor_path))
    });
}

const fn absent_target_outcome(
    workspace_scope: WorkspaceResolutionScope,
) -> EdtWritesResolutionOutcome {
    match workspace_scope {
        WorkspaceResolutionScope::Complete => EdtWritesResolutionOutcome::MissingTarget,
        WorkspaceResolutionScope::Partial => {
            EdtWritesResolutionOutcome::PartialWorkspaceTargetAbsent
        }
    }
}

#[cfg(test)]
mod tests {
    use oneagent_common::{EntityId, EntityName};
    use oneagent_graph::{EdgeKind, GraphNode, NodeKind, SemanticGraph};
    use oneagent_metadata::MetadataKind;
    use std::fs;
    use std::path::{Path, PathBuf};
    use tempfile::tempdir;

    use super::{EdtWritesResolutionIndex, EdtWritesResolutionOutcome};
    use crate::metadata_object::EdtDocumentRegisterDeclarationProvenance;
    use crate::query_source_resolution::WorkspaceResolutionScope;
    use crate::writes::{
        EdtWritesCandidate, EdtWritesParseOutcome, EdtWritesSourceLocation,
        extract_writes_candidates,
    };
    use crate::{
        EdtMetadataObjectDescriptor, EdtMetadataObjectReader, EdtModuleKind, EdtModuleReader,
        EdtSemanticGraphBuilder, FileSystemEdtMetadataObjectReader, FileSystemEdtModuleReader,
        FileSystemEdtSemanticGraphBuilder,
    };

    fn id(value: &str) -> EntityId {
        EntityId::new(value).expect("identifier must be valid")
    }

    fn name(value: &str) -> EntityName {
        EntityName::new(value).expect("name must be valid")
    }

    fn candidate(owner: &EdtMetadataObjectDescriptor, local_name: &str) -> EdtWritesCandidate {
        EdtWritesCandidate {
            owner_id: owner.id().clone(),
            owner_name: owner.name().clone(),
            module_id: id("document-generated:object_module"),
            module_path: PathBuf::from("ObjectModule.bsl"),
            procedure_id: id("document-generated:object_module:procedure:Posting"),
            procedure_name: name("Posting"),
            raw_statement: format!("RegisterRecords.{local_name}.Write();"),
            receiver_spelling: "RegisterRecords".to_owned(),
            local_name: local_name.to_owned(),
            method_spelling: "Write".to_owned(),
            lookup_key: local_name.to_lowercase(),
            zero_arguments: true,
            complete_statement: true,
            location: EdtWritesSourceLocation { line: 2, column: 1 },
        }
    }

    fn descriptor(
        identifier: &str,
        object_name: &str,
        kind: MetadataKind,
        path: &str,
    ) -> EdtMetadataObjectDescriptor {
        EdtMetadataObjectDescriptor::new(
            id(identifier),
            name(object_name),
            None,
            kind,
            None,
            PathBuf::from(path),
        )
    }

    fn write_document_xml(object_directory: &Path, register_records: &str) {
        fs::create_dir_all(object_directory).expect("object directory must be created");
        let xml = format!(
            r#"<mdclass:Document xmlns:mdclass="urn:test" uuid="document-generated">
    <name>GeneratedDocument</name>
{register_records}
</mdclass:Document>"#
        );
        fs::write(object_directory.join("GeneratedDocument.mdo"), xml)
            .expect("generated descriptor must be written");
    }

    fn read_document(object_directory: &Path) -> EdtMetadataObjectDescriptor {
        FileSystemEdtMetadataObjectReader
            .read(object_directory, MetadataKind::Document)
            .expect("generated Document must load")
    }

    fn generated_document(register_records: &str) -> EdtMetadataObjectDescriptor {
        let root = tempdir().expect("temporary directory must be created");
        let object_directory = root.path().join("GeneratedDocument");
        write_document_xml(&object_directory, register_records);
        read_document(&object_directory)
    }

    fn insert_node(graph: &mut SemanticGraph, identifier: &str, value: &str, kind: NodeKind) {
        graph.insert_node(GraphNode::new(id(identifier), name(value), kind));
    }

    fn accumulation_node(graph: &mut SemanticGraph, identifier: &str, value: &str) {
        insert_node(
            graph,
            identifier,
            value,
            NodeKind::Metadata(MetadataKind::AccumulationRegister),
        );
    }

    fn resolve_one(
        owner: &EdtMetadataObjectDescriptor,
        graph: &SemanticGraph,
        candidate: &EdtWritesCandidate,
        workspace_scope: WorkspaceResolutionScope,
    ) -> EdtWritesResolutionOutcome {
        EdtWritesResolutionIndex::new(std::slice::from_ref(owner), graph)
            .resolve(std::slice::from_ref(candidate), workspace_scope)
            .into_iter()
            .next()
            .expect("one candidate must yield one outcome")
    }

    fn writes_project_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/writes_project")
    }

    #[test]
    fn repository_backed_project_resolves_both_candidates_with_declaration_evidence() {
        let project_root = writes_project_root();
        let document_directory = project_root.join("src/Documents/RefundOfPaymentByOrder");
        let owner = FileSystemEdtMetadataObjectReader
            .read(&document_directory, MetadataKind::Document)
            .expect("fixture Document must load");
        let module = FileSystemEdtModuleReader
            .read_modules(owner.id(), owner.name(), &document_directory)
            .expect("fixture modules must load")
            .into_iter()
            .find(|module| module.kind() == EdtModuleKind::Object)
            .expect("fixture Object Module must exist");
        let source = fs::read_to_string(module.path()).expect("fixture module must load");
        let candidates = extract_writes_candidates(&owner, &module, &source)
            .into_iter()
            .map(|outcome| match outcome {
                EdtWritesParseOutcome::Candidate(candidate) => *candidate,
                EdtWritesParseOutcome::Rejected(rejection) => {
                    panic!("fixture statement must be accepted: {rejection:?}")
                }
            })
            .collect::<Vec<_>>();
        let graph = FileSystemEdtSemanticGraphBuilder
            .build_graph(&project_root)
            .expect("fixture graph must build");
        let node_count = graph.node_count();
        let edge_count = graph.edge_count();

        let outcomes = EdtWritesResolutionIndex::new(std::slice::from_ref(&owner), &graph)
            .resolve(&candidates, WorkspaceResolutionScope::Complete);

        assert_eq!(candidates.len(), 2);
        assert_eq!(outcomes.len(), 2);
        let expected = [
            (
                "CashAccountBalance",
                "AccumulationRegister.CashAccountBalance",
                "ac997c18-b62c-4bc3-9079-9a729ad5253c",
                1,
            ),
            (
                "RefundBankPayment",
                "AccumulationRegister.RefundBankPayment",
                "f014a53e-bf0e-4dc4-9a8c-93ef663d9108",
                2,
            ),
        ];

        for ((candidate, outcome), (local_name, raw_value, target_id, ordinal)) in
            candidates.iter().zip(&outcomes).zip(expected)
        {
            let EdtWritesResolutionOutcome::Resolved {
                declaration,
                target_id: resolved_target_id,
            } = outcome
            else {
                panic!("fixture candidate must resolve: {outcome:?}");
            };

            assert_eq!(candidate.local_name, local_name);
            assert_eq!(declaration.owner_id, *owner.id());
            assert_eq!(declaration.owner_name, *owner.name());
            assert_eq!(declaration.descriptor_path, owner.descriptor_path());
            assert_eq!(declaration.raw_value, raw_value);
            assert_eq!(declaration.namespace, "AccumulationRegister");
            assert_eq!(declaration.local_name, local_name);
            assert_eq!(declaration.lookup_key, candidate.lookup_key);
            assert_eq!(declaration.kind, Some(MetadataKind::AccumulationRegister));
            assert_eq!(resolved_target_id, &id(target_id));
            assert!(matches!(
                declaration.provenance,
                EdtDocumentRegisterDeclarationProvenance::Single(context)
                    if context.ordinal == ordinal
            ));
        }

        assert_eq!(graph.node_count(), node_count);
        assert_eq!(graph.edge_count(), edge_count);
        assert_eq!(
            graph
                .edges()
                .filter(|edge| edge.kind() == EdgeKind::Writes)
                .count(),
            2
        );
    }

    #[test]
    fn owner_resolution_uses_exact_id_and_name_and_reports_sorted_ambiguity() {
        let owner = descriptor(
            "document-owner",
            "DocumentOwner",
            MetadataKind::Document,
            "z/DocumentOwner.mdo",
        );
        let same_identity = descriptor(
            "document-owner",
            "DocumentOwner",
            MetadataKind::Document,
            "a/DocumentOwner.mdo",
        );
        let incompatible_kind = descriptor(
            "document-owner",
            "DocumentOwner",
            MetadataKind::Catalog,
            "Catalog.mdo",
        );
        let exact_candidate = candidate(&owner, "Stock");
        let mut wrong_id = exact_candidate.clone();
        wrong_id.owner_id = id("different-owner");
        let mut wrong_name = exact_candidate.clone();
        wrong_name.owner_name = name("DifferentOwner");
        let graph = SemanticGraph::new();

        let unique = EdtWritesResolutionIndex::new(&[incompatible_kind, owner.clone()], &graph)
            .resolve(
                std::slice::from_ref(&exact_candidate),
                WorkspaceResolutionScope::Complete,
            );
        let missing = EdtWritesResolutionIndex::new(std::slice::from_ref(&owner), &graph)
            .resolve(&[wrong_id, wrong_name], WorkspaceResolutionScope::Complete);
        let ambiguous = EdtWritesResolutionIndex::new(&[owner, same_identity], &graph).resolve(
            std::slice::from_ref(&exact_candidate),
            WorkspaceResolutionScope::Complete,
        );

        assert_eq!(unique, vec![EdtWritesResolutionOutcome::MissingDeclaration]);
        assert_eq!(
            missing,
            vec![
                EdtWritesResolutionOutcome::MissingOwner,
                EdtWritesResolutionOutcome::MissingOwner,
            ]
        );
        assert_eq!(
            ambiguous,
            vec![EdtWritesResolutionOutcome::AmbiguousOwner {
                descriptor_paths: vec![
                    PathBuf::from("a/DocumentOwner.mdo"),
                    PathBuf::from("z/DocumentOwner.mdo"),
                ],
            }]
        );
    }

    #[test]
    fn declaration_resolution_distinguishes_missing_unsupported_and_malformed() {
        let missing_owner = generated_document("");
        let unsupported_owner = generated_document(
            r"    <registerRecords>InformationRegister.Stock</registerRecords>
    <registerRecords>LocalizedRegister.STOCK</registerRecords>",
        );
        let malformed_owner = generated_document("    <registerRecords>Stock</registerRecords>");
        let graph = SemanticGraph::new();

        assert_eq!(
            resolve_one(
                &missing_owner,
                &graph,
                &candidate(&missing_owner, "Stock"),
                WorkspaceResolutionScope::Complete,
            ),
            EdtWritesResolutionOutcome::MissingDeclaration
        );
        let unsupported = resolve_one(
            &unsupported_owner,
            &graph,
            &candidate(&unsupported_owner, "stock"),
            WorkspaceResolutionScope::Complete,
        );
        let EdtWritesResolutionOutcome::UnsupportedDeclaration { declarations } = unsupported
        else {
            panic!("same-key unsupported declarations must remain typed");
        };
        assert_eq!(declarations.len(), 2);
        assert_eq!(
            declarations
                .iter()
                .map(|declaration| declaration.kind)
                .collect::<Vec<_>>(),
            vec![None, Some(MetadataKind::InformationRegister)]
        );
        assert_eq!(
            resolve_one(
                &malformed_owner,
                &graph,
                &candidate(&malformed_owner, "Stock"),
                WorkspaceResolutionScope::Complete,
            ),
            EdtWritesResolutionOutcome::MissingDeclaration
        );
    }

    #[test]
    fn unique_compatible_declaration_wins_over_same_key_unsupported_declarations() {
        let owner = generated_document(
            r"    <registerRecords>InformationRegister.STOCK</registerRecords>
    <registerRecords>AccumulationRegister.Stock</registerRecords>
    <registerRecords>LocalizedRegister.stock</registerRecords>",
        );
        let mut graph = SemanticGraph::new();
        accumulation_node(&mut graph, "accumulation.stock", "Stock");

        let outcome = resolve_one(
            &owner,
            &graph,
            &candidate(&owner, "stock"),
            WorkspaceResolutionScope::Complete,
        );
        let EdtWritesResolutionOutcome::Resolved {
            declaration,
            target_id,
        } = outcome
        else {
            panic!("unique compatible declaration must resolve");
        };

        assert_eq!(declaration.raw_value, "AccumulationRegister.Stock");
        assert_eq!(declaration.kind, Some(MetadataKind::AccumulationRegister));
        assert_eq!(target_id, id("accumulation.stock"));
    }

    #[test]
    fn exact_duplicate_declaration_resolves_once_and_preserves_duplicate_provenance() {
        let owner = generated_document(
            r"    <registerRecords>AccumulationRegister.Stock</registerRecords>
    <registerRecords>AccumulationRegister.Stock</registerRecords>
    <registerRecords>AccumulationRegister.Stock</registerRecords>",
        );
        let mut graph = SemanticGraph::new();
        accumulation_node(&mut graph, "accumulation.stock", "Stock");

        let outcome = resolve_one(
            &owner,
            &graph,
            &candidate(&owner, "Stock"),
            WorkspaceResolutionScope::Complete,
        );
        let EdtWritesResolutionOutcome::Resolved { declaration, .. } = outcome else {
            panic!("exact duplicate declaration must resolve once");
        };
        let EdtDocumentRegisterDeclarationProvenance::Duplicate(contexts) = declaration.provenance
        else {
            panic!("duplicate declaration provenance must be retained");
        };

        assert_eq!(
            contexts
                .iter()
                .map(|context| context.ordinal)
                .collect::<Vec<_>>(),
            vec![1, 2, 3]
        );
    }

    #[test]
    fn normalized_compatible_declaration_collision_is_ambiguous_before_target_lookup() {
        let owner = generated_document(
            r"    <registerRecords>AccumulationRegister.Stock</registerRecords>
    <registerRecords>AccumulationRegister.STOCK</registerRecords>",
        );
        let graph = SemanticGraph::new();

        let outcome = resolve_one(
            &owner,
            &graph,
            &candidate(&owner, "stock"),
            WorkspaceResolutionScope::Complete,
        );
        let EdtWritesResolutionOutcome::AmbiguousDeclaration { declarations } = outcome else {
            panic!("normalized compatible collision must be ambiguous");
        };

        assert_eq!(
            declarations
                .iter()
                .map(|declaration| declaration.raw_value.as_str())
                .collect::<Vec<_>>(),
            vec!["AccumulationRegister.STOCK", "AccumulationRegister.Stock",]
        );
    }

    #[test]
    fn target_resolution_distinguishes_scope_kind_unique_and_ambiguous_states() {
        let owner =
            generated_document("    <registerRecords>AccumulationRegister.Stock</registerRecords>");
        let writes_candidate = candidate(&owner, "Stock");
        let empty_graph = SemanticGraph::new();

        assert_eq!(
            resolve_one(
                &owner,
                &empty_graph,
                &writes_candidate,
                WorkspaceResolutionScope::Complete,
            ),
            EdtWritesResolutionOutcome::MissingTarget
        );
        assert_eq!(
            resolve_one(
                &owner,
                &empty_graph,
                &writes_candidate,
                WorkspaceResolutionScope::Partial,
            ),
            EdtWritesResolutionOutcome::PartialWorkspaceTargetAbsent
        );

        let mut incompatible = SemanticGraph::new();
        insert_node(
            &mut incompatible,
            "unknown.stock",
            "STOCK",
            NodeKind::Unknown,
        );
        insert_node(
            &mut incompatible,
            "catalog.stock",
            "Stock",
            NodeKind::Metadata(MetadataKind::Catalog),
        );
        assert_eq!(
            resolve_one(
                &owner,
                &incompatible,
                &writes_candidate,
                WorkspaceResolutionScope::Complete,
            ),
            EdtWritesResolutionOutcome::IncompatibleTargetKind {
                candidates: vec![id("catalog.stock"), id("unknown.stock")],
            }
        );

        accumulation_node(&mut incompatible, "accumulation.stock", "stock");
        let unique = resolve_one(
            &owner,
            &incompatible,
            &writes_candidate,
            WorkspaceResolutionScope::Complete,
        );
        assert!(matches!(
            unique,
            EdtWritesResolutionOutcome::Resolved { target_id, .. }
                if target_id == id("accumulation.stock")
        ));

        accumulation_node(&mut incompatible, "accumulation.z", "Stock");
        accumulation_node(&mut incompatible, "accumulation.a", "STOCK");
        assert_eq!(
            resolve_one(
                &owner,
                &incompatible,
                &writes_candidate,
                WorkspaceResolutionScope::Complete,
            ),
            EdtWritesResolutionOutcome::AmbiguousTarget {
                candidates: vec![
                    id("accumulation.a"),
                    id("accumulation.stock"),
                    id("accumulation.z"),
                ],
            }
        );
    }

    #[test]
    fn lookup_uses_unicode_lowercase_without_normalization() {
        let lowercase_owner = generated_document(
            "    <registerRecords>AccumulationRegister.i\u{307}tem</registerRecords>",
        );
        let mut lowercase_graph = SemanticGraph::new();
        accumulation_node(&mut lowercase_graph, "accumulation.expanded", "i\u{307}tem");
        let lowercase = resolve_one(
            &lowercase_owner,
            &lowercase_graph,
            &candidate(&lowercase_owner, "İTEM"),
            WorkspaceResolutionScope::Complete,
        );
        assert!(matches!(
            lowercase,
            EdtWritesResolutionOutcome::Resolved { target_id, .. }
                if target_id == id("accumulation.expanded")
        ));

        let decomposed_owner = generated_document(
            "    <registerRecords>AccumulationRegister.Cafe\u{301}</registerRecords>",
        );
        let mut decomposed_graph = SemanticGraph::new();
        accumulation_node(
            &mut decomposed_graph,
            "accumulation.decomposed",
            "Cafe\u{301}",
        );
        assert_eq!(
            resolve_one(
                &decomposed_owner,
                &decomposed_graph,
                &candidate(&decomposed_owner, "Café"),
                WorkspaceResolutionScope::Complete,
            ),
            EdtWritesResolutionOutcome::MissingDeclaration
        );
    }

    #[test]
    fn candidate_owner_target_and_repeated_run_order_are_deterministic() {
        let owner = generated_document(
            r"    <registerRecords>AccumulationRegister.Alpha</registerRecords>
    <registerRecords>AccumulationRegister.Beta</registerRecords>",
        );
        let unrelated = descriptor(
            "unrelated-document",
            "UnrelatedDocument",
            MetadataKind::Document,
            "UnrelatedDocument.mdo",
        );
        let candidates = vec![
            candidate(&owner, "Beta"),
            candidate(&owner, "Alpha"),
            candidate(&owner, "Beta"),
        ];
        let mut normal = SemanticGraph::new();
        accumulation_node(&mut normal, "target.beta", "BETA");
        accumulation_node(&mut normal, "target.alpha", "alpha");
        let mut reversed = SemanticGraph::new();
        accumulation_node(&mut reversed, "target.alpha", "alpha");
        accumulation_node(&mut reversed, "target.beta", "BETA");

        let normal_index =
            EdtWritesResolutionIndex::new(&[unrelated.clone(), owner.clone()], &normal);
        let reversed_index = EdtWritesResolutionIndex::new(&[owner, unrelated], &reversed);
        let first = normal_index.resolve(&candidates, WorkspaceResolutionScope::Complete);
        let repeated = normal_index.resolve(&candidates, WorkspaceResolutionScope::Complete);
        let reordered_inputs =
            reversed_index.resolve(&candidates, WorkspaceResolutionScope::Complete);
        let mut reversed_candidates = candidates.clone();
        reversed_candidates.reverse();
        let mut reversed_outcomes =
            normal_index.resolve(&reversed_candidates, WorkspaceResolutionScope::Complete);
        reversed_outcomes.reverse();

        assert_eq!(first, repeated);
        assert_eq!(first, reordered_inputs);
        assert_eq!(first, reversed_outcomes);
        assert_eq!(first.len(), 3);
        assert!(matches!(
            &first[0],
            EdtWritesResolutionOutcome::Resolved { target_id, .. }
                if target_id == &id("target.beta")
        ));
        assert!(matches!(
            &first[1],
            EdtWritesResolutionOutcome::Resolved { target_id, .. }
                if target_id == &id("target.alpha")
        ));
        assert_eq!(first[0], first[2]);
    }

    #[test]
    fn incompatible_declaration_order_does_not_change_resolution_content() {
        let root = tempdir().expect("temporary directory must be created");
        let object_directory = root.path().join("GeneratedDocument");
        let first = r"    <registerRecords>AccumulationRegister.Stock</registerRecords>
    <registerRecords>InformationRegister.Stock</registerRecords>
    <registerRecords>LocalizedRegister.Stock</registerRecords>";
        let second = r"    <registerRecords>AccumulationRegister.Stock</registerRecords>
    <registerRecords>LocalizedRegister.Stock</registerRecords>
    <registerRecords>InformationRegister.Stock</registerRecords>";
        write_document_xml(&object_directory, first);
        let first_owner = read_document(&object_directory);
        write_document_xml(&object_directory, second);
        let second_owner = read_document(&object_directory);
        let mut graph = SemanticGraph::new();
        accumulation_node(&mut graph, "accumulation.stock", "Stock");

        let first_outcome = resolve_one(
            &first_owner,
            &graph,
            &candidate(&first_owner, "Stock"),
            WorkspaceResolutionScope::Complete,
        );
        let second_outcome = resolve_one(
            &second_owner,
            &graph,
            &candidate(&second_owner, "Stock"),
            WorkspaceResolutionScope::Complete,
        );

        assert_eq!(first_outcome, second_outcome);
    }
}
