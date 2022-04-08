use rowan::ast::AstNode;

use crate::syntax::latex;

use super::LatexAnalyzerContext;

pub fn analyze_graphics_path(
    context: &mut LatexAnalyzerContext,
    node: latex::SyntaxNode,
) -> Option<()> {
    let definition = latex::GraphicsPath::cast(node)?;
    for path in definition
        .path_list()
        .filter_map(|group| group.key())
        .map(|path| path.to_string())
    {
        context.extras.graphics_paths.insert(path);
    }

    Some(())
}
