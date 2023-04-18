use base_db::data::BibtexEntryType;
use lsp_types::MarkupKind;
use syntax::bibtex;

use crate::util::cursor::CursorContext;

use super::HoverResult;

pub(super) fn find_hover(context: &CursorContext) -> Option<HoverResult> {
    let name = context
        .cursor
        .as_bib()
        .filter(|token| token.kind() == bibtex::TYPE)?;

    let documentation = BibtexEntryType::find(&name.text()[1..]).and_then(|ty| ty.documentation)?;
    Some(HoverResult {
        range: name.text_range(),
        value: String::from(documentation),
        value_kind: MarkupKind::Markdown,
    })
}
