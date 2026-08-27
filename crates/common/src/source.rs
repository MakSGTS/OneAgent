use std::fmt::{Display, Formatter};
use std::str::FromStr;

/// Maximum UTF-8 size accepted for one source evidence path.
pub const MAX_SOURCE_PATH_BYTES: usize = 4_096;

/// Validated UTF-8 path retained as source evidence.
///
/// Separators are stored as `/`. Containment remains a consumer concern because
/// producer evidence may be absolute or relative to a producer-owned root.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourcePath(String);

impl SourcePath {
    /// Creates a canonical source evidence path.
    ///
    /// # Errors
    ///
    /// Returns [`SourcePathError`] for an empty, over-bound, malformed, or
    /// traversal-bearing value.
    pub fn new(value: impl Into<String>) -> Result<Self, SourcePathError> {
        let value = value.into().replace('\\', "/");
        validate_source_path(&value)?;
        Ok(Self(value))
    }

    /// Returns the slash-normalized path.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for SourcePath {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl FromStr for SourcePath {
    type Err = SourcePathError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}

fn validate_source_path(value: &str) -> Result<(), SourcePathError> {
    if value.is_empty() {
        return Err(SourcePathError::new(SourcePathErrorKind::Empty));
    }
    if value.len() > MAX_SOURCE_PATH_BYTES {
        return Err(SourcePathError::new(SourcePathErrorKind::TooLong));
    }
    if value.contains('\0') {
        return Err(SourcePathError::new(SourcePathErrorKind::Malformed));
    }
    if value.ends_with('/') {
        return Err(SourcePathError::new(SourcePathErrorKind::Malformed));
    }

    let components = if let Some(rest) = value.strip_prefix("//") {
        rest
    } else if let Some(rest) = value.strip_prefix('/') {
        rest
    } else if value.as_bytes().get(1) == Some(&b':') {
        let bytes = value.as_bytes();
        if !bytes[0].is_ascii_alphabetic() || bytes.get(2) != Some(&b'/') {
            return Err(SourcePathError::new(SourcePathErrorKind::Malformed));
        }
        &value[3..]
    } else {
        value
    };

    if components.is_empty() {
        return Err(SourcePathError::new(SourcePathErrorKind::Malformed));
    }
    for component in components.split('/') {
        if component.is_empty() {
            return Err(SourcePathError::new(SourcePathErrorKind::Malformed));
        }
        if component == "." || component == ".." {
            return Err(SourcePathError::new(SourcePathErrorKind::Traversal));
        }
    }
    Ok(())
}

/// Closed source-path validation failure kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourcePathErrorKind {
    /// The value is empty.
    Empty,
    /// The UTF-8 byte bound is exceeded.
    TooLong,
    /// A `.` or `..` traversal component is present.
    Traversal,
    /// The path has an unsupported shape or separator layout.
    Malformed,
}

/// Error returned for an invalid source evidence path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourcePathError {
    kind: SourcePathErrorKind,
}

impl SourcePathError {
    const fn new(kind: SourcePathErrorKind) -> Self {
        Self { kind }
    }

    /// Returns the closed failure kind.
    #[must_use]
    pub const fn kind(self) -> SourcePathErrorKind {
        self.kind
    }
}

impl Display for SourcePathError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self.kind {
            SourcePathErrorKind::Empty => "source path must not be empty",
            SourcePathErrorKind::TooLong => "source path exceeds the byte bound",
            SourcePathErrorKind::Traversal => "source path contains a traversal component",
            SourcePathErrorKind::Malformed => "source path is malformed",
        })
    }
}

impl std::error::Error for SourcePathError {}

/// One-based source coordinate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourcePosition {
    line: u32,
    column: u32,
}

impl SourcePosition {
    /// Creates a one-based source position.
    ///
    /// # Errors
    ///
    /// Returns [`SourcePositionError`] when either coordinate is zero.
    pub const fn new(line: u32, column: u32) -> Result<Self, SourcePositionError> {
        if line == 0 || column == 0 {
            return Err(SourcePositionError);
        }
        Ok(Self { line, column })
    }

    /// Returns the one-based line.
    #[must_use]
    pub const fn line(self) -> u32 {
        self.line
    }

    /// Returns the one-based column.
    #[must_use]
    pub const fn column(self) -> u32 {
        self.column
    }
}

/// Error returned for a zero source coordinate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourcePositionError;

impl Display for SourcePositionError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("source position coordinates must be one-based")
    }
}

impl std::error::Error for SourcePositionError {}

/// Half-open source range, or a navigation point when both positions are equal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourceSpan {
    start: SourcePosition,
    end: SourcePosition,
}

