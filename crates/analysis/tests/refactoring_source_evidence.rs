use oneagent_analysis::refactoring::{
    BslModuleRole, ConfinedSourcePath, MAX_SOURCE_BYTES_PER_CONFIGURATION,
    MAX_SOURCE_DOCUMENT_BYTES, MAX_SOURCE_DOCUMENTS_PER_CONFIGURATION, MAX_SOURCE_IDENTIFIER_BYTES,
    MAX_SOURCE_IDENTITY_BYTES, MAX_SOURCE_OCCURRENCES_PER_DOCUMENT, SourceByteRange,
    SourceContentVersion, SourceDocument, SourceDocumentId, SourceEvidenceCompleteness,
    SourceEvidenceErrorKind, SourceEvidenceSet, SourceFormat, SourceOccurrence,
    SourceOccurrenceKind, SourceOccurrenceResolution,
};
use oneagent_common::{EntityId, SourcePath};

fn id(value: impl Into<String>) -> EntityId {
    EntityId::new(value).expect("identifier must be valid")
}

fn document_id() -> SourceDocumentId {
    SourceDocumentId::new(id("configuration.main"), id("module.sales.object"))
        .expect("document identity must be valid")
}

fn confined_path(value: &str) -> ConfinedSourcePath {
    ConfinedSourcePath::new(
        SourcePath::new(value).expect("source path must be valid"),
        &SourcePath::new("configuration").expect("configuration root must be valid"),
    )
    .expect("source path must be confined")
}

fn occurrence(
    document_id: &SourceDocumentId,
    version: SourceContentVersion,
    source: &str,
    token: &str,
    kind: SourceOccurrenceKind,
    target: &str,
) -> SourceOccurrence {
    let start = source.find(token).expect("token must exist");
    SourceOccurrence::new(
        document_id.clone(),
        version,
        SourceByteRange::new(start, start + token.len()).expect("range must be valid"),
        kind,
        token,
        Some(id(target)),
        SourceOccurrenceResolution::Unique,
    )
    .expect("occurrence must be valid")
}

fn empty_document(module: &str, path: &str, raw_content: Vec<u8>) -> SourceDocument {
    SourceDocument::new(
        SourceDocumentId::new(id("configuration.main"), id(module))
            .expect("document identity must be valid"),
        SourceFormat::Edt,
        BslModuleRole::Object,
        confined_path(path),
        raw_content,
        Vec::new(),
        SourceEvidenceCompleteness::BslCallableRenameV1,
    )
    .expect("document must be valid")
}

#[test]
fn document_binds_confined_raw_version_and_canonical_occurrences() {
    let raw = b"\xef\xbb\xbf\xd0\x9f\xd1\x80\xd0\xbe\xd1\x86\xd0\xb5\xd0\xb4\xd1\x83\xd1\x80\xd0\xb0 Test()\r\n    Test();\r\n\xd0\x9a\xd0\xbe\xd0\xbd\xd0\xb5\xd1\x86\xd0\x9f\xd1\x80\xd0\xbe\xd1\x86\xd0\xb5\xd0\xb4\xd1\x83\xd1\x80\xd1\x8b\r\n".to_vec();
    let source = std::str::from_utf8(&raw).expect("fixture must be UTF-8");
    let document_id = document_id();
    let version = SourceContentVersion::from_bytes(&raw);
    let declaration = occurrence(
        &document_id,
        version,
        source,
        "Test",
        SourceOccurrenceKind::Declaration,
        "module.sales.object:procedure:Test",
    );
    let call_start = source.rfind("Test").expect("call token must exist");
    let call = SourceOccurrence::new(
        document_id.clone(),
        version,
        SourceByteRange::new(call_start, call_start + 4).expect("range must be valid"),
        SourceOccurrenceKind::LocalCall,
        "Test",
        Some(id("module.sales.object:procedure:Test")),
        SourceOccurrenceResolution::Unique,
    )
    .expect("call must be valid");

    let document = SourceDocument::new(
        document_id.clone(),
        SourceFormat::Edt,
        BslModuleRole::Object,
        confined_path("configuration/Catalogs/Sales/Ext/ObjectModule.bsl"),
        raw.clone(),
        vec![call.clone(), declaration.clone(), call],
        SourceEvidenceCompleteness::BslCallableRenameV1,
    )
    .expect("document must be valid");

    assert_eq!(document.id(), &document_id);
    assert_eq!(document.raw_content(), raw);
    assert_eq!(document.content_version(), version);
    assert_eq!(document.occurrences().len(), 2);
    assert_eq!(document.occurrences()[0], declaration);
    assert_eq!(document.format(), SourceFormat::Edt);
    assert_eq!(document.module_role(), BslModuleRole::Object);
    assert_eq!(
        document.path().path().as_str(),
        "configuration/Catalogs/Sales/Ext/ObjectModule.bsl"
    );
    assert_eq!(document.clone(), document);
}

