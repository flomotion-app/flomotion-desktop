mod support;

use flomotion_desktop_lib::cli::files::FileMaterializer;
use flomotion_desktop_lib::cli::runner::CliRunner;
use flomotion_desktop_lib::cli::Command;
use flomotion_desktop_lib::error::Result;
use flomotion_desktop_lib::ipc::protocol::Response;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use support::{output, ready, FakeLauncher, FakeStore, FakeTransport, FakeUploads, TransportLog};


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
        let first = sent.iter().find(|request| request.command == "act").unwrap();
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
    assert_eq!(calls.sent.lock().unwrap().iter().filter(|request| request.command == "act").count(), 1);
}

#[test]
fn unknown_file_types_are_sent_as_octet_stream() {
    let path = image("firmware.bin");
    let (uploads, _) = FakeUploads::new();
    let (runner, calls) = attach_runner(uploads, vec![
        ready(),
        Ok(Response::success(act_response(json!({ "status": "already set" })))),
    ]);
    let mut out = vec![];
    runner.run(act_with_file("put_codebase_data_file", COVER_INPUT, &path), &mut out).unwrap();
    let sent = calls.sent.lock().unwrap();
    let first = sent.iter().find(|request| request.command == "act").unwrap();
    assert_eq!(first.payload["input"]["content_type"], "application/octet-stream");
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
    let first = sent.iter().find(|request| request.command == "act").unwrap();
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
    let first = sent.iter().find(|request| request.command == "act").unwrap();
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
