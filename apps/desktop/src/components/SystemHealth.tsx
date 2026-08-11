import { useCallback, useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

type HealthLevel = "Healthy" | "Attention" | "Degraded";
type Subsystem = "Storage" | "Processes" | "Network" | "Services";
type SystemIntelligenceSnapshot = {
  health: HealthLevel;
  storage_anomalies: number;
  process_anomalies: number;
  network_anomalies: number;
  service_anomalies: number;
  total_anomalies: number;
};
type HistoryPoint = SystemIntelligenceSnapshot & { timestamp: number };

const MAX_HISTORY = 60;
const healthClass: Record<HealthLevel, string> = {
  Healthy: "system-health__status system-health__status--healthy",
  Attention: "system-health__status system-health__status--attention",
  Degraded: "system-health__status system-health__status--degraded",
};
const guidance: Record<Subsystem, { healthy: string; action: string }> = {
  Storage: { healthy: "No storage anomaly signals were reported.", action: "Review filesystem usage and available space." },
  Processes: { healthy: "No process anomaly signals were reported.", action: "Review the process list for unusual activity or resource use." },
  Network: { healthy: "No network anomaly signals were reported.", action: "Review network interfaces and connectivity status." },
  Services: { healthy: "No service anomaly signals were reported.", action: "Review service status for any unavailable or degraded service." },
};

export function SystemHealth() {
  const [snapshot, setSnapshot] = useState<SystemIntelligenceSnapshot | null>(null);
  const [history, setHistory] = useState<HistoryPoint[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<SystemIntelligenceSnapshot>("system_intelligence", { storageRoot: "/" });
      setSnapshot(result);
      setHistory((current) => [...current, { ...result, timestamp: Date.now() }].slice(-MAX_HISTORY));
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => { void refresh(); }, [refresh]);

  const signals: Array<[Subsystem, number]> = snapshot
    ? [["Storage", snapshot.storage_anomalies], ["Processes", snapshot.process_anomalies], ["Network", snapshot.network_anomalies], ["Services", snapshot.service_anomalies]]
    : [];

  const trend = useMemo(() => {
    if (history.length < 2) return "Collecting trend data…";
    const first = history[0].total_anomalies;
    const latest = history[history.length - 1].total_anomalies;
    if (latest < first) return "Improving";
    if (latest > first) return "Worsening";
    return "Stable";
  }, [history]);

  const persistentSignals = useMemo(() => {
    if (history.length < 3) return 0;
    return ["storage_anomalies", "process_anomalies", "network_anomalies", "service_anomalies"].filter((key) => {
      const recent = history.slice(-3);
      return recent.every((point) => point[key as keyof SystemIntelligenceSnapshot] as number > 0);
    }).length;
  }, [history]);

  return (
    <section className="system-health" aria-labelledby="system-health-title">
      <div className="system-health__header">
        <div><p className="eyebrow">SYSTEM INTELLIGENCE</p><h2 id="system-health-title">System Health</h2><p className="subtitle">A read-only snapshot across storage, processes, network, and services.</p></div>
        <button className="primary" type="button" onClick={() => void refresh()} disabled={loading}>{loading ? "Refreshing…" : "Refresh"}</button>
      </div>
      {error ? <p role="alert" className="error">Unable to load system health: {error}</p> : loading && !snapshot ? <p className="system-health__loading">Reading system health…</p> : snapshot ? <>
        <div className="system-health__summary"><span className={healthClass[snapshot.health]}>{snapshot.health}</span><span>{snapshot.total_anomalies === 0 ? "No anomaly signals detected" : `${snapshot.total_anomalies} signal(s) requiring attention`}</span></div>
        <div className="system-health__grid">{signals.map(([label, value]) => <article className="system-health__signal card" key={label}><span className="label">{label.toUpperCase()}</span><strong>{value}</strong><span>{value === 0 ? "No signals detected" : "Signal(s) detected"}</span></article>)}</div>
        <div className="system-health__trend card"><div><p className="label">RECENT TREND</p><strong>{trend}</strong><span>{history.length} snapshots retained in memory</span></div><div><p className="label">PERSISTENT SIGNALS</p><strong>{persistentSignals}</strong><span>subsystems signaled in each of the last 3 snapshots</span></div></div>
        <div className="system-health__history card"><p className="label">HEALTH HISTORY</p><div className="system-health__history-bars" aria-label="Recent anomaly signal history">{history.map((point) => <span key={point.timestamp} className={`history-bar history-bar--${point.health.toLowerCase()}`} title={`${new Date(point.timestamp).toLocaleTimeString()} — ${point.total_anomalies} signal(s)`} style={{ height: `${Math.max(8, Math.min(100, point.total_anomalies * 20 + 8))}%` }} />)}</div><span>Each bar represents one read-only snapshot. History is bounded to the latest {MAX_HISTORY} snapshots.</span></div>
        <div className="system-health__guidance" aria-label="System health guidance"><div><p className="label">WHAT TO DO NEXT</p><p className="system-health__guidance-summary">{snapshot.total_anomalies === 0 ? "The current snapshot is healthy. No action is suggested." : "Use the subsystem guidance below to decide what to inspect next. These suggestions do not change system state."}</p></div><div className="system-health__guidance-grid">{signals.map(([label, value]) => <div className="system-health__guidance-item" key={`${label}-guidance`}><strong>{label}</strong><span>{value === 0 ? guidance[label].healthy : guidance[label].action}</span></div>)}</div></div>
      </> : null}
    </section>
  );
}
