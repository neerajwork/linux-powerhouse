import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

type AuditEntry = {
  id: string;
  timestamp: number;
  action: string;
  stage: string;
  confirmed: boolean;
  status: string;
  message: string;
  reversible: boolean;
  privilege: string;
  verification_status?: string;
  verification_message?: string;
};

const label = (action: string) => action.replaceAll("_", " ").replace(/\b\w/g, (letter) => letter.toUpperCase());

export function ActionAudit() {
  const [entries, setEntries] = useState<AuditEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    setLoading(true);
    setError(null);
    try { setEntries(await invoke<AuditEntry[]>("action_audit_history")); }
    catch (err) { setError(String(err)); }
    finally { setLoading(false); }
  }, []);

  useEffect(() => { void refresh(); }, [refresh]);

  return <section className="list">
    <div className="card"><span className="label">ACTION AUDIT</span><strong>{entries.length} recorded actions</strong><span>Local, append-oriented execution history with outcome verification. No remote telemetry.</span></div>
    {loading && <p className="muted">Loading audit history…</p>}
    {error && <div className="error">Unable to read action history: {error}</div>}
    {!loading && !entries.length && <div className="card"><strong>No actions recorded yet.</strong><span>Completed or failed safe actions will appear here.</span></div>}
    {[...entries].reverse().map((entry) => <article className="row" key={entry.id}>
      <div>
        <strong>{label(entry.action)}</strong>
        <span>{new Date(entry.timestamp).toLocaleString()} · {entry.stage} · {entry.confirmed ? "confirmed" : "not confirmed"}</span>
        <span>{entry.message}</span>
        {entry.verification_status && entry.verification_status !== "legacy" && <span>Outcome: {entry.verification_status}{entry.verification_message ? ` · ${entry.verification_message}` : ""}</span>}
      </div>
      <b>{entry.status}</b>
    </article>)}
  </section>;
}
