use lsp_types::{Position, Range};

pub trait RangeExt {
    fn new_simple(start_line: u64, start_character: u64, end_line: u64, end_character: u64)
        -> Self;

    fn contains(&self, pos: Position) -> bool;

    fn contains_exclusive(&self, pos: Position) -> bool;
}

impl RangeExt for Range {
    fn new_simple(
        start_line: u64,
        start_character: u64,
        end_line: u64,
        end_character: u64,
    ) -> Self {
        Self {
            start: Position::new(start_line, start_character),
            end: Position::new(end_line, end_character),
        }
    }

    fn contains(&self, pos: Position) -> bool {
        pos >= self.start && pos <= self.end
    }

    fn contains_exclusive(&self, pos: Position) -> bool {
        pos > self.start && pos < self.end
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contains_inside() {
        let range = Range::new_simple(1, 2, 3, 4);
        assert!(range.contains(Position::new(2, 5)));
    }

    #[test]
    fn contains_begin() {
        let range = Range::new_simple(1, 2, 3, 4);
        assert!(range.contains(range.start));
    }

    #[test]
    fn contains_end() {
        let range = Range::new_simple(1, 2, 3, 4);
        assert!(range.contains(range.end));
    }

    #[test]
    fn contains_outside_left() {
        let range = Range::new_simple(1, 2, 3, 4);
        assert!(!range.contains(Position::new(0, 5)));
    }

    #[test]
    fn contains_outside_right() {
        let range = Range::new_simple(1, 2, 3, 4);
        assert!(!range.contains(Position::new(5, 1)));
    }

    #[test]
    fn contains_exclusive_inside() {
        let range = Range::new_simple(1, 2, 3, 4);
        assert!(range.contains_exclusive(Position::new(2, 5)));
    }

    #[test]
    fn contains_exclusive_begin() {
        let range = Range::new_simple(1, 2, 3, 4);
        assert!(!range.contains_exclusive(range.start));
    }

    #[test]
    fn contains_exclusive_end() {
        let range = Range::new_simple(1, 2, 3, 4);
        assert!(!range.contains_exclusive(range.end));
    }

    #[test]
    fn contains_exclusive_outside_left() {
        let range = Range::new_simple(1, 2, 3, 4);
        assert!(!range.contains_exclusive(Position::new(0, 5)));
    }

    #[test]
    fn contains_exclusive_outside_right() {
        let range = Range::new_simple(1, 2, 3, 4);
        assert!(!range.contains_exclusive(Position::new(5, 1)));
    }
}
