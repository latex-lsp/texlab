use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BibtexEntryTypeCategory {
    Misc,
    String,
    Article,
    Book,
    Collection,
    Part,
    Thesis,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BibtexEntryTypeDoc {
    pub name: String,
    pub category: BibtexEntryTypeCategory,
    pub documentation: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BibtexFieldDoc {
    pub name: String,
    pub documentation: String,
}

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LanguageData {
    pub colors: Vec<String>,
    pub entry_types: Vec<BibtexEntryTypeDoc>,
    pub fields: Vec<BibtexFieldDoc>,
    pub pgf_libraries: Vec<String>,
    pub tikz_libraries: Vec<String>,
    pub math_environments: Vec<String>,
    pub enum_environments: Vec<String>,
}

impl LanguageData {
    pub fn find_entry_type(&self, name: &str) -> Option<&BibtexEntryTypeDoc> {
        let name = name.to_lowercase();
        self.entry_types
            .iter()
            .find(|ty| ty.name.to_lowercase() == name)
    }

    pub fn entry_type_documentation(&self, name: &str) -> Option<&str> {
        self.find_entry_type(name)
            .and_then(|ty| ty.documentation.as_ref().map(AsRef::as_ref))
    }

    pub fn field_documentation(&self, name: &str) -> Option<&str> {
        self.fields
            .iter()
            .find(|field| field.name.to_lowercase() == name.to_lowercase())
            .map(|field| field.documentation.as_ref())
    }
}

pub static LANGUAGE_DATA: Lazy<LanguageData> = Lazy::new(|| {
    const JSON: &str = include_str!("../data/lang_data.json");
    serde_json::from_str(JSON).expect("Failed to deserialize language.json")
});
