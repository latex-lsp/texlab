pub trait Matcher: Send + Sync {
    fn score(&self, choice: &str, pattern: &str) -> Option<i32>;
}

impl<T: fuzzy_matcher::FuzzyMatcher> Matcher for T {
    fn score(&self, choice: &str, pattern: &str) -> Option<i32> {
        fuzzy_matcher::FuzzyMatcher::fuzzy_match(self, choice, pattern)
    }
}

#[derive(Debug)]
pub struct Prefix;

impl Matcher for Prefix {
    fn score(&self, choice: &str, pattern: &str) -> Option<i32> {
        if choice.starts_with(pattern) {
            Some(-(choice.len() as i32))
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct PrefixIgnoreCase;

impl Matcher for PrefixIgnoreCase {
    fn score(&self, choice: &str, pattern: &str) -> Option<i32> {
        if pattern.len() > choice.len() {
            return None;
        }

        let mut cs = choice.chars();
        for p in pattern.chars() {
            if !cs.next().unwrap().eq_ignore_ascii_case(&p) {
                return None;
            }
        }

        Some(-(choice.len() as i32))
    }
}
