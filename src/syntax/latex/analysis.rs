mod command;
mod distro_file;
mod environment;
mod explicit_link;
mod implicit_link;
mod label_name;
mod label_number;
mod theorem;
mod types;

use crate::syntax::latex;

pub use self::types::*;
use self::{
    command::analyze_command,
    environment::analyze_begin,
    explicit_link::{analyze_import, analyze_include},
    implicit_link::analyze_implicit_links,
    label_name::analyze_label_name,
    label_number::analyze_label_number,
    theorem::analyze_theorem_definition,
};

pub fn analyze(context: &mut LatexAnalyzerContext, root: &latex::SyntaxNode) {
    analyze_implicit_links(context);
    for node in root.descendants() {
        analyze_command(context, node)
            .or_else(|| analyze_begin(context, node))
            .or_else(|| analyze_include(context, node))
            .or_else(|| analyze_import(context, node))
            .or_else(|| analyze_label_name(context, node))
            .or_else(|| analyze_label_number(context, node))
            .or_else(|| analyze_theorem_definition(context, node));
    }
    context.extras.has_document_environment = context.extras.environment_names.contains("document");
}
