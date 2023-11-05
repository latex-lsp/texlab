use base_db::Workspace;
use hover::{HoverData, HoverParams};

use crate::util::line_index_ext::LineIndexExt;

pub fn find(
    workspace: &Workspace,
    uri: &lsp_types::Url,
    position: lsp_types::Position,
) -> Option<lsp_types::Hover> {
    let document = workspace.lookup(uri)?;
    let offset = document.line_index.offset_lsp(position)?;
    let params = HoverParams::new(workspace, document, offset);
    let hover = ::hover::find(&params)?;

    let contents = match hover.data {
        HoverData::Citation(text) => lsp_types::MarkupContent {
            kind: lsp_types::MarkupKind::Markdown,
            value: text,
        },
        HoverData::Package(description) => lsp_types::MarkupContent {
            kind: lsp_types::MarkupKind::PlainText,
            value: description.into(),
        },
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
        range: document.line_index.line_col_lsp_range(hover.range),
    })
}
