use base_db::{semantics::tex::LabelKind, DocumentData};
use rowan::{ast::AstNode, TextRange};
use syntax::latex;

use crate::util::{self, cursor::CursorContext, label::LabeledObject, lsp_enums::Structure};

use super::builder::CompletionBuilder;

pub fn complete<'db>(
    context: &'db CursorContext,
    builder: &mut CompletionBuilder<'db>,
) -> Option<()> {
    let (range, is_math) = find_reference(context).or_else(|| find_reference_range(context))?;

    for document in &context.project.documents {
        let DocumentData::Tex(data) = &document.data else { continue };
        for label in data
            .semantics
            .labels
            .iter()
            .filter(|label| label.kind == LabelKind::Definition)
        {
            match util::label::render(context.workspace, &context.project, label) {
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

                    let header = rendered_label.detail();
                    let footer = match &rendered_label.object {
                        LabeledObject::Float { caption, .. } => Some(caption.clone()),
                        _ => None,
                    };

                    let text = format!("{} {}", label.name.text, rendered_label.reference());

                    builder.label(range, &label.name.text, kind, header, footer, text);
                }
                None => {
                    let kind = Structure::Label;
                    let header = None;
                    let footer = None;
                    let text = label.name.text.clone();
                    builder.label(range, &label.name.text, kind, header, footer, text);
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
