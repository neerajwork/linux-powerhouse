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

> Step 15 had multiple intermediate `v2`–`v12` branches during development. These were implementation iterations, not separate roadmap milestones; the merged service-intelligence PR is the canonical Step 15 milestone.

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

The Health Insights Explorer now supports local history, filtering, persistence, search, sorting, grouping, reusable built-in/custom presets, export, sharing, summaries, explanations, comparisons, and subsystem drill-down.

## Steps 39–47 — Safe System Intelligence Actions

The project then moved from **understanding system health** toward **helping the user decide and act**, while preserving the safety boundary that AI and recommendations do not receive unrestricted system access.

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

### Steps 43–47 — Execution Confidence and Guided Follow-up

The safe-action foundation was subsequently extended with deterministic outcome verification, remediation guidance, a unified action workspace, explicit outcome presentation, and confirmation-gated follow-up execution.

These steps preserve the same safety model: recommendations remain informational until the user explicitly approves an action; execution remains constrained to registered safe actions; outcomes are recorded and presented; and follow-up execution requires explicit confirmation.

## Next roadmap area — System Performance Intelligence

With the safe-action foundation established, the next phase should broaden Linux Powerhouse from health interpretation and remediation into deeper **system performance intelligence**. The next milestone should remain small, read-only, locally processed, and independently verifiable.

### Step 48 — Performance Baseline and Trends

Establish a unified read-only performance baseline for CPU, memory, storage I/O, and process activity, using existing monitoring/intelligence infrastructure where possible.

Acceptance goals:

- Present current CPU, memory, storage I/O, and process activity in a consistent model.
- Establish short-term local baselines and trends without requiring external services.
- Identify meaningful deviations from the local baseline deterministically.
- Reuse existing system-intelligence data instead of creating parallel collectors where practical.
- Keep the feature read-only; no remediation or automatic action execution.
- Expose data through the existing desktop architecture with focused, testable interfaces.
- Preserve bounded local data and existing privacy/safety guarantees.

### Future performance-intelligence steps

Step 48 should be treated as the foundation for a sequence of small performance-intelligence milestones rather than a large dashboard rewrite. Candidate follow-up areas include:

1. **Performance anomaly explanations** — explain why a CPU, memory, I/O, or process signal is unusual.
2. **Process performance drill-down** — correlate resource usage with processes and services.
3. **Performance history comparison** — compare current behavior with previous local periods.
4. **Resource optimization recommendations** — suggest safe, explainable opportunities without automatically changing the system.
5. **Cross-subsystem performance correlation** — connect CPU, memory, I/O, network, and service signals into higher-level narratives.

These remain candidate areas until each is defined as a focused roadmap step.

## Step 52 — Alert Controls and Notification Snoozing

Add deterministic controls for routine warning notifications while preserving critical-event visibility.

Status: **Complete**.

- Local snooze preferences with 7, 14, and 30-day options.
- Local dismissal and restore controls.
- Deterministic alert decisions exposed through Tauri.
- Critical events remain notification-eligible regardless of routine warning preferences.

## Step 53 — Alert Event History & Audit

Make alert decisions observable and auditable without changing the local-first, read-only safety model.

Acceptance goals:

- Record alert category, severity, signal value, timestamp, decision, and reason.
- Keep alert history bounded to a deterministic local limit.
- Preserve critical-event records even when routine warnings are snoozed or dismissed.
- Expose history and explicit clear-history controls through Tauri.
- Provide a focused Desktop Alert History view.
- Keep history local and in-memory; no external service or automatic remediation.
- Cover retention, clearing, snooze, dismissal, expiry, and critical override behavior with tests.

Status: **In progress**.

## Subsequent expansion areas

After performance intelligence, future roadmap areas should continue to be evaluated against architecture, safety, privacy, and user value rather than adding UI features mechanically. Candidate areas include:

1. **Network Intelligence expansion** — connectivity quality, DNS, service reachability, and network trends.
2. **Security Posture** — local security signals, configuration visibility, and safe recommendations.
3. **Resource Optimization** — explainable opportunities to reduce resource consumption.
4. **Automation** — carefully bounded recurring local workflows built on the action/audit foundation.
5. **Cross-subsystem Correlation** — connect related signals into higher-level system narratives.
6. **Linux Tool Ecosystem** — selectively expose mature command-line utilities through the capability-oriented Tool Registry.

These are candidates, not committed step numbers. Each should be broken into small roadmap steps after requirements and safety boundaries are defined.

## Definition of Done

A roadmap step is complete only when:

1. The feature is implemented on a dedicated branch.
2. The commit message clearly identifies the feature.
3. Rust CI is green for the feature commit.
4. Desktop CI is green for the feature commit.
5. A focused PR is reviewed and squash-merged.
6. Post-merge Rust and Desktop CI are green.
7. The roadmap is updated when the milestone materially changes project direction.

## CI tracking convention

For every milestone, record both the **feature commit/message** and the corresponding Rust/Desktop CI runs. Post-merge workflows may display only generic workflow names, so the feature commit remains the canonical link between implementation and validation.
