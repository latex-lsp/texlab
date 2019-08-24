use lsp_types::*;

pub trait RangeExt {
    fn new_simple(start_line: u64, start_character: u64, end_line: u64, end_character: u64)
        -> Self;

    fn contains(&self, position: Position) -> bool;

    fn contains_exclusive(&self, position: Position) -> bool;
}

impl RangeExt for Range {
    fn new_simple(
        start_line: u64,
        start_character: u64,
        end_line: u64,
        end_character: u64,
    ) -> Self {
        Range {
            start: Position::new(start_line, start_character),
            end: Position::new(end_line, end_character),
        }
    }

    fn contains(&self, position: Position) -> bool {
        position >= self.start && position <= self.end
    }

    fn contains_exclusive(&self, position: Position) -> bool {
        position > self.start && position < self.end
    }
}
