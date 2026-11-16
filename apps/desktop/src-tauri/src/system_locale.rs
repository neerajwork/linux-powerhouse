/// Collects available system locale candidates in precedence order.
///
/// The caller is responsible for normalization and validation.
pub(crate) fn collect_system_locale_candidates(
    lc_all: Option<&str>,
    lc_messages: Option<&str>,
    lang: Option<&str>,
) -> Vec<String> {
    [lc_all, lc_messages, lang]
        .into_iter()
        .flatten()
        .filter(|locale| !locale.trim().is_empty())
        .map(str::to_owned)
        .collect()
}

/// Reads system locale candidates without modifying process environment.
pub(crate) fn system_locale_candidates() -> Vec<String> {
    let lc_all = std::env::var("LC_ALL").ok();
    let lc_messages = std::env::var("LC_MESSAGES").ok();
    let lang = std::env::var("LANG").ok();

    collect_system_locale_candidates(lc_all.as_deref(), lc_messages.as_deref(), lang.as_deref())
}

#[cfg(test)]
mod tests {
    use super::collect_system_locale_candidates;

    #[test]
    fn locale_candidates_follow_linux_precedence_order() {
        let candidates = collect_system_locale_candidates(
            Some("kn_IN.UTF-8"),
            Some("en_GB.UTF-8"),
            Some("en_US.UTF-8"),
        );

        assert_eq!(
            candidates,
            vec!["kn_IN.UTF-8", "en_GB.UTF-8", "en_US.UTF-8"]
        );
    }

    #[test]
    fn locale_candidates_skip_missing_and_whitespace_values() {
        let candidates = collect_system_locale_candidates(Some("  "), None, Some("kn_IN.UTF-8"));

        assert_eq!(candidates, vec!["kn_IN.UTF-8"]);
    }

    #[test]
    fn locale_candidates_are_empty_when_all_sources_are_unavailable() {
        let candidates = collect_system_locale_candidates(None, None, None);

        assert!(candidates.is_empty());
    }
}
