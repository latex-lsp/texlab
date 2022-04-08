use rowan::ast::AstNode;

use crate::syntax::latex;

use super::LatexAnalyzerContext;

pub fn analyze_command(context: &mut LatexAnalyzerContext, node: latex::SyntaxNode) -> Option<()> {
    let command = latex::GenericCommand::cast(node)?;
    context
        .extras
        .command_names
        .insert(command.name()?.text().into());
    Some(())
}

pub fn analyze_command_definition(
    context: &mut LatexAnalyzerContext,
    node: latex::SyntaxNode,
) -> Option<()> {
    let definition = latex::CommandDefinition::cast(node)?;
    context
        .extras
        .command_names
        .insert(definition.name()?.command()?.text().into());
    Some(())
}
