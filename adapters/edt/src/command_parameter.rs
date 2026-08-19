//! Typed observations for EDT Command parameter types.

use oneagent_common::{EntityId, EntityName};
use oneagent_metadata::MetadataKind;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::EdtMetadataReferenceRole;

/// EDT source form that owns a Command parameter type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EdtCommandParameterSourceKind {
    /// Top-level Common Command descriptor.
    CommonCommand,

    /// Command declared inside a metadata object descriptor.
    SubordinateCommand,
}

/// Terminal parser classification for one Command parameter observation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EdtCommandParameterTypeOutcomeKind {
    /// The raw value maps to one accepted metadata target.
    Accepted,

    /// The source value is intentionally non-semantic for this reference slice.
    Ignored,

    /// The source value is well-formed but outside the accepted allowlist.
    Unsupported,

    /// The source layout or value is structurally invalid.
    Malformed,

    /// The Command does not declare a parameter-type container.
    Missing,
}

/// Typed reason for a non-accepted Command parameter observation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EdtCommandParameterTypeReason {
    /// No `commandParameterType` container is present.
    MissingContainer,

    /// The parameter-type container is present but contains no `types` values.
    EmptyContainer,

    /// More than one direct parameter-type container is present.
    DuplicateContainer,

    /// The value is an intentionally ignored primitive type.
    PrimitiveType,

    /// The value is a deferred `DefinedType` reference.
    DeferredDefinedType,

    /// The value is a non-reference platform type.
    UnsupportedPlatformType,

    /// The value uses an unrecognized reference prefix.
    UnsupportedPrefix,

    /// A direct `types` value is empty.
    EmptyValue,

    /// A reference-shaped value does not contain a target component.
    MissingComponent,

    /// A reference-shaped value contains more than two components.
    AdditionalComponents,

    /// A reference-shaped value contains an empty component.
    EmptyComponent,

    /// The target component cannot be represented as a canonical entity name.
    InvalidTargetName,
}

/// One deterministic parser observation owned by a canonical Command source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdtCommandParameterTypeObservation {
    source_id: EntityId,
    source_name: EntityName,
    source_kind: EdtCommandParameterSourceKind,
    descriptor_path: PathBuf,
    role: EdtMetadataReferenceRole,
    raw_token: Option<String>,
    target_kind: Option<MetadataKind>,
    target_name: Option<EntityName>,
    outcome: EdtCommandParameterTypeOutcomeKind,
    reason: Option<EdtCommandParameterTypeReason>,
    occurrence_count: usize,
}

impl EdtCommandParameterTypeObservation {
    /// Returns the canonical Command identifier that owns this observation.
    #[must_use]
    pub const fn source_id(&self) -> &EntityId {
        &self.source_id
    }

    /// Returns the canonical Command name that owns this observation.
    #[must_use]
    pub const fn source_name(&self) -> &EntityName {
        &self.source_name
    }

    /// Returns the EDT Command source form.
    #[must_use]
    pub const fn source_kind(&self) -> EdtCommandParameterSourceKind {
        self.source_kind
    }

    /// Returns the source descriptor path.
    #[must_use]
    pub fn descriptor_path(&self) -> &Path {
        &self.descriptor_path
    }

    /// Returns the distinct semantic reference role.
    #[must_use]
    pub const fn role(&self) -> EdtMetadataReferenceRole {
        self.role
    }

    /// Returns the preserved raw EDT token when the observation represents a value.
    #[must_use]
    pub fn raw_token(&self) -> Option<&str> {
        self.raw_token.as_deref()
    }

    /// Returns the mapped target kind for an accepted observation.
    #[must_use]
    pub const fn target_kind(&self) -> Option<MetadataKind> {
        self.target_kind
    }

    /// Returns the canonical target name for an accepted observation.
    #[must_use]
    pub const fn target_name(&self) -> Option<&EntityName> {
        self.target_name.as_ref()
    }

    /// Returns the terminal parser classification.
    #[must_use]
    pub const fn outcome(&self) -> EdtCommandParameterTypeOutcomeKind {
        self.outcome
    }

    /// Returns the typed reason for a non-accepted observation.
    #[must_use]
    pub const fn reason(&self) -> Option<EdtCommandParameterTypeReason> {
        self.reason
    }

    /// Returns how many equal raw values were aggregated into this observation.
    #[must_use]
    pub const fn occurrence_count(&self) -> usize {
        self.occurrence_count
    }
}

#[derive(Debug, Default)]
pub(crate) struct EdtCommandParameterTypeCollector {
    container_count: usize,
    values: Vec<String>,
}

