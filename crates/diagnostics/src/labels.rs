use std::borrow::Cow;

use base_db::{
    semantics::tex::{Label, LabelKind},
    util::queries,
    DocumentData, Workspace,
};
use itertools::Itertools;
use rustc_hash::FxHashSet;

use crate::{
    types::{DiagnosticData, TexError},
    Diagnostic, DiagnosticBuilder, DiagnosticSource,
};

#[derive(Default)]
pub struct LabelErrors;

impl DiagnosticSource for LabelErrors {
    fn publish<'db>(
        &'db mut self,
        workspace: &'db Workspace,
        builder: &mut DiagnosticBuilder<'db>,
    ) {
        detect_undefined_and_unused_labels(workspace, builder);
        detect_duplicate_labels(workspace, builder);
    }
}

fn detect_undefined_and_unused_labels<'db>(
    workspace: &'db Workspace,
    builder: &mut DiagnosticBuilder<'db>,
) {
    let graphs: Vec<_> = workspace
        .iter()
        .map(|start| base_db::graph::Graph::new(workspace, start))
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
                let diagnostic = Diagnostic {
                    range: label.name.range,
                    data: DiagnosticData::Tex(TexError::UndefinedLabel),
                };
                builder.push(&document.uri, Cow::Owned(diagnostic));
            }

            if label.kind == LabelKind::Definition && !label_refs.contains(&label.name.text) {
                let diagnostic = Diagnostic {
                    range: label.name.range,
                    data: DiagnosticData::Tex(TexError::UnusedLabel),
                };
                builder.push(&document.uri, Cow::Owned(diagnostic));
            }
        }
    }
}

fn detect_duplicate_labels<'db>(workspace: &'db Workspace, builder: &mut DiagnosticBuilder<'db>) {
    for conflict in queries::Conflict::find_all::<Label>(workspace) {
        let others = conflict
            .rest
            .iter()
            .map(|location| (location.document.uri.clone(), location.range))
            .collect();

        let diagnostic = Diagnostic {
            range: conflict.main.range,
            data: DiagnosticData::Tex(TexError::DuplicateLabel(others)),
        };

        builder.push(&conflict.main.document.uri, Cow::Owned(diagnostic));
    }
}
