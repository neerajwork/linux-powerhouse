use health_status::{AlertActionOutcome, AlertActionOutcomeStatus, AlertActionVerificationStatus};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
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

fn is_valid_outcome_message(outcome_status: &str, outcome_message: &str) -> bool {
    matches!(
        (outcome_status, outcome_message),
        (
            "verified",
            "action execution and verification produced a verified outcome."
        ) | (
            "rejected",
            "action outcome rejected because execution and verification evidence did not establish a verified result."
        )
    )
}

fn is_valid_outcome_evidence(action: &str, outcome: &AlertActionOutcome) -> bool {
    let action_matches = outcome.action_id == action
        && outcome.execution.action_id == outcome.action_id
        && outcome.verification.action_id == outcome.action_id;

    action_matches
        && !outcome.message.trim().is_empty()
        && is_valid_outcome_message(outcome_status_label(&outcome.status), &outcome.message)
        && match outcome.status {
            AlertActionOutcomeStatus::Verified => {
                outcome.execution.executed
                    && outcome.verification.status == AlertActionVerificationStatus::Passed
            }
            AlertActionOutcomeStatus::Rejected => {
                outcome.verification.status == AlertActionVerificationStatus::Failed
            }
        }
}

fn is_valid_verification_evidence(
    action: &str,
    verification_status: &str,
    verification_message: &str,
    outcome: &AlertActionOutcome,
) -> bool {
    outcome.verification.action_id == action
        && !outcome.verification.message.trim().is_empty()
        && outcome.verification.message == verification_message
        && matches!(
            (verification_status, &outcome.verification.status),
            ("verified", AlertActionVerificationStatus::Passed)
                | ("failed", AlertActionVerificationStatus::Failed)
        )
}

fn is_valid_verification_status(status: &str) -> bool {
    matches!(status, "legacy" | "verified" | "failed")
}

fn is_valid_verification_outcome_status(verification_status: &str, outcome_status: &str) -> bool {
    matches!(
        (verification_status, outcome_status),
        ("verified", "verified") | ("failed", "rejected")
    )
}

fn is_valid_stage_verification_status(stage: &str, verification_status: &str) -> bool {
    matches!(
        (stage, verification_status),
        ("verified", "verified") | ("failed", "failed")
    )
}

fn is_valid_stage_outcome_status(stage: &str, outcome_status: &str) -> bool {
    matches!(
        (stage, outcome_status),
        ("verified", "verified") | ("failed", "rejected")
    )
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

fn is_valid_stage_confirmation(stage: &str, confirmed: bool) -> bool {
    matches!((stage, confirmed), ("verified", true) | ("failed", true))
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

fn is_valid_stage_privilege(stage: &str, privilege: &str) -> bool {
    matches!(
        (stage, privilege),
        ("verified", "none" | "None") | ("failed", "Unknown")
    )
}

fn is_valid_stage_reversibility(stage: &str, reversible: bool) -> bool {
    matches!((stage, reversible), ("verified", true) | ("failed", false))
}

fn is_valid_complete_lifecycle(entry: &ActionAuditEntry) -> bool {
    entry.action == entry.outcome_action
        && is_valid_stage_status(&entry.stage, &entry.status)
        && is_valid_stage_confirmation(&entry.stage, entry.confirmed)
        && is_valid_stage_privilege(&entry.stage, &entry.privilege)
        && is_valid_stage_reversibility(&entry.stage, entry.reversible)
        && is_valid_stage_verification_status(&entry.stage, &entry.verification_status)
        && is_valid_stage_outcome_status(&entry.stage, &entry.outcome_status)
        && is_valid_verification_outcome_status(&entry.verification_status, &entry.outcome_status)
}

pub struct ActionAuditRecord<'a> {
    pub action: &'a str,
    pub stage: &'a str,
    pub confirmed: bool,
    pub status: &'a str,
    pub message: &'a str,
    pub reversible: bool,
    pub privilege: &'a str,
    pub verification_status: &'a str,
    pub verification_message: &'a str,
    pub outcome: &'a AlertActionOutcome,
}

#[derive(Clone, Default)]
pub struct ActionAudit;

fn record_audit_entry(
    record: &ActionAuditRecord<'_>,
    path: &Path,
) -> Result<ActionAuditEntry, String> {
    if !is_valid_outcome_evidence(record.action, record.outcome) {
        return Err("invalid action audit outcome evidence".to_owned());
    }

    if !is_valid_verification_evidence(
        record.action,
        record.verification_status,
        record.verification_message,
        record.outcome,
    ) {
        return Err("invalid action audit verification evidence".to_owned());
    }

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "system clock is before the Unix epoch".to_owned())?
        .as_millis() as u64;

    let entry = ActionAuditEntry {
        id: audit_id(),
        timestamp,
        action: record.action.to_owned(),
        stage: record.stage.to_owned(),
        confirmed: record.confirmed,
        status: record.status.to_owned(),
        message: record.message.to_owned(),
        reversible: record.reversible,
        privilege: record.privilege.to_owned(),
        verification_status: record.verification_status.to_owned(),
        verification_message: record.verification_message.to_owned(),
        outcome_status: outcome_status_label(&record.outcome.status).to_owned(),
        outcome_message: record.outcome.message.clone(),
        outcome_action: record.outcome.action_id.clone(),
    };

    if !is_valid_audit_entry(&entry) {
        return Err("invalid action audit entry".to_owned());
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|error| error.to_string())?;

    let line = serde_json::to_string(&entry).map_err(|error| error.to_string())?;
    writeln!(file, "{line}").map_err(|error| error.to_string())?;

    Ok(entry)
}

fn read_audit_history(path: &Path) -> Result<Vec<ActionAuditEntry>, String> {
    let file = match fs::File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.to_string()),
    };
    parse_audit_entries(BufReader::new(file))
}

impl ActionAudit {
    pub fn record(&self, record: &ActionAuditRecord<'_>) -> Result<ActionAuditEntry, String> {
        let path = audit_path()?;
        record_audit_entry(record, &path)
    }

    pub fn history(&self) -> Result<Vec<ActionAuditEntry>, String> {
        let path = audit_path()?;
        read_audit_history(&path)
    }
}

