//! Minimum typed parsing for repository-backed 1C query-language forms.

/// Half-open location inside the raw UTF-8 query text supplied to the parser.
///
/// Both bounds are zero-based byte offsets. The parser does not normalize input,
/// so whitespace and line endings contribute their original UTF-8 byte lengths.
/// Token boundaries always coincide with UTF-8 character boundaries. This range
/// does not describe or imply a mapping back to a BSL string literal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct QueryTextRange {
    start_byte: usize,
    end_byte: usize,
}

impl QueryTextRange {
    const fn new(start_byte: usize, end_byte: usize) -> Self {
        Self {
            start_byte,
            end_byte,
        }
    }

    /// Returns the inclusive zero-based UTF-8 byte offset at which the range starts.
    #[must_use]
    pub const fn start_byte(self) -> usize {
        self.start_byte
    }

    /// Returns the exclusive zero-based UTF-8 byte offset at which the range ends.
    #[must_use]
    pub const fn end_byte(self) -> usize {
        self.end_byte
    }
}

/// Statement categories recognized by the minimum query-language slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum QueryStatementKind {
    /// One top-level `SELECT` or evidence-backed Russian `ВЫБРАТЬ` statement.
    Select,
}

/// Normalized persistent source categories admitted by the first parser slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum QuerySourceCategory {
    /// A direct Catalog source, including the evidence-backed `Справочник` spelling.
    Catalog,
    /// A direct Information Register source.
    InformationRegister,
}

/// One direct persistent source occurrence parsed from raw query text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuerySourceOccurrence {
    raw_spelling: String,
    category: QuerySourceCategory,
    namespace: String,
    local_name: String,
    alias: Option<String>,
    location: QueryTextRange,
}

impl QuerySourceOccurrence {
    /// Returns the source spelling exactly as it appeared in the raw query text.
    #[must_use]
    pub fn raw_spelling(&self) -> &str {
        &self.raw_spelling
    }

    /// Returns the normalized persistent source category.
    #[must_use]
    pub const fn category(&self) -> QuerySourceCategory {
        self.category
    }

    /// Returns the namespace spelling exactly as it appeared in the raw query text.
    #[must_use]
    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    /// Returns the local metadata name exactly as it appeared in the raw query text.
    #[must_use]
    pub fn local_name(&self) -> &str {
        &self.local_name
    }

    /// Returns the optional source alias.
    #[must_use]
    pub fn alias(&self) -> Option<&str> {
        self.alias.as_deref()
    }

    /// Returns the source range in the raw query text.
    #[must_use]
    pub const fn location(&self) -> QueryTextRange {
        self.location
    }
}

/// Completely parsed program accepted by the minimum query-language slice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedQueryProgram {
    statement_kind: QueryStatementKind,
    sources: Vec<QuerySourceOccurrence>,
}

impl ParsedQueryProgram {
    /// Returns the parsed top-level statement category.
    #[must_use]
    pub const fn statement_kind(&self) -> QueryStatementKind {
        self.statement_kind
    }

    /// Returns direct source occurrences in raw query-text order.
    #[must_use]
    pub fn sources(&self) -> &[QuerySourceOccurrence] {
        &self.sources
    }
}

/// Typed query-language diagnostic categories produced by the minimum slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum QueryLanguageDiagnosticKind {
    /// The raw query text is empty, incomplete, invalid, or not fully consumed.
    MalformedSyntax,
    /// A qualified persistent namespace is outside the first-slice allowlist.
    UnsupportedPersistentNamespace,
    /// A parameter occupies a data-source position.
    ExternalOrParameterDataSource,
}

impl QueryLanguageDiagnosticKind {
    /// Returns the stable machine-readable diagnostic code.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MalformedSyntax => "query_language.malformed_syntax",
            Self::UnsupportedPersistentNamespace => {
                "query_language.unsupported_persistent_namespace"
            }
            Self::ExternalOrParameterDataSource => {
                "query_language.external_or_parameter_data_source"
            }
        }
    }
}

/// Structured deterministic diagnostic for raw query-language input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QueryLanguageDiagnostic {
    kind: QueryLanguageDiagnosticKind,
    message: &'static str,
    location: QueryTextRange,
}

impl QueryLanguageDiagnostic {
    const fn new(
        kind: QueryLanguageDiagnosticKind,
        message: &'static str,
        location: QueryTextRange,
    ) -> Self {
        Self {
            kind,
            message,
            location,
        }
    }

