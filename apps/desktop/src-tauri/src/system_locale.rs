/// Collects available system locale candidates in precedence order.
///
/// The caller is responsible for normalization and validation.
/// LANGUAGE may contain a colon-separated list of preferences.
pub(crate) fn collect_system_locale_candidates(
    lc_all: Option<&str>,
    lang: Option<&str>,
    language: Option<&str>,
) -> Vec<String> {
    let mut candidates = Vec::new();

    if let Some(locale) = lc_all.filter(|value| !value.trim().is_empty()) {
        candidates.push(locale.to_owned());
    }

    if let Some(locale) = lang.filter(|value| !value.trim().is_empty()) {
        candidates.push(locale.to_owned());
    }

    if let Some(preferences) = language {
        candidates.extend(
            preferences
                .split(':')
                .map(str::trim)
                .filter(|locale| !locale.is_empty())
                .map(str::to_owned),
        );
    }

    candidates
}

/// Reads system locale candidates without modifying process environment.
pub(crate) fn system_locale_candidates() -> Vec<String> {
    let lc_all = std::env::var("LC_ALL").ok();
    let lang = std::env::var("LANG").ok();
    let language = std::env::var("LANGUAGE").ok();

    collect_system_locale_candidates(lc_all.as_deref(), lang.as_deref(), language.as_deref())
}

#[cfg(test)]
mod tests {
    use super::collect_system_locale_candidates;

    #[test]
    fn locale_candidates_follow_linux_precedence_order() {
        let candidates = collect_system_locale_candidates(
            Some("kn_IN.UTF-8"),
            Some("en_US.UTF-8"),
            Some("fr_FR:de_DE"),
        );

        assert_eq!(
            candidates,
            vec!["kn_IN.UTF-8", "en_US.UTF-8", "fr_FR", "de_DE",]
        );
    }

    #[test]
    fn locale_candidates_skip_missing_and_whitespace_values() {
        let candidates = collect_system_locale_candidates(Some("  "), None, Some("kn_IN.UTF-8"));

        assert_eq!(candidates, vec!["kn_IN.UTF-8"]);
    }

    #[test]
    fn language_preferences_skip_empty_entries() {
        let candidates =
            collect_system_locale_candidates(None, None, Some("kn_IN.UTF-8::en_US.UTF-8:"));

        assert_eq!(candidates, vec!["kn_IN.UTF-8", "en_US.UTF-8"]);
    }

    #[test]
    fn locale_candidates_are_empty_when_all_sources_are_unavailable() {
        let candidates = collect_system_locale_candidates(None, None, None);

        assert!(candidates.is_empty());
    }
}
