mod support;

use flomotion_desktop_lib::cli::imports::{ImportPayload, ImportTransport, INLINE_LIMIT_BYTES};
use std::fs;
use std::path::PathBuf;

fn write(name: &str, bytes: usize) -> PathBuf {
    let dir = std::env::temp_dir().join("flomotion-import-tests");
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join(name);
    fs::write(&path, vec![b'z'; bytes]).unwrap();
    path
}

#[test]
fn small_files_travel_inline() {
    let payload = ImportPayload::build(&write("small.step", 9), ImportTransport::Auto).unwrap();
    assert_eq!(payload["name"], "small.step");
    assert_eq!(payload["data_base64"], "enp6enp6enp6");
    assert_eq!(payload["size"], 9);
    assert!(payload.get("path").is_none());
}

#[test]
fn large_files_travel_as_a_path() {
    let path = write("big.zip", INLINE_LIMIT_BYTES as usize + 1);
    let payload = ImportPayload::build(&path, ImportTransport::Auto).unwrap();
    assert_eq!(payload["name"], "big.zip");
    assert!(payload.get("data_base64").is_none());
    assert!(payload["path"].as_str().unwrap().ends_with("/big.zip"));
}

#[test]
fn the_path_is_absolute_and_forward_slashed() {
    let path = write("rel.step", 4);
    let payload = ImportPayload::build(&path, ImportTransport::Path).unwrap();
    let sent = payload["path"].as_str().unwrap();
    assert!(!sent.contains('\\'), "{sent} still has backslashes");
    assert!(PathBuf::from(sent).is_absolute(), "{sent} is not absolute");
    assert!(!sent.starts_with("\\?\\"), "{sent} keeps the windows verbatim prefix");
    assert!(sent.ends_with("/rel.step"));
}

#[test]
fn transport_can_be_forced_either_way() {
    let path = write("forced.step", 12);
    assert!(ImportPayload::build(&path, ImportTransport::Path).unwrap().get("data_base64").is_none());
    assert!(ImportPayload::build(&path, ImportTransport::Inline).unwrap().get("path").is_none());
}

#[test]
fn forcing_inline_on_a_large_file_is_refused() {
    let path = write("toobig.zip", INLINE_LIMIT_BYTES as usize + 1);
    let error = ImportPayload::build(&path, ImportTransport::Inline).unwrap_err();
    assert!(error.to_string().contains("--transport path"), "{error}");
}

#[test]
fn missing_files_and_directories_are_rejected() {
    let missing = ImportPayload::build(&PathBuf::from("no-such-file.step"), ImportTransport::Auto).unwrap_err();
    assert!(missing.to_string().contains("cannot read"), "{missing}");
    let dir = std::env::temp_dir().join("flomotion-import-tests");
    let error = ImportPayload::build(&dir, ImportTransport::Auto).unwrap_err();
    assert!(error.to_string().contains("not a file"), "{error}");
}

#[test]
fn unknown_transport_names_are_rejected() {
    let error = ImportTransport::parse(Some("magic")).unwrap_err();
    assert!(error.to_string().contains("expected path, inline or auto"), "{error}");
}
