use rowan::ast::AstNode;

use crate::{
    db::{Document, Workspace},
    syntax::bibtex::{self, HasName},
    Db,
};

use super::{Context, Token, TokenBuilder, TokenKind, TokenModifiers};

pub(super) fn find(context: Context, builder: &mut TokenBuilder) -> Option<()> {
    let db = context.db;
    let analysis = context.document.parse(db).as_tex()?.analyze(db);
    for citation in analysis
        .citations(db)
        .iter()
        .filter(|citation| context.viewport.intersect(citation.range).is_some())
    {
        let modifiers = if !is_entry_defined(db, context.document, &citation.key) {
            TokenModifiers::UNDEFINED
        } else {
            TokenModifiers::NONE
        };

        builder.push(Token {
            range: citation.range,
            kind: TokenKind::Citation,
            modifiers,
        });
    }

    Some(())
}

fn is_entry_defined(db: &dyn Db, child: Document, key: &str) -> bool {
    Workspace::get(db)
        .related(db, child)
        .iter()
        .filter_map(|document| document.parse(db).as_bib())
        .flat_map(|data| data.root(db).children())
        .filter_map(bibtex::Entry::cast)
        .filter_map(|entry| entry.name_token())
        .any(|token| token.text() == key)
}
