use base_db::data::BibtexEntryType;
use syntax::bibtex;

use crate::{Hover, HoverData, HoverParams};

pub(super) fn find_hover<'a>(params: &HoverParams<'a>) -> Option<Hover<'a>> {
    let data = params.feature.document.data.as_bib()?;
    let root = data.root_node();
    let name = root
        .token_at_offset(params.offset)
        .find(|x| x.kind() == bibtex::TYPE)?;

    let entry_type = BibtexEntryType::find(name.text())?;
    Some(Hover {
        range: name.text_range(),
        data: HoverData::EntryType(entry_type),
    })
}
