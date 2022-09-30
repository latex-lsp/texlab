use lsp_types::{DocumentHighlight, DocumentHighlightKind, DocumentHighlightParams};
use rowan::ast::AstNode;

use crate::{features::cursor::CursorContext, syntax::latex, LineIndexExt};

pub fn find_label_highlights(
    context: &CursorContext<DocumentHighlightParams>,
) -> Option<Vec<DocumentHighlight>> {
    let (name_text, _) = context.find_label_name_key()?;

    let main_document = context.request.main_document();
    let data = main_document.data().as_latex()?;

    let mut highlights = Vec::new();
    for node in latex::SyntaxNode::new_root(data.green.clone()).descendants() {
        if let Some(label_name) = latex::LabelDefinition::cast(node.clone())
            .and_then(|label| label.name())
            .and_then(|label_name| label_name.key())
            .filter(|label_name| label_name.to_string() == name_text)
        {
            let range = main_document
                .line_index()
                .line_col_lsp_range(latex::small_range(&label_name));

            highlights.push(DocumentHighlight {
                range,
                kind: Some(DocumentHighlightKind::WRITE),
            });
        } else if let Some(label) = latex::LabelReference::cast(node.clone()) {
            for label_name in label
                .name_list()
                .into_iter()
                .flat_map(|name| name.keys())
                .filter(|label_name| label_name.to_string() == name_text)
            {
                let range = main_document
                    .line_index()
                    .line_col_lsp_range(latex::small_range(&label_name));

                highlights.push(DocumentHighlight {
                    range,
                    kind: Some(DocumentHighlightKind::READ),
                });
            }
        } else if let Some(label) = latex::LabelReferenceRange::cast(node.clone()) {
            if let Some(label_name) = label
                .from()
                .and_then(|label_name| label_name.key())
                .filter(|label_name| label_name.to_string() == name_text)
            {
                let range = main_document
                    .line_index()
                    .line_col_lsp_range(latex::small_range(&label_name));

                highlights.push(DocumentHighlight {
                    range,
                    kind: Some(DocumentHighlightKind::READ),
                });
            }

            if let Some(label_name) = label
                .to()
                .and_then(|label_name| label_name.key())
                .filter(|label_name| label_name.to_string() == name_text)
            {
                let range = main_document
                    .line_index()
                    .line_col_lsp_range(latex::small_range(&label_name));

                highlights.push(DocumentHighlight {
                    range,
                    kind: Some(DocumentHighlightKind::READ),
                });
            }
        }
    }

    Some(highlights)
}
