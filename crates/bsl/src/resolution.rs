//! Local resolution of BSL calls to declarations.

use crate::{BslCall, BslSymbol};
use oneagent_common::{EntityId, EntityName};
use std::collections::BTreeMap;

/// A call resolved between two declarations in the same BSL module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedBslCall {
    origin_id: EntityId,
    destination_id: EntityId,
    line: usize,
}

impl ResolvedBslCall {
    /// Creates a resolved call.
    #[must_use]
    pub const fn new(origin_id: EntityId, destination_id: EntityId, line: usize) -> Self {
        Self {
            origin_id,
            destination_id,
            line,
        }
    }

    /// Returns the calling procedure or function identifier.
    #[must_use]
    pub const fn origin_id(&self) -> &EntityId {
        &self.origin_id
    }

    /// Returns the called procedure or function identifier.
    #[must_use]
    pub const fn destination_id(&self) -> &EntityId {
        &self.destination_id
    }

    /// Returns the one-based source line.
    #[must_use]
    pub const fn line(&self) -> usize {
        self.line
    }
}

/// Reason why a call could not be resolved locally.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnresolvedCallReason {
    /// The call is located outside a procedure or function.
    MissingSourceScope,

    /// The containing procedure or function was not found.
    SourceSymbolNotFound,

    /// The target name is qualified and requires cross-module resolution.
    QualifiedTarget,

    /// No local declaration matches the target name.
    TargetSymbolNotFound,
}

/// A call that could not be resolved inside the current module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnresolvedBslCall {
    source_name: Option<EntityName>,
    target_name: EntityName,
    line: usize,
    reason: UnresolvedCallReason,
}

impl UnresolvedBslCall {
    /// Creates an unresolved call.
    #[must_use]
    pub const fn new(
        source_name: Option<EntityName>,
        target_name: EntityName,
        line: usize,
        reason: UnresolvedCallReason,
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

    /// Returns the unresolved target name.
    #[must_use]
    pub const fn target_name(&self) -> &EntityName {
        &self.target_name
    }

    /// Returns the one-based source line.
    #[must_use]
    pub const fn line(&self) -> usize {
        self.line
    }

    /// Returns the resolution failure reason.
    #[must_use]
    pub const fn reason(&self) -> UnresolvedCallReason {
        self.reason
    }
}

/// Result of resolving calls inside one BSL module.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct BslCallResolution {
    resolved: Vec<ResolvedBslCall>,
    unresolved: Vec<UnresolvedBslCall>,
}

impl BslCallResolution {
    /// Creates a resolution result.
    #[must_use]
    pub const fn new(resolved: Vec<ResolvedBslCall>, unresolved: Vec<UnresolvedBslCall>) -> Self {
        Self {
            resolved,
            unresolved,
        }
    }

    /// Returns locally resolved calls.
    #[must_use]
    pub fn resolved(&self) -> &[ResolvedBslCall] {
        &self.resolved
    }

    /// Returns calls requiring another resolution stage.
    #[must_use]
    pub fn unresolved(&self) -> &[UnresolvedBslCall] {
        &self.unresolved
    }
}

/// Resolves calls against declarations from the same BSL module.
pub trait BslCallResolver {
    /// Resolves local calls.
    fn resolve(&self, symbols: &[BslSymbol], calls: &[BslCall]) -> BslCallResolution;
}

/// Case-insensitive resolver for declarations within one module.
#[derive(Debug, Default, Clone, Copy)]
pub struct LocalBslCallResolver;

