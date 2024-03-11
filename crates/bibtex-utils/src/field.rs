use rustc_hash::FxHashMap;

pub mod author;
pub mod date;
pub mod number;
pub mod text;

/// A cache used to accelerate related field parses.
pub type FieldParseCache = FxHashMap<String, String>;