use super::{label_name, selection_range};
use crate::symbol::{LatexSymbol, LatexSymbolKind};
use texlab_workspace::*;
use texlab_protocol::Range;
use texlab_syntax::*;

pub fn symbols(view: &DocumentView, tree: &LatexSyntaxTree) -> Vec<LatexSymbol> {
    let mut symbols = Vec::new();
    for equation in &tree.math.equations {
        symbols.push(make_symbol(view, tree, equation.range()));
    }

    for equation in &tree.env.environments {
        if equation.left.is_math() {
            symbols.push(make_symbol(view, tree, equation.range()));
        }
    }
    symbols
}

fn make_symbol(view: &DocumentView, tree: &LatexSyntaxTree, full_range: Range) -> LatexSymbol {
    let label = tree.find_label_by_range(full_range);

    let name = match label
        .as_ref()
        .and_then(|label| OutlineContext::find_number(view, label))
    {
        Some(num) => format!("Equation ({})", num),
        None => "Equation".to_owned(),
    };

    LatexSymbol {
        name,
        label: label_name(label),
        kind: LatexSymbolKind::Equation,
        deprecated: false,
        full_range,
        selection_range: selection_range(full_range, label),
        children: Vec::new(),
    }
}
