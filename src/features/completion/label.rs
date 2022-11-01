use rowan::{ast::AstNode, TextRange};

use crate::{
    features::lsp_kinds::Structure,
    syntax::latex,
    util::{self, cursor::CursorContext, label::LabeledObject},
};

use super::types::{InternalCompletionItem, InternalCompletionItemData};

pub fn complete_labels<'db>(
    context: &'db CursorContext,
    items: &mut Vec<InternalCompletionItem<'db>>,
) -> Option<()> {
    let (range, is_math) = find_reference(context).or_else(|| find_reference_range(context))?;

    let db = context.db;
    for document in context
        .workspace
        .related(db, context.distro, context.document)
    {
        if let Some(data) = document.parse(db).as_tex() {
            for label in data
                .analyze(db)
                .labels(db)
                .iter()
                .filter(|label| label.origin(db).as_definition().is_some())
            {
                match util::label::render(db, *document, *label) {
                    Some(rendered_label) => {
                        let kind = match &rendered_label.object {
                            LabeledObject::Section { .. } => Structure::Section,
                            LabeledObject::Float { .. } => Structure::Float,
                            LabeledObject::Theorem { .. } => Structure::Theorem,
                            LabeledObject::Equation => Structure::Equation,
                            LabeledObject::EnumItem => Structure::Item,
                        };

                        if is_math && kind != Structure::Equation {
                            continue;
                        }

                        let header = rendered_label.detail(db);
                        let footer = match &rendered_label.object {
                            LabeledObject::Float { caption, .. } => Some(caption.clone()),
                            _ => None,
                        };

                        let text = format!(
                            "{} {}",
                            label.name(db).text(db),
                            rendered_label.reference(db)
                        );

                        let item = InternalCompletionItem::new(
                            range,
                            InternalCompletionItemData::Label {
                                name: label.name(db).text(db),
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
                        let text = label.name(db).text(db).clone();
                        let item = InternalCompletionItem::new(
                            range,
                            InternalCompletionItemData::Label {
                                name: label.name(db).text(db),
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

    Some(())
}

fn find_reference(context: &CursorContext) -> Option<(TextRange, bool)> {
    let (_, range, group) = context.find_curly_group_word_list()?;
    let reference = latex::LabelReference::cast(group.syntax().parent()?)?;
    let is_math = reference.command()?.text() == "\\eqref";
    Some((range, is_math))
}

fn find_reference_range(context: &CursorContext) -> Option<(TextRange, bool)> {
    let (_, range, group) = context.find_curly_group_word()?;
    latex::LabelReferenceRange::cast(group.syntax().parent()?)?;
    Some((range, false))
}
