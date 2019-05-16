use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct LatexSymbolDatabase {
    pub commands: Vec<LatexCommandSymbol>,
    pub arguments: Vec<LatexArgumentSymbolGroup>,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct LatexCommandSymbol {
    pub command: String,
    pub component: Option<String>,
    pub image: String,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct LatexArgumentSymbolGroup {
    pub command: String,
    pub component: Option<String>,
    pub index: usize,
    pub arguments: Vec<LatexArgumentSymbol>,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct LatexArgumentSymbol {
    pub argument: String,
    pub image: String,
}

const JSON: &'static str = include_str!("symbols.json");

lazy_static! {
    pub static ref DATABASE: LatexSymbolDatabase = serde_json::from_str(JSON).unwrap();
}
