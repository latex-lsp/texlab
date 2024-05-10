use rowan::ast::AstNode;
use rustc_hash::FxHashSet;
use syntax::latex;

use crate::{
    util::{find_curly_group_word_list, CompletionBuilder},
    CompletionItem, CompletionItemData, CompletionParams,
};

pub fn complete_imports<'a>(
    params: &'a CompletionParams<'a>,
    builder: &mut CompletionBuilder<'a>,
) -> Option<()> {
    let (cursor, group) = find_curly_group_word_list(params)?;

    let kind = group.syntax().parent()?.kind();
    let extension = match kind {
        latex::PACKAGE_INCLUDE => "sty",
        latex::CLASS_INCLUDE => "cls",
        _ => return Some(()),
    };

    let mut file_names = FxHashSet::default();
    for file_name in completion_data::DATABASE
        .iter()
        .flat_map(|package| package.file_names.iter())
        .filter(|file_name| file_name.ends_with(extension))
    {
        file_names.insert(file_name);
        let stem = &file_name[0..file_name.len() - 4];
        if kind == latex::PACKAGE_INCLUDE {
            if let Some(score) = builder.matcher.score(stem, &cursor.text) {
                let data = CompletionItemData::Package(stem);
                builder
                    .items
                    .push(CompletionItem::new_simple(score, cursor.range, data));
            }
        } else if let Some(score) = builder.matcher.score(stem, &cursor.text) {
            let data = CompletionItemData::DocumentClass(stem);
            builder
                .items
                .push(CompletionItem::new_simple(score, cursor.range, data));
        }
    }

    let file_name_db = &params.feature.workspace.distro().file_name_db;
    for file_name in file_name_db
        .iter()
        .map(|(file_name, _)| file_name)
        .filter(|file_name| file_name.ends_with(extension) && !file_names.contains(file_name))
    {
        let stem = &file_name[0..file_name.len() - 4];
        if kind == latex::PACKAGE_INCLUDE {
            if let Some(score) = builder.matcher.score(stem, &cursor.text) {
                let data = CompletionItemData::Package(stem);
                builder
                    .items
                    .push(CompletionItem::new_simple(score, cursor.range, data));
            }
        } else if let Some(score) = builder.matcher.score(stem, &cursor.text) {
            let data = CompletionItemData::DocumentClass(stem);
            builder
                .items
                .push(CompletionItem::new_simple(score, cursor.range, data));
        }
    }

    Some(())
}
