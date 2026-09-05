fn main() {
    let manifest = tauri_build::AppManifest::new().commands(&["cli_ready", "cli_respond"]);
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(manifest)).expect("failed to run tauri-build");
}
