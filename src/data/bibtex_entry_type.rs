use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct BibtexEntryType {
    pub name: String,
    pub documentation: Option<String>,
}

const JSON: &'static str = include_str!("bibtex_entry_type.json");

lazy_static! {
    pub static ref BIBTEX_ENTRY_TYPES: Vec<BibtexEntryType> = serde_json::from_str(JSON).unwrap();
}

pub fn get_documentation(name: &str) -> Option<&'static str> {
    for ty in BIBTEX_ENTRY_TYPES.iter() {
        if ty.name.to_lowercase() == name.to_lowercase() {
            if let Some(documentation) = &ty.documentation {
                return Some(&documentation);
            }
        }
    }
    return None;
}
