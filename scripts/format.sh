#!/usr/bin/env bash
set -euo pipefail

# Format all Rust code using the same formatter enforced by CI.
cargo fmt --all
