use base_db::{diagnostics::ErrorCode, Document, Workspace};
use distro::Language;
use lsp_types::{DiagnosticSeverity, NumberOrString};
use rustc_hash::FxHashMap;
use syntax::BuildErrorLevel;

use crate::util;

use super::line_index_ext::LineIndexExt;

pub fn collect(workspace: &Workspace) -> FxHashMap<&Document, Vec<lsp_types::Diagnostic>> {
    let mut results = FxHashMap::default();

    for document in workspace.iter() {
        let lsp_diagnostics = document
            .diagnostics
            .iter()
            .map(|diagnostic| create_diagnostic(document, diagnostic))
            .collect::<Vec<_>>();

        results.insert(document, lsp_diagnostics);
    }

    for document in workspace
        .iter()
        .filter(|document| document.language == Language::Log)
    {
        for (document, diagnostics) in base_db::diagnostics::log::analyze(workspace, document) {
            let lsp_diagnostics = diagnostics
                .iter()
                .map(|diagnostic| create_diagnostic(document, diagnostic))
                .collect::<Vec<_>>();

            results.get_mut(document).unwrap().extend(lsp_diagnostics);
        }
    }

    results
}

fn create_diagnostic(
    document: &Document,
    diagnostic: &base_db::diagnostics::Diagnostic,
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
        | ErrorCode::ExpectingFieldValue => "texlab",
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
            util::regex_filter::filter(
                &diagnostic.message,
                &config.allowed_patterns,
                &config.ignored_patterns,
            )
        });
    }
}