#[test]
fn content_version_preserves_length_and_all_digest_bytes_deterministically() {
    let first = SourceContentVersion::from_bytes(b"abc");
    let repeated = SourceContentVersion::from_bytes(b"abc");
    let different = SourceContentVersion::from_bytes(b"abc\n");

    assert_eq!(first, repeated);
    assert_ne!(first, different);
    assert_eq!(first.raw_byte_len(), 3);
    assert_eq!(
        first.digest(),
        [
            0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea, 0x41, 0x41, 0x40, 0xde, 0x5d, 0xae,
            0x22, 0x23, 0xb0, 0x03, 0x61, 0xa3, 0x96, 0x17, 0x7a, 0x9c, 0xb4, 0x10, 0xff, 0x61,
            0xf2, 0x00, 0x15, 0xad,
        ]
    );
}

#[test]
fn document_accepts_exact_bounds_and_rejects_one_over_without_partial_output() {
    let exact = vec![b'a'; MAX_SOURCE_DOCUMENT_BYTES];
    let accepted = SourceDocument::new(
        document_id(),
        SourceFormat::DesignerXml,
        BslModuleRole::Common,
        confined_path("configuration/CommonModules/Shared/Ext/Module.bsl"),
        exact,
        Vec::new(),
        SourceEvidenceCompleteness::BslCallableRenameV1,
    )
    .expect("exact document byte bound must pass");
    assert_eq!(accepted.raw_content().len(), MAX_SOURCE_DOCUMENT_BYTES);

    let error = SourceDocument::new(
        document_id(),
        SourceFormat::DesignerXml,
        BslModuleRole::Common,
        confined_path("configuration/CommonModules/Shared/Ext/Module.bsl"),
        vec![b'a'; MAX_SOURCE_DOCUMENT_BYTES + 1],
        Vec::new(),
        SourceEvidenceCompleteness::BslCallableRenameV1,
    )
    .expect_err("one-over document byte bound must fail");
    assert_eq!(error.kind(), SourceEvidenceErrorKind::BoundExceeded);
    assert_eq!(error.actual(), Some(MAX_SOURCE_DOCUMENT_BYTES + 1));
    assert_eq!(error.maximum(), Some(MAX_SOURCE_DOCUMENT_BYTES));
}

