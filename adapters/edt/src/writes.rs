//! Private candidate parsing for the first EDT Writes slice.

use oneagent_bsl::BslSymbolKind;
use oneagent_common::{EntityId, EntityName};
use oneagent_metadata::MetadataKind;
use std::path::PathBuf;

use crate::{EdtMetadataObjectDescriptor, EdtModuleDescriptor, EdtModuleKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum EdtWritesParseOutcome {
    Candidate(Box<EdtWritesCandidate>),
    Rejected(EdtWritesRejection),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EdtWritesCandidate {
    pub(crate) owner_id: EntityId,
    pub(crate) owner_name: EntityName,
    pub(crate) module_id: EntityId,
    pub(crate) module_path: PathBuf,
    pub(crate) procedure_id: EntityId,
    pub(crate) procedure_name: EntityName,
    pub(crate) raw_statement: String,
    pub(crate) receiver_spelling: String,
    pub(crate) local_name: String,
    pub(crate) method_spelling: String,
    pub(crate) lookup_key: String,
    pub(crate) zero_arguments: bool,
    pub(crate) complete_statement: bool,
    pub(crate) location: EdtWritesSourceLocation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EdtWritesRejection {
    pub(crate) raw_statement: String,
    pub(crate) location: EdtWritesSourceLocation,
    pub(crate) reason: EdtWritesRejectionReason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EdtWritesRejectionReason {
    MalformedOrIncompleteStatement,
    ExpressionRemainder,
    ComputedReceiver,
    ExtraReceiverComponents,
    CollectionLevelWrite,
    RequiresValueFlow,
    UnsupportedReceiver,
    NonEmptyArguments,
    MissingContainingSymbol,
    UnsupportedContainingSymbol(BslSymbolKind),
    UnsupportedModuleKind(EdtModuleKind),
    UnsupportedOwnerKind(MetadataKind),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EdtWritesSourceLocation {
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
    location: EdtWritesSourceLocation,
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
    Dot,
    LeftParenthesis,
    RightParenthesis,
    Semicolon,
    Newline,
    Other,
}

/// Extracts ordered private candidate and rejection outcomes from one BSL source artifact.
pub(crate) fn extract_writes_candidates(
    owner: &EdtMetadataObjectDescriptor,
    module: &EdtModuleDescriptor,
    source: &str,
) -> Vec<EdtWritesParseOutcome> {
    let statements = split_statements(source, &tokenize(source));
    let mut containing_symbol = None;
    let mut outcomes = Vec::new();

    for statement in statements {
        if let Some(transition) = scope_transition(&statement, module) {
            match transition {
                ScopeTransition::Start(symbol) => containing_symbol = Some(symbol),
                ScopeTransition::End => containing_symbol = None,
            }
            continue;
        }

        let significant = significant_tokens(&statement.tokens);
        for (position, token) in significant.iter().enumerate() {
            if !identifier_equals(token, "Write")
                || !matches!(
                    significant.get(position + 1).map(|token| &token.kind),
                    Some(TokenKind::LeftParenthesis)
                )
            {
                continue;
            }

            outcomes.push(classify_write(
                owner,
                module,
                containing_symbol.as_ref(),
                &statement,
                &significant,
                position,
            ));
        }
    }

    outcomes
}

fn classify_write(
    owner: &EdtMetadataObjectDescriptor,
    module: &EdtModuleDescriptor,
    containing_symbol: Option<&ContainingSymbol>,
    statement: &Statement,
    tokens: &[&Token],
    method_position: usize,
) -> EdtWritesParseOutcome {
    let open_parenthesis = method_position + 1;
    let Some(close_parenthesis) = matching_parenthesis(tokens, open_parenthesis) else {
        return rejection(
            statement,
            EdtWritesRejectionReason::MalformedOrIncompleteStatement,
        );
    };

    let Some(chain) = receiver_chain(tokens, method_position) else {
        let reason =
            if method_position >= 2 && matches!(tokens[method_position - 1].kind, TokenKind::Dot) {
                EdtWritesRejectionReason::ComputedReceiver
            } else {
                EdtWritesRejectionReason::UnsupportedReceiver
            };

        return rejection(statement, reason);
    };

    if has_expression_remainder(tokens, chain.start, close_parenthesis) {
        return rejection(statement, EdtWritesRejectionReason::ExpressionRemainder);
    }

    let receiver_components = &chain.components[..chain.components.len() - 1];

    if receiver_components.len() > 2 {
        return rejection(statement, EdtWritesRejectionReason::ExtraReceiverComponents);
    }

    if receiver_components.len() == 1
        && receiver_components[0].eq_ignore_ascii_case("RegisterRecords")
    {
        return rejection(statement, EdtWritesRejectionReason::CollectionLevelWrite);
    }

    if receiver_components.len() == 1 {
        return rejection(statement, EdtWritesRejectionReason::RequiresValueFlow);
    }

    if receiver_components.len() != 2
        || !receiver_components[0].eq_ignore_ascii_case("RegisterRecords")
    {
        return rejection(statement, EdtWritesRejectionReason::UnsupportedReceiver);
    }

    if close_parenthesis > open_parenthesis + 1 {
        return rejection(statement, EdtWritesRejectionReason::NonEmptyArguments);
    }

    let Some(symbol) = containing_symbol else {
        return rejection(statement, EdtWritesRejectionReason::MissingContainingSymbol);
    };

    if symbol.kind != BslSymbolKind::Procedure {
        return rejection(
            statement,
            EdtWritesRejectionReason::UnsupportedContainingSymbol(symbol.kind),
        );
    }

    if module.kind() != EdtModuleKind::Object {
        return rejection(
            statement,
            EdtWritesRejectionReason::UnsupportedModuleKind(module.kind()),
        );
    }

    if owner.kind() != MetadataKind::Document {
        return rejection(
            statement,
            EdtWritesRejectionReason::UnsupportedOwnerKind(owner.kind()),
        );
    }

    let local_name = receiver_components[1].clone();

    EdtWritesParseOutcome::Candidate(Box::new(EdtWritesCandidate {
        owner_id: owner.id().clone(),
        owner_name: owner.name().clone(),
        module_id: module.id().clone(),
        module_path: module.path().to_path_buf(),
        procedure_id: symbol.id.clone(),
        procedure_name: symbol.name.clone(),
        raw_statement: statement.raw.clone(),
        receiver_spelling: receiver_components[0].clone(),
        local_name: local_name.clone(),
        method_spelling: chain
            .components
            .last()
            .expect("method component must exist")
            .clone(),
        lookup_key: local_name.to_lowercase(),
        zero_arguments: true,
        complete_statement: true,
        location: statement.location,
    }))
}

fn rejection(statement: &Statement, reason: EdtWritesRejectionReason) -> EdtWritesParseOutcome {
    EdtWritesParseOutcome::Rejected(EdtWritesRejection {
        raw_statement: statement.raw.clone(),
        location: statement.location,
        reason,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ReceiverChain {
    start: usize,
    components: Vec<String>,
}

fn receiver_chain(tokens: &[&Token], method_position: usize) -> Option<ReceiverChain> {
    let mut position = method_position;
    let mut components = vec![identifier(tokens[method_position])?.to_owned()];

    loop {
        if position < 2 || !matches!(tokens[position - 1].kind, TokenKind::Dot) {
            break;
        }

        let receiver = identifier(tokens[position - 2])?;
        components.push(receiver.to_owned());
        position -= 2;
    }

    if components.len() == 1 {
        return None;
    }

    components.reverse();

    Some(ReceiverChain {
        start: position,
        components,
    })
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

fn has_expression_remainder(
    tokens: &[&Token],
    chain_start: usize,
    close_parenthesis: usize,
) -> bool {
    if chain_start != 0 {
        return true;
    }

    match tokens.get(close_parenthesis + 1..) {
        None | Some([]) => false,
        Some([token]) => !matches!(token.kind, TokenKind::Semicolon),
        Some(_) => true,
    }
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

    let raw_name = tokens.get(1).and_then(|token| identifier(token))?;
    if !matches!(
        tokens.get(2).map(|token| &token.kind),
        Some(TokenKind::LeftParenthesis)
    ) {
        return None;
    }

    let name = EntityName::new(raw_name).ok()?;
    let id = EntityId::new(format!(
        "{}:{}:{}",
        module.id().as_str(),
        kind.as_str(),
        raw_name
    ))
    .ok()?;

    Some(ScopeTransition::Start(ContainingSymbol { id, name, kind }))
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ScopeTransition {
    Start(ContainingSymbol),
    End,
}

fn is_procedure(value: &str) -> bool {
    value.eq_ignore_ascii_case("Procedure") || value.to_lowercase() == "процедура"
}

fn is_function(value: &str) -> bool {
    value.eq_ignore_ascii_case("Function") || value.to_lowercase() == "функция"
}

fn is_scope_end(value: &str) -> bool {
    value.eq_ignore_ascii_case("EndProcedure")
        || value.eq_ignore_ascii_case("EndFunction")
        || matches!(
            value.to_lowercase().as_str(),
            "конецпроцедуры" | "конецфункции"
        )
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

fn identifier_equals(token: &Token, expected: &str) -> bool {
    identifier(token).is_some_and(|value| value.eq_ignore_ascii_case(expected))
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
                if contains_write_call(&tokens[start..position])
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
        location: EdtWritesSourceLocation {
            line: first.line,
            column: first.column,
        },
    });
}

fn contains_write_call(tokens: &[Token]) -> bool {
    tokens.windows(2).any(|window| {
        identifier_equals(&window[0], "Write")
            && matches!(window[1].kind, TokenKind::LeftParenthesis)
    })
}

fn starts_recovery_boundary(tokens: &[Token]) -> bool {
    let significant = tokens
        .iter()
        .skip_while(|token| matches!(token.kind, TokenKind::Newline))
        .take_while(|token| !matches!(token.kind, TokenKind::Newline))
        .collect::<Vec<_>>();

    let Some(first) = significant.first().and_then(|token| identifier(token)) else {
        return false;
    };

    is_procedure(first)
        || is_function(first)
        || is_scope_end(first)
        || first.eq_ignore_ascii_case("RegisterRecords")
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
            tokens.push(scan_comment(source, &mut cursor, line, &mut column));
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
            '.' => TokenKind::Dot,
            '(' => TokenKind::LeftParenthesis,
            ')' => TokenKind::RightParenthesis,
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

fn scan_comment(source: &str, cursor: &mut usize, line: usize, column: &mut usize) -> Token {
    let start = *cursor;
    let start_column = *column;

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

    Token {
        kind: TokenKind::Other,
        start,
        end: *cursor,
        line,
        column: start_column,
    }
}

fn scan_string(source: &str, cursor: &mut usize, line: &mut usize, column: &mut usize) -> Token {
    let start = *cursor;
    let start_line = *line;
    let start_column = *column;
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
                *cursor += width * 2;
                *column += 2;
                continue;
            }
            *cursor += width;
            *column += 1;
            break;
        }

        if current == '\r' || current == '\n' {
            let _newline = scan_newline(source, cursor, line, column);
            continue;
        }

        *cursor += width;
        *column += 1;
    }

    Token {
        kind: TokenKind::Other,
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
        EdtWritesCandidate, EdtWritesParseOutcome, EdtWritesRejectionReason,
        extract_writes_candidates,
    };
    use crate::{
        EdtMetadataObjectDescriptor, EdtMetadataObjectReader, EdtModuleDescriptor, EdtModuleKind,
        EdtModuleReader, FileSystemEdtMetadataObjectReader, FileSystemEdtModuleReader,
    };
    use oneagent_common::{EntityId, EntityName};
    use oneagent_metadata::MetadataKind;
    use std::fs;
    use std::path::{Path, PathBuf};

    const CORPUS_FIXTURES: [&str; 16] = [
        "aliased_register_record_set.bsl",
        "archive_file_write.bsl",
        "argument_bearing_information_register.bsl",
        "async_scope_write.bsl",
        "binary_file_write.bsl",
        "chained_common_module_receiver.bsl",
        "chained_manager_receiver.bsl",
        "collection_level_write.bsl",
        "comment_only_write.bsl",
        "computed_receiver_write.bsl",
        "external_input_file_write.bsl",
        "local_document_value_flow.bsl",
        "local_predefined_item_value_flow.bsl",
        "property_assignment_and_call.bsl",
        "text_file_write.bsl",
        "ui_form_write.bsl",
    ];

    fn id(value: &str) -> EntityId {
        EntityId::new(value).expect("identifier must be valid")
    }

    fn name(value: &str) -> EntityName {
        EntityName::new(value).expect("name must be valid")
    }

    fn owner(kind: MetadataKind) -> EdtMetadataObjectDescriptor {
        EdtMetadataObjectDescriptor::new(
            id("document-owner"),
            name("DocumentOwner"),
            None,
            kind,
            None,
            PathBuf::from("DocumentOwner.mdo"),
        )
    }

    fn module(kind: EdtModuleKind) -> EdtModuleDescriptor {
        EdtModuleDescriptor::new(
            id("document-owner:object_module"),
            name("ObjectModule"),
            kind,
            PathBuf::from("ObjectModule.bsl"),
        )
    }

    fn candidates(outcomes: &[EdtWritesParseOutcome]) -> Vec<&EdtWritesCandidate> {
        outcomes
            .iter()
            .filter_map(|outcome| match outcome {
                EdtWritesParseOutcome::Candidate(candidate) => Some(candidate.as_ref()),
                EdtWritesParseOutcome::Rejected(_) => None,
            })
            .collect()
    }

    fn reasons(outcomes: &[EdtWritesParseOutcome]) -> Vec<EdtWritesRejectionReason> {
        outcomes
            .iter()
            .filter_map(|outcome| match outcome {
                EdtWritesParseOutcome::Candidate(_) => None,
                EdtWritesParseOutcome::Rejected(rejection) => Some(rejection.reason),
            })
            .collect()
    }

    fn writes_project_document_directory() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/writes_project/src/Documents/RefundOfPaymentByOrder")
    }

    fn corpus_directory() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../crates/bsl/tests/fixtures/writes")
    }

    #[test]
    fn repository_backed_writes_project_yields_two_ordered_candidates() {
        let object_directory = writes_project_document_directory();
        let owner = FileSystemEdtMetadataObjectReader
            .read(&object_directory, MetadataKind::Document)
            .expect("fixture Document must load");
        let module = FileSystemEdtModuleReader
            .read_modules(owner.id(), owner.name(), &object_directory)
            .expect("fixture modules must load")
            .into_iter()
            .find(|module| module.kind() == EdtModuleKind::Object)
            .expect("fixture Object Module must exist");
        let source = fs::read_to_string(module.path()).expect("fixture source must load");

        let outcomes = extract_writes_candidates(&owner, &module, &source);
        let candidates = candidates(&outcomes);

        assert_eq!(outcomes.len(), 2);
        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[0].local_name, "CashAccountBalance");
        assert_eq!(candidates[1].local_name, "RefundBankPayment");
        assert_eq!(candidates[0].lookup_key, "cashaccountbalance");
        assert_eq!(candidates[1].lookup_key, "refundbankpayment");
        assert_eq!(candidates[0].location.line, 2);
        assert_eq!(candidates[1].location.line, 3);
        assert_eq!(candidates[0].owner_id, *owner.id());
        assert_eq!(candidates[0].owner_name, *owner.name());
        assert_eq!(candidates[0].module_id, *module.id());
        assert_eq!(candidates[0].module_path, module.path());
        assert_eq!(candidates[0].procedure_name.as_str(), "Posting");
        assert_eq!(
            candidates[0].procedure_id.as_str(),
            format!("{}:procedure:Posting", module.id().as_str())
        );
        assert_eq!(
            candidates[0].raw_statement,
            "RegisterRecords.CashAccountBalance.Write();"
        );
        assert_eq!(candidates[0].receiver_spelling, "RegisterRecords");
        assert_eq!(candidates[0].method_spelling, "Write");
        assert!(candidates[0].zero_arguments);
        assert!(candidates[0].complete_statement);
    }

    #[test]
    fn repository_backed_boundary_corpus_never_promotes_unsupported_writes() {
        let owner = owner(MetadataKind::Document);
        let module = module(EdtModuleKind::Object);

        for fixture in CORPUS_FIXTURES {
            if fixture == "property_assignment_and_call.bsl" {
                continue;
            }
            let source = fs::read_to_string(corpus_directory().join(fixture))
                .expect("corpus fixture must load");
            let outcomes = extract_writes_candidates(&owner, &module, &source);

            assert!(
                candidates(&outcomes).is_empty(),
                "{fixture} must not yield an eligible candidate"
            );
        }
    }

    #[test]
    fn repository_backed_argument_write_is_typed_as_non_empty() {
        let source = fs::read_to_string(
            corpus_directory().join("argument_bearing_information_register.bsl"),
        )
        .expect("argument fixture must load");
        let outcomes = extract_writes_candidates(
            &owner(MetadataKind::Document),
            &module(EdtModuleKind::Object),
            &source,
        );

        assert!(candidates(&outcomes).is_empty());
        assert_eq!(
            reasons(&outcomes),
            vec![EdtWritesRejectionReason::NonEmptyArguments]
        );
        let rejection = match &outcomes[0] {
            EdtWritesParseOutcome::Rejected(rejection) => rejection,
            EdtWritesParseOutcome::Candidate(_) => panic!("write must be rejected"),
        };
        assert_eq!(rejection.location.line, 14);
    }

    #[test]
    fn repository_backed_comment_only_fixture_has_no_outcome() {
        let source = fs::read_to_string(corpus_directory().join("comment_only_write.bsl"))
            .expect("comment fixture must load");

        assert!(
            extract_writes_candidates(
                &owner(MetadataKind::Document),
                &module(EdtModuleKind::Object),
                &source,
            )
            .is_empty()
        );
    }

    #[test]
    fn repository_backed_property_assignment_is_ignored_and_call_is_independent() {
        let fixture =
            fs::read_to_string(corpus_directory().join("property_assignment_and_call.bsl"))
                .expect("property fixture must load");
        let mut lines = fixture.lines();
        let _removed_context_line = lines.next().expect("fixture must not be empty");
        let source = std::iter::once("Procedure Posting()")
            .chain(lines)
            .collect::<Vec<_>>()
            .join("\n");

        let outcomes = extract_writes_candidates(
            &owner(MetadataKind::Document),
            &module(EdtModuleKind::Object),
            &source,
        );
        let candidates = candidates(&outcomes);

        assert_eq!(outcomes.len(), 1);
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].location.line, 13);
        assert_eq!(candidates[0].local_name, "ProductsInStorageBins");
        assert_eq!(
            candidates[0].raw_statement,
            "RegisterRecords.ProductsInStorageBins.Write();"
        );
    }

    #[test]
    fn generated_lexical_contract_accepts_whitespace_semicolon_and_duplicates() {
        let source = concat!(
            "Procedure Posting()\n",
            " RegisterRecords . É . Write ( )\n",
            "\tRegisterRecords.İ.Write();\n",
            " RegisterRecords . É . Write ( ) ;\n",
            "EndProcedure\n",
        );
        let outcomes = extract_writes_candidates(
            &owner(MetadataKind::Document),
            &module(EdtModuleKind::Object),
            source,
        );
        let candidates = candidates(&outcomes);

        assert_eq!(candidates.len(), 3);
        assert_eq!(candidates[0].lookup_key, "é");
        assert_eq!(candidates[1].lookup_key, "i\u{307}");
        assert_eq!(candidates[2].lookup_key, "é");
        assert_ne!(candidates[0].lookup_key, candidates[1].lookup_key);
        assert_eq!(
            candidates
                .iter()
                .map(|candidate| candidate.location.line)
                .collect::<Vec<_>>(),
            vec![2, 3, 4]
        );
    }

    #[test]
    fn generated_lexical_contract_ignores_empty_comment_string_and_property_input() {
        let source = concat!(
            "// RegisterRecords.CommentOnly.Write();\n",
            "Procedure Posting()\n",
            " Payload = \"RegisterRecords.StringOnly.Write();\";\n",
            " RegisterRecords.PropertyOnly.Write = True;\n",
            "EndProcedure\n",
        );

        assert!(
            extract_writes_candidates(
                &owner(MetadataKind::Document),
                &module(EdtModuleKind::Object),
                source,
            )
            .is_empty()
        );
        assert!(
            extract_writes_candidates(
                &owner(MetadataKind::Document),
                &module(EdtModuleKind::Object),
                "",
            )
            .is_empty()
        );
    }

    #[test]
    fn generated_context_contract_distinguishes_scope_module_and_owner_rejections() {
        let exact_statement = "RegisterRecords.Stock.Write();";
        let missing = extract_writes_candidates(
            &owner(MetadataKind::Document),
            &module(EdtModuleKind::Object),
            exact_statement,
        );
        let function = extract_writes_candidates(
            &owner(MetadataKind::Document),
            &module(EdtModuleKind::Object),
            &format!("Function ReadStock()\n{exact_statement}\nEndFunction"),
        );
        let manager = extract_writes_candidates(
            &owner(MetadataKind::Document),
            &module(EdtModuleKind::Manager),
            &format!("Procedure Posting()\n{exact_statement}\nEndProcedure"),
        );
        let catalog = extract_writes_candidates(
            &owner(MetadataKind::Catalog),
            &module(EdtModuleKind::Object),
            &format!("Procedure Posting()\n{exact_statement}\nEndProcedure"),
        );

        assert_eq!(
            reasons(&missing),
            vec![EdtWritesRejectionReason::MissingContainingSymbol]
        );
        assert_eq!(
            reasons(&function),
            vec![EdtWritesRejectionReason::UnsupportedContainingSymbol(
                oneagent_bsl::BslSymbolKind::Function
            )]
        );
        assert_eq!(
            reasons(&manager),
            vec![EdtWritesRejectionReason::UnsupportedModuleKind(
                EdtModuleKind::Manager
            )]
        );
        assert_eq!(
            reasons(&catalog),
            vec![EdtWritesRejectionReason::UnsupportedOwnerKind(
                MetadataKind::Catalog
            )]
        );
    }

    #[test]
    fn generated_rejection_precedence_is_deterministic_and_private() {
        let source = concat!(
            "Procedure Posting()\n",
            " Value = Recorder.RegisterRecords.Stock.Write(Force) + 1;\n",
            " Recorder.RegisterRecords.Stock.Write();\n",
            " RegisterRecords.Write();\n",
            " LocalRecordSet.Write();\n",
            " Module.Factory().Write();\n",
            " RegisterRecords.Stock.Write(Force);\n",
            "EndProcedure\n",
        );
        let outcomes = extract_writes_candidates(
            &owner(MetadataKind::Catalog),
            &module(EdtModuleKind::Manager),
            source,
        );

        assert_eq!(
            reasons(&outcomes),
            vec![
                EdtWritesRejectionReason::ExpressionRemainder,
                EdtWritesRejectionReason::ExtraReceiverComponents,
                EdtWritesRejectionReason::CollectionLevelWrite,
                EdtWritesRejectionReason::RequiresValueFlow,
                EdtWritesRejectionReason::ComputedReceiver,
                EdtWritesRejectionReason::NonEmptyArguments,
            ]
        );
    }

    #[test]
    fn generated_malformed_statement_recovers_before_independent_candidate() {
        let source = concat!(
            "Procedure Posting()\n",
            " RegisterRecords.Broken.Write(\n",
            " RegisterRecords.Valid.Write();\n",
            "EndProcedure\n",
        );
        let outcomes = extract_writes_candidates(
            &owner(MetadataKind::Document),
            &module(EdtModuleKind::Object),
            source,
        );

        assert_eq!(outcomes.len(), 2);
        assert_eq!(
            reasons(&outcomes),
            vec![EdtWritesRejectionReason::MalformedOrIncompleteStatement]
        );
        assert_eq!(candidates(&outcomes)[0].local_name, "Valid");
        assert_eq!(candidates(&outcomes)[0].location.line, 3);
    }
}
