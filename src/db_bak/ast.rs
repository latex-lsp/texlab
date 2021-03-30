use std::sync::Arc;

use cstree::{GreenNode, TextRange, TextSize};
use derive_more::From;
use smol_str::SmolStr;

use crate::{
    latex::{self, HasBraces},
    CstNode,
};

use super::{Cst, CstDatabase, Document};

#[salsa::query_group(AstDatabaseStorage)]
pub trait AstDatabase: CstDatabase {
    #[salsa::interned]
    fn intern_latex_group(&self, group: LatexGroupData) -> LatexGroup;

    #[salsa::interned]
    fn intern_latex_generic_command(&self, cmd: LatexGenericCommandData) -> LatexGenericCommand;

    #[salsa::interned]
    fn intern_latex_environment(&self, env: LatexEnvironmentData) -> LatexEnvironment;

    #[salsa::interned]
    fn intern_latex_equation(&self, equation: LatexEquationData) -> LatexEquation;

    #[salsa::interned]
    fn intern_latex_section(&self, section: LatexSectionData) -> LatexSection;

    #[salsa::interned]
    fn intern_latex_enum_item(&self, item: LatexEnumItemData) -> LatexEnumItem;

    #[salsa::interned]
    fn intern_latex_formula(&self, formula: LatexFormulaData) -> LatexFormula;

    #[salsa::interned]
    fn intern_latex_caption(&self, caption: LatexCaptionData) -> LatexCaption;

    #[salsa::interned]
    fn intern_latex_citation(&self, citation: LatexCitationData) -> LatexCitation;

    #[salsa::interned]
    fn intern_latex_include(&self, include: LatexIncludeData) -> LatexInclude;

    #[salsa::interned]
    fn intern_latex_import(&self, import: LatexImportData) -> LatexImport;

    fn ast(&self, document: Document) -> Arc<Ast>;
}

