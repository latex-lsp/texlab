use std::sync::Arc;

use derive_more::From;
use rowan::{GreenNode, TextRange, TextSize};
use smol_str::SmolStr;

use crate::{
    latex::{self, HasBraces},
    CstNode,
};

use super::{Cst, CstDatabase, Document};

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum SymbolTree {
    Latex(LatexSymbolTree),
    Bibtex,
    BuildLog,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct LatexSymbolTree {
    pub children: Vec<LatexSymbol>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, From)]
pub enum LatexSymbol {
    Section(LatexSection),
    EnumItem(LatexEnumItem),
    Equation(LatexEquation),
    Environment(LatexEnvironment),
    LabelDefinition(LatexLabelDefinition),
    LabelReference(LatexLabelReference),
    LabelReferenceRange(LatexLabelReferenceRange),
    LabelNumber(LatexLabelNumber),
    Include(LatexInclude),
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Span {
    pub range: TextRange,
    pub text: SmolStr,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, PartialOrd, Ord)]
pub struct LatexSection(salsa::InternId);

impl salsa::InternKey for LatexSection {
    fn from_intern_id(id: salsa::InternId) -> Self {
        Self(id)
    }

    fn as_intern_id(&self) -> salsa::InternId {
        self.0
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct LatexSectionData {
    pub full_range: TextRange,
    pub text: String,
    pub children: Vec<LatexSymbol>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, PartialOrd, Ord)]
pub struct LatexEnumItem(salsa::InternId);

impl salsa::InternKey for LatexEnumItem {
    fn from_intern_id(id: salsa::InternId) -> Self {
        Self(id)
    }

    fn as_intern_id(&self) -> salsa::InternId {
        self.0
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct LatexEnumItemData {
    pub full_range: TextRange,
    pub label: Option<SmolStr>,
    pub children: Vec<LatexSymbol>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, PartialOrd, Ord)]
pub struct LatexEquation(salsa::InternId);

impl salsa::InternKey for LatexEquation {
    fn from_intern_id(id: salsa::InternId) -> Self {
        Self(id)
    }

    fn as_intern_id(&self) -> salsa::InternId {
        self.0
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct LatexEquationData {
    pub full_range: TextRange,
    pub children: Vec<LatexSymbol>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, PartialOrd, Ord)]
pub struct LatexEnvironment(salsa::InternId);

impl salsa::InternKey for LatexEnvironment {
    fn from_intern_id(id: salsa::InternId) -> Self {
        Self(id)
    }

    fn as_intern_id(&self) -> salsa::InternId {
        self.0
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct LatexEnvironmentData {
    pub full_range: TextRange,
    pub begin: Span,
    pub end: Span,
    pub children: Vec<LatexSymbol>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, PartialOrd, Ord)]
pub struct LatexLabelDefinition(salsa::InternId);

impl salsa::InternKey for LatexLabelDefinition {
    fn from_intern_id(id: salsa::InternId) -> Self {
        Self(id)
    }

    fn as_intern_id(&self) -> salsa::InternId {
        self.0
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct LatexLabelDefinitionData {
    pub full_range: TextRange,
    pub name: Span,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, PartialOrd, Ord)]
pub struct LatexLabelReference(salsa::InternId);

impl salsa::InternKey for LatexLabelReference {
    fn from_intern_id(id: salsa::InternId) -> Self {
        Self(id)
    }

    fn as_intern_id(&self) -> salsa::InternId {
        self.0
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct LatexLabelReferenceData {
    pub full_range: TextRange,
    pub name: Span,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, PartialOrd, Ord)]
pub struct LatexLabelReferenceRange(salsa::InternId);

impl salsa::InternKey for LatexLabelReferenceRange {
    fn from_intern_id(id: salsa::InternId) -> Self {
        Self(id)
    }

    fn as_intern_id(&self) -> salsa::InternId {
        self.0
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct LatexLabelReferenceRangeData {
    pub full_range: TextRange,
    pub from: Span,
    pub to: Span,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, PartialOrd, Ord)]
pub struct LatexLabelNumber(salsa::InternId);

impl salsa::InternKey for LatexLabelNumber {
    fn from_intern_id(id: salsa::InternId) -> Self {
        Self(id)
    }

