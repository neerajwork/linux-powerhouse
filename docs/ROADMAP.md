# Linux Powerhouse Roadmap

## Purpose

Linux Powerhouse is being developed as a local-first system intelligence platform. The roadmap favors small, independently validated increments, with Rust and Desktop CI required for every feature step.

### Core principles

- **Local-first:** system observations and user preferences remain local unless an explicitly requested sharing/export flow is used.
- **Read-only by default:** health and intelligence features do not mutate the system automatically.
- **Explicit actions:** remediation capabilities must be previewed, confirmed, narrowly scoped, and auditable.
- **Incremental delivery:** each roadmap step should produce a focused commit and pass Rust + Desktop CI before merge.
- **Bounded data:** retained local history must have explicit limits and safe reset behavior.
- **No silent privilege escalation:** privileged operations require explicit user intent and appropriate authorization.

## Recovered completed roadmap milestones

The repository's historical PRs and branch names recover the roadmap from **Step 7 onward**. Steps 1–6 are not represented by dedicated feature branches in the available GitHub history, so they remain unreconstructed.

### Steps 7–11 — Foundation, Desktop, Monitoring and Health Engine

| Step | Milestone | Status |
| --- | --- | --- |
| 7 | Bootstrap Linux Powerhouse architecture | Complete |
| 8 | Desktop shell with initial system dashboard | Complete |
| 9 | System Dashboard v1 — storage, processes and network | Complete |
| 10 | Realtime monitoring engine and live dashboard | Complete |
| 11 | System health and deterministic anomaly engine | Complete |

### Steps 12–16 — Linux Intelligence Layers

| Step | Milestone | Status |
| --- | --- | --- |
| 12 | Storage Intelligence | Complete |
| 13 | Process Intelligence | Complete |
| 14 | Network Intelligence | Complete |
| 15 | Service Intelligence | Complete |
| 16 | Unified System Intelligence | Complete |

### Steps 17–21 — Desktop Health and Persistent History

| Step | Milestone | Status |
| --- | --- | --- |
| 17 | Expose Unified System Health to Desktop | Complete |
| 18 | Desktop System Health View | Complete |
| 19 | Actionable System Health Guidance | Complete |
| 20 | System Health History & Trends | Complete |
| 21 | Persistent System Health History | Complete |

### Steps 22–31 — Health History and Insight Explorer

| Step | Milestone | Status |
| --- | --- | --- |
| 22 | System Health History Controls | Complete |
| 23 | System Health History Insights | Complete |
| 24 | System Health Insights Explanations | Complete |
| 25 | Subsystem Health Details | Complete |
| 26 | System Health History Export | Complete |
| 27 | Health History Period Comparison | Complete |
| 28 | Health Insight Timeline | Complete |
| 29 | Health Insight Timeline Filters | Complete |
| 30 | Health Insight Summary | Complete |
| 31 | Actionable Health Insights | Complete |

### Steps 32–38 — Health Insights Explorer

| Step | Milestone | Status |
| --- | --- | --- |
| 32 | Focused Health Insight Export | Complete |
| 33 | Local Health Insight Sharing | Complete |
| 34 | Health Insight Filter Persistence | Complete |
| 35 | Health Insight Search | Complete |
| 36 | Health Insight Sorting | Complete |
| 37 | Health Insight Grouping | Complete |
| 38 | Health Insight Presets | Complete |

### Steps 39–47 — Safe System Intelligence Actions

| Step | Milestone | Status |
| --- | --- | --- |
| 39 | Health Insight Recommendations | Complete |
| 40 | Action Preview | Complete |
| 41 | Safe System Actions | Complete |
| 42 | Action History and Audit | Complete |
| 43 | Action Outcome Verification | Complete |
| 44 | Action Remediation Suggestions | Complete |
| 45 | Unified Health Action Workspace | Complete |
| 46 | Action Execution Outcome | Complete |
| 47 | Safe Follow-up Execution | Complete |

## System Performance Intelligence

### Step 48 — Performance Baseline and Trends
Status: **Complete**.

### Step 49 — Performance Anomaly Explanations
Status: **Complete**.

### Step 50 — Process Performance Drill-down
Status: **Complete**.

### Step 51 — Performance History Comparison
Status: **Complete**.

### Step 52 — Alert Controls and Notification Snoozing
Status: **Complete**.

- Local snooze preferences with 7, 14, and 30-day options.
- Local dismissal and restore controls.
- Deterministic alert decisions exposed through Tauri.
- Critical events remain notification-eligible regardless of routine warning preferences.

### Step 53 — Alert Event History & Audit
Status: **Complete**.

- Record alert category, severity, signal value, timestamp, decision, and reason.
- Keep alert history bounded to a deterministic local limit.
- Preserve critical-event records even when routine warnings are snoozed or dismissed.
- Expose history and explicit clear-history controls through Tauri.
- Provide a focused Desktop Alert History view.

### Step 54 — Persistent Alert History
Status: **Complete**.

- Store and restore bounded history in the local application data directory.
- Preserve the 100-event retention limit.
- Persist clear-history operations using atomic replacement.
- Gracefully recover from missing or unreadable history files.

### Step 55 — Alert History Controls
Status: **Complete**.

