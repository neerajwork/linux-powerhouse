//! Read-only filesystem capacity information for Linux.

use serde::{Deserialize, Serialize};
use std::ffi::CString;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageStatusError {
    #[error("invalid filesystem path: {0}")]
    InvalidPath(String),
    #[error("failed to inspect filesystem {path}: {source}")]
    Stat { path: String, source: std::io::Error },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FilesystemStatus {
    pub mount_point: String,
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub used_bytes: u64,
    pub usage_percent: u8,
}

/// Return capacity information for the mounted filesystems visible in `/proc/mounts`.
pub fn collect() -> Result<Vec<FilesystemStatus>, StorageStatusError> {
    let mounts = std::fs::read_to_string("/proc/mounts")
        .map_err(|source| StorageStatusError::Stat { path: "/proc/mounts".into(), source })?;
    let mut result = Vec::new();
    for line in mounts.lines() {
        let mut fields = line.split_whitespace();
        let _device = fields.next();
        let mount = fields.next();
        let filesystem = fields.next();
        let Some(mount) = mount else { continue };
        let Some(filesystem) = filesystem else { continue };
        if !matches!(filesystem, "ext4" | "ext3" | "xfs" | "btrfs" | "zfs" | "f2fs" | "tmpfs" | "vfat" | "ntfs" | "overlay") {
            continue;
        }
        let mount = mount.replace("\\040", " ").replace("\\011", "\t");
        if let Ok(status) = statvfs(&mount) {
            result.push(status);
        }
    }
    result.sort_by(|a, b| a.mount_point.cmp(&b.mount_point));
    result.dedup_by(|a, b| a.mount_point == b.mount_point);
    Ok(result)
}

fn statvfs(mount_point: &str) -> Result<FilesystemStatus, StorageStatusError> {
    let c_path = CString::new(mount_point).map_err(|_| StorageStatusError::InvalidPath(mount_point.into()))?;
    let mut stats = std::mem::MaybeUninit::<libc::statvfs>::uninit();
    let result = unsafe { libc::statvfs(c_path.as_ptr(), stats.as_mut_ptr()) };
    if result != 0 {
        return Err(StorageStatusError::Stat {
            path: mount_point.into(),
            source: std::io::Error::last_os_error(),
        });
    }
    let stats = unsafe { stats.assume_init() };
    let block = stats.f_frsize as u64;
    let total = (stats.f_blocks as u64).saturating_mul(block);
    let available = (stats.f_bavail as u64).saturating_mul(block);
    let used = total.saturating_sub((stats.f_bfree as u64).saturating_mul(block));
    let usage_percent = if total == 0 { 0 } else { ((used.saturating_mul(100)) / total).min(100) as u8 };
    Ok(FilesystemStatus { mount_point: mount_point.into(), total_bytes: total, available_bytes: available, used_bytes: used, usage_percent })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_filesystem_is_readable() {
        let result = statvfs("/").expect("root filesystem should be readable");
        assert!(result.total_bytes > 0);
        assert!(result.available_bytes <= result.total_bytes);
        assert!(result.usage_percent <= 100);
    }
}
