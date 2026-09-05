use crate::app::bridge::PageChannel;
use crate::config::AppConfig;
use crate::ipc::protocol::{Request, RequestHandler, Response, COMMAND_STATUS};
use serde_json::json;
use std::sync::Arc;

pub struct Dispatcher {
    config: AppConfig,
    page: Arc<dyn PageChannel>,
}

impl Dispatcher {
    pub fn new(config: AppConfig, page: Arc<dyn PageChannel>) -> Self {
        Self { config, page }
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
        match self.page.call(request) {
            Ok(response) => response,
            Err(error) => Response::failure(error.to_string()),
        }
    }
}
