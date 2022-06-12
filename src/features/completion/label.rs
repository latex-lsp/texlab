use lsp_types::CompletionParams;
use rowan::{ast::AstNode, TextRange};

use crate::{
    features::{cursor::CursorContext, lsp_kinds::Structure},
    render_label,
    syntax::latex,
    LabelledObject,
};

use super::types::{InternalCompletionItem, InternalCompletionItemData};

pub fn complete_labels<'a>(
    context: &'a CursorContext<CompletionParams>,
    items: &mut Vec<InternalCompletionItem<'a>>,
) -> Option<()> {
    let (range, is_math) = find_reference(context).or_else(|| find_reference_range(context))?;

    for document in context.request.workspace.documents_by_uri.values() {
        if let Some(data) = document.data.as_latex() {
            for label in latex::SyntaxNode::new_root(data.green.clone())
                .descendants()
                .filter_map(latex::LabelDefinition::cast)
            {
                if let Some(name) = label
                    .name()
                    .and_then(|name| name.key())
                    .map(|name| name.to_string())
                {
                    match render_label(&context.request.workspace, &name, Some(label)) {
                        Some(rendered_label) => {
                            let kind = match &rendered_label.object {
                                LabelledObject::Section { .. } => Structure::Section,
                                LabelledObject::Float { .. } => Structure::Float,
                                LabelledObject::Theorem { .. } => Structure::Theorem,
                                LabelledObject::Equation => Structure::Equation,
                                LabelledObject::EnumItem => Structure::Item,
                            };

                            if is_math && kind != Structure::Equation {
                                continue;
                            }

                            let header = rendered_label.detail();
                            let footer = match &rendered_label.object {
                                LabelledObject::Float { caption, .. } => Some(caption.clone()),
                                _ => None,
                            };

                            let text = format!("{} {}", name, rendered_label.reference());

                            let item = InternalCompletionItem::new(
                                range,
                                InternalCompletionItemData::Label {
                                    name,
                                    kind,
                                    header,
                                    footer,
                                    text,
                                },
                            );
                            items.push(item);
                        }
                        None => {
                            let kind = Structure::Label;
                            let header = None;
                            let footer = None;
                            let text = name.to_string();
                            let item = InternalCompletionItem::new(
                                range,
                                InternalCompletionItemData::Label {
                                    name,
                                    kind,
                                    header,
                                    footer,
                                    text,
                                },
                            );
                            items.push(item);
                        }
                    }
                }
            }
        }
    }

    Some(())
}

fn find_reference(context: &CursorContext<CompletionParams>) -> Option<(TextRange, bool)> {
    let (_, range, group) = context.find_curly_group_word_list()?;
    let reference = latex::LabelReference::cast(group.syntax().parent()?)?;
    let is_math = reference.command()?.text() == "\\eqref";
    Some((range, is_math))
}

fn find_reference_range(context: &CursorContext<CompletionParams>) -> Option<(TextRange, bool)> {
    let (_, range, group) = context.find_curly_group_word()?;
    latex::LabelReferenceRange::cast(group.syntax().parent()?)?;
    Some((range, false))
}
