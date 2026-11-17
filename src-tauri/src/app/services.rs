use crate::app::bridge::WebviewBridge;
use crate::app::files::PageFiles;
use crate::app::serial::SerialHub;
use crate::app::window::WindowFactory;
use crate::ipc::server::IpcServer;
use std::sync::Arc;

pub struct AppServices {
    pub bridge: Arc<WebviewBridge>,
    pub files: Arc<PageFiles>,
    pub serial: Arc<SerialHub>,
    pub ipc: IpcServer,
    pub window: WindowFactory,
}
