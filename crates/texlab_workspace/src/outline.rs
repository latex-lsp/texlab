use super::{Document, DocumentView};
use lsp_types::*;
use std::collections::HashSet;
use texlab_syntax::*;

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct Outline<'a> {
    sections: Vec<OutlineSection<'a>>,
}

impl<'a> Outline<'a> {
    fn new(sections: Vec<OutlineSection<'a>>) -> Self {
        Self { sections }
    }

    pub fn find(&self, uri: &Uri, position: Position) -> Option<&'a LatexSection> {
        self.sections
            .iter()
            .filter(|sec| sec.document.uri == *uri)
            .rev()
            .find(|sec| sec.item.end() <= position)
            .map(|sec| sec.item)
    }
}

impl<'a> From<&'a DocumentView> for Outline<'a> {
    fn from(view: &'a DocumentView) -> Self {
        let mut finder = OutlineSectionFinder::default();
        let document = if let Some(parent) = view.workspace.find_parent(&view.document.uri) {
            view.related_documents
                .iter()
                .find(|doc| doc.uri == parent.uri)
                .unwrap()
        } else {
            &view.document
        };
        finder.analyze(view, &document);
        Outline::new(finder.sections)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct OutlineSection<'a> {
    pub document: &'a Document,
    pub item: &'a LatexSection,
}

impl<'a> OutlineSection<'a> {
    fn new(document: &'a Document, item: &'a LatexSection) -> Self {
        Self { document, item }
    }
}

#[derive(Debug, Default)]
struct OutlineSectionFinder<'a> {
    visited: HashSet<&'a Uri>,
    sections: Vec<OutlineSection<'a>>,
}

impl<'a> OutlineSectionFinder<'a> {
    fn analyze(&mut self, view: &'a DocumentView, document: &'a Document) {
        if !self.visited.insert(&document.uri) {
            return;
        }

        if let SyntaxTree::Latex(tree) = &document.tree {
            let mut items = Vec::new();
            for section in &tree.sections {
                items.push(OutlineItem::Section(section));
            }
            for include in &tree.includes {
                items.push(OutlineItem::Include(include));
            }
            items.sort_by_key(SyntaxNode::start);

            for item in items {
                match item {
                    OutlineItem::Section(item) => {
                        let section = OutlineSection::new(document, item);
                        self.sections.push(section);
                    }
                    OutlineItem::Include(item) => {
                        for document in &view.related_documents {
                            for targets in &item.all_targets {
                                if targets.contains(&document.uri) {
                                    self.analyze(view, document);
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum OutlineItem<'a> {
    Section(&'a LatexSection),
    Include(&'a LatexInclude),
}

impl<'a> SyntaxNode for OutlineItem<'a> {
    fn range(&self) -> Range {
        match self {
            OutlineItem::Section(section) => section.range(),
            OutlineItem::Include(include) => include.range(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct OutlineContext {
    pub section: Option<String>,
    pub caption: Option<String>,
}

impl OutlineContext {
    pub fn find(outline: &Outline, document: &Document, position: Position) -> Self {
        let section = Self::find_section(outline, document, position);
        let caption = Self::find_caption(document, position);
        Self { section, caption }
    }

    fn find_section(outline: &Outline, document: &Document, position: Position) -> Option<String> {
        let section = outline.find(&document.uri, position)?;
        let content = &section.command.args[section.index];
        Some(Self::extract(document, content)?)
    }

    fn find_caption(document: &Document, position: Position) -> Option<String> {
        if let SyntaxTree::Latex(tree) = &document.tree {
            let environment = tree
                .environments
                .iter()
                .filter(|env| env.left.name().map(LatexToken::text) != Some("document"))
                .find(|env| env.range().contains(position))?;

            let caption = tree
                .captions
                .iter()
                .find(|cap| environment.range().contains(cap.start()))?;
            let content = &caption.command.args[caption.index];
            Some(Self::extract(document, content)?)
        } else {
            None
        }
    }

    fn extract(document: &Document, content: &LatexGroup) -> Option<String> {
        let right = content.right.as_ref()?;
        let range = Range::new_simple(
            content.left.start().line,
            content.left.start().character + 1,
            right.end().line,
            right.end().character - 1,
        );
        Some(CharStream::extract(&document.text, range))
    }

    pub fn documentation(&self) -> Option<MarkupContent> {
        let text = match (&self.section, &self.caption) {
            (Some(section), Some(caption)) => format!("*{}*  \n{}", section, caption),
            (Some(section), None) => format!("*{}*", section),
            (None, Some(caption)) => caption.to_owned(),
            (None, None) => return None,
        };

        Some(MarkupContent {
            kind: MarkupKind::Markdown,
            value: text.into(),
        })
    }
}
