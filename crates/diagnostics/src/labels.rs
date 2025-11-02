use base_db::{
    DocumentData, Workspace,
    semantics::tex::{Label, LabelKind},
    util::queries,
};
use itertools::Itertools;
use rustc_hash::{FxHashMap, FxHashSet};
use url::Url;

use crate::types::{Diagnostic, TexError};

pub fn detect_undefined_and_unused_labels(
    workspace: &Workspace,
    results: &mut FxHashMap<Url, Vec<Diagnostic>>,
) {
    for document in workspace.iter() {
        let DocumentData::Tex(data) = &document.data else {
            continue;
        };

        let mut label_refs = FxHashSet::default();
        let mut label_defs = FxHashSet::default();
        let project = workspace
            .graphs()
            .values()
            .filter(|graph| graph.preorder(workspace).contains(&document))
            .flat_map(|graph| graph.preorder(workspace));

        for label in project
            .filter_map(|child| child.data.as_tex())
            .flat_map(|data| data.semantics.labels.iter())
        {
            if label.kind == LabelKind::Definition {
                label_defs.insert(&label.name.text);
            } else {
                label_refs.insert(&label.name.text);
            }
        }

        for label in &data.semantics.labels {
            if label.kind != LabelKind::Definition && !label_defs.contains(&label.name.text) {
                let diagnostic = Diagnostic::Tex(label.name.range, TexError::UndefinedLabel);
                results
                    .entry(document.uri.clone())
                    .or_default()
                    .push(diagnostic);
            }

            if label.kind == LabelKind::Definition && !label_refs.contains(&label.name.text) {
                let diagnostic = Diagnostic::Tex(label.name.range, TexError::UnusedLabel);
                results
                    .entry(document.uri.clone())
                    .or_default()
                    .push(diagnostic);
            }
        }
    }
}

pub fn detect_duplicate_labels(
    workspace: &Workspace,
    results: &mut FxHashMap<Url, Vec<Diagnostic>>,
) {
    for conflict in queries::Conflict::find_all::<Label>(workspace) {
        let others = conflict
            .rest
            .iter()
            .map(|location| (location.document.uri.clone(), location.range))
            .collect();

        let diagnostic = Diagnostic::Tex(conflict.main.range, TexError::DuplicateLabel(others));
        results
            .entry(conflict.main.document.uri.clone())
            .or_default()
            .push(diagnostic);
    }
}
