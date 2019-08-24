use crate::range::RangeExt;
use crate::syntax::*;
use crate::workspace::*;
use futures_boxed::boxed;
use lsp_types::*;

fn make_label_symbols(
    view: &DocumentView,
    outline: &Outline,
    label: &LatexLabel,
) -> Vec<DocumentSymbol> {
    let mut symbols = Vec::new();
    let position = label.start();
    if let Some(context) = OutlineContext::parse(view, position, outline) {
        match &context.item {
            OutlineContextItem::Equation => {
                for name in label.names() {
                    let symbol = DocumentSymbol {
                        name: name.text().to_owned().into(),
                        detail: None,
                        kind: SymbolKind::Number,
                        deprecated: Some(false),
                        range: context.range,
                        selection_range: name.range(),
                        children: None,
                    };
                    symbols.push(symbol);
                }
            }
            OutlineContextItem::Caption(caption) => {
                for name in label.names() {
                    let symbol = DocumentSymbol {
                        name: caption.clone().into(),
                        detail: None,
                        kind: SymbolKind::Method,
                        deprecated: Some(false),
                        range: context.range,
                        selection_range: name.range(),
                        children: None,
                    };
                    symbols.push(symbol);
                }
            }
            OutlineContextItem::Theorem {
                kind: _,
                description: _,
            } => {
                for name in label.names() {
                    let symbol = DocumentSymbol {
                        name: context.item.clone().reference().into(),
                        detail: None,
                        kind: SymbolKind::EnumMember,
                        deprecated: Some(false),
                        range: context.range,
                        selection_range: name.range(),
                        children: None,
                    };
                    symbols.push(symbol);
                }
            }
            OutlineContextItem::Section(_) => {}
        }
    }
    symbols
}

struct LatexSectionNode<'a> {
    section: &'a LatexSection,
    full_range: Range,
    full_text: &'a str,
    labels: Vec<&'a LatexLabel>,
    children: Vec<Self>,
}

impl<'a> LatexSectionNode<'a> {
    fn new(section: &'a LatexSection) -> Self {
        Self {
            section,
            full_range: Range::default(),
            full_text: "",
            labels: Vec::new(),
            children: Vec::new(),
        }
    }

    fn set_full_text(&mut self, full_text: &'a str) {
        self.full_text = full_text;
        for child in &mut self.children {
            child.set_full_text(full_text);
        }
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

    fn into_symbol(&self, view: &DocumentView, outline: &Outline) -> DocumentSymbol {
        let name = self
            .section
            .extract_text(self.full_text)
            .unwrap_or_else(|| "Unknown".to_owned());

        let mut children = Vec::new();
        self.children
            .iter()
            .map(|child| child.into_symbol(view, outline))
            .for_each(|sec| children.push(sec));

        for label in &self.labels {
            children.append(&mut make_label_symbols(view, outline, label));
        }

        DocumentSymbol {
            name: name.into(),
            detail: None,
            kind: SymbolKind::Module,
            deprecated: Some(false),
            range: self.full_range,
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

    fn insert_label(&mut self, label: &'a LatexLabel) -> bool {
        if !self.full_range.contains(label.start()) {
            return false;
        }

        for child in &mut self.children {
            if child.insert_label(label) {
                return true;
            }
        }

        self.labels.push(label);
        true
    }
}

struct LatexSectionTree<'a> {
    children: Vec<LatexSectionNode<'a>>,
}

impl<'a> LatexSectionTree<'a> {
    fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }

    fn set_full_text(&mut self, full_text: &'a str) {
        for child in &mut self.children {
            child.set_full_text(full_text);
        }
    }

    fn insert_label(&mut self, label: &'a LatexLabel) -> bool {
        for child in &mut self.children {
            if child.insert_label(label) {
                return true;
            }
        }
        false
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
            section_tree.set_full_text(&request.document().text);
            let mut stream = CharStream::new(&request.document().text);
            while stream.next().is_some() {}
            let end_position = tree
                .environments
                .iter()
                .find(|env| env.left.name().map(LatexToken::text) == Some("document"))
                .map(|env| env.right.start())
                .unwrap_or(stream.current_position);
            LatexSectionNode::set_full_range(&mut section_tree.children, end_position);

            let outline = Outline::from(&request.view);
            for label in &tree.labels {
                if label.kind == LatexLabelKind::Definition {
                    if !section_tree.insert_label(label) {
                        symbols.append(&mut make_label_symbols(&request.view, &outline, label));
                    }
                }
            }

            for child in &section_tree.children {
                symbols.push(child.into_symbol(&request.view, &outline));
            }
        }
        symbols
    }
}
