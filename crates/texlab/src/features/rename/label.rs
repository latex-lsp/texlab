use rowan::{ast::AstNode, TextRange};
use rustc_hash::FxHashMap;
use syntax::latex;

use crate::util::cursor::CursorContext;

use super::{Indel, Params, RenameResult};

pub(super) fn prepare_rename<T>(context: &CursorContext<T>) -> Option<TextRange> {
    let (_, range) = context.find_label_name_key()?;
    Some(range)
}

pub(super) fn rename(context: &CursorContext<Params>) -> Option<RenameResult> {
    prepare_rename(context)?;
    let (name_text, _) = context.find_label_name_key()?;

    let mut changes = FxHashMap::default();
    for document in context.related() {
        if let Some(data) = document.parse(context.db).as_tex() {
            let mut edits = Vec::new();
            for node in data.root(context.db).descendants() {
                if let Some(range) = latex::LabelDefinition::cast(node.clone())
                    .and_then(|label| label.name())
                    .and_then(|name| name.key())
                    .filter(|name| name.to_string() == name_text)
                    .map(|name| latex::small_range(&name))
                {
                    edits.push(Indel {
                        delete: range,
                        insert: context.params.new_name.clone(),
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
                            insert: context.params.new_name.clone(),
                        });
                    });

                if let Some(label) = latex::LabelReferenceRange::cast(node.clone()) {
                    if let Some(name_from) = label
                        .from()
                        .and_then(|name| name.key())
                        .filter(|name| name.to_string() == name_text)
                    {
                        edits.push(Indel {
                            delete: latex::small_range(&name_from),
                            insert: context.params.new_name.clone(),
                        });
                    }

                    if let Some(name_to) = label
                        .to()
                        .and_then(|name| name.key())
                        .filter(|name| name.to_string() == name_text)
                    {
                        edits.push(Indel {
                            delete: latex::small_range(&name_to),
                            insert: context.params.new_name.clone(),
                        });
                    }
                }
            }

            changes.insert(document, edits);
        }
    }

    Some(RenameResult { changes })
}