impl EdtCommandParameterTypeCollector {
    pub(crate) const fn observe_container(&mut self) {
        self.container_count += 1;
    }

    pub(crate) fn observe_value(&mut self, raw_token: String) {
        self.values.push(raw_token);
    }

    pub(crate) fn finish(
        self,
        source_id: &EntityId,
        source_name: &EntityName,
        source_kind: EdtCommandParameterSourceKind,
        descriptor_path: &Path,
    ) -> Vec<EdtCommandParameterTypeObservation> {
        if self.container_count == 0 {
            return vec![non_value_observation(
                source_id,
                source_name,
                source_kind,
                descriptor_path,
                EdtCommandParameterTypeOutcomeKind::Missing,
                EdtCommandParameterTypeReason::MissingContainer,
                1,
            )];
        }

        if self.container_count > 1 {
            return vec![non_value_observation(
                source_id,
                source_name,
                source_kind,
                descriptor_path,
                EdtCommandParameterTypeOutcomeKind::Malformed,
                EdtCommandParameterTypeReason::DuplicateContainer,
                self.container_count,
            )];
        }

        if self.values.is_empty() {
            return vec![non_value_observation(
                source_id,
                source_name,
                source_kind,
                descriptor_path,
                EdtCommandParameterTypeOutcomeKind::Ignored,
                EdtCommandParameterTypeReason::EmptyContainer,
                1,
            )];
        }

        let mut values = BTreeMap::<String, usize>::new();
        for value in self.values {
            *values.entry(value).or_default() += 1;
        }

        values
            .into_iter()
            .map(|(raw_token, occurrence_count)| {
                parse_value(
                    source_id,
                    source_name,
                    source_kind,
                    descriptor_path,
                    raw_token,
                    occurrence_count,
                )
            })
            .collect()
    }
}

fn parse_value(
    source_id: &EntityId,
    source_name: &EntityName,
    source_kind: EdtCommandParameterSourceKind,
    descriptor_path: &Path,
    raw_token: String,
    occurrence_count: usize,
) -> EdtCommandParameterTypeObservation {
    let value = raw_token.trim();
    if value.is_empty() {
        return malformed_value_observation(
            source_id,
            source_name,
            source_kind,
            descriptor_path,
            raw_token,
            occurrence_count,
            EdtCommandParameterTypeReason::EmptyValue,
        );
    }

    let value = value.rsplit(':').next().unwrap_or(value).to_owned();
    parse_nonempty_value(
        source_id,
        source_name,
        source_kind,
        descriptor_path,
        raw_token,
        occurrence_count,
        &value,
    )
}

#[allow(clippy::too_many_arguments)]
fn parse_nonempty_value(
    source_id: &EntityId,
    source_name: &EntityName,
    source_kind: EdtCommandParameterSourceKind,
    descriptor_path: &Path,
    raw_token: String,
    occurrence_count: usize,
    value: &str,
) -> EdtCommandParameterTypeObservation {
    let components = value.split('.').collect::<Vec<_>>();
    if components.len() == 1 {
        let (outcome, reason) =
            if command_parameter_reference_kind(value).is_some() || value == "DefinedType" {
                (
                    EdtCommandParameterTypeOutcomeKind::Malformed,
                    EdtCommandParameterTypeReason::MissingComponent,
                )
            } else if is_primitive_type(value) {
                (
                    EdtCommandParameterTypeOutcomeKind::Ignored,
                    EdtCommandParameterTypeReason::PrimitiveType,
                )
            } else {
                (
                    EdtCommandParameterTypeOutcomeKind::Unsupported,
                    EdtCommandParameterTypeReason::UnsupportedPlatformType,
                )
            };
        return value_observation(
            source_id,
            source_name,
            source_kind,
            descriptor_path,
            raw_token,
            occurrence_count,
            outcome,
            Some(reason),
            None,
            None,
        );
    }
    if components.len() > 2 {
        return malformed_value_observation(
            source_id,
            source_name,
            source_kind,
            descriptor_path,
            raw_token,
            occurrence_count,
            EdtCommandParameterTypeReason::AdditionalComponents,
        );
    }
    if components.iter().any(|component| component.is_empty()) {
        return malformed_value_observation(
            source_id,
            source_name,
            source_kind,
            descriptor_path,
            raw_token,
            occurrence_count,
            EdtCommandParameterTypeReason::EmptyComponent,
        );
    }

    let prefix = components[0];
    let target_name = components[1];
    let Some(target_kind) = command_parameter_reference_kind(prefix) else {
        let reason = if prefix == "DefinedType" {
            EdtCommandParameterTypeReason::DeferredDefinedType
        } else {
            EdtCommandParameterTypeReason::UnsupportedPrefix
        };
        return value_observation(
            source_id,
            source_name,
            source_kind,
            descriptor_path,
            raw_token,
            occurrence_count,
            EdtCommandParameterTypeOutcomeKind::Unsupported,
            Some(reason),
            None,
            None,
        );
    };
    let Ok(target_name) = EntityName::new(target_name) else {
        return malformed_value_observation(
            source_id,
            source_name,
            source_kind,
            descriptor_path,
            raw_token,
            occurrence_count,
            EdtCommandParameterTypeReason::InvalidTargetName,
        );
    };

    value_observation(
        source_id,
        source_name,
        source_kind,
        descriptor_path,
        raw_token,
        occurrence_count,
        EdtCommandParameterTypeOutcomeKind::Accepted,
        None,
        Some(target_kind),
        Some(target_name),
    )
}

