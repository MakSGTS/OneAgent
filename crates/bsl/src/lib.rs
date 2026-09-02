//! BSL source models, extraction, and minimum query-language parsing for `OneAgent`.

use oneagent_common::{EntityId, EntityIdError, EntityName};
use std::fmt::{Display, Formatter};

mod calls;
mod cross_module_resolution;
mod queries;
mod query_language;
mod resolution;

pub use calls::{BslCall, BslCallError, BslCallExtractor, BslCallKind, LineBslCallExtractor};
pub use cross_module_resolution::{
    BslModuleSymbols, CrossModuleCallResolution, CrossModuleCallResolver, QualifiedBslCallResolver,
    ResolvedCrossModuleCall, UnresolvedCrossModuleCall, UnresolvedCrossModuleCallReason,
};
pub use queries::{BslQuery, BslQueryError, BslQueryExtractor, LineBslQueryExtractor};
pub use query_language::{
    ParsedQueryProgram, QueryLanguageDiagnostic, QueryLanguageDiagnosticKind,
    QueryLanguageParseResult, QueryLanguageParser, QuerySourceCategory, QuerySourceOccurrence,
    QueryStatementKind, QueryTextRange,
};
pub use resolution::{
    BslCallResolution, BslCallResolver, LocalBslCallResolver, ResolvedBslCall, UnresolvedBslCall,
    UnresolvedCallReason,
};

/// Non-empty half-open identifier range in raw BSL UTF-8 bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BslIdentifierRange {
    start_byte: usize,
    end_byte: usize,
}

impl BslIdentifierRange {
    /// Creates a non-empty half-open byte range.
    #[must_use]
    pub const fn new(start_byte: usize, end_byte: usize) -> Option<Self> {
        if start_byte < end_byte {
            Some(Self {
                start_byte,
                end_byte,
            })
        } else {
            None
        }
    }

    /// Returns the inclusive raw UTF-8 byte offset.
    #[must_use]
    pub const fn start_byte(self) -> usize {
        self.start_byte
    }

    /// Returns the exclusive raw UTF-8 byte offset.
    #[must_use]
    pub const fn end_byte(self) -> usize {
        self.end_byte
    }
}

/// Returns the canonical case-insensitive BSL name key.
#[must_use]
pub fn bsl_name_key(value: &str) -> String {
    value.to_lowercase()
}

/// Returns whether two BSL names are equivalent for resolution.
#[must_use]
pub fn bsl_names_equal(left: &str, right: &str) -> bool {
    bsl_name_key(left) == bsl_name_key(right)
}

/// Supported BSL symbol kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BslSymbolKind {
    /// BSL procedure.
    Procedure,
    /// BSL function.
    Function,
}

/// Creates the stable identifier used for one BSL callable declaration.
///
/// # Errors
///
/// Returns [`EntityIdError`] when the composed identifier is invalid.
pub fn bsl_callable_id(
    module_id: &EntityId,
    kind: BslSymbolKind,
    name: &str,
) -> Result<EntityId, EntityIdError> {
    EntityId::new(format!("{}:{}:{name}", module_id.as_str(), kind.as_str()))
}

impl BslSymbolKind {
    /// Returns a stable machine-readable name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Procedure => "procedure",
            Self::Function => "function",
        }
    }
}

/// A top-level declaration found in a BSL module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BslSymbol {
    id: EntityId,
    name: EntityName,
    kind: BslSymbolKind,
    line: usize,
    exported: bool,
    identifier_range: Option<BslIdentifierRange>,
}

impl BslSymbol {
    /// Creates a BSL symbol.
    #[must_use]
    pub const fn new(
        id: EntityId,
        name: EntityName,
        kind: BslSymbolKind,
        line: usize,
        exported: bool,
    ) -> Self {
        Self {
            id,
            name,
            kind,
            line,
            exported,
            identifier_range: None,
        }
    }

    /// Creates an extracted BSL symbol with its exact raw identifier range.
    #[must_use]
    pub const fn new_with_identifier_range(
        id: EntityId,
        name: EntityName,
        kind: BslSymbolKind,
        line: usize,
        exported: bool,
        identifier_range: BslIdentifierRange,
    ) -> Self {
        Self {
            id,
            name,
            kind,
            line,
            exported,
            identifier_range: Some(identifier_range),
        }
    }

    /// Returns the stable symbol identifier.
    #[must_use]
    pub const fn id(&self) -> &EntityId {
        &self.id
    }

    /// Returns the symbol name.
    #[must_use]
    pub const fn name(&self) -> &EntityName {
        &self.name
    }

