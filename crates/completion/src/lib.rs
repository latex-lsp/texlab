mod providers;
mod util;

use base_db::{
    data::{BibtexEntryType, BibtexFieldType},
    semantics::bib,
    util::RenderedObject,
    Document, FeatureParams,
};
use rowan::{TextRange, TextSize};
use util::CompletionBuilder;

pub const LIMIT: usize = 50;

#[derive(Debug)]
pub struct CompletionParams<'a> {
    pub feature: FeatureParams<'a>,
    pub offset: TextSize,
}

#[derive(Debug, Default)]
pub struct CompletionResult<'a> {
    pub items: Vec<CompletionItem<'a>>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct CompletionItem<'a> {
    pub score: i32,
    pub range: TextRange,
    pub preselect: bool,
    pub data: CompletionItemData<'a>,
}

impl<'a> CompletionItem<'a> {
    pub fn new_simple(score: i32, range: TextRange, data: CompletionItemData<'a>) -> Self {
        Self {
            score,
            range,
            preselect: false,
            data,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum CompletionItemData<'a> {
    Command(CommandData<'a>),
    CommandLikeDelimiter(&'a str, &'a str),
    BeginEnvironment,
    Citation(CitationData<'a>),
    Environment(EnvironmentData<'a>),
    GlossaryEntry(GlossaryEntryData),
    Label(LabelData<'a>),
    Color(&'a str),
    ColorModel(&'a str),
    File(String),
    Directory(String),
    Argument(ArgumentData<'a>),
    Package(&'a str),
    DocumentClass(&'a str),
    EntryType(EntryTypeData<'a>),
    Field(FieldTypeData<'a>),
    TikzLibrary(&'a str),
}

impl<'a> CompletionItemData<'a> {
    pub fn label<'b: 'a>(&'b self) -> &'a str {
        match self {
            Self::Command(data) => data.name,
            Self::CommandLikeDelimiter(left, _) => left,
            Self::BeginEnvironment => "begin",
            Self::Citation(data) => &data.entry.name.text,
            Self::Environment(data) => data.name,
            Self::GlossaryEntry(data) => &data.name,
            Self::Label(data) => data.name,
            Self::Color(name) => name,
            Self::ColorModel(name) => name,
            Self::File(name) => &name,
            Self::Directory(name) => &name,
            Self::Argument(data) => &data.0.name,
            Self::Package(name) => name,
            Self::DocumentClass(name) => name,
            Self::EntryType(data) => data.0.name,
            Self::Field(data) => data.0.name,
            Self::TikzLibrary(name) => name,
        }
    }

    /// Returns a number that can be used to sort the completion items further before resorting to the label.
    /// This is useful for making snippets more visible.
    pub fn sort_index(&self) -> u8 {
        match self {
            Self::Command(_) => 1,
            Self::CommandLikeDelimiter(_, _) => 0,
            Self::BeginEnvironment => 1,
            Self::Citation(_) => 1,
            Self::Environment(_) => 1,
            Self::GlossaryEntry(_) => 1,
            Self::Label(_) => 1,
            Self::Color(_) => 1,
            Self::ColorModel(_) => 1,
            Self::File(_) => 1,
            Self::Directory(_) => 1,
            Self::Argument(_) => 1,
            Self::Package(_) => 1,
            Self::DocumentClass(_) => 1,
            Self::EntryType(_) => 1,
            Self::Field(_) => 1,
            Self::TikzLibrary(_) => 1,
        }
    }
}

#[derive(PartialEq, Eq)]
pub struct CommandData<'a> {
    pub name: &'a str,
    pub glyph: Option<&'a str>,
    pub image: Option<&'a str>,
    pub package: Option<&'a completion_data::Package<'a>>,
}

impl<'a> std::fmt::Debug for CommandData<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommandData")
            .field("name", &self.name)
            .field("package", self.package.map_or(&"<user>", |p| &p.file_names))
            .finish()
    }
}

#[derive(PartialEq, Eq)]
pub struct EnvironmentData<'a> {
    pub name: &'a str,
    pub package: Option<&'a completion_data::Package<'a>>,
}

impl<'a> std::fmt::Debug for EnvironmentData<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EnvironmentData")
            .field("name", &self.name)
            .field("package", self.package.map_or(&"<user>", |p| &p.file_names))
            .finish()
    }
}

#[derive(PartialEq, Eq)]
pub struct ArgumentData<'a>(pub &'a completion_data::Argument<'a>);

impl<'a> std::fmt::Debug for ArgumentData<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ArgumentData").field(&self.0.name).finish()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct CitationData<'a> {
    pub document: &'a Document,
    pub entry: &'a bib::Entry,
}

#[derive(Debug, PartialEq, Eq)]
pub struct GlossaryEntryData {
    pub name: String,
}

#[derive(Debug, PartialEq, Eq)]
pub struct LabelData<'a> {
    pub name: &'a str,
    pub header: Option<String>,
    pub footer: Option<&'a str>,
    pub object: Option<RenderedObject<'a>>,
    pub keywords: String,
}

#[derive(PartialEq, Eq)]
pub struct EntryTypeData<'a>(pub BibtexEntryType<'a>);

impl<'a> std::fmt::Debug for EntryTypeData<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("EntryTypeData").field(&self.0.name).finish()
    }
}

#[derive(PartialEq, Eq)]
pub struct FieldTypeData<'a>(pub BibtexFieldType<'a>);

impl<'a> std::fmt::Debug for FieldTypeData<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("FieldTypeData").field(&self.0.name).finish()
    }
}

pub fn complete<'a>(params: &'a CompletionParams<'a>) -> CompletionResult<'a> {
    let mut builder = CompletionBuilder::from(params.feature.workspace);
    providers::complete_commands(params, &mut builder);
    providers::complete_environments(params, &mut builder);
    providers::complete_citations(params, &mut builder);
    providers::complete_acronyms(params, &mut builder);
    providers::complete_glossaries(params, &mut builder);
    providers::complete_label_references(params, &mut builder);
    providers::complete_label_definitions(params, &mut builder);
    providers::complete_colors(params, &mut builder);
    providers::complete_color_models(params, &mut builder);
    providers::complete_includes(params, &mut builder);
    providers::complete_arguments(params, &mut builder);
    providers::complete_imports(params, &mut builder);
    providers::complete_entry_types(params, &mut builder);
    providers::complete_fields(params, &mut builder);
    providers::complete_tikz_libraries(params, &mut builder);
    builder.finish()
}

#[cfg(test)]
mod tests;
