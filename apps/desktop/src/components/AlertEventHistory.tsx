import { useCallback, useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

type AlertEvent = {
  timestamp_ms: number;
  kind: "Cpu" | "Memory" | "Swap" | "Storage" | "Network";
  severity: "Warning" | "Critical";
  value: number;
  decision: "Notify" | "Suppressed";
  reason:
    | "ActivePolicy"
    | "Snoozed"
    | "Dismissed"
    | "SnoozeExpired"
    | "CriticalOverride";
};

type SeverityFilter = "All" | AlertEvent["severity"];
type CategoryFilter = "All" | AlertEvent["kind"];
type DecisionFilter = "All" | AlertEvent["decision"];
type TimeFilter = "All" | "24h" | "7d" | "30d";

const categoryLabel = (kind: AlertEvent["kind"]): string =>
  kind === "Cpu" ? "CPU" : kind;

const reasonLabel = (reason: AlertEvent["reason"]): string => {
  switch (reason) {
    case "ActivePolicy":
      return "Active policy";
    case "Snoozed":
      return "Snoozed";
    case "Dismissed":
      return "Dismissed";
    case "SnoozeExpired":
      return "Snooze expired";
    case "CriticalOverride":
      return "Critical override";
  }
};

export function AlertEventHistory() {
  const [events, setEvents] = useState<AlertEvent[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [severity, setSeverity] = useState<SeverityFilter>("All");
  const [category, setCategory] = useState<CategoryFilter>("All");
  const [decision, setDecision] = useState<DecisionFilter>("All");
  const [time, setTime] = useState<TimeFilter>("All");

  const load = useCallback(async () => {
    try {
      setError(null);
      setEvents(await invoke<AlertEvent[]>("alert_event_history"));
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
    const cutoff =
      time === "24h"
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
          <p className="subtitle">
            Review warning and critical decisions with focused local filters.
          </p>
        </div>
        <button className="secondary" onClick={() => void clearHistory()} disabled={!events.length}>
          Clear history
        </button>
      </div>

      <div className="alert-history__filters" aria-label="Alert history filters">
        <label>
          Severity
          <select value={severity} onChange={(event) => setSeverity(event.target.value as SeverityFilter)}>
            <option value="All">All</option>
            <option value="Warning">Warning</option>
            <option value="Critical">Critical</option>
          </select>
        </label>
        <label>
          Category
          <select value={category} onChange={(event) => setCategory(event.target.value as CategoryFilter)}>
            <option value="All">All</option>
            <option value="Cpu">CPU</option>
            <option value="Memory">Memory</option>
            <option value="Swap">Swap</option>
            <option value="Storage">Storage</option>
            <option value="Network">Network</option>
          </select>
        </label>
        <label>
          Decision
          <select value={decision} onChange={(event) => setDecision(event.target.value as DecisionFilter)}>
            <option value="All">All</option>
            <option value="Notify">Notified</option>
            <option value="Suppressed">Suppressed</option>
          </select>
        </label>
        <label>
          Time
          <select value={time} onChange={(event) => setTime(event.target.value as TimeFilter)}>
            <option value="All">All time</option>
            <option value="24h">Last 24 hours</option>
            <option value="7d">Last 7 days</option>
            <option value="30d">Last 30 days</option>
          </select>
        </label>
        <button className="secondary" onClick={resetFilters} disabled={!filtersActive}>
          Reset filters
        </button>
      </div>

      {error && <div className="error">Unable to read alert history: {error}</div>}

      {!filteredEvents.length && !error && (
        <article className="card">
          <strong>{events.length ? "No matching alert events." : "No alert events recorded yet."}</strong>
          <span>{events.length ? "Adjust or reset the filters to broaden the history view." : "New warning and critical decisions will appear here."}</span>
        </article>
      )}

      {!!filteredEvents.length && (
        <div className="list alert-history__list">
          {[...filteredEvents].reverse().map((event, index) => (
            <article className="row" key={`${event.timestamp_ms}-${event.kind}-${index}`}>
              <div>
                <strong>{categoryLabel(event.kind)} · {event.severity}</strong>
                <span>
                  {new Date(event.timestamp_ms).toLocaleString()} · {event.value.toFixed(1)}%
                </span>
                <small>{reasonLabel(event.reason)}</small>
              </div>
              <b>{event.decision === "Notify" ? "Notified" : "Suppressed"}</b>
            </article>
          ))}
        </div>
      )}

      <small className="monitor-note">
        Showing {filteredEvents.length} of {events.length} retained events. Filtering only changes the view; history remains local, persistent, and bounded to the latest 100 events.
      </small>
    </section>
  );
}
