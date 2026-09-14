use flomotion_desktop_lib::app::files::{AssetScope, PageFiles};
use flomotion_desktop_lib::error::Result;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

struct RecordingScope {
    allowed: Mutex<Vec<PathBuf>>,
}

impl AssetScope for RecordingScope {
    fn allow(&self, path: &Path) -> Result<()> {
        self.allowed.lock().unwrap().push(path.to_path_buf());
        Ok(())
    }
}

fn write(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join("flomotion-pagefile-tests");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(name);
    std::fs::write(&path, b"data").unwrap();
    path
}

#[test]
fn a_url_is_returned_and_the_file_is_granted() {
    let scope = Box::new(RecordingScope { allowed: Mutex::new(vec![]) });
    let files = PageFiles::new(scope);
    let path = write("model.zip");
    let url = files.url(path.to_str().unwrap()).unwrap();
    assert!(url.contains("asset"), "{url}");
    assert!(url.ends_with("model.zip"), "{url}");
    assert!(!url.contains('\\'), "{url}");
}

#[test]
fn spaces_are_percent_encoded() {
    let files = PageFiles::new(Box::new(RecordingScope { allowed: Mutex::new(vec![]) }));
    let url = files.url(write("robot arm.zip").to_str().unwrap()).unwrap();
    assert!(url.ends_with("robot%20arm.zip"), "{url}");
}

#[test]
fn missing_files_are_rejected_before_granting() {
    let files = PageFiles::new(Box::new(RecordingScope { allowed: Mutex::new(vec![]) }));
    let error = files.url("definitely-not-here.zip").unwrap_err();
    assert!(error.to_string().contains("cannot read"), "{error}");
}
