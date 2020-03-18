mod bibtex;
mod factory;
mod preselect;
mod quality;

pub use self::factory::CompletionItemData;

use self::{
    bibtex::{
        cmd::BibtexCommandCompletionProvider, entry_type::BibtexEntryTypeCompletionProvider,
        field_name::BibtexFieldNameCompletionProvider,
    },
    preselect::PreselectCompletionProvider,
    quality::OrderByQualityCompletionProvider,
};
use crate::{
    feature::{ConcatProvider, FeatureProvider, FeatureRequest},
    protocol::{CompletionItem, CompletionParams},
};
use futures_boxed::boxed;
use itertools::Itertools;
use std::hash::{Hash, Hasher};

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
