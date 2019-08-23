use super::{Document, DocumentView};
use crate::syntax::*;
use crate::workspace::eq_uri;
use lsp_types::*;
use std::collections::HashSet;

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
                                if targets.iter().any(|uri| eq_uri(uri, &document.uri)) {
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
pub struct OutlineCaption {
    pub range: Range,
    pub text: String,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct OutlineTheorem {
    pub range: Range,
    pub kind: String,
    pub description: Option<String>,
}

impl OutlineTheorem {
    pub fn to_string(&self) -> String {
        match &self.description {
            Some(description) => format!("{} ({})", self.kind, description),
            None => self.kind.clone(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct OutlineContext {
    pub section: Option<String>,
    pub caption: Option<OutlineCaption>,
    pub theorem: Option<OutlineTheorem>,
    pub equation: Option<Range>,
}

impl OutlineContext {
    pub fn find(outline: &Outline, view: &DocumentView, position: Position) -> Self {
        let section = Self::find_section(outline, &view.document, position);
        let caption = Self::find_caption(&view.document, position);
        let theorem = Self::find_theorem(view, position);
        let equation = Self::find_equation(view, position);
        Self {
            section,
            caption,
            theorem,
            equation,
        }
    }

    fn find_section(outline: &Outline, document: &Document, position: Position) -> Option<String> {
        let section = outline.find(&document.uri, position)?;
        let content = &section.command.args[section.index];
        Some(Self::extract(document, content)?)
    }

    fn find_caption(document: &Document, position: Position) -> Option<OutlineCaption> {
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
            let text = Self::extract(document, content)?;
            Some(OutlineCaption {
                range: environment.range(),
                text,
            })
        } else {
            None
        }
    }

    fn find_theorem(view: &DocumentView, position: Position) -> Option<OutlineTheorem> {
        if let SyntaxTree::Latex(tree) = &view.document.tree {
            let environment = tree
                .environments
                .iter()
                .find(|env| env.range().contains(position))?;

            let environment_name = environment.left.name().map(LatexToken::text)?;

            for document in &view.related_documents {
                if let SyntaxTree::Latex(tree) = &document.tree {
                    for definition in &tree.theorem_definitions {
                        if environment_name == definition.name().text() {
                            let kind = definition
                                .command
                                .args
                                .get(definition.index + 1)
                                .and_then(|group| Self::extract(document, group))
                                .unwrap_or_else(|| Self::titlelize(environment_name));

                            let description = environment
                                .left
                                .command
                                .options
                                .get(0)
                                .and_then(|opts| Self::extract(&view.document, opts));

                            return Some(OutlineTheorem { range: environment.range(), kind, description });
                        }
                    }
                }
            }
        }
        None
    }

    fn find_equation(view: &DocumentView, position: Position) -> Option<Range> {
        if let SyntaxTree::Latex(tree) = &view.document.tree {
            tree.environments.iter()
            .filter(|env| env.left.is_math())
            .map(|env| env.range())
            .find(|range| range.contains(position))
        } else {
            None
        }
    }

    fn titlelize(string: &str) -> String {
        let mut chars = string.chars();
        match chars.next() {
            None => String::new(),
            Some(c) => c.to_uppercase().chain(chars).collect(),
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

    pub fn formatted_reference(&self) -> Option<MarkupContent> {
        let text: String = match self.theorem.as_ref().map(OutlineTheorem::to_string) {
            Some(documentation) => Some(documentation),
            None => self
                .caption
                .as_ref()
                .map(|cap| &cap.text)
                .or(self.section.as_ref())
                .map(ToOwned::to_owned),
        }?;

        Some(MarkupContent {
            kind: MarkupKind::Markdown,
            value: text.into(),
        })
    }
}
