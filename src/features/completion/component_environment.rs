use crate::{component_db::COMPONENT_DATABASE, util::cursor::CursorContext};

use super::types::{InternalCompletionItem, InternalCompletionItemData};

pub fn complete_component_environments<'db>(
    context: &'db CursorContext,
    items: &mut Vec<InternalCompletionItem<'db>>,
) -> Option<()> {
    let (_, range) = context.find_environment_name()?;

    for component in COMPONENT_DATABASE.linked_components(context.db, context.document) {
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
