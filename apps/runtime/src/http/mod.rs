//! Runtime-owned HTTP health and Graph Query service.

mod graph_query;

use std::net::SocketAddr;
use std::sync::Arc;

use axum::extract::State;
use axum::http::{StatusCode, header};
use axum::middleware::map_response;
use axum::response::{IntoResponse, Response};
use axum::routing::{MethodFilter, MethodRouter, on};
use axum::{Json, Router};
use serde::Serialize;
use tokio::net::TcpListener;
use tokio::sync::watch;

use crate::{
    AppState, BoxError, GraphQueryService, RuntimeService, ServiceContext, ServiceStartFuture,
    ServiceTask,
};

#[derive(Debug, Clone)]
struct HttpRouterState {
    app: Arc<AppState>,
    graph_query: Option<GraphQueryService>,
}

/// Runtime service that owns the public HTTP listener and health routes.
#[derive(Debug)]
pub struct HttpService {
    bound_address: watch::Sender<Option<SocketAddr>>,
    graph_query: Option<GraphQueryService>,
}

impl HttpService {
    /// Creates an unbound HTTP service.
    #[must_use]
    pub fn new() -> Self {
        let (bound_address, _receiver) = watch::channel(None);
        Self {
            bound_address,
            graph_query: None,
        }
    }

    /// Creates an unbound HTTP service with the complete Graph Query route set.
    #[must_use]
    pub fn with_graph_query(graph_query: GraphQueryService) -> Self {
        let (bound_address, _receiver) = watch::channel(None);
        Self {
            bound_address,
            graph_query: Some(graph_query),
        }
    }

    /// Subscribes to the actual listener address selected during service startup.
    #[must_use]
    pub fn subscribe_bound_address(&self) -> watch::Receiver<Option<SocketAddr>> {
        self.bound_address.subscribe()
    }
}

impl Default for HttpService {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimeService for HttpService {
    fn start(self: Box<Self>, context: ServiceContext) -> ServiceStartFuture {
        Box::pin(async move {
            let Self {
                bound_address,
                graph_query,
            } = *self;
            let (state, mut cancellation) = context.into_parts();
            let listener = TcpListener::bind(state.configuration().http_bind_address()).await?;
            let actual_address = listener.local_addr()?;
            let router = http_router(HttpRouterState {
                app: state,
                graph_query,
            });
            bound_address.send_replace(Some(actual_address));

            let task: ServiceTask = Box::pin(async move {
                let result = axum::serve(listener, router)
                    .with_graceful_shutdown(async move { cancellation.cancelled().await })
                    .await;
                bound_address.send_replace(None);
                result.map_err(|error| Box::new(error) as BoxError)
            });

            Ok(task)
        })
    }
}

fn http_router(state: HttpRouterState) -> Router {
    let mut router = Router::new()
        .route("/health/live", get_only(liveness_response))
        .route("/health/ready", get_only(readiness_response));
    if state.graph_query.is_some() {
        router = router.merge(graph_query::routes());
    }
    router
        .layer(map_response(normalize_method_response))
        .with_state(state)
}

fn get_only<H, T, S>(handler: H) -> MethodRouter<S>
where
    H: axum::handler::Handler<T, S>,
    T: 'static,
    S: Clone + Send + Sync + 'static,
{
    on(MethodFilter::GET, handler).on(MethodFilter::HEAD, method_not_allowed)
}

async fn method_not_allowed() -> impl IntoResponse {
    (StatusCode::METHOD_NOT_ALLOWED, [(header::ALLOW, "GET")])
}

async fn normalize_method_response(mut response: Response) -> Response {
    if response.status() == StatusCode::METHOD_NOT_ALLOWED {
        response
            .headers_mut()
            .insert(header::ALLOW, header::HeaderValue::from_static("GET"));
    }
    response
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: &'static str,
}

async fn liveness_response() -> Json<HealthResponse> {
    Json(HealthResponse { status: "alive" })
}

async fn readiness_response(State(state): State<HttpRouterState>) -> impl IntoResponse {
    let (status, value) = if state.app.health().snapshot().is_ready() {
        (StatusCode::OK, "ready")
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, "not_ready")
    };

    (status, Json(HealthResponse { status: value }))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::convert::Infallible;
    use std::fs;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    use std::path::{Path, PathBuf};
    use std::time::Duration;

