use interprocess::local_socket::Name;
use std::io;

const SOCKET_FILE: &str = "flomotion.sock";

pub struct SocketName;

impl SocketName {
    #[cfg(windows)]
    pub fn resolve() -> io::Result<Name<'static>> {
        use interprocess::local_socket::{GenericNamespaced, ToNsName};
        SOCKET_FILE.to_string().to_ns_name::<GenericNamespaced>()
    }

    #[cfg(not(windows))]
    pub fn resolve() -> io::Result<Name<'static>> {
        use interprocess::local_socket::{GenericFilePath, ToFsName};
        Self::socket_path()?.to_fs_name::<GenericFilePath>()
    }

    #[cfg(not(windows))]
    pub fn socket_path() -> io::Result<std::path::PathBuf> {
        use std::os::unix::fs::PermissionsExt;
        let dirs = directories::BaseDirs::new()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no home directory"))?;
        let dir = dirs
            .runtime_dir()
            .map(|d| d.join("flomotion"))
            .unwrap_or_else(|| dirs.home_dir().join(".flomotion").join("run"));
        std::fs::create_dir_all(&dir)?;
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700))?;
        Ok(dir.join(SOCKET_FILE))
    }

    #[cfg(not(windows))]
    pub fn remove_stale() {
        if let Ok(path) = Self::socket_path() {
            let _ = std::fs::remove_file(path);
        }
    }

    #[cfg(windows)]
    pub fn remove_stale() {}
}
