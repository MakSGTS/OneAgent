//! Extraction of simple call expressions from BSL source.

use oneagent_common::{EntityId, EntityName};
use std::fmt::{Display, Formatter};

use crate::{
    BslIdentifierRange, bsl_names_equal, is_callable_scope_end, leading_bsl_token, source_lines,
};

/// Lexical form of one extracted direct BSL call target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BslCallKind {
    /// An unqualified `CallableName(...)` target.
    Local,
    /// A `ModuleName.CallableName(...)` target.
    Qualified,
    /// A call-shaped token retained only for legacy semantic compatibility.
    Unsupported,
}

/// A call expression found in a BSL module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BslCall {
    id: EntityId,
    source_symbol: Option<EntityName>,
    target_symbol: EntityName,
    line: usize,
    kind: Option<BslCallKind>,
    identifier_range: Option<BslIdentifierRange>,
}

impl BslCall {
    /// Creates a BSL call.
    #[must_use]
    pub const fn new(
        id: EntityId,
        source_symbol: Option<EntityName>,
        target_symbol: EntityName,
        line: usize,
    ) -> Self {
        Self {
            id,
            source_symbol,
            kind: None,
            target_symbol,
            line,
            identifier_range: None,
        }
    }

    /// Creates an extracted BSL call with its exact final identifier range.
    #[must_use]
    pub fn new_with_identifier_range(
        id: EntityId,
        source_symbol: Option<EntityName>,
        target_symbol: EntityName,
        line: usize,
        kind: BslCallKind,
        identifier_range: BslIdentifierRange,
    ) -> Self {
        Self {
            id,
            source_symbol,
            target_symbol,
            line,
            kind: Some(kind),
            identifier_range: Some(identifier_range),
        }
    }

    /// Returns the stable call identifier.
    #[must_use]
    pub const fn id(&self) -> &EntityId {
        &self.id
    }

    /// Returns the procedure or function containing the call.
    #[must_use]
    pub fn source_symbol(&self) -> Option<&EntityName> {
        self.source_symbol.as_ref()
    }

    /// Returns the called symbol name.
    #[must_use]
    pub const fn target_symbol(&self) -> &EntityName {
        &self.target_symbol
    }

    /// Returns the one-based source line.
    #[must_use]
    pub const fn line(&self) -> usize {
        self.line
    }

    /// Returns the local or qualified lexical target form.
    #[must_use]
    pub const fn kind(&self) -> Option<BslCallKind> {
        self.kind
    }

    /// Returns the exact final identifier range when produced by an extractor.
    #[must_use]
    pub const fn identifier_range(&self) -> Option<BslIdentifierRange> {
        self.identifier_range
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
///
/// This implementation also tracks the current top-level procedure or function.
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
        let mut current_scope: Option<EntityName> = None;
        let mut in_string = false;

        for source_line in source_lines(source) {
            let (line, bom_bytes) = if source_line.number == 1 {
                source_line
                    .text
                    .strip_prefix('\u{feff}')
                    .map_or((source_line.text, 0), |line| (line, 3))
            } else {
                (source_line.text, 0)
            };
            let trimmed = line.trim_start();
            let trimmed_start =
                source_line.start_byte + bom_bytes + line.len().saturating_sub(trimmed.len());

            if !in_string
                && (trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with('#'))
            {
                continue;
            }

            if !in_string {
                if let Some(scope_name) = parse_scope_start(trimmed, source_line.number)? {
                    if current_scope.is_some() {
                        return Err(BslCallError::NestedScope(source_line.number));
                    }
                    current_scope = Some(scope_name);
                    continue;
                }

                if is_callable_scope_end(trimmed) {
                    current_scope = None;
                    continue;
                }
            }

            for callee in extract_calls(trimmed, trimmed_start, &mut in_string) {
                ordinal += 1;

                let id = EntityId::new(format!(
                    "{}:call:{}:{}",
                    module_id.as_str(),
                    source_line.number,
                    ordinal
                ))
                .map_err(|_| BslCallError::InvalidIdentifier(source_line.number))?;

                let target_symbol = EntityName::new(callee.target)
                    .map_err(|_| BslCallError::InvalidName(source_line.number))?;

                calls.push(BslCall::new_with_identifier_range(
                    id,
                    current_scope.clone(),
                    target_symbol,
                    source_line.number,
                    callee.kind,
                    callee.identifier_range,
                ));
            }
        }

        Ok(calls)
    }
}