impl BslCallResolver for LocalBslCallResolver {
    fn resolve(&self, symbols: &[BslSymbol], calls: &[BslCall]) -> BslCallResolution {
        let symbol_index = symbols
            .iter()
            .map(|symbol| (normalize_name(symbol.name().as_str()), symbol.id().clone()))
            .collect::<BTreeMap<_, _>>();

        let mut resolved = Vec::new();
        let mut unresolved = Vec::new();

        for call in calls {
            let Some(source_name) = call.source_symbol() else {
                unresolved.push(UnresolvedBslCall::new(
                    None,
                    call.target_symbol().clone(),
                    call.line(),
                    UnresolvedCallReason::MissingSourceScope,
                ));
                continue;
            };

            let Some(origin_id) = symbol_index.get(&normalize_name(source_name.as_str())) else {
                unresolved.push(UnresolvedBslCall::new(
                    Some(source_name.clone()),
                    call.target_symbol().clone(),
                    call.line(),
                    UnresolvedCallReason::SourceSymbolNotFound,
                ));
                continue;
            };

            if call.target_symbol().as_str().contains('.') {
                unresolved.push(UnresolvedBslCall::new(
                    Some(source_name.clone()),
                    call.target_symbol().clone(),
                    call.line(),
                    UnresolvedCallReason::QualifiedTarget,
                ));
                continue;
            }

            let Some(destination_id) =
                symbol_index.get(&normalize_name(call.target_symbol().as_str()))
            else {
                unresolved.push(UnresolvedBslCall::new(
                    Some(source_name.clone()),
                    call.target_symbol().clone(),
                    call.line(),
                    UnresolvedCallReason::TargetSymbolNotFound,
                ));
                continue;
            };

            resolved.push(ResolvedBslCall::new(
                origin_id.clone(),
                destination_id.clone(),
                call.line(),
            ));
        }

        BslCallResolution::new(resolved, unresolved)
    }
}

fn normalize_name(value: &str) -> String {
    value.to_lowercase()
}

#[cfg(test)]
mod tests {
    use oneagent_common::{EntityId, EntityName};

    use crate::{
        BslCall, BslCallResolver, BslSymbol, BslSymbolKind, LocalBslCallResolver,
        UnresolvedCallReason,
    };

    fn id(value: &str) -> EntityId {
        EntityId::new(value).expect("identifier must be valid")
    }

    fn name(value: &str) -> EntityName {
        EntityName::new(value).expect("name must be valid")
    }

    fn symbol(identifier: &str, symbol_name: &str, kind: BslSymbolKind) -> BslSymbol {
        BslSymbol::new(id(identifier), name(symbol_name), kind, 1, false)
    }

    #[test]
    fn resolves_local_call_case_insensitively() {
        let symbols = vec![
            symbol("module:procedure:Post", "Post", BslSymbolKind::Procedure),
            symbol(
                "module:procedure:FillMovements",
                "FillMovements",
                BslSymbolKind::Procedure,
            ),
        ];

        let calls = vec![BslCall::new(
            id("module:call:2:1"),
            Some(name("POST")),
            name("fillmovements"),
            2,
        )];

        let result = LocalBslCallResolver.resolve(&symbols, &calls);

        assert_eq!(result.resolved().len(), 1);
        assert!(result.unresolved().is_empty());

        assert_eq!(
            result.resolved()[0].origin_id().as_str(),
            "module:procedure:Post"
        );

        assert_eq!(
            result.resolved()[0].destination_id().as_str(),
            "module:procedure:FillMovements"
        );
    }

    #[test]
    fn leaves_qualified_call_for_cross_module_resolution() {
        let symbols = vec![symbol(
            "module:procedure:Post",
            "Post",
            BslSymbolKind::Procedure,
        )];

        let calls = vec![BslCall::new(
            id("module:call:2:1"),
            Some(name("Post")),
            name("AccessManagement.CheckRights"),
            2,
        )];

        let result = LocalBslCallResolver.resolve(&symbols, &calls);

        assert!(result.resolved().is_empty());
        assert_eq!(result.unresolved().len(), 1);
        assert_eq!(
            result.unresolved()[0].reason(),
            UnresolvedCallReason::QualifiedTarget
        );
    }

    #[test]
    fn reports_missing_local_target() {
        let symbols = vec![symbol(
            "module:procedure:Post",
            "Post",
            BslSymbolKind::Procedure,
        )];

        let calls = vec![BslCall::new(
            id("module:call:2:1"),
            Some(name("Post")),
            name("UnknownProcedure"),
            2,
        )];

        let result = LocalBslCallResolver.resolve(&symbols, &calls);

        assert_eq!(
            result.unresolved()[0].reason(),
            UnresolvedCallReason::TargetSymbolNotFound
        );
    }

    #[test]
    fn reports_call_outside_symbol_scope() {
        let calls = vec![BslCall::new(id("module:call:1:1"), None, name("DoWork"), 1)];

        let result = LocalBslCallResolver.resolve(&[], &calls);

        assert_eq!(
            result.unresolved()[0].reason(),
            UnresolvedCallReason::MissingSourceScope
        );
    }
}