    use tempfile::tempdir;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::{TcpListener, TcpStream};
    use tokio::sync::{oneshot, watch};
    use tokio::time::timeout;

    use crate::{
        App, ConfigurationProvider, GraphQueryService, LifecycleState, RuntimeConfig,
        RuntimeErrorKind, WorkspaceService,
    };

    use super::HttpService;

    #[derive(Debug, Clone, Copy)]
    struct TestConfigurationProvider {
        address: SocketAddr,
    }

    impl ConfigurationProvider for TestConfigurationProvider {
        fn load(&self) -> Result<RuntimeConfig, Box<dyn std::error::Error + Send + Sync>> {
            Ok(RuntimeConfig::new("OneAgent Runtime", "test").with_http_bind_address(self.address))
        }
    }

    #[derive(Debug, Clone)]
    struct QueryConfigurationProvider {
        address: SocketAddr,
        workspace_root: PathBuf,
    }

    impl ConfigurationProvider for QueryConfigurationProvider {
        fn load(&self) -> Result<RuntimeConfig, Box<dyn std::error::Error + Send + Sync>> {
            Ok(RuntimeConfig::new("OneAgent Runtime", "test")
                .with_http_bind_address(self.address)
                .with_workspace_root(self.workspace_root.clone()))
        }
    }

    #[derive(Debug)]
    struct Response {
        status: u16,
        headers: BTreeMap<String, String>,
        body: String,
    }

    async fn wait_for_address(receiver: &mut watch::Receiver<Option<SocketAddr>>) -> SocketAddr {
        loop {
            if let Some(address) = *receiver.borrow() {
                return address;
            }
            timeout(Duration::from_secs(1), receiver.changed())
                .await
                .expect("HTTP address wait must not hang")
                .expect("HTTP service must retain address ownership");
        }
    }

    async fn wait_for_lifecycle(
        lifecycle: &mut watch::Receiver<LifecycleState>,
        expected: LifecycleState,
    ) {
        while *lifecycle.borrow() != expected {
            timeout(Duration::from_secs(1), lifecycle.changed())
                .await
                .expect("lifecycle wait must not hang")
                .expect("application must retain lifecycle ownership");
        }
    }

    async fn request(address: SocketAddr, method: &str, path: &str) -> Response {
        let mut stream = timeout(Duration::from_secs(1), TcpStream::connect(address))
            .await
            .expect("HTTP connect must not hang")
            .expect("HTTP listener must accept loopback connections");
        let request =
            format!("{method} {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n");
        stream
            .write_all(request.as_bytes())
            .await
            .expect("HTTP request must write");

        let mut bytes = Vec::new();
        timeout(Duration::from_secs(1), stream.read_to_end(&mut bytes))
            .await
            .expect("HTTP response must not hang")
            .expect("HTTP response must read");
        parse_response(&bytes)
    }

    fn parse_response(bytes: &[u8]) -> Response {
        let response = std::str::from_utf8(bytes).expect("HTTP response must be UTF-8");
        let (head, body) = response
            .split_once("\r\n\r\n")
            .expect("HTTP response must contain a header terminator");
        let mut lines = head.lines();
        let status = lines
            .next()
            .expect("HTTP response must contain a status line")
            .split_whitespace()
            .nth(1)
            .expect("HTTP status line must contain a code")
            .parse()
            .expect("HTTP status code must be numeric");
        let headers = lines
            .map(|line| {
                let (name, value) = line
                    .split_once(':')
                    .expect("HTTP header must contain a colon");
                (name.to_ascii_lowercase(), value.trim().to_owned())
            })
            .collect();

        Response {
            status,
            headers,
            body: body.to_owned(),
        }
    }

    fn copy_tree(source: &Path, destination: &Path) {
        fs::create_dir_all(destination).expect("fixture destination must be created");
        for entry in fs::read_dir(source).expect("fixture directory must be readable") {
            let entry = entry.expect("fixture entry must be readable");
            let destination = destination.join(entry.file_name());
            if entry
                .file_type()
                .expect("fixture entry type must be readable")
                .is_dir()
            {
                copy_tree(&entry.path(), &destination);
            } else {
                fs::copy(entry.path(), destination).expect("fixture file must be copied");
            }
        }
    }

