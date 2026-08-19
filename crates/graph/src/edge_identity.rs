//! Shared stable semantic edge identity construction.

use crate::{EdgeId, EdgeKind};

/// Builds the stable identity for one canonical semantic relation.
pub(crate) fn edge_id(source: &str, target: &str, kind: EdgeKind) -> EdgeId {
    EdgeId::new(format!(
        "edge:source#{}:{};target#{}:{};kind:{}",
        source.len(),
        source,
        target.len(),
        target,
        edge_kind_code(kind)
    ))
}

const fn edge_kind_code(kind: EdgeKind) -> &'static str {
    match kind {
        EdgeKind::Contains => "contains",
        EdgeKind::Calls => "calls",
        EdgeKind::References => "references",
        EdgeKind::Reads => "reads",
        EdgeKind::Writes => "writes",
        EdgeKind::Grants => "grants",
        EdgeKind::Includes => "includes",
        EdgeKind::Extends => "extends",
        EdgeKind::DependsOn => "depends_on",
    }
}

#[cfg(test)]
mod tests {
    use super::edge_id;
    use crate::EdgeKind;

    #[test]
    fn identity_preserves_the_existing_encoded_contract() {
        let id = edge_id("source", "target", EdgeKind::DependsOn);

        assert_eq!(
            id.as_str(),
            "edge:source#6:source;target#6:target;kind:depends_on"
        );
    }

    #[test]
    fn length_prefixes_disambiguate_concatenated_components() {
        assert_ne!(
            edge_id("a:b", "c", EdgeKind::Calls),
            edge_id("a", "b:c", EdgeKind::Calls)
        );
    }
}
