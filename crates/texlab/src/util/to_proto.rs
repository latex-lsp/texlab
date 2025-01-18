use std::collections::HashMap;

use base_db::{
    data::BibtexEntryTypeCategory, util::RenderedObject, Document, DocumentLocation, Workspace,
};
use definition::DefinitionResult;
use diagnostics::{BibError, ChktexSeverity, Diagnostic, TexError};
use folding::{FoldingRange, FoldingRangeKind};
use highlights::{Highlight, HighlightKind};
use hover::{Hover, HoverData};
use inlay_hints::{InlayHint, InlayHintData};
use line_index::LineIndex;
use lsp_types::NumberOrString;
use rename::RenameResult;
use rowan::TextRange;
use syntax::BuildErrorLevel;

use super::{line_index_ext::LineIndexExt, ClientFlags};

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

pub fn inlay_hint(
    hint: InlayHint,
    line_index: &LineIndex,
    max_length: Option<usize>,
) -> Option<lsp_types::InlayHint> {
    let position = line_index.line_col_lsp(hint.offset)?;
    let trim_text = |text: &mut String| match max_length {
        Some(max_length) if text.len() > max_length => {
            text.truncate(max_length);
            text.push('…');
        }
        _ => {}
    };

    Some(match hint.data {
        InlayHintData::LabelDefinition(label) => {
            let number = label.number?;

            let mut text = match &label.object {
                RenderedObject::Section { prefix, .. } => {
                    format!("{} {}", prefix, number)
                }
                RenderedObject::Float { kind, .. } => {
                    format!("{} {}", kind.as_str(), number)
                }
                RenderedObject::Theorem { kind, .. } => {
                    format!("{} {}", kind, number)
                }
                RenderedObject::Equation => format!("Equation ({})", number),
                RenderedObject::EnumItem => format!("Item {}", number),
            };

            trim_text(&mut text);

            lsp_types::InlayHint {
                position,
                label: lsp_types::InlayHintLabel::String(format!(" {text} ",)),
                kind: None,
                text_edits: None,
                tooltip: None,
                padding_left: Some(true),
                padding_right: None,
                data: None,
            }
        }
        InlayHintData::LabelReference(label) => {
            let mut text = label.reference();
            trim_text(&mut text);

            lsp_types::InlayHint {
                position,
                label: lsp_types::InlayHintLabel::String(format!(" {text} ")),
                kind: None,
                text_edits: None,
                tooltip: None,
                padding_left: Some(true),
                padding_right: None,
                data: None,
            }
        }
    })
}

pub fn document_link(
    link: DocumentLocation,
    line_index: &LineIndex,
) -> Option<lsp_types::DocumentLink> {
    Some(lsp_types::DocumentLink {
        data: None,
        tooltip: None,
        target: Some(link.document.uri.clone()),
        range: line_index.line_col_lsp_range(link.range)?,
    })
}

pub fn folding_range(
    folding: FoldingRange,
    line_index: &LineIndex,
    client_flags: &ClientFlags,
) -> Option<serde_json::Value> {
    let range = line_index.line_col_lsp_range(folding.range)?;

    let kind = if client_flags.folding_custom_kinds {
        Some(match folding.kind {
            FoldingRangeKind::Section => "section",
            FoldingRangeKind::Environment => "environment",
            FoldingRangeKind::Entry => "entry",
        })
    } else {
        None
    };

    Some(serde_json::json!({
        "startLine": range.start.line,
        "startCharacter": range.start.character,
        "endLine": range.end.line,
        "endCharacter": range.end.character,
        "kind": kind,
    }))
}

