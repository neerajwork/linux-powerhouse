# Step 50 — Process Performance Drill-down

## Scope

Step 50 extends performance intelligence from aggregate anomaly explanations into a bounded, read-only process-level view. It identifies current process consumers associated with resource activity without terminating, pausing, reprioritizing, or otherwise mutating processes.

## Included

- Bounded Linux process analysis using the existing process-intelligence crate.
- Current process CPU-time ranking using cumulative `/proc/<pid>/stat` CPU ticks.
- Memory usage, process state, parent PID, child count, and deterministic process anomalies.
- Desktop exposure through the existing performance explanation surface.
- Explicit presentation that CPU ranking is cumulative rather than interval CPU utilization.
- No privileged operations and no automatic remediation.

## Safety boundary

Step 50 remains **Observe → Compare → Explain**. It does not execute process actions. Any future process remediation must continue through the existing explicit-confirmation and safe-action architecture.
