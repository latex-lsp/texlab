use std::io::Read;

use flate2::read::GzDecoder;
use itertools::Itertools;
use lsp_types::{MarkupContent, MarkupKind};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

use crate::Workspace;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentDatabase {
    pub components: Vec<Component>,
    pub metadata: Vec<ComponentMetadata>,
}

impl ComponentDatabase {
    pub fn find(&self, name: &str) -> Option<&Component> {
        self.components.iter().find(|component| {
            component
                .file_names
                .iter()
                .any(|file_name| file_name == name)
        })
    }

    pub fn find_no_ext(&self, name: &str) -> Option<&Component> {
        self.components.iter().find(|component| {
            component
                .file_names
                .iter()
                .any(|file_name| &file_name[0..file_name.len() - 4] == name)
        })
    }

    pub fn linked_components(&self, workspace: &Workspace) -> Vec<&Component> {
        let mut start_components = vec![self.kernel()];
        for document in workspace.documents_by_uri.values() {
            if let Some(data) = document.data.as_latex() {
                data.extras
                    .explicit_links
                    .iter()
                    .filter_map(|link| link.as_component_name())
                    .filter_map(|name| self.find(&name))
                    .for_each(|component| start_components.push(component));
            }
        }

        let mut all_components = Vec::new();
        for component in start_components {
            all_components.push(component);
            component
                .references
                .iter()
                .flat_map(|file| self.find(file))
                .for_each(|component| all_components.push(component))
        }

        all_components
            .into_iter()
            .unique_by(|component| &component.file_names)
            .collect()
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
    pub file_names: Vec<SmolStr>,
    pub references: Vec<SmolStr>,
    pub commands: Vec<ComponentCommand>,
    pub environments: Vec<SmolStr>,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentCommand {
    pub name: SmolStr,
    pub image: Option<String>,
    pub glyph: Option<SmolStr>,
    pub parameters: Vec<ComponentParameter>,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentParameter(pub Vec<ComponentArgument>);

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentArgument {
    pub name: SmolStr,
    pub image: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentMetadata {
    pub name: String,
    pub caption: Option<String>,
    pub description: Option<String>,
}

const JSON_GZ: &[u8] = include_bytes!("../data/components.json.gz");

pub static COMPONENT_DATABASE: Lazy<ComponentDatabase> = Lazy::new(|| {
    let mut decoder = GzDecoder::new(JSON_GZ);
    let mut buf = String::new();
    decoder.read_to_string(&mut buf).unwrap();
    serde_json::from_str(&buf).unwrap()
});