fn ast(db: &dyn AstDatabase, document: Document) -> Arc<Ast> {
    match db.cst(document) {
        Cst::Latex(green) => latex_ast(db, green),
        Cst::Bibtex(_) => Arc::new(Ast::Bibtex),
        Cst::BuildLog => Arc::new(Ast::BuildLog),
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Ast {
    Latex(LatexAst),
    Bibtex,
    BuildLog,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct LatexAst {
    children: Vec<LatexNode>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, From)]
pub enum LatexNode {
    Group(LatexGroup),
    GenericCommand(LatexGenericCommand),
    Environment(LatexEnvironment),
    Equation(LatexEquation),
    Section(LatexSection),
    EnumItem(LatexEnumItem),
    Formula(LatexFormula),
    Caption(LatexCaption),
    Citation(LatexCitation),
    Include(LatexInclude),
    Import(LatexImport),
}

macro_rules! salsa_key {
    ($name:ident) => {
        #[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, PartialOrd, Ord)]
        pub struct $name(salsa::InternId);

        impl salsa::InternKey for $name {
            fn from_intern_id(id: salsa::InternId) -> Self {
                Self(id)
            }

            fn as_intern_id(&self) -> salsa::InternId {
                self.0
            }
        }
    };
}

salsa_key!(LatexGroup);
salsa_key!(LatexGenericCommand);
salsa_key!(LatexEnvironment);
salsa_key!(LatexEquation);
salsa_key!(LatexSection);
salsa_key!(LatexEnumItem);
salsa_key!(LatexFormula);
salsa_key!(LatexCaption);
salsa_key!(LatexCitation);
salsa_key!(LatexInclude);
salsa_key!(LatexImport);

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Span {
    pub range: TextRange,
    pub text: SmolStr,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, PartialOrd, Ord)]
pub enum LatexGroupKind {
    Brace,
    Bracket,
    Paren,
    Mixed,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct LatexGroupData {
    pub full_range: TextRange,
    pub kind: LatexGroupKind,
    pub children: Vec<LatexNode>,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct LatexGenericCommandData {
    pub full_range: TextRange,
    pub name: Span,
    pub children: Vec<LatexNode>,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct LatexEnvironmentData {
    pub full_range: TextRange,
    pub begin_name: Span,
    pub end_name: Span,
    pub children: Vec<LatexNode>,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct LatexEquationData {
    pub full_range: TextRange,
    pub children: Vec<LatexNode>,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct LatexSectionData {
    pub full_range: TextRange,
    pub text: String,
    pub children: Vec<LatexNode>,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct LatexEnumItemData {
    pub full_range: TextRange,
    pub label: Option<SmolStr>,
    pub children: Vec<LatexNode>,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct LatexFormulaData {
    pub full_range: TextRange,
    pub children: Vec<LatexNode>,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct LatexCaptionData {
    pub full_range: TextRange,
    pub long: String,
    pub children: Vec<LatexNode>,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct LatexCitationData {
    pub key: Span,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, PartialOrd, Ord)]
pub enum LatexIncludeKind {
    Package,
    Class,
    Latex,
    Bibtex,
    Graphics,
    Svg,
    Inkscape,
    Verbatim,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct LatexIncludeData {
    pub kind: LatexIncludeKind,
    pub path: Span,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct LatexImportData {
    pub full_range: TextRange,
    pub directory: Span,
    pub file: Span,
}

fn latex_ast(db: &dyn AstDatabase, green: GreenNode) -> Arc<Ast> {
    let mut children = Vec::new();
    visit_latex_node(db, latex::SyntaxNode::new_root(green), &mut children);
    Arc::new(Ast::Latex(LatexAst { children }))
}

fn visit_latex_node(db: &dyn AstDatabase, node: latex::SyntaxNode, siblings: &mut Vec<LatexNode>) {
    visit_latex_group(db, node.clone(), siblings)
        .or_else(|| visit_latex_generic_command(db, node.clone(), siblings))
        .or_else(|| visit_latex_environment(db, node.clone(), siblings))
        .or_else(|| visit_latex_equation(db, node.clone(), siblings))
        .or_else(|| visit_latex_section(db, node.clone(), siblings))
        .or_else(|| visit_latex_enum_item(db, node.clone(), siblings))
        .or_else(|| visit_latex_formula(db, node.clone(), siblings))
        .or_else(|| visit_latex_caption(db, node.clone(), siblings))
        .or_else(|| visit_latex_citation(db, node.clone(), siblings))
        .or_else(|| visit_latex_include(db, node.clone(), siblings))
        .or_else(|| visit_latex_import(db, node.clone(), siblings))
        // .or_else(|| visit_latex_label_definition(db, node.clone(), siblings))
        // .or_else(|| visit_latex_label_reference(db, node.clone(), siblings))
        // .or_else(|| visit_latex_label_reference_range(db, node.clone(), siblings))
        // .or_else(|| visit_latex_label_number(db, node.clone(), siblings))
        .unwrap_or_else(|| {
            for child in node.children() {
                visit_latex_node(db, child, siblings);
            }
        });
}

fn visit_latex_group(
    db: &dyn AstDatabase,
    node: latex::SyntaxNode,
    siblings: &mut Vec<LatexNode>,
) -> Option<()> {
    let kind = match node.kind() {
        latex::BRACE_GROUP => LatexGroupKind::Brace,
        latex::BRACKET_GROUP => LatexGroupKind::Bracket,
        latex::PAREN_GROUP => LatexGroupKind::Paren,
        latex::MIXED_GROUP => LatexGroupKind::Mixed,
        _ => return None,
    };

    let full_range = node.text_range();
    let mut children = Vec::new();
    for child in node.children() {
        visit_latex_node(db, child, &mut children);
    }

    let group = db.intern_latex_group(LatexGroupData {
        kind,
        full_range,
        children,
    });

    siblings.push(group.into());
    Some(())
}

fn visit_latex_generic_command(
    db: &dyn AstDatabase,
    node: latex::SyntaxNode,
    siblings: &mut Vec<LatexNode>,
) -> Option<()> {
    let node = latex::GenericCommand::cast(node)?;
    let name = node.name()?;

    let full_range = node.syntax().text_range();
    let mut children = Vec::new();
    for child in node.syntax().children() {
        visit_latex_node(db, child, &mut children);
    }

    let command = db.intern_latex_generic_command(LatexGenericCommandData {
        full_range,
        name: Span {
            range: name.text_range(),
            text: name.text().into(),
        },
        children,
    });

    siblings.push(command.into());
    Some(())
}

fn visit_latex_environment(
    db: &dyn AstDatabase,
    node: latex::SyntaxNode,
    siblings: &mut Vec<LatexNode>,
) -> Option<()> {
    let node = latex::Environment::cast(node)?;
    let name1 = node.begin()?.name()?.word()?;
    let name2 = node.end()?.name()?.word()?;

    let full_range = node.syntax().text_range();
    let mut children = Vec::new();
    for child in node.syntax().children() {
        visit_latex_node(db, child, &mut children);
    }

    let environment = db.intern_latex_environment(LatexEnvironmentData {
        full_range,
        begin_name: Span {
            range: name1.text_range(),
            text: name1.text().into(),
        },
        end_name: Span {
            range: name2.text_range(),
            text: name2.text().into(),
        },
        children,
    });

    siblings.push(environment.into());
    Some(())
}

fn visit_latex_equation(
    db: &dyn AstDatabase,
    node: latex::SyntaxNode,
    siblings: &mut Vec<LatexNode>,
) -> Option<()> {
    let node = latex::Equation::cast(node)?;
    let full_range = node.syntax().text_range();

    let mut children = Vec::new();
    for child in node.syntax().children() {
        visit_latex_node(db, child, &mut children);
    }

    let equation = db.intern_latex_equation(LatexEquationData {
        full_range,
        children,
    });

    siblings.push(equation.into());
    Some(())
}

fn visit_latex_section(
    db: &dyn AstDatabase,
    node: latex::SyntaxNode,
    siblings: &mut Vec<LatexNode>,
) -> Option<()> {
    let node = latex::Section::cast(node)?;
    let name = node.name()?;
    name.left_brace()?;
    name.right_brace()?;
    let text = name.syntax().text();
    let text = text
        .slice(1.into()..text.len() - TextSize::from(1))
        .to_string();
    let full_range = node.syntax().text_range();

    let mut children = Vec::new();
    for child in node.syntax().children() {
        visit_latex_node(db, child, &mut children);
    }

    let section = db.intern_latex_section(LatexSectionData {
        full_range,
        text,
        children,
    });

    siblings.push(section.into());
    Some(())
}

fn visit_latex_enum_item(
    db: &dyn AstDatabase,
    node: latex::SyntaxNode,
    siblings: &mut Vec<LatexNode>,
) -> Option<()> {
    let node = latex::EnumItem::cast(node)?;
    let label = node
        .label()
        .and_then(|label| label.word())
        .map(|word| word.text().into());

    let full_range = node.syntax().text_range();

    let mut children = Vec::new();
    for child in node.syntax().children() {
        visit_latex_node(db, child, &mut children);
    }

    let enum_item = db.intern_latex_enum_item(LatexEnumItemData {
        full_range,
        label,
        children,
    });

    siblings.push(enum_item.into());
    Some(())
}

fn visit_latex_formula(
    db: &dyn AstDatabase,
    node: latex::SyntaxNode,
    siblings: &mut Vec<LatexNode>,
) -> Option<()> {
    let node = latex::Formula::cast(node)?;
    let full_range = node.syntax().text_range();

    let mut children = Vec::new();
    for child in node.syntax().children() {
        visit_latex_node(db, child, &mut children);
    }

    let formula = db.intern_latex_formula(LatexFormulaData {
        full_range,
        children,
    });

    siblings.push(formula.into());
    Some(())
}

fn visit_latex_caption(
    db: &dyn AstDatabase,
    node: latex::SyntaxNode,
    siblings: &mut Vec<LatexNode>,
) -> Option<()> {
    let node = latex::Caption::cast(node)?;
    let long = node.long()?;
    long.left_brace()?;
    long.right_brace()?;
    let long = long.syntax().text();
    let long = long
        .slice(1.into()..long.len() - TextSize::from(1))
        .to_string();
    let full_range = node.syntax().text_range();

    let mut children = Vec::new();
    for child in node.syntax().children() {
        visit_latex_node(db, child, &mut children);
    }

    let section = db.intern_latex_caption(LatexCaptionData {
        full_range,
        long,
        children,
    });

    siblings.push(section.into());
    Some(())
}

fn visit_latex_citation(
    db: &dyn AstDatabase,
    node: latex::SyntaxNode,
    siblings: &mut Vec<LatexNode>,
) -> Option<()> {
    let node = latex::Citation::cast(node)?;

    for key in node.key_list()?.words() {
        let citation = db.intern_latex_citation(LatexCitationData {
            key: Span {
                range: key.text_range(),
                text: key.text().into(),
            },
        });
        siblings.push(citation.into());
    }
    Some(())
}

fn visit_latex_include(
    db: &dyn AstDatabase,
    node: latex::SyntaxNode,
    siblings: &mut Vec<LatexNode>,
) -> Option<()> {
    let node = latex::Include::cast(node)?;
    let kind = match node.syntax().kind() {
        latex::PACKAGE_INCLUDE => LatexIncludeKind::Package,
        latex::CLASS_INCLUDE => LatexIncludeKind::Class,
        latex::BIBTEX_INCLUDE | latex::BIBLATEX_INCLUDE => LatexIncludeKind::Bibtex,
        latex::LATEX_INCLUDE => LatexIncludeKind::Latex,
        latex::GRAPHICS_INCLUDE => LatexIncludeKind::Graphics,
        latex::INKSCAPE_INCLUDE => LatexIncludeKind::Inkscape,
        latex::VERBATIM_INCLUDE => LatexIncludeKind::Verbatim,
        _ => unreachable!(),
    };

    for name in node.path_list()?.words() {
        let include = db.intern_latex_include(LatexIncludeData {
            kind,
            path: Span {
                range: name.text_range(),
                text: name.text().into(),
            },
        });
        siblings.push(include.into());
    }
    Some(())
}

fn visit_latex_import(
    db: &dyn AstDatabase,
    node: latex::SyntaxNode,
    siblings: &mut Vec<LatexNode>,
) -> Option<()> {
    let node = latex::Import::cast(node)?;
    let full_range = node.syntax().text_range();
    let directory = node.directory()?.word()?;
    let file = node.directory()?.word()?;

    let import = db.intern_latex_import(LatexImportData {
        full_range,
        directory: Span {
            range: directory.text_range(),
            text: directory.text().into(),
        },
        file: Span {
            range: file.text_range(),
            text: file.text().into(),
        },
    });

    siblings.push(import.into());
    Some(())
}

// fn visit_latex_label_definition(
//     db: &dyn AstDatabase,
//     node: latex::SyntaxNode,
//     symbols: &mut Vec<LatexNode>,
// ) -> Option<()> {
//     let node = latex::LabelDefinition::cast(node)?;
//     let full_range = node.syntax().text_range();
//     let name = node.name()?.word()?;

//     let label = db.intern_latex_label_definition(LatexLabelDefinitionData {
//         full_range,
//         name: Span {
//             range: name.text_range(),
//             text: name.text().into(),
//         },
//     });

//     symbols.push(label.into());
//     Some(())
// }

// fn visit_latex_label_reference(
//     db: &dyn AstDatabase,
//     node: latex::SyntaxNode,
//     symbols: &mut Vec<LatexNode>,
// ) -> Option<()> {
//     let node = latex::LabelReference::cast(node)?;
//     let full_range = node.syntax().text_range();
//     for name in node.name_list()?.words() {
//         let label = db.intern_latex_label_reference(LatexLabelReferenceData {
//             full_range,
//             name: Span {
//                 range: name.text_range(),
//                 text: name.text().into(),
//             },
//         });
//         symbols.push(label.into());
//     }
//     Some(())
// }

// fn visit_latex_label_reference_range(
//     db: &dyn AstDatabase,
//     node: latex::SyntaxNode,
//     symbols: &mut Vec<LatexNode>,
// ) -> Option<()> {
//     let node = latex::LabelReferenceRange::cast(node)?;
//     let full_range = node.syntax().text_range();
//     let from = node.from()?.word()?;
//     let to = node.to()?.word()?;

//     let label = db.intern_latex_label_reference_range(LatexLabelReferenceRangeData {
//         full_range,
//         from: Span {
//             range: from.text_range(),
//             text: from.text().into(),
//         },
//         to: Span {
//             range: to.text_range(),
//             text: to.text().into(),
//         },
//     });

//     symbols.push(label.into());
//     Some(())
// }

// fn visit_latex_label_number(
//     db: &dyn AstDatabase,
//     node: latex::SyntaxNode,
//     siblings: &mut Vec<LatexNode>,
// ) -> Option<()> {
//     let node = latex::LabelNumber::cast(node)?;
//     let name = node.name()?.word()?.text().into();
//     let text = node
//         .text()?
//         .syntax()
//         .descendants()
//         .find_map(latex::Text::cast)?
//         .syntax()
//         .text()
//         .to_string();

//     let label = db.intern_latex_label_number(LatexLabelNumberData { name, text });
//     siblings.push(label.into());
//     Some(())
// }
