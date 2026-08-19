use oneagent_common::{EntityId, EntityName};
use oneagent_edt::{
    EdtMetadataChildDescriptor, EdtMetadataChildKind, EdtMetadataObjectDescriptor,
    EdtMetadataObjectReader, EdtModuleError, EdtModuleKind, EdtModuleLayoutOutcomeKind,
    EdtModuleLayoutRejectionReason, EdtModuleOwnerKind, EdtModuleReader,
    FileSystemEdtMetadataObjectReader, FileSystemEdtModuleReader,
};
use oneagent_metadata::MetadataKind;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

fn id(value: &str) -> EntityId {
    EntityId::new(value).expect("identifier must be valid")
}

fn name(value: &str) -> EntityName {
    EntityName::new(value).expect("name must be valid")
}

fn object(
    directory: &Path,
    value: &str,
    object_name: &str,
    kind: MetadataKind,
) -> EdtMetadataObjectDescriptor {
    EdtMetadataObjectDescriptor::new(
        id(value),
        name(object_name),
        None,
        kind,
        None,
        directory.join(format!("{object_name}.mdo")),
    )
}

fn child(
    value: &str,
    child_name: &str,
    kind: EdtMetadataChildKind,
    parent: &EntityId,
) -> EdtMetadataChildDescriptor {
    EdtMetadataChildDescriptor::new(
        id(value),
        name(child_name),
        kind,
        parent.clone(),
        Vec::new(),
    )
}

fn write_module(path: &Path) {
    fs::create_dir_all(path.parent().expect("module parent must exist"))
        .expect("module directory must be created");
    fs::write(path, "Procedure Test()\nEndProcedure").expect("module source must be created");
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root must exist")
}

#[test]
fn module_reader_accepts_exact_subordinate_and_common_command_layouts() {
    let temporary = tempdir().expect("temporary directory must be created");
    let object_directory = temporary.path().join("Sales");
    let owner = object(
        &object_directory,
        "metadata.sales",
        "Sales",
        MetadataKind::Catalog,
    );
    let children = vec![
        child(
            "form.item",
            "ItemForm",
            EdtMetadataChildKind::Form,
            owner.id(),
        ),
        child(
            "command.open",
            "Open",
            EdtMetadataChildKind::Command,
            owner.id(),
        ),
    ];
    write_module(&object_directory.join("Forms/ItemForm/Module.bsl"));
    write_module(&object_directory.join("Commands/Open/CommandModule.bsl"));

    let observations = FileSystemEdtModuleReader
        .read_form_command_modules(&owner, &children, &object_directory)
        .expect("accepted layouts must parse");
    let modules = observations
        .iter()
        .filter_map(|observation| observation.module())
        .collect::<Vec<_>>();

    assert_eq!(modules.len(), 2);
    assert_eq!(modules[0].id().as_str(), "command.open:command_module");
    assert_eq!(modules[0].name().as_str(), "CommandModule");
    assert_eq!(modules[0].kind(), EdtModuleKind::Command);
    assert_eq!(modules[1].id().as_str(), "form.item:form_module");
    assert_eq!(modules[1].name().as_str(), "FormModule");
    assert_eq!(modules[1].kind(), EdtModuleKind::Form);
    assert!(observations.iter().all(|observation| {
        observation.outcome() == EdtModuleLayoutOutcomeKind::Accepted
            && observation.owner_id().is_some()
            && observation.owner_name().is_some()
    }));

    let common_directory = temporary.path().join("GlobalOpen");
    let common = object(
        &common_directory,
        "common.command.global_open",
        "GlobalOpen",
        MetadataKind::Command,
    );
    write_module(&common_directory.join("CommandModule.bsl"));
    let common_observations = FileSystemEdtModuleReader
        .read_form_command_modules(&common, &[], &common_directory)
        .expect("Common Command layout must parse");

    assert_eq!(common_observations.len(), 1);
    assert_eq!(
        common_observations[0].owner_kind(),
        Some(EdtModuleOwnerKind::CommonCommand)
    );
    assert_eq!(
        common_observations[0]
            .module()
            .expect("Common Command module must exist")
            .id()
            .as_str(),
        "common.command.global_open:command_module"
    );
}

