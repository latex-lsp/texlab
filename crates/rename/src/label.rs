use base_db::{
    semantics::{tex, Span},
    util::queries::{self, Object},
};
use rustc_hash::FxHashMap;

use crate::{RenameBuilder, RenameInformation, RenameParams};

pub(super) fn prepare_rename(params: &RenameParams) -> Option<Span> {
    let data = params.feature.document.data.as_tex()?;
    let labels = &data.semantics.labels;
    let label = queries::object_at_cursor(labels, params.offset, queries::SearchMode::Name)?;
    Some(Span::new(label.object.name.text.clone(), label.range))
}

struct PrefixInformation<'a> {
    def_prefixes: &'a FxHashMap<String, String>,
    ref_prefixes: &'a FxHashMap<String, String>,
}

fn label_has_prefix(pref_info: &PrefixInformation, label: &tex::Label) -> Option<String> {
    match label.kind {
        tex::LabelKind::Definition => pref_info
            .def_prefixes
            .get(&label.cmd.clone().unwrap_or(String::new()))
            .cloned(),
        _ => pref_info
            .ref_prefixes
            .get(&label.cmd.clone().unwrap_or(String::new()))
            .cloned(),
    }
}

fn find_prefix_in_any(
    builder: &mut RenameBuilder,
    pref_info: &PrefixInformation,
    name: &str,
) -> Option<String> {
    let project = &builder.params.feature.project;
    queries::objects_with_name::<tex::Label>(project, name)
        .find_map(|(_, label)| label_has_prefix(&pref_info, label))
}

pub(super) fn rename(builder: &mut RenameBuilder) -> Option<()> {
    let name = prepare_rename(&builder.params)?;

    let syn = &builder.params.feature.workspace.config().syntax;
    let pref_info = PrefixInformation {
        def_prefixes: &syn.label_definition_prefixes,
        ref_prefixes: &syn.label_reference_prefixes,
    };
    let prefix = find_prefix_in_any(builder, &pref_info, &name.text);

    let project = &builder.params.feature.project;
    for (document, label) in queries::objects_with_name::<tex::Label>(project, &name.text) {
        let prefix = label_has_prefix(&pref_info, label).map_or(prefix.clone(), |_| None);

        let entry = builder.result.changes.entry(document);
        entry.or_default().push(RenameInformation {
            range: label.name_range(),
            prefix: prefix.clone(),
        });
    }

    Some(())
}
