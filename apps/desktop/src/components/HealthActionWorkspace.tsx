import { useCallback, useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { actionForSubsystem, type SafeActionDefinition } from "../safeActions";

type HealthLevel = "Healthy" | "Attention" | "Degraded";
type Subsystem = "Storage" | "Processes" | "Network" | "Services";
type Snapshot = {
  health: HealthLevel;
  storage_anomalies: number;
  process_anomalies: number;
  network_anomalies: number;
  service_anomalies: number;
  total_anomalies: number;
};
type ActionResult = {
  action: string;
  status: string;
  message: string;
  reversible: boolean;
  privilege: string;
  verification_status: string;
  verification_message: string;
};
type RemediationSuggestion = {
  action: string;
  reason: string;
  suggested_action: string;
  requires_confirmation: boolean;
};

const subsystems: Array<[Subsystem, keyof Omit<Snapshot, "health" | "total_anomalies">]> = [
  ["Storage", "storage_anomalies"],
  ["Processes", "process_anomalies"],
  ["Network", "network_anomalies"],
  ["Services", "service_anomalies"],
];

const descriptions: Record<Subsystem, string> = {
  Storage: "Review capacity and storage-health signals without changing files or mount configuration.",
  Processes: "Inspect current process-health signals without terminating or modifying workloads.",
  Network: "Inspect network-health signals without changing interfaces, routes, or configuration.",
  Services: "Inspect service-health signals without restarting or changing background services.",
};

const label = (value: string) =>
  value.replaceAll("_", " ").replace(/\b\w/g, (letter) => letter.toUpperCase());

export function HealthActionWorkspace() {
  const [snapshot, setSnapshot] = useState<Snapshot | null>(null);
  const [selected, setSelected] = useState<Subsystem>("Storage");
  const [confirmed, setConfirmed] = useState(false);
  const [loading, setLoading] = useState(true);
  const [running, setRunning] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [result, setResult] = useState<ActionResult | null>(null);
  const [remediation, setRemediation] = useState<RemediationSuggestion[]>([]);

  const refresh = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const next = await invoke<Snapshot>("system_intelligence", { storageRoot: "/" });
      setSnapshot(next);
      const active = subsystems.filter(([, key]) => next[key] > 0);
      if (active.length && !active.some(([name]) => name === selected)) setSelected(active[0][0]);
      if (!active.length) setSelected("Storage");
      setConfirmed(false);
      setResult(null);
      setRemediation([]);
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  }, [selected]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const activeSubsystems = useMemo(
    () => snapshot ? subsystems.filter(([, key]) => snapshot[key] > 0) : [],
    [snapshot],
  );

  const dominant = useMemo(() => {
    if (!snapshot) return null;
    return [...subsystems].sort((a, b) => snapshot[b[1]] - snapshot[a[1]])[0][0];
  }, [snapshot]);

  const action: SafeActionDefinition = actionForSubsystem(selected);
  const noActionNeeded = snapshot?.total_anomalies === 0;

  const runAction = async () => {
    if (!confirmed || noActionNeeded) return;
    setRunning(true);
    setError(null);
    setResult(null);
    setRemediation([]);
    try {
      const next = await invoke<ActionResult>("safe_system_action", {
  action: action.id,
  confirmed,
});
      setResult(next);
      setConfirmed(false);
      const suggestions = await invoke<RemediationSuggestion[]>("action_remediation_suggestions", {
        action: next.action,
        status: next.status,
        verificationStatus: next.verification_status,
      });
      setRemediation(suggestions);
      const refreshed = await invoke<Snapshot>("system_intelligence", { storageRoot: "/" });
      setSnapshot(refreshed);
    } catch (err) {
      setError(String(err));
    } finally {
      setRunning(false);
    }
  };

  return (
    <section className="health-workspace">
      <div className="health-workspace__intro">
        <div>
          <span className="label">HEALTH ACTION WORKSPACE</span>
          <h2>Understand first. Act safely.</h2>
          <p>
            A focused path from a current health signal to a narrowly scoped, read-only diagnostic.
            Nothing runs automatically.
          </p>
        </div>
        <button className="secondary" onClick={() => void refresh()} disabled={loading || running}>
          {loading ? "Refreshing…" : "Refresh health"}
        </button>
      </div>

      {error && <div className="error">Unable to complete the health action workflow: {error}</div>}

      {snapshot && (
        <>
          <div className="health-workspace__status">
            <div>
              <span className="label">CURRENT STATE</span>
              <strong className={`system-health__status system-health__status--${snapshot.health.toLowerCase()}`}>
                {snapshot.health}
              </strong>
            </div>
            <div>
              <strong>{snapshot.total_anomalies}</strong>
              <span>current signal(s)</span>
            </div>
            <div>
              <strong>{dominant ?? "None"}</strong>
              <span>dominant subsystem</span>
            </div>
          </div>

          {noActionNeeded ? (
            <div className="health-workspace__safe card">
              <span className="label">NO ACTION REQUIRED</span>
              <strong>The current health snapshot is clean.</strong>
              <span>Continue normal monitoring. A diagnostic is not recommended when there are no active signals.</span>
            </div>
          ) : (
            <>
              <div className="health-workspace__signals">
                {activeSubsystems.map(([name, key]) => (
                  <button
                    key={name}
                    className={`health-workspace__signal ${selected === name ? "active" : ""}`}
                    onClick={() => { setSelected(name); setConfirmed(false); setResult(null); setRemediation([]); }}
                  >
                    <span>{name}</span>
                    <strong>{snapshot[key]}</strong>
                    <small>active signal(s)</small>
                  </button>
                ))}
              </div>

              <div className="health-workspace__flow">
                <article className="health-workspace__panel">
                  <span className="label">1 · UNDERSTAND</span>
                  <h3>{selected} needs review</h3>
                  <p>{descriptions[selected]}</p>
                  <span className="muted">This workspace only proposes read-only diagnostics.</span>
                </article>

                <article className="health-workspace__panel">
                  <span className="label">2 · PREVIEW</span>
                  <h3>{action.label}</h3>
                  <div className="health-workspace__facts">
                    <span>Changes system state <strong>{action.changesSystemState ? "Yes" : "No"}</strong></span>
                    <span>Reversible <strong>{action.reversible ? "Yes" : "No"}</strong></span>
                    <span>Privilege <strong>{action.privilege}</strong></span>
                  </div>
                  <p>This action inspects or refreshes health data only. It does not terminate processes, restart services, change networking, or modify storage.</p>
                </article>
              </div>

              <article className="health-workspace__confirm">
                <div>
                  <span className="label">3 · CONFIRM</span>
                  <h3>Ready to run the diagnostic?</h3>
                  <p>Linux Powerhouse will execute only the selected allowlisted action. The result will be verified and recorded in Action Audit.</p>
                </div>
                <label className="health-workspace__checkbox">
                  <input type="checkbox" checked={confirmed} onChange={(event) => setConfirmed(event.target.checked)} disabled={running} />
                  <span>I understand this is a read-only diagnostic.</span>
                </label>
                <button className="primary" onClick={() => void runAction()} disabled={!confirmed || running}>
                  {running ? "Running…" : `Confirm & run ${label(action.id)}`}
                </button>
              </article>
            </>
          )}

          {result && (
            <>
              <article className="health-workspace__result">
                <span className="label">4 · VERIFY</span>
                <h3>Action {result.status}</h3>
                <p>{result.message}</p>
                <div className="health-workspace__facts">
                  <span>Verification <strong>{result.verification_status || "pending"}</strong></span>
                  <span>Details <strong>{result.verification_message || "No additional verification details."}</strong></span>
                </div>
                <span className="muted">The execution was recorded locally in Action Audit.</span>
              </article>

              {remediation.map((suggestion) => (
                <article className="health-workspace__result health-workspace__remediation" key={`${suggestion.action}-${suggestion.suggested_action}`}>
                  <span className="label">5 · NEXT STEP</span>
                  <h3>Recommended follow-up: {label(suggestion.suggested_action)}</h3>
                  <p>{suggestion.reason}</p>
                  <span className="muted">
                    {suggestion.requires_confirmation ? "Explicit confirmation is required before this follow-up can run." : "No confirmation is required."}
                  </span>
                </article>
              ))}
            </>
          )}
        </>
      )}
    </section>
  );
}
