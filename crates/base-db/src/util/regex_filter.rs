use regex::Regex;

pub fn filter_regex_patterns(
    text: &str,
    allowed_patterns: &[Regex],
    ignored_patterns: &[Regex],
) -> bool {
    if !allowed_patterns.is_empty()
        && !allowed_patterns
            .iter()
            .any(|pattern| pattern.is_match(text))
    {
        return false;
    }

    if ignored_patterns
        .iter()
        .any(|pattern| pattern.is_match(text))
    {
        return false;
    }

    true
}
