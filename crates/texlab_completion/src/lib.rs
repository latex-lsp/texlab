mod bibtex;
mod factory;
mod latex;
mod preselect;
mod quality;

pub use self::factory::CompletionItemData;

use self::{
    bibtex::{
        cmd::BibtexCommandCompletionProvider, entry_type::BibtexEntryTypeCompletionProvider,
        field_name::BibtexFieldNameCompletionProvider,
    },
    latex::{
        argument::LatexArgumentCompletionProvider,
        begin_command::LatexBeginCommandCompletionProvider,
        citation::LatexCitationCompletionProvider,
        color::LatexColorCompletionProvider,
        color_model::LatexColorModelCompletionProvider,
        component::{
            LatexComponentCommandCompletionProvider, LatexComponentEnvironmentCompletionProvider,
        },
        glossary::LatexGlossaryCompletionProvider,
        import::{LatexClassImportProvider, LatexPackageImportProvider},
        include::LatexIncludeCompletionProvider,
        label::LatexLabelCompletionProvider,
        theorem::LatexTheoremEnvironmentCompletionProvider,
        tikz_lib::{LatexPgfLibraryCompletionProvider, LatexTikzLibraryCompletionProvider},
        user::{LatexUserCommandCompletionProvider, LatexUserEnvironmentCompletionProvider},
    },
    preselect::PreselectCompletionProvider,
    quality::OrderByQualityCompletionProvider,
};
use futures_boxed::boxed;
use itertools::Itertools;
use std::hash::{Hash, Hasher};
use texlab_feature::{ConcatProvider, FeatureProvider, FeatureRequest};
use texlab_protocol::{CompletionItem, CompletionParams};

pub const COMPLETION_LIMIT: usize = 50;

type MergeProvider = ConcatProvider<CompletionParams, CompletionItem>;

pub struct CompletionProvider {
    provider: OrderByQualityCompletionProvider<PreselectCompletionProvider<MergeProvider>>,
}

impl CompletionProvider {
    pub fn new() -> Self {
        Self {
            provider: OrderByQualityCompletionProvider(PreselectCompletionProvider(
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
    async fn execute<'a>(&'a self, req: &'a FeatureRequest<Self::Params>) -> Self::Output {
        self.provider
            .execute(req)
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
