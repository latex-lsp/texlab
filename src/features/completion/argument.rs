use rowan::{ast::AstNode, TextRange};

use crate::{component_db::COMPONENT_DATABASE, syntax::latex, util::cursor::CursorContext};

use super::types::{InternalCompletionItem, InternalCompletionItemData};

pub fn complete_arguments<'db>(
    context: &'db CursorContext,
    items: &mut Vec<InternalCompletionItem<'db>>,
) -> Option<()> {
    let token = context.cursor.as_tex()?;

    let range = if token.kind() == latex::WORD {
        token.text_range()
    } else {
        TextRange::empty(context.offset)
    };

    let group = latex::CurlyGroup::cast(token.parent()?)
        .or_else(|| {
            token
                .parent()
                .and_then(|node| node.parent())
                .and_then(latex::CurlyGroup::cast)
        })
        .filter(|group| context.is_inside_latex_curly(group))?;

    let command = latex::GenericCommand::cast(group.syntax().parent()?)?;

    let index = command
        .syntax()
        .children()
        .filter_map(latex::CurlyGroup::cast)
        .position(|g| g.syntax().text_range() == group.syntax().text_range())?;

    let command_name = command.name()?;
    let command_name = &command_name.text()[1..];

    for component in COMPONENT_DATABASE.linked_components(context.db, context.document) {
        for component_command in component
            .commands
            .iter()
            .filter(|command| command.name == command_name)
        {
            for (_, param) in component_command
                .parameters
                .iter()
                .enumerate()
                .filter(|(i, _)| *i == index)
            {
                for arg in &param.0 {
                    let item = InternalCompletionItem::new(
                        range,
                        InternalCompletionItemData::Argument {
                            name: &arg.name,
                            image: arg.image.as_deref(),
                        },
                    );
                    items.push(item);
                }
            }
        }
    }

    Some(())
}