pub fn location_link(
    result: DefinitionResult,
    line_index: &LineIndex,
) -> Option<lsp_types::LocationLink> {
    let origin_selection_range = line_index.line_col_lsp_range(result.origin_selection_range);

    let target_line_index = &result.target.line_index;
    let target_uri = result.target.uri.clone();
    let target_range = target_line_index.line_col_lsp_range(result.target_range)?;
    let target_selection_range =
        target_line_index.line_col_lsp_range(result.target_selection_range)?;

    Some(lsp_types::LocationLink {
        origin_selection_range,
        target_uri,
        target_range,
        target_selection_range,
    })
}

pub fn document_symbol(
    symbol: symbols::Symbol,
    line_index: &LineIndex,
) -> Option<lsp_types::DocumentSymbol> {
    let children = symbol
        .children
        .into_iter()
        .filter_map(|child| document_symbol(child, line_index))
        .collect();

    #[allow(deprecated)]
    Some(lsp_types::DocumentSymbol {
        name: symbol.name,
        detail: symbol.label.map(|label| label.text),
        kind: symbol_kind(symbol.kind),
        deprecated: Some(false),
        range: line_index.line_col_lsp_range(symbol.full_range)?,
        selection_range: line_index.line_col_lsp_range(symbol.selection_range)?,
        children: Some(children),
        tags: None,
    })
}

pub fn symbol_information(
    symbol: symbols::Symbol,
    document: &Document,
    results: &mut Vec<lsp_types::SymbolInformation>,
) -> Option<()> {
    let range = document.line_index.line_col_lsp_range(symbol.full_range)?;

    #[allow(deprecated)]
    results.push(lsp_types::SymbolInformation {
        name: symbol.name,
        kind: symbol_kind(symbol.kind),
        deprecated: Some(false),
        location: lsp_types::Location::new(document.uri.clone(), range),
        tags: None,
        container_name: None,
    });

    for child in symbol.children {
        symbol_information(child, document, results);
    }

    Some(())
}

pub fn symbol_kind(value: symbols::SymbolKind) -> lsp_types::SymbolKind {
    match value {
        symbols::SymbolKind::Section => lsp_types::SymbolKind::MODULE,
        symbols::SymbolKind::Figure => lsp_types::SymbolKind::METHOD,
        symbols::SymbolKind::Algorithm => lsp_types::SymbolKind::METHOD,
        symbols::SymbolKind::Table => lsp_types::SymbolKind::METHOD,
        symbols::SymbolKind::Listing => lsp_types::SymbolKind::METHOD,
        symbols::SymbolKind::Enumeration => lsp_types::SymbolKind::ENUM,
        symbols::SymbolKind::EnumerationItem => lsp_types::SymbolKind::ENUM_MEMBER,
        symbols::SymbolKind::Theorem => lsp_types::SymbolKind::VARIABLE,
        symbols::SymbolKind::Equation => lsp_types::SymbolKind::CONSTANT,
        symbols::SymbolKind::Entry(category) => match category {
            BibtexEntryTypeCategory::Misc => lsp_types::SymbolKind::INTERFACE,
            BibtexEntryTypeCategory::String => lsp_types::SymbolKind::STRING,
            BibtexEntryTypeCategory::Article => lsp_types::SymbolKind::EVENT,
            BibtexEntryTypeCategory::Thesis => lsp_types::SymbolKind::OBJECT,
            BibtexEntryTypeCategory::Book => lsp_types::SymbolKind::STRUCT,
            BibtexEntryTypeCategory::Part => lsp_types::SymbolKind::OPERATOR,
            BibtexEntryTypeCategory::Collection => lsp_types::SymbolKind::TYPE_PARAMETER,
        },
        symbols::SymbolKind::Field => lsp_types::SymbolKind::FIELD,
        symbols::SymbolKind::Environment => lsp_types::SymbolKind::FUNCTION,
    }
}

