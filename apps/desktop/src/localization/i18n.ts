import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";

import en from "./resources/en.json";
import kn from "./resources/kn.json";
import { selectApplicationLocaleFromCandidates } from "./locale";

const STORAGE_KEY = "linux-powerhouse.locale.v1";

let initializationPromise: Promise<void> | null = null;

async function initialize(): Promise<void> {
  let savedPreference: string | null = null;

  try {
    savedPreference = localStorage.getItem(STORAGE_KEY);
  } catch {
    // Continue with system locale detection.
  }

  let systemLocaleCandidates: string[] = [];

  try {
    systemLocaleCandidates = await invoke<string[]>(
      "system_locale_candidates",
    );
  } catch {
    // Fall back to the saved preference or English.
  }

  const locale = selectApplicationLocaleFromCandidates(
    savedPreference,
    systemLocaleCandidates,
  );

  await i18n.use(initReactI18next).init({
    resources: {
      en: { translation: en },
      kn: { translation: kn },
    },
    lng: locale,
    fallbackLng: "en",
    supportedLngs: ["en", "kn"],
    interpolation: {
      escapeValue: false,
    },
  });

  document.documentElement.lang = locale;
}

export function initializeI18n(): Promise<void> {
  if (initializationPromise === null) {
    initializationPromise = initialize();
  }

  return initializationPromise;
}

export default i18n;