#[test]
fn confinement_encoding_bom_identity_and_mapping_fail_closed() {
    for (path, root) in [
        ("/configuration/Module.bsl", "configuration"),
        ("other/Module.bsl", "configuration"),
        ("configuration", "configuration"),
    ] {
        let error = ConfinedSourcePath::new(
            SourcePath::new(path).expect("source path shape must be valid"),
            &SourcePath::new(root).expect("root path shape must be valid"),
        )
        .expect_err("unconfined source path must fail");
        assert_eq!(error.kind(), SourceEvidenceErrorKind::InvalidConfinedPath);
        assert!(!error.to_string().contains(path));
    }

    let direct_root = ConfinedSourcePath::new_at_workspace_root(
        SourcePath::new("src/CommonModules/Sales/Module.bsl")
            .expect("direct-root source path shape must be valid"),
    )
    .expect("Workspace-root Configuration source must remain confined");
    assert_eq!(
        direct_root.path().as_str(),
        "src/CommonModules/Sales/Module.bsl"
    );
    let absolute_error = ConfinedSourcePath::new_at_workspace_root(
        SourcePath::new("/workspace/Module.bsl").expect("absolute source path shape must be valid"),
    )
    .expect_err("absolute direct-root source path must fail");
    assert_eq!(
        absolute_error.kind(),
        SourceEvidenceErrorKind::InvalidConfinedPath
    );

    let over_identity =
        SourceDocumentId::new(id("a".repeat(MAX_SOURCE_IDENTITY_BYTES + 1)), id("module"))
            .expect_err("one-over identity must fail");
    assert_eq!(over_identity.kind(), SourceEvidenceErrorKind::BoundExceeded);

    let invalid_mapping = SourceOccurrence::new(
        document_id(),
        SourceContentVersion::from_bytes(b"Call"),
        SourceByteRange::new(0, 4).expect("range must be valid"),
        SourceOccurrenceKind::LocalCall,
        "Call",
        None,
        SourceOccurrenceResolution::Unique,
    )
    .expect_err("unique resolution without a target must fail");
    assert_eq!(
        invalid_mapping.kind(),
        SourceEvidenceErrorKind::InvalidOccurrence
    );

    for (raw, kind) in [
        (vec![0xff], SourceEvidenceErrorKind::UnsupportedEncoding),
        (
            [UTF8_BOM_BYTES, UTF8_BOM_BYTES].concat(),
            SourceEvidenceErrorKind::MalformedBom,
        ),
    ] {
        let error = SourceDocument::new(
            document_id(),
            SourceFormat::Edt,
            BslModuleRole::Object,
            confined_path("configuration/Module.bsl"),
            raw,
            Vec::new(),
            SourceEvidenceCompleteness::BslCallableRenameV1,
        )
        .expect_err("invalid source bytes must fail");
        assert_eq!(error.kind(), kind);
    }
}

const UTF8_BOM_BYTES: &[u8] = b"\xef\xbb\xbf";

#[test]
fn identity_and_occurrence_token_byte_bounds_are_exact() {
    let exact_identity = id("i".repeat(MAX_SOURCE_IDENTITY_BYTES));
    SourceDocumentId::new(exact_identity, id("module"))
        .expect("exact identity byte bound must pass");
    let identity_error =
        SourceDocumentId::new(id("i".repeat(MAX_SOURCE_IDENTITY_BYTES + 1)), id("module"))
            .expect_err("one-over identity byte bound must fail");
    assert_eq!(
        identity_error.kind(),
        SourceEvidenceErrorKind::BoundExceeded
    );

    let exact_token = "я".repeat(MAX_SOURCE_IDENTIFIER_BYTES / "я".len());
    SourceOccurrence::new(
        document_id(),
        SourceContentVersion::from_bytes(exact_token.as_bytes()),
        SourceByteRange::new(0, exact_token.len()).expect("range must be valid"),
        SourceOccurrenceKind::Declaration,
        exact_token,
        None,
        SourceOccurrenceResolution::Unresolved,
    )
    .expect("exact identifier byte bound must pass");

    let over_token = "a".repeat(MAX_SOURCE_IDENTIFIER_BYTES + 1);
    let token_error = SourceOccurrence::new(
        document_id(),
        SourceContentVersion::from_bytes(over_token.as_bytes()),
        SourceByteRange::new(0, over_token.len()).expect("range must be valid"),
        SourceOccurrenceKind::Declaration,
        over_token,
        None,
        SourceOccurrenceResolution::Unresolved,
    )
    .expect_err("one-over identifier byte bound must fail");
    assert_eq!(token_error.kind(), SourceEvidenceErrorKind::BoundExceeded);

    let empty_error = SourceOccurrence::new(
        document_id(),
        SourceContentVersion::from_bytes(b""),
        SourceByteRange::new(0, 1).expect("range shape must be valid"),
        SourceOccurrenceKind::Declaration,
        "",
        None,
        SourceOccurrenceResolution::Unresolved,
    )
    .expect_err("empty occurrence token must fail");
    assert_eq!(
        empty_error.kind(),
        SourceEvidenceErrorKind::InvalidOccurrence
    );
}

