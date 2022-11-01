mod label;

use lsp_types::{DocumentHighlight, Position, Url};

use crate::{util::cursor::CursorContext, Db};

pub fn find_all(db: &dyn Db, uri: &Url, position: Position) -> Option<Vec<DocumentHighlight>> {
    let context = CursorContext::new(db, uri, position, ())?;
    label::find_label_highlights(&context)
}
