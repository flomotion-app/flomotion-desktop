mod support;

use flomotion_desktop_lib::app::dispatcher::Dispatcher;
use flomotion_desktop_lib::ipc::protocol::{Request, RequestHandler};
use serde_json::{json, Value};
use support::{config, FakePage, FakeWindow};

#[test]
fn status_is_answered_without_the_page() {
    let dispatcher = Dispatcher::new(config(), FakePage::new(false), FakeWindow::new());
    let response = dispatcher.handle(Request::status());
    assert!(response.ok);
    assert_eq!(response.result["page_ready"], Value::Bool(false));
    assert_eq!(response.result["version"], "0.0.1");
}

#[test]
fn other_commands_go_to_the_page() {
    let page = FakePage::new(true);
    let dispatcher = Dispatcher::new(config(), page.clone(), FakeWindow::new());
    let response = dispatcher.handle(Request::new("agent", json!({"reset": false})));
    assert!(response.ok);
    assert_eq!(page.calls.lock().unwrap()[0].command, "agent");
}

#[test]
fn page_errors_become_failure_responses() {
    let dispatcher = Dispatcher::new(config(), FakePage::new(false), FakeWindow::new());
    let response = dispatcher.handle(Request::new("agent", Value::Null));
    assert!(!response.ok);
    assert!(response.error.unwrap().contains("not connected"));
}

#[test]
fn focus_is_handled_by_the_window_without_the_page() {
    let window = FakeWindow::new();
    let dispatcher = Dispatcher::new(config(), FakePage::new(false), window.clone());
    let response = dispatcher.handle(Request::focus());
    assert!(response.ok);
    assert_eq!(window.focused.load(std::sync::atomic::Ordering::SeqCst), 1);
}
