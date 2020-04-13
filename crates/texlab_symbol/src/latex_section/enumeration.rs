use super::{label_name, selection_range};
use crate::types::{LatexSymbol, LatexSymbolKind};
use texlab_feature::{DocumentView, OutlineContext};
use texlab_protocol::{Range, RangeExt};
use texlab_syntax::{latex, SyntaxNode};
use titlecase::titlecase;

pub fn symbols(view: &DocumentView, table: &latex::SymbolTable) -> Vec<LatexSymbol> {
    table
        .environments
        .iter()
        .filter(|env| env.left.is_enum(&table))
        .map(|enumeration| make_symbol(view, table, *enumeration))
        .collect()
}

fn make_symbol(
    view: &DocumentView,
    table: &latex::SymbolTable,
    enumeration: latex::Environment,
) -> LatexSymbol {
    let name = titlecase(enumeration.left.name(&table).unwrap().text());

    let items: Vec<_> = table
        .items
        .iter()
        .filter(|item| table.is_enum_item(enumeration, **item))
        .collect();

    let mut children = Vec::new();
    for i in 0..items.len() {
        let start = table[items[i].parent].start();
        let end = items
            .get(i + 1)
            .map(|item| table[item.parent].start())
            .unwrap_or_else(|| table[enumeration.right.parent].start());
        let range = Range::new(start, end);

        let label = find_item_label(table, range);

        let number = items[i]
            .name(&table)
            .or_else(|| label.and_then(|label| OutlineContext::find_number(view, table, *label)));

        let name = number.unwrap_or_else(|| "Item".into());
        children.push(LatexSymbol {
            name,
            label: label_name(table, label),
            kind: LatexSymbolKind::EnumerationItem,
            deprecated: false,
            full_range: range,
            selection_range: selection_range(table, table[items[i].parent].range(), label),
            children: Vec::new(),
        });
    }

    LatexSymbol {
        name,
        label: None,
        kind: LatexSymbolKind::Enumeration,
        deprecated: false,
        full_range: enumeration.range(&table),
        selection_range: enumeration.range(&table),
        children,
    }
}

fn find_item_label(table: &latex::SymbolTable, item_range: Range) -> Option<&latex::Label> {
    table.find_label_by_range(item_range).filter(|label| {
        table
            .environments
            .iter()
            .filter(|env| item_range.contains(env.range(&table).start))
            .all(|env| !env.range(&table).contains(table[label.parent].start()))
    })
}
