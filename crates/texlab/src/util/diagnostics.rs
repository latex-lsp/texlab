use base_db::{util::filter_regex_patterns, Document, Workspace};
use diagnostics::{DiagnosticSource, ErrorCode, LabelErrorCode};
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

    let severity = match &diagnostic.code {
        ErrorCode::UnexpectedRCurly
        | ErrorCode::RCurlyInserted
        | ErrorCode::MismatchedEnvironment
        | ErrorCode::ExpectingLCurly
        | ErrorCode::ExpectingKey
        | ErrorCode::ExpectingRCurly
        | ErrorCode::ExpectingEq
        | ErrorCode::ExpectingFieldValue => DiagnosticSeverity::ERROR,
        ErrorCode::Build(error) => match error.level {
            BuildErrorLevel::Error => DiagnosticSeverity::ERROR,
            BuildErrorLevel::Warning => DiagnosticSeverity::WARNING,
        },
        ErrorCode::Label(LabelErrorCode::Undefined) => DiagnosticSeverity::HINT,
        ErrorCode::Label(LabelErrorCode::Unused) => DiagnosticSeverity::HINT,
    };

    let code = match &diagnostic.code {
        ErrorCode::UnexpectedRCurly => Some(1),
        ErrorCode::RCurlyInserted => Some(2),
        ErrorCode::MismatchedEnvironment => Some(3),
        ErrorCode::ExpectingLCurly => Some(4),
        ErrorCode::ExpectingKey => Some(5),
        ErrorCode::ExpectingRCurly => Some(6),
        ErrorCode::ExpectingEq => Some(7),
        ErrorCode::ExpectingFieldValue => Some(8),
        ErrorCode::Label(LabelErrorCode::Undefined) => Some(9),
        ErrorCode::Label(LabelErrorCode::Unused) => Some(10),
        ErrorCode::Build(_) => None,
    };

    let source = match &diagnostic.code {
        ErrorCode::UnexpectedRCurly
        | ErrorCode::RCurlyInserted
        | ErrorCode::MismatchedEnvironment
        | ErrorCode::ExpectingLCurly
        | ErrorCode::ExpectingKey
        | ErrorCode::ExpectingRCurly
        | ErrorCode::ExpectingEq
        | ErrorCode::ExpectingFieldValue
        | ErrorCode::Label(_) => "texlab",
        ErrorCode::Build(_) => "latex",
    };

    let message = String::from(match &diagnostic.code {
        ErrorCode::UnexpectedRCurly => "Unexpected \"}\"",
        ErrorCode::RCurlyInserted => "Missing \"}\" inserted",
        ErrorCode::MismatchedEnvironment => "Mismatched environment",
        ErrorCode::ExpectingLCurly => "Expecting a curly bracket: \"{\"",
        ErrorCode::ExpectingKey => "Expecting a key",
        ErrorCode::ExpectingRCurly => "Expecting a curly bracket: \"}\"",
        ErrorCode::ExpectingEq => "Expecting an equality sign: \"=\"",
        ErrorCode::ExpectingFieldValue => "Expecting a field value",
        ErrorCode::Label(LabelErrorCode::Undefined) => "Potentially undefined label",
        ErrorCode::Label(LabelErrorCode::Unused) => "Potentially unused label",
        ErrorCode::Build(error) => &error.message,
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
