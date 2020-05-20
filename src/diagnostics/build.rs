use crate::{
    protocol::{Diagnostic, DiagnosticSeverity, Options, Position, Range, Uri},
    workspace::{Document, Snapshot},
};
use futures::lock::Mutex;
use once_cell::sync::Lazy;
use regex::{Match, Regex};
use std::{
    cmp::Ordering,
    collections::HashMap,
    io,
    path::{Path, PathBuf},
    str,
    time::SystemTime,
};
use tokio::fs;

#[derive(Debug, PartialEq, Eq, Clone)]
struct LogFile {
    path: PathBuf,
    modified: SystemTime,
}

#[derive(Debug, Default)]
pub struct BuildDiagnosticsProvider {
    diagnostics_by_uri: Mutex<HashMap<Uri, Vec<Diagnostic>>>,
    log_files: Mutex<Vec<LogFile>>,
}

impl BuildDiagnosticsProvider {
    pub async fn get(&self, doc: &Document) -> Vec<Diagnostic> {
        match self.diagnostics_by_uri.lock().await.get(&doc.uri) {
            Some(diagnostics) => diagnostics.to_owned(),
            None => Vec::new(),
        }
    }

    pub async fn update(
        &self,
        snapshot: &Snapshot,
        tex_uri: &Uri,
        options: &Options,
        current_dir: &Path,
    ) -> io::Result<bool> {
        let log_uris = snapshot
            .resolve_aux_targets(tex_uri, options, current_dir, "log")
            .unwrap_or_default();
        if let Some(log_path) = log_uris
            .into_iter()
            .filter(|uri| uri.scheme() == "file")
            .filter_map(|uri| uri.to_file_path().ok())
            .find(|path| path.exists())
        {
            let modified = fs::metadata(&log_path).await?.modified()?;
            let mut log_files = self.log_files.lock().await;
            for log_file in log_files.iter_mut() {
                if log_file.path == log_path {
                    return if modified > log_file.modified {
                        log_file.modified = modified;
                        self.update_diagnostics(tex_uri, &log_path).await
                    } else {
                        Ok(false)
                    };
                }
            }

            self.update_diagnostics(tex_uri, &log_path).await?;
            log_files.push(LogFile {
                path: log_path,
                modified,
            });
        }

        Ok(true)
    }

