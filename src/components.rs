use crate::protocol::{MarkupContent, MarkupKind};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Database {
    pub components: Vec<Component>,
    pub metadata: Vec<Metadata>,
}

impl Database {
    pub fn find(&self, name: &str) -> Option<&Component> {
        self.components.iter().find(|component| {
            component
                .file_names
                .iter()
                .any(|file_name| file_name == name)
        })
    }

    pub fn contains(&self, short_name: &str) -> bool {
        let sty = format!("{}.sty", short_name);
        let cls = format!("{}.cls", short_name);
        self.find(&sty).is_some() || self.find(&cls).is_some()
    }

    pub fn kernel(&self) -> &Component {
        self.components
            .iter()
            .find(|component| component.file_names.is_empty())
            .unwrap()
    }

    pub fn exists(&self, file_name: &str) -> bool {
        self.components
            .iter()
            .any(|component| component.file_names.iter().any(|f| f == file_name))
    }

    pub fn documentation(&self, name: &str) -> Option<MarkupContent> {
        let metadata = self
            .metadata
            .iter()
            .find(|metadata| metadata.name == name)?;

        let desc = metadata.description.to_owned()?;
        Some(MarkupContent {
            kind: MarkupKind::PlainText,
            value: desc,
        })
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Component {
    pub file_names: Vec<String>,
    pub references: Vec<String>,
    pub commands: Vec<Command>,
    pub environments: Vec<String>,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Command {
    pub name: String,
    pub image: Option<String>,
    pub glyph: Option<String>,
    pub parameters: Vec<Parameter>,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Parameter(pub Vec<Argument>);

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Argument {
    pub name: String,
    pub image: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    pub name: String,
    pub caption: Option<String>,
    pub description: Option<String>,
}

const JSON: &str = include_str!("components.json");

pub static COMPONENT_DATABASE: Lazy<Database> = Lazy::new(|| serde_json::from_str(JSON).unwrap());
