use std::str::FromStr;

use base_db::{deps::Project, semantics::Span, util::FloatKind, Config, SymbolEnvironmentConfig};
use rowan::ast::AstNode;
use syntax::latex::{self, HasBrack, HasCurly, LatexLanguage};
use titlecase::titlecase;

use crate::{Symbol, SymbolKind};

#[derive(Debug)]
pub struct SymbolBuilder<'a> {
    project: &'a Project<'a>,
    config: &'a Config,
}

impl<'a> SymbolBuilder<'a> {
    pub fn new(project: &'a Project<'a>, config: &'a Config) -> Self {
        Self { project, config }
    }

    pub fn visit(&self, node: &latex::SyntaxNode) -> Vec<Symbol> {
        let symbol = if let Some(section) = latex::Section::cast(node.clone()) {
            self.visit_section(&section)
        } else if let Some(enum_item) = latex::EnumItem::cast(node.clone()) {
            self.visit_enum_item(&enum_item)
        } else if let Some(equation) = latex::Equation::cast(node.clone()) {
            self.visit_equation(&equation)
        } else if let Some(environment) = latex::Environment::cast(node.clone()) {
            environment.begin().and_then(|begin| {
                let name = begin.name()?.key()?.to_string();
                if let Some(config) = self.config.symbols.custom_environments.get(&name) {
                    self.visit_environment(&environment, SymbolKind::Environment, config)
                } else if self.config.syntax.math_environments.contains(&name) {
                    self.visit_equation(&environment)
                } else if self.config.syntax.enum_environments.contains(&name) {
                    self.visit_enumeration(&environment, &name)
                } else if let Ok(float_kind) = FloatKind::from_str(&name) {
                    self.visit_float(&environment, float_kind)
                } else {
                    self.visit_theorem(&environment, &name)
                }
            })
        } else {
            None
        };

        match symbol {
            Some(mut parent) => {
                for child in node.children() {
                    parent.children.append(&mut self.visit(&child));
                }

                vec![parent]
            }
            None => {
                let mut symbols = Vec::new();
                for child in node.children() {
                    symbols.append(&mut self.visit(&child));
                }

                symbols
            }
        }
    }

    fn visit_section(&self, section: &latex::Section) -> Option<Symbol> {
        let range = latex::small_range(section);
        let group = section.name()?;
        let group_text = group.content_text()?;
        let kind = SymbolKind::Section;

        let name = match self.find_section_number(&group_text) {
            Some(number) => format!("{number} {group_text}"),
            None => group_text,
        };

        let symbol = match self.find_label(section.syntax()) {
            Some(label) => Symbol::new_label(name, kind, range, label),
            None => Symbol::new_simple(name, kind, range, range),
        };

        Some(symbol)
    }

    fn visit_enum_item(&self, enum_item: &latex::EnumItem) -> Option<Symbol> {
        let enum_envs = &self.config.syntax.enum_environments;
        if !enum_item
            .syntax()
            .ancestors()
            .filter_map(latex::Environment::cast)
            .filter_map(|environment| environment.begin())
            .filter_map(|begin| begin.name())
            .filter_map(|name| name.key())
            .any(|name| enum_envs.contains(&name.to_string()))
        {
            return None;
        }

        let range = latex::small_range(enum_item);
        let kind = SymbolKind::EnumerationItem;
        let name = enum_item
            .label()
            .and_then(|label| label.content_text())
            .filter(|text| !text.is_empty())
            .unwrap_or_else(|| "Item".into());

        let symbol = match self.find_label(enum_item.syntax()) {
            Some(label) => {
                let name = self
                    .find_label_number(&label.text)
                    .map_or_else(|| name.clone(), String::from);

                Symbol::new_label(name, kind, range, label)
            }
            None => Symbol::new_simple(name, kind, range, range),
        };

        Some(symbol)
    }

