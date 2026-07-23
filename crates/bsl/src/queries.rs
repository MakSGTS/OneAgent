//! Extraction of static BSL Query declarations.

use oneagent_common::{EntityId, EntityName};
use std::collections::{BTreeMap, btree_map::Entry};
use std::fmt::{Display, Formatter};

/// A static query declaration found inside a BSL procedure or function.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BslQuery {
    id: EntityId,
    owner_id: EntityId,
    owner_name: EntityName,
    binding_name: EntityName,
    text: String,
    line: usize,
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
}

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

        for (index, line) in source.lines().enumerate() {
            let line_number = index + 1;
            let trimmed = line.trim_start();

            if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with('#') {
                continue;
            }

            if let Some(scope) = parse_scope_start(module_id, trimmed, line_number)? {
                current_scope = Some(scope);
                candidates.clear();
                continue;
            }

            if is_scope_end(trimmed) {
                flush_candidates(&mut queries, &mut candidates);
                current_scope = None;
                continue;
            }

            let Some(scope) = current_scope.as_ref() else {
                continue;
            };

            if let Some(declaration) = parse_query_constructor(trimmed) {
                let key = declaration.binding.as_str().to_lowercase();
                let id = query_id(scope.owner_id(), declaration.binding.as_str())?;
                match candidates.entry(key) {
                    Entry::Occupied(mut entry) => {
                        entry.insert(QueryCandidate::ambiguous());
                    }
                    Entry::Vacant(entry) => {
                        entry.insert(QueryCandidate::new(
                            BslQuery::new(
                                id,
                                scope.owner_id().clone(),
                                scope.owner_name().clone(),
                                declaration.binding,
                                declaration.text.unwrap_or_default(),
                                line_number,
                            ),
                            declaration.has_text,
                        ));
                    }
                }
                continue;
            }

            if let Some(assignment) = parse_query_text_assignment(trimmed) {
                let key = assignment.binding.as_str().to_lowercase();
                let Some(candidate) = candidates.get_mut(&key) else {
                    continue;
                };

                match (
                    candidate.is_supported(),
                    candidate.has_text,
                    assignment.text,
                ) {
                    (true, false, Some(text)) => {
                        candidate.query.text = text;
                        candidate.has_text = true;
                    }
                    _ => candidate.ambiguous = true,
                }
            }
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
    text: Option<String>,
    has_text: bool,
}

#[derive(Debug, Clone)]
struct QueryTextAssignment {
    binding: EntityName,
    text: Option<String>,
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

fn parse_query_constructor(line: &str) -> Option<QueryConstructor> {
    let (left, right) = split_assignment(line)?;
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
        return Some(QueryConstructor {
            binding,
            text: None,
            has_text: false,
        });
    }

    if !(remainder.starts_with('(') && remainder.ends_with(')')) {
        return None;
    }

    let text = parse_static_string_literal(remainder[1..remainder.len() - 1].trim())?;
    Some(QueryConstructor {
        binding,
        text: Some(text),
        has_text: true,
    })
}

fn parse_query_text_assignment(line: &str) -> Option<QueryTextAssignment> {
    let (left, right) = split_assignment(line)?;
    let (binding, property) = left.split_once('.')?;
    if !matches!(property.trim().to_lowercase().as_str(), "text" | "текст") {
        return None;
    }

    let binding = EntityName::new(binding.trim()).ok()?;
    let text = parse_static_string_literal(strip_statement_end(right.trim()));
    Some(QueryTextAssignment { binding, text })
}

fn split_assignment(line: &str) -> Option<(&str, &str)> {
    let (left, right) = line.split_once('=')?;
    if left.trim().is_empty() || right.trim().is_empty() {
        return None;
    }
    Some((left, right))
}

fn strip_statement_end(value: &str) -> &str {
    value.trim().strip_suffix(';').unwrap_or(value).trim()
}

fn parse_static_string_literal(value: &str) -> Option<String> {
    let value = value.trim();
    if !(value.starts_with('"') && value.ends_with('"')) {
        return None;
    }

    let mut text = String::new();
    let mut characters = value[1..value.len() - 1].chars().peekable();
    while let Some(character) = characters.next() {
        if character == '"' {
            if characters.peek() == Some(&'"') {
                characters.next();
                text.push('"');
            } else {
                return None;
            }
        } else {
            text.push(character);
        }
    }

    Some(text)
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

    use super::{BslQueryExtractor, LineBslQueryExtractor};

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
}