impl SourceSpan {
    /// Creates an ordered half-open source span.
    ///
    /// # Errors
    ///
    /// Returns [`SourceSpanError`] when `end` precedes `start`.
    pub const fn new(start: SourcePosition, end: SourcePosition) -> Result<Self, SourceSpanError> {
        if end.line < start.line || (end.line == start.line && end.column < start.column) {
            return Err(SourceSpanError);
        }
        Ok(Self { start, end })
    }

    /// Returns the inclusive start position.
    #[must_use]
    pub const fn start(self) -> SourcePosition {
        self.start
    }

    /// Returns the exclusive end position, equal to start for a point.
    #[must_use]
    pub const fn end(self) -> SourcePosition {
        self.end
    }
}

/// Error returned for a reversed source span.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceSpanError;

impl Display for SourceSpanError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("source span end must not precede its start")
    }
}

impl std::error::Error for SourceSpanError {}

/// Typed source evidence attached to a semantic fact.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourceLocation {
    path: SourcePath,
    span: Option<SourceSpan>,
}

impl SourceLocation {
    /// Creates file or ranged source evidence.
    #[must_use]
    pub const fn new(path: SourcePath, span: Option<SourceSpan>) -> Self {
        Self { path, span }
    }

    /// Returns the source evidence path.
    #[must_use]
    pub const fn path(&self) -> &SourcePath {
        &self.path
    }

    /// Returns the optional half-open source span.
    #[must_use]
    pub const fn span(&self) -> Option<SourceSpan> {
        self.span
    }
}

#[cfg(test)]
mod tests {
    use super::{
        MAX_SOURCE_PATH_BYTES, SourceLocation, SourcePath, SourcePathErrorKind, SourcePosition,
        SourceSpan,
    };

    #[test]
    fn source_path_normalizes_supported_absolute_and_relative_values() {
        let relative = SourcePath::new("src\\CommonModules\\Sales\\Module.bsl")
            .expect("relative source path must pass");
        let posix = SourcePath::new("/workspace/configuration/Module.bsl")
            .expect("POSIX source path must pass");
        let windows = SourcePath::new("C:\\workspace\\configuration\\Module.bsl")
            .expect("Windows source path must pass");
        let unc =
            SourcePath::new("\\\\server\\share\\Module.bsl").expect("UNC source path must pass");

        assert_eq!(relative.as_str(), "src/CommonModules/Sales/Module.bsl");
        assert_eq!(posix.as_str(), "/workspace/configuration/Module.bsl");
        assert_eq!(windows.as_str(), "C:/workspace/configuration/Module.bsl");
        assert_eq!(unc.as_str(), "//server/share/Module.bsl");
    }

    #[test]
    fn source_path_enforces_exact_bound_and_closed_invalid_shapes() {
        assert!(SourcePath::new("a".repeat(MAX_SOURCE_PATH_BYTES)).is_ok());
        assert_eq!(
            SourcePath::new("a".repeat(MAX_SOURCE_PATH_BYTES + 1))
                .expect_err("one-over path must fail")
                .kind(),
            SourcePathErrorKind::TooLong
        );

        for (value, kind) in [
            ("", SourcePathErrorKind::Empty),
            ("src//Module.bsl", SourcePathErrorKind::Malformed),
            ("src/./Module.bsl", SourcePathErrorKind::Traversal),
            ("src/../Module.bsl", SourcePathErrorKind::Traversal),
            ("src/Module.bsl/", SourcePathErrorKind::Malformed),
            ("C:Module.bsl", SourcePathErrorKind::Malformed),
            ("src/\0/Module.bsl", SourcePathErrorKind::Malformed),
        ] {
            assert_eq!(
                SourcePath::new(value)
                    .expect_err("invalid source path must fail")
                    .kind(),
                kind
            );
        }
    }

    #[test]
    fn source_positions_spans_and_locations_are_one_based_and_ordered() {
        assert!(SourcePosition::new(0, 1).is_err());
        assert!(SourcePosition::new(1, 0).is_err());
        let start = SourcePosition::new(12, 1).expect("position must pass");
        let end = SourcePosition::new(12, 8).expect("position must pass");
        let point = SourceSpan::new(start, start).expect("point span must pass");
        let range = SourceSpan::new(start, end).expect("ordered span must pass");

        assert!(SourceSpan::new(end, start).is_err());
        assert_eq!(point.start(), point.end());
        assert_eq!(range.start().line(), 12);
        assert_eq!(range.end().column(), 8);

        let path = SourcePath::new("src/Module.bsl").expect("path must pass");
        let location = SourceLocation::new(path.clone(), Some(point));
        assert_eq!(location.path(), &path);
        assert_eq!(location.span(), Some(point));
    }
}