    fn as_intern_id(&self) -> salsa::InternId {
        self.0
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct LatexLabelNumberData {
    pub name: SmolStr,
    pub text: String,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, PartialOrd, Ord)]
pub enum LatexIncludeKind {
    Package,
    Class,
    Latex,
    Bibtex,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, PartialOrd, Ord)]
pub struct LatexInclude(salsa::InternId);

impl salsa::InternKey for LatexInclude {
    fn from_intern_id(id: salsa::InternId) -> Self {
        Self(id)
    }

    fn as_intern_id(&self) -> salsa::InternId {
        self.0
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct LatexIncludeData {
    pub kind: LatexIncludeKind,
    pub path: Span,
}

#[salsa::query_group(SymbolTreeDatabaseStorage)]
pub trait SymbolTreeDatabase: CstDatabase {
    #[salsa::interned]
    fn intern_latex_section(&self, section: LatexSectionData) -> LatexSection;

    #[salsa::interned]
    fn intern_latex_enum_item(&self, item: LatexEnumItemData) -> LatexEnumItem;

    #[salsa::interned]
    fn intern_latex_equation(&self, equation: LatexEquationData) -> LatexEquation;

    #[salsa::interned]
    fn intern_latex_environment(&self, environment: LatexEnvironmentData) -> LatexEnvironment;

    #[salsa::interned]
    fn intern_latex_label_definition(
        &self,
        label: LatexLabelDefinitionData,
    ) -> LatexLabelDefinition;

    #[salsa::interned]
    fn intern_latex_label_reference(&self, label: LatexLabelReferenceData) -> LatexLabelReference;

    #[salsa::interned]
    fn intern_latex_label_reference_range(
        &self,
        label: LatexLabelReferenceRangeData,
    ) -> LatexLabelReferenceRange;

    #[salsa::interned]
    fn intern_latex_label_number(&self, label: LatexLabelNumberData) -> LatexLabelNumber;

    #[salsa::interned]
    fn intern_latex_include(&self, include: LatexIncludeData) -> LatexInclude;

    fn symbol_tree(&self, document: Document) -> Arc<SymbolTree>;
}

fn symbol_tree(db: &dyn SymbolTreeDatabase, document: Document) -> Arc<SymbolTree> {
    match db.cst(document) {
        Cst::Latex(green) => latex_symbol_tree(db, green),
        Cst::Bibtex(_) => todo!(),
        Cst::BuildLog => Arc::new(SymbolTree::BuildLog),
    }
}

fn latex_symbol_tree(db: &dyn SymbolTreeDatabase, green: GreenNode) -> Arc<SymbolTree> {
    let mut children = Vec::new();
    visit_latex_symbol(db, latex::SyntaxNode::new_root(green), &mut children);
    Arc::new(SymbolTree::Latex(LatexSymbolTree { children }))
}

fn visit_latex_symbol(
    db: &dyn SymbolTreeDatabase,
    node: latex::SyntaxNode,
    symbols: &mut Vec<LatexSymbol>,
) {
    visit_latex_section(db, node.clone(), symbols)
        .or_else(|| visit_latex_enum_item(db, node.clone(), symbols))
        .or_else(|| visit_latex_equation(db, node.clone(), symbols))
        .or_else(|| visit_latex_label_definition(db, node.clone(), symbols))
        .or_else(|| visit_latex_label_reference(db, node.clone(), symbols))
        .or_else(|| visit_latex_label_reference_range(db, node.clone(), symbols))
        .or_else(|| visit_latex_label_number(db, node.clone(), symbols))
        .or_else(|| visit_latex_include(db, node.clone(), symbols))
        .unwrap_or_else(|| {
            for child in node.children() {
                visit_latex_symbol(db, child, symbols);
            }
        });
}

fn visit_latex_section(
    db: &dyn SymbolTreeDatabase,
    node: latex::SyntaxNode,
    symbols: &mut Vec<LatexSymbol>,
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
        visit_latex_symbol(db, child, &mut children);
    }

    let section = db.intern_latex_section(LatexSectionData {
        full_range,
        text,
        children,
    });

    symbols.push(section.into());
    Some(())
}

fn visit_latex_enum_item(
    db: &dyn SymbolTreeDatabase,
    node: latex::SyntaxNode,
    symbols: &mut Vec<LatexSymbol>,
) -> Option<()> {
    let node = latex::EnumItem::cast(node)?;
    let label = node
        .label()
        .and_then(|label| label.word())
        .map(|word| word.text().into());

    let full_range = node.syntax().text_range();

    let mut children = Vec::new();
    for child in node.syntax().children() {
        visit_latex_symbol(db, child, &mut children);
    }

    let enum_item = db.intern_latex_enum_item(LatexEnumItemData {
        full_range,
        label,
        children,
    });

    symbols.push(enum_item.into());
    Some(())
}

fn visit_latex_equation(
    db: &dyn SymbolTreeDatabase,
    node: latex::SyntaxNode,
    symbols: &mut Vec<LatexSymbol>,
) -> Option<()> {
    let node = latex::Equation::cast(node)?;
    let full_range = node.syntax().text_range();

    let mut children = Vec::new();
    for child in node.syntax().children() {
        visit_latex_symbol(db, child, &mut children);
    }

    let equation = db.intern_latex_equation(LatexEquationData {
        full_range,
        children,
    });

    symbols.push(equation.into());
    Some(())
}

fn visit_latex_label_definition(
    db: &dyn SymbolTreeDatabase,
    node: latex::SyntaxNode,
    symbols: &mut Vec<LatexSymbol>,
) -> Option<()> {
    let node = latex::LabelDefinition::cast(node)?;
    let full_range = node.syntax().text_range();
    let name = node.name()?.word()?;

    let label = db.intern_latex_label_definition(LatexLabelDefinitionData {
        full_range,
        name: Span {
            range: name.text_range(),
            text: name.text().into(),
        },
    });

    symbols.push(label.into());
    Some(())
}

fn visit_latex_label_reference(
    db: &dyn SymbolTreeDatabase,
    node: latex::SyntaxNode,
    symbols: &mut Vec<LatexSymbol>,
) -> Option<()> {
    let node = latex::LabelReference::cast(node)?;
    let full_range = node.syntax().text_range();
    for name in node.name_list()?.words() {
        let label = db.intern_latex_label_reference(LatexLabelReferenceData {
            full_range,
            name: Span {
                range: name.text_range(),
                text: name.text().into(),
            },
        });
        symbols.push(label.into());
    }
    Some(())
}

fn visit_latex_label_reference_range(
    db: &dyn SymbolTreeDatabase,
    node: latex::SyntaxNode,
    symbols: &mut Vec<LatexSymbol>,
) -> Option<()> {
    let node = latex::LabelReferenceRange::cast(node)?;
    let full_range = node.syntax().text_range();
    let from = node.from()?.word()?;
    let to = node.to()?.word()?;

    let label = db.intern_latex_label_reference_range(LatexLabelReferenceRangeData {
        full_range,
        from: Span {
            range: from.text_range(),
            text: from.text().into(),
        },
        to: Span {
            range: to.text_range(),
            text: to.text().into(),
        },
    });

    symbols.push(label.into());
    Some(())
}

fn visit_latex_label_number(
    db: &dyn SymbolTreeDatabase,
    node: latex::SyntaxNode,
    symbols: &mut Vec<LatexSymbol>,
) -> Option<()> {
    let node = latex::LabelNumber::cast(node)?;
    let name = node.name()?.word()?.text().into();
    let text = node
        .text()?
        .syntax()
        .descendants()
        .find_map(latex::Text::cast)?
        .syntax()
        .text()
        .to_string();

    let label = db.intern_latex_label_number(LatexLabelNumberData { name, text });
    symbols.push(label.into());
    Some(())
}

fn visit_latex_include(
    db: &dyn SymbolTreeDatabase,
    node: latex::SyntaxNode,
    symbols: &mut Vec<LatexSymbol>,
) -> Option<()> {
    let node = latex::Include::cast(node)?;
    let kind = match node.syntax().kind() {
        latex::PACKAGE_INCLUDE => LatexIncludeKind::Package,
        latex::CLASS_INCLUDE => LatexIncludeKind::Class,
        latex::BIBTEX_INCLUDE | latex::BIBLATEX_INCLUDE => LatexIncludeKind::Bibtex,
        _ => LatexIncludeKind::Latex,
    };

    for name in node.path_list()?.words() {
        let include = db.intern_latex_include(LatexIncludeData {
            kind,
            path: Span {
                range: name.text_range(),
                text: name.text().into(),
            },
        });
        symbols.push(include.into());
    }
    Some(())
}
