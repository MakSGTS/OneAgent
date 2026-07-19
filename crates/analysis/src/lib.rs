//! Semantic analysis pipeline for `OneAgent`.
//!
//! The pipeline intentionally uses two passes:
//!
//! 1. collect every module and declaration into the semantic graph;
//! 2. extract and resolve local and cross-module calls.

use oneagent_bsl::{
    BslCallError, BslCallExtractor, BslCallResolver, BslDeclarationExtractor, BslModuleSymbols,
    BslParseError, CrossModuleCallResolver, LineBslCallExtractor, LineBslDeclarationExtractor,
    LocalBslCallResolver, QualifiedBslCallResolver, UnresolvedBslCall, UnresolvedCrossModuleCall,
};
use oneagent_common::{EntityId, EntityName};
use oneagent_graph::{EdgeKind, GraphEdge, GraphError, GraphNode, NodeKind, SemanticGraph};
use std::fmt::{Display, Formatter};

/// Source module supplied to the semantic analysis pipeline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnalysisModule {
    id: EntityId,
    name: EntityName,
    source: String,
}

impl AnalysisModule {
    /// Creates a module analysis input.
    #[must_use]
    pub fn new(id: EntityId, name: EntityName, source: impl Into<String>) -> Self {
        Self {
            id,
            name,
            source: source.into(),
        }
    }

    /// Returns the stable module identifier.
    #[must_use]
    pub const fn id(&self) -> &EntityId {
        &self.id
    }

    /// Returns the logical module name used by qualified BSL calls.
    #[must_use]
    pub const fn name(&self) -> &EntityName {
        &self.name
    }

    /// Returns the BSL source.
    #[must_use]
    pub fn source(&self) -> &str {
        &self.source
    }
}

/// Calls that were not resolved by the current pipeline.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct AnalysisDiagnostics {
    unresolved_local_calls: Vec<ModuleLocalDiagnostic>,
    unresolved_cross_module_calls: Vec<ModuleCrossModuleDiagnostic>,
}

impl AnalysisDiagnostics {
    /// Returns unresolved local-call diagnostics.
    #[must_use]
    pub fn unresolved_local_calls(&self) -> &[ModuleLocalDiagnostic] {
        &self.unresolved_local_calls
    }

    /// Returns unresolved qualified-call diagnostics.
    #[must_use]
    pub fn unresolved_cross_module_calls(&self) -> &[ModuleCrossModuleDiagnostic] {
        &self.unresolved_cross_module_calls
    }

    /// Returns `true` when no unresolved calls were recorded.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.unresolved_local_calls.is_empty() && self.unresolved_cross_module_calls.is_empty()
    }
}

/// Local resolution diagnostic associated with its source module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleLocalDiagnostic {
    module_id: EntityId,
    call: UnresolvedBslCall,
}

impl ModuleLocalDiagnostic {
    /// Creates a module-local diagnostic.
    #[must_use]
    pub const fn new(module_id: EntityId, call: UnresolvedBslCall) -> Self {
        Self { module_id, call }
    }

    /// Returns the module identifier.
    #[must_use]
    pub const fn module_id(&self) -> &EntityId {
        &self.module_id
    }

    /// Returns the unresolved call.
    #[must_use]
    pub const fn call(&self) -> &UnresolvedBslCall {
        &self.call
    }
}

/// Cross-module resolution diagnostic associated with its source module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleCrossModuleDiagnostic {
    module_id: EntityId,
    call: UnresolvedCrossModuleCall,
}

impl ModuleCrossModuleDiagnostic {
    /// Creates a cross-module diagnostic.
    #[must_use]
    pub const fn new(module_id: EntityId, call: UnresolvedCrossModuleCall) -> Self {
        Self { module_id, call }
    }

    /// Returns the module identifier.
    #[must_use]
    pub const fn module_id(&self) -> &EntityId {
        &self.module_id
    }

    /// Returns the unresolved call.
    #[must_use]
    pub const fn call(&self) -> &UnresolvedCrossModuleCall {
        &self.call
    }
}

/// Result of semantic analysis.
#[derive(Debug, Clone)]
pub struct AnalysisResult {
    graph: SemanticGraph,
    diagnostics: AnalysisDiagnostics,
}

impl AnalysisResult {
    /// Creates an analysis result.
    #[must_use]
    pub const fn new(graph: SemanticGraph, diagnostics: AnalysisDiagnostics) -> Self {
        Self { graph, diagnostics }
    }

