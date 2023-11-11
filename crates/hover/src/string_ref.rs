use bibtex_utils::field::text::TextFieldData;
use rowan::ast::AstNode;
use syntax::bibtex::{self, HasName, HasValue};

use crate::{Hover, HoverData, HoverParams};

pub(super) fn find_hover<'a>(params: &HoverParams<'a>) -> Option<Hover<'a>> {
    let data = params.feature.document.data.as_bib()?;
    let root = bibtex::Root::cast(data.root_node())?;
    let name = root
        .syntax()
        .token_at_offset(params.offset)
        .find(|token| token.kind() == bibtex::NAME)
        .filter(|token| {
            let parent = token.parent().unwrap();
            bibtex::Value::can_cast(parent.kind()) || bibtex::StringDef::can_cast(parent.kind())
        })?;

    for string in root.strings() {
        if !string
            .name_token()
            .map_or(false, |token| token.text() == name.text())
        {
            continue;
        }

        let value = TextFieldData::parse(&string.value()?)?.text;
        return Some(Hover {
            range: name.text_range(),
            data: HoverData::StringRef(value),
        });
    }

    None
}
