mod support;

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use flomotion_desktop_lib::cli::files::FileMaterializer;
use serde_json::json;
use support::FakeStore;

#[test]
fn values_without_files_pass_through() {
    let (store, _) = FakeStore::new();
    let materializer = FileMaterializer::new(Box::new(store));
    let value = json!({"result": {"ok": 1}});
    assert_eq!(materializer.materialize(value.clone()).unwrap(), value);
}

#[test]
fn files_are_decoded_and_replaced_by_paths() {
    let (store, written) = FakeStore::new();
    let materializer = FileMaterializer::new(Box::new(store));
    let value = json!({"files": [{"name": "shot.png", "data_base64": STANDARD.encode(b"png")}]});
    let out = materializer.materialize(value).unwrap();
    assert_eq!(out["files"][0], "/tmp/shot.png");
    assert_eq!(written.lock().unwrap()[0], ("shot.png".to_string(), b"png".to_vec()));
}

#[test]
fn missing_data_is_an_error() {
    let (store, _) = FakeStore::new();
    let materializer = FileMaterializer::new(Box::new(store));
    let value = json!({"files": [{"name": "shot.png"}]});
    assert!(materializer.materialize(value).is_err());
}
