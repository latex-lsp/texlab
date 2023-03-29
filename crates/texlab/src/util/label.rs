use std::str::FromStr;

use rowan::{ast::AstNode, TextRange};
use syntax::latex::{self, HasBrack, HasCurly};

use crate::{
    db::{analysis::label, Document, Word, Workspace},
    Db,
};

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
pub enum LabeledObject {
    Section {
        prefix: &'static str,
        text: String,
    },
    Float {
        kind: LabeledFloatKind,
        caption: String,
    },
    Theorem {
        kind: Word,
        description: Option<String>,
    },
    Equation,
    EnumItem,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct RenderedLabel {
    pub range: TextRange,
    pub number: Option<Word>,
    pub object: LabeledObject,
}

impl RenderedLabel {
    pub fn reference(&self, db: &dyn Db) -> String {
        match &self.number {
            Some(number) => match &self.object {
                Section { prefix, text } => format!("{} {} ({})", prefix, number.text(db), text),
                Float { kind, caption } => {
                    format!("{} {}: {}", kind.as_str(), number.text(db), caption)
                }
                Theorem {
                    kind,
                    description: None,
                } => format!("{} {}", kind.text(db), number.text(db)),
                Theorem {
                    kind,
                    description: Some(description),
                } => format!("{} {} ({})", kind.text(db), number.text(db), description),
                Equation => format!("Equation ({})", number.text(db)),
                EnumItem => format!("Item {}", number.text(db)),
            },
            None => match &self.object {
                Section { prefix, text } => format!("{} ({})", prefix, text),
                Float { kind, caption } => format!("{}: {}", kind.as_str(), caption),
                Theorem {
                    kind,
                    description: None,
                } => kind.text(db).into(),
                Theorem {
                    kind,
                    description: Some(description),
                } => format!("{} ({})", kind.text(db), description),
                Equation => "Equation".into(),
                EnumItem => "Item".into(),
            },
        }
    }

    pub fn detail(&self, db: &dyn Db) -> Option<String> {
        match &self.object {
            Section { .. } | Theorem { .. } | Equation | EnumItem => Some(self.reference(db)),
            Float { kind, .. } => {
                let result = match &self.number {
                    Some(number) => format!("{} {}", kind.as_str(), number.text(db)),
                    None => kind.as_str().to_owned(),
                };
                Some(result)
            }
        }
    }
}

pub fn render(db: &dyn Db, document: Document, label_def: label::Name) -> Option<RenderedLabel> {
    let workspace = Workspace::get(db);
    let label_num = workspace.number_of_label(db, document, label_def.name(db));
    let root = document.parse(db).as_tex()?.root(db);

    label_def
        .origin(db)
        .as_definition()?
        .to_node(&root)
        .syntax()
        .ancestors()
        .find_map(|parent| {
            render_label_float(parent.clone(), label_num)
                .or_else(|| render_label_section(parent.clone(), label_num))
                .or_else(|| render_label_enum_item(db, parent.clone(), label_num))
                .or_else(|| render_label_equation(db, parent.clone(), label_num))
                .or_else(|| render_label_theorem(db, document, parent, label_num))
        })
}

pub fn find_label_definition(
    db: &dyn Db,
    child: Document,
    name: Word,
) -> Option<(Document, label::Name)> {
    Workspace::get(db)
        .related(db, child)
        .iter()
        .find_map(|document| {
            let data = document.parse(db).as_tex()?;
            let label = data
                .analyze(db)
                .labels(db)
                .iter()
                .filter(|label| label.origin(db).as_definition().is_some())
                .find(|label| label.name(db) == name)?;

            Some((*document, *label))
        })
}

fn render_label_float(parent: latex::SyntaxNode, number: Option<Word>) -> Option<RenderedLabel> {
    let environment = latex::Environment::cast(parent.clone())?;
    let environment_name = environment.begin()?.name()?.key()?.to_string();
    let kind = LabeledFloatKind::from_str(&environment_name).ok()?;
    let caption = find_caption_by_parent(&parent)?;
    Some(RenderedLabel {
        range: latex::small_range(&environment),
        number,
        object: LabeledObject::Float { caption, kind },
    })
}

fn render_label_section(parent: latex::SyntaxNode, number: Option<Word>) -> Option<RenderedLabel> {
    let section = latex::Section::cast(parent)?;
    let text_group = section.name()?;
    let text = text_group.content_text()?;

    Some(RenderedLabel {
        range: latex::small_range(&section),
        number,
        object: LabeledObject::Section {
            prefix: match section.syntax().kind() {
                latex::PART => "Part",
                latex::CHAPTER => "Chapter",
                latex::SECTION => "Section",
                latex::SUBSECTION => "Subsection",
                latex::SUBSUBSECTION => "Subsubsection",
                latex::PARAGRAPH => "Paragraph",
                latex::SUBPARAGRAPH => "Subparagraph",
                _ => unreachable!(),
            },
            text,
        },
    })
}

fn render_label_enum_item(
    db: &dyn Db,
    parent: latex::SyntaxNode,
    number: Option<Word>,
) -> Option<RenderedLabel> {
    let enum_item = latex::EnumItem::cast(parent)?;
    Some(RenderedLabel {
        range: latex::small_range(&enum_item),
        number: enum_item
            .label()
            .and_then(|label| label.content_text())
            .map(|text| Word::new(db, text))
            .or(number),
        object: LabeledObject::EnumItem,
    })
}

fn render_label_equation(
    db: &dyn Db,
    parent: latex::SyntaxNode,
    number: Option<Word>,
) -> Option<RenderedLabel> {
    let env = latex::Environment::cast(parent)?;
    let env_name = env.begin()?.name()?.key()?.to_string();

    if !db.config().syntax.math_environments.contains(&env_name) {
        return None;
    }

    Some(RenderedLabel {
        range: latex::small_range(&env),
        number,
        object: LabeledObject::Equation,
    })
}

fn render_label_theorem(
    db: &dyn Db,
    document: Document,
    parent: latex::SyntaxNode,
    number: Option<Word>,
) -> Option<RenderedLabel> {
    let environment = latex::Environment::cast(parent)?;
    let begin = environment.begin()?;
    let description = begin.options().and_then(|options| options.content_text());

    let environment_name = begin.name()?.key()?.to_string();

    let kind = Workspace::get(db)
        .related(db, document)
        .iter()
        .filter_map(|document| document.parse(db).as_tex())
        .flat_map(|data| data.analyze(db).theorem_environments(db))
        .find(|env| env.name(db).text(db) == &environment_name)
        .map(|env| env.description(db))?;

    Some(RenderedLabel {
        range: latex::small_range(&environment),
        number,
        object: LabeledObject::Theorem { kind, description },
    })
}

pub fn find_caption_by_parent(parent: &latex::SyntaxNode) -> Option<String> {
    parent
        .children()
        .filter_map(latex::Caption::cast)
        .find_map(|node| node.long())
        .and_then(|node| node.content_text())
}