- Filter by severity, category, notification decision, and recent time period.
- Reset filters and show filtered-versus-retained counts.
- Keep filtering presentation-only and preserve local persistence.

### Step 56 — Alert History Summary & Trends
Status: **Complete**.

- Show retained event totals and warning/critical/notified/suppressed counts.
- Identify the most frequent alert category.
- Compare the latest 7 days with the preceding 7 days.
- Present deterministic increasing/decreasing/stable activity direction.

### Step 57 — Alert History Insights
Status: **Complete**.

- Identify the alert category with the largest recent change versus the preceding 7 days.
- Report whether critical events are increasing, decreasing, or stable.
- Report whether suppressed events are increasing, decreasing, or stable.
- Provide a concise primary attention signal based on retained local history.
- Keep insights deterministic, local, read-only, and presentation-only.

### Step 58 — Alert ↔ Performance Correlation
Status: **Complete**.

- Correlate an alert with the nearest monitoring snapshot within a bounded 30-second window.
- Preserve CPU, memory, swap, storage I/O, process-count, and running-process context.
- Produce deterministic primary evidence for the correlated signal.
- Keep correlation local, read-only, bounded, and independent of remediation.

### Step 59 — Alert Explanations
Status: **Complete**.

- Explain the signal, observed value, severity, decision, and policy reason in plain language.
- Distinguish notified versus suppressed routine warnings.
- Preserve an explicit critical-event override explanation.
- Explain snoozed, dismissed, expired, and active-policy outcomes deterministically.
- Surface explanations directly in the Desktop Alert History view.
- Keep explanations local, deterministic, read-only, and presentation-only.
- Do not trigger remediation or alter stored alert history.

### Step 60 — Alert Evidence View
Status: **Complete**.

- Show the nearest correlated monitoring snapshot for an alert when it falls within the existing 30-second window.
- Show signal-specific primary evidence for CPU, memory, swap, storage, and network alerts.
- Surface supporting context including memory, swap, process count, and running-process count.
- Show the age of the correlated snapshot relative to the alert context.
- Gracefully explain when no matching performance snapshot is available.
- Keep the evidence view local, deterministic, read-only, and presentation-only.
- Do not change alert decisions, persistence, retention, or remediation behavior.

### Step 61 — Alert Process Evidence
Status: **Complete**.

- Surface bounded CPU and memory process contributors alongside alert evidence.
- Preserve PID, process name, rank, and relevant resource metric.
- Keep process observations explicitly non-causal.
- Keep evidence local, deterministic, read-only, and bounded.

### Step 62 — Alert Remediation Guidance
Status: **Complete**.

- Provide deterministic investigation guidance for CPU, memory, swap, storage, and network alerts.
- Prioritize verification for critical events.
- Encourage persistence checks for snoozed and dismissed warnings.
- Keep guidance informational, non-mutating, and non-privileged.
- Expose guidance through Tauri and render it with existing alert evidence.

### Step 63 — Alert Action Preview
Status: **In progress**.

Turn Step 62 guidance into an explicit, inspectable preview layer before any future execution boundary.

Acceptance goals:

- Provide deterministic possible-action previews for CPU, memory, swap, storage, and network alerts.
- Place critical-condition verification first for critical alerts.
- Provide a recheck preview for snoozed and dismissed warnings.
- Expose previews through a dedicated Tauri command.
- Render previews in Desktop Alert History alongside evidence and guidance.
- Mark every preview as non-executable and non-privileged.
- Preserve the rule that previews do not authorize, execute, or mutate system state.
- Do not change alert policy, persistence, retention, snooze, dismissal, or existing remediation behavior.

## Subsequent expansion areas

After the deterministic alert action-preview boundary, future roadmap areas should continue to be evaluated against architecture, safety, privacy, and user value. Candidate areas include:

1. **Explicit Action Confirmation** — introduce a clear user-confirmation boundary for narrowly scoped, safe operations.
2. **Cross-subsystem Alert Correlation** — connect alert categories with deeper performance, process, service, and network evidence.
3. **Network Intelligence expansion** — connectivity quality, DNS, service reachability, and network trends.
4. **Security Posture** — local security signals, configuration visibility, and safe recommendations.
5. **Resource Optimization** — explainable opportunities to reduce resource consumption.
6. **Automation** — carefully bounded recurring local workflows built on the action/audit foundation.
7. **Linux Tool Ecosystem** — selectively expose mature command-line utilities through the capability-oriented Tool Registry.

These are candidates, not committed step numbers. Each should be broken into small roadmap steps after requirements and safety boundaries are defined.

## Definition of Done

A roadmap step is complete only when:

1. The feature is implemented on a dedicated branch.
2. The commit message clearly identifies the feature.
3. Rust CI is green for the feature commit.
4. Desktop CI is green for the feature commit.
5. A focused PR is reviewed and merged.
6. Post-merge Rust and Desktop CI are green.
7. The roadmap is updated when the milestone materially changes project direction.

## CI tracking convention

For every milestone, record both the **feature commit/message** and the corresponding Rust/Desktop CI runs. Post-merge workflows may display only generic workflow names, so the feature commit remains the canonical link between implementation and validation.
