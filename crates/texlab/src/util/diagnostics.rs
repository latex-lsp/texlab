use base_db::{util::filter_regex_patterns, Document, Workspace};
use diagnostics::{
    types::{BibError, Diagnostic, DiagnosticData, TexError},
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
        DiagnosticData::Tex(error) => match error {
            TexError::UnexpectedRCurly => lsp_types::DiagnosticSeverity::ERROR,
            TexError::ExpectingRCurly => lsp_types::DiagnosticSeverity::ERROR,
            TexError::MismatchedEnvironment => lsp_types::DiagnosticSeverity::ERROR,
            TexError::UnusedLabel => lsp_types::DiagnosticSeverity::HINT,
            TexError::UndefinedLabel => lsp_types::DiagnosticSeverity::ERROR,
            TexError::UndefinedCitation => lsp_types::DiagnosticSeverity::ERROR,
        },
        DiagnosticData::Bib(error) => match error {
            BibError::ExpectingLCurly => lsp_types::DiagnosticSeverity::ERROR,
            BibError::ExpectingKey => lsp_types::DiagnosticSeverity::ERROR,
            BibError::ExpectingRCurly => lsp_types::DiagnosticSeverity::ERROR,
            BibError::ExpectingEq => lsp_types::DiagnosticSeverity::ERROR,
            BibError::ExpectingFieldValue => lsp_types::DiagnosticSeverity::ERROR,
            BibError::UnusedEntry => lsp_types::DiagnosticSeverity::HINT,
            BibError::DuplicateEntry(_) => lsp_types::DiagnosticSeverity::ERROR,
        },
        DiagnosticData::Build(error) => match error.level {
            BuildErrorLevel::Error => lsp_types::DiagnosticSeverity::ERROR,
            BuildErrorLevel::Warning => lsp_types::DiagnosticSeverity::WARNING,
        },
    };

    let code = match &diagnostic.data {
        DiagnosticData::Tex(error) => match error {
            TexError::UnexpectedRCurly => Some(1),
            TexError::ExpectingRCurly => Some(2),
            TexError::MismatchedEnvironment => Some(3),
            TexError::UnusedLabel => Some(9),
            TexError::UndefinedLabel => Some(10),
            TexError::UndefinedCitation => Some(11),
        },
        DiagnosticData::Bib(error) => match error {
            BibError::ExpectingLCurly => Some(4),
            BibError::ExpectingKey => Some(5),
            BibError::ExpectingRCurly => Some(6),
            BibError::ExpectingEq => Some(7),
            BibError::ExpectingFieldValue => Some(8),
            BibError::UnusedEntry => Some(12),
            BibError::DuplicateEntry(_) => Some(13),
        },
        DiagnosticData::Build(_) => None,
    };

    let source = match &diagnostic.data {
        DiagnosticData::Tex(_) | DiagnosticData::Bib(_) => "texlab",
        DiagnosticData::Build(_) => "latex",
    };

    let message = String::from(match &diagnostic.data {
        DiagnosticData::Tex(error) => match error {
            TexError::UnexpectedRCurly => "Unexpected \"}\"",
            TexError::ExpectingRCurly => "Expecting a curly bracket: \"}\"",
            TexError::MismatchedEnvironment => "Mismatched environment",
            TexError::UnusedLabel => "Unused label",
            TexError::UndefinedLabel => "Undefined reference",
            TexError::UndefinedCitation => "Undefined reference",
        },
        DiagnosticData::Bib(error) => match error {
            BibError::ExpectingLCurly => "Expecting a curly bracket: \"{\"",
            BibError::ExpectingKey => "Expecting a key",
            BibError::ExpectingRCurly => "Expecting a curly bracket: \"}\"",
            BibError::ExpectingEq => "Expecting an equality sign: \"=\"",
            BibError::ExpectingFieldValue => "Expecting a field value",
            BibError::UnusedEntry => "Unused entry",
            BibError::DuplicateEntry(_) => "Duplicate entry key",
        },
        DiagnosticData::Build(error) => &error.message,
    });

    let tags = match &diagnostic.data {
        DiagnosticData::Tex(error) => match error {
            TexError::UnexpectedRCurly => None,
            TexError::ExpectingRCurly => None,
            TexError::MismatchedEnvironment => None,
            TexError::UnusedLabel => Some(vec![lsp_types::DiagnosticTag::UNNECESSARY]),
            TexError::UndefinedLabel => None,
            TexError::UndefinedCitation => None,
        },
        DiagnosticData::Bib(error) => match error {
            BibError::ExpectingLCurly => None,
            BibError::ExpectingKey => None,
            BibError::ExpectingRCurly => None,
            BibError::ExpectingEq => None,
            BibError::ExpectingFieldValue => None,
            BibError::UnusedEntry => Some(vec![lsp_types::DiagnosticTag::UNNECESSARY]),
            BibError::DuplicateEntry(_) => None,
        },
        DiagnosticData::Build(_) => None,
    };

    let related_information = match &diagnostic.data {
        DiagnosticData::Tex(_) => None,
        DiagnosticData::Bib(error) => match error {
            BibError::ExpectingLCurly => None,
            BibError::ExpectingKey => None,
            BibError::ExpectingRCurly => None,
            BibError::ExpectingEq => None,
            BibError::ExpectingFieldValue => None,
            BibError::UnusedEntry => None,
            BibError::DuplicateEntry(ranges) => {
                let mut items = Vec::new();
                for range in ranges {
                    let range = document.line_index.line_col_lsp_range(*range);
                    let message = String::from("entry defined here");
                    let location = lsp_types::Location::new(document.uri.clone(), range);
                    items.push(lsp_types::DiagnosticRelatedInformation { location, message });
                }

                Some(items)
            }
        },
        DiagnosticData::Build(_) => None,
    };

    lsp_types::Diagnostic {
        severity: Some(severity),
        code: code.map(lsp_types::NumberOrString::Number),
        source: Some(String::from(source)),
        tags,
        related_information,
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
