/**
 * Normalizes a POSIX locale identifier into a canonical BCP 47 locale.
 *
 * Handles language, territory, encoding, and the Latin script modifier.
 */
export function normalizePosixLocale(locale: string): string | null {
  const trimmed = locale.trim();

  if (trimmed.length === 0) {
    return null;
  }

  let localeWithoutModifier = trimmed;
  let scriptModifier: string | null = null;

  const modifierSeparator = trimmed.indexOf("@");

  if (modifierSeparator !== -1) {
    const modifier = trimmed.slice(modifierSeparator + 1);

    if (
      modifier.length === 0 ||
      trimmed.indexOf("@", modifierSeparator + 1) !== -1
    ) {
      return null;
    }

    if (modifier.toLowerCase() !== "latin") {
      return null;
    }

    scriptModifier = "Latn";
    localeWithoutModifier = trimmed.slice(0, modifierSeparator);
  }

  const encodingSeparator = localeWithoutModifier.indexOf(".");
  let localeWithoutEncoding = localeWithoutModifier;

  if (encodingSeparator !== -1) {
    const encoding = localeWithoutModifier.slice(encodingSeparator + 1);

    if (
      encoding.length === 0 ||
      !/^[A-Za-z0-9_-]+$/.test(encoding) ||
      localeWithoutModifier.indexOf(".", encodingSeparator + 1) !== -1
    ) {
      return null;
    }

    localeWithoutEncoding = localeWithoutModifier.slice(
      0,
      encodingSeparator,
    );
  }

  const specialLocale = localeWithoutEncoding.toLowerCase();

  if (specialLocale === "c" || specialLocale === "posix") {
    return null;
  }

  const bcp47Locale = localeWithoutEncoding.replaceAll("_", "-");

  try {
    const canonicalLocale =
      Intl.getCanonicalLocales(bcp47Locale)[0];

    if (!canonicalLocale) {
      return null;
    }

    if (!scriptModifier) {
      return canonicalLocale;
    }

    const subtags = canonicalLocale.split("-");
    const existingScript = subtags[1];

    if (existingScript && /^[A-Za-z]{4}$/.test(existingScript)) {
      if (existingScript !== scriptModifier) {
        return null;
      }

      return canonicalLocale;
    }

    subtags.splice(1, 0, scriptModifier);

    return Intl.getCanonicalLocales(subtags.join("-"))[0] ?? null;
  } catch {
    return null;
  }
}

/**
 * Resolves a normalized BCP 47 locale to a supported application locale.
 *
 * Regional variants resolve to their supported base language.
 * Unsupported, malformed, or missing locales fall back to English.
 */
export function resolveSupportedLocale(
  locale: string | null,
): string {
  if (locale === null) {
    return "en";
  }

  try {
    const canonicalLocale = Intl.getCanonicalLocales(locale)[0];

    if (!canonicalLocale) {
      return "en";
    }

    const language = canonicalLocale.split("-")[0].toLowerCase();

    if (language === "kn") {
      return "kn";
    }

    return "en";
  } catch {
    return "en";
  }
}

/**
 * Selects the application locale using saved preference precedence.
 *
 * Exact supported saved preferences take priority over the system locale.
 * Unsupported, regional, or malformed preferences fall back to the
 * system locale.
 */
export function selectApplicationLocale(
  savedPreference: string | null,
  systemLocale: string | null,
): string {
  if (savedPreference !== null) {
    const preferredLocale = resolveSupportedLocale(savedPreference);

    if (preferredLocale === savedPreference) {
      return preferredLocale;
    }
  }

  return resolveSupportedLocale(systemLocale);
}
