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
    let description = description.content_text()?;

    context
        .extras
        .theorem_environments
        .push(TheoremEnvironment { name, description });

    Some(())
}
