# Linux Powerhouse Roadmap

## Purpose

Linux Powerhouse is being developed as a local-first system intelligence platform. The roadmap favors small, independently validated increments, with Rust and Desktop CI required for every feature step.

### Core principles

- **Local-first:** system observations and user preferences remain local unless an explicitly requested sharing/export flow is used.
- **Read-only by default:** health and intelligence features do not mutate the system automatically.
- **Explicit actions:** future remediation capabilities must be previewed, confirmed, narrowly scoped, and auditable.
- **Incremental delivery:** each roadmap step should produce a focused commit and pass Rust + Desktop CI before merge.
- **Bounded data:** retained local history must have explicit limits and safe reset behavior.
- **No silent privilege escalation:** privileged operations, when eventually introduced, require explicit user intent and appropriate authorization.

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

## Next roadmap area — Safe System Intelligence Actions

The project now moves from **understanding system health** toward **helping the user decide what to do**, while preserving the read-only safety boundary until an action is explicitly approved.

### Step 39 — Health Insight Recommendations

Provide contextual, read-only recommendations for detected health signals.

Acceptance goals:

- Explain why a signal matters.
- Associate recommendations with the relevant subsystem.
- Show severity and confidence where meaningful.
- Provide concise next-step guidance.
- Never execute remediation automatically.

### Step 40 — Action Preview

Introduce a preview layer for proposed system actions without executing them.

Acceptance goals:

- Describe exactly what an action would change.
- Identify affected subsystem/resources.
- Show expected impact and reversibility.
- Identify permission requirements.
- Require explicit confirmation before any execution path.

### Step 41 — Safe System Actions

Allow narrowly scoped, explicitly approved actions.

Acceptance goals:

- No silent remediation.
- Explicit user confirmation.
- Least-privilege execution.
- Clear success/failure reporting.
- Safe handling of unavailable permissions or unsupported actions.

### Step 42 — Action History and Audit

Record user-approved system actions and their outcomes.

Acceptance goals:

- Timestamp and action identity.
- Reason/context for the action.
- Confirmation state.
- Result and error details.
- Reversibility/rollback state where applicable.
- Local retention with bounded history and clear reset behavior.

## Subsequent expansion areas

After the safe-action foundation is complete, future roadmap areas should be evaluated against the architecture and user value rather than adding UI features mechanically. Candidate areas include:

1. **System Performance Intelligence** — CPU, memory, storage I/O, process and service trends.
2. **Network Intelligence** — connectivity quality, interfaces, DNS and service reachability insights.
3. **Security Posture** — local security signals, configuration visibility, and safe recommendations.
4. **Resource Optimization** — explainable opportunities to reduce resource consumption.
5. **Automation** — carefully bounded recurring local workflows built on the action/audit foundation.
6. **Cross-subsystem Correlation** — connect related signals into higher-level system narratives.

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
