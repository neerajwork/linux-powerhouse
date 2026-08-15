# Step 51 — Performance History Comparison

## Scope

Step 51 adds a deterministic, read-only comparison between the most recent local performance period and the immediately preceding local period. It reuses the existing bounded monitoring history rather than introducing a second history store.

## Included

- Compare CPU, memory, storage read/write throughput, process count, and running-process count.
- Use two adjacent bounded windows of 10 samples each.
- Report current and previous averages, absolute deltas, percentage deltas, and direction.
- Handle insufficient history safely.
- Handle a zero previous average without producing an infinite or undefined percentage.
- Expose the comparison through the existing Tauri performance command.
- Present the comparison in the existing Desktop performance surface.

## Interpretation boundary

Step 51 reports historical change; it does not automatically classify every increase as a problem. Higher storage or CPU activity can be legitimate workload. Anomaly interpretation remains the responsibility of the existing performance explanation layer.

## Safety boundary

Step 51 remains local-first and read-only. It does not execute system actions, change process state, require privileges, or send telemetry externally.

## Relationship to alert controls

Future alert-control work may allow users to dismiss or snooze routine warnings. Such controls must suppress notifications rather than disable monitoring, and genuinely critical events must remain eligible for notification regardless of routine-warning snooze preferences.
