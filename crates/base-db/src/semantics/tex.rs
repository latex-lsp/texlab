use rowan::{ast::AstNode, TextRange};
use rustc_hash::FxHashSet;
use syntax::latex::{self, HasBrack, HasCurly};
use titlecase::titlecase;

use super::Span;

#[derive(Debug, Clone, Default)]
pub struct Semantics {
    pub links: Vec<Link>,
    pub labels: Vec<Label>,
    pub citations: Vec<Citation>,
    pub commands: Vec<Span>,
    pub environments: Vec<Span>,
    pub theorem_definitions: Vec<TheoremDefinition>,
    pub graphics_paths: FxHashSet<String>,
    pub can_be_root: bool,
    pub can_be_compiled: bool,
}

impl Semantics {
    pub fn process_root(&mut self, root: &latex::SyntaxNode) {
        for node in root.descendants_with_tokens() {
            match node {
                latex::SyntaxElement::Node(node) => {
                    self.process_node(&node);
                }
                latex::SyntaxElement::Token(token) => {
                    if token.kind() == latex::COMMAND_NAME {
                        self.commands.push(Span::command(&token));
                    }
                }
            };
        }

        self.can_be_root = self.can_be_compiled
            && !self
                .links
                .iter()
                .any(|link| link.kind == LinkKind::Cls && link.path.text == "subfiles");
    }

    fn process_node(&mut self, node: &latex::SyntaxNode) {
        if let Some(include) = latex::Include::cast(node.clone()) {
            self.process_include(include);
        } else if let Some(import) = latex::Import::cast(node.clone()) {
            self.process_import(import);
        } else if let Some(label) = latex::LabelDefinition::cast(node.clone()) {
            self.process_label_definition(label);
        } else if let Some(label) = latex::LabelReference::cast(node.clone()) {
            self.process_label_reference(label);
        } else if let Some(label) = latex::LabelReferenceRange::cast(node.clone()) {
            self.process_label_reference_range(label);
        } else if let Some(citation) = latex::Citation::cast(node.clone()) {
            self.process_citation(citation);
        } else if let Some(environment) = latex::Environment::cast(node.clone()) {
            self.process_environment(environment);
        } else if let Some(theorem_def) = latex::TheoremDefinition::cast(node.clone()) {
            self.process_theorem_definition(theorem_def);
        } else if let Some(graphics_path) = latex::GraphicsPath::cast(node.clone()) {
            self.process_graphics_path(graphics_path);
        }
    }

    fn process_include(&mut self, include: latex::Include) {
        let Some(list) = include.path_list() else {
            return;
        };

        for path in list.keys() {
            let kind = match include.syntax().kind() {
                latex::PACKAGE_INCLUDE => LinkKind::Sty,
                latex::CLASS_INCLUDE => LinkKind::Cls,
                latex::LATEX_INCLUDE => LinkKind::Tex,
                latex::BIBLATEX_INCLUDE => LinkKind::Bib,
                latex::BIBTEX_INCLUDE => LinkKind::Bib,
                _ => continue,
            };

            self.links.push(Link {
                kind,
                path: Span::from(&path),
                base_dir: None,
            });
        }
    }

    fn process_import(&mut self, import: latex::Import) {
        let Some(mut base_dir) = import
            .directory()
            .and_then(|dir| dir.key())
            .map(|key| key.to_string())
        else {
            return;
        };

        if !base_dir.ends_with('/') {
            base_dir.push('/');
        }

        let Some(path) = import.file().and_then(|path| path.key()) else {
            return;
        };
        let text = format!("{base_dir}{}", path.to_string());
        let range = latex::small_range(&path);

        self.links.push(Link {
            kind: LinkKind::Tex,
            path: Span { text, range },
            base_dir: Some(base_dir),
        });
    }

    fn process_label_definition(&mut self, label: latex::LabelDefinition) {
        let Some(name) = label.name().and_then(|group| group.key()) else {
            return;
        };

        let full_range = latex::small_range(&label);
        let mut objects = Vec::new();
        for node in label.syntax().ancestors() {
            if let Some(section) = latex::Section::cast(node.clone()) {
                let Some(text) = section.name().and_then(|group| group.content_text()) else {
                    continue;
                };
                let range = latex::small_range(&section);
                let prefix = String::from(match section.syntax().kind() {
                    latex::PART => "Part",
                    latex::CHAPTER => "Chapter",
                    latex::SECTION => "Section",
                    latex::SUBSECTION => "Subsection",
                    latex::SUBSUBSECTION => "Subsubsection",
                    latex::PARAGRAPH => "Paragraph",
                    latex::SUBPARAGRAPH => "Subparagraph",
                    _ => unreachable!(),
                });

                let kind = LabelObject::Section { prefix, text };
                objects.push(LabelTarget {
                    object: kind,
                    range,
                });
            } else if let Some(environment) = latex::Environment::cast(node.clone()) {
                let Some(name) = environment
                    .begin()
                    .and_then(|begin| begin.name())
                    .and_then(|group| group.key())
                    .map(|key| key.to_string())
                else {
                    continue;
                };

                let caption = environment
                    .syntax()
                    .children()
                    .filter_map(latex::Caption::cast)
                    .find_map(|node| node.long())
                    .and_then(|node| node.content_text());

                let options = environment
                    .begin()
                    .and_then(|begin| begin.options())
                    .and_then(|options| options.content_text());

                let range = latex::small_range(&environment);
                let kind = LabelObject::Environment {
                    name,
                    options,
                    caption,
                };

                objects.push(LabelTarget {
                    object: kind,
                    range,
                });
            } else if let Some(enum_item) = latex::EnumItem::cast(node.clone()) {
                let range = latex::small_range(&enum_item);
                let kind = LabelObject::EnumItem;
                objects.push(LabelTarget {
                    object: kind,
                    range,
                });
            }
        }

        self.labels.push(Label {
            kind: LabelKind::Definition,
            name: Span::from(&name),
            targets: objects,
            full_range,
        });
    }