#[test]
fn qualified_occurrence_requires_exact_bounded_lexical_owner_context() {
    let raw = b"Module.Call()".to_vec();
    let version = SourceContentVersion::from_bytes(&raw);
    let document_id = document_id();
    let range = SourceByteRange::new(7, 11).expect("qualified range must be valid");

    let missing_owner = SourceOccurrence::new(
        document_id.clone(),
        version,
        range,
        SourceOccurrenceKind::QualifiedCall,
        "Call",
        None,
        SourceOccurrenceResolution::Unresolved,
    )
    .expect_err("qualified occurrence without an owner must fail");
    assert_eq!(
        missing_owner.kind(),
        SourceEvidenceErrorKind::InvalidOccurrence
    );

    let unexpected_owner = SourceOccurrence::new_with_lexical_owner(
        document_id.clone(),
        version,
        range,
        SourceOccurrenceKind::LocalCall,
        "Call",
        Some("Module".to_owned()),
        None,
        SourceOccurrenceResolution::Unresolved,
    )
    .expect_err("unqualified occurrence with an owner must fail");
    assert_eq!(
        unexpected_owner.kind(),
        SourceEvidenceErrorKind::InvalidOccurrence
    );

    let occurrence = SourceOccurrence::new_with_lexical_owner(
        document_id.clone(),
        version,
        range,
        SourceOccurrenceKind::QualifiedCall,
        "Call",
        Some("Module".to_owned()),
        None,
        SourceOccurrenceResolution::Unresolved,
    )
    .expect("exact qualified owner must be retained");
    assert_eq!(occurrence.lexical_owner_token(), Some("Module"));

    SourceDocument::new(
        document_id.clone(),
        SourceFormat::Edt,
        BslModuleRole::Common,
        confined_path("configuration/Module.bsl"),
        raw.clone(),
        vec![occurrence],
        SourceEvidenceCompleteness::BslCallableRenameV1,
    )
    .expect("qualified owner must match captured bytes");

    let mismatched = SourceOccurrence::new_with_lexical_owner(
        document_id.clone(),
        version,
        range,
        SourceOccurrenceKind::QualifiedCall,
        "Call",
        Some("Other".to_owned()),
        None,
        SourceOccurrenceResolution::Unresolved,
    )
    .expect("mismatched owner shape must remain constructible");
    let mismatch_error = SourceDocument::new(
        document_id,
        SourceFormat::Edt,
        BslModuleRole::Common,
        confined_path("configuration/Module.bsl"),
        raw,
        vec![mismatched],
        SourceEvidenceCompleteness::BslCallableRenameV1,
    )
    .expect_err("mismatched owner bytes must fail document validation");
    assert_eq!(
        mismatch_error.kind(),
        SourceEvidenceErrorKind::InvalidOccurrence
    );
}

