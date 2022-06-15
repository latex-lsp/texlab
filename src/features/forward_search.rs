use std::{
    io,
    path::Path,
    process::{Command, Stdio},
};

use log::error;
use lsp_types::TextDocumentPositionParams;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use super::FeatureRequest;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize_repr, Deserialize_repr)]
#[repr(i32)]
pub enum ForwardSearchStatus {
    SUCCESS = 0,
    ERROR = 1,
    FAILURE = 2,
    UNCONFIGURED = 3,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct ForwardSearchResult {
    pub status: ForwardSearchStatus,
}

pub fn execute_forward_search(
    request: FeatureRequest<TextDocumentPositionParams>,
) -> Option<ForwardSearchResult> {
    let options = &request.workspace.environment.options.forward_search;

    if options.executable.is_none() || options.args.is_none() {
        return Some(ForwardSearchResult {
            status: ForwardSearchStatus::UNCONFIGURED,
        });
    }

    let root_document = request
        .workspace
        .documents_by_uri
        .values()
        .find(|document| {
            if let Some(data) = document.data.as_latex() {
                data.extras.has_document_environment
                    && !data
                        .extras
                        .explicit_links
                        .iter()
                        .filter_map(|link| link.as_component_name())
                        .any(|name| name == "subfiles.cls")
            } else {
                false
            }
        })
        .filter(|document| document.uri.scheme() == "file")?;

    let data = root_document.data.as_latex()?;
    let pdf_path = data
        .extras
        .implicit_links
        .pdf
        .iter()
        .filter_map(|uri| uri.to_file_path().ok())
        .find(|path| path.exists())?;

    let tex_path = request.main_document().uri.to_file_path().ok()?;

    let args: Vec<String> = options
        .args
        .as_ref()
        .unwrap()
        .iter()
        .flat_map(|arg| {
            replace_placeholder(&tex_path, &pdf_path, request.params.position.line, arg)
        })
        .collect();

    let status = match run_process(options.executable.as_ref().unwrap(), args) {
        Ok(()) => ForwardSearchStatus::SUCCESS,
        Err(why) => {
            error!("Unable to execute forward search: {}", why);
            ForwardSearchStatus::FAILURE
        }
    };
    Some(ForwardSearchResult { status })
}

/// Iterate overs chunks of a string. Either returns a slice of the
/// original string, or the placeholder replacement.
pub struct PlaceHolderIterator<'a> {
    remainder: &'a str,
    tex_file: &'a str,
    pdf_file: &'a str,
    line_number: &'a str,
}

impl<'a> PlaceHolderIterator<'a> {
    pub fn new(s: &'a str, tex_file: &'a str, pdf_file: &'a str, line_number: &'a str) -> Self {
        Self {
            remainder: s,
            tex_file,
            pdf_file,
            line_number,
        }
    }

    pub fn yield_remainder(&mut self) -> Option<&'a str> {
        let chunk = self.remainder;
        self.remainder = "";
        Some(chunk)
    }

    pub fn yield_placeholder(&mut self) -> Option<&'a str> {
        if self.remainder.len() >= 2 {
            let placeholder = self.remainder;
            self.remainder = &self.remainder[2..];
            match &placeholder[1..2] {
                "f" => Some(self.tex_file),
                "p" => Some(self.pdf_file),
                "l" => Some(self.line_number),
                "%" => Some("%"), // escape %
                _ => Some(&placeholder[0..2]),
            }
        } else {
            self.remainder = &self.remainder[1..];
            Some("%")
        }
    }

    pub fn yield_str(&mut self, end: usize) -> Option<&'a str> {
        let chunk = &self.remainder[..end];
        self.remainder = &self.remainder[end..];
        Some(chunk)
    }
}

impl<'a> Iterator for PlaceHolderIterator<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        return if self.remainder.is_empty() {
            None
        } else if self.remainder.starts_with("%") {
            self.yield_placeholder()
        } else {
            // yield up to the next % or to the end
            match self.remainder.find("%") {
                None => self.yield_remainder(),
                Some(end) => self.yield_str(end),
            }
        };
    }
}

fn replace_placeholder(
    tex_file: &Path,
    pdf_file: &Path,
    line_number: u32,
    argument: &str,
) -> Option<String> {
    let result = if argument.starts_with('"') || argument.ends_with('"') {
        argument.to_string()
    } else {
        let line = &(line_number + 1).to_string();
        let it = PlaceHolderIterator::new(argument, tex_file.to_str()?, pdf_file.to_str()?, line);
        it.collect::<Vec<&str>>().join("")
    };
    Some(result)
}

fn run_process(executable: &str, args: Vec<String>) -> io::Result<()> {
    Command::new(executable)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;
    Ok(())
}
