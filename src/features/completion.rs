mod acronym_ref;
mod argument;
mod begin_snippet;
pub mod builder;
mod citation;
mod color;
mod color_model;
mod component_command;
mod component_environment;
mod entry_type;
mod field;
mod glossary_ref;
mod import;
mod include;
mod label;
mod theorem;
mod tikz_library;
mod user_command;
mod user_environment;

use lsp_types::{CompletionList, Position, Url};

use crate::{features::completion::builder::CompletionBuilder, util::cursor::CursorContext, Db};

pub const COMPLETION_LIMIT: usize = 50;

pub fn complete(db: &dyn Db, uri: &Url, position: Position) -> Option<CompletionList> {
    let context = CursorContext::new(db, uri, position, ())?;
    let mut builder = CompletionBuilder::new(&context);
    log::debug!("[Completion] Cursor: {:?}", context.cursor);
    entry_type::complete_entry_types(&context, &mut builder);
    field::complete_fields(&context, &mut builder);
    argument::complete_arguments(&context, &mut builder);
    citation::complete_citations(&context, &mut builder);
    import::complete_imports(&context, &mut builder);
    color::complete_colors(&context, &mut builder);
    color_model::complete_color_models(&context, &mut builder);
    acronym_ref::complete_acronyms(&context, &mut builder);
    glossary_ref::complete_glossary_entries(&context, &mut builder);
    include::complete_includes(&context, &mut builder);
    label::complete_labels(&context, &mut builder);
    tikz_library::complete_tikz_libraries(&context, &mut builder);
    component_environment::complete_component_environments(&context, &mut builder);
    theorem::complete_theorem_environments(&context, &mut builder);
    user_environment::complete_user_environments(&context, &mut builder);
    begin_snippet::complete_begin_snippet(&context, &mut builder);
    component_command::complete_component_commands(&context, &mut builder);
    user_command::complete_user_commands(&context, &mut builder);
    Some(builder.finish())
}