#[test]
fn module_reader_returns_deterministic_missing_and_rejected_layout_outcomes() {
    let temporary = tempdir().expect("temporary directory must be created");
    let object_directory = temporary.path().join("Sales");
    let owner = object(
        &object_directory,
        "metadata.sales",
        "Sales",
        MetadataKind::Catalog,
    );
    let children = vec![
        child(
            "form.missing",
            "Missing",
            EdtMetadataChildKind::Form,
            owner.id(),
        ),
        child(
            "form.case",
            "CaseForm",
            EdtMetadataChildKind::Form,
            owner.id(),
        ),
        child(
            "form.unsupported",
            "Unsupported",
            EdtMetadataChildKind::Form,
            owner.id(),
        ),
        child(
            "command.wrong",
            "WrongKind",
            EdtMetadataChildKind::Command,
            owner.id(),
        ),
        child(
            "command.dup.1",
            "Duplicate",
            EdtMetadataChildKind::Command,
            owner.id(),
        ),
        child(
            "command.dup.2",
            "Duplicate",
            EdtMetadataChildKind::Command,
            owner.id(),
        ),
    ];
    write_module(&object_directory.join("Forms/caseform/Module.bsl"));
    write_module(&object_directory.join("Forms/Unsupported/CommandModule.bsl"));
    write_module(&object_directory.join("Forms/WrongKind/Module.bsl"));
    write_module(&object_directory.join("Forms/Orphan/Module.bsl"));
    write_module(&object_directory.join("Commands/Duplicate/CommandModule.bsl"));

    let reader = FileSystemEdtModuleReader;
    let first = reader
        .read_form_command_modules(&owner, &children, &object_directory)
        .expect("typed negative layouts must parse");
    let mut reversed_children = children.clone();
    reversed_children.reverse();
    let reversed = reader
        .read_form_command_modules(&owner, &reversed_children, &object_directory)
        .expect("reordered layouts must parse");
    let repeated = reader
        .read_form_command_modules(&owner, &children, &object_directory)
        .expect("repeated layouts must parse");
    let outcomes = first
        .iter()
        .map(oneagent_edt::EdtModuleLayoutObservation::outcome)
        .collect::<BTreeSet<_>>();

    assert_eq!(first, reversed);
    assert_eq!(first, repeated);
    assert!(
        first
            .iter()
            .all(|observation| observation.module().is_none())
    );
    assert!(outcomes.contains(&EdtModuleLayoutOutcomeKind::Missing));
    for reason in [
        EdtModuleLayoutRejectionReason::OrphanDirectory,
        EdtModuleLayoutRejectionReason::NameMismatch,
        EdtModuleLayoutRejectionReason::DuplicateOwner,
        EdtModuleLayoutRejectionReason::WrongOwnerKind,
        EdtModuleLayoutRejectionReason::UnsupportedLayout,
    ] {
        assert!(outcomes.contains(&EdtModuleLayoutOutcomeKind::Rejected(reason)));
    }
}

#[test]
fn module_reader_reports_unreadable_utf8_without_guessing_a_descriptor() {
    let temporary = tempdir().expect("temporary directory must be created");
    let object_directory = temporary.path().join("Sales");
    let owner = object(
        &object_directory,
        "metadata.sales",
        "Sales",
        MetadataKind::Catalog,
    );
    let child = child(
        "form.item",
        "ItemForm",
        EdtMetadataChildKind::Form,
        owner.id(),
    );
    let module_path = object_directory.join("Forms/ItemForm/Module.bsl");
    fs::create_dir_all(module_path.parent().expect("module parent must exist"))
        .expect("module directory must be created");
    fs::write(&module_path, [0xff, 0xfe]).expect("invalid UTF-8 source must be created");

    let error = FileSystemEdtModuleReader
        .read_form_command_modules(&owner, &[child], &object_directory)
        .expect_err("invalid UTF-8 must remain a typed read error");

    match error {
        EdtModuleError::ReadFile { path, source } => {
            assert_eq!(path, module_path);
            assert_eq!(source.kind(), std::io::ErrorKind::InvalidData);
        }
        other => panic!("unexpected error: {other}"),
    }
}

#[test]
fn module_reader_types_missing_mismatched_and_wrong_kind_common_command_layouts() {
    let temporary = tempdir().expect("temporary directory must be created");
    let reader = FileSystemEdtModuleReader;

    let missing_directory = temporary.path().join("MissingCommand");
    let missing = object(
        &missing_directory,
        "common.command.missing",
        "MissingCommand",
        MetadataKind::Command,
    );
    let missing_outcomes = reader
        .read_form_command_modules(&missing, &[], &missing_directory)
        .expect("missing Common Command module must be typed");
    assert_eq!(missing_outcomes.len(), 1);
    assert_eq!(
        missing_outcomes[0].outcome(),
        EdtModuleLayoutOutcomeKind::Missing
    );
    assert_eq!(missing_outcomes[0].owner_id(), Some(missing.id()));

    let mismatched_directory = temporary.path().join("ActualName");
    let mismatched = object(
        &mismatched_directory,
        "common.command.expected",
        "ExpectedName",
        MetadataKind::Command,
    );
    write_module(&mismatched_directory.join("CommandModule.bsl"));
    let mismatched_outcomes = reader
        .read_form_command_modules(&mismatched, &[], &mismatched_directory)
        .expect("mismatched Common Command directory must be typed");
    assert_eq!(
        mismatched_outcomes[0].outcome(),
        EdtModuleLayoutOutcomeKind::Rejected(EdtModuleLayoutRejectionReason::NameMismatch)
    );
    assert!(mismatched_outcomes[0].owner_id().is_none());

    let wrong_kind_directory = temporary.path().join("CatalogOwner");
    let wrong_kind = object(
        &wrong_kind_directory,
        "metadata.catalog.owner",
        "CatalogOwner",
        MetadataKind::Catalog,
    );
    write_module(&wrong_kind_directory.join("CommandModule.bsl"));
    let wrong_kind_outcomes = reader
        .read_form_command_modules(&wrong_kind, &[], &wrong_kind_directory)
        .expect("wrong top-level owner kind must be typed");
    assert_eq!(
        wrong_kind_outcomes[0].outcome(),
        EdtModuleLayoutOutcomeKind::Rejected(EdtModuleLayoutRejectionReason::WrongOwnerKind)
    );
    assert!(wrong_kind_outcomes[0].module().is_none());
}