    #[tokio::test]
    async fn http_service_publishes_address_serves_health_and_releases_listener() {
        let service = HttpService::new();
        let mut address = service.subscribe_bound_address();
        let provider = TestConfigurationProvider {
            address: SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0),
        };
        let app = App::builder()
            .configure(&provider)
            .expect("test configuration must load")
            .register_service("http", service)
            .expect("HTTP service must register")
            .build()
            .expect("application must build");
        let mut lifecycle = app.subscribe_lifecycle();
        let (shutdown_sender, shutdown) = oneshot::channel::<()>();
        let run = tokio::spawn(app.run(shutdown));
        let actual_address = wait_for_address(&mut address).await;
        wait_for_lifecycle(&mut lifecycle, LifecycleState::Running).await;

        let live = request(actual_address, "GET", "/health/live").await;
        assert_eq!(live.status, 200);
        assert_eq!(
            live.headers.get("content-type").map(String::as_str),
            Some("application/json")
        );
        assert_eq!(live.body, "{\"status\":\"alive\"}");

        let ready = request(actual_address, "GET", "/health/ready").await;
        assert_eq!(ready.status, 200);
        assert_eq!(ready.body, "{\"status\":\"ready\"}");

        shutdown_sender
            .send(())
            .expect("shutdown request must be observed");
        timeout(Duration::from_secs(1), run)
            .await
            .expect("HTTP Runtime shutdown must not hang")
            .expect("Runtime task must join")
            .expect("requested shutdown must succeed");
        wait_for_lifecycle(&mut lifecycle, LifecycleState::Stopped).await;
        assert_eq!(*address.borrow(), None);

