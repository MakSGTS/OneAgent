use oneagent_analysis::refactoring::{
    MAX_SOURCE_DOCUMENT_BYTES, SourceEvidenceCompleteness, SourceEvidenceErrorKind,
    SourceOccurrence, SourceOccurrenceKind, SourceOccurrenceResolution,
};
use oneagent_edt::{
    EdtGraphError, EdtModuleError, EdtSemanticGraphBuilder, EdtSourceEvidenceError,
    FileSystemEdtSemanticGraphBuilder,
};
use oneagent_graph::EdgeKind;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

fn paired_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../designer-xml/tests/fixtures/sprint14_conformance")
        .canonicalize()
        .expect("paired fixture root must be canonical")
}

fn project_root() -> PathBuf {
    paired_root().join("edt")
}

#[test]
fn production_builder_captures_exact_complete_deterministic_occurrences() {
    let direct_root = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_source_evidence(&project_root(), &project_root())
        .expect("EDT Configuration at the Workspace root must build");
    assert_eq!(
        direct_root.source_evidence().documents()[0]
            .path()
            .path()
            .as_str(),
        "src/CommonModules/DynamicSecurityOverridable/Module.bsl"
    );
    let first = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_source_evidence(&paired_root(), &project_root())
        .expect("paired EDT source evidence must build");
    let repeated = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_source_evidence(&paired_root(), &project_root())
        .expect("repeated EDT source evidence must build");

    assert_eq!(first.source_evidence(), repeated.source_evidence());
    assert!(first.graph().diff(repeated.graph()).is_empty());
    assert_eq!(first.source_evidence().documents().len(), 1);
    let document = &first.source_evidence().documents()[0];
    assert_eq!(
        document.completeness(),
        SourceEvidenceCompleteness::BslCallableRenameV1
    );
    assert_eq!(
        document.content_version().raw_byte_len(),
        document.raw_content().len()
    );
    assert_eq!(document.occurrences().len(), 4);
    assert_eq!(
        document
            .occurrences()
            .iter()
            .map(SourceOccurrence::kind)
            .collect::<Vec<_>>(),
        [
            SourceOccurrenceKind::Declaration,
            SourceOccurrenceKind::Declaration,
            SourceOccurrenceKind::LocalCall,
            SourceOccurrenceKind::QualifiedCall,
        ]
    );
    for occurrence in document.occurrences() {
        assert_eq!(occurrence.resolution(), SourceOccurrenceResolution::Unique);
        assert!(occurrence.mapped_target_id().is_some());
        assert_eq!(
            std::str::from_utf8(
                &document.raw_content()
                    [occurrence.range().start_byte()..occurrence.range().end_byte()]
            )
            .expect("occurrence bytes must be UTF-8"),
            occurrence.token()
        );
        assert_eq!(
            occurrence.lexical_owner_token(),
            (occurrence.kind() == SourceOccurrenceKind::QualifiedCall)
                .then_some("DynamicSecurityOverridable")
        );
    }
    assert_eq!(
        first
            .graph()
            .edges()
            .filter(|edge| edge.kind() == EdgeKind::Calls)
            .count(),
        1,
        "equal local and qualified self calls preserve one canonical Graph edge"
    );
}

#[test]
fn controlled_source_change_changes_only_retained_version_and_graph_provenance() {
    let temporary = tempdir().expect("temporary workspace must be created");
    let project = temporary.path().join("edt");
    copy_tree(&project_root(), &project);
    let baseline = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_source_evidence(temporary.path(), &project)
        .expect("baseline must build");
    let retained = baseline.source_evidence().documents()[0]
        .raw_content()
        .to_vec();
    let module = project.join("src/CommonModules/DynamicSecurityOverridable/Module.bsl");
    let source = fs::read_to_string(&module).expect("module must be readable");
    fs::write(
        &module,
        source.replacen("FillSecurityCollection", "CollectSecurity", 3),
    )
    .expect("controlled source must be written");
    let changed = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_source_evidence(temporary.path(), &project)
        .expect("changed source must build");

    assert_ne!(
        baseline.source_evidence().documents()[0].content_version(),
        changed.source_evidence().documents()[0].content_version()
    );
    assert_eq!(
        baseline.source_evidence().documents()[0]
            .occurrences()
            .len(),
        4
    );
    assert_eq!(
        changed.source_evidence().documents()[0].occurrences().len(),
        4
    );
    assert_eq!(
        baseline.source_evidence().documents()[0].raw_content(),
        retained
    );
}

