use crate::syntax::{latex, CstNode};

use super::LatexAnalyzerContext;

pub fn analyze_begin(context: &mut LatexAnalyzerContext, node: &latex::SyntaxNode) -> Option<()> {
    let begin = latex::Begin::cast(node)?;
    let name = begin.name()?.word()?.text();
    let extras = &mut context.extras;
    extras.environment_names.insert(name.into());
    Some(())
}