fn parse_scope_start(line: &str, line_number: usize) -> Result<Option<EntityName>, BslCallError> {
    let Some((_, after_keyword)) = leading_bsl_token(line).filter(|(token, _)| {
        ["procedure", "процедура", "function", "функция"]
            .into_iter()
            .any(|keyword| bsl_names_equal(token, keyword))
    }) else {
        return Ok(None);
    };

    let remainder = after_keyword.trim_start();

    let Some(open_parenthesis) = remainder.find('(') else {
        return Err(BslCallError::MalformedScope {
            line: line_number,
            text: line.to_owned(),
        });
    };

    let name = remainder[..open_parenthesis].trim();

    if name.is_empty() {
        return Err(BslCallError::MalformedScope {
            line: line_number,
            text: line.to_owned(),
        });
    }

    EntityName::new(name)
        .map(Some)
        .map_err(|_| BslCallError::InvalidName(line_number))
}

#[derive(Debug)]
struct ExtractedCall {
    target: String,
    kind: BslCallKind,
    identifier_range: BslIdentifierRange,
}

fn extract_calls(line: &str, line_start: usize, in_string: &mut bool) -> Vec<ExtractedCall> {
    let mut calls = Vec::new();
    let characters = line.char_indices().collect::<Vec<_>>();
    let mut index = 0_usize;
    let mut in_comment = false;

    while index < characters.len() {
        let (_, character) = characters[index];

        if !in_comment && character == '"' {
            if characters
                .get(index + 1)
                .is_some_and(|(_, next)| *next == '"' && *in_string)
            {
                index += 2;
                continue;
            }
            *in_string = !*in_string;
            index += 1;
            continue;
        }

        if !*in_string
            && !in_comment
            && character == '/'
            && characters
                .get(index + 1)
                .is_some_and(|(_, next)| *next == '/')
        {
            in_comment = true;
            index += 2;
            continue;
        }

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
                let final_start = candidate.rfind('.').map_or(start, |dot| start + dot + 1);
                if let Some(identifier_range) =
                    BslIdentifierRange::new(line_start + final_start, line_start + end)
                {
                    calls.push(ExtractedCall {
                        target: candidate.to_owned(),
                        kind: if *in_string || in_comment {
                            BslCallKind::Unsupported
                        } else if final_start == start {
                            BslCallKind::Local
                        } else {
                            BslCallKind::Qualified
                        },
                        identifier_range,
                    });
                }
            }
        }
    }

    calls
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
    /// A call or scope name could not be represented.
    InvalidName(usize),

    /// A call identifier could not be represented.
    InvalidIdentifier(usize),

    /// A procedure or function declaration is malformed.
    MalformedScope {
        /// One-based source line.
        line: usize,

        /// Original source text.
        text: String,
    },

    /// A callable declaration appeared inside another callable scope.
    NestedScope(usize),
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

            Self::MalformedScope { line, text } => {
                write!(
                    formatter,
                    "malformed BSL scope declaration at line {line}: {text}"
                )
            }

            Self::NestedScope(line) => {
                write!(formatter, "nested BSL callable scope at line {line}")
            }
        }
    }
}

impl std::error::Error for BslCallError {}

#[cfg(test)]
mod tests {
    use oneagent_common::EntityId;

    use super::{BslCall, BslCallError, BslCallExtractor, BslCallKind, LineBslCallExtractor};

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

        assert_eq!(
            calls[0]
                .source_symbol()
                .expect("caller must exist")
                .as_str(),
            "Post"
        );

        assert_eq!(calls[0].target_symbol().as_str(), "FillMovements");

