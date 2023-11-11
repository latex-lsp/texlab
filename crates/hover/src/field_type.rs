use base_db::data::BibtexFieldType;
use rowan::ast::AstNode;
use syntax::bibtex;

use crate::{Hover, HoverData, HoverParams};

pub(super) fn find_hover<'a>(params: &HoverParams<'a>) -> Option<Hover<'a>> {
    let data = params.feature.document.data.as_bib()?;
    let root = data.root_node();
    let name = root
        .token_at_offset(params.offset)
        .find(|token| token.kind() == bibtex::NAME)?;

    bibtex::Field::cast(name.parent()?)?;

    let field_type = BibtexFieldType::find(name.text())?;
    Some(Hover {
        range: name.text_range(),
        data: HoverData::FieldType(field_type),
    })
}
