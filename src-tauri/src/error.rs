use std::io;

#[derive(Debug, thiserror::Error)]
pub enum DesktopError {
    #[error("io: {0}")]
    Io(#[from] io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("app is not running")]
    AppNotRunning,
    #[error("app did not start within {0} seconds")]
    AppStartTimeout(u64),
    #[error("page is not connected yet, open the FloMotion window and sign in")]
    PageNotReady,
    #[error("page did not answer within {0} seconds")]
    PageTimeout(u64),
    #[error("{0}")]
    Remote(String),
}

pub type Result<T> = std::result::Result<T, DesktopError>;
