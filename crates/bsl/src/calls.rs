//! Extraction of simple call expressions from BSL source.

use oneagent_common::{EntityId, EntityName};
use std::fmt::{Display, Formatter};

use crate::{
    BslIdentifierRange, BslParseError, ParsedCallableScope, parse_callable_scopes, source_lines,
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
        let parsed_module = parse_callable_scopes(module_id, source).map_err(scope_parse_error)?;
        let mut calls = Vec::new();
        let mut ordinal = 0_usize;
        let mut in_string = false;
        let mut member_continuation = None;

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

            let scope = containing_scope(parsed_module.scopes(), source_line.number);
            if scope.is_some_and(|scope| {
                source_line.number <= scope.header_end_line()
                    || source_line.number == scope.end_line()
            }) {
                member_continuation = None;
                continue;
            }

            for mut callee in extract_calls(
                trimmed,
                trimmed_start,
                &mut in_string,
                &mut member_continuation,
            ) {
                if callee.kind == BslCallKind::Qualified
                    && callee.target.split_once('.').is_some_and(|(qualifier, _)| {
                        scope.map_or_else(
                            || parsed_module.shadows(qualifier),
                            |scope| scope.shadows(qualifier),
                        )
                    })
                {
                    callee.kind = BslCallKind::Unsupported;
                }
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
                    scope.map(|scope| scope.symbol().name().clone()),
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

fn containing_scope(scopes: &[ParsedCallableScope], line: usize) -> Option<&ParsedCallableScope> {
    scopes
        .iter()
        .find(|scope| scope.symbol().line() <= line && line <= scope.end_line())
}

fn scope_parse_error(error: BslParseError) -> BslCallError {
    match error {
        BslParseError::NestedDeclaration(line) => BslCallError::NestedScope(line),
        BslParseError::MalformedDeclaration { line, text } => {
            BslCallError::MalformedScope { line, text }
        }
        BslParseError::InvalidName(line) => BslCallError::InvalidName(line),
        BslParseError::InvalidIdentifier(line) => BslCallError::InvalidIdentifier(line),
    }
}

#[derive(Debug)]
struct ExtractedCall {
    target: String,
    kind: BslCallKind,
    identifier_range: BslIdentifierRange,
}

#[derive(Debug, Clone)]
enum MemberContinuation {
    DirectQualifier {
        qualifier: String,
        supported: bool,
        dot_consumed: bool,
    },
    Computed {
        dot_consumed: bool,
    },
}

enum ContinuationUpdate {
    Preserve,
    Clear,
    Set(MemberContinuation),
}

fn extract_calls(
    line: &str,
    line_start: usize,
    in_string: &mut bool,
    member_continuation: &mut Option<MemberContinuation>,
) -> Vec<ExtractedCall> {
    let mut calls = Vec::new();
    let starts_in_string = *in_string;
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
                    let current_member_prefix =
                        final_start == start && has_member_access_prefix(line, start);
                    let prefix = line[..start].trim();
                    let inherited_member = (final_start == start)
                        .then_some(member_continuation.as_ref())
                        .flatten()
                        .filter(|continuation| match continuation {
                            MemberContinuation::DirectQualifier { dot_consumed, .. }
                            | MemberContinuation::Computed { dot_consumed } => {
                                (*dot_consumed && prefix.is_empty())
                                    || (!*dot_consumed && prefix == ".")
                            }
                        });
                    let current_member = current_member_prefix.then(|| {
                        let receiver = line[..start]
                            .trim_end()
                            .strip_suffix('.')
                            .expect("a detected member prefix ends with a dot")
                            .trim_end();
                        classify_member_receiver(receiver, true)
                    });
                    let effective_member = inherited_member.or(current_member.as_ref());
                    let (target, kind) = classify_extracted_call(
                        candidate,
                        final_start == start,
                        *in_string || in_comment,
                        effective_member,
                    );
                    calls.push(ExtractedCall {
                        target,
                        kind,
                        identifier_range,
                    });
                }
            }
        }
    }

    match continuation_after_line(line, starts_in_string) {
        ContinuationUpdate::Preserve => {}
        ContinuationUpdate::Clear => *member_continuation = None,
        ContinuationUpdate::Set(next) => *member_continuation = Some(next),
    }
    calls
}