fn is_valid_audit_entry(entry: &ActionAuditEntry) -> bool {
    entry.timestamp > 0
        && !entry.id.trim().is_empty()
        && !entry.action.trim().is_empty()
        && (entry.verification_status == "legacy" || is_valid_action(&entry.action))
        && (entry.verification_status == "legacy" || is_valid_stage(&entry.stage))
        && (entry.verification_status == "legacy" || is_valid_complete_lifecycle(entry))
        && is_valid_status(&entry.status)
        && !entry.message.trim().is_empty()
        && is_valid_privilege(&entry.privilege)
        && is_valid_verification_status(&entry.verification_status)
        && (entry.verification_status == "legacy" || !entry.verification_message.trim().is_empty())
        && is_valid_outcome_status(&entry.outcome_status)
        && (entry.verification_status != "legacy" || entry.outcome_status == "legacy")
        && (entry.outcome_status == "legacy"
            || is_valid_outcome_message(&entry.outcome_status, &entry.outcome_message))
        && (entry.outcome_status == "legacy"
            || (!entry.outcome_action.trim().is_empty() && entry.action == entry.outcome_action))
}

fn parse_audit_entries<R: BufRead>(reader: R) -> Result<Vec<ActionAuditEntry>, String> {
    let mut entries = Vec::new();
    let mut seen_ids = HashSet::new();

    for line in reader.lines() {
        let line = line.map_err(|error| error.to_string())?;
        if line.trim().is_empty() {
            continue;
        }

        if let Ok(entry) = serde_json::from_str::<ActionAuditEntry>(&line)
            && is_valid_audit_entry(&entry)
            && seen_ids.insert(entry.id.clone())
        {
            entries.push(entry);
        }
    }

    Ok(entries)
}

fn audit_path_from_environment(
    state_home: Option<&str>,
    home: Option<&str>,
) -> Result<PathBuf, String> {
    if let Some(state_home) = state_home {
        return Ok(PathBuf::from(state_home)
            .join("linux-powerhouse")
            .join("action-audit.jsonl"));
    }

    if let Some(home) = home {
        return Ok(PathBuf::from(home)
            .join(".local")
            .join("state")
            .join("linux-powerhouse")
            .join("action-audit.jsonl"));
    }

    Err("unable to determine a local state directory for the action audit".to_owned())
}

