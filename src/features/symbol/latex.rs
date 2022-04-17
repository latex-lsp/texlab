use std::str::FromStr;

use lsp_types::Range;
use rowan::ast::AstNode;
use smol_str::SmolStr;
use titlecase::titlecase;

use crate::{
    find_caption_by_parent, find_label_number,
    syntax::latex::{self, HasBrack, HasCurly},
    LabelledFloatKind, LatexDocumentData, LineIndexExt, WorkspaceSubset, LANGUAGE_DATA,
};

use super::types::{InternalSymbol, InternalSymbolKind};

pub fn find_latex_symbols(subset: &WorkspaceSubset, buf: &mut Vec<InternalSymbol>) -> Option<()> {
    let main_document = subset
        .documents
        .first()
        .filter(|document| document.uri.as_str().ends_with(".tex"))?;

    let data = main_document.data.as_latex()?;
    let mut context = Context { subset, data };

    let root = context.data.green.clone();
    let mut symbols = visit(&mut context, latex::SyntaxNode::new_root(root));
    buf.append(&mut symbols);
    Some(())
}

struct Context<'a> {
    subset: &'a WorkspaceSubset,
    data: &'a LatexDocumentData,
}

fn visit(context: &mut Context, node: latex::SyntaxNode) -> Vec<InternalSymbol> {
    let symbol = match node.kind() {
        latex::PART
        | latex::CHAPTER
        | latex::SECTION
        | latex::SUBSECTION
        | latex::SUBSUBSECTION
        | latex::PARAGRAPH
        | latex::SUBPARAGRAPH => visit_section(context, node.clone()),
        latex::ENUM_ITEM => visit_enum_item(context, node.clone()),
        latex::EQUATION => visit_equation(context, node.clone()),
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
                    visit_equation_environment(context, node.clone())
                } else if LANGUAGE_DATA
                    .enum_environments
                    .iter()
                    .any(|env| env == &name)
                {
                    visit_enumeration(context, node.clone(), &name)
                } else if let Ok(float_kind) = LabelledFloatKind::from_str(&name) {
                    visit_float(context, node.clone(), float_kind)
                } else {
                    visit_theorem(context, node.clone(), &name)
                }
            }),
        _ => None,
    };

    match symbol {
        Some(mut parent) => {
            for child in node.children() {
                parent.children.append(&mut visit(context, child));
            }
            vec![parent]
        }
        None => {
            let mut symbols = Vec::new();
            for child in node.children() {
                symbols.append(&mut visit(context, child));
            }
            symbols
        }
    }
}

