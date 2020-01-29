use once_cell::sync::Lazy;
use path_clean::PathClean;
use regex::{Match, Regex};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::str;
use std::time::SystemTime;
use texlab_protocol::*;
use texlab_workspace::Document;

#[derive(Debug, PartialEq, Eq, Clone)]
struct LogFile {
    path: PathBuf,
    modified: SystemTime,
}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct BuildDiagnosticsProvider {
    diagnostics_by_uri: HashMap<Uri, Vec<Diagnostic>>,
    log_files: Vec<LogFile>,
}

impl BuildDiagnosticsProvider {
    pub fn get(&self, document: &Document) -> Vec<Diagnostic> {
        match self.diagnostics_by_uri.get(&document.uri) {
            Some(diagnostics) => diagnostics.to_owned(),
            None => Vec::new(),
        }
    }

    pub fn update(&mut self, tex_uri: &Uri, options: &LatexOptions) -> io::Result<bool> {
        if tex_uri.scheme() != "file" {
            return Ok(false);
        }

        let tex_path = tex_uri.to_file_path().unwrap();
        let log_path = options.resolve_output_file(&tex_path, "log").unwrap();
        if !log_path.exists() {
            return Ok(false);
        }

        let modified = fs::metadata(&log_path)?.modified()?;

        for log_file in &mut self.log_files {
            if log_file.path == log_path {
                return if modified > log_file.modified {
                    log_file.modified = modified;
                    self.update_diagnostics(tex_uri, &log_path)
                } else {
                    Ok(false)
                };
            }
        }

        self.update_diagnostics(tex_uri, &log_path)?;
        self.log_files.push(LogFile {
            path: log_path,
            modified,
        });
        Ok(true)
    }

    fn update_diagnostics(&mut self, tex_uri: &Uri, log_path: &Path) -> io::Result<bool> {
        let log = String::from_utf8_lossy(&fs::read(log_path)?).into_owned();
        self.diagnostics_by_uri.clear();
        for error in parse_build_log(tex_uri, &log) {
            let diagnostics = self
                .diagnostics_by_uri
                .entry(error.uri.clone())
                .or_insert_with(Vec::new);
            diagnostics.push(error.into());
        }
        Ok(true)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BuildErrorKind {
    Error,
    Warning,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BuildError {
    pub uri: Uri,
    pub kind: BuildErrorKind,
    pub message: String,
    pub line: Option<u64>,
}

impl BuildError {
    pub fn new(uri: Uri, kind: BuildErrorKind, message: String, line: Option<u64>) -> Self {
        Self {
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
            Some("latex".into()),
            self.message,
            None,
        )
    }
}

const MAX_LINE_LENGTH: usize = 79;

pub static PACKAGE_MESSAGE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new("^\\([a-zA-Z_\\-]+\\)\\s*(?P<msg>.*)$").unwrap());

pub static FILE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new("\\((?P<file>[^\r\n()]+\\.(tex|sty|cls))").unwrap());

pub static TEX_ERROR_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new("(?m)^! ((?P<msg1>(.|\r|\n)*?)\r?\nl\\.(?P<line>\\d+)|(?P<msg2>[^\r\n]*))").unwrap()
});

pub static WARNING_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new("(LaTeX|Package [a-zA-Z_\\-]+) Warning: (?P<msg>[^\r\n]*)").unwrap());

pub static BAD_BOX_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new("(?P<msg>(Ov|Und)erfull \\\\[hv]box[^\r\n]*lines? (?P<line>\\d+)[^\r\n]*)").unwrap()
});

fn parse_build_log(uri: &Uri, log: &str) -> Vec<BuildError> {
    let log = prepare_log(log);
    let mut ranges: Vec<FileRange> = FILE_REGEX
        .find_iter(&log)
        .map(|result| create_file_range(uri.clone(), &log, result))
        .collect();
    ranges.sort();

    let tex_errors = extract_matches(&log, &uri, &ranges, &TEX_ERROR_REGEX, BuildErrorKind::Error);
    let warnings = extract_matches(&log, &uri, &ranges, &WARNING_REGEX, BuildErrorKind::Warning);
    let bad_boxes = extract_matches(&log, &uri, &ranges, &BAD_BOX_REGEX, BuildErrorKind::Warning);

    vec![tex_errors, warnings, bad_boxes].concat()
}

fn extract_matches(
    log: &str,
    parent_uri: &Uri,
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

fn prepare_log(log: &str) -> String {
    let mut old_lines = log.lines();
    let mut new_lines: Vec<String> = Vec::new();
    while let Some(line) = old_lines.next() {
        if PACKAGE_MESSAGE_REGEX.is_match(&line) {
            let captures = PACKAGE_MESSAGE_REGEX.captures(&line).unwrap();
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

fn create_file_range(parent: Uri, log: &str, result: Match) -> FileRange {
    let mut balance = 1;
    let mut end = result.start() + 1;
    let chars = (&log[result.start() + 1..]).chars();
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
    let mut base_path = PathBuf::from(parent.path());
    base_path.pop();
    let mut full_path = base_path.clone();
    full_path.push(captures.name("file").unwrap().as_str());
    let uri = if full_path.starts_with(base_path) {
        let mut full_path = PathBuf::from(full_path.to_string_lossy().replace("\\", "/"))
            .clean()
            .to_string_lossy()
            .into_owned();
        if cfg!(windows) && full_path.starts_with('/') {
            full_path.remove(0);
        }
        Uri::from_file_path(full_path).ok()
    } else {
        None
    };

    FileRange::new(uri, result.start(), end)
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct FileRange {
    pub uri: Option<Uri>,
    pub start: usize,
    pub end: usize,
}

impl FileRange {
    fn new(uri: Option<Uri>, start: usize, end: usize) -> Self {
        Self { uri, start, end }
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

    fn create_uri(name: &str) -> Uri {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(name);
        Uri::from_file_path(path.to_str().unwrap()).unwrap()
    }

    fn verify(name: &str, expected: Vec<BuildError>) {
        let log_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("logs")
            .join(name);

        let log = std::fs::read_to_string(log_path).unwrap();
        let actual = parse_build_log(&create_uri("parent.tex"), &log);
        assert_eq!(expected, actual);
    }

    #[test]
    fn bad_box() {
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
    fn related_documents() {
        let error = BuildError::new(
            create_uri("child.tex"),
            BuildErrorKind::Error,
            "Undefined control sequence.".to_owned(),
            Some(0),
        );
        verify("child-error.log", vec![error]);
    }

    #[test]
    fn citation_warning() {
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
    fn package_error() {
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
    fn package_warning() {
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
    fn tex_error() {
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
