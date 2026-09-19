use crate::window_name::WindowName;
use interprocess::local_socket::Name;
use std::io;

pub struct SocketName;

impl SocketName {
    #[cfg(windows)]
    pub fn resolve(window: &WindowName) -> io::Result<Name<'static>> {
        use interprocess::local_socket::{GenericNamespaced, ToNsName};
        window.socket_file().to_ns_name::<GenericNamespaced>()
    }

    #[cfg(not(windows))]
    pub fn resolve(window: &WindowName) -> io::Result<Name<'static>> {
        use interprocess::local_socket::{GenericFilePath, ToFsName};
        Self::socket_path(window)?.to_fs_name::<GenericFilePath>()
    }

    #[cfg(not(windows))]
    pub fn socket_path(window: &WindowName) -> io::Result<std::path::PathBuf> {
        use std::os::unix::fs::PermissionsExt;
        let dirs = directories::BaseDirs::new()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no home directory"))?;
        let dir = dirs
            .runtime_dir()
            .map(|d| d.join("flomotion"))
            .unwrap_or_else(|| dirs.home_dir().join(".flomotion").join("run"));
        std::fs::create_dir_all(&dir)?;
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700))?;
        Ok(dir.join(window.socket_file()))
    }

    #[cfg(not(windows))]
    pub fn remove_stale(window: &WindowName) {
        if let Ok(path) = Self::socket_path(window) {
            let _ = std::fs::remove_file(path);
        }
    }

    #[cfg(windows)]
    pub fn remove_stale(_window: &WindowName) {}
}
