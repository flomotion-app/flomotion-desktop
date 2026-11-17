use super::ports::BoardPort;
use super::SerialHub;
use std::sync::Arc;
use tauri::State;

type Hub<'a> = State<'a, Arc<SerialHub>>;

async fn blocking<T: Send + 'static>(work: impl FnOnce() -> Result<T, String> + Send + 'static) -> Result<T, String> {
    tauri::async_runtime::spawn_blocking(work).await.map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn serial_ports(hub: Hub<'_>) -> Result<Vec<BoardPort>, String> {
    let hub = Arc::clone(&hub);
    blocking(move || Ok(hub.ports())).await
}

#[tauri::command]
pub async fn serial_open(hub: Hub<'_>, path: String, baud: u32) -> Result<(), String> {
    let hub = Arc::clone(&hub);
    blocking(move || hub.open(&path, baud)).await
}

#[tauri::command]
pub async fn serial_close(hub: Hub<'_>, path: String) -> Result<(), String> {
    let hub = Arc::clone(&hub);
    blocking(move || {
        hub.close(&path);
        Ok(())
    })
    .await
}

#[tauri::command]
pub async fn serial_flash(hub: Hub<'_>, path: String, chip: String, image: String, offset: Option<u32>) -> Result<(), String> {
    let hub = Arc::clone(&hub);
    blocking(move || hub.flash(&path, &chip, &image, offset.unwrap_or(0))).await
}
