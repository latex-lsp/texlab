use lsp_types::CompletionParams;
use rowan::ast::AstNode;
use rustc_hash::FxHashSet;
use smol_str::SmolStr;

use crate::{component_db::COMPONENT_DATABASE, features::cursor::CursorContext, syntax::latex};

use super::types::{InternalCompletionItem, InternalCompletionItemData};

pub fn complete_imports<'a>(
    context: &'a CursorContext<CompletionParams>,
    items: &mut Vec<InternalCompletionItem<'a>>,
) -> Option<()> {
    let (_, range, group) = context.find_curly_group_word_list()?;

    let (extension, mut factory): (
        &str,
        Box<dyn FnMut(SmolStr) -> InternalCompletionItemData<'a>>,
    ) = match group.syntax().parent()?.kind() {
        latex::PACKAGE_INCLUDE => (
            "sty",
            Box::new(|name| InternalCompletionItemData::Package { name }),
        ),
        latex::CLASS_INCLUDE => (
            "cls",
            Box::new(|name| InternalCompletionItemData::Class { name }),
        ),
        _ => return None,
    };

    let mut file_names = FxHashSet::default();
    for file_name in COMPONENT_DATABASE
        .components
        .iter()
        .flat_map(|comp| comp.file_names.iter())
        .filter(|file_name| file_name.ends_with(extension))
    {
        file_names.insert(file_name);
        let stem = &file_name[0..file_name.len() - 4];
        let data = factory(stem.into());
        items.push(InternalCompletionItem::new(range, data));
    }

    let resolver = &context.request.workspace.environment.resolver;
    for file_name in resolver
        .files_by_name
        .keys()
        .filter(|file_name| file_name.ends_with(extension) && !file_names.contains(file_name))
    {
        let stem = &file_name[0..file_name.len() - 4];
        let data = factory(stem.into());
        items.push(InternalCompletionItem::new(range, data));
    }

    Some(())
}
