use std::collections::BTreeMap;
use std::convert::Infallible;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;

use oneagent_runtime::{
    App, BoxError, ConfigurationProvider, HttpService, LifecycleState, RuntimeConfig,
    RuntimeErrorKind, RuntimeService, ServiceContext, ServiceStartFuture, ServiceTask,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{oneshot, watch};
use tokio::time::timeout;

const TEST_TIMEOUT: Duration = Duration::from_secs(1);

#[derive(Debug, Clone, Copy)]
struct TestConfigurationProvider {
    address: SocketAddr,
}

impl ConfigurationProvider for TestConfigurationProvider {
    fn load(&self) -> Result<RuntimeConfig, BoxError> {
        Ok(RuntimeConfig::new("OneAgent Runtime", "test").with_http_bind_address(self.address))
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Response {
    status: u16,
    headers: BTreeMap<String, String>,
    body: String,
}

fn loopback_port_zero() -> SocketAddr {
    SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0)
}

async fn wait_for_address(receiver: &mut watch::Receiver<Option<SocketAddr>>) -> SocketAddr {
    loop {
        if let Some(address) = *receiver.borrow() {
            return address;
        }
        timeout(TEST_TIMEOUT, receiver.changed())
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
        timeout(TEST_TIMEOUT, lifecycle.changed())
            .await
            .expect("lifecycle wait must not hang")
            .expect("application must retain lifecycle ownership");
    }
}

async fn request(address: SocketAddr, method: &str, path: &str) -> Response {
    let mut stream = timeout(TEST_TIMEOUT, TcpStream::connect(address))
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
    timeout(TEST_TIMEOUT, stream.read_to_end(&mut bytes))
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

fn assert_json(response: &Response, status: u16, body: &str) {
    assert_eq!(response.status, status);
    assert_eq!(
        response.headers.get("content-type").map(String::as_str),
        Some("application/json")
    );
    assert_eq!(response.body, body);
}

fn gated_service(
    start_attempted: oneshot::Sender<()>,
    start_release: oneshot::Receiver<()>,
    cancellation_observed: oneshot::Sender<()>,
    stop_release: oneshot::Receiver<()>,
) -> impl RuntimeService {
    move |context: ServiceContext| -> ServiceStartFuture {
        Box::pin(async move {
            start_attempted
                .send(())
                .expect("start attempt must be observed");
            start_release.await.expect("start release must be sent");
            let mut cancellation = context.cancellation();
            let task: ServiceTask = Box::pin(async move {
                cancellation.cancelled().await;
                cancellation_observed
                    .send(())
                    .expect("cancellation must be observed");
                stop_release.await.expect("stop release must be sent");
                Ok(())
            });
            Ok(task)
        })
    }
}

#[tokio::test]
async fn public_http_health_tracks_initializing_running_and_stopping() {
    let http = HttpService::new();
    let mut address = http.subscribe_bound_address();
    let (start_attempted_sender, start_attempted) = oneshot::channel();
    let (start_release_sender, start_release) = oneshot::channel();
    let (cancellation_sender, cancellation_observed) = oneshot::channel();
    let (stop_release_sender, stop_release) = oneshot::channel();
    let provider = TestConfigurationProvider {
        address: loopback_port_zero(),
    };
    let app = App::builder()
        .configure(&provider)
        .expect("test configuration must load")
        .register_service("http", http)
        .expect("HTTP service must register")
        .register_service(
            "gated",
            gated_service(
                start_attempted_sender,
                start_release,
                cancellation_sender,
                stop_release,
            ),
        )
        .expect("gated service must register")
        .build()
        .expect("application must build");
    let mut lifecycle = app.subscribe_lifecycle();
    let (shutdown_sender, shutdown) = oneshot::channel::<()>();
    let run = tokio::spawn(app.run(shutdown));

    let actual_address = wait_for_address(&mut address).await;
    timeout(TEST_TIMEOUT, start_attempted)
        .await
        .expect("gated startup must not hang")
        .expect("gated startup must be observed");
    assert_eq!(*lifecycle.borrow(), LifecycleState::Initializing);
    assert_json(
        &request(actual_address, "GET", "/health/live").await,
        200,
        "{\"status\":\"alive\"}",
    );
    assert_json(
        &request(actual_address, "GET", "/health/ready").await,
        503,
        "{\"status\":\"not_ready\"}",
    );

    start_release_sender
        .send(())
        .expect("gated startup release must be observed");
    wait_for_lifecycle(&mut lifecycle, LifecycleState::Running).await;
    assert_json(
        &request(actual_address, "GET", "/health/ready").await,
        200,
        "{\"status\":\"ready\"}",
    );

    shutdown_sender
        .send(())
        .expect("shutdown request must be observed");
    timeout(TEST_TIMEOUT, cancellation_observed)
        .await
        .expect("gated cancellation must not hang")
        .expect("gated cancellation must be observed");
    assert_eq!(*lifecycle.borrow(), LifecycleState::Stopping);
    assert_json(
        &request(actual_address, "GET", "/health/live").await,
        200,
        "{\"status\":\"alive\"}",
    );
    assert_json(
        &request(actual_address, "GET", "/health/ready").await,
        503,
        "{\"status\":\"not_ready\"}",
    );

    stop_release_sender
        .send(())
        .expect("gated stop release must be observed");
    timeout(TEST_TIMEOUT, run)
        .await
        .expect("Runtime shutdown must not hang")
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
async fn public_http_health_enforces_exact_wire_matrix() {
    let http = HttpService::new();
    let mut address = http.subscribe_bound_address();
    let provider = TestConfigurationProvider {
        address: loopback_port_zero(),
    };
    let app = App::builder()
        .configure(&provider)
        .expect("test configuration must load")
        .register_service("http", http)
        .expect("HTTP service must register")
        .build()
        .expect("application must build");
    let mut lifecycle = app.subscribe_lifecycle();
    let (shutdown_sender, shutdown) = oneshot::channel::<()>();
    let run = tokio::spawn(app.run(shutdown));
    let actual_address = wait_for_address(&mut address).await;
    wait_for_lifecycle(&mut lifecycle, LifecycleState::Running).await;

    assert_json(
        &request(actual_address, "GET", "/health/live").await,
        200,
        "{\"status\":\"alive\"}",
    );
    assert_json(
        &request(actual_address, "GET", "/health/ready").await,
        200,
        "{\"status\":\"ready\"}",
    );

    for path in ["/health/live", "/health/ready"] {
        for method in ["HEAD", "POST"] {
            let response = request(actual_address, method, path).await;
            assert_eq!(response.status, 405);
            assert_eq!(
                response.headers.get("allow").map(String::as_str),
                Some("GET")
            );
            assert_eq!(response.body, "");
        }
    }
    for path in ["/", "/health", "/health/live/", "/health/ready/"] {
        let response = request(actual_address, "GET", path).await;
        assert_eq!(response.status, 404);
        assert_eq!(response.body, "");
    }

    shutdown_sender
        .send(())
        .expect("shutdown request must be observed");
    timeout(TEST_TIMEOUT, run)
        .await
        .expect("Runtime shutdown must not hang")
        .expect("Runtime task must join")
        .expect("requested shutdown must succeed");
}

#[tokio::test]
async fn public_http_bind_failure_is_named_and_resource_free() {
    let occupied = TcpListener::bind(loopback_port_zero())
        .await
        .expect("occupied listener must bind");
    let occupied_address = occupied
        .local_addr()
        .expect("occupied listener must expose its address");
    let http = HttpService::new();
    let address = http.subscribe_bound_address();
    let provider = TestConfigurationProvider {
        address: occupied_address,
    };
    let app = App::builder()
        .configure(&provider)
        .expect("test configuration must load")
        .register_service("http", http)
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
    assert_eq!(*address.borrow(), None);
}

#[tokio::test]
async fn public_http_health_repeats_fresh_runs_and_releases_each_listener() {
    let mut observed = Vec::new();

    for _ in 0..2 {
        let http = HttpService::new();
        let mut address = http.subscribe_bound_address();
        let provider = TestConfigurationProvider {
            address: loopback_port_zero(),
        };
        let app = App::builder()
            .configure(&provider)
            .expect("test configuration must load")
            .register_service("http", http)
            .expect("HTTP service must register")
            .build()
            .expect("application must build");
        let mut lifecycle = app.subscribe_lifecycle();
        let (shutdown_sender, shutdown) = oneshot::channel::<()>();
        let run = tokio::spawn(app.run(shutdown));
        let actual_address = wait_for_address(&mut address).await;
        wait_for_lifecycle(&mut lifecycle, LifecycleState::Running).await;

        observed.push((
            request(actual_address, "GET", "/health/live").await,
            request(actual_address, "GET", "/health/ready").await,
        ));

        shutdown_sender
            .send(())
            .expect("shutdown request must be observed");
        timeout(TEST_TIMEOUT, run)
            .await
            .expect("Runtime shutdown must not hang")
            .expect("Runtime task must join")
            .expect("requested shutdown must succeed");
        assert_eq!(*address.borrow(), None);
        let rebound = TcpListener::bind(actual_address)
            .await
            .expect("fresh HTTP listener address must be released");
        drop(rebound);
    }

    assert_eq!(observed[0], observed[1]);
}
