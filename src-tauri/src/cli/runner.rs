use crate::cli::files::FileMaterializer;
use crate::cli::imports::{ImportPayload, ImportTransport};
use crate::cli::launcher::AppLauncher;
use crate::cli::uploads::{PendingUpload, UploadTransport};
use crate::cli::Command;
use crate::error::{DesktopError, Result};
use crate::ipc::protocol::{Request, Response, Transport};
use serde_json::{json, Value};
use std::io::Write;
use std::path::Path;
use std::time::{Duration, Instant};

const SKILL_TEXT: &str = include_str!("../../../AGENT.md");
const POLL_INTERVAL: Duration = Duration::from_millis(250);
const CONTENT_TYPE_FIELD: &str = "content_type";

pub struct CliRunner {
    transport: Box<dyn Transport>,
    launcher: Box<dyn AppLauncher>,
    files: FileMaterializer,
    uploads: Box<dyn UploadTransport>,
    start_timeout: Duration,
}

impl CliRunner {
    pub fn new(
        transport: Box<dyn Transport>,
        launcher: Box<dyn AppLauncher>,
        files: FileMaterializer,
        uploads: Box<dyn UploadTransport>,
        start_timeout: Duration,
    ) -> Self {
        Self { transport, launcher, files, uploads, start_timeout }
    }

    pub fn run(&self, command: Command, out: &mut dyn Write) -> Result<()> {
        let value = match command {
            Command::Skill => return Ok(out.write_all(SKILL_TEXT.as_bytes())?),
            Command::Status => self.status()?,
            Command::Agent { reset } => self.call("agent", json!({ "reset": reset }))?,
            Command::Act { name, input, input_file, wait, file } => {
                let input = Self::parse_input(input, input_file.as_deref())?;
                match file {
                    Some(path) => self.act_with_file(&name, input, &path, wait)?,
                    None => self.act(&name, input, wait)?,
                }
            }
            Command::Login { code } => self.call("login", Self::login_payload(code))?,
            Command::Diagnose => self.call("diagnose", json!({}))?,
            Command::Job { id, wait } => self.call("job", Self::with_wait(json!({ "job_id": id }), wait))?,
            Command::Import { path, transport } => {
                let payload = ImportPayload::build(&path, ImportTransport::parse(transport.as_deref())?)?;
                self.call("import", payload)?
            }
            Command::Export { kind, id, out } => {
                self.ensure_page_ready()?;
                let response = self.transport.send(Request::new("export", json!({ "kind": kind, "id": id })))?;
                self.files.materialize_into(Self::unwrap(response)?, out.as_deref())?
            }
        };
        writeln!(out, "{}", serde_json::to_string_pretty(&value)?)?;
        Ok(())
    }

    fn status(&self) -> Result<Value> {
        match self.transport.send(Request::status()) {
            Ok(response) => Ok(Self::unwrap(response)?),
            Err(DesktopError::AppNotRunning) => Ok(json!({ "running": false })),
            Err(error) => Err(error),
        }
    }

    fn call(&self, command: &str, payload: Value) -> Result<Value> {
        self.ensure_page_ready()?;
        let response = self.transport.send(Request::new(command, payload))?;
        self.files.materialize(Self::unwrap(response)?)
    }

    fn act(&self, name: &str, input: Value, wait: Option<u64>) -> Result<Value> {
        self.call("act", Self::with_wait(json!({ "name": name, "input": input }), wait))
    }

    fn login_payload(code: Option<String>) -> Value {
        match code {
            Some(code) => json!({ "code": code }),
            None => json!({}),
        }
    }

    fn with_wait(mut payload: Value, wait: Option<u64>) -> Value {
        if let Some(seconds) = wait {
            payload["wait"] = json!(seconds);
        }
        payload
    }

    fn act_with_file(&self, name: &str, mut input: Value, path: &Path, wait: Option<u64>) -> Result<Value> {
        let metadata = std::fs::metadata(path)
            .map_err(|e| DesktopError::Remote(format!("cannot read {}: {e}", path.display())))?;
        if !metadata.is_file() {
            return Err(DesktopError::Remote(format!("{} is not a file", path.display())));
        }
        if input.get(CONTENT_TYPE_FIELD).is_none() {
            input[CONTENT_TYPE_FIELD] = Value::String(Self::content_type(path)?);
        }
        let started = self.act(name, input, wait)?;
        let Some(upload) = PendingUpload::find(&started["result"])? else {
            return Ok(started);
        };
        self.uploads.put(&upload.url, &upload.headers, path)?;
        let (confirm, confirm_input) = upload.confirm_call()?;
        self.act(&confirm, confirm_input, wait)
    }

    fn content_type(path: &Path) -> Result<String> {
        let extension = path
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        let known = match extension.as_str() {
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "webp" => "image/webp",
            "gif" => "image/gif",
            "pdf" => "application/pdf",
            "mp4" => "video/mp4",
            "mov" => "video/quicktime",
            "webm" => "video/webm",
            "txt" => "text/plain",
            "csv" => "text/csv",
            "zip" => "application/zip",
            other => {
                return Err(DesktopError::Remote(format!(
                    "cannot tell the type of '{other}' files; put \"{CONTENT_TYPE_FIELD}\" in the input"
                )))
            }
        };
        Ok(known.into())
    }

    fn ensure_page_ready(&self) -> Result<()> {
        match self.transport.send(Request::status()) {
            Ok(response) if Self::page_ready(&response) => return Ok(()),
            Ok(_) => {}
            Err(DesktopError::AppNotRunning) => self.launcher.launch()?,
            Err(error) => return Err(error),
        }
        self.wait_for_page()
    }

    fn wait_for_page(&self) -> Result<()> {
        let deadline = Instant::now() + self.start_timeout;
        while Instant::now() < deadline {
            if let Ok(response) = self.transport.send(Request::status()) {
                if Self::page_ready(&response) {
                    return Ok(());
                }
            }
            std::thread::sleep(POLL_INTERVAL);
        }
        Err(DesktopError::AppStartTimeout(self.start_timeout.as_secs()))
    }

    fn page_ready(response: &Response) -> bool {
        response.ok && response.result["page_ready"] == Value::Bool(true)
    }

    fn unwrap(response: Response) -> Result<Value> {
        if response.ok {
            return Ok(response.result);
        }
        Err(DesktopError::Remote(response.error.unwrap_or_else(|| "unknown error".into())))
    }

    fn parse_input(inline: Option<String>, file: Option<&Path>) -> Result<Value> {
        let text = match (inline, file) {
            (_, Some(path)) => std::fs::read_to_string(path)?,
            (Some(text), None) => text,
            (None, None) => return Ok(json!({})),
        };
        let value: Value = serde_json::from_str(&text)?;
        if !value.is_object() {
            return Err(DesktopError::Remote("tool input must be a JSON object".into()));
        }
        Ok(value)
    }
}
