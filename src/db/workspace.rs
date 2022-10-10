use std::borrow::Cow;

use dashmap::DashMap;

use crate::{db::Document, Db};

use super::{FileId, Language, OpenedBy};

#[derive(Debug, Clone, Default)]
pub struct Workspace {
    documents: DashMap<FileId, Option<Document>>,
}

impl Workspace {
    pub fn open(
        &self,
        db: &mut dyn Db,
        file: FileId,
        text: String,
        language: Language,
    ) -> Document {
        match self.get(file) {
            Some(document) => {
                document
                    .set_text(db)
                    .with_durability(salsa::Durability::LOW)
                    .to(text);

                document
                    .set_language(db)
                    .with_durability(salsa::Durability::HIGH)
                    .to(language);

                document
                    .set_opened_by(db)
                    .with_durability(salsa::Durability::LOW)
                    .to(OpenedBy::Client);

                document
            }
            None => {
                let document = Document::new(db, file, text, language, OpenedBy::Client);
                self.documents.insert(file, Some(document));
                document
            }
        }
    }

    pub fn load(&self, db: &dyn Db, file: FileId) -> Option<Document> {
        *self.documents.entry(file).or_insert_with(|| {
            let path = file.path(db).as_deref()?;
            let data = std::fs::read(path).ok()?;
            let text = match String::from_utf8_lossy(&data) {
                Cow::Borrowed(_) => unsafe { String::from_utf8_unchecked(data) },
                Cow::Owned(text) => text,
            };

            let language = file.language(db)?;
            Some(Document::new(db, file, text, language, OpenedBy::Server))
        })
    }

    pub fn close(&self, db: &mut dyn Db, file: FileId) {
        self.documents.alter(&file, |_, document| {
            if let Some(doc) = document {
                doc.set_opened_by(db)
                    .with_durability(salsa::Durability::LOW)
                    .to(OpenedBy::Server);
            }

            document
        });
    }

    pub fn remove_if<F>(&self, file: FileId, predicate: F)
    where
        F: FnOnce(Document) -> bool,
    {
        self.documents
            .remove_if(&file, |_, doc| doc.map_or(false, predicate));
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = Document> + 'a {
        self.documents.iter().filter_map(|entry| *entry)
    }

    fn get(&self, file: FileId) -> Option<Document> {
        *self.documents.get(&file)?
    }
}
