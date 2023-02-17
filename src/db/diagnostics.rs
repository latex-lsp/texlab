pub mod bib;
pub mod log;
pub mod tex;

use lsp_types::{DiagnosticSeverity, NumberOrString, Range};
use rustc_hash::FxHashMap;

use crate::{db::workspace::Workspace, util, Db};

use super::document::{Document, Language};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Diagnostic {
    pub severity: DiagnosticSeverity,
    pub range: Range,
    pub code: DiagnosticCode,
    pub message: String,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub enum DiagnosticCode {
    Tex(TexCode),
    Bib(BibCode),
    Log(Document),
    Chktex(String),
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub enum TexCode {
    UnexpectedRCurly,
    RCurlyInserted,
    MismatchedEnvironment,
}

impl From<TexCode> for String {
    fn from(code: TexCode) -> Self {
        match code {
            TexCode::UnexpectedRCurly => "Unexpected \"}\"".to_string(),
            TexCode::RCurlyInserted => "Missing \"}\" inserted".to_string(),
            TexCode::MismatchedEnvironment => "Mismatched environment".to_string(),
        }
    }
}

impl From<TexCode> for NumberOrString {
    fn from(code: TexCode) -> Self {
        match code {
            TexCode::UnexpectedRCurly => NumberOrString::Number(1),
            TexCode::RCurlyInserted => NumberOrString::Number(2),
            TexCode::MismatchedEnvironment => NumberOrString::Number(3),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
#[allow(clippy::enum_variant_names)]
pub enum BibCode {
    ExpectingLCurly,
    ExpectingKey,
    ExpectingRCurly,
    ExpectingEq,
    ExpectingFieldValue,
}

impl From<BibCode> for String {
    fn from(code: BibCode) -> Self {
        match code {
            BibCode::ExpectingLCurly => "Expecting a curly bracket: \"{\"".to_string(),
            BibCode::ExpectingKey => "Expecting a key".to_string(),
            BibCode::ExpectingRCurly => "Expecting a curly bracket: \"}\"".to_string(),
            BibCode::ExpectingEq => "Expecting an equality sign: \"=\"".to_string(),
            BibCode::ExpectingFieldValue => "Expecting a field value".to_string(),
        }
    }
}

impl From<BibCode> for NumberOrString {
    fn from(code: BibCode) -> Self {
        match code {
            BibCode::ExpectingLCurly => NumberOrString::Number(4),
            BibCode::ExpectingKey => NumberOrString::Number(5),
            BibCode::ExpectingRCurly => NumberOrString::Number(6),
            BibCode::ExpectingEq => NumberOrString::Number(7),
            BibCode::ExpectingFieldValue => NumberOrString::Number(8),
        }
    }
}

#[salsa::tracked(return_ref)]
pub fn collect(db: &dyn Db, workspace: Workspace) -> FxHashMap<Document, Vec<Diagnostic>> {
    let mut results: FxHashMap<Document, Vec<Diagnostic>> = FxHashMap::default();

    for document in workspace.documents(db).iter().copied() {
        match document.language(db) {
            Language::Tex => {
                results.entry(document).or_default().extend(
                    tex::collect(db, document)
                        .iter()
                        .chain(document.linter(db).chktex(db))
                        .cloned(),
                );
            }
            Language::Bib => {
                results
                    .entry(document)
                    .or_default()
                    .extend(bib::collect(db, document).iter().cloned());
            }
            Language::Log => {
                log::collect(db, workspace, document)
                    .iter()
                    .for_each(|(document, diagnostics)| {
                        results
                            .entry(*document)
                            .or_default()
                            .extend(diagnostics.clone());
                    });
            }
            Language::TexlabRoot | Language::Tectonic => {}
        }
    }

    results
}

#[salsa::tracked]
pub fn collect_filtered(
    db: &dyn Db,
    workspace: Workspace,
) -> FxHashMap<Document, Vec<lsp_types::Diagnostic>> {
    let all_diagnostics = collect(db, workspace);
    let mut all_filtered: FxHashMap<Document, Vec<lsp_types::Diagnostic>> = FxHashMap::default();

    let options = &workspace.options(db).diagnostics;
    for document in workspace.documents(db) {
        let mut filtered = Vec::new();
        if !matches!(document.language(db), Language::Tex | Language::Bib) {
            continue;
        }

        if let Some(diagnostics) = all_diagnostics.get(document) {
            for diagnostic in diagnostics.iter().filter(|diag| {
                util::regex_filter::filter(
                    &diag.message,
                    &options.allowed_patterns,
                    &options.ignored_patterns,
                )
            }) {
                let source = match diagnostic.code {
                    DiagnosticCode::Tex(_) | DiagnosticCode::Bib(_) => "texlab",
                    DiagnosticCode::Log(_) => "latex-build",
                    DiagnosticCode::Chktex(_) => "chktex",
                };

                let code = match diagnostic.code.clone() {
                    DiagnosticCode::Tex(code) => Some(code.into()),
                    DiagnosticCode::Bib(code) => Some(code.into()),
                    DiagnosticCode::Chktex(code) => Some(NumberOrString::String(code)),
                    DiagnosticCode::Log(_) => None,
                };

                filtered.push(lsp_types::Diagnostic {
                    range: diagnostic.range,
                    code,
                    severity: Some(diagnostic.severity),
                    message: diagnostic.message.clone(),
                    source: Some(source.to_string()),
                    ..Default::default()
                });
            }
        }

        all_filtered.insert(*document, filtered);
    }

    all_filtered
}
