use rowan::{ast::AstNode, TextRange};
use rustc_hash::FxHashSet;
use syntax::latex::{self, HasBrack, HasCurly};
use titlecase::titlecase;

use parser::SyntaxConfig;

use super::Span;
use crate::semantics::tex::latex::SyntaxToken;

fn maybe_prepend_prefix(
    map: &Vec<(String, String)>,
    command: &Option<SyntaxToken>,
    name: &Span,
) -> Span {
    match command {
        Some(x) => Span::new(
            map.iter()
                .find_map(|(k, v)| if k == &x.text()[1..] { Some(v) } else { None })
                .unwrap_or(&String::new())
                .to_owned()
                + &name.text,
            name.range,
        ),
        None => name.clone(),
    }
}

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
    pub diagnostic_suppressions: Vec<TextRange>,
    pub warning_suppression_ranges: Vec<(TextRange, TextRange)>,
    pub bibitems: FxHashSet<Span>,
}

impl Semantics {
    pub fn process_root(&mut self, conf: &SyntaxConfig, root: &latex::SyntaxNode) {
        for node in root.descendants_with_tokens() {
            match node {
                latex::SyntaxElement::Node(node) => {
                    self.process_node(conf, &node);
                }
                latex::SyntaxElement::Token(token) => match token.kind() {
                    latex::COMMAND_NAME => {
                        self.commands.push(Span::command(&token));
                    }
                    latex::COMMENT if token.text().contains("texlab: ignore") => {
                        self.diagnostic_suppressions.push(token.text_range());
                    }
                    latex::COMMENT if token.text().contains("texlab: warnings off") => {
                        let start_range = token.text_range();
                        let mut current = token.clone();
                        while let Some(next) = current.next_token() {
                            current = next;
                            if current.kind() == latex::COMMENT
                                && current.text().contains("texlab: warnings on")
                            {
                                self.warning_suppression_ranges
                                    .push((start_range, current.text_range()));
                                break;
                            }
                        }
                    }
                    _ => {}
                },
            };
        }

        self.can_be_root = self.can_be_compiled
            && !self
                .links
                .iter()
                .any(|link| link.kind == LinkKind::Cls && link.path.text == "subfiles");
    }

    fn process_node(&mut self, conf: &SyntaxConfig, node: &latex::SyntaxNode) {
        if let Some(include) = latex::Include::cast(node.clone()) {
            self.process_include(include);
        } else if let Some(import) = latex::Import::cast(node.clone()) {
            self.process_import(import);
        } else if let Some(label) = latex::LabelDefinition::cast(node.clone()) {
            self.process_label_definition(conf, label);
        } else if let Some(label) = latex::LabelReference::cast(node.clone()) {
            self.process_label_reference(conf, label);
        } else if let Some(label) = latex::LabelReferenceRange::cast(node.clone()) {
            self.process_label_reference_range(conf, label);
        } else if let Some(citation) = latex::Citation::cast(node.clone()) {
            self.process_citation(citation);
        } else if let Some(environment) = latex::Environment::cast(node.clone()) {
            self.process_environment(environment);
        } else if let Some(theorem_def) = latex::TheoremDefinition::cast(node.clone()) {
            self.process_theorem_definition(theorem_def);
        } else if let Some(graphics_path) = latex::GraphicsPath::cast(node.clone()) {
            self.process_graphics_path(graphics_path);
        } else if let Some(bibitem) = latex::BibItem::cast(node.clone()) {
            self.process_bibitem(bibitem);
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

    fn process_label_definition(&mut self, conf: &SyntaxConfig, label: latex::LabelDefinition) {
        let Some(name) = label.name().and_then(|group| group.key()) else {
            return;
        };

        let name = Span::from(&name);
        if name.text.contains('#') {
            return;
        }

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
            cmd: label.command().map(|x| x.text()[1..].to_string()),
            name: maybe_prepend_prefix(&conf.label_definition_prefixes, &label.command(), &name),
            targets: objects,
            full_range,
        });
    }

    fn process_label_reference(&mut self, conf: &SyntaxConfig, label: latex::LabelReference) {
        let Some(name_list) = label.name_list() else {
            return;
        };

        let full_range = latex::small_range(&label);
        for name in name_list.keys() {
            let name = Span::from(&name);
            if !name.text.contains('#') {
                self.labels.push(Label {
                    kind: LabelKind::Reference,
                    cmd: label.command().map(|x| x.text()[1..].to_string()),
                    name: maybe_prepend_prefix(
                        &conf.label_reference_prefixes,
                        &label.command(),
                        &name,
                    ),
                    targets: Vec::new(),
                    full_range,
                });
            }
        }
    }

    fn process_label_reference_range(
        &mut self,
        conf: &SyntaxConfig,
        label: latex::LabelReferenceRange,
    ) {
        let full_range = latex::small_range(&label);
        if let Some(from) = label.from().and_then(|group| group.key()) {
            let name = Span::from(&from);
            if !name.text.contains('#') {
                self.labels.push(Label {
                    kind: LabelKind::ReferenceRange,
                    cmd: label.command().map(|x| x.text()[1..].to_string()),
                    name: maybe_prepend_prefix(
                        &conf.label_reference_prefixes,
                        &label.command(),
                        &name,
                    ),
                    targets: Vec::new(),
                    full_range,
                });
            }
        }

        if let Some(to) = label.to().and_then(|group| group.key()) {
            let name = Span::from(&to);
            if !name.text.contains('#') {
                self.labels.push(Label {
                    kind: LabelKind::ReferenceRange,
                    cmd: label.command().map(|x| x.text()[1..].to_string()),
                    name: maybe_prepend_prefix(
                        &conf.label_reference_prefixes,
                        &label.command(),
                        &name,
                    ),
                    targets: Vec::new(),
                    full_range,
                });
            }
        }
    }

    fn process_citation(&mut self, citation: latex::Citation) {
        let full_range = latex::small_range(&citation);
        if let Some(list) = citation.key_list() {
            for key in list.keys() {
                let name = Span::from(&key);
                if !name.text.contains('#') {
                    self.citations.push(Citation {
                        name: Span::from(&key),
                        full_range,
                    });
                }
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

    fn process_bibitem(&mut self, bibitem: latex::BibItem) {
        if let Some(name) = bibitem.name() {
            if let Some(key) = name.key() {
                self.bibitems.insert(Span::from(&key));
            }
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
    pub cmd: Option<String>,
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
