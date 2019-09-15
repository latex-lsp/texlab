use super::{label_name, selection_range};
use crate::symbol::{LatexSymbol, LatexSymbolKind};
use crate::syntax::*;
use crate::workspace::*;
use lsp_types::Range;

pub fn symbols(view: &DocumentView, tree: &LatexSyntaxTree) -> Vec<LatexSymbol> {
    let mut symbols = Vec::new();
    for environment in &tree.environments {
        if environment.left.is_enum() {
            symbols.push(make_symbol(view, tree, environment));
        }
    }
    symbols
}

fn make_symbol(
    view: &DocumentView,
    tree: &LatexSyntaxTree,
    enumeration: &LatexEnvironment,
) -> LatexSymbol {
    let name = titlelize(enumeration.left.name().unwrap().text());

    let items: Vec<_> = tree
        .items
        .iter()
        .filter(|item| tree.is_enumeration_item(enumeration, item))
        .collect();

    let mut children = Vec::new();
    for i in 0..items.len() {
        let start = items[i].start();
        let end = items
            .get(i + 1)
            .map(|item| item.start())
            .unwrap_or_else(|| enumeration.right.start());
        let range = Range::new(start, end);

        let label = tree.find_label_definition(range);
        let number = items[i].name().or_else(|| {
            label
                .as_ref()
                .and_then(|label| OutlineContext::find_number(view, label))
        });

        let name = number.unwrap_or_else(|| "Item".into());
        children.push(LatexSymbol {
            name,
            label: label_name(label.clone()),
            kind: LatexSymbolKind::EnumerationItem,
            deprecated: false,
            full_range: range,
            selection_range: selection_range(items[i].range(), label),
            children: Vec::new(),
        });
    }

    LatexSymbol {
        name,
        label: None,
        kind: LatexSymbolKind::Enumeration,
        deprecated: false,
        full_range: enumeration.range(),
        selection_range: enumeration.range(),
        children,
    }
}
