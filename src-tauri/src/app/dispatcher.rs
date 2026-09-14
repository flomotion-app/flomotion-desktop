use crate::app::bridge::PageChannel;
use crate::app::files::PageFiles;
use crate::config::AppConfig;
use crate::app::window::WindowControl;
use crate::ipc::protocol::{Request, RequestHandler, Response, COMMAND_FOCUS, COMMAND_STATUS};
use serde_json::{json, Value};
use std::sync::Arc;

pub struct Dispatcher {
    config: AppConfig,
    page: Arc<dyn PageChannel>,
    window: Arc<dyn WindowControl>,
    files: Arc<PageFiles>,
}

impl Dispatcher {
    pub fn new(config: AppConfig, page: Arc<dyn PageChannel>, window: Arc<dyn WindowControl>, files: Arc<PageFiles>) -> Self {
        Self { config, page, window, files }
    }

    fn grant(&self, request: &Request) {
        if let Some(path) = request.payload.get("path").and_then(Value::as_str) {
            if let Err(error) = self.files.url(path) {
                eprintln!("could not grant {path}: {error}");
            }
        }
    }

    fn status(&self) -> Response {
        Response::success(json!({
            "version": self.config.version,
            "web_url": self.config.web_url,
            "page_ready": self.page.is_ready(),
        }))
    }
}

impl RequestHandler for Dispatcher {
    fn handle(&self, request: Request) -> Response {
        if request.command == COMMAND_STATUS {
            return self.status();
        }
        if request.command == COMMAND_FOCUS {
            self.window.focus();
            return Response::success(Value::Null);
        }
        self.grant(&request);
        match self.page.call(request) {
            Ok(response) => response,
            Err(error) => Response::failure(error.to_string()),
        }
    }
}