    /// Returns the typed diagnostic category.
    #[must_use]
    pub const fn kind(self) -> QueryLanguageDiagnosticKind {
        self.kind
    }

    /// Returns the stable English diagnostic message.
    #[must_use]
    pub const fn message(self) -> &'static str {
        self.message
    }

    /// Returns the diagnostic range in the raw query text.
    #[must_use]
    pub const fn location(self) -> QueryTextRange {
        self.location
    }
}

/// Result of parsing one raw query-language program.
///
/// A parsed program is exposed only when the complete first-slice source set was
/// proved. Unsupported, malformed, or unconsumed input returns no partial
/// program and sets [`Self::is_source_set_complete`] to `false`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryLanguageParseResult {
    program: Option<ParsedQueryProgram>,
    diagnostics: Vec<QueryLanguageDiagnostic>,
    complete_source_set: bool,
}

impl QueryLanguageParseResult {
    fn accepted(program: ParsedQueryProgram) -> Self {
        Self {
            program: Some(program),
            diagnostics: Vec::new(),
            complete_source_set: true,
        }
    }

    fn rejected(diagnostic: QueryLanguageDiagnostic) -> Self {
        Self {
            program: None,
            diagnostics: vec![diagnostic],
            complete_source_set: false,
        }
    }

    /// Returns the parsed program only after complete first-slice proof.
    #[must_use]
    pub const fn program(&self) -> Option<&ParsedQueryProgram> {
        self.program.as_ref()
    }

    /// Returns deterministic diagnostics in raw query-text order.
    #[must_use]
    pub fn diagnostics(&self) -> &[QueryLanguageDiagnostic] {
        &self.diagnostics
    }

    /// Returns whether the complete accepted first-slice source set was proved.
    #[must_use]
    pub const fn is_source_set_complete(&self) -> bool {
        self.complete_source_set
    }
}

/// Parser for the minimum repository-backed 1C query-language slice.
///
/// The parser recognizes only exact keyword and namespace spellings represented
/// by the current fixture corpus. It performs typed tokenization and structural
/// parsing and never discovers sources through regular expressions or substring
/// matching.
#[derive(Debug, Default, Clone, Copy)]
pub struct QueryLanguageParser;

impl QueryLanguageParser {
    /// Parses one complete raw query-language program.
    #[must_use]
    pub fn parse(&self, source: &str) -> QueryLanguageParseResult {
        let mut parser = Parser::new(source);
        match parser.parse_program() {
            Ok(program) => QueryLanguageParseResult::accepted(program),
            Err(diagnostic) => QueryLanguageParseResult::rejected(diagnostic),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum QueryLanguage {
    English,
    Russian,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TokenKind<'a> {
    Word(&'a str),
    Number,
    Dot,
    Ampersand,
    Invalid,
    EndOfInput,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Token<'a> {
    kind: TokenKind<'a>,
    location: QueryTextRange,
}

struct Parser<'a> {
    source: &'a str,
    tokens: Vec<Token<'a>>,
    cursor: usize,
}

impl<'a> Parser<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            source,
            tokens: tokenize(source),
            cursor: 0,
        }
    }

    fn parse_program(&mut self) -> Result<ParsedQueryProgram, QueryLanguageDiagnostic> {
        let language = match self.current().kind {
            TokenKind::Word("SELECT") => QueryLanguage::English,
            TokenKind::Word("ВЫБРАТЬ") => QueryLanguage::Russian,
            _ => {
                return Err(self.malformed_current(
                    "expected the exact SELECT or ВЫБРАТЬ keyword at program start",
                ));
            }
        };
        self.advance();

        if language == QueryLanguage::English && self.consume_word("TOP") {
            self.expect_number("expected a numeric TOP limit")?;
        }

        self.parse_identifier_path("expected a projection identifier")?;

        let source_keyword = match language {
            QueryLanguage::English => "FROM",
            QueryLanguage::Russian => "ИЗ",
        };
        self.expect_word(
            source_keyword,
            "expected the matching source-clause keyword",
        )?;

        let mut source = self.parse_source()?;

        if language == QueryLanguage::English && self.consume_word("AS") {
            let alias = self.expect_identifier("expected a source alias after AS")?;
            source.alias = Some(alias.to_owned());
        }

        if self.current().kind != TokenKind::EndOfInput {
            return Err(self.malformed_current("unexpected unconsumed input after the data source"));
        }

        Ok(ParsedQueryProgram {
            statement_kind: QueryStatementKind::Select,
            sources: vec![source],
        })
    }

