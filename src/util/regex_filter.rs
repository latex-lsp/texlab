use crate::RegexPattern;

pub fn filter(
    text: &str,
    allowed_patterns: &[RegexPattern],
    ignored_patterns: &[RegexPattern],
) -> bool {
    if !allowed_patterns.is_empty()
        && !allowed_patterns
            .iter()
            .any(|pattern| pattern.0.is_match(text))
    {
        return false;
    }

    if ignored_patterns
        .iter()
        .any(|pattern| pattern.0.is_match(text))
    {
        return false;
    }

    true
}
