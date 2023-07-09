use std::path::PathBuf;
use std::time::Duration;

use base_db::{Config, Formatter, SynctexConfig};
use regex::Regex;
use serde::{Deserialize, Serialize};

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
    pub diagnostics_delay: Option<u64>,
    pub build: BuildOptions,
    pub chktex: ChktexOptions,
    pub symbols: SymbolOptions,
    pub latexindent: LatexindentOptions,
    pub forward_search: ForwardSearchOptions,
    pub completion: CompletionOptions,
    pub experimental: ExperimentalOptions,
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
    pub executable: Option<String>,
    pub args: Option<Vec<String>>,
    pub on_save: bool,
    pub forward_search_after: bool,
    pub filename: Option<String>,
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct ExperimentalOptions {
    pub math_environments: Vec<String>,
    pub enum_environments: Vec<String>,
    pub verbatim_environments: Vec<String>,
    pub citation_commands: Vec<String>,
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

impl From<Options> for Config {
    fn from(value: Options) -> Self {
        let mut config = Config::default();
        config.root_dir = value.root_directory;

        config.build.program = value.build.executable.unwrap_or(config.build.program);
        config.build.args = value.build.args.unwrap_or(config.build.args);
        config.build.on_save = value.build.on_save;
        config.build.forward_search_after = value.build.forward_search_after;
        config.build.output_dir = value.aux_directory.unwrap_or_else(|| String::from("."));
        config.build.output_filename = value.build.filename.map(PathBuf::from);

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

        config.diagnostics.delay = value
            .diagnostics_delay
            .map_or(config.diagnostics.delay, Duration::from_millis);

        config.diagnostics.chktex.on_open = value.chktex.on_open_and_save;
        config.diagnostics.chktex.on_save = value.chktex.on_open_and_save;
        config.diagnostics.chktex.on_edit = value.chktex.on_edit;

        config.formatting.tex_formatter = match value.latex_formatter {
            LatexFormatter::None => Formatter::Null,
            LatexFormatter::Texlab => Formatter::Server,
            LatexFormatter::Latexindent => Formatter::LatexIndent,
        };

        config.formatting.bib_formatter = match value.bibtex_formatter {
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

        config.completion.matcher = match value.completion.matcher {
            CompletionMatcher::Fuzzy => base_db::MatchingAlgo::Skim,
            CompletionMatcher::FuzzyIgnoreCase => base_db::MatchingAlgo::SkimIgnoreCase,
            CompletionMatcher::Prefix => base_db::MatchingAlgo::Prefix,
            CompletionMatcher::PrefixIgnoreCase => base_db::MatchingAlgo::PrefixIgnoreCase,
        };

        config
            .syntax
            .math_environments
            .extend(value.experimental.math_environments);

        config
            .syntax
            .enum_environments
            .extend(value.experimental.enum_environments);

        config
            .syntax
            .verbatim_environments
            .extend(value.experimental.verbatim_environments);

        config
            .syntax
            .citation_commands
            .extend(value.experimental.citation_commands);

        config
    }
}