#[test]
fn qualified_lexical_owner_byte_bound_is_exact() {
    let version = SourceContentVersion::from_bytes(b"Call");
    let range = SourceByteRange::new(0, 4).expect("range must be valid");
    SourceOccurrence::new_with_lexical_owner(
        document_id(),
        version,
        range,
        SourceOccurrenceKind::QualifiedCall,
        "Call",
        Some("a".repeat(MAX_SOURCE_IDENTIFIER_BYTES)),
        None,
        SourceOccurrenceResolution::Unresolved,
    )
    .expect("exact lexical-owner byte bound must pass");
    let owner_error = SourceOccurrence::new_with_lexical_owner(
        document_id(),
        version,
        range,
        SourceOccurrenceKind::QualifiedCall,
        "Call",
        Some("a".repeat(MAX_SOURCE_IDENTIFIER_BYTES + 1)),
        None,
        SourceOccurrenceResolution::Unresolved,
    )
    .expect_err("one-over lexical-owner byte bound must fail");
    assert_eq!(owner_error.kind(), SourceEvidenceErrorKind::BoundExceeded);
}

#[test]
fn ranges_versions_and_tokens_are_checked() {
    let raw = "Тестaaaa".as_bytes().to_vec();
    let source = std::str::from_utf8(&raw).expect("fixture must be UTF-8");
    let document_id = document_id();
    let version = SourceContentVersion::from_bytes(&raw);

    let invalid_ranges = [
        SourceByteRange::new(1, "Тест".len()).expect("non-empty range shape must pass"),
        SourceByteRange::new(0, raw.len() + 1).expect("non-empty range shape must pass"),
    ];
    for range in invalid_ranges {
        let candidate = SourceOccurrence::new(
            document_id.clone(),
            version,
            range,
            SourceOccurrenceKind::Declaration,
            "Тест",
            Some(id("target")),
            SourceOccurrenceResolution::Unique,
        )
        .expect("occurrence shape must pass before document validation");
        let error = SourceDocument::new(
            document_id.clone(),
            SourceFormat::Edt,
            BslModuleRole::Object,
            confined_path("configuration/Module.bsl"),
            raw.clone(),
            vec![candidate],
            SourceEvidenceCompleteness::BslCallableRenameV1,
        )
        .expect_err("invalid raw range must fail");
        assert_eq!(error.kind(), SourceEvidenceErrorKind::InvalidOccurrence);
    }

    let valid = occurrence(
        &document_id,
        version,
        source,
        "Тест",
        SourceOccurrenceKind::Declaration,
        "target",
    );
    let stale = SourceOccurrence::new(
        document_id.clone(),
        SourceContentVersion::from_bytes(b"different"),
        valid.range(),
        SourceOccurrenceKind::Declaration,
        "Тест",
        Some(id("target")),
        SourceOccurrenceResolution::Unique,
    )
    .expect("stale occurrence shape must pass");
    let stale_error = SourceDocument::new(
        document_id.clone(),
        SourceFormat::Edt,
        BslModuleRole::Object,
        confined_path("configuration/Module.bsl"),
        raw.clone(),
        vec![stale],
        SourceEvidenceCompleteness::BslCallableRenameV1,
    )
    .expect_err("stale occurrence version must fail");
    assert_eq!(
        stale_error.kind(),
        SourceEvidenceErrorKind::IncompatibleEvidence
    );
}

