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

type ProcessInsight = {
  pid: number;
  name: string;
  state: string;
  parent_pid: number;
  memory_bytes: number;
  memory_percent: number;
  cpu_time_ticks: number;
  child_count: number;
  anomaly: string | null;
};

type PerformanceAnomalyReport = {
  overall: Level;
  anomalies: PerformanceAnomaly[];
  summary: string;
};

type ProcessAnalysis = {
  total_processes: number;
  entries_scanned: number;
  skipped_entries: number;
  truncated: boolean;
  zombie_count: number;
  top_consumers: ProcessInsight[];
  top_cpu_consumers: ProcessInsight[];
  anomalies: ProcessInsight[];
};

type PerformanceDrilldown = {
  performance: PerformanceAnomalyReport;
  processes: ProcessAnalysis;
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
const formatMemory = (bytes: number) => {
  if (bytes >= 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
  if (bytes >= 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / 1024).toFixed(1)} KB`;
};

export function PerformanceAnomalies() {
  const [report, setReport] = useState<PerformanceDrilldown | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let active = true;
    const load = async () => {
      try {
        const next = await invoke<PerformanceDrilldown>("process_performance_drilldown");
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
        <strong>{report.performance.overall}</strong>
      </div>
    </div>
    <p className="muted">{report.performance.summary}</p>
    <div className="performance-anomalies">
      {report.performance.anomalies.map((item) => <div className="rate-row" key={item.metric}>
        <div>
          <strong>{labels[item.metric]}</strong>
          <span>{item.explanation}</span>
        </div>
        <b className={`performance-level ${levelClass(item.level)}`}>{item.level}</b>
      </div>)}
    </div>

    <div className="monitor-heading">
      <div>
        <span className="label">PROCESS PERFORMANCE</span>
        <strong>{report.processes.total_processes} processes</strong>
      </div>
      <span className="muted">{report.processes.zombie_count} zombies · {report.processes.skipped_entries} skipped</span>
    </div>
    <p className="muted">Top process consumers are ranked from the current read-only Linux process snapshot. CPU ordering uses cumulative process CPU time; it does not trigger or modify processes.</p>
    <div className="performance-anomalies">
      {report.processes.top_cpu_consumers.slice(0, 8).map((process) => <div className="rate-row" key={process.pid}>
        <div>
          <strong>{process.name} · PID {process.pid}</strong>
          <span>CPU time {process.cpu_time_ticks} ticks · Memory {formatMemory(process.memory_bytes)} ({process.memory_percent.toFixed(1)}%) · {process.child_count} children</span>
        </div>
        {process.anomaly && <b className="performance-level elevated">{process.anomaly}</b>}
      </div>)}
    </div>
  </article>;
}
