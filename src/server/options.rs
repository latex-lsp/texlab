use std::time::Duration;

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::{Config, Formatter, SynctexConfig};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct Options {
    pub root_directory: Option<String>,
    pub aux_directory: Option<String>,
    pub bibtex_formatter: BibtexFormatter,
    pub latex_formatter: LatexFormatter,
    pub formatter_line_length: Option<i32>,
    pub diagnostics: DiagnosticsOptions,
    pub diagnostics_delay: DiagnosticsDelay,
    pub build: BuildOptions,
    pub chktex: ChktexOptions,
    pub symbols: SymbolOptions,
    pub latexindent: LatexindentOptions,
    pub forward_search: ForwardSearchOptions,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub struct DiagnosticsDelay(#[serde(with = "serde_millis")] pub Duration);

impl Default for DiagnosticsDelay {
    fn default() -> Self {
        Self(Duration::from_millis(300))
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BibtexFormatter {
    None,
    Texlab,
    Latexindent,
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
}

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct BuildOptions {
    pub executable: BuildExecutable,
    pub args: BuildArgs,
    pub on_save: bool,
    pub forward_search_after: bool,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BuildExecutable(pub String);

impl Default for BuildExecutable {
    fn default() -> Self {
        Self("latexmk".to_string())
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BuildArgs(pub Vec<String>);

impl Default for BuildArgs {
    fn default() -> Self {
        Self(vec![
            "-pdf".to_string(),
            "-interaction=nonstopmode".to_string(),
            "-synctex=1".to_string(),
            "%f".to_string(),
        ])
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct ChktexOptions {
    pub on_open_and_save: bool,
    pub on_edit: bool,
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
pub struct SymbolOptions {
    pub allowed_patterns: Vec<RegexPattern>,
    pub ignored_patterns: Vec<RegexPattern>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegexPattern(#[serde(with = "serde_regex")] pub Regex);

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct StartupOptions {
    pub skip_distro: bool,
}

impl From<Options> for Config {
    fn from(value: Options) -> Self {
        let mut config = Config::default();
        config.root_dir = value.root_directory;

        config.build.program = value.build.executable.0;
        config.build.args = value.build.args.0;
        config.build.on_save = value.build.on_save;
        config.build.forward_search_after = value.build.forward_search_after;
        config.build.output_dir = value.aux_directory.unwrap_or_else(|| String::from("."));

        config.diagnostics.allowed_patterns = value
            .diagnostics
            .allowed_patterns
            .into_iter()
            .map(|pattern| pattern.0)
            .collect();

        config.diagnostics.ignored_patterns = value
            .diagnostics
            .ignored_patterns
            .into_iter()
            .map(|pattern| pattern.0)
            .collect();

        config.diagnostics.delay = value.diagnostics_delay.0;
        config.diagnostics.chktex.on_open = value.chktex.on_open_and_save;
        config.diagnostics.chktex.on_save = value.chktex.on_open_and_save;
        config.diagnostics.chktex.on_edit = value.chktex.on_edit;

        config.formatting.tex_formatter = match value.latex_formatter {
            LatexFormatter::None => Formatter::Null,
            LatexFormatter::Texlab => Formatter::Server,
            LatexFormatter::Latexindent => Formatter::LatexIndent,
        };

        config.formatting.tex_formatter = match value.bibtex_formatter {
            BibtexFormatter::None => Formatter::Null,
            BibtexFormatter::Texlab => Formatter::Server,
            BibtexFormatter::Latexindent => Formatter::LatexIndent,
        };

        config.formatting.line_length =
            value
                .formatter_line_length
                .map_or(80, |len| if len < 0 { usize::MAX } else { len as usize });

        config.formatting.latex_indent.local = value.latexindent.local;
        config.formatting.latex_indent.modify_line_breaks = value.latexindent.modify_line_breaks;

        config.synctex = value
            .forward_search
            .executable
            .zip(value.forward_search.args)
            .map(|(program, args)| SynctexConfig { program, args });

        config.symbols.allowed_patterns = value
            .symbols
            .allowed_patterns
            .into_iter()
            .map(|pattern| pattern.0)
            .collect();

        config.symbols.ignored_patterns = value
            .symbols
            .ignored_patterns
            .into_iter()
            .map(|pattern| pattern.0)
            .collect();

        config
    }
}
