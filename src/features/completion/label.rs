use cancellation::CancellationToken;
use cstree::TextRange;
use lsp_types::CompletionParams;

use crate::{
    features::{cursor::CursorContext, lsp_kinds::Structure},
    render_label,
    syntax::{latex, CstNode},
    LabelledObject,
};

use super::types::{InternalCompletionItem, InternalCompletionItemData};

pub fn complete_labels<'a>(
    context: &'a CursorContext<CompletionParams>,
    items: &mut Vec<InternalCompletionItem<'a>>,
    cancellation_token: &CancellationToken,
) -> Option<()> {
    cancellation_token.result().ok()?;

    let token = context.cursor.as_latex()?;
    let is_math = latex::CurlyGroupWordList::cast(token.parent())
        .filter(|group| context.is_inside_latex_curly(group))
        .and_then(|group| group.syntax().parent())
        .and_then(|reference| latex::LabelReference::cast(reference))
        .and_then(|refernce| refernce.command())
        .map(|reference| reference.text() == "\\eqref")
        .or_else(|| {
            latex::CurlyGroupWord::cast(token.parent())
                .filter(|group| context.is_inside_latex_curly(group))
                .and_then(|group| group.syntax().parent())
                .and_then(|reference| latex::LabelReferenceRange::cast(reference))
                .map(|_| false)
        })?;

    let range = if token.kind() == latex::WORD {
        token.text_range()
    } else {
        TextRange::empty(context.offset)
    };

    for document in &context.request.subset.documents {
        if let Some(data) = document.data.as_latex() {
            for label in data
                .root
                .descendants()
                .filter(|_| !cancellation_token.is_canceled())
                .filter_map(latex::LabelDefinition::cast)
            {
                if let Some(name) = label
                    .name()
                    .and_then(|name| name.word())
                    .map(|name| name.text())
                {
                    match render_label(&context.request.subset, name, Some(label)) {
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

#[cfg(test)]
mod tests {
    use cstree::TextRange;

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
}
