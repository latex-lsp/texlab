use base_db::{util::RenderedObject, FeatureParams, Workspace};
use inlay_hints::{InlayHintData, InlayHintParams};

use crate::util::line_index_ext::LineIndexExt;

pub fn find_all(
    workspace: &Workspace,
    uri: &lsp_types::Url,
    range: lsp_types::Range,
) -> Option<Vec<lsp_types::InlayHint>> {
    let document = workspace.lookup(uri)?;
    let line_index = &document.line_index;
    let range = line_index.offset_lsp_range(range)?;

    let feature = FeatureParams::new(workspace, document);
    let params = InlayHintParams { range, feature };
    let hints = inlay_hints::find_all(params)?;
    let hints = hints.into_iter().filter_map(|hint| {
        let position = line_index.line_col_lsp(hint.offset)?;
        Some(match hint.data {
            InlayHintData::LabelDefinition(label) => {
                let number = label.number?;

                let text = match &label.object {
                    RenderedObject::Section { prefix, .. } => {
                        format!("{} {}", prefix, number)
                    }
                    RenderedObject::Float { kind, .. } => {
                        format!("{} {}", kind.as_str(), number)
                    }
                    RenderedObject::Theorem { kind, .. } => {
                        format!("{} {}", kind, number)
                    }
                    RenderedObject::Equation => format!("Equation ({})", number),
                    RenderedObject::EnumItem => format!("Item {}", number),
                };

                lsp_types::InlayHint {
                    position,
                    label: lsp_types::InlayHintLabel::String(format!(" {text} ")),
                    kind: None,
                    text_edits: None,
                    tooltip: None,
                    padding_left: Some(true),
                    padding_right: None,
                    data: None,
                }
            }
            InlayHintData::LabelReference(label) => {
                let text = label.reference();

                lsp_types::InlayHint {
                    position,
                    label: lsp_types::InlayHintLabel::String(format!(" {text} ")),
                    kind: None,
                    text_edits: None,
                    tooltip: None,
                    padding_left: Some(true),
                    padding_right: None,
                    data: None,
                }
            }
            InlayHintData::Citation(text) => lsp_types::InlayHint {
                position,
                label: lsp_types::InlayHintLabel::String(format!(" {text} ")),
                kind: None,
                text_edits: None,
                tooltip: None,
                padding_left: Some(true),
                padding_right: None,
                data: None,
            },
        })
    });

    Some(hints.collect())
}
