use once_cell::sync::Lazy;
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
pub enum LatexLabelReferenceSource {
    Everything,
    Math,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LatexLabelKind {
    Definition,
    Reference(LatexLabelReferenceSource),
}

impl LatexLabelKind {
    pub fn is_reference(self) -> bool {
        match self {
            LatexLabelKind::Definition => false,
            LatexLabelKind::Reference(_) => true,
        }
    }
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
    pub prefix: String,
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
    pub fn extensions(self) -> Option<&'static [&'static str]> {
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

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LatexGlossaryEntryKind {
    General,
    Acronym,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LatexGlossaryEntryDefinitionCommand {
    pub name: String,
    pub label_index: usize,
    pub kind: LatexGlossaryEntryKind,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LatexGlossaryEntryReferenceCommand {
    pub name: String,
    pub index: usize,
    pub kind: LatexGlossaryEntryKind,
}

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
    pub glossary_entry_definition_commands: Vec<LatexGlossaryEntryDefinitionCommand>,
    pub glossary_entry_reference_commands: Vec<LatexGlossaryEntryReferenceCommand>,
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
    const JSON: &str = include_str!("lang_data.json");
    serde_json::from_str(JSON).expect("Failed to deserialize language.json")
});
