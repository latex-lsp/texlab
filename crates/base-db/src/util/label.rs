use std::str::FromStr;

use rowan::TextRange;

use self::RenderedObject::*;

use crate::{
    Workspace,
    deps::Project,
    semantics::tex::{Label, LabelObject},
};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FloatKind {
    Figure,
    Table,
    Listing,
    Algorithm,
}

impl FloatKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Figure => "Figure",
            Self::Table => "Table",
            Self::Listing => "Listing",
            Self::Algorithm => "Algorithm",
        }
    }
}

impl FromStr for FloatKind {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "figure" | "figure*" | "subfigure" => Ok(Self::Figure),
            "table" | "table*" | "subtable" => Ok(Self::Table),
            "listing" | "listing*" | "lstlisting" | "lstlisting*" => Ok(Self::Listing),
            "algorithm" | "algorithm*" => Ok(Self::Algorithm),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum RenderedObject<'a> {
    Section {
        prefix: &'a str,
        text: &'a str,
    },
    Float {
        kind: FloatKind,
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
    pub object: RenderedObject<'a>,
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

pub fn render_label<'a>(
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
                    object: RenderedObject::Section { prefix, text },
                });
            }
            LabelObject::EnumItem => {
                return Some(RenderedLabel {
                    range: target.range,
                    number,
                    object: RenderedObject::EnumItem,
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
                        object: RenderedObject::Equation,
                    });
                }

                if let Ok(kind) = FloatKind::from_str(name) {
                    return Some(RenderedLabel {
                        range: target.range,
                        number,
                        object: RenderedObject::Float {
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
                        object: RenderedObject::Theorem {
                            kind: &theorem.heading,
                            description: options.as_deref(),
                        },
                    });
                }
            }
        };
    }

    None
}
