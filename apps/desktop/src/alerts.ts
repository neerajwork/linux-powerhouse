export type AlertCategory = "Cpu" | "Memory" | "Swap" | "Storage" | "Network";

export type AlertPreferenceState = "Active" | "Dismissed" | "Snoozed";

export type AlertPreference = {
  category: AlertCategory;
  state: AlertPreferenceState;
  snoozedUntil: number | null;
};

export const ALERT_PREFERENCES_KEY = "linux-powerhouse.alert-preferences.v1";

export const ALERT_CATEGORIES: AlertCategory[] = [
  "Cpu",
  "Memory",
  "Swap",
  "Storage",
  "Network",
];

const defaultPreferences = (): AlertPreference[] =>
  ALERT_CATEGORIES.map((category) => ({
    category,
    state: "Active",
    snoozedUntil: null,
  }));

export function readAlertPreferences(): AlertPreference[] {
  try {
    const raw = localStorage.getItem(ALERT_PREFERENCES_KEY);
    if (!raw) return defaultPreferences();

    const parsed = JSON.parse(raw);
    if (!Array.isArray(parsed)) return defaultPreferences();

    const stored = new Map<string, AlertPreference>();

    for (const item of parsed) {
      if (
        item &&
        ALERT_CATEGORIES.includes(item.category) &&
        ["Active", "Dismissed", "Snoozed"].includes(item.state) &&
        (item.snoozedUntil === null || typeof item.snoozedUntil === "number")
      ) {
        stored.set(item.category, {
          category: item.category,
          state: item.state,
          snoozedUntil: item.snoozedUntil,
        });
      }
    }

    return ALERT_CATEGORIES.map(
      (category) =>
        stored.get(category) ?? {
          category,
          state: "Active",
          snoozedUntil: null,
        },
    );
  } catch {
    return defaultPreferences();
  }
}

export function writeAlertPreferences(preferences: AlertPreference[]): void {
  try {
    localStorage.setItem(ALERT_PREFERENCES_KEY, JSON.stringify(preferences));
  } catch {
    // Local persistence is best-effort; alert evaluation must continue.
  }
}

export function restoreExpiredSnoozes(
  preferences: AlertPreference[],
  now = Date.now(),
): AlertPreference[] {
  return preferences.map((preference) => {
    if (
      preference.state === "Snoozed" &&
      preference.snoozedUntil !== null &&
      preference.snoozedUntil <= now
    ) {
      return {
        ...preference,
        state: "Active",
        snoozedUntil: null,
      };
    }

    return preference;
  });
}
