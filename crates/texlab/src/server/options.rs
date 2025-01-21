use serde::{Deserialize, Serialize};

use regex::Regex;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct Options {
    pub aux_directory: Option<String>,
    pub bibtex_formatter: BibtexFormatter,
    pub latex_formatter: LatexFormatter,
    pub formatter_line_length: Option<i32>,
    pub diagnostics: DiagnosticsOptions,
    pub diagnostics_delay: Option<u64>,
    pub build: BuildOptions,
    pub chktex: ChktexOptions,
    pub symbols: SymbolOptions,
    pub latexindent: LatexindentOptions,
    pub forward_search: ForwardSearchOptions,
    pub completion: CompletionOptions,
    pub inlay_hints: InlayHintOptions,
    pub experimental: ExperimentalOptions,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BibtexFormatter {
    None,
    Texlab,
    Latexindent,
    TexFmt,
}

impl Default for BibtexFormatter {
    fn default() -> Self {
        Self::Texlab
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LatexFormatter {
    None,
    Texlab,
    Latexindent,
    TexFmt,
}

impl Default for LatexFormatter {
    fn default() -> Self {
        Self::Latexindent
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct LatexindentOptions {
    pub local: Option<String>,
    pub modify_line_breaks: bool,
    pub replacement: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct BuildOptions {
    pub executable: Option<String>,
    pub args: Option<Vec<String>>,
    pub on_save: bool,
    pub forward_search_after: bool,
    pub aux_directory: Option<String>,
    pub log_directory: Option<String>,
    pub pdf_directory: Option<String>,
    pub filename: Option<String>,
    pub use_file_list: bool,
}

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct ChktexOptions {
    pub on_open_and_save: bool,
    pub on_edit: bool,
    pub additional_args: Option<Vec<String>>,
}

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct ForwardSearchOptions {
    pub executable: Option<String>,
    pub args: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct DiagnosticsOptions {
    pub allowed_patterns: Vec<RegexPattern>,
    pub ignored_patterns: Vec<RegexPattern>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct InlayHintOptions {
    pub label_definitions: Option<bool>,
    pub label_references: Option<bool>,
    pub max_length: Option<usize>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct SymbolOptions {
    pub allowed_patterns: Vec<RegexPattern>,
    pub ignored_patterns: Vec<RegexPattern>,
    pub custom_environments: Vec<SymbolEnvironmentOptions>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct SymbolEnvironmentOptions {
    pub name: String,
    pub display_name: Option<String>,
    pub label: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegexPattern(#[serde(with = "serde_regex")] pub Regex);

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct ExperimentalOptions {
    pub follow_package_links: bool,
    pub math_environments: Vec<String>,
    pub enum_environments: Vec<String>,
    pub verbatim_environments: Vec<String>,
    pub citation_commands: Vec<String>,
    pub label_definition_commands: Vec<String>,
    pub label_definition_prefixes: Vec<(String, String)>,
    pub label_reference_commands: Vec<String>,
    pub label_reference_prefixes: Vec<(String, String)>,
    pub label_reference_range_commands: Vec<String>,
}

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct StartupOptions {
    pub skip_distro: bool,
}

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct CompletionOptions {
    pub matcher: CompletionMatcher,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CompletionMatcher {
    Fuzzy,
    FuzzyIgnoreCase,
    Prefix,
    PrefixIgnoreCase,
}

impl Default for CompletionMatcher {
    fn default() -> Self {
        Self::FuzzyIgnoreCase
    }
}
