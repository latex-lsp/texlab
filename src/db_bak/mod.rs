// mod ast;
mod cst;
mod document;
mod line_index;
// mod symbol_tree;

pub use self::{/*ast::*,*/ cst::*, document::*, line_index::*};

#[salsa::database(
    // AstDatabaseStorage,
    CstDatabaseStorage,
    DocumentDatabaseStorage,
    LineIndexDatabaseStorage
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
