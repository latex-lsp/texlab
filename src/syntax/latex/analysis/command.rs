use crate::syntax::{latex, CstNode};

use super::LatexAnalyzerContext;

pub fn analyze_command(context: &mut LatexAnalyzerContext, node: &latex::SyntaxNode) -> Option<()> {
    let command = latex::GenericCommand::cast(node)?;
    context
        .extras
        .command_names
        .insert(command.name()?.text().into());
    Some(())
}
