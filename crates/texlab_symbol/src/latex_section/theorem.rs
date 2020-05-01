use super::{label_name, selection_range};
use crate::types::{LatexSymbol, LatexSymbolKind};
use texlab_feature::{DocumentContent, DocumentView, OutlineContext};
use texlab_syntax::latex;
use titlecase::titlecase;

pub fn symbols(view: &DocumentView, table: &latex::SymbolTable) -> Vec<LatexSymbol> {
    table
        .environments
        .iter()
        .filter_map(|env| make_symbol(view, table, *env))
        .collect()
}

fn make_symbol(
    view: &DocumentView,
    main_table: &latex::SymbolTable,
    env: latex::Environment,
) -> Option<LatexSymbol> {
    let env_name = env.left.name(&main_table).map(latex::Token::text)?;

    for document in &view.related {
        if let DocumentContent::Latex(table) = &document.content {
            for definition in &table.theorem_definitions {
                if definition.name(&table).text() == env_name {
                    let kind = table
                        .print_group_content(
                            definition.parent,
                            latex::GroupKind::Group,
                            definition.arg_index + 1,
                        )
                        .unwrap_or_else(|| titlecase(env_name));

                    let desc = main_table.print_group_content(
                        env.left.parent,
                        latex::GroupKind::Options,
                        0,
                    );

                    let label = main_table.find_label_by_environment(env);
                    let number = label
                        .and_then(|label| OutlineContext::find_number(view, &main_table, *label));

                    let name = match (desc, number) {
                        (Some(desc), Some(num)) => format!("{} {} ({})", kind, num, desc),
                        (Some(desc), None) => format!("{} ({})", kind, desc),
                        (None, Some(num)) => format!("{} {}", kind, num),
                        (None, None) => kind,
                    };

                    let symbol = LatexSymbol {
                        name,
                        label: label_name(main_table, label),
                        kind: LatexSymbolKind::Theorem,
                        deprecated: false,
                        full_range: env.range(&main_table),
                        selection_range: selection_range(main_table, env.range(&main_table), label),
                        children: Vec::new(),
                    };
                    return Some(symbol);
                }
            }
        }
    }
    None
}