#[test]
fn production_capture_rejects_non_utf8_symlink_and_one_over_bound_atomically() {
    let temporary = tempdir().expect("temporary workspace must be created");
    let project = temporary.path().join("edt");
    copy_tree(&project_root(), &project);
    let module = project.join("src/CommonModules/DynamicSecurityOverridable/Module.bsl");

    fs::write(&module, [0xff]).expect("invalid source must be written");
    assert!(matches!(
        FileSystemEdtSemanticGraphBuilder
            .build_graph_with_source_evidence(temporary.path(), &project),
        Err(EdtGraphError::Module(EdtModuleError::ReadFile { source, .. }))
            if source.kind() == std::io::ErrorKind::InvalidData
    ));

    fs::write(
        &module,
        b"\xef\xbb\xbf\xef\xbb\xbfProcedure Test()\nEndProcedure\n",
    )
    .expect("malformed BOM source must be written");
    assert!(matches!(
        FileSystemEdtSemanticGraphBuilder.build_graph_with_source_evidence(temporary.path(), &project),
        Err(EdtGraphError::SourceEvidence(EdtSourceEvidenceError::Domain(error)))
            if error.kind() == SourceEvidenceErrorKind::MalformedBom
    ));

    fs::write(&module, vec![b' '; MAX_SOURCE_DOCUMENT_BYTES])
        .expect("exact-bound source must be written");
    let exact = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_source_evidence(temporary.path(), &project)
        .expect("exact source byte bound must be accepted");
    assert_eq!(
        exact.source_evidence().documents()[0].raw_content().len(),
        MAX_SOURCE_DOCUMENT_BYTES
    );

    fs::write(&module, vec![b' '; MAX_SOURCE_DOCUMENT_BYTES + 1])
        .expect("over-bound source must be written");
    assert!(matches!(
        FileSystemEdtSemanticGraphBuilder
            .build_graph_with_source_evidence(temporary.path(), &project),
        Err(EdtGraphError::Module(
            EdtModuleError::SourceBoundExceeded { .. }
        ))
    ));

    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        fs::remove_file(&module).expect("module must be removed");
        let target = project.join("outside.bsl");
        fs::write(&target, "Procedure Test()\nEndProcedure\n").expect("target must be written");
        symlink(&target, &module).expect("module symlink must be created");
        assert!(matches!(
            FileSystemEdtSemanticGraphBuilder
                .build_graph_with_source_evidence(temporary.path(), &project),
            Err(EdtGraphError::Module(EdtModuleError::SymlinkArtifact(_)))
        ));
    }
}

#[test]
fn complete_ledger_retains_ambiguous_unresolved_and_unsupported_candidates() {
    let temporary = tempdir().expect("temporary workspace must be created");
    let project = temporary.path().join("edt");
    copy_tree(&project_root(), &project);
    fs::write(
        project.join("src/CommonModules/DynamicSecurityOverridable/Module.bsl"),
        "Procedure Target() Export\nEndProcedure\nProcedure target() Export\nEndProcedure\nProcedure Caller()\nTarget(); Missing(); A.B.C();\nEndProcedure\n",
    )
    .expect("controlled module must be written");
    let result = FileSystemEdtSemanticGraphBuilder
        .build_graph_with_source_evidence(temporary.path(), &project)
        .expect("complete non-unique ledger must build without guessed mappings");
    let outcomes = result.source_evidence().documents()[0]
        .occurrences()
        .iter()
        .filter(|occurrence| occurrence.kind() != SourceOccurrenceKind::Declaration)
        .map(|occurrence| (occurrence.resolution(), occurrence.lexical_owner_token()))
        .collect::<Vec<_>>();
    assert_eq!(
        outcomes,
        [
            (SourceOccurrenceResolution::Ambiguous, None),
            (SourceOccurrenceResolution::Unresolved, None),
            (SourceOccurrenceResolution::Unsupported, Some("B")),
        ]
    );
}

fn copy_tree(source: &Path, target: &Path) {
    fs::create_dir_all(target).expect("target directory must be created");
    for entry in fs::read_dir(source).expect("source directory must be readable") {
        let entry = entry.expect("source entry must be readable");
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());
        if entry
            .file_type()
            .expect("entry type must be readable")
            .is_dir()
        {
            copy_tree(&source_path, &target_path);
        } else {
            fs::copy(&source_path, &target_path).expect("fixture file must be copied");
        }
    }
}
