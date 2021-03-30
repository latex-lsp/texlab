mod document;
mod line_index;
mod link;
mod parser;

pub use self::{document::*, line_index::*, link::*, parser::*};

#[salsa::database(
    DocumentDatabaseStorage,
    LineIndexDatabaseStorage,
    ParserDatabaseStorage
)]
#[derive(Default)]
pub struct RootDatabase {
    storage: salsa::Storage<Self>,
}

impl salsa::Database for RootDatabase {}

impl salsa::ParallelDatabase for RootDatabase {
    fn snapshot(&self) -> salsa::Snapshot<Self> {
        salsa::Snapshot::new(Self {
            storage: self.storage.snapshot(),
        })
    }
}
