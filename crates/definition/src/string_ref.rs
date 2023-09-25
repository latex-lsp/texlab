use rowan::ast::AstNode;
use syntax::bibtex;

use crate::DefinitionContext;

use super::DefinitionResult;

pub(super) fn goto_definition(context: &mut DefinitionContext) -> Option<()> {
    let data = context.params.document.data.as_bib()?;
    let root = data.root_node();
    let name = root
        .token_at_offset(context.params.offset)
        .filter(|token| token.kind() == bibtex::NAME)
        .find(|token| {
            let parent = token.parent().unwrap();
            bibtex::Value::can_cast(parent.kind()) || bibtex::StringDef::can_cast(parent.kind())
        })?;

    let origin_selection_range = name.text_range();

    let strings = &data.semantics.strings;
    for string in strings
        .iter()
        .filter(|string| string.name.text == name.text())
    {
        context.results.insert(DefinitionResult {
            origin_selection_range,
            target: context.params.document,
            target_range: string.full_range,
            target_selection_range: string.name.range,
        });
    }

    Some(())
}
