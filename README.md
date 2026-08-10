# Linux Powerhouse

**Powerful Linux. Simplified for everyone.**

Linux Powerhouse is an open-source, AI-powered desktop toolkit that brings the power of Linux's CLI tools, system utilities, automation, diagnostics, local AI, and open-source ecosystem to everyone through a safe, intuitive graphical experience.

## Vision

Linux already contains an extraordinary collection of powerful tools. The problem is that much of that power remains inaccessible to people who are not comfortable with terminals, command syntax, system administration, or complex configuration.

Linux Powerhouse aims to become the human-friendly layer over that ecosystem:

- **Discover** powerful Linux capabilities without memorizing commands.
- **Understand** what is happening on a Linux system through clear visualizations and explanations.
- **Ask AI** for help using natural language.
- **Act safely** through structured tools, permissions, previews, and verification.
- **Automate** repeatable workflows without turning the application into an unrestricted shell.
- **Extend** the platform through a secure, capability-oriented plugin ecosystem.

## Core principle

> **AI proposes. Tools constrain. Policies decide. Permissions authorize. Linux enforces. Verification confirms. The user remains in control.**

Linux Powerhouse will not treat an AI model as a trusted administrator. AI requests are translated into structured capabilities defined by the Tool Registry and then evaluated by deterministic safety and permission layers before execution.

## Architecture

```text
                         USER
                           │
                           ▼
                    ┌─────────────┐
                    │     GUI     │
                    └──────┬──────┘
                           │
                           ▼
                    ┌─────────────┐
                    │     AI      │
                    │  UNTRUSTED  │
                    └──────┬──────┘
                           │
                    Structured Intent
                           │
                           ▼
                    ┌─────────────┐
                    │    TOOL     │
                    │   REGISTRY  │
                    └──────┬──────┘
                           │
                           ▼
                    ┌─────────────┐
                    │    POLICY   │
                    │    ENGINE   │
                    └──────┬──────┘
                           │
                           ▼
                    ┌─────────────┐
                    │ PERMISSION  │
                    │    ENGINE   │
                    └──────┬──────┘
                           │
                           ▼
                    ┌─────────────┐
                    │  EXECUTION  │
                    │    ENGINE   │
                    └──────┬──────┘
                           │
                           ▼
                         LINUX
                           │
                           ▼
                    ┌─────────────┐
                    │ VERIFICATION│
                    └─────────────┘
```

## Initial technology direction

The first implementation is intentionally small and Linux-native:

- **Desktop:** Tauri
- **Frontend:** React + TypeScript
- **Core:** Rust
- **Persistence:** SQLite
- **Async runtime:** Tokio
- **Linux integration:** D-Bus, systemd, UDisks2, `/proc`, `/sys`
- **Privilege mediation:** Polkit
- **AI runtime:** provider abstraction with local-first support for Ollama and llama.cpp
- **Initial model candidates:** Qwen3 and Phi-family local models, subject to hardware and license validation

External utilities such as `smartmontools`, `ripgrep`, `fd`, `restic`, FFmpeg, Tesseract, and others will be integrated selectively through the Tool Registry rather than copied or reinvented.

## Repository layout

```text
linux-powerhouse/
├── apps/
│   └── desktop/             # Tauri desktop application
├── crates/
│   ├── powerhouse-core/     # Shared domain types and application primitives
│   ├── tool-registry/       # Capability definitions and schemas
│   ├── policy-engine/       # Risk and policy evaluation
│   └── execution-engine/    # Validated tool execution and verification
├── docs/
│   ├── architecture/
│   ├── security/
│   └── tools/
├── schemas/                 # Machine-readable registry schemas
├── scripts/                 # Development and CI helpers
├── tests/                   # Integration and security tests
└── .github/                 # CI, issue templates, and repository automation
```

## Development status

**Early architecture / bootstrap stage.**

The project is currently establishing its core architecture, Tool Registry, AI safety model, and development infrastructure before implementing user-facing capabilities.

## Roadmap

### Phase 1 — Foundation

- [x] Product vision
- [x] MVP architecture
- [x] Tool Registry specification
- [x] AI safety and execution model
- [x] Initial open-source technology inventory
- [ ] Rust workspace and desktop shell
- [ ] Tool Registry MVP
- [ ] Policy engine MVP
- [ ] Execution engine MVP
- [ ] CI and security scanning

### Phase 2 — First useful release

- System dashboard
- Storage analyzer
- Large-file finder
- Duplicate-file finder
- File search
- Disk-health diagnostics
- AI assistant with safe tool calling
- Action history and verification

### Phase 3 — Linux Powerhouse ecosystem

- Backup workflows
- Network diagnostics
- Document/OCR tools
- Media tools
- Application/package management
- Local AI model manager
- Plugin system
- Automation workflows

## Open-source philosophy

Linux Powerhouse will prefer mature open-source technologies over reinventing proven infrastructure. The project's unique contribution is the integration layer: human-friendly UX, AI orchestration, safety, permissions, tool discovery, verification, and automation.

Every dependency and model will be evaluated for license compatibility, security, maintenance health, and suitability for redistribution.

## Contributing

The project is being built in public. Early contributions, architectural discussion, testing, documentation, security review, UX ideas, and Linux distribution feedback are welcome.

Contribution guidelines will be added as the initial development workflow is established.

## License

Linux Powerhouse is licensed under the Apache License 2.0. See [`LICENSE`](LICENSE) for details.

Third-party dependencies and AI models may carry their own licenses and terms. They remain the responsibility of their respective projects and will be documented as integrations are added.
