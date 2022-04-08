use cancellation::CancellationToken;
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
    cancellation_token: &CancellationToken,
) -> Option<()> {
    cancellation_token.result().ok()?;

    let (range, is_math) = find_reference(context).or_else(|| find_reference_range(context))?;

    for document in &context.request.subset.documents {
        if let Some(data) = document.data.as_latex() {
            for label in latex::SyntaxNode::new_root(data.root.clone())
                .descendants()
                .filter(|_| !cancellation_token.is_canceled())
                .filter_map(latex::LabelDefinition::cast)
            {
                if let Some(name) = label
                    .name()
                    .and_then(|name| name.key())
                    .map(|name| name.to_string())
                {
                    match render_label(&context.request.subset, &name, Some(label)) {
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

#[cfg(test)]
mod tests {
    use rowan::TextRange;

    use crate::features::testing::FeatureTester;

    use super::*;

    #[test]
    fn test_empty_latex_document() {
        let request = FeatureTester::builder()
            .files(vec![("main.tex", "")])
            .main("main.tex")
            .line(0)
            .character(0)
            .build()
            .completion();

        let context = CursorContext::new(request);
        let mut actual_items = Vec::new();
        complete_labels(&context, &mut actual_items, CancellationToken::none());

        assert!(actual_items.is_empty());
    }

    #[test]
    fn test_empty_bibtex_document() {
        let request = FeatureTester::builder()
            .files(vec![("main.bib", "")])
            .main("main.bib")
            .line(0)
            .character(0)
            .build()
            .completion();

        let context = CursorContext::new(request);
        let mut actual_items = Vec::new();
        complete_labels(&context, &mut actual_items, CancellationToken::none());

        assert!(actual_items.is_empty());
    }

    #[test]
    fn test_simple() {
        let request = FeatureTester::builder()
            .files(vec![("main.tex", "\\ref{}\\label{foo}")])
            .main("main.tex")
            .line(0)
            .character(5)
            .build()
            .completion();

        let context = CursorContext::new(request);
        let mut actual_items = Vec::new();
        complete_labels(&context, &mut actual_items, CancellationToken::none());

        assert!(!actual_items.is_empty());
        for item in actual_items {
            assert_eq!(item.range, TextRange::new(5.into(), 5.into()));
        }
    }

    #[test]
    fn test_simple_range() {
        let request = FeatureTester::builder()
            .files(vec![("main.tex", "\\crefrange{\n\\label{foo}")])
            .main("main.tex")
            .line(0)
            .character(11)
            .build()
            .completion();

        let context = CursorContext::new(request);
        let mut actual_items = Vec::new();
        complete_labels(&context, &mut actual_items, CancellationToken::none());

        assert!(!actual_items.is_empty());
        for item in actual_items {
            assert_eq!(item.range, TextRange::new(11.into(), 11.into()));
        }
    }

    #[test]
    fn test_multi_word() {
        let request = FeatureTester::builder()
            .files(vec![("main.tex", "\\ref{foo}\\label{foo bar}")])
            .main("main.tex")
            .line(0)
            .character(8)
            .build()
            .completion();

        let context = CursorContext::new(request);
        let mut actual_items = Vec::new();
        complete_labels(&context, &mut actual_items, CancellationToken::none());

        assert!(!actual_items.is_empty());
        for item in actual_items {
            assert_eq!(item.range, TextRange::new(5.into(), 8.into()));
        }
    }
}