    async fn update_diagnostics(&self, tex_uri: &Uri, log_path: &Path) -> io::Result<bool> {
        let log = String::from_utf8_lossy(&fs::read(log_path).await?).into_owned();
        let mut diagnostics_by_uri = self.diagnostics_by_uri.lock().await;
        diagnostics_by_uri.clear();
        for error in parse_build_log(tex_uri, &log) {
            let diagnostics = diagnostics_by_uri
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
        Diagnostic {
            range,
            severity: Some(severity),
            source: Some("latex".into()),
            code: None,
            message: self.message,
            tags: None,
            related_information: None,
        }
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

pub fn prepare_log(log: &str) -> String {
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

pub fn create_file_range(parent: Uri, log: &str, result: Match) -> FileRange {
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
    let base_path = PathBuf::from(parent.path());
    let mut full_path = base_path.clone();
    full_path.push(captures.name("file").unwrap().as_str());
    let uri = if full_path.starts_with(base_path) {
        parent
            .join(captures.name("file").unwrap().as_str())
            .map(Into::into)
            .ok()
    } else {
        None
    };

    FileRange::new(uri, result.start(), end)
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FileRange {
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
    use indoc::indoc;

    fn create_uri(name: &str) -> Uri {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(name);
        Uri::from_file_path(path.to_str().unwrap()).unwrap()
    }

    #[test]
    fn bad_box() {
        let log = indoc!(
            r#"
                This is pdfTeX, Version 3.14159265-2.6-1.40.18 (TeX Live 2017/W32TeX) (preloaded format=pdflatex 2018.3.30)  26 DEC 2018 16:50
                entering extended mode
                restricted \write18 enabled.
                %&-line parsing enabled.
                **./parent.tex
                (./parent.tex
                LaTeX2e <2017-04-15>
                Babel <3.10> and hyphenation patterns for 84 language(s) loaded.
                (/TexLive/texmf-dist/tex/latex/base/article.cls
                Document Class: article 2014/09/29 v1.4h Standard LaTeX document class
                (/TexLive/texmf-dist/tex/latex/base/size10.clo
                File: size10.clo 2014/09/29 v1.4h Standard LaTeX file (size option)
                )
                \c@part=\count79
                \c@section=\count80
                \c@subsection=\count81
                \c@subsubsection=\count82
                \c@paragraph=\count83
                \c@subparagraph=\count84
                \c@figure=\count85
                \c@table=\count86
                \abovecaptionskip=\skip41
                \belowcaptionskip=\skip42
                \bibindent=\dimen102
                )
                (/TexLive/texmf-dist/tex/latex/multirow/bigstrut.sty
                Package: bigstrut 2016/11/25 v2.2 Provide larger struts in tabulars
                \bigstrutjot=\dimen103
                )
                (/TexLive/texmf-dist/tex/latex/multirow/multirow.sty
                Package: multirow 2016/11/25 v2.2 Span multiple rows of a table
                \multirow@colwidth=\skip43
                \multirow@cntb=\count87
                \multirow@dima=\skip44
                ) (./parent.aux)
                \openout1 = `parent.aux'.

                LaTeX Font Info:    Checking defaults for OML/cmm/m/it on input line 6.
                LaTeX Font Info:    ... okay on input line 6.
                LaTeX Font Info:    Checking defaults for T1/cmr/m/n on input line 6.
                LaTeX Font Info:    ... okay on input line 6.
                LaTeX Font Info:    Checking defaults for OT1/cmr/m/n on input line 6.
                LaTeX Font Info:    ... okay on input line 6.
                LaTeX Font Info:    Checking defaults for OMS/cmsy/m/n on input line 6.
                LaTeX Font Info:    ... okay on input line 6.
                LaTeX Font Info:    Checking defaults for OMX/cmex/m/n on input line 6.
                LaTeX Font Info:    ... okay on input line 6.
                LaTeX Font Info:    Checking defaults for U/cmr/m/n on input line 6.
                LaTeX Font Info:    ... okay on input line 6.

                Overfull \hbox (200.00162pt too wide) in paragraph at lines 8--9
                []\OT1/cmr/m/n/10 aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
                aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa 
                []

                LaTeX Font Info:    External font `cmex10' loaded for size
                (Font)              <7> on input line 11.
                LaTeX Font Info:    External font `cmex10' loaded for size
                (Font)              <5> on input line 11.

                Overfull \vbox (3.19998pt too high) detected at line 23
                []

                [1

                {/TexLive/texmf-var/fonts/map/pdftex/updmap/pdftex.map}] (./parent.aux) )
                Here is how much of TeX's memory you used:
                265 strings out of 492995
                3121 string characters out of 6138727
                55074 words of memory out of 5000000
                3896 multiletter control sequences out of 15000+600000
                3640 words of font info for 14 fonts, out of 8000000 for 9000
                1141 hyphenation exceptions out of 8191
                23i,20n,20p,124b,282s stack positions out of 5000i,500n,10000p,200000b,80000s
                </
                TexLive/texmf-dist/fonts/type1/public/amsfonts/cm/cmr10.pfb></TexLive/texmf-d
                ist/fonts/type1/public/amsfonts/cm/cmr7.pfb>
                Output written on parent.pdf (1 page, 17505 bytes).
                PDF statistics:
                16 PDF objects out of 1000 (max. 8388607)
                10 compressed objects within 1 object stream
                0 named destinations out of 1000 (max. 500000)
                1 words of extra memory for PDF output out of 10000 (max. 10000000)"#
        );

        let actual_errors = parse_build_log(&create_uri("parent.tex"), log);

        let error1 = BuildError::new(
            create_uri("parent.tex"),
            BuildErrorKind::Warning,
            "Overfull \\hbox (200.00162pt too wide) in paragraph at lines 8--9".into(),
            Some(7),
        );
        let error2 = BuildError::new(
            create_uri("parent.tex"),
            BuildErrorKind::Warning,
            "Overfull \\vbox (3.19998pt too high) detected at line 23".into(),
            Some(22),
        );
        assert_eq!(actual_errors, vec![error1, error2]);
    }

    #[test]
    fn related() {
        let log = indoc!(
            r#"
                This is pdfTeX, Version 3.14159265-2.6-1.40.18 (TeX Live 2017/W32TeX) (preloaded format=pdflatex 2018.3.30)  26 DEC 2018 16:52
                entering extended mode
                restricted \write18 enabled.
                %&-line parsing enabled.
                **./parent.tex
                (./parent.tex
                LaTeX2e <2017-04-15>
                Babel <3.10> and hyphenation patterns for 84 language(s) loaded.
                (/TexLive/texmf-dist/tex/latex/base/article.cls
                Document Class: article 2014/09/29 v1.4h Standard LaTeX document class
                (/TexLive/texmf-dist/tex/latex/base/size10.clo
                File: size10.clo 2014/09/29 v1.4h Standard LaTeX file (size option)
                )
                \c@part=\count79
                \c@section=\count80
                \c@subsection=\count81
                \c@subsubsection=\count82
                \c@paragraph=\count83
                \c@subparagraph=\count84
                \c@figure=\count85
                \c@table=\count86
                \abovecaptionskip=\skip41
                \belowcaptionskip=\skip42
                \bibindent=\dimen102
                ) (./parent.aux (./child.aux))
                \openout1 = `parent.aux'.
                
                LaTeX Font Info:    Checking defaults for OML/cmm/m/it on input line 4.
                LaTeX Font Info:    ... okay on input line 4.
                LaTeX Font Info:    Checking defaults for T1/cmr/m/n on input line 4.
                LaTeX Font Info:    ... okay on input line 4.
                LaTeX Font Info:    Checking defaults for OT1/cmr/m/n on input line 4.
                LaTeX Font Info:    ... okay on input line 4.
                LaTeX Font Info:    Checking defaults for OMS/cmsy/m/n on input line 4.
                LaTeX Font Info:    ... okay on input line 4.
                LaTeX Font Info:    Checking defaults for OMX/cmex/m/n on input line 4.
                LaTeX Font Info:    ... okay on input line 4.
                LaTeX Font Info:    Checking defaults for U/cmr/m/n on input line 4.
                LaTeX Font Info:    ... okay on input line 4.
                (./child.tex
                ! Undefined control sequence.
                l.1 \foo
                        
                The control sequence at the end of the top line
                of your error message was never \def'ed. If you have
                misspelled it (e.g., `\hobx'), type `I' and the correct
                spelling (e.g., `I\hbox'). Otherwise just continue,
                and I'll forget about whatever was undefined.
                
                ) (./parent.aux) ) 
                Here is how much of TeX's memory you used:
                205 strings out of 492995
                2149 string characters out of 6138727
                54074 words of memory out of 5000000
                3841 multiletter control sequences out of 15000+600000
                3640 words of font info for 14 fonts, out of 8000000 for 9000
                1141 hyphenation exceptions out of 8191
                23i,1n,17p,116b,36s stack positions out of 5000i,500n,10000p,200000b,80000s
                
                No pages of output.
                PDF statistics:
                0 PDF objects out of 1000 (max. 8388607)
                0 named destinations out of 1000 (max. 500000)
                1 words of extra memory for PDF output out of 10000 (max. 10000000)"#
        );

        let actual_errors = parse_build_log(&create_uri("parent.tex"), log);

        let error = BuildError::new(
            create_uri("child.tex"),
            BuildErrorKind::Error,
            "Undefined control sequence.".into(),
            Some(0),
        );
        assert_eq!(actual_errors, vec![error]);
    }

    #[test]
    fn citation_warning() {
        let log = indoc!(
            r#"
                This is pdfTeX, Version 3.14159265-2.6-1.40.18 (TeX Live 2017/W32TeX) (preloaded format=pdflatex 2018.3.30)  26 DEC 2018 16:51
                entering extended mode
                restricted \write18 enabled.
                %&-line parsing enabled.
                **./parent.tex
                (./parent.tex
                LaTeX2e <2017-04-15>
                Babel <3.10> and hyphenation patterns for 84 language(s) loaded.
                (/TexLive/texmf-dist/tex/latex/base/article.cls
                Document Class: article 2014/09/29 v1.4h Standard LaTeX document class
                (/TexLive/texmf-dist/tex/latex/base/size10.clo
                File: size10.clo 2014/09/29 v1.4h Standard LaTeX file (size option)
                )
                \c@part=\count79
                \c@section=\count80
                \c@subsection=\count81
                \c@subsubsection=\count82
                \c@paragraph=\count83
                \c@subparagraph=\count84
                \c@figure=\count85
                \c@table=\count86
                \abovecaptionskip=\skip41
                \belowcaptionskip=\skip42
                \bibindent=\dimen102
                ) (./parent.aux)
                \openout1 = `parent.aux'.
                
                LaTeX Font Info:    Checking defaults for OML/cmm/m/it on input line 4.
                LaTeX Font Info:    ... okay on input line 4.
                LaTeX Font Info:    Checking defaults for T1/cmr/m/n on input line 4.
                LaTeX Font Info:    ... okay on input line 4.
                LaTeX Font Info:    Checking defaults for OT1/cmr/m/n on input line 4.
                LaTeX Font Info:    ... okay on input line 4.
                LaTeX Font Info:    Checking defaults for OMS/cmsy/m/n on input line 4.
                LaTeX Font Info:    ... okay on input line 4.
                LaTeX Font Info:    Checking defaults for OMX/cmex/m/n on input line 4.
                LaTeX Font Info:    ... okay on input line 4.
                LaTeX Font Info:    Checking defaults for U/cmr/m/n on input line 4.
                LaTeX Font Info:    ... okay on input line 4.
                
                
                LaTeX Warning: Citation `foo' on page 1 undefined on input line 6.
                
                [1
                
                {/TexLive/texmf-var/fonts/map/pdftex/updmap/pdftex.map}] (./parent.aux)
                
                LaTeX Warning: There were undefined references.
                
                ) 
                Here is how much of TeX's memory you used:
                204 strings out of 492995
                2142 string characters out of 6138727
                54074 words of memory out of 5000000
                3842 multiletter control sequences out of 15000+600000
                3948 words of font info for 15 fonts, out of 8000000 for 9000
                1141 hyphenation exceptions out of 8191
                23i,4n,21p,116b,107s stack positions out of 5000i,500n,10000p,200000b,80000s
                </TexLive/texmf-dist/fonts/type1/public/amsfonts/cm/cmbx10.pfb></TexLive/
                texmf-dist/fonts/type1/public/amsfonts/cm/cmr10.pfb>
                Output written on parent.pdf (1 page, 17339 bytes).
                PDF statistics:
                16 PDF objects out of 1000 (max. 8388607)
                10 compressed objects within 1 object stream
                0 named destinations out of 1000 (max. 500000)
                1 words of extra memory for PDF output out of 10000 (max. 10000000)"#
        );

        let actual_errors = parse_build_log(&create_uri("parent.tex"), log);

        let error1 = BuildError::new(
            create_uri("parent.tex"),
            BuildErrorKind::Warning,
            "Citation `foo' on page 1 undefined on input line 6.".into(),
            None,
        );
        let error2 = BuildError::new(
            create_uri("parent.tex"),
            BuildErrorKind::Warning,
            "There were undefined references.".into(),
            None,
        );
        assert_eq!(actual_errors, vec![error1, error2]);
    }

    #[test]
    fn package_error() {
        let log = indoc!(
            r#"
                This is pdfTeX, Version 3.14159265-2.6-1.40.18 (TeX Live 2017/W32TeX) (preloaded format=pdflatex 2018.3.30)  26 DEC 2018 16:40
                entering extended mode
                restricted \write18 enabled.
                %&-line parsing enabled.
                **./parent.tex
                (./parent.tex
                LaTeX2e <2017-04-15>
                Babel <3.10> and hyphenation patterns for 84 language(s) loaded.
                (/TexLive/texmf-dist/tex/latex/base/article.cls
                Document Class: article 2014/09/29 v1.4h Standard LaTeX document class
                (/TexLive/texmf-dist/tex/latex/base/size10.clo
                File: size10.clo 2014/09/29 v1.4h Standard LaTeX file (size option)
                )
                \c@part=\count79
                \c@section=\count80
                \c@subsection=\count81
                \c@subsubsection=\count82
                \c@paragraph=\count83
                \c@subparagraph=\count84
                \c@figure=\count85
                \c@table=\count86
                \abovecaptionskip=\skip41
                \belowcaptionskip=\skip42
                \bibindent=\dimen102
                )
                (/TexLive/texmf-dist/tex/generic/babel/babel.sty
                Package: babel 2017/05/19 3.10 The Babel package
                
                
                ! Package babel Error: Unknown option `foo'. Either you misspelled it
                (babel)                or the language definition file foo.ldf was not found.
                
                See the babel package documentation for explanation.
                Type  H <return>  for immediate help.
                ...                                              
                                                                
                l.393 \ProcessOptions*
                                    
                Valid options are: shorthands=, KeepShorthandsActive,
                activeacute, activegrave, noconfigs, safe=, main=, math=
                headfoot=, strings=, config=, hyphenmap=, or a language name.
                
                
                ! Package babel Error: You haven't specified a language option.
                
                See the babel package documentation for explanation.
                Type  H <return>  for immediate help.
                ...                                              
                                                                
                l.426 ...ry to proceed from here, type x to quit.}
                                                                
                You need to specify a language, either as a global option
                or as an optional argument to the \usepackage command;
                You shouldn't try to proceed from here, type x to quit.
                
                ) (./parent.aux)
                \openout1 = `parent.aux'.
                
                LaTeX Font Info:    Checking defaults for OML/cmm/m/it on input line 5.
                LaTeX Font Info:    ... okay on input line 5.
                LaTeX Font Info:    Checking defaults for T1/cmr/m/n on input line 5.
                LaTeX Font Info:    ... okay on input line 5.
                LaTeX Font Info:    Checking defaults for OT1/cmr/m/n on input line 5.
                LaTeX Font Info:    ... okay on input line 5.
                LaTeX Font Info:    Checking defaults for OMS/cmsy/m/n on input line 5.
                LaTeX Font Info:    ... okay on input line 5.
                LaTeX Font Info:    Checking defaults for OMX/cmex/m/n on input line 5.
                LaTeX Font Info:    ... okay on input line 5.
                LaTeX Font Info:    Checking defaults for U/cmr/m/n on input line 5.
                LaTeX Font Info:    ... okay on input line 5.
                (./parent.aux) ) 
                Here is how much of TeX's memory you used:
                304 strings out of 492995
                3421 string characters out of 6138727
                56035 words of memory out of 5000000
                3938 multiletter control sequences out of 15000+600000
                3640 words of font info for 14 fonts, out of 8000000 for 9000
                1141 hyphenation exceptions out of 8191
                23i,1n,17p,116b,36s stack positions out of 5000i,500n,10000p,200000b,80000s
                
                No pages of output.
                PDF statistics:
                0 PDF objects out of 1000 (max. 8388607)
                0 named destinations out of 1000 (max. 500000)
                1 words of extra memory for PDF output out of 10000 (max. 10000000)"#
        );

        let actual_errors = parse_build_log(&create_uri("parent.tex"), log);

        let error1 = BuildError::new(
            create_uri("parent.tex"),
            BuildErrorKind::Error,
            "Package babel Error: Unknown option `foo'. Either you misspelled it or the language definition file foo.ldf was not found."
                .into(),
            Some(392),
        );
        let error2 = BuildError::new(
            create_uri("parent.tex"),
            BuildErrorKind::Error,
            "Package babel Error: You haven't specified a language option.".into(),
            Some(425),
        );
        assert_eq!(actual_errors, vec![error1, error2]);
    }

    #[test]
    fn package_warning() {
        let log = indoc!(
            r#"
                This is pdfTeX, Version 3.14159265-2.6-1.40.18 (TeX Live 2017/W32TeX) (preloaded format=pdflatex 2018.3.30)  26 DEC 2018 16:45
                entering extended mode
                restricted \write18 enabled.
                %&-line parsing enabled.
                **./parent.tex
                (./parent.tex
                LaTeX2e <2017-04-15>
                Babel <3.10> and hyphenation patterns for 84 language(s) loaded.
                (/TexLive/texmf-dist/tex/latex/base/article.cls
                Document Class: article 2014/09/29 v1.4h Standard LaTeX document class
                (/TexLive/texmf-dist/tex/latex/base/size10.clo
                File: size10.clo 2014/09/29 v1.4h Standard LaTeX file (size option)
                )
                \c@part=\count79
                \c@section=\count80
                \c@subsection=\count81
                \c@subsubsection=\count82
                \c@paragraph=\count83
                \c@subparagraph=\count84
                \c@figure=\count85
                \c@table=\count86
                \abovecaptionskip=\skip41
                \belowcaptionskip=\skip42
                \bibindent=\dimen102
                )
                (/TexLive/texmf-dist/tex/generic/babel/babel.sty
                Package: babel 2017/05/19 3.10 The Babel package
                
                (/TexLive/texmf-dist/tex/generic/babel-spanish/spanish.ldf
                Language: spanish.ldf 2016/03/03 v5.0p Spanish support from the babel system
                
                (/TexLive/texmf-dist/tex/generic/babel/babel.def
                File: babel.def 2017/05/19 3.10 Babel common definitions
                \babel@savecnt=\count87
                \U@D=\dimen103
                )
                \es@quottoks=\toks14
                \es@quotdepth=\count88
                Package babel Info: Making " an active character on input line 561.
                Package babel Info: Making . an active character on input line 662.
                Package babel Info: Making < an active character on input line 707.
                Package babel Info: Making > an active character on input line 707.
                ))
                (/TexLive/texmf-dist/tex/latex/biblatex/biblatex.sty
                Package: biblatex 2016/12/05 v3.7 programmable bibliographies (PK/JW/AB)
                
                (/TexLive/texmf-dist/tex/generic/oberdiek/pdftexcmds.sty
                Package: pdftexcmds 2017/03/19 v0.25 Utility functions of pdfTeX for LuaTeX (HO
                )
                
                (/TexLive/texmf-dist/tex/generic/oberdiek/infwarerr.sty
                Package: infwarerr 2016/05/16 v1.4 Providing info/warning/error messages (HO)
                )
                (/TexLive/texmf-dist/tex/generic/oberdiek/ifluatex.sty
                Package: ifluatex 2016/05/16 v1.4 Provides the ifluatex switch (HO)
                Package ifluatex Info: LuaTeX not detected.
                )
                (/TexLive/texmf-dist/tex/generic/oberdiek/ltxcmds.sty
                Package: ltxcmds 2016/05/16 v1.23 LaTeX kernel commands for general use (HO)
                )
                (/TexLive/texmf-dist/tex/generic/oberdiek/ifpdf.sty
                Package: ifpdf 2017/03/15 v3.2 Provides the ifpdf switch
                )
                Package pdftexcmds Info: LuaTeX not detected.
                Package pdftexcmds Info: \pdf@primitive is available.
                Package pdftexcmds Info: \pdf@ifprimitive is available.
                Package pdftexcmds Info: \pdfdraftmode found.
                )
                (/TexLive/texmf-dist/tex/latex/etoolbox/etoolbox.sty
                Package: etoolbox 2017/01/02 v2.4 e-TeX tools for LaTeX (JAW)
                \etb@tempcnta=\count89
                )
                (/TexLive/texmf-dist/tex/latex/graphics/keyval.sty
                Package: keyval 2014/10/28 v1.15 key=value parser (DPC)
                \KV@toks@=\toks15
                )
                (/TexLive/texmf-dist/tex/latex/oberdiek/kvoptions.sty
                Package: kvoptions 2016/05/16 v3.12 Key value format for package options (HO)
                
                (/TexLive/texmf-dist/tex/generic/oberdiek/kvsetkeys.sty
                Package: kvsetkeys 2016/05/16 v1.17 Key value parser (HO)
                
                (/TexLive/texmf-dist/tex/generic/oberdiek/etexcmds.sty
                Package: etexcmds 2016/05/16 v1.6 Avoid name clashes with e-TeX commands (HO)
                Package etexcmds Info: Could not find \expanded.
                (etexcmds)             That can mean that you are not using pdfTeX 1.50 or
                (etexcmds)             that some package has redefined \expanded.
                (etexcmds)             In the latter case, load this package earlier.
                )))
                (/TexLive/texmf-dist/tex/latex/logreq/logreq.sty
                Package: logreq 2010/08/04 v1.0 xml request logger
                \lrq@indent=\count90
                
                (/TexLive/texmf-dist/tex/latex/logreq/logreq.def
                File: logreq.def 2010/08/04 v1.0 logreq spec v1.0
                ))
                (/TexLive/texmf-dist/tex/latex/base/ifthen.sty
                Package: ifthen 2014/09/29 v1.1c Standard LaTeX ifthen package (DPC)
                )
                (/TexLive/texmf-dist/tex/latex/url/url.sty
                \Urlmuskip=\muskip10
                Package: url 2013/09/16  ver 3.4  Verb mode for urls, etc.
                )
                (/TexLive/texmf-dist/tex/generic/xstring/xstring.sty
                (/TexLive/texmf-dist/tex/generic/xstring/xstring.tex
                \@xs@message=\write3
                \integerpart=\count91
                \decimalpart=\count92
                )
                Package: xstring 2013/10/13  v1.7c  String manipulations (C Tellechea)
                )
                \c@tabx@nest=\count93
                \c@listtotal=\count94
                \c@listcount=\count95
                \c@liststart=\count96
                \c@liststop=\count97
                \c@citecount=\count98
                \c@citetotal=\count99
                \c@multicitecount=\count100
                \c@multicitetotal=\count101
                \c@instcount=\count102
                \c@maxnames=\count103
                \c@minnames=\count104
                \c@maxitems=\count105
                \c@minitems=\count106
                \c@citecounter=\count107
                \c@savedcitecounter=\count108
                \c@uniquelist=\count109
                \c@uniquename=\count110
                \c@refsection=\count111
                \c@refsegment=\count112
                \c@maxextratitle=\count113
                \c@maxextratitleyear=\count114
                \c@maxextrayear=\count115
                \c@maxextraalpha=\count116
                \c@abbrvpenalty=\count117
                \c@highnamepenalty=\count118
                \c@lownamepenalty=\count119
                \c@maxparens=\count120
                \c@parenlevel=\count121
                \blx@tempcnta=\count122
                \blx@tempcntb=\count123
                \blx@tempcntc=\count124
                \blx@maxsection=\count125
                \blx@maxsegment@0=\count126
                \blx@notetype=\count127
                \blx@parenlevel@text=\count128
                \blx@parenlevel@foot=\count129
                \blx@sectionciteorder@0=\count130
                \labelnumberwidth=\skip43
                \labelalphawidth=\skip44
                \biblabelsep=\skip45
                \bibitemsep=\skip46
                \bibnamesep=\skip47
                \bibinitsep=\skip48
                \bibparsep=\skip49
                \bibhang=\skip50
                \blx@bcfin=\read1
                \blx@bcfout=\write4
                \c@mincomprange=\count131
                \c@maxcomprange=\count132
                \c@mincompwidth=\count133
                Package biblatex Info: Trying to load biblatex default data model...
                Package biblatex Info: ... file 'blx-dm.def' found.
                
                (/TexLive/texmf-dist/tex/latex/biblatex/blx-dm.def)
                Package biblatex Info: Trying to load biblatex style data model...
                Package biblatex Info: ... file 'ieee.dbx' not found.
                Package biblatex Info: Trying to load biblatex custom data model...
                Package biblatex Info: ... file 'biblatex-dm.cfg' not found.
                \c@afterword=\count134
                \c@savedafterword=\count135
                \c@annotator=\count136
                \c@savedannotator=\count137
                \c@author=\count138
                \c@savedauthor=\count139
                \c@bookauthor=\count140
                \c@savedbookauthor=\count141
                \c@commentator=\count142
                \c@savedcommentator=\count143
                \c@editor=\count144
                \c@savededitor=\count145
                \c@editora=\count146
                \c@savededitora=\count147
                \c@editorb=\count148
                \c@savededitorb=\count149
                \c@editorc=\count150
                \c@savededitorc=\count151
                \c@foreword=\count152
                \c@savedforeword=\count153
                \c@holder=\count154
                \c@savedholder=\count155
                \c@introduction=\count156
                \c@savedintroduction=\count157
                \c@namea=\count158
                \c@savednamea=\count159
                \c@nameb=\count160
                \c@savednameb=\count161
                \c@namec=\count162
                \c@savednamec=\count163
                \c@translator=\count164
                \c@savedtranslator=\count165
                \c@shortauthor=\count166
                \c@savedshortauthor=\count167
                \c@shorteditor=\count168
                \c@savedshorteditor=\count169
                \c@labelname=\count170
                \c@savedlabelname=\count171
                \c@institution=\count172
                \c@savedinstitution=\count173
                \c@lista=\count174
                \c@savedlista=\count175
                \c@listb=\count176
                \c@savedlistb=\count177
                \c@listc=\count178
                \c@savedlistc=\count179
                \c@listd=\count180
                \c@savedlistd=\count181
                \c@liste=\count182
                \c@savedliste=\count183
                \c@listf=\count184
                \c@savedlistf=\count185
                \c@location=\count186
                \c@savedlocation=\count187
                \c@organization=\count188
                \c@savedorganization=\count189
                \c@origlocation=\count190
                \c@savedoriglocation=\count191
                \c@origpublisher=\count192
                \c@savedorigpublisher=\count193
                \c@publisher=\count194
                \c@savedpublisher=\count195
                \c@language=\count196
                \c@savedlanguage=\count197
                \c@pageref=\count198
                \c@savedpageref=\count199
                \shorthandwidth=\skip51
                \shortjournalwidth=\skip52
                \shortserieswidth=\skip53
                \shorttitlewidth=\skip54
                \shortauthorwidth=\skip55
                \shorteditorwidth=\skip56
                Package biblatex Info: Trying to load compatibility code...
                Package biblatex Info: ... file 'blx-compat.def' found.
                
                (/TexLive/texmf-dist/tex/latex/biblatex/blx-compat.def
                File: blx-compat.def 2016/12/05 v3.7 biblatex compatibility (PK/JW/AB)
                )
                Package biblatex Info: Trying to load generic definitions...
                Package biblatex Info: ... file 'biblatex.def' found.
                
                (/TexLive/texmf-dist/tex/latex/biblatex/biblatex.def
                File: biblatex.def 2016/12/05 v3.7 biblatex compatibility (PK/JW/AB)
                \c@textcitecount=\count266
                \c@textcitetotal=\count267
                \c@textcitemaxnames=\count268
                \c@biburlnumpenalty=\count269
                \c@biburlucpenalty=\count270
                \c@biburllcpenalty=\count271
                \c@smartand=\count272
                )
                Package biblatex Info: Trying to load bibliography style 'ieee'...
                Package biblatex Info: ... file 'ieee.bbx' found.
                
                (/TexLive/texmf-dist/tex/latex/biblatex-ieee/ieee.bbx
                File: ieee.bbx 2017/03/27 v1.2d biblatex bibliography style
                Package biblatex Info: Trying to load bibliography style 'numeric-comp'...
                Package biblatex Info: ... file 'numeric-comp.bbx' found.
                
                (/TexLive/texmf-dist/tex/latex/biblatex/bbx/numeric-comp.bbx
                File: numeric-comp.bbx 2016/12/05 v3.7 biblatex bibliography style (PK/JW/AB)
                Package biblatex Info: Trying to load bibliography style 'numeric'...
                Package biblatex Info: ... file 'numeric.bbx' found.
                
                (/TexLive/texmf-dist/tex/latex/biblatex/bbx/numeric.bbx
                File: numeric.bbx 2016/12/05 v3.7 biblatex bibliography style (PK/JW/AB)
                Package biblatex Info: Trying to load bibliography style 'standard'...
                Package biblatex Info: ... file 'standard.bbx' found.
                
                (/TexLive/texmf-dist/tex/latex/biblatex/bbx/standard.bbx
                File: standard.bbx 2016/12/05 v3.7 biblatex bibliography style (PK/JW/AB)
                \c@bbx:relatedcount=\count273
                \c@bbx:relatedtotal=\count274
                ))))
                Package biblatex Info: Trying to load citation style 'ieee'...
                Package biblatex Info: ... file 'ieee.cbx' found.
                
                (/TexLive/texmf-dist/tex/latex/biblatex-ieee/ieee.cbx
                File: ieee.cbx 2017/03/27 v1.2d biblatex citation style
                Package biblatex Info: Trying to load citation style 'numeric-comp'...
                Package biblatex Info: ... file 'numeric-comp.cbx' found.
                
                (/TexLive/texmf-dist/tex/latex/biblatex/cbx/numeric-comp.cbx
                File: numeric-comp.cbx 2016/12/05 v3.7 biblatex citation style (PK/JW/AB)
                \c@cbx@tempcnta=\count275
                \c@cbx@tempcntb=\count276
                Package biblatex Info: Redefining '\cite'.
                Package biblatex Info: Redefining '\parencite'.
                Package biblatex Info: Redefining '\footcite'.
                Package biblatex Info: Redefining '\footcitetext'.
                Package biblatex Info: Redefining '\smartcite'.
                Package biblatex Info: Redefining '\supercite'.
                Package biblatex Info: Redefining '\textcite'.
                Package biblatex Info: Redefining '\textcites'.
                Package biblatex Info: Redefining '\cites'.
                Package biblatex Info: Redefining '\parencites'.
                Package biblatex Info: Redefining '\smartcites'.
                )
                Package biblatex Info: Redefining '\cite'.
                Package biblatex Info: Redefining '\cites'.
                )
                Package biblatex Info: Trying to load configuration file...
                Package biblatex Info: ... file 'biblatex.cfg' found.
                
                (/TexLive/texmf-dist/tex/latex/biblatex/biblatex.cfg
                File: biblatex.cfg 
                ))
                Package biblatex Info: Trying to load language 'spanish'...
                Package biblatex Info: ... file 'spanish.lbx' found.
                
                (/TexLive/texmf-dist/tex/latex/biblatex/lbx/spanish.lbx
                File: spanish.lbx 2016/12/05 v3.7 biblatex localization (PK/JW/AB)
                )
                
                Package biblatex Warning: 'babel/polyglossia' detected but 'csquotes' missing.
                (biblatex)                Loading 'csquotes' recommended.
                
                \@quotelevel=\count277
                \@quotereset=\count278
                (./parent.aux)
                \openout1 = `parent.aux'.
                
                LaTeX Font Info:    Checking defaults for OML/cmm/m/it on input line 6.
                LaTeX Font Info:    ... okay on input line 6.
                LaTeX Font Info:    Checking defaults for T1/cmr/m/n on input line 6.
                LaTeX Font Info:    ... okay on input line 6.
                LaTeX Font Info:    Checking defaults for OT1/cmr/m/n on input line 6.
                LaTeX Font Info:    ... okay on input line 6.
                LaTeX Font Info:    Checking defaults for OMS/cmsy/m/n on input line 6.
                LaTeX Font Info:    ... okay on input line 6.
                LaTeX Font Info:    Checking defaults for OMX/cmex/m/n on input line 6.
                LaTeX Font Info:    ... okay on input line 6.
                LaTeX Font Info:    Checking defaults for U/cmr/m/n on input line 6.
                LaTeX Font Info:    ... okay on input line 6.
                LaTeX Info: Redefining \sptext on input line 6.
                LaTeX Info: Redefining \. on input line 6.
                LaTeX Info: Redefining \% on input line 6.
                Package biblatex Info: No input encoding detected.
                (biblatex)             Assuming 'ascii'.
                Package biblatex Info: Automatic encoding selection.
                (biblatex)             Assuming data encoding 'ascii'.
                \openout4 = `parent.bcf'.
                
                Package biblatex Info: Trying to load bibliographic data...
                Package biblatex Info: ... file 'parent.bbl' not found.
                
                No file parent.bbl.
                Package biblatex Info: Reference section=0 on input line 6.
                Package biblatex Info: Reference segment=0 on input line 6.
                (./parent.aux)
                
                LaTeX Warning: There were undefined references.
                
                
                Package biblatex Warning: Please (re)run Biber on the file:
                (biblatex)                parent
                (biblatex)                and rerun LaTeX afterwards.
                
                Package logreq Info: Writing requests to 'parent.run.xml'.
                \openout1 = `parent.run.xml'.
                
                ) 
                Here is how much of TeX's memory you used:
                7717 strings out of 492995
                133301 string characters out of 6138727
                557258 words of memory out of 5000000
                11248 multiletter control sequences out of 15000+600000
                3640 words of font info for 14 fonts, out of 8000000 for 9000
                1141 hyphenation exceptions out of 8191
                35i,1n,30p,856b,700s stack positions out of 5000i,500n,10000p,200000b,80000s
                
                No pages of output.
                PDF statistics:
                0 PDF objects out of 1000 (max. 8388607)
                0 named destinations out of 1000 (max. 500000)
                1 words of extra memory for PDF output out of 10000 (max. 10000000)"#
        );

        let actual_errors = parse_build_log(&create_uri("parent.tex"), log);

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
            "There were undefined references.".into(),
            None,
        );
        let error3 = BuildError::new(
            create_uri("parent.tex"),
            BuildErrorKind::Warning,
            "Please (re)run Biber on the file: parent and rerun LaTeX afterwards.".into(),
            None,
        );
        assert_eq!(actual_errors, vec![error1, error2, error3]);
    }

    #[test]
    fn tex_error() {
        let log = indoc!(
            r#"
                This is pdfTeX, Version 3.14159265-2.6-1.40.18 (TeX Live 2017/W32TeX) (preloaded format=pdflatex 2018.3.30)  26 DEC 2018 16:33
                entering extended mode
                restricted \write18 enabled.
                %&-line parsing enabled.
                **./parent.tex
                (./parent.tex
                LaTeX2e <2017-04-15>
                Babel <3.10> and hyphenation patterns for 84 language(s) loaded.
                (/TexLive/texmf-dist/tex/latex/base/article.cls
                Document Class: article 2014/09/29 v1.4h Standard LaTeX document class
                (/TexLive/texmf-dist/tex/latex/base/size10.clo
                File: size10.clo 2014/09/29 v1.4h Standard LaTeX file (size option)
                )
                \c@part=\count79
                \c@section=\count80
                \c@subsection=\count81
                \c@subsubsection=\count82
                \c@paragraph=\count83
                \c@subparagraph=\count84
                \c@figure=\count85
                \c@table=\count86
                \abovecaptionskip=\skip41
                \belowcaptionskip=\skip42
                \bibindent=\dimen102
                ) (./parent.aux
                (./child.tex.aux))
                \openout1 = `parent.aux'.
                
                LaTeX Font Info:    Checking defaults for OML/cmm/m/it on input line 3.
                LaTeX Font Info:    ... okay on input line 3.
                LaTeX Font Info:    Checking defaults for T1/cmr/m/n on input line 3.
                LaTeX Font Info:    ... okay on input line 3.
                LaTeX Font Info:    Checking defaults for OT1/cmr/m/n on input line 3.
                LaTeX Font Info:    ... okay on input line 3.
                LaTeX Font Info:    Checking defaults for OMS/cmsy/m/n on input line 3.
                LaTeX Font Info:    ... okay on input line 3.
                LaTeX Font Info:    Checking defaults for OMX/cmex/m/n on input line 3.
                LaTeX Font Info:    ... okay on input line 3.
                LaTeX Font Info:    Checking defaults for U/cmr/m/n on input line 3.
                LaTeX Font Info:    ... okay on input line 3.
                \openout2 = `child.aux'.
                
                (./child.tex)
                ! Undefined control sequence.
                l.7 \foo
                        
                The control sequence at the end of the top line
                of your error message was never \def'ed. If you have
                misspelled it (e.g., `\hobx'), type `I' and the correct
                spelling (e.g., `I\hbox'). Otherwise just continue,
                and I'll forget about whatever was undefined.
                
                ! Missing $ inserted.
                <inserted text> 
                                $
                l.8 \bar
                        
                I've inserted a begin-math/end-math symbol since I think
                you left one out. Proceed, with fingers crossed.
                
                LaTeX Font Info:    External font `cmex10' loaded for size
                (Font)              <7> on input line 8.
                LaTeX Font Info:    External font `cmex10' loaded for size
                (Font)              <5> on input line 8.
                ! Undefined control sequence.
                l.9 \baz
                        
                The control sequence at the end of the top line
                of your error message was never \def'ed. If you have
                misspelled it (e.g., `\hobx'), type `I' and the correct
                spelling (e.g., `I\hbox'). Otherwise just continue,
                and I'll forget about whatever was undefined.
                
                ! Missing { inserted.
                <to be read again> 
                                \par 
                l.10 
                    
                A left brace was mandatory here, so I've put one in.
                You might want to delete and/or insert some corrections
                so that I will find a matching right brace soon.
                (If you're confused by all this, try typing `I}' now.)
                
                ! Missing $ inserted.
                <inserted text> 
                                $
                l.10 
                    
                I've inserted a begin-math/end-math symbol since I think
                you left one out. Proceed, with fingers crossed.
                
                ! Missing } inserted.
                <inserted text> 
                                }
                l.10 
                    
                I've inserted something that you may have forgotten.
                (See the <inserted text> above.)
                With luck, this will get me unwedged. But if you
                really didn't forget anything, try typing `2' now; then
                my insertion and my current dilemma will both disappear.
                
                [1
                
                
                
                {/TexLive/texmf-var/fonts/map/pdftex/updmap/pdftex.map}] (./parent.aux
                (./child.aux)) ) 
                Here is how much of TeX's memory you used:
                212 strings out of 492995
                2238 string characters out of 6138727
                54074 words of memory out of 5000000
                3843 multiletter control sequences out of 15000+600000
                3640 words of font info for 14 fonts, out of 8000000 for 9000
                1141 hyphenation exceptions out of 8191
                23i,4n,17p,116b,107s stack positions out of 5000i,500n,10000p,200000b,80000s
                </TexLive/texmf-dist/fonts/type1/public/amsfonts/cm/cmr10.pfb
                >
                Output written on parent.pdf (1 page, 8329 bytes).
                PDF statistics:
                12 PDF objects out of 1000 (max. 8388607)
                7 compressed objects within 1 object stream
                0 named destinations out of 1000 (max. 500000)
                1 words of extra memory for PDF output out of 10000 (max. 10000000)"#
        );

        let actual_errors = parse_build_log(&create_uri("parent.tex"), log);

        let error1 = BuildError::new(
            create_uri("parent.tex"),
            BuildErrorKind::Error,
            "Undefined control sequence.".into(),
            Some(6),
        );
        let error2 = BuildError::new(
            create_uri("parent.tex"),
            BuildErrorKind::Error,
            "Missing $ inserted.".into(),
            Some(7),
        );
        let error3 = BuildError::new(
            create_uri("parent.tex"),
            BuildErrorKind::Error,
            "Undefined control sequence.".into(),
            Some(8),
        );
        let error4 = BuildError::new(
            create_uri("parent.tex"),
            BuildErrorKind::Error,
            "Missing { inserted.".into(),
            Some(9),
        );
        let error5 = BuildError::new(
            create_uri("parent.tex"),
            BuildErrorKind::Error,
            "Missing $ inserted.".into(),
            Some(9),
        );
        let error6 = BuildError::new(
            create_uri("parent.tex"),
            BuildErrorKind::Error,
            "Missing } inserted.".into(),
            Some(9),
        );
        assert_eq!(
            actual_errors,
            vec![error1, error2, error3, error4, error5, error6]
        );
    }
}
