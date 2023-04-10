use std::str::FromStr;

use base_db::{semantics::Span, Document, DocumentData, Project, Workspace};
use lsp_types::Range;
use rowan::{ast::AstNode, TextRange};
use syntax::latex::{self, HasBrack, HasCurly};
use titlecase::titlecase;

use crate::util::{
    label::{find_caption_by_parent, LabeledFloatKind},
    line_index_ext::LineIndexExt,
};

use super::types::{InternalSymbol, InternalSymbolKind};

pub fn find_symbols(
    workspace: &Workspace,
    project: &Project,
    document: &Document,
    buf: &mut Vec<InternalSymbol>,
) {
    let DocumentData::Tex(data) = &document.data else { return };

    let mut symbols = visit(workspace, project, document, data.root_node());
    buf.append(&mut symbols);
}

fn visit(
    workspace: &Workspace,
    project: &Project,
    document: &Document,
    node: latex::SyntaxNode,
) -> Vec<InternalSymbol> {
    let symbol = match node.kind() {
        latex::PART
        | latex::CHAPTER
        | latex::SECTION
        | latex::SUBSECTION
        | latex::SUBSUBSECTION
        | latex::PARAGRAPH
        | latex::SUBPARAGRAPH => visit_section(project, document, node.clone()),
        latex::ENUM_ITEM => visit_enum_item(workspace, project, document, node.clone()),
        latex::EQUATION => visit_equation(project, document, node.clone()),
        latex::ENVIRONMENT => latex::Environment::cast(node.clone())
            .and_then(|env| env.begin())
            .and_then(|begin| begin.name())
            .and_then(|name| name.key())
            .map(|name| name.to_string())
            .and_then(|name| {
                let config = &workspace.config().syntax;

                if config.math_environments.contains(&name) {
                    visit_equation_environment(project, document, node.clone())
                } else if config.enum_environments.contains(&name) {
                    visit_enumeration(project, document, node.clone(), &name)
                } else if let Ok(float_kind) = LabeledFloatKind::from_str(&name) {
                    visit_float(project, document, node.clone(), float_kind)
                } else {
                    visit_theorem(project, document, node.clone(), &name)
                }
            }),
        _ => None,
    };

    match symbol {
        Some(mut parent) => {
            for child in node.children() {
                parent
                    .children
                    .append(&mut visit(workspace, project, document, child));
            }

            vec![parent]
        }
        None => {
            let mut symbols = Vec::new();
            for child in node.children() {
                symbols.append(&mut visit(workspace, project, document, child));
            }

            symbols
        }
    }
}

