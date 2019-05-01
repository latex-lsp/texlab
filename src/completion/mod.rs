mod factory;
mod latex;
mod quality;

use crate::completion::latex::color::LatexColorCompletionProvider;
use crate::completion::latex::color_model::LatexColorModelCompletionProvider;
use crate::completion::latex::kernel_command::LatexKernelCommandCompletionProvider;
use crate::completion::latex::kernel_environment::LatexKernelEnvironmentCompletionProvider;
use crate::completion::latex::label::LatexLabelCompletionProvider;
use crate::completion::latex::pgf_library::LatexPgfLibraryCompletionProvider;
use crate::completion::latex::tikz_library::LatexTikzLibraryCompletionProvider;
use crate::completion::latex::user_command::LatexUserCommandCompletionProvider;
use crate::completion::quality::OrderByQualityCompletionProvider;
use crate::concat_feature;
use crate::feature::FeatureRequest;
use itertools::Itertools;
use lsp_types::{CompletionItem, CompletionParams};
use std::collections::{HashMap, HashSet};

pub const COMPLETION_LIMIT: usize = 50;

pub struct CompletionProvider;

impl CompletionProvider {
    pub async fn execute(request: &FeatureRequest<CompletionParams>) -> Vec<CompletionItem> {
        let items = await!(OrderByQualityCompletionProvider::execute(
            request,
            async move |_| {
                concat_feature!(
                    &request,
                    LatexKernelEnvironmentCompletionProvider,
                    LatexPgfLibraryCompletionProvider,
                    LatexTikzLibraryCompletionProvider,
                    LatexColorCompletionProvider,
                    LatexColorModelCompletionProvider,
                    LatexLabelCompletionProvider,
                    LatexKernelCommandCompletionProvider,
                    LatexUserCommandCompletionProvider
                )
            }
        ));

        items
            .into_iter()
            .unique_by(|item| item.label.clone())
            .take(COMPLETION_LIMIT)
            .collect()
    }
}
