use base_db::{MatchingAlgo, semantics::Span};
use rayon::prelude::{IntoParallelRefIterator, ParallelExtend, ParallelIterator};
use rowan::ast::AstNode;
use syntax::latex;

use crate::{
    CitationData, CompletionItem, CompletionItemData, CompletionParams,
    util::{CompletionBuilder, find_curly_group_word_list},
};

pub fn complete_citations<'a>(
    params: &'a CompletionParams<'a>,
    builder: &mut CompletionBuilder<'a>,
) -> Option<()> {
    let cursor = find_citation(params)?;
    let prefix_matcher = matches!(
        params.feature.workspace.config().completion.matcher,
        MatchingAlgo::Prefix | MatchingAlgo::PrefixIgnoreCase
    );

    for document in &params.feature.project.documents {
        if let Some(data) = document.data.as_bib() {
            let items = data.semantics.entries.par_iter().filter_map(|entry| {
                let score = builder
                    .matcher
                    .score(&entry.name.text, &cursor.text)
                    .or_else(|| {
                        let metadata = entry
                            .keywords
                            .strip_prefix(entry.name.text.as_str())
                            .unwrap_or(&entry.keywords);
                        if prefix_matcher {
                            metadata
                                .split_whitespace()
                                .filter_map(|word| builder.matcher.score(word, &cursor.text))
                                .max()
                        } else {
                            builder.matcher.score(metadata, &cursor.text)
                        }
                    })?;
                let data = CompletionItemData::Citation(CitationData { document, entry });
                Some(CompletionItem::new_simple(score, cursor.range, data))
            });

            builder.items.par_extend(items);
        }
    }

    Some(())
}

fn find_citation(params: &CompletionParams) -> Option<Span> {
    find_citation_command(params).or_else(|| find_citation_acronym(params))
}

fn find_citation_command(params: &CompletionParams) -> Option<Span> {
    let (span, group) = find_curly_group_word_list(params)?;
    latex::Citation::cast(group.syntax().parent()?)?;
    Some(span)
}

fn find_citation_acronym(params: &CompletionParams) -> Option<Span> {
    let offset = params.offset;
    let data = params.feature.document.data.as_tex()?;
    let root = data.root_node();
    let tokens = root.token_at_offset(offset);
    let token = tokens
        .clone()
        .find(|token| token.kind() == latex::WORD)
        .or_else(|| tokens.left_biased())?;

    let span = if token.kind() == latex::WORD {
        let name = latex::Text::cast(token.parent()?)?;
        Span::new(token.text().into(), latex::small_range(&name))
    } else {
        Span::empty(offset)
    };

    let pair = token
        .parent_ancestors()
        .find_map(latex::KeyValuePair::cast)?;

    if pair.key()?.to_string() == "cite" {
        let body = pair.syntax().parent()?;
        let group = body.parent()?;
        latex::AcronymDeclaration::cast(group.parent()?)?;
        Some(span)
    } else {
        None
    }
}