pub fn document_symbol_response(
    document: &Document,
    symbols: Vec<symbols::Symbol>,
    client_flags: &ClientFlags,
) -> lsp_types::DocumentSymbolResponse {
    if client_flags.hierarchical_document_symbols {
        let results = symbols
            .into_iter()
            .filter_map(|symbol| document_symbol(symbol, &document.line_index))
            .collect();

        lsp_types::DocumentSymbolResponse::Nested(results)
    } else {
        let mut results = Vec::new();
        for symbol in symbols {
            symbol_information(symbol, document, &mut results);
        }

        lsp_types::DocumentSymbolResponse::Flat(results)
    }
}

pub fn workspace_edit(result: RenameResult, new_name: &str) -> lsp_types::WorkspaceEdit {
    let mut changes = HashMap::default();
    for (document, ranges) in result.changes {
        let mut edits = Vec::new();
        ranges
            .into_iter()
            .filter_map(|info| {
                document.line_index.line_col_lsp_range(info.range).map(|i| {
                    (
                        i,
                        info.prefix.map_or(new_name.into(), |p| p + new_name.into()),
                    )
                })
            })
            .for_each(|(range, new_name)| edits.push(lsp_types::TextEdit::new(range, new_name)));

        changes.insert(document.uri.clone(), edits);
    }

    lsp_types::WorkspaceEdit::new(changes)
}

pub fn location(location: DocumentLocation) -> Option<lsp_types::Location> {
    let document = location.document;
    let range = document.line_index.line_col_lsp_range(location.range)?;
    Some(lsp_types::Location::new(document.uri.clone(), range))
}

pub fn document_highlight(
    highlight: Highlight,
    line_index: &LineIndex,
) -> Option<lsp_types::DocumentHighlight> {
    let range = line_index.line_col_lsp_range(highlight.range)?;
    let kind = Some(match highlight.kind {
        HighlightKind::Write => lsp_types::DocumentHighlightKind::WRITE,
        HighlightKind::Read => lsp_types::DocumentHighlightKind::READ,
    });

    Some(lsp_types::DocumentHighlight { range, kind })
}

pub fn hover(
    hover: Hover,
    line_index: &LineIndex,
    client_flags: &ClientFlags,
) -> Option<lsp_types::Hover> {
    fn inline_image(
        command: &completion_data::Command,

        client_flags: &ClientFlags,
    ) -> Option<lsp_types::MarkupContent> {
        let name = &command.name;
        command
            .image
            .map(|base64| format!("![{name}](data:image/png;base64,{base64}|width=48,height=48)"))
            .filter(|_| client_flags.hover_markdown)
            .map(|value| lsp_types::MarkupContent {
                kind: lsp_types::MarkupKind::Markdown,
                value,
            })
            .or_else(|| {
                Some(lsp_types::MarkupContent {
                    kind: lsp_types::MarkupKind::PlainText,
                    value: command.glyph.as_deref()?.into(),
                })
            })
    }

    let contents = match hover.data {
        HoverData::Citation(text) => lsp_types::MarkupContent {
            kind: lsp_types::MarkupKind::Markdown,
            value: text,
        },
        HoverData::Package(description) => lsp_types::MarkupContent {
            kind: lsp_types::MarkupKind::PlainText,
            value: description.into(),
        },
        HoverData::Command(command) => inline_image(command, client_flags)?,
        HoverData::EntryType(type_) => lsp_types::MarkupContent {
            kind: lsp_types::MarkupKind::Markdown,
            value: type_.documentation?.into(),
        },
        HoverData::FieldType(type_) => lsp_types::MarkupContent {
            kind: lsp_types::MarkupKind::Markdown,
            value: type_.documentation.into(),
        },
        HoverData::Label(label) => lsp_types::MarkupContent {
            kind: lsp_types::MarkupKind::PlainText,
            value: label.reference(),
        },
        HoverData::StringRef(text) => lsp_types::MarkupContent {
            kind: lsp_types::MarkupKind::PlainText,
            value: text,
        },
    };

    Some(lsp_types::Hover {
        contents: lsp_types::HoverContents::Markup(contents),
        range: line_index.line_col_lsp_range(hover.range),
    })
}
