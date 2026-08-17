export function AlertHistorySummary({ events }: { events: Array<{ severity: "Warning" | "Critical"; kind: "Cpu" | "Memory" | "Swap" | "Storage" | "Network"; decision: "Notify" | "Suppressed"; timestamp_ms: number }> }) {
  const warnings = events.filter((event) => event.severity === "Warning").length;
  const critical = events.filter((event) => event.severity === "Critical").length;
  const notified = events.filter((event) => event.decision === "Notify").length;
  const suppressed = events.filter((event) => event.decision === "Suppressed").length;
  const categoryCounts = events.reduce<Record<string, number>>((counts, event) => {
    counts[event.kind] = (counts[event.kind] ?? 0) + 1;
    return counts;
  }, {});
  const topCategory = Object.entries(categoryCounts).sort((a, b) => b[1] - a[1])[0];
  const now = Date.now();
  const recentCutoff = now - 7 * 24 * 60 * 60 * 1000;
  const previousCutoff = now - 14 * 24 * 60 * 60 * 1000;
  const recent = events.filter((event) => event.timestamp_ms >= recentCutoff).length;
  const previous = events.filter((event) => event.timestamp_ms >= previousCutoff && event.timestamp_ms < recentCutoff).length;
  const trend = recent > previous ? "Increasing" : recent < previous ? "Decreasing" : "Stable";

  return (
    <div className="alert-history__summary" aria-label="Alert history summary">
      <article className="card"><strong>{events.length}</strong><span>Total retained events</span></article>
      <article className="card"><strong>{critical}</strong><span>Critical events</span></article>
      <article className="card"><strong>{notified}</strong><span>Notified</span></article>
      <article className="card"><strong>{suppressed}</strong><span>Suppressed</span></article>
      <article className="card"><strong>{topCategory ? topCategory[0] : "—"}</strong><span>Most frequent category</span></article>
      <article className="card"><strong>{trend}</strong><span>7-day event trend</span></article>
      <small className="monitor-note">Warning: {warnings} · Critical: {critical} · 7 days: {recent} · Previous 7 days: {previous}</small>
    </div>
  );
}
