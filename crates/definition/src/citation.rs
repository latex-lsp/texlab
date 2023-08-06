use base_db::{
    semantics::bib,
    util::queries::{self, Object},
};

use crate::DefinitionContext;

use super::DefinitionResult;

pub(super) fn goto_definition(context: &mut DefinitionContext) -> Option<()> {
    let data = context.params.document.data.as_tex()?;

    let citation = queries::object_at_cursor(
        &data.semantics.citations,
        context.params.offset,
        queries::SearchMode::Full,
    )?;

    let name = citation.object.name_text();
    for (document, entry) in queries::objects_with_name::<bib::Entry>(&context.project, name) {
        context.results.push(DefinitionResult {
            origin_selection_range: citation.object.name_range(),
            target: document,
            target_range: entry.full_range,
            target_selection_range: entry.name.range,
        });
    }

    Some(())
}