    /// Returns the symbol kind.
    #[must_use]
    pub const fn kind(&self) -> BslSymbolKind {
        self.kind
    }

    /// Returns the one-based source line.
    #[must_use]
    pub const fn line(&self) -> usize {
        self.line
    }

    /// Returns whether the declaration is exported.
    #[must_use]
    pub const fn is_exported(&self) -> bool {
        self.exported
    }

    /// Returns the exact identifier range when the symbol came from an extractor.
    #[must_use]
    pub const fn identifier_range(&self) -> Option<BslIdentifierRange> {
        self.identifier_range
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct BslSourceLine<'source> {
    pub(crate) number: usize,
    pub(crate) start_byte: usize,
    pub(crate) text: &'source str,
}

pub(crate) fn source_lines(source: &str) -> Vec<BslSourceLine<'_>> {
    let bytes = source.as_bytes();
    let mut result = Vec::new();
    let mut start = 0;
    let mut number = 1;

    while start < bytes.len() {
        let mut end = start;
        while end < bytes.len() && !matches!(bytes[end], b'\r' | b'\n') {
            end += 1;
        }
        result.push(BslSourceLine {
            number,
            start_byte: start,
            text: &source[start..end],
        });
        if end == bytes.len() {
            break;
        }
        start = if bytes[end] == b'\r' && bytes.get(end + 1) == Some(&b'\n') {
            end + 2
        } else {
            end + 1
        };
        number += 1;
    }
    result
}

/// Extracts top-level declarations from a BSL module.
pub trait BslDeclarationExtractor {
    /// Extracts declarations using `module_id` as the stable parent identifier.
    ///
    /// # Errors
    ///
    /// Returns an error when a declaration cannot be represented by the domain model.
    fn extract(&self, module_id: &EntityId, source: &str) -> Result<Vec<BslSymbol>, BslParseError>;
}

/// Deterministic line-oriented extractor for top-level BSL declarations.
///
/// This component intentionally extracts only declarations. Full expressions,
/// scopes, calls and type inference belong to later parser stages.
#[derive(Debug, Default, Clone, Copy)]
pub struct LineBslDeclarationExtractor;

impl BslDeclarationExtractor for LineBslDeclarationExtractor {
    fn extract(&self, module_id: &EntityId, source: &str) -> Result<Vec<BslSymbol>, BslParseError> {
        let mut symbols = Vec::new();

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

            if trimmed.starts_with("//") || trimmed.starts_with('#') {
                continue;
            }

            if let Some(symbol) = parse_declaration(
                module_id,
                trimmed,
                trimmed_start,
                source_line.number,
                BslSymbolKind::Procedure,
            )? {
                symbols.push(symbol);
                continue;
            }

            if let Some(symbol) = parse_declaration(
                module_id,
                trimmed,
                trimmed_start,
                source_line.number,
                BslSymbolKind::Function,
            )? {
                symbols.push(symbol);
            }
        }

        Ok(symbols)
    }
}

fn parse_declaration(
    module_id: &EntityId,
    line: &str,
    line_start: usize,
    line_number: usize,
    kind: BslSymbolKind,
) -> Result<Option<BslSymbol>, BslParseError> {
    let keywords = match kind {
        BslSymbolKind::Procedure => ["procedure", "процедура"],
        BslSymbolKind::Function => ["function", "функция"],
    };

    let lowercase = line.to_lowercase();
    let Some(keyword) = keywords
        .iter()
        .find(|keyword| lowercase.starts_with(**keyword))
    else {
        return Ok(None);
    };

    let after_keyword = &line[keyword.len()..];
    let remainder = after_keyword.trim_start();
    let remainder_start = keyword.len() + after_keyword.len().saturating_sub(remainder.len());
    let Some(open_parenthesis) = remainder.find('(') else {
        return Err(BslParseError::MalformedDeclaration {
            line: line_number,
            text: line.to_owned(),
        });
    };

    let before_parenthesis = &remainder[..open_parenthesis];
    let raw_name = before_parenthesis.trim();
    if raw_name.is_empty() {
        return Err(BslParseError::MalformedDeclaration {
            line: line_number,
            text: line.to_owned(),
        });
    }

    let exported =
        remainder.to_lowercase().contains("export") || remainder.to_lowercase().contains("экспорт");

    let name_start = line_start
        + remainder_start
        + before_parenthesis
            .len()
            .saturating_sub(before_parenthesis.trim_start().len());
    let identifier_range = BslIdentifierRange::new(name_start, name_start + raw_name.len())
        .expect("a non-empty extracted identifier must have a non-empty range");
    let name = EntityName::new(raw_name).map_err(|_| BslParseError::InvalidName(line_number))?;
    let id = bsl_callable_id(module_id, kind, raw_name)
        .map_err(|_| BslParseError::InvalidIdentifier(line_number))?;

    Ok(Some(BslSymbol::new_with_identifier_range(
        id,
        name,
        kind,
        line_number,
        exported,
        identifier_range,
    )))
}