        let rebound = TcpListener::bind(actual_address)
            .await
            .expect("HTTP listener address must be released");
        drop(rebound);
    }

    #[tokio::test]
    async fn http_service_enforces_get_only_routes_and_default_fallback() {
        let service = HttpService::new();
        let mut address = service.subscribe_bound_address();
        let provider = TestConfigurationProvider {
            address: SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0),
        };
        let app = App::builder()
            .configure(&provider)
            .expect("test configuration must load")
            .register_service("http", service)
            .expect("HTTP service must register")
            .build()
            .expect("application must build");
        let (shutdown_sender, shutdown) = oneshot::channel::<()>();
        let run = tokio::spawn(app.run(shutdown));
        let actual_address = wait_for_address(&mut address).await;

        for method in ["HEAD", "POST"] {
            let response = request(actual_address, method, "/health/live").await;
            assert_eq!(response.status, 405);
            assert_eq!(
                response.headers.get("allow").map(String::as_str),
                Some("GET")
            );
            assert_eq!(response.body, "");
        }
        let unknown = request(actual_address, "GET", "/health/live/").await;
        assert_eq!(unknown.status, 404);
        assert_eq!(unknown.body, "");

        shutdown_sender
            .send(())
            .expect("shutdown request must be observed");
        timeout(Duration::from_secs(1), run)
            .await
            .expect("HTTP Runtime shutdown must not hang")
            .expect("Runtime task must join")
            .expect("requested shutdown must succeed");
    }

    #[tokio::test]
    #[allow(clippy::too_many_lines)]
    async fn query_enabled_http_serves_all_routes_through_the_owned_listener() {
        const CONFIGURATION_ID: &str = "408a41e7-907a-4fb3-8999-83d1e8b6e093";

        let root = tempdir().expect("temporary Workspace root must be created");
        copy_tree(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/workspace_service"),
            root.path(),
        );
        let workspace = WorkspaceService::new();
        let graph_query = GraphQueryService::new(workspace.snapshot_observer());
        let http = HttpService::with_graph_query(graph_query);
        let mut address = http.subscribe_bound_address();
        let provider = QueryConfigurationProvider {
            address: SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0),
            workspace_root: root.path().to_path_buf(),
        };
        let app = App::builder()
            .configure(&provider)
            .expect("query test configuration must load")
            .register_service("http", http)
            .expect("HTTP service must register")
            .register_service("workspace", workspace)
            .expect("Workspace service must register")
            .build()
            .expect("application must build");
        let mut lifecycle = app.subscribe_lifecycle();
        let (shutdown_sender, shutdown) = oneshot::channel::<()>();
        let run = tokio::spawn(app.run(shutdown));
        let actual_address = wait_for_address(&mut address).await;
        wait_for_lifecycle(&mut lifecycle, LifecycleState::Running).await;

        let configurations = request(actual_address, "GET", "/api/v1/configurations").await;
        assert_eq!(configurations.status, 200);
        assert_eq!(
            configurations
                .headers
                .get("content-type")
                .map(String::as_str),
            Some("application/json")
        );
        let configurations: serde_json::Value = serde_json::from_str(&configurations.body)
            .expect("configuration response must be JSON");
        assert_eq!(
            configurations["configurations"]
                .as_array()
                .expect("configurations must be an array")
                .len(),
            2
        );

        let node = request(
            actual_address,
            "GET",
            &format!(
                "/api/v1/graph/node?configuration_id={CONFIGURATION_ID}&node_id={CONFIGURATION_ID}"
            ),
        )
        .await;
        assert_eq!(node.status, 200);
        let node: serde_json::Value =
            serde_json::from_str(&node.body).expect("node response must be JSON");
        assert_eq!(node["node"]["kind"], "metadata");
        assert_eq!(node["node"]["metadata_kind"], "configuration");

        let relations = request(
            actual_address,
            "GET",
            &format!(
                "/api/v1/graph/relations?configuration_id={CONFIGURATION_ID}&node_id={CONFIGURATION_ID}&direction=outgoing&edge_kind=contains&limit=1"
            ),
        )
        .await;
        assert_eq!(relations.status, 200);
        let relations: serde_json::Value =
            serde_json::from_str(&relations.body).expect("relation response must be JSON");
        assert_eq!(relations["direction"], "outgoing");
        assert_eq!(relations["edge_kind"], "contains");
        assert_eq!(
            relations["relations"]
                .as_array()
                .expect("relations must be an array")
                .len(),
            1
        );

        let traversal = request(
            actual_address,
            "GET",
            &format!(
                "/api/v1/graph/traverse?configuration_id={CONFIGURATION_ID}&node_id={CONFIGURATION_ID}&direction=outgoing&max_depth=0&include_start=true"
            ),
        )
        .await;
        assert_eq!(traversal.status, 200);
        let traversal: serde_json::Value =
            serde_json::from_str(&traversal.body).expect("traversal response must be JSON");
        assert_eq!(traversal["nodes"][0]["node"]["id"], CONFIGURATION_ID);
        assert_eq!(traversal["nodes"][0]["depth"], 0);
        assert_eq!(
            traversal["nodes"][0]["via_edge_id"],
            serde_json::Value::Null
        );

        let malformed = request(
            actual_address,
            "GET",
            "/api/v1/graph/node?configuration_id=value&node_id=%FF",
        )
        .await;
        assert_eq!(malformed.status, 400);
        assert_eq!(
            malformed.body,
            "{\"error\":{\"code\":\"invalid_query\",\"message\":\"query parameters are invalid\"}}"
        );

        shutdown_sender
            .send(())
            .expect("shutdown request must be observed");
        timeout(Duration::from_secs(1), run)
            .await
            .expect("query-enabled Runtime shutdown must not hang")
            .expect("Runtime task must join")
            .expect("requested shutdown must succeed");
        assert_eq!(*address.borrow(), None);
    }

    #[tokio::test]
    async fn http_bind_failure_is_a_named_service_start_failure() {
        let occupied = TcpListener::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0))
            .await
            .expect("occupied listener must bind");
        let occupied_address = occupied
            .local_addr()
            .expect("occupied listener must expose its address");
        let provider = TestConfigurationProvider {
            address: occupied_address,
        };
        let app = App::builder()
            .configure(&provider)
            .expect("test configuration must load")
            .register_service("http", HttpService::new())
            .expect("HTTP service must register")
            .build()
            .expect("application must build");
        let lifecycle = app.subscribe_lifecycle();

        let error = app
            .run(std::future::pending::<Result<(), Infallible>>())
            .await
            .expect_err("occupied address must fail HTTP startup");

        assert_eq!(error.kind(), RuntimeErrorKind::ServiceStartFailed);
        assert_eq!(error.service_name(), Some("http"));
        assert_eq!(*lifecycle.borrow(), LifecycleState::Stopped);
    }
}
