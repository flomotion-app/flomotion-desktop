mod support;

use flomotion_desktop_lib::cli::files::FileMaterializer;
use flomotion_desktop_lib::cli::runner::CliRunner;
use flomotion_desktop_lib::cli::Command;
use flomotion_desktop_lib::error::{DesktopError, Result};
use flomotion_desktop_lib::ipc::protocol::Response;
use serde_json::{json, Value};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use support::{act, output, ready, FakeLauncher, FakeStore, FakeTransport, FakeUploads, TransportLog};

fn runner(responses: Vec<Result<Response>>) -> (CliRunner, Arc<TransportLog>, Arc<AtomicUsize>) {
    let (transport, log) = FakeTransport::scripted(responses);
    let (launcher, launches) = FakeLauncher::new();
    let (store, _) = FakeStore::new();
    build(transport, launcher, store, log, launches)
}

fn build(transport: FakeTransport, launcher: FakeLauncher, store: FakeStore, log: Arc<TransportLog>, launches: Arc<AtomicUsize>) -> (CliRunner, Arc<TransportLog>, Arc<AtomicUsize>) {
    let runner = CliRunner::new(
        Box::new(transport),
        Box::new(launcher),
        FileMaterializer::new(Box::new(store)),
        Box::new(FakeUploads::new().0),
        Duration::from_secs(2),
    );
    (runner, log, launches)
}

#[test]
fn status_reports_not_running_when_socket_is_closed() {
    let (runner, _, launches) = runner(vec![Err(DesktopError::AppNotRunning)]);
    let mut out = Vec::new();
    runner.run(Command::Status, &mut out).unwrap();
    assert_eq!(output(out)["running"], Value::Bool(false));
    assert_eq!(launches.load(Ordering::SeqCst), 0);
}

#[test]
fn act_launches_the_app_when_it_is_not_running() {
    let (runner, log, launches) = runner(vec![
        Err(DesktopError::AppNotRunning),
        ready(),
        Ok(Response::success(json!({"result": {"projects": []}}))),
    ]);
    let mut out = Vec::new();
    runner.run(act("list_projects", None), &mut out).unwrap();
    assert_eq!(launches.load(Ordering::SeqCst), 1);
    let sent = log.sent.lock().unwrap();
    let request = sent.iter().find(|request| request.command == "act").unwrap();
    assert_eq!(request.payload["name"], "list_projects");
    assert_eq!(request.payload["input"], json!({}));
}

#[test]
fn act_waits_until_the_page_is_ready() {
    let (runner, log, launches) = runner(vec![
        Ok(Response::success(json!({"page_ready": false}))),
        Ok(Response::success(json!({"page_ready": false}))),
        ready(),
        Ok(Response::success(json!({"result": {}}))),
    ]);
    let mut out = Vec::new();
    runner.run(act("list_projects", None), &mut out).unwrap();
    assert_eq!(launches.load(Ordering::SeqCst), 0);
    assert_eq!(log.sent.lock().unwrap().iter().filter(|request| request.command == "status").count(), 3);
}

#[test]
fn act_passes_the_wait_limit_to_the_page() {
    let (runner, log, _) = runner(vec![ready(), Ok(Response::success(json!({"result": {"job_id": "j1"}})))]);
    let mut out = Vec::new();
    let command = Command::Act { name: "validate".into(), input: None, input_file: None, wait: Some(30), file: None };
    runner.run(command, &mut out).unwrap();
    assert_eq!(log.sent.lock().unwrap()[1].payload["wait"], 30);
    assert_eq!(output(out)["result"]["job_id"], "j1");
}

#[test]
fn login_forwards_the_code_only_when_given() {
    let (runner, log, _) = runner(vec![ready(), Ok(Response::success(json!({"url": "https://x/device?f=a"}))), ready(), Ok(Response::success(json!({"signed_in": true})))]);
    runner.run(Command::Login { code: None }, &mut Vec::new()).unwrap();
    runner.run(Command::Login { code: Some("u.r".into()) }, &mut Vec::new()).unwrap();
    let sent = log.sent.lock().unwrap();
    let logins: Vec<_> = sent.iter().filter(|request| request.command == "login").collect();
    assert_eq!(logins[0].payload, json!({}));
    assert_eq!(logins[1].payload, json!({ "code": "u.r" }));
}

