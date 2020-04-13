mod enumeration;
mod equation;
mod float;
mod theorem;

use super::types::{LatexSymbol, LatexSymbolKind};
use futures_boxed::boxed;
use std::path::Path;
use texlab_feature::{
    DocumentContent, DocumentView, FeatureProvider, FeatureRequest, Outline, OutlineContext,
    OutlineContextItem,
};
use texlab_protocol::{DocumentSymbolParams, Options, Position, Range, RangeExt};
use texlab_syntax::{latex, CharStream, LatexLabelKind};

fn label_name(table: &latex::SymbolTable, label: Option<&latex::Label>) -> Option<String> {
    label.map(|label| label.names(&table.tree)[0].text().to_owned())
}

fn selection_range(
    table: &latex::SymbolTable,
    full_range: Range,
    label: Option<&latex::Label>,
) -> Range {
    label
        .map(|label| table.tree.range(label.parent))
        .unwrap_or(full_range)
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct LatexSectionSymbolProvider;

impl FeatureProvider for LatexSectionSymbolProvider {
    type Params = DocumentSymbolParams;
    type Output = Vec<LatexSymbol>;

    #[boxed]
    async fn execute<'a>(&'a self, req: &'a FeatureRequest<Self::Params>) -> Self::Output {
        let mut symbols = Vec::new();
        if let DocumentContent::Latex(table) = &req.current().content {
            let mut section_tree =
                build_section_tree(&req.view, table, &req.options, &req.current_dir);
            for symbol in enumeration::symbols(&req.view, table) {
                section_tree.insert_symbol(&symbol);
            }

            for symbol in equation::symbols(&req.view, table) {
                section_tree.insert_symbol(&symbol);
            }

            for symbol in float::symbols(&req.view, table) {
                section_tree.insert_symbol(&symbol);
            }

            for symbol in theorem::symbols(&req.view, table) {
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
    table: &'a latex::SymbolTable,
    options: &'a Options,
    current_dir: &'a Path,
) -> LatexSectionTree<'a> {
    let mut section_tree = LatexSectionTree::from(table);
    section_tree.set_full_text(&view.current.text);
    let end_position = compute_end_position(table, &view.current.text);
    LatexSectionNode::set_full_range(&mut section_tree.children, table, end_position);
    let outline = Outline::analyze(view, options, current_dir);
    for child in &mut section_tree.children {
        child.set_label(view, &outline);
    }
    section_tree
}

fn compute_end_position(table: &latex::SymbolTable, text: &str) -> Position {
    let mut stream = CharStream::new(text);
    while stream.next().is_some() {}
    table
        .environments
        .iter()
        .find(|env| env.left.name(&table.tree).map(latex::Token::text) == Some("document"))
        .map(|env| table.tree.range(env.right.parent).start)
        .unwrap_or(stream.current_position)
}

#[derive(Debug, Clone)]
pub struct LatexSectionNode<'a> {
    pub table: &'a latex::SymbolTable,
    pub section: &'a latex::Section,
    pub full_range: Range,
    full_text: &'a str,
    label: Option<String>,
    number: Option<String>,
    symbols: Vec<LatexSymbol>,
    children: Vec<Self>,
}

impl<'a> LatexSectionNode<'a> {
    fn new(table: &'a latex::SymbolTable, section: &'a latex::Section) -> Self {
        Self {
            table,
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
        self.table
            .tree
            .print_group_content(
                self.section.parent,
                latex::GroupKind::Group,
                self.section.arg_index,
            )
            .unwrap_or_else(|| "Unknown".into())
    }

    fn set_full_range(
        children: &mut Vec<Self>,
        table: &latex::SymbolTable,
        end_position: Position,
    ) {
        for i in 0..children.len() {
            let current_end = children
                .get(i + 1)
                .map(|next| table.tree.range(next.section.parent).start)
                .unwrap_or(end_position);

            let mut current = &mut children[i];
            current.full_range =
                Range::new(table.tree.range(current.section.parent).start, current_end);
            Self::set_full_range(&mut current.children, table, current_end);
        }
    }

    fn set_label(&mut self, view: &DocumentView, outline: &Outline) {
        if let Some(label) = self
            .table
            .labels
            .iter()
            .filter(|label| label.kind == LatexLabelKind::Definition)
            .find(|label| {
                self.full_range
                    .contains(self.table.tree.range(label.parent).start)
            })
        {
            if let Some(ctx) = OutlineContext::parse(view, outline, *label) {
                let mut is_section = false;
                if let OutlineContextItem::Section { text, .. } = &ctx.item {
                    if self.name() == *text {
                        for name in label.names(&self.table.tree) {
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
            child.set_label(view, outline);
        }
    }

    fn insert_section(
        nodes: &mut Vec<Self>,
        table: &'a latex::SymbolTable,
        section: &'a latex::Section,
    ) {
        match nodes.last_mut() {
            Some(parent) => {
                if parent.section.level < section.level {
                    Self::insert_section(&mut parent.children, table, section);
                } else {
                    nodes.push(LatexSectionNode::new(table, section));
                }
            }
            None => {
                nodes.push(LatexSectionNode::new(table, section));
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
            selection_range: self.table.tree.range(self.section.parent),
            children,
        }
    }
}

#[derive(Debug, Clone)]
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

impl<'a> From<&'a latex::SymbolTable> for LatexSectionTree<'a> {
    fn from(table: &'a latex::SymbolTable) -> Self {
        let mut root = Self::new();
        for section in &table.sections {
            LatexSectionNode::insert_section(&mut root.children, table, section);
        }
        root
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    use texlab_feature::FeatureTester;

    #[tokio::test]
    async fn empty_latex_document() {
        let actual_symbols = FeatureTester::new()
            .file("main.tex", "")
            .main("main.tex")
            .test_symbol(LatexSectionSymbolProvider)
            .await;

        assert!(actual_symbols.is_empty());
    }

    #[tokio::test]
    async fn empty_bibtex_document() {
        let actual_symbols = FeatureTester::new()
            .file("main.bib", "")
            .main("main.bib")
            .test_symbol(LatexSectionSymbolProvider)
            .await;

        assert!(actual_symbols.is_empty());
    }

    #[tokio::test]
    async fn subsection() {
        let actual_symbols = FeatureTester::new()
            .file(
                "main.tex",
                indoc!(
                    r#"
                        \section{Foo}
                        \subsection{Bar}\label{sec:bar}
                        \subsection{Baz}
                        \section{Qux}
                    "#
                ),
            )
            .file(
                "main.aux",
                r#"\newlabel{sec:bar}{{\relax 2.1}{4}{Bar\relax }{figure.caption.4}{}}"#,
            )
            .main("main.tex")
            .test_symbol(LatexSectionSymbolProvider)
            .await;

        let expected_symbols = vec![
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
            },
        ];

        assert_eq!(actual_symbols, expected_symbols);
    }

    #[tokio::test]
    async fn section_inside_document_environment() {
        let actual_symbols = FeatureTester::new()
            .file(
                "main.tex",
                indoc!(
                    r#"
                        \begin{document}\section{Foo}\relax
                        \end{document}
                    "#
                ),
            )
            .main("main.tex")
            .test_symbol(LatexSectionSymbolProvider)
            .await;

        let expected_symbols = vec![LatexSymbol {
            name: "Foo".into(),
            label: None,
            kind: LatexSymbolKind::Section,
            deprecated: false,
            full_range: Range::new_simple(0, 16, 1, 0),
            selection_range: Range::new_simple(0, 16, 0, 29),
            children: Vec::new(),
        }];

        assert_eq!(actual_symbols, expected_symbols);
    }

    #[tokio::test]
    async fn enumeration() {
        let actual_symbols = FeatureTester::new()
            .file(
                "main.tex",
                indoc!(
                    r#"
                        \section{Foo}
                        \begin{enumerate}
                        \end{enumerate}
                    "#
                ),
            )
            .main("main.tex")
            .test_symbol(LatexSectionSymbolProvider)
            .await;

        let expected_symbols = vec![LatexSymbol {
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
            }],
        }];

        assert_eq!(actual_symbols, expected_symbols);
    }

    #[tokio::test]
    async fn equation() {
        let actual_symbols = FeatureTester::new()
            .file(
                "main.tex",
                indoc!(
                    r#"
                        \[Foo\]
                        \begin{equation}\label{eq:foo}\end{equation}
                    "#
                ),
            )
            .main("main.tex")
            .test_symbol(LatexSectionSymbolProvider)
            .await;

        let expected_symbols = vec![
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
        ];

        assert_eq!(actual_symbols, expected_symbols);
    }

    #[tokio::test]
    async fn equation_number() {
        let actual_symbols = FeatureTester::new()
            .file("main.tex", r#"\[\label{eq:foo}\]"#)
            .file(
                "main.aux",
                r#"\newlabel{eq:foo}{{\relax 2.1}{4}{Bar\relax }{figure.caption.4}{}}"#,
            )
            .main("main.tex")
            .test_symbol(LatexSectionSymbolProvider)
            .await;

        let expected_symbols = vec![LatexSymbol {
            name: "Equation (2.1)".into(),
            label: Some("eq:foo".into()),
            kind: LatexSymbolKind::Equation,
            deprecated: false,
            full_range: Range::new_simple(0, 0, 0, 18),
            selection_range: Range::new_simple(0, 2, 0, 16),
            children: Vec::new(),
        }];

        assert_eq!(actual_symbols, expected_symbols);
    }

    #[tokio::test]
    async fn table() {
        let actual_symbols = FeatureTester::new()
            .file("main.tex", r#"\begin{table}\caption{Foo}\end{table}"#)
            .main("main.tex")
            .test_symbol(LatexSectionSymbolProvider)
            .await;

        let expected_symbols = vec![LatexSymbol {
            name: "Table: Foo".into(),
            label: None,
            kind: LatexSymbolKind::Table,
            deprecated: false,
            full_range: Range::new_simple(0, 0, 0, 37),
            selection_range: Range::new_simple(0, 0, 0, 37),
            children: Vec::new(),
        }];

        assert_eq!(actual_symbols, expected_symbols);
    }

    #[tokio::test]
    async fn figure_number() {
        let actual_symbols = FeatureTester::new()
            .file(
                "main.tex",
                r#"\begin{figure}\caption{Foo}\label{fig:foo}\end{figure}"#,
            )
            .file(
                "main.aux",
                r#"\newlabel{fig:foo}{{\relax 2.1}{4}{Bar\relax }{figure.caption.4}{}}"#,
            )
            .main("main.tex")
            .test_symbol(LatexSectionSymbolProvider)
            .await;

        let expected_symbols = vec![LatexSymbol {
            name: "Figure 2.1: Foo".into(),
            label: Some("fig:foo".into()),
            kind: LatexSymbolKind::Figure,
            deprecated: false,
            full_range: Range::new_simple(0, 0, 0, 54),
            selection_range: Range::new_simple(0, 27, 0, 42),
            children: Vec::new(),
        }];

        assert_eq!(actual_symbols, expected_symbols);
    }

    #[tokio::test]
    async fn lemma() {
        let actual_symbols = FeatureTester::new()
            .file(
                "main.tex",
                r#"\newtheorem{lemma}{Lemma}\begin{lemma}\end{lemma}"#,
            )
            .main("main.tex")
            .test_symbol(LatexSectionSymbolProvider)
            .await;

        let expected_symbols = vec![LatexSymbol {
            name: "Lemma".into(),
            label: None,
            kind: LatexSymbolKind::Theorem,
            deprecated: false,
            full_range: Range::new_simple(0, 25, 0, 49),
            selection_range: Range::new_simple(0, 25, 0, 49),
            children: Vec::new(),
        }];

        assert_eq!(actual_symbols, expected_symbols);
    }

    #[tokio::test]
    async fn lemma_number() {
        let actual_symbols = FeatureTester::new()
            .file(
                "main.tex",
                indoc!(
                    r#"
                        \newtheorem{lemma}{Lemma}
                        \begin{lemma}\label{thm:foo}\end{lemma}
                    "#
                ),
            )
            .file(
                "main.aux",
                r#"\newlabel{thm:foo}{{\relax 2.1}{4}{Bar\relax }{figure.caption.4}{}}"#,
            )
            .main("main.tex")
            .test_symbol(LatexSectionSymbolProvider)
            .await;

        let expected_symbols = vec![LatexSymbol {
            name: "Lemma 2.1".into(),
            label: Some("thm:foo".into()),
            kind: LatexSymbolKind::Theorem,
            deprecated: false,
            full_range: Range::new_simple(1, 0, 1, 39),
            selection_range: Range::new_simple(1, 13, 1, 28),
            children: Vec::new(),
        }];

        assert_eq!(actual_symbols, expected_symbols);
    }

    #[tokio::test]
    async fn lemma_description() {
        let actual_symbols = FeatureTester::new()
            .file(
                "main.tex",
                r#"\newtheorem{lemma}{Lemma}\begin{lemma}[Foo]\end{lemma}"#,
            )
            .main("main.tex")
            .test_symbol(LatexSectionSymbolProvider)
            .await;

        let expected_symbols = vec![LatexSymbol {
            name: "Lemma (Foo)".into(),
            label: None,
            kind: LatexSymbolKind::Theorem,
            deprecated: false,
            full_range: Range::new_simple(0, 25, 0, 54),
            selection_range: Range::new_simple(0, 25, 0, 54),
            children: Vec::new(),
        }];

        assert_eq!(actual_symbols, expected_symbols);
    }
}