fn classify_extracted_call(
    candidate: &str,
    is_unqualified_candidate: bool,
    is_legacy_candidate: bool,
    member: Option<&MemberContinuation>,
) -> (String, BslCallKind) {
    let (target, inferred_kind) = match member {
        Some(MemberContinuation::DirectQualifier {
            qualifier,
            supported,
            ..
        }) => (
            format!("{qualifier}.{candidate}"),
            if *supported {
                BslCallKind::Qualified
            } else {
                BslCallKind::Unsupported
            },
        ),
        Some(MemberContinuation::Computed { .. }) => {
            (candidate.to_owned(), BslCallKind::Unsupported)
        }
        None if is_unqualified_candidate => (candidate.to_owned(), BslCallKind::Local),
        None => (candidate.to_owned(), BslCallKind::Qualified),
    };
    let kind = if is_legacy_candidate {
        BslCallKind::Unsupported
    } else {
        inferred_kind
    };
    (target, kind)
}

fn has_member_access_prefix(line: &str, start: usize) -> bool {
    line.get(..start)
        .and_then(|prefix| prefix.chars().rev().find(|scalar| !scalar.is_whitespace()))
        == Some('.')
}

fn continuation_after_line(line: &str, mut in_string: bool) -> ContinuationUpdate {
    let characters = line.chars().collect::<Vec<_>>();
    let mut index = 0_usize;
    let mut visible = String::with_capacity(line.len());
    while index < characters.len() {
        let scalar = characters[index];
        if scalar == '"' {
            visible.push('"');
            if in_string && characters.get(index + 1) == Some(&'"') {
                index += 2;
                continue;
            }
            in_string = !in_string;
        } else if !in_string && scalar == '/' && characters.get(index + 1) == Some(&'/') {
            break;
        } else if !in_string {
            visible.push(scalar);
        }
        index += 1;
    }
    let visible = visible.trim_end();
    if visible.is_empty() {
        return ContinuationUpdate::Preserve;
    }
    let (receiver, dot_consumed) = if let Some(before_dot) = visible.strip_suffix('.') {
        (before_dot.trim_end(), true)
    } else if visible.chars().next_back().is_some_and(|scalar| {
        is_identifier_continue_without_dot(scalar) || matches!(scalar, ')' | ']')
    }) {
        (visible, false)
    } else {
        return ContinuationUpdate::Clear;
    };
    ContinuationUpdate::Set(classify_member_receiver(receiver, dot_consumed))
}

fn classify_member_receiver(receiver: &str, dot_consumed: bool) -> MemberContinuation {
    let identifier_start = receiver
        .char_indices()
        .rev()
        .find(|(_, scalar)| *scalar != '_' && !scalar.is_alphanumeric())
        .map_or(0, |(position, scalar)| position + scalar.len_utf8());
    let qualifier = &receiver[identifier_start..];
    if !qualifier.chars().next().is_some_and(is_identifier_start)
        || !qualifier.chars().all(is_identifier_continue_without_dot)
    {
        return MemberContinuation::Computed { dot_consumed };
    }
    let supported = !is_excluded_keyword(qualifier)
        && !receiver[..identifier_start]
            .trim_end()
            .ends_with(['.', ')', ']']);
    MemberContinuation::DirectQualifier {
        qualifier: qualifier.to_owned(),
        supported,
        dot_consumed,
    }
}

fn is_identifier_start(character: char) -> bool {
    character == '_' || character.is_alphabetic()
}

fn is_identifier_continue(character: char) -> bool {
    character == '_' || character == '.' || character.is_alphanumeric()
}

