# Step 58 — Alert Performance Correlation

## Goal

Connect retained alert events to the nearest available read-only performance snapshot so an alert can eventually be explained using the system conditions that surrounded it.

## Scope

- Add an optional performance-snapshot timestamp to alert events.
- Preserve backward compatibility with existing persisted alert history.
- Add a deterministic correlation model in `health-status`.
- Match an alert to the nearest monitoring snapshot within a 30-second bounded window.
- Report CPU, memory, swap, storage I/O, process counts, and a concise primary evidence statement.
- Return no correlation when performance context is unavailable or outside the bounded window.

## Safety

- Read-only correlation only.
- No alert decisions are changed.
- No alert history is deleted or modified by the correlation algorithm.
- No external analytics or network service is introduced.
- No automatic remediation is triggered.

## Compatibility

Existing alert-history JSON remains readable because the new performance timestamp is optional and defaults to `None` when loading older records.

## Validation

The correlation module includes tests for nearest-snapshot selection, bounded-window rejection, and missing performance context.
