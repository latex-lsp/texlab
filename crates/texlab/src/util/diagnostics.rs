use base_db::{util::filter_regex_patterns, Document, Workspace};
use diagnostics::{
    types::{CitationError, Diagnostic, DiagnosticData, LabelError, SyntaxError},
    DiagnosticBuilder, DiagnosticSource,
};
use rustc_hash::FxHashMap;
use syntax::BuildErrorLevel;

use super::line_index_ext::LineIndexExt;

pub fn collect<'db>(
    workspace: &'db Workspace,
    source: &mut dyn DiagnosticSource,
) -> FxHashMap<&'db Document, Vec<lsp_types::Diagnostic>> {
    let mut builder = DiagnosticBuilder::default();
    source.publish(workspace, &mut builder);
    builder
        .iter()
        .into_iter()
        .filter_map(|(uri, diags)| workspace.lookup(uri).map(|document| (document, diags)))
        .map(|(document, diags)| {
            let diags = diags
                .into_iter()
                .map(|diag| create_diagnostic(document, &diag))
                .collect::<Vec<_>>();

            (document, diags)
        })
        .collect()
}

fn create_diagnostic(document: &Document, diagnostic: &Diagnostic) -> lsp_types::Diagnostic {
    let range = document.line_index.line_col_lsp_range(diagnostic.range);

    let severity = match &diagnostic.data {
        DiagnosticData::Syntax(_) => lsp_types::DiagnosticSeverity::ERROR,
        DiagnosticData::Build(error) => match error.level {
            BuildErrorLevel::Error => lsp_types::DiagnosticSeverity::ERROR,
            BuildErrorLevel::Warning => lsp_types::DiagnosticSeverity::WARNING,
        },
        DiagnosticData::Label(LabelError::Undefined) => lsp_types::DiagnosticSeverity::ERROR,
        DiagnosticData::Label(LabelError::Unused) => lsp_types::DiagnosticSeverity::HINT,
        DiagnosticData::Citation(CitationError::Undefined) => lsp_types::DiagnosticSeverity::ERROR,
        DiagnosticData::Citation(CitationError::Unused) => lsp_types::DiagnosticSeverity::HINT,
    };

    let code = match &diagnostic.data {
        DiagnosticData::Syntax(error) => match error {
            SyntaxError::UnexpectedRCurly => Some(1),
            SyntaxError::RCurlyInserted => Some(2),
            SyntaxError::MismatchedEnvironment => Some(3),
            SyntaxError::ExpectingLCurly => Some(4),
            SyntaxError::ExpectingKey => Some(5),
            SyntaxError::ExpectingRCurly => Some(6),
            SyntaxError::ExpectingEq => Some(7),
            SyntaxError::ExpectingFieldValue => Some(8),
        },
        DiagnosticData::Label(LabelError::Undefined) => Some(9),
        DiagnosticData::Label(LabelError::Unused) => Some(10),
        DiagnosticData::Citation(CitationError::Undefined) => Some(11),
        DiagnosticData::Citation(CitationError::Unused) => Some(12),
        DiagnosticData::Build(_) => None,
    };

    let source = match &diagnostic.data {
        DiagnosticData::Syntax(_) | DiagnosticData::Label(_) | DiagnosticData::Citation(_) => {
            "texlab"
        }
        DiagnosticData::Build(_) => "latex",
    };

    let message = String::from(match &diagnostic.data {
        DiagnosticData::Syntax(error) => match error {
            SyntaxError::UnexpectedRCurly => "Unexpected \"}\"",
            SyntaxError::RCurlyInserted => "Missing \"}\" inserted",
            SyntaxError::MismatchedEnvironment => "Mismatched environment",
            SyntaxError::ExpectingLCurly => "Expecting a curly bracket: \"{\"",
            SyntaxError::ExpectingKey => "Expecting a key",
            SyntaxError::ExpectingRCurly => "Expecting a curly bracket: \"}\"",
            SyntaxError::ExpectingEq => "Expecting an equality sign: \"=\"",
            SyntaxError::ExpectingFieldValue => "Expecting a field value",
        },
        DiagnosticData::Label(LabelError::Undefined) => "Undefined reference",
        DiagnosticData::Label(LabelError::Unused) => "Unused label",
        DiagnosticData::Citation(CitationError::Undefined) => "Undefined reference",
        DiagnosticData::Citation(CitationError::Unused) => "Unused entry",
        DiagnosticData::Build(error) => &error.message,
    });

    let tags = match &diagnostic.data {
        DiagnosticData::Syntax(_)
        | DiagnosticData::Build(_)
        | DiagnosticData::Label(LabelError::Undefined)
        | DiagnosticData::Citation(CitationError::Undefined) => None,
        DiagnosticData::Label(LabelError::Unused) => {
            Some(vec![lsp_types::DiagnosticTag::UNNECESSARY])
        }
        DiagnosticData::Citation(CitationError::Unused) => {
            Some(vec![lsp_types::DiagnosticTag::UNNECESSARY])
        }
    };

    lsp_types::Diagnostic {
        severity: Some(severity),
        code: code.map(lsp_types::NumberOrString::Number),
        source: Some(String::from(source)),
        tags,
        ..lsp_types::Diagnostic::new_simple(range, message)
    }
}

pub fn filter(
    all_diagnostics: &mut FxHashMap<&Document, Vec<lsp_types::Diagnostic>>,
    workspace: &Workspace,
) {
    let config = &workspace.config().diagnostics;
    for diagnostics in all_diagnostics.values_mut() {
        diagnostics.retain(|diagnostic| {
            filter_regex_patterns(
                &diagnostic.message,
                &config.allowed_patterns,
                &config.ignored_patterns,
            )
        });
    }
}
