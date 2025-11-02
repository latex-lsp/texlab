use base_db::{
    semantics::{
        Span,
        tex::{Label, LabelKind},
    },
    util::queries::Object,
};
use rowan::ast::AstNode;
use rustc_hash::FxHashSet;
use syntax::latex;

use crate::{
    CompletionParams,
    util::{CompletionBuilder, find_curly_group_word},
};

pub fn complete_label_definitions<'a>(
    params: &'a CompletionParams<'a>,
    builder: &mut CompletionBuilder<'a>,
) -> Option<()> {
    let cursor = find_definition(params)?;

    let label_defs: FxHashSet<&str> = Label::find_all(&params.feature.project)
        .filter(|(_, label)| label.kind == LabelKind::Definition)
        .map(|(_, label)| label.name_text())
        .collect();

    let label_refs: FxHashSet<&str> = Label::find_all(&params.feature.project)
        .filter(|(_, label)| label.kind == LabelKind::Reference)
        .map(|(_, label)| label.name_text())
        .collect();

    for label in label_refs.difference(&label_defs) {
        let Some(score) = builder.matcher.score(label, &cursor.text) else {
            continue;
        };

        let data = crate::LabelData {
            name: label,
            header: None,
            footer: None,
            object: None,
            keywords: label.to_string(),
        };

        builder.items.push(crate::CompletionItem::new_simple(
            score,
            cursor.range,
            crate::CompletionItemData::Label(data),
        ));
    }

    Some(())
}

fn find_definition(params: &CompletionParams) -> Option<Span> {
    let (cursor, group) = find_curly_group_word(params)?;
    latex::LabelDefinition::cast(group.syntax().parent()?)?;
    Some(cursor)
}
