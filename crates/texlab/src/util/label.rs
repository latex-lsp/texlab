use std::str::FromStr;

use base_db::{
    semantics::tex::{Label, LabelObject},
    Project, Workspace,
};
use rowan::{ast::AstNode, TextRange};
use syntax::latex::{self, HasCurly};

use self::LabeledObject::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum LabeledFloatKind {
    Figure,
    Table,
    Listing,
    Algorithm,
}

impl LabeledFloatKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Figure => "Figure",
            Self::Table => "Table",
            Self::Listing => "Listing",
            Self::Algorithm => "Algorithm",
        }
    }
}

impl FromStr for LabeledFloatKind {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "figure" | "subfigure" => Ok(Self::Figure),
            "table" | "subtable" => Ok(Self::Table),
            "listing" | "lstlisting" => Ok(Self::Listing),
            "algorithm" => Ok(Self::Algorithm),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum LabeledObject<'a> {
    Section {
        prefix: &'a str,
        text: &'a str,
    },
    Float {
        kind: LabeledFloatKind,
        caption: &'a str,
    },
    Theorem {
        kind: &'a str,
        description: Option<&'a str>,
    },
    Equation,
    EnumItem,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct RenderedLabel<'a> {
    pub range: TextRange,
    pub number: Option<&'a str>,
    pub object: LabeledObject<'a>,
}

impl<'a> RenderedLabel<'a> {
    pub fn reference(&self) -> String {
        match &self.number {
            Some(number) => match &self.object {
                Section { prefix, text } => format!("{} {} ({})", prefix, number, text),
                Float { kind, caption } => {
                    format!("{} {}: {}", kind.as_str(), number, caption)
                }
                Theorem {
                    kind,
                    description: None,
                } => format!("{} {}", kind, number),
                Theorem {
                    kind,
                    description: Some(description),
                } => format!("{} {} ({})", kind, number, description),
                Equation => format!("Equation ({})", number),
                EnumItem => format!("Item {}", number),
            },
            None => match &self.object {
                Section { prefix, text } => format!("{} ({})", prefix, text),
                Float { kind, caption } => format!("{}: {}", kind.as_str(), caption),
                Theorem {
                    kind,
                    description: None,
                } => String::from(*kind),
                Theorem {
                    kind,
                    description: Some(description),
                } => format!("{} ({})", kind, description),
                Equation => "Equation".into(),
                EnumItem => "Item".into(),
            },
        }
    }

    pub fn detail(&self) -> Option<String> {
        match &self.object {
            Section { .. } | Theorem { .. } | Equation | EnumItem => Some(self.reference()),
            Float { kind, .. } => {
                let result = match &self.number {
                    Some(number) => format!("{} {}", kind.as_str(), number),
                    None => kind.as_str().to_owned(),
                };
                Some(result)
            }
        }
    }
}

pub fn render<'a>(
    workspace: &'a Workspace,
    project: &Project<'a>,
    label: &'a Label,
) -> Option<RenderedLabel<'a>> {
    let number = project
        .documents
        .iter()
        .filter_map(|document| document.data.as_aux())
        .find_map(|data| data.semantics.label_numbers.get(&label.name.text))
        .map(|number| number.as_str());

    for target in &label.targets {
        match &target.object {
            LabelObject::Section { prefix, text } => {
                return Some(RenderedLabel {
                    range: target.range,
                    number,
                    object: LabeledObject::Section { prefix, text },
                });
            }
            LabelObject::EnumItem => {
                return Some(RenderedLabel {
                    range: target.range,
                    number,
                    object: LabeledObject::EnumItem,
                });
            }
            LabelObject::Environment {
                name,
                options,
                caption,
            } => {
                let config = &workspace.config().syntax;
                if config.math_environments.contains(name.as_str()) {
                    return Some(RenderedLabel {
                        range: target.range,
                        number,
                        object: LabeledObject::Equation,
                    });
                }

                if let Ok(kind) = LabeledFloatKind::from_str(name) {
                    return Some(RenderedLabel {
                        range: target.range,
                        number,
                        object: LabeledObject::Float {
                            kind,
                            caption: caption.as_deref()?,
                        },
                    });
                }

                if let Some(theorem) = project
                    .documents
                    .iter()
                    .filter_map(|document| document.data.as_tex())
                    .flat_map(|data| data.semantics.theorem_definitions.iter())
                    .find(|theorem| theorem.name.text == *name)
                {
                    return Some(RenderedLabel {
                        range: target.range,
                        number,
                        object: LabeledObject::Theorem {
                            kind: &theorem.description,
                            description: options.as_deref(),
                        },
                    });
                }
            }
        };
    }

    None
}

pub(crate) fn find_caption_by_parent(parent: &latex::SyntaxNode) -> Option<String> {
    parent
        .children()
        .filter_map(latex::Caption::cast)
        .find_map(|node| node.long())
        .and_then(|node| node.content_text())
}
