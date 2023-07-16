use text_size::{TextRange, TextSize};

use crate::{
    semantics::{bib, tex},
    Project,
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub enum SearchMode {
    Name,
    Full,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub enum ObjectKind {
    Definition,
    Reference,
}

pub trait Object {
    fn name_text(&self) -> &str;

    fn name_range(&self) -> TextRange;

    fn full_range(&self) -> TextRange;

    fn find<'db>(project: &'db Project) -> Box<dyn Iterator<Item = &'db Self> + 'db>;

    fn kind(&self) -> ObjectKind;
}

impl Object for tex::Label {
    fn name_text(&self) -> &str {
        &self.name.text
    }

    fn name_range(&self) -> TextRange {
        self.name.range
    }

    fn full_range(&self) -> TextRange {
        self.full_range
    }

    fn kind(&self) -> ObjectKind {
        match self.kind {
            tex::LabelKind::Definition => ObjectKind::Definition,
            tex::LabelKind::Reference => ObjectKind::Reference,
            tex::LabelKind::ReferenceRange => ObjectKind::Reference,
        }
    }

    fn find<'db>(project: &'db Project) -> Box<dyn Iterator<Item = &'db Self> + 'db> {
        let iter = project
            .documents
            .iter()
            .filter_map(|document| document.data.as_tex())
            .flat_map(|data| data.semantics.labels.iter());

        Box::new(iter)
    }
}

impl Object for tex::Citation {
    fn name_text(&self) -> &str {
        &self.name.text
    }

    fn name_range(&self) -> TextRange {
        self.name.range
    }

    fn full_range(&self) -> TextRange {
        self.full_range
    }

    fn find<'db>(project: &'db Project) -> Box<dyn Iterator<Item = &'db Self> + 'db> {
        let iter = project
            .documents
            .iter()
            .filter_map(|document| document.data.as_tex())
            .flat_map(|data| data.semantics.citations.iter());

        Box::new(iter)
    }

    fn kind(&self) -> ObjectKind {
        ObjectKind::Reference
    }
}

impl Object for bib::Entry {
    fn name_text(&self) -> &str {
        &self.name.text
    }

    fn name_range(&self) -> TextRange {
        self.name.range
    }

    fn full_range(&self) -> TextRange {
        self.full_range
    }

    fn find<'db>(project: &'db Project) -> Box<dyn Iterator<Item = &'db Self> + 'db> {
        let iter = project
            .documents
            .iter()
            .filter_map(|document| document.data.as_bib())
            .flat_map(|data| data.semantics.entries.iter());

        Box::new(iter)
    }

    fn kind(&self) -> ObjectKind {
        ObjectKind::Definition
    }
}

#[derive(Debug)]
pub struct ObjectWithRange<T> {
    pub object: T,
    pub range: TextRange,
}

impl<T> ObjectWithRange<T> {
    pub fn new(object: T, range: TextRange) -> Self {
        Self { object, range }
    }
}

pub fn object_at_cursor<T: Object>(
    objs: &[T],
    offset: TextSize,
    mode: SearchMode,
) -> Option<ObjectWithRange<&T>> {
    let mut result = objs
        .iter()
        .find(|obj| obj.name_range().contains_inclusive(offset))
        .map(|obj| ObjectWithRange::new(obj, obj.name_range()));

    if mode == SearchMode::Full {
        result = result.or_else(|| {
            objs.iter()
                .find(|obj| obj.full_range().contains_inclusive(offset))
                .map(|obj| ObjectWithRange::new(obj, obj.full_range()))
        });
    }

    result
}

pub fn definition<'db, T: Object>(project: &'db Project, name: &str) -> Option<&'db T> {
    T::find(project)
        .filter(|obj| obj.kind() == ObjectKind::Definition)
        .find(|obj| obj.name_text() == name)
}
