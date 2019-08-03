use crate::syntax::*;
use crate::workspace::*;
use futures_boxed::boxed;
use itertools::Itertools;
use lsp_types::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexSectionNode<'a> {
    section: &'a LatexSection,
    numbers: Vec<u64>,
    full_range: Range,
    text: &'a str,
    children: Vec<Self>,
}

impl<'a> LatexSectionNode<'a> {
    pub fn new(section: &'a LatexSection) -> Self {
        Self {
            section,
            numbers: Vec::new(),
            full_range: Range::default(),
            text: "",
            children: Vec::new(),
        }
    }

    fn populate_text(&mut self, text: &'a str) {
        self.text = text;
        for child in &mut self.children {
            child.populate_text(text);
        }
    }

    fn push_number(&mut self, number: u64) {
        self.numbers.push(number);
        for child in &mut self.children {
            child.push_number(number);
        }
    }

    fn populate_numbers(&mut self, number: u64) {
        self.push_number(number);
        for (i, child) in self.children.iter_mut().enumerate() {
            child.populate_numbers((i + 1) as u64);
        }
    }

    pub fn make_symbol(&self) -> DocumentSymbol {
        let name = self
            .section
            .extract_text(self.text)
            .unwrap_or_else(|| "Unknown".to_owned());

        let number = self.numbers.iter().map(u64::to_string).join(".");

        let children = self.children.iter().map(Self::make_symbol).collect();

        DocumentSymbol {
            name: format!("{} {}", number, name).into(),
            detail: None,
            kind: SymbolKind::Module,
            deprecated: Some(false),
            range: self.section.range(),
            selection_range: self.section.range(),
            children: Some(children),
        }
    }

    fn insert(nodes: &mut Vec<Self>, section: &'a LatexSection) {
        match nodes.last_mut() {
            Some(parent) => {
                if parent.section.level < section.level {
                    Self::insert(&mut parent.children, section);
                } else {
                    nodes.push(LatexSectionNode::new(section));
                }
            }
            None => {
                nodes.push(LatexSectionNode::new(section));
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexSectionTree<'a> {
    children: Vec<LatexSectionNode<'a>>,
}

impl<'a> LatexSectionTree<'a> {
    fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }

    fn populate_text(&mut self, text: &'a str) {
        for child in &mut self.children {
            child.populate_text(text);
        }
    }

    fn populate_numbers(&mut self) {
        for (i, child) in self.children.iter_mut().enumerate() {
            child.populate_numbers((i + 1) as u64);
        }
    }
}

impl<'a> From<&'a LatexSyntaxTree> for LatexSectionTree<'a> {
    fn from(tree: &'a LatexSyntaxTree) -> Self {
        let mut root = Self::new();
        for section in &tree.sections {
            LatexSectionNode::insert(&mut root.children, section);
        }
        root
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct LatexSectionSymbolProvider;

impl FeatureProvider for LatexSectionSymbolProvider {
    type Params = DocumentSymbolParams;
    type Output = Vec<DocumentSymbol>;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<Self::Params>) -> Self::Output {
        let mut symbols = Vec::new();
        if let SyntaxTree::Latex(tree) = &request.document().tree {
            let mut section_tree = LatexSectionTree::from(tree);
            section_tree.populate_text(&request.document().text);
            section_tree.populate_numbers();
            for child in &section_tree.children {
                symbols.push(child.make_symbol());
            }
        }
        symbols
    }
}