fn is_identifier_continue_without_dot(character: char) -> bool {
    character == '_' || character.is_alphanumeric()
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
            "  Text = \"Module.  HiddenMember()\"; // Module. HiddenComment()\n",
            "  Text = \"MultilineCall()\n",
            "  | StillHiddenCall()\";\n",
            "  VisibleCall();\n",
            "EndProcedure\n",
        );
        let calls = LineBslCallExtractor
            .extract_calls(&module_id(), source)
            .expect("calls must parse");

        assert_eq!(calls.len(), 9);
        assert_eq!(calls[0].target_symbol().as_str(), "HiddenCall");
        assert_eq!(calls[1].target_symbol().as_str(), "QuotedCall");
        assert_eq!(calls[2].target_symbol().as_str(), "RealCall");
        assert_eq!(calls[3].target_symbol().as_str(), "CommentedCall");
        assert_eq!(calls[4].target_symbol().as_str(), "Module.HiddenMember");
        assert_eq!(calls[5].target_symbol().as_str(), "Module.HiddenComment");
        assert_eq!(calls[6].target_symbol().as_str(), "MultilineCall");
        assert_eq!(calls[7].target_symbol().as_str(), "StillHiddenCall");
        assert_eq!(calls[8].target_symbol().as_str(), "VisibleCall");
        assert_eq!(calls[0].kind(), Some(BslCallKind::Unsupported));
        assert_eq!(calls[1].kind(), Some(BslCallKind::Unsupported));
        assert_eq!(calls[2].kind(), Some(BslCallKind::Local));
        assert_eq!(calls[3].kind(), Some(BslCallKind::Unsupported));
        assert_eq!(calls[4].kind(), Some(BslCallKind::Unsupported));
        assert_eq!(calls[5].kind(), Some(BslCallKind::Unsupported));
        assert_eq!(calls[6].kind(), Some(BslCallKind::Unsupported));
        assert_eq!(calls[7].kind(), Some(BslCallKind::Unsupported));
        assert_eq!(calls[8].kind(), Some(BslCallKind::Local));
    }

    #[test]
    fn computed_member_calls_are_unsupported_instead_of_local() {
        let source = concat!(
            "Procedure Test()\n",
            "  GetObject().Target(); Items[0].Target();\n",
            "  GetObject().\n",
            "  Target();\n",
            "  Stable.\n",
            "  Target();\n",
            "  GetObject()\n",
            "  .Target();\n",
            "  DirectModule\n",
            "  .Target();\n",
            "  Target();\n",
            "  DirectSpaced.  Target();\n",
            "  DirectSeparated . Target();\n",
            "  GetObject() . Target();\n",
            "EndProcedure\n",
        );
        let calls = LineBslCallExtractor
            .extract_calls(&module_id(), source)
            .expect("computed receivers must remain recoverable evidence");
        let targets = calls
            .iter()
            .filter(|call| call.target_symbol().as_str().ends_with("Target"))
            .collect::<Vec<_>>();
        assert_eq!(targets.len(), 10);
        assert_eq!(targets[0].kind(), Some(BslCallKind::Unsupported));
        assert_eq!(targets[1].kind(), Some(BslCallKind::Unsupported));
        assert_eq!(targets[2].kind(), Some(BslCallKind::Unsupported));
        assert_eq!(targets[3].target_symbol().as_str(), "Stable.Target");
        assert_eq!(targets[3].kind(), Some(BslCallKind::Qualified));
        assert_eq!(targets[4].kind(), Some(BslCallKind::Unsupported));
        assert_eq!(targets[5].target_symbol().as_str(), "DirectModule.Target");
        assert_eq!(targets[5].kind(), Some(BslCallKind::Qualified));
        assert_eq!(targets[6].kind(), Some(BslCallKind::Local));
        assert_eq!(targets[7].target_symbol().as_str(), "DirectSpaced.Target");
        assert_eq!(targets[7].kind(), Some(BslCallKind::Qualified));
        assert_eq!(
            targets[8].target_symbol().as_str(),
            "DirectSeparated.Target"
        );
        assert_eq!(targets[8].kind(), Some(BslCallKind::Qualified));
        assert_eq!(targets[9].kind(), Some(BslCallKind::Unsupported));
    }

    #[test]
    fn parameters_and_local_bindings_shadow_qualified_module_calls() {
        let source = concat!(
            "Var GlobalModule Export;\n",
            "Procedure Test(Module)\n",
            "  Module.Target(); Stable.Target(); Assigned.Target(); GlobalModule.Target();\n",
            "  Assigned = GetObject();\n",
            "EndProcedure\n",
        );
        let calls = LineBslCallExtractor
            .extract_calls(&module_id(), source)
            .expect("shadowed receivers must remain recoverable evidence");
        let kinds = calls
            .iter()
            .filter(|call| call.target_symbol().as_str().ends_with(".Target"))
            .map(|call| call.kind().expect("extracted calls have a kind"))
            .collect::<Vec<_>>();
        assert_eq!(
            kinds,
            [
                BslCallKind::Unsupported,
                BslCallKind::Qualified,
                BslCallKind::Unsupported,
                BslCallKind::Unsupported,
            ]
        );

        for malformed_module_binding in [
            "Var;\nProcedure Test()\nStable.Target();\nEndProcedure\n",
            "Var\nModule;\nProcedure Test()\nStable.Target();\nEndProcedure\n",
        ] {
            let calls = LineBslCallExtractor
                .extract_calls(&module_id(), malformed_module_binding)
                .expect("incomplete module bindings must remain recoverable");
            assert_eq!(calls.len(), 1);
            assert_eq!(calls[0].kind(), Some(BslCallKind::Unsupported));
        }
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
