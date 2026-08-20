# Step 63 — Alert Action Preview

Step 63 introduces a deterministic, read-only action-preview layer between alert guidance and future user-confirmed execution.

## Scope

An alert now exposes a bounded list of **possible actions to consider**. These previews describe investigation or user-controlled follow-up without executing, authorizing, or requesting privileged remediation.

The Rust `health-status` crate is the single source of truth through `preview_alert_actions()`. The Desktop application receives previews through the `alert_action_preview` Tauri command.

## Safety boundary

Step 63 is preview-only.

- No preview is executable.
- No preview requires privilege.
- No system mutation is performed.
- No process is stopped.
- No file is deleted or modified.
- No network or service configuration is changed.
- Critical alerts place verification first.
- Snoozed and dismissed warnings receive an explicit recheck option.
- Existing alert policy, persistence, retention, snooze, dismissal, evidence, and guidance behavior remain unchanged.

## Action categories

| Alert | Preview examples |
| --- | --- |
| CPU | Review process contributors; recheck CPU condition |
| Memory | Review process contributors; recheck memory condition |
| Swap | Review memory pressure; recheck swap condition |
| Storage | Review storage usage; verify data safeguards |
| Network | Review network diagnostics; recheck network condition |
| Critical | Verify critical condition is inserted first |
| Snoozed / dismissed | Recheck previously suppressed warning |

## Architecture

```text
Alert Event
    ↓
Performance Evidence
    ↓
Process Evidence
    ↓
Human-readable Guidance
    ↓
Action Preview (Step 63)
    ↓
Future explicit confirmation boundary
    ↓
Future execution milestone
```

Step 63 intentionally stops before execution. This preserves a clean safety boundary for a later milestone that can introduce explicit user confirmation and audited execution independently.

## Acceptance criteria

- [x] Deterministic previews exist for CPU, memory, swap, storage, and network alerts.
- [x] Critical alerts prioritize verification.
- [x] Snoozed/dismissed alerts receive a recheck preview.
- [x] Previews are exposed through a Tauri command.
- [x] Desktop Alert History renders previews alongside existing evidence and guidance.
- [x] All previews are explicitly non-executable.
- [x] All previews are non-privileged.
- [x] No system state is changed by Step 63.
- [x] Existing alert behavior remains unchanged.
