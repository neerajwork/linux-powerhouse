# Development Checks

## Rust formatting

Before opening or updating a pull request, format Rust code with:

```bash
./scripts/format.sh
```

Then run the local preflight:

```bash
./scripts/preflight.sh
```

The preflight runs the same Rust formatting, Clippy, test, and check commands enforced by CI. This is intended to catch routine failures locally before they become CI failures.

If the local environment cannot run the preflight, the existing CI workflow remains the authoritative validation gate.
