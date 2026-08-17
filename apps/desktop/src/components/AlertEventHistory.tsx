import { useCallback, useEffect, useState } from "react";
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

  async function clearHistory() {
    try {
      setError(null);
      await invoke("clear_alert_event_history");
      setEvents([]);
    } catch (err) {
      setError(String(err));
    }
  }

  return (
    <section className="alert-history" aria-labelledby="alert-history-title">
      <div className="alert-history__header">
        <div>
          <p className="eyebrow">ALERT HISTORY</p>
          <h2 id="alert-history-title">Alert event history</h2>
          <p className="subtitle">
            A bounded local record of warning and critical alert decisions.
          </p>
        </div>
        <button className="secondary" onClick={() => void clearHistory()} disabled={!events.length}>
          Clear history
        </button>
      </div>

      {error && <div className="error">Unable to read alert history: {error}</div>}

      {!events.length && !error && (
        <article className="card">
          <strong>No alert events recorded yet.</strong>
          <span>New warning and critical decisions will appear here.</span>
        </article>
      )}

      {!!events.length && (
        <div className="list alert-history__list">
          {[...events].reverse().map((event, index) => (
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
        History is retained locally in memory and is bounded to the latest 100 events. Critical events are never suppressed by alert preferences.
      </small>
    </section>
  );
}
