use lsp_types::{MarkupContent, MarkupKind};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

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

    // pub fn linked_components(
    //     &self,
    //     workspace: &Workspace,
    //     file_uris: &[Arc<Uri>],
    // ) -> Vec<&'static Component> {
    //     let mut start_components = vec![COMPONENT_DATABASE.kernel()];
    //     for file_uri in file_uris {
    //         workspace
    //             .extras(&file_uri)
    //             .explicit_links
    //             .iter()
    //             .filter_map(|link| link.as_component_name())
    //             .filter_map(|name| COMPONENT_DATABASE.find(&name))
    //             .for_each(|component| start_components.push(component));
    //     }

    //     let mut all_components = Vec::new();
    //     for component in start_components {
    //         all_components.push(component);
    //         component
    //             .references
    //             .iter()
    //             .flat_map(|file| COMPONENT_DATABASE.find(&file))
    //             .for_each(|component| all_components.push(component))
    //     }

    //     all_components
    //         .into_iter()
    //         .unique_by(|component| &component.file_names)
    //         .collect()
    // }

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

const JSON: &str = include_str!("../data/components.json");

pub static COMPONENT_DATABASE: Lazy<ComponentDatabase> =
    Lazy::new(|| serde_json::from_str(JSON).unwrap());
