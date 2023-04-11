use std::{
    io,
    path::{Path, PathBuf},
    process::Stdio,
};

use base_db::Workspace;
use log::error;
use lsp_types::{Position, Url};
use thiserror::Error;

use crate::util::line_index_ext::LineIndexExt;

#[derive(Debug, Error)]
pub enum Error {
    #[error("TeX document '{0}' not found")]
    TexNotFound(Url),

    #[error("TeX document '{0}' is invalid")]
    InvalidTexFile(Url),

    #[error("PDF document '{0}' not found")]
    PdfNotFound(PathBuf),

    #[error("TeX document '{0}' is not a local file")]
    NoLocalFile(Url),

    #[error("PDF viewer is not configured")]
    Unconfigured,

    #[error("Failed to spawn process: {0}")]
    Spawn(io::Error),
}

pub struct Command {
    program: String,
    args: Vec<String>,
}

impl Command {
    pub fn configure(
        workspace: &Workspace,
        uri: &Url,
        position: Option<Position>,
    ) -> Result<Self, Error> {
        let child = workspace
            .lookup(uri)
            .ok_or_else(|| Error::TexNotFound(uri.clone()))?;

        let parent = *workspace.parents(child).iter().next().unwrap_or(&child);
        if parent.uri.scheme() != "file" {
            return Err(Error::NoLocalFile(parent.uri.clone()));
        }

        let output_dir = workspace
            .output_dir(&workspace.current_dir(&parent.dir))
            .to_file_path()
            .unwrap();

        let tex_path = child
            .path
            .as_deref()
            .ok_or_else(|| Error::NoLocalFile(uri.clone()))?;

        let pdf_path = match parent
            .path
            .as_deref()
            .unwrap()
            .file_stem()
            .and_then(|stem| stem.to_str())
        {
            Some(stem) => output_dir.join(format!("{}.pdf", stem)),
            None => return Err(Error::InvalidTexFile(uri.clone())),
        };

        if !pdf_path.exists() {
            return Err(Error::PdfNotFound(pdf_path));
        }

        let position = position.unwrap_or_else(|| child.line_index.line_col_lsp(child.cursor));

        let Some(config) = &workspace.config().synctex else {
            return Err(Error::Unconfigured);
        };

        let program = config.program.clone();

        let args: Vec<_> = config
            .args
            .iter()
            .flat_map(|arg| replace_placeholder(tex_path, &pdf_path, position.line, arg))
            .collect();

        Ok(Self { program, args })
    }
}

impl Command {
    pub fn run(self) -> Result<(), Error> {
        log::debug!("Executing forward search: {} {:?}", self.program, self.args);

        std::process::Command::new(self.program)
            .args(self.args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map_err(Error::Spawn)?;

        Ok(())
    }
}

/// Iterate overs chunks of a string. Either returns a slice of the
/// original string, or the placeholder replacement.
struct PlaceHolderIterator<'a> {
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
        } else if self.remainder.starts_with('%') {
            self.yield_placeholder()
        } else {
            // yield up to the next % or to the end
            match self.remainder.find('%') {
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