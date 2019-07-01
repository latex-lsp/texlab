use once_cell::sync::OnceCell;
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

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
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
    Pdf,
    Everything,
}

impl LatexIncludeKind {
    pub fn extensions(&self) -> Option<&'static [&'static str]> {
        match self {
            LatexIncludeKind::Package => Some(&["sty"]),
            LatexIncludeKind::Class => Some(&["cls"]),
            LatexIncludeKind::Latex => Some(&["tex"]),
            LatexIncludeKind::Bibliography => Some(&["bib"]),
            LatexIncludeKind::Image => Some(&["pdf", "png", "jpg", "jpeg", "bmp"]),
            LatexIncludeKind::Svg => Some(&["svg"]),
            LatexIncludeKind::Pdf => Some(&["pdf"]),
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
pub struct LatexTheoremDefinitionCommand {
    pub name: String,
    pub index: usize,
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

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct BibtexField {
    pub name: String,
    pub documentation: String,
}

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LanguageData {
    pub environment_commands: Vec<LatexEnvironmentCommand>,
    pub citation_commands: Vec<LatexCitationCommand>,
    pub label_commands: Vec<LatexLabelCommand>,
    pub section_commands: Vec<LatexSectionCommand>,
    pub include_commands: Vec<LatexIncludeCommand>,
    pub command_definition_commands: Vec<LatexCommandDefinitionCommand>,
    pub math_operator_commands: Vec<LatexMathOperatorCommand>,
    pub theorem_definition_commands: Vec<LatexTheoremDefinitionCommand>,
    pub colors: Vec<String>,
    pub color_commands: Vec<LatexColorCommand>,
    pub color_model_commands: Vec<LatexColorModelCommand>,
    pub entry_types: Vec<BibtexEntryType>,
    pub fields: Vec<BibtexField>,
    pub pgf_libraries: Vec<String>,
    pub tikz_libraries: Vec<String>,
    pub tikz_commands: Vec<String>,
}

impl LanguageData {
    pub fn entry_type_documentation(&self, name: &str) -> Option<&str> {
        for ty in self.entry_types.iter() {
            if ty.name.to_lowercase() == name.to_lowercase() {
                if let Some(documentation) = &ty.documentation {
                    return Some(&documentation);
                }
            }
        }
        None
    }

    pub fn field_documentation(&self, name: &str) -> Option<&str> {
        self.fields
            .iter()
            .find(|field| field.name.to_lowercase() == name.to_lowercase())
            .map(|field| field.documentation.as_ref())
    }
}

pub fn language_data() -> &'static LanguageData {
    static INSTANCE: OnceCell<LanguageData> = OnceCell::new();
    INSTANCE.get_or_init(|| {
        const JSON: &str = include_str!("language.json");
        serde_json::from_str(JSON).expect("Failed to deserialize language.json")
    })
}
