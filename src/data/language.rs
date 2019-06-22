use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LatexEnvironmentCommand {
    pub name: String,
    pub index: usize,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LatexCitationCommand {
    pub name: String,
    pub index: usize,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LatexLabelKind {
    Definition,
    Reference,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LatexLabelCommand {
    pub name: String,
    pub index: usize,
    pub kind: LatexLabelKind,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LatexSectionCommand {
    pub name: String,
    pub index: usize,
    pub level: i32,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LatexIncludeKind {
    Package,
    Class,
    Latex,
    Bibliography,
    Image,
    Svg,
    Everything,
}

impl LatexIncludeKind {
    pub fn extensions(&self) -> Option<Vec<&'static str>> {
        match self {
            LatexIncludeKind::Package => Some(vec!["sty"]),
            LatexIncludeKind::Class => Some(vec!["cls"]),
            LatexIncludeKind::Latex => Some(vec!["tex"]),
            LatexIncludeKind::Bibliography => Some(vec!["bib"]),
            LatexIncludeKind::Image => Some(vec!["pdf", "png", "jpg", "jpeg", "bmp"]),
            LatexIncludeKind::Svg => Some(vec!["svg"]),
            LatexIncludeKind::Everything => None,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LatexIncludeCommand {
    pub name: String,
    pub index: usize,
    pub kind: LatexIncludeKind,
    pub include_extension: bool,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LatexCommandDefinitionCommand {
    pub name: String,
    pub definition_index: usize,
    pub argument_count_index: usize,
    pub implementation_index: usize,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LatexMathOperatorCommand {
    pub name: String,
    pub definition_index: usize,
    pub implementation_index: usize,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LatexColorCommand {
    pub name: String,
    pub index: usize,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LatexColorModelCommand {
    pub name: String,
    pub index: usize,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BibtexEntryType {
    pub name: String,
    pub documentation: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LanguageOptions {
    pub environment_commands: Vec<LatexEnvironmentCommand>,
    pub citation_commands: Vec<LatexCitationCommand>,
    pub label_commands: Vec<LatexLabelCommand>,
    pub section_commands: Vec<LatexSectionCommand>,
    pub include_commands: Vec<LatexIncludeCommand>,
    pub command_definition_commands: Vec<LatexCommandDefinitionCommand>,
    pub math_operator_commands: Vec<LatexMathOperatorCommand>,
    pub colors: Vec<String>,
    pub color_commands: Vec<LatexColorCommand>,
    pub color_model_commands: Vec<LatexColorModelCommand>,
    pub entry_types: Vec<BibtexEntryType>,
}

impl LanguageOptions {
    pub fn get_entry_type_doc(&self, name: &str) -> Option<&str> {
        for ty in self.entry_types.iter() {
            if ty.name.to_lowercase() == name.to_lowercase() {
                if let Some(documentation) = &ty.documentation {
                    return Some(&documentation);
                }
            }
        }
        None
    }
}

const JSON: &str = include_str!("language.json");

lazy_static! {
    pub static ref LANGUAGE_OPTIONS: LanguageOptions = serde_json::from_str(JSON).unwrap();
}
