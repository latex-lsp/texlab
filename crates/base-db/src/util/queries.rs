use text_size::{TextRange, TextSize};

use crate::{
    semantics::{bib, tex},
    Document, Project,
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

    fn kind(&self) -> ObjectKind;

    fn find<'db>(document: &'db Document) -> Box<dyn Iterator<Item = &'db Self> + 'db>;

    fn find_all<'a, 'db>(
        project: &'a Project<'db>,
    ) -> Box<dyn Iterator<Item = (&'db Document, &'db Self)> + 'a> {
        let iter = project
            .documents
            .iter()
            .flat_map(|document| Self::find(document).map(|obj| (*document, obj)));

        Box::new(iter)
    }
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

    fn find<'db>(document: &'db Document) -> Box<dyn Iterator<Item = &'db Self> + 'db> {
        let data = document.data.as_tex();
        let iter = data
            .into_iter()
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

    fn find<'db>(document: &'db Document) -> Box<dyn Iterator<Item = &'db Self> + 'db> {
        let data = document.data.as_tex();
        let iter = data
            .into_iter()
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

    fn find<'db>(document: &'db Document) -> Box<dyn Iterator<Item = &'db Self> + 'db> {
        let data = document.data.as_bib();
        let iter = data
            .into_iter()
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

pub fn objects_with_name<'a, 'db, T: Object + 'static>(
    project: &'a Project<'db>,
    name: &'a str,
) -> impl Iterator<Item = (&'db Document, &'db T)> + 'a {
    T::find_all(project).filter(move |(_, obj)| obj.name_text() == name)
}