#[test]
fn module_reader_owner_scoped_identity_prevents_equal_name_collisions() {
    let temporary = tempdir().expect("temporary directory must be created");
    let reader = FileSystemEdtModuleReader;
    let mut module_ids = Vec::new();

    for (owner_id, directory_name, form_id) in [
        ("metadata.first", "First", "form.first.item"),
        ("metadata.second", "Second", "form.second.item"),
    ] {
        let directory = temporary.path().join(directory_name);
        let owner = object(&directory, owner_id, directory_name, MetadataKind::Catalog);
        let form = child(form_id, "ItemForm", EdtMetadataChildKind::Form, owner.id());
        write_module(&directory.join("Forms/ItemForm/Module.bsl"));
        let observations = reader
            .read_form_command_modules(&owner, &[form], &directory)
            .expect("form layout must parse");
        module_ids.push(
            observations[0]
                .module()
                .expect("form module must exist")
                .id()
                .clone(),
        );
    }

    assert_ne!(module_ids[0], module_ids[1]);
    assert_eq!(module_ids[0].as_str(), "form.first.item:form_module");
    assert_eq!(module_ids[1].as_str(), "form.second.item:form_module");
}

#[test]
fn module_reader_repository_artifacts_match_declared_form_and_command_owners() {
    let root = repository_root();
    let object_directory = root.join("OneAgent_EDTproject/src/Catalogs/CounterpartiesProducts");
    let descriptor = FileSystemEdtMetadataObjectReader
        .read(&object_directory, MetadataKind::Catalog)
        .expect("repository Catalog descriptor must parse");
    let children = vec![
        child(
            "44be8175-75db-4b20-84e1-9c43395a1353",
            "PriceImport",
            EdtMetadataChildKind::Form,
            descriptor.id(),
        ),
        child(
            "fe3b9f49-74ea-4540-8e52-262e06739b1e",
            "CounterpartiesProductsPriceImport",
            EdtMetadataChildKind::Command,
            descriptor.id(),
        ),
    ];
    let observations = FileSystemEdtModuleReader
        .read_form_command_modules(&descriptor, &children, &object_directory)
        .expect("repository layouts must parse");
    let price_import = children
        .iter()
        .find(|child| {
            child.kind() == EdtMetadataChildKind::Form && child.name().as_str() == "PriceImport"
        })
        .expect("PriceImport Form declaration must exist");
    let price_command = children
        .iter()
        .find(|child| {
            child.kind() == EdtMetadataChildKind::Command
                && child.name().as_str() == "CounterpartiesProductsPriceImport"
        })
        .expect("PriceImport Command declaration must exist");

    for (owner, suffix) in [
        (price_import, "form_module"),
        (price_command, "command_module"),
    ] {
        let observation = observations
            .iter()
            .find(|observation| observation.owner_id() == Some(owner.id()))
            .expect("declared owner must have a layout observation");
        assert_eq!(observation.outcome(), EdtModuleLayoutOutcomeKind::Accepted);
        assert_eq!(
            observation
                .module()
                .expect("module must exist")
                .id()
                .as_str(),
            format!("{}:{suffix}", owner.id().as_str())
        );
    }
}

#[test]
fn module_reader_repository_common_command_and_common_form_preserve_roles() {
    let root = repository_root();
    let common_command_directory = root.join("OneAgent_EDTproject/src/CommonCommands/AccessRights");
    let common_command = FileSystemEdtMetadataObjectReader
        .read(&common_command_directory, MetadataKind::Command)
        .expect("Common Command descriptor must parse");
    let command_observations = FileSystemEdtModuleReader
        .read_form_command_modules(&common_command, &[], &common_command_directory)
        .expect("Common Command layout must parse");
    assert_eq!(command_observations.len(), 1);
    assert_eq!(
        command_observations[0].outcome(),
        EdtModuleLayoutOutcomeKind::Accepted
    );
    assert_eq!(
        command_observations[0].owner_id(),
        Some(common_command.id())
    );

    let common_form_directory = root.join("OneAgent_EDTproject/src/CommonForms/AccessRights");
    let common_form = FileSystemEdtMetadataObjectReader
        .read(&common_form_directory, MetadataKind::CommonForm)
        .expect("Common Form descriptor must parse");
    let existing = FileSystemEdtModuleReader
        .read_modules(common_form.id(), common_form.name(), &common_form_directory)
        .expect("existing Common Form module must parse");

    assert_eq!(existing.len(), 1);
    assert_eq!(existing[0].kind(), EdtModuleKind::Common);
    assert_eq!(existing[0].name(), common_form.name());
    assert_eq!(
        existing[0].id().as_str(),
        format!("{}:common_module", common_form.id().as_str())
    );
}
