use rowan::{
    ast::{AstNode, AstPtr},
    TextRange,
};
use syntax::latex;

use crate::{db::Word, Db};

#[salsa::tracked]
pub struct Number {
    pub name: Word,
    pub text: Word,
}

impl Number {
    pub(super) fn of_number(
        db: &dyn Db,
        node: latex::SyntaxNode,
        results: &mut Vec<Self>,
    ) -> Option<()> {
        let number = latex::LabelNumber::cast(node)?;
        let name = number.name()?.key()?.to_string();
        let text = number
            .text()?
            .syntax()
            .descendants_with_tokens()
            .filter_map(|element| element.into_node())
            .find(|node| node.kind() == latex::TEXT || node.kind() == latex::MIXED_GROUP)?
            .text()
            .to_string();

        results.push(Self::new(db, Word::new(db, name), Word::new(db, text)));

        Some(())
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Origin {
    Definition(AstPtr<latex::LabelDefinition>),
    Reference(AstPtr<latex::LabelReference>),
    ReferenceRange(AstPtr<latex::LabelReferenceRange>),
}

impl Origin {
    pub fn as_definition(&self) -> Option<&AstPtr<latex::LabelDefinition>> {
        match self {
            Self::Definition(ptr) => Some(ptr),
            _ => None,
        }
    }
}

#[salsa::tracked]
pub struct Name {
    pub origin: Origin,
    pub name: Word,
    pub range: TextRange,
}

impl Name {
    pub(super) fn of_definition(
        db: &dyn Db,
        node: latex::SyntaxNode,
        results: &mut Vec<Self>,
    ) -> Option<()> {
        let label = latex::LabelDefinition::cast(node)?;
        let name = label.name()?.key()?;
        results.push(Self::new(
            db,
            Origin::Definition(AstPtr::new(&label)),
            Word::new(db, name.to_string()),
            latex::small_range(&name),
        ));

        Some(())
    }

    pub(super) fn of_reference(
        db: &dyn Db,
        node: latex::SyntaxNode,
        results: &mut Vec<Self>,
    ) -> Option<()> {
        let label = latex::LabelReference::cast(node)?;
        for name in label.name_list()?.keys() {
            results.push(Self::new(
                db,
                Origin::Reference(AstPtr::new(&label)),
                Word::new(db, name.to_string()),
                latex::small_range(&name),
            ));
        }

        Some(())
    }

    pub(super) fn of_reference_range(
        db: &dyn Db,
        node: latex::SyntaxNode,
        results: &mut Vec<Self>,
    ) -> Option<()> {
        let label = latex::LabelReferenceRange::cast(node)?;
        if let Some(name) = label.from().and_then(|name| name.key()) {
            results.push(Self::new(
                db,
                Origin::ReferenceRange(AstPtr::new(&label)),
                Word::new(db, name.to_string()),
                latex::small_range(&name),
            ));
        }

        if let Some(name) = label.to().and_then(|name| name.key()) {
            results.push(Self::new(
                db,
                Origin::ReferenceRange(AstPtr::new(&label)),
                Word::new(db, name.to_string()),
                latex::small_range(&name),
            ));
        }

        Some(())
    }
}
