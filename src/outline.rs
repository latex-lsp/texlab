use crate::{
    feature::DocumentView,
    protocol::{MarkupContent, MarkupKind, Options, Position, Range, RangeExt, Uri},
    syntax::{latex, SyntaxNode},
    workspace::{Document, DocumentContent},
};
use std::{borrow::Cow, collections::HashSet, path::Path};
use titlecase::titlecase;
use OutlineContextItem::*;

#[derive(Debug, Clone, Copy)]
pub struct OutlineSection<'a> {
    pub document: &'a Document,
    pub item: &'a latex::Section,
}

pub struct Outline<'a> {
    sections: Vec<OutlineSection<'a>>,
}

impl<'a> Outline<'a> {
    pub fn find(&'a self, uri: &Uri, pos: Position) -> Option<&'a latex::Section> {
        self.sections
            .iter()
            .filter(|section| section.document.uri == *uri)
            .rev()
            .find(|section| {
                let table = section.document.content.as_latex().unwrap();
                table.tree.graph[section.item.parent].end() <= pos
            })
            .map(|sec| sec.item)
    }

    pub fn analyze(view: &'a DocumentView, options: &Options, current_dir: &Path) -> Self {
        let mut finder = OutlineSectionFinder::default();
        let doc = view
            .snapshot
            .parent(&view.current.uri, options, current_dir)
            .map(|parent| {
                view.related
                    .iter()
                    .find(|doc| doc.uri == parent.uri)
                    .unwrap()
            })
            .unwrap_or(&view.current);

        finder.analyze(view, doc);
        Self {
            sections: finder.sections,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum OutlineItem<'a> {
    Section(&'a latex::Section),
    Include(&'a latex::Include),
}

#[derive(Debug, Default)]
struct OutlineSectionFinder<'a> {
    visited: HashSet<&'a Uri>,
    sections: Vec<OutlineSection<'a>>,
}

impl<'a> OutlineSectionFinder<'a> {
    fn analyze(&mut self, view: &'a DocumentView, doc: &'a Document) {
        if !self.visited.insert(&doc.uri) {
            return;
        }

        if let DocumentContent::Latex(table) = &doc.content {
            let mut items = Vec::new();
            for section in &table.sections {
                items.push(OutlineItem::Section(section));
            }
            for include in &table.includes {
                items.push(OutlineItem::Include(include));
            }
            items.sort_by_key(|item| match item {
                OutlineItem::Include(include) => table.tree.graph[include.parent].start(),
                OutlineItem::Section(section) => table.tree.graph[section.parent].start(),
            });

            for item in items {
                match item {
                    OutlineItem::Section(item) => {
                        let section = OutlineSection {
                            document: doc,
                            item,
                        };
                        self.sections.push(section);
                    }
                    OutlineItem::Include(item) => {
                        for doc in &view.related {
                            for targets in &item.all_targets {
                                if targets.contains(&doc.uri) {
                                    self.analyze(view, doc);
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

    pub fn parse(env_name: &str) -> Option<Self> {
        match env_name {
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
        prefix: Cow<'static, str>,
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
        match &self.number {
            Some(number) => match &self.item {
                Section { prefix, text } => format!("{} {} ({})", prefix, number, text),
                Caption { kind: None, text } => format!("{} {}", number, text),
                Caption {
                    kind: Some(kind),
                    text,
                } => format!("{} {}: {}", kind.as_str(), number, text),
                Theorem {
                    kind,
                    description: None,
                } => format!("{} {}", kind, number),
                Theorem {
                    kind,
                    description: Some(description),
                } => format!("{} {} ({})", kind, number, description),
                Equation => format!("Equation ({})", number),
                Item => format!("Item {}", number),
            },
            None => match &self.item {
                Section { prefix, text } => format!("{} ({})", prefix, text),
                Caption { kind: None, text } => text.clone(),
                Caption {
                    kind: Some(kind),
                    text,
                } => format!("{}: {}", kind.as_str(), text),
                Theorem {
                    kind,
                    description: None,
                } => kind.into(),
                Theorem {
                    kind,
                    description: Some(description),
                } => format!("{} ({})", kind, description),
                Equation => "Equation".into(),
                Item => "Item".into(),
            },
        }
    }

    pub fn detail(&self) -> Option<String> {
        match &self.item {
            Section { .. } | Theorem { .. } | Equation | Item => Some(self.reference()),
            Caption {
                kind: Some(kind), ..
            } => {
                let result = match &self.number {
                    Some(number) => format!("{} {}", kind.as_str(), number),
                    None => kind.as_str().to_owned(),
                };
                Some(result)
            }
            Caption { .. } => None,
        }
    }

    pub fn documentation(&self) -> MarkupContent {
        MarkupContent {
            kind: MarkupKind::PlainText,
            value: self.reference(),
        }
    }

    pub fn parse(view: &DocumentView, outline: &Outline, label: latex::Label) -> Option<Self> {
        if let DocumentContent::Latex(table) = &view.current.content {
            Self::find_caption(view, table, label)
                .or_else(|| Self::find_theorem(view, table, label))
                .or_else(|| Self::find_equation(view, table, label))
                .or_else(|| Self::find_item(view, table, label))
                .or_else(|| Self::find_section(view, outline, table, label))
        } else {
            None
        }
    }

    fn find_caption(
        view: &DocumentView,
        table: &latex::SymbolTable,
        label: latex::Label,
    ) -> Option<Self> {
        let label_range = table.tree.graph[label.parent].range();
        let caption_env = table
            .environments
            .iter()
            .filter(|env| !env.is_root(&table.tree))
            .find(|env| env.range(&table.tree).contains(label_range.start))?;

        let caption = table
            .captions
            .iter()
            .find(|cap| table.is_direct_child(*caption_env, table.tree.range(cap.parent).start))?;

        let caption_text = table.tree.print_group_content(
            caption.parent,
            latex::GroupKind::Group,
            caption.arg_index,
        )?;

        let caption_kind = caption_env
            .left
            .name(&table.tree)
            .map(latex::Token::text)
            .and_then(OutlineCaptionKind::parse);

        Some(Self {
            range: caption_env.range(&table.tree),
            number: Self::find_number(view, table, label),
            item: Caption {
                kind: caption_kind,
                text: caption_text,
            },
        })
    }

    fn find_theorem(
        view: &DocumentView,
        table: &latex::SymbolTable,
        label: latex::Label,
    ) -> Option<Self> {
        let label_range = table.tree.range(label.parent);
        let env = table
            .environments
            .iter()
            .find(|env| env.range(&table.tree).contains(label_range.start))?;

        let env_name = env.left.name(&table.tree).map(latex::Token::text)?;

        for doc in &view.related {
            if let DocumentContent::Latex(table) = &doc.content {
                for def in &table.theorem_definitions {
                    if env_name == def.name(&table.tree).text() {
                        let kind = table
                            .tree
                            .print_group_content(
                                def.parent,
                                latex::GroupKind::Group,
                                def.arg_index + 1,
                            )
                            .unwrap_or_else(|| titlecase(&env_name));

                        let description = table.tree.print_group_content(
                            env.left.parent,
                            latex::GroupKind::Options,
                            0,
                        );

                        return Some(Self {
                            range: env.range(&table.tree),
                            number: Self::find_number(view, table, label),
                            item: Theorem { kind, description },
                        });
                    }
                }
            }
        }
        None
    }

    fn find_equation(
        view: &DocumentView,
        table: &latex::SymbolTable,
        label: latex::Label,
    ) -> Option<Self> {
        let label_range = table.tree.range(label.parent);
        table
            .environments
            .iter()
            .filter(|env| env.left.is_math(&table.tree))
            .map(|env| env.range(&table.tree))
            .find(|range| range.contains(label_range.start))
            .map(|range| Self {
                range,
                number: Self::find_number(view, table, label),
                item: Equation,
            })
    }

    fn find_item(
        view: &DocumentView,
        table: &latex::SymbolTable,
        label: latex::Label,
    ) -> Option<Self> {
        let label_range = table.tree.range(label.parent);
        struct LatexItemNode {
            item: latex::Item,
            range: Range,
        }

        let enumeration = table
            .environments
            .iter()
            .filter(|env| env.left.is_enum(&table.tree))
            .find(|env| env.range(&table.tree).contains(label_range.start))?;

        let mut item_nodes: Vec<_> = table
            .items
            .iter()
            .filter(|item| table.is_enum_item(*enumeration, **item))
            .map(|item| LatexItemNode {
                item: *item,
                range: Range::default(),
            })
            .collect();

        for i in 0..item_nodes.len() {
            let start = table.tree.range(item_nodes[i].item.parent).start;
            let end = item_nodes
                .get(i + 1)
                .map(|node| table.tree.range(node.item.parent).start)
                .unwrap_or_else(|| table.tree.range(enumeration.right.parent).start);
            item_nodes[i].range = Range::new(start, end);
        }

        let node = item_nodes
            .iter()
            .find(|node| node.range.contains(label_range.start))?;

        let number = node
            .item
            .name(&table.tree)
            .or_else(|| Self::find_number(view, table, label));

        Some(Self {
            range: enumeration.range(&table.tree),
            number,
            item: Item,
        })
    }

    fn find_section(
        view: &DocumentView,
        outline: &Outline,
        table: &latex::SymbolTable,
        label: latex::Label,
    ) -> Option<Self> {
        let label_range = table.tree.range(label.parent);
        let section = outline.find(&view.current.uri, label_range.start)?;
        Some(Self {
            range: table.tree.range(section.parent),
            number: Self::find_number(view, table, label),
            item: Section {
                prefix: section.prefix.clone(),
                text: table.tree.print_group_content(
                    section.parent,
                    latex::GroupKind::Group,
                    section.arg_index,
                )?,
            },
        })
    }

    pub fn find_number(
        view: &DocumentView,
        table: &latex::SymbolTable,
        label: latex::Label,
    ) -> Option<String> {
        let label_names = label.names(&table.tree);
        if label_names.len() != 1 {
            return None;
        }

        for doc in &view.related {
            if let DocumentContent::Latex(table) = &doc.content {
                for numbering in &table.label_numberings {
                    if numbering.name(&table.tree).text() == label_names[0].text() {
                        return Some(numbering.number.clone());
                    }
                }
            }
        }
        None
    }
}
