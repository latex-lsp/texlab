use rowan::ast::AstNode;
use rustc_hash::FxHashSet;
use syntax::latex;

use crate::util::cursor::CursorContext;

use super::builder::CompletionBuilder;

pub fn complete<'db>(
    context: &'db CursorContext,
    builder: &mut CompletionBuilder<'db>,
) -> Option<()> {
    let (_, range, group) = context.find_curly_group_word_list()?;

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
            builder.package(range, stem);
        } else {
            builder.class(range, stem);
        }
    }

    let file_name_db = &context.workspace.distro().file_name_db;
    for file_name in file_name_db
        .iter()
        .map(|(file_name, _)| file_name)
        .filter(|file_name| file_name.ends_with(extension) && !file_names.contains(file_name))
    {
        let stem = &file_name[0..file_name.len() - 4];
        if kind == latex::PACKAGE_INCLUDE {
            builder.package(range, stem);
        } else {
            builder.class(range, stem);
        }
    }

    Some(())
}
