use health_status::{AlertEventHistory, DEFAULT_ALERT_HISTORY_LIMIT};
use std::fs;
use std::path::{Path, PathBuf};

const FILE_NAME: &str = "alert-history.json";

pub fn path(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join(FILE_NAME)
}

pub fn load(path: &Path) -> AlertEventHistory {
    fs::read(path)
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_else(|| AlertEventHistory::new(DEFAULT_ALERT_HISTORY_LIMIT))
}

pub fn save(path: &Path, history: &AlertEventHistory) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }

    let bytes = serde_json::to_vec_pretty(history).map_err(|error| error.to_string())?;
    let temporary = path.with_extension("json.tmp");
    fs::write(&temporary, bytes).map_err(|error| error.to_string())?;
    fs::rename(&temporary, path).map_err(|error| error.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use health_status::{AlertDecision, AlertEvent, AlertEventReason, AlertSeverity, SignalKind};

    #[test]
    fn round_trip_preserves_history() {
        let root = std::env::temp_dir().join(format!(
            "linux-powerhouse-alert-history-{}",
            std::process::id()
        ));
        let file = path(&root);
        let mut history = AlertEventHistory::new(2);
        history.record(AlertEvent {
            timestamp_ms: 123,
            kind: SignalKind::Cpu,
            severity: AlertSeverity::Warning,
            value: 85.0,
            decision: AlertDecision::Notify,
            reason: AlertEventReason::ActivePolicy,
        });

        save(&file, &history).unwrap();
        let loaded = load(&file);
        assert_eq!(loaded, history);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn missing_file_returns_bounded_default_history() {
        let file = std::env::temp_dir().join(format!(
            "linux-powerhouse-missing-alert-history-{}-{}.json",
            std::process::id(),
            1
        ));
        let loaded = load(&file);
        assert_eq!(loaded.limit(), DEFAULT_ALERT_HISTORY_LIMIT);
        assert!(loaded.events().is_empty());
    }
}
