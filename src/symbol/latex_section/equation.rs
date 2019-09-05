use super::{label_name, selection_range};
use crate::symbol::{LatexSymbol, LatexSymbolKind};
use crate::syntax::*;
use crate::workspace::*;
use lsp_types::Range;

pub fn symbols(view: &DocumentView, tree: &LatexSyntaxTree) -> Vec<LatexSymbol> {
    let mut symbols = Vec::new();
    for equation in &tree.equations {
        symbols.push(make_symbol(view, tree, equation.range()));
    }

    for equation in &tree.environments {
        if equation.left.is_math() {
            symbols.push(make_symbol(view, tree, equation.range()));
        }
    }
    symbols.into_iter().filter_map(|sym| sym).collect()
}

fn make_symbol(
    view: &DocumentView,
    tree: &LatexSyntaxTree,
    full_range: Range,
) -> Option<LatexSymbol> {
    let label = tree.find_label_definition(full_range);

    let name = match label
        .as_ref()
        .and_then(|label| OutlineContext::find_number(view, label))
    {
        Some(num) => format!("Equation ({})", num),
        None => "Equation".to_owned(),
    };

    let symbol = LatexSymbol {
        name,
        label: label_name(label.clone()),
        kind: LatexSymbolKind::Equation,
        deprecated: false,
        full_range,
        selection_range: selection_range(full_range, label),
        children: Vec::new(),
    };
    Some(symbol)
}
