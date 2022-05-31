use std::sync::Arc;

use lsp_types::RenameParams;
use rowan::{ast::AstNode, TextRange};
use rustc_hash::FxHashMap;

use crate::{
    features::cursor::{CursorContext, HasPosition},
    syntax::latex,
};

use super::{Indel, RenameResult};

pub(super) fn prepare_label_rename<P: HasPosition>(
    context: &CursorContext<P>,
) -> Option<TextRange> {
    let (_, range) = context.find_label_name_key()?;
    Some(range)
}

pub(super) fn rename_label(context: &CursorContext<RenameParams>) -> Option<RenameResult> {
    prepare_label_rename(context)?;
    let (name_text, _) = context.find_label_name_key()?;

    let mut changes = FxHashMap::default();
    for document in context.request.workspace.documents_by_uri.values() {
        if let Some(data) = document.data.as_latex() {
            let mut edits = Vec::new();
            for node in latex::SyntaxNode::new_root(data.green.clone()).descendants() {
                if let Some(range) = latex::LabelDefinition::cast(node.clone())
                    .and_then(|label| label.name())
                    .and_then(|name| name.key())
                    .filter(|name| name.to_string() == name_text)
                    .map(|name| latex::small_range(&name))
                {
                    edits.push(Indel {
                        delete: range,
                        insert: context.request.params.new_name.clone(),
                    });
                }

                latex::LabelReference::cast(node.clone())
                    .and_then(|label| label.name_list())
                    .into_iter()
                    .flat_map(|label| label.keys())
                    .filter(|name| name.to_string() == name_text)
                    .for_each(|name| {
                        edits.push(Indel {
                            delete: latex::small_range(&name),
                            insert: context.request.params.new_name.clone(),
                        });
                    });

                if let Some(label) = latex::LabelReferenceRange::cast(node.clone()) {
                    if let Some(name1) = label
                        .from()
                        .and_then(|name| name.key())
                        .filter(|name| name.to_string() == name_text)
                    {
                        edits.push(Indel {
                            delete: latex::small_range(&name1),
                            insert: context.request.params.new_name.clone(),
                        });
                    }

                    if let Some(name2) = label
                        .from()
                        .and_then(|name| name.key())
                        .filter(|name| name.to_string() == name_text)
                    {
                        edits.push(Indel {
                            delete: latex::small_range(&name2),
                            insert: context.request.params.new_name.clone(),
                        });
                    }
                }
            }

            changes.insert(Arc::clone(&document.uri), edits);
        }
    }

    Some(RenameResult { changes })
}
