import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ActionAudit } from "./components/ActionAudit";
import { HealthActionWorkspace } from "./components/HealthActionWorkspace";
import { PerformanceAnomalies } from "./components/PerformanceAnomalies";
import { SystemHealth } from "./components/SystemHealth";

type OperatingSystem = { id: string | null; name: string | null; version: string | null };
type SystemStatus = { operating_system: OperatingSystem; kernel_version: string; architecture: string; hostname: string; cpu_model: string | null; cpu_logical_cores: number; memory_total_bytes: number; memory_available_bytes: number; swap_total_bytes: number; swap_free_bytes: number; uptime_seconds: number };
type FilesystemStatus = { mount_point: string; total_bytes: number; available_bytes: number; used_bytes: number; usage_percent: number };
type ProcessInfo = { pid: number; name: string; state: string; memory_bytes: number };
type NetworkInterface = { name: string; is_up: boolean; rx_bytes: number; tx_bytes: number };
type MonitorSnapshot = { timestamp_ms: number; cpu_percent: number; memory_percent: number; swap_percent: number; network: { name: string; rx_bytes_per_second: number; tx_bytes_per_second: number }[] };
type Section = "Dashboard" | "System Health" | "Health Workspace" | "Monitoring" | "Storage" | "Processes" | "Network" | "Action Audit";

const formatBytes = (bytes: number) => { const units = ["B", "KB", "MB", "GB", "TB"]; let value = bytes; let index = 0; while (value >= 1024 && index < units.length - 1) { value /= 1024; index += 1; } return `${value.toFixed(index === 0 ? 0 : 1)} ${units[index]}`; };
const formatRate = (bytes: number) => `${formatBytes(bytes)}/s`;
const formatUptime = (seconds: number) => `${Math.floor(seconds / 86400)}d ${Math.floor((seconds % 86400) / 3600)}h ${Math.floor((seconds % 3600) / 60)}m`;

function Sparkline({ values }: { values: number[] }) { if (values.length < 2) return <div className="spark-empty">Collecting samples…</div>; const width = 420; const height = 100; const max = Math.max(...values, 1); const min = Math.min(...values, 0); const range = Math.max(max - min, 1); const points = values.map((value, index) => `${(index / (values.length - 1)) * width},${height - ((value - min) / range) * height}`).join(" "); return <svg className="sparkline" viewBox={`0 0 ${width} ${height}`} preserveAspectRatio="none"><polyline points={points} fill="none" stroke="currentColor" strokeWidth="2.5" vectorEffect="non-scaling-stroke" /></svg>; }

