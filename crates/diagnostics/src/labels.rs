use base_db::{
    deps,
    semantics::tex::{Label, LabelKind},
    util::queries,
    DocumentData, Workspace,
};
use itertools::Itertools;
use multimap::MultiMap;
use rustc_hash::FxHashSet;
use url::Url;

use crate::types::{Diagnostic, TexError};

pub fn detect_undefined_and_unused_labels(
    workspace: &Workspace,
    results: &mut MultiMap<Url, Diagnostic>,
) {
    let graphs: Vec<_> = workspace
        .iter()
        .map(|start| deps::Graph::new(workspace, start))
        .collect();

    for document in workspace.iter() {
        let DocumentData::Tex(data) = &document.data else {
            continue;
        };

        let mut label_refs = FxHashSet::default();
        let mut label_defs = FxHashSet::default();
        let project = graphs
            .iter()
            .filter(|graph| graph.preorder().contains(&document))
            .flat_map(|graph| graph.preorder());

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
                results.insert(document.uri.clone(), diagnostic);
            }

            if label.kind == LabelKind::Definition && !label_refs.contains(&label.name.text) {
                let diagnostic = Diagnostic::Tex(label.name.range, TexError::UnusedLabel);
                results.insert(document.uri.clone(), diagnostic);
            }
        }
    }
}

pub fn detect_duplicate_labels(workspace: &Workspace, results: &mut MultiMap<Url, Diagnostic>) {
    for conflict in queries::Conflict::find_all::<Label>(workspace) {
        let others = conflict
            .rest
            .iter()
            .map(|location| (location.document.uri.clone(), location.range))
            .collect();

        let diagnostic = Diagnostic::Tex(conflict.main.range, TexError::DuplicateLabel(others));
        results.insert(conflict.main.document.uri.clone(), diagnostic);
    }
}
