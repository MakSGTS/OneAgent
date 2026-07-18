//! Extraction of simple call expressions from BSL source.

use oneagent_common::{EntityId, EntityName};
use std::fmt::{Display, Formatter};

/// A call expression found in a BSL module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BslCall {
    id: EntityId,
    callee: EntityName,
    line: usize,
}

impl BslCall {
    /// Creates a BSL call.
    #[must_use]
    pub const fn new(id: EntityId, callee: EntityName, line: usize) -> Self {
        Self { id, callee, line }
    }

    /// Returns the stable call identifier.
    #[must_use]
    pub const fn id(&self) -> &EntityId {
        &self.id
    }

    /// Returns the called symbol name.
    #[must_use]
    pub const fn callee(&self) -> &EntityName {
        &self.callee
    }

    /// Returns the one-based source line.
    #[must_use]
    pub const fn line(&self) -> usize {
        self.line
    }
}

/// Extracts simple calls from BSL source.
pub trait BslCallExtractor {
    /// Extracts calls using `module_id` as a stable parent identifier.
    ///
    /// # Errors
    ///
    /// Returns an error when a call cannot be represented by the domain model.
    fn extract_calls(
        &self,
        module_id: &EntityId,
        source: &str,
    ) -> Result<Vec<BslCall>, BslCallError>;
}

/// Conservative line-oriented extractor for direct BSL calls.
#[derive(Debug, Default, Clone, Copy)]
pub struct LineBslCallExtractor;

impl BslCallExtractor for LineBslCallExtractor {
    fn extract_calls(
        &self,
        module_id: &EntityId,
        source: &str,
    ) -> Result<Vec<BslCall>, BslCallError> {
        let mut calls = Vec::new();
        let mut ordinal = 0_usize;

        for (index, line) in source.lines().enumerate() {
            let line_number = index + 1;
            let trimmed = line.trim_start();

            if should_skip_line(trimmed) {
                continue;
            }

            for callee in extract_call_names(trimmed) {
                ordinal += 1;

                let id = EntityId::new(format!(
                    "{}:call:{}:{}",
                    module_id.as_str(),
                    line_number,
                    ordinal
                ))
                .map_err(|_| BslCallError::InvalidIdentifier(line_number))?;

                let callee =
                    EntityName::new(callee).map_err(|_| BslCallError::InvalidName(line_number))?;

                calls.push(BslCall::new(id, callee, line_number));
            }
        }

        Ok(calls)
    }
}

fn should_skip_line(line: &str) -> bool {
    let lowercase = line.to_lowercase();

    line.is_empty()
        || line.starts_with("//")
        || line.starts_with('#')
        || lowercase.starts_with("procedure ")
        || lowercase.starts_with("процедура ")
        || lowercase.starts_with("function ")
        || lowercase.starts_with("функция ")
}

fn extract_call_names(line: &str) -> Vec<String> {
    let mut names = Vec::new();
    let characters = line.char_indices().collect::<Vec<_>>();
    let mut index = 0_usize;

    while index < characters.len() {
        let (_, character) = characters[index];

        if !is_identifier_start(character) {
            index += 1;
            continue;
        }

        let start = characters[index].0;
        index += 1;

        while index < characters.len() && is_identifier_continue(characters[index].1) {
            index += 1;
        }

        let end = characters
            .get(index)
            .map_or(line.len(), |(position, _)| *position);

        let mut lookahead = index;

        while lookahead < characters.len() && characters[lookahead].1.is_whitespace() {
            lookahead += 1;
        }

        if lookahead < characters.len() && characters[lookahead].1 == '(' {
            let candidate = &line[start..end];

            if !is_excluded_keyword(candidate) {
                names.push(candidate.to_owned());
            }
        }
    }

    names
}

fn is_identifier_start(character: char) -> bool {
    character == '_' || character.is_alphabetic()
}

fn is_identifier_continue(character: char) -> bool {
    character == '_' || character == '.' || character.is_alphanumeric()
}

fn is_excluded_keyword(candidate: &str) -> bool {
    matches!(
        candidate.to_lowercase().as_str(),
        "if" | "если"
            | "elsif"
            | "иначеесли"
            | "while"
            | "пока"
            | "for"
            | "для"
            | "foreach"
            | "длякаждого"
            | "return"
            | "возврат"
            | "procedure"
            | "процедура"
            | "function"
            | "функция"
    )
}

/// Error produced while extracting BSL calls.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BslCallError {
    /// A call name could not be represented.
    InvalidName(usize),
    /// A call identifier could not be represented.
    InvalidIdentifier(usize),
}

impl Display for BslCallError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidName(line) => {
                write!(formatter, "invalid BSL call name at line {line}")
            }
            Self::InvalidIdentifier(line) => {
                write!(formatter, "invalid BSL call identifier at line {line}")
            }
        }
    }
}

impl std::error::Error for BslCallError {}

#[cfg(test)]
mod tests {
    use oneagent_common::EntityId;

    use super::{BslCallExtractor, LineBslCallExtractor};

    fn module_id() -> EntityId {
        EntityId::new("module.sales.object").expect("identifier must be valid")
    }

    #[test]
    fn extracts_direct_and_qualified_calls() {
        let source = r"
Procedure Post()
    FillMovements();
    AccessManagement.CheckRights(User);
EndProcedure
";

        let calls = LineBslCallExtractor
            .extract_calls(&module_id(), source)
            .expect("calls must parse");

        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].callee().as_str(), "FillMovements");
        assert_eq!(calls[1].callee().as_str(), "AccessManagement.CheckRights");
    }

    #[test]
    fn ignores_comments_and_control_flow() {
        let source = r"
// CommentedCall();
Procedure Test()
    If IsReady() Then
        RealCall();
    EndIf;
EndProcedure
";

        let calls = LineBslCallExtractor
            .extract_calls(&module_id(), source)
            .expect("calls must parse");

        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].callee().as_str(), "IsReady");
        assert_eq!(calls[1].callee().as_str(), "RealCall");
    }

    #[test]
    fn records_source_line() {
        let source = "\n\nDoWork();";

        let calls = LineBslCallExtractor
            .extract_calls(&module_id(), source)
            .expect("calls must parse");

        assert_eq!(calls[0].line(), 3);
    }
}
