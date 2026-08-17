# Step 55 — Alert History Controls

Add focused local filtering to the persistent Alert History view.

## Scope

- Filter by severity: all, warning, critical.
- Filter by category: all, CPU, memory, swap, storage, network.
- Filter by decision: all, notified, suppressed.
- Filter by time: all, 24 hours, 7 days, 30 days.
- Reset all filters explicitly.
- Keep filtering client-side and read-only; no changes to stored history.
- Preserve the existing bounded persistent history and clear-history behavior.

## Safety

Filtering changes presentation only. It does not suppress alerts, delete events, or mutate system state.

## Status

In progress.
