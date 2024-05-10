use base_db::semantics::Span;
use rowan::{ast::AstNode, TokenAtOffset};
use syntax::latex;

use crate::{
    util::{included_packages, is_inside_latex_curly, CompletionBuilder},
    ArgumentData, CompletionItem, CompletionItemData, CompletionParams,
};

pub fn complete_arguments<'a>(
    params: &'a CompletionParams<'a>,
    builder: &mut CompletionBuilder<'a>,
) -> Option<()> {
    let (cursor, group) = find_argument(params)?;

    let command = latex::GenericCommand::cast(group.syntax().parent()?)?;

    let index = command
        .syntax()
        .children()
        .filter_map(latex::CurlyGroup::cast)
        .position(|g| g.syntax().text_range() == group.syntax().text_range())?;

    let command_name = command.name()?;
    let command_name = &command_name.text()[1..];

    for package in included_packages(&params.feature) {
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
                    if let Some(score) = builder.matcher.score(arg.name, &cursor.text) {
                        let data = CompletionItemData::Argument(ArgumentData(arg));
                        builder
                            .items
                            .push(CompletionItem::new_simple(score, cursor.range, data));
                    }
                }
            }
        }
    }

    Some(())
}

fn find_argument(params: &CompletionParams) -> Option<(Span, latex::CurlyGroup)> {
    let data = params.feature.document.data.as_tex()?;
    let tokens = data.root_node().token_at_offset(params.offset);

    let (span, token) = match tokens.clone().find(|token| token.kind() == latex::WORD) {
        Some(token) => (Span::from(&token), token),
        None if matches!(tokens, TokenAtOffset::Between(_, _)) => {
            (Span::empty(params.offset), tokens.left_biased()?)
        }
        None => return None,
    };

    let group = latex::CurlyGroup::cast(token.parent()?)
        .or_else(|| {
            token
                .parent()
                .and_then(|node| node.parent())
                .and_then(latex::CurlyGroup::cast)
        })
        .filter(|group| is_inside_latex_curly(group, params.offset))?;

    Some((span, group))
}
