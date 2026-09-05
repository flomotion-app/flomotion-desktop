use crate::cli::files::FileMaterializer;
use crate::cli::launcher::AppLauncher;
use crate::cli::Command;
use crate::error::{DesktopError, Result};
use crate::ipc::protocol::{Request, Response, Transport};
use serde_json::{json, Value};
use std::io::Write;
use std::path::Path;
use std::time::{Duration, Instant};

const SKILL_TEXT: &str = include_str!("../../../AGENT.md");
const POLL_INTERVAL: Duration = Duration::from_millis(250);

pub struct CliRunner {
    transport: Box<dyn Transport>,
    launcher: Box<dyn AppLauncher>,
    files: FileMaterializer,
    start_timeout: Duration,
}

impl CliRunner {
    pub fn new(transport: Box<dyn Transport>, launcher: Box<dyn AppLauncher>, files: FileMaterializer, start_timeout: Duration) -> Self {
        Self { transport, launcher, files, start_timeout }
    }

    pub fn run(&self, command: Command, out: &mut dyn Write) -> Result<()> {
        let value = match command {
            Command::Skill => return Ok(out.write_all(SKILL_TEXT.as_bytes())?),
            Command::Status => self.status()?,
            Command::Agent { reset } => self.call("agent", json!({ "reset": reset }))?,
            Command::Act { name, input, input_file, wait } => {
                let input = Self::parse_input(input, input_file.as_deref())?;
                self.call("act", json!({ "name": name, "input": input, "wait": wait }))?
            }
            Command::Job { id, wait } => self.call("job", json!({ "job_id": id, "wait": wait }))?,
            Command::Import { path } => self.call("import", FileMaterializer::encode(&path)?)?,
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
