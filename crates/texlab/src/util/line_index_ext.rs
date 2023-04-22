use base_db::util::{LineCol, LineColUtf16, LineIndex};
use lsp_types::{Position, Range};
use rowan::{TextRange, TextSize};

pub trait LineIndexExt {
    fn offset_lsp(&self, line_col: Position) -> TextSize;

    fn offset_lsp_range(&self, line_col: Range) -> TextRange;

    fn line_col_lsp(&self, offset: TextSize) -> Position;

    fn line_col_lsp_range(&self, offset: TextRange) -> Range;
}

impl LineIndexExt for LineIndex {
    fn offset_lsp(&self, line_col: Position) -> TextSize {
        let line_col = LineColUtf16 {
            line: line_col.line,
            col: line_col.character,
        };
        self.offset(self.to_utf8(line_col))
    }

    fn offset_lsp_range(&self, line_col: Range) -> TextRange {
        let start = self.offset_lsp(line_col.start);
        let end = self.offset_lsp(line_col.end);
        TextRange::new(start, end)
    }

    fn line_col_lsp(&self, offset: TextSize) -> Position {
        let position = self.line_col(offset);
        let LineColUtf16 { line, col } = self.to_utf16(position);
        Position::new(line, col)
    }

    fn line_col_lsp_range(&self, offset: TextRange) -> Range {
        let start = self.line_col_lsp(offset.start());
        let mut end = self.line_col_lsp(offset.end());
        if end.line != start.line && end.character == 0 {
            // Prefer keeping multi-line ranges on the same line
            let line_end = self.offset(LineCol {
                line: end.line,
                col: 0,
            });

            end = self.line_col_lsp(line_end - TextSize::from(1));
        }

        Range::new(start, end)
    }
}
