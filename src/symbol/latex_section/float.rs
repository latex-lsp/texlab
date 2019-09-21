use super::{label_name, selection_range};
use crate::symbol::{LatexSymbol, LatexSymbolKind};
use crate::syntax::*;
use crate::workspace::*;

pub fn symbols(view: &DocumentView, tree: &LatexSyntaxTree) -> Vec<LatexSymbol> {
    tree.captions
        .iter()
        .filter_map(|caption| make_symbol(view, tree, caption))
        .collect()
}

fn make_symbol(
    view: &DocumentView,
    tree: &LatexSyntaxTree,
    caption: &LatexCaption,
) -> Option<LatexSymbol> {
    let environment = tree
        .environments
        .iter()
        .find(|env| tree.is_direct_child(env, caption.start()))?;
    let text = extract_group(&caption.command.args[caption.index]);

    let kind = environment
        .left
        .name()
        .map(LatexToken::text)
        .and_then(OutlineCaptionKind::parse)?;

    let label = tree.find_label_by_environment(environment);
    let number = label
        .as_ref()
        .and_then(|label| OutlineContext::find_number(view, label));

    let name = match &number {
        Some(number) => format!("{} {}: {}", kind.as_str(), number, text),
        None => format!("{}: {}", kind.as_str(), text),
    };

    let symbol = LatexSymbol {
        name,
        label: label_name(label.clone()),
        kind: match kind {
            OutlineCaptionKind::Figure => LatexSymbolKind::Figure,
            OutlineCaptionKind::Table => LatexSymbolKind::Table,
            OutlineCaptionKind::Listing => LatexSymbolKind::Listing,
            OutlineCaptionKind::Algorithm => LatexSymbolKind::Algorithm,
        },
        deprecated: false,
        full_range: environment.range(),
        selection_range: selection_range(environment.range(), label),
        children: Vec::new(),
    };
    Some(symbol)
}
