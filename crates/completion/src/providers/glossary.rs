use base_db::semantics::Span;
use rowan::ast::AstNode;
use syntax::latex;

use crate::{
    util::{find_curly_group_word, CompletionBuilder, ProviderContext},
    CompletionItem, CompletionItemData, CompletionParams, GlossaryEntryData,
};

pub fn complete_acronyms<'a>(
    params: &'a CompletionParams<'a>,
    builder: &mut CompletionBuilder<'a>,
) -> Option<()> {
    let cursor = find_acronym(params)?;
    let mut proc = Processor(ProviderContext {
        builder,
        params,
        cursor,
    });

    proc.add_acronyms();
    Some(())
}

pub fn complete_glossaries<'a>(
    params: &'a CompletionParams<'a>,
    builder: &mut CompletionBuilder<'a>,
) -> Option<()> {
    let cursor = find_glossary(params)?;
    let mut proc = Processor(ProviderContext {
        builder,
        params,
        cursor,
    });

    proc.add_acronyms();
    proc.add_glossaries();
    Some(())
}

struct Processor<'a, 'b>(ProviderContext<'a, 'b>);

impl<'a, 'b> Processor<'a, 'b> {
    pub fn add_acronyms(&mut self) {
        self.add_generic(|node| latex::AcronymDefinition::cast(node)?.name());
    }

    pub fn add_glossaries(&mut self) {
        self.add_generic(|node| latex::GlossaryEntryDefinition::cast(node)?.name());
    }

    fn add_generic<F>(&mut self, extract: F)
    where
        F: Fn(latex::SyntaxNode) -> Option<latex::CurlyGroupWord>,
    {
        let documents = self.0.params.feature.project.documents.iter();
        for data in documents.filter_map(|document| document.data.as_tex()) {
            for name in data
                .root_node()
                .descendants()
                .filter_map(|node| extract(node))
                .filter_map(|name| name.key())
                .map(|name| name.to_string())
            {
                if let Some(score) = self.0.builder.matcher.score(&name, &self.0.cursor.text) {
                    let data = CompletionItemData::GlossaryEntry(GlossaryEntryData { name });
                    self.0.builder.items.push(CompletionItem::new_simple(
                        score,
                        self.0.cursor.range,
                        data,
                    ));
                }
            }
        }
    }
}

fn find_acronym(params: &CompletionParams) -> Option<Span> {
    let (cursor, group) = find_curly_group_word(params)?;
    latex::AcronymReference::cast(group.syntax().parent()?)?;
    Some(cursor)
}

fn find_glossary(params: &CompletionParams) -> Option<Span> {
    let (cursor, group) = find_curly_group_word(params)?;
    latex::GlossaryEntryReference::cast(group.syntax().parent()?)?;
    Some(cursor)
}
