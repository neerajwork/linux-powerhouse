import { useCallback, useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { AlertHistoryInsights } from "./AlertHistoryInsights";
import { AlertHistorySummary } from "./AlertHistorySummary";

type AlertEvent = {
  timestamp_ms: number;
  performance_timestamp_ms?: number | null;
  kind: "Cpu" | "Memory" | "Swap" | "Storage" | "Network";
  severity: "Warning" | "Critical";
  value: number;
  decision: "Notify" | "Suppressed";
  reason: "ActivePolicy" | "Snoozed" | "Dismissed" | "SnoozeExpired" | "CriticalOverride";
};

type MonitorSnapshot = {
  timestamp_ms: number;
  cpu_percent: number;
  memory_percent: number;
  swap_percent: number;
  storage_read_bytes_per_second: number;
  storage_write_bytes_per_second: number;
  process_count: number;
  running_processes: number;
};

type SeverityFilter = "All" | AlertEvent["severity"];
type CategoryFilter = "All" | AlertEvent["kind"];
type DecisionFilter = "All" | AlertEvent["decision"];
type TimeFilter = "All" | "24h" | "7d" | "30d";

const CORRELATION_WINDOW_MS = 30_000;

const categoryLabel = (kind: AlertEvent["kind"]): string => (kind === "Cpu" ? "CPU" : kind);

const reasonLabel = (reason: AlertEvent["reason"]): string => {
  switch (reason) {
    case "ActivePolicy": return "Active policy";
    case "Snoozed": return "Snoozed";
    case "Dismissed": return "Dismissed";
    case "SnoozeExpired": return "Snooze expired";
    case "CriticalOverride": return "Critical override";
  }
};

const explanation = (event: AlertEvent): { headline: string; detail: string; action: string } => {
  const label = categoryLabel(event.kind);
  const headline = event.severity === "Critical" ? `Critical ${label} alert` : `${label} warning alert`;
  const detail = event.decision === "Notify"
    ? `${label} reached ${event.value.toFixed(1)}%, so the active alert policy notified you.`
    : `${label} reached ${event.value.toFixed(1)}%, but the routine warning was suppressed by the current alert policy.`;

  let action: string;
  switch (event.reason) {
    case "CriticalOverride":
      action = "Critical events remain visible regardless of routine warning preferences.";
      break;
    case "Snoozed":
      action = "The routine warning was snoozed; review the event if the underlying condition persists.";
      break;
    case "Dismissed":
      action = "The routine warning was dismissed; review the history if the condition returns.";
      break;
    case "SnoozeExpired":
      action = "The snooze period had expired, so the routine warning returned to the active policy.";
      break;
    case "ActivePolicy":
      action = "The active alert policy determined how this event was handled.";
      break;
  }

  return { headline, detail, action };
};

const nearestSnapshot = (event: AlertEvent, snapshots: MonitorSnapshot[]): MonitorSnapshot | null => {
  if (event.performance_timestamp_ms == null || snapshots.length === 0) return null;

  return snapshots
    .map((snapshot) => ({
      snapshot,
      age: Math.abs(snapshot.timestamp_ms - event.performance_timestamp_ms!),
    }))
    .filter((item) => item.age <= CORRELATION_WINDOW_MS)
    .sort((a, b) => a.age - b.age)[0]?.snapshot ?? null;
};

const primaryEvidence = (event: AlertEvent, snapshot: MonitorSnapshot): string => {
  switch (event.kind) {
    case "Cpu": return `CPU utilization was ${snapshot.cpu_percent.toFixed(1)}% near the alert.`;
    case "Memory": return `Memory utilization was ${snapshot.memory_percent.toFixed(1)}% near the alert.`;
    case "Swap": return `Swap utilization was ${snapshot.swap_percent.toFixed(1)}% near the alert.`;
    case "Storage": return `Storage I/O was ${snapshot.storage_read_bytes_per_second.toFixed(1)} read / ${snapshot.storage_write_bytes_per_second.toFixed(1)} write bytes/s near the alert.`;
    case "Network": return `Network activity was sampled near the alert; ${snapshot.process_count} processes were present at that time.`;
  }
};

export function AlertEventHistory() {
  const [events, setEvents] = useState<AlertEvent[]>([]);
  const [snapshots, setSnapshots] = useState<MonitorSnapshot[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [severity, setSeverity] = useState<SeverityFilter>("All");
  const [category, setCategory] = useState<CategoryFilter>("All");
  const [decision, setDecision] = useState<DecisionFilter>("All");
  const [time, setTime] = useState<TimeFilter>("All");

  const load = useCallback(async () => {
    try {
      setError(null);
      const [history, monitorHistory] = await Promise.all([
        invoke<AlertEvent[]>("alert_event_history"),
        invoke<MonitorSnapshot[]>("monitor_history"),
      ]);
      setEvents(history);
      setSnapshots(monitorHistory);
    } catch (err) {
      setError(String(err));
    }
  }, []);

  useEffect(() => {
    void load();
    const timer = window.setInterval(() => void load(), 5000);
    return () => window.clearInterval(timer);
  }, [load]);

  const filteredEvents = useMemo(() => {
    const now = Date.now();
    const cutoff = time === "24h"
      ? now - 24 * 60 * 60 * 1000
      : time === "7d"
        ? now - 7 * 24 * 60 * 60 * 1000
        : time === "30d"
          ? now - 30 * 24 * 60 * 60 * 1000
          : 0;

    return events.filter(
      (event) =>
        (severity === "All" || event.severity === severity) &&
        (category === "All" || event.kind === category) &&
        (decision === "All" || event.decision === decision) &&
        event.timestamp_ms >= cutoff,
    );
  }, [events, severity, category, decision, time]);

  async function clearHistory() {
    try {
      setError(null);
      await invoke("clear_alert_event_history");
      setEvents([]);
    } catch (err) {
      setError(String(err));
    }
  }

  function resetFilters() {
    setSeverity("All");
    setCategory("All");
    setDecision("All");
    setTime("All");
  }

  const filtersActive = severity !== "All" || category !== "All" || decision !== "All" || time !== "All";

  return (
    <section className="alert-history" aria-labelledby="alert-history-title">
      <div className="alert-history__header">
        <div>
          <p className="eyebrow">ALERT HISTORY</p>
          <h2 id="alert-history-title">Alert event history</h2>
          <p className="subtitle">Review warning and critical decisions with focused local filters, deterministic explanations, and performance evidence.</p>
        </div>
        <button className="secondary" onClick={() => void clearHistory()} disabled={!events.length}>Clear history</button>
      </div>

      <AlertHistorySummary events={events} />
      <AlertHistoryInsights events={events} />

      <div className="alert-history__filters" aria-label="Alert history filters">
        <label>Severity<select value={severity} onChange={(event) => setSeverity(event.target.value as SeverityFilter)}><option value="All">All</option><option value="Warning">Warning</option><option value="Critical">Critical</option></select></label>
        <label>Category<select value={category} onChange={(event) => setCategory(event.target.value as CategoryFilter)}><option value="All">All</option><option value="Cpu">CPU</option><option value="Memory">Memory</option><option value="Swap">Swap</option><option value="Storage">Storage</option><option value="Network">Network</option></select></label>
        <label>Decision<select value={decision} onChange={(event) => setDecision(event.target.value as DecisionFilter)}><option value="All">All</option><option value="Notify">Notified</option><option value="Suppressed">Suppressed</option></select></label>
        <label>Time<select value={time} onChange={(event) => setTime(event.target.value as TimeFilter)}><option value="All">All time</option><option value="24h">Last 24 hours</option><option value="7d">Last 7 days</option><option value="30d">Last 30 days</option></select></label>
        <button className="secondary" onClick={resetFilters} disabled={!filtersActive}>Reset filters</button>
      </div>

      {error && <div className="error">Unable to read alert history: {error}</div>}
      {!filteredEvents.length && !error && <article className="card"><strong>{events.length ? "No matching alert events." : "No alert events recorded yet."}</strong><span>{events.length ? "Adjust or reset the filters to broaden the history view." : "New warning and critical decisions will appear here."}</span></article>}
      {!!filteredEvents.length && <div className="list alert-history__list">{[...filteredEvents].reverse().map((event, index) => {
        const why = explanation(event);
        const snapshot = nearestSnapshot(event, snapshots);
        const evidence = snapshot ? primaryEvidence(event, snapshot) : "No matching performance snapshot was available for this alert.";
        const evidenceAge = snapshot && event.performance_timestamp_ms != null
          ? Math.abs(snapshot.timestamp_ms - event.performance_timestamp_ms)
          : null;

        return (
          <article className="row" key={`${event.timestamp_ms}-${event.kind}-${index}`}>
            <div>
              <strong>{categoryLabel(event.kind)} · {event.severity}</strong>
              <span>{new Date(event.timestamp_ms).toLocaleString()} · {event.value.toFixed(1)}%</span>
              <small>{reasonLabel(event.reason)}</small>
              <details>
                <summary>Why did this alert happen?</summary>
                <div className="alert-history__explanation">
                  <strong>{why.headline}</strong>
                  <span>{why.detail}</span>
                  <small>{why.action}</small>
                  <div className="alert-history__evidence">
                    <strong>Performance evidence</strong>
                    <span>{evidence}</span>
                    {snapshot && evidenceAge != null && (
                      <small>
                        Snapshot {evidenceAge} ms from the alert context · memory {snapshot.memory_percent.toFixed(1)}% · swap {snapshot.swap_percent.toFixed(1)}% · {snapshot.process_count} processes · {snapshot.running_processes} running
                      </small>
                    )}
                  </div>
                </div>
              </details>
            </div>
            <b>{event.decision === "Notify" ? "Notified" : "Suppressed"}</b>
          </article>
        );
      })}</div>}
      <small className="monitor-note">Showing {filteredEvents.length} of {events.length} retained events. Filtering, explanations, and evidence only change the view; history remains local, persistent, and bounded to the latest 100 events.</small>
    </section>
  );
}
