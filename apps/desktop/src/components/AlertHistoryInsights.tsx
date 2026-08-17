type AlertEvent = {
  timestamp_ms: number;
  kind: "Cpu" | "Memory" | "Swap" | "Storage" | "Network";
  severity: "Warning" | "Critical";
  decision: "Notify" | "Suppressed";
};

const label = (kind: AlertEvent["kind"]) => (kind === "Cpu" ? "CPU" : kind);

export function AlertHistoryInsights({ events }: { events: AlertEvent[] }) {
  const now = Date.now();
  const recentCutoff = now - 7 * 24 * 60 * 60 * 1000;
  const previousCutoff = now - 14 * 24 * 60 * 60 * 1000;
  const recent = events.filter((event) => event.timestamp_ms >= recentCutoff);
  const previous = events.filter(
    (event) => event.timestamp_ms >= previousCutoff && event.timestamp_ms < recentCutoff,
  );

  const categoryCount = (items: AlertEvent[]) =>
    items.reduce<Record<string, number>>((counts, event) => {
      counts[event.kind] = (counts[event.kind] ?? 0) + 1;
      return counts;
    }, {});

  const recentCounts = categoryCount(recent);
  const previousCounts = categoryCount(previous);
  const categories = Object.keys({ ...recentCounts, ...previousCounts });

  const changes = categories
    .map((kind) => ({
      kind: kind as AlertEvent["kind"],
      recent: recentCounts[kind] ?? 0,
      previous: previousCounts[kind] ?? 0,
      change: (recentCounts[kind] ?? 0) - (previousCounts[kind] ?? 0),
    }))
    .sort((a, b) => Math.abs(b.change) - Math.abs(a.change));

  const mostChanged = changes[0];
  const recentCritical = recent.filter((event) => event.severity === "Critical").length;
  const previousCritical = previous.filter((event) => event.severity === "Critical").length;
  const recentSuppressed = recent.filter((event) => event.decision === "Suppressed").length;
  const previousSuppressed = previous.filter((event) => event.decision === "Suppressed").length;

  const criticalDirection = recentCritical > previousCritical ? "increasing" : recentCritical < previousCritical ? "decreasing" : "stable";
  const suppressionDirection = recentSuppressed > previousSuppressed ? "increasing" : recentSuppressed < previousSuppressed ? "decreasing" : "stable";

  const priority = mostChanged && mostChanged.change !== 0
    ? `${label(mostChanged.kind)} changed the most (${mostChanged.change > 0 ? "+" : ""}${mostChanged.change} events).`
    : recentCritical > 0
      ? `${recentCritical} critical event${recentCritical === 1 ? "" : "s"} occurred in the last 7 days.`
      : "No alert category shows a meaningful recent change.";

  return (
    <section className="alert-history__insights" aria-labelledby="alert-history-insights-title">
      <div>
        <p className="eyebrow">LOCAL INSIGHTS</p>
        <h3 id="alert-history-insights-title">What changed?</h3>
        <p className="subtitle">Deterministic observations from the retained alert history.</p>
      </div>
      <div className="alert-history__insights-grid">
        <article className="card"><strong>{priority}</strong><span>Primary attention signal</span></article>
        <article className="card"><strong>{criticalDirection}</strong><span>Critical events vs previous 7 days</span></article>
        <article className="card"><strong>{suppressionDirection}</strong><span>Suppressed events vs previous 7 days</span></article>
      </div>
      <small className="monitor-note">
        Recent: {recent.length} events · Previous: {previous.length} · Critical: {recentCritical} vs {previousCritical} · Suppressed: {recentSuppressed} vs {previousSuppressed}
      </small>
    </section>
  );
}
