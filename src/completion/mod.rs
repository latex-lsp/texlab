mod bibtex;
mod data;
mod factory;
mod latex;
mod preselect;
mod quality;

use self::bibtex::command::BibtexCommandCompletionProvider;
use self::bibtex::entry_type::BibtexEntryTypeCompletionProvider;
use self::bibtex::field_name::BibtexFieldNameCompletionProvider;
pub use self::data::DATABASE;
pub use self::factory::CompletionItemData;
use self::latex::argument::LatexArgumentCompletionProvider;
use self::latex::begin_command::LatexBeginCommandCompletionProvider;
use self::latex::citation::LatexCitationCompletionProvider;
use self::latex::color::LatexColorCompletionProvider;
use self::latex::color_model::LatexColorModelCompletionProvider;
use self::latex::component::*;
use self::latex::glossary::LatexGlossaryCompletionProvider;
use self::latex::import::{LatexClassImportProvider, LatexPackageImportProvider};
use self::latex::include::LatexIncludeCompletionProvider;
use self::latex::label::LatexLabelCompletionProvider;
use self::latex::theorem::LatexTheoremEnvironmentCompletionProvider;
use self::latex::tikz::*;
use self::latex::user::*;
use self::preselect::PreselectCompletionProvider;
use self::quality::OrderByQualityCompletionProvider;
use crate::workspace::*;
use futures_boxed::boxed;
use itertools::Itertools;
use std::hash::{Hash, Hasher};
use texlab_protocol::{CompletionItem, CompletionParams};

pub const COMPLETION_LIMIT: usize = 50;

type MergeProvider = ConcatProvider<CompletionParams, CompletionItem>;

pub struct CompletionProvider {
    provider: OrderByQualityCompletionProvider<PreselectCompletionProvider<MergeProvider>>,
}

impl CompletionProvider {
    pub fn new() -> Self {
        Self {
            provider: OrderByQualityCompletionProvider::new(PreselectCompletionProvider::new(
                ConcatProvider::new(vec![
                    Box::new(BibtexEntryTypeCompletionProvider),
                    Box::new(BibtexFieldNameCompletionProvider),
                    Box::new(BibtexCommandCompletionProvider),
                    Box::new(LatexPgfLibraryCompletionProvider),
                    Box::new(LatexTikzLibraryCompletionProvider),
                    Box::new(LatexColorCompletionProvider),
                    Box::new(LatexColorModelCompletionProvider),
                    Box::new(LatexArgumentCompletionProvider),
                    Box::new(LatexComponentEnvironmentCompletionProvider),
                    Box::new(LatexTheoremEnvironmentCompletionProvider),
                    Box::new(LatexLabelCompletionProvider),
                    Box::new(LatexCitationCompletionProvider),
                    Box::new(LatexGlossaryCompletionProvider),
                    Box::new(LatexIncludeCompletionProvider),
                    Box::new(LatexClassImportProvider),
                    Box::new(LatexPackageImportProvider),
                    Box::new(LatexBeginCommandCompletionProvider),
                    Box::new(LatexComponentCommandCompletionProvider),
                    Box::new(LatexUserCommandCompletionProvider),
                    Box::new(LatexUserEnvironmentCompletionProvider),
                ]),
            )),
        }
    }
}

impl Default for CompletionProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl FeatureProvider for CompletionProvider {
    type Params = CompletionParams;
    type Output = Vec<CompletionItem>;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<Self::Params>) -> Self::Output {
        self.provider
            .execute(request)
            .await
            .into_iter()
            .map(LabeledCompletionItem)
            .unique()
            .map(|item| item.0)
            .take(COMPLETION_LIMIT)
            .collect()
    }
}

#[derive(Debug, Clone)]
struct LabeledCompletionItem(CompletionItem);

impl PartialEq for LabeledCompletionItem {
    fn eq(&self, other: &Self) -> bool {
        self.0.label == other.0.label
    }
}

impl Eq for LabeledCompletionItem {}

impl Hash for LabeledCompletionItem {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.label.hash(state);
    }
}
