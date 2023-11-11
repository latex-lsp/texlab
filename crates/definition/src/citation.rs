use base_db::{
    semantics::bib,
    util::queries::{self, Object},
};

use crate::DefinitionContext;

use super::DefinitionResult;

pub(super) fn goto_definition(context: &mut DefinitionContext) -> Option<()> {
    let feature = &context.params.feature;
    let data = feature.document.data.as_tex()?;

    let citation = queries::object_at_cursor(
        &data.semantics.citations,
        context.params.offset,
        queries::SearchMode::Full,
    )?;

    let name = citation.object.name_text();
    for (document, entry) in queries::objects_with_name::<bib::Entry>(&feature.project, name) {
        context.results.insert(DefinitionResult {
            origin_selection_range: citation.object.name_range(),
            target: document,
            target_range: entry.full_range,
            target_selection_range: entry.name.range,
        });
    }

    Some(())
}
