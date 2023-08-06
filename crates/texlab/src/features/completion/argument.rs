use rowan::{ast::AstNode, TextRange};
use syntax::latex;

use crate::util::cursor::CursorContext;

use super::builder::CompletionBuilder;

pub fn complete<'a>(context: &'a CursorContext, builder: &mut CompletionBuilder<'a>) -> Option<()> {
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

    for package in context.included_packages() {
        for package_command in package
            .commands
            .iter()
            .filter(|command| command.name == command_name)
        {
            for (_, param) in package_command
                .parameters
                .iter()
                .enumerate()
                .filter(|(i, _)| *i == index)
            {
                for arg in &param.0 {
                    builder.generic_argument(range, arg.name, arg.image);
                }
            }
        }
    }

    Some(())
}