        assert_eq!(
            calls[1].target_symbol().as_str(),
            "AccessManagement.CheckRights"
        );
    }

    #[test]
    fn tracks_different_symbol_scopes() {
        let source = r"
Процедура ЗаписатьДокумент()
    ПроверитьДанные();
КонецПроцедуры

Функция ПолучитьСумму()
    Возврат РассчитатьСумму();
КонецФункции
";

        let calls = LineBslCallExtractor
            .extract_calls(&module_id(), source)
            .expect("calls must parse");

        assert_eq!(calls.len(), 2);

        assert_eq!(
            calls[0]
                .source_symbol()
                .expect("caller must exist")
                .as_str(),
            "ЗаписатьДокумент"
        );

        assert_eq!(
            calls[1]
                .source_symbol()
                .expect("caller must exist")
                .as_str(),
            "ПолучитьСумму"
        );
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
        assert_eq!(calls[0].target_symbol().as_str(), "IsReady");
        assert_eq!(calls[1].target_symbol().as_str(), "RealCall");
    }

    #[test]
    fn records_source_line() {
        let source = "\n\nDoWork();";

        let calls = LineBslCallExtractor
            .extract_calls(&module_id(), source)
            .expect("calls must parse");

        assert_eq!(calls[0].line(), 3);
        assert!(calls[0].source_symbol().is_none());
    }

    #[test]
    fn call_ranges_preserve_raw_unicode_line_endings_repeats_and_final_identifier() {
        let source = concat!(
            "\u{feff}Процедура Тест()\r\n",
            "  Проверить(); Проверить ();\r",
            "  ОбщийМодуль.Проверить(1);\n",
            "КонецПроцедуры\n",
        );
        let first = LineBslCallExtractor
            .extract_calls(&module_id(), source)
            .expect("calls must parse");
        let repeated = LineBslCallExtractor
            .extract_calls(&module_id(), source)
            .expect("repeated calls must parse");

        assert_eq!(first, repeated);
        assert_eq!(first.len(), 3);
        assert_eq!(first[0].kind(), Some(BslCallKind::Local));
        assert_eq!(first[1].kind(), Some(BslCallKind::Local));
        assert_eq!(first[2].kind(), Some(BslCallKind::Qualified));
        assert_eq!(first[0].line(), 2);
        assert_eq!(first[2].line(), 3);
        for call in &first {
            let range = call
                .identifier_range()
                .expect("extracted call must have an exact range");
            assert_eq!(&source[range.start_byte()..range.end_byte()], "Проверить");
        }
    }

    #[test]
    fn scope_keywords_are_token_exact_and_nested_scopes_fail() {
        let calls = LineBslCallExtractor
            .extract_calls(
                &module_id(),
                "Procedure Host()\nProcedureCall();\nEndProcedure\n",
            )
            .expect("keyword-prefixed calls must remain calls");
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].target_symbol().as_str(), "ProcedureCall");
        assert_eq!(
            calls[0]
                .source_symbol()
                .expect("call must retain its scope")
                .as_str(),
            "Host"
        );

        let nested = "Procedure Outer()\nFunction Inner()\nEndFunction\nEndProcedure\n";
        assert_eq!(
            LineBslCallExtractor.extract_calls(&module_id(), nested),
            Err(BslCallError::NestedScope(2))
        );
    }

    #[test]
    fn call_extraction_marks_legacy_string_and_inline_comment_candidates_unsupported() {
        let source = concat!(
            "Procedure Test()\n",
            "  Text = \"HiddenCall() and \"\"QuotedCall()\"\"\"; RealCall(); // CommentedCall()\n",
            "  Text = \"MultilineCall()\n",
            "  | StillHiddenCall()\";\n",
            "  VisibleCall();\n",
            "EndProcedure\n",
        );
        let calls = LineBslCallExtractor
            .extract_calls(&module_id(), source)
            .expect("calls must parse");

        assert_eq!(calls.len(), 7);
        assert_eq!(calls[0].target_symbol().as_str(), "HiddenCall");
        assert_eq!(calls[1].target_symbol().as_str(), "QuotedCall");
        assert_eq!(calls[2].target_symbol().as_str(), "RealCall");
        assert_eq!(calls[3].target_symbol().as_str(), "CommentedCall");
        assert_eq!(calls[4].target_symbol().as_str(), "MultilineCall");
        assert_eq!(calls[5].target_symbol().as_str(), "StillHiddenCall");
        assert_eq!(calls[6].target_symbol().as_str(), "VisibleCall");
        assert_eq!(calls[0].kind(), Some(BslCallKind::Unsupported));
        assert_eq!(calls[1].kind(), Some(BslCallKind::Unsupported));
        assert_eq!(calls[2].kind(), Some(BslCallKind::Local));
        assert_eq!(calls[3].kind(), Some(BslCallKind::Unsupported));
        assert_eq!(calls[4].kind(), Some(BslCallKind::Unsupported));
        assert_eq!(calls[5].kind(), Some(BslCallKind::Unsupported));
        assert_eq!(calls[6].kind(), Some(BslCallKind::Local));
    }

    #[test]
    fn legacy_call_constructor_preserves_target_kind_without_claiming_a_range() {
        let call = BslCall::new(
            EntityId::new("module:call:1:1").expect("identifier must be valid"),
            None,
            oneagent_common::EntityName::new("Module.Call").expect("name must be valid"),
            1,
        );

        assert!(call.kind().is_none());
        assert!(call.identifier_range().is_none());
    }
}
