use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct BibtexField {
    pub name: String,
    pub documentation: String,
}

const JSON: &'static str = include_str!("bibtex_field.json");

lazy_static! {
    pub static ref BIBTEX_FIELDS: Vec<BibtexField> = serde_json::from_str(JSON).unwrap();
}

pub fn get_documentation(name: &str) -> Option<&'static str> {
    BIBTEX_FIELDS
        .iter()
        .find(|field| field.name.to_lowercase() == name.to_lowercase())
        .map(|field| field.documentation.as_ref())
}
