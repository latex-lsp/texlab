use super::{Document, DocumentView};
use crate::range::RangeExt;
use crate::syntax::*;
use crate::workspace::Uri;
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
            for section in &tree.structure.sections {
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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum OutlineCaptionKind {
    Figure,
    Table,
    Listing,
    Algorithm,
}

impl OutlineCaptionKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Figure => "Figure",
            Self::Table => "Table",
            Self::Listing => "Listing",
            Self::Algorithm => "Algorithm",
        }
    }

    pub fn parse(environment_name: &str) -> Option<Self> {
        match environment_name {
            "figure" | "subfigure" => Some(Self::Figure),
            "table" | "subtable" => Some(Self::Table),
            "listing" | "lstlisting" => Some(Self::Listing),
            "algorithm" => Some(Self::Algorithm),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum OutlineContextItem {
    Section {
        prefix: &'static str,
        text: String,
    },
    Caption {
        kind: Option<OutlineCaptionKind>,
        text: String,
    },
    Theorem {
        kind: String,
        description: Option<String>,
    },
    Equation,
    Item,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct OutlineContext {
    pub range: Range,
    pub number: Option<String>,
    pub item: OutlineContextItem,
}

impl OutlineContext {
    pub fn reference(&self) -> String {
        match (&self.number, &self.item) {
            (Some(number), OutlineContextItem::Section { prefix, text }) => {
                format!("{} {} ({})", prefix, number, text)
            }
            (Some(number), OutlineContextItem::Caption { kind: None, text }) => {
                format!("{} {}", number, text)
            }
            (
                Some(number),
                OutlineContextItem::Caption {
                    kind: Some(kind),
                    text,
                },
            ) => format!("{} {}: {}", kind.as_str(), number, text),
            (
                Some(number),
                OutlineContextItem::Theorem {
                    kind,
                    description: None,
                },
            ) => format!("{} {}", kind, number),
            (
                Some(number),
                OutlineContextItem::Theorem {
                    kind,
                    description: Some(description),
                },
            ) => format!("{} {} ({})", kind, number, description),
            (Some(number), OutlineContextItem::Equation) => format!("Equation ({})", number),
            (Some(number), OutlineContextItem::Item) => format!("Item {}", number),
            (None, OutlineContextItem::Section { prefix, text }) => {
                format!("{} ({})", prefix, text)
            }
            (None, OutlineContextItem::Caption { kind: None, text }) => text.clone(),
            (
                None,
                OutlineContextItem::Caption {
                    kind: Some(kind),
                    text,
                },
            ) => format!("{}: {}", kind.as_str(), text),
            (
                None,
                OutlineContextItem::Theorem {
                    kind,
                    description: None,
                },
            ) => kind.to_owned(),
            (
                None,
                OutlineContextItem::Theorem {
                    kind,
                    description: Some(description),
                },
            ) => format!("{} ({})", kind, description),
            (None, OutlineContextItem::Equation) => "Equation".to_owned(),
            (None, OutlineContextItem::Item) => "Item".to_owned(),
        }
    }

    pub fn detail(&self) -> Option<String> {
        match &self.item {
            OutlineContextItem::Section { .. }
            | OutlineContextItem::Theorem { .. }
            | OutlineContextItem::Equation
            | OutlineContextItem::Item => Some(self.reference()),
            OutlineContextItem::Caption {
                kind: Some(kind), ..
            } => Some(match &self.number {
                Some(number) => format!("{} {}", kind.as_str(), number),
                None => kind.as_str().to_owned(),
            }),
            OutlineContextItem::Caption { .. } => None,
        }
    }

    pub fn documentation(&self) -> MarkupContent {
        MarkupContent {
            kind: MarkupKind::PlainText,
            value: self.reference(),
        }
    }

    pub fn parse(view: &DocumentView, label: &LatexLabel, outline: &Outline) -> Option<Self> {
        if let SyntaxTree::Latex(tree) = &view.document.tree {
            Self::find_caption(view, label, tree)
                .or_else(|| Self::find_theorem(view, label, tree))
                .or_else(|| Self::find_equation(view, label, tree))
                .or_else(|| Self::find_item(view, label, tree))
                .or_else(|| Self::find_section(view, label, outline))
        } else {
            None
        }
    }

    fn find_caption(
        view: &DocumentView,
        label: &LatexLabel,
        tree: &LatexSyntaxTree,
    ) -> Option<Self> {
        let caption_env = tree
            .env
            .environments
            .iter()
            .filter(|env| env.left.name().map(LatexToken::text) != Some("document"))
            .find(|env| env.range().contains(label.start()))?;

        let caption = tree
            .structure
            .captions
            .iter()
            .find(|cap| tree.is_direct_child(caption_env, cap.start()))?;

        let caption_content = &caption.command.args[caption.index];
        let caption_text = extract_group(caption_content);
        let caption_kind = caption_env
            .left
            .name()
            .map(LatexToken::text)
            .and_then(OutlineCaptionKind::parse);

        Some(Self {
            range: caption_env.range(),
            number: Self::find_number(view, label),
            item: OutlineContextItem::Caption {
                kind: caption_kind,
                text: caption_text,
            },
        })
    }

    fn find_theorem(
        view: &DocumentView,
        label: &LatexLabel,
        tree: &LatexSyntaxTree,
    ) -> Option<Self> {
        let env = tree
            .env
            .environments
            .iter()
            .find(|env| env.range().contains(label.start()))?;

        let env_name = env.left.name().map(LatexToken::text)?;

        for document in &view.related_documents {
            if let SyntaxTree::Latex(tree) = &document.tree {
                for definition in &tree.math.theorem_definitions {
                    if env_name == definition.name().text() {
                        let kind = definition
                            .command
                            .args
                            .get(definition.index + 1)
                            .map(|content| extract_group(&content))
                            .unwrap_or_else(|| titlelize(env_name));

                        let description = env
                            .left
                            .command
                            .options
                            .get(0)
                            .map(|content| extract_group(&content));

                        return Some(Self {
                            range: env.range(),
                            number: Self::find_number(view, label),
                            item: OutlineContextItem::Theorem { kind, description },
                        });
                    }
                }
            }
        }
        None
    }

    fn find_equation(
        view: &DocumentView,
        label: &LatexLabel,
        tree: &LatexSyntaxTree,
    ) -> Option<Self> {
        tree.env
            .environments
            .iter()
            .filter(|env| env.left.is_math())
            .map(|env| env.range())
            .find(|range| range.contains(label.start()))
            .map(|range| Self {
                range,
                number: Self::find_number(view, label),
                item: OutlineContextItem::Equation,
            })
    }

    fn find_item(view: &DocumentView, label: &LatexLabel, tree: &LatexSyntaxTree) -> Option<Self> {
        struct LatexItemNode<'a> {
            item: &'a LatexItem,
            range: Range,
        }

        let enumeration = tree
            .env
            .environments
            .iter()
            .find(|env| env.left.is_enum() && env.range().contains(label.start()))?;

        let mut item_nodes: Vec<_> = tree
            .structure
            .items
            .iter()
            .filter(|item| tree.is_enumeration_item(enumeration, item))
            .map(|item| LatexItemNode {
                item,
                range: Range::default(),
            })
            .collect();

        for i in 0..item_nodes.len() {
            let start = item_nodes[i].item.start();
            let end = item_nodes
                .get(i + 1)
                .map(|node| node.item.start())
                .unwrap_or_else(|| enumeration.right.start());
            item_nodes[i].range = Range::new(start, end);
        }

        let node = item_nodes
            .iter()
            .find(|node| node.range.contains(label.start()))?;

        let number = node.item.name().or_else(|| Self::find_number(view, label));

        Some(Self {
            range: enumeration.range(),
            number,
            item: OutlineContextItem::Item,
        })
    }

    fn find_section(view: &DocumentView, label: &LatexLabel, outline: &Outline) -> Option<Self> {
        let section = outline.find(&view.document.uri, label.start())?;
        let content = &section.command.args[section.index];
        Some(Self {
            range: section.range(),
            number: Self::find_number(view, label),
            item: OutlineContextItem::Section {
                prefix: section.prefix,
                text: extract_group(content),
            },
        })
    }

    pub fn find_number(view: &DocumentView, label: &LatexLabel) -> Option<String> {
        let label_names = label.names();
        if label_names.len() != 1 {
            return None;
        }

        for document in &view.related_documents {
            if let SyntaxTree::Latex(tree) = &document.tree {
                for numbering in &tree.structure.label_numberings {
                    if numbering.name().text() == label_names[0].text() {
                        return Some(numbering.number.clone());
                    }
                }
            }
        }
        None
    }
}
