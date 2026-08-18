//! Extraction of static BSL Query declarations.

use oneagent_common::{EntityId, EntityName};
use std::collections::{BTreeMap, btree_map::Entry};
use std::fmt::{Display, Formatter};
use std::ops::Range;

/// A static query declaration found inside a BSL procedure or function.
#[derive(Debug, Clone)]
pub struct BslQuery {
    id: EntityId,
    owner_id: EntityId,
    owner_name: EntityName,
    binding_name: EntityName,
    text: String,
    line: usize,
    source_map: Option<DecodedBslSourceMap>,
}

impl BslQuery {
    /// Creates a BSL query declaration.
    #[must_use]
    pub const fn new(
        id: EntityId,
        owner_id: EntityId,
        owner_name: EntityName,
        binding_name: EntityName,
        text: String,
        line: usize,
    ) -> Self {
        Self {
            id,
            owner_id,
            owner_name,
            binding_name,
            text,
            line,
            source_map: None,
        }
    }

    /// Returns the stable query identifier.
    #[must_use]
    pub const fn id(&self) -> &EntityId {
        &self.id
    }

    /// Returns the owner procedure or function identifier.
    #[must_use]
    pub const fn owner_id(&self) -> &EntityId {
        &self.owner_id
    }

    /// Returns the owner procedure or function name.
    #[must_use]
    pub const fn owner_name(&self) -> &EntityName {
        &self.owner_name
    }

    /// Returns the local binding that declares the query.
    #[must_use]
    pub const fn binding_name(&self) -> &EntityName {
        &self.binding_name
    }

    /// Returns the complete static query text.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Returns the one-based source line of the local query declaration.
    #[must_use]
    pub const fn line(&self) -> usize {
        self.line
    }

    fn set_decoded_text(&mut self, decoded: DecodedBslString) {
        self.text = decoded.text;
        self.source_map = Some(decoded.source_map);
    }
}

impl PartialEq for BslQuery {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.owner_id == other.owner_id
            && self.owner_name == other.owner_name
            && self.binding_name == other.binding_name
            && self.text == other.text
            && self.line == other.line
    }
}

impl Eq for BslQuery {}

/// Extracts static query declarations from BSL source.
pub trait BslQueryExtractor {
    /// Extracts query declarations using `module_id` as the stable module identifier.
    ///
    /// # Errors
    ///
    /// Returns an error when a supported query declaration cannot be represented
    /// by the domain model.
    fn extract_queries(
        &self,
        module_id: &EntityId,
        source: &str,
    ) -> Result<Vec<BslQuery>, BslQueryError>;
}

/// Conservative line-oriented extractor for static BSL Query declarations.
///
/// The first production slice supports a local query binding inside a known
/// procedure or function. The complete query text must be supplied either in
/// the `New Query("...")` constructor or in one static `.Text = "..."` assignment
/// following `New Query`.
#[derive(Debug, Default, Clone, Copy)]
pub struct LineBslQueryExtractor;

