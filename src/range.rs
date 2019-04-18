use lsp_types::{Position, Range};

pub fn create(start_line: u64, start_character: u64, end_line: u64, end_character: u64) -> Range {
    let start = Position::new(start_line, start_character);
    let end = Position::new(end_line, end_character);
    Range::new(start, end)
}

pub fn contains(range: Range, position: Position) -> bool {
    position >= range.start && position <= range.end
}

pub fn contains_exclusive(range: Range, position: Position) -> bool {
    position > range.start && position < range.end
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create() {
        let range = create(0, 1, 2, 3);
        assert_eq!(Position::new(0, 1), range.start);
        assert_eq!(Position::new(2, 3), range.end);
    }

    #[test]
    fn test_contains() {
        let range = create(0, 1, 2, 3);
        assert_eq!(true, contains(range, Position::new(1, 2)));
        assert_eq!(true, contains(range, Position::new(0, 1)));
        assert_eq!(true, contains(range, Position::new(2, 3)));
        assert_eq!(false, contains(range, Position::new(3, 0)));
    }

    #[test]
    fn test_contains_exclusive() {
        let range = create(0, 1, 2, 3);
        assert_eq!(true, contains(range, Position::new(1, 2)));
        assert_eq!(false, contains_exclusive(range, Position::new(0, 1)));
        assert_eq!(false, contains_exclusive(range, Position::new(2, 3)));
        assert_eq!(false, contains(range, Position::new(3, 0)));
    }
}
