use std::str::FromStr;

use lsp_types::Range;
use rowan::ast::AstNode;
use titlecase::titlecase;

use crate::{
    db::{Document, Word, Workspace},
    syntax::latex::{self, HasBrack, HasCurly},
    util::{
        label::{find_caption_by_parent, LabeledFloatKind},
        lang_data::LANGUAGE_DATA,
        line_index_ext::LineIndexExt,
    },
    Db,
};

use super::types::{InternalSymbol, InternalSymbolKind};

pub fn find_symbols(db: &dyn Db, document: Document, buf: &mut Vec<InternalSymbol>) -> Option<()> {
    let data = document.parse(db).as_tex()?;
    let mut symbols = visit(db, document, data.root(db));
    buf.append(&mut symbols);
    Some(())
}

fn visit(db: &dyn Db, document: Document, node: latex::SyntaxNode) -> Vec<InternalSymbol> {
    let symbol = match node.kind() {
        latex::PART
        | latex::CHAPTER
        | latex::SECTION
        | latex::SUBSECTION
        | latex::SUBSUBSECTION
        | latex::PARAGRAPH
        | latex::SUBPARAGRAPH => visit_section(db, document, node.clone()),
        latex::ENUM_ITEM => visit_enum_item(db, document, node.clone()),
        latex::EQUATION => visit_equation(db, document, node.clone()),
        latex::ENVIRONMENT => latex::Environment::cast(node.clone())
            .and_then(|env| env.begin())
            .and_then(|begin| begin.name())
            .and_then(|name| name.key())
            .map(|name| name.to_string())
            .and_then(|name| {
                if LANGUAGE_DATA
                    .math_environments
                    .iter()
                    .any(|env| env == &name)
                {
                    visit_equation_environment(db, document, node.clone())
                } else if LANGUAGE_DATA
                    .enum_environments
                    .iter()
                    .any(|env| env == &name)
                {
                    visit_enumeration(db, document, node.clone(), &name)
                } else if let Ok(float_kind) = LabeledFloatKind::from_str(&name) {
                    visit_float(db, document, node.clone(), float_kind)
                } else {
                    visit_theorem(db, document, node.clone(), &name)
                }
            }),
        _ => None,
    };

    match symbol {
        Some(mut parent) => {
            for child in node.children() {
                parent.children.append(&mut visit(db, document, child));
            }
            vec![parent]
        }
        None => {
            let mut symbols = Vec::new();
            for child in node.children() {
                symbols.append(&mut visit(db, document, child));
            }
            symbols
        }
    }
}

fn visit_section(
    db: &dyn Db,
    document: Document,
    node: latex::SyntaxNode,
) -> Option<InternalSymbol> {
    let section = latex::Section::cast(node)?;
    let full_range = document
        .contents(db)
        .line_index(db)
        .line_col_lsp_range(latex::small_range(&section));

    let group = section.name()?;
    let group_text = group.content_text()?;

    let symbol = match find_label_by_parent(db, document, section.syntax()) {
        Some(NumberedLabel {
            name: label,
            range: selection_range,
            number,
        }) => {
            let name = match number {
                Some(number) => format!("{} {}", number.text(db), group_text),
                None => group_text,
            };

            InternalSymbol {
                name,
                label: Some(label),
                kind: InternalSymbolKind::Section,
                deprecated: false,
                full_range,
                selection_range,
                children: Vec::new(),
            }
        }
        None => InternalSymbol {
            name: group_text,
            label: None,
            kind: InternalSymbolKind::Section,
            deprecated: false,
            full_range,
            selection_range: full_range,
            children: Vec::new(),
        },
    };

    Some(symbol)
}

