use lsp_types::{InlayHint, InlayHintLabel, InlayHintParams};
use rowan::ast::AstNode;

use crate::{
    features::FeatureRequest,
    render_label,
    syntax::latex::{self, LabelDefinition},
    LineIndexExt,
};

pub fn find_label_inlay_hints(
    request: &FeatureRequest<InlayHintParams>,
    hints: &mut Vec<InlayHint>,
) -> Option<()> {
    let main_document = request.main_document();
    let data = main_document.data().as_latex()?;
    let root = latex::SyntaxNode::new_root(data.green.clone());

    let range = main_document
        .line_index()
        .offset_lsp_range(request.params.range);

    hints.extend(
        root.descendants()
            .filter_map(latex::LabelDefinition::cast)
            .filter(|label| label.syntax().text_range().intersect(range).is_some())
            .filter_map(|label| create_hint(request, &label)),
    );

    Some(())
}

fn create_hint(
    request: &FeatureRequest<InlayHintParams>,
    label: &LabelDefinition,
) -> Option<InlayHint> {
    let name = label.name().and_then(|group| group.key())?;

    let rendered = render_label(&request.workspace, &name.to_string(), Some(label.clone()))?;

    let position = request
        .main_document()
        .line_index()
        .line_col_lsp(name.syntax().text_range().end());

    Some(InlayHint {
        position,
        label: InlayHintLabel::String(rendered.reference()),
        kind: None,
        text_edits: None,
        tooltip: None,
        padding_left: Some(true),
        padding_right: None,
        data: None,
    })
}
