use std::convert::Infallible;
use std::future::pending;
use std::io;
use std::time::Duration;

use oneagent_runtime::{
    App, AppBuilder, BoxError, DefaultConfigurationProvider, LifecycleState, RuntimeErrorKind,
    RuntimeService, ServiceContext, ServiceStartFuture, ServiceTask,
};
use tokio::sync::{mpsc, oneshot};
use tokio::time::timeout;

#[derive(Debug, PartialEq, Eq)]
enum Event {
    Started(&'static str),
    StartAttempted(&'static str),
    Stopped(&'static str),
}

fn configured_builder() -> AppBuilder {
    App::builder()
        .configure(&DefaultConfigurationProvider)
        .expect("default configuration must load")
}

fn waiting_service(
    name: &'static str,
    events: mpsc::UnboundedSender<Event>,
) -> impl RuntimeService {
    move |context: ServiceContext| -> ServiceStartFuture {
        Box::pin(async move {
            events
                .send(Event::Started(name))
                .expect("event receiver must remain available");
            let mut cancellation = context.cancellation();
            let task: ServiceTask = Box::pin(async move {
                cancellation.cancelled().await;
                events
                    .send(Event::Stopped(name))
                    .expect("event receiver must remain available");
                Ok(())
            });
            Ok(task)
        })
    }
}

fn failing_start_service(
    name: &'static str,
    events: mpsc::UnboundedSender<Event>,
) -> impl RuntimeService {
    move |_context: ServiceContext| -> ServiceStartFuture {
        Box::pin(async move {
            events
                .send(Event::StartAttempted(name))
                .expect("event receiver must remain available");
            Err(Box::new(io::Error::other("start failed")) as BoxError)
        })
    }
}

fn failing_service(
    name: &'static str,
    events: mpsc::UnboundedSender<Event>,
    trigger: oneshot::Receiver<()>,
) -> impl RuntimeService {
    move |_context: ServiceContext| -> ServiceStartFuture {
        Box::pin(async move {
            events
                .send(Event::Started(name))
                .expect("event receiver must remain available");
            let task: ServiceTask = Box::pin(async move {
                trigger.await.expect("failure trigger must be sent");
                Err(Box::new(io::Error::other("service failed")) as BoxError)
            });
            Ok(task)
        })
    }
}

fn successful_exit_service(trigger: oneshot::Receiver<()>) -> impl RuntimeService {
    move |_context: ServiceContext| -> ServiceStartFuture {
        Box::pin(async move {
            let task: ServiceTask = Box::pin(async move {
                trigger.await.expect("exit trigger must be sent");
                Ok(())
            });
            Ok(task)
        })
    }
}

fn panicking_service(trigger: oneshot::Receiver<()>) -> impl RuntimeService {
    move |_context: ServiceContext| -> ServiceStartFuture {
        Box::pin(async move {
            let task: ServiceTask = Box::pin(async move {
                trigger.await.expect("panic trigger must be sent");
                panic!("public probe task panic");
            });
            Ok(task)
        })
    }
}

async fn next_event(events: &mut mpsc::UnboundedReceiver<Event>) -> Event {
    timeout(Duration::from_secs(1), events.recv())
        .await
        .expect("event wait must not hang")
        .expect("event sender must remain available")
}

async fn assert_events_closed(events: &mut mpsc::UnboundedReceiver<Event>) {
    let event = timeout(Duration::from_secs(1), events.recv())
        .await
        .expect("event channel closure must not hang");
    assert_eq!(event, None, "no service event sender may survive App::run");
}

async fn wait_for_state(
    lifecycle: &mut tokio::sync::watch::Receiver<LifecycleState>,
    expected: LifecycleState,
) {
    while *lifecycle.borrow() != expected {
        timeout(Duration::from_secs(1), lifecycle.changed())
            .await
            .expect("lifecycle wait must not hang")
            .expect("application must retain lifecycle ownership");
    }
}

#[tokio::test]
async fn public_runtime_waits_for_shutdown_and_stops_in_reverse_order() {
    let (sender, mut events) = mpsc::unbounded_channel();
    let app = configured_builder()
        .register_service("first", waiting_service("first", sender.clone()))
        .expect("first service must register")
        .register_service("second", waiting_service("second", sender.clone()))
        .expect("second service must register")
        .build()
        .expect("application must build");
    drop(sender);
    let mut lifecycle = app.subscribe_lifecycle();
    let (shutdown_sender, shutdown) = oneshot::channel();
    let run = tokio::spawn(app.run(shutdown));

    assert_eq!(next_event(&mut events).await, Event::Started("first"));
    assert_eq!(next_event(&mut events).await, Event::Started("second"));
    wait_for_state(&mut lifecycle, LifecycleState::Running).await;
    assert!(
        !run.is_finished(),
        "Runtime must wait for injected shutdown"
    );

    shutdown_sender
        .send(())
        .expect("shutdown request must be observed");
    assert_eq!(next_event(&mut events).await, Event::Stopped("second"));
    assert_eq!(next_event(&mut events).await, Event::Stopped("first"));
    timeout(Duration::from_secs(1), run)
        .await
        .expect("Runtime shutdown must not hang")
        .expect("Runtime task must join")
        .expect("requested shutdown must succeed");
    wait_for_state(&mut lifecycle, LifecycleState::Stopped).await;
    assert_events_closed(&mut events).await;
}

#[tokio::test]
async fn public_runtime_rolls_back_partial_startup() {
    let (sender, mut events) = mpsc::unbounded_channel();
    let app = configured_builder()
        .register_service("first", waiting_service("first", sender.clone()))
        .expect("first service must register")
        .register_service("second", failing_start_service("second", sender.clone()))
        .expect("second service must register")
        .build()
        .expect("application must build");
    drop(sender);
    let lifecycle = app.subscribe_lifecycle();

    let error = app
        .run(pending::<Result<(), Infallible>>())
        .await
        .expect_err("startup failure must reach the public caller");

    assert_eq!(error.kind(), RuntimeErrorKind::ServiceStartFailed);
    assert_eq!(error.service_name(), Some("second"));
    assert_eq!(*lifecycle.borrow(), LifecycleState::Stopped);
    assert_eq!(next_event(&mut events).await, Event::Started("first"));
    assert_eq!(
        next_event(&mut events).await,
        Event::StartAttempted("second")
    );
    assert_eq!(next_event(&mut events).await, Event::Stopped("first"));
    assert_events_closed(&mut events).await;
}

#[tokio::test]
async fn public_runtime_propagates_service_failure_and_cleans_siblings() {
    let (sender, mut events) = mpsc::unbounded_channel();
    let (failure_sender, failure) = oneshot::channel();
    let app = configured_builder()
        .register_service("first", waiting_service("first", sender.clone()))
        .expect("first service must register")
        .register_service(
            "failing",
            failing_service("failing", sender.clone(), failure),
        )
        .expect("failing service must register")
        .register_service("last", waiting_service("last", sender.clone()))
        .expect("last service must register")
        .build()
        .expect("application must build");
    drop(sender);
    let lifecycle = app.subscribe_lifecycle();
    let run = tokio::spawn(app.run(pending::<Result<(), Infallible>>()));

    assert_eq!(next_event(&mut events).await, Event::Started("first"));
    assert_eq!(next_event(&mut events).await, Event::Started("failing"));
    assert_eq!(next_event(&mut events).await, Event::Started("last"));
    failure_sender
        .send(())
        .expect("failure trigger must be observed");

    let error = timeout(Duration::from_secs(1), run)
        .await
        .expect("failed Runtime must not hang")
        .expect("Runtime task must join")
        .expect_err("service failure must reach the public caller");
    assert_eq!(error.kind(), RuntimeErrorKind::ServiceFailed);
    assert_eq!(error.service_name(), Some("failing"));
    assert_eq!(*lifecycle.borrow(), LifecycleState::Stopped);
    assert_eq!(next_event(&mut events).await, Event::Stopped("last"));
    assert_eq!(next_event(&mut events).await, Event::Stopped("first"));
    assert_events_closed(&mut events).await;
}

#[tokio::test]
async fn public_runtime_distinguishes_unexpected_exit_and_task_panic() {
    let (exit_sender, exit) = oneshot::channel();
    let app = configured_builder()
        .register_service("early", successful_exit_service(exit))
        .expect("early service must register")
        .build()
        .expect("application must build");
    let run = tokio::spawn(app.run(pending::<Result<(), Infallible>>()));
    exit_sender.send(()).expect("exit trigger must be observed");
    let error = timeout(Duration::from_secs(1), run)
        .await
        .expect("early-exit Runtime must not hang")
        .expect("Runtime task must join")
        .expect_err("unexpected success must fail the Runtime");
    assert_eq!(error.kind(), RuntimeErrorKind::UnexpectedServiceExit);
    assert_eq!(error.service_name(), Some("early"));

    let (panic_sender, panic_trigger) = oneshot::channel();
    let app = configured_builder()
        .register_service("panic", panicking_service(panic_trigger))
        .expect("panic service must register")
        .build()
        .expect("application must build");
    let run = tokio::spawn(app.run(pending::<Result<(), Infallible>>()));
    panic_sender
        .send(())
        .expect("panic trigger must be observed");
    let error = timeout(Duration::from_secs(1), run)
        .await
        .expect("panicked Runtime must not hang")
        .expect("Runtime task must join")
        .expect_err("task panic must fail the Runtime");
    assert_eq!(error.kind(), RuntimeErrorKind::ServiceTaskJoinFailed);
    assert_eq!(error.service_name(), Some("panic"));
}

#[tokio::test]
async fn public_runtime_distinguishes_shutdown_source_failure() {
    let (sender, mut events) = mpsc::unbounded_channel();
    let app = configured_builder()
        .register_service("worker", waiting_service("worker", sender.clone()))
        .expect("worker must register")
        .build()
        .expect("application must build");
    drop(sender);

    let error = app
        .run(async { Err::<(), _>(io::Error::other("shutdown source failed")) })
        .await
        .expect_err("shutdown source failure must reach the public caller");

    assert_eq!(error.kind(), RuntimeErrorKind::ShutdownSourceFailed);
    assert_eq!(error.service_name(), None);
    assert_eq!(next_event(&mut events).await, Event::Started("worker"));
    assert_eq!(next_event(&mut events).await, Event::Stopped("worker"));
    assert_events_closed(&mut events).await;
}

#[tokio::test]
async fn public_runtime_repeats_fresh_build_and_run_without_shared_state() {
    for name in ["first-run", "second-run"] {
        let (sender, mut events) = mpsc::unbounded_channel();
        let app = configured_builder()
            .register_service(name, waiting_service(name, sender.clone()))
            .expect("fresh service must register")
            .build()
            .expect("fresh application must build");
        drop(sender);

        app.run(async { Ok::<(), Infallible>(()) })
            .await
            .expect("fresh application shutdown must succeed");

        assert_eq!(next_event(&mut events).await, Event::Started(name));
        assert_eq!(next_event(&mut events).await, Event::Stopped(name));
        assert_events_closed(&mut events).await;
    }
}
