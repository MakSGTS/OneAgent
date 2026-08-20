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
        EdgeKind::Opens => "opens",
        EdgeKind::Triggers => "triggers",
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
    fn opens_has_a_stable_machine_identity_without_changing_existing_kinds() {
        let opens = edge_id("source", "target", EdgeKind::Opens);
        let depends_on = edge_id("source", "target", EdgeKind::DependsOn);

        assert_eq!(
            opens.as_str(),
            "edge:source#6:source;target#6:target;kind:opens"
        );
        assert_eq!(
            depends_on.as_str(),
            "edge:source#6:source;target#6:target;kind:depends_on"
        );
        assert_ne!(opens, depends_on);
    }

    #[test]
    fn triggers_has_a_stable_machine_identity() {
        let triggers = edge_id("source", "target", EdgeKind::Triggers);

        assert_eq!(
            triggers.as_str(),
            "edge:source#6:source;target#6:target;kind:triggers"
        );
        assert_ne!(triggers, edge_id("source", "target", EdgeKind::References));
    }

    #[test]
    fn length_prefixes_disambiguate_concatenated_components() {
        assert_ne!(
            edge_id("a:b", "c", EdgeKind::Calls),
            edge_id("a", "b:c", EdgeKind::Calls)
        );
    }
}