#[test]
fn act_without_wait_leaves_the_default_to_the_page() {
    let (runner, log, _) = runner(vec![ready(), Ok(Response::success(json!({"result": {"job_id": "j1"}})))]);
    let mut out = Vec::new();
    let command = Command::Act { name: "validate".into(), input: None, input_file: None, wait: None, file: None };
    runner.run(command, &mut out).unwrap();
    assert!(log.sent.lock().unwrap()[1].payload.get("wait").is_none());
}

#[test]
fn job_command_is_forwarded_with_its_wait_limit() {
    let (runner, log, _) = runner(vec![ready(), Ok(Response::success(json!({"status": "done", "result": {"summary": "ok"}})))]);
    let mut out = Vec::new();
    runner.run(Command::Job { id: "j3".into(), wait: Some(5) }, &mut out).unwrap();
    let sent = log.sent.lock().unwrap();
    assert_eq!(sent[1].command, "job");
    assert_eq!(sent[1].payload["job_id"], "j3");
    assert_eq!(sent[1].payload["wait"], 5);
    assert_eq!(output(out)["result"]["summary"], "ok");
}

#[test]
fn agent_reset_is_forwarded() {
    let (runner, log, _) = runner(vec![ready(), Ok(Response::success(json!({"role": "bootstrap"})))]);
    let mut out = Vec::new();
    runner.run(Command::Agent { reset: true }, &mut out).unwrap();
    assert_eq!(log.sent.lock().unwrap()[1].payload["reset"], Value::Bool(true));
    assert_eq!(output(out)["role"], "bootstrap");
}

#[test]
fn page_failures_become_errors() {
    let (runner, _, _) = runner(vec![ready(), Ok(Response::failure("nope"))]);
    let mut out = Vec::new();
    let error = runner.run(Command::Agent { reset: false }, &mut out).unwrap_err();
    assert_eq!(error.to_string(), "nope");
}

#[test]
fn non_object_input_is_rejected() {
    let (runner, _, _) = runner(vec![ready()]);
    let mut out = Vec::new();
    let error = runner.run(act("x", Some("[1]")), &mut out).unwrap_err();
    assert!(error.to_string().contains("JSON object"));
}

#[test]
fn import_sends_the_file_as_base64() {
    let dir = std::env::temp_dir().join("flomotion-test-import");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("part.step");
    std::fs::write(&path, b"ISO-10303").unwrap();
    let (runner, log, _) = runner(vec![ready(), Ok(Response::success(json!({"file_id": "f1"})))]);
    let mut out = Vec::new();
    runner.run(Command::Import { path, transport: None }, &mut out).unwrap();
    let sent = log.sent.lock().unwrap();
    assert_eq!(sent[1].command, "import");
    assert_eq!(sent[1].payload["name"], "part.step");
    assert_eq!(sent[1].payload["data_base64"], "SVNPLTEwMzAz");
    assert_eq!(sent[1].payload["size"], 9);
    assert_eq!(output(out)["file_id"], "f1");
}

#[test]
fn export_writes_files_into_the_requested_directory() {
    let (transport, log) = FakeTransport::scripted(vec![
        ready(),
        Ok(Response::success(json!({"files": [{"name": "arm.step", "data_base64": "SVNP"}]}))),
    ]);
    let (launcher, launches) = FakeLauncher::new();
    let (store, written) = FakeStore::new();
    let (runner, log, _) = build(transport, launcher, store, log, launches);
    let mut out = Vec::new();
    let command = Command::Export { kind: "step".into(), id: Some("arm".into()), out: Some(PathBuf::from("/exports")) };
    runner.run(command, &mut out).unwrap();
    let sent = log.sent.lock().unwrap();
    assert_eq!(sent[1].payload["kind"], "step");
    assert_eq!(sent[1].payload["id"], "arm");
    assert_eq!(written.lock().unwrap()[0].1, b"ISO".to_vec());
    assert_eq!(output(out)["files"][0].as_str().unwrap(), PathBuf::from("/exports").join("arm.step").to_string_lossy());
}

#[test]
fn skill_prints_agent_instructions() {
    let (runner, log, _) = runner(vec![]);
    let mut out = Vec::new();
    runner.run(Command::Skill, &mut out).unwrap();
    assert!(String::from_utf8(out).unwrap().starts_with("# Driving FloMotion"));
    assert!(log.sent.lock().unwrap().is_empty());
}
