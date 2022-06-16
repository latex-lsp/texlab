mod bibtex;
mod build;
mod chktex;
mod latex;

use std::sync::Arc;

use dashmap::DashMap;
use lsp_types::{DiagnosticSeverity, NumberOrString, Range, Url};
use regex::Regex;

use crate::Workspace;

use self::{
    bibtex::collect_bibtex_diagnostics, build::collect_build_diagnostics,
    chktex::collect_chktex_diagnostics, latex::collect_latex_diagnostics,
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Diagnostic {
    pub severity: DiagnosticSeverity,
    pub range: Range,
    pub code: DiagnosticCode,
    pub message: String,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub enum DiagnosticCode {
    Latex(LatexCode),
    Bibtex(BibtexCode),
    Chktex(String),
    Build(Arc<Url>),
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub enum LatexCode {
    UnexpectedRCurly,
    RCurlyInserted,
    MismatchedEnvironment,
}

impl From<LatexCode> for String {
    fn from(code: LatexCode) -> Self {
        match code {
            LatexCode::UnexpectedRCurly => "Unexpected \"}\"".to_string(),
            LatexCode::RCurlyInserted => "Missing \"}\" inserted".to_string(),
            LatexCode::MismatchedEnvironment => "Mismatched environment".to_string(),
        }
    }
}

impl From<LatexCode> for NumberOrString {
    fn from(code: LatexCode) -> Self {
        match code {
            LatexCode::UnexpectedRCurly => NumberOrString::Number(1),
            LatexCode::RCurlyInserted => NumberOrString::Number(2),
            LatexCode::MismatchedEnvironment => NumberOrString::Number(3),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
#[allow(clippy::enum_variant_names)]
pub enum BibtexCode {
    ExpectingLCurly,
    ExpectingKey,
    ExpectingRCurly,
    ExpectingEq,
    ExpectingFieldValue,
}

impl From<BibtexCode> for String {
    fn from(code: BibtexCode) -> Self {
        match code {
            BibtexCode::ExpectingLCurly => "Expecting a curly bracket: \"{\"".to_string(),
            BibtexCode::ExpectingKey => "Expecting a key".to_string(),
            BibtexCode::ExpectingRCurly => "Expecting a curly bracket: \"}\"".to_string(),
            BibtexCode::ExpectingEq => "Expecting an equality sign: \"=\"".to_string(),
            BibtexCode::ExpectingFieldValue => "Expecting a field value".to_string(),
        }
    }
}

impl From<BibtexCode> for NumberOrString {
    fn from(code: BibtexCode) -> Self {
        match code {
            BibtexCode::ExpectingLCurly => NumberOrString::Number(4),
            BibtexCode::ExpectingKey => NumberOrString::Number(5),
            BibtexCode::ExpectingRCurly => NumberOrString::Number(6),
            BibtexCode::ExpectingEq => NumberOrString::Number(7),
            BibtexCode::ExpectingFieldValue => NumberOrString::Number(8),
        }
    }
}

#[derive(Default, Clone)]
pub struct DiagnosticManager {
    all_diagnostics: Arc<DashMap<Arc<Url>, Vec<Diagnostic>>>,
}

impl DiagnosticManager {
    pub fn push_syntax(&self, workspace: &Workspace, uri: &Url) {
        collect_bibtex_diagnostics(&self.all_diagnostics, workspace, uri)
            .or_else(|| collect_latex_diagnostics(&self.all_diagnostics, workspace, uri))
            .or_else(|| collect_build_diagnostics(&self.all_diagnostics, workspace, uri));
    }

    pub fn push_chktex(&self, workspace: &Workspace, uri: &Url) {
        collect_chktex_diagnostics(&self.all_diagnostics, workspace, uri);
    }

    pub fn publish(&self, workspace: &Workspace, uri: &Url) -> Vec<lsp_types::Diagnostic> {
        let options = &workspace.environment.options.diagnostics;

        let mut results = Vec::new();
        if let Some(diagnostics) = self.all_diagnostics.get(uri) {
            for diagnostic in diagnostics.iter() {
                if !options.allowed_patterns.is_empty()
                    && !options
                        .allowed_patterns
                        .iter()
                        .any(|pattern| pattern.0.is_match(&diagnostic.message))
                {
                    continue;
                }

                if options
                    .ignored_patterns
                    .iter()
                    .any(|pattern| pattern.0.is_match(&diagnostic.message))
                {
                    continue;
                }

                let source = match diagnostic.code {
                    DiagnosticCode::Latex(_) | DiagnosticCode::Bibtex(_) => "texlab",
                    DiagnosticCode::Chktex(_) => "chktex",
                    DiagnosticCode::Build(_) => "latex-build",
                };

                let code = match diagnostic.code.clone() {
                    DiagnosticCode::Latex(code) => Some(code.into()),
                    DiagnosticCode::Bibtex(code) => Some(code.into()),
                    DiagnosticCode::Chktex(code) => Some(NumberOrString::String(code)),
                    DiagnosticCode::Build(_) => None,
                };

                results.push(lsp_types::Diagnostic {
                    range: diagnostic.range,
                    code,
                    severity: Some(diagnostic.severity),
                    message: diagnostic.message.clone(),
                    source: Some(source.to_string()),
                    ..Default::default()
                });
            }
        }

        results
    }
}

#[derive(Debug, Default)]
pub struct DiagnosticFilter {
    pub allowed_patterns: Vec<Regex>,
    pub ignored_patterns: Vec<Regex>,
}
