# Step 13 — Process Intelligence

Process Intelligence enriches the existing read-only process inventory with bounded analysis of process hierarchy, memory concentration, cumulative CPU time, and deterministic anomaly signals.

## Safety boundary

- Read-only `/proc` inspection only.
- Maximum 500 processes per analysis by default.
- Maximum 20 reported consumers/anomalies by default.
- No process termination, pausing, priority changes, signals, or service manipulation.
- The `process.analyze` capability is low risk and requires explicit user confirmation.
- Anomalies are deterministic signals; future AI features may explain or recommend actions but do not receive direct process-control primitives.
