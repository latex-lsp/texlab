use lsp_types::CompletionParams;

use crate::features::cursor::CursorContext;

use super::types::{InternalCompletionItem, InternalCompletionItemData};

pub fn complete_user_commands<'a>(
    context: &'a CursorContext<CompletionParams>,
    items: &mut Vec<InternalCompletionItem<'a>>,
) -> Option<()> {
    let range = context.cursor.command_range(context.offset)?;
    let token = context.cursor.as_latex()?;

    for document in context.request.workspace.documents_by_uri.values() {
        if let Some(data) = document.data.as_latex() {
            for name in data
                .extras
                .command_names
                .iter()
                .filter(|name| name.as_str() != token.text())
                .cloned()
            {
                items.push(InternalCompletionItem::new(
                    range,
                    InternalCompletionItemData::UserCommand { name },
                ));
            }
        }
    }

    Some(())
}
