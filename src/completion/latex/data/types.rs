use crate::syntax::SyntaxTree;
use crate::workspace::Document;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LatexComponent {
    pub files: Vec<String>,
    pub references: Vec<String>,
    pub commands: Vec<String>,
    pub environments: Vec<String>,
}

#[derive(Debug, PartialEq, Eq, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LatexComponentDatabase {
    pub components: Vec<Arc<LatexComponent>>,
}

impl LatexComponentDatabase {
    pub fn new(components: Vec<Arc<LatexComponent>>) -> Self {
        LatexComponentDatabase { components }
    }

    pub fn related_components(&self, documents: &[Arc<Document>]) -> Vec<Arc<LatexComponent>> {
        let mut start_components = Vec::new();
        for document in documents {
            if let SyntaxTree::Latex(tree) = &document.tree {
                tree.components
                    .iter()
                    .flat_map(|file| self.find(&file))
                    .for_each(|component| start_components.push(component))
            }
        }

        let mut all_components = Vec::new();
        for component in start_components {
            all_components.push(Arc::clone(&component));
            component
                .references
                .iter()
                .flat_map(|file| self.find(&file))
                .for_each(|component| all_components.push(component))
        }

        all_components
            .iter()
            .unique_by(|component| &component.files)
            .map(Arc::clone)
            .collect()
    }

    fn find(&self, name: &String) -> Option<Arc<LatexComponent>> {
        self.components
            .iter()
            .find(|component| component.files.contains(name))
            .map(Arc::clone)
    }
}
