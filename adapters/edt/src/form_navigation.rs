//! Private extraction for the first static EDT Form-navigation slice.

use oneagent_bsl::BslSymbolKind;
use oneagent_common::{EntityId, EntityName};
use oneagent_metadata::MetadataKind;
use std::path::PathBuf;

use crate::{EdtModuleDescriptor, EdtModuleKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum EdtFormNavigationParseOutcome {
    Candidate(Box<EdtFormNavigationCandidate>),
    Rejected(Box<EdtFormNavigationRejection>),
}

impl EdtFormNavigationParseOutcome {
    pub(crate) const fn kind(&self) -> EdtFormNavigationOutcomeKind {
        match self {
            Self::Candidate(_) => EdtFormNavigationOutcomeKind::Accepted,
            Self::Rejected(rejection) => rejection.reason.outcome_kind(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum EdtFormNavigationOutcomeKind {
    Accepted,
    Malformed,
    Unsupported,
    Dynamic,
    Incomplete,
    WrongModule,
    WrongCallable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EdtFormNavigationCandidate {
    pub(crate) module_id: EntityId,
    pub(crate) module_path: PathBuf,
    pub(crate) procedure_id: EntityId,
    pub(crate) procedure_name: EntityName,
    pub(crate) raw_statement: String,
    pub(crate) literal: String,
    pub(crate) target: EdtFormNavigationTarget,
    pub(crate) location: EdtFormNavigationSourceLocation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum EdtFormNavigationTarget {
    CommonForm {
        form_name: EntityName,
    },
    SubordinateForm {
        owner_kind: MetadataKind,
        owner_name: EntityName,
        form_name: EntityName,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EdtFormNavigationRejection {
    pub(crate) module_id: EntityId,
    pub(crate) module_path: PathBuf,
    pub(crate) module_kind: EdtModuleKind,
    pub(crate) containing_symbol_id: Option<EntityId>,
    pub(crate) containing_symbol_name: Option<EntityName>,
    pub(crate) containing_symbol_kind: Option<BslSymbolKind>,
    pub(crate) raw_statement: String,
    pub(crate) literal: Option<String>,
    pub(crate) location: EdtFormNavigationSourceLocation,
    pub(crate) reason: EdtFormNavigationRejectionReason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EdtFormNavigationRejectionReason {
    MalformedStatement,
    UnsupportedTarget(EdtFormNavigationUnsupportedTargetReason),
    DynamicFirstArgument,
    IncompleteStatement,
    UnsupportedModuleKind(EdtModuleKind),
    MissingContainingSymbol,
    UnsupportedContainingSymbol(BslSymbolKind),
}

impl EdtFormNavigationRejectionReason {
    const fn outcome_kind(self) -> EdtFormNavigationOutcomeKind {
        match self {
            Self::MalformedStatement => EdtFormNavigationOutcomeKind::Malformed,
            Self::UnsupportedTarget(_) => EdtFormNavigationOutcomeKind::Unsupported,
            Self::DynamicFirstArgument => EdtFormNavigationOutcomeKind::Dynamic,
            Self::IncompleteStatement => EdtFormNavigationOutcomeKind::Incomplete,
            Self::UnsupportedModuleKind(_) => EdtFormNavigationOutcomeKind::WrongModule,
            Self::MissingContainingSymbol | Self::UnsupportedContainingSymbol(_) => {
                EdtFormNavigationOutcomeKind::WrongCallable
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EdtFormNavigationUnsupportedTargetReason {
    DefaultFormAlias,
    ShorthandForm,
    UnsupportedPrefix,
    InvalidTargetShape,
    InvalidName,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EdtFormNavigationSourceLocation {
    pub(crate) line: usize,
    pub(crate) column: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ContainingSymbol {
    id: EntityId,
    name: EntityName,
    kind: BslSymbolKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Statement {
    tokens: Vec<Token>,
    raw: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Token {
    kind: TokenKind,
    start: usize,
    end: usize,
    line: usize,
    column: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TokenKind {
    Identifier(String),
    StringLiteral { value: String, terminated: bool },
    LeftParenthesis,
    RightParenthesis,
    Comma,
    Semicolon,
    Newline,
    Other,
}

/// Extracts ordered candidate and rejection outcomes without graph resolution or emission.
pub(crate) fn extract_form_navigation_candidates(
    module: &EdtModuleDescriptor,
    source: &str,
) -> Vec<EdtFormNavigationParseOutcome> {
    let statements = split_statements(source, &tokenize(source));
    let mut containing_symbol = None;
    let mut outcomes = Vec::new();

    for statement in statements {
        if let Some(transition) = scope_transition(&statement, module) {
            match transition {
                ScopeTransition::Start(symbol) => containing_symbol = symbol,
                ScopeTransition::End => containing_symbol = None,
            }
            continue;
        }

        let tokens = significant_tokens(&statement.tokens);
        for (position, token) in tokens.iter().enumerate() {
            if identifier(token) == Some("OpenForm") {
                outcomes.push(classify_open_form(
                    module,
                    containing_symbol.as_ref(),
                    &statement,
                    &tokens,
                    position,
                ));
            }
        }
    }

    outcomes
}

fn classify_open_form(
    module: &EdtModuleDescriptor,
    containing_symbol: Option<&ContainingSymbol>,
    statement: &Statement,
    tokens: &[&Token],
    callee_position: usize,
) -> EdtFormNavigationParseOutcome {
    let callee = tokens[callee_position];
    let location = EdtFormNavigationSourceLocation {
        line: callee.line,
        column: callee.column,
    };
    let reject = |reason, literal| {
        rejection(
            module,
            containing_symbol,
            statement,
            location,
            reason,
            literal,
        )
    };
    let open_parenthesis = callee_position + 1;
    if !matches!(
        tokens.get(open_parenthesis).map(|token| &token.kind),
        Some(TokenKind::LeftParenthesis)
    ) {
        return reject(EdtFormNavigationRejectionReason::MalformedStatement, None);
    }
    let Some(close_parenthesis) = matching_parenthesis(tokens, open_parenthesis) else {
        return reject(EdtFormNavigationRejectionReason::IncompleteStatement, None);
    };
    if !is_complete_call_statement(tokens, callee_position, close_parenthesis) {
        return reject(EdtFormNavigationRejectionReason::MalformedStatement, None);
    }

    let first_argument = open_parenthesis + 1;
    let Some(first_argument_token) = tokens.get(first_argument) else {
        return reject(EdtFormNavigationRejectionReason::MalformedStatement, None);
    };
    let TokenKind::StringLiteral { value, terminated } = &first_argument_token.kind else {
        return reject(EdtFormNavigationRejectionReason::DynamicFirstArgument, None);
    };
    if !terminated {
        return reject(
            EdtFormNavigationRejectionReason::IncompleteStatement,
            Some(value.clone()),
        );
    }
    if !matches!(
        tokens.get(first_argument + 1).map(|token| &token.kind),
        Some(TokenKind::Comma | TokenKind::RightParenthesis)
    ) {
        return reject(
            EdtFormNavigationRejectionReason::DynamicFirstArgument,
            Some(value.clone()),
        );
    }

    if module.kind() != EdtModuleKind::Command {
        return reject(
            EdtFormNavigationRejectionReason::UnsupportedModuleKind(module.kind()),
            Some(value.clone()),
        );
    }
    let Some(symbol) = containing_symbol else {
        return reject(
            EdtFormNavigationRejectionReason::MissingContainingSymbol,
            Some(value.clone()),
        );
    };
    if symbol.kind != BslSymbolKind::Procedure {
        return reject(
            EdtFormNavigationRejectionReason::UnsupportedContainingSymbol(symbol.kind),
            Some(value.clone()),
        );
    }

    let target = match parse_target(value) {
        Ok(target) => target,
        Err(reason) => {
            return reject(
                EdtFormNavigationRejectionReason::UnsupportedTarget(reason),
                Some(value.clone()),
            );
        }
    };

    EdtFormNavigationParseOutcome::Candidate(Box::new(EdtFormNavigationCandidate {
        module_id: module.id().clone(),
        module_path: module.path().to_path_buf(),
        procedure_id: symbol.id.clone(),
        procedure_name: symbol.name.clone(),
        raw_statement: statement.raw.clone(),
        literal: value.clone(),
        target,
        location,
    }))
}

fn parse_target(
    literal: &str,
) -> Result<EdtFormNavigationTarget, EdtFormNavigationUnsupportedTargetReason> {
    let components = literal.split('.').collect::<Vec<_>>();
    if let ["CommonForm", form_name] = components.as_slice() {
        return Ok(EdtFormNavigationTarget::CommonForm {
            form_name: parse_name(form_name)?,
        });
    }
    if let [prefix, owner_name, "Form", form_name] = components.as_slice() {
        let owner_kind = owner_kind(prefix)
            .ok_or(EdtFormNavigationUnsupportedTargetReason::UnsupportedPrefix)?;
        return Ok(EdtFormNavigationTarget::SubordinateForm {
            owner_kind,
            owner_name: parse_name(owner_name)?,
            form_name: parse_name(form_name)?,
        });
    }
    if let [prefix, _, "Form"] = components.as_slice()
        && owner_kind(prefix).is_some()
    {
        return Err(EdtFormNavigationUnsupportedTargetReason::DefaultFormAlias);
    }
    if let [prefix, _, shorthand] = components.as_slice()
        && owner_kind(prefix).is_some()
        && matches!(*shorthand, "ListForm" | "ObjectForm")
    {
        return Err(EdtFormNavigationUnsupportedTargetReason::ShorthandForm);
    }
    Err(EdtFormNavigationUnsupportedTargetReason::InvalidTargetShape)
}

fn parse_name(value: &str) -> Result<EntityName, EdtFormNavigationUnsupportedTargetReason> {
    EntityName::new(value).map_err(|_| EdtFormNavigationUnsupportedTargetReason::InvalidName)
}

const fn owner_kind(prefix: &str) -> Option<MetadataKind> {
    match prefix.as_bytes() {
        b"Catalog" => Some(MetadataKind::Catalog),
        b"Document" => Some(MetadataKind::Document),
        b"Report" => Some(MetadataKind::Report),
        b"DataProcessor" => Some(MetadataKind::DataProcessor),
        b"InformationRegister" => Some(MetadataKind::InformationRegister),
        b"AccumulationRegister" => Some(MetadataKind::AccumulationRegister),
        b"AccountingRegister" => Some(MetadataKind::AccountingRegister),
        b"CalculationRegister" => Some(MetadataKind::CalculationRegister),
        b"BusinessProcess" => Some(MetadataKind::BusinessProcess),
        b"Task" => Some(MetadataKind::Task),
        _ => None,
    }
}

fn rejection(
    module: &EdtModuleDescriptor,
    containing_symbol: Option<&ContainingSymbol>,
    statement: &Statement,
    location: EdtFormNavigationSourceLocation,
    reason: EdtFormNavigationRejectionReason,
    literal: Option<String>,
) -> EdtFormNavigationParseOutcome {
    EdtFormNavigationParseOutcome::Rejected(Box::new(EdtFormNavigationRejection {
        module_id: module.id().clone(),
        module_path: module.path().to_path_buf(),
        module_kind: module.kind(),
        containing_symbol_id: containing_symbol.map(|symbol| symbol.id.clone()),
        containing_symbol_name: containing_symbol.map(|symbol| symbol.name.clone()),
        containing_symbol_kind: containing_symbol.map(|symbol| symbol.kind),
        raw_statement: statement.raw.clone(),
        literal,
        location,
        reason,
    }))
}

enum ScopeTransition {
    Start(Option<ContainingSymbol>),
    End,
}

fn scope_transition(
    statement: &Statement,
    module: &EdtModuleDescriptor,
) -> Option<ScopeTransition> {
    let tokens = significant_tokens(&statement.tokens);
    let first = tokens.first().and_then(|token| identifier(token))?;
    if is_scope_end(first) {
        return Some(ScopeTransition::End);
    }
    let kind = if is_procedure(first) {
        BslSymbolKind::Procedure
    } else if is_function(first) {
        BslSymbolKind::Function
    } else {
        return None;
    };
    let Some(name) = tokens.get(1).and_then(|token| identifier(token)) else {
        return Some(ScopeTransition::Start(None));
    };
    if !matches!(
        tokens.get(2).map(|token| &token.kind),
        Some(TokenKind::LeftParenthesis)
    ) {
        return Some(ScopeTransition::Start(None));
    }
    let Ok(name) = EntityName::new(name) else {
        return Some(ScopeTransition::Start(None));
    };
    let Ok(id) = EntityId::new(format!(
        "{}:{}:{}",
        module.id().as_str(),
        kind.as_str(),
        name.as_str()
    )) else {
        return Some(ScopeTransition::Start(None));
    };
    Some(ScopeTransition::Start(Some(ContainingSymbol {
        id,
        name,
        kind,
    })))
}

fn matching_parenthesis(tokens: &[&Token], open_parenthesis: usize) -> Option<usize> {
    let mut depth = 0_usize;
    for (position, token) in tokens.iter().enumerate().skip(open_parenthesis) {
        match token.kind {
            TokenKind::LeftParenthesis => depth += 1,
            TokenKind::RightParenthesis => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(position);
                }
            }
            _ => {}
        }
    }
    None
}

fn is_complete_call_statement(
    tokens: &[&Token],
    callee_position: usize,
    close_parenthesis: usize,
) -> bool {
    if callee_position != 0 {
        return false;
    }
    close_parenthesis + 1 == tokens.len()
        || (close_parenthesis + 2 == tokens.len()
            && matches!(tokens[close_parenthesis + 1].kind, TokenKind::Semicolon))
}

fn significant_tokens(tokens: &[Token]) -> Vec<&Token> {
    tokens
        .iter()
        .filter(|token| !matches!(token.kind, TokenKind::Newline))
        .collect()
}

fn identifier(token: &Token) -> Option<&str> {
    match &token.kind {
        TokenKind::Identifier(value) => Some(value),
        _ => None,
    }
}

fn is_procedure(value: &str) -> bool {
    value.eq_ignore_ascii_case("Procedure") || value.eq_ignore_ascii_case("Процедура")
}

fn is_function(value: &str) -> bool {
    value.eq_ignore_ascii_case("Function") || value.eq_ignore_ascii_case("Функция")
}

fn is_scope_end(value: &str) -> bool {
    value.eq_ignore_ascii_case("EndProcedure")
        || value.eq_ignore_ascii_case("КонецПроцедуры")
        || value.eq_ignore_ascii_case("EndFunction")
        || value.eq_ignore_ascii_case("КонецФункции")
}

fn split_statements(source: &str, tokens: &[Token]) -> Vec<Statement> {
    let mut statements = Vec::new();
    let mut start = 0_usize;
    let mut depth = 0_usize;

    for (position, token) in tokens.iter().enumerate() {
        match token.kind {
            TokenKind::LeftParenthesis => depth += 1,
            TokenKind::RightParenthesis => depth = depth.saturating_sub(1),
            TokenKind::Semicolon if depth == 0 => {
                push_statement(source, &tokens[start..=position], &mut statements);
                start = position + 1;
            }
            TokenKind::Newline if depth == 0 => {
                push_statement(source, &tokens[start..position], &mut statements);
                start = position + 1;
            }
            TokenKind::Newline
                if contains_open_form(&tokens[start..position])
                    && starts_recovery_boundary(&tokens[position + 1..]) =>
            {
                push_statement(source, &tokens[start..position], &mut statements);
                start = position + 1;
                depth = 0;
            }
            _ => {}
        }
    }

    push_statement(source, &tokens[start..], &mut statements);
    statements
}

fn push_statement(source: &str, tokens: &[Token], statements: &mut Vec<Statement>) {
    let Some(first) = tokens
        .iter()
        .find(|token| !matches!(token.kind, TokenKind::Newline))
    else {
        return;
    };
    let Some(last) = tokens
        .iter()
        .rfind(|token| !matches!(token.kind, TokenKind::Newline))
    else {
        return;
    };
    statements.push(Statement {
        tokens: tokens.to_vec(),
        raw: source[first.start..last.end].to_owned(),
    });
}

fn contains_open_form(tokens: &[Token]) -> bool {
    tokens
        .iter()
        .any(|token| identifier(token) == Some("OpenForm"))
}

fn starts_recovery_boundary(tokens: &[Token]) -> bool {
    let first = tokens
        .iter()
        .skip_while(|token| matches!(token.kind, TokenKind::Newline))
        .take_while(|token| !matches!(token.kind, TokenKind::Newline))
        .find_map(identifier);
    first.is_some_and(|value| {
        is_procedure(value) || is_function(value) || is_scope_end(value) || value == "OpenForm"
    })
}

fn tokenize(source: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut cursor = 0_usize;
    let mut line = 1_usize;
    let mut column = 1_usize;

    while cursor < source.len() {
        let character = source[cursor..]
            .chars()
            .next()
            .expect("cursor must be at a character boundary");
        let width = character.len_utf8();
        if character == '\r' || character == '\n' {
            tokens.push(scan_newline(source, &mut cursor, &mut line, &mut column));
            continue;
        }
        if character.is_whitespace() {
            cursor += width;
            column += 1;
            continue;
        }
        if character == '/' && source[cursor + width..].starts_with('/') {
            skip_comment(source, &mut cursor, &mut column);
            continue;
        }
        if character == '"' {
            tokens.push(scan_string(source, &mut cursor, &mut line, &mut column));
            continue;
        }
        if is_identifier_start(character) {
            tokens.push(scan_identifier(source, &mut cursor, line, &mut column));
            continue;
        }
        let kind = match character {
            '(' => TokenKind::LeftParenthesis,
            ')' => TokenKind::RightParenthesis,
            ',' => TokenKind::Comma,
            ';' => TokenKind::Semicolon,
            _ => TokenKind::Other,
        };
        tokens.push(Token {
            kind,
            start: cursor,
            end: cursor + width,
            line,
            column,
        });
        cursor += width;
        column += 1;
    }
    tokens
}

fn scan_newline(source: &str, cursor: &mut usize, line: &mut usize, column: &mut usize) -> Token {
    let start = *cursor;
    let start_line = *line;
    let start_column = *column;
    let character = source[*cursor..]
        .chars()
        .next()
        .expect("cursor must be at a character boundary");
    let width = character.len_utf8();
    if character == '\r' && source[*cursor + width..].starts_with('\n') {
        *cursor += width + '\n'.len_utf8();
    } else {
        *cursor += width;
    }
    *line += 1;
    *column = 1;
    Token {
        kind: TokenKind::Newline,
        start,
        end: *cursor,
        line: start_line,
        column: start_column,
    }
}

fn skip_comment(source: &str, cursor: &mut usize, column: &mut usize) {
    while *cursor < source.len() {
        let current = source[*cursor..]
            .chars()
            .next()
            .expect("cursor must be at a character boundary");
        if current == '\r' || current == '\n' {
            break;
        }
        *cursor += current.len_utf8();
        *column += 1;
    }
}

fn scan_string(source: &str, cursor: &mut usize, line: &mut usize, column: &mut usize) -> Token {
    let start = *cursor;
    let start_line = *line;
    let start_column = *column;
    let mut value = String::new();
    let mut terminated = false;
    *cursor += '"'.len_utf8();
    *column += 1;

    while *cursor < source.len() {
        let current = source[*cursor..]
            .chars()
            .next()
            .expect("cursor must be at a character boundary");
        let width = current.len_utf8();
        if current == '"' {
            if source[*cursor + width..].starts_with('"') {
                value.push('"');
                *cursor += width * 2;
                *column += 2;
                continue;
            }
            *cursor += width;
            *column += 1;
            terminated = true;
            break;
        }
        if current == '\r' || current == '\n' {
            let _newline = scan_newline(source, cursor, line, column);
            value.push('\n');
            continue;
        }
        value.push(current);
        *cursor += width;
        *column += 1;
    }
    Token {
        kind: TokenKind::StringLiteral { value, terminated },
        start,
        end: *cursor,
        line: start_line,
        column: start_column,
    }
}

fn scan_identifier(source: &str, cursor: &mut usize, line: usize, column: &mut usize) -> Token {
    let start = *cursor;
    let start_column = *column;
    while *cursor < source.len() {
        let current = source[*cursor..]
            .chars()
            .next()
            .expect("cursor must be at a character boundary");
        if !is_identifier_continue(current) {
            break;
        }
        *cursor += current.len_utf8();
        *column += 1;
    }
    Token {
        kind: TokenKind::Identifier(source[start..*cursor].to_owned()),
        start,
        end: *cursor,
        line,
        column: start_column,
    }
}

fn is_identifier_start(character: char) -> bool {
    character == '_' || character.is_alphabetic()
}

fn is_identifier_continue(character: char) -> bool {
    character == '_' || character.is_alphanumeric()
}

#[cfg(test)]
mod tests {
    use super::{
        EdtFormNavigationCandidate, EdtFormNavigationOutcomeKind, EdtFormNavigationParseOutcome,
        EdtFormNavigationRejectionReason, EdtFormNavigationTarget,
        EdtFormNavigationUnsupportedTargetReason, extract_form_navigation_candidates,
    };
    use crate::{EdtModuleDescriptor, EdtModuleKind};
    use oneagent_common::{EntityId, EntityName};
    use oneagent_metadata::MetadataKind;
    use std::fs;
    use std::path::{Path, PathBuf};

    fn module(kind: EdtModuleKind, path: PathBuf) -> EdtModuleDescriptor {
        EdtModuleDescriptor::new(
            EntityId::new("command-owner:command_module").expect("module id must be valid"),
            EntityName::new("CommandModule").expect("module name must be valid"),
            kind,
            path,
        )
    }

    fn command_module() -> EdtModuleDescriptor {
        module(EdtModuleKind::Command, PathBuf::from("CommandModule.bsl"))
    }

    fn repository_source(relative: &str) -> (EdtModuleDescriptor, String) {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../OneAgent_EDTproject/src")
            .join(relative);
        let source = fs::read_to_string(&path).expect("repository source must be readable");
        (module(EdtModuleKind::Command, path), source)
    }

    fn candidates(outcomes: &[EdtFormNavigationParseOutcome]) -> Vec<&EdtFormNavigationCandidate> {
        outcomes
            .iter()
            .filter_map(|outcome| match outcome {
                EdtFormNavigationParseOutcome::Candidate(candidate) => Some(candidate.as_ref()),
                EdtFormNavigationParseOutcome::Rejected(_) => None,
            })
            .collect()
    }

    fn reasons(
        outcomes: &[EdtFormNavigationParseOutcome],
    ) -> Vec<EdtFormNavigationRejectionReason> {
        outcomes
            .iter()
            .filter_map(|outcome| match outcome {
                EdtFormNavigationParseOutcome::Candidate(_) => None,
                EdtFormNavigationParseOutcome::Rejected(rejection) => Some(rejection.reason),
            })
            .collect()
    }

    #[test]
    fn reads_exact_repository_subordinate_and_common_form_calls() {
        let (subordinate_module, subordinate_source) = repository_source(
            "Catalogs/CounterpartiesProducts/Commands/CounterpartiesProductsPriceImport/CommandModule.bsl",
        );
        let subordinate =
            extract_form_navigation_candidates(&subordinate_module, &subordinate_source);
        let (common_module, common_source) =
            repository_source("CommonCommands/RelatedDocuments/CommandModule.bsl");
        let common = extract_form_navigation_candidates(&common_module, &common_source);

        let subordinate_candidates = candidates(&subordinate);
        let [subordinate] = subordinate_candidates.as_slice() else {
            panic!("exact subordinate source must yield one candidate");
        };
        assert_eq!(
            subordinate.literal,
            "Catalog.CounterpartiesProducts.Form.PriceImport"
        );
        assert_eq!(subordinate.procedure_name.as_str(), "CommandProcessing");
        assert_eq!(subordinate.location.line, 5);
        assert_eq!(subordinate.location.column, 2);
        assert_eq!(subordinate.module_id, *subordinate_module.id());
        assert_eq!(subordinate.module_path, subordinate_module.path());
        assert_eq!(
            subordinate.procedure_id.as_str(),
            "command-owner:command_module:procedure:CommandProcessing"
        );
        assert!(matches!(
            &subordinate.target,
            EdtFormNavigationTarget::SubordinateForm {
                owner_kind: MetadataKind::Catalog,
                owner_name,
                form_name,
            } if owner_name.as_str() == "CounterpartiesProducts"
                && form_name.as_str() == "PriceImport"
        ));

        let common_candidates = candidates(&common);
        let [common] = common_candidates.as_slice() else {
            panic!("exact Common Form source must yield one candidate");
        };
        assert_eq!(common.literal, "CommonForm.RelatedDocuments");
        assert_eq!(common.location.line, 20);
        assert!(matches!(
            &common.target,
            EdtFormNavigationTarget::CommonForm { form_name }
                if form_name.as_str() == "RelatedDocuments"
        ));
    }

    #[test]
    fn reads_exact_repository_multiline_first_argument() {
        let (module, source) =
            repository_source("Tasks/PerformerTask/Commands/MyTasks/CommandModule.bsl");
        let outcomes = extract_form_navigation_candidates(&module, &source);
        let candidates = candidates(&outcomes);
        let [candidate] = candidates.as_slice() else {
            panic!("multiline exact source must yield one candidate");
        };

        assert_eq!(candidate.location.line, 16);
        assert_eq!(candidate.literal, "Task.PerformerTask.Form.MyTasks");
        assert!(
            candidate
                .raw_statement
                .contains("CommandExecuteParameters.Window")
        );
    }

    #[test]
    fn repository_dynamic_default_shorthand_and_prefix_cases_are_typed() {
        let cases = [
            (
                "CommonCommands/AccessRights/CommandModule.bsl",
                EdtFormNavigationRejectionReason::DynamicFirstArgument,
            ),
            (
                "CommonCommands/BusinessProcessFlowchart/CommandModule.bsl",
                EdtFormNavigationRejectionReason::UnsupportedTarget(
                    EdtFormNavigationUnsupportedTargetReason::DefaultFormAlias,
                ),
            ),
            (
                "CommonCommands/OpenListOfSuppliers/CommandModule.bsl",
                EdtFormNavigationRejectionReason::UnsupportedTarget(
                    EdtFormNavigationUnsupportedTargetReason::ShorthandForm,
                ),
            ),
            (
                "ExchangePlans/MasterDataManagement/Commands/ExchangeNodes/CommandModule.bsl",
                EdtFormNavigationRejectionReason::UnsupportedTarget(
                    EdtFormNavigationUnsupportedTargetReason::InvalidTargetShape,
                ),
            ),
        ];

        for (relative, expected) in cases {
            let (module, source) = repository_source(relative);
            let outcomes = extract_form_navigation_candidates(&module, &source);
            assert!(candidates(&outcomes).is_empty(), "{relative}");
            assert_eq!(reasons(&outcomes), vec![expected], "{relative}");
        }
    }

    #[test]
    fn maps_the_exact_supported_owner_kind_allowlist() {
        let cases = [
            ("Catalog", MetadataKind::Catalog),
            ("Document", MetadataKind::Document),
            ("Report", MetadataKind::Report),
            ("DataProcessor", MetadataKind::DataProcessor),
            ("InformationRegister", MetadataKind::InformationRegister),
            ("AccumulationRegister", MetadataKind::AccumulationRegister),
            ("AccountingRegister", MetadataKind::AccountingRegister),
            ("CalculationRegister", MetadataKind::CalculationRegister),
            ("BusinessProcess", MetadataKind::BusinessProcess),
            ("Task", MetadataKind::Task),
        ];
        let statements = cases
            .iter()
            .map(|(prefix, _)| format!("OpenForm(\"{prefix}.Owner.Form.Target\");"))
            .collect::<Vec<_>>()
            .join("\n");
        let source = format!("Procedure OpenAll()\n{statements}\nEndProcedure");
        let outcomes = extract_form_navigation_candidates(&command_module(), &source);
        let parsed = candidates(&outcomes);

        assert_eq!(parsed.len(), cases.len());
        for (candidate, (_, expected_kind)) in parsed.into_iter().zip(cases) {
            assert!(matches!(
                candidate.target,
                EdtFormNavigationTarget::SubordinateForm { owner_kind, .. }
                    if owner_kind == expected_kind
            ));
        }
    }

    #[test]
    fn wrong_module_and_callable_contexts_are_typed() {
        let statement = "OpenForm(\"CommonForm.Target\");";
        let wrong_module_source = format!("Procedure Run()\n{statement}\nEndProcedure");
        let wrong_module = extract_form_navigation_candidates(
            &module(EdtModuleKind::Form, PathBuf::from("Module.bsl")),
            &wrong_module_source,
        );
        let function_source = format!("Function Run()\n{statement}\nEndFunction");
        let function = extract_form_navigation_candidates(&command_module(), &function_source);
        let missing = extract_form_navigation_candidates(&command_module(), statement);

        assert_eq!(
            wrong_module[0].kind(),
            EdtFormNavigationOutcomeKind::WrongModule
        );
        assert_eq!(
            function[0].kind(),
            EdtFormNavigationOutcomeKind::WrongCallable
        );
        assert_eq!(
            missing[0].kind(),
            EdtFormNavigationOutcomeKind::WrongCallable
        );
        let EdtFormNavigationParseOutcome::Rejected(function) = &function[0] else {
            panic!("Function call must be rejected");
        };
        assert_eq!(
            function.containing_symbol_kind,
            Some(oneagent_bsl::BslSymbolKind::Function)
        );
        assert_eq!(
            function
                .containing_symbol_name
                .as_ref()
                .map(EntityName::as_str),
            Some("Run")
        );
        assert!(function.containing_symbol_id.is_some());
        assert_eq!(function.module_kind, EdtModuleKind::Command);
    }

    #[test]
    fn malformed_dynamic_incomplete_comments_and_strings_are_conservative() {
        let source = concat!(
            "Procedure Test()\n",
            "OpenForm \"CommonForm.Malformed\";\n",
            "OpenForm(Name);\n",
            "OpenForm(\"CommonForm.\" + Name);\n",
            "OpenForm(\"ExchangePlan.Owner.Form.Target\");\n",
            "OpenForm(\"CommonForm.Incomplete\"\n",
            "OpenForm(\"CommonForm.Recovered\");\n",
            "EndProcedure\n",
            "// OpenForm(\"CommonForm.Comment\");\n",
            "Value = \"OpenForm(\"\"CommonForm.String\"\")\";\n",
        );
        let outcomes = extract_form_navigation_candidates(&command_module(), source);

        assert_eq!(
            outcomes
                .iter()
                .map(EdtFormNavigationParseOutcome::kind)
                .collect::<Vec<_>>(),
            vec![
                EdtFormNavigationOutcomeKind::Malformed,
                EdtFormNavigationOutcomeKind::Dynamic,
                EdtFormNavigationOutcomeKind::Dynamic,
                EdtFormNavigationOutcomeKind::Unsupported,
                EdtFormNavigationOutcomeKind::Incomplete,
                EdtFormNavigationOutcomeKind::Accepted,
            ]
        );
        let candidates = candidates(&outcomes);
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].literal, "CommonForm.Recovered");
        assert!(
            reasons(&outcomes).contains(&EdtFormNavigationRejectionReason::UnsupportedTarget(
                EdtFormNavigationUnsupportedTargetReason::UnsupportedPrefix,
            ))
        );
    }

    #[test]
    fn duplicate_order_location_and_repeated_extraction_are_deterministic() {
        let source = concat!(
            "Procedure Run()\n",
            "OpenForm(\"CommonForm.Beta\");\n",
            "OpenForm(\"CommonForm.Alpha\");\n",
            "OpenForm(\"CommonForm.Beta\");\n",
            "EndProcedure\n",
        );
        let first = extract_form_navigation_candidates(&command_module(), source);
        let repeated = extract_form_navigation_candidates(&command_module(), source);
        let candidates = candidates(&first);

        assert_eq!(first, repeated);
        assert_eq!(candidates.len(), 3);
        assert_eq!(
            candidates
                .iter()
                .map(|candidate| candidate.literal.as_str())
                .collect::<Vec<_>>(),
            vec!["CommonForm.Beta", "CommonForm.Alpha", "CommonForm.Beta"]
        );
        assert_eq!(
            candidates
                .iter()
                .map(|candidate| candidate.location.line)
                .collect::<Vec<_>>(),
            vec![2, 3, 4]
        );
        assert!(
            candidates
                .iter()
                .all(|candidate| candidate.location.column == 1)
        );
    }
}
