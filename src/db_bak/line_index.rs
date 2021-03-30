use std::sync::Arc;

use ra_ap_ide_db::line_index::LineIndex;

use super::{Document, DocumentDatabase};

#[salsa::query_group(LineIndexDatabaseStorage)]
pub trait LineIndexDatabase: DocumentDatabase {
    fn line_index(&self, document: Document) -> Arc<LineIndex>;
}

fn line_index(db: &dyn LineIndexDatabase, document: Document) -> Arc<LineIndex> {
    Arc::new(LineIndex::new(&db.source_code(document)))
}