fn malformed_value_observation(
    source_id: &EntityId,
    source_name: &EntityName,
    source_kind: EdtCommandParameterSourceKind,
    descriptor_path: &Path,
    raw_token: String,
    occurrence_count: usize,
    reason: EdtCommandParameterTypeReason,
) -> EdtCommandParameterTypeObservation {
    value_observation(
        source_id,
        source_name,
        source_kind,
        descriptor_path,
        raw_token,
        occurrence_count,
        EdtCommandParameterTypeOutcomeKind::Malformed,
        Some(reason),
        None,
        None,
    )
}

#[allow(clippy::too_many_arguments)]
fn value_observation(
    source_id: &EntityId,
    source_name: &EntityName,
    source_kind: EdtCommandParameterSourceKind,
    descriptor_path: &Path,
    raw_token: String,
    occurrence_count: usize,
    outcome: EdtCommandParameterTypeOutcomeKind,
    reason: Option<EdtCommandParameterTypeReason>,
    target_kind: Option<MetadataKind>,
    target_name: Option<EntityName>,
) -> EdtCommandParameterTypeObservation {
    EdtCommandParameterTypeObservation {
        source_id: source_id.clone(),
        source_name: source_name.clone(),
        source_kind,
        descriptor_path: descriptor_path.to_path_buf(),
        role: EdtMetadataReferenceRole::CommandParameterType,
        raw_token: Some(raw_token),
        target_kind,
        target_name,
        outcome,
        reason,
        occurrence_count,
    }
}

fn non_value_observation(
    source_id: &EntityId,
    source_name: &EntityName,
    source_kind: EdtCommandParameterSourceKind,
    descriptor_path: &Path,
    outcome: EdtCommandParameterTypeOutcomeKind,
    reason: EdtCommandParameterTypeReason,
    occurrence_count: usize,
) -> EdtCommandParameterTypeObservation {
    EdtCommandParameterTypeObservation {
        source_id: source_id.clone(),
        source_name: source_name.clone(),
        source_kind,
        descriptor_path: descriptor_path.to_path_buf(),
        role: EdtMetadataReferenceRole::CommandParameterType,
        raw_token: None,
        target_kind: None,
        target_name: None,
        outcome,
        reason: Some(reason),
        occurrence_count,
    }
}

const fn is_primitive_type(value: &str) -> bool {
    matches!(
        value.as_bytes(),
        b"Boolean" | b"Date" | b"Null" | b"Number" | b"String" | b"Undefined"
    )
}

