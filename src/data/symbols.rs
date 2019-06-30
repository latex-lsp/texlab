use once_cell::sync::Lazy;
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

static JSON: &'static str = include_str!("symbols.json");

pub static DATABASE: Lazy<LatexSymbolDatabase> = Lazy::new(|| serde_json::from_str(JSON).unwrap());
