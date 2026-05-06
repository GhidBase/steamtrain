//! Linux-specific Steam detection and operations.

use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Child};

use crate::{Result, SteamTrainError};

pub fn steam_root() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or(SteamTrainError::SteamRootNotFound)?;

    let candidates = [
        home.join(".local/share/Steam"),
        home.join(".steam/steam"),
        home.join(".steam/root"),
    ];

    for path in &candidates {
        if path.join("steamapps").exists() {
            return Ok(path.clone());
        }
    }

    Err(SteamTrainError::SteamRootNotFound)
}

pub fn file_allocated_size(path: &Path) -> Result<i64> {
    let metadata = std::fs::metadata(path)?;
    Ok(metadata.blocks() as i64 * 512)
}

pub fn launch_app_id_platform(app_id: &str, extra_args: &[&str]) -> Result<Child> {
    let mut cmd = Command::new("steam");
    cmd.arg("-applaunch").arg(app_id);
    cmd.args(extra_args);
    cmd.spawn().map_err(Into::into)
}

pub fn uninstall_via_steam(app_id: &str) -> Result<()> {
    let uri = format!("steam://uninstall/{}", app_id);
    Command::new("xdg-open").arg(&uri).spawn()?.wait()?;
    Ok(())
}

pub fn open_install_path(path: &Path) -> Result<()> {
    Command::new("xdg-open").arg(path).spawn()?;
    Ok(())
}

/// Get total and free disk space for a path using statvfs.
/// Returns (total_bytes, free_bytes).
pub fn disk_space(path: &Path) -> Result<(u64, u64)> {
    use std::ffi::CString;
    use std::mem::MaybeUninit;

    let path_str = path.to_str().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid path")
    })?;
    let c_path = CString::new(path_str).map_err(|_| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "path contains null")
    })?;

    let mut stat: MaybeUninit<libc::statvfs> = MaybeUninit::uninit();
    let ret = unsafe { libc::statvfs(c_path.as_ptr(), stat.as_mut_ptr()) };
    if ret != 0 {
        return Err(std::io::Error::last_os_error().into());
    }

    let stat = unsafe { stat.assume_init() };
    let total = stat.f_blocks as u64 * stat.f_frsize as u64;
    let free = stat.f_bfree as u64 * stat.f_frsize as u64;

    Ok((total, free))
}