    fn process_label_reference(&mut self, label: latex::LabelReference) {
        let Some(name_list) = label.name_list() else {
            return;
        };

        let full_range = latex::small_range(&label);
        for name in name_list.keys() {
            self.labels.push(Label {
                kind: LabelKind::Reference,
                name: Span::from(&name),
                targets: Vec::new(),
                full_range,
            });
        }
    }

    fn process_label_reference_range(&mut self, label: latex::LabelReferenceRange) {
        let full_range = latex::small_range(&label);
        if let Some(from) = label.from().and_then(|group| group.key()) {
            self.labels.push(Label {
                kind: LabelKind::ReferenceRange,
                name: Span::from(&from),
                targets: Vec::new(),
                full_range,
            });
        }

        if let Some(to) = label.to().and_then(|group| group.key()) {
            self.labels.push(Label {
                kind: LabelKind::ReferenceRange,
                name: Span::from(&to),
                targets: Vec::new(),
                full_range,
            });
        }
    }

    fn process_citation(&mut self, citation: latex::Citation) {
        let full_range = latex::small_range(&citation);
        if let Some(list) = citation.key_list() {
            for key in list.keys() {
                self.citations.push(Citation {
                    name: Span::from(&key),
                    full_range,
                });
            }
        }
    }

    fn process_environment(&mut self, environment: latex::Environment) {
        let Some(name) = environment
            .begin()
            .and_then(|begin| begin.name())
            .and_then(|group| group.key())
        else {
            return;
        };

        let name = Span::from(&name);
        self.can_be_compiled = self.can_be_compiled || name.text == "document";
        self.environments.push(name);
    }

    fn process_theorem_definition(&mut self, theorem_def: latex::TheoremDefinition) {
        for name in theorem_def.names() {
            let name = Span::from(&name);
            let heading = theorem_def
                .heading()
                .unwrap_or_else(|| titlecase(&name.text));

            self.theorem_definitions
                .push(TheoremDefinition { name, heading });
        }
    }

    fn process_graphics_path(&mut self, graphics_path: latex::GraphicsPath) {
        for path in graphics_path.path_list().filter_map(|path| path.key()) {
            self.graphics_paths.insert(path.to_string());
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub enum LinkKind {
    Sty,
    Cls,
    Tex,
    Bib,
}

impl LinkKind {
    pub fn extensions(self) -> &'static [&'static str] {
        match self {
            Self::Sty => &["sty"],
            Self::Cls => &["cls"],
            Self::Tex => &["tex"],
            Self::Bib => &["bib"],
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Link {
    pub kind: LinkKind,
    pub path: Span,
    pub base_dir: Option<String>,
}

impl Link {
    pub fn package_name(&self) -> Option<String> {
        match self.kind {
            LinkKind::Sty => Some(format!("{}.sty", self.path.text)),
            LinkKind::Cls => Some(format!("{}.cls", self.path.text)),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub enum LabelKind {
    Definition,
    Reference,
    ReferenceRange,
}

#[derive(Debug, Clone)]
pub struct Label {
    pub kind: LabelKind,
    pub name: Span,
    pub targets: Vec<LabelTarget>,
    pub full_range: TextRange,
}

#[derive(Debug, Clone)]
pub struct LabelTarget {
    pub object: LabelObject,
    pub range: TextRange,
}

#[derive(Debug, Clone)]
pub enum LabelObject {
    Section {
        prefix: String,
        text: String,
    },
    EnumItem,
    Environment {
        name: String,
        options: Option<String>,
        caption: Option<String>,
    },
}

#[derive(Debug, Clone)]
pub struct TheoremDefinition {
    pub name: Span,
    pub heading: String,
}

#[derive(Debug, Clone)]
pub struct Citation {
    pub name: Span,
    pub full_range: TextRange,
}
