use crate::error::{DesktopError, Result};
use crate::ipc::name::SocketName;
use crate::ipc::protocol::{Request, Response, Transport};
use interprocess::local_socket::{prelude::*, Stream};
use std::io::{BufRead, BufReader, Write};

pub struct SocketTransport;

impl Transport for SocketTransport {
    fn send(&self, request: Request) -> Result<Response> {
        let name = SocketName::resolve()?;
        let stream = Stream::connect(name).map_err(|_| DesktopError::AppNotRunning)?;
        let mut writer = &stream;
        writer.write_all(serde_json::to_string(&request)?.as_bytes())?;
        writer.write_all(b"\n")?;
        writer.flush()?;
        let mut line = String::new();
        BufReader::new(&stream).read_line(&mut line)?;
        if line.trim().is_empty() {
            return Err(DesktopError::Remote("app closed the connection".to_string()));
        }
        Ok(serde_json::from_str(&line)?)
    }
}