fn audit_path() -> Result<PathBuf, String> {
    audit_path_from_environment(
        std::env::var("XDG_STATE_HOME").ok().as_deref(),
        std::env::var("HOME").ok().as_deref(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn audit_path_prefers_xdg_state_home() {
        let path = audit_path_from_environment(Some("/tmp/state"), Some("/tmp/home")).unwrap();

        assert_eq!(
            path,
            PathBuf::from("/tmp/state")
                .join("linux-powerhouse")
                .join("action-audit.jsonl")
        );
    }

    #[test]
    fn audit_path_falls_back_to_home() {
        let path = audit_path_from_environment(None, Some("/tmp/home")).unwrap();

        assert_eq!(
            path,
            PathBuf::from("/tmp/home")
                .join(".local")
                .join("state")
                .join("linux-powerhouse")
                .join("action-audit.jsonl")
        );
    }

    #[test]
    fn audit_path_requires_a_local_state_directory() {
        let error = audit_path_from_environment(None, None).unwrap_err();

        assert_eq!(
            error,
            "unable to determine a local state directory for the action audit"
        );
    }

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
            outcome_message: "action execution and verification produced a verified outcome."
                .to_owned(),
            outcome_action: "refresh_health".to_owned(),
        }
    }

    fn test_verified_outcome() -> AlertActionOutcome {
        AlertActionOutcome {
            action_id: "refresh_health".to_owned(),
            execution: health_status::AlertActionExecutionResult {
                action_id: "refresh_health".to_owned(),
                executed: true,
                message: "execution completed".to_owned(),
            },
            verification: health_status::AlertActionVerificationResult {
                action_id: "refresh_health".to_owned(),
                status: AlertActionVerificationStatus::Passed,
                message: "verified".to_owned(),
            },
            status: AlertActionOutcomeStatus::Verified,
            message: "action execution and verification produced a verified outcome.".to_owned(),
        }
    }

    fn test_rejected_outcome() -> AlertActionOutcome {
        AlertActionOutcome {
            action_id: "refresh_health".to_owned(),
            execution: health_status::AlertActionExecutionResult {
                action_id: "refresh_health".to_owned(),
                executed: false,
                message: "execution failed".to_owned(),
            },
            verification: health_status::AlertActionVerificationResult {
                action_id: "refresh_health".to_owned(),
                status: AlertActionVerificationStatus::Failed,
                message: "verification failed".to_owned(),
            },
            status: AlertActionOutcomeStatus::Rejected,
            message: "action outcome rejected because execution and verification evidence did not establish a verified result.".to_owned(),
        }
    }

    #[test]
    fn outcome_evidence_validation_accepts_verified_and_rejected_evidence() {
        assert!(is_valid_outcome_evidence(
            "refresh_health",
            &test_verified_outcome()
        ));
        assert!(is_valid_outcome_evidence(
            "refresh_health",
            &test_rejected_outcome()
        ));
    }

    #[test]
    fn outcome_evidence_validation_rejects_mismatched_actions() {
        let mut outcome = test_verified_outcome();
        outcome.execution.action_id = "storage_diagnostic".to_owned();

        assert!(!is_valid_outcome_evidence("refresh_health", &outcome));
    }

    #[test]
    fn outcome_evidence_validation_rejects_blank_message() {
        let mut outcome = test_verified_outcome();
        outcome.message = "   ".to_owned();

        assert!(!is_valid_outcome_evidence("refresh_health", &outcome));
    }

    #[test]
    fn outcome_evidence_validation_rejects_non_canonical_message() {
        let mut outcome = test_verified_outcome();
        outcome.message = "arbitrary outcome message".to_owned();

        assert!(!is_valid_outcome_evidence("refresh_health", &outcome));
    }

    #[test]
    fn outcome_evidence_validation_rejects_verified_status_without_verified_evidence() {
        let mut outcome = test_verified_outcome();
        outcome.execution.executed = false;

        assert!(!is_valid_outcome_evidence("refresh_health", &outcome));
    }

    #[test]
    fn outcome_evidence_validation_rejects_rejected_status_with_verified_evidence() {
        let mut outcome = test_verified_outcome();
        outcome.status = AlertActionOutcomeStatus::Rejected;

        assert!(!is_valid_outcome_evidence("refresh_health", &outcome));
    }

    #[test]
    fn outcome_evidence_validation_rejects_non_executed_result_with_passed_verification() {
        let mut outcome = test_verified_outcome();
        outcome.execution.executed = false;
        outcome.status = AlertActionOutcomeStatus::Rejected;

        assert!(!is_valid_outcome_evidence("refresh_health", &outcome));
    }

    #[test]
    fn verification_evidence_validation_accepts_matching_verified_and_failed_evidence() {
        let verified = test_verified_outcome();
        assert!(is_valid_verification_evidence(
            "refresh_health",
            "verified",
            "verified",
            &verified,
        ));

        let rejected = test_rejected_outcome();
        assert!(is_valid_verification_evidence(
            "refresh_health",
            "failed",
            "verification failed",
            &rejected,
        ));
    }

    #[test]
    fn verification_evidence_validation_rejects_mismatched_action() {
        let mut outcome = test_verified_outcome();
        outcome.verification.action_id = "storage_diagnostic".to_owned();

        assert!(!is_valid_verification_evidence(
            "refresh_health",
            "verified",
            "verified",
            &outcome,
        ));
    }

    #[test]
    fn verification_evidence_validation_rejects_mismatched_status() {
        let outcome = test_verified_outcome();

        assert!(!is_valid_verification_evidence(
            "refresh_health",
            "failed",
            "verified",
            &outcome,
        ));
    }

    #[test]
    fn verification_evidence_validation_rejects_mismatched_message() {
        let outcome = test_verified_outcome();

        assert!(!is_valid_verification_evidence(
            "refresh_health",
            "verified",
            "different message",
            &outcome,
        ));
    }

    #[test]
    fn verification_evidence_validation_rejects_blank_message() {
        let mut outcome = test_verified_outcome();
        outcome.verification.message = "   ".to_owned();

        assert!(!is_valid_verification_evidence(
            "refresh_health",
            "verified",
            "   ",
            &outcome,
        ));
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
    fn audit_history_propagates_reader_errors() {
        struct FailingReader;

        impl std::io::Read for FailingReader {
            fn read(&mut self, _buffer: &mut [u8]) -> std::io::Result<usize> {
                Err(std::io::Error::other("simulated audit reader failure"))
            }
        }

        impl std::io::BufRead for FailingReader {
            fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
                Err(std::io::Error::other("simulated audit reader failure"))
            }

            fn consume(&mut self, _amount: usize) {}
        }

        let error = parse_audit_entries(FailingReader).unwrap_err();

        assert_eq!(error, "simulated audit reader failure");
    }

    #[test]
    fn history_propagates_audit_file_read_errors() {
        let root = std::env::temp_dir().join(format!(
            "linux-powerhouse-audit-history-read-error-{}",
            uuid::Uuid::new_v4()
        ));
        let path = root.join("action-audit.jsonl");

        std::fs::create_dir_all(&path).unwrap();

        let error = read_audit_history(&path).unwrap_err();

        assert!(!error.is_empty());

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn duplicate_audit_ids_keep_first_record_without_hiding_unique_history() {
        let first_entry = test_audit_entry("first");
        let mut duplicate_entry = test_audit_entry("first");
        duplicate_entry.message = "different duplicate message".to_owned();
        duplicate_entry.outcome_message = "different duplicate outcome".to_owned();

        let first = serde_json::to_string(&first_entry).unwrap();
        let duplicate = serde_json::to_string(&duplicate_entry).unwrap();
        let second = serde_json::to_string(&test_audit_entry("second")).unwrap();
        let input = format!("{first}\n{duplicate}\n{second}\n");

        let entries = parse_audit_entries(Cursor::new(input)).unwrap();

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0], first_entry);
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
    fn legacy_audit_entries_preserve_historical_stage_values() {
        let mut legacy_entry = test_audit_entry("legacy-historical-stage");
        legacy_entry.stage = "historical_stage".to_owned();
        legacy_entry.status = "failed".to_owned();
        legacy_entry.verification_status = "legacy".to_owned();
        legacy_entry.outcome_status = "legacy".to_owned();
        let legacy = serde_json::to_string(&legacy_entry).unwrap();

        let entries = parse_audit_entries(Cursor::new(format!("{legacy}\n"))).unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, "legacy-historical-stage");
        assert_eq!(entries[0].stage, "historical_stage");
        assert_eq!(entries[0].verification_status, "legacy");
        assert_eq!(entries[0].outcome_status, "legacy");
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
        failed_entry.outcome_status = "legacy".to_owned();
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
        failed_failed_entry.outcome_status = "legacy".to_owned();
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
        failed_entry.outcome_status = "legacy".to_owned();
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

        let mut failed_entry = test_audit_entry("failed-privilege");
        failed_entry.stage = "failed".to_owned();
        failed_entry.status = "failed".to_owned();
        failed_entry.reversible = false;
        failed_entry.privilege = "Unknown".to_owned();
        failed_entry.verification_status = "failed".to_owned();
        failed_entry.verification_message = "verification failed".to_owned();
        failed_entry.outcome_status = "rejected".to_owned();
        failed_entry.outcome_message = "action outcome rejected because execution and verification evidence did not establish a verified result.".to_owned();
        let failed = serde_json::to_string(&failed_entry).unwrap();

        let input = format!("{unknown}\n{current}\n{failed}\n");

        let entries = parse_audit_entries(Cursor::new(input)).unwrap();

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].id, "current-privilege");
        assert_eq!(entries[1].id, "failed-privilege");
    }

    #[test]
    fn inconsistent_audit_stage_privileges_are_ignored_without_hiding_valid_history() {
        let mut verified_unknown_entry = test_audit_entry("verified-unknown");
        verified_unknown_entry.privilege = "Unknown".to_owned();
        let verified_unknown = serde_json::to_string(&verified_unknown_entry).unwrap();

        let mut failed_none_entry = test_audit_entry("failed-none");
        failed_none_entry.stage = "failed".to_owned();
        failed_none_entry.status = "failed".to_owned();
        failed_none_entry.privilege = "None".to_owned();
        failed_none_entry.verification_status = "failed".to_owned();
        failed_none_entry.verification_message = "verification failed".to_owned();
        failed_none_entry.outcome_status = "rejected".to_owned();
        failed_none_entry.outcome_message = "action outcome rejected because execution and verification evidence did not establish a verified result.".to_owned();
        let failed_none = serde_json::to_string(&failed_none_entry).unwrap();

        let valid = serde_json::to_string(&test_audit_entry("valid")).unwrap();
        let input = format!("{verified_unknown}\n{failed_none}\n{valid}\n");

        let entries = parse_audit_entries(Cursor::new(input)).unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, "valid");
    }

    #[test]
    fn inconsistent_audit_stage_reversibility_is_ignored_without_hiding_valid_history() {
        let mut verified_irreversible_entry = test_audit_entry("verified-irreversible");
        verified_irreversible_entry.reversible = false;
        let verified_irreversible = serde_json::to_string(&verified_irreversible_entry).unwrap();

        let mut failed_reversible_entry = test_audit_entry("failed-reversible");
        failed_reversible_entry.stage = "failed".to_owned();
        failed_reversible_entry.status = "failed".to_owned();
        failed_reversible_entry.reversible = true;
        failed_reversible_entry.privilege = "Unknown".to_owned();
        failed_reversible_entry.verification_status = "failed".to_owned();
        failed_reversible_entry.verification_message = "verification failed".to_owned();
        failed_reversible_entry.outcome_status = "rejected".to_owned();
        failed_reversible_entry.outcome_message = "action outcome rejected because execution and verification evidence did not establish a verified result.".to_owned();
        let failed_reversible = serde_json::to_string(&failed_reversible_entry).unwrap();

        let valid = serde_json::to_string(&test_audit_entry("valid")).unwrap();
        let input = format!("{verified_irreversible}\n{failed_reversible}\n{valid}\n");

        let entries = parse_audit_entries(Cursor::new(input)).unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, "valid");
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
    fn failed_audit_records_with_rejected_outcomes_remain_visible() {
        let mut failed_entry = test_audit_entry("failed-rejected");
        failed_entry.stage = "failed".to_owned();
        failed_entry.status = "failed".to_owned();
        failed_entry.reversible = false;
        failed_entry.privilege = "Unknown".to_owned();
        failed_entry.verification_status = "failed".to_owned();
        failed_entry.verification_message = "verification failed".to_owned();
        failed_entry.outcome_status = "rejected".to_owned();
        failed_entry.outcome_message = "action outcome rejected because execution and verification evidence did not establish a verified result.".to_owned();

        let failed = serde_json::to_string(&failed_entry).unwrap();
        let verified = serde_json::to_string(&test_audit_entry("verified")).unwrap();
        let input = format!("{failed}\n{verified}\n");

        let entries = parse_audit_entries(Cursor::new(input)).unwrap();

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].id, "failed-rejected");
        assert_eq!(entries[1].id, "verified");
    }

    #[test]
    fn inconsistent_verified_stage_failed_outcome_path_is_ignored_without_hiding_valid_history() {
        let mut inconsistent_entry = test_audit_entry("verified-failed-rejected");
        inconsistent_entry.verification_status = "failed".to_owned();
        inconsistent_entry.verification_message = "verification failed".to_owned();
        inconsistent_entry.outcome_status = "rejected".to_owned();
        inconsistent_entry.outcome_message = "action outcome rejected because execution and verification evidence did not establish a verified result.".to_owned();
        let inconsistent = serde_json::to_string(&inconsistent_entry).unwrap();

        let valid = serde_json::to_string(&test_audit_entry("valid")).unwrap();
        let input = format!("{inconsistent}\n{valid}\n");

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
        rejected_entry.stage = "failed".to_owned();
        rejected_entry.status = "failed".to_owned();
        rejected_entry.reversible = false;
        rejected_entry.privilege = "Unknown".to_owned();
        rejected_entry.verification_status = "failed".to_owned();
        rejected_entry.verification_message = "verification failed".to_owned();
        rejected_entry.outcome_status = "rejected".to_owned();
        rejected_entry.outcome_message =
            "action outcome rejected because execution and verification evidence did not establish a verified result."
                .to_owned();
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
    fn legacy_audit_records_bypass_new_lifecycle_validation() {
        let input = r#"{"id":"legacy-invalid-lifecycle","timestamp":123,"action":"unknown_action","stage":"unknown_stage","confirmed":false,"status":"failed","message":"legacy message","reversible":true,"privilege":"none","verification_status":"legacy","verification_message":"","outcome_status":"legacy","outcome_message":"","outcome_action":"different_action"}"#;

        let entries = parse_audit_entries(Cursor::new(format!("{input}\n"))).unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, "legacy-invalid-lifecycle");
        assert_eq!(entries[0].action, "unknown_action");
        assert_eq!(entries[0].stage, "unknown_stage");
        assert!(!entries[0].confirmed);
        assert_eq!(entries[0].outcome_action, "different_action");
        assert_eq!(entries[0].verification_status, "legacy");
        assert_eq!(entries[0].outcome_status, "legacy");
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
    fn mixed_audit_persistence_records_preserve_valid_history() {
        let first = test_audit_entry("mixed-first");
        let second = test_audit_entry("mixed-second");

        let first_line = serde_json::to_string(&first).unwrap();
        let second_line = serde_json::to_string(&second).unwrap();

        let input = format!(
            "{first_line}\n\
         not-valid-json\n\
         \n\
         {first_line}\n\
         {second_line}\n"
        );

        let entries = parse_audit_entries(Cursor::new(input)).unwrap();

        assert_eq!(entries, vec![first, second]);
    }

    #[test]
    fn duplicate_audit_ids_are_ignored_without_hiding_valid_history() {
        let first = test_audit_entry("duplicate-first");
        let duplicate = first.clone();
        let second = test_audit_entry("duplicate-second");

        let first_line = serde_json::to_string(&first).unwrap();
        let duplicate_line = serde_json::to_string(&duplicate).unwrap();
        let second_line = serde_json::to_string(&second).unwrap();

        let input = format!(
            "{first_line}\n\
         {duplicate_line}\n\
         {second_line}\n"
        );

        let entries = parse_audit_entries(Cursor::new(input)).unwrap();

        assert_eq!(entries, vec![first, second]);
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
    fn record_persists_successful_audit_entry_through_recording_boundary() {
        let root =
            std::env::temp_dir().join(format!("linux-powerhouse-audit-record-{}", Uuid::new_v4()));
        let file = root.join("action-audit.jsonl");

        let outcome = test_verified_outcome();

        let record = ActionAuditRecord {
            action: "refresh_health",
            stage: "verified",
            confirmed: true,
            status: "success",
            message: "action completed",
            reversible: true,
            privilege: "none",
            verification_status: "verified",
            verification_message: "verified",
            outcome: &outcome,
        };

        let entry = record_audit_entry(&record, &file).unwrap();

        assert_eq!(entry.action, "refresh_health");
        assert_eq!(entry.stage, "verified");
        assert!(entry.confirmed);
        assert_eq!(entry.status, "success");
        assert_eq!(entry.message, "action completed");
        assert!(entry.reversible);
        assert_eq!(entry.privilege, "none");
        assert_eq!(entry.verification_status, "verified");
        assert_eq!(entry.verification_message, "verified");
        assert_eq!(entry.outcome_status, "verified");
        assert_eq!(
            entry.outcome_message,
            "action execution and verification produced a verified outcome."
        );
        assert_eq!(entry.outcome_action, "refresh_health");

        let contents = std::fs::File::open(&file).unwrap();
        let history = parse_audit_entries(std::io::BufReader::new(contents)).unwrap();

        assert_eq!(history, vec![entry]);

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn record_persists_failed_audit_entry_through_recording_boundary() {
        let root =
            std::env::temp_dir().join(format!("linux-powerhouse-audit-failed-{}", Uuid::new_v4()));
        let file = root.join("action-audit.jsonl");

        let outcome = test_rejected_outcome();

        let record = ActionAuditRecord {
            action: "refresh_health",
            stage: "failed",
            confirmed: true,
            status: "failed",
            message: "action failed",
            reversible: false,
            privilege: "Unknown",
            verification_status: "failed",
            verification_message: "verification failed",
            outcome: &outcome,
        };

        let entry = record_audit_entry(&record, &file).unwrap();

        assert_eq!(entry.action, "refresh_health");
        assert_eq!(entry.stage, "failed");
        assert!(entry.confirmed);
        assert_eq!(entry.status, "failed");
        assert_eq!(entry.message, "action failed");
        assert!(!entry.reversible);
        assert_eq!(entry.privilege, "Unknown");
        assert_eq!(entry.verification_status, "failed");
        assert_eq!(entry.verification_message, "verification failed");
        assert_eq!(entry.outcome_status, "rejected");
        assert_eq!(
            entry.outcome_message,
            "action outcome rejected because execution and verification evidence did not establish a verified result."
        );
        assert_eq!(entry.outcome_action, "refresh_health");

        let contents = std::fs::File::open(&file).unwrap();
        let history = parse_audit_entries(std::io::BufReader::new(contents)).unwrap();

        assert_eq!(history, vec![entry]);

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn record_appends_multiple_audit_entries_without_overwriting_history() {
        let root =
            std::env::temp_dir().join(format!("linux-powerhouse-audit-append-{}", Uuid::new_v4()));
        let file = root.join("action-audit.jsonl");

        let outcome = test_verified_outcome();

        let first_record = ActionAuditRecord {
            action: "refresh_health",
            stage: "verified",
            confirmed: true,
            status: "success",
            message: "first action completed",
            reversible: true,
            privilege: "none",
            verification_status: "verified",
            verification_message: "verified",
            outcome: &outcome,
        };

        let second_record = ActionAuditRecord {
            action: "refresh_health",
            stage: "verified",
            confirmed: true,
            status: "success",
            message: "second action completed",
            reversible: true,
            privilege: "none",
            verification_status: "verified",
            verification_message: "verified",
            outcome: &outcome,
        };

        let first = record_audit_entry(&first_record, &file).unwrap();
        let second = record_audit_entry(&second_record, &file).unwrap();

        let history = read_audit_history(&file).unwrap();

        assert_eq!(history, vec![first, second]);

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn record_propagates_audit_path_filesystem_errors() {
        let root = std::env::temp_dir().join(format!(
            "linux-powerhouse-audit-record-error-{}",
            Uuid::new_v4()
        ));
        let parent = root.join("audit-parent");
        let file = parent.join("action-audit.jsonl");

        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(&parent, b"not a directory").unwrap();

        let outcome = test_verified_outcome();

        let record = ActionAuditRecord {
            action: "refresh_health",
            stage: "verified",
            confirmed: true,
            status: "success",
            message: "action completed",
            reversible: true,
            privilege: "none",
            verification_status: "verified",
            verification_message: "verified",
            outcome: &outcome,
        };

        let error = record_audit_entry(&record, &file).unwrap_err();

        assert!(!error.is_empty());

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn record_propagates_audit_file_open_errors() {
        let root = std::env::temp_dir().join(format!(
            "linux-powerhouse-audit-open-error-{}",
            Uuid::new_v4()
        ));
        let file = root.join("action-audit.jsonl");

        std::fs::create_dir_all(&file).unwrap();

        let outcome = test_verified_outcome();

        let record = ActionAuditRecord {
            action: "refresh_health",
            stage: "verified",
            confirmed: true,
            status: "success",
            message: "action completed",
            reversible: true,
            privilege: "none",
            verification_status: "verified",
            verification_message: "verified",
            outcome: &outcome,
        };

        let error = record_audit_entry(&record, &file).unwrap_err();

        assert!(!error.is_empty());

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn record_propagates_audit_file_write_errors() {
        let file = Path::new("/dev/full");

        let outcome = test_verified_outcome();

        let record = ActionAuditRecord {
            action: "refresh_health",
            stage: "verified",
            confirmed: true,
            status: "success",
            message: "action completed",
            reversible: true,
            privilege: "none",
            verification_status: "verified",
            verification_message: "verified",
            outcome: &outcome,
        };

        let error = record_audit_entry(&record, file).unwrap_err();

        assert!(!error.is_empty());
    }

    #[test]
    fn history_reads_successful_audit_entries_through_history_boundary() {
        let root =
            std::env::temp_dir().join(format!("linux-powerhouse-audit-history-{}", Uuid::new_v4()));
        let file = root.join("action-audit.jsonl");

        let outcome = test_verified_outcome();

        let record = ActionAuditRecord {
            action: "refresh_health",
            stage: "verified",
            confirmed: true,
            status: "success",
            message: "action completed",
            reversible: true,
            privilege: "none",
            verification_status: "verified",
            verification_message: "verified",
            outcome: &outcome,
        };

        let entry = record_audit_entry(&record, &file).unwrap();
        let history = read_audit_history(&file).unwrap();

        assert_eq!(history, vec![entry]);

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn history_reads_failed_audit_entries_through_history_boundary() {
        let root = std::env::temp_dir().join(format!(
            "linux-powerhouse-audit-failed-history-{}",
            Uuid::new_v4()
        ));
        let file = root.join("action-audit.jsonl");

        let outcome = test_rejected_outcome();

        let record = ActionAuditRecord {
            action: "refresh_health",
            stage: "failed",
            confirmed: true,
            status: "failed",
            message: "action failed",
            reversible: false,
            privilege: "Unknown",
            verification_status: "failed",
            verification_message: "verification failed",
            outcome: &outcome,
        };

        let entry = record_audit_entry(&record, &file).unwrap();
        let history = read_audit_history(&file).unwrap();

        assert_eq!(history, vec![entry.clone()]);

        let persisted = &history[0];

        assert_eq!(persisted.action, "refresh_health");
        assert_eq!(persisted.stage, "failed");
        assert!(persisted.confirmed);
        assert_eq!(persisted.status, "failed");
        assert_eq!(persisted.message, "action failed");
        assert!(!persisted.reversible);
        assert_eq!(persisted.privilege, "Unknown");
        assert_eq!(persisted.verification_status, "failed");
        assert_eq!(persisted.verification_message, "verification failed");
        assert_eq!(persisted.outcome_status, "rejected");
        assert_eq!(
            persisted.outcome_message,
            "action outcome rejected because execution and verification evidence did not establish a verified result."
        );
        assert_eq!(persisted.outcome_action, "refresh_health");

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn history_returns_empty_when_audit_file_is_missing() {
        let root =
            std::env::temp_dir().join(format!("linux-powerhouse-audit-missing-{}", Uuid::new_v4()));
        let file = root.join("action-audit.jsonl");

        assert!(!file.exists());

        let history = read_audit_history(&file).unwrap();

        assert!(history.is_empty());
        assert!(!file.exists());

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn history_propagates_non_not_found_file_errors() {
        let root =
            std::env::temp_dir().join(format!("linux-powerhouse-audit-error-{}", Uuid::new_v4()));
        let file = root.join("audit-parent");

        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(&file, b"not a directory").unwrap();

        let error = read_audit_history(&file.join("action-audit.jsonl")).unwrap_err();

        assert!(!error.is_empty());

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn history_preserves_valid_records_in_mixed_audit_file() {
        let root = std::env::temp_dir().join(format!(
            "linux-powerhouse-audit-mixed-history-{}",
            Uuid::new_v4()
        ));
        let file = root.join("action-audit.jsonl");

        let first = test_audit_entry("mixed-history-first");
        let second = test_audit_entry("mixed-history-second");

        let first_line = serde_json::to_string(&first).unwrap();
        let second_line = serde_json::to_string(&second).unwrap();

        let input = format!(
            "{first_line}\n\
         not-valid-json\n\
         \n\
         {first_line}\n\
         {second_line}\n"
        );

        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(&file, input).unwrap();

        let history = read_audit_history(&file).unwrap();

        assert_eq!(history, vec![first, second]);

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn record_rejects_invalid_outcome_evidence_before_audit_validation() {
        let audit = ActionAudit;
        let mut outcome = test_verified_outcome();
        outcome.execution.executed = false;

        let error = audit
            .record(&ActionAuditRecord {
                action: "refresh_health",
                stage: "verified",
                confirmed: true,
                status: "success",
                message: "action completed",
                reversible: true,
                privilege: "none",
                verification_status: "verified",
                verification_message: "verification completed",
                outcome: &outcome,
            })
            .unwrap_err();

        assert_eq!(error, "invalid action audit outcome evidence");
    }

    #[test]
    fn record_rejects_mismatched_outcome_evidence_before_audit_validation() {
        let audit = ActionAudit;
        let mut outcome = test_verified_outcome();
        outcome.execution.action_id = "storage_diagnostic".to_owned();

        let error = audit
            .record(&ActionAuditRecord {
                action: "refresh_health",
                stage: "verified",
                confirmed: true,
                status: "success",
                message: "action completed",
                reversible: true,
                privilege: "none",
                verification_status: "verified",
                verification_message: "verification completed",
                outcome: &outcome,
            })
            .unwrap_err();

        assert_eq!(error, "invalid action audit outcome evidence");
    }

    #[test]
    fn record_rejects_invalid_verification_evidence_before_audit_validation() {
        let audit = ActionAudit;
        let outcome = test_verified_outcome();

        let error = audit
            .record(&ActionAuditRecord {
                action: "refresh_health",
                stage: "verified",
                confirmed: true,
                status: "success",
                message: "action completed",
                reversible: true,
                privilege: "none",
                verification_status: "failed",
                verification_message: "verified",
                outcome: &outcome,
            })
            .unwrap_err();

        assert_eq!(error, "invalid action audit verification evidence");
    }

    #[test]
    fn record_rejects_mismatched_verification_message_before_audit_validation() {
        let audit = ActionAudit;
        let outcome = test_verified_outcome();

        let error = audit
            .record(&ActionAuditRecord {
                action: "refresh_health",
                stage: "verified",
                confirmed: true,
                status: "success",
                message: "action completed",
                reversible: true,
                privilege: "none",
                verification_status: "verified",
                verification_message: "different verification message",
                outcome: &outcome,
            })
            .unwrap_err();

        assert_eq!(error, "invalid action audit verification evidence");
    }

    #[test]
    fn record_rejects_entries_that_fail_audit_validation() {
        let audit = ActionAudit;
        let outcome = test_rejected_outcome();

        let error = audit
            .record(&ActionAuditRecord {
                action: "refresh_health",
                stage: "failed",
                confirmed: true,
                status: "failed",
                message: "action failed",
                reversible: false,
                privilege: "none",
                verification_status: "failed",
                verification_message: "verification failed",
                outcome: &outcome,
            })
            .unwrap_err();

        assert_eq!(error, "invalid action audit entry");
    }

    #[test]
    fn stage_confirmation_validation_accepts_supported_lifecycle_paths() {
        assert!(is_valid_stage_confirmation("verified", true));
        assert!(is_valid_stage_confirmation("failed", true));
        assert!(!is_valid_stage_confirmation("verified", false));
        assert!(!is_valid_stage_confirmation("failed", false));
    }

    #[test]
    fn complete_lifecycle_validation_accepts_supported_verified_and_failed_paths() {
        assert!(is_valid_complete_lifecycle(&test_audit_entry("verified")));

        let mut failed_entry = test_audit_entry("failed");
        failed_entry.stage = "failed".to_owned();
        failed_entry.status = "failed".to_owned();
        failed_entry.reversible = false;
        failed_entry.privilege = "Unknown".to_owned();
        failed_entry.verification_status = "failed".to_owned();
        failed_entry.verification_message = "verification failed".to_owned();
        failed_entry.outcome_status = "rejected".to_owned();
        failed_entry.outcome_message = "action outcome rejected because execution and verification evidence did not establish a verified result.".to_owned();

        assert!(is_valid_complete_lifecycle(&failed_entry));
    }

    #[test]
    fn complete_lifecycle_validation_rejects_inconsistent_paths() {
        let mut failed_entry = test_audit_entry("failed-verified");
        failed_entry.stage = "failed".to_owned();
        failed_entry.status = "failed".to_owned();

        assert!(!is_valid_complete_lifecycle(&failed_entry));

        let mut unconfirmed_entry = test_audit_entry("unconfirmed");
        unconfirmed_entry.confirmed = false;

        assert!(!is_valid_complete_lifecycle(&unconfirmed_entry));
    }

    #[test]
    fn complete_lifecycle_validation_rejects_mismatched_outcome_action() {
        let mut entry = test_audit_entry("mismatched-action");
        entry.outcome_action = "storage_diagnostic".to_owned();

        assert!(!is_valid_complete_lifecycle(&entry));
    }

    #[test]
    fn complete_lifecycle_validation_rejects_invalid_lifecycle_invariants() {
        let mut invalid_privilege = test_audit_entry("invalid-privilege");
        invalid_privilege.privilege = "Unknown".to_owned();
        assert!(!is_valid_complete_lifecycle(&invalid_privilege));

        let mut invalid_reversibility = test_audit_entry("invalid-reversibility");
        invalid_reversibility.reversible = false;
        assert!(!is_valid_complete_lifecycle(&invalid_reversibility));

        let mut invalid_verification_status = test_audit_entry("invalid-verification-status");
        invalid_verification_status.verification_status = "failed".to_owned();
        assert!(!is_valid_complete_lifecycle(&invalid_verification_status));

        let mut invalid_outcome_status = test_audit_entry("invalid-outcome-status");
        invalid_outcome_status.outcome_status = "rejected".to_owned();
        invalid_outcome_status.outcome_message =
        "action outcome rejected because execution and verification evidence did not establish a verified result."
            .to_owned();
        assert!(!is_valid_complete_lifecycle(&invalid_outcome_status));

        let mut invalid_verification_outcome = test_audit_entry("invalid-verification-outcome");
        invalid_verification_outcome.outcome_status = "rejected".to_owned();
        invalid_verification_outcome.outcome_message =
        "action outcome rejected because execution and verification evidence did not establish a verified result."
            .to_owned();
        invalid_verification_outcome.verification_status = "verified".to_owned();
        assert!(!is_valid_complete_lifecycle(&invalid_verification_outcome));
    }

    #[test]
    fn shared_audit_validation_accepts_supported_verified_and_failed_paths() {
        assert!(is_valid_audit_entry(&test_audit_entry("verified")));

        let mut failed_entry = test_audit_entry("failed");
        failed_entry.stage = "failed".to_owned();
        failed_entry.status = "failed".to_owned();
        failed_entry.reversible = false;
        failed_entry.privilege = "Unknown".to_owned();
        failed_entry.verification_status = "failed".to_owned();
        failed_entry.verification_message = "verification failed".to_owned();
        failed_entry.outcome_status = "rejected".to_owned();
        failed_entry.outcome_message = "action outcome rejected because execution and verification evidence did not establish a verified result.".to_owned();

        assert!(is_valid_audit_entry(&failed_entry));
    }

    #[test]
    fn shared_audit_validation_rejects_legacy_verification_with_verified_outcome() {
        let mut entry = test_audit_entry("legacy-verification-verified-outcome");
        entry.verification_status = "legacy".to_owned();

        assert!(!is_valid_audit_entry(&entry));
    }

    #[test]
    fn shared_audit_validation_rejects_zero_timestamp() {
        let mut entry = test_audit_entry("zero-timestamp");
        entry.timestamp = 0;

        assert!(!is_valid_audit_entry(&entry));
    }

    #[test]
    fn shared_audit_validation_rejects_blank_audit_id() {
        let mut entry = test_audit_entry("blank-audit-id");
        entry.id = "   ".to_owned();

        assert!(!is_valid_audit_entry(&entry));
    }

    #[test]
    fn shared_audit_validation_rejects_blank_audit_action() {
        let mut entry = test_audit_entry("blank-audit-action");
        entry.action = "   ".to_owned();

        assert!(!is_valid_audit_entry(&entry));
    }

    #[test]
    fn shared_audit_validation_rejects_invalid_audit_action() {
        let mut entry = test_audit_entry("invalid-audit-action");
        entry.action = "unknown_action".to_owned();

        assert!(!is_valid_audit_entry(&entry));
    }

    #[test]
    fn shared_audit_validation_rejects_blank_audit_stage() {
        let mut entry = test_audit_entry("blank-audit-stage");
        entry.stage = "   ".to_owned();

        assert!(!is_valid_audit_entry(&entry));
    }

    #[test]
    fn shared_audit_validation_rejects_invalid_audit_stage() {
        let mut entry = test_audit_entry("invalid-audit-stage");
        entry.stage = "unknown_stage".to_owned();

        assert!(!is_valid_audit_entry(&entry));
    }

    #[test]
    fn shared_audit_validation_rejects_blank_audit_status() {
        let mut entry = test_audit_entry("blank-audit-status");
        entry.status = "   ".to_owned();

        assert!(!is_valid_audit_entry(&entry));
    }

    #[test]
    fn shared_audit_validation_rejects_invalid_audit_status() {
        let mut entry = test_audit_entry("invalid-audit-status");
        entry.status = "unknown_status".to_owned();

        assert!(!is_valid_audit_entry(&entry));
    }

    #[test]
    fn shared_audit_validation_rejects_blank_audit_message() {
        let mut entry = test_audit_entry("blank-audit-message");
        entry.message = "   ".to_owned();

        assert!(!is_valid_audit_entry(&entry));
    }

    #[test]
    fn shared_audit_validation_rejects_blank_audit_privilege() {
        let mut entry = test_audit_entry("blank-audit-privilege");
        entry.privilege = "   ".to_owned();

        assert!(!is_valid_audit_entry(&entry));
    }

    #[test]
    fn shared_audit_validation_rejects_invalid_audit_privilege() {
        let mut entry = test_audit_entry("invalid-audit-privilege");
        entry.privilege = "admin".to_owned();

        assert!(!is_valid_audit_entry(&entry));
    }

    #[test]
    fn shared_audit_validation_rejects_blank_audit_verification_status() {
        let mut entry = test_audit_entry("blank-audit-verification-status");
        entry.verification_status = "   ".to_owned();

        assert!(!is_valid_audit_entry(&entry));
    }

    #[test]
    fn shared_audit_validation_rejects_invalid_audit_verification_status() {
        let mut entry = test_audit_entry("invalid-audit-verification-status");
        entry.verification_status = "unknown_verification_status".to_owned();

        assert!(!is_valid_audit_entry(&entry));
    }

    #[test]
    fn shared_audit_validation_rejects_blank_verification_message() {
        let mut entry = test_audit_entry("blank-verification-message");
        entry.verification_message = "   ".to_owned();

        assert!(!is_valid_audit_entry(&entry));
    }

    #[test]
    fn shared_audit_validation_rejects_blank_audit_outcome_status() {
        let mut entry = test_audit_entry("blank-audit-outcome-status");
        entry.outcome_status = "   ".to_owned();

        assert!(!is_valid_audit_entry(&entry));
    }

    #[test]
    fn shared_audit_validation_rejects_invalid_audit_outcome_status() {
        let mut entry = test_audit_entry("invalid-audit-outcome-status");
        entry.outcome_status = "unknown_outcome_status".to_owned();

        assert!(!is_valid_audit_entry(&entry));
    }

    #[test]
    fn shared_audit_validation_rejects_blank_outcome_message() {
        let mut entry = test_audit_entry("blank-outcome-message");
        entry.outcome_message = "   ".to_owned();

        assert!(!is_valid_audit_entry(&entry));
    }

    #[test]
    fn shared_audit_validation_rejects_blank_audit_outcome_action() {
        let mut entry = test_audit_entry("blank-audit-outcome-action");
        entry.outcome_action = "   ".to_owned();

        assert!(!is_valid_audit_entry(&entry));
    }

    #[test]
    fn shared_audit_validation_rejects_invalid_audit_outcome_action() {
        let mut entry = test_audit_entry("invalid-audit-outcome-action");
        entry.outcome_action = "storage_diagnostic".to_owned();

        assert!(!is_valid_audit_entry(&entry));
    }

    #[test]
    fn shared_audit_validation_rejects_invalid_audit_reversibility() {
        let mut entry = test_audit_entry("invalid-audit-reversibility");
        entry.reversible = false;

        assert!(!is_valid_audit_entry(&entry));
    }

    #[test]
    fn shared_audit_validation_rejects_invalid_audit_confirmation() {
        let mut entry = test_audit_entry("invalid-audit-confirmation");
        entry.confirmed = false;

        assert!(!is_valid_audit_entry(&entry));
    }

    #[test]
    fn shared_audit_validation_rejects_invalid_audit_lifecycle() {
        let mut entry = test_audit_entry("invalid-audit-lifecycle");
        entry.status = "failed".to_owned();

        assert!(!is_valid_audit_entry(&entry));
    }

    #[test]
    fn shared_audit_validation_accepts_canonical_outcome_messages() {
        assert!(is_valid_audit_entry(&test_audit_entry("verified")));

        let mut rejected_entry = test_audit_entry("rejected");
        rejected_entry.stage = "failed".to_owned();
        rejected_entry.status = "failed".to_owned();
        rejected_entry.reversible = false;
        rejected_entry.privilege = "Unknown".to_owned();
        rejected_entry.verification_status = "failed".to_owned();
        rejected_entry.verification_message = "verification failed".to_owned();
        rejected_entry.outcome_status = "rejected".to_owned();
        rejected_entry.outcome_message =
        "action outcome rejected because execution and verification evidence did not establish a verified result."
            .to_owned();

        assert!(is_valid_audit_entry(&rejected_entry));
    }

    #[test]
    fn shared_audit_validation_rejects_mismatched_verified_outcome_message() {
        let mut entry = test_audit_entry("mismatched-verified-message");
        entry.outcome_message = "arbitrary outcome message".to_owned();

        assert!(!is_valid_audit_entry(&entry));
    }

    #[test]
    fn shared_audit_validation_rejects_mismatched_rejected_outcome_message() {
        let mut entry = test_audit_entry("mismatched-rejected-message");
        entry.stage = "failed".to_owned();
        entry.status = "failed".to_owned();
        entry.reversible = false;
        entry.privilege = "Unknown".to_owned();
        entry.verification_status = "failed".to_owned();
        entry.verification_message = "verification failed".to_owned();
        entry.outcome_status = "rejected".to_owned();
        entry.outcome_message = "arbitrary outcome message".to_owned();

        assert!(!is_valid_audit_entry(&entry));
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
