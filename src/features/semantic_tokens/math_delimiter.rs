use crate::syntax::latex;

use super::{Context, Token, TokenBuilder, TokenKind, TokenModifiers};

pub(super) fn find(context: Context, builder: &mut TokenBuilder) -> Option<()> {
    let db = context.db;
    let root = context.document.parse(db).as_tex()?.root(db);

    for token in root
        .covering_element(context.viewport)
        .as_node()?
        .descendants_with_tokens()
        .filter_map(|elem| elem.into_token())
        .filter(|token| token.kind() == latex::DOLLAR)
    {
        builder.push(Token {
            range: token.text_range(),
            kind: TokenKind::MathDelimiter,
            modifiers: TokenModifiers::DEPRECATED,
        });
    }

    Some(())
}
