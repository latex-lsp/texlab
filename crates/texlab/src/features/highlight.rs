mod label;

use base_db::Workspace;
use lsp_types::{DocumentHighlight, Position, Url};

use crate::util::cursor::CursorContext;

pub fn find_all(
    workspace: &Workspace,
    uri: &Url,
    position: Position,
) -> Option<Vec<DocumentHighlight>> {
    let context = CursorContext::new(workspace, uri, position, ())?;
    label::find_highlights(&context)
}
