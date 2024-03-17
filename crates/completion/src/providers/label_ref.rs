use base_db::{
    semantics::{tex::LabelKind, Span},
    util::{render_label, RenderedObject},
    DocumentData,
};
use rowan::ast::AstNode;
use syntax::latex;

use crate::{
    util::{find_curly_group_word, find_curly_group_word_list, CompletionBuilder},
    CompletionItem, CompletionItemData, CompletionParams,
};

pub fn complete_label_references<'a>(
    params: &'a CompletionParams<'a>,
    builder: &mut CompletionBuilder<'a>,
) -> Option<()> {
    let FindResult { cursor, is_math } =
        find_reference(params).or_else(|| find_reference_range(params))?;

    for document in &params.feature.project.documents {
        let DocumentData::Tex(data) = &document.data else {
            continue;
        };

        for label in data
            .semantics
            .labels
            .iter()
            .filter(|label| label.kind == LabelKind::Definition)
        {
            match render_label(params.feature.workspace, &params.feature.project, label) {
                Some(rendered_label) => {
                    if is_math && !matches!(rendered_label.object, RenderedObject::Equation) {
                        continue;
                    }

                    let header = rendered_label.detail();
                    let footer = match &rendered_label.object {
                        RenderedObject::Float { caption, .. } => Some(*caption),
                        _ => None,
                    };

                    let keywords = format!("{} {}", label.name.text, rendered_label.reference());

                    if let Some(score) = builder.matcher.score(&keywords, &cursor.text) {
                        let data = CompletionItemData::Label(crate::LabelData {
                            name: &label.name.text,
                            header,
                            footer,
                            object: Some(rendered_label.object),
                            keywords,
                        });

                        builder
                            .items
                            .push(CompletionItem::new_simple(score, cursor.range, data));
                    }
                }
                None => {
                    if let Some(score) = builder.matcher.score(&label.name.text, &cursor.text) {
                        let data = CompletionItemData::Label(crate::LabelData {
                            name: &label.name.text,
                            header: None,
                            footer: None,
                            object: None,
                            keywords: label.name.text.clone(),
                        });

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

struct FindResult {
    cursor: Span,
    is_math: bool,
}

fn find_reference(params: &CompletionParams) -> Option<FindResult> {
    let (cursor, group) = find_curly_group_word_list(params)?;
    let reference = latex::LabelReference::cast(group.syntax().parent()?)?;
    let is_math = reference.command()?.text() == "\\eqref";
    Some(FindResult { cursor, is_math })
}

fn find_reference_range(params: &CompletionParams) -> Option<FindResult> {
    let (cursor, group) = find_curly_group_word(params)?;
    latex::LabelReferenceRange::cast(group.syntax().parent()?)?;
    Some(FindResult {
        cursor,
        is_math: false,
    })
}
