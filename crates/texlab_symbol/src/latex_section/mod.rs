mod enumeration;
mod equation;
mod float;
mod theorem;

use super::{LatexSymbol, LatexSymbolKind};
use futures_boxed::boxed;
use texlab_protocol::RangeExt;
use texlab_protocol::*;
use texlab_syntax::*;
use texlab_workspace::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct LatexSectionSymbolProvider;

impl FeatureProvider for LatexSectionSymbolProvider {
    type Params = DocumentSymbolParams;
    type Output = Vec<LatexSymbol>;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<Self::Params>) -> Self::Output {
        let mut symbols = Vec::new();
        if let SyntaxTree::Latex(tree) = &request.document().tree {
            let mut section_tree = build_section_tree(&request.view, tree, &request.options);
            for symbol in enumeration::symbols(&request.view, tree) {
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

pub fn build_section_tree<'a>(
    view: &'a DocumentView,
    tree: &'a LatexSyntaxTree,
    options: &'a Options,
) -> LatexSectionTree<'a> {
    let mut section_tree = LatexSectionTree::from(tree);
    section_tree.set_full_text(&view.document.text);
    let end_position = compute_end_position(tree, &view.document.text);
    LatexSectionNode::set_full_range(&mut section_tree.children, end_position);
    let outline = Outline::analyze(view, options);
    for child in &mut section_tree.children {
        child.set_label(tree, view, &outline);
    }
    section_tree
}

fn compute_end_position(tree: &LatexSyntaxTree, text: &str) -> Position {
    let mut stream = CharStream::new(text);
    while stream.next().is_some() {}
    tree.env
        .environments
        .iter()
        .find(|env| env.left.name().map(LatexToken::text) == Some("document"))
        .map(|env| env.right.start())
        .unwrap_or(stream.current_position)
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexSectionNode<'a> {
    pub section: &'a LatexSection,
    pub full_range: Range,
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
            .structure
            .labels
            .iter()
            .filter(|label| label.kind == LatexLabelKind::Definition)
            .find(|label| self.full_range.contains(label.start()))
        {
            if let Some(ctx) = OutlineContext::parse(view, label, outline) {
                let mut is_section = false;
                if let OutlineContextItem::Section { text, .. } = &ctx.item {
                    if self.name() == *text {
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

    fn find(&self, label: &str) -> Option<&Self> {
        if self.label.as_ref().map(AsRef::as_ref) == Some(label) {
            Some(self)
        } else {
            for child in &self.children {
                let result = child.find(label);
                if result.is_some() {
                    return result;
                }
            }
            None
        }
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
pub struct LatexSectionTree<'a> {
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

    pub fn find(&self, label: &str) -> Option<&LatexSectionNode<'a>> {
        for child in &self.children {
            let result = child.find(label);
            if result.is_some() {
                return result;
            }
        }
        None
    }
}

impl<'a> From<&'a LatexSyntaxTree> for LatexSectionTree<'a> {
    fn from(tree: &'a LatexSyntaxTree) -> Self {
        let mut root = Self::new();
        for section in &tree.structure.sections {
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

#[cfg(test)]
mod tests {
    use super::*;
    use texlab_protocol::RangeExt;

    #[test]
    fn subsection() {
        let symbols = test_feature(
            LatexSectionSymbolProvider,
            FeatureSpec {
                files: vec![
                    FeatureSpec::file(
                        "foo.tex",
                        "\\section{Foo}\n\\subsection{Bar}\\label{sec:bar}\n\\subsection{Baz}\n\\section{Qux}",
                    ),
                    FeatureSpec::file(
                        "foo.aux", 
                        "\\newlabel{sec:bar}{{\\relax 2.1}{4}{Bar\\relax }{figure.caption.4}{}}"
                    ),
                ],
                main_file: "foo.tex",
                ..FeatureSpec::default()
            },
        );
        assert_eq!(
            symbols,
            vec![
                LatexSymbol {
                    name: "Foo".into(),
                    label: None,
                    kind: LatexSymbolKind::Section,
                    deprecated: false,
                    full_range: Range::new_simple(0, 0, 3, 0),
                    selection_range: Range::new_simple(0, 0, 0, 13),
                    children: vec![
                        LatexSymbol {
                            name: "2.1 Bar".into(),
                            label: Some("sec:bar".into()),
                            kind: LatexSymbolKind::Section,
                            deprecated: false,
                            full_range: Range::new_simple(1, 0, 2, 0),
                            selection_range: Range::new_simple(1, 0, 1, 16),
                            children: Vec::new(),
                        },
                        LatexSymbol {
                            name: "Baz".into(),
                            label: None,
                            kind: LatexSymbolKind::Section,
                            deprecated: false,
                            full_range: Range::new_simple(2, 0, 3, 0),
                            selection_range: Range::new_simple(2, 0, 2, 16),
                            children: Vec::new(),
                        },
                    ],
                },
                LatexSymbol {
                    name: "Qux".into(),
                    label: None,
                    kind: LatexSymbolKind::Section,
                    deprecated: false,
                    full_range: Range::new_simple(3, 0, 3, 13),
                    selection_range: Range::new_simple(3, 0, 3, 13),
                    children: Vec::new(),
                }
            ]
        );
    }

    #[test]
    fn section_inside_document_environment() {
        let symbols = test_feature(
            LatexSectionSymbolProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file(
                    "foo.tex",
                    "\\begin{document}\\section{Foo}\\relax\n\\end{document}",
                )],
                main_file: "foo.tex",
                ..FeatureSpec::default()
            },
        );
        assert_eq!(
            symbols,
            vec![LatexSymbol {
                name: "Foo".into(),
                label: None,
                kind: LatexSymbolKind::Section,
                deprecated: false,
                full_range: Range::new_simple(0, 16, 1, 0),
                selection_range: Range::new_simple(0, 16, 0, 29),
                children: Vec::new()
            }]
        );
    }

    #[test]
    fn enumeration() {
        let symbols = test_feature(
            LatexSectionSymbolProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file(
                    "foo.tex",
                    "\\section{Foo}\n\\begin{enumerate}\n\\end{enumerate}",
                )],
                main_file: "foo.tex",
                ..FeatureSpec::default()
            },
        );
        assert_eq!(
            symbols,
            vec![LatexSymbol {
                name: "Foo".into(),
                label: None,
                kind: LatexSymbolKind::Section,
                deprecated: false,
                full_range: Range::new_simple(0, 0, 2, 15),
                selection_range: Range::new_simple(0, 0, 0, 13),
                children: vec![LatexSymbol {
                    name: "Enumerate".into(),
                    label: None,
                    kind: LatexSymbolKind::Enumeration,
                    deprecated: false,
                    full_range: Range::new_simple(1, 0, 2, 15),
                    selection_range: Range::new_simple(1, 0, 2, 15),
                    children: Vec::new(),
                },],
            },]
        );
    }

    #[test]
    fn equation() {
        let symbols = test_feature(
            LatexSectionSymbolProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file(
                    "foo.tex",
                    "\\[Foo\\]\n\\begin{equation}\\label{eq:foo}\\end{equation}",
                )],
                main_file: "foo.tex",
                ..FeatureSpec::default()
            },
        );
        assert_eq!(
            symbols,
            vec![
                LatexSymbol {
                    name: "Equation".into(),
                    label: None,
                    kind: LatexSymbolKind::Equation,
                    deprecated: false,
                    full_range: Range::new_simple(0, 0, 0, 7),
                    selection_range: Range::new_simple(0, 0, 0, 7),
                    children: Vec::new(),
                },
                LatexSymbol {
                    name: "Equation".into(),
                    label: Some("eq:foo".into()),
                    kind: LatexSymbolKind::Equation,
                    deprecated: false,
                    full_range: Range::new_simple(1, 0, 1, 44),
                    selection_range: Range::new_simple(1, 16, 1, 30),
                    children: Vec::new(),
                },
            ]
        );
    }

    #[test]
    fn equation_number() {
        let symbols = test_feature(
            LatexSectionSymbolProvider,
            FeatureSpec {
                files: vec![
                    FeatureSpec::file("foo.tex", "\\[\\label{eq:foo}\\]"),
                    FeatureSpec::file(
                        "foo.aux",
                        "\\newlabel{eq:foo}{{\\relax 2.1}{4}{Bar\\relax }{figure.caption.4}{}}",
                    ),
                ],
                main_file: "foo.tex",
                ..FeatureSpec::default()
            },
        );
        assert_eq!(
            symbols,
            vec![LatexSymbol {
                name: "Equation (2.1)".into(),
                label: Some("eq:foo".into()),
                kind: LatexSymbolKind::Equation,
                deprecated: false,
                full_range: Range::new_simple(0, 0, 0, 18),
                selection_range: Range::new_simple(0, 2, 0, 16),
                children: Vec::new(),
            },]
        );
    }

    #[test]
    fn table() {
        let symbols = test_feature(
            LatexSectionSymbolProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file(
                    "foo.tex",
                    "\\begin{table}\\caption{Foo}\\end{table}",
                )],
                main_file: "foo.tex",
                ..FeatureSpec::default()
            },
        );
        assert_eq!(
            symbols,
            vec![LatexSymbol {
                name: "Table: Foo".into(),
                label: None,
                kind: LatexSymbolKind::Table,
                deprecated: false,
                full_range: Range::new_simple(0, 0, 0, 37),
                selection_range: Range::new_simple(0, 0, 0, 37),
                children: Vec::new(),
            },]
        );
    }

    #[test]
    fn figure_number() {
        let symbols = test_feature(
            LatexSectionSymbolProvider,
            FeatureSpec {
                files: vec![
                    FeatureSpec::file(
                        "foo.tex",
                        "\\begin{figure}\\caption{Foo}\\label{fig:foo}\\end{figure}",
                    ),
                    FeatureSpec::file(
                        "foo.aux",
                        "\\newlabel{fig:foo}{{\\relax 2.1}{4}{Bar\\relax }{figure.caption.4}{}}",
                    ),
                ],
                main_file: "foo.tex",
                ..FeatureSpec::default()
            },
        );
        assert_eq!(
            symbols,
            vec![LatexSymbol {
                name: "Figure 2.1: Foo".into(),
                label: Some("fig:foo".into()),
                kind: LatexSymbolKind::Figure,
                deprecated: false,
                full_range: Range::new_simple(0, 0, 0, 54),
                selection_range: Range::new_simple(0, 27, 0, 42),
                children: Vec::new(),
            },]
        );
    }

    #[test]
    fn lemma() {
        let symbols = test_feature(
            LatexSectionSymbolProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file(
                    "foo.tex",
                    "\\newtheorem{lemma}{Lemma}\\begin{lemma}\\end{lemma}",
                )],
                main_file: "foo.tex",
                ..FeatureSpec::default()
            },
        );
        assert_eq!(
            symbols,
            vec![LatexSymbol {
                name: "Lemma".into(),
                label: None,
                kind: LatexSymbolKind::Theorem,
                deprecated: false,
                full_range: Range::new_simple(0, 25, 0, 49),
                selection_range: Range::new_simple(0, 25, 0, 49),
                children: Vec::new(),
            },]
        );
    }

    #[test]
    fn lemma_number() {
        let symbols = test_feature(
            LatexSectionSymbolProvider,
            FeatureSpec {
                files: vec![
                    FeatureSpec::file(
                        "foo.tex",
                        "\\newtheorem{lemma}{Lemma}\n\\begin{lemma}\\label{thm:foo}\\end{lemma}",
                    ),
                    FeatureSpec::file(
                        "foo.aux",
                        "\\newlabel{thm:foo}{{\\relax 2.1}{4}{Bar\\relax }{figure.caption.4}{}}",
                    ),
                ],
                main_file: "foo.tex",
                ..FeatureSpec::default()
            },
        );
        assert_eq!(
            symbols,
            vec![LatexSymbol {
                name: "Lemma 2.1".into(),
                label: Some("thm:foo".into()),
                kind: LatexSymbolKind::Theorem,
                deprecated: false,
                full_range: Range::new_simple(1, 0, 1, 39),
                selection_range: Range::new_simple(1, 13, 1, 28),
                children: Vec::new(),
            },]
        );
    }

    #[test]
    fn lemma_description() {
        let symbols = test_feature(
            LatexSectionSymbolProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file(
                    "foo.tex",
                    "\\newtheorem{lemma}{Lemma}\\begin{lemma}[Foo]\\end{lemma}",
                )],
                main_file: "foo.tex",
                ..FeatureSpec::default()
            },
        );
        assert_eq!(
            symbols,
            vec![LatexSymbol {
                name: "Lemma (Foo)".into(),
                label: None,
                kind: LatexSymbolKind::Theorem,
                deprecated: false,
                full_range: Range::new_simple(0, 25, 0, 54),
                selection_range: Range::new_simple(0, 25, 0, 54),
                children: Vec::new(),
            },]
        );
    }
}