const fn command_parameter_reference_kind(prefix: &str) -> Option<MetadataKind> {
    match prefix.as_bytes() {
        b"CatalogRef" => Some(MetadataKind::Catalog),
        b"DocumentRef" => Some(MetadataKind::Document),
        b"EnumRef" => Some(MetadataKind::Enumeration),
        b"InformationRegisterRecordSet" | b"InformationRegisterRecordKey" => {
            Some(MetadataKind::InformationRegister)
        }
        b"AccumulationRegisterRecordSet" | b"AccumulationRegisterRecordKey" => {
            Some(MetadataKind::AccumulationRegister)
        }
        b"AccountingRegisterRecordSet" | b"AccountingRegisterRecordKey" => {
            Some(MetadataKind::AccountingRegister)
        }
        b"CalculationRegisterRecordSet" | b"CalculationRegisterRecordKey" => {
            Some(MetadataKind::CalculationRegister)
        }
        b"BusinessProcessRef" => Some(MetadataKind::BusinessProcess),
        b"TaskRef" => Some(MetadataKind::Task),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use oneagent_common::{EntityId, EntityName};
    use oneagent_metadata::MetadataKind;
    use std::path::Path;

    use super::{
        EdtCommandParameterSourceKind, EdtCommandParameterTypeCollector,
        EdtCommandParameterTypeOutcomeKind, EdtCommandParameterTypeReason,
    };

    fn observations(values: &[&str]) -> Vec<super::EdtCommandParameterTypeObservation> {
        let mut collector = EdtCommandParameterTypeCollector::default();
        collector.observe_container();
        for value in values {
            collector.observe_value((*value).to_owned());
        }
        collector.finish(
            &EntityId::new("command-id").expect("identifier must be valid"),
            &EntityName::new("Command").expect("name must be valid"),
            EdtCommandParameterSourceKind::CommonCommand,
            Path::new("CommonCommands/Command/Command.mdo"),
        )
    }

    #[test]
    fn maps_exact_nine_kind_allowlist() {
        let cases = [
            ("CatalogRef.Catalog", MetadataKind::Catalog),
            ("DocumentRef.Document", MetadataKind::Document),
            ("EnumRef.Enumeration", MetadataKind::Enumeration),
            (
                "InformationRegisterRecordSet.InformationRegister",
                MetadataKind::InformationRegister,
            ),
            (
                "AccumulationRegisterRecordKey.AccumulationRegister",
                MetadataKind::AccumulationRegister,
            ),
            (
                "AccountingRegisterRecordSet.AccountingRegister",
                MetadataKind::AccountingRegister,
            ),
            (
                "CalculationRegisterRecordKey.CalculationRegister",
                MetadataKind::CalculationRegister,
            ),
            (
                "BusinessProcessRef.BusinessProcess",
                MetadataKind::BusinessProcess,
            ),
            ("TaskRef.Task", MetadataKind::Task),
        ];
        let values = cases.iter().map(|(value, _)| *value).collect::<Vec<_>>();
        let parsed = observations(&values);

        assert_eq!(parsed.len(), cases.len());
        for (raw_token, target_kind) in cases {
            let observation = parsed
                .iter()
                .find(|observation| observation.raw_token() == Some(raw_token))
                .expect("allowlisted value must be preserved");
            assert_eq!(
                observation.outcome(),
                EdtCommandParameterTypeOutcomeKind::Accepted
            );
            assert_eq!(observation.target_kind(), Some(target_kind));
        }
    }

    #[test]
    fn classifies_negative_values_without_lower_confidence_mapping() {
        let parsed = observations(&[
            "String",
            "DefinedType.DocumentComment",
            "ValueTable",
            "UnknownRef.Target",
            "CatalogRef",
            "CatalogRef.",
            "CatalogRef.Target.Additional",
            "",
        ]);
        let classification = parsed
            .iter()
            .map(|observation| {
                (
                    observation.raw_token().unwrap_or_default(),
                    observation.outcome(),
                    observation.reason(),
                )
            })
            .collect::<Vec<_>>();

        assert!(classification.contains(&(
            "String",
            EdtCommandParameterTypeOutcomeKind::Ignored,
            Some(EdtCommandParameterTypeReason::PrimitiveType),
        )));
        assert!(classification.contains(&(
            "DefinedType.DocumentComment",
            EdtCommandParameterTypeOutcomeKind::Unsupported,
            Some(EdtCommandParameterTypeReason::DeferredDefinedType),
        )));
        assert!(classification.contains(&(
            "UnknownRef.Target",
            EdtCommandParameterTypeOutcomeKind::Unsupported,
            Some(EdtCommandParameterTypeReason::UnsupportedPrefix),
        )));
        assert!(classification.contains(&(
            "CatalogRef",
            EdtCommandParameterTypeOutcomeKind::Malformed,
            Some(EdtCommandParameterTypeReason::MissingComponent),
        )));
        assert!(parsed.iter().all(|observation| {
            observation.outcome() != EdtCommandParameterTypeOutcomeKind::Accepted
        }));
    }

    #[test]
    fn deduplicates_and_orders_values_independently_of_input_order() {
        let first = observations(&["TaskRef.Task", "CatalogRef.Catalog", "TaskRef.Task"]);
        let reordered = observations(&["TaskRef.Task", "TaskRef.Task", "CatalogRef.Catalog"]);

        assert_eq!(first, reordered);
        assert_eq!(first[0].raw_token(), Some("CatalogRef.Catalog"));
        assert_eq!(first[1].raw_token(), Some("TaskRef.Task"));
        assert_eq!(first[1].occurrence_count(), 2);
    }
}