fn visit_section(context: &mut Context, node: latex::SyntaxNode) -> Option<InternalSymbol> {
    let section = latex::Section::cast(node)?;
    let full_range = context
        .subset
        .documents
        .first()?
        .line_index
        .line_col_lsp_range(latex::small_range(&section));

    let group = section.name()?;
    let group_text = group.content_text()?;

    let symbol = match find_label_by_parent(context, section.syntax()) {
        Some(NumberedLabel {
            name: label,
            range: selection_range,
            number,
        }) => {
            let name = match number {
                Some(number) => format!("{} {}", number, group_text),
                None => group_text.to_string(),
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
            name: group_text.to_string(),
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

fn visit_enum_item(context: &mut Context, node: latex::SyntaxNode) -> Option<InternalSymbol> {
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

    let full_range = context
        .subset
        .documents
        .first()?
        .line_index
        .line_col_lsp_range(latex::small_range(&enum_item));

    let name = enum_item
        .label()
        .and_then(|label| label.content_text())
        .unwrap_or_else(|| "Item".to_string());

    let symbol = match find_label_by_parent(context, &node) {
        Some(NumberedLabel {
            name: label,
            range: selection_range,
            number,
        }) => InternalSymbol {
            name: number.map(Into::into).unwrap_or_else(|| name.to_string()),
            label: Some(label),
            kind: InternalSymbolKind::EnumerationItem,
            deprecated: false,
            full_range,
            selection_range,
            children: Vec::new(),
        },
        None => InternalSymbol {
            name: name.to_string(),
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

fn visit_equation(context: &mut Context, node: latex::SyntaxNode) -> Option<InternalSymbol> {
    let equation = latex::Equation::cast(node)?;

    let full_range = context
        .subset
        .documents
        .first()?
        .line_index
        .line_col_lsp_range(latex::small_range(&equation));

    make_equation_symbol(context, equation.syntax(), full_range)
}

fn visit_equation_environment(
    context: &mut Context,
    node: latex::SyntaxNode,
) -> Option<InternalSymbol> {
    let environment = latex::Environment::cast(node)?;

    let full_range = context
        .subset
        .documents
        .first()?
        .line_index
        .line_col_lsp_range(latex::small_range(&environment));

    make_equation_symbol(context, environment.syntax(), full_range)
}

fn make_equation_symbol(
    context: &mut Context,
    node: &latex::SyntaxNode,
    full_range: Range,
) -> Option<InternalSymbol> {
    let symbol = match find_label_by_parent(context, node) {
        Some(NumberedLabel {
            name: label,
            range: selection_range,
            number,
        }) => {
            let name = match number {
                Some(number) => format!("Equation ({})", number),
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
    context: &mut Context,
    node: latex::SyntaxNode,
    env_name: &str,
) -> Option<InternalSymbol> {
    let environment = latex::Environment::cast(node)?;
    let full_range = context
        .subset
        .documents
        .first()?
        .line_index
        .line_col_lsp_range(latex::small_range(&environment));

    let name = titlecase(env_name);
    let symbol = match find_label_by_parent(context, environment.syntax()) {
        Some(NumberedLabel {
            name: label,
            range: selection_range,
            number,
        }) => {
            let name = match number {
                Some(number) => format!("{} {}", name, number),
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
    context: &mut Context,
    node: latex::SyntaxNode,
    float_kind: LabelledFloatKind,
) -> Option<InternalSymbol> {
    let environment = latex::Environment::cast(node)?;
    let full_range = context
        .subset
        .documents
        .first()?
        .line_index
        .line_col_lsp_range(latex::small_range(&environment));

    let (float_kind, symbol_kind) = match float_kind {
        LabelledFloatKind::Algorithm => ("Algorithm", InternalSymbolKind::Algorithm),
        LabelledFloatKind::Figure => ("Figure", InternalSymbolKind::Figure),
        LabelledFloatKind::Listing => ("Listing", InternalSymbolKind::Listing),
        LabelledFloatKind::Table => ("Table", InternalSymbolKind::Table),
    };

    let caption = find_caption_by_parent(environment.syntax())?;
    let symbol = match find_label_by_parent(context, environment.syntax()) {
        Some(NumberedLabel {
            name: label,
            range: selection_range,
            number,
        }) => {
            let name = match number {
                Some(number) => format!("{} {}: {}", float_kind, number, caption),
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
    context: &mut Context,
    node: latex::SyntaxNode,
    environment_name: &str,
) -> Option<InternalSymbol> {
    let definition = context
        .subset
        .documents
        .iter()
        .filter_map(|document| document.data.as_latex())
        .find_map(|data| {
            data.extras
                .theorem_environments
                .iter()
                .find(|environment| environment.name == environment_name)
                .cloned()
        })?;

    let node = latex::Environment::cast(node)?;
    let theorem_description = node
        .begin()?
        .options()
        .and_then(|option| option.content_text());

    let full_range = context
        .subset
        .documents
        .first()?
        .line_index
        .line_col_lsp_range(latex::small_range(&node));

    let symbol = match find_label_by_parent(context, node.syntax()) {
        Some(NumberedLabel {
            name: label,
            range: selection_range,
            number,
        }) => {
            let name = match (number, theorem_description) {
                (Some(number), Some(desc)) => {
                    format!("{} {} ({})", definition.description, number, desc)
                }
                (Some(number), None) => format!("{} {}", definition.description, number),
                (None, Some(desc)) => format!("{} ({})", definition.description, desc),
                (None, None) => definition.description.clone(),
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
                Some(desc) => format!("{} ({})", definition.description, desc),
                None => definition.description.to_string(),
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
    name: String,
    range: Range,
    number: Option<SmolStr>,
}

fn find_label_by_parent(
    context: &mut Context,
    parent: &latex::SyntaxNode,
) -> Option<NumberedLabel> {
    let node = parent.children().find_map(latex::LabelDefinition::cast)?;

    let name = node.name()?.key()?.to_string();
    let range = context
        .subset
        .documents
        .first()?
        .line_index
        .line_col_lsp_range(latex::small_range(&node));

    let number = find_label_number(&context.subset, &name);
    Some(NumberedLabel {
        name: name.to_string(),
        range,
        number: number.map(Into::into),
    })
}
