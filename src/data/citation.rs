use crate::syntax::bibtex::BibtexSyntaxTree;
use lsp_types::*;
use std::borrow::Cow;
use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};
use tempfile::tempdir;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CitationError {
    InitializationFailed,
    InvalidEntry,
    NodeNotInstalled,
    ScriptFaulty,
    InvalidOutput,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Citation<'a> {
    entry_code: &'a str,
}

impl<'a> Citation<'a> {
    pub fn new(entry_code: &'a str) -> Self {
        Self { entry_code }
    }

    pub fn render(&self) -> Result<MarkupContent, CitationError> {
        let entry = BibtexSyntaxTree::from(self.entry_code);
        if entry.entries().iter().any(|entry| entry.fields.len() == 0) {
            return Err(CitationError::InvalidEntry);
        }

        let directory = tempdir().map_err(|_| CitationError::InitializationFailed)?;
        let entry_path = directory.path().join("entry.bib");
        fs::write(entry_path, self.entry_code).map_err(|_| CitationError::InitializationFailed)?;

        let mut process = Command::new("node")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .current_dir(directory.path())
            .spawn()
            .map_err(|_| CitationError::NodeNotInstalled)?;

        process
            .stdin
            .as_mut()
            .unwrap()
            .write_all(SCRIPT.as_bytes())
            .map_err(|_| CitationError::ScriptFaulty)?;

        let output = process
            .wait_with_output()
            .map_err(|_| CitationError::ScriptFaulty)?;

        if output.status.success() {
            let html =
                String::from_utf8(output.stdout).map_err(|_| CitationError::InvalidOutput)?;
            let markdown = html2md::parse_html(&html);
            Ok(MarkupContent {
                kind: MarkupKind::Markdown,
                value: Cow::from(markdown.trim().to_owned()),
            })
        } else {
            Err(CitationError::InvalidEntry)
        }
    }
}

const SCRIPT: &str = include_str!("../../citeproc/dist/citeproc.js");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid() {
        let citation =
            Citation::new("@article{foo, author = {Foo Bar}, title = {Baz Qux}, year = 1337}");
        assert_eq!(citation.render().unwrap().value, "Bar, F. (1337). Baz Qux.");
    }

    #[test]
    fn test_invalid() {
        let citation = Citation::new("@article{}");
        assert_eq!(citation.render(), Err(CitationError::InvalidEntry));
    }

    #[test]
    fn test_empty() {
        let citation = Citation::new("@article{foo,}");
        assert_eq!(citation.render(), Err(CitationError::InvalidEntry));
    }
}
