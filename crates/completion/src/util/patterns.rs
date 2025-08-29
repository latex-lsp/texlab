use base_db::semantics::Span;
use rowan::{ast::AstNode, TextRange, TextSize};
use syntax::latex;

use crate::CompletionParams;

pub fn find_curly_group_word(params: &CompletionParams) -> Option<(Span, latex::CurlyGroupWord)> {
    let offset = params.offset;
    let data = params.feature.document.data.as_tex()?;
    let root = data.root_node();
    let tokens = root.token_at_offset(offset);
    let token = tokens
        .clone()
        .find(|token| token.kind() == latex::WORD)
        .or_else(|| tokens.left_biased())?;

    let key = latex::Key::cast(token.parent()?);

    let group = key
        .as_ref()
        .and_then(|key| key.syntax().parent())
        .unwrap_or(token.parent()?);

    let group =
        latex::CurlyGroupWord::cast(group).filter(|group| is_inside_latex_curly(group, offset))?;

    let span = key.map_or_else(|| Span::empty(offset), |key| Span::from(&key));
    Some((span, group))
}

pub fn find_curly_group_word_list(
    params: &CompletionParams,
) -> Option<(Span, latex::CurlyGroupWordList)> {
    let offset = params.offset;
    let data = params.feature.document.data.as_tex()?;
    let root = data.root_node();
    let tokens = root.token_at_offset(offset);
    let token = tokens
        .clone()
        .find(|token| token.kind() == latex::WORD)
        .or_else(|| tokens.left_biased())?;

    let key = latex::Key::cast(token.parent()?);

    let group = key
        .as_ref()
        .and_then(|key| key.syntax().parent())
        .unwrap_or(token.parent()?);

    let group = latex::CurlyGroupWordList::cast(group)
        .filter(|group| is_inside_latex_curly(group, offset))?;

    let span = key.map_or_else(
        || Span::empty(offset),
        |key| {
            let range = if group
                .syntax()
                .last_token()
                .is_some_and(|tok| tok.kind() != latex::R_CURLY)
            {
                TextRange::new(latex::small_range(&key).start(), token.text_range().end())
            } else {
                latex::small_range(&key)
            };

            Span::new(token.text().into(), range)
        },
    );

    Some((span, group))
}

pub fn is_inside_latex_curly(group: &impl latex::HasCurly, offset: TextSize) -> bool {
    latex::small_range(group).contains(offset) || group.right_curly().is_none()
}
