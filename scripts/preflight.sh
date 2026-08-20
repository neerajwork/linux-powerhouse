#!/usr/bin/env bash
set -euo pipefail

# Run the same Rust checks used by CI before opening or updating a PR.
echo "==> Checking Rust formatting"
cargo fmt --all -- --check

echo "==> Running Clippy"
cargo clippy --workspace --exclude linux-powerhouse-desktop --all-targets --all-features -- -D warnings

echo "==> Running tests"
cargo test --workspace --exclude linux-powerhouse-desktop --all-features

echo "==> Running cargo check"
cargo check --workspace --exclude linux-powerhouse-desktop --all-features

echo "==> Rust preflight passed"
