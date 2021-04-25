use cstree::TextSize;

use crate::syntax::{
    latex::{self, HasCurly},
    CstNode,
};

use super::{LatexAnalyzerContext, TheoremEnvironment};

pub fn analyze_theorem_definition(
    context: &mut LatexAnalyzerContext,
    node: &latex::SyntaxNode,
) -> Option<()> {
    let theorem = latex::TheoremDefinition::cast(node)?;
    let name = theorem.name()?.word()?.text().into();
    let description = theorem.description()?;
    description.left_curly()?;
    description.right_curly()?;
    let description = description.syntax().text();
    let description = description
        .slice(TextSize::from(1)..description.len() - TextSize::from(1))
        .to_string()
        .trim()
        .to_string();

    context
        .extras
        .theorem_environments
        .push(TheoremEnvironment { name, description });

    Some(())
}
