use rowan::TextRange;

use crate::{db::Document, syntax::latex, Db};

use super::{Token, TokenBuilder, TokenKind, TokenModifiers};

pub fn find(
    db: &dyn Db,
    document: Document,
    viewport: TextRange,
    builder: &mut TokenBuilder,
) -> Option<()> {
    let root = document.parse(db).as_tex()?.root(db);

    for token in root
        .covering_element(viewport)
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
