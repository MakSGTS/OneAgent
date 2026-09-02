//! Cross-module resolution of qualified BSL calls.

use crate::{BslCall, BslSymbol, bsl_name_key};
use oneagent_common::{EntityId, EntityName};
use std::collections::BTreeMap;

/// Symbols belonging to one BSL module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BslModuleSymbols {
    module_id: EntityId,
    module_name: EntityName,
    symbols: Vec<BslSymbol>,
}

impl BslModuleSymbols {
    /// Creates a module symbol collection.
    #[must_use]
    pub const fn new(
        module_id: EntityId,
        module_name: EntityName,
        symbols: Vec<BslSymbol>,
    ) -> Self {
        Self {
            module_id,
            module_name,
            symbols,
        }
    }

    /// Returns the module identifier.
    #[must_use]
    pub const fn module_id(&self) -> &EntityId {
        &self.module_id
    }

    /// Returns the module name.
    #[must_use]
    pub const fn module_name(&self) -> &EntityName {
        &self.module_name
    }

    /// Returns declarations from the module.
    #[must_use]
    pub fn symbols(&self) -> &[BslSymbol] {
        &self.symbols
    }
}

/// A qualified call resolved between symbols from different modules.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedCrossModuleCall {
    origin_id: EntityId,
    destination_id: EntityId,
    line: usize,
}

impl ResolvedCrossModuleCall {
    /// Creates a resolved cross-module call.
    #[must_use]
    pub const fn new(origin_id: EntityId, destination_id: EntityId, line: usize) -> Self {
        Self {
            origin_id,
            destination_id,
            line,
        }
    }

    /// Returns the calling symbol identifier.
    #[must_use]
    pub const fn origin_id(&self) -> &EntityId {
        &self.origin_id
    }

    /// Returns the called symbol identifier.
    #[must_use]
    pub const fn destination_id(&self) -> &EntityId {
        &self.destination_id
    }

    /// Returns the source line.
    #[must_use]
    pub const fn line(&self) -> usize {
        self.line
    }
}

/// Reason why a qualified call could not be resolved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnresolvedCrossModuleCallReason {
    /// The call does not have a containing procedure or function.
    MissingSourceScope,

    /// The containing declaration was not found in the current module.
    SourceSymbolNotFound,

    /// The target is not a supported `Module.Symbol` expression.
    InvalidQualifiedTarget,

    /// The requested target module was not found.
    TargetModuleNotFound,

    /// The requested declaration was not found in the target module.
    TargetSymbolNotFound,

    /// The target declaration exists but is not exported.
    TargetSymbolNotExported,
}

/// A qualified call that could not be resolved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnresolvedCrossModuleCall {
    source_name: Option<EntityName>,
    target_name: EntityName,
    line: usize,
    reason: UnresolvedCrossModuleCallReason,
}

impl UnresolvedCrossModuleCall {
    /// Creates an unresolved cross-module call.
    #[must_use]
    pub const fn new(
        source_name: Option<EntityName>,
        target_name: EntityName,
        line: usize,
        reason: UnresolvedCrossModuleCallReason,
    ) -> Self {
        Self {
            source_name,
            target_name,
            line,
            reason,
        }
    }

    /// Returns the containing declaration name when available.
    #[must_use]
    pub fn source_name(&self) -> Option<&EntityName> {
        self.source_name.as_ref()
    }

    /// Returns the unresolved qualified target.
    #[must_use]
    pub const fn target_name(&self) -> &EntityName {
        &self.target_name
    }

    /// Returns the source line.
    #[must_use]
    pub const fn line(&self) -> usize {
        self.line
    }

    /// Returns the failure reason.
    #[must_use]
    pub const fn reason(&self) -> UnresolvedCrossModuleCallReason {
        self.reason
    }
}

/// Result of resolving qualified calls.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct CrossModuleCallResolution {
    resolved: Vec<ResolvedCrossModuleCall>,
    unresolved: Vec<UnresolvedCrossModuleCall>,
}

impl CrossModuleCallResolution {
    /// Creates a cross-module resolution result.
    #[must_use]
    pub const fn new(
        resolved: Vec<ResolvedCrossModuleCall>,
        unresolved: Vec<UnresolvedCrossModuleCall>,
    ) -> Self {
        Self {
            resolved,
            unresolved,
        }
    }

