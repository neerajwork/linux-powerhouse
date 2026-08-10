import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";

type OperatingSystem = {
  id: string | null;
  name: string | null;
  version: string | null;
};

type SystemStatus = {
  operating_system: OperatingSystem;
  kernel_version: string;
  architecture: string;
  hostname: string;
  cpu_model: string | null;
  cpu_logical_cores: number;
  memory_total_bytes: number;
  memory_available_bytes: number;
  swap_total_bytes: number;
  swap_free_bytes: number;
  uptime_seconds: number;
};

function formatBytes(bytes: number): string {
  const units = ["B", "KB", "MB", "GB", "TB"];
  let value = bytes;
  let index = 0;
  while (value >= 1024 && index < units.length - 1) {
    value /= 1024;
    index += 1;
  }
  return `${value.toFixed(index === 0 ? 0 : 1)} ${units[index]}`;
}

function formatUptime(seconds: number): string {
  const days = Math.floor(seconds / 86_400);
  const hours = Math.floor((seconds % 86_400) / 3_600);
  const minutes = Math.floor((seconds % 3_600) / 60);
  return `${days}d ${hours}h ${minutes}m`;
}

export default function App() {
  const [status, setStatus] = useState<SystemStatus | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function loadSystemStatus() {
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<SystemStatus>("system_status");
      setStatus(result);
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  }

  return (
    <main className="app-shell">
      <aside className="sidebar">
        <div className="brand">
          <div className="brand-mark">LP</div>
          <div>
            <strong>Linux Powerhouse</strong>
            <span>Powerful Linux. Simplified.</span>
          </div>
        </div>
        <nav>
          <button className="nav-item active">Dashboard</button>
          <button className="nav-item">System</button>
          <button className="nav-item">Storage</button>
          <button className="nav-item">Processes</button>
          <button className="nav-item">Services</button>
          <button className="nav-item">Network</button>
          <button className="nav-item">Security</button>
          <button className="nav-item">AI Assistant</button>
        </nav>
      </aside>

      <section className="content">
        <header className="header">
          <div>
            <p className="eyebrow">SYSTEM DASHBOARD</p>
            <h1>Welcome to Linux Powerhouse</h1>
            <p className="subtitle">A safe, human-friendly window into your Linux system.</p>
          </div>
          <button className="primary" onClick={loadSystemStatus} disabled={loading}>
            {loading ? "Reading system…" : "Refresh system"}
          </button>
        </header>

        {error && <div className="error">Unable to read system status: {error}</div>}

        {!status && !loading && !error && (
          <section className="welcome-card">
            <div>
              <span className="status-dot" />
              <h2>Your Linux system, made understandable.</h2>
              <p>Start with a read-only system snapshot. Nothing is changed and no shell command is executed.</p>
            </div>
            <button className="primary" onClick={loadSystemStatus}>Get system status</button>
          </section>
        )}

        {status && (
          <>
            <section className="grid">
              <article className="card wide">
                <span className="label">OPERATING SYSTEM</span>
                <strong>{status.operating_system.name ?? "Linux"}</strong>
                <span>{status.operating_system.version ?? "Version unavailable"}</span>
              </article>
              <article className="card">
                <span className="label">KERNEL</span>
                <strong>{status.kernel_version}</strong>
                <span>{status.architecture}</span>
              </article>
              <article className="card">
                <span className="label">HOSTNAME</span>
                <strong>{status.hostname}</strong>
                <span>{status.cpu_logical_cores} logical CPU cores</span>
              </article>
              <article className="card wide">
                <span className="label">MEMORY</span>
                <strong>{formatBytes(status.memory_available_bytes)} available</strong>
                <span>{formatBytes(status.memory_total_bytes)} total</span>
              </article>
              <article className="card">
                <span className="label">SWAP</span>
                <strong>{formatBytes(status.swap_free_bytes)} free</strong>
                <span>{formatBytes(status.swap_total_bytes)} total</span>
              </article>
              <article className="card">
                <span className="label">UPTIME</span>
                <strong>{formatUptime(status.uptime_seconds)}</strong>
                <span>System uptime</span>
              </article>
              <article className="card wide">
                <span className="label">PROCESSOR</span>
                <strong>{status.cpu_model ?? "CPU information unavailable"}</strong>
                <span>{status.cpu_logical_cores} logical cores</span>
              </article>
            </section>
          </>
        )}
      </section>
    </main>
  );
}
