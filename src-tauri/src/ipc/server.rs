use crate::ipc::name::SocketName;
use crate::ipc::protocol::{Request, RequestHandler, Response};
use interprocess::local_socket::{prelude::*, ListenerOptions, Stream};
use std::io::{self, BufRead, BufReader, Write};
use std::sync::Arc;
use std::thread;

pub struct IpcServer {
    handler: Arc<dyn RequestHandler>,
}

impl IpcServer {
    pub fn new(handler: Arc<dyn RequestHandler>) -> Self {
        Self { handler }
    }

    pub fn start(self) -> io::Result<()> {
        SocketName::remove_stale();
        let listener = ListenerOptions::new().name(SocketName::resolve()?).create_sync()?;
        thread::Builder::new().name("ipc-server".into()).spawn(move || {
            for connection in listener.incoming() {
                match connection {
                    Ok(stream) => self.spawn_connection(stream),
                    Err(error) => eprintln!("ipc accept failed: {error}"),
                }
            }
        })?;
        Ok(())
    }

    fn spawn_connection(&self, stream: Stream) {
        let handler = Arc::clone(&self.handler);
        thread::spawn(move || {
            if let Err(error) = Self::serve(&stream, handler.as_ref()) {
                eprintln!("ipc connection failed: {error}");
            }
        });
    }

    fn serve(stream: &Stream, handler: &dyn RequestHandler) -> io::Result<()> {
        let mut line = String::new();
        BufReader::new(stream).read_line(&mut line)?;
        let response = match serde_json::from_str::<Request>(&line) {
            Ok(request) => handler.handle(request),
            Err(error) => Response::failure(format!("malformed request: {error}")),
        };
        let mut writer = stream;
        writer.write_all(serde_json::to_string(&response)?.as_bytes())?;
        writer.write_all(b"\n")?;
        writer.flush()
    }
}
