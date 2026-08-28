use std::path::PathBuf;

pub(crate) fn project_root() -> PathBuf {
    let root = std::env::var_os("ONEAGENT_EDT_CORPUS")
        .map(PathBuf::from)
        .expect("ONEAGENT_EDT_CORPUS must name the external EDT corpus project");
    assert!(
        root.is_absolute(),
        "ONEAGENT_EDT_CORPUS must be an absolute path"
    );
    root
}