    fn visit_theorem(&self, environment: &latex::Environment, name: &str) -> Option<Symbol> {
        let heading = self
            .project
            .documents
            .iter()
            .filter_map(|document| document.data.as_tex())
            .flat_map(|data| data.semantics.theorem_definitions.iter())
            .find(|theorem| theorem.name.text == name)
            .map(|theorem| theorem.heading.as_str())?;

        let options = environment.begin().and_then(|begin| begin.options());
        let caption = options.and_then(|options| options.content_text());
        let range = latex::small_range(environment);
        let kind = SymbolKind::Theorem;

        let symbol = match self.find_label(environment.syntax()) {
            Some(label) => {
                let name = match (self.find_label_number(&label.text), caption) {
                    (Some(number), Some(caption)) => {
                        format!("{heading} {number} ({caption})")
                    }
                    (Some(number), None) => format!("{heading} {number}"),
                    (None, Some(caption)) => format!("{heading} ({caption})"),
                    (None, None) => heading.into(),
                };

                Symbol::new_label(name, kind, range, label)
            }
            None => {
                let name = caption.map_or_else(
                    || heading.into(),
                    |caption| format!("{heading} ({caption})"),
                );

                Symbol::new_simple(name, kind, range, range)
            }
        };

        Some(symbol)
    }

    fn visit_float(
        &self,
        environment: &latex::Environment,
        float_kind: FloatKind,
    ) -> Option<Symbol> {
        let range = latex::small_range(environment);

        let (float_kind, symbol_kind) = match float_kind {
            FloatKind::Algorithm => ("Algorithm", SymbolKind::Algorithm),
            FloatKind::Figure => ("Figure", SymbolKind::Figure),
            FloatKind::Listing => ("Listing", SymbolKind::Listing),
            FloatKind::Table => ("Table", SymbolKind::Table),
        };

        let caption = environment
            .syntax()
            .children()
            .filter_map(latex::Caption::cast)
            .find_map(|node| node.long())
            .and_then(|node| node.content_text())?;

        let symbol = match self.find_label(environment.syntax()) {
            Some(label) => {
                let name = match self.find_label_number(&label.text) {
                    Some(number) => format!("{float_kind} {number}: {caption}"),
                    None => format!("{float_kind}: {caption}"),
                };

                Symbol::new_label(name, symbol_kind, range, label)
            }
            None => {
                let name = format!("{float_kind}: {caption}");
                Symbol::new_simple(name, symbol_kind, range, range)
            }
        };

        Some(symbol)
    }

    fn visit_enumeration(
        &self,
        environment: &latex::Environment,
        environment_name: &str,
    ) -> Option<Symbol> {
        let display_name = titlecase(environment_name);
        let label = true;
        let config = SymbolEnvironmentConfig {
            display_name,
            label,
        };

        self.visit_environment(environment, SymbolKind::Enumeration, &config)
    }

    fn visit_equation(&self, node: &dyn AstNode<Language = LatexLanguage>) -> Option<Symbol> {
        let range = latex::small_range(node);
        let kind = SymbolKind::Equation;
        let symbol = match self.find_label(node.syntax()) {
            Some(label) => {
                let name = match self.find_label_number(&label.text) {
                    Some(number) => format!("Equation ({number})"),
                    None => "Equation".into(),
                };

                Symbol::new_label(name, kind, range, label)
            }
            None => Symbol::new_simple("Equation".into(), kind, range, range),
        };

        Some(symbol)
    }

    fn visit_environment(
        &self,
        environment: &latex::Environment,
        kind: SymbolKind,
        config: &SymbolEnvironmentConfig,
    ) -> Option<Symbol> {
        let range = latex::small_range(environment);

        let name = config.display_name.to_string();

        let symbol = if config.label {
            match self.find_label(environment.syntax()) {
                Some(label) => {
                    let name = match self.find_label_number(&label.text) {
                        Some(number) => format!("{name} {number}"),
                        None => name,
                    };

                    Symbol::new_label(name, kind, range, label)
                }
                None => Symbol::new_simple(name, kind, range, range),
            }
        } else {
            Symbol::new_simple(name, kind, range, range)
        };

        Some(symbol)
    }

    fn find_label(&self, parent: &latex::SyntaxNode) -> Option<Span> {
        let label = parent.children().find_map(latex::LabelDefinition::cast)?;
        let range = latex::small_range(&label);
        let text = label.name()?.key()?.to_string();
        Some(Span { text, range })
    }

    fn find_section_number(&self, name: &str) -> Option<&str> {
        self.project
            .documents
            .iter()
            .filter_map(|document| document.data.as_aux())
            .find_map(|data| data.semantics.section_numbers.get(name))
            .map(String::as_str)
    }

    fn find_label_number(&self, name: &str) -> Option<&str> {
        self.project
            .documents
            .iter()
            .filter_map(|document| document.data.as_aux())
            .find_map(|data| data.semantics.label_numbers.get(name))
            .map(String::as_str)
    }
}
