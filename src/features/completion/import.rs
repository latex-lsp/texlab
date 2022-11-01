use rowan::ast::AstNode;
use rustc_hash::FxHashSet;
use smol_str::SmolStr;

use crate::{
    component_db::COMPONENT_DATABASE, db::Distro, syntax::latex, util::cursor::CursorContext,
};

use super::types::{InternalCompletionItem, InternalCompletionItemData};

pub fn complete_imports<'db>(
    context: &'db CursorContext,
    items: &mut Vec<InternalCompletionItem<'db>>,
) -> Option<()> {
    let (_, range, group) = context.find_curly_group_word_list()?;

    let (extension, mut factory): (
        &str,
        Box<dyn FnMut(SmolStr) -> InternalCompletionItemData<'db>>,
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

    let file_name_db = Distro::get(context.db).file_name_db(context.db);
    for file_name in file_name_db
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
