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
use support::{ready, FakeLauncher, FakeStore, FakeTransport, FakeUploads, TransportLog};

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

fn act(name: &str, input: Option<&str>) -> Command {
    Command::Act { name: name.into(), input: input.map(String::from), input_file: None, wait: Some(1), file: None }
}

fn output(out: Vec<u8>) -> Value {
    serde_json::from_slice(&out).unwrap()
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
    let request = sent.iter().find(|r| r.command == "act").unwrap();
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
    assert_eq!(log.sent.lock().unwrap().iter().filter(|r| r.command == "status").count(), 3);
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

fn act_with_file(name: &str, input: &str, path: &std::path::Path) -> Command {
    Command::Act { name: name.into(), input: Some(input.into()), input_file: None, wait: Some(1), file: Some(path.to_path_buf()) }
}

const COVER_INPUT: &str = r#"{"listing_id":"L1","asset":"cover"}"#;

fn image(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join("flomotion-attach-tests");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(name);
    std::fs::write(&path, b"not really a png").unwrap();
    path
}

fn act_response(result: Value) -> Value {
    json!({ "result": result, "is_error": false, "role": "manager", "balance": 1.0, "changed": false, "files": [] })
}

fn envelope(url: &str) -> Value {
    act_response(json!({ "upload": {
        "url": url,
        "headers": { "Content-Type": "image/png", "x-goog-content-length-range": "0,10485760" },
        "confirm_with": { "name": "attach_robot_media", "input": { "listing_id": "L1", "asset": "cover", "confirm": "https://cdn/x.png" } }
    }}))
}

fn attach_runner(uploads: FakeUploads, responses: Vec<Result<Response>>) -> (CliRunner, Arc<TransportLog>) {
    let (transport, log) = FakeTransport::scripted(responses);
    let (launcher, _) = FakeLauncher::new();
    let (store, _) = FakeStore::new();
    let runner = CliRunner::new(
        Box::new(transport),
        Box::new(launcher),
        FileMaterializer::new(Box::new(store)),
        Box::new(uploads),
        Duration::from_secs(2),
    );
    (runner, log)
}

#[test]
fn attach_uploads_the_bytes_and_makes_the_confirm_call() {
    let path = image("banner.png");
    let (uploads, log_of_puts) = FakeUploads::new();
    let (runner, calls) = attach_runner(uploads, vec![
        ready(),
        Ok(Response::success(envelope("https://signed/put"))),
        ready(),
        Ok(Response::success(act_response(json!({ "asset": "cover", "status": "set" })))),
    ]);
    let mut out = vec![];
    runner.run(act_with_file("attach_robot_media", COVER_INPUT, &path), &mut out).unwrap();

    {
        let sent = calls.sent.lock().unwrap();
        let first = sent.iter().find(|r| r.command == "act").unwrap();
        assert_eq!(first.payload["input"]["content_type"], "image/png");
        assert_eq!(first.payload["input"]["listing_id"], "L1");
    }

    let puts = log_of_puts.lock().unwrap();
    assert_eq!(puts.len(), 1);
    assert_eq!(puts[0].0, "https://signed/put");
    assert!(puts[0].1.contains(&("x-goog-content-length-range".into(), "0,10485760".into())));
    assert_eq!(puts[0].2, path);

    let sent = calls.sent.lock().unwrap();
    let confirm = sent.last().unwrap();
    assert_eq!(confirm.payload["name"], "attach_robot_media");
    assert_eq!(confirm.payload["input"]["confirm"], "https://cdn/x.png");
    assert_eq!(output(out)["result"]["status"], "set");
}

#[test]
fn a_result_without_an_envelope_is_returned_as_is() {
    let path = image("plain.png");
    let (uploads, puts) = FakeUploads::new();
    let (runner, _) = attach_runner(uploads, vec![
        ready(),
        Ok(Response::success(act_response(json!({ "asset": "cover", "status": "already set" })))),
    ]);
    let mut out = vec![];
    runner.run(act_with_file("attach_robot_media", COVER_INPUT, &path), &mut out).unwrap();
    assert!(puts.lock().unwrap().is_empty(), "nothing should have been uploaded");
    assert_eq!(output(out)["result"]["status"], "already set");
}

#[test]
fn a_failed_upload_does_not_confirm() {
    let path = image("doomed.png");
    let (uploads, _) = FakeUploads::failing("the upload was refused (403)");
    let (runner, calls) = attach_runner(uploads, vec![
        ready(),
        Ok(Response::success(envelope("https://signed/put"))),
    ]);
    let mut out = vec![];
    let error = runner.run(act_with_file("attach_robot_media", COVER_INPUT, &path), &mut out).unwrap_err();
    assert!(error.to_string().contains("403"), "{error}");
    assert_eq!(calls.sent.lock().unwrap().iter().filter(|r| r.command == "act").count(), 1);
}

#[test]
fn unknown_file_types_are_refused_before_anything_is_called() {
    let path = image("banner.xyz");
    let (uploads, puts) = FakeUploads::new();
    let (runner, calls) = attach_runner(uploads, vec![]);
    let mut out = vec![];
    let error = runner.run(act_with_file("attach_robot_media", COVER_INPUT, &path), &mut out).unwrap_err();
    assert!(error.to_string().contains("content_type"), "{error}");
    assert!(puts.lock().unwrap().is_empty());
    assert!(calls.sent.lock().unwrap().is_empty(), "nothing should have been sent");
}

#[test]
fn a_missing_file_is_refused_before_anything_is_called() {
    let (uploads, _) = FakeUploads::new();
    let (runner, calls) = attach_runner(uploads, vec![]);
    let mut out = vec![];
    let error = runner.run(act_with_file("attach_robot_media", COVER_INPUT, &PathBuf::from("no-such.png")), &mut out).unwrap_err();
    assert!(error.to_string().contains("cannot read"), "{error}");
    assert!(calls.sent.lock().unwrap().is_empty());
}

#[test]
fn any_tool_that_answers_with_an_upload_is_handled_the_same_way() {
    let path = image("avatar.webp");
    let (uploads, puts) = FakeUploads::new();
    let mut envelope = envelope("https://signed/avatar");
    envelope["result"]["upload"]["confirm_with"] = json!({ "name": "manage_robot_profile", "input": { "profile_id": "P1", "confirm": "https://cdn/a.webp" } });
    let (runner, calls) = attach_runner(uploads, vec![
        ready(),
        Ok(Response::success(envelope)),
        ready(),
        Ok(Response::success(act_response(json!({ "status": "attached" })))),
    ]);
    let mut out = vec![];
    runner.run(act_with_file("manage_robot_profile", r#"{"profile_id":"P1"}"#, &path), &mut out).unwrap();
    assert_eq!(puts.lock().unwrap()[0].0, "https://signed/avatar");
    let sent = calls.sent.lock().unwrap();
    let first = sent.iter().find(|r| r.command == "act").unwrap();
    assert_eq!(first.payload["input"]["content_type"], "image/webp");
    let confirm = sent.last().unwrap();
    assert_eq!(confirm.payload["name"], "manage_robot_profile");
    assert_eq!(confirm.payload["input"]["profile_id"], "P1");
}

#[test]
fn an_explicit_content_type_in_the_input_is_kept() {
    let path = image("banner.png");
    let (uploads, _) = FakeUploads::new();
    let (runner, calls) = attach_runner(uploads, vec![
        ready(),
        Ok(Response::success(act_response(json!({ "status": "done" })))),
    ]);
    let mut out = vec![];
    let input = r#"{"listing_id":"L1","asset":"cover","content_type":"image/jpeg"}"#;
    runner.run(act_with_file("attach_robot_media", input, &path), &mut out).unwrap();
    let sent = calls.sent.lock().unwrap();
    let first = sent.iter().find(|r| r.command == "act").unwrap();
    assert_eq!(first.payload["input"]["content_type"], "image/jpeg");
}

#[test]
fn the_envelope_is_read_from_the_tool_result_not_the_response() {
    let path = image("level.png");
    let (uploads, puts) = FakeUploads::new();
    let stray = json!({ "upload": { "url": "https://signed/wrong-level", "confirm_with": { "name": "t" } }, "result": { "ok": true } });
    let (runner, _) = attach_runner(uploads, vec![ready(), Ok(Response::success(stray))]);
    let mut out = vec![];
    runner.run(act_with_file("attach_robot_media", COVER_INPUT, &path), &mut out).unwrap();
    assert!(puts.lock().unwrap().is_empty(), "a top-level upload key must not be treated as the envelope");
}

#[test]
fn a_tool_error_is_returned_without_uploading() {
    let path = image("err.png");
    let (uploads, puts) = FakeUploads::new();
    let response = json!({ "result": { "error": "listing not found" }, "is_error": true, "role": "manager", "balance": 1.0, "changed": false, "files": [] });
    let (runner, _) = attach_runner(uploads, vec![ready(), Ok(Response::success(response))]);
    let mut out = vec![];
    runner.run(act_with_file("attach_robot_media", COVER_INPUT, &path), &mut out).unwrap();
    assert!(puts.lock().unwrap().is_empty());
    assert_eq!(output(out)["is_error"], true);
}
