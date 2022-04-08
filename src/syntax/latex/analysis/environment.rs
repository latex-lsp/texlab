use rowan::ast::AstNode;

use crate::syntax::latex;

use super::LatexAnalyzerContext;

pub fn analyze_begin(context: &mut LatexAnalyzerContext, node: latex::SyntaxNode) -> Option<()> {
    let begin = latex::Begin::cast(node)?;
    let name = begin.name()?.key()?.to_string();
    let extras = &mut context.extras;
    extras.environment_names.insert(name.into());
    Some(())
}
