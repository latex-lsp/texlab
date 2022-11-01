use crate::util::cursor::CursorContext;

use super::types::{InternalCompletionItem, InternalCompletionItemData};

pub fn complete_user_environments<'db>(
    context: &'db CursorContext,
    items: &mut Vec<InternalCompletionItem<'db>>,
) -> Option<()> {
    let (name, range) = context.find_environment_name()?;

    for document in context.related() {
        if let Some(data) = document.parse(context.db).as_tex() {
            for name in data
                .analyze(context.db)
                .environment_names(context.db)
                .iter()
                .filter(|n| n.as_str() != name)
                .cloned()
            {
                items.push(InternalCompletionItem::new(
                    range,
                    InternalCompletionItemData::UserEnvironment { name },
                ));
            }
        }
    }

    Some(())
}