    fn parse_source(&mut self) -> Result<QuerySourceOccurrence, QueryLanguageDiagnostic> {
        if self.current().kind == TokenKind::Ampersand {
            let start = self.current().location.start_byte();
            self.advance();
            let TokenKind::Word(_) = self.current().kind else {
                return Err(self.malformed_current("expected a parameter name after '&'"));
            };
            let end = self.current().location.end_byte();
            self.advance();
            return Err(QueryLanguageDiagnostic::new(
                QueryLanguageDiagnosticKind::ExternalOrParameterDataSource,
                "parameter data source is unsupported in the minimum query-language slice",
                QueryTextRange::new(start, end),
            ));
        }

        let namespace_token = self.current();
        let namespace = self.expect_identifier("expected a data-source namespace")?;
        self.expect_dot("expected '.' in the qualified data-source name")?;
        let local_name_token = self.current();
        let local_name = self.expect_identifier("expected a local metadata name")?;

        let category = match namespace {
            "Catalog" | "Справочник" => QuerySourceCategory::Catalog,
            "InformationRegister" => QuerySourceCategory::InformationRegister,
            _ => {
                return Err(QueryLanguageDiagnostic::new(
                    QueryLanguageDiagnosticKind::UnsupportedPersistentNamespace,
                    "persistent namespace is outside the minimum query-language slice",
                    namespace_token.location,
                ));
            }
        };

        let location = QueryTextRange::new(
            namespace_token.location.start_byte(),
            local_name_token.location.end_byte(),
        );

        Ok(QuerySourceOccurrence {
            raw_spelling: self.source[location.start_byte()..location.end_byte()].to_owned(),
            category,
            namespace: namespace.to_owned(),
            local_name: local_name.to_owned(),
            alias: None,
            location,
        })
    }

    fn parse_identifier_path(
        &mut self,
        message: &'static str,
    ) -> Result<QueryTextRange, QueryLanguageDiagnostic> {
        let first = self.current();
        self.expect_identifier(message)?;
        let mut end = first.location.end_byte();

        while self.current().kind == TokenKind::Dot {
            self.advance();
            let component = self.current();
            self.expect_identifier("expected an identifier after '.'")?;
            end = component.location.end_byte();
        }

        Ok(QueryTextRange::new(first.location.start_byte(), end))
    }

    fn expect_identifier(
        &mut self,
        message: &'static str,
    ) -> Result<&'a str, QueryLanguageDiagnostic> {
        match self.current().kind {
            TokenKind::Word(word) => {
                self.advance();
                Ok(word)
            }
            _ => Err(self.malformed_current(message)),
        }
    }

    fn expect_word(
        &mut self,
        expected: &str,
        message: &'static str,
    ) -> Result<(), QueryLanguageDiagnostic> {
        if self.consume_word(expected) {
            Ok(())
        } else {
            Err(self.malformed_current(message))
        }
    }

    fn expect_number(&mut self, message: &'static str) -> Result<(), QueryLanguageDiagnostic> {
        if self.current().kind == TokenKind::Number {
            self.advance();
            Ok(())
        } else {
            Err(self.malformed_current(message))
        }
    }

    fn expect_dot(&mut self, message: &'static str) -> Result<(), QueryLanguageDiagnostic> {
        if self.current().kind == TokenKind::Dot {
            self.advance();
            Ok(())
        } else {
            Err(self.malformed_current(message))
        }
    }

    fn consume_word(&mut self, expected: &str) -> bool {
        if matches!(self.current().kind, TokenKind::Word(word) if word == expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn malformed_current(&self, message: &'static str) -> QueryLanguageDiagnostic {
        QueryLanguageDiagnostic::new(
            QueryLanguageDiagnosticKind::MalformedSyntax,
            message,
            self.current().location,
        )
    }

    fn current(&self) -> Token<'a> {
        self.tokens[self.cursor]
    }

    fn advance(&mut self) {
        if self.cursor + 1 < self.tokens.len() {
            self.cursor += 1;
        }
    }
}