/// Error produced while extracting BSL declarations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BslParseError {
    /// A declaration line has invalid syntax.
    MalformedDeclaration {
        /// One-based line number.
        line: usize,
        /// Original source line.
        text: String,
    },
    /// A symbol name could not be represented.
    InvalidName(usize),
    /// A symbol identifier could not be represented.
    InvalidIdentifier(usize),
}

impl Display for BslParseError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MalformedDeclaration { line, text } => {
                write!(
                    formatter,
                    "malformed BSL declaration at line {line}: {text}"
                )
            }
            Self::InvalidName(line) => {
                write!(formatter, "invalid BSL symbol name at line {line}")
            }
            Self::InvalidIdentifier(line) => {
                write!(formatter, "invalid BSL symbol identifier at line {line}")
            }
        }
    }
}

impl std::error::Error for BslParseError {}

#[cfg(test)]
mod tests {
    use oneagent_common::EntityId;

    use super::{
        BslDeclarationExtractor, BslParseError, BslSymbol, BslSymbolKind,
        LineBslDeclarationExtractor, bsl_callable_id, bsl_names_equal,
    };

    fn module_id() -> EntityId {
        EntityId::new("module.sales.object").expect("identifier must be valid")
    }

    #[test]
    fn extracts_russian_and_english_declarations() {
        let source = r"
Процедура ПередЗаписью(Отказ, РежимЗаписи) Экспорт
КонецПроцедуры

Function CalculateTotal()
EndFunction
";

        let symbols = LineBslDeclarationExtractor
            .extract(&module_id(), source)
            .expect("declarations must parse");

        assert_eq!(symbols.len(), 2);
        assert_eq!(symbols[0].name().as_str(), "ПередЗаписью");
        assert_eq!(symbols[0].kind(), BslSymbolKind::Procedure);
        assert!(symbols[0].is_exported());
        assert_eq!(symbols[1].name().as_str(), "CalculateTotal");
        assert_eq!(symbols[1].kind(), BslSymbolKind::Function);
    }

    #[test]
    fn ignores_comments_and_preprocessor_lines() {
        let source = r"
// Процедура Commented()
#If Client Then
Procedure RealProcedure()
EndProcedure
";

        let symbols = LineBslDeclarationExtractor
            .extract(&module_id(), source)
            .expect("declarations must parse");

        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name().as_str(), "RealProcedure");
    }

    #[test]
    fn reports_malformed_declaration() {
        let error = LineBslDeclarationExtractor
            .extract(&module_id(), "Procedure MissingParenthesis")
            .expect_err("malformed declaration must fail");

        assert!(matches!(
            error,
            BslParseError::MalformedDeclaration { line: 1, .. }
        ));
    }

    #[test]
    fn declaration_ranges_preserve_bom_unicode_whitespace_and_line_endings() {
        let source =
            "\u{feff}  Процедура Тест () Экспорт\r\n\tFunction CalculateTotal()\rEndFunction\n";
        let first = LineBslDeclarationExtractor
            .extract(&module_id(), source)
            .expect("declarations must parse");
        let repeated = LineBslDeclarationExtractor
            .extract(&module_id(), source)
            .expect("repeated declarations must parse");

        assert_eq!(first, repeated);
        assert_eq!(first.len(), 2);
        for symbol in &first {
            let range = symbol
                .identifier_range()
                .expect("extracted declaration must have an exact range");
            assert_eq!(
                &source[range.start_byte()..range.end_byte()],
                symbol.name().as_str()
            );
        }
        assert_eq!(first[0].line(), 1);
        assert_eq!(first[1].line(), 2);
    }

    #[test]
    fn callable_helpers_and_legacy_constructor_preserve_semantic_compatibility() {
        let identifier = bsl_callable_id(&module_id(), BslSymbolKind::Procedure, "DoWork")
            .expect("callable identifier must be valid");
        assert_eq!(identifier.as_str(), "module.sales.object:procedure:DoWork");
        assert!(bsl_names_equal("ПРОВЕРИТЬ", "проверить"));

        let symbol = BslSymbol::new(
            identifier,
            oneagent_common::EntityName::new("DoWork").expect("name must be valid"),
            BslSymbolKind::Procedure,
            1,
            false,
        );
        assert!(symbol.identifier_range().is_none());
    }
}
