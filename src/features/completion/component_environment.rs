use lsp_types::CompletionParams;

use crate::{component_db::COMPONENT_DATABASE, features::cursor::CursorContext};

use super::types::{InternalCompletionItem, InternalCompletionItemData};

pub fn complete_component_environments<'a>(
    context: &'a CursorContext<CompletionParams>,
    items: &mut Vec<InternalCompletionItem<'a>>,
) -> Option<()> {
    let (_, range) = context.find_environment_name()?;

    for component in COMPONENT_DATABASE.linked_components(&context.request.workspace) {
        for name in &component.environments {
            items.push(InternalCompletionItem::new(
                range,
                InternalCompletionItemData::ComponentEnvironment {
                    name,
                    file_names: &component.file_names,
                },
            ));
        }
    }

    Some(())
}
