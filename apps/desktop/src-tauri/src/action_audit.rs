use health_status::{AlertActionOutcome, AlertActionOutcomeStatus};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
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
    #[serde(default)]
    pub outcome_action: String,
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

fn is_valid_outcome_status(status: &str) -> bool {
    matches!(status, "legacy" | "verified" | "rejected")
}

fn is_valid_verification_status(status: &str) -> bool {
    matches!(status, "legacy" | "verified")
}

fn is_valid_verification_outcome_status(verification_status: &str, outcome_status: &str) -> bool {
    matches!(
        (verification_status, outcome_status),
        ("verified", "verified")
    )
}

fn is_valid_stage_verification_status(stage: &str, verification_status: &str) -> bool {
    matches!((stage, verification_status), ("verified", "verified"))
}

fn is_valid_stage_outcome_status(stage: &str, outcome_status: &str) -> bool {
    matches!((stage, outcome_status), ("verified", "verified"))
}

fn is_valid_stage(stage: &str) -> bool {
    matches!(stage, "verified" | "failed")
}

fn is_valid_status(status: &str) -> bool {
    matches!(status, "success" | "completed" | "failed")
}

fn is_valid_stage_status(stage: &str, status: &str) -> bool {
    matches!(
        (stage, status),
        ("verified", "success" | "completed") | ("failed", "failed")
    )
}

fn is_valid_action(action: &str) -> bool {
    matches!(
        action,
        "refresh_health"
            | "storage_diagnostic"
            | "process_diagnostic"
            | "network_diagnostic"
            | "service_diagnostic"
    )
}

fn is_valid_privilege(privilege: &str) -> bool {
    matches!(privilege, "none" | "None" | "Unknown")
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
            outcome_action: outcome.action_id.clone(),
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
        parse_audit_entries(BufReader::new(file))
    }
}