fn tokenize(source: &str) -> Vec<Token<'_>> {
    let mut tokens = Vec::new();
    let mut characters = source.char_indices().peekable();

    while let Some((start, character)) = characters.next() {
        if character.is_whitespace() {
            continue;
        }

        let end = start + character.len_utf8();
        let kind = if is_identifier_start(character) {
            let mut word_end = end;
            while let Some(&(position, next)) = characters.peek() {
                if !is_identifier_continue(next) {
                    break;
                }
                characters.next();
                word_end = position + next.len_utf8();
            }
            tokens.push(Token {
                kind: TokenKind::Word(&source[start..word_end]),
                location: QueryTextRange::new(start, word_end),
            });
            continue;
        } else if character.is_ascii_digit() {
            let mut number_end = end;
            while let Some(&(position, next)) = characters.peek() {
                if !next.is_ascii_digit() {
                    break;
                }
                characters.next();
                number_end = position + next.len_utf8();
            }
            tokens.push(Token {
                kind: TokenKind::Number,
                location: QueryTextRange::new(start, number_end),
            });
            continue;
        } else {
            match character {
                '.' => TokenKind::Dot,
                '&' => TokenKind::Ampersand,
                _ => TokenKind::Invalid,
            }
        };

        tokens.push(Token {
            kind,
            location: QueryTextRange::new(start, end),
        });
    }

    tokens.push(Token {
        kind: TokenKind::EndOfInput,
        location: QueryTextRange::new(source.len(), source.len()),
    });
    tokens
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
        QueryLanguageDiagnosticKind, QueryLanguageParser, QuerySourceCategory, QueryStatementKind,
    };

    const ACCEPTED_CATALOG_EN: &str =
        include_str!("../tests/fixtures/query_language/accepted_catalog_en.query");
    const ACCEPTED_CATALOG_RU: &str =
        include_str!("../tests/fixtures/query_language/accepted_catalog_ru.query");
    const ACCEPTED_INFORMATION_REGISTER_EN: &str =
        include_str!("../tests/fixtures/query_language/accepted_information_register_en.query");
    const UNSUPPORTED_PARAMETER_SOURCE_EN: &str =
        include_str!("../tests/fixtures/query_language/unsupported_parameter_source_en.query");

    #[test]
    fn query_language_parses_english_catalog_fixture() {
        let result = QueryLanguageParser.parse(ACCEPTED_CATALOG_EN);

        assert!(result.is_source_set_complete());
        assert!(result.diagnostics().is_empty());
        let program = result.program().expect("accepted fixture must parse");
        assert_eq!(program.statement_kind(), QueryStatementKind::Select);
        assert_eq!(program.sources().len(), 1);
        let source = &program.sources()[0];
        assert_eq!(source.raw_spelling(), "Catalog.Products");
        assert_eq!(source.category(), QuerySourceCategory::Catalog);
        assert_eq!(source.namespace(), "Catalog");
        assert_eq!(source.local_name(), "Products");
        assert_eq!(source.alias(), None);
        assert_eq!(source.location().start_byte(), 16);
        assert_eq!(source.location().end_byte(), 32);
    }

    #[test]
    fn query_language_parses_russian_catalog_fixture_with_utf8_byte_location() {
        let result = QueryLanguageParser.parse(ACCEPTED_CATALOG_RU);

        assert!(result.is_source_set_complete());
        assert!(result.diagnostics().is_empty());
        let program = result.program().expect("accepted fixture must parse");
        let source = &program.sources()[0];
        assert_eq!(source.raw_spelling(), "Справочник.Номенклатура");
        assert_eq!(source.category(), QuerySourceCategory::Catalog);
        assert_eq!(source.namespace(), "Справочник");
        assert_eq!(source.local_name(), "Номенклатура");
        assert_eq!(source.alias(), None);
        assert_eq!(source.location().start_byte(), 33);
        assert_eq!(source.location().end_byte(), 78);
        assert_eq!(
            &ACCEPTED_CATALOG_RU[source.location().start_byte()..source.location().end_byte()],
            source.raw_spelling()
        );
    }

    #[test]
    fn query_language_parses_information_register_fixture_with_alias() {
        let result = QueryLanguageParser.parse(ACCEPTED_INFORMATION_REGISTER_EN);

        assert!(result.is_source_set_complete());
        assert!(result.diagnostics().is_empty());
        let program = result.program().expect("accepted fixture must parse");
        let source = &program.sources()[0];
        assert_eq!(source.raw_spelling(), "InformationRegister.ObjectsToDelete");
        assert_eq!(source.category(), QuerySourceCategory::InformationRegister);
        assert_eq!(source.namespace(), "InformationRegister");
        assert_eq!(source.local_name(), "ObjectsToDelete");
        assert_eq!(source.alias(), Some("Tab"));
        assert_eq!(source.location().start_byte(), 32);
        assert_eq!(source.location().end_byte(), 67);
    }

    #[test]
    fn query_language_reports_parameter_source_without_partial_program() {
        let result = QueryLanguageParser.parse(UNSUPPORTED_PARAMETER_SOURCE_EN);

        assert!(!result.is_source_set_complete());
        assert!(result.program().is_none());
        assert_eq!(result.diagnostics().len(), 1);
        let diagnostic = result.diagnostics()[0];
        assert_eq!(
            diagnostic.kind(),
            QueryLanguageDiagnosticKind::ExternalOrParameterDataSource
        );
        assert_eq!(diagnostic.location().start_byte(), 29);
        assert_eq!(diagnostic.location().end_byte(), 47);
        assert_eq!(
            &UNSUPPORTED_PARAMETER_SOURCE_EN
                [diagnostic.location().start_byte()..diagnostic.location().end_byte()],
            "&MetadataTableName"
        );
    }

    #[test]
    fn query_language_reports_unsupported_namespace_without_partial_program() {
        let result = QueryLanguageParser.parse("SELECT Ref FROM Document.Sales");

        assert!(!result.is_source_set_complete());
        assert!(result.program().is_none());
        assert_eq!(result.diagnostics().len(), 1);
        let diagnostic = result.diagnostics()[0];
        assert_eq!(
            diagnostic.kind(),
            QueryLanguageDiagnosticKind::UnsupportedPersistentNamespace
        );
        assert_eq!(diagnostic.location().start_byte(), 16);
        assert_eq!(diagnostic.location().end_byte(), 24);
    }

    #[test]
    fn query_language_reports_empty_and_incomplete_input_as_malformed() {
        let empty = QueryLanguageParser.parse("");
        let incomplete = QueryLanguageParser.parse("SELECT Ref FROM");
        let incomplete_parameter = QueryLanguageParser.parse("SELECT Ref FROM &");

        for result in [&empty, &incomplete, &incomplete_parameter] {
            assert!(!result.is_source_set_complete());
            assert!(result.program().is_none());
            assert_eq!(result.diagnostics().len(), 1);
            assert_eq!(
                result.diagnostics()[0].kind(),
                QueryLanguageDiagnosticKind::MalformedSyntax
            );
        }
        assert_eq!(empty.diagnostics()[0].location().start_byte(), 0);
        assert_eq!(empty.diagnostics()[0].location().end_byte(), 0);
        assert_eq!(incomplete.diagnostics()[0].location().start_byte(), 15);
        assert_eq!(incomplete.diagnostics()[0].location().end_byte(), 15);
        assert_eq!(
            incomplete_parameter.diagnostics()[0]
                .location()
                .start_byte(),
            17
        );
        assert_eq!(
            incomplete_parameter.diagnostics()[0].location().end_byte(),
            17
        );
    }

    #[test]
    fn query_language_rejects_unconsumed_input_without_partial_program() {
        let result = QueryLanguageParser.parse("SELECT Ref FROM Catalog.Products EXTRA");

        assert!(!result.is_source_set_complete());
        assert!(result.program().is_none());
        assert_eq!(result.diagnostics().len(), 1);
        assert_eq!(
            result.diagnostics()[0].kind(),
            QueryLanguageDiagnosticKind::MalformedSyntax
        );
        assert_eq!(result.diagnostics()[0].location().start_byte(), 33);
        assert_eq!(result.diagnostics()[0].location().end_byte(), 38);
    }

    #[test]
    fn query_language_preserves_crlf_in_raw_byte_locations() {
        let result = QueryLanguageParser.parse("SELECT\r\nRef\r\nFROM\r\nCatalog.Products");

        assert!(result.is_source_set_complete());
        let source = &result
            .program()
            .expect("CRLF input must parse without normalization")
            .sources()[0];
        assert_eq!(source.location().start_byte(), 19);
        assert_eq!(source.location().end_byte(), 35);
    }

    #[test]
    fn query_language_repeated_parsing_is_deterministic() {
        let first = QueryLanguageParser.parse(ACCEPTED_INFORMATION_REGISTER_EN);
        let second = QueryLanguageParser.parse(ACCEPTED_INFORMATION_REGISTER_EN);

        assert_eq!(first, second);
    }
}
