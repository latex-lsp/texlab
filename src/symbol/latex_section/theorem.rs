use super::{label_name, selection_range};
use crate::symbol::{LatexSymbol, LatexSymbolKind};
use texlab_workspace::*;
use texlab_syntax::*;

pub fn symbols(view: &DocumentView, tree: &LatexSyntaxTree) -> Vec<LatexSymbol> {
    tree.env
        .environments
        .iter()
        .filter_map(|env| make_symbol(view, tree, env))
        .collect()
}

fn make_symbol(
    view: &DocumentView,
    main_tree: &LatexSyntaxTree,
    environment: &LatexEnvironment,
) -> Option<LatexSymbol> {
    let environment_name = environment.left.name().map(LatexToken::text)?;

    for document in &view.related_documents {
        if let SyntaxTree::Latex(tree) = &document.tree {
            for definition in &tree.math.theorem_definitions {
                if environment_name == definition.name().text() {
                    let kind = definition
                        .command
                        .args
                        .get(definition.index + 1)
                        .map(|content| extract_group(content))
                        .unwrap_or_else(|| titlelize(environment_name));

                    let description = environment
                        .left
                        .command
                        .options
                        .get(0)
                        .map(|content| extract_group(content));

                    let label = main_tree.find_label_by_environment(environment);
                    let number = label
                        .as_ref()
                        .and_then(|label| OutlineContext::find_number(view, label));

                    let name = match (description, number) {
                        (Some(desc), Some(num)) => format!("{} {} ({})", kind, num, desc),
                        (Some(desc), None) => format!("{} ({})", kind, desc),
                        (None, Some(num)) => format!("{} {}", kind, num),
                        (None, None) => kind,
                    };

                    let symbol = LatexSymbol {
                        name,
                        label: label_name(label),
                        kind: LatexSymbolKind::Theorem,
                        deprecated: false,
                        full_range: environment.range(),
                        selection_range: selection_range(environment.range(), label),
                        children: Vec::new(),
                    };
                    return Some(symbol);
                }
            }
        }
    }
    None
}