    /// Returns resolved calls.
    #[must_use]
    pub fn resolved(&self) -> &[ResolvedCrossModuleCall] {
        &self.resolved
    }

    /// Returns unresolved calls.
    #[must_use]
    pub fn unresolved(&self) -> &[UnresolvedCrossModuleCall] {
        &self.unresolved
    }
}

/// Resolves qualified calls between BSL modules.
pub trait CrossModuleCallResolver {
    /// Resolves calls originating from `current_module`.
    fn resolve_cross_module_calls(
        &self,
        current_module: &BslModuleSymbols,
        available_modules: &[BslModuleSymbols],
        calls: &[BslCall],
    ) -> CrossModuleCallResolution;
}

/// Case-insensitive resolver for `Module.Symbol()` calls.
#[derive(Debug, Default, Clone, Copy)]
pub struct QualifiedBslCallResolver;

impl CrossModuleCallResolver for QualifiedBslCallResolver {
    fn resolve_cross_module_calls(
        &self,
        current_module: &BslModuleSymbols,
        available_modules: &[BslModuleSymbols],
        calls: &[BslCall],
    ) -> CrossModuleCallResolution {
        let source_index = build_symbol_index(current_module.symbols());

        let module_index = available_modules
            .iter()
            .map(|module| (bsl_name_key(module.module_name().as_str()), module))
            .collect::<BTreeMap<_, _>>();

        let mut resolved = Vec::new();
        let mut unresolved = Vec::new();

        for call in calls {
            if !call.target_symbol().as_str().contains('.') {
                continue;
            }

            let Some(source_name) = call.source_symbol() else {
                unresolved.push(UnresolvedCrossModuleCall::new(
                    None,
                    call.target_symbol().clone(),
                    call.line(),
                    UnresolvedCrossModuleCallReason::MissingSourceScope,
                ));
                continue;
            };

            let Some(origin_symbol) = source_index.get(&bsl_name_key(source_name.as_str())) else {
                unresolved.push(UnresolvedCrossModuleCall::new(
                    Some(source_name.clone()),
                    call.target_symbol().clone(),
                    call.line(),
                    UnresolvedCrossModuleCallReason::SourceSymbolNotFound,
                ));
                continue;
            };

            let Some((module_name, symbol_name)) =
                split_qualified_target(call.target_symbol().as_str())
            else {
                unresolved.push(UnresolvedCrossModuleCall::new(
                    Some(source_name.clone()),
                    call.target_symbol().clone(),
                    call.line(),
                    UnresolvedCrossModuleCallReason::InvalidQualifiedTarget,
                ));
                continue;
            };

            let Some(target_module) = module_index.get(&bsl_name_key(module_name)) else {
                unresolved.push(UnresolvedCrossModuleCall::new(
                    Some(source_name.clone()),
                    call.target_symbol().clone(),
                    call.line(),
                    UnresolvedCrossModuleCallReason::TargetModuleNotFound,
                ));
                continue;
            };

            let destination_index = build_symbol_index(target_module.symbols());

            let Some(destination_symbol) = destination_index.get(&bsl_name_key(symbol_name)) else {
                unresolved.push(UnresolvedCrossModuleCall::new(
                    Some(source_name.clone()),
                    call.target_symbol().clone(),
                    call.line(),
                    UnresolvedCrossModuleCallReason::TargetSymbolNotFound,
                ));
                continue;
            };

            if !destination_symbol.is_exported() {
                unresolved.push(UnresolvedCrossModuleCall::new(
                    Some(source_name.clone()),
                    call.target_symbol().clone(),
                    call.line(),
                    UnresolvedCrossModuleCallReason::TargetSymbolNotExported,
                ));
                continue;
            }

            resolved.push(ResolvedCrossModuleCall::new(
                origin_symbol.id().clone(),
                destination_symbol.id().clone(),
                call.line(),
            ));
        }

        CrossModuleCallResolution::new(resolved, unresolved)
    }
}