fn visit_enum_item(
    db: &dyn Db,
    document: Document,
    node: latex::SyntaxNode,
) -> Option<InternalSymbol> {
    let enum_item = latex::EnumItem::cast(node.clone())?;
    if !enum_item
        .syntax()
        .ancestors()
        .filter_map(latex::Environment::cast)
        .filter_map(|environment| environment.begin())
        .filter_map(|begin| begin.name())
        .filter_map(|name| name.key())
        .any(|name| {
            LANGUAGE_DATA
                .enum_environments
                .iter()
                .any(|e| e == &name.to_string())
        })
    {
        return None;
    }

    let full_range = document
        .contents(db)
        .line_index(db)
        .line_col_lsp_range(latex::small_range(&enum_item));

    let name = enum_item
        .label()
        .and_then(|label| label.content_text())
        .unwrap_or_else(|| "Item".to_string());

    let symbol = match find_label_by_parent(db, document, &node) {
        Some(NumberedLabel {
            name: label,
            range: selection_range,
            number,
        }) => InternalSymbol {
            name: number
                .map(|num| num.text(db).clone())
                .unwrap_or_else(|| name.clone()),
            label: Some(label),
            kind: InternalSymbolKind::EnumerationItem,
            deprecated: false,
            full_range,
            selection_range,
            children: Vec::new(),
        },
        None => InternalSymbol {
            name,
            label: None,
            kind: InternalSymbolKind::EnumerationItem,
            deprecated: false,
            full_range,
            selection_range: full_range,
            children: Vec::new(),
        },
    };
    Some(symbol)
}

fn visit_equation(
    db: &dyn Db,
    document: Document,
    node: latex::SyntaxNode,
) -> Option<InternalSymbol> {
    let equation = latex::Equation::cast(node)?;

    let full_range = document
        .contents(db)
        .line_index(db)
        .line_col_lsp_range(latex::small_range(&equation));

    make_equation_symbol(db, document, equation.syntax(), full_range)
}

fn visit_equation_environment(
    db: &dyn Db,
    document: Document,
    node: latex::SyntaxNode,
) -> Option<InternalSymbol> {
    let environment = latex::Environment::cast(node)?;

    let full_range = document
        .contents(db)
        .line_index(db)
        .line_col_lsp_range(latex::small_range(&environment));

    make_equation_symbol(db, document, environment.syntax(), full_range)
}

fn make_equation_symbol(
    db: &dyn Db,
    document: Document,
    node: &latex::SyntaxNode,
    full_range: Range,
) -> Option<InternalSymbol> {
    let symbol = match find_label_by_parent(db, document, node) {
        Some(NumberedLabel {
            name: label,
            range: selection_range,
            number,
        }) => {
            let name = match number {
                Some(number) => format!("Equation ({})", number.text(db)),
                None => "Equation".to_string(),
            };

            InternalSymbol {
                name,
                label: Some(label),
                kind: InternalSymbolKind::Equation,
                deprecated: false,
                full_range,
                selection_range,
                children: Vec::new(),
            }
        }
        None => InternalSymbol {
            name: "Equation".to_string(),
            label: None,
            kind: InternalSymbolKind::Equation,
            deprecated: false,
            full_range,
            selection_range: full_range,
            children: Vec::new(),
        },
    };
    Some(symbol)
}

fn visit_enumeration(
    db: &dyn Db,
    document: Document,
    node: latex::SyntaxNode,
    env_name: &str,
) -> Option<InternalSymbol> {
    let environment = latex::Environment::cast(node)?;
    let full_range = document
        .contents(db)
        .line_index(db)
        .line_col_lsp_range(latex::small_range(&environment));

    let name = titlecase(env_name);
    let symbol = match find_label_by_parent(db, document, environment.syntax()) {
        Some(NumberedLabel {
            name: label,
            range: selection_range,
            number,
        }) => {
            let name = match number {
                Some(number) => format!("{} {}", name, number.text(db)),
                None => name,
            };

            InternalSymbol {
                name,
                label: Some(label),
                kind: InternalSymbolKind::Enumeration,
                deprecated: false,
                full_range,
                selection_range,
                children: Vec::new(),
            }
        }
        None => InternalSymbol {
            name,
            label: None,
            kind: InternalSymbolKind::Enumeration,
            deprecated: false,
            full_range,
            selection_range: full_range,
            children: Vec::new(),
        },
    };
    Some(symbol)
}

