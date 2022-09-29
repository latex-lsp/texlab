use std::sync::Arc;

use lsp_types::RenameParams;
use rowan::{ast::AstNode, TextRange};
use rustc_hash::FxHashMap;

use crate::{
    features::cursor::{CursorContext, HasPosition},
    syntax::{
        bibtex::{self, HasName},
        latex,
    },
    DocumentData,
};

use super::{Indel, RenameResult};

pub(super) fn prepare_entry_rename<P: HasPosition>(
    context: &CursorContext<P>,
) -> Option<TextRange> {
    let (_, range) = context
        .find_citation_key_word()
        .or_else(|| context.find_entry_key())?;

    Some(range)
}

pub(super) fn rename_entry(context: &CursorContext<RenameParams>) -> Option<RenameResult> {
    prepare_entry_rename(context)?;
    let (key_text, _) = context
        .find_citation_key_word()
        .or_else(|| context.find_entry_key())?;

    let mut changes = FxHashMap::default();
    for document in context.request.workspace.iter() {
        let uri = Arc::clone(&document.uri);
        match &document.data {
            DocumentData::Latex(data) => {
                let root = latex::SyntaxNode::new_root(data.green.clone());
                let edits: Vec<_> = root
                    .descendants()
                    .filter_map(latex::Citation::cast)
                    .filter_map(|citation| citation.key_list())
                    .flat_map(|keys| keys.keys())
                    .filter(|key| key.to_string() == key_text)
                    .map(|key| Indel {
                        delete: latex::small_range(&key),
                        insert: context.request.params.new_name.clone(),
                    })
                    .collect();
                changes.insert(uri, edits);
            }
            DocumentData::Bibtex(data) => {
                let root = bibtex::SyntaxNode::new_root(data.green.clone());
                let edits: Vec<_> = root
                    .descendants()
                    .filter_map(bibtex::Entry::cast)
                    .filter_map(|entry| entry.name_token())
                    .filter(|key| key.text() == key_text)
                    .map(|key| Indel {
                        delete: key.text_range(),
                        insert: context.request.params.new_name.clone(),
                    })
                    .collect();
                changes.insert(uri, edits);
            }
            DocumentData::BuildLog(_) => {}
        }
    }

    Some(RenameResult { changes })
}
