use lsp_types::{Position, Range};

pub trait RangeExt {
    fn new_simple(start_line: u32, start_character: u32, end_line: u32, end_character: u32)
        -> Self;

    fn contains(&self, pos: Position) -> bool;

    fn contains_inclusive(&self, pos: Position) -> bool;
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

    fn contains(&self, pos: Position) -> bool {
        pos >= self.start && pos <= self.end
    }

    fn contains_inclusive(&self, pos: Position) -> bool {
        pos > self.start && pos < self.end
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contains_inside() {
        let range = Range::new_simple(1, 2, 3, 4);
        assert!(range.contains(Position::new(2, 5)));
    }

    #[test]
    fn test_contains_begin() {
        let range = Range::new_simple(1, 2, 3, 4);
        assert!(range.contains(range.start));
    }

    #[test]
    fn test_contains_end() {
        let range = Range::new_simple(1, 2, 3, 4);
        assert!(range.contains(range.end));
    }

    #[test]
    fn test_contains_outside_left() {
        let range = Range::new_simple(1, 2, 3, 4);
        assert!(!range.contains(Position::new(0, 5)));
    }

    #[test]
    fn test_contains_outside_right() {
        let range = Range::new_simple(1, 2, 3, 4);
        assert!(!range.contains(Position::new(5, 1)));
    }

    #[test]
    fn test_contains_inclusive_inside() {
        let range = Range::new_simple(1, 2, 3, 4);
        assert!(range.contains_inclusive(Position::new(2, 5)));
    }

    #[test]
    fn test_contains_inclusive_begin() {
        let range = Range::new_simple(1, 2, 3, 4);
        assert!(!range.contains_inclusive(range.start));
    }

    #[test]
    fn test_contains_inclusive_end() {
        let range = Range::new_simple(1, 2, 3, 4);
        assert!(!range.contains_inclusive(range.end));
    }

    #[test]
    fn test_contains_inclusive_outside_left() {
        let range = Range::new_simple(1, 2, 3, 4);
        assert!(!range.contains_inclusive(Position::new(0, 5)));
    }

    #[test]
    fn test_contains_inclusive_outside_right() {
        let range = Range::new_simple(1, 2, 3, 4);
        assert!(!range.contains_inclusive(Position::new(5, 1)));
    }
}
