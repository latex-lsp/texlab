use base_db::{Document, Workspace};
use diagnostics::{BibError, ChktexSeverity, Diagnostic, TexError};
use lsp_types::NumberOrString;
use rowan::TextRange;
use syntax::BuildErrorLevel;

use super::line_index_ext::LineIndexExt;

pub fn diagnostic(
    workspace: &Workspace,
    document: &Document,
    diagnostic: &Diagnostic,
) -> Option<lsp_types::Diagnostic> {
    let range = match diagnostic {
        Diagnostic::Tex(range, _) | Diagnostic::Bib(range, _) | Diagnostic::Build(range, _) => {
            document.line_index.line_col_lsp_range(*range)?
        }
        Diagnostic::Chktex(range) => {
            let start = lsp_types::Position::new(range.start.line, range.start.col);
            let end = lsp_types::Position::new(range.end.line, range.end.col);
            lsp_types::Range::new(start, end)
        }
    };

    let severity = match diagnostic {
        Diagnostic::Tex(_, error) => match error {
            TexError::UnexpectedRCurly => lsp_types::DiagnosticSeverity::ERROR,
            TexError::ExpectingRCurly => lsp_types::DiagnosticSeverity::ERROR,
            TexError::MismatchedEnvironment => lsp_types::DiagnosticSeverity::ERROR,
            TexError::UnusedLabel => lsp_types::DiagnosticSeverity::HINT,
            TexError::UndefinedLabel => lsp_types::DiagnosticSeverity::ERROR,
            TexError::UndefinedCitation => lsp_types::DiagnosticSeverity::ERROR,
            TexError::DuplicateLabel(_) => lsp_types::DiagnosticSeverity::ERROR,
        },
        Diagnostic::Bib(_, error) => match error {
            BibError::ExpectingLCurly => lsp_types::DiagnosticSeverity::ERROR,
            BibError::ExpectingKey => lsp_types::DiagnosticSeverity::ERROR,
            BibError::ExpectingRCurly => lsp_types::DiagnosticSeverity::ERROR,
            BibError::ExpectingEq => lsp_types::DiagnosticSeverity::ERROR,
            BibError::ExpectingFieldValue => lsp_types::DiagnosticSeverity::ERROR,
            BibError::UnusedEntry => lsp_types::DiagnosticSeverity::HINT,
            BibError::DuplicateEntry(_) => lsp_types::DiagnosticSeverity::ERROR,
        },
        Diagnostic::Build(_, error) => match error.level {
            BuildErrorLevel::Error => lsp_types::DiagnosticSeverity::ERROR,
            BuildErrorLevel::Warning => lsp_types::DiagnosticSeverity::WARNING,
        },
        Diagnostic::Chktex(error) => match error.severity {
            ChktexSeverity::Message => lsp_types::DiagnosticSeverity::HINT,
            ChktexSeverity::Warning => lsp_types::DiagnosticSeverity::WARNING,
            ChktexSeverity::Error => lsp_types::DiagnosticSeverity::ERROR,
        },
    };

    let code: Option<NumberOrString> = match &diagnostic {
        Diagnostic::Tex(_, error) => match error {
            TexError::UnexpectedRCurly => Some(NumberOrString::Number(1)),
            TexError::ExpectingRCurly => Some(NumberOrString::Number(2)),
            TexError::MismatchedEnvironment => Some(NumberOrString::Number(3)),
            TexError::UnusedLabel => Some(NumberOrString::Number(9)),
            TexError::UndefinedLabel => Some(NumberOrString::Number(10)),
            TexError::UndefinedCitation => Some(NumberOrString::Number(11)),
            TexError::DuplicateLabel(_) => Some(NumberOrString::Number(14)),
        },
        Diagnostic::Bib(_, error) => match error {
            BibError::ExpectingLCurly => Some(NumberOrString::Number(4)),
            BibError::ExpectingKey => Some(NumberOrString::Number(5)),
            BibError::ExpectingRCurly => Some(NumberOrString::Number(6)),
            BibError::ExpectingEq => Some(NumberOrString::Number(7)),
            BibError::ExpectingFieldValue => Some(NumberOrString::Number(8)),
            BibError::UnusedEntry => Some(NumberOrString::Number(12)),
            BibError::DuplicateEntry(_) => Some(NumberOrString::Number(13)),
        },
        Diagnostic::Build(_, _) => None,
        Diagnostic::Chktex(error) => Some(NumberOrString::String(error.code.clone())),
    };

    let source = match &diagnostic {
        Diagnostic::Tex(_, _) | Diagnostic::Bib(_, _) => "texlab",
        Diagnostic::Build(_, _) => "latex",
        Diagnostic::Chktex(_) => "ChkTeX",
    };

    let message = String::from(match &diagnostic {
        Diagnostic::Tex(_, error) => match error {
            TexError::UnexpectedRCurly => "Unexpected \"}\"",
            TexError::ExpectingRCurly => "Expecting a curly bracket: \"}\"",
            TexError::MismatchedEnvironment => "Mismatched environment",
            TexError::UnusedLabel => "Unused label",
            TexError::UndefinedLabel => "Undefined reference",
            TexError::UndefinedCitation => "Undefined reference",
            TexError::DuplicateLabel(_) => "Duplicate label",
        },
        Diagnostic::Bib(_, error) => match error {
            BibError::ExpectingLCurly => "Expecting a curly bracket: \"{\"",
            BibError::ExpectingKey => "Expecting a key",
            BibError::ExpectingRCurly => "Expecting a curly bracket: \"}\"",
            BibError::ExpectingEq => "Expecting an equality sign: \"=\"",
            BibError::ExpectingFieldValue => "Expecting a field value",
            BibError::UnusedEntry => "Unused entry",
            BibError::DuplicateEntry(_) => "Duplicate entry key",
        },
        Diagnostic::Build(_, error) => &error.message,
        Diagnostic::Chktex(error) => &error.message,
    });

    let tags = match &diagnostic {
        Diagnostic::Tex(_, error) => match error {
            TexError::UnexpectedRCurly => None,
            TexError::ExpectingRCurly => None,
            TexError::MismatchedEnvironment => None,
            TexError::UnusedLabel => Some(vec![lsp_types::DiagnosticTag::UNNECESSARY]),
            TexError::UndefinedLabel => None,
            TexError::UndefinedCitation => None,
            TexError::DuplicateLabel(_) => None,
        },
        Diagnostic::Bib(_, error) => match error {
            BibError::ExpectingLCurly => None,
            BibError::ExpectingKey => None,
            BibError::ExpectingRCurly => None,
            BibError::ExpectingEq => None,
            BibError::ExpectingFieldValue => None,
            BibError::UnusedEntry => Some(vec![lsp_types::DiagnosticTag::UNNECESSARY]),
            BibError::DuplicateEntry(_) => None,
        },
        Diagnostic::Build(_, _) => None,
        Diagnostic::Chktex(_) => None,
    };

    fn make_conflict_info(
        workspace: &Workspace,
        locations: &Vec<(lsp_types::Url, TextRange)>,
        object: &str,
    ) -> Option<Vec<lsp_types::DiagnosticRelatedInformation>> {
        let mut items = Vec::new();
        for (uri, range) in locations {
            let range = workspace
                .lookup(uri)?
                .line_index
                .line_col_lsp_range(*range)?;

            let message = format!("conflicting {object} defined here");
            let location = lsp_types::Location::new(uri.clone(), range);
            items.push(lsp_types::DiagnosticRelatedInformation { location, message });
        }

        Some(items)
    }

    let related_information = match &diagnostic {
        Diagnostic::Tex(_, error) => match error {
            TexError::UnexpectedRCurly => None,
            TexError::ExpectingRCurly => None,
            TexError::MismatchedEnvironment => None,
            TexError::UnusedLabel => None,
            TexError::UndefinedLabel => None,
            TexError::UndefinedCitation => None,
            TexError::DuplicateLabel(others) => make_conflict_info(workspace, others, "label"),
        },
        Diagnostic::Bib(_, error) => match error {
            BibError::ExpectingLCurly => None,
            BibError::ExpectingKey => None,
            BibError::ExpectingRCurly => None,
            BibError::ExpectingEq => None,
            BibError::ExpectingFieldValue => None,
            BibError::UnusedEntry => None,
            BibError::DuplicateEntry(others) => make_conflict_info(workspace, others, "entry"),
        },
        Diagnostic::Build(_, _) => None,
        Diagnostic::Chktex(_) => None,
    };

    Some(lsp_types::Diagnostic {
        severity: Some(severity),
        code,
        source: Some(String::from(source)),
        tags,
        related_information,
        ..lsp_types::Diagnostic::new_simple(range, message)
    })
}
