use super::{label_name, selection_range};
use crate::{
    feature::DocumentView,
    outline::{OutlineCaptionKind, OutlineContext},
    symbol::types::{LatexSymbol, LatexSymbolKind},
    syntax::latex,
};

pub fn symbols(view: &DocumentView, table: &latex::SymbolTable) -> Vec<LatexSymbol> {
    table
        .captions
        .iter()
        .filter_map(|caption| make_symbol(view, table, *caption))
        .collect()
}

fn make_symbol(
    view: &DocumentView,
    table: &latex::SymbolTable,
    caption: latex::Caption,
) -> Option<LatexSymbol> {
    let env = table
        .environments
        .iter()
        .find(|env| table.is_direct_child(env, table.tree.range(caption.parent).start))?;

    let text = table.tree.print_group_content(
        caption.parent,
        latex::GroupKind::Group,
        caption.arg_index,
    )?;

    let kind = env
        .left
        .name(&table.tree)
        .map(latex::Token::text)
        .and_then(OutlineCaptionKind::parse)?;

    let label = table.find_label_by_environment(env);
    let number = label
        .as_ref()
        .and_then(|label| OutlineContext::find_number(view, table, label));

    let name = match &number {
        Some(number) => format!("{} {}: {}", kind.as_str(), number, text),
        None => format!("{}: {}", kind.as_str(), text),
    };

    let symbol = LatexSymbol {
        name,
        label: label_name(table, label),
        kind: match kind {
            OutlineCaptionKind::Figure => LatexSymbolKind::Figure,
            OutlineCaptionKind::Table => LatexSymbolKind::Table,
            OutlineCaptionKind::Listing => LatexSymbolKind::Listing,
            OutlineCaptionKind::Algorithm => LatexSymbolKind::Algorithm,
        },
        deprecated: false,
        full_range: env.range(&table.tree),
        selection_range: selection_range(table, env.range(&table.tree), label),
        children: Vec::new(),
    };
    Some(symbol)
}