fn visit_float(
    db: &dyn Db,
    document: Document,
    node: latex::SyntaxNode,
    float_kind: LabeledFloatKind,
) -> Option<InternalSymbol> {
    let environment = latex::Environment::cast(node)?;
    let full_range = document
        .contents(db)
        .line_index(db)
        .line_col_lsp_range(latex::small_range(&environment));

    let (float_kind, symbol_kind) = match float_kind {
        LabeledFloatKind::Algorithm => ("Algorithm", InternalSymbolKind::Algorithm),
        LabeledFloatKind::Figure => ("Figure", InternalSymbolKind::Figure),
        LabeledFloatKind::Listing => ("Listing", InternalSymbolKind::Listing),
        LabeledFloatKind::Table => ("Table", InternalSymbolKind::Table),
    };

    let caption = find_caption_by_parent(environment.syntax())?;
    let symbol = match find_label_by_parent(db, document, environment.syntax()) {
        Some(NumberedLabel {
            name: label,
            range: selection_range,
            number,
        }) => {
            let name = match number {
                Some(number) => format!("{} {}: {}", float_kind, number.text(db), caption),
                None => format!("{}: {}", float_kind, caption),
            };

            InternalSymbol {
                name,
                label: Some(label),
                kind: symbol_kind,
                deprecated: false,
                full_range,
                selection_range,
                children: Vec::new(),
            }
        }
        None => InternalSymbol {
            name: format!("{}: {}", float_kind, caption),
            label: None,
            kind: symbol_kind,
            deprecated: false,
            full_range,
            selection_range: full_range,
            children: Vec::new(),
        },
    };

    Some(symbol)
}

fn visit_theorem(
    db: &dyn Db,
    document: Document,
    node: latex::SyntaxNode,
    environment_name: &str,
) -> Option<InternalSymbol> {
    let definition = Workspace::get(db)
        .related(db, document)
        .iter()
        .filter_map(|document| document.parse(db).as_tex())
        .flat_map(|data| data.analyze(db).theorem_environments(db))
        .find(|env| env.name(db).text(db) == environment_name)?;

    let node = latex::Environment::cast(node)?;
    let theorem_description = node
        .begin()?
        .options()
        .and_then(|option| option.content_text());

    let full_range = document
        .contents(db)
        .line_index(db)
        .line_col_lsp_range(latex::small_range(&node));

    let symbol = match find_label_by_parent(db, document, node.syntax()) {
        Some(NumberedLabel {
            name: label,
            range: selection_range,
            number,
        }) => {
            let name = match (number, theorem_description) {
                (Some(number), Some(desc)) => {
                    format!(
                        "{} {} ({})",
                        definition.description(db).text(db),
                        number.text(db),
                        desc
                    )
                }
                (Some(number), None) => format!(
                    "{} {}",
                    definition.description(db).text(db),
                    number.text(db)
                ),
                (None, Some(desc)) => format!("{} ({})", definition.description(db).text(db), desc),
                (None, None) => definition.description(db).text(db).clone(),
            };

            InternalSymbol {
                name,
                label: Some(label),
                kind: InternalSymbolKind::Theorem,
                deprecated: false,
                full_range,
                selection_range,
                children: Vec::new(),
            }
        }
        None => {
            let name = match theorem_description {
                Some(desc) => format!("{} ({})", definition.description(db).text(db), desc),
                None => definition.description(db).text(db).clone(),
            };
            InternalSymbol {
                name,
                label: None,
                kind: InternalSymbolKind::Theorem,
                deprecated: false,
                full_range,
                selection_range: full_range,
                children: Vec::new(),
            }
        }
    };
    Some(symbol)
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct NumberedLabel {
    name: Word,
    range: Range,
    number: Option<Word>,
}

fn find_label_by_parent(
    db: &dyn Db,
    document: Document,
    parent: &latex::SyntaxNode,
) -> Option<NumberedLabel> {
    let node = parent.children().find_map(latex::LabelDefinition::cast)?;
    let name = Word::new(db, node.name()?.key()?.to_string());
    let range = document
        .contents(db)
        .line_index(db)
        .line_col_lsp_range(latex::small_range(&node));

    let number = Workspace::get(db).number_of_label(db, document, name);
    Some(NumberedLabel {
        name,
        range,
        number,
    })
}