#[test]
fn duplicate_and_overlapping_occurrences_are_rejected() {
    let raw = "Тестaaaa".as_bytes().to_vec();
    let source = std::str::from_utf8(&raw).expect("fixture must be UTF-8");
    let document_id = document_id();
    let version = SourceContentVersion::from_bytes(&raw);
    let valid = occurrence(
        &document_id,
        version,
        source,
        "Тест",
        SourceOccurrenceKind::Declaration,
        "target",
    );

    let conflicting = SourceOccurrence::new(
        document_id.clone(),
        version,
        valid.range(),
        SourceOccurrenceKind::LocalCall,
        "Тест",
        Some(id("target")),
        SourceOccurrenceResolution::Unique,
    )
    .expect("conflicting occurrence shape must pass");
    let conflict_error = SourceDocument::new(
        document_id.clone(),
        SourceFormat::Edt,
        BslModuleRole::Object,
        confined_path("configuration/Module.bsl"),
        raw.clone(),
        vec![valid, conflicting],
        SourceEvidenceCompleteness::BslCallableRenameV1,
    )
    .expect_err("same-range unequal evidence must fail");
    assert_eq!(
        conflict_error.kind(),
        SourceEvidenceErrorKind::DuplicateConflict
    );

    let overlapping_raw = b"Module.aaaaCall".to_vec();
    let overlapping_version = SourceContentVersion::from_bytes(&overlapping_raw);
    let overlapping = [
        ("aaaa", 7, 11, SourceOccurrenceKind::QualifiedCall),
        ("aCal", 10, 14, SourceOccurrenceKind::LocalCall),
    ]
    .into_iter()
    .map(|(token, start, end, kind)| {
        SourceOccurrence::new_with_lexical_owner(
            document_id.clone(),
            overlapping_version,
            SourceByteRange::new(start, end).expect("range must be valid"),
            kind,
            token,
            (kind == SourceOccurrenceKind::QualifiedCall).then(|| "Module".to_owned()),
            None,
            SourceOccurrenceResolution::Unresolved,
        )
        .expect("overlapping occurrence shape must pass")
    })
    .collect();
    let overlap_error = SourceDocument::new(
        document_id,
        SourceFormat::Edt,
        BslModuleRole::Object,
        confined_path("configuration/Module.bsl"),
        overlapping_raw,
        overlapping,
        SourceEvidenceCompleteness::BslCallableRenameV1,
    )
    .expect_err("overlapping evidence must fail");
    assert_eq!(
        overlap_error.kind(),
        SourceEvidenceErrorKind::OverlappingOccurrences
    );
}

#[test]
fn occurrence_input_bound_is_checked_before_exact_duplicate_collapse() {
    let raw = b"Call".to_vec();
    let document_id = document_id();
    let version = SourceContentVersion::from_bytes(&raw);
    let candidate = SourceOccurrence::new(
        document_id.clone(),
        version,
        SourceByteRange::new(0, 4).expect("range must be valid"),
        SourceOccurrenceKind::LocalCall,
        "Call",
        None,
        SourceOccurrenceResolution::Unresolved,
    )
    .expect("occurrence must be valid");

    let accepted = SourceDocument::new(
        document_id.clone(),
        SourceFormat::Edt,
        BslModuleRole::Object,
        confined_path("configuration/Module.bsl"),
        raw.clone(),
        vec![candidate.clone(); MAX_SOURCE_OCCURRENCES_PER_DOCUMENT],
        SourceEvidenceCompleteness::BslCallableRenameV1,
    )
    .expect("exact occurrence input bound must pass");
    assert_eq!(accepted.occurrences().len(), 1);

    let error = SourceDocument::new(
        document_id,
        SourceFormat::Edt,
        BslModuleRole::Object,
        confined_path("configuration/Module.bsl"),
        raw,
        vec![candidate; MAX_SOURCE_OCCURRENCES_PER_DOCUMENT + 1],
        SourceEvidenceCompleteness::BslCallableRenameV1,
    )
    .expect_err("one-over occurrence input bound must fail");
    assert_eq!(error.kind(), SourceEvidenceErrorKind::BoundExceeded);
}