impl BslQueryExtractor for LineBslQueryExtractor {
    fn extract_queries(
        &self,
        module_id: &EntityId,
        source: &str,
    ) -> Result<Vec<BslQuery>, BslQueryError> {
        let mut queries = Vec::new();
        let mut current_scope: Option<BslQueryScope> = None;
        let mut candidates = BTreeMap::<String, QueryCandidate>::new();
        let physical_lines = collect_physical_lines(source);
        let mut line_index = 0;

        while line_index < physical_lines.len() {
            let physical_line = &physical_lines[line_index];
            let line = &source[physical_line.content_range.clone()];
            let line_number = physical_line.line_number;
            let trimmed = line.trim_start();
            let trimmed_start = physical_line.content_range.start + line.len() - trimmed.len();

            if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with('#') {
                line_index += 1;
                continue;
            }

            if let Some(scope) = parse_scope_start(module_id, trimmed, line_number)? {
                current_scope = Some(scope);
                candidates.clear();
                line_index += 1;
                continue;
            }

            if is_scope_end(trimmed) {
                flush_candidates(&mut queries, &mut candidates);
                current_scope = None;
                line_index += 1;
                continue;
            }

            let Some(scope) = current_scope.as_ref() else {
                line_index += 1;
                continue;
            };

            if let Some(parsed) =
                parse_query_constructor(source, &physical_lines, line_index, trimmed, trimmed_start)
            {
                let declaration = parsed.value;
                let key = declaration.binding.as_str().to_lowercase();
                let id = query_id(scope.owner_id(), declaration.binding.as_str())?;
                match candidates.entry(key) {
                    Entry::Occupied(mut entry) => {
                        entry.insert(QueryCandidate::ambiguous());
                    }
                    Entry::Vacant(entry) => {
                        let has_text = declaration.text.is_some();
                        let mut query = BslQuery::new(
                            id,
                            scope.owner_id().clone(),
                            scope.owner_name().clone(),
                            declaration.binding,
                            String::new(),
                            line_number,
                        );
                        if let Some(decoded) = declaration.text {
                            debug_assert!(
                                decoded
                                    .source_map
                                    .is_consistent(decoded.text.len(), source.len())
                            );
                            query.set_decoded_text(decoded);
                        }
                        entry.insert(QueryCandidate::new(query, has_text));
                    }
                }
                line_index = parsed.last_line_index + 1;
                continue;
            }

            if let Some(parsed) = parse_query_text_assignment(
                source,
                &physical_lines,
                line_index,
                trimmed,
                trimmed_start,
            ) {
                let assignment = parsed.value;
                let key = assignment.binding.as_str().to_lowercase();
                if let Some(candidate) = candidates.get_mut(&key) {
                    match (
                        candidate.is_supported(),
                        candidate.has_text,
                        assignment.text,
                    ) {
                        (true, false, Some(decoded)) => {
                            debug_assert!(
                                decoded
                                    .source_map
                                    .is_consistent(decoded.text.len(), source.len())
                            );
                            candidate.query.set_decoded_text(decoded);
                            candidate.has_text = true;
                        }
                        _ => candidate.ambiguous = true,
                    }
                }
                line_index = parsed.last_line_index + 1;
                continue;
            }

            line_index += 1;
        }

        flush_candidates(&mut queries, &mut candidates);

        Ok(queries)
    }
}

#[derive(Debug, Clone)]
struct BslQueryScope {
    owner_id: EntityId,
    owner_name: EntityName,
}

impl BslQueryScope {
    const fn new(owner_id: EntityId, owner_name: EntityName) -> Self {
        Self {
            owner_id,
            owner_name,
        }
    }

    const fn owner_id(&self) -> &EntityId {
        &self.owner_id
    }

    const fn owner_name(&self) -> &EntityName {
        &self.owner_name
    }
}

#[derive(Debug, Clone)]
struct QueryConstructor {
    binding: EntityName,
    text: Option<DecodedBslString>,
}

#[derive(Debug, Clone)]
struct QueryTextAssignment {
    binding: EntityName,
    text: Option<DecodedBslString>,
}

#[derive(Debug, Clone)]
struct ParsedLineValue<T> {
    value: T,
    last_line_index: usize,
}

#[derive(Debug, Clone)]
struct DecodedBslString {
    text: String,
    source_map: DecodedBslSourceMap,
}

#[derive(Debug, Clone)]
struct DecodedBslLiteral {
    decoded: DecodedBslString,
    closing_quote_offset: usize,
    last_line_index: usize,
}

