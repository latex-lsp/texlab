use base_db::{semantics::tex::LabelKind, DocumentData, Workspace};
use itertools::Itertools;
use rustc_hash::{FxHashMap, FxHashSet};
use url::Url;

use crate::{Diagnostic, DiagnosticData, DiagnosticSource, LabelError};

#[derive(Debug, Default)]
pub struct LabelErrors {
    errors: FxHashMap<Url, Vec<Diagnostic>>,
}

impl DiagnosticSource for LabelErrors {
    fn publish<'a>(
        &'a mut self,
        workspace: &'a Workspace,
        results: &mut FxHashMap<&'a Url, Vec<&'a Diagnostic>>,
    ) {
        let graphs: Vec<_> = workspace
            .iter()
            .map(|start| base_db::graph::Graph::new(workspace, start))
            .collect();

        self.errors = Default::default();
        for document in workspace.iter() {
            let DocumentData::Tex(data) = &document.data else { continue };

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

            let mut errors = Vec::new();
            for label in &data.semantics.labels {
                if label.kind != LabelKind::Definition && !label_defs.contains(&label.name.text) {
                    errors.push(Diagnostic {
                        range: label.name.range,
                        data: DiagnosticData::Label(LabelError::Undefined),
                    });
                }

                if label.kind == LabelKind::Definition && !label_refs.contains(&label.name.text) {
                    errors.push(Diagnostic {
                        range: label.full_range,
                        data: DiagnosticData::Label(LabelError::Unused),
                    });
                }
            }

            self.errors.insert(document.uri.clone(), errors);
        }

        for document in workspace.iter() {
            let Some(diagnostics) = self.errors.get(&document.uri) else { continue };

            results
                .entry(&document.uri)
                .or_default()
                .extend(diagnostics.iter());
        }
    }
}