fn visit_section(
    project: &Project,
    document: &Document,
    node: latex::SyntaxNode,
) -> Option<InternalSymbol> {
    let section = latex::Section::cast(node)?;
    let full_range = document
        .line_index
        .line_col_lsp_range(latex::small_range(&section));

    let group = section.name()?;
    let group_text = group.content_text()?;

    let label = NumberedLabel::find(project, section.syntax());

    let symbol = match label {
        Some(label) => {
            let name = match label.number {
                Some(number) => format!("{} {}", number, group_text),
                None => group_text,
            };

            InternalSymbol {
                name,
                label: Some(label.name.text),
                kind: InternalSymbolKind::Section,
                deprecated: false,
                full_range,
                selection_range: document.line_index.line_col_lsp_range(label.range),
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
    workspace: &Workspace,
    project: &Project,
    document: &Document,
    node: latex::SyntaxNode,
) -> Option<InternalSymbol> {
    let enum_envs = &workspace.config().syntax.enum_environments;
    let enum_item = latex::EnumItem::cast(node.clone())?;
    if !enum_item
        .syntax()
        .ancestors()
        .filter_map(latex::Environment::cast)
        .filter_map(|environment| environment.begin())
        .filter_map(|begin| begin.name())
        .filter_map(|name| name.key())
        .any(|name| enum_envs.contains(&name.to_string()))
    {
        return None;
    }

    let full_range = document
        .line_index
        .line_col_lsp_range(latex::small_range(&enum_item));

    let name = enum_item
        .label()
        .and_then(|label| label.content_text())
        .unwrap_or_else(|| "Item".to_string());

    let symbol = match NumberedLabel::find(project, &node) {
        Some(label) => InternalSymbol {
            name: label.number.map_or_else(|| name.clone(), String::from),
            label: Some(label.name.text),
            kind: InternalSymbolKind::EnumerationItem,
            deprecated: false,
            full_range,
            selection_range: document.line_index.line_col_lsp_range(label.range),
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
    project: &Project,
    document: &Document,
    node: latex::SyntaxNode,
) -> Option<InternalSymbol> {
    let equation = latex::Equation::cast(node)?;

    let full_range = document
        .line_index
        .line_col_lsp_range(latex::small_range(&equation));

    make_equation_symbol(project, document, equation.syntax(), full_range)
}

fn visit_equation_environment(
    project: &Project,
    document: &Document,
    node: latex::SyntaxNode,
) -> Option<InternalSymbol> {
    let environment = latex::Environment::cast(node)?;

    let full_range = document
        .line_index
        .line_col_lsp_range(latex::small_range(&environment));

    make_equation_symbol(project, document, environment.syntax(), full_range)
}

fn make_equation_symbol(
    project: &Project,
    document: &Document,
    node: &latex::SyntaxNode,
    full_range: Range,
) -> Option<InternalSymbol> {
    let symbol = match NumberedLabel::find(project, node) {
        Some(label) => {
            let name = match label.number {
                Some(number) => format!("Equation ({})", number),
                None => "Equation".to_string(),
            };

            InternalSymbol {
                name,
                label: Some(label.name.text),
                kind: InternalSymbolKind::Equation,
                deprecated: false,
                full_range,
                selection_range: document.line_index.line_col_lsp_range(label.range),
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
    project: &Project,
    document: &Document,
    node: latex::SyntaxNode,
    env_name: &str,
) -> Option<InternalSymbol> {
    let environment = latex::Environment::cast(node)?;
    let full_range = document
        .line_index
        .line_col_lsp_range(latex::small_range(&environment));

    let name = titlecase(env_name);
    let symbol = match NumberedLabel::find(project, environment.syntax()) {
        Some(label) => {
            let name = match label.number {
                Some(number) => format!("{} {}", name, number),
                None => name,
            };

            InternalSymbol {
                name,
                label: Some(label.name.text),
                kind: InternalSymbolKind::Enumeration,
                deprecated: false,
                full_range,
                selection_range: document.line_index.line_col_lsp_range(label.range),
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
    project: &Project,
    document: &Document,
    node: latex::SyntaxNode,
    float_kind: LabeledFloatKind,
) -> Option<InternalSymbol> {
    let environment = latex::Environment::cast(node)?;
    let full_range = document
        .line_index
        .line_col_lsp_range(latex::small_range(&environment));

    let (float_kind, symbol_kind) = match float_kind {
        LabeledFloatKind::Algorithm => ("Algorithm", InternalSymbolKind::Algorithm),
        LabeledFloatKind::Figure => ("Figure", InternalSymbolKind::Figure),
        LabeledFloatKind::Listing => ("Listing", InternalSymbolKind::Listing),
        LabeledFloatKind::Table => ("Table", InternalSymbolKind::Table),
    };

    let caption = find_caption_by_parent(environment.syntax())?;
    let symbol = match NumberedLabel::find(project, environment.syntax()) {
        Some(label) => {
            let name = match label.number {
                Some(number) => format!("{} {}: {}", float_kind, number, caption),
                None => format!("{}: {}", float_kind, caption),
            };

            InternalSymbol {
                name,
                label: Some(label.name.text),
                kind: symbol_kind,
                deprecated: false,
                full_range,
                selection_range: document.line_index.line_col_lsp_range(label.range),
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
    project: &Project,
    document: &Document,
    node: latex::SyntaxNode,
    environment_name: &str,
) -> Option<InternalSymbol> {
    let definition = project
        .documents
        .iter()
        .filter_map(|document| document.data.as_tex())
        .flat_map(|data| data.semantics.theorem_definitions.iter())
        .find(|theorem| theorem.name.text == environment_name)?;

    let node = latex::Environment::cast(node)?;
    let theorem_description = node
        .begin()?
        .options()
        .and_then(|option| option.content_text());

    let full_range = document
        .line_index
        .line_col_lsp_range(latex::small_range(&node));

    let symbol = match NumberedLabel::find(project, node.syntax()) {
        Some(label) => {
            let name = match (label.number, theorem_description) {
                (Some(number), Some(desc)) => {
                    format!("{} {} ({})", definition.description, number, desc)
                }
                (Some(number), None) => format!("{} {}", definition.description, number),
                (None, Some(desc)) => format!("{} ({})", definition.description, desc),
                (None, None) => definition.description.clone(),
            };

            InternalSymbol {
                name,
                label: Some(label.name.text),
                kind: InternalSymbolKind::Theorem,
                deprecated: false,
                full_range,
                selection_range: document.line_index.line_col_lsp_range(label.range),
                children: Vec::new(),
            }
        }
        None => {
            let name = match theorem_description {
                Some(desc) => format!("{} ({})", definition.description, desc),
                None => definition.description.clone(),
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

#[derive(Debug)]
struct NumberedLabel<'a> {
    name: Span,
    range: TextRange,
    number: Option<&'a str>,
}

impl<'a> NumberedLabel<'a> {
    fn find(project: &Project<'a>, parent: &latex::SyntaxNode) -> Option<Self> {
        let label = parent.children().find_map(latex::LabelDefinition::cast)?;
        let name = Span::from(&label.name()?.key()?);
        let number = project
            .documents
            .iter()
            .filter_map(|document| document.data.as_aux())
            .find_map(|data| data.semantics.label_numbers.get(&name.text))
            .map(|number| number.as_str());

        Some(NumberedLabel {
            name,
            range: latex::small_range(&label),
            number,
        })
    }
}
