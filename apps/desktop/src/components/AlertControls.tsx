import { useEffect, useState } from "react";
import {
  ALERT_CATEGORIES,
  type AlertCategory,
  type AlertPreference,
  readAlertPreferences,
  restoreExpiredSnoozes,
  writeAlertPreferences,
} from "../alerts";

const SNOOZE_DAYS = [7, 14, 30] as const;

function categoryLabel(category: AlertCategory): string {
  switch (category) {
    case "Cpu":
      return "CPU";
    case "Memory":
      return "Memory";
    case "Swap":
      return "Swap";
    case "Storage":
      return "Storage";
    case "Network":
      return "Network";
  }
}

function formatSnooze(until: number): string {
  return `Snoozed until ${new Date(until).toLocaleDateString()}`;
}

export function AlertControls() {
  const [preferences, setPreferences] = useState<AlertPreference[]>(() =>
    restoreExpiredSnoozes(readAlertPreferences()),
  );

  useEffect(() => {
    writeAlertPreferences(preferences);
  }, [preferences]);

  function updatePreference(
    category: AlertCategory,
    update: Partial<AlertPreference>,
  ) {
    setPreferences((current) =>
      current.map((preference) =>
        preference.category === category
          ? { ...preference, ...update }
          : preference,
      ),
    );
  }
  function snooze(category: AlertCategory, days: number) {
    const until = Date.now() + days * 24 * 60 * 60 * 1000;

    updatePreference(category, {
      state: "Snoozed",
      snoozedUntil: until,
    });
  }

  function dismiss(category: AlertCategory) {
    updatePreference(category, {
      state: "Dismissed",
      snoozedUntil: null,
    });
  }

  function restore(category: AlertCategory) {
    updatePreference(category, {
      state: "Active",
      snoozedUntil: null,
    });
  }

  function preferenceLabel(preference: AlertPreference): string {
    if (
      preference.state === "Snoozed" &&
      preference.snoozedUntil !== null
    ) {
      return formatSnooze(preference.snoozedUntil);
    }

    if (preference.state === "Dismissed") {
      return "Dismissed";
    }

    return "Active";
  }
  return (
    <section className="card alert-controls" aria-labelledby="alert-controls-title">
      <div>
        <p className="label">ALERT CONTROLS</p>
        <h2 id="alert-controls-title">Routine warning controls</h2>
        <p>
          Control routine warnings without hiding critical events. Critical
events are always reported.
        </p>
      </div>

      <div className="alert-controls__list">
        {ALERT_CATEGORIES.map((category) => {
          const preference = preferences.find(
            (item) => item.category === category,
          );

          if (!preference) return null;

          return (
            <article className="alert-controls__item" key={category}>
              <div>
                <strong>{categoryLabel(category)}</strong>
                <span>{preferenceLabel(preference)}</span>
              </div>

              <div className="alert-controls__actions">
                {preference.state === "Active" && (
                  <>
                    {SNOOZE_DAYS.map((days) => (
                      <button
                        className="secondary"
                        key={days}
                        onClick={() => snooze(category, days)}
                      >
                        Snooze {days} days
                      </button>
                    ))}
                    <button
                      className="secondary"
                      onClick={() => dismiss(category)}
                    >
                      Disable routine warnings
                    </button>
                  </>
                )}

                {preference.state !== "Active" && (
                  <button
                    className="secondary"
                    onClick={() => restore(category)}
                  >
                    Restore
                  </button>
                )}
              </div>
            </article>
          );
        })}
      </div>

      <small>
        Critical events are always reported and are never affected by these
controls. Routine warning preferences are stored locally on this
computer.
      </small>
    </section>
  );
}
