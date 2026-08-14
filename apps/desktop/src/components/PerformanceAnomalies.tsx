import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

type Level = "Normal" | "Elevated" | "Significant";
type Metric = "Cpu" | "Memory" | "StorageRead" | "StorageWrite" | "ProcessCount" | "RunningProcesses";

type PerformanceAnomaly = {
  metric: Metric;
  level: Level;
  current_value: number;
  baseline_value: number;
  deviation: number;
  explanation: string;
};

type PerformanceAnomalyReport = {
  overall: Level;
  anomalies: PerformanceAnomaly[];
  summary: string;
};

const labels: Record<Metric, string> = {
  Cpu: "CPU utilization",
  Memory: "Memory utilization",
  StorageRead: "Storage read throughput",
  StorageWrite: "Storage write throughput",
  ProcessCount: "Process count",
  RunningProcesses: "Running processes",
};

const levelClass = (level: Level) => level.toLowerCase();

export function PerformanceAnomalies() {
  const [report, setReport] = useState<PerformanceAnomalyReport | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let active = true;
    const load = async () => {
      try {
        const next = await invoke<PerformanceAnomalyReport>("performance_anomaly_explanations");
        if (active) {
          setReport(next);
          setError(null);
        }
      } catch (err) {
        if (active) setError(String(err));
      }
    };
    void load();
    const timer = window.setInterval(() => void load(), 1000);
    return () => {
      active = false;
      window.clearInterval(timer);
    };
  }, []);

  if (error) return <article className="card"><span className="label">PERFORMANCE EXPLANATIONS</span><span className="muted">{error}</span></article>;
  if (!report) return <article className="card"><span className="label">PERFORMANCE EXPLANATIONS</span><span className="muted">Collecting performance history…</span></article>;

  return <article className="card">
    <div className="monitor-heading">
      <div>
        <span className="label">PERFORMANCE EXPLANATIONS</span>
        <strong>{report.overall}</strong>
      </div>
    </div>
    <p className="muted">{report.summary}</p>
    <div className="performance-anomalies">
      {report.anomalies.map((item) => <div className="rate-row" key={item.metric}>
        <div>
          <strong>{labels[item.metric]}</strong>
          <span>{item.explanation}</span>
        </div>
        <b className={`performance-level ${levelClass(item.level)}`}>{item.level}</b>
      </div>)}
    </div>
  </article>;
}