    /// Returns the generated semantic graph.
    #[must_use]
    pub const fn graph(&self) -> &SemanticGraph {
        &self.graph
    }

    /// Consumes the result and returns the semantic graph.
    #[must_use]
    pub fn into_graph(self) -> SemanticGraph {
        self.graph
    }

    /// Returns analysis diagnostics.
    #[must_use]
    pub const fn diagnostics(&self) -> &AnalysisDiagnostics {
        &self.diagnostics
    }
}

/// Two-pass semantic analysis pipeline.
#[derive(Debug, Default, Clone, Copy)]
pub struct SemanticAnalysisPipeline;

impl SemanticAnalysisPipeline {
    /// Builds a semantic graph from all supplied BSL modules.
    ///
    /// The complete declaration set is collected before any call is resolved.
    ///
    /// # Errors
    ///
    /// Returns an error when declarations or calls cannot be extracted, or when
    /// an invalid graph edge is produced.
    pub fn analyze(&self, modules: &[AnalysisModule]) -> Result<AnalysisResult, AnalysisError> {
        let mut context = BuildContext::default();

        Self::collect_symbols(modules, &mut context)?;
        Self::resolve_calls(&mut context)?;

        Ok(AnalysisResult::new(context.graph, context.diagnostics))
    }

    fn collect_symbols(
        modules: &[AnalysisModule],
        context: &mut BuildContext,
    ) -> Result<(), AnalysisError> {
        for module in modules {
            let symbols = LineBslDeclarationExtractor
                .extract(module.id(), module.source())
                .map_err(|source| AnalysisError::DeclarationExtraction {
                    module_id: module.id().clone(),
                    source,
                })?;

            context.graph.insert_node(GraphNode::new(
                module.id().clone(),
                module.name().clone(),
                NodeKind::Module,
            ));

            for symbol in &symbols {
                context.graph.insert_node(GraphNode::new(
                    symbol.id().clone(),
                    symbol.name().clone(),
                    match symbol.kind() {
                        oneagent_bsl::BslSymbolKind::Procedure => NodeKind::Procedure,
                        oneagent_bsl::BslSymbolKind::Function => NodeKind::Function,
                    },
                ));

                context.graph.insert_edge(GraphEdge::new(
                    module.id().clone(),
                    symbol.id().clone(),
                    EdgeKind::Contains,
                ))?;
            }

            context.modules.push(CollectedModule {
                source: module.source().to_owned(),
                symbols: BslModuleSymbols::new(module.id().clone(), module.name().clone(), symbols),
            });
        }

        Ok(())
    }

    fn resolve_calls(context: &mut BuildContext) -> Result<(), AnalysisError> {
        let available_modules = context
            .modules
            .iter()
            .map(|module| module.symbols.clone())
            .collect::<Vec<_>>();

        for module in &context.modules {
            let calls = LineBslCallExtractor
                .extract_calls(module.symbols.module_id(), &module.source)
                .map_err(|source| AnalysisError::CallExtraction {
                    module_id: module.symbols.module_id().clone(),
                    source,
                })?;

            let local_resolution = LocalBslCallResolver.resolve(module.symbols.symbols(), &calls);

            for call in local_resolution.resolved() {
                context.graph.insert_edge(GraphEdge::new(
                    call.origin_id().clone(),
                    call.destination_id().clone(),
                    EdgeKind::Calls,
                ))?;
            }

            context.diagnostics.unresolved_local_calls.extend(
                local_resolution
                    .unresolved()
                    .iter()
                    .filter(|call| {
                        call.reason() != oneagent_bsl::UnresolvedCallReason::QualifiedTarget
                    })
                    .cloned()
                    .map(|call| {
                        ModuleLocalDiagnostic::new(module.symbols.module_id().clone(), call)
                    }),
            );

            let cross_module_resolution = QualifiedBslCallResolver.resolve_cross_module_calls(
                &module.symbols,
                &available_modules,
                &calls,
            );

            for call in cross_module_resolution.resolved() {
                context.graph.insert_edge(GraphEdge::new(
                    call.origin_id().clone(),
                    call.destination_id().clone(),
                    EdgeKind::Calls,
                ))?;
            }

            context.diagnostics.unresolved_cross_module_calls.extend(
                cross_module_resolution
                    .unresolved()
                    .iter()
                    .cloned()
                    .map(|call| {
                        ModuleCrossModuleDiagnostic::new(module.symbols.module_id().clone(), call)
                    }),
            );
        }

        Ok(())
    }
}

