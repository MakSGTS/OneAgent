//! Reusable `OneAgent Runtime` composition and service boundaries.

mod app;
mod config;
mod error;
mod health;
mod http;
mod mcp;
mod mcp_tools;
mod service;
mod state;
mod workspace;

pub use app::{App, AppBuilder, Lifecycle, LifecycleState};
pub use config::{ConfigurationProvider, DefaultConfigurationProvider, RuntimeConfig};
pub use error::{BoxError, CleanupFailure, RuntimeError, RuntimeErrorKind};
pub use health::{RuntimeHealth, RuntimeHealthSnapshot};
pub use http::HttpService;
pub use mcp::{McpStdioError, McpStdioErrorKind, McpStdioOutcome, McpStdioTransport};
pub use mcp_tools::{McpSemanticServerError, graph_semantic_server};
pub use service::{
    Cancellation, RunningServices, RuntimeService, ServiceContainer, ServiceContainerBuilder,
    ServiceContext, ServiceStartFuture, ServiceTask,
};
pub use state::AppState;
pub use workspace::{
    GraphQueryConfiguration, GraphQueryConfigurationList, GraphQueryDirection, GraphQueryEdgeKind,
    GraphQueryError, GraphQueryErrorKind, GraphQueryLimit, GraphQueryMaxDepth,
    GraphQueryMetadataKind, GraphQueryNode, GraphQueryNodeKind, GraphQueryNodeResult,
    GraphQueryRelation, GraphQueryRelationResult, GraphQueryService, GraphQueryTraversalNode,
    GraphQueryTraversalResult, GraphQueryWorkspaceFormat, WorkspaceBuildError,
    WorkspaceBuildErrorKind, WorkspaceCacheLoadOutcome, WorkspaceCacheObserver,
    WorkspaceCacheStatus, WorkspaceCacheWriteOutcome, WorkspaceConfigurationSnapshot,
    WorkspaceService, WorkspaceSnapshot, WorkspaceSnapshotBuilder, WorkspaceSnapshotObserver,
    WorkspaceUpdateFailureKind, WorkspaceUpdateObserver, WorkspaceUpdatePhase,
    WorkspaceUpdateStatus,
};
