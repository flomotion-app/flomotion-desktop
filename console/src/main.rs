use std::ffi::OsString;
use std::path::PathBuf;
use std::process::{Command, ExitCode};

struct ConsoleFront {
    target: PathBuf,
    args: Vec<OsString>,
}

impl ConsoleFront {
    fn new(target: PathBuf, args: Vec<OsString>) -> Self {
        Self { target, args }
    }

    fn opens_window(&self) -> bool {
        let mut rest = self.args.iter().map(|arg| arg.to_string_lossy());
        while let Some(arg) = rest.next() {
            if arg == "--window" {
                rest.next();
            } else if !arg.starts_with("--window=") {
                return false;
            }
        }
        true
    }

    fn run(&self) -> ExitCode {
        let mut command = Command::new(&self.target);
        command.args(&self.args);
        if self.opens_window() {
            return match command.spawn() {
                Ok(_) => ExitCode::SUCCESS,
                Err(error) => self.failed(error),
            };
        }
        match command.status() {
            Ok(status) => ExitCode::from(status.code().unwrap_or(1).clamp(0, 255) as u8),
            Err(error) => self.failed(error),
        }
    }

    fn failed(&self, error: std::io::Error) -> ExitCode {
        eprintln!("error: cannot start {}: {error}", self.target.display());
        ExitCode::from(2)
    }
}

fn main() -> ExitCode {
    let target = match std::env::current_exe() {
        Ok(path) => path.with_extension("exe"),
        Err(error) => {
            eprintln!("error: {error}");
            return ExitCode::from(2);
        }
    };
    ConsoleFront::new(target, std::env::args_os().skip(1).collect()).run()
}