#[derive(Debug, Default)]
struct BuildContext {
    graph: SemanticGraph,
    modules: Vec<CollectedModule>,
    diagnostics: AnalysisDiagnostics,
}

#[derive(Debug)]
struct CollectedModule {
    source: String,
    symbols: BslModuleSymbols,
}

/// Error produced by the semantic analysis pipeline.
#[derive(Debug)]
pub enum AnalysisError {
    /// Declaration extraction failed.
    DeclarationExtraction {
        /// Module being analyzed.
        module_id: EntityId,
        /// Underlying BSL parsing error.
        source: BslParseError,
    },

    /// Call extraction failed.
    CallExtraction {
        /// Module being analyzed.
        module_id: EntityId,
        /// Underlying BSL call extraction error.
        source: BslCallError,
    },

    /// Semantic graph construction failed.
    Graph(GraphError),
}

impl Display for AnalysisError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DeclarationExtraction { module_id, source } => {
                write!(
                    formatter,
                    "failed to extract declarations from module {module_id}: {source}"
                )
            }
            Self::CallExtraction { module_id, source } => {
                write!(
                    formatter,
                    "failed to extract calls from module {module_id}: {source}"
                )
            }
            Self::Graph(source) => write!(formatter, "failed to build semantic graph: {source}"),
        }
    }
}

impl std::error::Error for AnalysisError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::DeclarationExtraction { source, .. } => Some(source),
            Self::CallExtraction { source, .. } => Some(source),
            Self::Graph(source) => Some(source),
        }
    }
}

impl From<GraphError> for AnalysisError {
    fn from(source: GraphError) -> Self {
        Self::Graph(source)
    }
}

#[cfg(test)]
mod tests {
    use oneagent_common::{EntityId, EntityName};
    use oneagent_graph::{EdgeKind, NodeKind};

    use super::{AnalysisModule, SemanticAnalysisPipeline};

    fn id(value: &str) -> EntityId {
        EntityId::new(value).expect("identifier must be valid")
    }

    fn name(value: &str) -> EntityName {
        EntityName::new(value).expect("name must be valid")
    }

    #[test]
    fn resolves_local_calls_after_collecting_symbols() {
        let module = AnalysisModule::new(
            id("module.sales"),
            name("Sales"),
            r"
Procedure Post()
    FillMovements();
EndProcedure

Procedure FillMovements()
EndProcedure
",
        );

        let result = SemanticAnalysisPipeline
            .analyze(&[module])
            .expect("analysis must succeed");

        let post_id = id("module.sales:procedure:Post");
        let calls = result.graph().outgoing_by_kind(&post_id, EdgeKind::Calls);

        assert_eq!(result.graph().nodes_by_kind(NodeKind::Procedure).len(), 2);
        assert_eq!(calls.len(), 1);
        assert_eq!(
            calls[0].target(),
            &id("module.sales:procedure:FillMovements")
        );
        assert!(result.diagnostics().is_empty());
    }

    #[test]
    fn resolves_cross_module_call_only_after_all_modules_are_collected() {
        let sales = AnalysisModule::new(
            id("module.sales"),
            name("Sales"),
            r"
Procedure Post()
    Inventory.Reserve();
EndProcedure
",
        );

        let inventory = AnalysisModule::new(
            id("module.inventory"),
            name("Inventory"),
            r"
Procedure Reserve() Export
EndProcedure
",
        );

        let result = SemanticAnalysisPipeline
            .analyze(&[sales, inventory])
            .expect("analysis must succeed");

        let post_id = id("module.sales:procedure:Post");
        let calls = result.graph().outgoing_by_kind(&post_id, EdgeKind::Calls);

        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].target(), &id("module.inventory:procedure:Reserve"));
        assert!(result.diagnostics().is_empty());
    }

    #[test]
    fn does_not_resolve_non_exported_cross_module_symbol() {
        let sales = AnalysisModule::new(
            id("module.sales"),
            name("Sales"),
            r"
Procedure Post()
    Inventory.Reserve();
EndProcedure
",
        );

        let inventory = AnalysisModule::new(
            id("module.inventory"),
            name("Inventory"),
            r"
Procedure Reserve()
EndProcedure
",
        );

        let result = SemanticAnalysisPipeline
            .analyze(&[sales, inventory])
            .expect("analysis must succeed");

        let post_id = id("module.sales:procedure:Post");

        assert!(
            result
                .graph()
                .outgoing_by_kind(&post_id, EdgeKind::Calls)
                .is_empty()
        );
        assert_eq!(
            result.diagnostics().unresolved_cross_module_calls().len(),
            1
        );
    }
}
