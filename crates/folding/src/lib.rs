use base_db::{Document, DocumentData};
use rowan::{ast::AstNode, TextRange};
use syntax::{
    bibtex::{self, HasDelims, HasName},
    latex,
};

#[derive(Debug)]
pub struct FoldingRange {
    pub range: TextRange,
    pub kind: FoldingRangeKind,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub enum FoldingRangeKind {
    Section,
    Environment,
    Entry,
}

pub fn find_all(document: &Document) -> Vec<FoldingRange> {
    let mut builder = FoldingBuilder::default();

    if let DocumentData::Tex(data) = &document.data {
        for node in data.root_node().descendants() {
            if let Some(section) = latex::Section::cast(node.clone()) {
                builder.fold_section(&section);
            } else if let Some(item) = latex::EnumItem::cast(node.clone()) {
                builder.fold_enum_item(&item);
            } else if let Some(env) = latex::Environment::cast(node) {
                builder.fold_environment(env);
            }
        }
    } else if let DocumentData::Bib(data) = &document.data {
        for node in data.root_node().descendants() {
            if let Some(entry) = bibtex::Entry::cast(node.clone()) {
                builder.fold_entry(&entry);
            } else if let Some(string) = bibtex::StringDef::cast(node) {
                builder.fold_entry(&string);
            }
        }
    }

    builder.ranges
}

#[derive(Debug, Default)]
struct FoldingBuilder {
    ranges: Vec<FoldingRange>,
}

impl FoldingBuilder {
    fn fold_section(&mut self, section: &latex::Section) -> Option<()> {
        let start = section
            .name()
            .map(|name| latex::small_range(&name).end())
            .or_else(|| section.command().map(|cmd| cmd.text_range().end()))?;
        let end = section.syntax().text_range().end();

        self.ranges.push(FoldingRange {
            range: TextRange::new(start, end),
            kind: FoldingRangeKind::Section,
        });

        Some(())
    }

    fn fold_enum_item(&mut self, item: &latex::EnumItem) -> Option<()> {
        let start = item
            .label()
            .map(|label| latex::small_range(&label).end())
            .or_else(|| item.command().map(|cmd| cmd.text_range().end()))?;

        let end = item.syntax().text_range().end();
        self.ranges.push(FoldingRange {
            range: TextRange::new(start, end),
            kind: FoldingRangeKind::Section,
        });

        Some(())
    }

    fn fold_environment(&mut self, env: latex::Environment) -> Option<()> {
        let start = latex::small_range(&env.begin()?).end();
        let end = latex::small_range(&env.end()?).start();
        self.ranges.push(FoldingRange {
            range: TextRange::new(start, end),
            kind: FoldingRangeKind::Environment,
        });

        Some(())
    }

    fn fold_entry(&mut self, entry: &(impl HasName + HasDelims)) -> Option<()> {
        let start = entry.name_token()?.text_range().end();
        let end = entry.right_delim_token()?.text_range().start();
        self.ranges.push(FoldingRange {
            range: TextRange::new(start, end),
            kind: FoldingRangeKind::Entry,
        });

        Some(())
    }
}

#[cfg(test)]
mod tests;
