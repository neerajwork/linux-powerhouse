import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

type HealthLevel = "Healthy" | "Attention" | "Degraded";

type SystemIntelligenceSnapshot = {
  health: HealthLevel;
  storage_anomalies: number;
  process_anomalies: number;
  network_anomalies: number;
  service_anomalies: number;
  total_anomalies: number;
};

const healthClass: Record<HealthLevel, string> = {
  Healthy: "system-health__status system-health__status--healthy",
  Attention: "system-health__status system-health__status--attention",
  Degraded: "system-health__status system-health__status--degraded",
};

export function SystemHealth() {
  const [snapshot, setSnapshot] = useState<SystemIntelligenceSnapshot | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<SystemIntelligenceSnapshot>("system_intelligence", {
        storageRoot: "/",
      });
      setSnapshot(result);
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  return (
    <section className="system-health" aria-labelledby="system-health-title">
      <div className="system-health__header">
        <div>
          <p className="eyebrow">SYSTEM INTELLIGENCE</p>
          <h2 id="system-health-title">System Health</h2>
          <p className="subtitle">A read-only snapshot across storage, processes, network, and services.</p>
        </div>
        <button className="primary" type="button" onClick={() => void refresh()} disabled={loading}>
          {loading ? "Refreshing…" : "Refresh"}
        </button>
      </div>

      {error ? (
        <p role="alert" className="error">Unable to load system health: {error}</p>
      ) : loading && !snapshot ? (
        <p className="system-health__loading">Reading system health…</p>
      ) : snapshot ? (
        <>
          <div className="system-health__summary">
            <span className={healthClass[snapshot.health]}>{snapshot.health}</span>
            <span>{snapshot.total_anomalies} signal(s) requiring attention</span>
          </div>
          <div className="system-health__grid">
            <Signal label="Storage" value={snapshot.storage_anomalies} />
            <Signal label="Processes" value={snapshot.process_anomalies} />
            <Signal label="Network" value={snapshot.network_anomalies} />
            <Signal label="Services" value={snapshot.service_anomalies} />
          </div>
        </>
      ) : null}
    </section>
  );
}

function Signal({ label, value }: { label: string; value: number }) {
  return (
    <article className="system-health__signal card">
      <span className="label">{label.toUpperCase()}</span>
      <strong>{value}</strong>
      <span>{value === 0 ? "No signals detected" : "Signal(s) detected"}</span>
    </article>
  );
}
