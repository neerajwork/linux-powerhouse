import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";

import {
  defaultLocalizationNamespace,
  localizationNamespaces,
  localizationResources,
} from "./resources";
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
    systemLocaleCandidates = await invoke<string[]>("system_locale_candidates");
  } catch {
    // Fall back to the saved preference or English.
  }

  const locale = selectApplicationLocaleFromCandidates(
    savedPreference,
    systemLocaleCandidates,
  );

  await i18n.use(initReactI18next).init({
    resources: localizationResources,
    lng: locale,
    fallbackLng: "en",
    supportedLngs: ["en", "kn"],
    ns: [...localizationNamespaces],
    defaultNS: defaultLocalizationNamespace,
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

export async function changeApplicationLocale(
  locale: string,
): Promise<void> {
  if (locale !== "en" && locale !== "kn") {
    return;
  }

  await i18n.changeLanguage(locale);

  document.documentElement.lang = locale;

  try {
    localStorage.setItem(STORAGE_KEY, locale);
  } catch {
    // Language switching remains available without persistence.
  }
}

export default i18n;
