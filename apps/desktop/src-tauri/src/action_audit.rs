use health_status::{AlertActionOutcome, AlertActionOutcomeStatus};
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ActionAuditEntry {
    pub id: String,
    pub timestamp: u64,
    pub action: String,
    pub stage: String,
    pub confirmed: bool,
    pub status: String,
    pub message: String,
    pub reversible: bool,
    pub privilege: String,
    #[serde(default = "default_verification_status")]
    pub verification_status: String,
    #[serde(default)]
    pub verification_message: String,
    #[serde(default = "default_outcome_status")]
    pub outcome_status: String,
    #[serde(default)]
    pub outcome_message: String,
}

fn audit_id() -> String {
    format!("action-{}", Uuid::new_v4())
}

fn default_verification_status() -> String {
    "legacy".to_owned()
}

fn default_outcome_status() -> String {
    "legacy".to_owned()
}

fn outcome_status_label(status: &AlertActionOutcomeStatus) -> &'static str {
    match status {
        AlertActionOutcomeStatus::Verified => "verified",
        AlertActionOutcomeStatus::Rejected => "rejected",
    }
}

#[derive(Clone, Default)]
pub struct ActionAudit;

impl ActionAudit {
    pub fn record(
        &self,
        action: &str,
        stage: &str,
        confirmed: bool,
        status: &str,
        message: &str,
        reversible: bool,
        privilege: &str,
        verification_status: &str,
        verification_message: &str,
        outcome: &AlertActionOutcome,
    ) -> Result<ActionAuditEntry, String> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| "system clock is before the Unix epoch".to_owned())?
            .as_millis() as u64;
        let entry = ActionAuditEntry {
            id: audit_id(),
            timestamp,
            action: action.to_owned(),
            stage: stage.to_owned(),
            confirmed,
            status: status.to_owned(),
            message: message.to_owned(),
            reversible,
            privilege: privilege.to_owned(),
            verification_status: verification_status.to_owned(),
            verification_message: verification_message.to_owned(),
            outcome_status: outcome_status_label(&outcome.status).to_owned(),
            outcome_message: outcome.message.clone(),
        };
        let path = audit_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|error| error.to_string())?;
        let line = serde_json::to_string(&entry).map_err(|error| error.to_string())?;
        writeln!(file, "{line}").map_err(|error| error.to_string())?;
        Ok(entry)
    }

    pub fn history(&self) -> Result<Vec<ActionAuditEntry>, String> {
        let path = audit_path()?;
        let file = match fs::File::open(path) {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => return Err(error.to_string()),
        };
        let mut entries = Vec::new();
        for line in BufReader::new(file).lines() {
            let line = line.map_err(|error| error.to_string())?;
            if line.trim().is_empty() {
                continue;
            }
            entries.push(
                serde_json::from_str::<ActionAuditEntry>(&line)
                    .map_err(|error| format!("invalid audit record: {error}"))?,
            );
        }
        Ok(entries)
    }
}

fn audit_path() -> Result<PathBuf, String> {
    if let Ok(state_home) = std::env::var("XDG_STATE_HOME") {
        return Ok(PathBuf::from(state_home)
            .join("linux-powerhouse")
            .join("action-audit.jsonl"));
    }
    if let Ok(home) = std::env::var("HOME") {
        return Ok(PathBuf::from(home)
            .join(".local")
            .join("state")
            .join("linux-powerhouse")
            .join("action-audit.jsonl"));
    }
    Err("unable to determine a local state directory for the action audit".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audit_ids_are_unique_and_use_the_action_prefix() {
        let first = audit_id();
        let second = audit_id();

        assert_ne!(first, second);
        assert!(first.starts_with("action-"));
        assert!(second.starts_with("action-"));
    }

    #[test]
    fn outcome_status_uses_explicit_stable_labels() {
        assert_eq!(
            outcome_status_label(&AlertActionOutcomeStatus::Verified),
            "verified"
        );
        assert_eq!(
            outcome_status_label(&AlertActionOutcomeStatus::Rejected),
            "rejected"
        );
    }
}
