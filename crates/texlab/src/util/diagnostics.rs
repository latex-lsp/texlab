use base_db::{util::filter_regex_patterns, Document, Workspace};
use diagnostics::{DiagnosticData, DiagnosticSource, LabelError, SyntaxError};
use lsp_types::{DiagnosticSeverity, NumberOrString};
use rustc_hash::FxHashMap;
use syntax::BuildErrorLevel;

use super::line_index_ext::LineIndexExt;

pub fn collect<'db>(
    workspace: &'db Workspace,
    source: &mut dyn DiagnosticSource,
) -> FxHashMap<&'db Document, Vec<lsp_types::Diagnostic>> {
    let mut results = FxHashMap::default();
    source.publish(workspace, &mut results);
    results
        .into_iter()
        .filter_map(|(uri, diags)| workspace.lookup(uri).map(|document| (document, diags)))
        .map(|(document, diags)| {
            let diags = diags
                .into_iter()
                .map(|diag| create_diagnostic(document, diag))
                .collect::<Vec<_>>();

            (document, diags)
        })
        .collect()
}

fn create_diagnostic(
    document: &Document,
    diagnostic: &diagnostics::Diagnostic,
) -> lsp_types::Diagnostic {
    let range = document.line_index.line_col_lsp_range(diagnostic.range);

    let severity = match &diagnostic.data {
        DiagnosticData::Syntax(_) => DiagnosticSeverity::ERROR,
        DiagnosticData::Build(error) => match error.level {
            BuildErrorLevel::Error => DiagnosticSeverity::ERROR,
            BuildErrorLevel::Warning => DiagnosticSeverity::WARNING,
        },
        DiagnosticData::Label(_) => DiagnosticSeverity::HINT,
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
        DiagnosticData::Build(_) => None,
    };

    let source = match &diagnostic.data {
        DiagnosticData::Syntax(_) | DiagnosticData::Label(_) => "texlab",
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
        DiagnosticData::Label(LabelError::Undefined) => "Potentially undefined label",
        DiagnosticData::Label(LabelError::Unused) => "Potentially unused label",
        DiagnosticData::Build(error) => &error.message,
    });

    lsp_types::Diagnostic {
        severity: Some(severity),
        code: code.map(NumberOrString::Number),
        source: Some(String::from(source)),
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
