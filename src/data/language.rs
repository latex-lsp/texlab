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
    pub additional_environment_commands: Option<Vec<LatexEnvironmentCommand>>,
    pub additional_citation_commands: Option<Vec<LatexCitationCommand>>,
    pub additional_label_commands: Option<Vec<LatexLabelCommand>>,
    pub additional_section_commands: Option<Vec<LatexSectionCommand>>,
    pub additional_include_commands: Option<Vec<LatexIncludeCommand>>,
    pub additional_command_definition_commands: Option<Vec<LatexCommandDefinitionCommand>>,
    pub additional_math_operator_commands: Option<Vec<LatexMathOperatorCommand>>,
    pub additional_colors: Option<Vec<String>>,
    pub additional_color_commands: Option<Vec<LatexColorCommand>>,
    pub additional_color_model_commands: Option<Vec<LatexColorModelCommand>>,
}

impl LatexLanguageOptions {
    pub fn environment_commands(&self) -> impl Iterator<Item = &LatexEnvironmentCommand> {
        Self::merge(
            &DEFAULT_OPTIONS.additional_environment_commands,
            &self.additional_environment_commands,
        )
    }

    pub fn citation_commands(&self) -> impl Iterator<Item = &LatexCitationCommand> {
        Self::merge(
            &DEFAULT_OPTIONS.additional_citation_commands,
            &self.additional_citation_commands,
        )
    }

    pub fn label_commands(&self) -> impl Iterator<Item = &LatexLabelCommand> {
        Self::merge(
            &DEFAULT_OPTIONS.additional_label_commands,
            &self.additional_label_commands,
        )
    }

    pub fn section_commands(&self) -> impl Iterator<Item = &LatexSectionCommand> {
        Self::merge(
            &DEFAULT_OPTIONS.additional_section_commands,
            &self.additional_section_commands,
        )
    }

    pub fn include_commands(&self) -> impl Iterator<Item = &LatexIncludeCommand> {
        Self::merge(
            &DEFAULT_OPTIONS.additional_include_commands,
            &self.additional_include_commands,
        )
    }

    pub fn command_definition_commands(
        &self,
    ) -> impl Iterator<Item = &LatexCommandDefinitionCommand> {
        Self::merge(
            &DEFAULT_OPTIONS.additional_command_definition_commands,
            &self.additional_command_definition_commands,
        )
    }

    pub fn math_operator_commands(&self) -> impl Iterator<Item = &LatexMathOperatorCommand> {
        Self::merge(
            &DEFAULT_OPTIONS.additional_math_operator_commands,
            &self.additional_math_operator_commands,
        )
    }

    pub fn colors(&self) -> impl Iterator<Item = &String> {
        Self::merge(&DEFAULT_OPTIONS.additional_colors, &self.additional_colors)
    }

    pub fn color_commands(&self) -> impl Iterator<Item = &LatexColorCommand> {
        Self::merge(
            &DEFAULT_OPTIONS.additional_color_commands,
            &self.additional_color_commands,
        )
    }

    pub fn color_model_commands(&self) -> impl Iterator<Item = &LatexColorModelCommand> {
        Self::merge(
            &DEFAULT_OPTIONS.additional_color_model_commands,
            &self.additional_color_model_commands,
        )
    }

    fn merge<'a, T>(
        left: &'a Option<Vec<T>>,
        right: &'a Option<Vec<T>>,
    ) -> impl Iterator<Item = &'a T> {
        left.iter().chain(right.iter()).flat_map(|item| item.iter())
    }
}

const DEFAULT_OPTIONS_JSON: &str = include_str!("language.json");

lazy_static! {
    static ref DEFAULT_OPTIONS: LatexLanguageOptions =
        serde_json::from_str(DEFAULT_OPTIONS_JSON).unwrap();
}
