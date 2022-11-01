use crate::util::cursor::CursorContext;

use super::types::{InternalCompletionItem, InternalCompletionItemData};

pub fn complete_theorem_environments<'db>(
    context: &'db CursorContext,
    items: &mut Vec<InternalCompletionItem<'db>>,
) -> Option<()> {
    let (_, range) = context.find_environment_name()?;

    let db = context.db;
    for document in context
        .workspace
        .related(db, context.distro, context.document)
    {
        if let Some(data) = document.parse(db).as_tex() {
            for environment in data.analyze(db).theorem_environments(db) {
                items.push(InternalCompletionItem::new(
                    range,
                    InternalCompletionItemData::UserEnvironment {
                        name: environment.name(db).text(db).clone(),
                    },
                ));
            }
        }
    }

    Some(())
}
