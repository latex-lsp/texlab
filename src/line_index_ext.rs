use cstree::{TextRange, TextSize};
use lsp_types::{Position, Range};

use crate::{LineCol, LineIndex};

pub trait LineIndexExt {
    fn offset_lsp(&self, line_col: Position) -> TextSize;

    fn offset_lsp_range(&self, line_col: Range) -> TextRange;

    fn line_col_lsp(&self, offset: TextSize) -> Position;

    fn line_col_lsp_range(&self, offset: TextRange) -> Range;
}

impl LineIndexExt for LineIndex {
    fn offset_lsp(&self, line_col: Position) -> TextSize {
        let line_col = LineCol {
            line: line_col.line,
            col: line_col.character,
        };
        self.offset(line_col)
    }

    fn offset_lsp_range(&self, line_col: Range) -> TextRange {
        let start = self.offset_lsp(line_col.start);
        let end = self.offset_lsp(line_col.end);
        TextRange::new(start, end)
    }

    fn line_col_lsp(&self, offset: TextSize) -> Position {
        let LineCol { line, col } = self.line_col(offset);
        Position::new(line, col)
    }

    fn line_col_lsp_range(&self, offset: TextRange) -> Range {
        let start = self.line_col_lsp(offset.start());
        let end = self.line_col_lsp(offset.end());
        Range::new(start, end)
    }
}