#[derive(Debug, Clone)]
struct DecodedBslSourceMap {
    segments: Vec<DecodedBslSourceSegment>,
    physical_lines: Vec<MappedPhysicalLine>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum DecodedBslSourceSegment {
    Copied {
        decoded_range: Range<usize>,
        source_range: Range<usize>,
    },
    CollapsedQuote {
        decoded_range: Range<usize>,
        source_range: Range<usize>,
    },
    InsertedLf {
        decoded_range: Range<usize>,
        previous_line_ending_range: Range<usize>,
        next_indentation_and_marker_range: Range<usize>,
    },
}

impl DecodedBslSourceSegment {
    fn decoded_range(&self) -> &Range<usize> {
        match self {
            Self::Copied { decoded_range, .. }
            | Self::CollapsedQuote { decoded_range, .. }
            | Self::InsertedLf { decoded_range, .. } => decoded_range,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MappedPhysicalLine {
    line_number: usize,
    raw_content_range: Range<usize>,
    line_ending_range: Range<usize>,
    marker_offset: Option<usize>,
    payload_start_offset: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PhysicalSourceLine {
    line_number: usize,
    content_range: Range<usize>,
    line_ending_range: Range<usize>,
}

#[derive(Debug, Clone)]
struct QueryCandidate {
    query: BslQuery,
    has_text: bool,
    ambiguous: bool,
}

impl QueryCandidate {
    const fn new(query: BslQuery, has_text: bool) -> Self {
        Self {
            query,
            has_text,
            ambiguous: false,
        }
    }

    fn ambiguous() -> Self {
        Self {
            query: BslQuery::new(
                EntityId::new("unsupported.query").expect("identifier must be valid"),
                EntityId::new("unsupported.owner").expect("identifier must be valid"),
                EntityName::new("UnsupportedOwner").expect("name must be valid"),
                EntityName::new("UnsupportedQuery").expect("name must be valid"),
                String::new(),
                0,
            ),
            has_text: false,
            ambiguous: true,
        }
    }

    const fn is_supported(&self) -> bool {
        !self.ambiguous
    }
}

fn flush_candidates(
    queries: &mut Vec<BslQuery>,
    candidates: &mut BTreeMap<String, QueryCandidate>,
) {
    for candidate in std::mem::take(candidates).into_values() {
        if candidate.is_supported() && candidate.has_text && !candidate.query.text().is_empty() {
            queries.push(candidate.query);
        }
    }

    queries.sort_by(|left, right| left.id().cmp(right.id()));
    queries.dedup_by(|left, right| left.id() == right.id());
}

fn parse_scope_start(
    module_id: &EntityId,
    line: &str,
    line_number: usize,
) -> Result<Option<BslQueryScope>, BslQueryError> {
    let lowercase = line.to_lowercase();
    let Some((keyword, kind)) = [
        ("procedure", "procedure"),
        ("процедура", "procedure"),
        ("function", "function"),
        ("функция", "function"),
    ]
    .into_iter()
    .find(|(keyword, _)| lowercase.starts_with(*keyword)) else {
        return Ok(None);
    };

    let remainder = line[keyword.len()..].trim_start();
    let Some(open_parenthesis) = remainder.find('(') else {
        return Err(BslQueryError::MalformedScope {
            line: line_number,
            text: line.to_owned(),
        });
    };
    let raw_name = remainder[..open_parenthesis].trim();
    if raw_name.is_empty() {
        return Err(BslQueryError::MalformedScope {
            line: line_number,
            text: line.to_owned(),
        });
    }

    let owner_name =
        EntityName::new(raw_name).map_err(|_| BslQueryError::InvalidName(line_number))?;
    let owner_id = EntityId::new(format!("{}:{}:{}", module_id.as_str(), kind, raw_name))
        .map_err(|_| BslQueryError::InvalidIdentifier(line_number))?;

    Ok(Some(BslQueryScope::new(owner_id, owner_name)))
}

fn is_scope_end(line: &str) -> bool {
    matches!(
        line.trim().to_lowercase().as_str(),
        "endprocedure" | "конецпроцедуры" | "endfunction" | "конецфункции"
    )
}

fn parse_query_constructor(
    source: &str,
    physical_lines: &[PhysicalSourceLine],
    line_index: usize,
    line: &str,
    line_start: usize,
) -> Option<ParsedLineValue<QueryConstructor>> {
    let (left, right, right_start) = split_assignment(line, line_start)?;
    if left.contains('.') {
        return None;
    }

    let binding = EntityName::new(left.trim()).ok()?;
    let constructor = strip_statement_end(right.trim());
    let constructor_lowercase = constructor.to_lowercase();
    let prefix = ["new query", "новый запрос"]
        .into_iter()
        .find(|prefix| constructor_lowercase.starts_with(*prefix))?;
    let remainder = constructor[prefix.len()..].trim();

    if remainder.is_empty() {
        return Some(ParsedLineValue {
            value: QueryConstructor {
                binding,
                text: None,
            },
            last_line_index: line_index,
        });
    }

    if !remainder.starts_with('(') {
        return None;
    }

    let remainder_offset = right.find(remainder)?;
    let after_parenthesis = &remainder[1..];
    let after_parenthesis_trimmed = after_parenthesis.trim_start();
    if !after_parenthesis_trimmed.starts_with('"') {
        return None;
    }
    let opening_quote_offset = right_start + remainder_offset + 1 + after_parenthesis.len()
        - after_parenthesis_trimmed.len();
    let literal =
        decode_static_string_literal(source, physical_lines, line_index, opening_quote_offset)?;
    let closing_line = &physical_lines[literal.last_line_index];
    let suffix = &source[literal.closing_quote_offset + 1..closing_line.content_range.end];
    if strip_statement_end(suffix) != ")" {
        return None;
    }

    Some(ParsedLineValue {
        value: QueryConstructor {
            binding,
            text: Some(literal.decoded),
        },
        last_line_index: literal.last_line_index,
    })
}

fn parse_query_text_assignment(
    source: &str,
    physical_lines: &[PhysicalSourceLine],
    line_index: usize,
    line: &str,
    line_start: usize,
) -> Option<ParsedLineValue<QueryTextAssignment>> {
    let (left, right, right_start) = split_assignment(line, line_start)?;
    let (binding, property) = left.split_once('.')?;
    if !matches!(property.trim().to_lowercase().as_str(), "text" | "текст") {
        return None;
    }

    let binding = EntityName::new(binding.trim()).ok()?;
    let value = right.trim_start();
    let mut last_line_index = line_index;
    let text = if value.starts_with('"') {
        let opening_quote_offset = right_start + right.len() - value.len();
        decode_static_string_literal(source, physical_lines, line_index, opening_quote_offset)
            .and_then(|literal| {
                let closing_line = &physical_lines[literal.last_line_index];
                let suffix =
                    &source[literal.closing_quote_offset + 1..closing_line.content_range.end];
                if strip_statement_end(suffix).is_empty() {
                    last_line_index = literal.last_line_index;
                    Some(literal.decoded)
                } else {
                    None
                }
            })
    } else {
        None
    };

    Some(ParsedLineValue {
        value: QueryTextAssignment { binding, text },
        last_line_index,
    })
}

fn split_assignment(line: &str, line_start: usize) -> Option<(&str, &str, usize)> {
    let assignment_offset = line.find('=')?;
    let left = &line[..assignment_offset];
    let right = &line[assignment_offset + 1..];
    if left.trim().is_empty() || right.trim().is_empty() {
        return None;
    }
    Some((left, right, line_start + assignment_offset + 1))
}

fn strip_statement_end(value: &str) -> &str {
    value.trim().strip_suffix(';').unwrap_or(value).trim()
}

fn decode_static_string_literal(
    source: &str,
    physical_lines: &[PhysicalSourceLine],
    opening_line_index: usize,
    opening_quote_offset: usize,
) -> Option<DecodedBslLiteral> {
    if source.as_bytes().get(opening_quote_offset) != Some(&b'"') {
        return None;
    }

    let mut segments = Vec::new();
    let mut mapped_lines = Vec::new();
    let mut text = String::new();
    let mut line_index = opening_line_index;
    let mut payload_start = opening_quote_offset + 1;
    let mut marker_offset = None;

    loop {
        let line = physical_lines.get(line_index)?;
        if payload_start > line.content_range.end {
            return None;
        }
        mapped_lines.push(MappedPhysicalLine {
            line_number: line.line_number,
            raw_content_range: line.content_range.clone(),
            line_ending_range: line.line_ending_range.clone(),
            marker_offset,
            payload_start_offset: payload_start,
        });

        let mut cursor = payload_start;
        let mut copied_start = cursor;
        while cursor < line.content_range.end {
            if source.as_bytes()[cursor] != b'"' {
                cursor += 1;
                continue;
            }

            if copied_start < cursor {
                push_copied_segment(source, copied_start..cursor, &mut text, &mut segments);
            }
            if cursor + 1 < line.content_range.end && source.as_bytes()[cursor + 1] == b'"' {
                let decoded_start = text.len();
                text.push('"');
                segments.push(DecodedBslSourceSegment::CollapsedQuote {
                    decoded_range: decoded_start..text.len(),
                    source_range: cursor..cursor + 2,
                });
                cursor += 2;
                copied_start = cursor;
                continue;
            }

            return Some(DecodedBslLiteral {
                decoded: DecodedBslString {
                    text,
                    source_map: DecodedBslSourceMap {
                        segments,
                        physical_lines: mapped_lines,
                    },
                },
                closing_quote_offset: cursor,
                last_line_index: line_index,
            });
        }

        if copied_start < line.content_range.end {
            push_copied_segment(
                source,
                copied_start..line.content_range.end,
                &mut text,
                &mut segments,
            );
        }
        if line.line_ending_range.is_empty() {
            return None;
        }

        let next_line_index = line_index + 1;
        let next_line = physical_lines.get(next_line_index)?;
        let next_content = &source[next_line.content_range.clone()];
        let indentation_bytes = next_content
            .as_bytes()
            .iter()
            .take_while(|byte| matches!(byte, b' ' | b'\t'))
            .count();
        let next_marker_offset = next_line.content_range.start + indentation_bytes;
        if source.as_bytes().get(next_marker_offset) != Some(&b'|') {
            return None;
        }
        let next_payload_start = next_marker_offset + 1;
        let decoded_start = text.len();
        text.push('\n');
        segments.push(DecodedBslSourceSegment::InsertedLf {
            decoded_range: decoded_start..text.len(),
            previous_line_ending_range: line.line_ending_range.clone(),
            next_indentation_and_marker_range: next_line.content_range.start..next_payload_start,
        });

        line_index = next_line_index;
        marker_offset = Some(next_marker_offset);
        payload_start = next_payload_start;
    }
}

fn push_copied_segment(
    source: &str,
    source_range: Range<usize>,
    text: &mut String,
    segments: &mut Vec<DecodedBslSourceSegment>,
) {
    let decoded_start = text.len();
    text.push_str(&source[source_range.clone()]);
    segments.push(DecodedBslSourceSegment::Copied {
        decoded_range: decoded_start..text.len(),
        source_range,
    });
}

fn collect_physical_lines(source: &str) -> Vec<PhysicalSourceLine> {
    let mut lines = Vec::new();
    let mut line_start = 0;
    let mut line_number = 1;

    for (newline_offset, _) in source.match_indices('\n') {
        let content_end =
            if newline_offset > line_start && source.as_bytes()[newline_offset - 1] == b'\r' {
                newline_offset - 1
            } else {
                newline_offset
            };
        lines.push(PhysicalSourceLine {
            line_number,
            content_range: line_start..content_end,
            line_ending_range: content_end..newline_offset + 1,
        });
        line_start = newline_offset + 1;
        line_number += 1;
    }

    if line_start < source.len() || source.is_empty() {
        lines.push(PhysicalSourceLine {
            line_number,
            content_range: line_start..source.len(),
            line_ending_range: source.len()..source.len(),
        });
    }

    lines
}

impl DecodedBslSourceMap {
    fn is_consistent(&self, decoded_len: usize, source_len: usize) -> bool {
        let mut next_decoded_offset = 0;
        for segment in &self.segments {
            let decoded_range = segment.decoded_range();
            if decoded_range.start != next_decoded_offset || decoded_range.end > decoded_len {
                return false;
            }
            let source_ranges_are_valid = match segment {
                DecodedBslSourceSegment::Copied { source_range, .. }
                | DecodedBslSourceSegment::CollapsedQuote { source_range, .. } => {
                    source_range.start <= source_range.end && source_range.end <= source_len
                }
                DecodedBslSourceSegment::InsertedLf {
                    previous_line_ending_range,
                    next_indentation_and_marker_range,
                    ..
                } => {
                    previous_line_ending_range.start < previous_line_ending_range.end
                        && previous_line_ending_range.end <= source_len
                        && next_indentation_and_marker_range.start
                            < next_indentation_and_marker_range.end
                        && next_indentation_and_marker_range.end <= source_len
                }
            };
            if !source_ranges_are_valid {
                return false;
            }
            next_decoded_offset = decoded_range.end;
        }

        next_decoded_offset == decoded_len
            && self.physical_lines.iter().all(|line| {
                line.line_number > 0
                    && line.raw_content_range.start <= line.raw_content_range.end
                    && line.raw_content_range.end <= source_len
                    && line.line_ending_range.start <= line.line_ending_range.end
                    && line.line_ending_range.end <= source_len
                    && line.payload_start_offset >= line.raw_content_range.start
                    && line.payload_start_offset <= line.raw_content_range.end
                    && line.marker_offset.is_none_or(|marker| {
                        marker >= line.raw_content_range.start && marker < line.payload_start_offset
                    })
            })
    }
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BslSourcePosition {
    line: usize,
    column: usize,
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BslSourceProjection {
    start: BslSourcePosition,
    end: BslSourcePosition,
}

#[cfg(test)]
impl DecodedBslSourceMap {
    fn project(&self, source: &str, decoded_range: Range<usize>) -> Option<BslSourceProjection> {
        let mut projected_start = None;
        let mut projected_end = None;

        for segment in &self.segments {
            let segment_range = segment.decoded_range();
            let overlap_start = decoded_range.start.max(segment_range.start);
            let overlap_end = decoded_range.end.min(segment_range.end);
            if overlap_start >= overlap_end {
                continue;
            }

            let source_range = match segment {
                DecodedBslSourceSegment::Copied {
                    decoded_range,
                    source_range,
                } => {
                    let relative_start = overlap_start - decoded_range.start;
                    let relative_end = overlap_end - decoded_range.start;
                    source_range.start + relative_start..source_range.start + relative_end
                }
                DecodedBslSourceSegment::CollapsedQuote { source_range, .. } => {
                    source_range.clone()
                }
                DecodedBslSourceSegment::InsertedLf {
                    next_indentation_and_marker_range,
                    ..
                } => {
                    next_indentation_and_marker_range.end - 1..next_indentation_and_marker_range.end
                }
            };
            projected_start.get_or_insert(source_range.start);
            projected_end = Some(source_range.end);
        }

        Some(BslSourceProjection {
            start: self.source_position(source, projected_start?)?,
            end: self.source_position(source, projected_end?)?,
        })
    }

    fn source_position(&self, source: &str, offset: usize) -> Option<BslSourcePosition> {
        let line = self.physical_lines.iter().find(|line| {
            offset >= line.raw_content_range.start && offset <= line.raw_content_range.end
        })?;
        Some(BslSourcePosition {
            line: line.line_number,
            column: source[line.raw_content_range.start..offset].chars().count() + 1,
        })
    }
}

fn query_id(owner_id: &EntityId, binding_name: &str) -> Result<EntityId, BslQueryError> {
    EntityId::new(format!("{}:query:{}", owner_id.as_str(), binding_name))
        .map_err(|_| BslQueryError::InvalidIdentifier(0))
}

/// Error produced while extracting static BSL query declarations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BslQueryError {
    /// A query or scope name could not be represented.
    InvalidName(usize),

    /// A query or owner identifier could not be represented.
    InvalidIdentifier(usize),

    /// A procedure or function declaration is malformed.
    MalformedScope {
        /// One-based source line.
        line: usize,

        /// Original source text.
        text: String,
    },
}

impl Display for BslQueryError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidName(line) => {
                write!(formatter, "invalid BSL query name at line {line}")
            }
            Self::InvalidIdentifier(line) => {
                write!(formatter, "invalid BSL query identifier at line {line}")
            }
            Self::MalformedScope { line, text } => {
                write!(
                    formatter,
                    "malformed BSL query scope declaration at line {line}: {text}"
                )
            }
        }
    }
}

impl std::error::Error for BslQueryError {}

#[cfg(test)]
mod tests {
    use oneagent_common::EntityId;

    use super::{
        BslQueryExtractor, BslSourcePosition, DecodedBslSourceSegment, LineBslQueryExtractor,
    };

    fn module_id() -> EntityId {
        EntityId::new("module.sales.object").expect("identifier must be valid")
    }

    #[test]
    fn extracts_constructor_query_inside_known_scope() {
        let source = r#"
Procedure Post()
    Query = New Query("SELECT Ref FROM Catalog.Products");
EndProcedure
"#;

        let queries = LineBslQueryExtractor
            .extract_queries(&module_id(), source)
            .expect("queries must parse");

        assert_eq!(queries.len(), 1);
        assert_eq!(queries[0].binding_name().as_str(), "Query");
        assert_eq!(queries[0].owner_name().as_str(), "Post");
        assert_eq!(
            queries[0].id().as_str(),
            "module.sales.object:procedure:Post:query:Query"
        );
        assert_eq!(queries[0].text(), "SELECT Ref FROM Catalog.Products");
    }

    #[test]
    fn extracts_text_assignment_query_inside_known_scope() {
        let source = r#"
Функция Получить()
    Запрос = Новый Запрос;
    Запрос.Текст = "ВЫБРАТЬ Ссылка ИЗ Справочник.Номенклатура";
КонецФункции
"#;

        let queries = LineBslQueryExtractor
            .extract_queries(&module_id(), source)
            .expect("queries must parse");

        assert_eq!(queries.len(), 1);
        assert_eq!(queries[0].binding_name().as_str(), "Запрос");
        assert_eq!(queries[0].owner_name().as_str(), "Получить");
        assert_eq!(
            queries[0].id().as_str(),
            "module.sales.object:function:Получить:query:Запрос"
        );
    }

    #[test]
    fn same_local_binding_in_different_owners_does_not_collide() {
        let source = r#"
Procedure Post()
    Query = New Query("SELECT Ref FROM Catalog.Products");
EndProcedure

Procedure Check()
    Query = New Query("SELECT Ref FROM Catalog.Products");
EndProcedure
"#;

        let queries = LineBslQueryExtractor
            .extract_queries(&module_id(), source)
            .expect("queries must parse");

        assert_eq!(queries.len(), 2);
        assert_ne!(queries[0].id(), queries[1].id());
    }

    #[test]
    fn ignores_dynamic_and_ambiguous_query_patterns() {
        let source = r#"
Procedure Post()
    Query = New Query;
    Query.Text = Text;
    OtherQuery = New Query;
    OtherQuery.Text = "SELECT Ref FROM Catalog.Products";
    OtherQuery.Text = "SELECT Ref FROM Catalog.Services";
    PlainText = "SELECT Ref FROM Catalog.Products";
EndProcedure
"#;

        let queries = LineBslQueryExtractor
            .extract_queries(&module_id(), source)
            .expect("queries must parse");

        assert!(queries.is_empty());
    }

    #[test]
    fn ignores_query_without_known_scope_or_static_text() {
        let source = r#"
Query = New Query("SELECT Ref FROM Catalog.Products");

Procedure Post()
    Query = New Query;
EndProcedure
"#;

        let queries = LineBslQueryExtractor
            .extract_queries(&module_id(), source)
            .expect("queries must parse");

        assert!(queries.is_empty());
    }

    #[test]
    fn multiline_constructor_decodes_confirmed_fragments_and_retains_private_map() {
        let source = "Procedure Post()\n    Query = New Query(\"OPEN\n \t|  NEXT \"\"quoted\"\"\n    |\n    |Юникод\");\nEndProcedure\n";

        let queries = LineBslQueryExtractor
            .extract_queries(&module_id(), source)
            .expect("multiline query must parse");

        assert_eq!(queries.len(), 1);
        let query = &queries[0];
        assert_eq!(query.text(), "OPEN\n  NEXT \"quoted\"\n\nЮникод");
        assert_eq!(query.line(), 2);
        assert_eq!(
            query.id().as_str(),
            "module.sales.object:procedure:Post:query:Query"
        );
        let source_map = query
            .source_map
            .as_ref()
            .expect("extracted query must retain a private source map");
        assert_eq!(source_map.physical_lines.len(), 4);
        assert_eq!(source_map.physical_lines[0].line_number, 2);
        assert_eq!(source_map.physical_lines[3].line_number, 5);
        assert!(
            source_map
                .segments
                .iter()
                .any(|segment| matches!(segment, DecodedBslSourceSegment::Copied { .. }))
        );
        assert!(source_map.segments.iter().any(|segment| matches!(
            segment,
            DecodedBslSourceSegment::CollapsedQuote { source_range, .. }
                if source_range.end - source_range.start == 2
        )));
        assert_eq!(
            source_map
                .segments
                .iter()
                .filter(|segment| matches!(segment, DecodedBslSourceSegment::InsertedLf { .. }))
                .count(),
            3
        );

        let first_lf = query.text().find('\n').expect("decoded LF must exist");
        let lf_projection = source_map
            .project(source, first_lf..first_lf + 1)
            .expect("inserted LF must project to the continuation marker");
        assert_eq!(
            lf_projection.start,
            BslSourcePosition { line: 3, column: 3 }
        );
        assert_eq!(lf_projection.end, BslSourcePosition { line: 3, column: 4 });

        let unicode_start = query.text().find('Ю').expect("Unicode payload must exist");
        let unicode_projection = source_map
            .project(source, unicode_start..unicode_start + 'Ю'.len_utf8())
            .expect("copied Unicode bytes must project to BSL scalar columns");
        assert_eq!(
            unicode_projection.start,
            BslSourcePosition { line: 5, column: 6 }
        );
        assert_eq!(
            unicode_projection.end,
            BslSourcePosition { line: 5, column: 7 }
        );
    }

    #[test]
    fn multiline_text_assignment_and_line_endings_are_equivalent() {
        let lf = "Функция Получить()\n\tЗапрос = Новый Запрос;\n\tЗапрос.Текст = \"SELECT\n\t\t| Ref\n\t\t|FROM Catalog.Products\";\nКонецФункции\n";
        let crlf = lf.replace('\n', "\r\n");

        let lf_queries = LineBslQueryExtractor
            .extract_queries(&module_id(), lf)
            .expect("LF query must parse");
        let crlf_queries = LineBslQueryExtractor
            .extract_queries(&module_id(), &crlf)
            .expect("CRLF query must parse");

        assert_eq!(lf_queries, crlf_queries);
        assert_eq!(lf_queries[0].text(), "SELECT\n Ref\nFROM Catalog.Products");
        assert_eq!(lf_queries[0].line(), 2);
        assert_eq!(
            lf_queries[0].binding_name().as_str(),
            crlf_queries[0].binding_name().as_str()
        );
    }

    #[test]
    fn multiline_extraction_remains_conservative_for_non_static_final_text() {
        let source = r#"Procedure Post()
    Dynamic = New Query;
    Dynamic.Text = GetText();
    Replaced = New Query;
    Replaced.Text = "SELECT
    | Ref FROM Catalog.Products";
    Replaced.Text = StrReplace(Replaced.Text, "Products", "Services");
    Reassigned = New Query("SELECT
    | Ref FROM Catalog.Products");
    Reassigned.Text = "SELECT Ref FROM Catalog.Services";
    Returned = New Query(GetText());
    Incomplete = New Query("SELECT
    | Ref FROM Catalog.Products
EndProcedure
"#;

        let queries = LineBslQueryExtractor
            .extract_queries(&module_id(), source)
            .expect("unsupported candidates must remain recoverable");

        assert!(queries.is_empty());
    }

    #[test]
    fn repeated_multiline_extraction_is_deterministic_and_one_line_behavior_is_preserved() {
        let source = r#"Query = New Query("SELECT Ref FROM Catalog.Outside");
Procedure Post()
    Query = New Query("SELECT Ref FROM Catalog.Products");
    Other = New Query("SELECT
    | Ref FROM Catalog.Services");
EndProcedure
"#;

        let first = LineBslQueryExtractor
            .extract_queries(&module_id(), source)
            .expect("queries must parse");
        let repeated = LineBslQueryExtractor
            .extract_queries(&module_id(), source)
            .expect("repeated queries must parse");

        assert_eq!(first, repeated);
        assert_eq!(first.len(), 2);
        assert!(
            first
                .iter()
                .any(|query| query.text() == "SELECT Ref FROM Catalog.Products")
        );
        assert!(
            first
                .iter()
                .any(|query| query.text() == "SELECT\n Ref FROM Catalog.Services")
        );
    }
}