fn parse_audit_entries<R: BufRead>(reader: R) -> Result<Vec<ActionAuditEntry>, String> {
    let mut entries = Vec::new();
    let mut seen_ids = HashSet::new();

    for line in reader.lines() {
        let line = line.map_err(|error| error.to_string())?;
        if line.trim().is_empty() {
            continue;
        }

        if let Ok(entry) = serde_json::from_str::<ActionAuditEntry>(&line) {
            if entry.timestamp > 0
                && !entry.id.trim().is_empty()
                && !entry.action.trim().is_empty()
                && (entry.verification_status == "legacy" || is_valid_action(&entry.action))
                && (entry.verification_status == "legacy" || is_valid_stage(&entry.stage))
                && (entry.verification_status == "legacy"
                    || is_valid_stage_status(&entry.stage, &entry.status))
                && (entry.verification_status == "legacy" || entry.confirmed)
                && is_valid_status(&entry.status)
                && !entry.message.trim().is_empty()
                && is_valid_privilege(&entry.privilege)
                && is_valid_verification_status(&entry.verification_status)
                && (entry.verification_status == "legacy"
                    || is_valid_stage_verification_status(&entry.stage, &entry.verification_status))
                && (entry.verification_status == "legacy"
                    || !entry.verification_message.trim().is_empty())
                && is_valid_outcome_status(&entry.outcome_status)
                && (entry.verification_status == "legacy"
                    || entry.outcome_status == "legacy"
                    || is_valid_stage_outcome_status(&entry.stage, &entry.outcome_status))
                && (entry.verification_status == "legacy"
                    || entry.outcome_status == "legacy"
                    || is_valid_verification_outcome_status(
                        &entry.verification_status,
                        &entry.outcome_status,
                    ))
                && (entry.outcome_status == "legacy" || !entry.outcome_message.trim().is_empty())
                && (entry.outcome_status == "legacy"
                    || (!entry.outcome_action.trim().is_empty()
                        && entry.action == entry.outcome_action))
                && seen_ids.insert(entry.id.clone())
            {
                entries.push(entry);
            }
        }
    }

    Ok(entries)
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
    use std::io::Cursor;

    fn test_audit_entry(id: &str) -> ActionAuditEntry {
        ActionAuditEntry {
            id: id.to_owned(),
            timestamp: 123,
            action: "refresh_health".to_owned(),
            stage: "verified".to_owned(),
            confirmed: true,
            status: "success".to_owned(),
            message: "test message".to_owned(),
            reversible: true,
            privilege: "none".to_owned(),
            verification_status: "verified".to_owned(),
            verification_message: "verified".to_owned(),
            outcome_status: "verified".to_owned(),
            outcome_message: "outcome verified".to_owned(),
            outcome_action: "refresh_health".to_owned(),
        }
    }

    #[test]
    fn malformed_audit_records_do_not_hide_valid_history() {
        let first = serde_json::to_string(&test_audit_entry("first")).unwrap();
        let second = serde_json::to_string(&test_audit_entry("second")).unwrap();
        let input = format!("{first}\nnot valid json\n{second}\n");

        let entries = parse_audit_entries(Cursor::new(input)).unwrap();

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].id, "first");
        assert_eq!(entries[1].id, "second");
    }

    #[test]
    fn empty_audit_lines_are_ignored() {
        let entry = serde_json::to_string(&test_audit_entry("first")).unwrap();
        let input = format!("\n{entry}\n\n");

        let entries = parse_audit_entries(Cursor::new(input)).unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, "first");
    }

    #[test]
    fn duplicate_audit_ids_do_not_hide_unique_history() {
        let first = serde_json::to_string(&test_audit_entry("first")).unwrap();
        let duplicate = serde_json::to_string(&test_audit_entry("first")).unwrap();
        let second = serde_json::to_string(&test_audit_entry("second")).unwrap();
        let input = format!("{first}\n{duplicate}\n{second}\n");

        let entries = parse_audit_entries(Cursor::new(input)).unwrap();

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].id, "first");
        assert_eq!(entries[1].id, "second");
    }

    #[test]
    fn empty_audit_ids_are_ignored_without_hiding_valid_history() {
        let empty = serde_json::to_string(&test_audit_entry("")).unwrap();
        let whitespace = serde_json::to_string(&test_audit_entry("   ")).unwrap();
        let valid = serde_json::to_string(&test_audit_entry("valid")).unwrap();
        let input = format!("{empty}\n{whitespace}\n{valid}\n");

        let entries = parse_audit_entries(Cursor::new(input)).unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, "valid");
    }

    #[test]
    fn zero_audit_timestamps_are_ignored_without_hiding_valid_history() {
        let mut invalid_entry = test_audit_entry("invalid");
        invalid_entry.timestamp = 0;
        let invalid = serde_json::to_string(&invalid_entry).unwrap();
        let valid = serde_json::to_string(&test_audit_entry("valid")).unwrap();
        let input = format!("{invalid}\n{valid}\n");

        let entries = parse_audit_entries(Cursor::new(input)).unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, "valid");
    }

    #[test]
    fn blank_audit_actions_are_ignored_without_hiding_valid_history() {
        let mut empty_entry = test_audit_entry("empty-action");
        empty_entry.action = "".to_owned();
        let empty = serde_json::to_string(&empty_entry).unwrap();

        let mut whitespace_entry = test_audit_entry("whitespace-action");
        whitespace_entry.action = "   ".to_owned();
        let whitespace = serde_json::to_string(&whitespace_entry).unwrap();

        let valid = serde_json::to_string(&test_audit_entry("valid")).unwrap();
        let input = format!("{empty}\n{whitespace}\n{valid}\n");

        let entries = parse_audit_entries(Cursor::new(input)).unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, "valid");
    }

    #[test]
    fn unknown_audit_actions_are_ignored_without_hiding_valid_history() {
        let mut unknown_entry = test_audit_entry("unknown-action");
        unknown_entry.action = "unknown_action".to_owned();
        let unknown = serde_json::to_string(&unknown_entry).unwrap();

        let actions = [
            "refresh_health",
            "storage_diagnostic",
            "process_diagnostic",
            "network_diagnostic",
            "service_diagnostic",
        ];

        let valid = actions
            .iter()
            .enumerate()
            .map(|(index, action)| {
                let mut entry = test_audit_entry(&format!("valid-action-{index}"));
                entry.action = (*action).to_owned();
                entry.outcome_action = (*action).to_owned();
                serde_json::to_string(&entry).unwrap()
            })
            .collect::<Vec<_>>();

        let input = format!("{unknown}\n{}\n", valid.join("\n"));

        let entries = parse_audit_entries(Cursor::new(input)).unwrap();

        assert_eq!(entries.len(), actions.len());
        for (entry, action) in entries.iter().zip(actions) {
            assert_eq!(entry.action, action);
        }
    }

    #[test]
    fn blank_audit_stages_are_ignored_without_hiding_valid_history() {
        let mut empty_entry = test_audit_entry("empty-stage");
        empty_entry.stage = "".to_owned();
        let empty = serde_json::to_string(&empty_entry).unwrap();

        let mut whitespace_entry = test_audit_entry("whitespace-stage");
        whitespace_entry.stage = "   ".to_owned();
        let whitespace = serde_json::to_string(&whitespace_entry).unwrap();

        let valid = serde_json::to_string(&test_audit_entry("valid")).unwrap();
        let input = format!("{empty}\n{whitespace}\n{valid}\n");

        let entries = parse_audit_entries(Cursor::new(input)).unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, "valid");
    }

    #[test]
    fn unknown_audit_stages_are_ignored_without_hiding_valid_history() {
        let mut unknown_entry = test_audit_entry("unknown-stage");
        unknown_entry.stage = "unknown".to_owned();
        let unknown = serde_json::to_string(&unknown_entry).unwrap();

        let mut verified_entry = test_audit_entry("verified");
        verified_entry.stage = "verified".to_owned();
        let verified = serde_json::to_string(&verified_entry).unwrap();

        let mut failed_entry = test_audit_entry("failed");
        failed_entry.stage = "failed".to_owned();
        failed_entry.status = "failed".to_owned();
        failed_entry.verification_status = "legacy".to_owned();
        let failed = serde_json::to_string(&failed_entry).unwrap();

        let input = format!("{unknown}\n{verified}\n{failed}\n");

        let entries = parse_audit_entries(Cursor::new(input)).unwrap();

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].id, "verified");
        assert_eq!(entries[1].id, "failed");
    }

    #[test]
    fn unconfirmed_audit_records_are_ignored_without_hiding_valid_history() {
        let mut unconfirmed_entry = test_audit_entry("unconfirmed");
        unconfirmed_entry.confirmed = false;
        let unconfirmed = serde_json::to_string(&unconfirmed_entry).unwrap();

        let valid = serde_json::to_string(&test_audit_entry("valid")).unwrap();
        let input = format!("{unconfirmed}\n{valid}\n");

        let entries = parse_audit_entries(Cursor::new(input)).unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, "valid");
    }

    #[test]
    fn blank_audit_statuses_are_ignored_without_hiding_valid_history() {
        let mut empty_entry = test_audit_entry("empty-status");
        empty_entry.status = "".to_owned();
        let empty = serde_json::to_string(&empty_entry).unwrap();

        let mut whitespace_entry = test_audit_entry("whitespace-status");
        whitespace_entry.status = "   ".to_owned();
        let whitespace = serde_json::to_string(&whitespace_entry).unwrap();

        let valid = serde_json::to_string(&test_audit_entry("valid")).unwrap();
        let input = format!("{empty}\n{whitespace}\n{valid}\n");

        let entries = parse_audit_entries(Cursor::new(input)).unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, "valid");
    }

    #[test]
    fn inconsistent_audit_stage_statuses_are_ignored_without_hiding_valid_history() {
        let mut verified_failed_entry = test_audit_entry("verified-failed");
        verified_failed_entry.stage = "verified".to_owned();
        verified_failed_entry.status = "failed".to_owned();
        let verified_failed = serde_json::to_string(&verified_failed_entry).unwrap();

        let mut failed_success_entry = test_audit_entry("failed-success");
        failed_success_entry.stage = "failed".to_owned();
        failed_success_entry.status = "success".to_owned();
        let failed_success = serde_json::to_string(&failed_success_entry).unwrap();

        let mut failed_completed_entry = test_audit_entry("failed-completed");
        failed_completed_entry.stage = "failed".to_owned();
        failed_completed_entry.status = "completed".to_owned();
        let failed_completed = serde_json::to_string(&failed_completed_entry).unwrap();

        let mut verified_success_entry = test_audit_entry("verified-success");
        verified_success_entry.stage = "verified".to_owned();
        verified_success_entry.status = "success".to_owned();
        let verified_success = serde_json::to_string(&verified_success_entry).unwrap();

        let valid = serde_json::to_string(&test_audit_entry("verified-completed")).unwrap();

        let mut failed_failed_entry = test_audit_entry("failed-failed");
        failed_failed_entry.stage = "failed".to_owned();
        failed_failed_entry.status = "failed".to_owned();
        failed_failed_entry.verification_status = "legacy".to_owned();
        let failed_failed = serde_json::to_string(&failed_failed_entry).unwrap();

        let input = format!(
            "{verified_failed}\n\
             {failed_success}\n\
             {failed_completed}\n\
             {verified_success}\n\
             {valid}\n\
             {failed_failed}\n"
        );

        let entries = parse_audit_entries(Cursor::new(input)).unwrap();

        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].id, "verified-success");
        assert_eq!(entries[1].id, "verified-completed");
        assert_eq!(entries[2].id, "failed-failed");
    }

    #[test]
    fn unknown_audit_statuses_are_ignored_without_hiding_valid_history() {
        let mut unknown_entry = test_audit_entry("unknown-status");
        unknown_entry.status = "unknown".to_owned();
        let unknown = serde_json::to_string(&unknown_entry).unwrap();

        let mut legacy_entry = test_audit_entry("legacy-status");
        legacy_entry.status = "success".to_owned();
        let legacy = serde_json::to_string(&legacy_entry).unwrap();

        let mut completed_entry = test_audit_entry("completed-status");
        completed_entry.status = "completed".to_owned();
        let completed = serde_json::to_string(&completed_entry).unwrap();

        let mut failed_entry = test_audit_entry("failed-status");
        failed_entry.stage = "failed".to_owned();
        failed_entry.status = "failed".to_owned();
        failed_entry.verification_status = "legacy".to_owned();
        let failed = serde_json::to_string(&failed_entry).unwrap();

        let input = format!("{unknown}\n{legacy}\n{completed}\n{failed}\n");

        let entries = parse_audit_entries(Cursor::new(input)).unwrap();

        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].id, "legacy-status");
        assert_eq!(entries[1].id, "completed-status");
        assert_eq!(entries[2].id, "failed-status");
    }

    #[test]
    fn blank_audit_messages_are_ignored_without_hiding_valid_history() {
        let mut empty_entry = test_audit_entry("empty-message");
        empty_entry.message = "".to_owned();
        let empty = serde_json::to_string(&empty_entry).unwrap();

        let mut whitespace_entry = test_audit_entry("whitespace-message");
        whitespace_entry.message = "   ".to_owned();
        let whitespace = serde_json::to_string(&whitespace_entry).unwrap();

        let valid = serde_json::to_string(&test_audit_entry("valid")).unwrap();
        let input = format!("{empty}\n{whitespace}\n{valid}\n");

        let entries = parse_audit_entries(Cursor::new(input)).unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, "valid");
    }

    #[test]
    fn blank_audit_privileges_are_ignored_without_hiding_valid_history() {
        let mut empty_entry = test_audit_entry("empty-privilege");
        empty_entry.privilege = "".to_owned();
        let empty = serde_json::to_string(&empty_entry).unwrap();

        let mut whitespace_entry = test_audit_entry("whitespace-privilege");
        whitespace_entry.privilege = "   ".to_owned();
        let whitespace = serde_json::to_string(&whitespace_entry).unwrap();

        let valid = serde_json::to_string(&test_audit_entry("valid")).unwrap();
        let input = format!("{empty}\n{whitespace}\n{valid}\n");

        let entries = parse_audit_entries(Cursor::new(input)).unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, "valid");
    }

    #[test]
    fn unknown_audit_privileges_are_ignored_without_hiding_valid_history() {
        let mut unknown_entry = test_audit_entry("unknown-privilege");
        unknown_entry.privilege = "admin".to_owned();
        let unknown = serde_json::to_string(&unknown_entry).unwrap();

        let mut current_entry = test_audit_entry("current-privilege");
        current_entry.privilege = "None".to_owned();
        let current = serde_json::to_string(&current_entry).unwrap();

        let mut failed_entry = test_audit_entry("unknown-privilege");
        failed_entry.id = "failed-privilege".to_owned();
        failed_entry.privilege = "Unknown".to_owned();
        let failed = serde_json::to_string(&failed_entry).unwrap();

        let input = format!("{unknown}\n{current}\n{failed}\n");

        let entries = parse_audit_entries(Cursor::new(input)).unwrap();

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].id, "current-privilege");
        assert_eq!(entries[1].id, "failed-privilege");
    }

    #[test]
    fn blank_audit_verification_statuses_are_ignored_without_hiding_valid_history() {
        let mut empty_entry = test_audit_entry("empty-verification-status");
        empty_entry.verification_status = "".to_owned();
        let empty = serde_json::to_string(&empty_entry).unwrap();

        let mut whitespace_entry = test_audit_entry("whitespace-verification-status");
        whitespace_entry.verification_status = "   ".to_owned();
        let whitespace = serde_json::to_string(&whitespace_entry).unwrap();

        let valid = serde_json::to_string(&test_audit_entry("valid")).unwrap();
        let input = format!("{empty}\n{whitespace}\n{valid}\n");

        let entries = parse_audit_entries(Cursor::new(input)).unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, "valid");
    }

    #[test]
    fn blank_audit_verification_messages_are_ignored_without_hiding_valid_history() {
        let mut empty_entry = test_audit_entry("empty-verification-message");
        empty_entry.verification_message = "".to_owned();
        let empty = serde_json::to_string(&empty_entry).unwrap();

        let mut whitespace_entry = test_audit_entry("whitespace-verification-message");
        whitespace_entry.verification_message = "   ".to_owned();
        let whitespace = serde_json::to_string(&whitespace_entry).unwrap();

        let valid = serde_json::to_string(&test_audit_entry("valid")).unwrap();
        let input = format!("{empty}\n{whitespace}\n{valid}\n");

        let entries = parse_audit_entries(Cursor::new(input)).unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, "valid");
    }

    #[test]
    fn blank_audit_outcome_statuses_are_ignored_without_hiding_valid_history() {
        let mut empty_entry = test_audit_entry("empty-outcome-status");
        empty_entry.outcome_status = "".to_owned();
        let empty = serde_json::to_string(&empty_entry).unwrap();

        let mut whitespace_entry = test_audit_entry("whitespace-outcome-status");
        whitespace_entry.outcome_status = "   ".to_owned();
        let whitespace = serde_json::to_string(&whitespace_entry).unwrap();

        let valid = serde_json::to_string(&test_audit_entry("valid")).unwrap();
        let input = format!("{empty}\n{whitespace}\n{valid}\n");

        let entries = parse_audit_entries(Cursor::new(input)).unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, "valid");
    }

    #[test]
    fn inconsistent_audit_stage_verification_statuses_are_ignored_without_hiding_valid_history() {
        let mut failed_verified_entry = test_audit_entry("failed-verified");
        failed_verified_entry.stage = "failed".to_owned();
        failed_verified_entry.status = "failed".to_owned();
        failed_verified_entry.verification_status = "verified".to_owned();
        let failed_verified = serde_json::to_string(&failed_verified_entry).unwrap();

        let verified_verified =
            serde_json::to_string(&test_audit_entry("verified-verified")).unwrap();

        let input = format!("{failed_verified}\n{verified_verified}\n");

        let entries = parse_audit_entries(Cursor::new(input)).unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, "verified-verified");
    }

    #[test]
    fn inconsistent_audit_stage_outcome_statuses_are_ignored_without_hiding_valid_history() {
        let mut failed_verified_entry = test_audit_entry("failed-verified");
        failed_verified_entry.stage = "failed".to_owned();
        failed_verified_entry.status = "failed".to_owned();
        failed_verified_entry.outcome_status = "verified".to_owned();
        let failed_verified = serde_json::to_string(&failed_verified_entry).unwrap();

        let verified_verified =
            serde_json::to_string(&test_audit_entry("verified-verified")).unwrap();

        let input = format!("{failed_verified}\n{verified_verified}\n");

        let entries = parse_audit_entries(Cursor::new(input)).unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, "verified-verified");
    }

    #[test]
    fn inconsistent_audit_verification_outcome_statuses_are_ignored_without_hiding_valid_history() {
        let mut verified_rejected_entry = test_audit_entry("verified-rejected");
        verified_rejected_entry.verification_status = "verified".to_owned();
        verified_rejected_entry.outcome_status = "rejected".to_owned();
        let verified_rejected = serde_json::to_string(&verified_rejected_entry).unwrap();

        let verified_verified =
            serde_json::to_string(&test_audit_entry("verified-verified")).unwrap();

        let input = format!("{verified_rejected}\n{verified_verified}\n");

        let entries = parse_audit_entries(Cursor::new(input)).unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, "verified-verified");
    }

    #[test]
    fn unknown_audit_verification_statuses_are_ignored_without_hiding_valid_history() {
        let mut unknown_entry = test_audit_entry("unknown-verification-status");
        unknown_entry.verification_status = "unknown".to_owned();
        let unknown = serde_json::to_string(&unknown_entry).unwrap();

        let verified = serde_json::to_string(&test_audit_entry("verified")).unwrap();

        let input = format!("{unknown}\n{verified}\n");

        let entries = parse_audit_entries(Cursor::new(input)).unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, "verified");
    }

    #[test]
    fn unknown_audit_outcome_statuses_are_ignored_without_hiding_valid_history() {
        let mut unknown_entry = test_audit_entry("unknown-outcome-status");
        unknown_entry.outcome_status = "unknown".to_owned();
        let unknown = serde_json::to_string(&unknown_entry).unwrap();

        let verified = serde_json::to_string(&test_audit_entry("verified")).unwrap();

        let mut rejected_entry = test_audit_entry("rejected");
        rejected_entry.verification_status = "legacy".to_owned();
        rejected_entry.outcome_status = "rejected".to_owned();
        let rejected = serde_json::to_string(&rejected_entry).unwrap();

        let input = format!("{unknown}\n{verified}\n{rejected}\n");

        let entries = parse_audit_entries(Cursor::new(input)).unwrap();

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].id, "verified");
        assert_eq!(entries[1].id, "rejected");
    }

    #[test]
    fn inconsistent_audit_outcome_actions_are_ignored_without_hiding_valid_history() {
        let mut inconsistent_entry = test_audit_entry("inconsistent-outcome-action");
        inconsistent_entry.outcome_action = "storage_diagnostic".to_owned();
        let inconsistent = serde_json::to_string(&inconsistent_entry).unwrap();

        let valid = serde_json::to_string(&test_audit_entry("valid")).unwrap();
        let input = format!("{inconsistent}\n{valid}\n");

        let entries = parse_audit_entries(Cursor::new(input)).unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, "valid");
    }

    #[test]
    fn blank_audit_outcome_actions_are_ignored_without_hiding_valid_history() {
        let mut blank_entry = test_audit_entry("blank-outcome-action");
        blank_entry.outcome_action = "".to_owned();
        let blank = serde_json::to_string(&blank_entry).unwrap();

        let mut whitespace_entry = test_audit_entry("whitespace-outcome-action");
        whitespace_entry.outcome_action = "   ".to_owned();
        let whitespace = serde_json::to_string(&whitespace_entry).unwrap();

        let valid = serde_json::to_string(&test_audit_entry("valid")).unwrap();
        let input = format!("{blank}\n{whitespace}\n{valid}\n");

        let entries = parse_audit_entries(Cursor::new(input)).unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, "valid");
    }

    #[test]
    fn blank_audit_outcome_messages_are_ignored_without_hiding_valid_history() {
        let mut empty_entry = test_audit_entry("empty-outcome-message");
        empty_entry.outcome_message = "".to_owned();
        let empty = serde_json::to_string(&empty_entry).unwrap();

        let mut whitespace_entry = test_audit_entry("whitespace-outcome-message");
        whitespace_entry.outcome_message = "   ".to_owned();
        let whitespace = serde_json::to_string(&whitespace_entry).unwrap();

        let valid = serde_json::to_string(&test_audit_entry("valid")).unwrap();
        let input = format!("{empty}\n{whitespace}\n{valid}\n");

        let entries = parse_audit_entries(Cursor::new(input)).unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, "valid");
    }

    #[test]
    fn legacy_audit_records_remain_visible_when_newer_fields_are_missing() {
        let input = r#"{"id":"legacy","timestamp":123,"action":"test_action","stage":"test_stage","confirmed":true,"status":"success","message":"test message","reversible":true,"privilege":"none"}"#;

        let entries = parse_audit_entries(Cursor::new(format!("{input}\n"))).unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, "legacy");
        assert_eq!(entries[0].verification_status, "legacy");
        assert_eq!(entries[0].verification_message, "");
        assert_eq!(entries[0].outcome_status, "legacy");
        assert_eq!(entries[0].outcome_message, "");
        assert_eq!(entries[0].outcome_action, "");
    }

    #[test]
    fn audit_ids_are_unique_and_use_the_action_prefix() {
        let first = audit_id();
        let second = audit_id();

        assert_ne!(first, second);
        assert!(first.starts_with("action-"));
        assert!(second.starts_with("action-"));
    }

    #[test]
    fn recorded_audit_entries_round_trip_through_persistence_format() {
        let entry = test_audit_entry("round-trip");
        let line = serde_json::to_string(&entry).unwrap();
        let root = std::env::temp_dir().join(format!("linux-powerhouse-audit-{}", Uuid::new_v4()));
        let file = root.join("action-audit.jsonl");

        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(&file, format!("{line}\n")).unwrap();

        let contents = std::fs::File::open(&file).unwrap();
        let history = parse_audit_entries(std::io::BufReader::new(contents)).unwrap();

        assert_eq!(history, vec![entry]);

        let _ = std::fs::remove_dir_all(root);
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