export default function App() {
  const [section, setSection] = useState<Section>("Dashboard");
  const [system, setSystem] = useState<SystemStatus | null>(null);
  const [storage, setStorage] = useState<FilesystemStatus[]>([]);
  const [processes, setProcesses] = useState<ProcessInfo[]>([]);
  const [network, setNetwork] = useState<NetworkInterface[]>([]);
  const [history, setHistory] = useState<MonitorSnapshot[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function refresh() { setLoading(true); setError(null); try { const [nextSystem, nextStorage, nextProcesses, nextNetwork] = await Promise.all([invoke<SystemStatus>("system_status"), invoke<FilesystemStatus[]>("storage_status"), invoke<ProcessInfo[]>("process_status"), invoke<NetworkInterface[]>("network_status")]); setSystem(nextSystem); setStorage(nextStorage); setProcesses(nextProcesses); setNetwork(nextNetwork); } catch (err) { setError(String(err)); } finally { setLoading(false); } }
  async function sampleMonitoring() { try { const snapshot = await invoke<MonitorSnapshot>("monitor_snapshot"); setHistory((current) => [...current.slice(-119), snapshot]); } catch (err) { setError(String(err)); } }
  useEffect(() => { void refresh(); void sampleMonitoring(); }, []);
  useEffect(() => { const timer = window.setInterval(() => { void sampleMonitoring(); }, 1000); return () => window.clearInterval(timer); }, []);
  const latest = history[history.length - 1];
  const averageCpu = useMemo(() => history.length ? history.reduce((sum, item) => sum + item.cpu_percent, 0) / history.length : 0, [history]);
  const peakCpu = useMemo(() => history.length ? Math.max(...history.map((item) => item.cpu_percent)) : 0, [history]);

  return <main className="app-shell">
    <aside className="sidebar">
      <div className="brand"><div className="brand-mark">LP</div><div><strong>Linux Powerhouse</strong><span>Powerful Linux. Simplified.</span></div></div>
      <nav>
        {(["Dashboard", "System Health", "Health Workspace", "Monitoring", "Storage", "Processes", "Network", "Action Audit"] as Section[]).map((item) => <button key={item} className={`nav-item ${section === item ? "active" : ""}`} onClick={() => setSection(item)}>{item}</button>)}
        <button className="nav-item" disabled>Services</button><button className="nav-item" disabled>Security</button><button className="nav-item" disabled>AI Assistant</button>
      </nav>
    </aside>
    <section className="content">
      <header className="header"><div><p className="eyebrow">{section.toUpperCase()}</p><h1>{section === "Dashboard" ? "Your Linux system, at a glance." : section}</h1><p className="subtitle">{section === "Monitoring" ? "Live read-only metrics with a bounded in-memory history." : section === "Action Audit" ? "Local execution history for explicitly confirmed safe actions." : section === "Health Workspace" ? "A focused, explicit path from health signals to safe diagnostics." : "Read-only insights, collected through Linux-native capabilities."}</p></div><button className="primary" onClick={() => void refresh()} disabled={loading}>{loading ? "Refreshing…" : "Refresh"}</button></header>
      {error && <div className="error">Unable to read system data: {error}</div>}
      {section === "System Health" && <SystemHealth />}
      {section === "Health Workspace" && <HealthActionWorkspace />}
      {section === "Action Audit" && <ActionAudit />}
      {section === "Monitoring" && <section className="monitoring"><div className="monitor-grid"><article className="monitor-card"><div className="monitor-heading"><span className="label">CPU</span><strong>{latest ? `${latest.cpu_percent.toFixed(1)}%` : "—"}</strong></div><Sparkline values={history.map((item) => item.cpu_percent)} /><span className="muted">Average {averageCpu.toFixed(1)}% · Peak {peakCpu.toFixed(1)}%</span></article><article className="monitor-card"><div className="monitor-heading"><span className="label">MEMORY</span><strong>{latest ? `${latest.memory_percent.toFixed(1)}%` : "—"}</strong></div><Sparkline values={history.map((item) => item.memory_percent)} /><span className="muted">Current memory utilization</span></article><article className="monitor-card"><div className="monitor-heading"><span className="label">SWAP</span><strong>{latest ? `${latest.swap_percent.toFixed(1)}%` : "—"}</strong></div><Sparkline values={history.map((item) => item.swap_percent)} /><span className="muted">Current swap utilization</span></article></div><article className="card"><span className="label">NETWORK THROUGHPUT</span>{latest?.network.map((item) => <div className="rate-row" key={item.name}><strong>{item.name}</strong><span>↓ {formatRate(item.rx_bytes_per_second)} · ↑ {formatRate(item.tx_bytes_per_second)}</span></div>)}</article><PerformanceAnomalies /><p className="monitor-note">Live history keeps the latest 120 samples in memory and is discarded when Linux Powerhouse exits.</p></section>}
      {section === "Dashboard" && system && <section className="grid"><article className="card wide"><span className="label">OPERATING SYSTEM</span><strong>{system.operating_system.name ?? "Linux"}</strong><span>{system.operating_system.version ?? "Version unavailable"} · {system.architecture}</span></article><article className="card"><span className="label">KERNEL</span><strong>{system.kernel_version}</strong><span>{system.hostname}</span></article><article className="card"><span className="label">MEMORY</span><strong>{formatBytes(system.memory_available_bytes)}</strong><span>available of {formatBytes(system.memory_total_bytes)}</span></article><article className="card"><span className="label">UPTIME</span><strong>{formatUptime(system.uptime_seconds)}</strong><span>{system.cpu_logical_cores} logical CPU cores</span></article><article className="card wide"><span className="label">STORAGE</span><strong>{storage.length} filesystems</strong><span>{storage.filter((item) => item.usage_percent >= 90).length} critically full · {storage.filter((item) => item.usage_percent >= 75).length} above 75%</span></article><article className="card"><span className="label">NETWORK</span><strong>{network.filter((item) => item.is_up).length} active</strong><span>{network.length} interfaces detected</span></article><article className="card wide"><span className="label">PROCESSES</span><strong>{processes.length} top processes</strong><span>Sorted by resident memory · read-only snapshot</span></article></section>}
      {section === "Storage" && <section className="list">{storage.map((item) => <article className="row" key={item.mount_point}><div><strong>{item.mount_point}</strong><span>{formatBytes(item.available_bytes)} available of {formatBytes(item.total_bytes)}</span></div><b>{item.usage_percent}%</b></article>)}</section>}
      {section === "Processes" && <section className="list">{processes.map((item) => <article className="row" key={item.pid}><div><strong>{item.name}</strong><span>PID {item.pid} · {item.state}</span></div><b>{formatBytes(item.memory_bytes)}</b></article>)}</section>}
      {section === "Network" && <section className="list">{network.map((item) => <article className="row" key={item.name}><div><strong>{item.name}</strong><span>{item.is_up ? "Link up" : "Link down"}</span></div><b>↓ {formatBytes(item.rx_bytes)} · ↑ {formatBytes(item.tx_bytes)}</b></article>)}</section>}
    </section>
  </main>;
}
