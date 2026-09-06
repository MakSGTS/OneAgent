//! BSL source models, extraction, and minimum query-language parsing for `OneAgent`.

use oneagent_common::{EntityId, EntityIdError, EntityName};
use std::collections::BTreeSet;
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

pub(crate) fn leading_bsl_token(line: &str) -> Option<(&str, &str)> {
    let trimmed = line.trim_start();
    let token_end = trimmed
        .char_indices()
        .find(|(_, scalar)| *scalar != '_' && !scalar.is_alphanumeric())
        .map_or(trimmed.len(), |(index, _)| index);
    (token_end > 0).then(|| (&trimmed[..token_end], &trimmed[token_end..]))
}

#[derive(Debug, Clone)]
pub(crate) struct ParsedCallableScope {
    symbol: BslSymbol,
    header_end_line: usize,
    end_line: usize,
    shadowed_names: BTreeSet<String>,
    bindings_complete: bool,
}

impl ParsedCallableScope {
    pub(crate) const fn symbol(&self) -> &BslSymbol {
        &self.symbol
    }

    pub(crate) const fn header_end_line(&self) -> usize {
        self.header_end_line
    }

    pub(crate) const fn end_line(&self) -> usize {
        self.end_line
    }

    pub(crate) fn shadows(&self, value: &str) -> bool {
        !self.bindings_complete || self.shadowed_names.contains(&bsl_name_key(value))
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ParsedCallableModule {
    scopes: Vec<ParsedCallableScope>,
    shadowed_names: BTreeSet<String>,
    bindings_complete: bool,
}

impl ParsedCallableModule {
    pub(crate) fn scopes(&self) -> &[ParsedCallableScope] {
        &self.scopes
    }

    pub(crate) fn shadows(&self, value: &str) -> bool {
        !self.bindings_complete || self.shadowed_names.contains(&bsl_name_key(value))
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
        parse_callable_scopes(module_id, source).map(|module| {
            module
                .scopes
                .into_iter()
                .map(|scope| scope.symbol)
                .collect()
        })
    }
}

impl LineBslDeclarationExtractor {
    /// Extracts declarations while rejecting the first symbol above `maximum`
    /// before retaining it in parser state.
    ///
    /// # Errors
    ///
    /// Returns a parse error or a bounded error for the first excess symbol.
    pub fn extract_bounded(
        &self,
        module_id: &EntityId,
        source: &str,
        maximum: usize,
    ) -> Result<Vec<BslSymbol>, BslParseError> {
        parse_callable_scopes_bounded(module_id, source, Some(maximum)).map(|module| {
            module
                .scopes
                .into_iter()
                .map(|scope| scope.symbol)
                .collect()
        })
    }
}

pub(crate) fn parse_callable_scopes(
    module_id: &EntityId,
    source: &str,
) -> Result<ParsedCallableModule, BslParseError> {
    parse_callable_scopes_bounded(module_id, source, None)
}

fn parse_callable_scopes_bounded(
    module_id: &EntityId,
    source: &str,
    maximum: Option<usize>,
) -> Result<ParsedCallableModule, BslParseError> {
    if source
        .as_bytes()
        .strip_prefix(b"\xef\xbb\xbf")
        .is_some_and(|remainder| remainder.starts_with(b"\xef\xbb\xbf"))
    {
        return Ok(ParsedCallableModule {
            scopes: Vec::new(),
            shadowed_names: BTreeSet::new(),
            bindings_complete: true,
        });
    }
    let lines = source_lines(source);
    let mut scopes = Vec::new();
    let mut index = 0_usize;

    while index < lines.len() {
        let source_line = lines[index];
        let (line, bom_bytes) = source_line_text(source_line);
        let trimmed = line.trim_start();
        if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with('#') {
            index += 1;
            continue;
        }
        if callable_end_kind(trimmed).is_some() {
            return Err(malformed_declaration(source_line.number, trimmed));
        }
        let Some(kind) = callable_start_kind(trimmed) else {
            index += 1;
            continue;
        };
        let (symbol, header_end_index, parameters, mut bindings_complete) =
            parse_callable_header(module_id, &lines, index, line, bom_bytes, kind)?;
        let mut shadowed_names = parameters;
        let mut body_index = header_end_index + 1;
        let end_line = loop {
            let Some(body_line) = lines.get(body_index).copied() else {
                return Err(malformed_declaration(source_line.number, trimmed));
            };
            let (body_text, _) = source_line_text(body_line);
            let body_trimmed = body_text.trim_start();
            if body_trimmed.starts_with("//") || body_trimmed.starts_with('#') {
                body_index += 1;
                continue;
            }
            if callable_start_kind(body_trimmed).is_some() {
                return Err(BslParseError::NestedDeclaration(body_line.number));
            }
            if let Some(actual_end) = callable_end_kind(body_trimmed) {
                if actual_end != kind || !valid_scope_end_tail(body_trimmed) {
                    return Err(malformed_declaration(body_line.number, body_trimmed));
                }
                break body_line.number;
            }
            bindings_complete &= collect_line_bindings(body_text, &mut shadowed_names);
            body_index += 1;
        };
        if maximum.is_some_and(|maximum| scopes.len() >= maximum) {
            return Err(BslParseError::BoundExceeded {
                actual: scopes.len().saturating_add(1),
                maximum: maximum.expect("bounded parser has a maximum"),
            });
        }
        scopes.push(ParsedCallableScope {
            symbol,
            header_end_line: lines[header_end_index].number,
            end_line,
            shadowed_names,
            bindings_complete,
        });
        index = body_index + 1;
    }
    let mut module_bindings = BTreeSet::new();
    let mut module_bindings_complete = true;
    for source_line in &lines {
        if scopes.iter().any(|scope| {
            scope.symbol.line() <= source_line.number && source_line.number <= scope.end_line
        }) {
            continue;
        }
        let (line, _) = source_line_text(*source_line);
        module_bindings_complete &= collect_line_bindings(line, &mut module_bindings);
    }
    for scope in &mut scopes {
        scope.shadowed_names.extend(module_bindings.iter().cloned());
        scope.bindings_complete &= module_bindings_complete;
    }
    Ok(ParsedCallableModule {
        scopes,
        shadowed_names: module_bindings,
        bindings_complete: module_bindings_complete,
    })
}

fn source_line_text(source_line: BslSourceLine<'_>) -> (&str, usize) {
    if source_line.number == 1 {
        source_line
            .text
            .strip_prefix('\u{feff}')
            .map_or((source_line.text, 0), |line| (line, 3))
    } else {
        (source_line.text, 0)
    }
}

fn callable_start_kind(line: &str) -> Option<BslSymbolKind> {
    callable_header_keyword(line).and_then(|(_, token, _)| {
        if ["procedure", "процедура"]
            .into_iter()
            .any(|keyword| bsl_names_equal(token, keyword))
        {
            Some(BslSymbolKind::Procedure)
        } else if ["function", "функция"]
            .into_iter()
            .any(|keyword| bsl_names_equal(token, keyword))
        {
            Some(BslSymbolKind::Function)
        } else {
            None
        }
    })
}

fn callable_header_keyword(line: &str) -> Option<(usize, &str, &str)> {
    let (first, after_first) = leading_bsl_token(line)?;
    if !bsl_names_equal(first, "async") && !bsl_names_equal(first, "асинх") {
        return Some((0, first, after_first));
    }
    let remainder = after_first.trim_start();
    let whitespace = after_first.len().saturating_sub(remainder.len());
    let (keyword, after_keyword) = leading_bsl_token(remainder)?;
    Some((first.len() + whitespace, keyword, after_keyword))
}

fn callable_end_kind(line: &str) -> Option<BslSymbolKind> {
    leading_bsl_token(line).and_then(|(token, _)| {
        if ["endprocedure", "конецпроцедуры"]
            .into_iter()
            .any(|keyword| bsl_names_equal(token, keyword))
        {
            Some(BslSymbolKind::Procedure)
        } else if ["endfunction", "конецфункции"]
            .into_iter()
            .any(|keyword| bsl_names_equal(token, keyword))
        {
            Some(BslSymbolKind::Function)
        } else {
            None
        }
    })
}

fn valid_scope_end_tail(line: &str) -> bool {
    leading_bsl_token(line).is_some_and(|(_, tail)| {
        let tail = tail.trim_start();
        tail.is_empty()
            || tail.starts_with("//")
            || tail.strip_prefix(';').is_some_and(|rest| {
                rest.trim_start().is_empty() || rest.trim_start().starts_with("//")
            })
    })
}

#[allow(clippy::too_many_arguments)]
fn parse_callable_header(
    module_id: &EntityId,
    lines: &[BslSourceLine<'_>],
    start_index: usize,
    first_line: &str,
    bom_bytes: usize,
    kind: BslSymbolKind,
) -> Result<(BslSymbol, usize, BTreeSet<String>, bool), BslParseError> {
    let source_line = lines[start_index];
    let trimmed = first_line.trim_start();
    let (keyword_start, keyword, after_keyword) = callable_header_keyword(trimmed)
        .ok_or_else(|| malformed_declaration(source_line.number, trimmed))?;
    let remainder = after_keyword.trim_start();
    let whitespace_after_keyword = after_keyword.len().saturating_sub(remainder.len());
    let Some((raw_name, after_name)) = leading_bsl_token(remainder) else {
        return Err(malformed_declaration(source_line.number, trimmed));
    };
    if !is_bsl_identifier(raw_name) || !after_name.trim_start().starts_with('(') {
        return Err(malformed_declaration(source_line.number, trimmed));
    }
    let name_start = source_line.start_byte
        + bom_bytes
        + first_line.len().saturating_sub(trimmed.len())
        + keyword_start
        + keyword.len()
        + whitespace_after_keyword;
    let identifier_range = BslIdentifierRange::new(name_start, name_start + raw_name.len())
        .expect("a validated identifier has a non-empty range");

    let open_in_trimmed = keyword_start
        + keyword.len()
        + whitespace_after_keyword
        + raw_name.len()
        + after_name
            .len()
            .saturating_sub(after_name.trim_start().len());
    let parsed_header = parse_callable_header_tail(
        lines,
        start_index,
        trimmed,
        open_in_trimmed,
        source_line.number,
    )?;
    let (parameters, bindings_complete) = parameter_bindings(&parsed_header.parameter_source);
    if !bindings_complete {
        return Err(malformed_declaration(source_line.number, trimmed));
    }
    let name =
        EntityName::new(raw_name).map_err(|_| BslParseError::InvalidName(source_line.number))?;
    let id = bsl_callable_id(module_id, kind, raw_name)
        .map_err(|_| BslParseError::InvalidIdentifier(source_line.number))?;
    Ok((
        BslSymbol::new_with_identifier_range(
            id,
            name,
            kind,
            source_line.number,
            parsed_header.exported,
            identifier_range,
        ),
        parsed_header.end_index,
        parameters,
        bindings_complete,
    ))
}

struct ParsedCallableHeaderTail {
    end_index: usize,
    parameter_source: String,
    exported: bool,
}

fn parse_callable_header_tail(
    lines: &[BslSourceLine<'_>],
    start_index: usize,
    trimmed_first_line: &str,
    open_in_trimmed: usize,
    declaration_line: usize,
) -> Result<ParsedCallableHeaderTail, BslParseError> {
    let mut depth = 0_usize;
    let mut in_string = false;
    let mut parameter_source = String::new();

    for (line_index, line) in lines.iter().enumerate().skip(start_index) {
        let (line_text, _) = source_line_text(*line);
        let segment = if line_index == start_index {
            &trimmed_first_line[open_in_trimmed..]
        } else {
            line_text
        };
        let characters = segment.char_indices().collect::<Vec<_>>();
        let mut character_index = 0_usize;
        while character_index < characters.len() {
            let (byte_index, scalar) = characters[character_index];
            if scalar == '"' {
                if in_string
                    && characters
                        .get(character_index + 1)
                        .is_some_and(|(_, next)| *next == '"')
                {
                    if depth > 0 {
                        parameter_source.push_str("\"\"");
                    }
                    character_index += 2;
                    continue;
                }
                in_string = !in_string;
                if depth > 0 {
                    parameter_source.push(scalar);
                }
                character_index += 1;
                continue;
            }
            if !in_string
                && scalar == '/'
                && characters
                    .get(character_index + 1)
                    .is_some_and(|(_, next)| *next == '/')
            {
                break;
            }
            if !in_string && scalar == '(' {
                depth = depth
                    .checked_add(1)
                    .ok_or_else(|| malformed_declaration(declaration_line, trimmed_first_line))?;
                if depth > 1 {
                    parameter_source.push(scalar);
                }
            } else if !in_string && scalar == ')' {
                if depth == 0 {
                    return Err(malformed_declaration(line.number, line_text));
                }
                depth -= 1;
                if depth == 0 {
                    let scalar_end = byte_index + scalar.len_utf8();
                    let exported = parse_export_tail(&segment[scalar_end..])
                        .ok_or_else(|| malformed_declaration(line.number, line_text))?;
                    return Ok(ParsedCallableHeaderTail {
                        end_index: line_index,
                        parameter_source,
                        exported,
                    });
                }
                parameter_source.push(scalar);
            } else if depth > 0 {
                parameter_source.push(scalar);
            }
            character_index += 1;
        }
        if depth > 0 {
            parameter_source.push('\n');
        }
    }
    Err(malformed_declaration(declaration_line, trimmed_first_line))
}

fn parse_export_tail(tail: &str) -> Option<bool> {
    let tail = tail.trim_start();
    if tail.is_empty() || tail.starts_with("//") {
        return Some(false);
    }
    let (token, rest) = leading_bsl_token(tail)?;
    if !bsl_names_equal(token, "export") && !bsl_names_equal(token, "экспорт") {
        return None;
    }
    let rest = rest.trim_start();
    (rest.is_empty() || rest.starts_with("//")).then_some(true)
}

fn parameter_bindings(source: &str) -> (BTreeSet<String>, bool) {
    let mut result = BTreeSet::new();
    if source.trim().is_empty() {
        return (result, true);
    }
    let mut complete = true;
    for parameter in split_top_level(source, ',') {
        let before_default = split_top_level(parameter, '=')
            .into_iter()
            .next()
            .unwrap_or_default()
            .trim();
        if before_default.is_empty() {
            complete = false;
            continue;
        }
        let tokens = before_default
            .split_whitespace()
            .filter(|token| !bsl_names_equal(token, "val") && !bsl_names_equal(token, "знач"))
            .collect::<Vec<_>>();
        if tokens.len() != 1 || !is_bsl_identifier(tokens[0]) {
            complete = false;
            continue;
        }
        result.insert(bsl_name_key(tokens[0]));
    }
    (result, complete)
}

fn split_top_level(source: &str, delimiter: char) -> Vec<&str> {
    let mut result = Vec::new();
    let mut start = 0_usize;
    let mut depth = 0_usize;
    let mut in_string = false;
    let characters = source.char_indices().collect::<Vec<_>>();
    let mut index = 0_usize;
    while index < characters.len() {
        let (byte_index, scalar) = characters[index];
        if scalar == '"' {
            if in_string
                && characters
                    .get(index + 1)
                    .is_some_and(|(_, next)| *next == '"')
            {
                index += 2;
                continue;
            }
            in_string = !in_string;
        } else if !in_string && scalar == '(' {
            depth += 1;
        } else if !in_string && scalar == ')' {
            depth = depth.saturating_sub(1);
        } else if !in_string && depth == 0 && scalar == delimiter {
            result.push(&source[start..byte_index]);
            start = byte_index + scalar.len_utf8();
        }
        index += 1;
    }
    result.push(&source[start..]);
    result
}

fn collect_line_bindings(line: &str, bindings: &mut BTreeSet<String>) -> bool {
    let visible = visible_bsl_prefix(line);
    let mut complete = true;
    for statement in visible.split(';') {
        let statement = statement.trim_start();
        let Some((first, rest)) = leading_bsl_token(statement) else {
            continue;
        };
        if bsl_names_equal(first, "var") || bsl_names_equal(first, "перем") {
            for candidate in rest.split(',').map(str::trim) {
                if let Some(name) = variable_binding_name(candidate) {
                    bindings.insert(bsl_name_key(name));
                } else {
                    complete = false;
                }
            }
            continue;
        }
        if is_bsl_identifier(first) && rest.trim_start().starts_with('=') {
            bindings.insert(bsl_name_key(first));
            continue;
        }
        if bsl_names_equal(first, "for") || bsl_names_equal(first, "для") {
            let rest = rest.trim_start();
            let candidate = leading_bsl_token(rest).and_then(|(token, tail)| {
                if bsl_names_equal(token, "each") || bsl_names_equal(token, "каждого") {
                    leading_bsl_token(tail.trim_start()).map(|(name, _)| name)
                } else {
                    Some(token)
                }
            });
            if let Some(candidate) = candidate.filter(|candidate| is_bsl_identifier(candidate)) {
                bindings.insert(bsl_name_key(candidate));
            } else {
                complete = false;
            }
        }
    }
    complete
}

fn variable_binding_name(candidate: &str) -> Option<&str> {
    let mut tokens = candidate.split_whitespace();
    let name = tokens.next()?;
    if !is_bsl_identifier(name) {
        return None;
    }
    match (tokens.next(), tokens.next()) {
        (None, None) => Some(name),
        (Some(export), None)
            if bsl_names_equal(export, "export") || bsl_names_equal(export, "экспорт") =>
        {
            Some(name)
        }
        _ => None,
    }
}

fn visible_bsl_prefix(line: &str) -> &str {
    let mut in_string = false;
    let characters = line.char_indices().collect::<Vec<_>>();
    let mut index = 0_usize;
    while index < characters.len() {
        let (byte_index, scalar) = characters[index];
        if scalar == '"' {
            if in_string
                && characters
                    .get(index + 1)
                    .is_some_and(|(_, next)| *next == '"')
            {
                index += 2;
                continue;
            }
            in_string = !in_string;
        } else if !in_string
            && scalar == '/'
            && characters
                .get(index + 1)
                .is_some_and(|(_, next)| *next == '/')
        {
            return &line[..byte_index];
        }
        index += 1;
    }
    line
}

fn is_bsl_identifier(value: &str) -> bool {
    let mut scalars = value.chars();
    scalars.next().is_some_and(|first| {
        (first == '_' || first.is_alphabetic())
            && scalars.all(|scalar| scalar == '_' || scalar.is_alphanumeric())
    })
}

fn malformed_declaration(line: usize, text: &str) -> BslParseError {
    BslParseError::MalformedDeclaration {
        line,
        text: text.to_owned(),
    }
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
    /// A callable declaration appeared inside another callable scope.
    NestedDeclaration(usize),
    /// A symbol name could not be represented.
    InvalidName(usize),
    /// A symbol identifier could not be represented.
    InvalidIdentifier(usize),
    /// The declaration collection exceeded an inclusive caller-supplied bound.
    BoundExceeded {
        /// First rejected collection size.
        actual: usize,
        /// Accepted maximum.
        maximum: usize,
    },
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
            Self::NestedDeclaration(line) => {
                write!(formatter, "nested BSL declaration at line {line}")
            }
            Self::InvalidName(line) => {
                write!(formatter, "invalid BSL symbol name at line {line}")
            }
            Self::InvalidIdentifier(line) => {
                write!(formatter, "invalid BSL symbol identifier at line {line}")
            }
            Self::BoundExceeded { actual, maximum } => write!(
                formatter,
                "BSL declaration count {actual} exceeds maximum {maximum}"
            ),
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

Async Procedure LoadData()
EndProcedure

Асинх Функция ЗагрузитьДанные()
КонецФункции
";

        let symbols = LineBslDeclarationExtractor
            .extract(&module_id(), source)
            .expect("declarations must parse");

        assert_eq!(symbols.len(), 4);
        assert_eq!(symbols[0].name().as_str(), "ПередЗаписью");
        assert_eq!(symbols[0].kind(), BslSymbolKind::Procedure);
        assert!(symbols[0].is_exported());
        assert_eq!(symbols[1].name().as_str(), "CalculateTotal");
        assert_eq!(symbols[1].kind(), BslSymbolKind::Function);
        assert_eq!(symbols[2].name().as_str(), "LoadData");
        assert_eq!(symbols[2].kind(), BslSymbolKind::Procedure);
        assert_eq!(symbols[3].name().as_str(), "ЗагрузитьДанные");
        assert_eq!(symbols[3].kind(), BslSymbolKind::Function);
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
        let source = concat!(
            "\u{feff}  Процедура Тест () Экспорт\r\n",
            "КонецПроцедуры\r\n",
            "\tFunction CalculateTotal()\r",
            "EndFunction\n",
        );
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
        assert_eq!(first[1].line(), 3);
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

    #[test]
    fn declaration_and_export_keywords_are_token_exact_and_nested_scopes_fail() {
        let source = concat!(
            "Procedure Host()\n",
            "    ProcedureCall();\n",
            "EndProcedure\n",
            "Procedure ExportData()\n",
            "EndProcedure\n",
            "Процедура ЭкспортДанных()\n",
            "КонецПроцедуры\n",
            "Function Published() Export\n",
            "EndFunction\n",
            "Функция Опубликована() Экспорт\n",
            "КонецФункции\n",
        );
        let symbols = LineBslDeclarationExtractor
            .extract(&module_id(), source)
            .expect("token-exact declarations must parse");

        assert_eq!(
            symbols
                .iter()
                .map(|symbol| (symbol.name().as_str(), symbol.is_exported()))
                .collect::<Vec<_>>(),
            [
                ("Host", false),
                ("ExportData", false),
                ("ЭкспортДанных", false),
                ("Published", true),
                ("Опубликована", true),
            ]
        );

        let nested = "Procedure Outer()\nProcedure Inner()\nEndProcedure\nEndProcedure\n";
        assert_eq!(
            LineBslDeclarationExtractor.extract(&module_id(), nested),
            Err(BslParseError::NestedDeclaration(2))
        );
    }

    #[test]
    fn multiline_headers_preserve_export_parameters_and_exact_name_ranges() {
        let source = concat!(
            "Procedure Publish(\n",
            "    Val Module,\n",
            "    Amount = CalculateDefault(1, 2)) Export\n",
            "    Module.Target();\n",
            "EndProcedure\n",
        );
        let symbols = LineBslDeclarationExtractor
            .extract(&module_id(), source)
            .expect("multiline declaration must parse");
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name().as_str(), "Publish");
        assert!(symbols[0].is_exported());
        let range = symbols[0]
            .identifier_range()
            .expect("multiline declaration must retain its exact name range");
        assert_eq!(&source[range.start_byte()..range.end_byte()], "Publish");
    }

    #[test]
    fn malformed_names_headers_and_scope_ends_fail_closed() {
        for source in [
            "Procedure Invalid Name()\nEndProcedure\n",
            "Procedure MissingClose(\nEndProcedure\n",
            "Procedure MissingEnd()\n",
            "Procedure WrongEnd()\nEndFunction\n",
            "Procedure TrailingParameter(Value,)\nEndProcedure\n",
            "Procedure LeadingParameter(, Value)\nEndProcedure\n",
            "EndProcedure\n",
        ] {
            assert!(matches!(
                LineBslDeclarationExtractor.extract(&module_id(), source),
                Err(BslParseError::MalformedDeclaration { .. })
            ));
        }
    }

    #[test]
    fn bounded_declaration_extraction_rejects_before_the_first_excess_push() {
        let source = concat!(
            "Procedure P0()\nEndProcedure\n",
            "Procedure P1()\nEndProcedure\n",
            "Procedure P2()\nEndProcedure\n",
            "Procedure P3()\nEndProcedure\n",
            "Procedure P4()\nEndProcedure\n",
        );
        assert_eq!(
            LineBslDeclarationExtractor.extract_bounded(&module_id(), source, 4),
            Err(BslParseError::BoundExceeded {
                actual: 5,
                maximum: 4,
            })
        );
    }
}
