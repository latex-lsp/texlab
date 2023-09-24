use base_db::semantics::Span;
use rowan::{ast::AstNode, TokenAtOffset};
use syntax::bibtex::{self, HasName};

use crate::{
    util::CompletionBuilder, CompletionItem, CompletionItemData, CompletionParams, FieldTypeData,
};

pub fn complete_fields<'a>(
    params: &'a CompletionParams<'a>,
    builder: &mut CompletionBuilder<'a>,
) -> Option<()> {
    let cursor = find_field(params)?;

    for field in base_db::data::BIBTEX_FIELD_TYPES {
        if let Some(score) = builder.matcher.score(field.name, &cursor.text) {
            let data = CompletionItemData::Field(FieldTypeData(*field));
            builder
                .items
                .push(CompletionItem::new_simple(score, cursor.range, data));
        }
    }

    Some(())
}

fn find_field(params: &CompletionParams) -> Option<Span> {
    let token = select_token(params)?;
    if token.kind() == bibtex::TYPE {
        return None;
    }

    let parent = token.parent()?;
    if let Some(entry) = bibtex::Entry::cast(parent.clone()) {
        if entry.name_token()?.text_range() == token.text_range() {
            return None;
        }
    } else {
        bibtex::Field::cast(parent)?;
    }

    Some(if token.kind() == bibtex::NAME {
        Span::from(&token)
    } else {
        Span::empty(params.offset)
    })
}

fn select_token(params: &CompletionParams) -> Option<bibtex::SyntaxToken> {
    let data = params.feature.document.data.as_bib()?;
    Some(match data.root_node().token_at_offset(params.offset) {
        TokenAtOffset::Between(_, r) if r.kind() == bibtex::TYPE => r,
        TokenAtOffset::Between(l, _) if l.kind() == bibtex::TYPE => l,
        TokenAtOffset::Between(l, _) if l.kind() == bibtex::COMMAND_NAME => l,
        TokenAtOffset::Between(l, _) if l.kind() == bibtex::ACCENT_NAME => l,
        TokenAtOffset::Between(_, r) if r.kind() == bibtex::WORD => r,
        TokenAtOffset::Between(_, r) if r.kind() == bibtex::NAME => r,
        TokenAtOffset::Between(l, _) if l.kind() == bibtex::WORD => l,
        TokenAtOffset::Between(l, _) if l.kind() == bibtex::NAME => l,
        TokenAtOffset::Between(_, r) if r.kind() == bibtex::COMMAND_NAME => r,
        TokenAtOffset::Between(_, r) if r.kind() == bibtex::ACCENT_NAME => r,
        TokenAtOffset::Between(_, r) => r,
        TokenAtOffset::Single(t) => t,
        TokenAtOffset::None => return None,
    })
}
