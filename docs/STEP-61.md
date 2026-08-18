# Step 61 — Alert Process Evidence

Status: **In progress**

## Goal

Extend alert history so CPU and memory alerts retain a small, deterministic snapshot of the processes that were the largest observed contributors when the alert event was recorded.

## Acceptance goals

- Capture bounded process evidence when a CPU or memory alert event is created.
- Rank CPU alerts by cumulative process CPU time and memory alerts by resident-memory percentage.
- Retain at most five process contributors per alert.
- Store PID, process name, memory percentage, CPU time ticks, and rank with the alert event.
- Persist the evidence with existing local alert history.
- Display process contributors alongside the existing alert explanation and performance evidence.
- Clearly describe contributors as observations, not proof of causation.
- Keep collection local, deterministic, read-only, bounded, and presentation-only.
- Do not change alert policy, snoozing, dismissal, remediation, or retention behavior.
- Preserve compatibility with older alert history records through serde defaults.

## Scope

This step deliberately does not attempt to reconstruct instantaneous historical CPU percentages for processes. The existing process intelligence model exposes cumulative CPU ticks, so the UI presents those values as contributor evidence rather than causal attribution.

## Safety

Process collection reads `/proc` only. It does not terminate, pause, signal, modify, or otherwise control processes. Collection is bounded to a maximum scan and five retained contributors.
