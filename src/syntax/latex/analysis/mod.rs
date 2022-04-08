mod command;
mod distro_file;
mod environment;
mod explicit_link;
mod graphics_path;
mod implicit_link;
mod label_name;
mod label_number;
mod theorem;
mod types;

use crate::syntax::latex;

pub use self::types::*;
use self::{
    command::{analyze_command, analyze_command_definition},
    environment::analyze_begin,
    explicit_link::{analyze_import, analyze_include},
    graphics_path::analyze_graphics_path,
    implicit_link::analyze_implicit_links,
    label_name::analyze_label_name,
    label_number::analyze_label_number,
    theorem::analyze_theorem_definition,
};

pub fn analyze(context: &mut LatexAnalyzerContext, root: &latex::SyntaxNode) {
    analyze_implicit_links(context);
    for node in root.descendants() {
        analyze_command(context, node.clone())
            .or_else(|| analyze_command_definition(context, node.clone()))
            .or_else(|| analyze_begin(context, node.clone()))
            .or_else(|| analyze_include(context, node.clone()))
            .or_else(|| analyze_import(context, node.clone()))
            .or_else(|| analyze_label_name(context, node.clone()))
            .or_else(|| analyze_label_number(context, node.clone()))
            .or_else(|| analyze_theorem_definition(context, node.clone()))
            .or_else(|| analyze_graphics_path(context, node));
    }
    context.extras.has_document_environment = context.extras.environment_names.contains("document");
}
