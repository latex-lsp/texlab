use crate::symbol::{LatexSymbol, LatexSymbolKind};
use crate::syntax::*;

pub fn symbols(tree: &LatexSyntaxTree) -> Vec<LatexSymbol> {
    let mut symbols = Vec::new();
    for environment in &tree.environments {
        if environment.left.is_enum() {
            symbols.push(make_symbol(environment));
        }
    }
    symbols
}

fn make_symbol(environment: &LatexEnvironment) -> LatexSymbol {
    let name = titlelize(environment.left.name().unwrap().text());
    LatexSymbol {
        name,
        label: None,
        kind: LatexSymbolKind::Enumeration,
        deprecated: false,
        full_range: environment.range(),
        selection_range: environment.range(),
        children: Vec::new(),
    }
}
