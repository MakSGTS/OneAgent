use oneagent_metadata::MetadataKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NodeKind {
    /// Metadata object node.
    Metadata(MetadataKind),
    /// Module node.
    Module,
    /// Procedure node.
    Procedure,
    /// Function node.
    Function,
    /// Query node.
    Query,
    /// Report-owned Data Composition Schema node.
    DataCompositionSchema,
    /// Direct Data Set node owned by a Data Composition Schema.
    DataSet,
    /// Direct named Data Composition Field node owned by a Data Set.
    DataCompositionField,
    /// Form node.
    Form,
    /// Command node.
    Command,
    /// Attribute node.
    Attribute,
    /// Platform-provided standard attribute node.
    StandardAttribute,
    /// Tabular section node.
    TabularSection,
    /// Register dimension node.
    Dimension,
    /// Register resource node.
    Resource,
    /// Accounting register measure node.
    Measure,
    /// Role node.
    Role,
    /// Scoped access-right node.
    AccessRight,
    /// Subsystem node.
    Subsystem,
    /// Unknown or not-yet-supported node.
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EdgeKind {
    /// Parent contains child.
    Contains,
    /// Source calls target.
    Calls,
    /// Source references target.
    References,
    /// Source reads target.
    Reads,
    /// Source writes target.
    Writes,
    /// Source grants access to target.
    Grants,
    /// Source includes target.
    Includes,
    /// Source extends target.
    Extends,
    /// Source depends on target.
    DependsOn,
    /// Source opens target form.
    Opens,
    /// Event subscription invokes its handler procedure.
    Triggers,
}
