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

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LatexLanguageOptions {
    pub additional_environment_commands: Vec<LatexEnvironmentCommand>,
    pub additional_citation_commands: Vec<LatexCitationCommand>,
    pub additional_label_commands: Vec<LatexLabelCommand>,
    pub additional_section_commands: Vec<LatexSectionCommand>,
    pub additional_include_commands: Vec<LatexIncludeCommand>,
    pub additional_command_definition_commands: Vec<LatexCommandDefinitionCommand>,
    pub additional_math_operator_commands: Vec<LatexMathOperatorCommand>,
    pub additional_colors: Vec<String>,
    pub additional_color_commands: Vec<LatexColorCommand>,
    pub additional_color_model_commands: Vec<LatexColorModelCommand>,
}

impl LatexLanguageOptions {
    pub fn environment_commands(&self) -> impl Iterator<Item = &LatexEnvironmentCommand> {
        DEFAULT_OPTIONS
            .additional_environment_commands
            .iter()
            .chain(self.additional_environment_commands.iter())
    }

    pub fn citation_commands(&self) -> impl Iterator<Item = &LatexCitationCommand> {
        DEFAULT_OPTIONS
            .additional_citation_commands
            .iter()
            .chain(self.additional_citation_commands.iter())
    }

    pub fn label_commands(&self) -> impl Iterator<Item = &LatexLabelCommand> {
        DEFAULT_OPTIONS
            .additional_label_commands
            .iter()
            .chain(self.additional_label_commands.iter())
    }

    pub fn section_commands(&self) -> impl Iterator<Item = &LatexSectionCommand> {
        DEFAULT_OPTIONS
            .additional_section_commands
            .iter()
            .chain(self.additional_section_commands.iter())
    }

    pub fn include_commands(&self) -> impl Iterator<Item = &LatexIncludeCommand> {
        DEFAULT_OPTIONS
            .additional_include_commands
            .iter()
            .chain(self.additional_include_commands.iter())
    }

    pub fn command_definition_commands(
        &self,
    ) -> impl Iterator<Item = &LatexCommandDefinitionCommand> {
        DEFAULT_OPTIONS
            .additional_command_definition_commands
            .iter()
            .chain(self.additional_command_definition_commands.iter())
    }

    pub fn math_operator_commands(&self) -> impl Iterator<Item = &LatexMathOperatorCommand> {
        DEFAULT_OPTIONS
            .additional_math_operator_commands
            .iter()
            .chain(self.additional_math_operator_commands.iter())
    }

    pub fn colors(&self) -> impl Iterator<Item = &String> {
        DEFAULT_OPTIONS
            .additional_colors
            .iter()
            .chain(self.additional_colors.iter())
    }

    pub fn color_commands(&self) -> impl Iterator<Item = &LatexColorCommand> {
        DEFAULT_OPTIONS
            .additional_color_commands
            .iter()
            .chain(self.additional_color_commands.iter())
    }

    pub fn color_model_commands(&self) -> impl Iterator<Item = &LatexColorModelCommand> {
        DEFAULT_OPTIONS
            .additional_color_model_commands
            .iter()
            .chain(self.additional_color_model_commands.iter())
    }
}

const DEFAULT_OPTIONS_JSON: &str = include_str!("language.json");

lazy_static! {
    static ref DEFAULT_OPTIONS: LatexLanguageOptions =
        serde_json::from_str(DEFAULT_OPTIONS_JSON).unwrap();
}
