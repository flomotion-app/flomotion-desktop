use crate::error::Result;

pub trait AppLauncher: Send + Sync {
    fn launch(&self) -> Result<()>;
}

pub struct ProcessLauncher;

impl AppLauncher for ProcessLauncher {
    fn launch(&self) -> Result<()> {
        let exe = std::env::current_exe()?;
        platform::spawn_detached(&exe)
    }
}

#[cfg(not(windows))]
mod platform {
    use crate::error::Result;
    use std::path::Path;
    use std::process::{Command, Stdio};

    pub fn spawn_detached(exe: &Path) -> Result<()> {
        Command::new(exe).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn()?;
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

    pub fn spawn_detached(exe: &Path) -> Result<()> {
        let mut command_line = quoted_wide(exe.as_os_str());
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

    fn quoted_wide(exe: &OsStr) -> Vec<u16> {
        let mut wide: Vec<u16> = vec![u16::from(b'"')];
        wide.extend(exe.encode_wide());
        wide.push(u16::from(b'"'));
        wide.push(0);
        wide
    }
}
