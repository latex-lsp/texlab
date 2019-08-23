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
pub enum OutlineContextItem {
    Section(String),
    Caption(String),
    Theorem {
        kind: String,
        description: Option<String>,
    },
    Equation,
}

impl OutlineContextItem {
    pub fn reference(self) -> String {
        match self {
            Self::Section(text) => text,
            Self::Caption(text) => text,
            Self::Theorem {
                kind,
                description: None,
            } => kind,
            Self::Theorem {
                kind,
                description: Some(description),
            } => format!("{} ({})", kind, description),
            Self::Equation => "Equation".to_owned(),
        }
    }

    pub fn documentation(self) -> MarkupContent {
        MarkupContent {
            kind: MarkupKind::PlainText,
            value: self.reference().into(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct OutlineContext {
    pub range: Range,
    pub item: OutlineContextItem,
}

impl OutlineContext {
    pub fn parse(view: &DocumentView, position: Position, outline: &Outline) -> Option<Self> {
        if let SyntaxTree::Latex(tree) = &view.document.tree {
            Self::find_caption(position, tree)
                .or_else(|| Self::find_theorem(view, position, tree))
                .or_else(|| Self::find_equation(position, tree))
                .or_else(|| Self::find_section(view, position, outline))
        } else {
            None
        }
    }

    fn find_caption(
        position: Position,
        tree: &LatexSyntaxTree,
    ) -> Option<Self> {
        let caption_env = tree
            .environments
            .iter()
            .filter(|env| env.left.name().map(LatexToken::text) != Some("document"))
            .find(|env| env.range().contains(position))?;

        let caption = tree
            .captions
            .iter()
            .find(|cap| caption_env.range().contains(cap.start()))?;

        let caption_content = &caption.command.args[caption.index];
        let caption_text = Self::extract(caption_content);

        Some(Self {
            range: caption_env.range(),
            item: OutlineContextItem::Caption(caption_text),
        })
    }

    fn find_theorem(
        view: &DocumentView,
        position: Position,
        tree: &LatexSyntaxTree,
    ) -> Option<Self> {
        let env = tree
            .environments
            .iter()
            .find(|env| env.range().contains(position))?;

        let env_name = env.left.name().map(LatexToken::text)?;

        for document in &view.related_documents {
            if let SyntaxTree::Latex(tree) = &document.tree {
                for definition in &tree.theorem_definitions {
                    if env_name == definition.name().text() {
                        let kind = definition
                            .command
                            .args
                            .get(definition.index + 1)
                            .map(|content| Self::extract(&content))
                            .unwrap_or_else(|| Self::titlelize(env_name));

                        let description = env
                            .left
                            .command
                            .options
                            .get(0)
                            .map(|content| Self::extract(&content));

                        return Some(Self {
                            range: env.range(),
                            item: OutlineContextItem::Theorem { kind, description },
                        });
                    }
                }
            }
        }
        None
    }

    fn find_equation(position: Position, tree: &LatexSyntaxTree) -> Option<Self> {
        tree.environments
            .iter()
            .filter(|env| env.left.is_math())
            .map(|env| env.range())
            .find(|range| range.contains(position))
            .map(|range| Self {
                range,
                item: OutlineContextItem::Equation,
            })
    }

    fn find_section(view: &DocumentView, position: Position, outline: &Outline) -> Option<Self> {
        let section = outline.find(&view.document.uri, position)?;
        let content = &section.command.args[section.index];
        Some(Self {
            range: section.range(),
            item: OutlineContextItem::Section(Self::extract(content)),
        })
    }

    fn titlelize(string: &str) -> String {
        let mut chars = string.chars();
        match chars.next() {
            None => String::new(),
            Some(c) => c.to_uppercase().chain(chars).collect(),
        }
    }

    fn extract(content: &LatexGroup) -> String {
        if content.children.len() == 0 || content.right.is_none() {
            return String::new();
        }

        let mut printer = LatexPrinter::new(content.children[0].start());
        for child in &content.children {
            child.accept(&mut printer);
        }
        printer.output
    }
}
