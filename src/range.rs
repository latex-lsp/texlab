use lsp_types::{Position, Range};

pub trait RangeExt {
    fn new_simple(start_line: u32, start_character: u32, end_line: u32, end_character: u32)
        -> Self;
}

impl RangeExt for Range {
    fn new_simple(
        start_line: u32,
        start_character: u32,
        end_line: u32,
        end_character: u32,
    ) -> Self {
        Self {
            start: Position::new(start_line, start_character),
            end: Position::new(end_line, end_character),
        }
    }
}