fn build_symbol_index(symbols: &[BslSymbol]) -> BTreeMap<String, &BslSymbol> {
    symbols
        .iter()
        .map(|symbol| (bsl_name_key(symbol.name().as_str()), symbol))
        .collect()
}

fn split_qualified_target(value: &str) -> Option<(&str, &str)> {
    let (module_name, symbol_name) = value.split_once('.')?;

    if module_name.is_empty() || symbol_name.is_empty() || symbol_name.contains('.') {
        return None;
    }

    Some((module_name, symbol_name))
}

#[cfg(test)]
mod tests {
    use oneagent_common::{EntityId, EntityName};

    use crate::{
        BslCall, BslModuleSymbols, BslSymbol, BslSymbolKind, CrossModuleCallResolver,
        QualifiedBslCallResolver, UnresolvedCrossModuleCallReason,
    };

    fn id(value: &str) -> EntityId {
        EntityId::new(value).expect("identifier must be valid")
    }

    fn name(value: &str) -> EntityName {
        EntityName::new(value).expect("name must be valid")
    }

    fn symbol(
        identifier: &str,
        symbol_name: &str,
        kind: BslSymbolKind,
        exported: bool,
    ) -> BslSymbol {
        BslSymbol::new(id(identifier), name(symbol_name), kind, 1, exported)
    }

    #[test]
    fn resolves_exported_qualified_call() {
        let source_module = BslModuleSymbols::new(
            id("document-module"),
            name("SalesObjectModule"),
            vec![symbol(
                "document-module:procedure:Post",
                "Post",
                BslSymbolKind::Procedure,
                false,
            )],
        );

        let target_module = BslModuleSymbols::new(
            id("access-module"),
            name("AccessManagement"),
            vec![symbol(
                "access-module:procedure:CheckRights",
                "CheckRights",
                BslSymbolKind::Procedure,
                true,
            )],
        );

        let calls = vec![BslCall::new(
            id("document-module:call:2:1"),
            Some(name("Post")),
            name("AccessManagement.CheckRights"),
            2,
        )];

        let result = QualifiedBslCallResolver.resolve_cross_module_calls(
            &source_module,
            &[target_module],
            &calls,
        );

        assert_eq!(result.resolved().len(), 1);
        assert!(result.unresolved().is_empty());

        assert_eq!(
            result.resolved()[0].origin_id().as_str(),
            "document-module:procedure:Post"
        );

        assert_eq!(
            result.resolved()[0].destination_id().as_str(),
            "access-module:procedure:CheckRights"
        );
    }

    #[test]
    fn rejects_non_exported_target() {
        let source_module = BslModuleSymbols::new(
            id("document-module"),
            name("SalesObjectModule"),
            vec![symbol(
                "document-module:procedure:Post",
                "Post",
                BslSymbolKind::Procedure,
                false,
            )],
        );

        let target_module = BslModuleSymbols::new(
            id("access-module"),
            name("AccessManagement"),
            vec![symbol(
                "access-module:procedure:InternalCheck",
                "InternalCheck",
                BslSymbolKind::Procedure,
                false,
            )],
        );

        let calls = vec![BslCall::new(
            id("document-module:call:2:1"),
            Some(name("Post")),
            name("AccessManagement.InternalCheck"),
            2,
        )];

        let result = QualifiedBslCallResolver.resolve_cross_module_calls(
            &source_module,
            &[target_module],
            &calls,
        );

        assert_eq!(
            result.unresolved()[0].reason(),
            UnresolvedCrossModuleCallReason::TargetSymbolNotExported
        );
    }

    #[test]
    fn reports_missing_target_module() {
        let source_module = BslModuleSymbols::new(
            id("document-module"),
            name("SalesObjectModule"),
            vec![symbol(
                "document-module:procedure:Post",
                "Post",
                BslSymbolKind::Procedure,
                false,
            )],
        );

        let calls = vec![BslCall::new(
            id("document-module:call:2:1"),
            Some(name("Post")),
            name("UnknownModule.DoWork"),
            2,
        )];

        let result =
            QualifiedBslCallResolver.resolve_cross_module_calls(&source_module, &[], &calls);

        assert_eq!(
            result.unresolved()[0].reason(),
            UnresolvedCrossModuleCallReason::TargetModuleNotFound
        );
    }
}
