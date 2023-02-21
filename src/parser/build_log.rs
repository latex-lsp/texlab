use std::{cmp::Ordering, path::PathBuf};

use once_cell::sync::Lazy;
use regex::{Match, Regex};

use crate::syntax::{BuildError, BuildErrorLevel, BuildLog};

const MAX_LINE_LENGTH: usize = 79;

static PACKAGE_MESSAGE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new("^\\([a-zA-Z_\\-]+\\)\\s*(?P<msg>.*)$").unwrap());

static FILE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new("\\((?P<file>[^\r\n()]+\\.(tex|sty|cls))").unwrap());

static TEX_ERROR_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new("(?m)^! ((?P<msg1>(.|\r|\n)*?)\r?\nl\\.(?P<line>\\d+)( (\\.\\.\\.)?(?P<hint>[^\r\n]+))?|(?P<msg2>[^\r\n]*))").unwrap()
});

static WARNING_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new("(?P<msg>(LaTeX|Package [a-zA-Z_\\-]+) Warning: [^\r\n]*?(on input line (?P<line>\\d+))?\\.)[\r\n]").unwrap()
});

static BAD_BOX_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new("(?P<msg>(Ov|Und)erfull \\\\[hv]box[^\r\n]*lines? (?P<line>\\d+)[^\r\n]*)").unwrap()
});

pub fn parse_build_log(log: &str) -> BuildLog {
    let log = prepare_log(log);
    let mut ranges: Vec<FileRange> = FILE_REGEX
        .find_iter(&log)
        .map(|result| FileRange::create(&log, result))
        .collect();
    ranges.sort();

    let tex_errors = extract_matches(&log, &ranges, &TEX_ERROR_REGEX, BuildErrorLevel::Error);
    let warnings = extract_matches(&log, &ranges, &WARNING_REGEX, BuildErrorLevel::Warning);
    let bad_boxes = extract_matches(&log, &ranges, &BAD_BOX_REGEX, BuildErrorLevel::Warning);

    let errors = vec![tex_errors, warnings, bad_boxes].concat();
    BuildLog { errors }
}

fn extract_matches(
    log: &str,
    ranges: &[FileRange],
    regex: &Regex,
    level: BuildErrorLevel,
) -> Vec<BuildError> {
    let mut errors = Vec::new();
    for result in regex.find_iter(log) {
        let captures = regex.captures(&log[result.start()..result.end()]).unwrap();
        let message = captures
            .name("msg")
            .or_else(|| captures.name("msg1"))
            .or_else(|| captures.name("msg2"))
            .unwrap()
            .as_str()
            .lines()
            .next()
            .unwrap_or_default()
            .to_owned();

        if let Some(range) = ranges.iter().find(|range| range.contains(result.start())) {
            let line = captures
                .name("line")
                .map(|result| result.as_str().parse::<u32>().unwrap() - 1);

            let hint: Option<String> = if line.is_some() {
                captures
                    .name("hint")
                    .map(|r| String::from(r.as_str().trim()))
            } else {
                None
            };

            errors.push(BuildError {
                relative_path: range.path.clone(),
                level,
                message,
                line,
                hint,
            });
        }
    }
    errors
}

fn prepare_log(log: &str) -> String {
    let mut old_lines = log.lines();
    let mut new_lines: Vec<String> = Vec::new();
    while let Some(line) = old_lines.next() {
        if PACKAGE_MESSAGE_REGEX.is_match(line) {
            let captures = PACKAGE_MESSAGE_REGEX.captures(line).unwrap();
            if let Some(last_line) = new_lines.last_mut() {
                last_line.push(' ');
                last_line.push_str(captures.name("msg").unwrap().as_str());
            }
        } else if line.ends_with("...") {
            let mut new_line = line[line.len() - 3..].to_owned();
            if let Some(old_line) = old_lines.next() {
                new_line.push_str(old_line);
            }
            new_lines.push(new_line);
        } else if line.chars().count() == MAX_LINE_LENGTH {
            let mut new_line = String::new();
            new_line.push_str(line);
            for old_line in old_lines.by_ref() {
                new_line.push_str(old_line);
                if old_line.chars().count() != MAX_LINE_LENGTH {
                    break;
                }
            }
            new_lines.push(new_line);
        } else {
            new_lines.push(line.to_owned());
        }
    }
    new_lines.join("\n")
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct FileRange {
    pub path: PathBuf,
    pub start: usize,
    pub end: usize,
}

impl FileRange {
    fn create(log: &str, result: Match) -> Self {
        let mut balance = 1;
        let mut end = result.start() + 1;
        let chars = log[result.start() + 1..].chars();
        for c in chars {
            if balance <= 0 {
                break;
            }

            if c == '(' {
                balance += 1;
            } else if c == ')' {
                balance -= 1;
            }
            end += c.len_utf8();
        }

        let captures = FILE_REGEX.captures(result.as_str()).unwrap();
        let path = PathBuf::from(captures.name("file").unwrap().as_str());
        Self {
            path,
            start: result.start(),
            end,
        }
    }

    fn len(&self) -> usize {
        self.end - self.start + 1
    }

    fn contains(&self, index: usize) -> bool {
        index >= self.start && index <= self.end
    }
}

impl Ord for FileRange {
    fn cmp(&self, other: &Self) -> Ordering {
        self.len().cmp(&other.len())
    }
}

impl PartialOrd for FileRange {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::parse_build_log;

    #[test]
    fn test_parse() {
        insta::glob!("test_data/build_log/*.txt", |path| {
            let text = std::fs::read_to_string(path).unwrap().replace("\r\n", "\n");
            insta::assert_debug_snapshot!(parse_build_log(&text));
        });
    }
}
