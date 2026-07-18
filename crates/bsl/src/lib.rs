//! BSL source model and declaration extraction for `OneAgent`.

use oneagent_common::{EntityId, EntityName};
use std::fmt::{Display, Formatter};

mod calls;
mod cross_module_resolution;
mod resolution;

pub use calls::{BslCall, BslCallError, BslCallExtractor, LineBslCallExtractor};
pub use cross_module_resolution::{
    BslModuleSymbols, CrossModuleCallResolution, CrossModuleCallResolver, QualifiedBslCallResolver,
    ResolvedCrossModuleCall, UnresolvedCrossModuleCall, UnresolvedCrossModuleCallReason,
};
pub use resolution::{
    BslCallResolution, BslCallResolver, LocalBslCallResolver, ResolvedBslCall, UnresolvedBslCall,
    UnresolvedCallReason,
};

/// Supported BSL symbol kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BslSymbolKind {
    /// BSL procedure.
    Procedure,
    /// BSL function.
    Function,
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

        for (index, line) in source.lines().enumerate() {
            let line_number = index + 1;
            let trimmed = line.trim_start();

            if trimmed.starts_with("//") || trimmed.starts_with('#') {
                continue;
            }

            if let Some(symbol) =
                parse_declaration(module_id, trimmed, line_number, BslSymbolKind::Procedure)?
            {
                symbols.push(symbol);
                continue;
            }

            if let Some(symbol) =
                parse_declaration(module_id, trimmed, line_number, BslSymbolKind::Function)?
            {
                symbols.push(symbol);
            }
        }

        Ok(symbols)
    }
}

fn parse_declaration(
    module_id: &EntityId,
    line: &str,
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

    let remainder = line[keyword.len()..].trim_start();
    let Some(open_parenthesis) = remainder.find('(') else {
        return Err(BslParseError::MalformedDeclaration {
            line: line_number,
            text: line.to_owned(),
        });
    };

    let raw_name = remainder[..open_parenthesis].trim();
    if raw_name.is_empty() {
        return Err(BslParseError::MalformedDeclaration {
            line: line_number,
            text: line.to_owned(),
        });
    }

    let exported =
        remainder.to_lowercase().contains("export") || remainder.to_lowercase().contains("экспорт");

    let name = EntityName::new(raw_name).map_err(|_| BslParseError::InvalidName(line_number))?;
    let id = EntityId::new(format!(
        "{}:{}:{}",
        module_id.as_str(),
        kind.as_str(),
        raw_name
    ))
    .map_err(|_| BslParseError::InvalidIdentifier(line_number))?;

    Ok(Some(BslSymbol::new(id, name, kind, line_number, exported)))
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
        BslDeclarationExtractor, BslParseError, BslSymbolKind, LineBslDeclarationExtractor,
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
}
