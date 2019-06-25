use crate::syntax::bibtex::BibtexSyntaxTree;
use futures::compat::*;
use lsp_types::*;
use std::borrow::Cow;
use std::process::{Command, Stdio};
use tempfile::tempdir;
use tokio_process::CommandExt;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CitationError {
    InitializationFailed,
    InvalidEntry,
    NodeNotInstalled,
    ScriptFaulty,
    InvalidOutput,
}

pub async fn render_citation(entry_code: &str) -> Result<MarkupContent, CitationError> {
    let tree = BibtexSyntaxTree::from(entry_code);
    if tree.entries().iter().any(|entry| entry.fields.len() == 0) {
        return Err(CitationError::InvalidEntry);
    }

    let directory = tempdir().map_err(|_| CitationError::InitializationFailed)?;
    let entry_path = directory.path().join("entry.bib");
    tokio::fs::write(entry_path, &entry_code)
        .compat()
        .await
        .map_err(|_| CitationError::InitializationFailed)?;

    let mut process = Command::new("node")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .current_dir(directory.path())
        .spawn_async()
        .map_err(|_| CitationError::NodeNotInstalled)?;

    tokio::io::write_all(process.stdin().as_mut().unwrap(), SCRIPT)
        .compat()
        .await
        .map_err(|_| CitationError::ScriptFaulty)?;

    let output = process
        .wait_with_output()
        .compat()
        .await
        .map_err(|_| CitationError::ScriptFaulty)?;

    if output.status.success() {
        let html = String::from_utf8(output.stdout).map_err(|_| CitationError::InvalidOutput)?;
        let markdown = html2md::parse_html(&html);
        Ok(MarkupContent {
            kind: MarkupKind::Markdown,
            value: Cow::from(markdown.trim().to_owned()),
        })
    } else {
        Err(CitationError::InvalidEntry)
    }
}

const SCRIPT: &str = include_str!("../../citeproc/dist/citeproc.js");

#[cfg(test)]
mod tests {
    use super::*;

    #[runtime::test(runtime_tokio::Tokio)]
    async fn test_valid() {
        let markdown =
            render_citation("@article{foo, author = {Foo Bar}, title = {Baz Qux}, year = 1337}")
                .await;
        assert_eq!(markdown.unwrap().value, "Bar, F. (1337). Baz Qux.");
    }

    #[runtime::test(runtime_tokio::Tokio)]
    async fn test_invalid() {
        let markdown = render_citation("@article{}").await;
        assert_eq!(markdown, Err(CitationError::InvalidEntry));
    }

    #[runtime::test(runtime_tokio::Tokio)]
    async fn test_empty() {
        let markdown = render_citation("@article{foo,}").await;
        assert_eq!(markdown, Err(CitationError::InvalidEntry));
    }
}