#[test]
fn source_evidence_set_is_canonical_and_rejects_duplicate_ids_paths_and_formats() {
    let first = empty_document(
        "module.a",
        "configuration/Modules/A/Module.bsl",
        b"A".to_vec(),
    );
    let second = empty_document(
        "module.b",
        "configuration/Modules/B/Module.bsl",
        b"BB".to_vec(),
    );
    let set = SourceEvidenceSet::new(
        id("configuration.main"),
        vec![second.clone(), first.clone()],
    )
    .expect("source evidence set must be valid");
    assert_eq!(set.documents()[0].id(), first.id());
    assert_eq!(set.documents()[1].id(), second.id());
    assert_eq!(set.total_raw_bytes(), 3);

    let duplicate_id =
        SourceEvidenceSet::new(id("configuration.main"), vec![first.clone(), first.clone()])
            .expect_err("duplicate document identity must fail");
    assert_eq!(
        duplicate_id.kind(),
        SourceEvidenceErrorKind::DuplicateConflict
    );

    let aliased = SourceDocument::new(
        SourceDocumentId::new(id("configuration.main"), id("module.c"))
            .expect("document identity must be valid"),
        SourceFormat::Edt,
        BslModuleRole::Object,
        first.path().clone(),
        b"C".to_vec(),
        Vec::new(),
        SourceEvidenceCompleteness::BslCallableRenameV1,
    )
    .expect("aliased document shape must be valid");
    let duplicate_path = SourceEvidenceSet::new(id("configuration.main"), vec![first, aliased])
        .expect_err("path alias must fail");
    assert_eq!(
        duplicate_path.kind(),
        SourceEvidenceErrorKind::DuplicateConflict
    );

    let unsupported = SourceDocument::new(
        document_id(),
        SourceFormat::DesignerXml,
        BslModuleRole::Form,
        confined_path("configuration/Forms/Main/Ext/Form/Module.bsl"),
        Vec::new(),
        Vec::new(),
        SourceEvidenceCompleteness::BslCallableRenameV1,
    )
    .expect_err("unsupported Designer module role must fail");
    assert_eq!(
        unsupported.kind(),
        SourceEvidenceErrorKind::UnsupportedSourceFormat
    );
}

#[test]
fn source_evidence_set_accepts_exact_document_and_aggregate_bounds_and_rejects_one_over() {
    let exact_documents = (0..MAX_SOURCE_DOCUMENTS_PER_CONFIGURATION)
        .map(|index| {
            empty_document(
                &format!("module.{index:04}"),
                &format!("configuration/Modules/{index:04}/Module.bsl"),
                Vec::new(),
            )
        })
        .collect::<Vec<_>>();
    let exact_count = SourceEvidenceSet::new(id("configuration.main"), exact_documents.clone())
        .expect("exact document count must pass");
    assert_eq!(
        exact_count.documents().len(),
        MAX_SOURCE_DOCUMENTS_PER_CONFIGURATION
    );

    let mut one_over_documents = exact_documents;
    one_over_documents.push(empty_document(
        "module.over",
        "configuration/Modules/over/Module.bsl",
        Vec::new(),
    ));
    let count_error = SourceEvidenceSet::new(id("configuration.main"), one_over_documents)
        .expect_err("one-over document count must fail");
    assert_eq!(count_error.kind(), SourceEvidenceErrorKind::BoundExceeded);

    let exact_aggregate = (0..64)
        .map(|index| {
            empty_document(
                &format!("module.aggregate.{index:02}"),
                &format!("configuration/Aggregate/{index:02}/Module.bsl"),
                vec![b'a'; MAX_SOURCE_DOCUMENT_BYTES],
            )
        })
        .collect::<Vec<_>>();
    let exact_bytes = SourceEvidenceSet::new(id("configuration.main"), exact_aggregate.clone())
        .expect("exact aggregate byte bound must pass");
    assert_eq!(
        exact_bytes.total_raw_bytes(),
        MAX_SOURCE_BYTES_PER_CONFIGURATION
    );

    let mut one_over_aggregate = exact_aggregate;
    one_over_aggregate.push(empty_document(
        "module.aggregate.over",
        "configuration/Aggregate/over/Module.bsl",
        vec![b'a'],
    ));
    let aggregate_error = SourceEvidenceSet::new(id("configuration.main"), one_over_aggregate)
        .expect_err("one-over aggregate byte bound must fail");
    assert_eq!(
        aggregate_error.kind(),
        SourceEvidenceErrorKind::BoundExceeded
    );
    assert_eq!(
        aggregate_error.actual(),
        Some(MAX_SOURCE_BYTES_PER_CONFIGURATION + 1)
    );
}
