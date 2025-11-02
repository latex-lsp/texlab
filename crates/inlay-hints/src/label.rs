use base_db::{
    FeatureParams,
    semantics::tex::{Label, LabelKind},
    util::{queries::Object, render_label},
};
use rustc_hash::FxHashMap;

use crate::{InlayHint, InlayHintBuilder, InlayHintData};

pub(super) fn find_hints(builder: &mut InlayHintBuilder) -> Option<()> {
    let definitions = base_db::semantics::tex::Label::find_all(&builder.params.feature.project)
        .filter(|(_, label)| label.kind == LabelKind::Definition)
        .map(|(_, label)| (label.name_text(), label))
        .collect::<FxHashMap<_, _>>();

    let params = &builder.params.feature;
    let data = params.document.data.as_tex()?;
    let range = builder.params.range;
    for label in data
        .semantics
        .labels
        .iter()
        .filter(|label| label.name.range.intersect(range).is_some())
    {
        if let Some(hint) = process_label(params, &definitions, label) {
            builder.hints.push(hint);
        }
    }

    Some(())
}

fn process_label<'a>(
    params: &FeatureParams<'a>,
    definitions: &FxHashMap<&str, &'a Label>,
    label: &'a Label,
) -> Option<InlayHint<'a>> {
    let config = &params.workspace.config().inlay_hints;
    let offset = label.full_range.end();
    let data = if label.kind == LabelKind::Definition {
        if !config.label_definitions {
            return None;
        }

        let label = render_label(params.workspace, &params.project, label)?;
        InlayHintData::LabelDefinition(label)
    } else {
        if !config.label_references {
            return None;
        }

        let label = definitions.get(label.name.text.as_str())?;
        let label = render_label(params.workspace, &params.project, label)?;
        InlayHintData::LabelReference(label)
    };

    Some(InlayHint { offset, data })
}
