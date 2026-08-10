# Development Environment

## Prerequisites

Linux Powerhouse is developed on Linux first.

Install:

- Git
- Rust stable toolchain with `rustfmt` and `clippy`
- Node.js LTS and npm (required when the desktop frontend is introduced)
- SQLite development tooling
- Tauri system dependencies for the target distribution

## Verify the Rust workspace

```bash
cargo fmt --all -- --check
cargo check --workspace --all-features
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

## Development rules

1. Keep Linux-specific integrations behind small, testable interfaces.
2. Do not add direct shell execution to the AI layer.
3. Add new capabilities through the Tool Registry.
4. Add a policy test for every new risk-sensitive capability.
5. Prefer native Linux APIs over parsing human-oriented CLI output when practical.
6. Keep dependencies minimal and document why each significant dependency exists.
7. Do not commit model weights, credentials, build artifacts, or local environment files.

## Branching

Use short-lived branches such as:

```text
feature/tool-storage-analyzer
feature/ai-provider-interface
fix/path-validation
```

Pull requests should target `main` until a separate integration branch is justified by project scale.
