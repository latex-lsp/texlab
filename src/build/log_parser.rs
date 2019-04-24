use lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range};
use path_clean::PathClean;
use regex::Captures;
use regex::Match;
use regex::Regex;
use std::cmp::Ordering;
use std::path::PathBuf;
use std::str;
use url::Url;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BuildErrorKind {
    Error,
    Warning,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BuildError {
    pub uri: Url,
    pub kind: BuildErrorKind,
    pub message: String,
    pub line: Option<u64>,
}

impl BuildError {
    pub fn new(uri: Url, kind: BuildErrorKind, message: String, line: Option<u64>) -> Self {
        BuildError {
            uri,
            kind,
            message,
            line,
        }
    }
}

impl Into<Diagnostic> for BuildError {
    fn into(self) -> Diagnostic {
        let position = Position::new(self.line.unwrap_or(0), 0);
        let severity = match self.kind {
            BuildErrorKind::Error => DiagnosticSeverity::Error,
            BuildErrorKind::Warning => DiagnosticSeverity::Warning,
        };
        let range = Range::new(position, position);
        Diagnostic::new(
            range,
            Some(severity),
            None,
            Some("latex".to_owned()),
            self.message,
            None,
        )
    }
}

const MAX_LINE_LENGTH: usize = 79;

const PACKAGE_MESSAGE_PATTERN: &'static str = "^\\([a-zA-Z_\\-]+\\)\\s*(?P<msg>.*)$";

const FILE_PATTERN: &'static str = "\\((?P<file>[^\r\n()]+\\.(tex|sty|cls))";

const TEX_ERROR_PATTERN: &'static str =
    "(?m)^! ((?P<msg1>(.|\r|\n)*?)\r?\nl\\.(?P<line>\\d+)|(?P<msg2>[^\r\n]*))";

const WARNING_PATTERN: &'static str = "(LaTeX|Package [a-zA-Z_\\-]+) Warning: (?P<msg>[^\r\n]*)";

const BAD_BOX_PATTERN: &'static str =
    "(?P<msg>(Ov|Und)erfull \\\\[hv]box[^\r\n]*lines? (?P<line>\\d+)[^\r\n]*)";

struct BuildErrorParser {
    package_message_regex: Regex,
    file_regex: Regex,
    tex_error_regex: Regex,
    warning_regex: Regex,
    bad_box_regex: Regex,
}

impl BuildErrorParser {
    pub fn new() -> Self {
        BuildErrorParser {
            package_message_regex: Regex::new(PACKAGE_MESSAGE_PATTERN).unwrap(),
            file_regex: Regex::new(FILE_PATTERN).unwrap(),
            tex_error_regex: Regex::new(TEX_ERROR_PATTERN).unwrap(),
            warning_regex: Regex::new(WARNING_PATTERN).unwrap(),
            bad_box_regex: Regex::new(BAD_BOX_PATTERN).unwrap(),
        }
    }

    pub fn parse(&self, uri: Url, log: &str) -> Vec<BuildError> {
        let log = self.prepare_log(log);
        let mut ranges: Vec<FileRange> = self
            .file_regex
            .find_iter(&log)
            .map(|result| self.create_file_range(uri.clone(), &log, result))
            .collect();
        ranges.sort();

        let tex_errors = BuildErrorParser::extract_matches(
            &log,
            &uri,
            &ranges,
            &self.tex_error_regex,
            BuildErrorKind::Error,
        );
        let warnings = BuildErrorParser::extract_matches(
            &log,
            &uri,
            &ranges,
            &self.warning_regex,
            BuildErrorKind::Warning,
        );
        let bad_boxes = BuildErrorParser::extract_matches(
            &log,
            &uri,
            &ranges,
            &self.bad_box_regex,
            BuildErrorKind::Warning,
        );

        vec![tex_errors, warnings, bad_boxes].concat()
    }

    fn extract_matches(
        log: &str,
        parent_uri: &Url,
        ranges: &[FileRange],
        regex: &Regex,
        kind: BuildErrorKind,
    ) -> Vec<BuildError> {
        let mut errors = Vec::new();
        for result in regex.find_iter(&log) {
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
                    .map(|result| u64::from_str_radix(result.as_str(), 10).unwrap() - 1);

                let uri = range.uri.as_ref().unwrap_or(parent_uri);
                errors.push(BuildError::new(uri.clone(), kind, message, line));
            }
        }
        errors
    }

    fn prepare_log(&self, log: &str) -> String {
        let mut old_lines = log.lines();
        let mut new_lines: Vec<String> = Vec::new();
        while let Some(line) = old_lines.next() {
            if self.package_message_regex.is_match(&line) {
                let captures = self.package_message_regex.captures(&line).unwrap();
                if let Some(last_line) = new_lines.last_mut() {
                    last_line.push(' ');
                    last_line.push_str(captures.name("msg").unwrap().as_str());
                }
            } else if line.ends_with("...") {
                let mut new_line = line[line.len() - 3..].to_owned();
                if let Some(old_line) = old_lines.next() {
                    new_line.push_str(&old_line);
                }
                new_lines.push(new_line);
            } else if line.chars().count() == MAX_LINE_LENGTH {
                let mut new_line = String::new();
                new_line.push_str(line);
                if let Some(old_line) = old_lines.next() {
                    new_line.push_str(old_line);
                }
                new_lines.push(new_line);
            } else {
                new_lines.push(line.to_owned());
            }
        }
        new_lines.join("\n")
    }

    fn create_file_range(&self, parent: Url, log: &str, result: Match) -> FileRange {
        let mut balance = 1;
        let mut end = result.start() + 1;
        let mut chars = (&log[result.start() + 1..]).chars();
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

        let captures = self.file_regex.captures(result.as_str()).unwrap();
        let mut base_path = PathBuf::from(parent.path());
        base_path.pop();
        let mut full_path = base_path.clone();
        full_path.push(captures.name("file").unwrap().as_str());
        let url = if full_path.starts_with(base_path) {
            let mut full_path = PathBuf::from(full_path.to_string_lossy().replace("\\", "/"))
                .clean()
                .to_string_lossy()
                .into_owned();
            if cfg!(windows) && full_path.starts_with("/") {
                full_path.remove(0);
            }
            Url::from_file_path(full_path).ok()
        } else {
            None
        };

        FileRange::new(url, result.start(), end)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct FileRange {
    pub uri: Option<Url>,
    pub start: usize,
    pub end: usize,
}

impl FileRange {
    fn new(uri: Option<Url>, start: usize, end: usize) -> Self {
        FileRange { uri, start, end }
    }

    fn length(&self) -> usize {
        self.end - self.start + 1
    }

    fn contains(&self, index: usize) -> bool {
        index >= self.start && index <= self.end
    }
}

impl Ord for FileRange {
    fn cmp(&self, other: &FileRange) -> Ordering {
        self.length().cmp(&other.length())
    }
}

impl PartialOrd for FileRange {
    fn partial_cmp(&self, other: &FileRange) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn create_uri(name: &str) -> Url {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(name);
        Url::from_file_path(path.to_str().unwrap()).unwrap()
    }

    fn verify(name: &str, expected: Vec<BuildError>) {
        let log_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("test")
            .join("data")
            .join("logs")
            .join(name);

        let log = fs::read_to_string(log_path).unwrap();
        let parser = BuildErrorParser::new();
        let actual = parser.parse(create_uri("parent.tex"), &log);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_bad_box() {
        let error1 = BuildError::new(
            create_uri("parent.tex"),
            BuildErrorKind::Warning,
            "Overfull \\hbox (200.00162pt too wide) in paragraph at lines 8--9".to_owned(),
            Some(7),
        );
        let error2 = BuildError::new(
            create_uri("parent.tex"),
            BuildErrorKind::Warning,
            "Overfull \\vbox (3.19998pt too high) detected at line 23".to_owned(),
            Some(22),
        );
        verify("bad-box.log", vec![error1, error2]);
    }

    #[test]
    fn test_related_documents() {
        let error = BuildError::new(
            create_uri("child.tex"),
            BuildErrorKind::Error,
            "Undefined control sequence.".to_owned(),
            Some(0),
        );
        verify("child-error.log", vec![error]);
    }

    #[test]
    fn test_citation_warning() {
        let error1 = BuildError::new(
            create_uri("parent.tex"),
            BuildErrorKind::Warning,
            "Citation `foo' on page 1 undefined on input line 6.".to_owned(),
            None,
        );
        let error2 = BuildError::new(
            create_uri("parent.tex"),
            BuildErrorKind::Warning,
            "There were undefined references.".to_owned(),
            None,
        );
        verify("citation-warning.log", vec![error1, error2]);
    }

    #[test]
    fn test_package_error() {
        let error1 = BuildError::new(
            create_uri("parent.tex"),
            BuildErrorKind::Error,
            "Package babel Error: Unknown option `foo'. Either you misspelled it or the language definition file foo.ldf was not found."
                .to_owned(),
            Some(392),
        );
        let error2 = BuildError::new(
            create_uri("parent.tex"),
            BuildErrorKind::Error,
            "Package babel Error: You haven't specified a language option.".to_owned(),
            Some(425),
        );
        verify("package-error.log", vec![error1, error2]);
    }

    #[test]
    fn test_package_warning() {
        let error1 = BuildError::new(
            create_uri("parent.tex"),
            BuildErrorKind::Warning,
            "'babel/polyglossia' detected but 'csquotes' missing. Loading 'csquotes' recommended."
                .to_owned(),
            None,
        );
        let error2 = BuildError::new(
            create_uri("parent.tex"),
            BuildErrorKind::Warning,
            "There were undefined references.".to_owned(),
            None,
        );
        let error3 = BuildError::new(
            create_uri("parent.tex"),
            BuildErrorKind::Warning,
            "Please (re)run Biber on the file: parent and rerun LaTeX afterwards.".to_owned(),
            None,
        );
        verify("package-warning.log", vec![error1, error2, error3]);
    }

    #[test]
    fn test_tex_error() {
        let error1 = BuildError::new(
            create_uri("parent.tex"),
            BuildErrorKind::Error,
            "Undefined control sequence.".to_owned(),
            Some(6),
        );
        let error2 = BuildError::new(
            create_uri("parent.tex"),
            BuildErrorKind::Error,
            "Missing $ inserted.".to_owned(),
            Some(7),
        );
        let error3 = BuildError::new(
            create_uri("parent.tex"),
            BuildErrorKind::Error,
            "Undefined control sequence.".to_owned(),
            Some(8),
        );
        let error4 = BuildError::new(
            create_uri("parent.tex"),
            BuildErrorKind::Error,
            "Missing { inserted.".to_owned(),
            Some(9),
        );
        let error5 = BuildError::new(
            create_uri("parent.tex"),
            BuildErrorKind::Error,
            "Missing $ inserted.".to_owned(),
            Some(9),
        );
        let error6 = BuildError::new(
            create_uri("parent.tex"),
            BuildErrorKind::Error,
            "Missing } inserted.".to_owned(),
            Some(9),
        );
        verify(
            "tex-error.log",
            vec![error1, error2, error3, error4, error5, error6],
        );
    }
}
