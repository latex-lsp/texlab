use rowan::ast::AstNode;

use crate::{
    db::Workspace,
    syntax::bibtex::{self, HasName, HasType},
    util::lang_data::{BibtexEntryTypeCategory, LANGUAGE_DATA},
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
        let entry = Workspace::get(db)
            .related(db, context.document)
            .iter()
            .filter_map(|document| document.parse(db).as_bib())
            .flat_map(|data| data.root(db).children())
            .filter_map(bibtex::Entry::cast)
            .find(|entry| {
                entry
                    .name_token()
                    .map_or(false, |name| name.text() == &citation.key)
            });

        let modifiers = if entry.is_some() {
            TokenModifiers::NONE
        } else {
            TokenModifiers::UNDEFINED
        };

        let kind = match entry
            .and_then(|entry| entry.type_token())
            .and_then(|token| LANGUAGE_DATA.find_entry_type(&token.text()[1..]))
            .map(|doc| doc.category)
        {
            Some(BibtexEntryTypeCategory::String) => unreachable!(),
            Some(BibtexEntryTypeCategory::Misc) => TokenKind::GenericCitation,
            Some(BibtexEntryTypeCategory::Article) => TokenKind::ArticleCitation,
            Some(BibtexEntryTypeCategory::Book) => TokenKind::BookCitation,
            Some(BibtexEntryTypeCategory::Part) => TokenKind::PartCitation,
            Some(BibtexEntryTypeCategory::Thesis) => TokenKind::ThesisCitation,
            Some(BibtexEntryTypeCategory::Collection) => TokenKind::CollectionCitation,
            None => TokenKind::GenericCitation,
        };

        builder.push(Token {
            range: citation.range,
            kind,
            modifiers,
        });
    }

    Some(())
}
