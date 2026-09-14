use crate::app::bridge::WebviewBridge;
use crate::app::files::PageFiles;
use crate::ipc::protocol::Response;
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub fn cli_ready(bridge: State<'_, Arc<WebviewBridge>>) {
    bridge.set_ready();
}

#[tauri::command]
pub fn cli_respond(bridge: State<'_, Arc<WebviewBridge>>, id: u64, response: Response) {
    bridge.complete(id, response);
}

#[tauri::command]
pub fn cli_file_url(files: State<'_, Arc<PageFiles>>, path: String) -> std::result::Result<String, String> {
    files.url(&path).map_err(|e| e.to_string())
}
