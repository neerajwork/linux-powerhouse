# Linux Powerhouse

**Powerful Linux. Simplified for everyone.**

Linux Powerhouse is an open-source, local-first Linux desktop application focused on system intelligence, monitoring, health insights, and safe system actions.

It aims to make Linux capabilities easier to understand and use through an intuitive graphical experience, deterministic system intelligence, and carefully controlled operations.

## Vision

Linux already provides powerful system tools and interfaces. Linux Powerhouse aims to bring these capabilities together in a human-friendly desktop experience.

The long-term vision is to help users:

- **Understand** system activity, resource usage, and system health.
- **Discover** useful Linux capabilities without memorizing commands.
- **Investigate** issues using clear evidence and subsystem intelligence.
- **Act safely** through explicit confirmation, scoped authorization, and verification.
- **Automate** repeatable workflows within clearly defined safety boundaries.
- **Extend** functionality through a capability-oriented ecosystem.

## Core principle

> **AI proposes. Tools constrain. Policies decide. Permissions authorize. Linux enforces. Verification confirms. The user remains in control.**

AI must not be treated as a trusted system administrator. Any future AI-assisted operation must use structured capabilities and pass deterministic policy, authorization, execution, and verification boundaries.

## Current development status

**Active development — system intelligence and safe-action foundation implemented.**

Linux Powerhouse has progressed beyond its initial architecture and bootstrap stage.

The current implementation includes:

- A Rust workspace with 15 library crates and a Tauri desktop crate.
- A desktop application built with Tauri 2, React 19, TypeScript, and Vite.
- System, storage, process, network, and service status and intelligence.
- Unified system intelligence and deterministic health evaluation.
- Performance monitoring, history, anomaly explanations, and drill-down.
- Alert decisions, local alert history, evidence, and remediation guidance.
- Action previews, explicit confirmation, scoped authorization, and allowlisted diagnostic actions.
- Execution-result verification, derived outcomes, remediation suggestions, and structured action auditing.
- Local persistence with validation and resilience for alert and action history.

The current action capabilities are deliberately narrow and bounded. Linux Powerhouse does not provide an unrestricted shell, arbitrary command execution, or general privileged system administration.

Recent development has emphasized audit persistence correctness, parser resilience, lifecycle validation, and robust XDG/HOME state-path handling.

For the detailed implementation history and completed milestones, see [`docs/ROADMAP.md`](docs/ROADMAP.md).

## Architecture

The project uses a capability-oriented architecture that separates system intelligence from controlled actions.

### System intelligence

```text
Linux system interfaces
        |
        v
Status and observation crates
        |
        v
Subsystem intelligence
        |
        v
Unified system intelligence
        |
        v
Monitoring and health evaluation
        |
        v
Desktop presentation
```

### Controlled action lifecycle

```text
User intent
    |
    v
Action preview
    |
    v
Explicit confirmation
    |
    v
Eligibility and policy evaluation
    |
    v
Scoped authorization
    |
    v
Allowlisted execution
    |
    v
Result verification
    |
    v
Outcome derivation and audit
```

These paths have distinct responsibilities. System observations remain read-only by default. Actions require explicit user intent and must pass the applicable safety boundaries.

## Technology stack

| Area | Technology |
|---|---|
| Core and system logic | Rust |
| Desktop framework | Tauri 2 |
| Frontend | React 19, TypeScript |
| Frontend build | Vite |
| Serialization | Serde and JSON |
| Linux integration | Linux system interfaces and platform facilities |
| Testing | Rust unit and integration tests, Rust CI and Desktop CI |

The project integrates with Linux capabilities through dedicated components rather than treating arbitrary shell execution as its application interface.

## Repository layout

```text
linux-powerhouse/
├── apps/
│   └── desktop/             # Tauri desktop application
├── crates/                  # Rust capability and intelligence crates
├── docs/
│   ├── architecture/
│   ├── security/
│   └── tools/
├── schemas/                 # Machine-readable schemas
├── scripts/                 # Development and CI helpers
├── tests/                   # Integration and security tests
└── .github/                 # CI and repository automation
```

## Development roadmap

The project is being developed incrementally, with focused changes and validation through Rust and Desktop CI.

Current and future areas of development include:

- Continued system intelligence and health improvements.
- Expanded subsystem evidence and cross-subsystem correlation.
- Carefully scoped system-action capabilities.
- Security posture and network intelligence.
- Explainable resource optimization.
- Bounded automation workflows.
- Localization and broader desktop usability.
- A capability-oriented Linux tool ecosystem.

These are development directions, not claims that all capabilities are already implemented.

See [`docs/ROADMAP.md`](docs/ROADMAP.md) for the implementation snapshot, completed milestones, development principles, and future expansion areas.

## Development principles

- **Local-first:** system observations and preferences remain local unless an explicit sharing or export operation is requested.
- **Read-only by default:** monitoring and intelligence do not automatically mutate system state.
- **Explicit actions:** operations must be previewed, confirmed, narrowly scoped, and auditable.
- **Deterministic safety:** policies and authorization boundaries must not depend on AI judgment.
- **Incremental delivery:** changes are developed and validated in focused steps.
- **Bounded persistence:** local history has explicit retention and recovery behavior.
- **No silent privilege escalation:** privileged operations require explicit intent and appropriate authorization.

## Contributing

Linux Powerhouse is being developed in public. Contributions involving architecture, Rust, Linux integration, testing, documentation, security review, and desktop UX are welcome.

Please review the repository roadmap and existing project conventions before proposing substantial changes.

## License

Linux Powerhouse is licensed under the Apache License 2.0. See [`LICENSE`](LICENSE) for details.

Third-party dependencies and integrations may have their own licenses and terms. They remain subject to their respective projects and should be reviewed before redistribution.
