use base_db::{MatchingAlgo, Workspace};
use fuzzy_matcher::skim::SkimMatcherV2;

use crate::{CompletionItem, CompletionResult};

use super::matchers::{self, Matcher};

pub struct CompletionBuilder<'a> {
    pub matcher: Box<dyn Matcher>,
    pub items: Vec<CompletionItem<'a>>,
}

impl<'a> From<&Workspace> for CompletionBuilder<'a> {
    fn from(workspace: &Workspace) -> Self {
        let matcher: Box<dyn Matcher> = match workspace.config().completion.matcher {
            MatchingAlgo::Skim => Box::<SkimMatcherV2>::default(),
            MatchingAlgo::SkimIgnoreCase => Box::new(SkimMatcherV2::default().ignore_case()),
            MatchingAlgo::Prefix => Box::new(matchers::Prefix),
            MatchingAlgo::PrefixIgnoreCase => Box::new(matchers::PrefixIgnoreCase),
        };

        Self {
            matcher,
            items: Vec::new(),
        }
    }
}

impl<'a> CompletionBuilder<'a> {
    pub fn finish(mut self) -> CompletionResult<'a> {
        self.items.sort_by(|a, b| {
            b.preselect
                .cmp(&a.preselect)
                .then_with(|| b.score.cmp(&a.score))
                .then_with(|| a.data.sort_index().cmp(&b.data.sort_index()))
                .then_with(|| a.data.label().cmp(b.data.label()))
        });

        self.items.dedup_by(|a, b| a.data.label() == b.data.label());
        self.items.truncate(crate::LIMIT);
        let Self { items, .. } = self;
        CompletionResult { items }
    }
}
