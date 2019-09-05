mod enumeration;
mod equation;
mod float;
mod theorem;

use super::{LatexSymbol, LatexSymbolKind};
use crate::range::RangeExt;
use crate::syntax::*;
use crate::workspace::*;
use futures_boxed::boxed;
use lsp_types::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct LatexSectionSymbolProvider;

impl FeatureProvider for LatexSectionSymbolProvider {
    type Params = DocumentSymbolParams;
    type Output = Vec<LatexSymbol>;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<Self::Params>) -> Self::Output {
        let mut symbols = Vec::new();
        if let SyntaxTree::Latex(tree) = &request.document().tree {
            let mut section_tree = LatexSectionTree::from(tree.as_ref());

            section_tree.set_full_text(&request.document().text);

            let end_position = Self::compute_end_position(tree, &request.document().text);
            LatexSectionNode::set_full_range(&mut section_tree.children, end_position);

            let outline = Outline::from(&request.view);
            for child in &mut section_tree.children {
                child.set_label(tree, &request.view, &outline);
            }

            for symbol in enumeration::symbols(tree) {
                section_tree.insert_symbol(&symbol);
            }

            for symbol in equation::symbols(&request.view, tree) {
                section_tree.insert_symbol(&symbol);
            }

            for symbol in float::symbols(&request.view, tree) {
                section_tree.insert_symbol(&symbol);
            }

            for symbol in theorem::symbols(&request.view, tree) {
                section_tree.insert_symbol(&symbol);
            }

            for symbol in section_tree.symbols {
                symbols.push(symbol);
            }

            for child in section_tree.children {
                symbols.push(child.into());
            }
        }
        symbols
    }
}

impl LatexSectionSymbolProvider {
    fn compute_end_position(tree: &LatexSyntaxTree, text: &str) -> Position {
        let mut stream = CharStream::new(text);
        while stream.next().is_some() {}
        tree.environments
            .iter()
            .find(|env| env.left.name().map(LatexToken::text) == Some("document"))
            .map(|env| env.right.start())
            .unwrap_or(stream.current_position)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct LatexSectionNode<'a> {
    section: &'a LatexSection,
    full_range: Range,
    full_text: &'a str,
    label: Option<String>,
    number: Option<String>,
    symbols: Vec<LatexSymbol>,
    children: Vec<Self>,
}

impl<'a> LatexSectionNode<'a> {
    fn new(section: &'a LatexSection) -> Self {
        Self {
            section,
            full_range: Range::default(),
            full_text: "",
            label: None,
            number: None,
            symbols: Vec::new(),
            children: Vec::new(),
        }
    }

    fn set_full_text(&mut self, full_text: &'a str) {
        self.full_text = full_text;
        for child in &mut self.children {
            child.set_full_text(full_text);
        }
    }

    fn name(&self) -> String {
        self.section
            .extract_text(self.full_text)
            .unwrap_or_else(|| "Unknown".to_owned())
    }

    fn set_full_range(children: &mut Vec<Self>, end_position: Position) {
        for i in 0..children.len() {
            let current_end = children
                .get(i + 1)
                .map(|next| next.section.start())
                .unwrap_or(end_position);

            let mut current = &mut children[i];
            current.full_range = Range::new(current.section.start(), current_end);
            Self::set_full_range(&mut current.children, current_end);
        }
    }

    fn set_label(&mut self, tree: &LatexSyntaxTree, view: &DocumentView, outline: &Outline) {
        if let Some(label) = tree
            .labels
            .iter()
            .filter(|label| label.kind == LatexLabelKind::Definition)
            .find(|label| self.full_range.contains(label.start()))
        {
            if let Some(ctx) = OutlineContext::parse(view, label, outline) {
                let mut is_section = false;
                if let OutlineContextItem::Section(section) = &ctx.item {
                    if self.name() == *section {
                        for name in label.names() {
                            self.label = Some(name.text().to_owned());
                        }

                        is_section = true;
                    }
                }

                if is_section {
                    self.number = ctx.number;
                }
            }
        }

        for child in &mut self.children {
            child.set_label(tree, view, outline);
        }
    }

    fn insert_section(nodes: &mut Vec<Self>, section: &'a LatexSection) {
        match nodes.last_mut() {
            Some(parent) => {
                if parent.section.level < section.level {
                    Self::insert_section(&mut parent.children, section);
                } else {
                    nodes.push(LatexSectionNode::new(section));
                }
            }
            None => {
                nodes.push(LatexSectionNode::new(section));
            }
        }
    }

    fn insert_symbol(&mut self, symbol: &LatexSymbol) -> bool {
        if !self.full_range.contains(symbol.selection_range.start) {
            return false;
        }

        for child in &mut self.children {
            if child.insert_symbol(symbol) {
                return true;
            }
        }

        self.symbols.push(symbol.clone());
        true
    }
}

impl<'a> Into<LatexSymbol> for LatexSectionNode<'a> {
    fn into(self) -> LatexSymbol {
        let name = self.name();

        let mut children: Vec<LatexSymbol> = self.children.into_iter().map(Into::into).collect();

        for symbol in self.symbols {
            children.push(symbol);
        }

        let full_name = match &self.number {
            Some(number) => format!("{} {}", number, name),
            None => name,
        };

        LatexSymbol {
            name: full_name,
            label: self.label,
            kind: LatexSymbolKind::Section,
            deprecated: false,
            full_range: self.full_range,
            selection_range: self.section.range(),
            children,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct LatexSectionTree<'a> {
    symbols: Vec<LatexSymbol>,
    children: Vec<LatexSectionNode<'a>>,
}

impl<'a> LatexSectionTree<'a> {
    fn new() -> Self {
        Self {
            symbols: Vec::new(),
            children: Vec::new(),
        }
    }

    fn set_full_text(&mut self, full_text: &'a str) {
        for child in &mut self.children {
            child.set_full_text(full_text);
        }
    }

    fn insert_symbol(&mut self, symbol: &LatexSymbol) {
        for child in &mut self.children {
            if child.insert_symbol(symbol) {
                return;
            }
        }
        self.symbols.push(symbol.clone());
    }
}

impl<'a> From<&'a LatexSyntaxTree> for LatexSectionTree<'a> {
    fn from(tree: &'a LatexSyntaxTree) -> Self {
        let mut root = Self::new();
        for section in &tree.sections {
            LatexSectionNode::insert_section(&mut root.children, section);
        }
        root
    }
}

pub fn label_name(label: Option<&LatexLabel>) -> Option<String> {
    label.map(|label| label.names()[0].text().to_owned())
}

pub fn selection_range(full_range: Range, label: Option<&LatexLabel>) -> Range {
    label.map(|label| label.range()).unwrap_or(full_range)
}
