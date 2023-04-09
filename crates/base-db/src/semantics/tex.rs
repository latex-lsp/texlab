use rowan::{ast::AstNode, TextLen};
use rustc_hash::FxHashSet;
use syntax::latex::{self, HasBrack, HasCurly};
use text_size::TextRange;

use super::Span;

#[derive(Debug, Clone, Default)]
pub struct Semantics {
    pub links: Vec<Link>,
    pub labels: Vec<Label>,
    pub commands: Vec<(TextRange, String)>,
    pub environments: FxHashSet<String>,
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
                        let range = token.text_range();
                        let range = TextRange::new(range.start() + "\\".text_len(), range.end());
                        let text = String::from(&token.text()[1..]);
                        self.commands.push((range, text));
                    }
                }
            };
        }

        self.can_be_root = self
            .links
            .iter()
            .filter(|link| link.kind == LinkKind::Cls)
            .any(|link| link.path.text != "subfiles");

        self.can_be_compiled = self.can_be_root || self.environments.contains("document");
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
        } else if let Some(environment) = latex::Environment::cast(node.clone()) {
            self.process_environment(environment);
        } else if let Some(theorem_def) = latex::TheoremDefinition::cast(node.clone()) {
            self.process_theorem_definition(theorem_def);
        }
    }

    fn process_include(&mut self, include: latex::Include) {
        let Some(list) = include.path_list() else { return };

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
            .map(|key| key.to_string()) else { return };

        if !base_dir.ends_with('/') {
            base_dir.push('/');
        }

        let Some(path) = import.file().and_then(|path| path.key()) else { return };

        self.links.push(Link {
            kind: LinkKind::Tex,
            path: Span::from(&path),
            base_dir: Some(base_dir),
        });
    }

    fn process_label_definition(&mut self, label: latex::LabelDefinition) {
        let Some(name) = label.name().and_then(|group| group.key()) else { return };

        let mut objects = Vec::new();
        for node in label.syntax().ancestors() {
            if let Some(section) = latex::Section::cast(node.clone()) {
                let Some(text) = section.name().and_then(|group| group.content_text()) else { continue };
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
                let Some(name) = environment.begin()
                    .and_then(|begin| begin.name())
                    .and_then(|group| group.key())
                    .map(|key| key.to_string()) else { continue };

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
        });
    }

    fn process_label_reference(&mut self, label: latex::LabelReference) {
        let Some(name_list) = label.name_list() else { return };

        for name in name_list.keys() {
            self.labels.push(Label {
                kind: LabelKind::Reference,
                name: Span::from(&name),
                targets: Vec::new(),
            });
        }
    }

    fn process_label_reference_range(&mut self, label: latex::LabelReferenceRange) {
        if let Some(from) = label.from().and_then(|group| group.key()) {
            self.labels.push(Label {
                kind: LabelKind::ReferenceRange,
                name: Span::from(&from),
                targets: Vec::new(),
            });
        }

        if let Some(to) = label.to().and_then(|group| group.key()) {
            self.labels.push(Label {
                kind: LabelKind::ReferenceRange,
                name: Span::from(&to),
                targets: Vec::new(),
            });
        }
    }

    fn process_environment(&mut self, environment: latex::Environment) {
        let Some(name) = environment
            .begin()
            .and_then(|begin| begin.name())
            .and_then(|group| group.key()) else { return };

        self.environments.insert(String::from(name.syntax().text()));
    }

    fn process_theorem_definition(&mut self, theorem_def: latex::TheoremDefinition) {
        let Some(name) = theorem_def.name().and_then(|name| name.key()) else { return };

        let Some(description) = theorem_def
            .description()
            .and_then(|group| group.content_text()) else { return };

        self.theorem_definitions.push(TheoremDefinition {
            name: Span::from(&name),
            description,
        });
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
    pub description: String,
}
