//! Bounded, read-only storage analysis for user-approved paths.
//!
//! This crate intentionally avoids deletion, mutation, symlink traversal, and
//! unbounded recursive scans. It reports space consumers as recommendations;
//! it never performs cleanup itself.

use serde::{Deserialize, Serialize};
use std::{
    fs, io,
    path::{Path, PathBuf},
};
use thiserror::Error;

const DEFAULT_MAX_DEPTH: usize = 4;
const DEFAULT_MAX_ENTRIES: usize = 10_000;
const DEFAULT_TOP_N: usize = 20;

#[derive(Debug, Error)]
pub enum StorageIntelligenceError {
    #[error("storage path does not exist: {0}")]
    MissingPath(String),
    #[error("storage path is not a directory: {0}")]
    NotDirectory(String),
    #[error("failed to inspect storage path {path}: {source}")]
    Io { path: String, source: io::Error },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StorageConsumer {
    pub path: PathBuf,
    pub bytes: u64,
    pub is_directory: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StorageAnalysis {
    pub root: PathBuf,
    pub total_bytes: u64,
    pub entries_scanned: usize,
    pub skipped_entries: usize,
    pub truncated: bool,
    pub top_consumers: Vec<StorageConsumer>,
}

#[derive(Debug, Clone, Copy)]
pub struct ScanLimits {
    pub max_depth: usize,
    pub max_entries: usize,
    pub top_n: usize,
}

impl Default for ScanLimits {
    fn default() -> Self {
        Self {
            max_depth: DEFAULT_MAX_DEPTH,
            max_entries: DEFAULT_MAX_ENTRIES,
            top_n: DEFAULT_TOP_N,
        }
    }
}

/// Analyze a directory using bounded, read-only traversal.
pub fn analyze(root: impl AsRef<Path>) -> Result<StorageAnalysis, StorageIntelligenceError> {
    analyze_with_limits(root, ScanLimits::default())
}

/// Analyze a directory with explicit traversal limits.
pub fn analyze_with_limits(
    root: impl AsRef<Path>,
    limits: ScanLimits,
) -> Result<StorageAnalysis, StorageIntelligenceError> {
    let root = root.as_ref();
    let metadata = fs::symlink_metadata(root).map_err(|source| {
        if source.kind() == io::ErrorKind::NotFound {
            StorageIntelligenceError::MissingPath(root.display().to_string())
        } else {
            StorageIntelligenceError::Io {
                path: root.display().to_string(),
                source,
            }
        }
    })?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(StorageIntelligenceError::NotDirectory(
            root.display().to_string(),
        ));
    }

    let mut state = ScanState {
        limits,
        entries_scanned: 0,
        skipped_entries: 0,
        truncated: false,
        consumers: Vec::new(),
    };
    let total_bytes = scan_directory(root, 0, &mut state)?;
    state
        .consumers
        .sort_by(|a, b| b.bytes.cmp(&a.bytes).then_with(|| a.path.cmp(&b.path)));
    state.consumers.truncate(state.limits.top_n);

    Ok(StorageAnalysis {
        root: root.to_path_buf(),
        total_bytes,
        entries_scanned: state.entries_scanned,
        skipped_entries: state.skipped_entries,
        truncated: state.truncated,
        top_consumers: state.consumers,
    })
}

struct ScanState {
    limits: ScanLimits,
    entries_scanned: usize,
    skipped_entries: usize,
    truncated: bool,
    consumers: Vec<StorageConsumer>,
}

fn scan_directory(
    path: &Path,
    depth: usize,
    state: &mut ScanState,
) -> Result<u64, StorageIntelligenceError> {
    if state.entries_scanned >= state.limits.max_entries {
        state.truncated = true;
        return Ok(0);
    }

    let mut total = 0u64;
    let entries = fs::read_dir(path).map_err(|source| StorageIntelligenceError::Io {
        path: path.display().to_string(),
        source,
    })?;

    for entry in entries {
        if state.entries_scanned >= state.limits.max_entries {
            state.truncated = true;
            break;
        }
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => {
                state.skipped_entries += 1;
                continue;
            }
        };
        state.entries_scanned += 1;
        let entry_path = entry.path();
        let file_type = match entry.file_type() {
            Ok(kind) => kind,
            Err(_) => {
                state.skipped_entries += 1;
                continue;
            }
        };
        if file_type.is_symlink() {
            state.skipped_entries += 1;
            continue;
        }

        let bytes = if file_type.is_dir() {
            if depth >= state.limits.max_depth {
                state.truncated = true;
                0
            } else {
                scan_directory(&entry_path, depth + 1, state).unwrap_or_else(|_| {
                    state.skipped_entries += 1;
                    0
                })
            }
        } else if file_type.is_file() {
            entry
                .metadata()
                .map(|metadata| metadata.len())
                .unwrap_or_else(|_| {
                    state.skipped_entries += 1;
                    0
                })
        } else {
            0
        };

        total = total.saturating_add(bytes);
        if bytes > 0 {
            state.consumers.push(StorageConsumer {
                path: entry_path,
                bytes,
                is_directory: file_type.is_dir(),
            });
        }
    }
    Ok(total)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounded_analysis_reports_existing_directory() {
        let result = analyze_with_limits(
            "/tmp",
            ScanLimits {
                max_depth: 1,
                max_entries: 100,
                top_n: 5,
            },
        );
        let result = result.expect("/tmp should be readable on Linux CI");
        assert_eq!(result.root, PathBuf::from("/tmp"));
        assert!(result.entries_scanned <= 100);
        assert!(result.top_consumers.len() <= 5);
    }

    #[test]
    fn missing_path_is_reported() {
        let result = analyze("/definitely/not/a/linux-powerhouse-path");
        assert!(matches!(
            result,
            Err(StorageIntelligenceError::MissingPath(_))
        ));
    }

    #[test]
    fn symlink_root_is_rejected() {
        let path = std::env::temp_dir().join("linux-powerhouse-storage-intelligence-link");
        let target = std::env::temp_dir();
        let _ = fs::remove_file(&path);
        #[cfg(unix)]
        std::os::unix::fs::symlink(&target, &path).expect("create symlink");
        #[cfg(unix)]
        assert!(matches!(
            analyze(&path),
            Err(StorageIntelligenceError::NotDirectory(_))
        ));
        let _ = fs::remove_file(path);
    }
}
