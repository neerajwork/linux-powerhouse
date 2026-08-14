# Step 49 — Performance Anomaly Explanations

## Scope

Step 49 adds a deterministic, read-only explanation layer on top of the performance baseline and deviation data established by Step 48.

## Behavior

- Classifies CPU and memory deviations as Normal, Elevated, or Significant.
- Classifies storage read/write throughput deviations using local baseline-relative and bounded absolute thresholds.
- Classifies process-count and running-process deviations using bounded local thresholds.
- Produces human-readable explanations for every tracked performance metric.
- Represents insufficient baseline history explicitly without inventing an anomaly.
- Exposes the explanation report through the existing Tauri desktop command layer.
- Surfaces explanations in the existing Monitoring view.

## Safety

- Read-only observation only.
- No remediation is triggered.
- No privileged operations are introduced.
- Existing safe-action confirmation and audit boundaries are unchanged.

## Validation target

- Rust CI green.
- Desktop CI green.
- Focused unit coverage for insufficient history, normal behavior, elevated CPU, significant I/O, and process deviations.
