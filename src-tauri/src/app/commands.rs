use crate::app::bridge::WebviewBridge;
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
