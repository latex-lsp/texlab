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

fn trim_prefix<'a>(prefix: Option<&'a str>, text: &'a str) -> &'a str {
    prefix
        .and_then(|pref| text.strip_prefix(pref))
        .unwrap_or(text)
}

pub fn complete_label_references<'a>(
    params: &'a CompletionParams<'a>,
    builder: &mut CompletionBuilder<'a>,
) -> Option<()> {
    let FindResult {
        cursor,
        is_math,
        command,
    } = find_reference(params).or_else(|| find_reference_range(params))?;
    let ref_pref = params
        .feature
        .workspace
        .config()
        .syntax
        .label_reference_prefixes
        .get(&command)
        .map(|x| x.as_str());

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
            if ref_pref.map_or(false, |pref| !label.name.text.starts_with(pref)) {
                continue;
            }
            let labeltext = trim_prefix(ref_pref, &label.name.text);
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

                    let keywords = format!("{} {}", labeltext, rendered_label.reference());

                    if let Some(score) = builder.matcher.score(&keywords, &cursor.text) {
                        let name = trim_prefix(ref_pref, &label.name.text);
                        let data = CompletionItemData::Label(crate::LabelData {
                            name,
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
                        let name = trim_prefix(ref_pref, &label.name.text);
                        let data = CompletionItemData::Label(crate::LabelData {
                            name,
                            header: None,
                            footer: None,
                            object: None,
                            keywords: labeltext.to_string(),
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
    command: String,
}

fn find_reference(params: &CompletionParams) -> Option<FindResult> {
    let (cursor, group) = find_curly_group_word_list(params)?;
    let reference = latex::LabelReference::cast(group.syntax().parent()?)?;
    let is_math = reference.command()?.text() == "\\eqref";
    Some(FindResult {
        cursor,
        is_math,
        command: reference.command()?.text()[1..].to_string(),
    })
}

fn find_reference_range(params: &CompletionParams) -> Option<FindResult> {
    let (cursor, group) = find_curly_group_word(params)?;
    let refrange = latex::LabelReferenceRange::cast(group.syntax().parent()?)?;
    Some(FindResult {
        cursor,
        is_math: false,
        command: refrange.command()?.text()[1..].to_string(),
    })
}
