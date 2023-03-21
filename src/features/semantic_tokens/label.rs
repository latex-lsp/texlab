use crate::{
    db::{analysis::label, Document, Workspace},
    util, Db,
};

use super::{Context, Token, TokenBuilder, TokenKind, TokenModifiers};

pub(super) fn find(context: Context, builder: &mut TokenBuilder) -> Option<()> {
    let db = context.db;
    let labels = context.document.parse(db).as_tex()?.analyze(db).labels(db);
    for label in labels
        .iter()
        .filter(|label| context.viewport.intersect(label.range(db)).is_some())
        .copied()
    {
        let kind = token_type(context, label);
        let modifiers = token_modifiers(context, label);
        let range = label.range(db);
        builder.push(Token {
            range,
            kind,
            modifiers,
        });
    }

    Some(())
}

fn token_type(context: Context, label: label::Name) -> TokenKind {
    let db = context.db;
    let definition = match label.origin(db) {
        label::Origin::Definition(_) => Some((context.document, label)),
        label::Origin::Reference(_) | label::Origin::ReferenceRange(_) => {
            util::label::find_label_definition(db, context.document, label.name(db))
        }
    };

    match definition
        .and_then(|(doc, label)| util::label::render(db, doc, label))
        .map(|label| label.object)
    {
        Some(util::label::LabeledObject::Section { .. }) => TokenKind::SectionLabel,
        Some(util::label::LabeledObject::Float { .. }) => TokenKind::FloatLabel,
        Some(util::label::LabeledObject::EnumItem { .. }) => TokenKind::EnumItemLabel,
        Some(util::label::LabeledObject::Equation { .. }) => TokenKind::EquationLabel,
        Some(util::label::LabeledObject::Theorem { .. }) => TokenKind::TheoremLabel,
        None => TokenKind::GenericLabel,
    }
}

fn token_modifiers(context: Context, label: label::Name) -> TokenModifiers {
    let db = context.db;
    let name = label.name(db).text(db);
    match label.origin(db) {
        label::Origin::Definition(_) => {
            if !is_label_referenced(db, context.document, name) {
                TokenModifiers::UNUSED
            } else {
                TokenModifiers::NONE
            }
        }
        label::Origin::Reference(_) | label::Origin::ReferenceRange(_) => {
            if !is_label_defined(db, context.document, name) {
                TokenModifiers::UNDEFINED
            } else {
                TokenModifiers::NONE
            }
        }
    }
}

fn is_label_defined(db: &dyn Db, child: Document, name: &str) -> bool {
    Workspace::get(db)
        .related(db, child)
        .iter()
        .filter_map(|document| document.parse(db).as_tex())
        .flat_map(|data| data.analyze(db).labels(db))
        .filter(|label| matches!(label.origin(db), label::Origin::Definition(_)))
        .any(|label| label.name(db).text(db) == name)
}

fn is_label_referenced(db: &dyn Db, child: Document, name: &str) -> bool {
    Workspace::get(db)
        .related(db, child)
        .iter()
        .filter_map(|document| document.parse(db).as_tex())
        .flat_map(|data| data.analyze(db).labels(db))
        .filter(|label| {
            matches!(
                label.origin(db),
                label::Origin::Reference(_) | label::Origin::ReferenceRange(_)
            )
        })
        .any(|label| label.name(db).text(db) == name)
}
