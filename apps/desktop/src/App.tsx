import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

type OperatingSystem = { id: string | null; name: string | null; version: string | null };
type SystemStatus = { operating_system: OperatingSystem; kernel_version: string; architecture: string; hostname: string; cpu_model: string | null; cpu_logical_cores: number; memory_total_bytes: number; memory_available_bytes: number; swap_total_bytes: number; swap_free_bytes: number; uptime_seconds: number };
type FilesystemStatus = { mount_point: string; total_bytes: number; available_bytes: number; used_bytes: number; usage_percent: number };
type ProcessInfo = { pid: number; name: string; state: string; memory_bytes: number };
type NetworkInterface = { name: string; is_up: boolean; rx_bytes: number; tx_bytes: number };
type Section = "Dashboard" | "Storage" | "Processes" | "Network";

const formatBytes = (bytes: number) => {
  const units = ["B", "KB", "MB", "GB", "TB"];
  let value = bytes;
  let index = 0;
  while (value >= 1024 && index < units.length - 1) { value /= 1024; index += 1; }
  return `${value.toFixed(index === 0 ? 0 : 1)} ${units[index]}`;
};
const formatUptime = (seconds: number) => `${Math.floor(seconds / 86400)}d ${Math.floor((seconds % 86400) / 3600)}h ${Math.floor((seconds % 3600) / 60)}m`;

export default function App() {
  const [section, setSection] = useState<Section>("Dashboard");
  const [system, setSystem] = useState<SystemStatus | null>(null);
  const [storage, setStorage] = useState<FilesystemStatus[]>([]);
  const [processes, setProcesses] = useState<ProcessInfo[]>([]);
  const [network, setNetwork] = useState<NetworkInterface[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function refresh() {
    setLoading(true);
    setError(null);
    try {
      const [nextSystem, nextStorage, nextProcesses, nextNetwork] = await Promise.all([
        invoke<SystemStatus>("system_status"),
        invoke<FilesystemStatus[]>("storage_status"),
        invoke<ProcessInfo[]>("process_status"),
        invoke<NetworkInterface[]>("network_status"),
      ]);
      setSystem(nextSystem); setStorage(nextStorage); setProcesses(nextProcesses); setNetwork(nextNetwork);
    } catch (err) { setError(String(err)); } finally { setLoading(false); }
  }

  useEffect(() => { void refresh(); }, []);

  return (
    <main className="app-shell">
      <aside className="sidebar">
        <div className="brand"><div className="brand-mark">LP</div><div><strong>Linux Powerhouse</strong><span>Powerful Linux. Simplified.</span></div></div>
        <nav>
          {(["Dashboard", "Storage", "Processes", "Network"] as Section[]).map((item) => (
            <button key={item} className={`nav-item ${section === item ? "active" : ""}`} onClick={() => setSection(item)}>{item}</button>
          ))}
          <button className="nav-item" disabled>Services</button>
          <button className="nav-item" disabled>Security</button>
          <button className="nav-item" disabled>AI Assistant</button>
        </nav>
      </aside>

      <section className="content">
        <header className="header">
          <div><p className="eyebrow">{section.toUpperCase()}</p><h1>{section === "Dashboard" ? "Your Linux system, at a glance." : section}</h1><p className="subtitle">Read-only insights, collected through Linux-native capabilities.</p></div>
          <button className="primary" onClick={() => void refresh()} disabled={loading}>{loading ? "Refreshing…" : "Refresh"}</button>
        </header>
        {error && <div className="error">Unable to refresh system data: {error}</div>}

        {section === "Dashboard" && system && (
          <section className="grid">
            <article className="card wide"><span className="label">OPERATING SYSTEM</span><strong>{system.operating_system.name ?? "Linux"}</strong><span>{system.operating_system.version ?? "Version unavailable"} · {system.architecture}</span></article>
            <article className="card"><span className="label">KERNEL</span><strong>{system.kernel_version}</strong><span>{system.hostname}</span></article>
            <article className="card"><span className="label">MEMORY</span><strong>{formatBytes(system.memory_available_bytes)}</strong><span>available of {formatBytes(system.memory_total_bytes)}</span></article>
            <article className="card"><span className="label">UPTIME</span><strong>{formatUptime(system.uptime_seconds)}</strong><span>{system.cpu_logical_cores} logical CPU cores</span></article>
            <article className="card wide"><span className="label">STORAGE</span><strong>{storage.length} filesystems</strong><span>{storage.filter((item) => item.usage_percent >= 90).length} critically full · {storage.filter((item) => item.usage_percent >= 75).length} above 75%</span></article>
            <article className="card"><span className="label">NETWORK</span><strong>{network.filter((item) => item.is_up).length} active</strong><span>{network.length} interfaces detected</span></article>
            <article className="card wide"><span className="label">PROCESSES</span><strong>{processes.length} top processes</strong><span>Sorted by resident memory · read-only snapshot</span></article>
          </section>
        )}

        {section === "Storage" && <section className="list">{storage.map((item) => <article className="row" key={item.mount_point}><div><strong>{item.mount_point}</strong><span>{formatBytes(item.available_bytes)} available of {formatBytes(item.total_bytes)}</span></div><b>{item.usage_percent}%</b></article>)}</section>}
        {section === "Processes" && <section className="list">{processes.map((item) => <article className="row" key={item.pid}><div><strong>{item.name}</strong><span>PID {item.pid} · {item.state}</span></div><b>{formatBytes(item.memory_bytes)}</b></article>)}</section>}
        {section === "Network" && <section className="list">{network.map((item) => <article className="row" key={item.name}><div><strong>{item.name}</strong><span>{item.is_up ? "Link up" : "Link down"}</span></div><b>↓ {formatBytes(item.rx_bytes)} · ↑ {formatBytes(item.tx_bytes)}</b></article>)}</section>}
      </section>
    </main>
  );
}
