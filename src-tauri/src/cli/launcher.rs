use crate::error::Result;
use crate::window_name::WindowName;

pub trait AppLauncher: Send + Sync {
    fn launch(&self) -> Result<()>;
}

pub struct ProcessLauncher {
    window: WindowName,
}

impl ProcessLauncher {
    pub fn new(window: WindowName) -> Self {
        Self { window }
    }

    // The arguments the spawned window is started with, so a CLI run under a
    // name always brings up a window of that name.
    pub fn args(window: &WindowName) -> Vec<String> {
        vec!["--window".to_string(), window.as_str().to_string()]
    }
}

impl AppLauncher for ProcessLauncher {
    fn launch(&self) -> Result<()> {
        let exe = std::env::current_exe()?;
        platform::spawn_detached(&exe, self.window.as_str())
    }
}

#[cfg(not(windows))]
mod platform {
    use crate::error::Result;
    use std::path::Path;
    use std::process::{Command, Stdio};

    pub fn spawn_detached(exe: &Path, window: &str) -> Result<()> {
        Command::new(exe)
            .args(super::ProcessLauncher::args(&crate::window_name::WindowName::parse(window).unwrap_or_default()))
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
        Ok(())
    }
}

#[cfg(windows)]
mod platform {
    use crate::error::Result;
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::path::Path;
    use std::{io, ptr};
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::Threading::{
        CreateProcessW, CREATE_NEW_PROCESS_GROUP, DETACHED_PROCESS, PROCESS_INFORMATION, STARTUPINFOW,
    };

    pub fn spawn_detached(exe: &Path, window: &str) -> Result<()> {
        let mut command_line = quoted_wide(exe.as_os_str(), window);
        let mut startup: STARTUPINFOW = unsafe { std::mem::zeroed() };
        startup.cb = std::mem::size_of::<STARTUPINFOW>() as u32;
        let mut info: PROCESS_INFORMATION = unsafe { std::mem::zeroed() };
        let created = unsafe {
            CreateProcessW(
                ptr::null(),
                command_line.as_mut_ptr(),
                ptr::null(),
                ptr::null(),
                0,
                DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP,
                ptr::null(),
                ptr::null(),
                &startup,
                &mut info,
            )
        };
        if created == 0 {
            return Err(io::Error::last_os_error().into());
        }
        unsafe {
            CloseHandle(info.hThread);
            CloseHandle(info.hProcess);
        }
        Ok(())
    }

    fn quoted_wide(exe: &OsStr, window: &str) -> Vec<u16> {
        let mut wide: Vec<u16> = vec![u16::from(b'"')];
        wide.extend(exe.encode_wide());
        wide.push(u16::from(b'"'));
        wide.extend(OsStr::new(" --window ").encode_wide());
        wide.extend(OsStr::new(window).encode_wide());
        wide.push(0);
        wide
    }
}
