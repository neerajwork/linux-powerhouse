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
  outcome_status?: string;
  outcome_message?: string;
};

type RemediationSuggestion = {
  action: string;
  reason: string;
  suggested_action: string;
  requires_confirmation: boolean;
};

const label = (action: string) =>
  action.replaceAll("_", " ").replace(/\b\w/g, (letter) => letter.toUpperCase());

export function ActionAudit() {
  const [entries, setEntries] = useState<AuditEntry[]>([]);
  const [suggestions, setSuggestions] = useState<Record<string, RemediationSuggestion[]>>({});
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      setEntries(await invoke<AuditEntry[]>("action_audit_history"));
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  useEffect(() => {
    if (!entries.length) {
      setSuggestions({});
      return;
    }

    let cancelled = false;
    const loadSuggestions = async () => {
      const resolved = await Promise.all(
        entries.map(async (entry) => [
          entry.id,
          await invoke<RemediationSuggestion[]>("action_remediation_suggestions", {
            action: entry.action,
            status: entry.status,
            verificationStatus: entry.verification_status ?? "legacy",
          }),
        ] as const),
      );
      if (!cancelled) setSuggestions(Object.fromEntries(resolved));
    };

    void loadSuggestions();
    return () => {
      cancelled = true;
    };
  }, [entries]);

  return (
    <section className="list">
      <div className="card">
        <span className="label">ACTION AUDIT</span>
        <strong>{entries.length} recorded actions</strong>
        <span>
          Local, append-oriented execution history with independently recorded execution,
          verification, and derived outcome evidence. No automatic remediation or remote telemetry.
        </span>
      </div>
      {loading && <p className="muted">Loading audit history…</p>}
      {error && <div className="error">Unable to read action history: {error}</div>}
      {!loading && !entries.length && (
        <div className="card">
          <strong>No actions recorded yet.</strong>
          <span>Completed or failed safe actions will appear here.</span>
        </div>
      )}
      {[...entries].reverse().map((entry) => (
        <article className="row" key={entry.id}>
          <div>
            <strong>{label(entry.action)}</strong>
            <span>
              {new Date(entry.timestamp).toLocaleString()} · {entry.stage} · {entry.confirmed ? "confirmed" : "not confirmed"}
            </span>
            <span>Execution: {entry.status} · {entry.message}</span>
            {entry.verification_status && entry.verification_status !== "legacy" && (
              <span>
                Verification: {entry.verification_status}
                {entry.verification_message ? ` · ${entry.verification_message}` : ""}
              </span>
            )}
            {entry.outcome_status && entry.outcome_status !== "legacy" && (
              <span>
                Outcome: {entry.outcome_status}
                {entry.outcome_message ? ` · ${entry.outcome_message}` : ""}
              </span>
            )}
            {!!suggestions[entry.id]?.length && (
              <div className="card">
                <span className="label">SUGGESTED NEXT STEP</span>
                {suggestions[entry.id].map((suggestion) => (
                  <span key={`${entry.id}-${suggestion.suggested_action}`}>
                    {suggestion.reason} Next: <strong>{label(suggestion.suggested_action)}</strong>. Explicit confirmation is required before execution.
                  </span>
                ))}
              </div>
            )}
          </div>
          <b>{entry.outcome_status && entry.outcome_status !== "legacy" ? entry.outcome_status : entry.status}</b>
        </article>
      ))}
    </section>
  );
}
